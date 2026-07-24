//! Graph Invariant Ownership & Loader Validation Matrix
//!
//! Specification & Source: `docs/hologram_formal_analysis_direction.md` PDF §9;
//! `docs/transformerless/R4G1.md` §6; GitHub Issue #135.
//!
//! This module formalizes the ownership matrix for all 8 normative graph invariants:
//! 1. Bounded Node Degree & Active Frontier Width
//! 2. Valid Aligned Ranges & No Dangling References
//! 3. Deterministic Node/Edge Canonical Ordering
//! 4. Evidence Non-Duplication
//! 5. Provenance & Reverse-Index Completeness
//! 6. Fixed-Width Q8.8 Arithmetic & Overflow Safety
//! 7. Refinement Acyclicity
//! 8. Bounded Candidate Work & Declared Fallback Limits
//!
//! [`GraphInvariantOwnershipMatrix::validate_graph_structure`] is a
//! reference implementation covering invariants 1, 2, and 4 only; it
//! shares [`FormatError`] with the authoritative R4G1 loader path. The
//! production stage-1 (`crate::view::validate`) and stage-2
//! (`crate::stage2::validate`) validators enforce the LoaderValidation-owned
//! invariants directly against packed artifact bytes and are the actual
//! code path exercised when parsing an artifact via
//! [`crate::GraphView::parse`]; this module's function does not replace or
//! get called from that path.

use crate::error::FormatError;

/// Declared owner component for an invariant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvariantOwner {
    CompilerConstruction,
    Packer,
    LoaderValidation,
    RuntimeKernel,
    Certifier,
    PropertyTest,
    FuzzTarget,
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

    /// Validate a graph representation against a subset of the 8 normative
    /// invariants — invariant 1 (bounded node degree), invariant 2 (no
    /// dangling edge references), and invariant 4 (evidence
    /// non-duplication) — using the shared [`FormatError`] taxonomy also
    /// used by stage-1/stage-2 loader validation (no_std compatible).
    ///
    /// Node degree is derived entirely from `edges`; no caller-supplied
    /// degree value is trusted, so a malformed artifact cannot bypass the
    /// check by misreporting it. Invariants 3, 5, 6, 7, and 8 are owned
    /// and enforced elsewhere per [`Self::get_matrix`] and are not checked
    /// here.
    pub fn validate_graph_structure(
        node_count: usize,
        degree_limit: usize,
        edges: &[(u32, u32)], // (src, dst)
        evidence_ids: &[u32],
    ) -> Result<usize, FormatError> {
        // 1. Check degree limit, derived from the edge list itself.
        for node in 0..node_count as u32 {
            let degree = edges
                .iter()
                .filter(|&&(src, dst)| src == node || dst == node)
                .count();
            if degree > degree_limit {
                return Err(FormatError::NodeDegreeExceeded {
                    node,
                    degree: degree as u32,
                    limit: degree_limit as u32,
                });
            }
        }

        // 2. Check dangling references
        for (i, &(src, dst)) in edges.iter().enumerate() {
            if src as usize >= node_count || dst as usize >= node_count {
                return Err(FormatError::EdgeEndpointOutOfBounds {
                    edge: i as u32,
                    src,
                    dst,
                });
            }
        }

        // 3. Check duplicate evidence (no_std friendly O(N^2) comparison)
        for i in 0..evidence_ids.len() {
            for j in (i + 1)..evidence_ids.len() {
                if evidence_ids[i] == evidence_ids[j] {
                    return Err(FormatError::DuplicateEvidence {
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
        // 1. Degree limit failure: node 0 has degree 12 (one edge to each
        // of nodes 1..=12), which exceeds the limit of 10.
        let degree_12_edges: Vec<(u32, u32)> = (1..=12).map(|dst| (0, dst)).collect();
        let res1 = GraphInvariantOwnershipMatrix::validate_graph_structure(
            13,
            10,
            &degree_12_edges,
            &[101, 102],
        );
        assert!(matches!(
            res1.unwrap_err(),
            FormatError::NodeDegreeExceeded {
                node: 0,
                degree: 12,
                limit: 10
            }
        ));

        // 2. Dangling reference failure
        let res2 =
            GraphInvariantOwnershipMatrix::validate_graph_structure(5, 10, &[(0, 99)], &[101, 102]);
        assert!(matches!(
            res2.unwrap_err(),
            FormatError::EdgeEndpointOutOfBounds { .. }
        ));

        // 3. Duplicate evidence failure
        let res3 =
            GraphInvariantOwnershipMatrix::validate_graph_structure(5, 10, &[(0, 1)], &[101, 101]);
        assert!(matches!(
            res3.unwrap_err(),
            FormatError::DuplicateEvidence { .. }
        ));

        // 4. Clean validation
        let res4 =
            GraphInvariantOwnershipMatrix::validate_graph_structure(5, 10, &[(0, 1)], &[101, 102]);
        assert_eq!(res4.unwrap(), 5);
    }
}
