//! The murmuration read, stage S2: the FBS ternary tree (M4 #2032, card
//! 6151093135, amendment M1 S2-a/b/c 6151441631; Codex FIT for the S2 finite
//! domain 6151549249).
//!
//! **Tree.** Per document (BOS resets it), level 0 is the tokens and every 3
//! consecutive closed level-`l` nodes close into one level-`l+1` node, so
//! the level-`l` node `i` of a document starting at `s` covers the leaves
//! `[s + i 3^l, s + (i + 1) 3^l)`. A node **arrives** at the first position
//! after its last leaf, so a reader at `t` only ever sees nodes whose leaves
//! all lie strictly before `t`; an arrival that would cross a document start
//! is dropped. This structure is index metadata ([`TreePlan`], host).
//!
//! **Parent (S2-a).** With children `k0, k1, k2` (earliest first), continuous
//! addresses `a_k`, signed cells `c_k` and values `v_k`:
//! * `s_k = +1` if `<a_k, a_k0> >= 0`, else `-1` (Codex's tie law);
//!   `a_P = normalise(sum s_k a_k)`, or `a_k0` when that sum's norm is below
//!   1e-6; the parent cell `c_P` is the nearest icosian (K1's law, one shared
//!   kernel). Addresses and cells take no gradient.
//! * `v_P = (1/3) sum_k (c_P (x) conj(c_k)) (x) v_k` with each child's **signed**
//!   cell (the hemisphere flip aligns addresses only, never value transports).
//! * A parent with an address-null child is refused before any arithmetic (no
//!   null-parent policy is registered); a non-finite address is a blocker.
//!
//! **Read (S2-b).** At each level the reader's flock is K1's ring selection
//! over arrived nodes (kernel `murm_ring_select`, same score and tie law, no
//! exploration), transported into the reader's frame exactly as at level 0,
//! and `Delta_tree = sum_l K_{h,l} (m_{t,l} - v_t)` over non-empty levels, one
//! coupling per head and level (initialised at 0 by the caller). Levels add
//! as different scales' summaries; no deduplication against lower levels.
//! The total read is S1's `Delta` plus this. Model arithmetic is CUDA-only.

use candle_core::{DType, Tensor};

use super::kernels::{nearest_cells, ring_select};
#[cfg(test)]
use super::murmuration::hamilton;
use super::murmuration::{carry, Chirality, MAX_FLOCK, NULL_ADDRESS_NORM};
#[cfg(test)]
use super::{geometry, CELLS};
use super::{require_cuda, require_finite, NativeStarling, ABSENT};
use crate::{invalid, Result};

/// The deepest tree level a plan builds (3^10 = 59,049 positions).
pub const MAX_LEVELS: usize = 10;

/// One tree level of a [`TreePlan`]: index metadata for `batch` documents
/// rows, nodes padded to the widest batch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelPlan {
    /// Nodes per batch row (padded).
    pub nodes: usize,
    /// `[batch, nodes, 3]` child indices into the previous level (level 0:
    /// positions), earliest child first.
    pub children: Vec<u32>,
    /// `[batch, nodes]` 1 where the node exists (0 for padding).
    pub exists: Vec<u32>,
    /// `[batch, time]` the node arriving at each position, or [`ABSENT`].
    pub arrival: Vec<u32>,
}

/// The causal FBS tree of a batch: levels `1..=L` (level 0 is the tokens).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreePlan {
    pub batch: usize,
    pub time: usize,
    pub levels: Vec<LevelPlan>,
    /// Document start positions per batch row (index metadata).
    pub starts: Vec<Vec<usize>>,
}

impl TreePlan {
    /// The plan for a `[batch, time]` document-start mask (1 at BOS), with at
    /// most `levels` levels above the tokens. Index metadata only.
    pub fn new(reset: &[u32], batch: usize, time: usize, levels: usize) -> Result<Self> {
        if reset.len() != batch * time || levels > MAX_LEVELS {
            return Err(invalid(format!(
                "a tree plan needs a [batch, time] reset mask and at most {MAX_LEVELS} levels"
            )));
        }
        // Document segments per batch row: [start, end).
        let segments: Vec<Vec<(usize, usize)>> = (0..batch)
            .map(|b| {
                let mut starts: Vec<usize> = (0..time)
                    .filter(|&t| t == 0 || reset[b * time + t] != 0)
                    .collect();
                starts.dedup();
                let mut out = Vec::with_capacity(starts.len());
                for (i, &s) in starts.iter().enumerate() {
                    out.push((s, starts.get(i + 1).copied().unwrap_or(time)));
                }
                out
            })
            .collect();
        // offset[b][seg] = index of the segment's first node in the previous
        // level's per-batch list (level 0: the segment start position).
        let mut offset: Vec<Vec<usize>> = segments
            .iter()
            .map(|segs| segs.iter().map(|&(s, _)| s).collect())
            .collect();
        let mut plans = Vec::new();
        for l in 1..=levels {
            let span = 3usize.pow(l as u32);
            let counts: Vec<Vec<usize>> = segments
                .iter()
                .map(|segs| segs.iter().map(|&(s, e)| (e - s) / span).collect())
                .collect();
            let nodes = counts
                .iter()
                .map(|c| c.iter().sum::<usize>())
                .max()
                .unwrap_or(0);
            if nodes == 0 {
                break;
            }
            let mut children = vec![0u32; batch * nodes * 3];
            let mut exists = vec![0u32; batch * nodes];
            let mut arrival = vec![ABSENT; batch * time];
            let mut next_offset = Vec::with_capacity(batch);
            for b in 0..batch {
                let mut node = 0usize;
                let mut offsets = Vec::with_capacity(segments[b].len());
                for (seg, &(s, e)) in segments[b].iter().enumerate() {
                    offsets.push(node);
                    for i in 0..counts[b][seg] {
                        for k in 0..3 {
                            children[(b * nodes + node) * 3 + k] =
                                (offset[b][seg] + 3 * i + k) as u32;
                        }
                        exists[b * nodes + node] = 1;
                        let arrive = s + (i + 1) * span; // first position after the last leaf
                        if arrive < e {
                            arrival[b * time + arrive] = node as u32;
                        }
                        node += 1;
                    }
                }
                next_offset.push(offsets);
            }
            offset = next_offset;
            plans.push(LevelPlan {
                nodes,
                children,
                exists,
                arrival,
            });
        }
        Ok(Self {
            batch,
            time,
            levels: plans,
            starts: segments
                .iter()
                .map(|segs| segs.iter().map(|&(s, _)| s).collect())
                .collect(),
        })
    }

