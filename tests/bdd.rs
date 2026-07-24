//! Cucumber runner for behavior-level R4G1 checks.
//!
//! The feature files live under `features/suites`, following the upstream
//! Hologram layout. Keep the scenarios focused on externally meaningful
//! behavior; implementation details stay in the server module.

use cucumber::{given, then, when, World};
use std::path::Path;
use uor_r4_core::transformerless::bott_fock::BottFockContextStore;
use uor_r4_core::transformerless::compiler::SIG_BYTES;
use uor_r4_core::transformerless::endomorphism::EndomorphismAlgebra;
use uor_r4_core::transformerless::lie_jordan::{universal_product_u8, LieJordanSplit};
use uor_r4_graph_compiler::induction::Observation;
use uor_r4_graph_compiler::quantum_cover::{
    quantum_entropy_gain, DensityOperator, QuantumCoverConfig,
};
use uor_r4_wasm_router::cd_space_fold;
use uor_r4_wasm_router::r4g1::validate_quality_report;
use uor_r4_wasm_router::server::{
    is_usable_generated_text, r4g1_unavailable_response, select_synthesis_engine,
    validate_r4g1_corpus_inputs,
};

#[derive(Debug, Default, World)]
struct R4g1World {
    response: String,
    usable: Option<bool>,
    requested_engine: Option<&'static str>,
    selected_engine: Option<&'static str>,
    endpoint_status: Option<u16>,
    endpoint_body: Option<serde_json::Value>,
    compile_error: Option<String>,
    quality_report: Option<serde_json::Value>,
    quality_error: Option<String>,
    // Façade & Scaling fields
    facade_input: String,
    folded_matrix: Vec<i16>,
    seq_lengths: Vec<usize>,
    bench_latency_us: f64,
    bench_matrix_bytes: usize,
    // Quantum cover fields
    density: Option<DensityOperator>,
    entropy: Option<f32>,
    observations: Vec<Observation>,
    entropy_gain: Option<f64>,
    partition_accepted: Option<bool>,
    // Lie-Jordan fields
    op_matrix: Option<EndomorphismAlgebra>,
    split_result: Option<LieJordanSplit>,
    u8_a: u8,
    u8_b: u8,
    u8_res: u8,
    // Lower Semantic Regions fields (#130)
    lower_bool_region: Option<uor_r4_graph_compiler::lower_semantic_regions::LoweredBooleanRegion>,
    lower_witness: Option<uor_r4_graph_compiler::lower_semantic_regions::LoweringWitnessEntry>,
    lower_q_normal: Option<uor_r4_graph_compiler::lower_semantic_regions::LoweredFixedPointScore>,
    lower_q_max: Option<uor_r4_graph_compiler::lower_semantic_regions::LoweredFixedPointScore>,
    lower_q_min: Option<uor_r4_graph_compiler::lower_semantic_regions::LoweredFixedPointScore>,
    lower_error: Option<uor_r4_graph_compiler::lower_semantic_regions::LoweringError>,
    // Behavioral Probe fields (#128)
    probe_baseline_obs: String,
    probe_suite_report: Option<uor_r4_graph_compiler::behavioral_probes::BehavioralProbeReport>,
    probe_suite_error: Option<uor_r4_graph_compiler::behavioral_probes::BehavioralProbeError>,
    probe_record_error: Option<uor_r4_graph_compiler::behavioral_probes::BehavioralProbeError>,
    // Semantic State Space fields (#124)
    state_s0: Option<uor_r4_graph_compiler::semantic_state::SemanticState>,
    state_eval_res: Option<
        Result<
            uor_r4_graph_compiler::semantic_state::SemanticState,
            uor_r4_graph_compiler::semantic_state::SemanticStateError,
        >,
    >,
    hazard_evaluator: Option<uor_r4_graph_compiler::semantic_state::TransitionEvaluator>,
    goal_satisfied: Option<bool>,
    belief_in: Option<f32>,
    belief_out: Option<f32>,
    trajectory_error: Option<uor_r4_graph_compiler::semantic_state::SemanticStateError>,
}

#[given("the R4G1 runtime returned the browser's repetitive hello response")]
fn repetitive_hello(w: &mut R4g1World) {
    w.response = "how this works like im 5 imagine you have a magic box and inside it are all the rules of geometry think of it like routing a message through a maze i use the math of curves and angles to find the most efficient path for information to go from where you want to go that is how i work go from where you start to where you want to go that is how i work go from where you start to where you start to where you want to go that is how i work go from where you want to go that is how i work go from where you want to go that is how i work go from where you start".to_string();
}

