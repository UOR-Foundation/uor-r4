//! DRAFT / NOT_RUN: rank-flock replacement of harmonic pair scores.
//!
//! No module registration, support selector, fitted model or runtime result.
//! DeepSeek supplies the causal support in descending Lorentz-rank order, with
//! lowest-position ties. This module assigns RAW weights 1/(rank+1); passing a
//! normalized sparse output would define a different hybrid. The quadratic
//! training path replaces scores before a SINGLE normalization. The detached
//! row helper corrects both a recurrent numerator and its mass, subtracting
//! selected harmonic contributions exactly once. Full key/value retention and
//! the selector's scan/sort remain additional costs, not O(k) claims.

use std::collections::BTreeSet;

use candle_core::{DType, Tensor};

use crate::{invalid, Result};

pub const SPARSE_WEIGHT_VERSION: &str = "raw-reciprocal-lorentz-rank-v1";

/// One row per [batch,head,query], flattened in that order. Positions must be
/// deduplicated and inclusive-causal. Order is supplied by the external selector
/// and is part of its bound identity; this helper cannot validate Lorentz ranks.
pub struct RankedSupport {
    batch: usize,
    heads: usize,
    time: usize,
    rows: Vec<Vec<usize>>,
}

impl RankedSupport {
    pub fn new(
        batch: usize,
        heads: usize,
        time: usize,
        rows: Vec<Vec<usize>>,
        max_selected_per_row: usize,
    ) -> Result<Self> {
        let expected = product(&[batch, heads, time])?;
        if batch == 0 || heads == 0 || time == 0 || rows.len() != expected {
            return Err(invalid("hybrid support shape mismatch"));
        }
        for (row, positions) in rows.iter().enumerate() {
            if positions.len() > max_selected_per_row {
                return Err(invalid("hybrid support allocation bound exceeded"));
            }
            let query = row % time;
            let mut seen = BTreeSet::new();
            for &position in positions {
                if position > query || !seen.insert(position) {
                    return Err(invalid("hybrid support is future-facing or duplicated"));
                }
            }
        }
        Ok(Self {
            batch,
            heads,
            time,
            rows,
        })
    }
}

fn product(factors: &[usize]) -> Result<usize> {
    factors.iter().try_fold(1usize, |n, &factor| {
        n.checked_mul(factor)
            .ok_or_else(|| invalid("hybrid shape overflow"))
    })
}

fn raw_rank_weight(rank: usize) -> Result<f32> {
    let denominator = rank
        .checked_add(1)
        .ok_or_else(|| invalid("rank overflow"))?;
    let result = (1.0 / denominator as f64) as f32;
    if !result.is_finite() || result <= 0.0 {
        return Err(invalid("raw rank weight is not positive finite F32"));
    }
    Ok(result)
}

pub struct HybridResult {
    pub attended: Tensor,
    pub minimum_mass: f32,
    pub replaced_pairs: usize,
}

/// F32 harmonic_scores[B,H,T,T], expanded values[B,H,T,D]. Differentiable in
/// unselected harmonic scores and all contributing values; discrete support and
/// fixed rank weights are detached. No straight-through selector is invented.
/// This is QUADRATIC in T and performs an explicit host finite/positivity check;
/// it is not the eventual fast recurrent runtime. Bounds apply before its new
/// dense masks, score validation copy and output validation copy are allocated.
pub fn quadratic_rank_hybrid(
    harmonic_scores: &Tensor,
    values: &Tensor,
    support: &RankedSupport,
    max_dense_entries: usize,
    max_value_entries: usize,
) -> Result<HybridResult> {
    let (batch, heads, time, keys) = harmonic_scores.dims4()?;
    let (vb, vh, vt, width) = values.dims4()?;
    if (batch, heads, time) != (support.batch, support.heads, support.time)
        || keys != time
        || (vb, vh, vt) != (batch, heads, time)
        || width == 0
        || harmonic_scores.dtype() != DType::F32
        || values.dtype() != DType::F32
    {
        return Err(invalid("hybrid tensor shape/dtype mismatch"));
    }
    let dense_entries = product(&[batch, heads, time, time])?;
    let value_entries = product(&[batch, heads, time, width])?;
    if dense_entries > max_dense_entries || value_entries > max_value_entries {
        return Err(invalid("hybrid dense allocation bound exceeded"));
    }
    let raw = harmonic_scores.detach().flatten_all()?.to_vec1::<f32>()?;
    // mode=1 means keep the harmonic entry; zero replaces it by sparse weight
    // or causal zero. The same where_cond governs gradients and normalization.
    let mut keep_harmonic = vec![0u8; dense_entries];
    let mut replacements = vec![0f32; dense_entries];
    let mut replaced_pairs = 0usize;
    for (row, selected) in support.rows.iter().enumerate() {
        let query = row % time;
        let start = row * time; // bounded by checked dense_entries above
        for key in 0..=query {
            let value = raw[start + key];
            if !value.is_finite() || value <= 0.0 {
                return Err(invalid("causal harmonic score is nonpositive/nonfinite"));
            }
            keep_harmonic[start + key] = 1;
        }
        for (rank, &key) in selected.iter().enumerate() {
            keep_harmonic[start + key] = 0;
            replacements[start + key] = raw_rank_weight(rank)?;
            replaced_pairs += 1; // at most checked dense_entries
        }
    }
    let shape = harmonic_scores.shape();
    let keep = Tensor::from_vec(keep_harmonic, shape, harmonic_scores.device())?;
    let replacement = Tensor::from_vec(replacements, shape, harmonic_scores.device())?;
    let scores = keep.where_cond(harmonic_scores, &replacement)?;
    let mass = scores.sum_keepdim(3)?;
    let masses = mass.detach().flatten_all()?.to_vec1::<f32>()?;
    if masses.iter().any(|x| !x.is_finite() || *x <= 0.0) {
        return Err(invalid("hybrid mass is nonpositive/nonfinite"));
    }
    let minimum_mass = masses.into_iter().fold(f32::INFINITY, f32::min);
    let attended = scores.broadcast_div(&mass)?.matmul(&values.contiguous()?)?;
    if attended
        .detach()
        .flatten_all()?
        .to_vec1::<f32>()?
        .iter()
        .any(|x| !x.is_finite())
    {
        return Err(invalid("hybrid attended output is nonfinite"));
    }
    Ok(HybridResult {
        attended,
        minimum_mass,
        replaced_pairs,
    })
}

