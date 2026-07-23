//! Hidden-state trace lane tests (graph-compiler plan §5 Phase 2, issue
//! #71): a FakeOracle with planted hidden-state geometry proves
//! `build_trace_observations` aligns 1:1 with the bundle lane and that
//! `induce_cover` recovers the planted regions from trace observations;
//! the dimension gate rejects non-288 oracles; the two-lane comparison is
//! byte-identical across a double run; the adoption rule's edges are unit
//! tested. The fixture-corpus comparison (150k-position teacher replay)
//! is a release-only workload and is `#[ignore]`d by default, mirroring
//! the κ-reproduction convention: run it with
//! `cargo test -p uor-r4-core --release --offline --test trace_lane -- --ignored --nocapture`.

use uor_r4_core::transformerless::compiler::{self, Corpus, D, K, SIG_BYTES, STAGES};
use uor_r4_core::transformerless::cover::{self, CoverConfig, Observation};
use uor_r4_core::transformerless::cover_sweep::{self, SweepInputs};
use uor_r4_core::transformerless::runtime;
use uor_r4_core::transformerless::score::{GateCMetrics, ScoreConfig};
use uor_r4_core::transformerless::teacher::{
    BehaviorSource, LlamaOracle, RepresentationSource, TeacherOracle,
};
use uor_r4_core::transformerless::trace_lane;

const LEGACY_CHECKPOINT: &str = "/tmp/ref/out/model.bin";

// ------------------------------------------------------- synthetic data --
// Mirrors tests/cover_sweep.rs: five planted groups on the 288-dim sphere
// over 10 stories of 42 positions; the trace lane's FakeOracle plants the
// same geometry as hidden states.

const GROUP_SIZES: [usize; 5] = [100, 100, 60, 60, 100];
const COARSE: [usize; 5] = [0, 1, 2, 2, 3];
const STORY_LEN: u32 = 42;
const RECOVERY_RECALL_FLOOR: f64 = 0.95;

fn xorshift(s: &mut u64) -> u64 {
    let mut x = *s;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *s = x;
    x
}

fn planted_center(coarse: usize) -> Vec<f32> {
    let mut center = vec![0f32; D];
    for d in 0..72 {
        center[coarse * 72 + d] = 1.0 / (72f32).sqrt();
    }
    center
}

fn planted_center_g2(sub_b: bool) -> Vec<f32> {
    let mut center = vec![0f32; D];
    for d in 0..36 {
        center[144 + d] = if sub_b { 0.8 } else { 1.2 };
        center[180 + d] = if sub_b { 1.2 } else { 0.8 };
    }
    center
}

/// Group of a corpus position under the planted layout.
fn group_of(position: usize) -> usize {
    let mut rest = position;
    for (group, &size) in GROUP_SIZES.iter().enumerate() {
        if rest < size {
            return group;
        }
        rest -= size;
    }
    panic!("position {position} past the planted corpus");
}

