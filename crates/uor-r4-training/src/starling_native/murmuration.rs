//! The murmuration read, stage S1: the level-0 flock transport mean (M4 #2032,
//! card 6151093135, amendment M1 6151441631 with erratum 6151450159; Codex
//! FIT for S1 6151326908).
//!
//! **Forward**, per (batch row, head) row and position `t`, on 4-d values:
//! * the flock `N_t` is K1's first `K` kept slots with `j < t` (the reader is
//!   K1's slot 0 and is dropped; its own value enters as `-v_t`), excluding
//!   absent slots and address-null neighbours;
//! * every neighbour value is carried into the reader's frame by the exact
//!   icosian `g_tj = c_t (x) conj(c_j)`, looked up in the 120 x 120 relative
//!   table built by exact `Z[phi]` arithmetic ([`relative_table`]), and applied
//!   as a left Hamilton product: `T_tj(v) = g_tj (x) v`;
//! * `m_t = (1/|N_t|) sum T_tj(v_j)` with the reciprocal read from a fixed
//!   table of `1/1 .. 1/7`; `Delta_t = K_h (m_t - v_t)`, and `Delta_t = 0` for
//!   an empty flock or an address-null reader.
//!
//! **Backward**: the values and the per-head coupling `K_h` get the exact
//! derivatives of that forward (device tensor arithmetic). Cells and slots are
//! discrete and take no gradient. The registered address credit (A1's address
//! margin on the one shared address) is injected by [`AddressCapture`], the
//! identity on `Delta` whose backward receives `dL/dDelta`:
//! * `g_t = K_h dL/dDelta_t`, the utility of a kept or exploration candidate
//!   is `u_tj = -<g_t, T_tj(v_j)>` (exploration slots have zero forward weight
//!   and are not in `|N_t|`);
//! * when the best off-home candidate `j*` (ties: the later position) clears
//!   `u_tj* - mean_kept u > delta`, the address dot `<a_t, c_j*>` receives
//!   `-address * address_margin / queries`; through the unit-address
//!   normalisation that is the tangent push toward `c_j*` of M1.
//!
//! No exponential, no normalised exponential, no squashing, no learned matrix
//! product: a fixed icosian left action, a bounded mean and a learned scalar
//! per head. Model arithmetic is CUDA-only (D24); every CPU entry refuses.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use candle_core::{CpuStorage, CustomOp2, DType, Layout, Shape, Tensor};

use super::{geometry, require_cuda, NativeStarling, ABSENT, CELLS};
use crate::{invalid, Result};

/// The largest flock: K1 keeps at most this many neighbours per query for S1.
pub const MAX_FLOCK: usize = 7;
/// A raw address with a smaller norm is address-null (M1).
pub const NULL_ADDRESS_NORM: f64 = 1e-6;

/// The K1 specification the murmuration read runs on: one extra kept slot, so
/// the reader (always among K1's candidates, which are `j <= t`) can be
/// dropped **before** the top-K truncation (Codex 6151549249 item 3). Refuses
/// `K > MAX_FLOCK` or a window that exceeds K1's slot limit.
pub fn selection_spec(spec: &NativeStarling) -> Result<NativeStarling> {
    if spec.kept == 0 || spec.kept > MAX_FLOCK {
        return Err(invalid(format!(
            "the murmuration flock keeps 1..={MAX_FLOCK} neighbours (K = {})",
            spec.kept
        )));
    }
    let selector = NativeStarling {
        kept: spec.kept + 1,
        ..*spec
    };
    selector.validate()?;
    Ok(selector)
}

/// An element of `Z[phi]`, `a + b phi` with `phi^2 = phi + 1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Golden(i64, i64);

impl Golden {
    fn add(self, o: Golden) -> Golden {
        Golden(self.0 + o.0, self.1 + o.1)
    }
    fn sub(self, o: Golden) -> Golden {
        Golden(self.0 - o.0, self.1 - o.1)
    }
    fn mul(self, o: Golden) -> Golden {
        // (a + b phi)(c + d phi) = ac + bd + (ad + bc + bd) phi
        Golden(
            self.0 * o.0 + self.1 * o.1,
            self.0 * o.1 + self.1 * o.0 + self.1 * o.1,
        )
    }
    fn neg(self) -> Golden {
        Golden(-self.0, -self.1)
    }
    fn scale(self, k: i64) -> Golden {
        Golden(self.0 * k, self.1 * k)
    }
    fn value(self) -> f64 {
        let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
        self.0 as f64 + self.1 as f64 * phi
    }
}

/// A quaternion with `Z[phi]` coordinates, scalar first.
type ExactQ = [Golden; 4];

/// Hamilton product, scalar first: the convention of [`hamilton`].
fn exact_mul(a: &ExactQ, b: &ExactQ) -> ExactQ {
    let [w1, x1, y1, z1] = *a;
    let [w2, x2, y2, z2] = *b;
    [
        w1.mul(w2).sub(x1.mul(x2)).sub(y1.mul(y2)).sub(z1.mul(z2)),
        w1.mul(x2).add(x1.mul(w2)).add(y1.mul(z2)).sub(z1.mul(y2)),
        w1.mul(y2).sub(x1.mul(z2)).add(y1.mul(w2)).add(z1.mul(x2)),
        w1.mul(z2).add(x1.mul(y2)).sub(y1.mul(x2)).add(z1.mul(w2)),
    ]
}

fn exact_conj(a: &ExactQ) -> ExactQ {
    [a[0], a[1].neg(), a[2].neg(), a[3].neg()]
}

/// The 120 unit icosians, **doubled** (every coordinate `2x` lies in
/// `{0, +-1, +-2, +-phi, +-(phi - 1)}`), rebuilt in exact `Z[phi]` arithmetic in
/// the order of `canonical_h4_roots`: the 8 axis roots, the 16 half roots,
/// then the 96 even permutations of `(0, 1/2, phi/2, (phi - 1)/2)` with signs.
fn doubled_icosians() -> Vec<ExactQ> {
    let zero = Golden(0, 0);
    let mut list = Vec::with_capacity(CELLS);
    for axis in 0..4 {
        for sign in [-1i64, 1] {
            let mut q = [zero; 4];
            q[axis] = Golden(2 * sign, 0);
            list.push(q);
        }
    }
    for i in 0..16usize {
        let s = |bit: usize| if (i >> bit) & 1 == 1 { 1 } else { -1 };
        list.push([
            Golden(s(0), 0),
            Golden(s(1), 0),
            Golden(s(2), 0),
            Golden(s(3), 0),
        ]);
    }
    let even_perms: [[usize; 4]; 12] = [
        [0, 1, 2, 3],
        [0, 2, 3, 1],
        [0, 3, 1, 2],
        [1, 0, 3, 2],
        [1, 2, 0, 3],
        [1, 3, 2, 0],
        [2, 0, 1, 3],
        [2, 1, 3, 0],
        [2, 3, 0, 1],
        [3, 0, 2, 1],
        [3, 1, 0, 2],
        [3, 2, 1, 0],
    ];
    // 2 * (0, 1/2, phi/2, (phi - 1)/2) = (0, 1, phi, phi - 1).
    let base = [Golden(0, 0), Golden(1, 0), Golden(0, 1), Golden(-1, 1)];
    for perm in &even_perms {
        for s1 in [-1i64, 1] {
            for s2 in [-1i64, 1] {
                for s3 in [-1i64, 1] {
                    let signed = [
                        base[0],
                        base[1].scale(s1),
                        base[2].scale(s2),
                        base[3].scale(s3),
                    ];
                    list.push([
                        signed[perm[0]],
                        signed[perm[1]],
                        signed[perm[2]],
                        signed[perm[3]],
                    ]);
                }
            }
        }
    }
    list
}

/// The exact relative table `rel[a * 120 + b]` = the index of the icosian
/// `c_a (x) conj(c_b)`, built by exact `Z[phi]` products of the doubled roots
/// (`2c_a (x) conj(2c_b) = 4 g`, matched against `2 * (2 c_k)`). It refuses if
/// the rebuilt roots disagree with the f32 codebook K1 uses, or if any product
/// leaves the group: either would mean the transport is not the icosian action.
pub fn relative_table() -> Result<&'static [u32]> {
    static TABLE: OnceLock<std::result::Result<Vec<u32>, String>> = OnceLock::new();
    TABLE
        .get_or_init(|| build_table(|a, b| exact_mul(a, &exact_conj(b))).map_err(|e| e.to_string()))
        .as_deref()
        .map_err(|e| invalid(e.clone()))
}

