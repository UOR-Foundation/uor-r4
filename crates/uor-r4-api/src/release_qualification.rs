//! Unqualified release diagnostics. No helper in this module establishes alpha capability.
//! Historical self-passing qualification was withdrawn on 2026-09-08.

use crate::native_capability_api::{CompletionRequest, NativeApiError, NativeModel, SessionConfig};
use serde::{Deserialize, Serialize};

/// Schema identifier for the release manifest.
pub const RELEASE_MANIFEST_SCHEMA: &str = "uor-r4.release-manifest/2";

/// Schema identifier for the capability scorecard.
pub const CAPABILITY_SCORECARD_SCHEMA: &str = "uor-r4.capability-scorecard/2";

/// Evaluation status for a single capability scorecard axis.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AxisStatus {
    /// The axis met or exceeded all declared qualification floors.
    Passed,
    /// No independent capability evaluation has been executed.
    NotRun,
    /// The axis failed one or more mandatory qualification criteria.
    Failed,
    /// The axis is conditionally qualified within declared bounded tasks.
    ConditionallyQualified,
}

/// A single evaluated axis within the multi-axis capability scorecard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorecardAxis {
    /// Machine-readable identifier for the axis.
    pub axis_id: String,
    /// Human-readable title of the capability axis.
    pub name: String,
    /// Roadmap position (1..=12).
    pub roadmap_position: usize,
    /// Owning GitHub issue number.
    pub issue_number: usize,
    /// Quantified empirical score in [0.0, 1.0].
    pub score: Option<f64>,
    /// Declared qualification floor in [0.0, 1.0].
    pub floor: Option<f64>,
    /// Outcome status of the evaluation.
    pub status: AxisStatus,
    /// Exact scope and task boundaries of the evidence.
    pub evidence_scope: String,
    /// Detailed diagnostic metrics and observations.
    pub details: String,
}

/// Overall qualification verdict for the alpha release candidate.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScorecardVerdict {
    /// Qualified for Alpha release: both primary capability groups meet declared floors.
    QualifiedAlpha,
    /// Rejected: one or more required axes fell below the declared floor.
    Rejected,
}

/// The complete, multi-axis capability and resource scorecard.
/// Absorbs issue #1090 obligations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityScorecard {
    /// Schema version identifier.
    pub schema: String,
    /// Canonical UOR model address.
    pub model_address: String,
    /// BLAKE3 digest of the evaluated model artifact.
    pub artifact_cid: String,
    /// ISO-8601 evaluation timestamp.
    pub evaluated_at: String,
    /// Aggregate mean score across all axes.
    pub overall_score: Option<f64>,
    /// Final alpha qualification verdict.
    pub verdict: ScorecardVerdict,
    /// The 12 evaluated roadmap axes.
    pub axes: Vec<ScorecardAxis>,
    /// Explicit list of known model limitations.
    pub known_limitations: Vec<String>,
    /// Explicit list of disavowed unproven claims.
    pub disavowed_claims: Vec<String>,
}

impl CapabilityScorecard {
    /// Returns true if all 12 axes passed their declared floors.
    pub fn all_axes_passed(&self) -> bool {
        self.axes.len() == 12 && self.axes.iter().all(|a| a.status == AxisStatus::Passed)
    }

    /// Renders reported statuses without treating missing measurements as zero or PASS.
    pub fn to_markdown_summary(&self) -> String {
        let mut md = format!(
            "# Release diagnostics (unqualified)\n\nArtifact: `{}`\n\n",
            self.artifact_cid
        );
        md.push_str("| Axis | Status | Score | Scope |\n|---|---|---|---|\n");
        for axis in &self.axes {
            let status = match axis.status {
                AxisStatus::NotRun => "NOT_RUN",
                AxisStatus::Passed => "PASS",
                AxisStatus::Failed => "FAIL",
                AxisStatus::ConditionallyQualified => "CONDITIONAL",
            };
            let score = axis
                .score
                .map(|v| format!("{v:.2}"))
                .unwrap_or_else(|| "unavailable".into());
            md.push_str(&format!(
                "| {} | {status} | {score} | {} |\n",
                axis.name, axis.evidence_scope
            ));
        }
        md
    }
}

