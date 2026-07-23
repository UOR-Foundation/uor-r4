//! Hidden-state trace lane for cover induction (graph-compiler plan §5
//! Phase 2, issue #71): a second observation lane beside the deterministic
//! context-bundle lane of [`cover::build_observations`], sourced from the
//! [`TeacherOracle::hidden_state`] trace surface, plus the lane-comparison
//! driver that measures both lanes at one fixed region budget and records
//! an adoption decision.
//!
//! # The lane
//!
//! [`build_trace_observations`] replays the corpus through the oracle in
//! story order — `reset` at each story boundary, then
//! `step(input[i], pos_in_story)` per position — and reads the final
//! hidden state (post-final-rmsnorm, pre-classifier activation) after each
//! step. The replay is strictly sequential in corpus position order, so it
//! is bit-reproducible on the same machine by construction (no
//! parallelism, no RNG, no clock; the κ-baseline macOS-pinned caveat
//! below applies to the f32 values themselves).
//!
//! The produced [`Observation`] rows keep `position`, `sample`
//! ([`observe::sample_id`] of the same context window), and `next`
//! **identical to the bundle lane**, so [`cover::split_positions`],
//! entropy gating, reference recall, and Gate C align 1:1 across lanes —
//! only the vector and signature change:
//!
//! - `vector`: the L2-normalized hidden state (the k-means input).
//! - `sig`: sign bits of the raw hidden state, bit `d` set iff coordinate
//!   `d > 0.0` — exactly the [`cover::binarize_prototype`] convention.
//!   Unlike the bundle lane there is no per-dimension thresholds
//!   centering: for hidden states, **sign vs 0.0 IS the centering** (the
//!   final rmsnorm removes the scale/offset freedom the thresholds
//!   calibrate in the bundle lane). L2 normalization divides by a positive
//!   scalar, so the sign bits are identical taken before or after
//!   normalization.
//!
//! **Dimension gate (v1):** the lane requires
//! `oracle.source_dimension() == compiler::D` (the legacy stories15M
//! teacher, dim 288) so the hidden-state signature drops straight into
//! the same 288-bit Hamming membership path as the bundle lane. Oracles
//! with any other width (e.g. the raw 576-dim Hugging Face teacher) are
//! rejected with a plain error — no silent truncation or projection.
//!
//! Everything downstream is reused untouched: [`cover::induce_cover`],
//! [`cover::ReferenceClassifier`], [`cover::evaluate_held_out`],
//! [`cover::build_edges`], the score compiler, and
//! [`cover_sweep::run_point`] are all lane-agnostic given observations —
//! Gate C for each lane routes with that lane's own sigs because
//! `run_point` feeds the lane's observation rows into
//! `score::evaluate_gate_c` (the `score --cover` CLI is bundle-hardcoded
//! and is NOT used here).
//!
//! # The comparison
//!
//! [`run_compare`] builds the hidden-lane observations over one fixed
//! observation sample (the CLI uses the full pinned fixture corpus: every
//! position in file order, split into train/held-out by the usual 80/20
//! story cut) and runs ONE fixed region-budget point — the confirmed
//! default operating point `k0 = 8`, entropy gain 0.25 bits, regions
//! budget 256 (the 42-region default row of the issue-#70 sweep) — on
//! both lanes through [`cover_sweep::run_point`]. The report holds, per
//! lane, the per-depth reference top-1/top-M recall and frontier width,
//! the scored artifact bytes, and the Gate C Rule 1+2 top-1 agreement and
//! bits/token, plus the TLA3 store baseline row. The baseline is
//! inherently bundle-lane (it routes `runtime::assign_plain` class codes,
//! not cover regions); it is recorded once as a shared reference row and
//! labeled as such.
//!
//! # The recorded lane choice
//!
//! The adoption rule is deterministic and stated in the report:
//!
//! > adopt the hidden lane iff its Gate C Rule 1+2 top-1 agreement
//! > STRICTLY exceeds the bundle lane's at the same fixed point; ties and
//! > regressions keep the deterministic bundle lane.
//!
//! [`LaneChoice`] records the decision, the rule, the justifying deltas
//! (top-1 agreement, bits/token, deepest-depth reference recall), and a
//! written rationale. On "adopted", plan §4.1 requires an i8/sign
//! quantized spill for the hidden-state vectors (per-vector max-abs
//! scale, ~0.3 KB/obs) before the lane ships; on "rejected", the
//! comparison table itself is the rejection evidence and no spill is
//! built.
//!
//! # Report schema (`lane_compare.json`, schema = 1)
//!
//! ```text
//! schema:          1
//! inputs:          {artifact_kappa, corpus_kappa, teacher_kappa, sample,
//!                   stories, positions, train_observations,
//!                   held_out_observations}
//! point:           {label, k0, depths, entropy_gain_bits, regions_budget,
//!                   min_support, memory_budget_bytes}
//! tla3_baseline:   {positions, top1_agreement, bits_per_token}
//!                   (shared reference row — inherently bundle-lane)
//! lanes:           [{lane: "bundle" | "hidden", <SweepRow fields>}]
//! choice:          {decision, lane, rule, delta_rule12_top1,
//!                   delta_bits_per_token, delta_reference_top1_deepest,
//!                   rationale}
//! determinism:     note string
//! ```
//!
//! # Determinism
//!
//! Same-machine double runs are byte-identical (asserted in
//! `tests/trace_lane.rs`): the replay is sequential in fixed story order
//! and every consumed compiler is deterministic by construction. The f32
//! hidden states are libm-sensitive cross-platform — the macOS-pinned
//! status of the κ baseline, inherited here exactly as
//! `cover_sweep.rs:643-649` records it; cross-platform byte equality
//! awaits the D2 canonical deterministic compile mode. This module is
//! compiler-side ONLY: nothing here touches `runtime.rs` (the
//! machine-checked P-4 source scan enforces the mul-free kernel there).

