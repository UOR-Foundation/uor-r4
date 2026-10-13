use super::{GgmlDType, QStorage};
use crate::quantized::k_quants::GgmlType;
use crate::{backend::BackendDevice, cuda_backend::WrapErr};
use crate::{builder_arg as barg, CudaDevice, CudaStorage, Result};
use half::f16;

use cudarc::driver::{CudaSlice, CudaView, PushKernelArg};

#[derive(Clone, Debug)]
struct PaddedCudaSlice {
    inner: CudaSlice<u8>,
    len: usize,
}

#[derive(Clone, Debug)]
pub struct QCudaStorage {
    data: PaddedCudaSlice,
    dtype: GgmlDType,
    device: CudaDevice,
}

static FORCE_DMMV: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_force_dmmv(f: bool) {
    FORCE_DMMV.store(f, std::sync::atomic::Ordering::Relaxed)
}

pub const WARP_SIZE: usize = 32;
pub const MMQ_X_Q4_0_AMPERE: usize = 4;
pub const MMQ_Y_Q4_0_AMPERE: usize = 32;
pub const NWARPS_Q4_0_AMPERE: usize = 4;
pub const GGML_CUDA_MMV_X: usize = 32;
pub const GGML_CUDA_MMV_Y: usize = 1;
pub const CUDA_QUANTIZE_BLOCK_SIZE: usize = 256;
pub const CUDA_DEQUANTIZE_BLOCK_SIZE: usize = 256;
pub const MATRIX_ROW_PADDING: usize = 512;

fn ceil_div(p: usize, q: usize) -> usize {
    p.div_ceil(q)
}

fn pad(p: usize, q: usize) -> usize {
    ceil_div(p, q) * q
}

fn quantize_q8_1(
    src: &CudaView<f32>,
    dst: &mut CudaSlice<u8>,
    k: usize,
    ky: usize,
    dev: &CudaDevice,
) -> Result<()> {
    let kx_padded = pad(k, MATRIX_ROW_PADDING);
    let num_blocks = ceil_div(kx_padded, CUDA_QUANTIZE_BLOCK_SIZE);

    let total_rows = ky;
    // Get Q8_1 metadata.
    let q8_1_block_size = GgmlDType::Q8_1.block_size();
    let q8_1_type_size = GgmlDType::Q8_1.type_size();

    // Calculate the size of the output buffer in bytes.
    let num_blocks_per_row = kx_padded / q8_1_block_size;
    let dst_row_size_bytes = num_blocks_per_row * q8_1_type_size;

    const CHUNK_SIZE: usize = 65535; // gridDim.y limit
    let func = dev.get_or_load_func("quantize_q8_1", &candle_kernels::QUANTIZED)?;

    let mut rows_processed = 0;
    while rows_processed < total_rows {
        // --- calculate the number of rows for this chunk ---
        let remaining_rows = total_rows - rows_processed;
        // This is our gridDim.y, now <= 65535
        let rows_in_chunk = std::cmp::min(CHUNK_SIZE, remaining_rows);

        // --- slice the source (f32) tensor by elements ---
        let src_start_elem = rows_processed * k;
        let src_num_elems = rows_in_chunk * k;
        let src_chunk = src.slice(src_start_elem..(src_start_elem + src_num_elems));

        // --- slice the destination (u8) tensor by bytes ---
        let dst_start_byte = rows_processed * dst_row_size_bytes;
        let dst_num_bytes = rows_in_chunk * dst_row_size_bytes;
        let dst_chunk = dst.slice(dst_start_byte..(dst_start_byte + dst_num_bytes));

        let cfg = cudarc::driver::LaunchConfig {
            grid_dim: (num_blocks as u32, rows_in_chunk as u32, 1),
            block_dim: (CUDA_QUANTIZE_BLOCK_SIZE as u32, 1, 1),
            shared_mem_bytes: 0,
        };

        let mut builder = func.builder();
        builder.arg(&src_chunk);
        builder.arg(&dst_chunk);
        barg!(builder, k as i32, kx_padded as i32);
        unsafe { builder.launch(cfg) }.w()?;

        rows_processed += rows_in_chunk;
    }

    Ok(())
}

