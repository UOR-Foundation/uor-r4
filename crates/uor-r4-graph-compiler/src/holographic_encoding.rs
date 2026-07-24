//! Holographic Encoding ($H(x)$), Partial Reconstruction ($R$), and Progressive Fidelity
//!
//! Specification & Source: `docs/hologram_formal_analysis_direction.md` PDF §5;
//! `docs/formal_vocabulary.md` §3; GitHub Issue #126.
//!
//! This module turns "hologram" from a metaphor into a measurable representation contract:
//! - For an observation $x$, the compiler emits an overlapping projection family
//!   $H(x) = \{h_0, h_1, \dots, h_k\}$ across progressive depth tiers.
//! - Partial reconstruction operator $R(H(x))$ accumulates distributed evidence across projections.
//! - Progressive fidelity measure demonstrates monotonic divergence reduction $D(P_\theta, P_G)$
//!   as projection count $k$ increases.
//! - Ablation protocol measures graceful degradation under missing or perturbed sub-projections.
//! - Degenerate encodings (empty projections, single-bit memorization, duplicate projections)
//!   are rejected by quality gate checks.

use std::collections::HashSet;
use std::fmt;

/// Errors arising during holographic encoding or certificate validation.
#[derive(Debug, Clone, PartialEq)]
pub enum HolographicEncodingError {
    /// Encoding contains no projections.
    EmptyProjectionSet,
    /// Projection family contains duplicate projections.
    DuplicateProjection { projection_id: String },
    /// Degenerate encoding: single-node / single-bit memorization.
    SingleNodeMemorization { node_id: String },
    /// Sub-projection dimension mismatch.
    DimensionMismatch { expected: usize, actual: usize },
    /// Invalid divergence or probability distribution.
    InvalidDivergence { reason: String },
}

impl fmt::Display for HolographicEncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProjectionSet => write!(f, "Holographic encoding set H(x) cannot be empty"),
            Self::DuplicateProjection { projection_id } => {
                write!(f, "Duplicate sub-projection ID '{projection_id}' in H(x)")
            }
            Self::SingleNodeMemorization { node_id } => write!(
                f,
                "Degenerate encoding rejected: single-node memorization at '{node_id}'"
            ),
            Self::DimensionMismatch { expected, actual } => write!(
                f,
                "Sub-projection dimension mismatch: expected {expected}, found {actual}"
            ),
            Self::InvalidDivergence { reason } => {
                write!(f, "Invalid divergence calculation: {reason}")
            }
        }
    }
}

impl std::error::Error for HolographicEncodingError {}

/// A single sub-projection $h_i \in H(x)$ in the holographic projection family.
#[derive(Debug, Clone, PartialEq)]
pub struct SubProjection {
    /// Unique identifier for sub-projection.
    pub id: String,
    /// Depth tier level ($0..D-1$).
    pub depth: usize,
    /// Projection subspace vector weights.
    pub weights: Vec<f32>,
    /// Sign-bit signature mask.
    pub bit_mask: u64,
    /// Information entropy payload contribution.
    pub entropy_contribution: f32,
}

impl SubProjection {
    pub fn new(
        id: impl Into<String>,
        depth: usize,
        weights: Vec<f32>,
        bit_mask: u64,
        entropy_contribution: f32,
    ) -> Self {
        Self {
            id: id.into(),
            depth,
            weights,
            bit_mask,
            entropy_contribution: entropy_contribution.max(0.0),
        }
    }
}

/// Overlapping holographic projection family $H(x) = \{h_0, h_1, \dots, h_k\}$ for observation $x$.
#[derive(Debug, Clone, PartialEq)]
pub struct HolographicEncoding {
    /// Observation source identifier.
    pub observation_id: String,
    /// Family of sub-projections $\{h_0, \dots, h_k\}$.
    pub projections: Vec<SubProjection>,
    /// Total latent dimension.
    pub dimension: usize,
}

