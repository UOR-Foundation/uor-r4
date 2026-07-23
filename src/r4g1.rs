//! Native R4G1 graph-runtime adapter used by the HTTP server.
//!
//! The graph scorer is intentionally kept separate from the exploratory f64
//! router. It derives the packed input signature with the transformerless
//! artifact, then selects a token from the validated R4G1 graph.
//!
//! # ResolutionStatus-driven behavior (issue #78, decision D4)
//!
//! Every prediction carries a resolution status and the deployed behavior is
//! declared as data in [`StatusPolicy`] (the D4 manifest policy):
//!
//! | status | default action |
//! |---|---|
//! | `exact_context` | [`StatusAction::Serve`] |
//! | `graph` | [`StatusAction::Serve`] |
//! | `novel` | [`StatusAction::WidenOnce`] (then abstain) |
//! | `contradictory` | [`StatusAction::Abstain`] (reserved; the scorer does not produce it yet) |
//!
//! `WidenOnce` retries the prediction with the per-depth membership set
//! widened to [`WIDENED_TOP_M`] exactly once; a signature still Novel after
//! widening is remembered in a bounded FIFO so identical probes abstain
//! without widening again (threat model: fallback denial-of-service).
//! Abstention is a typed outcome — no token is emitted, none is guessed,
//! and the server surfaces the status. An optional override is read from
//! the graph's `score_report.json` (`config.status_policy`, e.g.
//! `{"novel": "abstain"}` with values `serve` / `widen_once` / `abstain`);
//! absent or invalid rows keep the defaults.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use uor_r4_core::transformerless::compiler::{self, Compiled, SIG_BYTES, STAGES, WINDOW};
use uor_r4_core::transformerless::runtime;
use uor_r4_core::transformerless::scenarios::Tokenizer;
use uor_r4_graph_certify::{
    GraphScorer, ScoreStatus, StepState, DEFAULT_EXCT_TOP_X, DEFAULT_ROOT_TOP_B, TOP_M,
    WIDENED_TOP_M,
};

// BEGIN DEPLOYED STATUS POLICY (INTEGER-ONLY) -------------------------
// The D4 manifest policy and the status-aware prediction path below are
// part of the deployed integer contract: no float, no multiply/divide/
// modulo in value arithmetic, and no per-prediction allocation in
// steady state (one-time buffers are built in `load`). The
// status-policy test suite machine-checks this delimited block by
// source scan (the P-4 pattern) and censuses the prediction calls.

/// The adapter-level status space of the D4 manifest policy: the
/// scorer's three [`ScoreStatus`] outcomes plus the reserved
/// `Contradictory` (glossary "Resolution status"). The scorer does not
/// produce `Contradictory` yet; the policy arm is declared and enforced
/// now so the deployed behavior is total when it lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyStatus {
    /// Exact-context evidence resolved (Rule 2).
    ExactContext,
    /// Graph residuals resolved (Rule 1 with a covered chain).
    Graph,
    /// No calibrated region covers the input.
    Novel,
    /// Active regions materially disagree (reserved).
    Contradictory,
}

impl From<ScoreStatus> for PolicyStatus {
    fn from(status: ScoreStatus) -> Self {
        match status {
            ScoreStatus::ExactContext => PolicyStatus::ExactContext,
            ScoreStatus::Graph => PolicyStatus::Graph,
            ScoreStatus::Novel => PolicyStatus::Novel,
        }
    }
}

impl PolicyStatus {
    /// The wire label used in server JSON responses.
    pub fn label(self) -> &'static str {
        match self {
            PolicyStatus::ExactContext => "exact_context",
            PolicyStatus::Graph => "graph",
            PolicyStatus::Novel => "novel",
            PolicyStatus::Contradictory => "contradictory",
        }
    }
}

/// One row of the manifest policy: what the deployed adapter does with
/// a prediction of this status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusAction {
    /// Emit the selected token.
    Serve,
    /// Retry once with the membership set widened to [`WIDENED_TOP_M`];
    /// if the retry does not resolve to a served status, abstain. A
    /// signature confirmed Novel after widening is remembered (bounded)
    /// so identical probes abstain without widening again.
    WidenOnce,
    /// Emit no token; the caller surfaces the status.
    Abstain,
}