fn dequantize_f32(
    data: &PaddedCudaSlice,
    dtype: GgmlDType,
    elem_count: usize,
    dev: &CudaDevice,
) -> Result<CudaStorage> {
    let nb = elem_count.div_ceil(256);
    let (kernel_name, is_k, block_dim, num_blocks) = match dtype {
        GgmlDType::Q4_0 => ("dequantize_block_q4_0_f32", false, 32, nb),
        GgmlDType::Q4_1 => ("dequantize_block_q4_1_f32", false, 32, nb),
        GgmlDType::Q5_0 => (
            "dequantize_block_q5_0_f32",
            false,
            CUDA_DEQUANTIZE_BLOCK_SIZE,
            ceil_div(elem_count, 2 * CUDA_DEQUANTIZE_BLOCK_SIZE),
        ),
        GgmlDType::Q5_1 => (
            "dequantize_block_q5_1_f32",
            false,
            CUDA_DEQUANTIZE_BLOCK_SIZE,
            ceil_div(elem_count, 2 * CUDA_DEQUANTIZE_BLOCK_SIZE),
        ),
        GgmlDType::Q8_0 => ("dequantize_block_q8_0_f32", false, 32, nb),
        GgmlDType::Q2K => ("dequantize_block_q2_K_f32", true, 64, nb),
        GgmlDType::Q3K => ("dequantize_block_q3_K_f32", true, 64, nb),
        GgmlDType::Q4K => ("dequantize_block_q4_K_f32", true, 32, nb),
        GgmlDType::Q5K => ("dequantize_block_q5_K_f32", true, 64, nb),
        GgmlDType::Q6K => ("dequantize_block_q6_K_f32", true, 64, nb),
        GgmlDType::Q8K => ("dequantize_block_q8_K_f32", true, 32, nb),
        _ => crate::bail!("unsupported dtype for dequantize {dtype:?}"),
    };
    let func = dev.get_or_load_func(kernel_name, &candle_kernels::QUANTIZED)?;
    let dst = unsafe { dev.alloc::<f32>(elem_count)? };
    // See e.g.
    // https://github.com/ggerganov/llama.cpp/blob/cbbd1efa06f8c09f9dff58ff9d9af509cc4c152b/ggml-cuda.cu#L7270
    let cfg = cudarc::driver::LaunchConfig {
        grid_dim: (num_blocks as u32, 1, 1),
        block_dim: (block_dim as u32, 1, 1),
        shared_mem_bytes: 0,
    };

    if is_k {
        let mut builder = func.builder();
        builder.arg(&data.inner);
        builder.arg(&dst);
        unsafe { builder.launch(cfg) }.w()?;
    } else {
        let nb32 = match dtype {
            GgmlDType::Q5_0 | GgmlDType::Q5_1 => elem_count,
            _ => elem_count / 32,
        };
        let mut builder = func.builder();
        builder.arg(&data.inner);
        builder.arg(&dst);
        barg!(builder, nb32 as i32);
        unsafe { builder.launch(cfg) }.w()?;
    }
    Ok(CudaStorage::wrap_cuda_slice(dst, dev.clone()))
}