/// The exact right-action table `rel_R[a * 120 + b]` = the index of
/// `conj(c_b) (x) c_a` (M2), built like [`relative_table`]. The right transport
/// `T^R_tj(v) = v (x) h_tj` with `h_tj = conj(c_j) (x) c_t` is the other SU(2)
/// factor of the 600-cell's rotation group.
pub fn right_relative_table() -> Result<&'static [u32]> {
    static TABLE: OnceLock<std::result::Result<Vec<u32>, String>> = OnceLock::new();
    TABLE
        .get_or_init(|| build_table(|a, b| exact_mul(&exact_conj(b), a)).map_err(|e| e.to_string()))
        .as_deref()
        .map_err(|e| invalid(e.clone()))
}

/// The exact table of `product(c_a, c_b)` over all ordered pairs, as icosian
/// indices; refuses a product outside 2I or a codebook mismatch with K1.
fn build_table(product: impl Fn(&ExactQ, &ExactQ) -> ExactQ) -> Result<Vec<u32>> {
    let doubled = doubled_icosians();
    let (roots, _) = geometry()?;
    if doubled.len() != CELLS || roots.len() != CELLS * 4 {
        return Err(invalid("the icosian codebook must have 120 roots"));
    }
    // The exact rebuild must be K1's codebook, in K1's order.
    for (k, q) in doubled.iter().enumerate() {
        for (axis, coordinate) in q.iter().enumerate() {
            let exact = coordinate.value() / 2.0;
            if (exact - f64::from(roots[4 * k + axis])).abs() > 1e-6 {
                return Err(invalid(format!(
                    "exact icosian {k} disagrees with the f32 codebook at axis {axis}"
                )));
            }
        }
    }
    let lookup: HashMap<ExactQ, u32> = doubled
        .iter()
        .enumerate()
        .map(|(k, q)| {
            (
                [q[0].scale(2), q[1].scale(2), q[2].scale(2), q[3].scale(2)],
                k as u32,
            )
        })
        .collect();
    if lookup.len() != CELLS {
        return Err(invalid("the exact icosians are not distinct"));
    }
    let mut table = Vec::with_capacity(CELLS * CELLS);
    for a in &doubled {
        for b in &doubled {
            let product = product(a, b);
            let Some(&k) = lookup.get(&product) else {
                return Err(invalid("an icosian quotient left the group 2I"));
            };
            table.push(k);
        }
    }
    Ok(table)
}

/// Left Hamilton product `a (x) b` over the last axis (4, scalar first), by
/// fixed component equations: elementwise device arithmetic, no matrix
/// product.
pub fn hamilton(a: &Tensor, b: &Tensor) -> Result<Tensor> {
    let last = a
        .rank()
        .checked_sub(1)
        .ok_or_else(|| invalid("a quaternion needs an axis"))?;
    if a.dim(last)? != 4 || b.dim(last)? != 4 {
        return Err(invalid("Hamilton products need 4-d quaternions"));
    }
    let part = |t: &Tensor, i: usize| t.narrow(last, i, 1);
    let (w1, x1, y1, z1) = (part(a, 0)?, part(a, 1)?, part(a, 2)?, part(a, 3)?);
    let (w2, x2, y2, z2) = (part(b, 0)?, part(b, 1)?, part(b, 2)?, part(b, 3)?);
    let m = |p: &Tensor, q: &Tensor| p.broadcast_mul(q);
    let w = m(&w1, &w2)?
        .sub(&m(&x1, &x2)?)?
        .sub(&m(&y1, &y2)?)?
        .sub(&m(&z1, &z2)?)?;
    let x = m(&w1, &x2)?
        .add(&m(&x1, &w2)?)?
        .add(&m(&y1, &z2)?)?
        .sub(&m(&z1, &y2)?)?;
    let y = m(&w1, &y2)?
        .sub(&m(&x1, &z2)?)?
        .add(&m(&y1, &w2)?)?
        .add(&m(&z1, &x2)?)?;
    let z = m(&w1, &z2)?
        .add(&m(&x1, &y2)?)?
        .sub(&m(&y1, &x2)?)?
        .add(&m(&z1, &w2)?)?;
    Ok(Tensor::cat(&[&w, &x, &y, &z], last)?)
}

/// Which value transport each head uses (amendment M2). Handedness is a
/// value-transport property only: addresses, cells, K1 and the tree are
/// unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chirality {
    /// Every head transports by the left action `g (x) v` (S1/S2 as frozen).
    Left,
    /// Even heads left, odd heads right (`v (x) h`): both SU(2) factors of the
    /// 600-cell's rotation group in every layer. Needs an even head count.
    Paired,
}

impl Chirality {
    /// Whether head `h` transports by the right action.
    pub fn right(self, head: usize) -> bool {
        self == Chirality::Paired && !head.is_multiple_of(2)
    }

    pub fn validate(self, heads: usize) -> Result<()> {
        if self == Chirality::Paired && (heads == 0 || !heads.is_multiple_of(2)) {
            return Err(invalid(format!(
                "paired (left/right) heads need an even head count, got {heads}"
            )));
        }
        Ok(())
    }
}

/// Carry values into their readers' frames, row by row in each row's
/// handedness: `T^L(v) = roots[rel_L[home, other]] (x) v` on left rows and
/// `T^R(v) = v (x) roots[rel_R[home, other]]` on right rows. One Hamilton
/// product per element either way: a right row only swaps the operands.
/// `home`/`other` are U32 cells of one shape whose first axis is the row
/// (`rows = batch * heads`, head `r % heads`); `value` is that shape plus 4.
pub(crate) fn carry(
    chirality: Chirality,
    heads: usize,
    home: &Tensor,
    other: &Tensor,
    value: &Tensor,
) -> Result<Tensor> {
    let device = value.device();
    let dims = home.dims().to_vec();
    let rows = dims.first().copied().unwrap_or(0);
    if other.dims() != dims.as_slice()
        || value.dims().split_last().map(|(_, d)| d) != Some(&dims[..])
    {
        return Err(invalid("carry operands disagree in shape"));
    }
    let count: usize = dims.iter().product();
    let index = home
        .contiguous()?
        .mul(&Tensor::full(CELLS as u32, dims.as_slice(), device)?)?
        .add(&other.contiguous()?)?
        .flatten_all()?;
    let roots = Tensor::from_vec(geometry()?.0, (CELLS, 4), device)?;
    let left_table = Tensor::from_slice(relative_table()?, CELLS * CELLS, device)?;
    let left = roots
        .index_select(&left_table.index_select(&index, 0)?, 0)?
        .reshape(value.shape())?;
    if chirality == Chirality::Left {
        return hamilton(&left, value);
    }
    let right_table = Tensor::from_slice(right_relative_table()?, CELLS * CELLS, device)?;
    let right = roots
        .index_select(&right_table.index_select(&index, 0)?, 0)?
        .reshape(value.shape())?;
    // Per-row handedness (index metadata), broadcast over the row's elements.
    let per_row = count / rows.max(1);
    let mask: Vec<u8> = (0..rows)
        .flat_map(|r| std::iter::repeat_n(u8::from(chirality.right(r % heads)), per_row * 4))
        .collect();
    let mask = Tensor::from_vec(mask, value.shape(), device)?;
    let lhs = mask.where_cond(value, &left)?;
    let rhs = mask.where_cond(&right, value)?;
    hamilton(&lhs, &rhs)
}

/// Device scalars recorded per backward for reporting: the queries whose
/// address was credited and the mean utility gap that triggered it.
#[derive(Clone, Debug)]
pub struct AddressReport {
    pub credited: Tensor,
    pub queries: Tensor,
}

/// Where a model collects [`AddressReport`]s (optional; reporting only).
pub type AddressSink = Arc<Mutex<Vec<AddressReport>>>;