/// The D4 manifest policy as data — one action per status. The defaults
/// implement the plan's D4 recommendation (exact-residual evidence,
/// then abstain): ExactContext to Serve, Graph to Serve, Novel to
/// WidenOnce (then Abstain), Contradictory to Abstain.
///
/// Optional override: the `status_policy` key inside `config` of the
/// graph's `score_report.json`, for example
/// `{"exact_context": "serve", "graph": "serve", "novel": "widen_once",
/// "contradictory": "abstain"}`. Missing keys and unknown values keep
/// the default for that row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusPolicy {
    pub exact_context: StatusAction,
    pub graph: StatusAction,
    pub novel: StatusAction,
    pub contradictory: StatusAction,
}

impl Default for StatusPolicy {
    fn default() -> Self {
        Self {
            exact_context: StatusAction::Serve,
            graph: StatusAction::Serve,
            novel: StatusAction::WidenOnce,
            contradictory: StatusAction::Abstain,
        }
    }
}

impl StatusPolicy {
    /// The action declared for `status`.
    pub fn action(&self, status: PolicyStatus) -> StatusAction {
        match status {
            PolicyStatus::ExactContext => self.exact_context,
            PolicyStatus::Graph => self.graph,
            PolicyStatus::Novel => self.novel,
            PolicyStatus::Contradictory => self.contradictory,
        }
    }

    /// Read the optional `status_policy` override from the score
    /// report's `config` section; absent or invalid rows fall back to
    /// the D4 defaults.
    pub fn from_report(report: Option<&serde_json::Value>) -> Self {
        let defaults = Self::default();
        let overrides = report
            .and_then(|r| r.get("config"))
            .and_then(|c| c.get("status_policy"));
        let parse = |key: &str, default: StatusAction| {
            overrides
                .and_then(|o| o.get(key))
                .and_then(serde_json::Value::as_str)
                .and_then(parse_action)
                .unwrap_or(default)
        };
        Self {
            exact_context: parse("exact_context", defaults.exact_context),
            graph: parse("graph", defaults.graph),
            novel: parse("novel", defaults.novel),
            contradictory: parse("contradictory", defaults.contradictory),
        }
    }
}

/// Parse one override value (`serve`, `widen_once`, `abstain`).
fn parse_action(value: &str) -> Option<StatusAction> {
    match value {
        "serve" => Some(StatusAction::Serve),
        "widen_once" => Some(StatusAction::WidenOnce),
        "abstain" => Some(StatusAction::Abstain),
        _ => None,
    }
}

/// Observable counters of the status-aware path: the widen-once bound
/// and the abstain/serve rates, asserted by the probe suite and
/// reportable by the server.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PolicyCounters {
    /// Status-aware predictions run.
    pub predicts: u64,
    /// Predictions served a token.
    pub serves: u64,
    /// Predictions abstained.
    pub abstains: u64,
    /// Widened re-probes run (at most one per distinct Novel
    /// signature; the bounded memory answers the rest).
    pub widen_attempts: u64,
    /// Novel signatures answered from the widen-once memory instead of
    /// re-widening.
    pub widen_skipped_seen: u64,
}

/// A served prediction: the selected token, the status that resolved,
/// and whether a widened re-probe ran for this prediction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PredictOutcome {
    pub token: u32,
    pub status: ScoreStatus,
    pub widened: bool,
}

/// A typed abstention: no token was emitted and none is guessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbstainOutcome {
    pub status: ScoreStatus,
    pub widened: bool,
}

/// The status-aware prediction result of the deployed adapter: either
/// the policy serves a token or it abstains with the status recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredictDecision {
    Serve(PredictOutcome),
    Abstain(AbstainOutcome),
}

/// The result of a status-aware generation run: tokens written, the
/// final step's status, whether any step widened, and whether the run
/// stopped on an abstention (no guessed token was emitted).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenerateStatus {
    pub count: usize,
    pub status: Option<ScoreStatus>,
    pub widened: bool,
    pub abstained: bool,
}