#[given("the R4G1 runtime returned replacement-character gibberish")]
fn replacement_gibberish(w: &mut R4g1World) {
    w.response = "��������������������������������".to_string();
}

#[given("the R4G1 runtime returned low-readability symbol output")]
fn low_readability_symbols(w: &mut R4g1World) {
    w.response = "☃☄☂☀▓▒░".to_string();
}

#[given("the R4G1 runtime returned a long identical-character run")]
fn identical_character_run(w: &mut R4g1World) {
    w.response = "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!".to_string();
}

#[given("the R4G1 runtime returned a concise readable hello response")]
fn concise_hello(w: &mut R4g1World) {
    w.response = "Hello! I can help you explore the compiled R4G1 graph.".to_string();
}

#[given("the R4G1 runtime returned a readable response with ordinary repetition")]
fn ordinary_repetition(w: &mut R4g1World) {
    w.response =
        "The graph can route messages. It can route messages efficiently when the graph is ready."
            .to_string();
}

#[when("the server validates the generated response")]
fn validate_response(w: &mut R4g1World) {
    w.usable = Some(is_usable_generated_text(&w.response));
}

#[then("the response is rejected as unusable")]
fn response_rejected(w: &mut R4g1World) {
    assert_eq!(w.usable, Some(false));
}

#[then("the response is accepted as usable")]
fn response_accepted(w: &mut R4g1World) {
    assert_eq!(w.usable, Some(true));
}

#[given("the browser has no saved engine selection")]
fn no_saved_engine(_w: &mut R4g1World) {}

#[when("the server resolves the synthesis engine")]
fn resolve_engine(w: &mut R4g1World) {
    w.selected_engine = Some(select_synthesis_engine(w.requested_engine));
}

#[then("the selected engine is R4G1")]
fn selected_engine_is_r4g1(w: &mut R4g1World) {
    assert_eq!(w.selected_engine, Some("r4g1"));
}

#[given("the browser explicitly selected the legacy engine")]
fn explicit_legacy(w: &mut R4g1World) {
    w.requested_engine = Some("transformerless-legacy");
}

#[then("the selected engine is Legacy TLA/TLS")]
fn selected_engine_is_legacy(w: &mut R4g1World) {
    assert_eq!(w.selected_engine, Some("transformerless-legacy"));
}

