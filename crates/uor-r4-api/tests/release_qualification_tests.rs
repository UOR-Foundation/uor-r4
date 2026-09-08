//! Regression checks for honest release diagnostics, not alpha acceptance.
use uor_r4_api::native_capability_api::NativeModel;
use uor_r4_api::release_qualification::{AxisStatus, ReleaseQualificationSuite, ScorecardVerdict};
use uor_r4_core::native_geometric::{Config, Document, Trainer};

fn fixture_model() -> NativeModel {
    let docs = vec![Document {
        id: "release-diagnostic-fixture".into(),
        text: "hello river.\nhello forest.\nThe river flows.\n".into(),
    }];
    let mut trainer = Trainer::new(Config::default(), &docs).expect("fixture config");
    trainer.train_documents(&docs).expect("fixture training");
    let bytes = trainer
        .compile()
        .expect("fixture compile")
        .to_bytes()
        .expect("fixture bytes");
    NativeModel::load_from_bytes(&bytes).expect("explicit fixture artifact")
}

#[test]
fn release_scorecard_does_not_qualify_metadata_or_nonempty_output() {
    let model = fixture_model();
    let scorecard = ReleaseQualificationSuite::evaluate_full_scorecard(&model).expect("diagnostic");
    assert_eq!(scorecard.verdict, ScorecardVerdict::Rejected);
    assert!(!scorecard.all_axes_passed());
    assert!(scorecard.overall_score.is_none());
    assert_eq!(scorecard.evaluated_at, "NOT_RUN");
    assert_eq!(scorecard.axes.len(), 12);
    for axis in &scorecard.axes {
        assert_eq!(axis.status, AxisStatus::NotRun);
        assert!(axis.score.is_none());
        assert!(axis.floor.is_none());
    }
    assert!(scorecard.to_markdown_summary().contains("NOT_RUN"));
    let manifest = ReleaseQualificationSuite::build_release_manifest(&scorecard, "test-revision");
    assert!(!manifest.verify_integrity());
    assert!(manifest.hot_path_matmul_count.is_none());
    assert!(manifest.hot_path_float_count.is_none());
    assert!(manifest.external_api_calls.is_none());
    assert!(manifest.memory_ring_capacity_bytes.is_none());
    assert!(!manifest.is_zero_matmul_verified);
    assert!(!manifest.is_no_std_kernel_verified);
    assert!(manifest.target_triples.is_empty());
}

#[test]
fn absent_security_and_installation_evidence_cannot_pass() {
    let audit = ReleaseQualificationSuite::audit_security_and_boundaries();
    assert_eq!(audit.status, AxisStatus::NotRun);
    assert!(!audit.passed);
    assert!(!audit.path_traversal_protected);
    assert!(!audit.zero_external_network_sockets);
    assert!(!audit.zero_runtime_dynamic_code_exec);
    assert!(!audit.zero_external_provider_dependencies);
    assert!(!audit.memory_bounds_enforced);
    assert!(ReleaseQualificationSuite::verify_installation_and_rollback(&[]).is_err());
}
