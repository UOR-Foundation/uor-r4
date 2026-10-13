//! The murmuration read's ahead-of-time CUDA kernels (nvcc, `starling_native.cu`;
//! no NVRTC): the shared nearest-cell function and K1's ring selection over a
//! timeline of arriving keys with a per-row validity mask. Selection is hard
//! and takes no gradient; model arithmetic is CUDA-only (D24).

use candle_core::{CpuStorage, CustomOp1, CustomOp3, DType, Layout, Shape, Tensor};

use super::{require_cuda, NativeStarling};
use crate::{invalid, Result};

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

/// K1's ring selection over a timeline of arriving keys (kernel
/// `murm_ring_select`): `node_address`/`node_cell` hold the key arriving at each
/// position of each row, `valid` (`[rows, time]` U32) marks real arrivals, and
/// `reset` (`[batch, time]`) clears the rings at BOS. Returns
/// `[rows, time, spec.kept + 2]`: slot positions in rank order (ABSENT when
/// fewer), the home cell, and the arriving key's cell (ABSENT when none).
pub(crate) fn ring_select(
    spec: &NativeStarling,
    heads: usize,
    query: &Tensor,
    node_address: &Tensor,
    node_cell: &Tensor,
    valid: &Tensor,
    reset: &Tensor,
) -> Result<Tensor> {
    require_cuda(query, "ring selection")?;
    let (rows, time, _) = query.dims3()?;
    // Codex 6153323237: reject zero or indivisible shapes before any division
    // or launch (rows = 3, heads = 2 would otherwise read reset out of bounds).
    if heads == 0 || rows == 0 || rows % heads != 0 {
        return Err(invalid(format!(
            "ring selection needs rows > 0 divisible by heads > 0 (rows {rows}, heads {heads})"
        )));
    }
    let batch = rows / heads;
    if node_address.dims() != [rows, time, 4]
        || node_cell.dims() != [rows, time]
        || valid.dims() != [rows, time]
        || reset.dims() != [batch, time]
    {
        return Err(invalid("ring selection tensors disagree"));
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
        &RingSelect {
            spec: *spec,
            heads,
            batch,
        },
    )?)
}

struct RingSelect {
    #[cfg_attr(not(feature = "cuda"), allow(dead_code))]
    spec: NativeStarling,
    #[cfg_attr(not(feature = "cuda"), allow(dead_code))]
    heads: usize,
    #[cfg_attr(not(feature = "cuda"), allow(dead_code))]
    batch: usize,
}

impl CustomOp3 for RingSelect {
    fn name(&self) -> &'static str {
        "murmuration-ring-select"
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
        candle_core::bail!("ring selection is GPU-only; no host fallback (D24)")
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
        cuda::ring_select(self, query, ql, node, nl, meta, ml)
    }
}

#[cfg(feature = "cuda")]
mod cuda {
    use candle_core::{CudaStorage, DType, Layout, Result as CResult, Shape};
    use cudarc::driver::CudaView;

    use super::RingSelect;
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

    pub(super) fn ring_select(
        op: &RingSelect,
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
            _ => candle_core::bail!("ring selection queries must be [rows, time, 4]"),
        };
        let batch = op.batch;
        if nl.shape().dims() != ql.shape().dims()
            || ml.shape().elem_count() != 2 * rows * time + batch * time
        {
            candle_core::bail!("ring selection operands disagree");
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
            Arg::U(view_u32(meta, ml, rows * time, rows * time)?),
            Arg::U(view_u32(meta, ml, 2 * rows * time, batch * time)?),
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
        launch_prebuilt(device, MODULE, PTX, "murm_ring_select", rows, &args)?;
        drop(args);
        check(device, &error, "ring selection")?;
        Ok((
            CudaStorage::wrap_cuda_slice(out, device.clone()),
            Shape::from_dims(&[rows, time, slots + 2]),
        ))
    }
}