#[then("the browser UI selects R4G1 and does not offer automatic fallback")]
fn browser_selects_r4g1(_w: &mut R4g1World) {
    let source = include_str!("../index.html");
    assert!(source.contains(r#"<option value="r4g1" selected>"#));
    assert!(!source.contains("Auto: R4G1 → Legacy TLA/TLS"));
}

#[given("the R4G1 runtime is unavailable")]
fn unavailable_runtime(_w: &mut R4g1World) {}

#[when("the R4G1 chat endpoint builds its unavailable response")]
fn unavailable_response(w: &mut R4g1World) {
    let (status, body) = r4g1_unavailable_response();
    w.endpoint_status = Some(status);
    w.endpoint_body = Some(body);
}

#[then("it returns HTTP 503 without invoking a fallback engine")]
fn no_fallback_response(w: &mut R4g1World) {
    assert_eq!(w.endpoint_status, Some(503));
    let body = w.endpoint_body.as_ref().expect("endpoint body");
    assert_eq!(body["engine"], "r4g1");
    assert!(body["error"]
        .as_str()
        .unwrap_or_default()
        .contains("no fallback"));
}

#[given("the configured corpus metadata path is missing")]
fn missing_corpus_metadata(_w: &mut R4g1World) {}

#[when("R4G1 compilation inputs are validated")]
fn validate_missing_corpus(w: &mut R4g1World) {
    w.compile_error = validate_r4g1_corpus_inputs(
        Path::new("/tmp/r4g1-bdd-missing/corpus.meta"),
        Path::new("/tmp/r4g1-bdd-missing/corpus.records"),
    )
    .err();
}

#[then("compilation fails with the missing metadata error")]
fn missing_metadata_error(w: &mut R4g1World) {
    assert!(w
        .compile_error
        .as_deref()
        .unwrap_or_default()
        .contains("configured corpus metadata is missing"));
}

#[given("a graph quality report below the TLA baseline")]
fn below_baseline_report(w: &mut R4g1World) {
    w.quality_report = Some(serde_json::json!({
        "gate_c": {
            "rule12_precedence": {"top1_agreement": 0.0035},
            "tla3_baseline": {"top1_agreement": 0.1811}
        }
    }));
}

#[when("the R4G1 quality gate validates the report")]
fn quality_gate_validates_report(w: &mut R4g1World) {
    w.quality_error =
        validate_quality_report(w.quality_report.as_ref().expect("quality report")).err();
}

#[then("the quality gate rejects the graph below baseline")]
fn quality_gate_rejects(w: &mut R4g1World) {
    assert!(w
        .quality_error
        .as_deref()
        .unwrap_or_default()
        .contains("below TLA baseline"));
}

#[given("a graph quality report at or above the TLA baseline")]
fn passing_baseline_report(w: &mut R4g1World) {
    w.quality_report = Some(serde_json::json!({
        "gate_c": {
            "rule12_precedence": {"top1_agreement": 0.1811},
            "tla3_baseline": {"top1_agreement": 0.1811}
        }
    }));
}

#[then("the quality gate accepts the graph")]
fn quality_gate_accepts(w: &mut R4g1World) {
    assert!(w.quality_error.is_none());
}

#[given("a graph quality report at the pinned quality anchors")]
fn pinned_anchors_report(w: &mut R4g1World) {
    // #65-chain anchors: 31.7086% top-1, 9.8612 bits/token (era note in
    // src/r4g1.rs QUALITY_FLOOR_*).
    w.quality_report = Some(serde_json::json!({
        "gate_c": {
            "rule12_precedence": {"top1_agreement": 0.3171, "bits_per_token": 9.8612},
            "tla3_baseline": {"top1_agreement": 0.1811, "bits_per_token": 11.8781}
        }
    }));
}

#[given("a graph quality report using a same-corpus TLA quality profile")]
fn same_corpus_tla_report(w: &mut R4g1World) {
    w.quality_report = Some(serde_json::json!({
        "config": {"quality_profile": "relative_tla"},
        "gate_c": {
            "rule12_precedence": {"top1_agreement": 0.1596, "bits_per_token": 14.2959},
            "tla3_baseline": {"top1_agreement": 0.1596, "bits_per_token": 20.7655}
        }
    }));
}

#[given("a graph quality report with digressed bits per token")]
fn digressed_bits_report(w: &mut R4g1World) {
    // Agreement still clears the baseline, so only the absolute floor can fire.
    w.quality_report = Some(serde_json::json!({
        "gate_c": {
            "rule12_precedence": {"top1_agreement": 0.3171, "bits_per_token": 10.5},
            "tla3_baseline": {"top1_agreement": 0.1811, "bits_per_token": 11.8781}
        }
    }));
}

#[given("a graph quality report with digressed top-1 agreement")]
fn digressed_agreement_report(w: &mut R4g1World) {
    // Agreement still clears the baseline, so only the absolute floor can fire.
    w.quality_report = Some(serde_json::json!({
        "gate_c": {
            "rule12_precedence": {"top1_agreement": 0.25, "bits_per_token": 9.8612},
            "tla3_baseline": {"top1_agreement": 0.1811, "bits_per_token": 11.8781}
        }
    }));
}

#[then("the quality gate rejects the graph for digression")]
fn quality_gate_rejects_digression(w: &mut R4g1World) {
    assert!(w
        .quality_error
        .as_deref()
        .unwrap_or_default()
        .contains("digresses"));
}

#[given("an arbitrary text input string")]
fn arbitrary_text_input(w: &mut R4g1World) {
    w.facade_input = "uor-r4 quantum geometric transformerless engine".to_string();
}

#[when("the Wasm façade folds the text using cd_space_fold")]
fn fold_text_facade(w: &mut R4g1World) {
    w.folded_matrix = cd_space_fold(&w.facade_input).to_vec();
}

#[then("a 256-element integer state matrix is returned")]
fn state_matrix_256_elements(w: &mut R4g1World) {
    assert_eq!(w.folded_matrix.len(), 256);
}

#[then("the state matrix has a non-zero parameter checksum")]
fn state_matrix_nonzero_checksum(w: &mut R4g1World) {
    let sum: i64 = w.folded_matrix.iter().map(|&x| x.abs() as i64).sum();
    assert!(sum > 0, "state matrix sum must be non-zero");
}

#[given("context sequence lengths of 1000, 10000, and 100000 tokens")]
fn sequence_lengths_config(w: &mut R4g1World) {
    w.seq_lengths = vec![1_000, 10_000, 100_000];
}

#[when("the context scaling benchmark is evaluated")]
fn eval_context_scaling(w: &mut R4g1World) {
    use std::time::Instant;
    let mut total_us = 0.0;
    let dummy_token = [10i16; 16];

    for &n in &w.seq_lengths {
        let mut store = BottFockContextStore::new();
        let start = Instant::now();
        for _ in 0..n {
            store.append_token(&dummy_token);
        }
        let elapsed = start.elapsed();
        total_us += elapsed.as_micros() as f64 / (n as f64);
        w.bench_matrix_bytes = store.state().len() * std::mem::size_of::<i16>();
    }
    w.bench_latency_us = total_us / (w.seq_lengths.len() as f64);
}

#[then("the state matrix memory footprint remains constant at 512 bytes")]
fn footprint_constant_512(w: &mut R4g1World) {
    assert_eq!(w.bench_matrix_bytes, 512);
}

#[then("the per-token update latency remains bounded under 50 microseconds")]
fn latency_bounded_50us(w: &mut R4g1World) {
    assert!(
        w.bench_latency_us < 50.0,
        "latency {} us exceeds 50 us limit",
        w.bench_latency_us
    );
}

#[given("a maximum-entropy density operator of dimension 8")]
fn max_entropy_density(w: &mut R4g1World) {
    w.density = Some(DensityOperator::max_entropy(8).expect("dimension non-zero"));
}

#[given("a density operator with a pure distribution")]
fn pure_density(w: &mut R4g1World) {
    w.density = Some(DensityOperator::from_weights(&[1.0, 0.0, 0.0]).expect("valid weights"));
}

#[when("its von Neumann entropy is computed")]
fn compute_entropy(w: &mut R4g1World) {
    w.entropy = Some(
        w.density
            .as_ref()
            .expect("density operator")
            .von_neumann_entropy(),
    );
}

#[then("the entropy equals the natural logarithm of 8")]
fn entropy_is_ln_8(w: &mut R4g1World) {
    let entropy = w.entropy.expect("entropy computed");
    let expected = 8f32.ln();
    assert!(
        (entropy - expected).abs() < 1e-6,
        "S((1/n)I) = ln n: got {entropy}, want {expected}"
    );
}

#[then("the entropy is zero")]
fn entropy_is_zero(w: &mut R4g1World) {
    assert_eq!(w.entropy, Some(0.0));
}

#[given("observations whose halves predict disjoint tokens")]
fn disjoint_halves_observations(w: &mut R4g1World) {
    w.observations = (0..100u32)
        .map(|i| Observation {
            position: i,
            sample: [0u8; 32],
            vector: Vec::new(),
            sig: [0u8; SIG_BYTES],
            prev: 0u32,
            next: if i < 50 { 1 } else { 2 },
        })
        .collect();
}

#[when("the quantum entropy gain of the aligned split is evaluated")]
fn aligned_split_gain(w: &mut R4g1World) {
    let members: Vec<usize> = (0..100).collect();
    let children = vec![(0..50).collect::<Vec<_>>(), (50..100).collect::<Vec<_>>()];
    let gain = quantum_entropy_gain(&w.observations, &members, &children);
    w.entropy_gain = Some(gain);
    w.partition_accepted = Some(QuantumCoverConfig::default().accept_partition(gain));
}

#[when("the quantum entropy gain of the interleaved split is evaluated")]
fn interleaved_split_gain(w: &mut R4g1World) {
    let members: Vec<usize> = (0..100).collect();
    let children = vec![
        (0..100).step_by(2).collect::<Vec<_>>(),
        (1..100).step_by(2).collect::<Vec<_>>(),
    ];
    let gain = quantum_entropy_gain(&w.observations, &members, &children);
    w.entropy_gain = Some(gain);
    w.partition_accepted = Some(QuantumCoverConfig::default().accept_partition(gain));
}

#[then("the gain equals ln 2 and the partition is accepted")]
fn gain_ln2_accepted(w: &mut R4g1World) {
    let gain = w.entropy_gain.expect("gain evaluated");
    assert!(
        (gain - std::f64::consts::LN_2).abs() < 1e-4,
        "gain {gain}, want ln 2"
    );
    assert_eq!(w.partition_accepted, Some(true));
}

#[then("the gain is zero and the partition is rejected")]
fn gain_zero_rejected(w: &mut R4g1World) {
    let gain = w.entropy_gain.expect("gain evaluated");
    assert!(gain.abs() < 1e-4, "gain {gain}, want 0");
    assert_eq!(w.partition_accepted, Some(false));
}

#[given("a Clifford generator matrix operator in 16D Cayley-Dickson space")]
fn clifford_generator_op(w: &mut R4g1World) {
    w.op_matrix = Some(EndomorphismAlgebra::clifford_generator(1));
}

#[when("Lie-Jordan decomposition is performed on the operator")]
fn decompose_op(w: &mut R4g1World) {
    let op = w.op_matrix.as_ref().expect("operator matrix");
    w.split_result = Some(LieJordanSplit::decompose(op));
}

#[then("the Lie component is strictly anti-Hermitian")]
fn lie_anti_hermitian(w: &mut R4g1World) {
    let split = w.split_result.as_ref().expect("split result");
    assert!(LieJordanSplit::is_anti_hermitian(&split.lie));
}

#[then("the Jordan component is strictly Hermitian")]
fn jordan_hermitian(w: &mut R4g1World) {
    let split = w.split_result.as_ref().expect("split result");
    assert!(LieJordanSplit::is_hermitian(&split.jordan));
}

#[then("the reconstructed operator matches the original matrix")]
fn reconstructed_matches(w: &mut R4g1World) {
    let split = w.split_result.as_ref().expect("split result");
    let orig = w.op_matrix.as_ref().expect("original operator");
    let rec = split.reconstruct();
    for (a, b) in orig.matrix.iter().zip(&rec.matrix) {
        assert!((a - b).abs() < 1e-5);
    }
}

#[given("a pair of 8-bit integer operator state bytes")]
fn integer_operator_bytes(w: &mut R4g1World) {
    w.u8_a = 0b1100_1010;
    w.u8_b = 0b1010_1100;
}

#[when("the hot-path universal product kernel is evaluated for Lie anti-Hermitian symmetry")]
fn eval_u8_kernel(w: &mut R4g1World) {
    w.u8_res = universal_product_u8(w.u8_a, w.u8_b, true);
}

#[then("the result matches the bitwise XOR and rotation transformation")]
fn u8_kernel_matches(w: &mut R4g1World) {
    let expected = w.u8_a ^ (w.u8_b.rotate_left(1));
    assert_eq!(w.u8_res, expected);
}

#[then("zero floating-point operations or multiplications are executed")]
fn u8_kernel_zero_floats(_w: &mut R4g1World) {
    let source = include_str!("../crates/uor-r4-core/src/transformerless/lie_jordan.rs");
    let kernel_start = source
        .find("pub fn universal_product_u8")
        .expect("kernel function");
    let kernel_code = &source[kernel_start..];
    assert!(!kernel_code.contains("f32") && !kernel_code.contains("f64"));
    assert!(!kernel_code.contains(" * ") && !kernel_code.contains(" / "));
}

// =========================================================================
// Lower Semantic Regions BDD Steps (#130)
// =========================================================================
use uor_r4_graph_compiler::lower_semantic_regions::{
    BooleanLoweringCompiler, LoweredFixedPointScore, LoweringError,
};

#[given(
    "a reference semantic region with signature [true, false, true, true] and Hamming radius 1.0"
)]
fn bdd_given_ref_region(_w: &mut R4g1World) {}

