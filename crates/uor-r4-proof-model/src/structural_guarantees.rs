//! Structural Graph and Planner Guarantees Proof Model
//!
//! Specification & Source: `docs/hologram_formal_analysis_direction.md` PDF §13;
//! `docs/formal_vocabulary.md` §7; GitHub Issue #132.
//!
//! This module provides executable proof specifications for structural graph properties
//! currently implemented here:
//! - Determinism (repeated-invocation output equality)
//! - Bounded Memory, Latency, Frontier Size, and Degree Bounds — any generic
//!   metric bound (`verify_resource_bound`)
//! - Constraint Preservation ($s_i \notin C$) (`verify_constraint_safety`)
//! - Proof matrix status auditing (`audit_proof_matrix_entry`)
//!
//! Canonical serialization, replay determinism/witness content integrity, evidence
//! non-duplication, and safe-arithmetic obligations are tracked in
//! `docs/hologram_formal_analysis_direction.md` PDF §13 and `docs/formal_vocabulary.md` §7
//! but are not yet covered by executable verifiers in this module.

use crate::proof_matrix::{ProofStatus, ProofStatusMatrix};
use std::fmt;

/// Errors arising during structural proof obligation verification.
#[derive(Debug, Clone, PartialEq)]
pub enum ProofValidationError {
    /// Graph or planner output violates determinism obligation.
    NondeterministicOutput { obligation_id: String },
    /// Resource usage exceeds declared bound limit.
    ResourceBoundExceeded {
        obligation_id: String,
        metric: String,
        actual: usize,
        limit: usize,
    },
    /// State sequence violates forbidden constraint. `state_id` is the violating state
    /// and `region_id` is the forbidden region it entered; both fields are kept distinct
    /// for forward compatibility with region groupings, though the current flat
    /// forbidden-state model treats each forbidden state as its own singleton region,
    /// so today `region_id` equals `state_id`.
    ConstraintSafetyViolated { state_id: String, region_id: String },
    /// Proof matrix status drift detected.
    StatusDrift {
        obligation_id: String,
        expected: String,
        actual: String,
    },
}

impl fmt::Display for ProofValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NondeterministicOutput { obligation_id } => {
                write!(f, "Determinism obligation '{obligation_id}' failed: outputs differ")
            }
            Self::ResourceBoundExceeded {
                obligation_id,
                metric,
                actual,
                limit,
            } => write!(
                f,
                "Resource bound obligation '{obligation_id}' exceeded for '{metric}': actual {actual} > limit {limit}"
            ),
            Self::ConstraintSafetyViolated { state_id, region_id } => write!(
                f,
                "Constraint safety obligation violated: state '{state_id}' entered forbidden region '{region_id}'"
            ),
            Self::StatusDrift { obligation_id, expected, actual } => write!(
                f,
                "Proof matrix status drift for '{obligation_id}': expected '{expected}', found '{actual}'"
            ),
        }
    }
}

impl std::error::Error for ProofValidationError {}

/// Category of structural proof obligation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructuralObligationKind {
    Determinism,
    BoundedResource,
    ConstraintSafety,
    EvidenceIntegrity,
    ReplayWitness,
}

/// Report summarizing proof obligation evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct ProofVerificationReport {
    pub obligation_id: String,
    pub kind: StructuralObligationKind,
    pub status: ProofStatus,
    pub verified: bool,
    pub details: String,
}

/// Executable verifier for structural graph and planner guarantees.
pub struct StructuralGuaranteeVerifier;

impl StructuralGuaranteeVerifier {
    /// Verify determinism obligation by checking identical outputs across multiple invocations.
    pub fn verify_determinism<F, T>(
        obligation_id: impl Into<String>,
        run_fn: F,
    ) -> Result<ProofVerificationReport, ProofValidationError>
    where
        F: Fn() -> T,
        T: PartialEq + fmt::Debug,
    {
        let obl_id = obligation_id.into();
        let run1 = run_fn();
        let run2 = run_fn();

        if run1 != run2 {
            return Err(ProofValidationError::NondeterministicOutput {
                obligation_id: obl_id,
            });
        }

        Ok(ProofVerificationReport {
            obligation_id: obl_id,
            kind: StructuralObligationKind::Determinism,
            status: ProofStatus::Verified,
            verified: true,
            details: "Output determinism verified across independent runs".to_string(),
        })
    }

