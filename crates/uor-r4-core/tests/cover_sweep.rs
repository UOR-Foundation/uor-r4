//! Cover fineness sweep tests (graph-compiler plan §5, issue #70): the
//! 9-point grid shape, an end-to-end sweep on the planted synthetic
//! corpus with honest per-point counts, the deterministic double-run of
//! one point (byte-identical scored artifact + metrics), the
//! `cover_sweep.json` schema, and the recommendation rule's edge cases
//! (empty grid, single point, ties, dominance, the slope floor). The
//! fixture-corpus sweep is a release-only workload and is `#[ignore]`d by
//! default, mirroring the κ-reproduction convention — it is also the
//! CLI-deferred driver path (the `cover-sweep` command is pending while
//! `command.rs` settles): run it with
//! `cargo test -p uor-r4-core --release --offline --test cover_sweep -- --ignored --nocapture`.

use uor_r4_core::transformerless::compiler::{self, Corpus, D, K, SIG_BYTES, STAGES};
use uor_r4_core::transformerless::cover::{self, CoverConfig, Observation};
use uor_r4_core::transformerless::cover_sweep::{self, Recommendation, SweepInputs, SweepRow};
use uor_r4_core::transformerless::runtime;
use uor_r4_core::transformerless::score::{GateCMetrics, ScoreConfig};

// ------------------------------------------------------- synthetic data --
// Mirrors tests/score.rs: five planted groups on the 288-dim sphere with
// planted next-token distributions over 10 stories of 42 positions.

const GROUP_SIZES: [usize; 5] = [100, 100, 60, 60, 100];
const COARSE: [usize; 5] = [0, 1, 2, 2, 3];
const STORY_LEN: u32 = 42;

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

fn normalize(v: &mut [f32]) {
    let nn = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
    for x in v.iter_mut() {
        *x /= nn;
    }
}