#[when("the region is lowered into a LoweredBooleanRegion")]
fn bdd_lower_region_step(w: &mut R4g1World) {
    let (region, witness) = BooleanLoweringCompiler::lower_region(
        "reg_bdd_1",
        &[true, false, true, true],
        1.0,
        "cid_bdd_ref_101",
        101,
        0,
    )
    .unwrap();
    w.lower_bool_region = Some(region);
    w.lower_witness = Some(witness);
}

#[then("the integer predicate evaluates to true for signatures within Hamming distance 1")]
fn bdd_integer_predicate_within_distance(w: &mut R4g1World) {
    let region = w.lower_bool_region.as_ref().expect("region");
    // Exact 0b1101 = 13 (distance 0)
    assert!(region.evaluate_runtime_integer(0b1101));
    // Distance 1 (0b1100)
    assert!(region.evaluate_runtime_integer(0b1100));
}

#[then("evaluates to false for signatures outside Hamming distance 1")]
fn bdd_integer_predicate_outside_distance(w: &mut R4g1World) {
    let region = w.lower_bool_region.as_ref().expect("region");
    // Distance 2 (0b0000)
    assert!(!region.evaluate_runtime_integer(0b0000));
}

#[then("a LoweringWitnessEntry is recorded")]
fn bdd_witness_recorded_check(w: &mut R4g1World) {
    let witness = w.lower_witness.as_ref().expect("witness");
    assert_eq!(witness.reference_cid, "cid_bdd_ref_101");
}

