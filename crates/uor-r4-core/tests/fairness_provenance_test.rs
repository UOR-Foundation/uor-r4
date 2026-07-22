use uor_r4_core::transformerless::fairness_provenance::{
    FairnessAndProvenanceCertificate, FairnessEvaluator, ProvenanceDeletionWitness,
};

#[test]
fn test_bias_amplification_and_rare_group_erasure() {
    let teacher_counts = vec![
        ("majority_slice".to_string(), 1000),
        ("minority_slice".to_string(), 20),
    ];

    let graph_counts = vec![
        ("majority_slice".to_string(), 1020), // 1.02x amplification
        ("minority_slice".to_string(), 20),   // 100% retention
    ];

    let (bias_metrics, rare_group_retention) =
        FairnessEvaluator::evaluate_bias_and_erasure(&teacher_counts, &graph_counts);

    let witness = FairnessEvaluator::verify_provenance_deletion("doc_999", 5);

    let cert = FairnessAndProvenanceCertificate::new(
        bias_metrics,
        rare_group_retention,
        witness,
    );

    assert!(cert.bias_amplification_passed);
    assert!(cert.rare_group_erasure_passed);
    assert!(cert.verify_cid());
}

#[test]
fn test_fairness_certificate_threshold_failure() {
    let teacher_counts = vec![
        ("majority_slice".to_string(), 500),
        ("minority_slice".to_string(), 40),
    ];

    let graph_counts = vec![
        ("majority_slice".to_string(), 995), // 0.995 / 0.9259 = 1.074x amplification (> 1.05)
        ("minority_slice".to_string(), 5),    // 5/40 = 12.5% retention (< 0.95)
    ];

    let (bias_metrics, rare_group_retention) =
        FairnessEvaluator::evaluate_bias_and_erasure(&teacher_counts, &graph_counts);

    let witness = FairnessEvaluator::verify_provenance_deletion("doc_888", 2);

    let cert = FairnessAndProvenanceCertificate::new(
        bias_metrics,
        rare_group_retention,
        witness,
    );

    assert!(!cert.bias_amplification_passed, "Must fail bias amplification > 1.05");
    assert!(!cert.rare_group_erasure_passed, "Must fail rare group retention < 0.95");
    assert!(cert.verify_cid());
}

#[test]
fn test_fairness_certificate_cbor_roundtrip() {
    let witness = ProvenanceDeletionWitness {
        deleted_slice_id: "doc_123".to_string(),
        tombstoned_provenance_nodes: 3,
        graph_integrity_verified: true,
    };

    let cert = FairnessAndProvenanceCertificate::new(vec![], vec![], witness);

    let cbor_bytes = cert.to_cbor_bytes().expect("serialize CBOR");
    let decoded = FairnessAndProvenanceCertificate::from_cbor_bytes(&cbor_bytes).expect("deserialize CBOR");

    assert_eq!(cert, decoded);
    assert!(decoded.verify_cid());
}
