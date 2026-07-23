use std::time::{SystemTime, UNIX_EPOCH};
use uor_r4_core::transformerless::compiler::load_corpus_from;
use uor_r4_core::transformerless::scenarios::export_hf_bytelevel_tokenizer_with_lengths;

fn unique_path(name: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("uor-r4-{name}-{nanos}"))
}

fn write_meta(path: &std::path::Path, n: u64, stories: u64) {
    let mut meta = [0u8; 25];
    meta[0..8].copy_from_slice(&n.to_le_bytes());
    meta[8..16].copy_from_slice(&stories.to_le_bytes());
    meta[16..24].copy_from_slice(&0x5EED_u64.to_le_bytes());
    meta[24] = 1;
    std::fs::write(path, meta).expect("write meta");
}

#[test]
fn load_corpus_v3_preserves_span_and_byte_anchors() {
    let meta = unique_path("anchors-v3.meta");
    let recs = unique_path("anchors-v3.records");
    write_meta(&meta, 2, 1);
    let mut bytes = Vec::new();
    for (next, span_start, span_end, byte_start, byte_end) in [
        (11u32, 0u32, 1u32, 0u32, 3u32),
        (12u32, 1u32, 2u32, 3u32, 5u32),
    ] {
        let mut record = [0u8; 48];
        record[0..4].copy_from_slice(&7u32.to_le_bytes());
        record[4..8].copy_from_slice(&next.to_le_bytes());
        record[8..12].copy_from_slice(&next.to_le_bytes());
        record[20..24].copy_from_slice(&100u32.to_le_bytes());
        record[32..36].copy_from_slice(&span_start.to_le_bytes());
        record[36..40].copy_from_slice(&span_end.to_le_bytes());
        record[40..44].copy_from_slice(&byte_start.to_le_bytes());
        record[44..48].copy_from_slice(&byte_end.to_le_bytes());
        bytes.extend_from_slice(&record);
    }
    std::fs::write(&recs, bytes).expect("write recs");

    let corpus = load_corpus_from(
        meta.to_str().expect("meta path utf-8"),
        recs.to_str().expect("records path utf-8"),
    )
    .expect("load corpus");

    assert_eq!(corpus.span_start, vec![0, 1]);
    assert_eq!(corpus.span_end, vec![1, 2]);
    assert_eq!(corpus.byte_start, vec![0, 3]);
    assert_eq!(corpus.byte_end, vec![3, 5]);
}

#[test]
fn load_corpus_v4_preserves_span_and_byte_anchors() {
    let meta = unique_path("anchors-v4.meta");
    let recs = unique_path("anchors-v4.records");
    write_meta(&meta, 2, 1);
    let mut bytes = Vec::new();
    for (next, span_start, span_end, byte_start, byte_end) in [
        (11u32, 0u32, 1u32, 0u32, 3u32),
        (12u32, 1u32, 2u32, 3u32, 5u32),
    ] {
        let mut record = [0u8; 88];
        record[0..4].copy_from_slice(&7u32.to_le_bytes());
        record[4..8].copy_from_slice(&next.to_le_bytes());
        record[8..12].copy_from_slice(&next.to_le_bytes());
        record[40..44].copy_from_slice(&100u32.to_le_bytes());
        record[72..76].copy_from_slice(&span_start.to_le_bytes());
        record[76..80].copy_from_slice(&span_end.to_le_bytes());
        record[80..84].copy_from_slice(&byte_start.to_le_bytes());
        record[84..88].copy_from_slice(&byte_end.to_le_bytes());
        bytes.extend_from_slice(&record);
    }
    std::fs::write(&recs, bytes).expect("write recs");

    let corpus = load_corpus_from(
        meta.to_str().expect("meta path utf-8"),
        recs.to_str().expect("records path utf-8"),
    )
    .expect("load corpus");

    assert_eq!(corpus.span_start, vec![0, 1]);
    assert_eq!(corpus.span_end, vec![1, 2]);
    assert_eq!(corpus.byte_start, vec![0, 3]);
    assert_eq!(corpus.byte_end, vec![3, 5]);
}

#[test]
fn load_corpus_v2_backfills_token_spans_and_unknown_bytes() {
    let meta = unique_path("anchors-v2.meta");
    let recs = unique_path("anchors-v2.records");
    write_meta(&meta, 3, 2);
    let mut bytes = Vec::new();
    for (story, next) in [(2u32, 21u32), (2u32, 22u32), (3u32, 31u32)] {
        let mut record = [0u8; 32];
        record[0..4].copy_from_slice(&story.to_le_bytes());
        record[4..8].copy_from_slice(&next.to_le_bytes());
        record[8..12].copy_from_slice(&next.to_le_bytes());
        record[20..24].copy_from_slice(&100u32.to_le_bytes());
        bytes.extend_from_slice(&record);
    }
    std::fs::write(&recs, bytes).expect("write recs");

    let corpus = load_corpus_from(
        meta.to_str().expect("meta path utf-8"),
        recs.to_str().expect("records path utf-8"),
    )
    .expect("load corpus");

    assert_eq!(corpus.span_start, vec![0, 1, 0]);
    assert_eq!(corpus.span_end, vec![1, 2, 1]);
    assert_eq!(corpus.byte_start, vec![u32::MAX; 3]);
    assert_eq!(corpus.byte_end, vec![u32::MAX; 3]);
}

#[test]
fn tokenizer_export_returns_per_token_byte_lengths() {
    let source = unique_path("tokenizer.json");
    let destination = unique_path("tokenizer.bin");
    std::fs::write(&source, r#"{"model":{"vocab":{"a":0,"b":1,"ab":2}}}"#)
        .expect("write tokenizer json");

    let lengths = export_hf_bytelevel_tokenizer_with_lengths(&source, &destination)
        .expect("export tokenizer with lengths");
    assert_eq!(lengths, vec![1, 1, 2]);
}