#[given("floating-point scores 1.5, 500.0, and -500.0")]
fn bdd_given_float_scores(_w: &mut R4g1World) {}

#[when("scores are quantized into Q8.8 fixed-point representation")]
fn bdd_quantize_scores_step(w: &mut R4g1World) {
    w.lower_q_normal = Some(LoweredFixedPointScore::quantize_q88(1.5).unwrap());
    w.lower_q_max = Some(LoweredFixedPointScore::quantize_q88(500.0).unwrap());
    w.lower_q_min = Some(LoweredFixedPointScore::quantize_q88(-500.0).unwrap());
}

#[then("1.5 quantizes to 384 without saturation")]
fn bdd_quantize_1_5_check(w: &mut R4g1World) {
    let q = w.lower_q_normal.as_ref().expect("normal q");
    assert_eq!(q.q88_value, 384);
    assert!(!q.saturated);
}

#[then("extreme scores saturate at i16 MAX and i16 MIN")]
fn bdd_quantize_extreme_check(w: &mut R4g1World) {
    let q_max = w.lower_q_max.as_ref().expect("max q");
    assert_eq!(q_max.q88_value, i16::MAX);
    assert!(q_max.saturated);

    let q_min = w.lower_q_min.as_ref().expect("min q");
    assert_eq!(q_min.q88_value, i16::MIN);
    assert!(q_min.saturated);
}