/// The planted hidden state of a corpus position: the group center plus
/// deterministic sign-independent jitter (the tests/cover.rs pattern).
fn planted_hidden(position: usize) -> Vec<f32> {
    let group = group_of(position);
    let mut vector = match group {
        2 => planted_center_g2(false),
        3 => planted_center_g2(true),
        _ => planted_center(COARSE[group]),
    };
    let mut rng = (position as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    for x in vector.iter_mut() {
        let draw = xorshift(&mut rng);
        let magnitude = ((draw >> 9) % 1000) as f32 * 1e-5;
        *x += if draw & 1 == 1 { magnitude } else { -magnitude };
    }
    vector
}

fn synthetic_corpus() -> Corpus {
    let mut story = Vec::new();
    let mut input = Vec::new();
    let mut next = Vec::new();
    let mut t_argmax = Vec::new();
    let mut top_tokens = Vec::new();
    let mut top_weights = Vec::new();
    let mut position = 0u32;
    for (group, &size) in GROUP_SIZES.iter().enumerate() {
        for i in 0..size {
            let next_token = match group {
                0 => 30,
                1 => 40 + (i % 4) as u32,
                2 => 10,
                3 => 20,
                _ => 50 + (i % 4) as u32,
            };
            story.push(position / STORY_LEN);
            input.push(if position.is_multiple_of(STORY_LEN) {
                1
            } else {
                *next.last().expect("previous next token")
            });
            next.push(next_token);
            t_argmax.push(next_token);
            top_tokens.push([next_token, 0, 0, 0, 0, 0, 0, 0]);
            top_weights.push([100, 0, 0, 0, 0, 0, 0, 0]);
            position += 1;
        }
    }
    let n = story.len();
    Corpus {
        n,
        stories: u64::from(position / STORY_LEN),
        story,
        input,
        next,
        t_argmax,
        top_tokens,
        top_weights,
        span_start: (0..n as u32).collect(),
        span_end: (1..=n as u32).collect(),
        byte_start: vec![u32::MAX; n],
        byte_end: vec![u32::MAX; n],
    }
}

/// A minimal Compiled (random-but-deterministic tables; mirrors
/// tests/cover_sweep.rs).
fn synthetic_compiled() -> compiler::Compiled {
    let vocab = 64usize;
    let mut rng = 0xC0DE42u64;
    let mut rand_bytes =
        |n: usize| -> Vec<u8> { (0..n).map(|_| (xorshift(&mut rng) & 0xff) as u8).collect() };
    compiler::Compiled {
        token_codes: rand_bytes(vocab * STAGES),
        stage_books: (0..STAGES)
            .map(|_| rand_bytes(K * D).iter().map(|&b| b as i8).collect())
            .collect(),
        stage_shifts: vec![0; STAGES],
        thresholds: vec![0; D],
        class_sigs: (0..STAGES).map(|_| rand_bytes(K * SIG_BYTES)).collect(),
        ctx_cb: Vec::new(),
        token_stage_kappas: Vec::new(),
    }
}

// -------------------------------------------------------- planted oracle --

/// FakeOracle planting the five-group hidden-state geometry: step number
/// (counted across resets, matching the lane's sequential story-order
/// replay) maps to the corpus position of the same index.
struct PlantedOracle {
    calls: usize,
    hidden: Vec<f32>,
    dim: usize,
}

impl PlantedOracle {
    fn new() -> Self {
        PlantedOracle {
            calls: 0,
            hidden: vec![0f32; D],
            dim: D,
        }
    }
}

impl RepresentationSource for PlantedOracle {
    fn vocab_size(&self) -> usize {
        64
    }
    fn source_dimension(&self) -> usize {
        self.dim
    }
    fn tokenizer_address(&self) -> &str {
        "planted-tokenizer"
    }
    fn read_embedding_rows(
        &self,
        _range: std::ops::Range<usize>,
        output: &mut [f32],
    ) -> Result<(), String> {
        output.fill(0.0);
        Ok(())
    }
}

impl BehaviorSource for PlantedOracle {
    fn reset(&mut self) {}
    fn step(&mut self, _token: usize, _pos: usize, logits: &mut [f32]) {
        self.hidden = planted_hidden(self.calls);
        self.calls += 1;
        for (i, logit) in logits.iter_mut().enumerate() {
            *logit = i as f32;
        }
    }
}

impl TeacherOracle for PlantedOracle {
    fn vocab(&self) -> usize {
        64
    }
    fn dim(&self) -> usize {
        self.dim
    }
    fn seq_len(&self) -> usize {
        64
    }
    fn kappa(&self) -> String {
        "blake3:planted-oracle".to_owned()
    }
    fn source_bytes(&self) -> usize {
        0
    }
    fn embedding(&self, _token: usize, out: &mut [f32]) {
        out.fill(0.0);
    }
    fn hidden_state(&self) -> Option<&[f32]> {
        Some(&self.hidden)
    }
}

// ------------------------------------------------------------- the lane --

#[test]
fn trace_observations_align_with_the_bundle_lane() {
    let corpus = synthetic_corpus();
    let artifacts = synthetic_compiled();
    let positions: Vec<usize> = (0..corpus.n).collect();
    let mut oracle = PlantedOracle::new();
    let trace = trace_lane::build_trace_observations(&mut oracle, &corpus, &positions)
        .expect("trace observations");
    let bundle = cover::build_observations(&artifacts, &corpus, &positions);

    assert_eq!(trace.len(), corpus.n);
    assert_eq!(oracle.calls, corpus.n, "one forward per corpus position");
    let mut vectors_differ = 0usize;
    for (t, b) in trace.iter().zip(bundle.iter()) {
        // The lane contract: position, sample, and next are identical to
        // the bundle lane; only the vector and sig change.
        assert_eq!(t.position, b.position);
        assert_eq!(t.sample, b.sample);
        assert_eq!(t.next, b.next);
        // Unit-normalized vector, sign-bit signature vs 0.0.
        let norm = t.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-4, "unit vector, got norm {norm}");
        for (d, &x) in t.vector.iter().enumerate() {
            let bit = t.sig[d / 8] >> (d % 8) & 1;
            assert_eq!(bit == 1, x > 0.0, "sig bit {d} matches the sign");
        }
        if t.vector != b.vector {
            vectors_differ += 1;
        }
    }
    assert!(
        vectors_differ > corpus.n / 2,
        "the hidden lane is a genuinely different geometry ({vectors_differ} differing vectors)"
    );
}