use serde::Serialize;
use std::fmt::Write as _;
use std::path::PathBuf;

use super::compiler::{self, D, SIG_BYTES};
use super::cover::{self, Observation};
use super::cover_sweep::{self, SweepInputs, SweepPoint, SweepRow, SweepRowConfig};
use super::observe;
use super::score::{GateCMetrics, ScoreConfig};
use super::teacher::{LlamaOracle, TeacherOracle};

/// The `lane_compare.json` schema version (module docs).
pub const LANE_COMPARE_SCHEMA: u32 = 1;

/// Lane label of the deterministic context-bundle lane.
pub const LANE_BUNDLE: &str = "bundle";
/// Lane label of the teacher hidden-state trace lane.
pub const LANE_HIDDEN: &str = "hidden";

/// Default source checkpoint for the trace replay (the legacy stories15M
/// teacher — the only in-tree oracle with `source_dimension() == D`).
pub const DEFAULT_TRACE_CHECKPOINT: &str = "/tmp/ref/out/model.bin";

/// The adoption rule, recorded verbatim in every report (module docs).
pub const ADOPTION_RULE: &str = "adopt the hidden lane iff its Gate C Rule 1+2 top-1 agreement \
                                 strictly exceeds the bundle lane's at the same fixed point; \
                                 ties and regressions keep the deterministic bundle lane";

/// The fixed observation sample the CLI compares over (module docs).
pub const SAMPLE_NOTE: &str =
    "the full pinned corpus: every position in file order, train/held-out split by the \
     compiler::train_cut 80/20 story cut (same sample on both lanes)";