#[given("a reference region with a 100-bit signature")]
fn bdd_given_100bit_sig(_w: &mut R4g1World) {}

#[when("region lowering is attempted")]
fn bdd_attempt_100bit_lowering(w: &mut R4g1World) {
    let long_sig = vec![true; 100];
    if let Err(e) =
        BooleanLoweringCompiler::lower_region("reg_overflow", &long_sig, 1.0, "cid_err", 101, 0)
    {
        w.lower_error = Some(e);
    }
}

#[then("lowering fails with an unrepresentable region error")]
fn bdd_unrepresentable_error_check(w: &mut R4g1World) {
    let err = w.lower_error.as_ref().expect("lower error");
    assert!(matches!(err, LoweringError::UnrepresentableRegion { .. }));
}

// =========================================================================
// Behavioral Probes BDD Steps (#128)
// =========================================================================
use uor_r4_graph_compiler::behavioral_probes::{
    BehavioralProbeError, BehavioralProbeHarness, ExpectedRelation, InterventionKind,
    InterventionRecord,
};

#[given("a baseline observation \"Context text sample\"")]
fn bdd_baseline_observation(w: &mut R4g1World) {
    w.probe_baseline_obs = "Context text sample".to_string();
}

#[when("an invariant surface variation probe and a sensitive goal change probe are evaluated")]
fn bdd_evaluate_probes(w: &mut R4g1World) {
    let obs = &w.probe_baseline_obs;
    let p_inv = InterventionRecord::new(
        obs,
        InterventionKind::SurfaceVariation,
        (0, 7),
        ExpectedRelation::Invariant,
        vec![0.9, 0.1],
        vec![0.905, 0.095],
    )
    .unwrap();

    let p_sens = InterventionRecord::new(
        obs,
        InterventionKind::GoalChange,
        (0, 7),
        ExpectedRelation::Sensitive,
        vec![0.9, 0.1],
        vec![0.1, 0.9],
    )
    .unwrap();

    let report = BehavioralProbeHarness::evaluate_suite(&[p_inv, p_sens], 0.05, 0.5).unwrap();
    w.probe_suite_report = Some(report);
}

#[then("both invariance and sensitivity expectations pass cleanly")]
fn bdd_invariance_sensitivity_pass(w: &mut R4g1World) {
    let report = w.probe_suite_report.as_ref().expect("report");
    assert_eq!(report.invariance_score, 1.0);
    assert_eq!(report.sensitivity_score, 1.0);
}

#[then("the anti-memorization guard succeeds")]
fn bdd_memorization_guard_succeeds(w: &mut R4g1World) {
    let report = w.probe_suite_report.as_ref().expect("report");
    assert!(report.memorization_check_passed);
}