impl HolographicEncoding {
    /// Construct a new `HolographicEncoding` and validate against degenerate forms.
    pub fn new(
        observation_id: impl Into<String>,
        projections: Vec<SubProjection>,
        dimension: usize,
    ) -> Result<Self, HolographicEncodingError> {
        if projections.is_empty() {
            return Err(HolographicEncodingError::EmptyProjectionSet);
        }

        // Check for duplicates
        let mut seen = HashSet::new();
        for p in &projections {
            if !seen.insert(&p.id) {
                return Err(HolographicEncodingError::DuplicateProjection {
                    projection_id: p.id.clone(),
                });
            }
            if p.weights.len() != dimension {
                return Err(HolographicEncodingError::DimensionMismatch {
                    expected: dimension,
                    actual: p.weights.len(),
                });
            }
        }

        // Reject single-node memorization (only 1 sub-projection taking 100% weight)
        if projections.len() == 1 && projections[0].entropy_contribution == 0.0 {
            return Err(HolographicEncodingError::SingleNodeMemorization {
                node_id: projections[0].id.clone(),
            });
        }

        Ok(Self {
            observation_id: observation_id.into(),
            projections,
            dimension,
        })
    }

    /// Number of sub-projections $k = |H(x)|$.
    pub fn len(&self) -> usize {
        self.projections.len()
    }

    pub fn is_empty(&self) -> bool {
        self.projections.is_empty()
    }

    /// Partial reconstruction operator $R(H_k(x))$ over the first $k$ projections ($1 \le k \le |H(x)|$).
    pub fn reconstruct_partial(&self, k: usize) -> Result<Vec<f32>, HolographicEncodingError> {
        if k == 0 || k > self.projections.len() {
            return Err(HolographicEncodingError::EmptyProjectionSet);
        }

        let mut accumulated = vec![0.0f32; self.dimension];
        let sub_set = &self.projections[0..k];

        for p in sub_set {
            for (acc, w) in accumulated.iter_mut().zip(p.weights.iter()) {
                *acc += w;
            }
        }

        // Normalize by projection count (progressive fidelity averaging)
        let scale = 1.0 / (sub_set.len() as f32);
        for acc in &mut accumulated {
            *acc *= scale;
        }

        Ok(accumulated)
    }

    /// Perform ablation by removing a specified subset of sub-projection IDs.
    pub fn ablate(&self, ablated_ids: &[&str]) -> Result<Self, HolographicEncodingError> {
        let remaining: Vec<SubProjection> = self
            .projections
            .iter()
            .filter(|p| !ablated_ids.contains(&p.id.as_str()))
            .cloned()
            .collect();

        Self::new(
            format!("{}_ablated", self.observation_id),
            remaining,
            self.dimension,
        )
    }
}

/// Divergence metric evaluating reconstruction fidelity $D(P_\theta(\cdot|x), P_G(\cdot|H(x)))$.
pub struct DivergenceEvaluator;

impl DivergenceEvaluator {
    /// Compute Jensen-Shannon Divergence $D_{JS}(P_\theta \parallel P_G)$ between teacher and graph distributions.
    pub fn jensen_shannon(
        p_teacher: &[f32],
        p_graph: &[f32],
    ) -> Result<f32, HolographicEncodingError> {
        if p_teacher.len() != p_graph.len() || p_teacher.is_empty() {
            return Err(HolographicEncodingError::DimensionMismatch {
                expected: p_teacher.len(),
                actual: p_graph.len(),
            });
        }

        let mut js_div = 0.0f32;
        for (&p, &q) in p_teacher.iter().zip(p_graph.iter()) {
            let p_clamped = p.max(1e-9);
            let q_clamped = q.max(1e-9);
            let m = 0.5 * (p_clamped + q_clamped);
            js_div += 0.5 * (p_clamped * (p_clamped / m).ln() + q_clamped * (q_clamped / m).ln());
        }

        Ok(js_div.max(0.0))
    }
}

/// Certificate recording progressive fidelity, ablation metrics, and corpus CIDs.
#[derive(Debug, Clone, PartialEq)]
pub struct HolographicFidelityCertificate {
    /// Pinned observation / corpus CID hash.
    pub corpus_cid: String,
    /// Progressive fidelity divergence curve $D(k)$ for projection counts $k = 1 \dots N$.
    pub fidelity_curve: Vec<(usize, f32)>,
    /// Divergence under 25% ablation protocol.
    pub ablation_25pct_divergence: f32,
    /// Sample count evaluated.
    pub sample_count: usize,
    /// Is monotonic fidelity progression verified?
    pub is_progressive_fidelity_verified: bool,
}