/// Build the hidden-state trace observations of the given corpus
/// positions (module docs: the lane). `positions` must be ascending and
/// in range; the replay walks the corpus once in position order, resets
/// at story boundaries, and stops after the last wanted position. The
/// returned rows align 1:1 with [`cover::build_observations`] on
/// `position`, `sample`, and `next` for the same `positions`.
pub fn build_trace_observations(
    oracle: &mut dyn TeacherOracle,
    corpus: &compiler::Corpus,
    positions: &[usize],
) -> Result<Vec<Observation>, String> {
    if oracle.source_dimension() != D {
        return Err(format!(
            "trace lane v1 requires source dimension {D} (compiler::D); this oracle exposes {} \
             — no silent truncation or projection",
            oracle.source_dimension()
        ));
    }
    for (k, &p) in positions.iter().enumerate() {
        if p >= corpus.n {
            return Err(format!(
                "position {p} is out of range (corpus has {})",
                corpus.n
            ));
        }
        if k > 0 && p <= positions[k - 1] {
            return Err("positions must be strictly ascending".to_owned());
        }
    }
    let mut logits = vec![0f32; oracle.vocab()];
    let mut observations = Vec::with_capacity(positions.len());
    let mut wanted = positions.iter().peekable();
    let mut pos_in_story = 0usize;
    oracle.reset();
    for i in 0..corpus.n {
        if i > 0 && corpus.story[i] != corpus.story[i - 1] {
            oracle.reset();
            pos_in_story = 0;
        }
        oracle.step(corpus.input[i] as usize, pos_in_story, &mut logits);
        pos_in_story += 1;
        if wanted.peek() == Some(&&i) {
            wanted.next();
            let hidden = oracle
                .hidden_state()
                .ok_or_else(|| "oracle exposes no hidden_state trace surface".to_owned())?;
            if hidden.len() != D {
                return Err(format!(
                    "hidden_state length {} != compiler::D {D}",
                    hidden.len()
                ));
            }
            let mut sig = [0u8; SIG_BYTES];
            let mut vector = vec![0f32; D];
            let mut nn = 0f32;
            for (d, &x) in hidden.iter().enumerate() {
                if x > 0.0 {
                    sig[d / 8] |= 1 << (d % 8);
                }
                vector[d] = x;
                nn += x * x;
            }
            let nn = nn.sqrt().max(1e-9);
            for x in vector.iter_mut() {
                *x /= nn;
            }
            observations.push(Observation {
                position: i as u32,
                sample: observe::sample_id(&cover::context_window(corpus, i)),
                vector,
                sig,
                next: corpus.next[i],
            });
            if wanted.peek().is_none() {
                break;
            }
        }
    }
    if observations.len() != positions.len() {
        return Err(format!(
            "replay produced {} of {} wanted observations",
            observations.len(),
            positions.len()
        ));
    }
    Ok(observations)
}

/// One lane's comparison row: its label plus the full sweep-point row.
#[derive(Debug, Clone, Serialize)]
pub struct LaneRow {
    /// `bundle` or `hidden`.
    pub lane: String,
    /// The lane's regions × bytes × agreement row (cover_sweep schema).
    #[serde(flatten)]
    pub row: SweepRow,
}

/// The recorded lane choice (module docs for the rule).
#[derive(Debug, Clone, Serialize)]
pub struct LaneChoice {
    /// `adopted` or `rejected`.
    pub decision: String,
    /// The lane the decision keeps in the compiler path.
    pub lane: String,
    /// The adoption rule applied ([`ADOPTION_RULE`]).
    pub rule: String,
    /// hidden − bundle Gate C Rule 1+2 top-1 agreement.
    pub delta_rule12_top1: f64,
    /// hidden − bundle Gate C bits/token (negative is better).
    pub delta_bits_per_token: f64,
    /// hidden − bundle reference top-1 recall at the deepest depth.
    pub delta_reference_top1_deepest: f64,
    /// The written justification (numbers + the rule's application).
    pub rationale: String,
}

/// Shared-input provenance of the comparison.
#[derive(Debug, Clone, Serialize)]
pub struct LaneCompareInputs {
    pub artifact_kappa: String,
    pub corpus_kappa: String,
    /// κ of the teacher checkpoint the hidden lane replayed.
    pub teacher_kappa: String,
    /// The fixed observation sample ([`SAMPLE_NOTE`]).
    pub sample: String,
    pub stories: u64,
    pub positions: usize,
    pub train_observations: usize,
    pub held_out_observations: usize,
}

/// The fixed point's cover configuration columns.
#[derive(Debug, Clone, Serialize)]
pub struct LaneComparePoint {
    pub label: String,
    #[serde(flatten)]
    pub config: SweepRowConfig,
}

/// The `lane_compare.json` document (schema in the module docs).
#[derive(Debug, Clone, Serialize)]
pub struct LaneCompareReport {
    pub schema: u32,
    pub inputs: LaneCompareInputs,
    /// The single fixed region-budget point both lanes ran.
    pub point: LaneComparePoint,
    /// The cover-independent TLA3 store baseline — a shared reference row,
    /// inherently bundle-lane (`assign_plain` class codes), recorded once.
    pub tla3_baseline: GateCMetrics,
    /// One row per lane, bundle first.
    pub lanes: Vec<LaneRow>,
    /// The recorded adoption decision.
    pub choice: LaneChoice,
    /// Determinism status note.
    pub determinism: String,
}

/// The single fixed comparison point: the confirmed default operating
/// point (module docs; the 42-region baseline row of the #70 sweep).
pub fn compare_point() -> SweepPoint {
    let config = cover::CoverConfig::default();
    SweepPoint {
        label: format!(
            "k0={}/gain={}/budget={} (default)",
            config.k0, config.entropy_gain_bits, config.regions_budget
        ),
        baseline: true,
        config,
    }
}