/// Bound of the widen-once memory: confirmed-Novel signatures whose
/// widening would deterministically re-resolve Novel (scoring is a pure
/// function of the loaded artifact, so the memory is sound for the
/// lifetime of this state; a recompile replaces the whole state).
const NOVEL_SEEN_CAPACITY: usize = 1024;

/// Fixed-capacity FIFO of confirmed-Novel signatures (threat model:
/// repeated adversarial out-of-distribution probes abstain after one
/// membership scan instead of forcing constant widening).
struct NovelSeen {
    sigs: Vec<[u8; SIG_BYTES]>,
    next: usize,
}

impl NovelSeen {
    fn new(capacity: usize) -> Self {
        Self {
            sigs: Vec::with_capacity(capacity),
            next: 0,
        }
    }
    fn contains(&self, sig: &[u8; SIG_BYTES]) -> bool {
        self.sigs.iter().any(|entry| entry == sig)
    }
    fn insert(&mut self, sig: &[u8; SIG_BYTES]) {
        if self.sigs.capacity() == 0 {
            return;
        }
        if self.sigs.len() < self.sigs.capacity() {
            self.sigs.push(*sig);
        } else {
            self.sigs[self.next] = *sig;
        }
        self.next += 1;
        if self.next == self.sigs.capacity() {
            self.next = 0;
        }
    }
}

/// One scored probe: the selection and its status at the given
/// membership width.
struct ScoredProbe {
    token: u32,
    status: ScoreStatus,
}

/// A loaded, CID-verified scored graph and the teacher artifact needed to
/// derive input signatures from token ids.
pub struct R4g1State {
    artifacts: Compiled,
    scorer: GraphScorer,
    rotations: [usize; WINDOW + 1],
    tokenizer: Option<Tokenizer>,
    /// Number of token rows in the teacher artifact: the exclusive upper
    /// bound of decodable token ids (checked at the prediction boundary
    /// so an out-of-vocabulary window is a typed error, not a panic).
    token_rows: u32,
    /// The D4 manifest policy in force (defaults or report override).
    policy: StatusPolicy,
    /// Fixed-capacity scoring scratch, allocated once in `load`.
    step: RefCell<StepState>,
    /// False only for legacy TLS1 exact-context artifacts, which stay on
    /// the reference scorer (no widening there).
    step_supported: bool,
    counters: RefCell<PolicyCounters>,
    novel_seen: RefCell<NovelSeen>,
}

impl R4g1State {
    /// The manifest policy in force (D4 defaults or the score-report
    /// override).
    pub fn policy(&self) -> StatusPolicy {
        self.policy
    }

    /// A snapshot of the status-path counters.
    pub fn policy_counters(&self) -> PolicyCounters {
        *self.counters.borrow()
    }

    /// Update one counter group.
    fn bump(&self, update: impl Fn(&mut PolicyCounters)) {
        update(&mut self.counters.borrow_mut());
    }

    /// Derive the packed input signature of a token window.
    fn derive_sig(&self, window: &[u32]) -> [u8; SIG_BYTES] {
        let bundle = runtime::bundle_window_plain(&self.artifacts, &self.rotations, window);
        runtime::sig_plain(&self.artifacts, &bundle)
    }

    /// Reject a window carrying a token id the teacher artifact cannot
    /// decode (boundary check: the decode below indexes by token id).
    fn check_window(&self, window: &[u32]) -> Result<(), String> {
        if window.iter().any(|&t| t >= self.token_rows) {
            return Err(format!(
                "token id outside the teacher vocabulary ({})",
                self.token_rows
            ));
        }
        Ok(())
    }

    /// Score one signature at the given membership width. Artifacts
    /// with legacy TLS1 exact-context evidence stay on the reference
    /// scorer (the deployed step requires residualized RX1 evidence);
    /// that path ignores the width — widening is unavailable there.
    fn score_sig(
        &self,
        sig: &[u8; SIG_BYTES],
        top_m: usize,
        recent_tokens: &[u32],
    ) -> Result<ScoredProbe, String> {
        if self.step_supported {
            let outcome = self.scorer.score_step_with_recent(
                sig,
                top_m,
                &mut self.step.borrow_mut(),
                recent_tokens,
            )?;
            Ok(ScoredProbe {
                token: outcome.selected,
                status: outcome.status,
            })
        } else {
            let outcome = self.scorer.score_candidates(sig, recent_tokens)?;
            Ok(ScoredProbe {
                token: outcome.selected,
                status: outcome.witness.status,
            })
        }
    }