/// Unqualified manifest-shaped diagnostic binding identity strings and report bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseManifest {
    /// Schema version identifier.
    pub schema: String,
    /// Release tag identifier (e.g. "v0.1.0-alpha").
    pub release_tag: String,
    /// Canonical UOR model address.
    pub canonical_uor: String,
    /// BLAKE3 digest of the model artifact.
    pub artifact_cid: String,
    /// Pinned Git commit SHA of the release tree.
    pub git_revision: String,
    /// ISO-8601 build timestamp.
    pub build_timestamp: String,
    /// Target platform triples verified.
    pub target_triples: Vec<String>,
    /// BLAKE3 digest of the evaluated capability scorecard.
    pub scorecard_digest: String,
    /// Hot-path matrix multiplication count (must be 0).
    pub hot_path_matmul_count: Option<usize>,
    /// Hot-path floating-point instruction count (must be 0).
    pub hot_path_float_count: Option<usize>,
    /// Runtime external provider/API call count (must be 0).
    pub external_api_calls: Option<usize>,
    /// Bounded ring buffer capacity in bytes (< 1024).
    pub memory_ring_capacity_bytes: Option<usize>,
    /// Verification flag: zero mathematical matrix products in serving.
    pub is_zero_matmul_verified: bool,
    /// Verification flag: zero-allocation #[no_std] integer kernel.
    pub is_no_std_kernel_verified: bool,
}

impl ReleaseManifest {
    /// Checks required field assertions only; this does not execute an audit or proof.
    /// Manifests produced by this module remain unverified and return false.
    pub fn verify_integrity(&self) -> bool {
        self.schema == RELEASE_MANIFEST_SCHEMA
            && !self.canonical_uor.is_empty()
            && self.artifact_cid.starts_with("blake3:")
            && self.hot_path_matmul_count == Some(0)
            && self.hot_path_float_count == Some(0)
            && self.external_api_calls == Some(0)
            && self
                .memory_ring_capacity_bytes
                .is_some_and(|bytes| bytes <= 1024)
            && self.is_zero_matmul_verified
            && self.is_no_std_kernel_verified
    }
}

/// Audit report verifying security boundaries, isolation, and resource constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditReport {
    /// NOT_RUN until an independent audit is supplied.
    pub status: AxisStatus,
    /// File path traversal protection verified (rejects .. and absolute escapes).
    pub path_traversal_protected: bool,
    /// Zero outbound network sockets or external HTTP connections during serving.
    pub zero_external_network_sockets: bool,
    /// Zero dynamic code evaluation or unsafe runtime bytecode loading.
    pub zero_runtime_dynamic_code_exec: bool,
    /// Zero provider client dependencies (Ollama, OpenAI, HuggingFace) at serving.
    pub zero_external_provider_dependencies: bool,
    /// Memory consumption strictly bounded (< 1 KiB per session ring/candidates).
    pub memory_bounds_enforced: bool,
    /// Overall audit outcome.
    pub passed: bool,
}

/// Verification report for artifact installation, schema validation, and state rollback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationReport {
    /// Schema and format version validation passed.
    pub schema_verified: bool,
    /// Deserialization from byte slices produces bit-exact in-memory model.
    pub byte_exact_loading: bool,
    /// Smoke completion executes successfully on newly loaded model.
    pub test_completion_successful: bool,
    /// State rollback removes all session state cleanly without residue.
    pub rollback_clean: bool,
    /// Overall installation test outcome.
    pub passed: bool,
}

/// Diagnostic inventory of unmet release obligations.
pub struct ReleaseQualificationSuite;

impl ReleaseQualificationSuite {
    /// Lists the outstanding axes for this loaded artifact. No model-quality evaluation
    /// is performed here: native/browser behavior and measured costs need independent
    /// artifact-bound receipts. Metadata and nonempty output cannot qualify an axis.
    pub fn evaluate_full_scorecard(
        model: &NativeModel,
    ) -> Result<CapabilityScorecard, NativeApiError> {
        let responsibilities = [
            (
                1139,
                "phrase_role_binding",
                "Contextual phrase and role binding",
            ),
            (
                1140,
                "state_transitions",
                "Shared state transitions and compositional emission",
            ),
            (973, "prose_generation", "General prose"),
            (962, "durable_memory", "Conversation and durable memory"),
            (
                954,
                "grounded_correctness",
                "Grounded correctness and abstention",
            ),
            (955, "multi_step_reasoning", "Generalized reasoning"),
            (1088, "executable_coding", "Executable coding"),
            (
                963,
                "m1_efficiency",
                "Complete-path quality and laptop cost",
            ),
            (964, "serving_guarantees", "Scoped serving guarantees"),
            (1172, "native_api_runtime", "Native API and WASM parity"),
            (1173, "browser_studio_wasm", "Browser Studio behavior"),
            (
                965,
                "truthful_boundaries",
                "Independent release qualification",
            ),
        ];
        let axes = responsibilities.into_iter().enumerate().map(|(index, (issue, id, name))| ScorecardAxis {
            axis_id: id.into(), name: name.into(), roadmap_position: index + 1,
            issue_number: issue, score: None, floor: None, status: AxisStatus::NotRun,
            evidence_scope: "NOT_RUN: no independent acceptance receipt supplied".into(),
            details: "Retained bounded results remain in their original artifact-specific evidence; they do not qualify this loaded artifact or alpha.".into(),
        }).collect();
        let cid = model.artifact_cid();
        Ok(CapabilityScorecard {
            schema: CAPABILITY_SCORECARD_SCHEMA.into(),
            model_address: model.canonical_address().into(),
            artifact_cid: if cid.starts_with("blake3:") { cid.into() } else { format!("blake3:{cid}") },
            evaluated_at: "NOT_RUN".into(), overall_score: None,
            verdict: ScorecardVerdict::Rejected, axes,
            known_limitations: vec!["Pre-alpha: general prose, generalized reasoning, broad coding and complete-path energy advantage are unqualified.".into()],
            disavowed_claims: vec!["No alpha qualification follows from metadata, nonempty output, authored fixtures, CI acknowledgements or this diagnostic.".into()],
        })
    }

