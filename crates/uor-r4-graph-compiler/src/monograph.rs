//! Hologram/R4 Formal Monograph Validator & Traceability Harness
//!
//! Specification & Source: `docs/hologram_formal_analysis_direction.md` PDF §§14–17;
//! `docs/formal_vocabulary.md`; GitHub Issue #133.
//!
//! This module provides programmatic validation for the formal monograph:
//! - Section completeness verification across all 19 monograph sections.
//! - Traceability link validation connecting implementation modules to proof matrix entries.
//! - Verification of explicit non-goals and claim-wording boundaries.

use std::fmt;

/// Errors arising during monograph validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonographValidationError {
    /// Required section missing from formal monograph.
    MissingSection { section_title: String },
    /// Implementation module traceability link broken or unreferenced.
    MissingTraceabilityLink { module_name: String },
    /// Non-goal disavowal missing from problem statement section.
    MissingNonGoalDisavowal { non_goal: String },
}

impl fmt::Display for MonographValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSection { section_title } => {
                write!(
                    f,
                    "Formal monograph missing required section: '{section_title}'"
                )
            }
            Self::MissingTraceabilityLink { module_name } => write!(
                f,
                "Traceability link missing for implementation module: '{module_name}'"
            ),
            Self::MissingNonGoalDisavowal { non_goal } => write!(
                f,
                "Monograph missing explicit non-goal disavowal for: '{non_goal}'"
            ),
        }
    }
}

impl std::error::Error for MonographValidationError {}

/// Monograph validation report metrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonographValidationReport {
    pub total_sections_verified: usize,
    pub total_modules_linked: usize,
    pub non_goals_disavowed: usize,
    pub verified: bool,
}

/// Monograph Traceability Verifier.
pub struct MonographTraceabilityVerifier;

impl MonographTraceabilityVerifier {
    /// Validate full formal monograph markdown text.
    pub fn validate_monograph_text(
        content: &str,
    ) -> Result<MonographValidationReport, MonographValidationError> {
        let required_sections = [
            "Section 1: Problem Statement and Non-Goals",
            "Section 2: Semantic State Spaces and Holographic Projections",
            "Section 3: Graph Induction & Multi-Edge Algebras",
            "Section 4: Predictive Entropy & Information Bottleneck",
            "Section 5: Unsupervised Behavioral Probes & Anti-Memorization",
            "Section 6: Future-State Optimization & Bounded Planning",
            "Section 7: Reference Intermediate Representation (IR)",
            "Section 8: Boolean / Integer Lowering & R4G1 Format",
            "Section 9: Structural Proofs & Proof Matrix",
            "Section 10: Decoupled Semantic Reasoning & Language Emission",
            "Section 11: Graph Invariant Ownership & Validation",
            "Section 12: Rate-Distortion Semantic Compression",
            "Section 13: PDF-to-Implementation Traceability",
            "Section 14: Complete Traceability & Proof Status Matrix",
            "Section 15: Issue Dependency Graph",
            "Section 16: Review Checklist vs Repos & Specifications",
            "Section 17: Known Negative Results & Disavowals",
            "Section 18: Legacy-Preserving Migration Path",
            "Section 19: Empirical Certification & Quality Gates",
        ];

        let required_modules = [
            "semantic_state.rs",
            "records.rs",
            "holographic_encoding.rs",
            "behavioral_probes.rs",
            "reference_compiler_ir.rs",
            "lower_semantic_regions.rs",
            "future_state_planner.rs",
            "structural_guarantees.rs",
            "semantic_emission_decoupling.rs",
            "invariant_ownership.rs",
            "rate_distortion_compression.rs",
            "pdf_traceability.rs",
        ];

        let required_non_goals = [
            "No Human-Level Reasoning Claim",
            "No Exact Teacher Equivalence",
            "No Floating-Point Runtime Hot Path",
        ];

        let mut sections_count = 0;
        for sec in &required_sections {
            if !content.contains(sec) {
                return Err(MonographValidationError::MissingSection {
                    section_title: sec.to_string(),
                });
            }
            sections_count += 1;
        }

        let mut modules_count = 0;
        for mod_name in &required_modules {
            if !content.contains(mod_name) {
                return Err(MonographValidationError::MissingTraceabilityLink {
                    module_name: mod_name.to_string(),
                });
            }
            modules_count += 1;
        }

        let mut non_goals_count = 0;
        for ng in &required_non_goals {
            if !content.contains(ng) {
                return Err(MonographValidationError::MissingNonGoalDisavowal {
                    non_goal: ng.to_string(),
                });
            }
            non_goals_count += 1;
        }

        Ok(MonographValidationReport {
            total_sections_verified: sections_count,
            total_modules_linked: modules_count,
            non_goals_disavowed: non_goals_count,
            verified: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monograph_validation_passes() {
        let content = include_str!("../../../docs/hologram_r4_formal_monograph.md");
        let report = MonographTraceabilityVerifier::validate_monograph_text(content).unwrap();

        assert_eq!(report.total_sections_verified, 19);
        assert_eq!(report.total_modules_linked, 12);
        assert_eq!(report.non_goals_disavowed, 3);
        assert!(report.verified);
    }
}