    /// Verify resource bound obligation (memory, frontier, latency).
    pub fn verify_resource_bound(
        obligation_id: impl Into<String>,
        metric: &str,
        actual_val: usize,
        limit_val: usize,
    ) -> Result<ProofVerificationReport, ProofValidationError> {
        let obl_id = obligation_id.into();
        if actual_val > limit_val {
            return Err(ProofValidationError::ResourceBoundExceeded {
                obligation_id: obl_id,
                metric: metric.to_string(),
                actual: actual_val,
                limit: limit_val,
            });
        }

        Ok(ProofVerificationReport {
            obligation_id: obl_id,
            kind: StructuralObligationKind::BoundedResource,
            status: ProofStatus::Verified,
            verified: true,
            details: format!("Metric '{metric}' ({actual_val}) within bound limit ({limit_val})"),
        })
    }

    /// Verify constraint preservation obligation for state trajectories.
    pub fn verify_constraint_safety(
        obligation_id: impl Into<String>,
        state_sequence: &[&str],
        forbidden_states: &[&str],
    ) -> Result<ProofVerificationReport, ProofValidationError> {
        let obl_id = obligation_id.into();
        for &s in state_sequence {
            if forbidden_states.contains(&s) {
                return Err(ProofValidationError::ConstraintSafetyViolated {
                    state_id: s.to_string(),
                    region_id: s.to_string(),
                });
            }
        }

        Ok(ProofVerificationReport {
            obligation_id: obl_id,
            kind: StructuralObligationKind::ConstraintSafety,
            status: ProofStatus::Verified,
            verified: true,
            details: "No forbidden states entered across trajectory".to_string(),
        })
    }

    /// Audit proof matrix status against expected status.
    pub fn audit_proof_matrix_entry(
        matrix: &ProofStatusMatrix,
        theorem_name: &str,
        expected_status: ProofStatus,
    ) -> Result<ProofVerificationReport, ProofValidationError> {
        let entry = matrix
            .entries
            .iter()
            .find(|e| e.name == theorem_name)
            .ok_or_else(|| ProofValidationError::StatusDrift {
                obligation_id: theorem_name.to_string(),
                expected: format!("{expected_status:?}"),
                actual: "MissingEntry".to_string(),
            })?;

        if entry.status != expected_status {
            return Err(ProofValidationError::StatusDrift {
                obligation_id: theorem_name.to_string(),
                expected: format!("{expected_status:?}"),
                actual: format!("{:?}", entry.status),
            });
        }

        Ok(ProofVerificationReport {
            obligation_id: theorem_name.to_string(),
            kind: StructuralObligationKind::EvidenceIntegrity,
            status: entry.status,
            verified: true,
            details: format!(
                "Proof matrix entry '{theorem_name}' matches status {:?}",
                entry.status
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_determinism_success() {
        let report =
            StructuralGuaranteeVerifier::verify_determinism("OBL-DET-01", || vec![1, 2, 3, 4])
                .unwrap();

        assert!(report.verified);
        assert_eq!(report.status, ProofStatus::Verified);
    }

    #[test]
    fn test_verify_resource_bound_success_and_failure() {
        let ok_report = StructuralGuaranteeVerifier::verify_resource_bound(
            "OBL-MEM-01",
            "memory_bytes",
            512,
            1024,
        )
        .unwrap();
        assert!(ok_report.verified);

        let err = StructuralGuaranteeVerifier::verify_resource_bound(
            "OBL-MEM-01",
            "memory_bytes",
            2048,
            1024,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ProofValidationError::ResourceBoundExceeded { .. }
        ));
    }

    #[test]
    fn test_verify_constraint_safety() {
        let report = StructuralGuaranteeVerifier::verify_constraint_safety(
            "OBL-SAFE-01",
            &["s0", "s1", "s2"],
            &["hazard_0"],
        )
        .unwrap();
        assert!(report.verified);

        let err = StructuralGuaranteeVerifier::verify_constraint_safety(
            "OBL-SAFE-01",
            &["s0", "hazard_0", "s2"],
            &["hazard_0"],
        )
        .unwrap_err();
        assert!(matches!(
            err,
            ProofValidationError::ConstraintSafetyViolated { .. }
        ));
    }
}
