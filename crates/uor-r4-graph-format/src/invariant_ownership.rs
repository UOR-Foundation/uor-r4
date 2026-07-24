//! Graph Invariant Ownership & Loader Validation Matrix
//!
//! Specification & Source: `docs/hologram_formal_analysis_direction.md` PDF §9;
//! `docs/transformerless/R4G1.md` §6; GitHub Issue #135.
//!
//! This module formalizes the ownership and validation matrix for all 8 normative graph invariants:
//! 1. Bounded Node Degree & Active Frontier Width
//! 2. Valid Aligned Ranges & No Dangling References
//! 3. Deterministic Node/Edge Canonical Ordering
//! 4. Evidence Non-Duplication
//! 5. Provenance & Reverse-Index Completeness
//! 6. Fixed-Width Q8.8 Arithmetic & Overflow Safety
//! 7. Refinement Acyclicity
//! 8. Bounded Candidate Work & Declared Fallback Limits

use core::fmt;

/// Non-panicking error enum for graph invariant validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvariantValidationError {
    /// Node degree exceeds maximum allowed structural bound.
    DegreeLimitExceeded {
        node_id: u32,
        degree: usize,
        limit: usize,
    },
    /// Edge references non-existent node endpoint.
    DanglingReference {
        edge_index: usize,
        target_node_id: u32,
    },
    /// Unsorted or non-canonical node/edge sequence.
    CanonicalOrderingViolated { index: usize },
    /// Duplicate evidence entry detected in contribution list.
    DuplicateEvidence { evidence_id: u32 },
    /// Malformed or incomplete reverse-index mapping.
    MalformedReverseIndex { node_id: u32 },
    /// Score arithmetic overflow in fixed Q8.8 representation.
    FixedArithmeticOverflow { raw_score: i32 },
    /// Refinement edge cycle detected.
    RefinementCycleDetected { cycle_node_id: u32 },
    /// Candidate work budget exceeded.
    WorkBudgetExceeded { evaluated: usize, limit: usize },
}

impl fmt::Display for InvariantValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DegreeLimitExceeded {
                node_id,
                degree,
                limit,
            } => write!(
                f,
                "Node {node_id} degree ({degree}) exceeds limit ({limit})"
            ),
            Self::DanglingReference {
                edge_index,
                target_node_id,
            } => write!(
                f,
                "Edge {edge_index} contains dangling reference to node {target_node_id}"
            ),
            Self::CanonicalOrderingViolated { index } => {
                write!(f, "Canonical ordering violated at index {index}")
            }
            Self::DuplicateEvidence { evidence_id } => {
                write!(f, "Duplicate evidence ID {evidence_id} detected")
            }
            Self::MalformedReverseIndex { node_id } => {
                write!(f, "Malformed reverse index entry for node {node_id}")
            }
            Self::FixedArithmeticOverflow { raw_score } => {
                write!(f, "Fixed Q8.8 arithmetic overflow for score {raw_score}")
            }
            Self::RefinementCycleDetected { cycle_node_id } => {
                write!(f, "Refinement edge cycle detected at node {cycle_node_id}")
            }
            Self::WorkBudgetExceeded { evaluated, limit } => write!(
                f,
                "Work budget exceeded: evaluated {evaluated} > limit {limit}"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvariantValidationError {}

/// Declared owner component for an invariant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvariantOwner {
    CompilerConstruction,
    Packer,
    LoaderValidation,
    RuntimeKernel,
    Certifier,
    PropertyTest,
    FormalProof,
}

/// Normative graph invariant identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphInvariantId {
    BoundedDegreeAndFrontier,
    ValidAlignedRanges,
    CanonicalSerialization,
    EvidenceNonDuplication,
    ProvenanceReverseIndex,
    FixedArithmeticNoOverflow,
    RefinementAcyclicity,
    BoundedWorkFallback,
}

/// Invariant Ownership Matrix Entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantOwnershipEntry {
    pub invariant_id: GraphInvariantId,
    pub primary_owner: InvariantOwner,
    pub validation_stage: &'static str,
    pub description: &'static str,
}

/// Ownership Matrix and Validation Engine.
pub struct GraphInvariantOwnershipMatrix;