fn synthetic_corpus() -> (Vec<Observation>, Corpus) {
    let mut observations = Vec::new();
    let mut story = Vec::new();
    let mut input = Vec::new();
    let mut next = Vec::new();
    let mut t_argmax = Vec::new();
    let mut top_tokens = Vec::new();
    let mut top_weights = Vec::new();
    let mut position = 0u32;
    for (group, &size) in GROUP_SIZES.iter().enumerate() {
        let center = match group {
            2 => planted_center_g2(false),
            3 => planted_center_g2(true),
            _ => planted_center(COARSE[group]),
        };
        for i in 0..size {
            let index = position as usize;
            let mut vector = center.clone();
            let mut rng = (index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
            for x in vector.iter_mut() {
                let draw = xorshift(&mut rng);
                let magnitude = ((draw >> 9) % 1000) as f32 * 1e-5;
                *x += if draw & 1 == 1 { magnitude } else { -magnitude };
            }
            normalize(&mut vector);
            let mut sig = [0u8; SIG_BYTES];
            for (d, &x) in vector.iter().enumerate() {
                if x > 0.0 {
                    sig[d / 8] |= 1 << (d % 8);
                }
            }
            let next_token = match group {
                0 => 30,
                1 => 40 + (i % 4) as u32,
                2 => 10,
                3 => 20,
                _ => 50 + (i % 4) as u32,
            };
            let story_id = position / STORY_LEN;
            story.push(story_id);
            input.push(if position.is_multiple_of(STORY_LEN) {
                1
            } else {
                *next.last().expect("previous next token")
            });
            next.push(next_token);
            t_argmax.push(next_token);
            top_tokens.push([next_token, 0, 0, 0, 0, 0, 0, 0]);
            top_weights.push([100, 0, 0, 0, 0, 0, 0, 0]);
            observations.push(Observation {
                position,
                sample: blake3::hash(&position.to_le_bytes()).into(),
                vector,
                sig,
                next: next_token,
            });
            position += 1;
        }
    }
    let n = observations.len();
    let corpus = Corpus {
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
    };
    (observations, corpus)
}

/// A minimal Compiled (random-but-deterministic tables; mirrors
/// tests/cover.rs).
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

/// The shared sweep inputs over the synthetic corpus (the same bundle
/// the CLI loader assembles from disk).
fn synthetic_inputs() -> SweepInputs {
    let (observations, corpus) = synthetic_corpus();
    let artifacts = synthetic_compiled();
    let artifact_container = compiler::artifact_bytes(&artifacts);
    let cut = compiler::train_cut(&corpus);
    let train: Vec<Observation> = observations
        .iter()
        .filter(|o| corpus.story[o.position as usize] < cut)
        .cloned()
        .collect();
    let held_out: Vec<Observation> = observations
        .iter()
        .filter(|o| corpus.story[o.position as usize] >= cut)
        .cloned()
        .collect();
    let (store, _) = runtime::build_store(&artifacts, &corpus);
    let tls1 = runtime::store_bytes(&store);
    SweepInputs {
        artifact_kappa: format!("blake3:{}", blake3::hash(&artifact_container).to_hex()),
        corpus_kappa: "blake3:synthetic-corpus".to_owned(),
        artifact_container,
        artifacts,
        corpus,
        meta_bytes: b"synthetic-meta".to_vec(),
        recs_bytes: b"synthetic-recs".to_vec(),
        train,
        held_out,
        store,
        tls1,
    }
}

// ------------------------------------------------------------- the grid --

#[test]
fn sweep_grid_covers_the_documented_nine_points() {
    let grid = cover_sweep::sweep_grid();
    assert_eq!(grid.len(), 9, "8 grid points + the baseline row");
    assert_eq!(
        grid.iter().filter(|p| p.baseline).count(),
        1,
        "exactly one baseline row"
    );
    for &k0 in &cover_sweep::SWEEP_K0 {
        for &gain in &cover_sweep::SWEEP_ENTROPY_GAIN_BITS {
            for &budget in &cover_sweep::SWEEP_REGIONS_BUDGET {
                assert!(
                    grid.iter().any(|p| !p.baseline
                        && p.config.k0 == k0
                        && p.config.entropy_gain_bits == gain
                        && p.config.regions_budget == budget),
                    "grid point k0={k0} gain={gain} budget={budget} present"
                );
            }
        }
    }
    let baseline = grid.iter().find(|p| p.baseline).expect("baseline row");
    assert_eq!(
        baseline.config,
        CoverConfig::default(),
        "the baseline row is the current default operating point"
    );
    // All nine labels are distinct (the report's row identity).
    let mut labels: Vec<&str> = grid.iter().map(|p| p.label.as_str()).collect();
    labels.sort_unstable();
    labels.dedup();
    assert_eq!(labels.len(), 9);
}

// ------------------------------------------------------- end-to-end run --

#[test]
fn sweep_runs_end_to_end_with_honest_counts() {
    let inputs = synthetic_inputs();
    let train_n = inputs.train.len();
    let held_out_n = inputs.held_out.len();
    let report = cover_sweep::run_sweep(&inputs, &ScoreConfig::default()).expect("sweep runs");

    assert_eq!(report.schema, cover_sweep::SWEEP_REPORT_SCHEMA);
    assert_eq!(report.points.len(), 9);
    assert_eq!(report.inputs.train_observations, train_n);
    assert_eq!(report.inputs.held_out_observations, held_out_n);
    assert_eq!(report.tla3_baseline.positions, held_out_n);
    assert_eq!(
        report.scorer.smoothing,
        ScoreConfig::default().smoothing.label(),
        "the fixed scorer's smoothing rule is recorded"
    );
    for row in &report.points {
        // Honest counts: regions within budget, depth-1 count within k0
        // (empty clusters are dropped by induction), per-depth summing
        // to the total, every recall rate a probability.
        assert!(
            row.regions.total <= row.config.regions_budget,
            "{}: {} regions within the {} budget",
            row.label,
            row.regions.total,
            row.config.regions_budget
        );
        assert!(row.regions.total >= 1);
        assert_eq!(
            row.regions.per_depth.iter().sum::<u32>() as usize,
            row.regions.total,
            "{}: per-depth counts sum to the total",
            row.label
        );
        assert_eq!(row.regions.per_depth.len(), row.regions.max_depth);
        assert!((row.regions.per_depth[0] as usize) <= row.config.k0);
        assert_eq!(row.recall.len(), row.regions.max_depth);
        for depth in &row.recall {
            assert_eq!(depth.evaluated, held_out_n);
            assert!((0.0..=1.0).contains(&depth.reference_top1));
            assert!((0.0..=1.0).contains(&depth.reference_topm));
            assert!(
                depth.reference_topm >= depth.reference_top1,
                "top-M recall dominates top-1"
            );
            assert!(depth.frontier_mean >= 1.0);
            assert!(depth.frontier_max <= cover::TOP_M as u32);
        }
        assert!(row.artifact_bytes > 0);
        assert!(
            row.graph_kappa.starts_with("blake3:"),
            "graph κ recorded per point"
        );
        assert_eq!(row.gate_c_rule12.positions, held_out_n);
        assert!((0.0..=1.0).contains(&row.gate_c_rule12.top1_agreement));
        assert!(row.gate_c_rule12.bits_per_token.is_finite());
        assert!(row.gate_c_rule12.bits_per_token >= 0.0);
    }
    // The recommendation is present and names a swept point.
    let recommendation = report.recommendation.expect("a recommendation");
    assert!(
        report
            .points
            .iter()
            .any(|p| p.label == recommendation.label),
        "the recommendation names a swept point"
    );
    assert!(recommendation.delta_bytes_vs_baseline.is_some());
    assert!(recommendation.delta_agreement_vs_baseline.is_some());
    assert!(!recommendation.rationale.is_empty());
}

// ----------------------------------------------------------- determinism --

/// Any single point run twice produces a byte-identical scored artifact
/// and identical metrics (the issue's determinism acceptance).
#[test]
fn single_point_double_run_is_byte_identical() {
    let inputs = synthetic_inputs();
    let grid = cover_sweep::sweep_grid();
    let point = grid.iter().find(|p| p.baseline).expect("baseline point");
    let config = ScoreConfig::default();
    let (row1, tla1, bytes1) = cover_sweep::run_point(&inputs, point, &config).expect("first run");
    let (row2, tla2, bytes2) = cover_sweep::run_point(&inputs, point, &config).expect("second run");
    assert_eq!(bytes1, bytes2, "byte-identical scored artifact");
    assert_eq!(row1.graph_kappa, row2.graph_kappa);
    let ser1 = serde_json::to_string(&row1).expect("row serializes");
    let ser2 = serde_json::to_string(&row2).expect("row serializes");
    assert_eq!(ser1, ser2, "identical metrics across the double run");
    assert_eq!(
        serde_json::to_string(&tla1).expect("serializes"),
        serde_json::to_string(&tla2).expect("serializes")
    );
}

// --------------------------------------------------------------- schema --

#[test]
fn report_json_matches_the_documented_schema() {
    let inputs = synthetic_inputs();
    let report = cover_sweep::run_sweep(&inputs, &ScoreConfig::default()).expect("sweep runs");
    let json = serde_json::to_value(&report).expect("report serializes");

    assert_eq!(json["schema"], 1);
    for key in [
        "inputs",
        "scorer",
        "tla3_baseline",
        "recommendation",
        "points",
        "determinism",
    ] {
        assert!(json.get(key).is_some(), "top-level key {key} present");
    }
    for key in [
        "artifact_kappa",
        "corpus_kappa",
        "train_observations",
        "held_out_observations",
    ] {
        assert!(json["inputs"].get(key).is_some(), "inputs.{key}");
    }
    for key in [
        "transition_out_degree",
        "emission_entries",
        "root_top_b",
        "exct_top_x",
        "witness_sample",
        "smoothing",
    ] {
        assert!(json["scorer"].get(key).is_some(), "scorer.{key}");
    }
    for key in ["positions", "top1_agreement", "bits_per_token"] {
        assert!(
            json["tla3_baseline"].get(key).is_some(),
            "tla3_baseline.{key}"
        );
    }
    let points = json["points"].as_array().expect("points is an array");
    assert_eq!(points.len(), 9);
    assert_eq!(
        points.iter().filter(|p| p["baseline"] == true).count(),
        1,
        "exactly one baseline row in the report"
    );
    for point in points {
        for key in [
            "label",
            "baseline",
            "config",
            "regions",
            "recall",
            "artifact_bytes",
            "graph_kappa",
            "gate_c_rule12",
        ] {
            assert!(point.get(key).is_some(), "point.{key} present");
        }
        for key in [
            "k0",
            "depths",
            "entropy_gain_bits",
            "regions_budget",
            "min_support",
            "memory_budget_bytes",
        ] {
            assert!(point["config"].get(key).is_some(), "point.config.{key}");
        }
        for key in ["total", "per_depth", "splits", "max_depth"] {
            assert!(point["regions"].get(key).is_some(), "point.regions.{key}");
        }
        for recall in point["recall"].as_array().expect("recall is an array") {
            for key in [
                "depth",
                "evaluated",
                "reference_top1",
                "reference_topm",
                "frontier_mean",
                "frontier_max",
            ] {
                assert!(recall.get(key).is_some(), "point.recall[].{key}");
            }
        }
        for key in ["positions", "top1_agreement", "bits_per_token"] {
            assert!(
                point["gate_c_rule12"].get(key).is_some(),
                "point.gate_c_rule12.{key}"
            );
        }
    }
    let recommendation = &json["recommendation"];
    for key in [
        "label",
        "bytes",
        "agreement",
        "slope_floor",
        "frontier",
        "delta_bytes_vs_baseline",
        "delta_agreement_vs_baseline",
        "rationale",
    ] {
        assert!(recommendation.get(key).is_some(), "recommendation.{key}");
    }
    assert_eq!(
        recommendation["slope_floor"],
        serde_json::to_value(cover_sweep::KNEE_SLOPE_FLOOR).expect("floor serializes"),
        "the recorded floor is the documented constant"
    );
}

// ------------------------------------------------- recommendation edges --

/// A minimal report row for the recommendation rule's unit cases.
fn hand_row(label: &str, bytes: usize, agreement: f64, baseline: bool) -> SweepRow {
    SweepRow {
        label: label.to_owned(),
        baseline,
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
        recall: Vec::new(),
        artifact_bytes: bytes,
        graph_kappa: "blake3:hand".to_owned(),
        gate_c_rule12: GateCMetrics {
            positions: 1,
            top1_agreement: agreement,
            bits_per_token: 0.0,
        },
    }
}

#[test]
fn recommendation_handles_the_empty_grid() {
    assert!(cover_sweep::recommend(&[]).is_none());
}

#[test]
fn recommendation_single_point_is_that_point() {
    let rec =
        cover_sweep::recommend(&[hand_row("only", 1000, 0.5, false)]).expect("a recommendation");
    assert_eq!(rec.label, "only");
    assert_eq!(rec.frontier, vec!["only".to_owned()]);
    assert!(rec.delta_bytes_vs_baseline.is_none());
}

#[test]
fn recommendation_breaks_ties_deterministically() {
    // Identical bytes and agreement: the ascending-label row wins, and
    // the tied row never reaches the frontier.
    let rows = [
        hand_row("b-tie", 1000, 0.5, false),
        hand_row("a-tie", 1000, 0.5, false),
    ];
    let rec = cover_sweep::recommend(&rows).expect("a recommendation");
    assert_eq!(rec.label, "a-tie");
    let rec_again = cover_sweep::recommend(&rows).expect("a recommendation");
    assert_eq!(rec.label, rec_again.label, "stable across calls");
}

#[test]
fn recommendation_drops_dominated_points() {
    // More bytes for the same or less agreement never wins.
    let rows = [
        hand_row("cheap", 1000, 0.5, false),
        hand_row("same-agreement-richer", 2000, 0.5, false),
        hand_row("worse-and-richer", 3000, 0.4, false),
    ];
    let rec = cover_sweep::recommend(&rows).expect("a recommendation");
    assert_eq!(rec.label, "cheap");
    assert_eq!(rec.frontier, vec!["cheap".to_owned()]);
}

#[test]
fn recommendation_applies_the_slope_floor_both_ways() {
    // Below the floor: +0.01pp over +1 MB is a 1e-10/byte slope — stay.
    let rows = [
        hand_row("cheap", 1000, 0.5, false),
        hand_row("rich", 1_001_000, 0.5001, false),
    ];
    let rec = cover_sweep::recommend(&rows).expect("a recommendation");
    assert_eq!(rec.label, "cheap", "marginal gain below the floor");

    // Above the floor: +10pp over +1 kB is a 1e-4/byte slope — advance.
    let rows = [
        hand_row("cheap", 1000, 0.5, false),
        hand_row("rich", 2000, 0.6, false),
    ];
    let rec = cover_sweep::recommend(&rows).expect("a recommendation");
    assert_eq!(rec.label, "rich", "marginal gain above the floor");
}

#[test]
fn recommendation_stops_at_the_first_below_floor_step() {
    // Non-concave frontier: the knee walk stops at the first below-floor
    // step even when a later step would clear the floor again — the later
    // fidelity cannot be bought without the below-floor step.
    let rows = [
        hand_row("a", 1000, 0.5, false),
        hand_row("b", 1_001_000, 0.5001, false),
        hand_row("c", 1_002_000, 0.9, false),
    ];
    let rec = cover_sweep::recommend(&rows).expect("a recommendation");
    assert_eq!(rec.label, "a");
    assert_eq!(
        rec.frontier,
        vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]
    );
}