    /// Level-`l` (1-based) nodes arrived at or before `t` in `b`'s current
    /// document: the count a reader at `t` can draw from (test metadata).
    pub fn available(&self, b: usize, t: usize, level: usize) -> usize {
        let Some(plan) = level.checked_sub(1).and_then(|i| self.levels.get(i)) else {
            return 0;
        };
        let t = t.min(self.time.saturating_sub(1));
        // Fix 4 (Codex 6152388126): count within the current document only.
        let start = self.starts[b]
            .iter()
            .copied()
            .filter(|&s| s <= t)
            .max()
            .unwrap_or(0);
        (start..=t)
            .filter(|&p| plan.arrival[b * self.time + p] != ABSENT)
            .count()
    }
}

/// Inputs of one S2 read on `rows = batch * heads` rows.
pub struct TreeInputs<'a> {
    /// Unit 4-d token addresses `[rows, time, 4]` (no gradient here).
    pub address: &'a Tensor,
    /// Raw address norms `[rows, time]` (null below [`NULL_ADDRESS_NORM`]).
    pub address_norm: &'a Tensor,
    /// Token values `[rows, time, 4]` F32.
    pub value: &'a Tensor,
    /// Token cells `[rows, time]` U32: K1's key cell (S1's selection column).
    pub cells: &'a Tensor,
    /// One coupling per level and head, `[levels, heads]` F32.
    pub coupling: &'a Tensor,
    /// The `[batch, time]` document-start mask on the device (U32).
    pub reset: &'a Tensor,
    pub heads: usize,
    /// Value-transport handedness (M2), the same as the S1 read's.
    pub chirality: Chirality,
}

/// One built level on the device.
pub struct LevelState {
    /// Parent unit addresses `[rows, nodes, 4]` (detached).
    pub address: Tensor,
    /// Parent cells `[rows, nodes]` U32.
    pub cell: Tensor,
    /// Parent values `[rows, nodes, 4]` (differentiable in the token values).
    pub value: Tensor,
}

/// What the S2 read returns (device tensors).
pub struct TreeOutput {
    /// `sum_l K_{h,l} (m_{t,l} - v_t)`, `[rows, time, 4]`.
    pub delta: Tensor,
    /// Flock size per level, `[levels, rows, time]` F32.
    pub flock: Vec<Tensor>,
    /// Per level, the ring selection `[rows, time, K + 2]` U32: arrival
    /// positions in rank order, the home cell and the arriving node's cell
    /// (diagnostics and controls).
    pub selection: Vec<Tensor>,
}

fn row_index(plan: &[u32], heads: usize, per_batch: usize, prev: usize) -> Vec<u32> {
    // Row r = b * heads + h reads batch b's metadata; indices become flat
    // offsets into a [rows, prev] tensor.
    let batch = plan.len() / per_batch.max(1);
    let mut out = Vec::with_capacity(batch * heads * per_batch);
    for b in 0..batch {
        for h in 0..heads {
            let r = b * heads + h;
            out.extend(
                plan[b * per_batch..(b + 1) * per_batch]
                    .iter()
                    .map(|&i| (r * prev) as u32 + i),
            );
        }
    }
    out
}

/// Build every level's parents on the device (S2-a), refusing null-containing
/// parents and non-finite addresses.
pub fn build_levels(plan: &TreePlan, inputs: &TreeInputs<'_>) -> Result<Vec<LevelState>> {
    inputs.chirality.validate(inputs.heads)?;
    require_cuda(inputs.value, "the murmuration tree")?;
    require_finite(inputs.value, "the tree values")?;
    require_finite(inputs.address, "the tree addresses")?;
    require_finite(inputs.address_norm, "the tree address norms")?;
    let (rows, time, _) = inputs.value.dims3()?;
    let heads = inputs.heads;
    if heads == 0 || rows != plan.batch * heads || time != plan.time {
        return Err(invalid("the tree inputs disagree with the plan"));
    }
    let device = inputs.value.device();
    let mut address = inputs.address.to_dtype(DType::F32)?.detach();
    let mut cell = inputs.cells.contiguous()?;
    let mut value = inputs.value.to_dtype(DType::F32)?;
    let mut null = inputs
        .address_norm
        .to_dtype(DType::F32)?
        .lt(NULL_ADDRESS_NORM)?
        .to_dtype(DType::F32)?;
    let mut prev = time;
    let mut built = Vec::with_capacity(plan.levels.len());
    for level in &plan.levels {
        let n = level.nodes;
        let index = Tensor::from_vec(
            row_index(&level.children, heads, n * 3, prev),
            rows * n * 3,
            device,
        )?;
        // Exists is per batch; expand to rows (index metadata).
        let exists = {
            let mut v = Vec::with_capacity(rows * n);
            for b in 0..plan.batch {
                for _ in 0..heads {
                    v.extend(level.exists[b * n..(b + 1) * n].iter().map(|&e| e as f32));
                }
            }
            Tensor::from_vec(v, (rows, n), device)?
        };
        let child_null = null
            .flatten_all()?
            .index_select(&index, 0)?
            .reshape((rows, n, 3))?;
        let refused = child_null
            .sum(2)?
            .mul(&exists)?
            .sum_all()?
            .to_scalar::<f32>()?;
        if refused != 0.0 {
            return Err(invalid(
                "a tree parent contains an address-null child; no null-parent policy is registered (S2 refuses it)",
            ));
        }
        let child_address = address
            .reshape((rows * prev, 4))?
            .index_select(&index, 0)?
            .reshape((rows, n, 3, 4))?;
        let first = child_address.narrow(2, 0, 1)?;
        // s_k = +1 for <a_k, a_k0> >= 0, else -1.
        let sign = child_address
            .broadcast_mul(&first)?
            .sum_keepdim(3)?
            .ge(0.0)?
            .to_dtype(DType::F32)?
            .affine(2.0, -1.0)?;
        let summed = child_address.broadcast_mul(&sign)?.sum(2)?; // [rows, n, 4]
        let norm = summed.sqr()?.sum_keepdim(2)?.sqrt()?;
        let degenerate = norm.lt(1e-6)?; // [rows, n, 1]
        let safe_norm = degenerate.where_cond(&norm.ones_like()?, &norm)?;
        let parent_address = degenerate
            .broadcast_as((rows, n, 4))?
            .where_cond(&first.squeeze(2)?, &summed.broadcast_div(&safe_norm)?)?
            .contiguous()?;
        let parent_cell =
            nearest_cells(&parent_address.reshape((rows * n, 4))?)?.reshape((rows, n))?;
        // v_P = (1/3) sum_k T_{P,k}(v_k) with signed child cells, in each
        // head's handedness (M2: left c_P conj(c_k) (x) v, right v (x) conj(c_k) c_P).
        let child_cell = cell
            .flatten_all()?
            .index_select(&index, 0)?
            .reshape((rows, n, 3))?;
        // Codex 6153323237: a padded (nonexistent) parent may gather an
        // address-null token's ABSENT cell; give it a valid cell BEFORE the
        // table and root gathers. Real null-containing parents were refused above.
        let real = exists
            .gt(0.0)?
            .unsqueeze(2)?
            .broadcast_as((rows, n, 3))?
            .contiguous()?;
        let child_cell = real.where_cond(
            &child_cell,
            &Tensor::zeros((rows, n, 3), DType::U32, device)?,
        )?;
        let child_value = value
            .contiguous()?
            .reshape((rows * prev, 4))?
            .index_select(&index, 0)?
            .reshape((rows, n, 3, 4))?;
        let carried = carry(
            inputs.chirality,
            heads,
            &parent_cell
                .unsqueeze(2)?
                .broadcast_as((rows, n, 3))?
                .contiguous()?,
            &child_cell,
            &child_value,
        )?;
        let parent_value = carried
            .sum(2)?
            .affine(1.0 / 3.0, 0.0)?
            .broadcast_mul(&exists.unsqueeze(2)?)?;
        address = parent_address.detach();
        cell = parent_cell;
        value = parent_value;
        null = Tensor::zeros((rows, n), DType::F32, device)?; // refused above otherwise
        prev = n;
        built.push(LevelState {
            address: address.clone(),
            cell: cell.clone(),
            value: value.clone(),
        });
    }
    Ok(built)
}

