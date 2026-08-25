//! Serving-surface held-out evaluation — the certify C row (issue #280).
//!
//! Measures the configuration that actually serves: token-free D4 policy
//! resolution composed with [`uor_r4_graph_runtime::R4G1Runtime`] candidate
//! selection, fed the same token windows as the HTTP server. The evaluator is
//! teacher-free: it consumes recorded `t_argmax` labels and the same-position
//! plain TLA comparator.
//!
//! The previous C row measured the `convert_r4g1` certify scaffold,
//! which was never a functional prediction path (issue #280 diagnosis:
//! no per-node emissions, kind=2 transition walks structurally absent,
//! stride-8 EXCT decode over a variable-stride container). That row is
//! retired; the recorded 0.0% stands in issue #280 as the reason.
//!
//! Discipline carried over from the retired row (#279/#282/#232):
//! deterministic ascending-prefix sample, wall-clock budgets that turn silent
//! stalls into recorded skips, and a readiness probe extended with an
//! accuracy spot-check — scored + non-constant demonstrably does not
//! imply functional, so the probe now requires at least one served
//! prediction that matches the recorded corpus/teacher continuation
//! before the full sample is spent.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde::Serialize;
use uor_r4_core::transformerless::compiler;
use uor_r4_core::transformerless::runtime;
use uor_r4_graph_compiler::induction;
use uor_r4_model_source::SourceUnavailable;

use crate::deployed_quality::{
    deployed_quality_positions_cid, derive_deployed_quality_bindings, ComparatorIdentity,
    DeployedQualityBindingMaterial, DeployedQualityReport, EvaluationEvidence, EvaluationMode,
    ExactRate, ExactSignedRate, NegativeControlEvidence, NegativeControlVerdict, PairedComparison,
    PairedCounts as QualityPairedCounts, PairedInterval, QualityMeasurements,
    QualityProfileIdentity, QualityVerdict, WitnessReplayEvidence, DEPLOYED_QUALITY_PROFILE_ID,
    DEPLOYED_QUALITY_PROFILE_VERSION, DEPLOYED_QUALITY_REPORT_SCHEMA, LABEL_SHUFFLED_CONTROL_ID,
    NORMATIVE_EXECUTION_SCOPE, RF31_MIN_LANE_DELTA_PPM, SECTIONS_ABSENT_COMPARATOR_ID,
    TLA_COMPARATOR_ID,
};
use crate::engine::{EngineParts, PolicyStatus};
use crate::serving::{
    CrossSurfaceParityEvidence, NormativeServingDecision, NormativeServingEngine,
};
use crate::witness_replay::{
    parse_and_validate_normative_witness_replay, NormativeWitnessReplayArtifact,
    NormativeWitnessReplayMaterial, NormativeWitnessReplaySpec, DEFAULT_NORMATIVE_WITNESS_SAMPLE,
};

/// Default size of the predeclared ascending held-out prefix instrument.
pub const SAMPLE_TARGET: usize = 6000;

/// Probe positions spent on the readiness/accuracy spot-check before
/// the full sample runs (#232 and its #280 extension).
pub const PROBE_POSITIONS: usize = 64;

const PROBE_PROGRESS_INTERVAL: usize = 16;
const EVAL_PROGRESS_INTERVAL: usize = 256;
const TLA_COMPARATOR_VERSION: &str = "plain-tla-same-position/1";
const SECTIONS_ABSENT_COMPARATOR_VERSION: &str = "r4g1-sections-absent/1";
const LABEL_SHUFFLED_CONTROL_VERSION: &str = "train-target-rotation-half-plus-one/1";
const REPORT_TERMINAL_SCHEMA: &str = "uor-r4-deployed-quality-terminal/1";
const REPORT_PROGRESS_SCHEMA: &str = "uor-r4-deployed-quality-progress/1";

/// Deterministic evaluation extent. A sample can guide whether a census is
/// worth running but can never authorize production admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ServingEvalMode {
    Sample { positions: usize },
    FullCensus,
}

/// What to evaluate: a compiled bundle directory holding the serving
/// artifacts. `graph/score.r4g1` is preferred, root `score.r4g1`
/// accepted; `score_report.json` is read from the graph's directory;
/// the teacher artifact and corpus files sit at the bundle root.
#[derive(Debug, Clone)]
pub struct ServingBundle {
    pub root: PathBuf,
    pub graph: PathBuf,
    /// Planted causal control: the same graph generation with SKMX/PSIB
    /// omitted. Absence keeps research evaluation available but makes
    /// production quality evidence UNAVAILABLE.
    pub sections_absent_graph: Option<PathBuf>,
    /// Planted negative control fitted after rotating TRAIN targets by
    /// `n/2 + 1`, then evaluated against pristine held-out labels.
    pub label_shuffled_graph: Option<PathBuf>,
    pub teacher: PathBuf,
    pub store: PathBuf,
    pub corpus_meta: PathBuf,
    pub corpus_records: PathBuf,
    pub tokenizer: Option<PathBuf>,
    pub tokenizer_adapter: Option<PathBuf>,
    pub score_report: Option<PathBuf>,
    pub compile_report: Option<PathBuf>,
}

/// One immutable, content-addressed capture of every bundle component used by
/// deployed-quality measurement or report construction.
///
/// Capturing once prevents a filesystem generation change from making the
/// measured row describe one set of bytes while the report binds another.
/// Wall-clock paths are retained only for diagnostics; [`Self::generation_cid`]
/// commits to component names, presence, and bytes, not their locations.
pub struct ServingBundleSnapshot {
    bundle: ServingBundle,
    graph: Vec<u8>,
    sections_absent_graph: Option<Vec<u8>>,
    label_shuffled_graph: Option<Vec<u8>>,
    teacher: Vec<u8>,
    store: Vec<u8>,
    corpus_meta: Vec<u8>,
    corpus_records: Vec<u8>,
    tokenizer: Option<Vec<u8>>,
    tokenizer_adapter: Option<Vec<u8>>,
    score_report: Option<Vec<u8>>,
    compile_report: Option<Vec<u8>>,
    generation_cid: String,
}

impl ServingBundleSnapshot {
    /// Capture a complete generation exactly once. A path discovered as
    /// present but unreadable is UNAVAILABLE; it is never silently downgraded
    /// to an absent optional component.
    pub fn capture(bundle: &ServingBundle) -> Result<Self, SourceUnavailable> {
        let read = |field: &str, path: &Path| {
            std::fs::read(path).map_err(|error| {
                SourceUnavailable::new(format!(
                    "capture deployed-quality {field} {}: {error}",
                    path.display()
                ))
            })
        };
        let read_optional =
            |field: &str, path: Option<&Path>| path.map(|path| read(field, path)).transpose();

        let graph = read("graph", &bundle.graph)?;
        let sections_absent_graph = read_optional(
            "sections-absent control",
            bundle.sections_absent_graph.as_deref(),
        )?;
        let label_shuffled_graph = read_optional(
            "label-shuffled control",
            bundle.label_shuffled_graph.as_deref(),
        )?;
        let teacher = read("teacher artifact", &bundle.teacher)?;
        let store = read("TLA store", &bundle.store)?;
        let corpus_meta = read("corpus metadata", &bundle.corpus_meta)?;
        let corpus_records = read("corpus records", &bundle.corpus_records)?;
        let tokenizer = read_optional("tokenizer", bundle.tokenizer.as_deref())?;
        let tokenizer_adapter =
            read_optional("tokenizer adapter", bundle.tokenizer_adapter.as_deref())?;
        let score_report = read_optional("score report", bundle.score_report.as_deref())?;
        let compile_report = read_optional("compile report", bundle.compile_report.as_deref())?;
        let generation_cid = serving_generation_cid(&[
            ("graph", Some(graph.as_slice())),
            ("sections_absent_graph", sections_absent_graph.as_deref()),
            ("label_shuffled_graph", label_shuffled_graph.as_deref()),
            ("teacher", Some(teacher.as_slice())),
            ("store", Some(store.as_slice())),
            ("corpus_meta", Some(corpus_meta.as_slice())),
            ("corpus_records", Some(corpus_records.as_slice())),
            ("tokenizer", tokenizer.as_deref()),
            ("tokenizer_adapter", tokenizer_adapter.as_deref()),
            ("score_report", score_report.as_deref()),
            ("compile_report", compile_report.as_deref()),
        ]);

        Ok(Self {
            bundle: bundle.clone(),
            graph,
            sections_absent_graph,
            label_shuffled_graph,
            teacher,
            store,
            corpus_meta,
            corpus_records,
            tokenizer,
            tokenizer_adapter,
            score_report,
            compile_report,
            generation_cid,
        })
    }

    pub fn generation_cid(&self) -> &str {
        &self.generation_cid
    }

    /// Refuse to continue an expensive phase when its fresh capture no longer
    /// matches the generation which passed the binding sample.
    pub fn require_generation(&self, expected: &str) -> Result<(), SourceUnavailable> {
        if self.generation_cid == expected {
            Ok(())
        } else {
            Err(SourceUnavailable::new(format!(
                "binding sample measured generation {expected}, but the fresh full-census capture is {}; full census was not launched",
                self.generation_cid
            )))
        }
    }

    pub fn graph(&self) -> &[u8] {
        &self.graph
    }

    pub fn signature_artifact(&self) -> &[u8] {
        &self.teacher
    }

    pub fn corpus_meta(&self) -> &[u8] {
        &self.corpus_meta
    }

    pub fn corpus_records(&self) -> &[u8] {
        &self.corpus_records
    }

    pub fn tokenizer(&self) -> Option<&[u8]> {
        self.tokenizer.as_deref()
    }

    pub fn score_report(&self) -> Option<&[u8]> {
        self.score_report.as_deref()
    }
}

/// Wall-clock budgets. Defaults match the retired C row's env contract
/// (`R4_CERTIFY_R4G1_BUDGET_SECS` / `R4_CERTIFY_R4G1_EVAL_BUDGET_SECS`
/// still override, so existing run scripts keep working).
#[derive(Debug, Clone, Copy)]
pub struct ServingEvalBudgets {
    pub probe: Duration,
    pub eval: Duration,
    pub mode: ServingEvalMode,
    pub workers: usize,
}

impl ServingEvalBudgets {
    /// Defaults (probe 120s, eval 600s) with the historical env overrides.
    pub fn from_env() -> Self {
        let secs = |name: &str, default: u64| {
            std::env::var(name)
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default)
        };
        Self {
            probe: Duration::from_secs(secs("R4_CERTIFY_R4G1_BUDGET_SECS", 120)),
            eval: Duration::from_secs(secs("R4_CERTIFY_R4G1_EVAL_BUDGET_SECS", 600)),
            mode: match std::env::var("R4_DEPLOYED_QUALITY_MODE").as_deref() {
                Ok("full") => ServingEvalMode::FullCensus,
                _ => ServingEvalMode::Sample {
                    positions: std::env::var("R4_DEPLOYED_QUALITY_POSITIONS")
                        .ok()
                        .and_then(|value| value.parse().ok())
                        .filter(|&value| value > 0)
                        .unwrap_or(SAMPLE_TARGET),
                },
            },
            workers: std::env::var("R4_DEPLOYED_QUALITY_WORKERS")
                .ok()
                .and_then(|value| value.parse().ok())
                .filter(|&value| value > 0)
                .unwrap_or_else(|| {
                    std::thread::available_parallelism()
                        .map(usize::from)
                        .unwrap_or(1)
                }),
        }
    }
}