impl HolographicFidelityCertificate {
    /// Generate a fidelity certificate for an encoding against teacher distribution $P_\theta$.
    pub fn evaluate(
        corpus_cid: impl Into<String>,
        encoding: &HolographicEncoding,
        teacher_target: &[f32],
    ) -> Result<Self, HolographicEncodingError> {
        let mut curve = Vec::new();
        let mut prev_div = f32::MAX;
        let mut is_monotonic = true;

        for k in 1..=encoding.len() {
            let rec = encoding.reconstruct_partial(k)?;
            let div = DivergenceEvaluator::jensen_shannon(teacher_target, &rec)?;
            curve.push((k, div));
            if div > prev_div + 1e-4 {
                is_monotonic = false;
            }
            prev_div = div;
        }

        // Evaluate 25% ablation
        let ablate_count = (encoding.len() / 4).max(1);
        let ablated_ids: Vec<&str> = encoding.projections[0..ablate_count]
            .iter()
            .map(|p| p.id.as_str())
            .collect();
        let ablated_enc = encoding.ablate(&ablated_ids)?;
        let ablated_rec = ablated_enc.reconstruct_partial(ablated_enc.len())?;
        let ablated_div = DivergenceEvaluator::jensen_shannon(teacher_target, &ablated_rec)?;

        Ok(Self {
            corpus_cid: corpus_cid.into(),
            fidelity_curve: curve,
            ablation_25pct_divergence: ablated_div,
            sample_count: encoding.len(),
            is_progressive_fidelity_verified: is_monotonic,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_holographic_encoding_progressive_fidelity() {
        let target = vec![0.5, 0.5];
        let p0 = SubProjection::new("h0", 0, vec![0.4, 0.6], 0b01, 0.5);
        let p1 = SubProjection::new("h1", 1, vec![0.5, 0.5], 0b10, 0.5);
        let p2 = SubProjection::new("h2", 2, vec![0.5, 0.5], 0b11, 0.5);

        let encoding = HolographicEncoding::new("obs_01", vec![p0, p1, p2], 2).unwrap();
        assert_eq!(encoding.len(), 3);

        let cert =
            HolographicFidelityCertificate::evaluate("cid_test_123", &encoding, &target).unwrap();
        assert!(cert.is_progressive_fidelity_verified);
        assert_eq!(cert.fidelity_curve.len(), 3);
    }

    #[test]
    fn test_degenerate_single_node_memorization_rejection() {
        let p0 = SubProjection::new("h0", 0, vec![1.0, 0.0], 0b01, 0.0);
        let err = HolographicEncoding::new("obs_bad", vec![p0], 2).unwrap_err();
        assert!(matches!(
            err,
            HolographicEncodingError::SingleNodeMemorization { .. }
        ));
    }

    #[test]
    fn test_duplicate_projection_rejection() {
        let p0 = SubProjection::new("h0", 0, vec![0.5, 0.5], 0b01, 0.5);
        let p0_dup = SubProjection::new("h0", 1, vec![0.5, 0.5], 0b10, 0.5);

        let err = HolographicEncoding::new("obs_dup", vec![p0, p0_dup], 2).unwrap_err();
        assert!(matches!(
            err,
            HolographicEncodingError::DuplicateProjection { .. }
        ));
    }

    #[test]
    fn test_ablation_semantics() {
        let p0 = SubProjection::new("h0", 0, vec![0.4, 0.6], 0b01, 0.5);
        let p1 = SubProjection::new("h1", 1, vec![0.6, 0.4], 0b10, 0.5);

        let encoding = HolographicEncoding::new("obs_ablate", vec![p0, p1], 2).unwrap();
        let ablated = encoding.ablate(&["h0"]).unwrap();

        assert_eq!(ablated.len(), 1);
        assert_eq!(ablated.projections[0].id, "h1");
    }
}