/// The S2 read: per level, K1's ring selection over arrived nodes, exact
/// transport, the flock mean and its own coupling (S2-b).
pub fn murmuration_tree_read(
    spec: &NativeStarling,
    plan: &TreePlan,
    inputs: &TreeInputs<'_>,
) -> Result<TreeOutput> {
    spec.validate()?;
    inputs.chirality.validate(inputs.heads)?;
    // Fix 3: GPU admission at the public entry, before any early return.
    require_cuda(inputs.value, "the murmuration tree")?;
    require_cuda(inputs.address, "the murmuration tree")?;
    // Fix 2: refuse NaN/Inf before any arithmetic.
    require_finite(inputs.value, "the tree values")?;
    require_finite(inputs.address, "the tree addresses")?;
    require_finite(inputs.address_norm, "the tree address norms")?;
    require_finite(inputs.coupling, "the tree coupling")?;
    let k = spec.kept;
    if k == 0 || k > MAX_FLOCK {
        return Err(invalid(format!(
            "the murmuration flock keeps 1..={MAX_FLOCK} neighbours"
        )));
    }
    let (rows, time, _) = inputs.value.dims3()?;
    let heads = inputs.heads;
    let levels = plan.levels.len();
    if inputs.coupling.dims() != [levels, heads]
        && !(levels == 0 && inputs.coupling.elem_count() == 0)
    {
        return Err(invalid(format!(
            "the tree coupling must be [levels = {levels}, heads = {heads}]"
        )));
    }
    let device = inputs.value.device();
    let value = inputs.value.to_dtype(DType::F32)?;
    let mut delta = Tensor::zeros((rows, time, 4), DType::F32, device)?;
    let mut flocks = Vec::with_capacity(levels);
    let mut selections = Vec::with_capacity(levels);
    if levels == 0 {
        return Ok(TreeOutput {
            delta,
            flock: flocks,
            selection: selections,
        });
    }
    let built = build_levels(plan, inputs)?;
    let reader_null = inputs
        .address_norm
        .to_dtype(DType::F32)?
        .lt(NULL_ADDRESS_NORM)?;
    let head_index = Tensor::from_vec(
        (0..rows).map(|r| (r % heads) as u32).collect::<Vec<_>>(),
        rows,
        device,
    )?;
    let reciprocal = Tensor::from_vec(
        (0..=MAX_FLOCK)
            .map(|n| if n == 0 { 0.0f32 } else { 1.0 / n as f32 })
            .collect::<Vec<_>>(),
        MAX_FLOCK + 1,
        device,
    )?;
    for (l, (level, state)) in plan.levels.iter().zip(&built).enumerate() {
        let n = level.nodes;
        // Arrival timeline per row: the node arriving at t (index metadata).
        let arrival_rows = row_index(&level.arrival, heads, time, 0);
        // Per-row arrival mask (index metadata), [rows, time].
        let valid: Vec<u32> = arrival_rows
            .iter()
            .map(|&a| u32::from(a != ABSENT))
            .collect();
        let node_at: Vec<u32> = arrival_rows
            .iter()
            .enumerate()
            .map(|(i, &a)| {
                let r = i / time;
                (r * n) as u32 + if a == ABSENT { 0 } else { a }
            })
            .collect();
        let node_at = Tensor::from_vec(node_at, rows * time, device)?;
        let node_address = state
            .address
            .reshape((rows * n, 4))?
            .index_select(&node_at, 0)?
            .reshape((rows, time, 4))?;
        let node_cell = state
            .cell
            .flatten_all()?
            .index_select(&node_at, 0)?
            .reshape((rows, time))?;
        let selection = ring_select(
            &NativeStarling {
                explore: 0,
                ..*spec
            },
            heads,
            &inputs.address.to_dtype(DType::F32)?.detach(),
            &node_address,
            &node_cell,
            &Tensor::from_vec(valid, (rows, time), device)?,
            inputs.reset,
        )?; // [rows, time, K + 2]
        let slots = selection.narrow(2, 0, k)?.contiguous()?; // arrival positions
        let home = selection.narrow(2, k, 1)?.squeeze(2)?.contiguous()?;
        let present = slots.ne(&Tensor::full(ABSENT, (rows, time, k), device)?)?;
        let safe =
            present.where_cond(&slots, &Tensor::zeros((rows, time, k), DType::U32, device)?)?;
        let base = Tensor::from_vec(
            (0..rows).map(|r| (r * time) as u32).collect::<Vec<_>>(),
            (rows, 1, 1),
            device,
        )?;
        let flat = safe.broadcast_add(&base)?.flatten_all()?;
        // Arrival position -> node (row-flat index into the level's nodes).
        let nodes = node_at.index_select(&flat, 0)?; // [rows * time * K]
        let neighbour_cell = state.cell.flatten_all()?.index_select(&nodes, 0)?;
        let neighbour_value = state
            .value
            .reshape((rows * n, 4))?
            .index_select(&nodes, 0)?
            .reshape((rows, time, k, 4))?;
        let live = present.to_dtype(DType::F32)?;
        let flock = live.sum(2)?; // [rows, time]
        let inverse = reciprocal
            .index_select(&flock.to_dtype(DType::U32)?.flatten_all()?, 0)?
            .reshape((rows, time, 1))?;
        let mean = carry(
            inputs.chirality,
            heads,
            &home
                .unsqueeze(2)?
                .broadcast_as((rows, time, k))?
                .contiguous()?,
            &neighbour_cell.reshape((rows, time, k))?,
            &neighbour_value,
        )?
        .broadcast_mul(&live.unsqueeze(3)?)?
        .sum(2)?
        .broadcast_mul(&inverse)?;
        let reads = flock
            .gt(0.0)?
            .mul(&reader_null.eq(0u8)?)?
            .to_dtype(DType::F32)?
            .unsqueeze(2)?;
        let coupling = inputs
            .coupling
            .to_dtype(DType::F32)?
            .narrow(0, l, 1)?
            .squeeze(0)?
            .index_select(&head_index, 0)?
            .reshape((rows, 1, 1))?;
        delta = delta.add(
            &mean
                .sub(&value)?
                .broadcast_mul(&coupling)?
                .broadcast_mul(&reads)?,
        )?;
        flocks.push(flock);
        selections.push(selection);
    }
    Ok(TreeOutput {
        delta,
        flock: flocks,
        selection: selections,
    })
}