/// Per-status decision counts (D4 policy vocabulary). Used for both
/// served and abstained positions — issue #234 item 3: every
/// evaluation run reports the count of held-out probes resolved at
/// each `ResolutionStatus` level, so blended headline numbers cannot
/// hide an exact-context-only distribution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct StatusBreakdown {
    pub exact_context: u64,
    pub graph: u64,
    pub novel: u64,
    pub contradictory: u64,
    /// Of `exact_context`, how many resolved via an explicit NGRAM
    /// context row rather than the EXCT probe (#362 attribution — the
    /// two mechanisms share the `ExactContext` status since e77b1d4,
    /// so era comparisons need the split).
    pub exact_context_ngram: u64,
}

/// Exact paired 2x2 counts against one same-position comparator.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct PairedCounts {
    pub both: u64,
    pub normative_only: u64,
    pub comparator_only: u64,
    pub neither: u64,
}

impl PairedCounts {
    fn record(&mut self, normative_hit: bool, comparator_hit: bool) {
        match (normative_hit, comparator_hit) {
            (true, true) => self.both += 1,
            (true, false) => self.normative_only += 1,
            (false, true) => self.comparator_only += 1,
            (false, false) => self.neither += 1,
        }
    }

    pub fn total(&self) -> u64 {
        self.both + self.normative_only + self.comparator_only + self.neither
    }
}

/// Live, monotonic evaluation counters. The callback receives this after the
/// probe and at least every 256 completed positions, so a run always exposes
/// rate, ETA, reachability, and both decision branches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ServingProgress {
    pub phase: &'static str,
    pub processed: usize,
    pub total: usize,
    pub served: u64,
    pub abstained: u64,
    pub declined: u64,
    pub normative_hits: u64,
    pub tla_hits: u64,
    pub lane_reachable: u64,
    pub lane_changed: u64,
    pub lane_toward: u64,
    pub lane_away: u64,
    pub sections_absent_hits: u64,
    pub label_shuffled_hits: u64,
    pub internal_base_control_checks: u64,
    pub internal_base_control_mismatches: u64,
    pub planted_controls_available: bool,
    pub elapsed_millis: u64,
    pub positions_per_second_milli: u64,
    pub eta_seconds: Option<u64>,
    pub workers: usize,
}

impl StatusBreakdown {
    fn record(&mut self, status: PolicyStatus, ngram_hit: bool) {
        match status {
            PolicyStatus::ExactContext => {
                self.exact_context += 1;
                if ngram_hit {
                    self.exact_context_ngram += 1;
                }
            }
            PolicyStatus::Graph => self.graph += 1,
            PolicyStatus::Novel => self.novel += 1,
            PolicyStatus::Contradictory => self.contradictory += 1,
        }
    }
    pub fn total(&self) -> u64 {
        self.exact_context + self.graph + self.novel + self.contradictory
    }
}

/// The measured serving-surface row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServingEvalRow {
    /// Bundle root the row measured.
    pub bundle: PathBuf,
    /// Exact component generation captured before any probe or measurement.
    /// The report builder rejects a row if it is not given this same snapshot.
    pub generation_cid: String,
    /// Stride-subsample size actually evaluated.
    pub sample_n: usize,
    /// Full held-out population from which this deterministic evaluation was
    /// selected.
    pub population_n: usize,
    pub mode: ServingEvalMode,
    pub workers: usize,
    pub elapsed_millis: u64,
    /// Positions where the policy served a token.
    pub served: u64,
    /// Served predictions that ran the widened re-probe first.
    pub served_widened: u64,
    /// Served predictions by resolving status (#234 item 3).
    pub served_by: StatusBreakdown,
    /// Abstentions by resolving status.
    pub abstained: StatusBreakdown,
    /// Served predictions matching the recorded corpus continuation.
    pub top1_served: u64,
    /// Served predictions matching the recorded teacher argmax.
    pub agree_served: u64,
    /// Same-position teacher-argmax hits by the plain TLA comparator.
    pub tla_hits: u64,
    /// Same-position teacher-argmax hits by the normative runtime with the
    /// SKMX/PSIB lane disabled (the causal base control).
    pub base_hits: u64,
    pub normative_vs_tla: PairedCounts,
    /// Main graph versus its in-runtime sections-absent candidate. Retained as
    /// an implementation cross-check, not used for RF-31 promotion.
    pub normative_vs_base: PairedCounts,
    /// Main graph versus the independently emitted sections-absent artifact.
    pub normative_vs_sections_absent: Option<PairedCounts>,
    /// Label-shuffled planted graph versus the independently emitted
    /// sections-absent artifact, both evaluated on pristine held-out labels.
    pub label_shuffled_vs_sections_absent: Option<PairedCounts>,
    pub sections_absent_hits: u64,
    pub label_shuffled_hits: u64,
    pub internal_base_control_checks: u64,
    pub internal_base_control_mismatches: u64,
    pub lane_reachable: u64,
    pub lane_changed: u64,
    pub lane_toward: u64,
    pub lane_away: u64,
    /// D4 permitted, but the normative runtime could not select a candidate.
    pub declined: u64,
    /// Probe evidence: positions probed / served / hits.
    pub probe_positions: usize,
    pub probe_served: u64,
    pub probe_hits: u64,
    /// Exact absolute-position identities (wall time and worker count are
    /// deliberately excluded).
    pub evaluated_positions_cid: String,
    pub population_positions_cid: String,
}

/// External evidence hooks whose raw counts cannot be inferred inside this
/// evaluator. The report builder validates every witness position against the
/// exact evaluated population and derives its CID itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServingReportEvidence {
    pub compiler_revision: String,
    /// Durable, deterministic parity artifact bytes. The report builder hashes
    /// and parses these bytes itself, recomputes their counts, and binds their
    /// graph/signature/tokenizer CIDs to the bundle. Bare claimed counts
    /// cannot enter.
    pub cross_surface_evidence: Vec<u8>,
    /// Canonical replay artifact bytes. Positions, runtime winner claims,
    /// lane attribution, replay verdicts, and counts are parsed and
    /// independently recomputed; no caller-provided counter is accepted.
    pub witness_replay_evidence: Vec<u8>,
}

impl ServingReportEvidence {
    /// Validate and decode the external parity hook against the exact serving
    /// artifacts. Certifiers should use this seam before accepting raw parity
    /// bytes; checks and mismatches are properties of the validated artifact.
    pub fn validated_cross_surface_evidence(
        &self,
        graph: &[u8],
        signature_artifact: &[u8],
        tokenizer: &[u8],
        score_report: &[u8],
    ) -> Result<CrossSurfaceParityEvidence, SourceUnavailable> {
        CrossSurfaceParityEvidence::parse_and_validate_for_production_bundle(
            &self.cross_surface_evidence,
            graph,
            signature_artifact,
            tokenizer,
            score_report,
        )
    }

    /// Replay the raw witness artifact against the exact serving generation
    /// and evaluator population before any count enters a quality report.
    pub fn validated_witness_replay_evidence(
        &self,
        spec: NormativeWitnessReplaySpec<'_>,
    ) -> Result<NormativeWitnessReplayArtifact, SourceUnavailable> {
        parse_and_validate_normative_witness_replay(&self.witness_replay_evidence, spec)
    }
}

/// Durable paths for one run. Callers should place these in a staging bundle;
/// a failed run never overwrites a production-admissible report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServingReportPaths {
    pub progress_jsonl: PathBuf,
    pub terminal_json: PathBuf,
    pub deployed_quality_json: PathBuf,
}

impl ServingReportPaths {
    pub fn in_bundle(bundle: &ServingBundle) -> Self {
        let graph_dir = bundle.graph.parent().unwrap_or(bundle.root.as_path());
        Self {
            progress_jsonl: graph_dir.join("deployed_quality_progress.jsonl"),
            terminal_json: graph_dir.join("deployed_quality_terminal.json"),
            deployed_quality_json: graph_dir.join("deployed_quality_report.json"),
        }
    }
}

/// High-level producer result. `report` is absent only for a skipped or
/// unavailable evaluation; sample reports are present but remain `Estimate`
/// and cannot pass production validation.
#[derive(Debug, Clone)]
pub struct RecordedServingEval {
    pub outcome: ServingEvalOutcome,
    pub report: Option<DeployedQualityReport>,
    pub report_cid: Option<String>,
}

/// A recorded skip: the row was not measured, and the reason is data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServingEvalSkip {
    /// The probe could not finish inside its budget.
    ProbeBudgetExceeded { probed: usize, elapsed: Duration },
    /// The probe served predictions but none matched the corpus
    /// continuation or teacher argmax (#280 functional spot-check).
    ProbeFunctionalCheckFailed { served: u64, probed: usize },
    /// The subsampled evaluation exceeded its budget; partial counts
    /// are discarded (a truncated prefix is biased by construction).
    EvalBudgetExceeded {
        done: usize,
        sample_n: usize,
        elapsed: Duration,
    },
}

/// Outcome of one serving-surface evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServingEvalOutcome {
    Row(Box<ServingEvalRow>),
    Skipped(ServingEvalSkip),
}

impl ServingBundle {
    /// Resolve a bundle directory into its serving artifact paths.
    /// Returns `None` when any required file is absent — callers treat
    /// that as "this directory is not a compiled serving bundle".
    pub fn discover(root: &Path) -> Option<Self> {
        let graph_nested = root.join("graph").join("score.r4g1");
        let graph_flat = root.join("score.r4g1");
        let graph = if graph_nested.is_file() {
            graph_nested
        } else if graph_flat.is_file() {
            graph_flat
        } else {
            return None;
        };
        let graph_dir = graph.parent()?.to_path_buf();
        let optional_file = |path: PathBuf| path.is_file().then_some(path);
        let sections_absent_graph = optional_file(graph_dir.join("score_sections_absent.r4g1"));
        let label_shuffled_graph = optional_file(graph_dir.join("score_label_shuffled.r4g1"));
        let teacher = root.join("tless_artifacts.bin");
        let store = root.join("tless_store.bin");
        let corpus_meta = root.join("corpus.meta");
        let corpus_records = root.join("corpus.records");
        if !teacher.is_file()
            || !store.is_file()
            || !corpus_meta.is_file()
            || !corpus_records.is_file()
        {
            return None;
        }
        Some(Self {
            root: root.to_path_buf(),
            graph,
            sections_absent_graph,
            label_shuffled_graph,
            teacher,
            store,
            corpus_meta,
            corpus_records,
            tokenizer: optional_file(root.join("tokenizer.bin")),
            tokenizer_adapter: optional_file(root.join("tokenizer_adapter.json")),
            score_report: optional_file(graph_dir.join("score_report.json")),
            compile_report: optional_file(root.join("graph-cover").join("cover_report.json")),
        })
    }