    /// The D4 policy decision for one input signature: score at the
    /// manifest membership width, then Serve / WidenOnce / Abstain per
    /// the declared policy. WidenOnce re-probes once at
    /// [`WIDENED_TOP_M`]; a signature still Novel after widening is
    /// remembered so identical probes abstain without widening again.
    pub fn predict_signature_status(
        &self,
        sig: &[u8; SIG_BYTES],
    ) -> Result<PredictDecision, String> {
        self.predict_signature_status_with_recent(sig, &[])
    }

    fn predict_signature_status_with_recent(
        &self,
        sig: &[u8; SIG_BYTES],
        recent_tokens: &[u32],
    ) -> Result<PredictDecision, String> {
        self.bump(|c| c.predicts += 1);
        let first = self.score_sig(sig, TOP_M, recent_tokens)?;
        match self.policy.action(first.status.into()) {
            StatusAction::Serve => {
                self.bump(|c| c.serves += 1);
                Ok(PredictDecision::Serve(PredictOutcome {
                    token: first.token,
                    status: first.status,
                    widened: false,
                }))
            }
            StatusAction::Abstain => {
                self.bump(|c| c.abstains += 1);
                Ok(PredictDecision::Abstain(AbstainOutcome {
                    status: first.status,
                    widened: false,
                }))
            }
            StatusAction::WidenOnce => {
                if !self.step_supported {
                    // No widening on the legacy reference path: abstain
                    // directly (documented degrade for TLS1 artifacts).
                    self.bump(|c| c.abstains += 1);
                    return Ok(PredictDecision::Abstain(AbstainOutcome {
                        status: first.status,
                        widened: false,
                    }));
                }
                if self.novel_seen.borrow().contains(sig) {
                    self.bump(|c| {
                        c.widen_skipped_seen += 1;
                        c.abstains += 1;
                    });
                    return Ok(PredictDecision::Abstain(AbstainOutcome {
                        status: first.status,
                        widened: false,
                    }));
                }
                self.bump(|c| c.widen_attempts += 1);
                let second = self.score_sig(sig, WIDENED_TOP_M, recent_tokens)?;
                if second.status == ScoreStatus::Novel {
                    self.novel_seen.borrow_mut().insert(sig);
                }
                if self.policy.action(second.status.into()) == StatusAction::Serve {
                    self.bump(|c| c.serves += 1);
                    Ok(PredictDecision::Serve(PredictOutcome {
                        token: second.token,
                        status: second.status,
                        widened: true,
                    }))
                } else {
                    self.bump(|c| c.abstains += 1);
                    Ok(PredictDecision::Abstain(AbstainOutcome {
                        status: second.status,
                        widened: true,
                    }))
                }
            }
        }
    }

    /// Score one token window through the D4 policy.
    pub fn predict_window_status(&self, window: &[u32]) -> Result<PredictDecision, String> {
        self.check_window(window)?;
        self.predict_signature_status_with_recent(&self.derive_sig(window), window)
    }

    /// Generate a greedy continuation with per-step policy decisions:
    /// stops at the first abstention (returning the count so far and
    /// the abstaining status) and never emits a guessed token.
    pub fn generate_into_status(
        &self,
        seed: &[u32],
        out: &mut [u32],
    ) -> Result<GenerateStatus, String> {
        self.check_window(seed)?;
        let mut window = [0u32; WINDOW];
        let seed = &seed[seed.len().saturating_sub(WINDOW)..];
        let mut window_len = seed.len();
        window[..window_len].copy_from_slice(seed);

        let mut last_status = None;
        let mut widened = false;
        for (generated, token) in out.iter_mut().enumerate() {
            match self.predict_window_status(&window[..window_len])? {
                PredictDecision::Serve(outcome) => {
                    let next = outcome.token;
                    last_status = Some(outcome.status);
                    widened = widened || outcome.widened;
                    *token = next;
                    if next == 1 || next == 2 {
                        return Ok(GenerateStatus {
                            count: generated,
                            status: last_status,
                            widened,
                            abstained: false,
                        });
                    }
                    if window_len < WINDOW {
                        window[window_len] = next;
                        window_len += 1;
                    } else {
                        window.copy_within(1.., 0);
                        window[WINDOW - 1] = next;
                    }
                }
                PredictDecision::Abstain(outcome) => {
                    return Ok(GenerateStatus {
                        count: generated,
                        status: Some(outcome.status),
                        widened: widened || outcome.widened,
                        abstained: true,
                    });
                }
            }
        }
        Ok(GenerateStatus {
            count: out.len(),
            status: last_status,
            widened,
            abstained: false,
        })
    }
}