#[cfg(test)]
#[allow(clippy::needless_range_loop, clippy::too_many_arguments)]
mod tests {
    use super::*;
    use crate::starling_native::murmuration::tests::{check_secant, secant};
    use crate::starling_native::murmuration::{
        murmuration_read, murmuration_select, selection_spec, MurmurationInputs,
    };
    use crate::starling_native::Credit;
    use candle_core::{Device, Var};

    /// S2-c (host, index metadata): level availability is floor(n_past/3^l)
    /// at the boundaries 2/3, 8/9, 26/27 and 4095/4096 of strictly-past
    /// tokens, and the touched-value bound at 4096 is 48 (conservative 56).
    #[test]
    fn murm_tree_level_counts_and_bounds() -> Result<()> {
        let time = 4097;
        let mut reset = vec![0u32; time];
        reset[0] = 1;
        let plan = TreePlan::new(&reset, 1, time, MAX_LEVELS)?;
        for n_past in [2usize, 3, 8, 9, 26, 27, 4095, 4096] {
            let t = n_past; // the reader at t has n_past = t strictly-past tokens
            for l in 1..=plan.levels.len() {
                assert_eq!(
                    plan.available(0, t, l),
                    n_past / 3usize.pow(l as u32),
                    "n_past {n_past}, level {l}"
                );
            }
        }
        let touched: usize = (0..=plan.levels.len())
            .map(|l| MAX_FLOCK.min(4096 / 3usize.pow(l as u32)))
            .sum();
        assert_eq!(touched, 48);
        assert_eq!(MAX_FLOCK * (1 + plan.levels.len()), 56);
        assert_eq!(plan.levels.len(), 7); // floor(log3 4096)
        Ok(())
    }

    /// The plan's triples are consecutive, disjoint, start at each document,
    /// and no arrival crosses a document start.
    #[test]
    fn murm_tree_plan_respects_documents() -> Result<()> {
        let time = 20;
        let mut reset = vec![0u32; time];
        reset[0] = 1;
        reset[10] = 1;
        let plan = TreePlan::new(&reset, 1, time, 3)?;
        let one = &plan.levels[0];
        // Document [0, 10): 3 level-1 nodes; document [10, 20): 3 more.
        assert_eq!(one.exists.iter().filter(|&&e| e == 1).count(), 6);
        let kids: Vec<[u32; 3]> = (0..6)
            .map(|i| {
                [
                    one.children[i * 3],
                    one.children[i * 3 + 1],
                    one.children[i * 3 + 2],
                ]
            })
            .collect();
        assert_eq!(kids[0], [0, 1, 2]);
        assert_eq!(kids[2], [6, 7, 8]);
        assert_eq!(kids[3], [10, 11, 12]);
        // Node 2 ([6, 9)) arrives at 9; the node [16, 19) arrives at 19.
        assert_eq!(one.arrival[9], 2);
        assert_eq!(one.arrival[19], 5);
        // Level 2: [0, 9) arrives at 9; [10, 19) arrives at 19.
        let two = &plan.levels[1];
        assert_eq!(two.arrival[9], 0);
        assert_eq!(two.arrival[19], 1);
        // No level-1 node arrives at a document start.
        assert_eq!(one.arrival[10], ABSENT);
        // Fix 4: availability counts the current document only.
        assert_eq!(plan.available(0, 9, 1), 3); // [0,3) [3,6) [6,9) arrived by t = 9
        assert_eq!(plan.available(0, 10, 1), 0); // BOS at 10: a fresh document
        assert_eq!(plan.available(0, 13, 1), 1); // [10,13) arrives at 13
        assert_eq!(plan.available(0, 19, 2), 1); // [10,19) arrives at 19
        Ok(())
    }

    #[test]
    fn murm_tree_refuses_cpu() -> Result<()> {
        // Refused at either layer (the vendored backend refuses CPU tensors;
        // the read refuses non-CUDA input), including a zero-level plan, which
        // must not bypass admission (fix 3, Codex 6152388126).
        let device = Device::Cpu;
        for time in [4usize, 2] {
            let attempt = || -> Result<TreeOutput> {
                let mut reset = vec![0u32; time];
                reset[0] = 1;
                let plan = TreePlan::new(&reset, 1, time, 1)?;
                let x = Tensor::zeros((1, time, 4), DType::F32, &device)?;
                let norm = Tensor::ones((1, time), DType::F32, &device)?;
                let cells = Tensor::zeros((1, time), DType::U32, &device)?;
                let coupling = Tensor::zeros((plan.levels.len(), 1), DType::F32, &device)?;
                let reset_t = Tensor::zeros((1, time), DType::U32, &device)?;
                murmuration_tree_read(
                    &spec(),
                    &plan,
                    &TreeInputs {
                        address: &x,
                        address_norm: &norm,
                        value: &x,
                        cells: &cells,
                        coupling: &coupling,
                        reset: &reset_t,
                        heads: 1,
                        chirality: Chirality::Left,
                    },
                )
            };
            assert!(
                attempt().is_err(),
                "time {time}: the CPU tree read was not refused"
            );
        }
        // The zero-level plan really has no levels (the early-return case).
        assert!(TreePlan::new(&[1, 0], 1, 2, 1)?.levels.is_empty());
        Ok(())
    }

    fn spec() -> NativeStarling {
        NativeStarling {
            depth: 8,
            kept: 7,
            explore: 0,
            bos_id: 0,
            credit: Credit::FIXTURE,
        }
    }