    /// Scan `.uor-models/compiled/*` under `base` for serving bundles,
    /// in deterministic (sorted) directory order.
    pub fn scan(base: &Path) -> Vec<Self> {
        let compiled = base.join(".uor-models").join("compiled");
        let Ok(entries) = std::fs::read_dir(&compiled) else {
            return Vec::new();
        };
        let mut roots: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect();
        roots.sort();
        roots.iter().filter_map(|r| Self::discover(r)).collect()
    }
}

#[derive(Debug, Clone, Copy)]
struct PositionRow {
    token: Option<u32>,
    internal_base_token: Option<u32>,
    sections_absent_token: Option<u32>,
    label_shuffled_token: Option<u32>,
    status: Option<PolicyStatus>,
    ngram_hit: bool,
    widened: bool,
    abstained: bool,
    declined: bool,
    lane_reachable: bool,
    lane_changed: bool,
    next: u32,
    teacher_argmax: u32,
    tla_token: u32,
}

#[derive(Clone, Copy)]
struct EvaluationContext<'a> {
    corpus: &'a compiler::Corpus,
    artifacts: &'a compiler::Compiled,
    store: &'a runtime::Store,
    tables: &'a runtime::AssignTables,
    rotations: &'a [usize; compiler::WINDOW + 1],
}

struct ControlEngines<'borrow, 'graph> {
    sections_absent: Option<&'borrow mut NormativeServingEngine<'graph>>,
    label_shuffled: Option<&'borrow mut NormativeServingEngine<'graph>>,
}

fn evaluate_position(
    engine: &mut NormativeServingEngine<'_>,
    controls: ControlEngines<'_, '_>,
    context: EvaluationContext<'_>,
    position: usize,
) -> Result<PositionRow, String> {
    // Each recorded position is an isolated teacher-forced decision. D4's
    // bounded session memory therefore resets between positions; no worker or
    // sharding order can change a verdict.
    engine.reset_policy_state();
    let window = induction::context_window(context.corpus, position);
    let code = runtime::code_plain_with(
        context.tables,
        context.artifacts,
        context.rotations,
        context.corpus,
        position,
    );
    let tla_token = runtime::predict_witness_plain(context.store, &code).token;
    let next = context.corpus.next[position];
    let teacher_argmax = context.corpus.t_argmax[position];
    let mut row = PositionRow {
        token: None,
        internal_base_token: None,
        sections_absent_token: None,
        label_shuffled_token: None,
        status: None,
        ngram_hit: false,
        widened: false,
        abstained: false,
        declined: false,
        lane_reachable: false,
        lane_changed: false,
        next,
        teacher_argmax,
        tla_token,
    };
    match engine
        .predict(&window)
        .map_err(|error| format!("serving decision: {error}"))?
    {
        NormativeServingDecision::Serve(outcome) => {
            row.token = Some(outcome.token);
            row.internal_base_token = Some(outcome.base_token);
            row.status = Some(outcome.status.into());
            row.ngram_hit = outcome.ngram_hit;
            row.widened = outcome.widened;
            row.lane_reachable = outcome.lane_reachable;
            row.lane_changed = outcome.token != outcome.base_token;
        }
        NormativeServingDecision::Abstain(outcome) => {
            row.status = Some(outcome.status.into());
            row.ngram_hit = outcome.ngram_hit;
            row.widened = outcome.widened;
            row.abstained = true;
        }
        NormativeServingDecision::Decline(_) => row.declined = true,
    }
    row.sections_absent_token = evaluate_control_token(controls.sections_absent, &window)?;
    row.label_shuffled_token = evaluate_control_token(controls.label_shuffled, &window)?;
    Ok(row)
}

fn evaluate_control_token(
    engine: Option<&mut NormativeServingEngine<'_>>,
    window: &[u32],
) -> Result<Option<u32>, String> {
    let Some(engine) = engine else {
        return Ok(None);
    };
    engine.reset_policy_state();
    match engine
        .predict(window)
        .map_err(|error| format!("control serving decision: {error}"))?
    {
        NormativeServingDecision::Serve(outcome) => Ok(Some(outcome.token)),
        NormativeServingDecision::Abstain(_) | NormativeServingDecision::Decline(_) => Ok(None),
    }
}

fn record_position(row: &mut ServingEvalRow, position: PositionRow) {
    let normative_hit = position.token == Some(position.teacher_argmax);
    let base_hit = position.internal_base_token == Some(position.teacher_argmax);
    let tla_hit = position.tla_token == position.teacher_argmax;
    row.normative_vs_tla.record(normative_hit, tla_hit);
    row.normative_vs_base.record(normative_hit, base_hit);
    row.tla_hits += u64::from(tla_hit);
    row.base_hits += u64::from(base_hit);
    if let Some(counts) = row.normative_vs_sections_absent.as_mut() {
        let absent_hit = position.sections_absent_token == Some(position.teacher_argmax);
        counts.record(normative_hit, absent_hit);
        row.sections_absent_hits += u64::from(absent_hit);
        row.internal_base_control_checks += 1;
        row.internal_base_control_mismatches +=
            u64::from(position.internal_base_token != position.sections_absent_token);
    }
    if let Some(counts) = row.label_shuffled_vs_sections_absent.as_mut() {
        let shuffled_hit = position.label_shuffled_token == Some(position.teacher_argmax);
        let absent_hit = position.sections_absent_token == Some(position.teacher_argmax);
        counts.record(shuffled_hit, absent_hit);
        row.label_shuffled_hits += u64::from(shuffled_hit);
    }
    row.lane_reachable += u64::from(position.lane_reachable);
    row.lane_changed += u64::from(position.lane_changed);
    if position.lane_changed {
        row.lane_toward += u64::from(normative_hit && !base_hit);
        row.lane_away += u64::from(!normative_hit && base_hit);
    }
    if position.declined {
        row.declined += 1;
    } else if position.abstained {
        if let Some(status) = position.status {
            row.abstained.record(status, position.ngram_hit);
        }
    } else if let Some(token) = position.token {
        row.served += 1;
        if let Some(status) = position.status {
            row.served_by.record(status, position.ngram_hit);
        }
        row.served_widened += u64::from(position.widened);
        row.top1_served += u64::from(token == position.next);
        row.agree_served += u64::from(normative_hit);
    }
}

fn progress_snapshot(
    phase: &'static str,
    row: &ServingEvalRow,
    processed: usize,
    total: usize,
    started: Instant,
) -> ServingProgress {
    let elapsed_millis = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let positions_per_second_milli = (processed as u64)
        .saturating_mul(1_000_000)
        .checked_div(elapsed_millis)
        .unwrap_or(0);
    let eta_seconds = (processed > 0 && processed < total).then(|| {
        elapsed_millis.saturating_mul((total - processed) as u64) / processed as u64 / 1000
    });
    ServingProgress {
        phase,
        processed,
        total,
        served: row.served,
        abstained: row.abstained.total(),
        declined: row.declined,
        normative_hits: row.agree_served,
        tla_hits: row.tla_hits,
        lane_reachable: row.lane_reachable,
        lane_changed: row.lane_changed,
        lane_toward: row.lane_toward,
        lane_away: row.lane_away,
        sections_absent_hits: row.sections_absent_hits,
        label_shuffled_hits: row.label_shuffled_hits,
        internal_base_control_checks: row.internal_base_control_checks,
        internal_base_control_mismatches: row.internal_base_control_mismatches,
        planted_controls_available: row.normative_vs_sections_absent.is_some()
            && row.label_shuffled_vs_sections_absent.is_some(),
        elapsed_millis,
        positions_per_second_milli,
        eta_seconds,
        workers: row.workers,
    }
}

fn probe_progress_snapshot(
    processed: usize,
    total: usize,
    served: u64,
    hits: u64,
    started: Instant,
    workers: usize,
    planted_controls_available: bool,
) -> ServingProgress {
    let elapsed_millis = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let positions_per_second_milli = (processed as u64)
        .saturating_mul(1_000_000)
        .checked_div(elapsed_millis)
        .unwrap_or(0);
    let eta_seconds = (processed > 0 && processed < total).then(|| {
        elapsed_millis.saturating_mul((total - processed) as u64) / processed as u64 / 1000
    });
    ServingProgress {
        phase: "probe",
        processed,
        total,
        served,
        abstained: processed as u64 - served,
        declined: 0,
        normative_hits: hits,
        tla_hits: 0,
        lane_reachable: 0,
        lane_changed: 0,
        lane_toward: 0,
        lane_away: 0,
        sections_absent_hits: 0,
        label_shuffled_hits: 0,
        internal_base_control_checks: 0,
        internal_base_control_mismatches: 0,
        planted_controls_available,
        elapsed_millis,
        positions_per_second_milli,
        eta_seconds,
        workers,
    }
}

fn select_positions(held_out: &[usize], mode: ServingEvalMode) -> Vec<usize> {
    match mode {
        ServingEvalMode::FullCensus => held_out.to_vec(),
        ServingEvalMode::Sample { positions } => held_out[..positions.min(held_out.len())].to_vec(),
    }
}

fn positions_cid_usize(positions: &[usize]) -> String {
    let positions: Vec<u64> = positions.iter().map(|&position| position as u64).collect();
    deployed_quality_positions_cid(&positions)
}

/// Evaluate one bundle's exact normative serving surface on its held-out
/// partition. Work is deterministically sharded over all configured workers;
/// results reduce in original position order, so worker count cannot change
/// artifact bytes or verdicts.
pub fn evaluate_serving_bundle(
    bundle: &ServingBundle,
    budgets: ServingEvalBudgets,
    progress: &mut dyn FnMut(ServingProgress),
) -> Result<ServingEvalOutcome, SourceUnavailable> {
    let snapshot = ServingBundleSnapshot::capture(bundle)?;
    evaluate_serving_snapshot(&snapshot, budgets, progress)
}

