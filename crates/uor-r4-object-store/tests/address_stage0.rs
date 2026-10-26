use std::error::Error;
use uor_r4_object_store::address_stage0::{
    build_manifest, normalize_emission, validate_manifest, values_missing_from_corpus,
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