fn dequantize_f16(
    data: &PaddedCudaSlice,
    dtype: GgmlDType,
    elem_count: usize,
    dev: &CudaDevice,
) -> Result<CudaStorage> {
    let nb = elem_count.div_ceil(256);
    let (kernel_name, is_k, block_dim, num_blocks) = match dtype {
        GgmlDType::Q4_0 => ("dequantize_block_q4_0_f16", false, 32, nb),
        GgmlDType::Q4_1 => ("dequantize_block_q4_1_f16", false, 32, nb),
        GgmlDType::Q5_0 => (
            "dequantize_block_q5_0_f16",
            false,
            CUDA_DEQUANTIZE_BLOCK_SIZE,
            ceil_div(elem_count, 2 * CUDA_DEQUANTIZE_BLOCK_SIZE),
        ),
        GgmlDType::Q5_1 => (
            "dequantize_block_q5_1_f16",
            false,
            CUDA_DEQUANTIZE_BLOCK_SIZE,
            ceil_div(elem_count, 2 * CUDA_DEQUANTIZE_BLOCK_SIZE),
        ),
        GgmlDType::Q8_0 => ("dequantize_block_q8_0_f16", false, 32, nb),
        GgmlDType::Q2K => ("dequantize_block_q2_K_f16", true, 64, nb),
        GgmlDType::Q3K => ("dequantize_block_q3_K_f16", true, 64, nb),
        GgmlDType::Q4K => ("dequantize_block_q4_K_f16", true, 32, nb),
        GgmlDType::Q5K => ("dequantize_block_q5_K_f16", true, 64, nb),
        GgmlDType::Q6K => ("dequantize_block_q6_K_f16", true, 64, nb),
        GgmlDType::Q8K => ("dequantize_block_q8_K_f16", true, 32, nb),
        _ => crate::bail!("unsupported dtype for dequantize {dtype:?}"),
    };
    let func = dev.get_or_load_func(kernel_name, &candle_kernels::QUANTIZED)?;
    let dst = unsafe { dev.alloc::<f16>(elem_count)? };
    // See e.g.
    // https://github.com/ggerganov/llama.cpp/blob/cbbd1efa06f8c09f9dff58ff9d9af509cc4c152b/ggml-cuda.cu#L7270
    let cfg = cudarc::driver::LaunchConfig {
        grid_dim: (num_blocks as u32, 1, 1),
        block_dim: (block_dim as u32, 1, 1),
        shared_mem_bytes: 0,
    };

    if is_k {
        let mut builder = func.builder();
        builder.arg(&data.inner);
        builder.arg(&dst);
        unsafe { builder.launch(cfg) }.w()?;
    } else {
        let nb32 = match dtype {
            GgmlDType::Q5_0 | GgmlDType::Q5_1 => elem_count,
            _ => elem_count / 32,
        };
        let mut builder = func.builder();
        builder.arg(&data.inner);
        builder.arg(&dst);
        barg!(builder, nb32 as i32);
        unsafe { builder.launch(cfg) }.w()?;
    }
    Ok(CudaStorage::wrap_cuda_slice(dst, dev.clone()))
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]

impl QCudaStorage {
    pub fn indexed_moe_forward(
        &self,
        self_shape: &crate::Shape, //[num_experts, n, k]
        input: &CudaStorage,       //[batch, topk or 1, k]
        input_l: &crate::Layout,
        ids: &CudaStorage, //[batch, topk]
        ids_l: &crate::Layout,
    ) -> Result<(CudaStorage, crate::Shape)> {
        let _ = (self_shape, input, input_l, ids, ids_l);
        crate::bail!(
            "D29: quantized CUDA MoE/dense contraction is forbidden; no vendor or host fallback"
        )
    }

    pub fn zeros(device: &CudaDevice, el_count: usize, dtype: GgmlDType) -> Result<Self> {
        let size_in_bytes = ceil_div(el_count, dtype.block_size()) * dtype.type_size();
        let padded_size_in_bytes =
            ceil_div(el_count + MATRIX_ROW_PADDING, dtype.block_size()) * dtype.type_size();
        let inner = device.alloc_zeros::<u8>(padded_size_in_bytes)?;
        Ok(QCudaStorage {
            data: PaddedCudaSlice {
                inner,
                len: size_in_bytes,
            },
            device: device.clone(),
            dtype,
        })
    }

    pub fn dtype(&self) -> GgmlDType {
        self.dtype
    }

    pub fn device(&self) -> &CudaDevice {
        &self.device
    }

    pub fn dequantize(&self, elem_count: usize) -> Result<CudaStorage> {
        let _ = elem_count;
        crate::bail!("D29: CUDA quantized decode is unavailable; CPU model fallback is forbidden")
    }

    pub fn dequantize_f16(&self, elem_count: usize) -> Result<CudaStorage> {
        dequantize_f16(&self.data, self.dtype, elem_count, self.device())
    }

    pub fn quantize(&mut self, src: &CudaStorage) -> Result<()> {
        let _ = src;
        crate::bail!("D29: CUDA quantization is unavailable; CPU model fallback is forbidden")
    }