/// Apply the adoption rule ([`ADOPTION_RULE`]) to the two lane rows.
pub fn choose_lane(bundle: &SweepRow, hidden: &SweepRow) -> LaneChoice {
    let delta_top1 = hidden.gate_c_rule12.top1_agreement - bundle.gate_c_rule12.top1_agreement;
    let delta_bits = hidden.gate_c_rule12.bits_per_token - bundle.gate_c_rule12.bits_per_token;
    let deepest = |row: &SweepRow| row.recall.last().map_or(0.0, |d| d.reference_top1);
    let delta_recall = deepest(hidden) - deepest(bundle);
    let adopted = delta_top1 > 0.0;
    let decision = if adopted { "adopted" } else { "rejected" };
    let lane = if adopted { LANE_HIDDEN } else { LANE_BUNDLE };
    let mut rationale = format!(
        "hidden lane Rule 1+2 top-1 {:.4} vs bundle {:.4} (Δ {:+.4}), bits/token Δ {:+.4}, \
         deepest-depth reference top-1 Δ {:+.4}",
        hidden.gate_c_rule12.top1_agreement,
        bundle.gate_c_rule12.top1_agreement,
        delta_top1,
        delta_bits,
        delta_recall
    );
    if adopted {
        rationale.push_str(
            " — the hidden lane strictly beats the bundle lane on the distortion axis at the \
             same region budget: adopted; plan §4.1 i8/sign spill of the hidden-state vectors \
             is now owed before the lane ships",
        );
    } else {
        rationale.push_str(
            " — no strict top-1 gain over the deterministic bundle lane: rejected; the \
             comparison table in this report is the rejection evidence and no i8 spill is built",
        );
    }
    LaneChoice {
        decision: decision.to_owned(),
        lane: lane.to_owned(),
        rule: ADOPTION_RULE.to_owned(),
        delta_rule12_top1: delta_top1,
        delta_bits_per_token: delta_bits,
        delta_reference_top1_deepest: delta_recall,
        rationale,
    }
}

/// Run both lanes at the fixed point over two already-loaded input sets
/// (identical except for the observation rows) and assemble the report.
/// `teacher_kappa` identifies the hidden lane's oracle. Deterministic:
/// two calls with the same inputs produce byte-identical JSON.
pub fn compare_lanes(
    bundle: &SweepInputs,
    hidden: &SweepInputs,
    point: &SweepPoint,
    score_config: &ScoreConfig,
    teacher_kappa: &str,
    sample_note: &str,
) -> Result<LaneCompareReport, String> {
    // 1:1 alignment is the lane contract (module docs): same positions,
    // samples, and next tokens on both lanes.
    for (b, h) in bundle.train.iter().zip(hidden.train.iter()) {
        if b.position != h.position || b.sample != h.sample || b.next != h.next {
            return Err(
                "lane misalignment: train observations differ in position/sample/next".to_owned(),
            );
        }
    }
    for (b, h) in bundle.held_out.iter().zip(hidden.held_out.iter()) {
        if b.position != h.position || b.sample != h.sample || b.next != h.next {
            return Err(
                "lane misalignment: held-out observations differ in position/sample/next"
                    .to_owned(),
            );
        }
    }
    let (bundle_row, tla3_baseline, _bytes) = cover_sweep::run_point(bundle, point, score_config)?;
    let (hidden_row, _tla3_again, _bytes) = cover_sweep::run_point(hidden, point, score_config)?;
    let choice = choose_lane(&bundle_row, &hidden_row);
    Ok(LaneCompareReport {
        schema: LANE_COMPARE_SCHEMA,
        inputs: LaneCompareInputs {
            artifact_kappa: bundle.artifact_kappa.clone(),
            corpus_kappa: bundle.corpus_kappa.clone(),
            teacher_kappa: teacher_kappa.to_owned(),
            sample: sample_note.to_owned(),
            stories: bundle.corpus.stories,
            positions: bundle.corpus.n,
            train_observations: bundle.train.len(),
            held_out_observations: bundle.held_out.len(),
        },
        point: LaneComparePoint {
            label: point.label.clone(),
            config: SweepRowConfig {
                k0: point.config.k0,
                depths: point.config.depths,
                entropy_gain_bits: point.config.entropy_gain_bits,
                regions_budget: point.config.regions_budget,
                min_support: point.config.min_support,
                memory_budget_bytes: point.config.memory_budget_bytes,
            },
        },
        tla3_baseline,
        lanes: vec![
            LaneRow {
                lane: LANE_BUNDLE.to_owned(),
                row: bundle_row,
            },
            LaneRow {
                lane: LANE_HIDDEN.to_owned(),
                row: hidden_row,
            },
        ],
        choice,
        determinism: "same-machine double runs are byte-identical (sequential story-order replay, \
                      content-addressed seeds, ordered reductions; asserted in \
                      tests/trace_lane.rs); the f32 hidden states are macOS-pinned and \
                      libm-sensitive cross-platform — the inherited status of the κ baseline \
                      (D2 resolves cross-platform byte equality later)"
            .to_owned(),
    })
}