#[test]
fn recommendation_records_the_baseline_comparison() {
    let rows = [
        hand_row("baseline", 1000, 0.5, true),
        hand_row("rich", 2000, 0.6, false),
    ];
    let rec: Recommendation = cover_sweep::recommend(&rows).expect("a recommendation");
    assert_eq!(rec.label, "rich");
    assert_eq!(rec.delta_bytes_vs_baseline, Some(1000));
    let delta = rec.delta_agreement_vs_baseline.expect("agreement delta");
    assert!((delta - 0.1).abs() < 1e-12, "delta agreement vs baseline");
    assert!(
        rec.rationale.contains("baseline"),
        "the written justification names the baseline comparison"
    );

    // The baseline itself can be the recommendation.
    let rows = [
        hand_row("baseline", 1000, 0.5, true),
        hand_row("rich", 1_001_000, 0.5001, false),
    ];
    let rec = cover_sweep::recommend(&rows).expect("a recommendation");
    assert_eq!(rec.label, "baseline");
    assert_eq!(rec.delta_bytes_vs_baseline, Some(0));
}

#[test]
fn recommendation_names_an_exact_tie_with_the_baseline() {
    // The recommended point differs from the baseline only in an inert
    // knob (identical bytes and fidelity): the verdict confirms the
    // default as adequate rather than claiming a cheaper win.
    let rows = [
        hand_row("zz-baseline", 1000, 0.5, true),
        hand_row("a-twin", 1000, 0.5, false),
    ];
    let rec = cover_sweep::recommend(&rows).expect("a recommendation");
    assert_eq!(rec.label, "a-twin", "label order breaks the exact tie");
    assert_eq!(rec.delta_bytes_vs_baseline, Some(0));
    assert_eq!(rec.delta_agreement_vs_baseline, Some(0.0));
    assert!(
        rec.rationale.contains("ties the baseline exactly"),
        "the exact-tie verdict, got: {}",
        rec.rationale
    );
    assert!(rec.rationale.contains("confirmed adequate"));
}

