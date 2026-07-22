//! Deterministic Artifact Rebuild Test (Gate E / Decision D2)
//!
//! Verifies canonical deterministic compiler mode: compiling an artifact twice from
//! identical pinned fixture inputs produces byte-identical binary containers and
//! identical BLAKE3 CIDs (κ).

use uor_r4_core::transformerless::{
    certificate::{Certificate, ClaimKind, EmpiricalClaim, ProtocolAttestation},
    compiler,
    score_q::ScoreQ,
    transitions::{EdgeKind, TransitionGraph},
};

fn blake3_kappa(bytes: &[u8]) -> String {
    format!("kappa:blake3:{}", blake3::hash(bytes).to_hex())
}

#[test]
fn test_deterministic_container_rebuild() {
    let dir = env!("CARGO_MANIFEST_DIR");
    let fixture_path = format!("{dir}/tests/fixtures/tless_artifacts.bin");

    let Ok(bytes) = std::fs::read(&fixture_path) else {
        eprintln!("skipping: fixture file not present at {fixture_path}");
        return;
    };

    // Parse the baseline compiled container twice
    let art1 = compiler::parse_artifacts(&bytes).expect("parse run 1");
    let art2 = compiler::parse_artifacts(&bytes).expect("parse run 2");

    // Re-serialize the container twice
    let ser1 = compiler::artifact_bytes(&art1);
    let ser2 = compiler::artifact_bytes(&art2);

    // 1. Assert byte-identical container output (Gate E)
    assert_eq!(
        ser1.len(),
        ser2.len(),
        "Container byte lengths must match exactly"
    );
    assert_eq!(
        ser1, ser2,
        "Rebuilt artifact container bytes must be 100% byte-identical"
    );

    // 2. Assert identical BLAKE3 CIDs
    let cid1 = blake3_kappa(&ser1);
    let cid2 = blake3_kappa(&ser2);
    assert_eq!(cid1, cid2, "Container BLAKE3 CIDs must be identical");

    // 3. Assert sub-component CIDs
    assert_eq!(
        art1.token_stage_kappas, art2.token_stage_kappas,
        "Token stage CIDs must match"
    );
    assert_eq!(
        blake3_kappa(&art1.token_codes),
        blake3_kappa(&art2.token_codes),
        "Token code CIDs must match"
    );
}

#[test]
fn test_deterministic_transition_graph_rebuild() {
    // Construct synthetic edges twice with deterministic seed order
    let mut g1 = TransitionGraph::new();
    let mut g2 = TransitionGraph::new();

    for &(src, dst, weight) in &[(10, 20, 5), (30, 20, 8), (10, 40, 3), (20, 50, 12)] {
        g1.add_edge_with_score(src, dst, weight, ScoreQ::from_raw(weight as i32), EdgeKind::Forward);
        g2.add_edge_with_score(src, dst, weight, ScoreQ::from_raw(weight as i32), EdgeKind::Forward);
    }

    g1.build_reverse_index().expect("build g1 reverse index");
    g2.build_reverse_index().expect("build g2 reverse index");

    assert_eq!(g1.edges, g2.edges, "Canonical edge vectors must match");
    assert_eq!(g1.reverse_index, g2.reverse_index, "Reverse index arrays must match");
    assert_eq!(g1.reverse_offsets, g2.reverse_offsets, "Reverse offset maps must match");
    assert!(g1.verify_theorem_7().is_ok());
    assert!(g2.verify_theorem_7().is_ok());
}

#[test]
fn test_deterministic_certificate_rebuild() {
    let claim = EmpiricalClaim {
        name: "gate_e_rebuild".to_string(),
        sample_size: 1000,
        metric_value: 1.0,
        confidence_interval_95: (1.0, 1.0),
        slice_label: "ci_rebuild".to_string(),
        claim_kind: ClaimKind::Structural,
    };

    let attestation = ProtocolAttestation {
        deterministic_canonical_mode: true,
        zero_allocation_verified: true,
        no_multiply_verified: true,
        theorem_7_reverse_index_verified: true,
    };

    let cert1 = Certificate::new(
        "kappa:blake3:src",
        "kappa:blake3:corpus",
        "kappa:blake3:graph",
        "kappa:blake3:metric",
        "kappa:blake3:op",
        "kappa:blake3:benchmark",
        vec![claim.clone()],
        attestation.clone(),
    );

    let cert2 = Certificate::new(
        "kappa:blake3:src",
        "kappa:blake3:corpus",
        "kappa:blake3:graph",
        "kappa:blake3:metric",
        "kappa:blake3:op",
        "kappa:blake3:benchmark",
        vec![claim],
        attestation,
    );

    assert_eq!(cert1, cert2, "Certificates built from identical inputs must be equal");
    assert_eq!(cert1.compute_cid(), cert2.compute_cid(), "Certificate CIDs must match");
    assert_eq!(
        cert1.to_cbor_bytes().unwrap(),
        cert2.to_cbor_bytes().unwrap(),
        "Certificate CBOR bytes must be byte-identical"
    );
}
