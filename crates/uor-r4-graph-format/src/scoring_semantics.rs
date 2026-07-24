//! Normative Fixed-Point Scoring Semantics Module
//!
//! Specification & Source: `docs/scoring_semantics.md`; `docs/inference_contract.md`;
//! `docs/hologram_formal_analysis_direction.md` PDF §§7, 12, 13; `docs/formal_vocabulary.md`; GitHub Issue #158.
//!
//! This module provides a machine-readable, `no_std`, `alloc`-free implementation of the
//! normative fixed-point scoring semantics:
//! - Implements pre-quantized residual accumulation via saturating integer operations (`+`, `-`).
//! - Enforces overlap residualization (no-double-counting rule) using fixed-capacity tracked evidence.
//! - Enforces deterministic tie-breaking (ScoreQ descending, ID ascending).

use core::cmp::Ordering;
use core::fmt;

/// Semantic Versioning for the Normative Scoring Semantics Specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoringSemanticsVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl ScoringSemanticsVersion {
    pub const V1_0_0: Self = Self {
        major: 1,
        minor: 0,
        patch: 0,
    };
}

impl fmt::Display for ScoringSemanticsVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Typed Residual Contribution Kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResidualContributionKind {
    /// Root node base prior score B(v).
    RootPrior,
    /// Hierarchical child node correction residual.
    ChildCorrection,
    /// Interaction residual between co-occurring concepts.
    InteractionResidual,
    /// Reward contribution for goal satisfaction.
    GoalReward,
    /// Penalty contribution for hazard/constraint proximity.
    ConstraintPenalty,
    /// Penalty contribution for variance or entropy uncertainty.
    UncertaintyPenalty,
    /// Residual for token emission prediction.
    TokenEmission,
}

/// Single pre-quantized residual contribution item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResidualContribution {
    pub kind: ResidualContributionKind,
    pub contribution_id: u32,
    pub raw_value: i32,
}

/// Non-panicking error enum for scoring semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoringError {
    /// Tracked evidence set capacity exceeded.
    EvidenceCapacityExceeded,
    /// Invalid storage descriptor shift or zero point.
    InvalidStorageDescriptor,
    /// Audit-only invariant failure within the machine-readable verifier.
    AuditInvariantFailed(&'static str),
}

impl fmt::Display for ScoringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EvidenceCapacityExceeded => {
                write!(f, "Fixed-capacity evidence tracking set limit exceeded")
            }
            Self::InvalidStorageDescriptor => {
                write!(
                    f,
                    "Invalid storage descriptor shift or zero point parameters"
                )
            }
            Self::AuditInvariantFailed(detail) => {
                write!(f, "Scoring semantics audit invariant failed: {detail}")
            }
        }
    }
}

/// Stack-allocated fixed-capacity score accumulator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreAccumulator<const MAX_EVIDENCE: usize = 32> {
    current_score: i32,
    tracked_evidence: [u32; MAX_EVIDENCE],
    evidence_count: usize,
}

impl<const MAX_EVIDENCE: usize> Default for ScoreAccumulator<MAX_EVIDENCE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX_EVIDENCE: usize> ScoreAccumulator<MAX_EVIDENCE> {
    /// Create a new zeroed score accumulator.
    pub const fn new() -> Self {
        Self {
            current_score: 0,
            tracked_evidence: [0; MAX_EVIDENCE],
            evidence_count: 0,
        }
    }

    /// Return the current accumulated score.
    pub const fn score(&self) -> i32 {
        self.current_score
    }

    /// Return the number of unique evidence contributions accumulated.
    pub const fn evidence_count(&self) -> usize {
        self.evidence_count
    }

    /// Check if a contribution ID has already been incorporated (no-double-counting rule).
    pub fn contains_evidence(&self, contribution_id: u32) -> bool {
        let mut i = 0;
        while i < self.evidence_count {
            if self.tracked_evidence[i] == contribution_id {
                return true;
            }
            i += 1;
        }
        false
    }

    /// Accumulate a residual contribution with saturating arithmetic and overlap residualization.
    pub fn accumulate(
        &mut self,
        contribution: &ResidualContribution,
    ) -> Result<bool, ScoringError> {
        // Enforce no-double-counting rule
        if self.contains_evidence(contribution.contribution_id) {
            return Ok(false); // Ignored duplicate contribution
        }

        if self.evidence_count >= MAX_EVIDENCE {
            return Err(ScoringError::EvidenceCapacityExceeded);
        }

        // Apply every pre-quantized residual as an already-signed ScoreQ contribution.
        self.current_score = self.current_score.saturating_add(contribution.raw_value);

        self.tracked_evidence[self.evidence_count] = contribution.contribution_id;
        self.evidence_count += 1;
        Ok(true)
    }

    /// Compare two candidate (score, ID) pairs with deterministic tie-breaking:
    /// 1. Score descending (higher score first).
    /// 2. ID ascending (lower ID first).
    pub fn compare_candidates(score_a: i32, id_a: u32, score_b: i32, id_b: u32) -> Ordering {
        match score_b.cmp(&score_a) {
            Ordering::Equal => id_a.cmp(&id_b),
            ord => ord,
        }
    }
}

/// Scoring Semantics Verifier Engine.
pub struct ScoringSemanticsVerifier;

impl ScoringSemanticsVerifier {
    /// Return the normative scoring semantics version.
    pub const fn version() -> ScoringSemanticsVersion {
        ScoringSemanticsVersion::V1_0_0
    }