    fn roots() -> Result<Vec<[f32; 4]>> {
        Ok(geometry()?
            .0
            .chunks(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect())
    }

    fn antipode(c: usize) -> Result<usize> {
        let r = roots()?;
        r.iter()
            .position(|q| (0..4).all(|i| (q[i] + r[c][i]).abs() < 1e-6))
            .ok_or_else(|| invalid("no antipode"))
    }

    /// Token tensors for one row at the given vertices, with K1's key cells.
    fn tokens(
        device: &Device,
        cells: &[usize],
        reset_at: &[usize],
    ) -> Result<(Tensor, Tensor, Tensor, Tensor, Vec<u32>)> {
        let r = roots()?;
        let time = cells.len();
        let address: Vec<f32> = cells.iter().flat_map(|&c| r[c]).collect();
        let value: Vec<f32> = (0..time * 4)
            .map(|i| (((i * 13) % 29) as f32 / 29.0) * 1.6 - 0.8)
            .collect();
        let mut reset = vec![0u32; time];
        for &t in reset_at {
            reset[t] = 1;
        }
        let address = Tensor::from_vec(address, (1, time, 4), device)?;
        let value = Tensor::from_vec(value, (1, time, 4), device)?;
        let reset_t = Tensor::from_vec(reset.clone(), (1, time), device)?;
        let s = selection_spec(&spec())?;
        let selection = murmuration_select(
            &spec(),
            1,
            &address,
            &Tensor::ones((1, address.dim(1)?), DType::F32, address.device())?,
            &reset_t,
        )?;
        let key_cell = selection
            .narrow(2, s.slots() + 1, 1)?
            .squeeze(2)?
            .contiguous()?;
        Ok((address, value, key_cell, reset_t, reset))
    }

    /// Control (S2 closure known answer): a level-1 parent of children at
    /// `a`, `-a` (a single-child antipodal flip) and a neighbour `b` has the
    /// hemisphere-aligned address, and its value transports each child by its
    /// **signed** cell; checked against direct f32 Hamilton products of the
    /// roots (not the exact table). The reader at t = 3 then sees exactly
    /// that node.
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_tree_closure_known_answer() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let r = roots()?;
        let (a, b) = (30usize, geometry()?.1[30 * 12] as usize);
        let cells = [a, antipode(a)?, b, a];
        let (address, value, key_cell, reset_t, reset) = tokens(&device, &cells, &[0])?;
        let plan = TreePlan::new(&reset, 1, 4, 1)?;
        let coupling = Tensor::new(&[[1.0f32]], &device)?;
        let norm = Tensor::ones((1, 4), DType::F32, &device)?;
        let inputs = TreeInputs {
            address: &address,
            address_norm: &norm,
            value: &value,
            cells: &key_cell,
            coupling: &coupling,
            reset: &reset_t,
            heads: 1,
            chirality: Chirality::Left,
        };
        let built = build_levels(&plan, &inputs)?;
        // Expected address: a + (-1)(-a) + b, normalised (b . a > 0).
        let expect: Vec<f32> = {
            let s: Vec<f32> = (0..4).map(|i| 2.0 * r[a][i] + r[b][i]).collect();
            let n = s.iter().map(|x| x * x).sum::<f32>().sqrt();
            s.iter().map(|x| x / n).collect()
        };
        let got = built[0]
            .address
            .flatten_all()?
            .narrow(0, 0, 4)?
            .to_vec1::<f32>()?;
        for i in 0..4 {
            assert!(
                (got[i] - expect[i]).abs() <= 1e-6,
                "parent address {got:?} vs {expect:?}"
            );
        }
        // Expected value: (1/3) sum_k (c_P (x) conj(c_k)) (x) v_k, signed c_k.
        // Fix 5: the expected parent cell is pinned independently. For 2a + b with
        // a, b adjacent (dot phi/2), <a^, a> ~ 0.979 beats every other vertex
        // (at most (phi + 1)/|2a + b| ~ 0.912), so the parent cell is a = 30.
        let c_p = a;
        let got_cell = built[0].cell.flatten_all()?.to_vec1::<u32>()?[0] as usize;
        assert_eq!(
            got_cell, c_p,
            "the parent cell is not the pinned nearest vertex"
        );
        let roots_t = Tensor::from_vec(geometry()?.0, (CELLS, 4), &device)?;
        let conj = Tensor::new(&[1f32, -1.0, -1.0, -1.0], &device)?;
        let cp = roots_t.narrow(0, c_p, 1)?.reshape(4)?;
        let mut sum = Tensor::zeros(4, DType::F32, &device)?;
        for (k, &ck) in cells[..3].iter().enumerate() {
            let g = hamilton(&cp, &roots_t.narrow(0, ck, 1)?.reshape(4)?.mul(&conj)?)?;
            sum = sum.add(&hamilton(&g, &value.narrow(1, k, 1)?.reshape(4)?)?)?;
        }
        let want = sum.affine(1.0 / 3.0, 0.0)?;
        let worst = built[0]
            .value
            .flatten_all()?
            .narrow(0, 0, 4)?
            .sub(&want)?
            .abs()?
            .max(0)?
            .to_scalar::<f32>()?;
        assert!(worst <= 1e-6, "parent value off by {worst}");
        // The reader at t = 3 (strictly after the node's last leaf) reads it.
        let out = murmuration_tree_read(&spec(), &plan, &inputs)?;
        let flock = out.flock[0].to_vec2::<f32>()?;
        assert_eq!(flock[0], vec![0.0, 0.0, 0.0, 1.0]);
        Ok(())
    }

