use serde_json::json;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::path::PathBuf;
use uor_r4_object_store::address_stage0::{
    build_manifest, clears_primary, collect_original_rows, corpus_value_leaks, mark_parity_stop,
    normalize_emission, parse_manifest, planted_wrong_holds, primary_pass, set_parity_aggregates,
    u16_le_tokens, validate_head, validate_manifest, values_missing_from_corpus, OriginalRow,
    PairedWins, MANIFEST_SHA256, RETAINED_RECEIPT_SHA256, STAGE0_SCHEMA, STAGE_T_ARCHIVE_SHA256,
};
use uor_r4_tokenizer::ByteBpeTokenizer;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const TOKENIZER_JSON: &str = r#"{
    "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
    "added_tokens":[{"id":8,"content":"<end>"}],
    "model":{"type":"BPE","vocab":{"a":0,"b":1,"c":2,"ab":3,"bc":4,"x":5,"y":6,"z":7},"merges":["b c","a b"]}
}"#;

fn tokenizer() -> TestResult<ByteBpeTokenizer> {
    ByteBpeTokenizer::from_tokenizer_json_bytes(TOKENIZER_JSON.as_bytes())
        .ok_or_else(|| "invalid fixture tokenizer".into())
}

#[test]
fn manifest_is_deterministic_matched_and_leak_free() -> TestResult {
    let tokenizer = tokenizer()?;
    let first = build_manifest(&tokenizer)?;
    let second = build_manifest(&tokenizer)?;
    assert_eq!(
        serde_json::to_string_pretty(&first)?,
        serde_json::to_string_pretty(&second)?
    );
    validate_manifest(&first)?;
    assert_eq!(first.rows.len(), 40);
    let mut values = std::collections::BTreeSet::new();
    for row in &first.rows {
        assert!(values.insert(row.value.clone()));
        assert_ne!(row.value, row.random_hint);
        assert_eq!(row.value_tokens.len(), row.random_tokens.len());
        assert_eq!(row.prompt_tokens.len(), row.random_prompt_tokens.len());
        assert!(row.prompt_tokens.len() <= 192);
        assert!(!row.no_hint_prompt.contains(&row.value));
        assert!(!row.random_prompt.contains(&row.value));
        assert!(!row.no_hint_prompt.contains("supplied value is"));
        assert!(row.prompt.contains("supplied value is"));
        assert!(row.random_prompt.contains("supplied value is"));
    }
    Ok(())
}

#[test]
fn decoder_accepts_only_one_exact_eight_letter_value() -> TestResult {
    assert_eq!(normalize_emission("abcdefgh")?, "abcdefgh");
    assert_eq!(normalize_emission("  a b c d e f g h  ")?, "abcdefgh");
    assert!(normalize_emission("abcdefg").is_err());
    assert!(normalize_emission("abcdefghi").is_err());
    assert!(normalize_emission("abcdefgh.").is_err());
    assert!(normalize_emission("abcde-fg").is_err());
    Ok(())
}

#[test]
fn corpus_leak_check_finds_an_exact_value_token_sequence() -> TestResult {
    let tokenizer = tokenizer()?;
    let manifest = build_manifest(&tokenizer)?;
    assert!(values_missing_from_corpus(&manifest, &[]));
    assert!(!values_missing_from_corpus(
        &manifest,
        &manifest.rows[0].value_tokens
    ));
    Ok(())
}

#[test]
fn validator_rejects_duplicate_values() -> TestResult {
    let tokenizer = tokenizer()?;
    let mut manifest = build_manifest(&tokenizer)?;
    manifest.rows[1].value = manifest.rows[0].value.clone();
    assert!(validate_manifest(&manifest).is_err());
    Ok(())
}

fn pinned_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/labs/m1-address-as-read/stage0-copy.json")
}

#[test]
fn pinned_copy_manifest_hashes_and_parses() -> TestResult {
    let path = pinned_manifest_path();
    let bytes = std::fs::read(&path)?;
    let digest = format!("{:x}", Sha256::digest(&bytes));
    assert_eq!(
        digest,
        MANIFEST_SHA256,
        "the pinned Stage 0 manifest moved: {}",
        path.display()
    );
    assert_eq!(
        MANIFEST_SHA256,
        "ea36011870e3f6d0add87e9fc1d25115ffc77f2a9206d13bfe304c6785351fa5"
    );
    let manifest = parse_manifest(&bytes)?;
    assert_eq!(manifest.schema, STAGE0_SCHEMA);
    assert_eq!(manifest.rows.len(), 40);
    for row in &manifest.rows {
        // The runner's copy probe is non-vacuous only with real tokens.
        assert!(!row.value_tokens.is_empty());
        assert!(!row.prompt_tokens.is_empty());
        assert!(!row.random_prompt_tokens.is_empty());
        assert!(!row.no_hint_prompt_tokens.is_empty());
        assert!(row.prompt_tokens.len() <= 192);
        assert!(row.random_prompt_tokens.len() <= 192);
    }
    validate_manifest(&manifest)?;
    Ok(())
}

