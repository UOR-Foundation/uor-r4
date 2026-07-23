//! Cucumber runner for behavior-level R4G1 checks.
//!
//! The feature files live under `features/suites`, following the upstream
//! Hologram layout. Keep the scenarios focused on externally meaningful
//! behavior; implementation details stay in the server module.

use cucumber::{given, then, when, World};
use std::path::Path;
use uor_r4_core::transformerless::bott_fock::BottFockContextStore;
use uor_r4_core::transformerless::compiler::SIG_BYTES;
use uor_r4_core::transformerless::cover::Observation;
use uor_r4_core::transformerless::endomorphism::EndomorphismAlgebra;
use uor_r4_core::transformerless::lie_jordan::{universal_product_u8, LieJordanSplit};
use uor_r4_core::transformerless::quantum_cover::{
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
    assert!(source.contains(r#"<option value="r4g1" selected>R4G1 Graph (Server)</option>"#));
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

#[tokio::main]
async fn main() {
    R4g1World::cucumber()
        .fail_on_skipped()
        .run_and_exit(concat!(env!("CARGO_MANIFEST_DIR"), "/features/suites"))
        .await;
}
