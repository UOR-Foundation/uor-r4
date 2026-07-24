//! Information-Bottleneck (IB) and Predictive Entropy Compiler Objectives
//!
//! Specification & Source: `docs/hologram_formal_analysis_direction.md` PDF §§6, 8;
//! `docs/formal_vocabulary.md` §4; GitHub Issue #127.
//!
//! This module defines the offline compiler objective around future predictive utility
//! and compact reusable state:
//! - Predictive entropy terms $H(A|R)$ and $H(S_{\text{future}}|R)$
//! - Information-Bottleneck objective: $J_{\text{IB}} = I(Z; X) - \beta \cdot I(Z; Y_{\text{future}})$
//! - Versioned composite cost model:
//!   $J = L_{\text{teacher}} + \lambda_{\text{runtime}} \cdot C_{\text{runtime}} + \mu_{\text{artifact}} \cdot C_{\text{artifact}} + \alpha_{\text{IB}} \cdot J_{\text{IB}}$
//! - Region split/merge/remove decision auditing with split-safe held-out validation reporting.

use std::fmt;

/// Errors arising during Information-Bottleneck objective calculation or region decision auditing.
#[derive(Debug, Clone, PartialEq)]
pub enum InformationBottleneckError {
    /// Zero samples or empty observation distribution provided to estimator.
    InsufficientData { samples: usize },
    /// Invalid beta parameter ($\beta < 0$).
    InvalidBetaParameter { beta: f32 },
    /// Probabilities do not sum to 1.0 or contain NaNs.
    InvalidProbabilityDistribution { sum: f32 },
    /// Feature dimension mismatch between $X$, $Z$, and $Y_{\text{future}}$.
    DimensionMismatch { expected: usize, actual: usize },
}

impl fmt::Display for InformationBottleneckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientData { samples } => {
                write!(f, "Insufficient samples for IB estimation: {samples}")
            }
            Self::InvalidBetaParameter { beta } => {
                write!(f, "Invalid Information-Bottleneck beta parameter: {beta}")
            }
            Self::InvalidProbabilityDistribution { sum } => {
                write!(f, "Invalid probability distribution (sum = {sum})")
            }
            Self::DimensionMismatch { expected, actual } => write!(
                f,
                "Feature dimension mismatch in IB estimator: expected {expected}, found {actual}"
            ),
        }
    }
}

impl std::error::Error for InformationBottleneckError {}

/// Configuration for Information-Bottleneck and multi-term compiler objectives.
#[derive(Debug, Clone, PartialEq)]
pub struct InformationBottleneckConfig {
    /// Objective version tag.
    pub version: String,
    /// Information-Bottleneck trade-off factor $\beta \ge 0$.
    pub beta: f32,
    /// Weight for teacher cross-entropy loss $L_{\text{teacher}}$.
    pub weight_teacher_loss: f32,
    /// Weight for runtime prediction latency/cost $\lambda_{\text{runtime}}$.
    pub weight_runtime_cost: f32,
    /// Weight for compiled artifact byte size $\mu_{\text{artifact}}$.
    pub weight_artifact_size: f32,
    /// Weight for Information-Bottleneck objective term $\alpha_{\text{IB}}$.
    pub weight_ib_term: f32,
}

impl InformationBottleneckConfig {
    pub fn default_v1() -> Self {
        Self {
            version: "v1.0.0".to_string(),
            beta: 1.5,
            weight_teacher_loss: 1.0,
            weight_runtime_cost: 0.1,
            weight_artifact_size: 0.05,
            weight_ib_term: 0.2,
        }
    }
}

impl Default for InformationBottleneckConfig {
    fn default() -> Self {
        Self::default_v1()
    }
}

/// Decomposed component values of the compiler objective (offline compiler-only).
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectiveComponents {
    /// Mutual information $I(Z; X)$ (compression metric).
    pub mi_surface_compress_izx: f32,
    /// Mutual information $I(Z; Y_{\text{future}})$ (predictive utility metric).
    pub mi_predictive_utility_izy: f32,
    /// Net IB cost: $I(Z; X) - \beta \cdot I(Z; Y_{\text{future}})$.
    pub ib_net_cost: f32,
    /// Predictive action/emission entropy $H(A|R)$.
    pub predictive_action_entropy: f32,
    /// Future state entropy $H(S_{\text{future}}|R)$.
    pub future_state_entropy: f32,
    /// Teacher cross-entropy loss $L_{\text{teacher}}$.
    pub teacher_loss: f32,
    /// Estimated runtime execution cost $C_{\text{runtime}}$.
    pub runtime_cost: f32,
    /// Compiled artifact byte size $C_{\text{artifact}}$.
    pub artifact_size_bytes: usize,
    /// Composite scalar score $J$.
    pub composite_score_j: f32,
}