/// Evaluate a previously captured immutable generation. Measurement, controls,
/// comparator, and the resulting row all borrow this single byte snapshot.
pub fn evaluate_serving_snapshot(
    snapshot: &ServingBundleSnapshot,
    budgets: ServingEvalBudgets,
    progress: &mut dyn FnMut(ServingProgress),
) -> Result<ServingEvalOutcome, SourceUnavailable> {
    progress(ServingProgress {
        phase: "load",
        processed: 0,
        total: 0,
        served: 0,
        abstained: 0,
        declined: 0,
        normative_hits: 0,
        tla_hits: 0,
        lane_reachable: 0,
        lane_changed: 0,
        lane_toward: 0,
        lane_away: 0,
        sections_absent_hits: 0,
        label_shuffled_hits: 0,
        internal_base_control_checks: 0,
        internal_base_control_mismatches: 0,
        planted_controls_available: snapshot.sections_absent_graph.is_some()
            && snapshot.label_shuffled_graph.is_some(),
        elapsed_millis: 0,
        positions_per_second_milli: 0,
        eta_seconds: None,
        workers: budgets.workers.max(1),
    });
    let graph_bytes = snapshot.graph.as_slice();
    let sections_absent_bytes = snapshot.sections_absent_graph.as_deref();
    let label_shuffled_bytes = snapshot.label_shuffled_graph.as_deref();
    validate_planted_control_graphs(graph_bytes, sections_absent_bytes, label_shuffled_bytes)?;
    let teacher_bytes = snapshot.teacher.as_slice();
    let store_bytes = snapshot.store.as_slice();
    let score_report = snapshot
        .score_report
        .as_deref()
        .filter(|bytes| serde_json::from_slice::<serde_json::Value>(bytes).is_ok());
    let artifacts = compiler::parse_artifacts(teacher_bytes)
        .ok_or_else(|| SourceUnavailable::new("not a supported plain-TLA artifact container"))?;
    let store = runtime::parse_store(store_bytes)
        .ok_or_else(|| SourceUnavailable::new("not a valid TLS1 TLA store"))?;
    let tables = runtime::AssignTables::new(&artifacts);
    let rotations = compiler::derive_rotations();

    let corpus = compiler::load_corpus_bytes(&snapshot.corpus_meta, &snapshot.corpus_records, None)
        .ok_or_else(|| {
            SourceUnavailable::new("corpus: captured bytes are incomplete or invalid")
        })?;
    let (_, held_out) = induction::split_positions(&corpus);
    if held_out.is_empty() {
        return Err(SourceUnavailable::new(
            "the bundle's held-out partition is empty",
        ));
    }
    let positions = select_positions(&held_out, budgets.mode);
    if positions.is_empty() {
        return Err(SourceUnavailable::new(
            "the configured serving evaluation selects zero positions",
        ));
    }
    if matches!(budgets.mode, ServingEvalMode::Sample { .. }) && positions.len() == held_out.len() {
        return Err(SourceUnavailable::new(
            "sample mode reaches the full population; select full-census mode explicitly",
        ));
    }
    let workers = budgets.workers.max(1).min(positions.len().max(1));
    let parts = EngineParts {
        graph: graph_bytes,
        signature_artifact: teacher_bytes,
        tokenizer: None,
        score_report,
    };
    let mut probe_engine = NormativeServingEngine::load_for_research(parts)?;
    let mut probe_sections_absent = sections_absent_bytes
        .map(|graph| NormativeServingEngine::load_for_research(EngineParts { graph, ..parts }))
        .transpose()?;
    let mut probe_label_shuffled = label_shuffled_bytes
        .map(|graph| NormativeServingEngine::load_for_research(EngineParts { graph, ..parts }))
        .transpose()?;
    let probe_n = positions.len().min(PROBE_POSITIONS);
    let probe_start = Instant::now();
    let (mut probe_served, mut probe_hits) = (0u64, 0u64);
    for (done, &position) in positions[..probe_n].iter().enumerate() {
        if probe_start.elapsed() >= budgets.probe {
            return Ok(ServingEvalOutcome::Skipped(
                ServingEvalSkip::ProbeBudgetExceeded {
                    probed: done,
                    elapsed: probe_start.elapsed(),
                },
            ));
        }
        let measured = evaluate_position(
            &mut probe_engine,
            ControlEngines {
                sections_absent: probe_sections_absent.as_mut(),
                label_shuffled: probe_label_shuffled.as_mut(),
            },
            EvaluationContext {
                corpus: &corpus,
                artifacts: &artifacts,
                store: &store,
                tables: &tables,
                rotations: &rotations,
            },
            position,
        )
        .map_err(SourceUnavailable::new)?;
        if let Some(token) = measured.token {
            probe_served += 1;
            probe_hits += u64::from(token == measured.next || token == measured.teacher_argmax);
        }
        let processed = done + 1;
        if processed.is_multiple_of(PROBE_PROGRESS_INTERVAL) || processed == probe_n {
            progress(probe_progress_snapshot(
                processed,
                probe_n,
                probe_served,
                probe_hits,
                probe_start,
                1,
                sections_absent_bytes.is_some() && label_shuffled_bytes.is_some(),
            ));
        }
    }
    if probe_served == 0 || probe_hits == 0 {
        return Ok(ServingEvalOutcome::Skipped(
            ServingEvalSkip::ProbeFunctionalCheckFailed {
                served: probe_served,
                probed: probe_n,
            },
        ));
    }

    let eval_start = Instant::now();
    let mut aggregate = ServingEvalRow {
        bundle: snapshot.bundle.root.clone(),
        generation_cid: snapshot.generation_cid.clone(),
        sample_n: positions.len(),
        population_n: held_out.len(),
        mode: budgets.mode,
        workers,
        elapsed_millis: 0,
        served: 0,
        served_widened: 0,
        served_by: StatusBreakdown::default(),
        abstained: StatusBreakdown::default(),
        top1_served: 0,
        agree_served: 0,
        tla_hits: 0,
        base_hits: 0,
        normative_vs_tla: PairedCounts::default(),
        normative_vs_base: PairedCounts::default(),
        normative_vs_sections_absent: sections_absent_bytes.is_some().then(PairedCounts::default),
        label_shuffled_vs_sections_absent: (sections_absent_bytes.is_some()
            && label_shuffled_bytes.is_some())
        .then(PairedCounts::default),
        sections_absent_hits: 0,
        label_shuffled_hits: 0,
        internal_base_control_checks: 0,
        internal_base_control_mismatches: 0,
        lane_reachable: 0,
        lane_changed: 0,
        lane_toward: 0,
        lane_away: 0,
        declined: 0,
        probe_positions: probe_n,
        probe_served,
        probe_hits,
        evaluated_positions_cid: positions_cid_usize(&positions),
        population_positions_cid: positions_cid_usize(&held_out),
    };
    let cancelled = AtomicBool::new(false);
    let (sender, receiver) = mpsc::channel::<(usize, Result<PositionRow, String>)>();
    let mut ordered = vec![None; positions.len()];
    let mut done = 0usize;
    let mut worker_error = None;
    let mut budget_exceeded = false;

    std::thread::scope(|scope| {
        for worker in 0..workers {
            let sender = sender.clone();
            let positions = &positions;
            let corpus = &corpus;
            let artifacts = &artifacts;
            let store = &store;
            let tables = &tables;
            let rotations = &rotations;
            let cancelled = &cancelled;
            scope.spawn(move || {
                let mut engine = match NormativeServingEngine::load_for_research(parts) {
                    Ok(engine) => engine,
                    Err(error) => {
                        let _ = sender.send((usize::MAX, Err(error.to_string())));
                        return;
                    }
                };
                let mut sections_absent_engine = match sections_absent_bytes
                    .map(|graph| {
                        NormativeServingEngine::load_for_research(EngineParts { graph, ..parts })
                    })
                    .transpose()
                {
                    Ok(engine) => engine,
                    Err(error) => {
                        let _ = sender.send((usize::MAX, Err(error.to_string())));
                        return;
                    }
                };
                let mut label_shuffled_engine = match label_shuffled_bytes
                    .map(|graph| {
                        NormativeServingEngine::load_for_research(EngineParts { graph, ..parts })
                    })
                    .transpose()
                {
                    Ok(engine) => engine,
                    Err(error) => {
                        let _ = sender.send((usize::MAX, Err(error.to_string())));
                        return;
                    }
                };
                for ordinal in (worker..positions.len()).step_by(workers) {
                    if cancelled.load(Ordering::Relaxed) {
                        break;
                    }
                    let result = evaluate_position(
                        &mut engine,
                        ControlEngines {
                            sections_absent: sections_absent_engine.as_mut(),
                            label_shuffled: label_shuffled_engine.as_mut(),
                        },
                        EvaluationContext {
                            corpus,
                            artifacts,
                            store,
                            tables,
                            rotations,
                        },
                        positions[ordinal],
                    );
                    if sender.send((ordinal, result)).is_err() {
                        break;
                    }
                }
            });
        }
        drop(sender);
        while done < positions.len() {
            if eval_start.elapsed() >= budgets.eval {
                budget_exceeded = true;
                cancelled.store(true, Ordering::Relaxed);
                break;
            }
            match receiver.recv_timeout(Duration::from_millis(200)) {
                Ok((ordinal, Ok(position))) if ordinal < ordered.len() => {
                    ordered[ordinal] = Some(position);
                    record_position(&mut aggregate, position);
                    done += 1;
                    if done.is_multiple_of(EVAL_PROGRESS_INTERVAL) || done == positions.len() {
                        progress(progress_snapshot(
                            "evaluate",
                            &aggregate,
                            done,
                            positions.len(),
                            eval_start,
                        ));
                    }
                }
                Ok((_, Err(error))) => {
                    worker_error = Some(error);
                    cancelled.store(true, Ordering::Relaxed);
                    break;
                }
                Ok((ordinal, Ok(_))) => {
                    worker_error = Some(format!("worker returned out-of-range ordinal {ordinal}"));
                    cancelled.store(true, Ordering::Relaxed);
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });
    if let Some(error) = worker_error {
        return Err(SourceUnavailable::new(error));
    }
    if budget_exceeded || done != positions.len() {
        return Ok(ServingEvalOutcome::Skipped(
            ServingEvalSkip::EvalBudgetExceeded {
                done,
                sample_n: positions.len(),
                elapsed: eval_start.elapsed(),
            },
        ));
    }

    // Re-reduce in canonical position order. The live accumulator above is
    // instrumentation only; this ordered reduction is the evidence value and
    // is invariant to worker count and completion order.
    let mut row = ServingEvalRow {
        bundle: snapshot.bundle.root.clone(),
        generation_cid: snapshot.generation_cid.clone(),
        sample_n: positions.len(),
        population_n: held_out.len(),
        mode: budgets.mode,
        workers,
        elapsed_millis: eval_start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
        served: 0,
        served_widened: 0,
        served_by: StatusBreakdown::default(),
        abstained: StatusBreakdown::default(),
        top1_served: 0,
        agree_served: 0,
        tla_hits: 0,
        base_hits: 0,
        normative_vs_tla: PairedCounts::default(),
        normative_vs_base: PairedCounts::default(),
        normative_vs_sections_absent: sections_absent_bytes.is_some().then(PairedCounts::default),
        label_shuffled_vs_sections_absent: (sections_absent_bytes.is_some()
            && label_shuffled_bytes.is_some())
        .then(PairedCounts::default),
        sections_absent_hits: 0,
        label_shuffled_hits: 0,
        internal_base_control_checks: 0,
        internal_base_control_mismatches: 0,
        lane_reachable: 0,
        lane_changed: 0,
        lane_toward: 0,
        lane_away: 0,
        declined: 0,
        probe_positions: probe_n,
        probe_served,
        probe_hits,
        evaluated_positions_cid: positions_cid_usize(&positions),
        population_positions_cid: positions_cid_usize(&held_out),
    };
    for position in ordered {
        record_position(
            &mut row,
            position.expect("every completed ordinal has a measured row"),
        );
    }
    debug_assert_eq!(row.normative_vs_tla.total(), positions.len() as u64);
    progress(progress_snapshot(
        "complete",
        &row,
        positions.len(),
        positions.len(),
        eval_start,
    ));
    Ok(ServingEvalOutcome::Row(Box::new(row)))
}

/// Build the deterministic, content-bound quality report for one completed
/// row. Every byte-derived identity is recomputed here through
/// [`derive_deployed_quality_bindings`]; only the compiler source revision and
/// separately executed parity/witness observations enter through `evidence`.
pub fn build_deployed_quality_report(
    bundle: &ServingBundle,
    row: &ServingEvalRow,
    evidence: &ServingReportEvidence,
) -> Result<DeployedQualityReport, SourceUnavailable> {
    let snapshot = ServingBundleSnapshot::capture(bundle)?;
    build_deployed_quality_report_from_snapshot(&snapshot, row, evidence)
}

fn build_deployed_quality_report_from_snapshot(
    snapshot: &ServingBundleSnapshot,
    row: &ServingEvalRow,
    evidence: &ServingReportEvidence,
) -> Result<DeployedQualityReport, SourceUnavailable> {
    if row.bundle != snapshot.bundle.root {
        return Err(SourceUnavailable::new(format!(
            "serving row names bundle {}, report builder received {}",
            row.bundle.display(),
            snapshot.bundle.root.display()
        )));
    }
    if row.generation_cid != snapshot.generation_cid {
        return Err(SourceUnavailable::new(format!(
            "serving row generation {} does not equal captured report generation {}",
            row.generation_cid, snapshot.generation_cid
        )));
    }
    let graph = snapshot.graph.as_slice();
    let teacher = snapshot.teacher.as_slice();
    let store = snapshot.store.as_slice();
    let corpus_meta = snapshot.corpus_meta.as_slice();
    let corpus_records = snapshot.corpus_records.as_slice();
    let tokenizer = required_snapshot_component("tokenizer", snapshot.tokenizer.as_deref())?;
    let tokenizer_adapter =
        required_snapshot_component("tokenizer adapter", snapshot.tokenizer_adapter.as_deref())?;
    let score_report =
        required_snapshot_component("score report", snapshot.score_report.as_deref())?;
    let cross_surface_evidence =
        evidence.validated_cross_surface_evidence(graph, teacher, tokenizer, score_report)?;
    let compile_report =
        required_snapshot_component("compile report", snapshot.compile_report.as_deref())?;

    let corpus = compiler::load_corpus_bytes(corpus_meta, corpus_records, None)
        .ok_or_else(|| SourceUnavailable::new("deployed-quality corpus parse failed"))?;
    let (_, held_out) = induction::split_positions(&corpus);
    let selected = select_positions(&held_out, row.mode);
    if selected.len() != row.sample_n || held_out.len() != row.population_n {
        return Err(SourceUnavailable::new(format!(
            "serving row population {}/{}, recomputed {}/{}",
            row.sample_n,
            row.population_n,
            selected.len(),
            held_out.len()
        )));
    }
    let full_positions = usize_positions_to_u64(&held_out)?;
    let evaluated_positions = usize_positions_to_u64(&selected)?;
    if deployed_quality_positions_cid(&full_positions) != row.population_positions_cid
        || deployed_quality_positions_cid(&evaluated_positions) != row.evaluated_positions_cid
    {
        return Err(SourceUnavailable::new(
            "serving row position identities do not reproduce from the loaded corpus",
        ));
    }
    let witness_artifact =
        evidence.validated_witness_replay_evidence(NormativeWitnessReplaySpec {
            material: NormativeWitnessReplayMaterial {
                graph,
                signature_artifact: teacher,
                tokenizer,
                score_report: Some(score_report),
                corpus_meta,
                corpus_records,
            },
            evaluated_positions: &evaluated_positions,
            sample_size: DEFAULT_NORMATIVE_WITNESS_SAMPLE,
        })?;

    let bindings = derive_deployed_quality_bindings(DeployedQualityBindingMaterial {
        graph,
        teacher_artifact: teacher,
        corpus_meta,
        corpus_records,
        tokenizer,
        tokenizer_adapter,
        score_report,
        compile_report,
        compiler_revision: &evidence.compiler_revision,
        full_population_positions: &full_positions,
        evaluated_positions: &evaluated_positions,
    })
    .map_err(|error| SourceUnavailable::new(error.to_string()))?;

    let tla_definition_cid = report_tagged_cid(
        b"r4-deployed-quality-tla-comparator/1",
        &[TLA_COMPARATOR_VERSION.as_bytes(), store],
    );
    let versus_tla = paired_comparison(
        TLA_COMPARATOR_ID,
        TLA_COMPARATOR_VERSION,
        tla_definition_cid,
        &bindings.partition.evaluated_positions_cid,
        row.normative_vs_tla,
    )?;

    let sections_absent_bytes = snapshot.sections_absent_graph.as_deref();
    let label_shuffled_bytes = snapshot.label_shuffled_graph.as_deref();
    let planted_controls_available = sections_absent_bytes.is_some()
        && label_shuffled_bytes.is_some()
        && row.normative_vs_sections_absent.is_some()
        && row.label_shuffled_vs_sections_absent.is_some();
    validate_planted_control_graphs(graph, sections_absent_bytes, label_shuffled_bytes)?;

    let (versus_sections_absent, label_control) = if planted_controls_available {
        let sections_absent_bytes = sections_absent_bytes.ok_or_else(|| {
            SourceUnavailable::new("sections-absent control disappeared during report build")
        })?;
        let label_shuffled_bytes = label_shuffled_bytes.ok_or_else(|| {
            SourceUnavailable::new("label-shuffled control disappeared during report build")
        })?;
        let absent_definition_cid = report_tagged_cid(
            b"r4-deployed-quality-sections-absent-comparator/1",
            &[
                SECTIONS_ABSENT_COMPARATOR_VERSION.as_bytes(),
                sections_absent_bytes,
            ],
        );
        let comparison = paired_comparison(
            SECTIONS_ABSENT_COMPARATOR_ID,
            SECTIONS_ABSENT_COMPARATOR_VERSION,
            absent_definition_cid.clone(),
            &bindings.partition.evaluated_positions_cid,
            row.normative_vs_sections_absent
                .ok_or_else(|| SourceUnavailable::new("missing sections-absent paired counts"))?,
        )?;
        let shuffled_counts = row
            .label_shuffled_vs_sections_absent
            .ok_or_else(|| SourceUnavailable::new("missing label-shuffled paired counts"))?;
        let shuffled_comparison = paired_comparison(
            SECTIONS_ABSENT_COMPARATOR_ID,
            SECTIONS_ABSENT_COMPARATOR_VERSION,
            absent_definition_cid,
            &bindings.partition.evaluated_positions_cid,
            shuffled_counts,
        )?;
        let identity_cid = report_tagged_cid(
            b"r4-deployed-quality-label-shuffled-control/1",
            &[
                LABEL_SHUFFLED_CONTROL_VERSION.as_bytes(),
                sections_absent_bytes,
                label_shuffled_bytes,
                bindings.partition.manifest_cid.as_bytes(),
            ],
        );
        let verdict = if shuffled_comparison.delta.numerator <= 0 {
            NegativeControlVerdict::Passed
        } else {
            NegativeControlVerdict::Failed
        };
        (
            comparison,
            NegativeControlEvidence {
                id: LABEL_SHUFFLED_CONTROL_ID.to_string(),
                identity_cid,
                verdict,
                comparison: Some(shuffled_comparison),
            },
        )
    } else {
        // Research estimates retain a same-position comparison against the
        // runtime's internal lane-absent candidate, but the absent planted
        // artifacts are explicitly UNAVAILABLE and can never produce PASS.
        let comparison = paired_comparison(
            "R4G1Runtime-internal-sections-absent",
            "internal-base-candidate/1",
            report_tagged_cid(b"r4-deployed-quality-internal-base-comparator/1", &[graph]),
            &bindings.partition.evaluated_positions_cid,
            row.normative_vs_base,
        )?;
        (
            comparison,
            NegativeControlEvidence {
                id: LABEL_SHUFFLED_CONTROL_ID.to_string(),
                identity_cid: report_tagged_cid(
                    b"r4-deployed-quality-label-shuffled-control-unavailable/1",
                    &[bindings.partition.manifest_cid.as_bytes()],
                ),
                verdict: NegativeControlVerdict::Unavailable,
                comparison: None,
            },
        )
    };

    let cross_surface_checks = cross_surface_evidence
        .checks
        .checked_add(row.internal_base_control_checks)
        .ok_or_else(|| SourceUnavailable::new("cross-surface check count overflow"))?;
    let cross_surface_mismatches = cross_surface_evidence
        .mismatches
        .checked_add(row.internal_base_control_mismatches)
        .ok_or_else(|| SourceUnavailable::new("cross-surface mismatch count overflow"))?;
    let measurements = QualityMeasurements {
        versus_tla,
        versus_sections_absent,
        internal_base_control_checks: row.internal_base_control_checks,
        internal_base_control_mismatches: row.internal_base_control_mismatches,
        cross_surface_checks,
        cross_surface_mismatches,
        cross_surface_evidence_cid: report_tagged_cid(
            b"r4-deployed-quality-cross-surface-evidence/1",
            &[
                &cross_surface_evidence.checks.to_le_bytes(),
                &cross_surface_evidence.mismatches.to_le_bytes(),
                &row.internal_base_control_checks.to_le_bytes(),
                &row.internal_base_control_mismatches.to_le_bytes(),
                cross_surface_evidence.graph_cid.as_bytes(),
                cross_surface_evidence.signature_artifact_cid.as_bytes(),
                &evidence.cross_surface_evidence,
            ],
        ),
    };
    let witness_replay = WitnessReplayEvidence {
        sample_cid: witness_artifact.sample_positions_cid.clone(),
        requested: witness_artifact.requested,
        replayed: witness_artifact.replayed,
        failures: witness_artifact.failures,
    };

    let promotion_failures = promotion_failures(PromotionGateInputs {
        row,
        measurements: &measurements,
        label_control: &label_control,
        planted_controls_available,
        external_cross_surface_checks: cross_surface_evidence.checks,
        combined_cross_surface_mismatches: cross_surface_mismatches,
        witness_replayed: witness_artifact.replayed,
        witness_failures: witness_artifact.failures,
    });
    let (mode, verdict, measurements) = match row.mode {
        ServingEvalMode::Sample { .. } => (
            EvaluationMode::Sample,
            QualityVerdict::Estimate {
                decision: if promotion_failures.is_empty() {
                    format!(
                        "PROCEED: binding sample satisfies all predeclared gates; lane_reachable={}/{}; run the full census on this exact bundle and evidence generation",
                        row.lane_reachable, row.sample_n
                    )
                } else {
                    format!("STOP: {}", promotion_failures.join("; "))
                },
            },
            Some(measurements),
        ),
        ServingEvalMode::FullCensus if !planted_controls_available => (
            EvaluationMode::FullCensus,
            QualityVerdict::Unavailable {
                reason:
                    "required sections-absent and label-shuffled planted controls are unavailable"
                        .to_string(),
            },
            None,
        ),
        ServingEvalMode::FullCensus => {
            let verdict = if promotion_failures.is_empty() {
                QualityVerdict::Pass
            } else {
                QualityVerdict::Fail {
                    reason: promotion_failures.join("; "),
                }
            };
            (EvaluationMode::FullCensus, verdict, Some(measurements))
        }
    };
    let report = DeployedQualityReport {
        schema: DEPLOYED_QUALITY_REPORT_SCHEMA,
        profile: QualityProfileIdentity {
            id: DEPLOYED_QUALITY_PROFILE_ID.to_string(),
            version: DEPLOYED_QUALITY_PROFILE_VERSION,
            execution_scope: NORMATIVE_EXECUTION_SCOPE.to_string(),
        },
        bindings,
        evaluation: EvaluationEvidence {
            mode,
            population_size: row.population_n as u64,
            evaluated_positions: row.sample_n as u64,
            verdict,
            measurements,
        },
        witness_replay,
        negative_controls: vec![label_control],
    };
    if let Some(error) = report.validate_for_research() {
        return Err(SourceUnavailable::new(format!(
            "constructed deployed-quality report failed validation: {error}"
        )));
    }
    Ok(report)
}

struct PromotionGateInputs<'a> {
    row: &'a ServingEvalRow,
    measurements: &'a QualityMeasurements,
    label_control: &'a NegativeControlEvidence,
    planted_controls_available: bool,
    external_cross_surface_checks: u64,
    combined_cross_surface_mismatches: u64,
    witness_replayed: u64,
    witness_failures: u64,
}

fn promotion_failures(inputs: PromotionGateInputs<'_>) -> Vec<String> {
    let PromotionGateInputs {
        row,
        measurements,
        label_control,
        planted_controls_available,
        external_cross_surface_checks,
        combined_cross_surface_mismatches,
        witness_replayed,
        witness_failures,
    } = inputs;
    let mut failures = Vec::new();
    if row.lane_reachable == 0 {
        failures.push("SKMX/PSIB lane is unreachable on the evaluated positions".to_string());
    }
    if measurements.versus_tla.interval.lower_delta_ppm < 0 {
        failures.push("paired lower bound versus TLA is below zero".to_string());
    }
    if !planted_controls_available {
        failures.push(
            "required sections-absent and label-shuffled planted controls are unavailable"
                .to_string(),
        );
    } else {
        let expected_internal_checks = u64::try_from(row.sample_n).ok();
        if expected_internal_checks != Some(row.internal_base_control_checks)
            || row.internal_base_control_mismatches != 0
        {
            failures.push(format!(
                "internal sections-absent identity covered {}/{} evaluated positions with {} mismatches",
                row.internal_base_control_checks,
                row.sample_n,
                row.internal_base_control_mismatches
            ));
        }
        if measurements.versus_sections_absent.interval.lower_delta_ppm < RF31_MIN_LANE_DELTA_PPM {
            failures.push(format!(
                "paired lane lower bound is below {RF31_MIN_LANE_DELTA_PPM} ppm"
            ));
        }
        if label_control.verdict != NegativeControlVerdict::Passed
            || label_control
                .comparison
                .as_ref()
                .is_none_or(|comparison| comparison.delta.numerator > 0)
        {
            failures.push("label-shuffled planted control did not pass".to_string());
        }
    }
    if external_cross_surface_checks == 0 || combined_cross_surface_mismatches != 0 {
        failures.push(format!(
            "external cross-surface evidence has {external_cross_surface_checks} checks; combined evidence has {combined_cross_surface_mismatches} mismatches"
        ));
    }
    if witness_replayed == 0 || witness_failures != 0 {
        failures.push(format!(
            "witness replay has {witness_replayed} replayed and {witness_failures} failures"
        ));
    }
    failures
}

/// Run the evaluator with a durable append-only progress stream, then write a
/// deterministic report and a terminal summary. Errors and budget skips still
/// receive a terminal record; they never emit a production-admissible PASS.
pub fn evaluate_serving_bundle_recorded(
    bundle: &ServingBundle,
    budgets: ServingEvalBudgets,
    evidence: &ServingReportEvidence,
    paths: &ServingReportPaths,
    progress: &mut dyn FnMut(ServingProgress),
) -> Result<RecordedServingEval, SourceUnavailable> {
    let snapshot = ServingBundleSnapshot::capture(bundle)?;
    evaluate_serving_snapshot_recorded(&snapshot, budgets, evidence, paths, progress)
}

/// Recorded evaluation over one caller-captured immutable generation. This is
/// the production orchestration seam: witness creation, measurement, report
/// construction, and terminal evidence can all share the same snapshot.
pub fn evaluate_serving_snapshot_recorded(
    snapshot: &ServingBundleSnapshot,
    budgets: ServingEvalBudgets,
    evidence: &ServingReportEvidence,
    paths: &ServingReportPaths,
    progress: &mut dyn FnMut(ServingProgress),
) -> Result<RecordedServingEval, SourceUnavailable> {
    let mut progress_file = create_progress_file(&paths.progress_jsonl)?;
    let cross_surface_evidence = match validate_snapshot_cross_surface_evidence(snapshot, evidence)
    {
        Ok(evidence) => evidence,
        Err(error) => {
            write_terminal(
                &paths.terminal_json,
                serde_json::json!({
                    "schema": REPORT_TERMINAL_SCHEMA,
                    "status": "unavailable",
                    "reason": error.to_string(),
                    "generation_cid": snapshot.generation_cid,
                    "evidence_hooks": terminal_evidence_hooks(evidence, None, None, None),
                    "report_emitted": false,
                }),
            )?;
            return Err(error);
        }
    };
    let evidence_hooks =
        terminal_evidence_hooks(evidence, Some(&cross_surface_evidence), None, None);
    let mut progress_error: Option<String> = None;
    let outcome = {
        let mut recorded_progress = |state: ServingProgress| {
            progress(state);
            if progress_error.is_none() {
                if let Err(error) = append_progress(&mut progress_file, &state) {
                    progress_error = Some(error.to_string());
                }
            }
        };
        evaluate_serving_snapshot(snapshot, budgets, &mut recorded_progress)
    };
    if let Some(error) = progress_error {
        write_terminal(
            &paths.terminal_json,
            serde_json::json!({
                "schema": REPORT_TERMINAL_SCHEMA,
                "status": "unavailable",
                "reason": format!("durable progress write failed: {error}"),
                "generation_cid": snapshot.generation_cid,
                "evidence_hooks": evidence_hooks,
                "report_emitted": false,
            }),
        )?;
        return Err(SourceUnavailable::new(format!(
            "durable progress write failed: {error}"
        )));
    }
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(error) => {
            write_terminal(
                &paths.terminal_json,
                serde_json::json!({
                    "schema": REPORT_TERMINAL_SCHEMA,
                    "status": "unavailable",
                    "reason": error.to_string(),
                    "generation_cid": snapshot.generation_cid,
                    "evidence_hooks": evidence_hooks,
                    "report_emitted": false,
                }),
            )?;
            return Err(error);
        }
    };
    match &outcome {
        ServingEvalOutcome::Skipped(skip) => {
            write_terminal(
                &paths.terminal_json,
                serde_json::json!({
                    "schema": REPORT_TERMINAL_SCHEMA,
                    "status": "skipped",
                    "reason": skip_reason(skip),
                    "generation_cid": snapshot.generation_cid,
                    "evidence_hooks": evidence_hooks,
                    "report_emitted": false,
                }),
            )?;
            Ok(RecordedServingEval {
                outcome,
                report: None,
                report_cid: None,
            })
        }
        ServingEvalOutcome::Row(row) => {
            let report = match build_deployed_quality_report_from_snapshot(snapshot, row, evidence)
            {
                Ok(report) => report,
                Err(error) => {
                    write_terminal(
                        &paths.terminal_json,
                        serde_json::json!({
                            "schema": REPORT_TERMINAL_SCHEMA,
                            "status": "unavailable",
                            "reason": error.to_string(),
                            "generation_cid": snapshot.generation_cid,
                            "evaluation": row,
                            "evidence_hooks": evidence_hooks,
                            "report_emitted": false,
                        }),
                    )?;
                    return Err(error);
                }
            };
            let report_bytes = report
                .deterministic_json_bytes()
                .map_err(|error| SourceUnavailable::new(format!("serialize report: {error}")))?;
            write_atomic(&paths.deployed_quality_json, &report_bytes)?;
            let report_cid = format!("blake3:{}", blake3::hash(&report_bytes).to_hex());
            let report_cross_surface_evidence_cid = report
                .evaluation
                .measurements
                .as_ref()
                .map(|measurements| measurements.cross_surface_evidence_cid.as_str());
            let evidence_hooks = terminal_evidence_hooks(
                evidence,
                Some(&cross_surface_evidence),
                report_cross_surface_evidence_cid,
                Some(&report.witness_replay),
            );
            let terminal_status = if matches!(
                report.evaluation.verdict,
                QualityVerdict::Unavailable { .. }
            ) {
                "unavailable"
            } else {
                "complete"
            };
            write_terminal(
                &paths.terminal_json,
                serde_json::json!({
                    "schema": REPORT_TERMINAL_SCHEMA,
                    "status": terminal_status,
                    "generation_cid": snapshot.generation_cid,
                    "evaluation": row,
                    "quality_verdict": &report.evaluation.verdict,
                    "report_path": paths.deployed_quality_json,
                    "report_cid": report_cid,
                    "evidence_hooks": evidence_hooks,
                    "report_emitted": true,
                }),
            )?;
            Ok(RecordedServingEval {
                outcome,
                report: Some(report),
                report_cid: Some(report_cid),
            })
        }
    }
}

fn terminal_evidence_hooks(
    evidence: &ServingReportEvidence,
    cross_surface_evidence: Option<&CrossSurfaceParityEvidence>,
    report_cross_surface_evidence_cid: Option<&str>,
    witness_replay: Option<&WitnessReplayEvidence>,
) -> serde_json::Value {
    let external_cross_surface_evidence_cid = report_tagged_cid(
        b"r4-deployed-quality-external-cross-surface-evidence/1",
        &[&evidence.cross_surface_evidence],
    );
    let external_witness_replay_evidence_cid = report_tagged_cid(
        b"r4-deployed-quality-external-witness-replay-evidence/1",
        &[&evidence.witness_replay_evidence],
    );
    serde_json::json!({
        "compiler_revision": evidence.compiler_revision,
        "external_cross_surface_evidence_cid": external_cross_surface_evidence_cid,
        "report_cross_surface_evidence_cid": report_cross_surface_evidence_cid,
        "cross_surface_validated": cross_surface_evidence.is_some(),
        "cross_surface_graph_cid": cross_surface_evidence.map(|artifact| artifact.graph_cid.as_str()),
        "cross_surface_signature_artifact_cid": cross_surface_evidence.map(|artifact| artifact.signature_artifact_cid.as_str()),
        "cross_surface_tokenizer_cid": cross_surface_evidence.and_then(|artifact| artifact.tokenizer_cid.as_deref()),
        "cross_surface_checks": cross_surface_evidence.map(|artifact| artifact.checks),
        "cross_surface_mismatches": cross_surface_evidence.map(|artifact| artifact.mismatches),
        "external_witness_replay_evidence_cid": external_witness_replay_evidence_cid,
        "witness_validated": witness_replay.is_some(),
        "witness_sample_cid": witness_replay.map(|artifact| artifact.sample_cid.as_str()),
        "witness_requested": witness_replay.map(|artifact| artifact.requested),
        "witness_replayed": witness_replay.map(|artifact| artifact.replayed),
        "witness_failures": witness_replay.map(|artifact| artifact.failures),
    })
}

fn validate_snapshot_cross_surface_evidence(
    snapshot: &ServingBundleSnapshot,
    evidence: &ServingReportEvidence,
) -> Result<CrossSurfaceParityEvidence, SourceUnavailable> {
    let tokenizer = required_snapshot_component("tokenizer", snapshot.tokenizer.as_deref())?;
    let score_report =
        required_snapshot_component("score report", snapshot.score_report.as_deref())?;
    evidence.validated_cross_surface_evidence(
        &snapshot.graph,
        &snapshot.teacher,
        tokenizer,
        score_report,
    )
}

fn paired_comparison(
    id: &str,
    version: &str,
    definition_cid: String,
    positions_cid: &str,
    counts: PairedCounts,
) -> Result<PairedComparison, SourceUnavailable> {
    let counts = QualityPairedCounts {
        both_correct: counts.both,
        selector_only_correct: counts.normative_only,
        comparator_only_correct: counts.comparator_only,
        neither_correct: counts.neither,
    };
    let denominator = counts
        .both_correct
        .checked_add(counts.selector_only_correct)
        .and_then(|value| value.checked_add(counts.comparator_only_correct))
        .and_then(|value| value.checked_add(counts.neither_correct))
        .ok_or_else(|| SourceUnavailable::new("paired counts overflow"))?;
    if denominator == 0 {
        return Err(SourceUnavailable::new("paired comparison is empty"));
    }
    let selector_hits = counts
        .both_correct
        .checked_add(counts.selector_only_correct)
        .ok_or_else(|| SourceUnavailable::new("selector hit count overflow"))?;
    let comparator_hits = counts
        .both_correct
        .checked_add(counts.comparator_only_correct)
        .ok_or_else(|| SourceUnavailable::new("comparator hit count overflow"))?;
    let delta_numerator = i64::try_from(counts.selector_only_correct)
        .ok()
        .and_then(|selector| {
            i64::try_from(counts.comparator_only_correct)
                .ok()
                .and_then(|comparator| selector.checked_sub(comparator))
        })
        .ok_or_else(|| SourceUnavailable::new("paired delta exceeds i64"))?;
    let exact_rate = |numerator: u64| ExactRate {
        numerator,
        denominator,
        ppm: ((u128::from(numerator) * 1_000_000) / u128::from(denominator)) as u32,
    };
    let delta = ExactSignedRate {
        numerator: delta_numerator,
        denominator,
        ppm: ((i128::from(delta_numerator) * 1_000_000) / i128::from(denominator)) as i64,
    };
    let interval = PairedInterval::from_counts(counts).ok_or_else(|| {
        SourceUnavailable::new("paired counts cannot produce the fixed-point interval")
    })?;
    Ok(PairedComparison {
        comparator: ComparatorIdentity {
            id: id.to_string(),
            version: version.to_string(),
            definition_cid,
            positions_cid: positions_cid.to_string(),
        },
        counts,
        selector_rate: exact_rate(selector_hits),
        comparator_rate: exact_rate(comparator_hits),
        delta,
        interval,
    })
}

fn usize_positions_to_u64(positions: &[usize]) -> Result<Vec<u64>, SourceUnavailable> {
    positions
        .iter()
        .map(|&position| {
            u64::try_from(position)
                .map_err(|_| SourceUnavailable::new("corpus position exceeds u64"))
        })
        .collect()
}

fn required_snapshot_component<'a>(
    field: &str,
    bytes: Option<&'a [u8]>,
) -> Result<&'a [u8], SourceUnavailable> {
    bytes.ok_or_else(|| {
        SourceUnavailable::new(format!(
            "deployed-quality {field} is UNAVAILABLE in captured generation"
        ))
    })
}