#[test]
fn parse_manifest_rejects_an_unknown_field_or_wrong_schema() -> TestResult {
    let tokenizer = tokenizer()?;
    let manifest = build_manifest(&tokenizer)?;
    let mut value = serde_json::to_value(&manifest)?;
    value["rows"][0]["extra"] = json!(1);
    assert!(parse_manifest(&serde_json::to_vec(&value)?).is_err());
    let mut value = serde_json::to_value(&manifest)?;
    value["schema"] = json!("uor-r4.address-stage0-copy/2");
    assert!(parse_manifest(&serde_json::to_vec(&value)?).is_err());
    Ok(())
}

#[test]
fn original_report_rows_are_collected_recursively_in_traversal_order() -> TestResult {
    let report = json!({
        "schema": "uor-r4.discrete-corpus-driver/1",
        "aggregate": {"scored_rows": 2, "top1_correct": 1, "target_admitted_rows": 2},
        "batches": [
            {"rows": [
                {"target_position": 41, "target_token": 1155,
                 "top8": [552, 561, 935, 542, 690, 772, 971, 1258],
                 "top1_correct": 0, "admitted_cells": 8, "unrelated": true}
            ]},
            {"nested": {"deeper": [
                {"target_position": 42, "target_token": 1156,
                 "top8": [1156, 2, 3, 4, 5, 6, 7, 8],
                 "top1_correct": 1, "admitted_cells": 7}
            ]}}
        ],
        "count_reference": {"top8": [1, 2, 3, 4, 5, 6, 7, 8], "top1_correct": 1, "scored_rows": 2}
    });
    let rows = collect_original_rows(&report)?;
    assert_eq!(
        rows,
        vec![
            OriginalRow {
                target_position: 41,
                target_token: 1155,
                top8: [552, 561, 935, 542, 690, 772, 971, 1258],
                top1_correct: 0,
                admitted_cells: 8,
            },
            OriginalRow {
                target_position: 42,
                target_token: 1156,
                top8: [1156, 2, 3, 4, 5, 6, 7, 8],
                top1_correct: 1,
                admitted_cells: 7,
            },
        ]
    );
    let malformed = json!({"target_position": 1, "target_token": 2,
                           "top8": [1, 2], "top1_correct": 0, "admitted_cells": 8});
    assert!(collect_original_rows(&malformed).is_err());
    let missing = json!({"target_position": 1, "target_token": 2,
                         "top8": [1, 2, 3, 4, 5, 6, 7, 8], "top1_correct": 0});
    assert!(collect_original_rows(&missing)?.is_empty());
    Ok(())
}

#[test]
fn primary_margin_uses_the_registered_strict_floor() {
    assert!(clears_primary(25, 0));
    assert!(!clears_primary(24, 0));
    assert!(clears_primary(26, 1));
    // d = 28 and d^2 = 4(b + c) exactly; the clause is strict.
    assert!(!clears_primary(112, 84));
    assert!(clears_primary(113, 84));
    assert!(!clears_primary(0, 0));
    assert!(primary_pass(
        PairedWins { b: 25, c: 0 },
        PairedWins { b: 26, c: 1 }
    ));
    assert!(!primary_pass(
        PairedWins { b: 25, c: 0 },
        PairedWins { b: 24, c: 0 }
    ));
}

#[test]
fn mismatch_stop_report_carries_all_three_aggregates_and_the_delta() {
    let mut report = json!({
        "schema": "uor-r4.address-stage0-parity/1",
        "rows": 14_287,
        "comparison": {"mismatched_rows": 3, "first_mismatches": []},
    });
    // The runner calls this immediately after the D11 aggregate and before
    // its mismatch, top-1/top-8 and admission stops.
    let admission_delta = set_parity_aggregates(
        &mut report,
        (460, 1_500, 5_010),
        (455, 1_484, 5_009),
        14_287,
    );
    // The runner's stop_parity uses this same transition.
    mark_parity_stop(&mut report, "3 per-row parity mismatches");

    assert_eq!(report["pass"].as_bool(), Some(false));
    assert_eq!(
        report["instrument_stop"].as_str(),
        Some("3 per-row parity mismatches")
    );
    assert_eq!(report["comparison"]["mismatched_rows"].as_u64(), Some(3));
    assert_eq!(report["d11_aggregate"]["rows"].as_u64(), Some(14_287));
    assert_eq!(report["d11_aggregate"]["top1"].as_u64(), Some(460));
    assert_eq!(report["d11_aggregate"]["top8"].as_u64(), Some(1_500));
    assert_eq!(
        report["d11_aggregate"]["target_admitted"].as_u64(),
        Some(5_010)
    );
    assert_eq!(report["original_aggregate"]["rows"].as_u64(), Some(14_287));
    assert_eq!(report["original_aggregate"]["top1"].as_u64(), Some(455));
    assert_eq!(report["original_aggregate"]["top8"].as_u64(), Some(1_484));
    assert_eq!(
        report["original_aggregate"]["target_admitted"].as_u64(),
        Some(5_009)
    );
    assert_eq!(report["position_strip"]["d11_top1"].as_u64(), Some(460));
    assert_eq!(
        report["position_strip"]["retained_top1"].as_u64(),
        Some(455)
    );
    assert_eq!(report["position_strip"]["top1_delta"].as_i64(), Some(5));
    assert_eq!(report["position_strip"]["d11_top8"].as_u64(), Some(1_500));
    assert_eq!(
        report["position_strip"]["retained_top8"].as_u64(),
        Some(1_484)
    );
    assert_eq!(report["position_strip"]["top8_delta"].as_i64(), Some(16));
    assert_eq!(
        report["position_strip"]["d11_target_admitted"].as_u64(),
        Some(5_010)
    );
    assert_eq!(
        report["position_strip"]["original_target_admitted"].as_u64(),
        Some(5_009)
    );
    assert_eq!(
        report["position_strip"]["admission_delta"].as_i64(),
        Some(1)
    );
    assert_eq!(
        report["position_strip"]["admission_aggregate_matches"].as_bool(),
        Some(false)
    );
    assert_eq!(admission_delta, 1);
}

