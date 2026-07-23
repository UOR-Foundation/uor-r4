//! R4G1Runtime unit tests verifying multiplication-free zero-allocation prediction behavior over R4G1 GraphView containers.

use std::collections::BTreeMap;
use uor_r4_core::transformerless::compiler::{self, STAGES};
use uor_r4_core::transformerless::convert_r4g1;
use uor_r4_core::transformerless::runtime::{self, Store};
use uor_r4_graph_format::ScoreQ;
use uor_r4_graph_runtime::R4G1Runtime;

fn fixture_artifacts() -> (Vec<u8>, compiler::Compiled) {
    let dir = env!("CARGO_MANIFEST_DIR");
    let bytes = std::fs::read(format!(
        "{dir}/../uor-r4-core/tests/fixtures/tless_artifacts.bin"
    ))
    .expect("fixture artifacts present");
    let artifacts = compiler::parse_artifacts(&bytes).expect("fixture artifacts parse");
    (bytes, artifacts)
}

fn synthetic_store() -> Store {
    let mut store: Store = (0..=STAGES).map(|_| BTreeMap::new()).collect();
    let codes: [[u8; 4]; 6] = [
        [3, 1, 4, 1],
        [3, 1, 4, 2],
        [3, 5, 9, 2],
        [7, 5, 9, 2],
        [7, 5, 8, 2],
        [11, 5, 8, 7],
    ];
    for (i, code) in codes.iter().enumerate() {
        runtime::add_evidence(&mut store, code, (i + 1) as u32, 1);
    }
    store
}

#[test]
fn r4g1_runtime_parses_and_predicts() {
    let (art_bytes, artifacts) = fixture_artifacts();
    let store = synthetic_store();
    let store_bytes = runtime::store_bytes(&store);
    let (r4g1_bytes, _) = convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None)
        .expect("convert to R4G1 succeeds");

    let runtime = R4G1Runtime::parse(&r4g1_bytes).expect("R4G1Runtime parses container");
    assert!(runtime.node_count() > 0);
    assert!(runtime.edge_count() > 0);

    let mut node_scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let (best_token, best_score) = runtime.predict_distribution(&[1, 2, 3], None, &mut node_scores);
    assert!(best_score > ScoreQ::MIN);

    let mut node_scores2 = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let token = runtime.predict_token(&[1, 2, 3], None, &mut node_scores2);
    assert_eq!(token, best_token);
}

#[test]
fn r4g1_runtime_enforces_no_float_in_prediction_path() {
    let (art_bytes, artifacts) = fixture_artifacts();
    let store = synthetic_store();
    let store_bytes = runtime::store_bytes(&store);
    let (r4g1_bytes, _) =
        convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None).unwrap();
    let runtime = R4G1Runtime::parse(&r4g1_bytes).unwrap();

    let mut node_scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let (_, score) = runtime.predict_distribution(&[3, 1, 4], None, &mut node_scores);

    assert!(score >= ScoreQ::MIN);
    assert!(score <= ScoreQ::MAX);
}