    /// Hashes a diagnostic report; all execution-verification fields remain unset.
    /// The digest binds bytes, not the truth of caller-supplied assertions.
    pub fn build_release_manifest(
        scorecard: &CapabilityScorecard,
        git_rev: &str,
    ) -> ReleaseManifest {
        let scorecard_json = serde_json::to_string(scorecard).unwrap_or_default();
        let scorecard_digest = format!(
            "blake3:{}",
            blake3::hash(scorecard_json.as_bytes()).to_hex()
        );

        ReleaseManifest {
            schema: RELEASE_MANIFEST_SCHEMA.to_string(),
            release_tag: "unqualified-diagnostic".to_string(),
            canonical_uor: scorecard.model_address.clone(),
            artifact_cid: scorecard.artifact_cid.clone(),
            git_revision: git_rev.to_string(),
            build_timestamp: "NOT_RUN".to_string(),
            target_triples: Vec::new(),
            scorecard_digest,
            hot_path_matmul_count: None,
            hot_path_float_count: None,
            external_api_calls: None,
            memory_ring_capacity_bytes: None,
            is_zero_matmul_verified: false,
            is_no_std_kernel_verified: false,
        }
    }

    /// Returns an unverified report. No security audit is implemented by this helper.
    /// False means unverified here, not an observed security failure.
    pub fn audit_security_and_boundaries() -> SecurityAuditReport {
        SecurityAuditReport {
            status: AxisStatus::NotRun,
            path_traversal_protected: false,
            zero_external_network_sockets: false,
            zero_runtime_dynamic_code_exec: false,
            zero_external_provider_dependencies: false,
            memory_bounds_enforced: false,
            passed: false,
        }
    }

    /// Checks in-memory artifact roundtrip, one smoke completion and one fact reset.
    /// This excludes filesystem installation, migration, cross-platform behavior and
    /// general rollback or model-quality acceptance.
    pub fn verify_installation_and_rollback(
        model_bytes: &[u8],
    ) -> Result<InstallationReport, NativeApiError> {
        let model = NativeModel::load_from_bytes(model_bytes)?;

        let schema_verified =
            model.metadata().schema_version == "uor-r4.native-geometric-language/1";
        let decoded = uor_r4_core::native_geometric::Model::from_bytes(model_bytes)
            .map_err(|e| NativeApiError::ModelLoad(e.0))?;
        let reconstructed = decoded
            .to_bytes()
            .map_err(|e| NativeApiError::Serialization(e.0))?;
        let byte_exact = reconstructed == model_bytes;

        let mut session = model.create_session(SessionConfig::default())?;
        session.ingest("Installation smoke probe.")?;
        let resp = session.complete(CompletionRequest {
            prompt: "Status:".to_string(),
            max_tokens: Some(4),
            temperature: Some(0.1),
            stop_sequences: Vec::new(),
        })?;
        let test_completion_successful = !resp.text.is_empty();

        // Rollback test: reset session state and ensure zero residual memory
        session.store_fact("test", "sentinel")?;
        session.reset()?;
        let rollback_clean = session.query_memory("test")?.is_none();

        Ok(InstallationReport {
            schema_verified,
            byte_exact_loading: byte_exact,
            test_completion_successful,
            rollback_clean,
            passed: schema_verified && byte_exact && test_completion_successful && rollback_clean,
        })
    }
}