#[test]
fn planted_wrong_value_must_differ_on_every_applicable_row() -> TestResult {
    let tokenizer = tokenizer()?;
    let manifest = build_manifest(&tokenizer)?;
    let different: Vec<Option<String>> = manifest
        .rows
        .iter()
        .map(|row| Some(row.random_hint.clone()))
        .collect();
    assert!(planted_wrong_holds(&manifest, &different)?);
    let mut one_exact = different.clone();
    one_exact[3] = Some(manifest.rows[3].value.clone());
    assert!(!planted_wrong_holds(&manifest, &one_exact)?);
    assert!(!planted_wrong_holds(&manifest, &vec![None; 40])?);
    assert!(planted_wrong_holds(&manifest, &different[..39]).is_err());
    Ok(())
}

#[test]
fn u16_corpus_reads_skip_the_uort_header_and_are_little_endian() -> TestResult {
    assert_eq!(u16_le_tokens(&[1, 0, 2, 0, 255, 255])?, vec![1, 2, 65_535]);
    let mut bytes = vec![0u8; 64];
    bytes[..4].copy_from_slice(b"UORT");
    bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
    bytes[8..16].copy_from_slice(&2u64.to_le_bytes());
    bytes[16..20].copy_from_slice(&4096u32.to_le_bytes());
    bytes.extend_from_slice(&[7, 0, 9, 0]);
    assert_eq!(u16_le_tokens(&bytes)?, vec![7, 9]);
    assert!(u16_le_tokens(&[1]).is_err());
    assert!(u16_le_tokens(&bytes[..40]).is_err());
    Ok(())
}

#[test]
fn corpus_value_leaks_names_rows_whose_exact_tokens_appear() -> TestResult {
    let tokenizer = tokenizer()?;
    let manifest = build_manifest(&tokenizer)?;
    assert!(corpus_value_leaks(&manifest, &[]).is_empty());
    let appears = |haystack: &[u32], needle: &[u32]| {
        !needle.is_empty()
            && haystack
                .windows(needle.len())
                .any(|window| window == needle)
    };
    for row in &manifest.rows {
        let leaks = corpus_value_leaks(&manifest, &row.value_tokens);
        if row.value_tokens.is_empty() {
            // The tiny fixture tokenizer drops bytes outside its vocabulary;
            // an empty sequence is never reported as a leak.
            assert!(!leaks.contains(&row.id));
            continue;
        }
        assert!(leaks.contains(&row.id));
        // The fixture tokenizer can map one value's token sequence into
        // another's; every leaked row must really appear in this corpus.
        for id in &leaks {
            let leaked = manifest.rows.iter().find(|r| &r.id == id).unwrap();
            assert!(appears(&row.value_tokens, &leaked.value_tokens));
        }
    }
    Ok(())
}

#[test]
fn stage0_head_must_be_a_full_lowercase_commit() {
    let full = "0123456789abcdef0123456789abcdef01234567";
    assert!(validate_head(full).is_ok());
    assert!(validate_head(&full[..39]).is_err());
    assert!(validate_head(&format!("{}g", &full[..39])).is_err());
    assert!(validate_head(&full.to_uppercase()).is_err());
    assert!(validate_head("").is_err());
}

#[test]
fn retained_receipt_and_stage_t_archive_pins_are_registered() {
    assert_eq!(
        RETAINED_RECEIPT_SHA256,
        "a7d23490c511fc7a391c95546646cd6aaef391950a5fb541fd13d2029a6edf8e"
    );
    assert_eq!(
        STAGE_T_ARCHIVE_SHA256,
        "c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6"
    );
}