fn serving_generation_cid(components: &[(&str, Option<&[u8]>)]) -> String {
    let mut hasher = blake3::Hasher::new();
    let tag = b"r4-deployed-quality-serving-generation/1";
    hasher.update(&(tag.len() as u64).to_le_bytes());
    hasher.update(tag);
    for (name, bytes) in components {
        hasher.update(&(name.len() as u64).to_le_bytes());
        hasher.update(name.as_bytes());
        match bytes {
            Some(bytes) => {
                hasher.update(&[1]);
                hasher.update(&(bytes.len() as u64).to_le_bytes());
                hasher.update(bytes);
            }
            None => {
                hasher.update(&[0]);
            }
        }
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn report_tagged_cid(tag: &[u8], parts: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&(tag.len() as u64).to_le_bytes());
    hasher.update(tag);
    for part in parts {
        hasher.update(&(part.len() as u64).to_le_bytes());
        hasher.update(part);
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn validate_planted_control_graphs(
    main: &[u8],
    sections_absent: Option<&[u8]>,
    label_shuffled: Option<&[u8]>,
) -> Result<(), SourceUnavailable> {
    use uor_r4_graph_format::SectionId;

    let main_view = parse_control_graph("main", main)?;
    if main_view.section(SectionId::SKMX).is_none() || main_view.section(SectionId::PSIB).is_none()
    {
        return Err(SourceUnavailable::new(
            "main graph lacks required SKMX/PSIB sections",
        ));
    }
    if let Some(bytes) = sections_absent {
        let absent_view = parse_control_graph("sections-absent", bytes)?;
        if absent_view.section(SectionId::SKMX).is_some()
            || absent_view.section(SectionId::PSIB).is_some()
        {
            return Err(SourceUnavailable::new(
                "sections-absent control still contains SKMX or PSIB",
            ));
        }
        require_non_lane_identity(&main_view, &absent_view, "sections-absent")?;
    }
    if let Some(bytes) = label_shuffled {
        let shuffled_view = parse_control_graph("label-shuffled", bytes)?;
        if shuffled_view.section(SectionId::SKMX).is_none()
            || shuffled_view.section(SectionId::PSIB).is_none()
        {
            return Err(SourceUnavailable::new(
                "label-shuffled control lacks SKMX or PSIB",
            ));
        }
        require_non_lane_identity(&main_view, &shuffled_view, "label-shuffled")?;
        let lane_differs = [SectionId::SKMX, SectionId::PSIB]
            .iter()
            .any(|&id| main_view.section(id) != shuffled_view.section(id));
        if !lane_differs {
            return Err(SourceUnavailable::new(
                "label-shuffled control is byte-identical to the main lane",
            ));
        }
    }
    Ok(())
}

fn parse_control_graph<'a>(
    label: &str,
    bytes: &'a [u8],
) -> Result<uor_r4_graph_format::GraphView<'a>, SourceUnavailable> {
    let view = uor_r4_graph_format::GraphView::parse(bytes)
        .map_err(|error| SourceUnavailable::new(format!("{label} graph parse failed: {error}")))?;
    view.verify_cids().map_err(|error| {
        SourceUnavailable::new(format!("{label} graph CID check failed: {error}"))
    })?;
    Ok(view)
}

fn require_non_lane_identity(
    main: &uor_r4_graph_format::GraphView<'_>,
    control: &uor_r4_graph_format::GraphView<'_>,
    label: &str,
) -> Result<(), SourceUnavailable> {
    use uor_r4_graph_format::SectionId;

    for section in main.sections() {
        if section.id == SectionId::SKMX || section.id == SectionId::PSIB {
            continue;
        }
        let Some(control_section) = control.sections().find(|row| row.id == section.id) else {
            return Err(SourceUnavailable::new(format!(
                "{label} control is missing non-lane section 0x{:08x}",
                section.id.raw()
            )));
        };
        if control_section.flags != section.flags || control_section.payload != section.payload {
            return Err(SourceUnavailable::new(format!(
                "{label} control changes non-lane section 0x{:08x}",
                section.id.raw()
            )));
        }
    }
    for section in control.sections() {
        if section.id == SectionId::SKMX || section.id == SectionId::PSIB {
            continue;
        }
        if main.section(section.id).is_none() {
            return Err(SourceUnavailable::new(format!(
                "{label} control adds non-lane section 0x{:08x}",
                section.id.raw()
            )));
        }
    }
    Ok(())
}

fn create_progress_file(path: &Path) -> Result<File, SourceUnavailable> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            SourceUnavailable::new(format!(
                "create progress directory {}: {error}",
                parent.display()
            ))
        })?;
    }
    OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .map_err(|error| {
            SourceUnavailable::new(format!(
                "open progress artifact {}: {error}",
                path.display()
            ))
        })
}