/// Inputs of one S1 read on `rows = batch * heads` rows of `time` positions.
pub struct MurmurationInputs<'a> {
    /// Unit 4-d addresses `[rows, time, 4]` F32 (the credit's target).
    pub address: &'a Tensor,
    /// Raw address norms `[rows, time]` F32 (null below [`NULL_ADDRESS_NORM`]).
    pub address_norm: &'a Tensor,
    /// 4-d values `[rows, time, 4]` F32.
    pub value: &'a Tensor,
    /// The per-head coupling `K_h`, `[heads]` F32.
    pub coupling: &'a Tensor,
    /// K1's output on [`selection_spec`]: `[rows, time, (K + 1) + E + 2]` U32.
    pub selection: &'a Tensor,
    pub heads: usize,
    /// Value-transport handedness (M2).
    pub chirality: Chirality,
}

/// What one S1 read returns: `Delta` and its diagnostics (device tensors).
pub struct MurmurationOutput {
    /// `Delta_t = K_h (m_t - v_t)`, `[rows, time, 4]`.
    pub delta: Tensor,
    /// Flock sizes `|N_t|` as F32, `[rows, time]`.
    pub flock: Tensor,
    /// 1 where the reader is address-null, `[rows, time]` F32.
    pub null: Tensor,
}

/// The S1 read. `credit` adds the address capture (training); without it the
/// forward and the value/coupling gradients are identical.
pub fn murmuration_read(
    spec: &NativeStarling,
    inputs: &MurmurationInputs<'_>,
    credit: bool,
    sink: Option<AddressSink>,
) -> Result<MurmurationOutput> {
    spec.validate()?;
    inputs.chirality.validate(inputs.heads)?;
    require_cuda(inputs.value, "the murmuration read")?;
    // K1 runs with one extra kept slot (see [`selection_spec`]) so the reader
    // can be filtered out before the top-K truncation.
    let selector = selection_spec(spec)?;
    let (k, window, s) = (spec.kept, selector.kept, selector.slots());
    let (rows, time, four) = inputs.value.dims3()?;
    let heads = inputs.heads;
    if four != 4
        || heads == 0
        || rows % heads != 0
        || inputs.address.dims() != [rows, time, 4]
        || inputs.address_norm.dims() != [rows, time]
        || inputs.coupling.dims() != [heads]
        || inputs.selection.dims() != [rows, time, s + 2]
        || inputs.selection.dtype() != DType::U32
    {
        return Err(invalid(
            "murmuration tensors disagree with the declared route",
        ));
    }
    let device = inputs.value.device();
    let value = inputs.value.to_dtype(DType::F32)?;
    let selection = inputs.selection;

    // Index metadata: slots, cells, positions.
    let index = selection.narrow(2, 0, s)?.contiguous()?; // [rows, time, S]
    let home = selection.narrow(2, s, 1)?.squeeze(2)?.contiguous()?; // [rows, time]
    let key_cell = selection.narrow(2, s + 1, 1)?.squeeze(2)?.contiguous()?;
    let present = index.ne(&Tensor::full(ABSENT, (rows, time, s), device)?)?;
    let safe = present.where_cond(&index, &Tensor::zeros((rows, time, s), DType::U32, device)?)?;
    let position = Tensor::arange(0u32, time as u32, device)?
        .reshape((1, time, 1))?
        .broadcast_as((rows, time, s))?
        .contiguous()?;
    let past = safe.lt(&position)?.mul(&present)?; // strictly causal, self dropped
    let null = inputs
        .address_norm
        .to_dtype(DType::F32)?
        .lt(NULL_ADDRESS_NORM)?; // [rows, time] u8
    let base = Tensor::from_vec(
        (0..rows).map(|r| (r * time) as u32).collect::<Vec<_>>(),
        (rows, 1, 1),
        device,
    )?;
    let flat = safe.broadcast_add(&base)?.flatten_all()?;
    let neighbour_null = null
        .to_dtype(DType::U32)?
        .flatten_all()?
        .index_select(&flat, 0)?
        .reshape((rows, time, s))?;
    let usable = past.mul(&neighbour_null.eq(0u32)?)?; // live candidate
    let live = usable.to_dtype(DType::F32)?; // [rows, time, S]

    // Exact transport: g_tj = roots[rel[c_t * 120 + c_j]].
    let neighbour_cell = key_cell
        .flatten_all()?
        .index_select(&flat, 0)?
        .reshape((rows, time, s))?;
    let (roots, _) = geometry()?;
    let roots = Tensor::from_vec(roots, (CELLS, 4), device)?;
    let neighbour_value = value
        .contiguous()?
        .reshape((rows * time, 4))?
        .index_select(&flat, 0)?
        .reshape((rows, time, s, 4))?;
    // T_tj(v_j) in each head's handedness (M2), [rows, time, S, 4].
    let carried = carry(
        inputs.chirality,
        heads,
        &home
            .unsqueeze(2)?
            .broadcast_as((rows, time, s))?
            .contiguous()?,
        &neighbour_cell,
        &neighbour_value,
    )?;

    // The flock: the first K live slots of K1's K + 1 window, in K1's rank
    // order (largest address dot, ties to the later position). K1 returns only
    // j <= t, so filtering j < t removes at most the reader itself and this is
    // exactly the top K of the strict past. Exploration has zero forward
    // weight. The running count is an explicit sum over the small window.
    let window_live = live.narrow(2, 0, window)?; // [rows, time, K + 1]
    let mut running = Tensor::zeros((rows, time, 1), DType::F32, device)?;
    let mut columns = Vec::with_capacity(window);
    for i in 0..window {
        let column = window_live.narrow(2, i, 1)?;
        running = running.add(&column)?;
        columns.push(column.mul(&running.le(k as f64)?.to_dtype(DType::F32)?)?);
    }
    let kept = Tensor::cat(&columns, 2)?; // [rows, time, K + 1], at most K ones
    let flock = kept.sum(2)?; // [rows, time]
    let reciprocal = Tensor::from_vec(
        (0..=MAX_FLOCK)
            .map(|n| if n == 0 { 0.0f32 } else { 1.0 / n as f32 })
            .collect::<Vec<_>>(),
        MAX_FLOCK + 1,
        device,
    )?;
    let inverse = reciprocal
        .index_select(&flock.to_dtype(DType::U32)?.flatten_all()?, 0)?
        .reshape((rows, time, 1))?;
    let mean = carried
        .narrow(2, 0, window)?
        .broadcast_mul(&kept.unsqueeze(3)?)?
        .sum(2)?
        .broadcast_mul(&inverse)?; // m_t, [rows, time, 4]
    let reads = flock
        .gt(0.0)?
        .mul(&null.eq(0u8)?)?
        .to_dtype(DType::F32)?
        .unsqueeze(2)?; // [rows, time, 1]
    let head_index = Tensor::from_vec(
        (0..rows).map(|r| (r % heads) as u32).collect::<Vec<_>>(),
        rows,
        device,
    )?;
    let coupling = inputs
        .coupling
        .to_dtype(DType::F32)?
        .index_select(&head_index, 0)?
        .reshape((rows, 1, 1))?;
    let delta = mean
        .sub(&value)?
        .broadcast_mul(&coupling)?
        .broadcast_mul(&reads)?;

    let delta = if credit {
        let address = inputs
            .address
            .to_dtype(DType::F32)?
            .reshape((rows, time, 1, 4))?;
        let cell_roots = roots
            .index_select(&neighbour_cell.flatten_all()?, 0)?
            .reshape((rows, time, s, 4))?;
        let dots = cell_roots.broadcast_mul(&address)?.sum(3)?; // <a_t, c_j>, [rows, time, S]
        let off_home = neighbour_cell
            .ne(&home
                .unsqueeze(2)?
                .broadcast_as((rows, time, s))?
                .contiguous()?)?
            .to_dtype(DType::F32)?;
        // Credit candidates: the kept flock and the live exploration slots.
        // A window slot cut by the truncation is neither.
        let exploration = live.narrow(2, window, s - window)?;
        let kept_full = Tensor::cat(&[&kept, &exploration.zeros_like()?], 2)?;
        let candidate = Tensor::cat(&[&kept, &exploration], 2)?;
        let state = CaptureState {
            spec: *spec,
            carried: carried.detach(),
            kept: kept_full,
            candidate,
            off_home,
            position: safe.to_dtype(DType::F32)?,
            coupling: coupling.detach(),
            reads: reads.detach(),
            sink,
        };
        delta
            .contiguous()?
            .apply_op2(&dots.contiguous()?, AddressCapture(state))?
    } else {
        delta
    };
    Ok(MurmurationOutput {
        delta,
        flock,
        null: null.to_dtype(DType::F32)?,
    })
}