#[test]
fn replay_stops_after_the_last_wanted_position() {
    let corpus = synthetic_corpus();
    let positions: Vec<usize> = (0..corpus.n).step_by(7).collect();
    let last = *positions.last().expect("nonempty");
    let mut oracle = PlantedOracle::new();
    let trace = trace_lane::build_trace_observations(&mut oracle, &corpus, &positions)
        .expect("trace observations");
    assert_eq!(trace.len(), positions.len());
    assert_eq!(
        oracle.calls,
        last + 1,
        "no forwards past the last wanted position"
    );
    for (obs, &p) in trace.iter().zip(positions.iter()) {
        assert_eq!(obs.position, p as u32);
    }
}

#[test]
fn dimension_gate_rejects_non_d_oracles() {
    let corpus = synthetic_corpus();
    let mut oracle = PlantedOracle {
        calls: 0,
        hidden: vec![0f32; 576],
        dim: 576, // the raw Hugging Face teacher width
    };
    let positions: Vec<usize> = (0..4).collect();
    let error = trace_lane::build_trace_observations(&mut oracle, &corpus, &positions)
        .expect_err("576-dim oracle is rejected, not truncated");
    assert!(error.contains("288"), "names the required width: {error}");
    assert!(error.contains("576"), "names the actual width: {error}");
}

#[test]
fn induce_cover_recovers_planted_regions_from_trace_observations() {
    let corpus = synthetic_corpus();
    let positions: Vec<usize> = (0..corpus.n).collect();
    let mut oracle = PlantedOracle::new();
    let trace = trace_lane::build_trace_observations(&mut oracle, &corpus, &positions)
        .expect("trace observations");
    let labels: Vec<usize> = (0..corpus.n).map(|p| COARSE[group_of(p)]).collect();
    let config = CoverConfig {
        depths: 3,
        k0: 4,
        min_support: 64,
        entropy_gain_bits: 0.25,
        ..CoverConfig::default()
    };
    let induced = cover::induce_cover(
        &trace,
        &config,
        "blake3:planted-artifact",
        "blake3:planted-corpus",
    )
    .expect("induction succeeds");
    let cover = &induced.cover;

    // Same planted geometry as tests/cover.rs: 4 depth-1 regions, the G2
    // region splits into 2 token-pure children, nothing deeper.
    assert_eq!(cover.regions.len(), 6, "4 depth-1 regions + 2 G2 children");
    assert_eq!(cover.regions_at_depth(1).len(), 4);
    assert_eq!(cover.regions_at_depth(2).len(), 2);
    for group in 0..4 {
        let group_members: Vec<usize> = (0..trace.len()).filter(|&i| labels[i] == group).collect();
        let best = cover
            .regions_at_depth(1)
            .iter()
            .map(|&r| {
                group_members
                    .iter()
                    .filter(|i| cover.members[r as usize].contains(i))
                    .count()
            })
            .max()
            .unwrap();
        let recall = best as f64 / group_members.len() as f64;
        assert!(
            recall >= RECOVERY_RECALL_FLOOR,
            "planted coarse group {group} recovered from the hidden lane with recall {recall:.3}"
        );
    }
}

// --------------------------------------------------- the comparison ------