/// The full driver: replay the corpus through `oracle` to build the
/// hidden lane over the same positions as the bundle-lane `inputs`, then
/// compare both lanes at the fixed point.
pub fn run_compare(
    inputs: &SweepInputs,
    oracle: &mut dyn TeacherOracle,
    score_config: &ScoreConfig,
) -> Result<LaneCompareReport, String> {
    let corpus = &inputs.corpus;
    let positions: Vec<usize> = (0..corpus.n).collect();
    eprintln!(
        "lane-compare: replaying {} positions through the teacher (hidden-state lane)...",
        corpus.n
    );
    let all = build_trace_observations(oracle, corpus, &positions)?;
    let cut = compiler::train_cut(corpus);
    let (trace_train, trace_held_out): (Vec<Observation>, Vec<Observation>) = all
        .into_iter()
        .partition(|o| corpus.story[o.position as usize] < cut);
    let trace_inputs = SweepInputs {
        artifact_container: inputs.artifact_container.clone(),
        artifacts: inputs.artifacts.clone(),
        corpus: inputs.corpus.clone(),
        meta_bytes: inputs.meta_bytes.clone(),
        recs_bytes: inputs.recs_bytes.clone(),
        train: trace_train,
        held_out: trace_held_out,
        store: inputs.store.clone(),
        tls1: inputs.tls1.clone(),
        artifact_kappa: inputs.artifact_kappa.clone(),
        corpus_kappa: inputs.corpus_kappa.clone(),
    };
    let teacher_kappa = oracle.kappa();
    compare_lanes(
        inputs,
        &trace_inputs,
        &compare_point(),
        score_config,
        &teacher_kappa,
        SAMPLE_NOTE,
    )
}

/// The console comparison table (module docs): per-depth reference recall
/// and frontier width per lane, then the Gate C rows plus the shared TLA3
/// baseline row, then the recorded lane choice.
pub fn render_table(report: &LaneCompareReport) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "lane comparison at {} — {} train / {} held-out observations:",
        report.point.label, report.inputs.train_observations, report.inputs.held_out_observations
    );
    let _ = writeln!(
        out,
        "per-depth reference recall and frontier width (held-out):"
    );
    let _ = writeln!(
        out,
        "  {:<8} {:>5} {:>9} {:>9} {:>9} {:>9}",
        "lane", "depth", "evaluated", "ref-top1", "ref-topM", "frontier"
    );
    for lane in &report.lanes {
        for d in &lane.row.recall {
            let _ = writeln!(
                out,
                "  {:<8} {:>5} {:>9} {:>8.1}% {:>8.1}% {:>4.2}/{}",
                lane.lane,
                d.depth,
                d.evaluated,
                100.0 * d.reference_top1,
                100.0 * d.reference_topm,
                d.frontier_mean,
                d.frontier_max
            );
        }
    }
    let _ = writeln!(out, "Gate C (held-out, Rule 1+2 precedence):");
    let _ = writeln!(
        out,
        "  {:<8} {:>7} {:>10} {:>10} {:>10}",
        "lane", "regions", "bytes", "R1+2 top1", "bits/token"
    );
    for lane in &report.lanes {
        let _ = writeln!(
            out,
            "  {:<8} {:>7} {:>10} {:>9.1}% {:>10.4}",
            lane.lane,
            lane.row.regions.total,
            lane.row.artifact_bytes,
            100.0 * lane.row.gate_c_rule12.top1_agreement,
            lane.row.gate_c_rule12.bits_per_token
        );
    }
    let _ = writeln!(
        out,
        "  {:<8} {:>7} {:>10} {:>9.1}% {:>10.4}   (TLA3 store baseline — bundle-lane class codes; shared reference row)",
        "tla3",
        "-",
        "-",
        100.0 * report.tla3_baseline.top1_agreement,
        report.tla3_baseline.bits_per_token
    );
    let _ = writeln!(
        out,
        "lane choice: {} — keep the {} lane ({})",
        report.choice.decision, report.choice.lane, report.choice.rationale
    );
    out
}