/// What the address credit needs from the forward, all detached.
struct CaptureState {
    spec: NativeStarling,
    /// `T_tj(v_j)` of every kept and exploration slot, `[rows, time, S, 4]`.
    carried: Tensor,
    /// 1 for a slot in the forward flock, `[rows, time, S]`.
    kept: Tensor,
    /// 1 for a credit candidate: the flock plus live exploration slots.
    candidate: Tensor,
    /// 1 where the candidate's cell differs from the reader's home cell.
    off_home: Tensor,
    /// Candidate positions as F32 (ties: the later position).
    position: Tensor,
    /// `K_h` per row, `[rows, 1, 1]`.
    coupling: Tensor,
    /// 1 where the reader reads, `[rows, time, 1]`.
    reads: Tensor,
    sink: Option<AddressSink>,
}

/// The identity on `Delta` whose backward injects the registered address
/// margin into the address dots (one capture point; values and coupling keep
/// their exact gradients).
struct AddressCapture(CaptureState);

impl CustomOp2 for AddressCapture {
    fn name(&self) -> &'static str {
        "murmuration-address-credit"
    }
    fn cpu_fwd(
        &self,
        _: &CpuStorage,
        _: &Layout,
        _: &CpuStorage,
        _: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        candle_core::bail!("murmuration credit capture is GPU-only; no host fallback (D24)")
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        delta: &candle_core::CudaStorage,
        layout: &Layout,
        _: &candle_core::CudaStorage,
        _: &Layout,
    ) -> candle_core::Result<(candle_core::CudaStorage, Shape)> {
        use candle_core::backend::BackendStorage;
        if !layout.is_contiguous() || layout.start_offset() != 0 {
            candle_core::bail!("murmuration credit capture needs a contiguous Delta");
        }
        Ok((delta.try_clone(layout)?, layout.shape().clone()))
    }
    fn bwd(
        &self,
        _delta: &Tensor,
        dots: &Tensor,
        _res: &Tensor,
        grad: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        let dot_grad = address_credit(&self.0, dots, grad)
            .map_err(|e| candle_core::Error::Msg(e.to_string()))?;
        Ok((Some(grad.clone()), Some(dot_grad)))
    }
}

/// The address-dot gradient `[rows, time, S]` of the registered margin, from
/// `dL/dDelta` (`[rows, time, 4]`). Device tensor arithmetic only.
fn address_credit(state: &CaptureState, dots: &Tensor, grad: &Tensor) -> Result<Tensor> {
    require_cuda(grad, "murmuration credit")?;
    let (rows, time, slots, _) = state.carried.dims4()?;
    if dots.dims() != [rows, time, slots] {
        return Err(invalid(
            "murmuration credit tensors disagree with the slots",
        ));
    }
    let device = grad.device();
    let credit = state.spec.credit;
    let queries = (rows * time) as f64;
    // g_t = dL/dm_t = K_h dL/dDelta_t on reading positions.
    let g = grad
        .to_dtype(DType::F32)?
        .broadcast_mul(&state.coupling)?
        .broadcast_mul(&state.reads)?
        .reshape((rows, time, 1, 4))?;
    let utility = state.carried.broadcast_mul(&g)?.sum(3)?.neg()?; // [rows, time, S]
    let kept_count = state.kept.sum(2)?;
    let kept_mean = utility
        .mul(&state.kept)?
        .sum(2)?
        .div(&kept_count.maximum(&Tensor::ones((rows, time), DType::F32, device)?)?)?;
    // An empty flock has no reference utility: its address credit is exactly 0.
    let has_flock = kept_count.gt(0.0)?.to_dtype(DType::F32)?;
    // The best off-home candidate (kept or exploration), ties: later position.
    let eligible = state.candidate.mul(&state.off_home)?;
    let eligible_mask = eligible.gt(0.0)?;
    let floor = Tensor::full(-3.0e38f32, (rows, time, slots), device)?;
    let masked = eligible_mask.where_cond(&utility, &floor)?;
    let best = masked.max_keepdim(2)?;
    let tied = masked.broadcast_eq(&best)?.mul(&eligible_mask)?;
    let tied_position = tied.where_cond(
        &state.position,
        &Tensor::full(-1.0f32, (rows, time, slots), device)?,
    )?;
    let best_position = tied_position.max_keepdim(2)?;
    let chosen = tied_position
        .broadcast_eq(&best_position)?
        .mul(&tied)?
        .to_dtype(DType::F32)?; // one-hot (positions are distinct)
    let any = eligible.sum(2)?.gt(0.0)?.to_dtype(DType::F32)?;
    let gap = best.squeeze(2)?.sub(&kept_mean)?;
    let on = gap
        .gt(credit.delta as f64)?
        .to_dtype(DType::F32)?
        .mul(&any)?
        .mul(&has_flock)?; // [rows, time]
    let scale = on.affine(
        -(credit.address * credit.address_margin) as f64 / queries,
        0.0,
    )?;

    if let Some(sink) = &state.sink {
        sink.lock()
            .map_err(|_| invalid("murmuration credit sink poisoned"))?
            .push(AddressReport {
                credited: on.sum_all()?,
                queries: Tensor::new(queries as f32, device)?,
            });
    }
    Ok(chosen.broadcast_mul(&scale.unsqueeze(2)?)?)
}

#[cfg(test)]
#[allow(clippy::needless_range_loop, clippy::type_complexity)]
mod tests {
    use super::*;
    use crate::starling_native::{select, unit_address, Credit};
    use candle_core::{Device, Var};

    /// Control 1 (host part): the relative table is the exact quotient of the
    /// group 2I, built in `Z[phi]` and checked against exact products, not
    /// against the f32 codebook it indexes.
    #[test]
    fn murm_table_matches_canonical_zphi() -> Result<()> {
        let table = relative_table()?;
        let doubled = doubled_icosians();
        assert_eq!(table.len(), CELLS * CELLS);
        let identity = doubled
            .iter()
            .position(|q| q == &[Golden(2, 0), Golden(0, 0), Golden(0, 0), Golden(0, 0)])
            .ok_or_else(|| invalid("no identity icosian"))? as u32;
        let four = [Golden(4, 0), Golden(0, 0), Golden(0, 0), Golden(0, 0)];
        for a in 0..CELLS {
            let mut seen = [false; CELLS];
            for b in 0..CELLS {
                let g = table[a * CELLS + b] as usize;
                assert!(!seen[g], "row {a} of the relative table repeats {g}");
                seen[g] = true;
                // 2g = (2c_a)(x)conj(2c_b) / 2, exactly.
                let product = exact_mul(&doubled[a], &exact_conj(&doubled[b]));
                let twice = [
                    doubled[g][0].scale(2),
                    doubled[g][1].scale(2),
                    doubled[g][2].scale(2),
                    doubled[g][3].scale(2),
                ];
                assert_eq!(product, twice, "rel[{a},{b}] is not c_a conj(c_b)");
                // g_ab (x) g_ba = 1.
                let back = table[b * CELLS + a] as usize;
                assert_eq!(exact_mul(&doubled[g], &doubled[back]), four);
            }
            assert_eq!(table[a * CELLS + a], identity);
        }
        Ok(())
    }