    /// Audit accumulator compliance and deterministic tie-breaking logic.
    pub fn audit_scoring_compliance() -> Result<(), ScoringError> {
        let mut acc = ScoreAccumulator::<16>::new();
        acc.accumulate(&ResidualContribution {
            kind: ResidualContributionKind::RootPrior,
            contribution_id: 1,
            raw_value: 100,
        })?;
        acc.accumulate(&ResidualContribution {
            kind: ResidualContributionKind::ConstraintPenalty,
            contribution_id: 2,
            raw_value: -25,
        })?;
        acc.accumulate(&ResidualContribution {
            kind: ResidualContributionKind::UncertaintyPenalty,
            contribution_id: 3,
            raw_value: -10,
        })?;
        if acc.score() != 65 {
            return Err(ScoringError::AuditInvariantFailed(
                "signed penalty residuals must lower the accumulated score",
            ));
        }

        // Test overlap residualization (duplicate ignored)
        let added = acc.accumulate(&ResidualContribution {
            kind: ResidualContributionKind::RootPrior,
            contribution_id: 1,
            raw_value: 100,
        })?;
        if added {
            return Err(ScoringError::AuditInvariantFailed(
                "duplicate contribution IDs must be ignored",
            ));
        }

        let mut saturating_high = ScoreAccumulator::<4>::new();
        saturating_high.accumulate(&ResidualContribution {
            kind: ResidualContributionKind::GoalReward,
            contribution_id: 11,
            raw_value: i32::MAX - 5,
        })?;
        saturating_high.accumulate(&ResidualContribution {
            kind: ResidualContributionKind::TokenEmission,
            contribution_id: 12,
            raw_value: 10,
        })?;
        if saturating_high.score() != i32::MAX {
            return Err(ScoringError::AuditInvariantFailed(
                "positive overflow must clamp to i32::MAX",
            ));
        }

        let mut saturating_low = ScoreAccumulator::<4>::new();
        saturating_low.accumulate(&ResidualContribution {
            kind: ResidualContributionKind::InteractionResidual,
            contribution_id: 21,
            raw_value: i32::MIN + 5,
        })?;
        saturating_low.accumulate(&ResidualContribution {
            kind: ResidualContributionKind::ConstraintPenalty,
            contribution_id: 22,
            raw_value: -10,
        })?;
        if saturating_low.score() != i32::MIN {
            return Err(ScoringError::AuditInvariantFailed(
                "negative overflow must clamp to i32::MIN",
            ));
        }

        // Test deterministic tie-breaking (equal scores => lower ID wins)
        let ord = ScoreAccumulator::<16>::compare_candidates(500, 10, 500, 20);
        if ord != Ordering::Less {
            return Err(ScoringError::AuditInvariantFailed(
                "equal scores must prefer the lower candidate ID",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_accumulator_saturating_arithmetic() {
        let mut acc = ScoreAccumulator::<8>::new();
        acc.accumulate(&ResidualContribution {
            kind: ResidualContributionKind::RootPrior,
            contribution_id: 100,
            raw_value: i32::MAX - 10,
        })
        .unwrap();

        acc.accumulate(&ResidualContribution {
            kind: ResidualContributionKind::ChildCorrection,
            contribution_id: 101,
            raw_value: 50,
        })
        .unwrap();

        assert_eq!(acc.score(), i32::MAX); // Saturation clamp
    }

    #[test]
    fn test_overlap_residualization_no_double_counting() {
        let mut acc = ScoreAccumulator::<8>::new();
        let item = ResidualContribution {
            kind: ResidualContributionKind::InteractionResidual,
            contribution_id: 42,
            raw_value: 250,
        };

        assert!(acc.accumulate(&item).unwrap());
        assert_eq!(acc.score(), 250);

        // Second accumulation of same ID must return false and not alter score
        assert!(!acc.accumulate(&item).unwrap());
        assert_eq!(acc.score(), 250);
        assert_eq!(acc.evidence_count(), 1);
    }

    #[test]
    fn test_signed_penalty_residuals_reduce_score() {
        let mut acc = ScoreAccumulator::<8>::new();
        acc.accumulate(&ResidualContribution {
            kind: ResidualContributionKind::GoalReward,
            contribution_id: 1,
            raw_value: 120,
        })
        .unwrap();
        acc.accumulate(&ResidualContribution {
            kind: ResidualContributionKind::ConstraintPenalty,
            contribution_id: 2,
            raw_value: -20,
        })
        .unwrap();
        acc.accumulate(&ResidualContribution {
            kind: ResidualContributionKind::UncertaintyPenalty,
            contribution_id: 3,
            raw_value: -15,
        })
        .unwrap();

        assert_eq!(acc.score(), 85);
    }

    #[test]
    fn test_score_accumulator_saturating_low_arithmetic() {
        let mut acc = ScoreAccumulator::<8>::new();
        acc.accumulate(&ResidualContribution {
            kind: ResidualContributionKind::RootPrior,
            contribution_id: 100,
            raw_value: i32::MIN + 10,
        })
        .unwrap();

        acc.accumulate(&ResidualContribution {
            kind: ResidualContributionKind::ConstraintPenalty,
            contribution_id: 101,
            raw_value: -50,
        })
        .unwrap();

        assert_eq!(acc.score(), i32::MIN); // Saturation clamp
    }

    #[test]
    fn test_deterministic_tie_breaking() {
        // Equal scores => lower candidate ID comes first
        assert_eq!(
            ScoreAccumulator::<4>::compare_candidates(1000, 5, 1000, 12),
            Ordering::Less
        );
        // Higher score comes first regardless of ID
        assert_eq!(
            ScoreAccumulator::<4>::compare_candidates(2000, 99, 1000, 1),
            Ordering::Less
        );
    }
}
