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
//! over arrived nodes (kernel `murm_tree_select`, same score and tie law, no
//! exploration), transported into the reader's frame exactly as at level 0,
//! and `Delta_tree = sum_l K_{h,l} (m_{t,l} - v_t)` over non-empty levels, one
//! coupling per head and level (initialised at 0 by the caller). Levels add
//! as different scales' summaries; no deduplication against lower levels.
//! The total read is S1's `Delta` plus this. Model arithmetic is CUDA-only.

use candle_core::{CpuStorage, CustomOp1, CustomOp3, DType, Layout, Shape, Tensor};

use super::murmuration::{hamilton, relative_table, MAX_FLOCK, NULL_ADDRESS_NORM};
use super::{geometry, require_cuda, NativeStarling, ABSENT, CELLS};
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
        })
    }

    /// Level-`l` (1-based) nodes arrived at or before `t` in `b`'s current
    /// document: the count a reader at `t` can draw from (test metadata).
    pub fn available(&self, b: usize, t: usize, level: usize) -> usize {
        let Some(plan) = level.checked_sub(1).and_then(|i| self.levels.get(i)) else {
            return 0;
        };
        (0..=t.min(self.time.saturating_sub(1)))
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
    require_cuda(inputs.value, "the murmuration tree")?;
    let (rows, time, _) = inputs.value.dims3()?;
    let heads = inputs.heads;
    if heads == 0 || rows != plan.batch * heads || time != plan.time {
        return Err(invalid("the tree inputs disagree with the plan"));
    }
    let device = inputs.value.device();
    let table = Tensor::from_slice(relative_table()?, CELLS * CELLS, device)?;
    let roots = Tensor::from_vec(geometry()?.0, (CELLS, 4), device)?;
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
        // v_P = (1/3) sum_k roots[rel[c_P, c_k]] (x) v_k, signed child cells.
        let child_cell = cell
            .flatten_all()?
            .index_select(&index, 0)?
            .reshape((rows, n, 3))?;
        let rel = parent_cell
            .unsqueeze(2)?
            .broadcast_as((rows, n, 3))?
            .contiguous()?
            .mul(&Tensor::full(CELLS as u32, (rows, n, 3), device)?)?
            .add(&child_cell)?
            .flatten_all()?;
        let transport = roots
            .index_select(&table.index_select(&rel, 0)?, 0)?
            .reshape((rows, n, 3, 4))?;
        let child_value = value
            .contiguous()?
            .reshape((rows * prev, 4))?
            .index_select(&index, 0)?
            .reshape((rows, n, 3, 4))?;
        let parent_value = hamilton(&transport, &child_value)?
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
    if levels == 0 {
        return Ok(TreeOutput {
            delta,
            flock: flocks,
        });
    }
    let built = build_levels(plan, inputs)?;
    let table = Tensor::from_slice(relative_table()?, CELLS * CELLS, device)?;
    let roots = Tensor::from_vec(geometry()?.0, (CELLS, 4), device)?;
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
        let valid: Vec<u32> = level
            .arrival
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
        let selection = tree_select(
            spec,
            heads,
            &inputs.address.to_dtype(DType::F32)?.detach(),
            &node_address,
            &node_cell,
            &Tensor::from_vec(valid, (plan.batch, time), device)?,
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
        let rel = home
            .unsqueeze(2)?
            .broadcast_as((rows, time, k))?
            .contiguous()?
            .mul(&Tensor::full(CELLS as u32, (rows, time, k), device)?)?
            .add(&neighbour_cell.reshape((rows, time, k))?)?
            .flatten_all()?;
        let transport = roots
            .index_select(&table.index_select(&rel, 0)?, 0)?
            .reshape((rows, time, k, 4))?;
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
        let mean = hamilton(&transport, &neighbour_value)?
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
    }
    Ok(TreeOutput {
        delta,
        flock: flocks,
    })
}