    #[test]
    fn murm_refuses_cpu_and_oversized_flocks() -> Result<()> {
        let device = Device::Cpu;
        let spec = NativeStarling {
            depth: 4,
            kept: 2,
            explore: 0,
            bos_id: 0,
            credit: Credit::FIXTURE,
        };
        let zeros4 = Tensor::zeros((1, 3, 4), DType::F32, &device)?;
        let norm = Tensor::ones((1, 3), DType::F32, &device)?;
        let coupling = Tensor::zeros(1, DType::F32, &device)?;
        let selection = Tensor::zeros((1, 3, 4), DType::U32, &device)?;
        let inputs = MurmurationInputs {
            address: &zeros4,
            address_norm: &norm,
            value: &zeros4,
            coupling: &coupling,
            selection: &selection,
            heads: 1,
            chirality: Chirality::Left,
        };
        assert!(murmuration_read(&spec, &inputs, false, None).is_err());
        let wide = NativeStarling { kept: 8, ..spec };
        assert!(murmuration_read(&wide, &inputs, false, None).is_err());
        Ok(())
    }

    // ---- GPU controls (run on the pod; ignored where CUDA is absent) ----

    fn cuda() -> Result<Device> {
        Ok(Device::new_cuda(0)?)
    }

    fn roots_host() -> Result<Vec<[f32; 4]>> {
        let (roots, _) = geometry()?;
        Ok(roots.chunks(4).map(|c| [c[0], c[1], c[2], c[3]]).collect())
    }

    fn adjacency_host() -> Result<Vec<u32>> {
        Ok(geometry()?.1)
    }

    /// A fixture of `rows = batch * heads` rows with addresses exactly at the
    /// given icosian vertices (`cells[row][t]`), BOS at `bos[b]` positions,
    /// and deterministic bounded values.
    struct Fixture {
        address: Tensor,
        norm: Tensor,
        value: Tensor,
        reset: Tensor,
        heads: usize,
    }

    fn fixture(
        device: &Device,
        cells: &[Vec<usize>],
        heads: usize,
        resets: &[Vec<usize>],
    ) -> Result<Fixture> {
        let roots = roots_host()?;
        let rows = cells.len();
        let time = cells[0].len();
        let mut address = Vec::with_capacity(rows * time * 4);
        let mut value = Vec::with_capacity(rows * time * 4);
        for (r, row) in cells.iter().enumerate() {
            for (t, &c) in row.iter().enumerate() {
                address.extend_from_slice(&roots[c]);
                for i in 0..4 {
                    // Fixture constants in [-0.9, 0.9]; no model arithmetic.
                    let x = ((r * 131 + t * 17 + i * 7) % 37) as f32 / 37.0;
                    value.push(1.8 * x - 0.9);
                }
            }
        }
        let batch = rows / heads;
        let mut reset = vec![0u32; batch * time];
        for (b, positions) in resets.iter().enumerate() {
            for &t in positions {
                reset[b * time + t] = 1;
            }
        }
        Ok(Fixture {
            address: Tensor::from_vec(address, (rows, time, 4), device)?,
            norm: Tensor::ones((rows, time), DType::F32, device)?,
            value: Tensor::from_vec(value, (rows, time, 4), device)?,
            reset: Tensor::from_vec(reset, (batch, time), device)?,
            heads,
        })
    }

    fn run(
        spec: &NativeStarling,
        f: &Fixture,
        value: &Tensor,
        coupling: &Tensor,
        credit: bool,
    ) -> Result<MurmurationOutput> {
        let selection = select(
            &selection_spec(spec)?,
            f.heads,
            &f.address,
            &f.address,
            &f.reset,
        )?;
        murmuration_read(
            spec,
            &MurmurationInputs {
                address: &f.address,
                address_norm: &f.norm,
                value,
                coupling,
                selection: &selection,
                heads: f.heads,
                chirality: Chirality::Left,
            },
            credit,
            None,
        )
    }

    fn spec(kept: usize, explore: usize) -> NativeStarling {
        NativeStarling {
            depth: 8,
            kept,
            explore,
            bos_id: 0,
            credit: Credit::FIXTURE,
        }
    }

    /// Two batches x two heads; antipodes, repeated (tied) cells, a vertex and
    /// its neighbours (distinct, noncommuting frames), a mid-sequence BOS.
    fn mixed_cells() -> Result<(Vec<Vec<usize>>, Vec<Vec<usize>>)> {
        let adjacency = adjacency_host()?;
        let n = |c: usize, i: usize| adjacency[c * 12 + i] as usize;
        let a = 30usize;
        let antipode = |c: usize| -> Result<usize> {
            let roots = roots_host()?;
            roots
                .iter()
                .position(|q| (0..4).all(|i| (q[i] + roots[c][i]).abs() < 1e-6))
                .ok_or_else(|| invalid("no antipode"))
        };
        let row0 = vec![
            a,
            a,
            n(a, 0),
            n(a, 1),
            antipode(a)?,
            a,
            n(a, 2),
            n(a, 0),
            a,
            n(a, 3),
            a,
            n(a, 4),
            a,
            a,
            n(a, 5),
            a,
        ];
        let b = 77usize;
        let row1: Vec<usize> = (0..16)
            .map(|t| if t % 3 == 0 { b } else { n(b, t % 12) })
            .collect();
        let row2: Vec<usize> = row0.iter().rev().copied().collect();
        let row3: Vec<usize> = (0..16).map(|t| n(a, (5 * t) % 12)).collect();
        // rows = batch * heads with heads = 2: rows 0,1 batch 0; rows 2,3 batch 1.
        Ok((vec![row0, row1, row2, row3], vec![vec![0, 9], vec![0]]))
    }

    /// Control 2: the read equals an independent device reference that forms
    /// each transport by direct Hamilton products of the f32 roots (not the
    /// exact table), gathers by narrow and sums slot by slot.
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_forward_independent_reference() -> Result<()> {
        let device = cuda()?;
        let (cells, resets) = mixed_cells()?;
        let f = fixture(&device, &cells, 2, &resets)?;
        let coupling = Tensor::new(&[0.75f32, -1.25], &device)?;
        for kept in [1usize, 7] {
            let s = spec(kept, 2);
            let out = run(&s, &f, &f.value, &coupling, false)?;
            let sel = selection_spec(&s)?;
            let selection = select(&sel, 2, &f.address, &f.address, &f.reset)?;
            let picks: Vec<Vec<Vec<u32>>> = selection.to_vec3()?; // index metadata
            let roots = Tensor::from_vec(geometry()?.0, (CELLS, 4), &device)?;
            let conj = Tensor::new(&[1f32, -1.0, -1.0, -1.0], &device)?;
            let (rows, time) = (cells.len(), cells[0].len());
            let mut reference = Vec::new();
            for r in 0..rows {
                for t in 0..time {
                    let v_t = f.value.narrow(0, r, 1)?.narrow(1, t, 1)?.reshape(4)?;
                    let c_t = roots
                        .narrow(0, picks[r][t][sel.slots()] as usize, 1)?
                        .reshape(4)?;
                    let mut sum = Tensor::zeros(4, DType::F32, &device)?;
                    let mut count = 0usize;
                    for &j in &picks[r][t][..kept + 1] {
                        if count == kept {
                            break;
                        }
                        if j == ABSENT || j as usize >= t {
                            continue;
                        }
                        let c_j = roots
                            .narrow(0, picks[r][j as usize][sel.slots() + 1] as usize, 1)?
                            .reshape(4)?;
                        let g = hamilton(&c_t, &c_j.mul(&conj)?)?;
                        let v_j = f
                            .value
                            .narrow(0, r, 1)?
                            .narrow(1, j as usize, 1)?
                            .reshape(4)?;
                        sum = sum.add(&hamilton(&g, &v_j)?)?;
                        count += 1;
                    }
                    let h = r % 2;
                    let k_h = coupling.narrow(0, h, 1)?;
                    let d = if count == 0 {
                        Tensor::zeros(4, DType::F32, &device)?
                    } else {
                        sum.affine(1.0 / count as f64, 0.0)?
                            .sub(&v_t)?
                            .broadcast_mul(&k_h)?
                    };
                    reference.push(d);
                }
            }
            let reference = Tensor::stack(&reference, 0)?.reshape((rows, time, 4))?;
            let worst = out
                .delta
                .sub(&reference)?
                .abs()?
                .max_keepdim(2)?
                .flatten_all()?
                .max(0)?
                .to_scalar::<f32>()?;
            assert!(
                worst <= 1e-6,
                "K = {kept}: max |read - reference| = {worst}"
            );
            // Flock sizes cover 0 and the kept maximum somewhere.
            let flock: Vec<Vec<f32>> = out.flock.to_vec2()?;
            assert!(flock.iter().flatten().any(|&n| n == 0.0));
            assert!(flock.iter().flatten().any(|&n| n as usize == kept));
        }
        Ok(())
    }