    /// Fix 5b: the registered hemisphere tie. Children at the axis roots
    /// (1,0,0,0), (0,1,0,0), (1,0,0,0): the second has dot exactly 0 with the
    /// first, so s = +1 and the parent address is (2,1,0,0)/sqrt5, distinct from
    /// the s = -1 alternative (2,-1,0,0)/sqrt5.
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_tree_hemisphere_tie() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let r = roots()?;
        let (x, y) = (1usize, 3usize);
        assert_eq!(r[x], [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(r[y], [0.0, 1.0, 0.0, 0.0]);
        let (address, value, key_cell, reset_t, reset) = tokens(&device, &[x, y, x, x], &[0])?;
        let plan = TreePlan::new(&reset, 1, 4, 1)?;
        let coupling = Tensor::new(&[[1.0f32]], &device)?;
        let norm = Tensor::ones((1, 4), DType::F32, &device)?;
        let built = build_levels(
            &plan,
            &TreeInputs {
                address: &address,
                address_norm: &norm,
                value: &value,
                cells: &key_cell,
                coupling: &coupling,
                reset: &reset_t,
                heads: 1,
                chirality: Chirality::Left,
            },
        )?;
        let got = built[0]
            .address
            .flatten_all()?
            .narrow(0, 0, 4)?
            .to_vec1::<f32>()?;
        let s5 = 5f32.sqrt();
        let want = [2.0 / s5, 1.0 / s5, 0.0, 0.0];
        for i in 0..4 {
            assert!(
                (got[i] - want[i]).abs() <= 1e-6,
                "tie parent {got:?} vs {want:?}"
            );
        }
        Ok(())
    }

    /// Control (S2 strict causality): perturbing token values at s > t leaves
    /// the tree Delta at every t bit-identical.
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_tree_strict_causality() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let adjacency = geometry()?.1;
        let a = 30usize;
        let cells: Vec<usize> = (0..30)
            .map(|t| adjacency[a * 12 + t % 12] as usize)
            .collect();
        let (address, value, key_cell, reset_t, reset) = tokens(&device, &cells, &[0, 17])?;
        let plan = TreePlan::new(&reset, 1, 30, 3)?;
        let coupling = Tensor::new(&[[0.7f32], [-0.4], [1.3]], &device)?;
        let norm = Tensor::ones((1, 30), DType::F32, &device)?;
        let run = |v: &Tensor| -> Result<Vec<Vec<Vec<f32>>>> {
            Ok(murmuration_tree_read(
                &spec(),
                &plan,
                &TreeInputs {
                    address: &address,
                    address_norm: &norm,
                    value: v,
                    cells: &key_cell,
                    coupling: &coupling,
                    reset: &reset_t,
                    heads: 1,
                    chirality: Chirality::Left,
                },
            )?
            .delta
            .to_vec3()?)
        };
        let before = run(&value)?;
        for cut in [8usize, 16, 26] {
            let mask = Tensor::from_vec(
                (0..30)
                    .map(|t| if t > cut { 0.29f32 } else { 0.0 })
                    .collect::<Vec<_>>(),
                (1, 30, 1),
                &device,
            )?;
            let after = run(&value.broadcast_add(&mask)?)?;
            for t in 0..=cut {
                for i in 0..4 {
                    assert_eq!(before[0][t][i].to_bits(), after[0][t][i].to_bits(), "t {t}");
                }
            }
        }
        Ok(())
    }

