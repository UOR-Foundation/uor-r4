use uor_r4_graph_certify::certificate::{
    Certificate, ClaimKind, EmpiricalClaim, ProtocolAttestation,
};

#[test]
fn test_certificate_cid_computation_and_verification() {
    let claim = EmpiricalClaim {
        name: "held_out_accuracy".to_string(),
        sample_size: 10000,
        metric_value: 0.942,
        confidence_interval_95: (0.935, 0.949),
        slice_label: "test_set_v1".to_string(),
        claim_kind: ClaimKind::Empirical,
    };

    let attestation = ProtocolAttestation {
        deterministic_canonical_mode: true,
        zero_allocation_verified: true,
        no_multiply_verified: true,
        theorem_7_reverse_index_verified: true,
    };

    let cert = Certificate::new(
        "kappa:blake3:source_123",
        "kappa:blake3:corpus_456",
        "kappa:blake3:graph_789",
        "kappa:blake3:metric_abc",
        "kappa:blake3:op_def",
        "kappa:blake3:benchmark_ghi",
        vec![claim],
        attestation,
    );

    assert!(cert.verify_cid(), "Certificate CID verification must pass");
    assert!(
        cert.certificate_cid.starts_with("kappa:blake3:"),
        "Certificate CID must carry kappa scheme prefix"
    );
    assert!(
        cert.verify_attestation().is_ok(),
        "Structural attestation checks must pass"
    );
}

#[test]
fn test_certificate_cbor_roundtrip() {
    let claim = EmpiricalClaim {
        name: "zero_alloc_step".to_string(),
        sample_size: 50000,
        metric_value: 0.0,
        confidence_interval_95: (0.0, 0.0),
        slice_label: "census_suite".to_string(),
        claim_kind: ClaimKind::Performance,
    };

    let attestation = ProtocolAttestation {
        deterministic_canonical_mode: true,
        zero_allocation_verified: true,
        no_multiply_verified: true,
        theorem_7_reverse_index_verified: true,
    };

    let cert = Certificate::new(
        "kappa:blake3:source_test",
        "kappa:blake3:corpus_test",
        "kappa:blake3:graph_test",
        "kappa:blake3:metric_test",
        "kappa:blake3:op_test",
        "kappa:blake3:benchmark_test",
        vec![claim],
        attestation,
    );

    let cbor_bytes = cert.to_cbor_bytes().expect("serialize CBOR");
    assert!(!cbor_bytes.is_empty());

    let decoded = Certificate::from_cbor_bytes(&cbor_bytes).expect("deserialize CBOR");
    assert_eq!(cert, decoded);
    assert!(decoded.verify_cid());
}

#[test]
fn test_certificate_attestation_failure() {
    let cert = Certificate::new(
        "kappa:blake3:source",
        "kappa:blake3:corpus",
        "kappa:blake3:graph",
        "kappa:blake3:metric",
        "kappa:blake3:op",
        "kappa:blake3:benchmark",
        vec![],
        ProtocolAttestation {
            deterministic_canonical_mode: true,
            zero_allocation_verified: false, // Intentionally false!
            no_multiply_verified: true,
            theorem_7_reverse_index_verified: true,
        },
    );

    assert!(
        cert.verify_attestation().is_err(),
        "Attestation check must fail when zero_allocation_verified is false"
    );
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