    /// Control 3: perturbing any strictly later value leaves earlier Deltas
    /// bit-identical (the reader's own value enters through -v_t, so s > t).
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_future_value_is_inert() -> Result<()> {
        let device = cuda()?;
        let (cells, resets) = mixed_cells()?;
        let f = fixture(&device, &cells, 2, &resets)?;
        let coupling = Tensor::new(&[0.5f32, 2.0], &device)?;
        let s = spec(7, 2);
        let before: Vec<Vec<Vec<f32>>> =
            run(&s, &f, &f.value, &coupling, false)?.delta.to_vec3()?;
        let cut = 7usize;
        let time = cells[0].len();
        let mask = Tensor::from_vec(
            (0..time)
                .map(|t| if t > cut { 1f32 } else { 0.0 })
                .collect::<Vec<_>>(),
            (1, time, 1),
            &device,
        )?;
        let noisy = f.value.broadcast_add(&mask.affine(0.37, 0.0)?)?;
        let after: Vec<Vec<Vec<f32>>> = run(&s, &f, &noisy, &coupling, false)?.delta.to_vec3()?;
        for (r, (b, a)) in before.iter().zip(&after).enumerate() {
            for t in 0..=cut {
                for i in 0..4 {
                    assert_eq!(b[t][i].to_bits(), a[t][i].to_bits(), "row {r} t {t}");
                }
            }
        }
        Ok(())
    }

    /// Control 4: the fixed point, the identity at K_h = 0, and address-null
    /// tokens (they neither read nor are read).
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_fixed_point_and_identity() -> Result<()> {
        let device = cuda()?;
        let adjacency = adjacency_host()?;
        let (a, b) = (30usize, adjacency[30 * 12] as usize);
        let f = fixture(&device, &[vec![a, b, b]], 1, &[vec![0]])?;
        let s = spec(7, 0);
        // v_0 = conj(g_10) (x) v_1, so the reader t = 1 sees exactly its own value.
        let table = relative_table()?;
        let roots = Tensor::from_vec(geometry()?.0, (CELLS, 4), &device)?;
        let g = roots
            .narrow(0, table[b * CELLS + a] as usize, 1)?
            .reshape(4)?;
        let conj = Tensor::new(&[1f32, -1.0, -1.0, -1.0], &device)?;
        let v1 = f.value.narrow(1, 1, 1)?.reshape(4)?;
        let v0 = hamilton(&g.mul(&conj)?, &v1)?;
        let value = Tensor::stack(&[&v0, &v1, &f.value.narrow(1, 2, 1)?.reshape(4)?], 0)?
            .reshape((1, 3, 4))?;
        let one = Tensor::new(&[1f32], &device)?;
        let out = run(&s, &f, &value, &one, false)?;
        let d1 = out
            .delta
            .narrow(1, 1, 1)?
            .abs()?
            .max_keepdim(2)?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(d1[0] <= 1e-6, "fixed point broke: {}", d1[0]);
        // K_h = 0 gives exactly zero.
        let zero = Tensor::new(&[0f32], &device)?;
        let out = run(&s, &f, &value, &zero, false)?;
        assert_eq!(out.delta.abs()?.sum_all()?.to_scalar::<f32>()?, 0.0);
        // Address-null token 0: t = 1's flock loses it; a null reader reads 0.
        let null_norm = Tensor::new(&[[0f32, 1.0, 0.0]], &device)?;
        let selection = select(&selection_spec(&s)?, 1, &f.address, &f.address, &f.reset)?;
        let out = murmuration_read(
            &s,
            &MurmurationInputs {
                address: &f.address,
                address_norm: &null_norm,
                value: &value,
                coupling: &one,
                selection: &selection,
                heads: 1,
                chirality: Chirality::Left,
            },
            false,
            None,
        )?;
        let flock: Vec<Vec<f32>> = out.flock.to_vec2()?;
        assert_eq!(flock[0][1], 0.0, "an address-null neighbour was read");
        let d2 = out
            .delta
            .narrow(1, 2, 1)?
            .abs()?
            .sum_all()?
            .to_scalar::<f32>()?;
        assert_eq!(d2, 0.0, "an address-null reader read");
        Ok(())
    }