#[given("a sensitive goal change probe that produces zero output divergence")]
fn bdd_zero_divergence_sensitive_probe(w: &mut R4g1World) {
    let p_mem = InterventionRecord::new(
        "Context text sample",
        InterventionKind::GoalChange,
        (0, 7),
        ExpectedRelation::Sensitive,
        vec![0.9, 0.1],
        vec![0.9, 0.1], // div = 0.0 -> memorization!
    )
    .unwrap();

    if let Err(e) = BehavioralProbeHarness::evaluate_suite(&[p_mem], 0.05, 0.5) {
        w.probe_suite_error = Some(e);
    }
}

#[when("the probe suite is evaluated by the behavioral harness")]
fn bdd_harness_eval_step(_w: &mut R4g1World) {}

#[then("evaluation fails with a memorization detected error")]
fn bdd_memorization_error_check(w: &mut R4g1World) {
    let err = w.probe_suite_error.as_ref().expect("suite error");
    assert!(matches!(
        err,
        BehavioralProbeError::MemorizationDetected { .. }
    ));
}

#[given("an observation of length 15")]
fn bdd_observation_len_15(_w: &mut R4g1World) {}

#[when("an intervention record is created with span [0..20]")]
fn bdd_create_out_of_bounds_span(w: &mut R4g1World) {
    if let Err(e) = InterventionRecord::new(
        "Short 15 char!!",
        InterventionKind::ContextAblation,
        (0, 20),
        ExpectedRelation::Invariant,
        vec![1.0],
        vec![1.0],
    ) {
        w.probe_record_error = Some(e);
    }
}

#[then("record creation fails with a span out of bounds error")]
fn bdd_span_out_of_bounds_check(w: &mut R4g1World) {
    let err = w.probe_record_error.as_ref().expect("record error");
    assert!(matches!(err, BehavioralProbeError::SpanOutOfBounds { .. }));
}

// =========================================================================
// Semantic State Space BDD Steps (#124)
// =========================================================================
use uor_r4_graph_compiler::semantic_state::{
    Action as SemAction, Belief as SemBelief, Constraint as SemConstraint, Goal as SemGoal,
    Region as SemRegion, SemanticState as SemState, SemanticStateError as SemError,
    Trajectory as SemTrajectory, TransitionEvaluator as SemEvaluator,
};

#[given("an initial semantic state \"s0\" with vector [0.0, 0.0] and signature [0]")]
fn bdd_initial_state_s0(w: &mut R4g1World) {
    w.state_s0 = Some(SemState::new("s0", vec![0.0, 0.0], vec![0], 1.0));
}

#[when(
    "a semantic action \"move_right\" with delta vector [1.0, 0.0] and mask flip [1] is applied"
)]
fn bdd_apply_move_right(w: &mut R4g1World) {
    let s0 = w.state_s0.as_ref().expect("initial state s0");
    let action = SemAction::new("move_right", vec![1.0, 0.0], vec![1]);
    let evaluator = SemEvaluator::new();
    w.state_eval_res = Some(evaluator.apply(s0, &action));
}

#[then("the transition succeeds with target state \"s0_move_right\"")]
fn bdd_transition_succeeds(w: &mut R4g1World) {
    let res = w.state_eval_res.as_ref().expect("transition result");
    assert!(res.is_ok());
    assert_eq!(res.as_ref().unwrap().id, "s0_move_right");
}

#[then("the target state has vector [1.0, 0.0] and signature [1]")]
fn bdd_target_state_values(w: &mut R4g1World) {
    let state = w.state_eval_res.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(state.vector, vec![1.0, 0.0]);
    assert_eq!(state.boolean_signature, vec![1]);
}

#[given("an initial semantic state \"s_invalid\" with negative vector [-1.0, 0.0]")]
fn bdd_initial_negative_state(w: &mut R4g1World) {
    w.state_s0 = Some(SemState::new("s_invalid", vec![-1.0, 0.0], vec![0], 1.0));
}

#[when("an action requiring non-negative coordinates is applied")]
fn bdd_apply_action_with_precondition(w: &mut R4g1World) {
    let s0 = w.state_s0.as_ref().expect("state");
    let action = SemAction::new("check_pos", vec![1.0, 0.0], vec![0])
        .with_precondition(|s| s.vector[0] >= 0.0);
    let evaluator = SemEvaluator::new();
    w.state_eval_res = Some(evaluator.apply(s0, &action));
}

