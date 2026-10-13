//! Lawful support for the murmuration read on the native graph (M4 #2032):
//! the 600-cell codebook, the GPU-only and finite-input admissions, unit
//! addresses and document-start masks. Index metadata and validation only; no
//! model arithmetic runs on the host.

use candle_core::{DType, Tensor};
use std::sync::OnceLock;

use super::CELLS;
use crate::{invalid, Result};

/// The fixed icosian codebook `[120 * 4]` (the order of `canonical_h4_roots`)
/// and each vertex's 12 nearest neighbours `[120 * 12]`, ascending. Neighbours
/// are found by the **exact** `Z[phi]` dot product `phi/2` of the doubled roots,
/// not by a float threshold.
/// Roots `[120 * 4]` and adjacency `[120 * 12]`.
type Codebook = (Vec<f32>, Vec<u32>);

pub(crate) fn geometry() -> Result<Codebook> {
    static GEOMETRY: OnceLock<std::result::Result<Codebook, String>> = OnceLock::new();
    GEOMETRY
        .get_or_init(|| build_geometry().map_err(|e| e.to_string()))
        .clone()
        .map_err(invalid)
}

fn build_geometry() -> Result<(Vec<f32>, Vec<u32>)> {
    let roots: Vec<f32> = uor_r4_core::native_geometric::learner::embedding::canonical_h4_roots()
        .iter()
        .flat_map(|root| root.to_array().map(|x| x as f32))
        .collect();
    let adjacency = super::murmuration::exact_adjacency()?;
    if roots.len() != CELLS * 4 || adjacency.len() != CELLS * 12 {
        return Err(invalid(
            "the 600-cell codebook must have 120 vertices of 12 neighbours",
        ));
    }
    Ok((roots, adjacency))
}

/// Refuse anything but a CUDA tensor: model arithmetic is GPU-only (D24).
pub(crate) fn require_cuda(tensor: &Tensor, what: &str) -> Result<()> {
    if !tensor.device().is_cuda() {
        return Err(invalid(format!(
            "{what} is GPU-only model compute; no CPU or Metal execution (D24)"
        )));
    }
    Ok(())
}

/// Refuse a tensor holding any NaN or infinity, before it reaches arithmetic
/// (masking does not remove NaN: `0 * NaN = NaN`). The check runs on the
/// device; only one validation scalar returns to the host.
pub(crate) fn require_finite(tensor: &Tensor, what: &str) -> Result<()> {
    if tensor.elem_count() == 0 {
        return Ok(());
    }
    let ok = tensor
        .to_dtype(DType::F32)?
        .abs()?
        .le(f32::MAX as f64)? // false for NaN and for +-inf
        .to_dtype(DType::F32)?
        .flatten_all()?
        .min(0)?
        .to_scalar::<f32>()?;
    if ok != 1.0 {
        return Err(invalid(format!(
            "{what} contains a non-finite value; refused before arithmetic"
        )));
    }
    Ok(())
}

/// Unit 4-d addresses `raw / sqrt(|raw|^2 + 1e-12)` over the last axis.
pub fn unit_address(raw: &Tensor) -> Result<Tensor> {
    let raw = raw.to_dtype(DType::F32)?;
    let last = raw
        .rank()
        .checked_sub(1)
        .ok_or_else(|| invalid("address needs an axis"))?;
    let norm = raw.sqr()?.sum_keepdim(last)?.affine(1.0, 1e-12)?.sqrt()?;
    Ok(raw.broadcast_div(&norm)?.contiguous()?)
}

/// The document-start mask of `ids` (`[batch * time]`): 1 where the token is
/// the BOS identity. Index metadata only.
pub fn reset_mask(ids: &[u32], batch: usize, time: usize, bos_id: u32) -> Result<Vec<u32>> {
    if ids.len() != batch * time {
        return Err(invalid("a reset mask needs batch * time ids"));
    }
    Ok(ids.iter().map(|&id| u32::from(id == bos_id)).collect())
}