// ------------------------------------------------- fixture end-to-end --

/// The full 9-point sweep on the pinned fixture corpus (150k legacy
/// records) — the issue-#70 deliverable numbers. Release-only workload;
/// this test doubles as the CLI-deferred driver: it writes
/// `/tmp/cover_sweep/cover_sweep.json` (the same document the deferred
/// `cover-sweep` command writes) and prints the rate–distortion table.
/// Run with
/// `cargo test -p uor-r4-core --release --offline --test cover_sweep -- --ignored --nocapture`.
#[test]
#[ignore = "release-only fixture workload"]
fn fixture_sweep_end_to_end() {
    let dir = env!("CARGO_MANIFEST_DIR");
    let inputs = cover_sweep::load_inputs(
        std::path::Path::new(&format!("{dir}/tests/fixtures/c_meta.bin")),
        std::path::Path::new(&format!("{dir}/tests/fixtures/c_recs.bin")),
        std::path::Path::new(&format!("{dir}/tests/fixtures/tless_artifacts.bin")),
    )
    .expect("fixture inputs load");
    assert_eq!(inputs.train.len() + inputs.held_out.len(), inputs.corpus.n);

    let report = cover_sweep::run_sweep(&inputs, &ScoreConfig::default()).expect("fixture sweep");
    print!("{}", cover_sweep::render_table(&report));

    let out_dir = std::path::Path::new("/tmp/cover_sweep");
    std::fs::create_dir_all(out_dir).expect("create /tmp/cover_sweep");
    let report_json = serde_json::to_string_pretty(&report).expect("report serializes");
    std::fs::write(out_dir.join("cover_sweep.json"), &report_json).expect("write report");

    // Honest counts across the grid (the same invariants as the
    // synthetic end-to-end test).
    let held_out_n = inputs.held_out.len();
    assert_eq!(report.points.len(), 9);
    for row in &report.points {
        assert!(row.regions.total <= row.config.regions_budget);
        assert_eq!(
            row.regions.per_depth.iter().sum::<u32>() as usize,
            row.regions.total
        );
        assert!((row.regions.per_depth[0] as usize) <= row.config.k0);
        assert_eq!(row.gate_c_rule12.positions, held_out_n);
        assert!((0.0..=1.0).contains(&row.gate_c_rule12.top1_agreement));
        for depth in &row.recall {
            assert_eq!(depth.evaluated, held_out_n);
            assert!((0.0..=1.0).contains(&depth.reference_top1));
            assert!(depth.reference_topm >= depth.reference_top1);
            assert!(depth.frontier_max <= cover::TOP_M as u32);
        }
    }

    // Fixture determinism evidence: the baseline point double-run is
    // byte-identical, artifact and metrics.
    let grid = cover_sweep::sweep_grid();
    let baseline = grid.iter().find(|p| p.baseline).expect("baseline point");
    let config = ScoreConfig::default();
    let (row1, _, bytes1) = cover_sweep::run_point(&inputs, baseline, &config).expect("rerun");
    let (row2, _, bytes2) = cover_sweep::run_point(&inputs, baseline, &config).expect("rerun");
    assert_eq!(
        bytes1, bytes2,
        "fixture double-run: byte-identical artifact"
    );
    assert_eq!(
        serde_json::to_string(&row1).expect("serializes"),
        serde_json::to_string(&row2).expect("serializes"),
        "fixture double-run: identical metrics"
    );
    let swept = report
        .points
        .iter()
        .find(|p| p.baseline)
        .expect("baseline row");
    assert_eq!(
        row1.graph_kappa, swept.graph_kappa,
        "the double-run point equals the sweep's baseline row"
    );
    println!("report written to /tmp/cover_sweep/cover_sweep.json");
}