/// The shared inputs of both lanes over the synthetic corpus: the bundle
/// lane from `cover::build_observations`, the hidden lane from the
/// PlantedOracle replay — everything else identical.
fn synthetic_lane_inputs() -> (SweepInputs, SweepInputs) {
    let corpus = synthetic_corpus();
    let artifacts = synthetic_compiled();
    let artifact_container = compiler::artifact_bytes(&artifacts);
    let (train_pos, held_pos) = cover::split_positions(&corpus);
    let bundle_train = cover::build_observations(&artifacts, &corpus, &train_pos);
    let bundle_held = cover::build_observations(&artifacts, &corpus, &held_pos);
    let all_positions: Vec<usize> = (0..corpus.n).collect();
    let mut oracle = PlantedOracle::new();
    let trace_all = trace_lane::build_trace_observations(&mut oracle, &corpus, &all_positions)
        .expect("trace observations");
    let cut = compiler::train_cut(&corpus);
    let (trace_train, trace_held): (Vec<Observation>, Vec<Observation>) = trace_all
        .into_iter()
        .partition(|o| corpus.story[o.position as usize] < cut);
    let (store, _) = runtime::build_store(&artifacts, &corpus);
    let tls1 = runtime::store_bytes(&store);
    let artifact_kappa = format!("blake3:{}", blake3::hash(&artifact_container).to_hex());
    let corpus_kappa = "blake3:synthetic-corpus".to_owned();
    let bundle = SweepInputs {
        artifact_kappa: artifact_kappa.clone(),
        corpus_kappa: corpus_kappa.clone(),
        artifact_container: artifact_container.clone(),
        artifacts: artifacts.clone(),
        corpus: corpus.clone(),
        meta_bytes: b"synthetic-meta".to_vec(),
        recs_bytes: b"synthetic-recs".to_vec(),
        train: bundle_train,
        held_out: bundle_held,
        store: store.clone(),
        tls1: tls1.clone(),
    };
    let trace = SweepInputs {
        artifact_kappa,
        corpus_kappa,
        artifact_container,
        artifacts,
        corpus,
        meta_bytes: b"synthetic-meta".to_vec(),
        recs_bytes: b"synthetic-recs".to_vec(),
        train: trace_train,
        held_out: trace_held,
        store,
        tls1,
    };
    (bundle, trace)
}

#[test]
fn comparison_runs_both_lanes_with_honest_counts() {
    let (bundle, trace) = synthetic_lane_inputs();
    let point = trace_lane::compare_point();
    let report = trace_lane::compare_lanes(
        &bundle,
        &trace,
        &point,
        &ScoreConfig::default(),
        "blake3:planted-oracle",
        trace_lane::SAMPLE_NOTE,
    )
    .expect("comparison runs");

    assert_eq!(report.schema, trace_lane::LANE_COMPARE_SCHEMA);
    assert_eq!(report.lanes.len(), 2);
    assert_eq!(report.lanes[0].lane, trace_lane::LANE_BUNDLE);
    assert_eq!(report.lanes[1].lane, trace_lane::LANE_HIDDEN);
    assert_eq!(report.inputs.train_observations, bundle.train.len());
    assert_eq!(report.inputs.held_out_observations, bundle.held_out.len());
    assert_eq!(report.tla3_baseline.positions, bundle.held_out.len());
    assert_eq!(report.point.config.k0, CoverConfig::default().k0);
    for lane in &report.lanes {
        assert!(lane.row.regions.total >= 1);
        assert!(lane.row.regions.total <= lane.row.config.regions_budget);
        assert_eq!(lane.row.gate_c_rule12.positions, bundle.held_out.len());
        assert!((0.0..=1.0).contains(&lane.row.gate_c_rule12.top1_agreement));
        assert!(lane.row.gate_c_rule12.bits_per_token.is_finite());
        for d in &lane.row.recall {
            assert_eq!(d.evaluated, bundle.held_out.len());
            assert!((0.0..=1.0).contains(&d.reference_top1));
            assert!((0.0..=1.0).contains(&d.reference_topm));
        }
    }
    // The recorded choice names a lane, quotes the rule, and justifies
    // itself with the two lanes' numbers.
    assert!(
        report.choice.decision == "adopted" || report.choice.decision == "rejected",
        "decision recorded: {}",
        report.choice.decision
    );
    assert_eq!(report.choice.rule, trace_lane::ADOPTION_RULE);
    assert!(!report.choice.rationale.is_empty());
    let bundle_top1 = report.lanes[0].row.gate_c_rule12.top1_agreement;
    let hidden_top1 = report.lanes[1].row.gate_c_rule12.top1_agreement;
    assert_eq!(report.choice.delta_rule12_top1, hidden_top1 - bundle_top1);
    assert_eq!(
        report.choice.decision == "adopted",
        hidden_top1 > bundle_top1,
        "the decision follows the recorded rule"
    );
}