    pub fn quantize_imatrix(
        &mut self,
        src: &CudaStorage,
        imatrix_weights: &[f32],
        n_per_row: usize,
    ) -> Result<()> {
        let _ = (src, imatrix_weights, n_per_row);
        crate::bail!(
            "D29: CUDA quantization calibration is unavailable; CPU model fallback is forbidden"
        )
    }

    pub fn quantize_imatrix_onto(
        &mut self,
        src: &crate::CpuStorage,
        imatrix_weights: &[f32],
        n_per_row: usize,
    ) -> Result<()> {
        let _ = (src, imatrix_weights, n_per_row);
        crate::bail!(
            "D29: CUDA quantization calibration is unavailable; CPU model fallback is forbidden"
        )
    }

    pub fn quantize_onto(&mut self, src: &crate::CpuStorage) -> Result<()> {
        let _ = src;
        crate::bail!("D29: CUDA quantization is unavailable; CPU model fallback is forbidden")
    }

    pub fn storage_size_in_bytes(&self) -> usize {
        self.data.len
    }

    pub fn fwd(
        &self,
        self_shape: &crate::Shape,
        storage: &CudaStorage,
        layout: &crate::Layout,
    ) -> Result<(CudaStorage, crate::Shape)> {
        let _ = (self_shape, storage, layout);
        crate::bail!(
            "D29: quantized CUDA matrix multiplication is forbidden; no vendor or host fallback"
        )
    }

    pub fn data(&self) -> Result<Vec<u8>> {
        let mut out = vec![0u8; self.data.len];
        self.device
            .memcpy_dtoh(&self.data.inner.slice(..self.data.len), &mut out)?;
        Ok(out)
    }

    pub fn device_ptr(&self) -> Result<*const u8> {
        use cudarc::driver::DevicePtr;
        Ok(self.data.inner.device_ptr(self.data.inner.stream()).0 as *const u8)
    }
}

impl QCudaStorage {}

pub fn load_quantized<T: super::GgmlType + Send + Sync + 'static>(
    device: &CudaDevice,
    data: &[T],
) -> Result<super::QStorage> {
    let data = unsafe {
        std::slice::from_raw_parts(data.as_ptr() as *const u8, core::mem::size_of_val(data))
    };
    let dtype = T::DTYPE;
    let padded_len = data.len() + MATRIX_ROW_PADDING * dtype.type_size() / dtype.block_size();
    let mut inner = unsafe { device.alloc::<u8>(padded_len)? };
    device.memcpy_htod(data, &mut inner.slice_mut(..data.len()))?;
    Ok(QStorage::Cuda(QCudaStorage {
        data: PaddedCudaSlice {
            inner,
            len: data.len(),
        },
        device: device.clone(),
        dtype,
    }))
}

// Historical dense/quantized CUDA controls are retained as inactive source.
// Their contraction kernels are excluded by D29 and are never test-admitted.
#[cfg(any())]
mod test {
    use super::*;

    #[test]
    fn cuda_quantize_q8_1() -> Result<()> {
        let dev = CudaDevice::new(0)?;
        let el = 256;
        let el_padded = pad(el, MATRIX_ROW_PADDING);
        let y_size_in_bytes =
            el_padded * GgmlDType::Q8_1.type_size() / GgmlDType::Q8_1.block_size();
        let mut y_q8_1 = unsafe { dev.alloc::<u8>(y_size_in_bytes)? };
        let vs: Vec<f32> = (0..el).map(|v| v as f32).collect();
        let y = dev.clone_htod(&vs)?;
        quantize_q8_1(&y.as_view(), &mut y_q8_1, el, 1, &dev)?;
        Ok(())
    }

