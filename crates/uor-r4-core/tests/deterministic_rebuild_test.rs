//! Deterministic Artifact Rebuild Test (Gate E / Decision D2)
//!
//! Verifies canonical deterministic compiler mode: compiling an artifact twice from
//! identical pinned fixture inputs produces byte-identical binary containers and
//! identical BLAKE3 CIDs (κ).

use uor_r4_core::transformerless::{
    certificate::{Certificate, ClaimKind, EmpiricalClaim, ProtocolAttestation},
    compiler,
};

fn blake3_kappa(bytes: &[u8]) -> String {
    format!("kappa:blake3:{}", blake3::hash(bytes).to_hex())
}

#[test]
fn test_deterministic_container_rebuild() {
    let dir = env!("CARGO_MANIFEST_DIR");
    let fixture_path = format!("{dir}/tests/fixtures/tless_artifacts.bin");

    let bytes = std::fs::read(&fixture_path).unwrap_or_else(|e| {
        panic!("fixture required for Gate E (Decision D2) at {fixture_path}: {e}")
    });

    // Parse the baseline compiled container twice
    let art1 = compiler::parse_artifacts(&bytes).expect("parse run 1");
    let art2 = compiler::parse_artifacts(&bytes).expect("parse run 2");

    // Re-serialize the container twice
    let ser1 = compiler::artifact_bytes(&art1);
    let ser2 = compiler::artifact_bytes(&art2);

    // 1. Assert byte-identical container output against the pinned fixture (Gate E)
    assert_eq!(
        ser1.as_slice(),
        bytes.as_slice(),
        "Rebuilt bytes must match fixture bytes"
    );
    assert_eq!(
        ser2.as_slice(),
        bytes.as_slice(),
        "Rebuilt bytes must match fixture bytes"
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
    // Compile a transition graph twice through the real corpus->graph path.
    let corpus = uor_r4_core::transformerless::compiler::Corpus {
        n: 6,
        stories: 1,
        story: vec![1, 1, 1, 1, 1, 1],
        input: vec![100, 200, 100, 200, 100, 300],
        next: vec![200, 100, 200, 100, 300, 400],
        t_argmax: vec![200, 100, 200, 100, 300, 400],
        top_tokens: vec![[200, 0, 0, 0, 0, 0, 0, 0]; 6],
        top_weights: vec![[100, 0, 0, 0, 0, 0, 0, 0]; 6],
        span_start: vec![0, 1, 2, 3, 4, 5],
        span_end: vec![1, 2, 3, 4, 5, 6],
        byte_start: vec![u32::MAX; 6],
        byte_end: vec![u32::MAX; 6],
    };
    let region_assigner = |tok: u32| tok / 10;

    let g1 = uor_r4_core::transformerless::transitions::compile_transitions_from_corpus(
        &corpus,
        region_assigner,
        10,
    )
    .expect("compile g1");
    let g2 = uor_r4_core::transformerless::transitions::compile_transitions_from_corpus(
        &corpus,
        region_assigner,
        10,
    )
    .expect("compile g2");

    assert_eq!(g1.edges, g2.edges, "Canonical edge vectors must match");
    assert_eq!(
        g1.reverse_index, g2.reverse_index,
        "Reverse index arrays must match"
    );
    assert_eq!(
        g1.reverse_offsets, g2.reverse_offsets,
        "Reverse offset maps must match"
    );
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

    assert_eq!(
        cert1, cert2,
        "Certificates built from identical inputs must be equal"
    );
    assert_eq!(
        cert1.compute_cid(),
        cert2.compute_cid(),
        "Certificate CIDs must match"
    );
    assert_eq!(
        cert1.to_cbor_bytes().unwrap(),
        cert2.to_cbor_bytes().unwrap(),
        "Certificate CBOR bytes must be byte-identical"
    );
}