/// The shared cell function on the device (kernel `murm_nearest_cells`):
/// nearest icosian of each unit address `[n, 4]`, `[n]` U32.
pub fn nearest_cells(address: &Tensor) -> Result<Tensor> {
    require_cuda(address, "the murmuration cell function")?;
    let (_, four) = address.dims2()?;
    if four != 4 || address.dtype() != DType::F32 {
        return Err(invalid("nearest cells need [n, 4] F32 addresses"));
    }
    Ok(address
        .detach()
        .contiguous()?
        .apply_op1_no_bwd(&NearestCells)?)
}

struct NearestCells;

impl CustomOp1 for NearestCells {
    fn name(&self) -> &'static str {
        "murmuration-nearest-cells"
    }
    fn cpu_fwd(&self, _: &CpuStorage, _: &Layout) -> candle_core::Result<(CpuStorage, Shape)> {
        candle_core::bail!("the murmuration cell function is GPU-only; no host fallback (D24)")
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        address: &candle_core::CudaStorage,
        layout: &Layout,
    ) -> candle_core::Result<(candle_core::CudaStorage, Shape)> {
        cuda::nearest_cells(address, layout)
    }
}

/// Tree-level selection (kernel `murm_tree_select`): `[rows, time, K + 2]`.
fn tree_select(
    spec: &NativeStarling,
    heads: usize,
    query: &Tensor,
    node_address: &Tensor,
    node_cell: &Tensor,
    valid: &Tensor,
    reset: &Tensor,
) -> Result<Tensor> {
    let (rows, time, _) = query.dims3()?;
    let batch = rows / heads;
    if node_address.dims() != [rows, time, 4]
        || node_cell.dims() != [rows, time]
        || valid.dims() != [batch, time]
        || reset.dims() != [batch, time]
    {
        return Err(invalid("tree selection tensors disagree"));
    }
    // One U32 operand: node cells, then the arrival mask, then the reset mask.
    let meta = Tensor::cat(
        &[
            &node_cell.flatten_all()?,
            &valid.flatten_all()?,
            &reset.to_dtype(DType::U32)?.flatten_all()?,
        ],
        0,
    )?;
    Ok(query.detach().contiguous()?.apply_op3_no_bwd(
        &node_address.detach().contiguous()?,
        &meta.contiguous()?,
        &TreeSelect {
            spec: *spec,
            heads,
            batch,
        },
    )?)
}

struct TreeSelect {
    #[cfg_attr(not(feature = "cuda"), allow(dead_code))]
    spec: NativeStarling,
    #[cfg_attr(not(feature = "cuda"), allow(dead_code))]
    heads: usize,
    #[cfg_attr(not(feature = "cuda"), allow(dead_code))]
    batch: usize,
}

impl CustomOp3 for TreeSelect {
    fn name(&self) -> &'static str {
        "murmuration-tree-select"
    }
    fn cpu_fwd(
        &self,
        _: &CpuStorage,
        _: &Layout,
        _: &CpuStorage,
        _: &Layout,
        _: &CpuStorage,
        _: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        candle_core::bail!("tree selection is GPU-only; no host fallback (D24)")
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        query: &candle_core::CudaStorage,
        ql: &Layout,
        node: &candle_core::CudaStorage,
        nl: &Layout,
        meta: &candle_core::CudaStorage,
        ml: &Layout,
    ) -> candle_core::Result<(candle_core::CudaStorage, Shape)> {
        cuda::tree_select(self, query, ql, node, nl, meta, ml)
    }
}

#[cfg(feature = "cuda")]
mod cuda {
    use candle_core::{CudaStorage, DType, Layout, Result as CResult, Shape};
    use cudarc::driver::CudaView;

    use super::TreeSelect;
    use crate::cuda_stack_kernels::cuda::{launch_prebuilt, zeros, Arg};

    const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/starling_native.ptx"));
    const MODULE: &str = "uor_r4_starling_native";