    #[test]
    fn cuda_mmv_q8_1() -> Result<()> {
        let dev = CudaDevice::new(0)?;
        let ncols = 256;
        let vs: Vec<f32> = (0..ncols).map(|v| v as f32).collect();
        let y = dev.clone_htod(&vs)?;
        let mut xs = QCudaStorage::zeros(&dev, ncols, GgmlDType::Q4_0)?;
        xs.quantize(&CudaStorage::wrap_cuda_slice(y.clone(), dev.clone()))?;
        let cuda_storage = mul_mat_vec_via_q8_1(
            &xs.data,
            &y.as_view(),
            /* dtype */ GgmlDType::Q4_0,
            /* ncols */ ncols,
            /* nrows */ 1,
            /* b_size */ 1,
            &dev,
        )?;
        let vs = cuda_storage.as_cuda_slice::<f32>()?;
        let vs = dev.clone_dtoh(&vs.as_view())?;
        assert_eq!(vs.len(), 1);
        // for n = 255, n.(n+1).(2n+1) / 6 = 5559680
        // Q8 means 1/256 precision.
        assert_eq!(vs[0], 5561664.5);

        let cuda_storage = dequantize_mul_mat_vec(
            &xs.data,
            &y.as_view(),
            /* dtype */ GgmlDType::Q4_0,
            /* ncols */ ncols,
            /* nrows */ 1,
            &dev,
        )?;
        let vs = cuda_storage.as_cuda_slice::<f32>()?;
        let vs = dev.clone_dtoh(&vs.as_view())?;
        assert_eq!(vs.len(), 1);
        assert_eq!(vs[0], 5561851.0);
        Ok(())
    }

    #[test]
    fn cuda_mm_q8_1() -> Result<()> {
        let dev = CudaDevice::new(0)?;
        let ncols = 256;
        let vs: Vec<f32> = (0..ncols * 4).map(|v| v as f32 / 4.).collect();
        let y = dev.clone_htod(&vs)?;
        let mut xs = QCudaStorage::zeros(&dev, ncols * 4, GgmlDType::Q4_0)?;
        xs.quantize(&CudaStorage::wrap_cuda_slice(y.clone(), dev.clone()))?;
        let cuda_storage = mul_mat_via_q8_1(
            &xs.data,
            &y.as_view(),
            /* dtype */ GgmlDType::Q4_0,
            /* x_rows */ 4,
            /* x_cols */ ncols,
            /* y_rows */ ncols,
            /* y_cols */ 4,
            &dev,
        )?;
        let vs = cuda_storage.as_cuda_slice::<f32>()?;
        let vs = dev.clone_dtoh(&vs.as_view())?;

        /*
           x = torch.tensor([float(v) for v in range(1024)]).reshape(4, 256)
           x @ x.t() / 16
        tensor([[  347480.0000,   869720.0000,  1391960.0000,  1914200.0000],
                [  869720.0000,  2440536.0000,  4011352.0000,  5582166.5000],
                [ 1391960.0000,  4011352.0000,  6630742.0000,  9250132.0000],
                [ 1914200.0000,  5582166.5000,  9250132.0000, 12918099.0000]])
                */
        assert_eq!(vs.len(), 16);
        assert_eq!(vs[0], 347604.0);
        assert_eq!(vs[1], 888153.06);
        assert_eq!(vs[4], 869780.7);
        assert_eq!(vs[5], 2483145.0);
        assert_eq!(vs[11], 9407368.0);
        assert_eq!(vs[14], 9470856.0);
        assert_eq!(vs[15], 13138824.0);
        Ok(())
    }

    // The following test used to fail under compute-sanitizer until #2526.
    #[test]
    fn cuda_mm_q8_1_pad() -> Result<()> {
        let dev = CudaDevice::new(0)?;
        let (x_rows, ncols, y_cols) = (4, 16, 2048);
        let vs: Vec<f32> = (0..ncols * y_cols).map(|v| v as f32 / 256.).collect();
        let y = dev.clone_htod(&vs)?;
        let mut xs = QCudaStorage::zeros(&dev, ncols * x_rows, GgmlDType::Q4_0)?;
        xs.quantize(&CudaStorage::wrap_cuda_slice(y.clone(), dev.clone()))?;
        let cuda_storage = mul_mat_via_q8_1(
            &xs.data,
            &y.as_view(),
            /* dtype */ GgmlDType::Q4_0,
            /* x_rows */ x_rows,
            /* x_cols */ ncols,
            /* y_rows */ ncols,
            /* y_cols */ y_cols,
            &dev,
        )?;
        let vs = cuda_storage.as_cuda_slice::<f32>()?;
        let _vs = dev.clone_dtoh(&vs.as_view())?;
        Ok(())
    }
}