// END DEPLOYED STATUS POLICY (INTEGER-ONLY) ---------------------------

impl R4g1State {
    /// Load and validate a scored graph. The teacher artifact supplies the
    /// compressed token rows used to derive input signatures. EXCT is not
    /// enabled because its reference implementation performs probe-time
    /// floating-point quantization.
    pub fn load(graph_path: &Path, teacher_path: &Path) -> Result<Self, String> {
        let graph_bytes = std::fs::read(graph_path)
            .map_err(|error| format!("{}: {error}", graph_path.display()))?;
        let teacher_bytes = std::fs::read(teacher_path)
            .map_err(|error| format!("{}: {error}", teacher_path.display()))?;
        let artifacts = compiler::parse_artifacts(&teacher_bytes).ok_or_else(|| {
            format!(
                "{}: not a TLA3/TLA4/TLA5 teacher artifact",
                teacher_path.display()
            )
        })?;
        let score_report = graph_path
            .parent()
            .and_then(|parent| std::fs::read(parent.join("score_report.json")).ok())
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
        let root_top_b = score_report
            .as_ref()
            .and_then(|report| report.pointer("/config/root_top_b"))
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .filter(|&value| value > 0)
            .unwrap_or(DEFAULT_ROOT_TOP_B);
        let exct_top_x = score_report
            .as_ref()
            .and_then(|report| report.pointer("/config/exct_top_x"))
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .filter(|&value| value > 0)
            .unwrap_or(DEFAULT_EXCT_TOP_X);
        // The compiled RX1 EXCT table contains integer residuals. Supplying
        // the teacher artifact here is only for integer class-code lookup;
        // no probe-time log quantization occurs in the deployed path.
        let scorer =
            GraphScorer::from_artifact(&graph_bytes, Some(&teacher_bytes), root_top_b, exct_top_x)
                .map_err(|error| format!("{}: {error}", graph_path.display()))?;
        if let Some(report) = score_report.as_ref() {
            validate_quality_report(report)?;
        }
        let tokenizer = teacher_path
            .parent()
            .map(|parent| parent.join("tokenizer.bin"))
            .filter(|path| path.is_file())
            .and_then(|path| Tokenizer::try_load(path).ok());

        let policy = StatusPolicy::from_report(score_report.as_ref());
        let step_supported = !scorer.has_legacy_exct();
        let step = scorer
            .step_state(WIDENED_TOP_M)
            .map_err(|error| format!("{}: {error}", graph_path.display()))?;
        let token_rows = u32::try_from(artifacts.token_codes.len() / STAGES)
            .map_err(|_| format!("{}: teacher token table too large", teacher_path.display()))?;

        Ok(Self {
            artifacts,
            scorer,
            rotations: compiler::derive_rotations(),
            tokenizer,
            token_rows,
            policy,
            step: RefCell::new(step),
            step_supported,
            counters: RefCell::new(PolicyCounters::default()),
            novel_seen: RefCell::new(NovelSeen::new(NOVEL_SEEN_CAPACITY)),
        })
    }

    /// Encode with the bundle-matched tokenizer when one is available.
    pub fn encode_into(&self, text: &str, out: &mut [u32]) -> Option<usize> {
        self.tokenizer.as_ref()?.encode_into(text, out).ok()
    }

    /// Decode with the bundle-matched tokenizer when one is available.
    pub fn decode_into(&self, tokens: &[u32], out: &mut [u8]) -> Option<usize> {
        self.tokenizer.as_ref()?.decode_into(tokens, out).ok()
    }

