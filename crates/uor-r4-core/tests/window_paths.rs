//! Fast integration witnesses for the migration seams: the corpus-free
//! window path (kernel and plain forms identical to the corpus path), the
//! TLA3 container roundtrip (PROOF.md P5(a) as a unit test), and the TLS1
//! store container.

use uor_r4_core::transformerless::compiler::{self, Corpus, STAGES, WINDOW};
use uor_r4_core::transformerless::runtime::{self, OpKernel, Store};

fn fixture() -> (compiler::Compiled, Corpus) {
    let dir = env!("CARGO_MANIFEST_DIR");
    let bytes = std::fs::read(format!("{dir}/tests/fixtures/tless_artifacts.bin")).unwrap();
    let art = compiler::parse_artifacts(&bytes).expect("fixture TLA3 parses");
    let c = compiler::load_corpus_from(
        &format!("{dir}/tests/fixtures/c_meta.bin"),
        &format!("{dir}/tests/fixtures/c_recs.bin"),
    )
    .expect("fixture corpus loads");
    (art, c)
}

#[test]
fn container_roundtrip_byte_identical() {
    let dir = env!("CARGO_MANIFEST_DIR");
    let bytes = std::fs::read(format!("{dir}/tests/fixtures/tless_artifacts.bin")).unwrap();
    let art = compiler::parse_artifacts(&bytes).expect("parse");
    assert_eq!(
        compiler::artifact_bytes(&art),
        bytes,
        "parse → serialize is byte-identical (P5(a) save → load → save)"
    );
    let baseline: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(format!("{dir}/tests/fixtures/baseline_kappa.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        format!("blake3:{}", blake3::hash(&bytes).to_hex()),
        baseline["bundle_derived_macos"]["container"]["kappa"]
            .as_str()
            .unwrap(),
        "fixture container matches the baseline κ"
    );
}

#[test]
fn window_path_matches_corpus_path() {
    let (art, c) = fixture();
    let rot = compiler::derive_rotations();
    let mut k = OpKernel::default();
    let mut rt = runtime::Runtime::new(&art);
    let mut checked = 0usize;
    for i in WINDOW - 1..c.n {
        // positions whose full WINDOW history is in-story
        if (1..WINDOW).any(|back| c.story[i - back] != c.story[i]) {
            continue;
        }
        let window: Vec<u16> = ((i + 1 - WINDOW)..=i).map(|idx| c.input[idx]).collect();
        assert_eq!(
            runtime::bundle_window_plain(&art, &rot, &window),
            runtime::bundle_plain(&art, &rot, &c, i),
            "plain bundle at position {i}"
        );
        assert_eq!(
            runtime::bundle_window_kernel(&mut k, &art, &rot, &window),
            runtime::bundle_plain(&art, &rot, &c, i),
            "kernel bundle at position {i}"
        );
        assert_eq!(
            rt.assign_window(&window),
            runtime::code_plain(&art, &rot, &c, i),
            "graded class code at position {i}"
        );
        checked += 1;
        if checked == 256 {
            break;
        }
    }
    assert_eq!(checked, 256, "enough full-history positions sampled");
}

#[test]
fn store_container_roundtrip() {
    let mut store: Store = (0..=STAGES).map(|_| Default::default()).collect();
    store[0].entry(vec![]).or_default().insert(7, 42);
    store[0].entry(vec![]).or_default().insert(9, 3);
    store[2].entry(vec![5, 6]).or_default().insert(11, 2);
    store[4].entry(vec![1, 2, 3, 4]).or_default().insert(12, 1);
    let bytes = runtime::store_bytes(&store);
    assert!(bytes.starts_with(b"TLS1"));
    let back = runtime::parse_store(&bytes).expect("TLS1 parses");
    assert_eq!(back, store, "store bytes → parse is the identity");
    assert!(
        runtime::parse_store(&bytes[..bytes.len() - 1]).is_none(),
        "truncated container rejected"
    );
    assert!(
        runtime::parse_store(b"XXXX").is_none(),
        "bad magic rejected"
    );
}

#[test]
fn predict_witness_depth_and_count() {
    let mut store: Store = (0..=STAGES).map(|_| Default::default()).collect();
    store[0].entry(vec![]).or_default().insert(1, 10);
    store[1].entry(vec![9]).or_default().insert(2, 5);
    let hit = runtime::predict_witness_plain(&store, &[9, 0, 0, 0]);
    assert_eq!(
        (hit.token, hit.depth, hit.count),
        (2, 1, 5),
        "deepest populated class answers with its evidence count"
    );
    let miss = runtime::predict_witness_plain(&store, &[7, 0, 0, 0]);
    assert_eq!(
        (miss.token, miss.depth, miss.count),
        (1, 0, 10),
        "backoff reaches level 0"
    );
}