    /// Control 5: analytic value, self-value and coupling gradients against
    /// device central secants, neighbour set frozen (addresses sit exactly on
    /// vertices; values do not move cells). Error convention, fixed before
    /// execution: with `h = 1e-3`, a check passes iff
    /// `|secant - analytic| <= 2e-3 * max(|secant|, |analytic|, 1e-3)`, so a zero
    /// derivative is held to the absolute floor `2e-6` (no division by zero).
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_backward_secants() -> Result<()> {
        let device = cuda()?;
        let (cells, resets) = mixed_cells()?;
        let f = fixture(&device, &cells, 2, &resets)?;
        let s = spec(7, 2);
        let (rows, time) = (cells.len(), cells[0].len());
        let weights = Tensor::from_vec(
            (0..rows * time * 4)
                .map(|i| ((i * 29 % 23) as f32 / 23.0) - 0.5)
                .collect::<Vec<_>>(),
            (rows, time, 4),
            &device,
        )?;
        let value = Var::from_tensor(&f.value)?;
        let coupling = Var::from_tensor(&Tensor::new(&[0.8f32, -0.6], &device)?)?;
        let loss_of = |v: &Tensor, k: &Tensor| -> Result<Tensor> {
            Ok(run(&s, &f, v, k, false)?.delta.mul(&weights)?.sum_all()?)
        };
        let grads = loss_of(value.as_tensor(), coupling.as_tensor())?.backward()?;
        let gv = grads
            .get(value.as_tensor())
            .ok_or_else(|| invalid("no value gradient"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let gk = grads
            .get(coupling.as_tensor())
            .ok_or_else(|| invalid("no coupling gradient"))?
            .to_vec1::<f32>()?;
        let h = 1e-3f64;
        let n = rows * time * 4;
        let check = |analytic: f32, plus: f32, minus: f32, what: &str| {
            let secant = (plus - minus) as f64 / (2.0 * h);
            let scale = secant.abs().max(analytic.abs() as f64).max(1e-3);
            let rel = (secant - analytic as f64).abs() / scale;
            assert!(
                rel <= 2e-3,
                "{what}: analytic {analytic} secant {secant} rel {rel}"
            );
        };
        for idx in [0usize, 5, 37, 70, 101, 130, 199, 255] {
            let bump = |sign: f64| -> Result<f32> {
                let e = Tensor::from_vec(
                    (0..n)
                        .map(|i| if i == idx { sign * h } else { 0.0 } as f32)
                        .collect::<Vec<_>>(),
                    (rows, time, 4),
                    &device,
                )?;
                Ok(loss_of(&f.value.add(&e)?, coupling.as_tensor())?.to_scalar::<f32>()?)
            };
            check(gv[idx], bump(1.0)?, bump(-1.0)?, &format!("value[{idx}]"));
        }
        for head in 0..2 {
            let bump = |sign: f64| -> Result<f32> {
                let e = Tensor::from_vec(
                    (0..2)
                        .map(|i| if i == head { sign * h } else { 0.0 } as f32)
                        .collect::<Vec<_>>(),
                    2,
                    &device,
                )?;
                Ok(loss_of(&f.value, &coupling.as_tensor().add(&e)?)?.to_scalar::<f32>()?)
            };
            check(
                gk[head],
                bump(1.0)?,
                bump(-1.0)?,
                &format!("coupling[{head}]"),
            );
        }
        Ok(())
    }

    /// Control 6: on a helpful off-home exploration candidate the injected
    /// address gradient points toward that candidate's vertex through the full
    /// K_h chain; exploration slots change neither Delta nor |N_t|.
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_address_margin_direction() -> Result<()> {
        let device = cuda()?;
        let adjacency = adjacency_host()?;
        let (a, b) = (30usize, adjacency[30 * 12] as usize);
        // t0 BOS at a, t1 at the neighbour b, t2 at a, reader t3 at a. K1 for
        // t3 ranks t3 (self), t2, t0 (dot 1, later first) then t1: with K = 2,
        // E = 2 the flock is {t2} and the exploration slots are {t0, t1}.
        let f = fixture(&device, &[vec![a, b, a, a]], 1, &[vec![0]])?;
        let w = Tensor::new(&[0.6f32, -0.48, 0.48, 0.42], &device)?; // |w| ~ 1
        let table = relative_table()?;
        let roots = Tensor::from_vec(geometry()?.0, (CELLS, 4), &device)?;
        let conj = Tensor::new(&[1f32, -1.0, -1.0, -1.0], &device)?;
        let g31 = roots
            .narrow(0, table[a * CELLS + b] as usize, 1)?
            .reshape(4)?;
        // T_31(v1) = -w (helpful), T_32(v2) = w (unhelpful kept neighbour).
        let v1 = hamilton(&g31.mul(&conj)?, &w.neg()?)?;
        let value = Tensor::stack(&[&w, &v1, &w, &w], 0)?.reshape((1, 4, 4))?;
        let one = Tensor::new(&[1f32], &device)?;
        let raw = Var::from_tensor(&f.address)?;
        let address = unit_address(raw.as_tensor())?;
        let s = spec(2, 2);
        let selection = select(&selection_spec(&s)?, 1, &address, &address, &f.reset)?;
        let weights = Tensor::stack(
            &[
                &Tensor::zeros(4, DType::F32, &device)?,
                &Tensor::zeros(4, DType::F32, &device)?,
                &Tensor::zeros(4, DType::F32, &device)?,
                &w,
            ],
            0,
        )?
        .reshape((1, 4, 4))?;
        let run_with = |sp: &NativeStarling, sel: &Tensor, credit: bool| {
            murmuration_read(
                sp,
                &MurmurationInputs {
                    address: &address,
                    address_norm: &f.norm,
                    value: &value,
                    coupling: &one,
                    selection: sel,
                    heads: 1,
                    chirality: Chirality::Left,
                },
                credit,
                None,
            )
        };
        let out = run_with(&s, &selection, true)?;
        let grads = out.delta.mul(&weights)?.sum_all()?.backward()?;
        let g_raw = grads
            .get(raw.as_tensor())
            .ok_or_else(|| invalid("no address gradient"))?
            .narrow(1, 3, 1)?
            .reshape(4)?;
        let c_b = roots.narrow(0, b, 1)?.reshape(4)?;
        let c_a = roots.narrow(0, a, 1)?.reshape(4)?;
        let tangent = c_b.sub(&c_a.broadcast_mul(&c_b.mul(&c_a)?.sum_all()?)?)?;
        let cos = g_raw.neg()?.mul(&tangent)?.sum_all()?.to_scalar::<f32>()?
            / (g_raw.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?
                * tangent.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?);
        assert!(
            cos > 0.99,
            "the address push is not toward the helpful vertex: cos {cos}"
        );
        // Exploration has zero forward weight and is not in |N_t|.
        let plain = spec(2, 0);
        let plain_selection = select(&selection_spec(&plain)?, 1, &address, &address, &f.reset)?;
        let without = run_with(&plain, &plain_selection, false)?;
        let with = run_with(&s, &selection, false)?;
        let d0: Vec<Vec<Vec<f32>>> = without.delta.to_vec3()?;
        let d1: Vec<Vec<Vec<f32>>> = with.delta.to_vec3()?;
        assert_eq!(d0, d1, "exploration slots changed Delta");
        assert_eq!(
            without.flock.to_vec2::<f32>()?,
            with.flock.to_vec2::<f32>()?
        );
        Ok(())
    }

    /// Control 4b (Codex 6151549249 item 2): at K_h = 0 the read is the exact
    /// identity, so the value gradient through it is exactly zero, while the
    /// coupling still receives a nonzero wake-up gradient
    /// `dL/dK_h = sum <dL/dDelta, m - v>`.
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_zero_coupling_wakes() -> Result<()> {
        let device = cuda()?;
        let (cells, resets) = mixed_cells()?;
        let f = fixture(&device, &cells, 2, &resets)?;
        let s = spec(7, 2);
        let (rows, time) = (cells.len(), cells[0].len());
        let weights = Tensor::from_vec(
            (0..rows * time * 4)
                .map(|i| ((i * 31 % 19) as f32 / 19.0) - 0.5)
                .collect::<Vec<_>>(),
            (rows, time, 4),
            &device,
        )?;
        let value = Var::from_tensor(&f.value)?;
        let coupling = Var::from_tensor(&Tensor::zeros(2, DType::F32, &device)?)?;
        let out = run(&s, &f, value.as_tensor(), coupling.as_tensor(), false)?;
        assert_eq!(out.delta.abs()?.sum_all()?.to_scalar::<f32>()?, 0.0);
        let grads = out.delta.mul(&weights)?.sum_all()?.backward()?;
        let gv = grads
            .get(value.as_tensor())
            .ok_or_else(|| invalid("no value gradient"))?
            .abs()?
            .sum_all()?
            .to_scalar::<f32>()?;
        assert_eq!(
            gv, 0.0,
            "values received credit through a zero-coupled read"
        );
        let gk = grads
            .get(coupling.as_tensor())
            .ok_or_else(|| invalid("no coupling gradient"))?
            .to_vec1::<f32>()?;
        assert!(
            gk.iter().all(|g| g.is_finite()) && gk.iter().any(|g| *g != 0.0),
            "the coupling has no wake-up gradient: {gk:?}"
        );
        Ok(())
    }

    // ---- Amendment M2: paired left/right heads ----

    /// The exact index multiplication table `mult[a * 120 + b]` = index of
    /// `c_a (x) c_b` (test-only, exact).
    fn multiplication_table() -> Result<Vec<u32>> {
        build_table(exact_mul)
    }

    /// M2 control 1 (host, exact integers): the right table is the exact
    /// `conj(c_b) (x) c_a`, and both tables are frame cocycles on all 120^3
    /// triples: `g_tk g_kj = g_tj` and `h_kj h_tk = h_tj`.
    #[test]
    fn murm_right_table_exact_and_cocycle() -> Result<()> {
        let left = relative_table()?;
        let right = right_relative_table()?;
        let mult = multiplication_table()?;
        let doubled = doubled_icosians();
        for a in 0..CELLS {
            for b in 0..CELLS {
                let h = right[a * CELLS + b] as usize;
                let product = exact_mul(&exact_conj(&doubled[b]), &doubled[a]);
                let twice = [
                    doubled[h][0].scale(2),
                    doubled[h][1].scale(2),
                    doubled[h][2].scale(2),
                    doubled[h][3].scale(2),
                ];
                assert_eq!(product, twice, "rel_R[{a},{b}] is not conj(c_b) c_a");
            }
        }
        for t in 0..CELLS {
            for k in 0..CELLS {
                for j in 0..CELLS {
                    let g =
                        mult[left[t * CELLS + k] as usize * CELLS + left[k * CELLS + j] as usize];
                    assert_eq!(g, left[t * CELLS + j], "left cocycle t {t} k {k} j {j}");
                    let h =
                        mult[right[k * CELLS + j] as usize * CELLS + right[t * CELLS + k] as usize];
                    assert_eq!(h, right[t * CELLS + j], "right cocycle t {t} k {k} j {j}");
                }
            }
        }
        Ok(())
    }

    #[test]
    fn murm_odd_heads_refused() -> Result<()> {
        assert!(Chirality::Paired.validate(3).is_err());
        assert!(Chirality::Paired.validate(0).is_err());
        assert!(Chirality::Paired.validate(4).is_ok());
        assert!(Chirality::Left.validate(3).is_ok());
        let device = Device::Cpu;
        let spec = spec(2, 0);
        let x = Tensor::zeros((3, 3, 4), DType::F32, &device)?;
        let norm = Tensor::ones((3, 3), DType::F32, &device)?;
        let coupling = Tensor::zeros(3, DType::F32, &device)?;
        let selection = Tensor::zeros((3, 3, 5), DType::U32, &device)?;
        let refused = murmuration_read(
            &spec,
            &MurmurationInputs {
                address: &x,
                address_norm: &norm,
                value: &x,
                coupling: &coupling,
                selection: &selection,
                heads: 3,
                chirality: Chirality::Paired,
            },
            false,
            None,
        );
        assert!(refused
            .err()
            .is_some_and(|e| e.to_string().contains("even head count")));
        Ok(())
    }

    /// M2 control 2: under `Paired`, the even (left) heads are bit-identical
    /// to `Left` mode.
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_left_heads_unchanged() -> Result<()> {
        let device = cuda()?;
        let (cells, resets) = mixed_cells()?;
        let f = fixture(&device, &cells, 2, &resets)?;
        let s = spec(7, 2);
        let coupling = Tensor::new(&[0.6f32, -0.9], &device)?;
        let selection = select(&selection_spec(&s)?, 2, &f.address, &f.address, &f.reset)?;
        let read = |c: Chirality| -> Result<Vec<Vec<Vec<f32>>>> {
            Ok(murmuration_read(
                &s,
                &MurmurationInputs {
                    address: &f.address,
                    address_norm: &f.norm,
                    value: &f.value,
                    coupling: &coupling,
                    selection: &selection,
                    heads: 2,
                    chirality: c,
                },
                false,
                None,
            )?
            .delta
            .to_vec3()?)
        };
        let (left, paired) = (read(Chirality::Left)?, read(Chirality::Paired)?);
        for r in (0..cells.len()).step_by(2) {
            for t in 0..cells[0].len() {
                for i in 0..4 {
                    assert_eq!(left[r][t][i].to_bits(), paired[r][t][i].to_bits());
                }
            }
        }
        // And the odd (right) heads genuinely differ somewhere.
        assert!((1..cells.len()).step_by(2).any(|r| left[r] != paired[r]));
        Ok(())
    }

    /// M2 control 3: right heads match an independent device reference that
    /// forms `h = conj(c_j) (x) c_t` by direct f32 products and applies it on
    /// the right, `v (x) h` (not the table).
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_right_forward_independent_reference() -> Result<()> {
        let device = cuda()?;
        let (cells, resets) = mixed_cells()?;
        let f = fixture(&device, &cells, 2, &resets)?;
        let s = spec(7, 2);
        let sel = selection_spec(&s)?;
        let coupling = Tensor::new(&[0.75f32, -1.25], &device)?;
        let selection = select(&sel, 2, &f.address, &f.address, &f.reset)?;
        let out = murmuration_read(
            &s,
            &MurmurationInputs {
                address: &f.address,
                address_norm: &f.norm,
                value: &f.value,
                coupling: &coupling,
                selection: &selection,
                heads: 2,
                chirality: Chirality::Paired,
            },
            false,
            None,
        )?;
        let picks: Vec<Vec<Vec<u32>>> = selection.to_vec3()?;
        let roots = Tensor::from_vec(geometry()?.0, (CELLS, 4), &device)?;
        let conj = Tensor::new(&[1f32, -1.0, -1.0, -1.0], &device)?;
        let time = cells[0].len();
        let k_h = coupling.narrow(0, 1, 1)?;
        for r in (1..cells.len()).step_by(2) {
            for t in 0..time {
                let v_t = f.value.narrow(0, r, 1)?.narrow(1, t, 1)?.reshape(4)?;
                let c_t = roots
                    .narrow(0, picks[r][t][sel.slots()] as usize, 1)?
                    .reshape(4)?;
                let mut sum = Tensor::zeros(4, DType::F32, &device)?;
                let mut count = 0usize;
                for &j in &picks[r][t][..s.kept + 1] {
                    if count == s.kept {
                        break;
                    }
                    if j == ABSENT || j as usize >= t {
                        continue;
                    }
                    let c_j = roots
                        .narrow(0, picks[r][j as usize][sel.slots() + 1] as usize, 1)?
                        .reshape(4)?;
                    let h = hamilton(&c_j.mul(&conj)?, &c_t)?;
                    let v_j = f
                        .value
                        .narrow(0, r, 1)?
                        .narrow(1, j as usize, 1)?
                        .reshape(4)?;
                    sum = sum.add(&hamilton(&v_j, &h)?)?;
                    count += 1;
                }
                let want = if count == 0 {
                    Tensor::zeros(4, DType::F32, &device)?
                } else {
                    sum.affine(1.0 / count as f64, 0.0)?
                        .sub(&v_t)?
                        .broadcast_mul(&k_h)?
                };
                let got = out.delta.narrow(0, r, 1)?.narrow(1, t, 1)?.reshape(4)?;
                let worst = got.sub(&want)?.abs()?.max(0)?.to_scalar::<f32>()?;
                assert!(worst <= 1e-6, "row {r} t {t}: right head off by {worst}");
            }
        }
        Ok(())
    }

    /// M2 control 4: right-head value and coupling gradients against device
    /// central secants (same convention as control 5).
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_right_backward_secants() -> Result<()> {
        let device = cuda()?;
        let (cells, resets) = mixed_cells()?;
        let f = fixture(&device, &cells, 2, &resets)?;
        let s = spec(7, 2);
        let (rows, time) = (cells.len(), cells[0].len());
        let selection = select(&selection_spec(&s)?, 2, &f.address, &f.address, &f.reset)?;
        // Weight only the right (odd) rows.
        let weights = Tensor::from_vec(
            (0..rows * time * 4)
                .map(|i| {
                    if (i / (time * 4)) % 2 == 1 {
                        ((i * 29 % 23) as f32 / 23.0) - 0.5
                    } else {
                        0.0
                    }
                })
                .collect::<Vec<_>>(),
            (rows, time, 4),
            &device,
        )?;
        let loss_of = |v: &Tensor, k: &Tensor| -> Result<Tensor> {
            Ok(murmuration_read(
                &s,
                &MurmurationInputs {
                    address: &f.address,
                    address_norm: &f.norm,
                    value: v,
                    coupling: k,
                    selection: &selection,
                    heads: 2,
                    chirality: Chirality::Paired,
                },
                false,
                None,
            )?
            .delta
            .mul(&weights)?
            .sum_all()?)
        };
        let value = Var::from_tensor(&f.value)?;
        let coupling = Var::from_tensor(&Tensor::new(&[0.8f32, -0.6], &device)?)?;
        let grads = loss_of(value.as_tensor(), coupling.as_tensor())?.backward()?;
        let gv = grads
            .get(value.as_tensor())
            .ok_or_else(|| invalid("no value gradient"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let gk = grads
            .get(coupling.as_tensor())
            .ok_or_else(|| invalid("no coupling gradient"))?
            .to_vec1::<f32>()?;
        let h = 1e-3f64;
        let n = rows * time * 4;
        let check = |a: f32, p: f32, m: f32, what: &str| {
            let s = (p - m) as f64 / (2.0 * h);
            let bound = 2e-3 * s.abs().max(a.abs() as f64).max(1e-3);
            assert!(
                (s - a as f64).abs() <= bound,
                "{what}: analytic {a} secant {s}"
            );
        };
        // Elements of the right rows (rows 1 and 3).
        for idx in [
            time * 4 + 3,
            time * 4 + 21,
            3 * time * 4 + 10,
            3 * time * 4 + 47,
        ] {
            let bump = |sign: f64| -> Result<f32> {
                let e = Tensor::from_vec(
                    (0..n)
                        .map(|i| if i == idx { (sign * h) as f32 } else { 0.0 })
                        .collect::<Vec<_>>(),
                    (rows, time, 4),
                    &device,
                )?;
                Ok(loss_of(&f.value.add(&e)?, coupling.as_tensor())?.to_scalar::<f32>()?)
            };
            check(gv[idx], bump(1.0)?, bump(-1.0)?, &format!("value[{idx}]"));
        }
        let bump = |sign: f64| -> Result<f32> {
            let e = Tensor::new(&[0.0f32, (sign * h) as f32], &device)?;
            Ok(loss_of(&f.value, &coupling.as_tensor().add(&e)?)?.to_scalar::<f32>()?)
        };
        check(gk[1], bump(1.0)?, bump(-1.0)?, "coupling[right head]");
        Ok(())
    }
}