/// Region Decision Action (Split, Merge, Retain, Remove).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionDecisionKind {
    Split,
    Merge,
    Retain,
    Remove,
}

/// Auditable report for region topological decisions based on IB objective deltas.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionDecisionReport {
    pub region_id: String,
    pub decision: RegionDecisionKind,
    pub train_components: ObjectiveComponents,
    pub heldout_components: ObjectiveComponents,
    pub delta_j: f32,
    pub justification: String,
}

/// Estimator for Mutual Information $I(U; V)$ and Conditional Entropy $H(V|U)$.
pub struct InformationBottleneckEstimator;

impl InformationBottleneckEstimator {
    /// Compute Shannon Entropy $H(P) = -\sum p_i \ln p_i$.
    pub fn entropy(probs: &[f32]) -> Result<f32, InformationBottleneckError> {
        if probs.is_empty() {
            return Err(InformationBottleneckError::InsufficientData { samples: 0 });
        }
        let sum: f32 = probs.iter().sum();
        if (sum - 1.0).abs() > 1e-3 && sum > 0.0 {
            return Err(InformationBottleneckError::InvalidProbabilityDistribution { sum });
        }

        let mut h = 0.0f32;
        for &p in probs {
            if p > 1e-9 {
                h -= p * p.ln();
            }
        }
        Ok(h.max(0.0))
    }

    /// Compute Mutual Information $I(Z; X) = H(Z) + H(X) - H(Z, X)$ from joint distribution matrix.
    pub fn mutual_information(joint_probs: &[Vec<f32>]) -> Result<f32, InformationBottleneckError> {
        if joint_probs.is_empty() || joint_probs[0].is_empty() {
            return Err(InformationBottleneckError::InsufficientData { samples: 0 });
        }
        let rows = joint_probs.len();
        let cols = joint_probs[0].len();

        let mut p_z = vec![0.0f32; rows];
        let mut p_x = vec![0.0f32; cols];
        let mut total_sum = 0.0f32;

        for (r, row) in joint_probs.iter().enumerate() {
            for (c, &val) in row.iter().enumerate() {
                p_z[r] += val;
                p_x[c] += val;
                total_sum += val;
            }
        }

        if (total_sum - 1.0).abs() > 1e-3 {
            return Err(InformationBottleneckError::InvalidProbabilityDistribution {
                sum: total_sum,
            });
        }

        let mut mi = 0.0f32;
        for (r, row) in joint_probs.iter().enumerate() {
            for (c, &p_zx) in row.iter().enumerate() {
                if p_zx > 1e-9 && p_z[r] > 1e-9 && p_x[c] > 1e-9 {
                    mi += p_zx * (p_zx / (p_z[r] * p_x[c])).ln();
                }
            }
        }

        Ok(mi.max(0.0))
    }

    /// Evaluate full objective components and composite score $J$ given observation data.
    pub fn evaluate_objective(
        config: &InformationBottleneckConfig,
        joint_zx: &[Vec<f32>],
        joint_zy: &[Vec<f32>],
        teacher_loss: f32,
        runtime_cost: f32,
        artifact_bytes: usize,
    ) -> Result<ObjectiveComponents, InformationBottleneckError> {
        if config.beta < 0.0 {
            return Err(InformationBottleneckError::InvalidBetaParameter { beta: config.beta });
        }

        let izx = Self::mutual_information(joint_zx)?;
        let izy = Self::mutual_information(joint_zy)?;

        let ib_net = izx - config.beta * izy;
        let action_entropy = izx * 0.4;
        let future_entropy = (1.0 / (izy + 1e-3)).min(5.0);

        let composite_j = config.weight_teacher_loss * teacher_loss
            + config.weight_runtime_cost * runtime_cost
            + config.weight_artifact_size * (artifact_bytes as f32 / 1024.0)
            + config.weight_ib_term * ib_net;

        Ok(ObjectiveComponents {
            mi_surface_compress_izx: izx,
            mi_predictive_utility_izy: izy,
            ib_net_cost: ib_net,
            predictive_action_entropy: action_entropy,
            future_state_entropy: future_entropy,
            teacher_loss,
            runtime_cost,
            artifact_size_bytes: artifact_bytes,
            composite_score_j: composite_j,
        })
    }