#[test]
fn comparison_double_run_is_byte_identical() {
    let (bundle, trace) = synthetic_lane_inputs();
    let point = trace_lane::compare_point();
    let config = ScoreConfig::default();
    let report1 = trace_lane::compare_lanes(
        &bundle,
        &trace,
        &point,
        &config,
        "blake3:planted-oracle",
        "s",
    )
    .expect("first run");
    let report2 = trace_lane::compare_lanes(
        &bundle,
        &trace,
        &point,
        &config,
        "blake3:planted-oracle",
        "s",
    )
    .expect("second run");
    let ser1 = serde_json::to_string_pretty(&report1).expect("serializes");
    let ser2 = serde_json::to_string_pretty(&report2).expect("serializes");
    assert_eq!(ser1, ser2, "byte-identical lane_compare.json");
}

#[test]
fn adoption_rule_edges() {
    let row = |top1: f64, bits: f64, recall: f64| cover_sweep::SweepRow {
        label: "x".to_owned(),
        baseline: false,
        config: cover_sweep::SweepRowConfig {
            k0: 8,
            depths: 3,
            entropy_gain_bits: 0.25,
            regions_budget: 256,
            min_support: 64,
            memory_budget_bytes: 0,
        },
        regions: cover_sweep::SweepRegions {
            total: 1,
            per_depth: vec![1],
            splits: 0,
            max_depth: 1,
        },
        recall: vec![cover_sweep::SweepDepthRecall {
            depth: 1,
            evaluated: 10,
            reference_top1: recall,
            reference_topm: recall,
            frontier_mean: 1.0,
            frontier_max: 1,
        }],
        artifact_bytes: 100,
        graph_kappa: "blake3:x".to_owned(),
        gate_c_rule12: GateCMetrics {
            positions: 10,
            top1_agreement: top1,
            bits_per_token: bits,
        },
    };
    // Strict gain → adopted.
    let choice = trace_lane::choose_lane(&row(0.50, 8.0, 0.9), &row(0.51, 8.1, 0.9));
    assert_eq!(choice.decision, "adopted");
    assert_eq!(choice.lane, trace_lane::LANE_HIDDEN);
    // Exact tie → rejected (the rule requires a STRICT gain).
    let choice = trace_lane::choose_lane(&row(0.50, 8.0, 0.9), &row(0.50, 7.9, 1.0));
    assert_eq!(choice.decision, "rejected");
    assert_eq!(choice.lane, trace_lane::LANE_BUNDLE);
    // Regression → rejected even with better recall and bits/token.
    let choice = trace_lane::choose_lane(&row(0.50, 8.0, 0.9), &row(0.49, 7.0, 1.0));
    assert_eq!(choice.decision, "rejected");
    assert!(choice.rationale.contains("rejection evidence"));
}

// ------------------------------------------------- fixture end-to-end ----

#[test]
#[ignore = "release-only fixture workload (150k-position teacher replay)"]
fn fixture_corpus_lane_compare_end_to_end() {
    if std::fs::metadata(LEGACY_CHECKPOINT).is_err() {
        eprintln!("skipping: source checkpoint not found at {LEGACY_CHECKPOINT}");
        return;
    }
    let dir = env!("CARGO_MANIFEST_DIR");
    let inputs = cover_sweep::load_inputs(
        std::path::Path::new(&format!("{dir}/tests/fixtures/c_meta.bin")),
        std::path::Path::new(&format!("{dir}/tests/fixtures/c_recs.bin")),
        std::path::Path::new(&format!("{dir}/tests/fixtures/tless_artifacts.bin")),
    )
    .expect("fixture inputs load");
    let mut oracle = LlamaOracle::load(LEGACY_CHECKPOINT);
    let report = trace_lane::run_compare(&inputs, &mut oracle, &ScoreConfig::default())
        .expect("fixture comparison");
    print!("{}", trace_lane::render_table(&report));

    assert_eq!(report.lanes.len(), 2);
    assert_eq!(
        report.inputs.train_observations + report.inputs.held_out_observations,
        report.inputs.positions
    );
    for lane in &report.lanes {
        assert!(lane.row.regions.total >= 1);
        for d in &lane.row.recall {
            assert!((0.0..=1.0).contains(&d.reference_top1));
            assert!((0.0..=1.0).contains(&d.reference_topm));
        }
        assert!((0.0..=1.0).contains(&lane.row.gate_c_rule12.top1_agreement));
    }
    assert!(
        report.choice.decision == "adopted" || report.choice.decision == "rejected",
        "decision recorded: {}",
        report.choice.decision
    );
}