    fn view_f32<'a>(s: &'a CudaStorage, l: &Layout) -> CResult<CudaView<'a, f32>> {
        if !l.is_contiguous() || s.dtype() != DType::F32 {
            candle_core::bail!("expected a contiguous F32 CUDA tensor");
        }
        let start = l.start_offset();
        match s
            .as_cuda_slice::<f32>()?
            .try_slice(start..start + l.shape().elem_count())
        {
            Some(v) => Ok(v),
            None => candle_core::bail!("CUDA input layout exceeds its buffer"),
        }
    }

    fn view_u32<'a>(
        s: &'a CudaStorage,
        l: &Layout,
        from: usize,
        len: usize,
    ) -> CResult<CudaView<'a, u32>> {
        if !l.is_contiguous() || s.dtype() != DType::U32 {
            candle_core::bail!("expected a contiguous U32 CUDA tensor");
        }
        let start = l.start_offset() + from;
        match s.as_cuda_slice::<u32>()?.try_slice(start..start + len) {
            Some(v) => Ok(v),
            None => candle_core::bail!("CUDA input layout exceeds its buffer"),
        }
    }

    fn narrow(v: usize) -> CResult<u32> {
        u32::try_from(v)
            .map_err(|_| candle_core::Error::Msg("murmuration dimension overflow".into()))
    }

    fn check(
        device: &candle_core::CudaDevice,
        error: &cudarc::driver::CudaSlice<u32>,
        what: &str,
    ) -> CResult<()> {
        let flag = device.clone_dtoh(error)?;
        if flag.first().copied() != Some(0) {
            candle_core::bail!("{what} rejected its input: {:?}", flag);
        }
        Ok(())
    }

    pub(super) fn nearest_cells(
        address: &CudaStorage,
        layout: &Layout,
    ) -> CResult<(CudaStorage, Shape)> {
        let device = &address.device;
        let n = match layout.shape().dims() {
            [n, 4] => *n,
            _ => candle_core::bail!("nearest cells need [n, 4] addresses"),
        };
        let (roots, _) = crate::starling_native::geometry()
            .map_err(|e| candle_core::Error::Msg(e.to_string()))?;
        let roots = device.clone_htod(&roots)?;
        let out = zeros::<u32>(device, n)?;
        let error = zeros::<u32>(device, 1)?;
        let args = [
            Arg::F(view_f32(address, layout)?),
            Arg::f(&roots),
            Arg::u(&out),
            Arg::u(&error),
            Arg::U32(narrow(n)?),
        ];
        launch_prebuilt(device, MODULE, PTX, "murm_nearest_cells", n, &args)?;
        drop(args);
        check(device, &error, "the murmuration cell function")?;
        Ok((
            CudaStorage::wrap_cuda_slice(out, device.clone()),
            Shape::from_dims(&[n]),
        ))
    }

    pub(super) fn tree_select(
        op: &TreeSelect,
        query: &CudaStorage,
        ql: &Layout,
        node: &CudaStorage,
        nl: &Layout,
        meta: &CudaStorage,
        ml: &Layout,
    ) -> CResult<(CudaStorage, Shape)> {
        let device = &query.device;
        let (rows, time) = match ql.shape().dims() {
            [rows, time, 4] => (*rows, *time),
            _ => candle_core::bail!("tree selection queries must be [rows, time, 4]"),
        };
        let batch = op.batch;
        if nl.shape().dims() != ql.shape().dims()
            || ml.shape().elem_count() != rows * time + 2 * batch * time
        {
            candle_core::bail!("tree selection operands disagree");
        }
        let (roots, adjacency) = crate::starling_native::geometry()
            .map_err(|e| candle_core::Error::Msg(e.to_string()))?;
        let roots = device.clone_htod(&roots)?;
        let adjacency = device.clone_htod(&adjacency)?;
        let cells = crate::starling_native::CELLS;
        let slots = op.spec.kept;
        let ring = zeros::<u32>(device, rows * cells * op.spec.depth)?;
        let ring_len = zeros::<u32>(device, rows * cells)?;
        let ring_head = zeros::<u32>(device, rows * cells)?;
        let out = zeros::<u32>(device, rows * time * (slots + 2))?;
        let error = zeros::<u32>(device, 1)?;
        let args = [
            Arg::F(view_f32(query, ql)?),
            Arg::F(view_f32(node, nl)?),
            Arg::U(view_u32(meta, ml, 0, rows * time)?),
            Arg::U(view_u32(meta, ml, rows * time, batch * time)?),
            Arg::U(view_u32(
                meta,
                ml,
                rows * time + batch * time,
                batch * time,
            )?),
            Arg::f(&roots),
            Arg::u(&adjacency),
            Arg::u(&ring),
            Arg::u(&ring_len),
            Arg::u(&ring_head),
            Arg::u(&out),
            Arg::u(&error),
            Arg::U32(narrow(rows)?),
            Arg::U32(narrow(time)?),
            Arg::U32(narrow(op.heads)?),
            Arg::U32(narrow(op.spec.depth)?),
            Arg::U32(narrow(slots)?),
        ];
        launch_prebuilt(device, MODULE, PTX, "murm_tree_select", rows, &args)?;
        drop(args);
        check(device, &error, "tree selection")?;
        Ok((
            CudaStorage::wrap_cuda_slice(out, device.clone()),
            Shape::from_dims(&[rows, time, slots + 2]),
        ))
    }
}