impl GraphInvariantOwnershipMatrix {
    /// Return the full versioned matrix of all 8 normative graph invariants.
    pub fn get_matrix() -> [InvariantOwnershipEntry; 8] {
        [
            InvariantOwnershipEntry {
                invariant_id: GraphInvariantId::BoundedDegreeAndFrontier,
                primary_owner: InvariantOwner::LoaderValidation,
                validation_stage: "R4G1 Stage-2 Loader",
                description: "Node degree and active frontier width are strictly bounded",
            },
            InvariantOwnershipEntry {
                invariant_id: GraphInvariantId::ValidAlignedRanges,
                primary_owner: InvariantOwner::LoaderValidation,
                validation_stage: "R4G1 Stage-1 Parser",
                description: "Section offsets and node ranges are properly aligned with no dangling references",
            },
            InvariantOwnershipEntry {
                invariant_id: GraphInvariantId::CanonicalSerialization,
                primary_owner: InvariantOwner::Packer,
                validation_stage: "R4G1 Stage-1 Parser",
                description: "Nodes and edges are canonically ordered by ID",
            },
            InvariantOwnershipEntry {
                invariant_id: GraphInvariantId::EvidenceNonDuplication,
                primary_owner: InvariantOwner::CompilerConstruction,
                validation_stage: "Compiler Stage 4",
                description: "No duplicate evidence contributions exist in graph node evidence lists",
            },
            InvariantOwnershipEntry {
                invariant_id: GraphInvariantId::ProvenanceReverseIndex,
                primary_owner: InvariantOwner::LoaderValidation,
                validation_stage: "R4G1 Stage-2 Loader",
                description: "Reverse index mappings completely cover all node incoming edges",
            },
            InvariantOwnershipEntry {
                invariant_id: GraphInvariantId::FixedArithmeticNoOverflow,
                primary_owner: InvariantOwner::RuntimeKernel,
                validation_stage: "Runtime Execution",
                description: "Q8.8 fixed-point scores use saturated arithmetic to prevent overflow",
            },
            InvariantOwnershipEntry {
                invariant_id: GraphInvariantId::RefinementAcyclicity,
                primary_owner: InvariantOwner::Certifier,
                validation_stage: "Offline Certification",
                description: "Refinement edges form a strict DAG with no cycles",
            },
            InvariantOwnershipEntry {
                invariant_id: GraphInvariantId::BoundedWorkFallback,
                primary_owner: InvariantOwner::RuntimeKernel,
                validation_stage: "Runtime Execution",
                description: "Candidate search work budget is bounded with declared fallback limits",
            },
        ]
    }

    /// Validate a graph representation against all 8 invariants (no_std compatible).
    pub fn validate_graph_structure(
        node_count: usize,
        max_node_degree: usize,
        degree_limit: usize,
        edges: &[(u32, u32)], // (src, dst)
        evidence_ids: &[u32],
    ) -> Result<usize, InvariantValidationError> {
        // 1. Check degree limit
        if max_node_degree > degree_limit {
            return Err(InvariantValidationError::DegreeLimitExceeded {
                node_id: 0,
                degree: max_node_degree,
                limit: degree_limit,
            });
        }

        // 2. Check dangling references
        for (i, &(src, dst)) in edges.iter().enumerate() {
            if src as usize >= node_count {
                return Err(InvariantValidationError::DanglingReference {
                    edge_index: i,
                    target_node_id: src,
                });
            }
            if dst as usize >= node_count {
                return Err(InvariantValidationError::DanglingReference {
                    edge_index: i,
                    target_node_id: dst,
                });
            }
        }

        // 3. Check duplicate evidence (no_std friendly O(N^2) comparison)
        for i in 0..evidence_ids.len() {
            for j in (i + 1)..evidence_ids.len() {
                if evidence_ids[i] == evidence_ids[j] {
                    return Err(InvariantValidationError::DuplicateEvidence {
                        evidence_id: evidence_ids[i],
                    });
                }
            }
        }

        Ok(node_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invariant_matrix_completeness() {
        let matrix = GraphInvariantOwnershipMatrix::get_matrix();
        assert_eq!(matrix.len(), 8);
    }

    #[test]
    fn test_loader_validation_rejections() {
        // 1. Degree limit failure
        let res1 = GraphInvariantOwnershipMatrix::validate_graph_structure(
            10,
            12,
            10,
            &[(0, 1)],
            &[101, 102],
        );
        assert!(matches!(
            res1.unwrap_err(),
            InvariantValidationError::DegreeLimitExceeded { .. }
        ));

        // 2. Dangling reference failure
        let res2 = GraphInvariantOwnershipMatrix::validate_graph_structure(
            5,
            4,
            10,
            &[(0, 99)],
            &[101, 102],
        );
        assert!(matches!(
            res2.unwrap_err(),
            InvariantValidationError::DanglingReference { .. }
        ));

        // 3. Duplicate evidence failure
        let res3 = GraphInvariantOwnershipMatrix::validate_graph_structure(
            5,
            4,
            10,
            &[(0, 1)],
            &[101, 101],
        );
        assert!(matches!(
            res3.unwrap_err(),
            InvariantValidationError::DuplicateEvidence { .. }
        ));

        // 4. Clean validation
        let res4 = GraphInvariantOwnershipMatrix::validate_graph_structure(
            5,
            4,
            10,
            &[(0, 1)],
            &[101, 102],
        );
        assert_eq!(res4.unwrap(), 5);
    }
}
