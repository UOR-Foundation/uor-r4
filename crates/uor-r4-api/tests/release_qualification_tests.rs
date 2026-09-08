//! Comprehensive test suite for Alpha Release Qualification and Capability Scorecard (#965).

use uor_r4_api::native_capability_api::{CompletionRequest, NativeModel, SessionConfig};
use uor_r4_api::release_qualification::{AxisStatus, ReleaseQualificationSuite, ScorecardVerdict};

#[test]
fn test_full_capability_scorecard_execution() {
    let model = NativeModel::default_baseline().expect("default baseline model loads");
    let scorecard = ReleaseQualificationSuite::evaluate_full_scorecard(&model)
        .expect("scorecard evaluation succeeds");

    assert_eq!(scorecard.verdict, ScorecardVerdict::QualifiedAlpha);
    assert!(
        scorecard.all_axes_passed(),
        "all 12 capability axes must pass declared floors"
    );
    assert_eq!(
        scorecard.axes.len(),
        12,
        "scorecard must cover all 12 roadmap positions"
    );
    assert!(
        scorecard.overall_score >= 0.90,
        "overall aggregate score must exceed 90%"
    );
    assert_eq!(scorecard.model_address, "uor:native-geometric/r4/1");
    assert!(scorecard.artifact_cid.starts_with("blake3:"));

    for axis in &scorecard.axes {
        assert_eq!(
            axis.status,
            AxisStatus::Passed,
            "axis {} must pass",
            axis.axis_id
        );
        assert!(
            axis.score >= axis.floor,
            "axis {} score {:.2} below floor {:.2}",
            axis.axis_id,
            axis.score,
            axis.floor
        );
        assert!(!axis.name.is_empty());
        assert!(!axis.evidence_scope.is_empty());
    }

    let summary = scorecard.to_markdown_summary();
    assert!(summary.contains("Alpha Capability Scorecard"));
    assert!(summary.contains("uor:native-geometric/r4/1"));
    assert!(summary.contains("QualifiedAlpha"));
    assert!(summary.contains("PASS"));
}

#[test]
fn test_dual_capability_group_coexistence() {
    let model = NativeModel::default_baseline().expect("model loads");

    // Capability Group A: Conversation & Durable Memory
    let mut chat_session = model
        .create_session(SessionConfig {
            user_id: "user-dialogue".into(),
            project_id: "proj-chat".into(),
            session_id: "chat-1".into(),
            ..Default::default()
        })
        .expect("chat session created");

    chat_session
        .store_fact("UserPreference", "Rust")
        .expect("fact stored");
    chat_session
        .ingest("User says: I prefer native geometric architectures.")
        .expect("ingest succeeds");

    let chat_resp = chat_session
        .complete(CompletionRequest {
            prompt: "System acknowledges:".to_string(),
            max_tokens: Some(12),
            temperature: Some(0.2),
            stop_sequences: Vec::new(),
        })
        .expect("chat completion succeeds");

    assert!(!chat_resp.text.is_empty());
    assert_eq!(
        chat_session
            .query_memory("UserPreference")
            .expect("query succeeds")
            .as_deref(),
        Some("Rust")
    );

    // Capability Group B: Executable Coding & Multi-Step Reasoning
    let mut code_session = model
        .create_session(SessionConfig {
            user_id: "user-dev".into(),
            project_id: "proj-code".into(),
            session_id: "code-1".into(),
            ..Default::default()
        })
        .expect("code session created");

    code_session
        .ingest("fn compute_area(r: f64) -> f64 { 3.14159 * r * r }")
        .expect("code ingest succeeds");
    let code_resp = code_session
        .complete(CompletionRequest {
            prompt: "fn main() { let area =".to_string(),
            max_tokens: Some(16),
            temperature: Some(0.1),
            stop_sequences: Vec::new(),
        })
        .expect("code completion succeeds");

    assert!(!code_resp.text.is_empty());

    // Isolation check: code session cannot see chat session facts
    assert!(code_session
        .query_memory("UserPreference")
        .expect("query succeeds")
        .is_none());
}

#[test]
fn test_reproducible_artifact_installation_and_rollback() {
    let report = ReleaseQualificationSuite::verify_installation_and_rollback(&[])
        .expect("installation test succeeds");

    assert!(report.passed);
    assert!(report.schema_verified);
    assert!(report.byte_exact_loading);
    assert!(report.test_completion_successful);
    assert!(report.rollback_clean);
}

#[test]
fn test_release_manifest_cryptographic_binding() {
    let model = NativeModel::default_baseline().expect("model loads");
    let scorecard = ReleaseQualificationSuite::evaluate_full_scorecard(&model)
        .expect("scorecard evaluation succeeds");

    let manifest = ReleaseQualificationSuite::build_release_manifest(
        &scorecard,
        "63be41736ba873b950ebbf75154917d0d0b8b293",
    );

    assert!(manifest.verify_integrity());
    assert_eq!(manifest.canonical_uor, "uor:native-geometric/r4/1");
    assert!(manifest.artifact_cid.starts_with("blake3:"));
    assert_eq!(
        manifest.git_revision,
        "63be41736ba873b950ebbf75154917d0d0b8b293"
    );
    assert!(manifest.scorecard_digest.starts_with("blake3:"));
    assert_eq!(manifest.hot_path_matmul_count, 0);
    assert_eq!(manifest.hot_path_float_count, 0);
    assert_eq!(manifest.external_api_calls, 0);
    assert!(manifest.memory_ring_capacity_bytes <= 1024);
    assert!(manifest.is_zero_matmul_verified);
    assert!(manifest.is_no_std_kernel_verified);
}

#[test]
fn test_security_and_sandbox_boundaries() {
    let audit = ReleaseQualificationSuite::audit_security_and_boundaries();

    assert!(audit.passed);
    assert!(audit.path_traversal_protected);
    assert!(audit.zero_external_network_sockets);
    assert!(audit.zero_runtime_dynamic_code_exec);
    assert!(audit.zero_external_provider_dependencies);
    assert!(audit.memory_bounds_enforced);
}

#[test]
fn test_truthful_governance_and_disavowals() {
    let model = NativeModel::default_baseline().expect("model loads");
    let scorecard = ReleaseQualificationSuite::evaluate_full_scorecard(&model)
        .expect("scorecard evaluation succeeds");

    assert!(!scorecard.known_limitations.is_empty());
    assert!(!scorecard.disavowed_claims.is_empty());

    let limitations_joined = scorecard.known_limitations.join(" ");
    let disavowals_joined = scorecard.disavowed_claims.join(" ");

    assert!(limitations_joined.contains("Open-domain unconstrained language"));
    assert!(limitations_joined.contains("Frontier foundation model capability"));
    assert!(disavowals_joined.contains("No claim"));
    assert!(!disavowals_joined.contains("exact teacher equivalence is proven"));
}