    /// Control (S2 secants): token-value and per-level coupling gradients of
    /// the tree read against device central secants; same convention as S1:
    /// pass iff |s - a| <= 2e-3 * max(|s|, |a|, 1e-3).
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_tree_backward_secants() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let adjacency = geometry()?.1;
        let a = 30usize;
        let cells: Vec<usize> = (0..28)
            .map(|t| {
                if t % 4 == 0 {
                    a
                } else {
                    adjacency[a * 12 + t % 12] as usize
                }
            })
            .collect();
        let (address, value, key_cell, reset_t, reset) = tokens(&device, &cells, &[0])?;
        let plan = TreePlan::new(&reset, 1, 28, 3)?;
        let norm = Tensor::ones((1, 28), DType::F32, &device)?;
        let weights = Tensor::from_vec(
            (0..28 * 4)
                .map(|i| ((i * 37 % 17) as f32 / 17.0) - 0.5)
                .collect::<Vec<_>>(),
            (1, 28, 4),
            &device,
        )?;
        let v = Var::from_tensor(&value)?;
        let k = Var::from_tensor(&Tensor::new(&[[0.9f32], [-0.7], [0.5]], &device)?)?;
        let loss = |v: &Tensor, k: &Tensor| -> Result<Tensor> {
            Ok(murmuration_tree_read(
                &spec(),
                &plan,
                &TreeInputs {
                    address: &address,
                    address_norm: &norm,
                    value: v,
                    cells: &key_cell,
                    coupling: k,
                    reset: &reset_t,
                    heads: 1,
                    chirality: Chirality::Left,
                },
            )?
            .delta)
        };
        let grads = loss(v.as_tensor(), k.as_tensor())?
            .mul(&weights)?
            .sum_all()?
            .backward()?;
        let gv = grads
            .get(v.as_tensor())
            .ok_or_else(|| invalid("no value grad"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let gk = grads
            .get(k.as_tensor())
            .ok_or_else(|| invalid("no coupling grad"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let h = 1e-3f64;
        for idx in [0usize, 9, 30, 47, 66, 90, 107] {
            let bump = |sign: f64| -> Result<Tensor> {
                let e = Tensor::from_vec(
                    (0..28 * 4)
                        .map(|i| if i == idx { (sign * h) as f32 } else { 0.0 })
                        .collect::<Vec<_>>(),
                    (1, 28, 4),
                    &device,
                )?;
                loss(&value.add(&e)?, k.as_tensor())
            };
            check_secant(
                gv[idx],
                secant(&bump(1.0)?, &bump(-1.0)?, &weights, h)?,
                &format!("value[{idx}]"),
            );
        }
        for l in 0..3 {
            let bump = |sign: f64| -> Result<Tensor> {
                let e = Tensor::from_vec(
                    (0..3)
                        .map(|i| if i == l { (sign * h) as f32 } else { 0.0 })
                        .collect::<Vec<_>>(),
                    (3, 1),
                    &device,
                )?;
                loss(&value, &k.as_tensor().add(&e)?)
            };
            check_secant(
                gk[l],
                secant(&bump(1.0)?, &bump(-1.0)?, &weights, h)?,
                &format!("coupling[{l}]"),
            );
        }
        Ok(())
    }

    /// Control (S2 null-parent refusal): a parent with an address-null child
    /// is refused before arithmetic; the S1 read on the same tokens is not.
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_tree_null_parent_refused() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let a = 30usize;
        let (address, value, key_cell, reset_t, reset) = tokens(&device, &[a, a, a, a], &[0])?;
        let plan = TreePlan::new(&reset, 1, 4, 1)?;
        let norm = Tensor::new(&[[1f32, 0.0, 1.0, 1.0]], &device)?;
        let coupling = Tensor::new(&[[1.0f32]], &device)?;
        let tree = murmuration_tree_read(
            &spec(),
            &plan,
            &TreeInputs {
                address: &address,
                address_norm: &norm,
                value: &value,
                cells: &key_cell,
                coupling: &coupling,
                reset: &reset_t,
                heads: 1,
                chirality: Chirality::Left,
            },
        );
        assert!(tree.is_err(), "a null-containing parent was not refused");
        let selection = murmuration_select(
            &spec(),
            1,
            &address,
            &Tensor::ones((1, address.dim(1)?), DType::F32, address.device())?,
            &reset_t,
        )?;
        let k = Tensor::new(&[1.0f32], &device)?;
        assert!(murmuration_read(
            &spec(),
            &MurmurationInputs {
                address: &address,
                address_norm: &norm,
                value: &value,
                coupling: &k,
                selection: &selection,
                heads: 1,
                chirality: Chirality::Left,
            },
            false,
            None,
        )
        .is_ok());
        Ok(())
    }

    /// Two rows (one batch, two heads) at the same vertices, distinct values
    /// per row; the token cells come from the murmuration selector.
    fn paired_tokens(
        device: &Device,
        cells: &[usize],
        reset_at: &[usize],
    ) -> Result<(Tensor, Tensor, Tensor, Tensor, Vec<u32>, Tensor)> {
        let r = roots()?;
        let time = cells.len();
        let address: Vec<f32> = (0..2)
            .flat_map(|_| cells.iter().flat_map(|&c| r[c]))
            .collect();
        let value: Vec<f32> = (0..2 * time * 4)
            .map(|i| (((i * 17) % 31) as f32 / 31.0) * 1.6 - 0.8)
            .collect();
        let mut reset = vec![0u32; time];
        for &t in reset_at {
            reset[t] = 1;
        }
        let address = Tensor::from_vec(address, (2, time, 4), device)?;
        let value = Tensor::from_vec(value, (2, time, 4), device)?;
        let reset_t = Tensor::from_vec(reset.clone(), (1, time), device)?;
        let norm = Tensor::ones((2, time), DType::F32, device)?;
        let s = selection_spec(&spec())?;
        let selection = murmuration_select(&spec(), 2, &address, &norm, &reset_t)?;
        let key_cell = selection
            .narrow(2, s.slots() + 1, 1)?
            .squeeze(2)?
            .contiguous()?;
        Ok((address, value, key_cell, reset_t, reset, norm))
    }

    fn near_cells(len: usize) -> Result<Vec<usize>> {
        let adjacency = geometry()?.1;
        let a = 30usize;
        Ok((0..len)
            .map(|t| {
                if t % 4 == 0 {
                    a
                } else {
                    adjacency[a * 12 + t % 12] as usize
                }
            })
            .collect())
    }

    /// Fix 2 control (S2): NaN or infinity in values, norms, addresses or
    /// couplings is refused before arithmetic.
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_tree_nonfinite_refused() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let cells = near_cells(28)?;
        let (address, value, key_cell, reset_t, reset) = tokens(&device, &cells, &[0])?;
        let plan = TreePlan::new(&reset, 1, 28, 3)?;
        let norm = Tensor::ones((1, 28), DType::F32, &device)?;
        let k = Tensor::new(&[[0.5f32], [0.0], [1.0]], &device)?;
        let poke = |x: &Tensor, at: usize, bad: f32| -> Result<Tensor> {
            let mask: Vec<f32> = (0..x.elem_count())
                .map(|i| if i == at { bad } else { 0.0 })
                .collect();
            Ok(x.add(&Tensor::from_vec(mask, x.shape(), &device)?)?)
        };
        let read = |a: &Tensor, v: &Tensor, n: &Tensor, k: &Tensor| {
            murmuration_tree_read(
                &spec(),
                &plan,
                &TreeInputs {
                    address: a,
                    address_norm: n,
                    value: v,
                    cells: &key_cell,
                    coupling: k,
                    reset: &reset_t,
                    heads: 1,
                    chirality: Chirality::Left,
                },
            )
        };
        assert!(read(&address, &value, &norm, &k).is_ok());
        assert!(read(&address, &poke(&value, 2, f32::NAN)?, &norm, &k).is_err());
        assert!(read(&address, &value, &poke(&norm, 7, f32::INFINITY)?, &k).is_err());
        assert!(read(&poke(&address, 11, f32::NAN)?, &value, &norm, &k).is_err());
        assert!(read(&address, &value, &norm, &poke(&k, 1, f32::NAN)?).is_err());
        Ok(())
    }

    fn tree_read(
        plan: &TreePlan,
        address: &Tensor,
        value: &Tensor,
        key_cell: &Tensor,
        norm: &Tensor,
        reset_t: &Tensor,
        coupling: &Tensor,
        chirality: Chirality,
    ) -> Result<TreeOutput> {
        murmuration_tree_read(
            &spec(),
            plan,
            &TreeInputs {
                address,
                address_norm: norm,
                value,
                cells: key_cell,
                coupling,
                reset: reset_t,
                heads: 2,
                chirality,
            },
        )
    }

    /// M2 control 2 on S2 (Codex 6152708506 item 2): under `Paired` the even
    /// (left) head's tree read is bit-identical to `Left`; the odd head differs.
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_tree_left_heads_unchanged() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let cells = near_cells(28)?;
        let (address, value, key_cell, reset_t, reset, norm) =
            paired_tokens(&device, &cells, &[0])?;
        let plan = TreePlan::new(&reset, 1, 28, 3)?;
        let k = Tensor::new(&[[0.9f32, -0.7], [0.5, 0.6], [-0.4, 1.1]], &device)?;
        let left: Vec<Vec<Vec<f32>>> = tree_read(
            &plan,
            &address,
            &value,
            &key_cell,
            &norm,
            &reset_t,
            &k,
            Chirality::Left,
        )?
        .delta
        .to_vec3()?;
        let paired: Vec<Vec<Vec<f32>>> = tree_read(
            &plan,
            &address,
            &value,
            &key_cell,
            &norm,
            &reset_t,
            &k,
            Chirality::Paired,
        )?
        .delta
        .to_vec3()?;
        for t in 0..28 {
            for i in 0..4 {
                assert_eq!(left[0][t][i].to_bits(), paired[0][t][i].to_bits(), "t {t}");
            }
        }
        assert_ne!(left[1], paired[1], "the right head did not change");
        Ok(())
    }