#[then("the transition fails with a precondition error")]
fn bdd_transition_fails_precondition(w: &mut R4g1World) {
    let res = w.state_eval_res.as_ref().expect("res");
    assert!(matches!(res, Err(SemError::PreconditionFailed { .. })));
}

#[given("a hazard constraint centered at [5.0, 5.0] with radius 1.0")]
fn bdd_hazard_constraint(w: &mut R4g1World) {
    let danger_region = SemRegion::new("danger", vec![5.0, 5.0], 1.0, "Hazard Zone");
    let constraint = SemConstraint::new("no_hazard", danger_region);
    let mut eval = SemEvaluator::new();
    eval.add_constraint(constraint);
    w.hazard_evaluator = Some(eval);
}

#[given("an initial state at [0.0, 0.0]")]
fn bdd_initial_zero_state(w: &mut R4g1World) {
    w.state_s0 = Some(SemState::new("s_zero", vec![0.0, 0.0], vec![0], 1.0));
}

#[when("an action attempts to step to [5.0, 5.0]")]
fn bdd_step_into_hazard(w: &mut R4g1World) {
    let s0 = w.state_s0.as_ref().expect("state");
    let action = SemAction::new("step_hazard", vec![5.0, 5.0], vec![0]);
    let evaluator = w.hazard_evaluator.as_ref().expect("evaluator");
    w.state_eval_res = Some(evaluator.apply(s0, &action));
}

#[then("the transition fails with a forbidden state error")]
fn bdd_transition_fails_forbidden(w: &mut R4g1World) {
    let res = w.state_eval_res.as_ref().expect("res");
    assert!(matches!(res, Err(SemError::ForbiddenState { .. })));
}

#[given("a goal target region centered at [10.0, 10.0] with radius 2.0 and minimum confidence 0.8")]
fn bdd_goal_target_region(_w: &mut R4g1World) {}

#[when("a state \"s_target\" at [10.0, 11.0] with confidence 0.9 is evaluated")]
fn bdd_evaluate_goal_and_belief(w: &mut R4g1World) {
    let target_region = SemRegion::new("target", vec![10.0, 10.0], 2.0, "Goal Zone");
    let goal = SemGoal::new("reach_target", target_region.clone(), 0.8);
    let belief = SemBelief::new("target_belief", target_region, 0.5);

    let s_target = SemState::new("s_target", vec![10.0, 11.0], vec![1], 0.9);
    let s_far = SemState::new("s_far", vec![0.0, 0.0], vec![0], 0.9);

    w.goal_satisfied = Some(goal.is_satisfied_by(&s_target));
    w.belief_in = Some(belief.evaluate(&s_target));
    w.belief_out = Some(belief.evaluate(&s_far));
}

#[then("the goal is satisfied by the state")]
fn bdd_goal_satisfied_check(w: &mut R4g1World) {
    assert_eq!(w.goal_satisfied, Some(true));
}

#[then("the belief likelihood is higher than a state at [0.0, 0.0]")]
fn bdd_belief_higher_check(w: &mut R4g1World) {
    let b_in = w.belief_in.expect("belief in");
    let b_out = w.belief_out.expect("belief out");
    assert!(b_in > b_out);
}

#[given("a trajectory with maximum 2 steps")]
fn bdd_max_2_steps_trajectory(w: &mut R4g1World) {
    w.state_s0 = Some(SemState::new("s_init", vec![0.0], vec![0], 1.0));
}

#[when("3 step actions are applied sequentially")]
fn bdd_apply_3_steps(w: &mut R4g1World) {
    let s0 = w.state_s0.take().expect("init state");
    let evaluator = SemEvaluator::new();
    let action = SemAction::new("step", vec![1.0], vec![0]);
    let mut traj = SemTrajectory::new(s0, 2);

    let _ = traj.step(&action, &evaluator);
    let _ = traj.step(&action, &evaluator);
    let res = traj.step(&action, &evaluator);

    if let Err(e) = res {
        w.trajectory_error = Some(e);
    }
}

#[then("the 3rd step fails with a maximum steps exceeded error")]
fn bdd_max_steps_error_check(w: &mut R4g1World) {
    let err = w.trajectory_error.as_ref().expect("trajectory error");
    assert!(matches!(err, SemError::MaxStepsExceeded { limit: 2 }));
}

#[tokio::main]
async fn main() {
    R4g1World::cucumber()
        .fail_on_skipped()
        .run_and_exit(concat!(env!("CARGO_MANIFEST_DIR"), "/features/suites"))
        .await;
}