pub struct CorrectedRow {
    pub attended: Vec<f32>,
    pub mass: f32,
    /// Sum of absolute mass terms divided by final mass. A diagnostic of
    /// cancellation, not a proof of numerical accuracy and not a clamp.
    pub mass_condition_proxy: f64,
}

/// Correct a single already-inclusive recurrent row. `harmonic_selected` and
/// `selected_values` must refer to exactly the row's ranked support, in order,
/// evaluated with the same projection, parameters, normalization and kernel as
/// the base M,z state. The caller binds those identities. No decay is supported.
/// Accumulation is explicitly F32, matching the prototype recurrent state.
pub fn correct_recurrent_rank_row(
    base_numerator: &[f32],
    base_mass: f32,
    harmonic_selected: &[f32],
    selected_values: &[f32],
    max_selected: usize,
) -> Result<CorrectedRow> {
    let width = base_numerator.len();
    let selected = harmonic_selected.len();
    if width == 0
        || selected > max_selected
        || selected_values.len() != product(&[selected, width])?
        || !base_mass.is_finite()
        || base_mass <= 0.0
        || base_numerator
            .iter()
            .chain(selected_values)
            .any(|x| !x.is_finite())
        || harmonic_selected
            .iter()
            .any(|x| !x.is_finite() || *x <= 0.0)
    {
        return Err(invalid("invalid recurrent hybrid row"));
    }
    let mut numerator = base_numerator.to_vec();
    let mut mass = base_mass;
    let mut absolute_mass_terms = f64::from(base_mass);
    for (rank, &harmonic) in harmonic_selected.iter().enumerate() {
        let correction = raw_rank_weight(rank)? - harmonic;
        mass += correction;
        absolute_mass_terms += f64::from(correction).abs();
        for lane in 0..width {
            numerator[lane] += correction * selected_values[rank * width + lane];
        }
    }
    if !mass.is_finite() || mass <= 0.0 {
        return Err(invalid("recurrent hybrid mass failed; no clamp applied"));
    }
    for value in &mut numerator {
        *value /= mass;
        if !value.is_finite() {
            return Err(invalid("recurrent hybrid output is nonfinite"));
        }
    }
    Ok(CorrectedRow {
        attended: numerator,
        mass,
        mass_condition_proxy: absolute_mass_terms / f64::from(mass),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::{Device, Var};

    #[test]
    fn recurrent_correction_replaces_overlap_and_normalizer() {
        // h=(.2,.3,.4), v=(2,5,7), support ranked as positions(2,0).
        // New scores=(.5,.3,1), rather than adding sparse weights to h.
        let result = correct_recurrent_rank_row(&[4.7], 0.9, &[0.4, 0.2], &[7., 2.], 2).unwrap();
        assert!((result.mass - 1.8).abs() < 1e-6);
        assert!((result.attended[0] - 9.5 / 1.8).abs() < 1e-6);
        let identity = correct_recurrent_rank_row(&[4.7], 0.9, &[], &[], 0).unwrap();
        assert!((identity.attended[0] - 4.7 / 0.9).abs() < 1e-6);
    }

    #[test]
    fn support_rejects_future_duplicate_and_over_budget_rows() {
        assert!(RankedSupport::new(1, 1, 2, vec![vec![1], vec![]], 2).is_err());
        assert!(RankedSupport::new(1, 1, 2, vec![vec![], vec![0, 0]], 2).is_err());
        assert!(RankedSupport::new(1, 1, 2, vec![vec![], vec![1, 0]], 1).is_err());
        assert!(RankedSupport::new(usize::MAX, 2, 2, vec![], 1).is_err());
    }

    #[test]
    fn dense_replacement_is_causal_and_keeps_unselected_gradient() {
        let device = Device::Cpu;
        let h = Var::from_vec(
            vec![0.2f32, 999., 999., 0.2, 0.3, 999., 0.2, 0.3, 0.4],
            (1, 1, 3, 3),
            &device,
        )
        .unwrap();
        let v = Tensor::from_vec(vec![2f32, 5., 7.], (1, 1, 3, 1), &device).unwrap();
        let support = RankedSupport::new(1, 1, 3, vec![vec![], vec![], vec![2, 0]], 2).unwrap();
        let out = quadratic_rank_hybrid(h.as_tensor(), &v, &support, 9, 3).unwrap();
        let values = out
            .attended
            .flatten_all()
            .unwrap()
            .to_vec1::<f32>()
            .unwrap();
        assert!((values[0] - 2.).abs() < 1e-6);
        assert!((values[2] - 9.5 / 1.8).abs() < 1e-6);
        let loss = out.attended.narrow(2, 2, 1).unwrap().sum_all().unwrap();
        let gradients = loss.backward().unwrap();
        let gradient = gradients
            .get(h.as_tensor())
            .unwrap()
            .flatten_all()
            .unwrap()
            .to_vec1::<f32>()
            .unwrap();
        assert_eq!(gradient[6], 0.);
        assert_ne!(gradient[7], 0.);
        assert_eq!(gradient[8], 0.);
    }
}