fn append_progress(file: &mut File, state: &ServingProgress) -> Result<(), std::io::Error> {
    let record = serde_json::json!({
        "schema": REPORT_PROGRESS_SCHEMA,
        "progress": state,
    });
    serde_json::to_writer(&mut *file, &record).map_err(std::io::Error::other)?;
    file.write_all(b"\n")?;
    file.flush()?;
    file.sync_data()
}

fn write_terminal(path: &Path, value: serde_json::Value) -> Result<(), SourceUnavailable> {
    let mut bytes = serde_json::to_vec_pretty(&value)
        .map_err(|error| SourceUnavailable::new(format!("serialize terminal record: {error}")))?;
    bytes.push(b'\n');
    write_atomic(path, &bytes)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), SourceUnavailable> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            SourceUnavailable::new(format!(
                "create artifact directory {}: {error}",
                parent.display()
            ))
        })?;
    }
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("artifact");
    let temporary = path.with_extension(format!("{extension}.tmp-{}", std::process::id()));
    let write_result = (|| {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.flush()?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)
    })();
    if let Err(error) = write_result {
        let _ = std::fs::remove_file(&temporary);
        return Err(SourceUnavailable::new(format!(
            "write durable artifact {}: {error}",
            path.display()
        )));
    }
    Ok(())
}

