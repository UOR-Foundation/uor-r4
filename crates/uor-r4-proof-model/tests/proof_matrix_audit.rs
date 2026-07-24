//! CI Audit Test for Proof Status Matrix Verification
//!
//! Specification & Source: `docs/hologram_formal_analysis_direction.md` PDF §13;
//! `docs/formal_vocabulary.md` §7; GitHub Issue #132.

use uor_r4_proof_model::proof_matrix::{ProofStatus, ProofStatusMatrix};
use uor_r4_proof_model::structural_guarantees::StructuralGuaranteeVerifier;

#[test]
fn test_ci_audit_proof_matrix_entries() {
    let matrix = ProofStatusMatrix::default();

    // Audit default theorem entries against expected status
    let report_alloc = StructuralGuaranteeVerifier::audit_proof_matrix_entry(
        &matrix,
        "Allocation Freedom",
        ProofStatus::Verified,
    )
    .expect("Allocation Freedom audit failed");
    assert!(report_alloc.verified);

    let report_bounded = StructuralGuaranteeVerifier::audit_proof_matrix_entry(
        &matrix,
        "Bounded Ranges",
        ProofStatus::Verified,
    )
    .expect("Bounded Ranges audit failed");
    assert!(report_bounded.verified);

    let report_det = StructuralGuaranteeVerifier::audit_proof_matrix_entry(
        &matrix,
        "Deterministic Serialization",
        ProofStatus::Verified,
    )
    .expect("Deterministic Serialization audit failed");
    assert!(report_det.verified);

    let report_topk = StructuralGuaranteeVerifier::audit_proof_matrix_entry(
        &matrix,
        "Deterministic Top-K Selection",
        ProofStatus::Verified,
    )
    .expect("Deterministic Top-K Selection audit failed");
    assert!(report_topk.verified);
}