    /// Score one token window using the validated graph artifact.
    ///
    /// Delegates to [`Self::predict_window_status`] and discards the
    /// status: served predictions return their token; a policy
    /// abstention is an error here — no guessed token is emitted.
    pub fn predict_window(&self, window: &[u32]) -> Result<u32, String> {
        match self.predict_window_status(window)? {
            PredictDecision::Serve(outcome) => Ok(outcome.token),
            PredictDecision::Abstain(outcome) => Err(format!(
                "R4G1 policy abstained (status: {})",
                PolicyStatus::from(outcome.status).label()
            )),
        }
    }

    /// Generate a greedy continuation from a token seed. This mirrors the
    /// legacy runtime's fixed-width window behavior while replacing its
    /// graded-store lookup with R4G1 graph scoring. Delegates to
    /// [`Self::generate_into_status`] and discards the status fields; on
    /// a policy abstention the tokens generated so far are returned.
    pub fn generate_into(&self, seed: &[u32], out: &mut [u32]) -> Result<usize, String> {
        Ok(self.generate_into_status(seed, out)?.count)
    }
}

/// Pinned quality floor (issue #110, era: #65-chain anchors). The deployed
/// graph must not digress from the Rule 1+2 anchors the quality chain
/// measured (31.7086% top-1, 9.8612 bits/token) beyond the margins the CI
/// trend alarm allows. Keep these constants in sync with
/// `scripts/check_gate_c_regression.py`; when a compiler redesign
/// legitimately moves the anchors, update both sites in the same commit with
/// an era note.
const QUALITY_FLOOR_TOP1_AGREEMENT: f64 = 0.317 - 0.02;
const QUALITY_FLOOR_BITS_PER_TOKEN: f64 = 9.86 + 0.10;

/// Validate the graph's Rule 1+2 quality against the TLA baseline and the
/// pinned absolute floor. Missing metrics remain compatible with older
/// reports; when present, the deployed graph must not be worse than the
/// baseline and must not digress below the #65-chain anchors.
pub fn validate_quality_report(report: &serde_json::Value) -> Result<(), String> {
    let graph_agreement = report
        .pointer("/gate_c/rule12_precedence/top1_agreement")
        .and_then(serde_json::Value::as_f64);
    let graph_bits = report
        .pointer("/gate_c/rule12_precedence/bits_per_token")
        .and_then(serde_json::Value::as_f64);
    let baseline_agreement = report
        .pointer("/gate_c/tla3_baseline/top1_agreement")
        .and_then(serde_json::Value::as_f64);
    if let (Some(graph), Some(baseline)) = (graph_agreement, baseline_agreement) {
        if graph < baseline {
            return Err(format!(
                "R4G1 quality gate failed: graph runtime top-1 {:.2}% is below TLA baseline {:.2}%",
                graph * 100.0,
                baseline * 100.0
            ));
        }
    }
    if let Some(graph) = graph_agreement {
        if graph < QUALITY_FLOOR_TOP1_AGREEMENT {
            return Err(format!(
                "R4G1 quality gate failed: graph runtime top-1 {:.2}% digresses below the pinned floor {:.2}%",
                graph * 100.0,
                QUALITY_FLOOR_TOP1_AGREEMENT * 100.0
            ));
        }
    }
    if let Some(bits) = graph_bits {
        if bits > QUALITY_FLOOR_BITS_PER_TOKEN {
            return Err(format!(
                "R4G1 quality gate failed: graph runtime {:.4} bits/token digresses above the pinned ceiling {:.4}",
                bits, QUALITY_FLOOR_BITS_PER_TOKEN
            ));
        }
    }
    Ok(())
}

/// Resolve the graph path from an explicit setting or the conventional
/// compiled-bundle location beside `tless_artifacts.bin`.
pub fn discover_path(explicit: Option<&str>, teacher_path: &Path) -> Option<PathBuf> {
    explicit.map(PathBuf::from).or_else(|| {
        teacher_path
            .parent()
            .map(|parent| parent.join("graph/score.r4g1"))
    })
}