    /// M2 control 3 on S2: the right head's multi-level tree read against an
    /// independent device reference. Parents are rebuilt from the children's
    /// signed cells with direct f32 products `v (x) (conj(c_k) (x) c_P)`, and each
    /// reader's per-level flock (taken from the selection, index metadata) is
    /// carried by `v_P (x) (conj(c_P) (x) c_t)`. Covers flocks of 0, 1 and 7.
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_tree_right_forward_reference() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let cells = near_cells(28)?;
        let (address, value, key_cell, reset_t, reset, norm) =
            paired_tokens(&device, &cells, &[0])?;
        let plan = TreePlan::new(&reset, 1, 28, 3)?;
        let k = Tensor::new(&[[0.9f32, -0.7], [0.5, 0.6], [-0.4, 1.1]], &device)?;
        let out = tree_read(
            &plan,
            &address,
            &value,
            &key_cell,
            &norm,
            &reset_t,
            &k,
            Chirality::Paired,
        )?;
        let inputs = TreeInputs {
            address: &address,
            address_norm: &norm,
            value: &value,
            cells: &key_cell,
            coupling: &k,
            reset: &reset_t,
            heads: 2,
            chirality: Chirality::Paired,
        };
        let built = build_levels(&plan, &inputs)?;
        let roots_t = Tensor::from_vec(geometry()?.0, (CELLS, 4), &device)?;
        let conj = Tensor::new(&[1f32, -1.0, -1.0, -1.0], &device)?;
        let root = |c: usize| -> Result<Tensor> { Ok(roots_t.narrow(0, c, 1)?.reshape(4)?) };
        let row = 1usize;
        // Parents, rebuilt independently for the right row.
        let mut prev_cells: Vec<usize> = key_cell.to_vec2::<u32>()?[row]
            .iter()
            .map(|&c| c as usize)
            .collect();
        let mut prev_vals: Vec<Tensor> = (0..28)
            .map(|t| Ok(value.narrow(0, row, 1)?.narrow(1, t, 1)?.reshape(4)?))
            .collect::<Result<_>>()?;
        let mut levels_ref = Vec::new();
        for (l, level) in plan.levels.iter().enumerate() {
            let pc: Vec<usize> = built[l].cell.to_vec2::<u32>()?[row]
                .iter()
                .map(|&c| c as usize)
                .collect();
            let mut vals = Vec::with_capacity(level.nodes);
            for node in 0..level.nodes {
                let mut sum = Tensor::zeros(4, DType::F32, &device)?;
                for kk in 0..3 {
                    let child = level.children[node * 3 + kk] as usize;
                    let h = hamilton(&root(prev_cells[child])?.mul(&conj)?, &root(pc[node])?)?;
                    sum = sum.add(&hamilton(&prev_vals[child], &h)?)?;
                }
                vals.push(sum.affine(1.0 / 3.0, 0.0)?);
            }
            let got = built[l]
                .value
                .narrow(0, row, 1)?
                .reshape((level.nodes, 4))?;
            let want = Tensor::stack(&vals, 0)?;
            let worst = got
                .sub(&want)?
                .abs()?
                .flatten_all()?
                .max(0)?
                .to_scalar::<f32>()?;
            assert!(
                worst <= 1e-6,
                "level {} right parents off by {worst}",
                l + 1
            );
            prev_cells = pc;
            prev_vals = vals.clone();
            levels_ref.push((prev_cells.clone(), vals));
        }
        // Readers.
        let k_rows = k.to_vec2::<f32>()?;
        let mut sizes = std::collections::BTreeSet::new();
        for t in 0..28 {
            let v_t = value.narrow(0, row, 1)?.narrow(1, t, 1)?.reshape(4)?;
            let mut want = Tensor::zeros(4, DType::F32, &device)?;
            for (l, level) in plan.levels.iter().enumerate() {
                let picks = out.selection[l].to_vec3::<u32>()?;
                let home = picks[row][t][spec().kept] as usize;
                let (node_cells, node_vals) = &levels_ref[l];
                let mut sum = Tensor::zeros(4, DType::F32, &device)?;
                let mut count = 0usize;
                for &p in &picks[row][t][..spec().kept] {
                    if p == ABSENT {
                        continue;
                    }
                    let node = level.arrival[p as usize] as usize;
                    let h = hamilton(&root(node_cells[node])?.mul(&conj)?, &root(home)?)?;
                    sum = sum.add(&hamilton(&node_vals[node], &h)?)?;
                    count += 1;
                }
                sizes.insert(count);
                if count > 0 {
                    want = want.add(
                        &sum.affine(1.0 / count as f64, 0.0)?
                            .sub(&v_t)?
                            .affine(k_rows[l][row] as f64, 0.0)?,
                    )?;
                }
            }
            let got = out.delta.narrow(0, row, 1)?.narrow(1, t, 1)?.reshape(4)?;
            let worst = got.sub(&want)?.abs()?.max(0)?.to_scalar::<f32>()?;
            assert!(worst <= 1e-6, "t {t}: right tree read off by {worst}");
        }
        assert!(
            sizes.contains(&0) && sizes.contains(&1) && sizes.contains(&7),
            "flock sizes {sizes:?}"
        );
        Ok(())
    }

    /// M2 control 4 on S2: right-head token-value and per-level coupling
    /// gradients of the tree read against device central secants (same
    /// convention: pass iff |s - a| <= 2e-3 * max(|s|, |a|, 1e-3)).
    #[test]
    #[cfg_attr(not(feature = "cuda"), ignore = "GPU control: runs on the CUDA pod")]
    fn murm_tree_right_backward_secants() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let cells = near_cells(28)?;
        let (address, value, key_cell, reset_t, reset, norm) =
            paired_tokens(&device, &cells, &[0])?;
        let plan = TreePlan::new(&reset, 1, 28, 3)?;
        let weights = Tensor::from_vec(
            (0..2 * 28 * 4)
                .map(|i| {
                    if i >= 28 * 4 {
                        ((i * 37 % 17) as f32 / 17.0) - 0.5
                    } else {
                        0.0
                    }
                })
                .collect::<Vec<_>>(),
            (2, 28, 4),
            &device,
        )?;
        let v = Var::from_tensor(&value)?;
        let k = Var::from_tensor(&Tensor::new(
            &[[0.0f32, 0.9], [0.0, -0.7], [0.0, 0.5]],
            &device,
        )?)?;
        let loss = |v: &Tensor, k: &Tensor| -> Result<Tensor> {
            Ok(tree_read(
                &plan,
                &address,
                v,
                &key_cell,
                &norm,
                &reset_t,
                k,
                Chirality::Paired,
            )?
            .delta)
        };
        let grads = loss(v.as_tensor(), k.as_tensor())?
            .mul(&weights)?
            .sum_all()?
            .backward()?;
        let gv = grads
            .get(v.as_tensor())
            .ok_or_else(|| invalid("no value grad"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let gk = grads
            .get(k.as_tensor())
            .ok_or_else(|| invalid("no coupling grad"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let h = 1e-3f64;
        for idx in [28 * 4 + 1, 28 * 4 + 30, 28 * 4 + 61, 28 * 4 + 99] {
            let bump = |sign: f64| -> Result<Tensor> {
                let e = Tensor::from_vec(
                    (0..2 * 28 * 4)
                        .map(|i| if i == idx { (sign * h) as f32 } else { 0.0 })
                        .collect::<Vec<_>>(),
                    (2, 28, 4),
                    &device,
                )?;
                loss(&value.add(&e)?, k.as_tensor())
            };
            check_secant(
                gv[idx],
                secant(&bump(1.0)?, &bump(-1.0)?, &weights, h)?,
                &format!("value[{idx}]"),
            );
        }
        for l in 0..3 {
            let at = l * 2 + 1; // the right head's coupling at level l
            let bump = |sign: f64| -> Result<Tensor> {
                let e = Tensor::from_vec(
                    (0..6)
                        .map(|i| if i == at { (sign * h) as f32 } else { 0.0 })
                        .collect::<Vec<_>>(),
                    (3, 2),
                    &device,
                )?;
                loss(&value, &k.as_tensor().add(&e)?)
            };
            check_secant(
                gk[at],
                secant(&bump(1.0)?, &bump(-1.0)?, &weights, h)?,
                &format!("coupling[level {l}, right]"),
            );
        }
        Ok(())
    }
}