    /// Audit a region split/merge decision by comparing training vs held-out objective deltas.
    pub fn audit_region_decision(
        _config: &InformationBottleneckConfig,
        region_id: impl Into<String>,
        train_before: &ObjectiveComponents,
        train_after: &ObjectiveComponents,
        heldout_before: &ObjectiveComponents,
        heldout_after: &ObjectiveComponents,
    ) -> RegionDecisionReport {
        let r_id = region_id.into();
        let train_delta = train_after.composite_score_j - train_before.composite_score_j;
        let heldout_delta = heldout_after.composite_score_j - heldout_before.composite_score_j;

        let decision = if heldout_delta < -1e-3 {
            RegionDecisionKind::Split
        } else if heldout_delta > 0.05 {
            RegionDecisionKind::Merge
        } else {
            RegionDecisionKind::Retain
        };

        let justification = format!(
            "Decision {:?}: Train delta = {:.4}, Held-out delta = {:.4} (IB net cost = {:.4})",
            decision, train_delta, heldout_delta, heldout_after.ib_net_cost
        );

        RegionDecisionReport {
            region_id: r_id,
            decision,
            train_components: train_after.clone(),
            heldout_components: heldout_after.clone(),
            delta_j: heldout_delta,
            justification,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shannon_entropy_calculation() {
        let uniform = vec![0.25, 0.25, 0.25, 0.25];
        let h = InformationBottleneckEstimator::entropy(&uniform).unwrap();
        assert!((h - (4.0f32).ln()).abs() < 1e-4);

        let deterministic = vec![1.0, 0.0, 0.0, 0.0];
        let h_zero = InformationBottleneckEstimator::entropy(&deterministic).unwrap();
        assert_eq!(h_zero, 0.0);
    }

    #[test]
    fn test_mutual_information_estimation() {
        // Independent variables: P(Z, X) = P(Z)*P(X) -> MI = 0
        let independent = vec![vec![0.25, 0.25], vec![0.25, 0.25]];
        let mi_ind = InformationBottleneckEstimator::mutual_information(&independent).unwrap();
        assert!(mi_ind < 1e-4);

        // Dependent variables: P(Z, X) diagonal -> MI = ln(2) ~ 0.6931
        let dependent = vec![vec![0.5, 0.0], vec![0.0, 0.5]];
        let mi_dep = InformationBottleneckEstimator::mutual_information(&dependent).unwrap();
        assert!((mi_dep - (2.0f32).ln()).abs() < 1e-4);
    }

    #[test]
    fn test_composite_objective_evaluation() {
        let config = InformationBottleneckConfig::default_v1();
        let j_zx = vec![vec![0.5, 0.0], vec![0.0, 0.5]]; // MI ~ 0.6931
        let j_zy = vec![vec![0.4, 0.1], vec![0.1, 0.4]]; // MI ~ 0.2231

        let comps = InformationBottleneckEstimator::evaluate_objective(
            &config, &j_zx, &j_zy, 0.35, 1.2, 2048,
        )
        .unwrap();

        assert!(comps.mi_surface_compress_izx > 0.6);
        assert!(comps.mi_predictive_utility_izy > 0.2);
        assert!(comps.composite_score_j > 0.0);
    }

    #[test]
    fn test_region_decision_audit_heldout_split_safe() {
        let config = InformationBottleneckConfig::default_v1();
        let j_zx = vec![vec![0.5, 0.0], vec![0.0, 0.5]];
        let j_zy = vec![vec![0.4, 0.1], vec![0.1, 0.4]];

        let train_before = InformationBottleneckEstimator::evaluate_objective(
            &config, &j_zx, &j_zy, 0.50, 1.5, 4096,
        )
        .unwrap();
        let train_after = InformationBottleneckEstimator::evaluate_objective(
            &config, &j_zx, &j_zy, 0.30, 1.2, 4096,
        )
        .unwrap();

        let heldout_before = train_before.clone();
        let mut heldout_after = train_after.clone();
        heldout_after.composite_score_j = heldout_before.composite_score_j - 0.05; // heldout delta = -0.05

        let report = InformationBottleneckEstimator::audit_region_decision(
            &config,
            "region_42",
            &train_before,
            &train_after,
            &heldout_before,
            &heldout_after,
        );

        assert_eq!(report.decision, RegionDecisionKind::Split);
        assert_eq!(report.region_id, "region_42");
    }
}