fn skip_reason(skip: &ServingEvalSkip) -> String {
    match skip {
        ServingEvalSkip::ProbeBudgetExceeded { probed, elapsed } => format!(
            "probe budget exceeded after {probed} positions and {} ms",
            elapsed.as_millis()
        ),
        ServingEvalSkip::ProbeFunctionalCheckFailed { served, probed } => format!(
            "probe functional check failed: {served} served across {probed} positions with zero hits"
        ),
        ServingEvalSkip::EvalBudgetExceeded {
            done,
            sample_n,
            elapsed,
        } => format!(
            "evaluation budget exceeded after {done}/{sample_n} positions and {} ms; partial counts discarded",
            elapsed.as_millis()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_requires_all_bundle_files() {
        let root = std::env::temp_dir().join(format!("r4-serving-eval-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("graph")).unwrap();
        assert!(ServingBundle::discover(&root).is_none());
        for name in [
            "tless_artifacts.bin",
            "tless_store.bin",
            "corpus.meta",
            "corpus.records",
        ] {
            std::fs::write(root.join(name), b"x").unwrap();
        }
        assert!(
            ServingBundle::discover(&root).is_none(),
            "no graph artifact yet"
        );
        std::fs::write(root.join("graph").join("score.r4g1"), b"x").unwrap();
        let bundle = ServingBundle::discover(&root).expect("bundle");
        assert_eq!(bundle.graph, root.join("graph").join("score.r4g1"));
        assert_eq!(bundle.teacher, root.join("tless_artifacts.bin"));
        assert_eq!(bundle.store, root.join("tless_store.bin"));
        assert!(bundle.sections_absent_graph.is_none());
        assert!(bundle.label_shuffled_graph.is_none());
        std::fs::write(
            root.join("graph").join("score_sections_absent.r4g1"),
            b"control",
        )
        .unwrap();
        std::fs::write(
            root.join("graph").join("score_label_shuffled.r4g1"),
            b"control",
        )
        .unwrap();
        let with_controls = ServingBundle::discover(&root).expect("bundle with controls");
        assert!(with_controls.sections_absent_graph.is_some());
        assert!(with_controls.label_shuffled_graph.is_some());
        let first = ServingBundleSnapshot::capture(&with_controls).expect("capture generation");
        assert_eq!(first.generation_cid().len(), "blake3:".len() + 64);
        std::fs::write(root.join("tless_store.bin"), b"changed").unwrap();
        let second = ServingBundleSnapshot::capture(&with_controls).expect("recapture generation");
        assert_ne!(
            first.generation_cid(),
            second.generation_cid(),
            "a comparator-byte change must create a different generation"
        );
        let error = second
            .require_generation(first.generation_cid())
            .expect_err("a changed full-census capture must be refused");
        assert!(error.to_string().contains("full census was not launched"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn abstain_breakdown_records_by_status() {
        let mut b = StatusBreakdown::default();
        b.record(PolicyStatus::Novel, false);
        b.record(PolicyStatus::Novel, false);
        b.record(PolicyStatus::Graph, false);
        assert_eq!(b.novel, 2);
        assert_eq!(b.graph, 1);
        assert_eq!(b.total(), 3);
    }

    /// #362 attribution: the NGRAM subcount tracks only exact-context
    /// records, and the flag is ignored on other statuses.
    #[test]
    fn breakdown_splits_exact_context_by_ngram() {
        let mut b = StatusBreakdown::default();
        b.record(PolicyStatus::ExactContext, true);
        b.record(PolicyStatus::ExactContext, false);
        b.record(PolicyStatus::Graph, true);
        assert_eq!(b.exact_context, 2);
        assert_eq!(b.exact_context_ngram, 1);
        assert_eq!(b.graph, 1);
        assert_eq!(b.total(), 3, "the ngram split is not a fourth bucket");
    }

    #[test]
    fn sample_is_the_exact_ascending_certification_prefix() {
        let held_out: Vec<usize> = (100..1_100).collect();
        let selected = select_positions(&held_out, ServingEvalMode::Sample { positions: 7 });
        assert_eq!(selected, held_out[..7]);
    }

    #[test]
    fn durable_progress_is_newline_delimited_and_parseable() {
        let root = std::env::temp_dir().join(format!("r4-serving-progress-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let path = root.join("progress.jsonl");
        let mut file = create_progress_file(&path).expect("progress file");
        let progress = ServingProgress {
            phase: "evaluate",
            processed: 256,
            total: 1_000,
            served: 200,
            abstained: 50,
            declined: 6,
            normative_hits: 75,
            tla_hits: 70,
            lane_reachable: 40,
            lane_changed: 20,
            lane_toward: 15,
            lane_away: 5,
            sections_absent_hits: 60,
            label_shuffled_hits: 55,
            internal_base_control_checks: 256,
            internal_base_control_mismatches: 0,
            planted_controls_available: true,
            elapsed_millis: 1_000,
            positions_per_second_milli: 256_000,
            eta_seconds: Some(3),
            workers: 8,
        };
        append_progress(&mut file, &progress).expect("append");
        drop(file);
        let bytes = std::fs::read(&path).expect("read");
        assert_eq!(bytes.last(), Some(&b'\n'));
        let line = bytes.strip_suffix(b"\n").expect("newline");
        let value: serde_json::Value = serde_json::from_slice(line).expect("JSON line");
        assert_eq!(value["schema"], REPORT_PROGRESS_SCHEMA);
        assert_eq!(value["progress"]["workers"], 8);
        let _ = std::fs::remove_dir_all(&root);
    }

    fn gate_comparison(lower_delta_ppm: i64, delta_numerator: i64) -> PairedComparison {
        let counts = QualityPairedCounts {
            both_correct: 70,
            selector_only_correct: u64::try_from(delta_numerator.max(0)).unwrap(),
            comparator_only_correct: u64::try_from((-delta_numerator).max(0)).unwrap(),
            neither_correct: 30,
        };
        let denominator = counts.both_correct
            + counts.selector_only_correct
            + counts.comparator_only_correct
            + counts.neither_correct;
        PairedComparison {
            comparator: ComparatorIdentity {
                id: "fixture".to_owned(),
                version: "1".to_owned(),
                definition_cid: "blake3:fixture".to_owned(),
                positions_cid: "blake3:positions".to_owned(),
            },
            counts,
            selector_rate: ExactRate {
                numerator: 70 + counts.selector_only_correct,
                denominator,
                ppm: 0,
            },
            comparator_rate: ExactRate {
                numerator: 70 + counts.comparator_only_correct,
                denominator,
                ppm: 0,
            },
            delta: ExactSignedRate {
                numerator: delta_numerator,
                denominator,
                ppm: 0,
            },
            interval: PairedInterval {
                method: "fixture".to_owned(),
                confidence_ppm: 950_000,
                lower_delta_ppm,
                estimate_delta_ppm: delta_numerator,
                upper_delta_ppm: lower_delta_ppm.saturating_add(1),
            },
        }
    }

    fn gate_row() -> ServingEvalRow {
        ServingEvalRow {
            bundle: PathBuf::from("fixture"),
            generation_cid: "blake3:generation".to_owned(),
            sample_n: SAMPLE_TARGET,
            population_n: 72_130,
            mode: ServingEvalMode::Sample {
                positions: SAMPLE_TARGET,
            },
            workers: 8,
            elapsed_millis: 1,
            served: SAMPLE_TARGET as u64,
            served_widened: 0,
            served_by: StatusBreakdown::default(),
            abstained: StatusBreakdown::default(),
            top1_served: 0,
            agree_served: 0,
            tla_hits: 0,
            base_hits: 0,
            normative_vs_tla: PairedCounts::default(),
            normative_vs_base: PairedCounts::default(),
            normative_vs_sections_absent: Some(PairedCounts::default()),
            label_shuffled_vs_sections_absent: Some(PairedCounts::default()),
            sections_absent_hits: 0,
            label_shuffled_hits: 0,
            internal_base_control_checks: SAMPLE_TARGET as u64,
            internal_base_control_mismatches: 0,
            lane_reachable: 100,
            lane_changed: 100,
            lane_toward: 100,
            lane_away: 0,
            declined: 0,
            probe_positions: 256,
            probe_served: 256,
            probe_hits: 1,
            evaluated_positions_cid: "blake3:evaluated".to_owned(),
            population_positions_cid: "blake3:population".to_owned(),
        }
    }

    #[test]
    fn binding_sample_gate_proceeds_only_when_every_falsifier_passes() {
        let row = gate_row();
        let measurements = QualityMeasurements {
            versus_tla: gate_comparison(0, 1),
            versus_sections_absent: gate_comparison(RF31_MIN_LANE_DELTA_PPM, 1),
            internal_base_control_checks: SAMPLE_TARGET as u64,
            internal_base_control_mismatches: 0,
            cross_surface_checks: SAMPLE_TARGET as u64 + 6,
            cross_surface_mismatches: 0,
            cross_surface_evidence_cid: "blake3:evidence".to_owned(),
        };
        let label_control = NegativeControlEvidence {
            id: LABEL_SHUFFLED_CONTROL_ID.to_owned(),
            identity_cid: "blake3:control".to_owned(),
            verdict: NegativeControlVerdict::Passed,
            comparison: Some(gate_comparison(-1, 0)),
        };
        assert!(promotion_failures(PromotionGateInputs {
            row: &row,
            measurements: &measurements,
            label_control: &label_control,
            planted_controls_available: true,
            external_cross_surface_checks: 6,
            combined_cross_surface_mismatches: 0,
            witness_replayed: 64,
            witness_failures: 0,
        })
        .is_empty());
    }

    #[test]
    fn binding_sample_gate_stops_on_reachability_quality_and_replay_failures() {
        let mut row = gate_row();
        row.lane_reachable = 0;
        row.internal_base_control_checks -= 1;
        row.internal_base_control_mismatches = 1;
        let measurements = QualityMeasurements {
            versus_tla: gate_comparison(-1, 0),
            versus_sections_absent: gate_comparison(RF31_MIN_LANE_DELTA_PPM - 1, 0),
            internal_base_control_checks: row.internal_base_control_checks,
            internal_base_control_mismatches: row.internal_base_control_mismatches,
            cross_surface_checks: row.internal_base_control_checks,
            cross_surface_mismatches: row.internal_base_control_mismatches,
            cross_surface_evidence_cid: "blake3:evidence".to_owned(),
        };
        let label_control = NegativeControlEvidence {
            id: LABEL_SHUFFLED_CONTROL_ID.to_owned(),
            identity_cid: "blake3:control".to_owned(),
            verdict: NegativeControlVerdict::Failed,
            comparison: Some(gate_comparison(1, 1)),
        };
        let failures = promotion_failures(PromotionGateInputs {
            row: &row,
            measurements: &measurements,
            label_control: &label_control,
            planted_controls_available: true,
            external_cross_surface_checks: 0,
            combined_cross_surface_mismatches: 1,
            witness_replayed: 0,
            witness_failures: 1,
        });
        for expected in [
            "unreachable",
            "versus TLA",
            "paired lane lower bound",
            "sections-absent identity",
            "label-shuffled",
            "cross-surface",
            "witness replay",
        ] {
            assert!(
                failures.iter().any(|failure| failure.contains(expected)),
                "missing {expected:?} from {failures:?}"
            );
        }
    }
}