// ------------------------------------------------------------ CLI --------

#[derive(Debug, PartialEq, Eq)]
struct LaneCompareOptions {
    corpus_meta: PathBuf,
    corpus_recs: PathBuf,
    artifacts: PathBuf,
    checkpoint: PathBuf,
    output: PathBuf,
}

fn parse_lane_compare_options(args: &[String]) -> Result<LaneCompareOptions, String> {
    let (default_meta, default_recs) = compiler::corpus_paths();
    let mut options = LaneCompareOptions {
        corpus_meta: PathBuf::from(default_meta),
        corpus_recs: PathBuf::from(default_recs),
        artifacts: PathBuf::from(compiler::ART_PATH),
        checkpoint: PathBuf::from(DEFAULT_TRACE_CHECKPOINT),
        output: PathBuf::from("lane_compare"),
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--corpus-meta" => options.corpus_meta = PathBuf::from(value),
            "--corpus-recs" => options.corpus_recs = PathBuf::from(value),
            "--artifacts" => options.artifacts = PathBuf::from(value),
            "--checkpoint" => options.checkpoint = PathBuf::from(value),
            "--out" => options.output = PathBuf::from(value),
            _ => return Err(format!("unknown lane-compare option: {flag}")),
        }
        index += 2;
    }
    Ok(options)
}

/// Hidden-state trace lane comparison (issue #71, module docs): run both
/// observation lanes at the fixed default point, write
/// `lane_compare.json`, and print the comparison table. Release-mode
/// workload on the fixture corpus (the replay is one forward per corpus
/// position).
pub fn lane_compare_command(args: &[String]) -> Result<(), String> {
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make the replay much slower; use `cargo run --release -- transformerless lane-compare ...`"
    );
    let options = parse_lane_compare_options(args)?;
    let inputs = cover_sweep::load_inputs(
        &options.corpus_meta,
        &options.corpus_recs,
        &options.artifacts,
    )?;
    let checkpoint = options
        .checkpoint
        .to_str()
        .ok_or_else(|| "checkpoint path is not UTF-8".to_owned())?;
    let mut oracle = LlamaOracle::load(checkpoint);
    let report = run_compare(&inputs, &mut oracle, &ScoreConfig::default())?;

    std::fs::create_dir_all(&options.output).map_err(|error| error.to_string())?;
    let report_json = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    let report_path = options.output.join("lane_compare.json");
    std::fs::write(&report_path, &report_json)
        .map_err(|error| format!("{}: {error}", report_path.display()))?;

    print!("{}", render_table(&report));
    println!("  report: {}", report_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_defaults_and_overrides() {
        let options = parse_lane_compare_options(&[]).expect("defaults");
        let (default_meta, default_recs) = compiler::corpus_paths();
        assert_eq!(options.corpus_meta, PathBuf::from(default_meta));
        assert_eq!(options.corpus_recs, PathBuf::from(default_recs));
        assert_eq!(options.artifacts, PathBuf::from(compiler::ART_PATH));
        assert_eq!(options.checkpoint, PathBuf::from(DEFAULT_TRACE_CHECKPOINT));
        assert_eq!(options.output, PathBuf::from("lane_compare"));

        let args = [
            "--corpus-meta",
            "/tmp/m.bin",
            "--corpus-recs",
            "/tmp/r.bin",
            "--artifacts",
            "/tmp/a.bin",
            "--checkpoint",
            "/tmp/model.bin",
            "--out",
            "/tmp/lc",
        ]
        .map(str::to_owned);
        let options = parse_lane_compare_options(&args).expect("valid options");
        assert_eq!(options.corpus_meta, PathBuf::from("/tmp/m.bin"));
        assert_eq!(options.checkpoint, PathBuf::from("/tmp/model.bin"));
        assert_eq!(options.output, PathBuf::from("/tmp/lc"));

        let bad = ["--k0", "16"].map(str::to_owned);
        assert!(parse_lane_compare_options(&bad).is_err());
        let missing = ["--out"].map(str::to_owned);
        assert!(parse_lane_compare_options(&missing).is_err());
    }
}