#[cfg(test)]
#[allow(clippy::needless_range_loop)]
mod tests {
    use super::*;
    use crate::starling_native::murmuration::{
        murmuration_read, selection_spec, MurmurationInputs,
    };
    use crate::starling_native::{select, Credit};
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
        Ok(())
    }

    #[test]
    fn murm_tree_refuses_cpu() -> Result<()> {
        let device = Device::Cpu;
        let plan = TreePlan::new(&[1, 0, 0, 0], 1, 4, 1)?;
        let x = Tensor::zeros((1, 4, 4), DType::F32, &device)?;
        let norm = Tensor::ones((1, 4), DType::F32, &device)?;
        let cells = Tensor::zeros((1, 4), DType::U32, &device)?;
        let coupling = Tensor::zeros((1, 1), DType::F32, &device)?;
        let reset = Tensor::zeros((1, 4), DType::U32, &device)?;
        let inputs = TreeInputs {
            address: &x,
            address_norm: &norm,
            value: &x,
            cells: &cells,
            coupling: &coupling,
            reset: &reset,
            heads: 1,
        };
        assert!(murmuration_tree_read(&spec(), &plan, &inputs).is_err());
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
        let selection = select(&s, 1, &address, &address, &reset_t)?;
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
        let c_p = built[0].cell.flatten_all()?.to_vec1::<u32>()?[0] as usize;
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
                },
            )?
            .delta
            .mul(&weights)?
            .sum_all()?)
        };
        let grads = loss(v.as_tensor(), k.as_tensor())?.backward()?;
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
        let check = |a: f32, p: f32, m: f32, what: &str| {
            let s = (p - m) as f64 / (2.0 * h);
            let bound = 2e-3 * s.abs().max(a.abs() as f64).max(1e-3);
            assert!(
                (s - a as f64).abs() <= bound,
                "{what}: analytic {a} secant {s}"
            );
        };
        for idx in [0usize, 9, 30, 47, 66, 90, 107] {
            let bump = |sign: f64| -> Result<f32> {
                let e = Tensor::from_vec(
                    (0..28 * 4)
                        .map(|i| if i == idx { (sign * h) as f32 } else { 0.0 })
                        .collect::<Vec<_>>(),
                    (1, 28, 4),
                    &device,
                )?;
                Ok(loss(&value.add(&e)?, k.as_tensor())?.to_scalar::<f32>()?)
            };
            check(gv[idx], bump(1.0)?, bump(-1.0)?, &format!("value[{idx}]"));
        }
        for l in 0..3 {
            let bump = |sign: f64| -> Result<f32> {
                let e = Tensor::from_vec(
                    (0..3)
                        .map(|i| if i == l { (sign * h) as f32 } else { 0.0 })
                        .collect::<Vec<_>>(),
                    (3, 1),
                    &device,
                )?;
                Ok(loss(&value, &k.as_tensor().add(&e)?)?.to_scalar::<f32>()?)
            };
            check(gk[l], bump(1.0)?, bump(-1.0)?, &format!("coupling[{l}]"));
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
            },
        );
        assert!(tree.is_err(), "a null-containing parent was not refused");
        let s = selection_spec(&spec())?;
        let selection = select(&s, 1, &address, &address, &reset_t)?;
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
            },
            false,
            None,
        )
        .is_ok());
        Ok(())
    }
}
