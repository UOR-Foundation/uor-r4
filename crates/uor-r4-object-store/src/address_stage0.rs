//! Deterministic Stage 0 copy-probe manifest and exact-emission protocol.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_tokenizer::ByteBpeTokenizer;

pub const STAGE0_SCHEMA: &str = "uor-r4.address-stage0-copy/1";
pub const STAGE0_SEED: &str = "m1-address-stage0-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyManifest {
    pub schema: String,
    pub seed: String,
    pub rows: Vec<CopyRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyRow {
    pub id: String,
    pub value: String,
    pub random_hint: String,
    pub value_tokens: Vec<u32>,
    pub random_tokens: Vec<u32>,
    pub prompt: String,
    pub random_prompt: String,
    pub no_hint_prompt: String,
    pub prompt_tokens: Vec<u32>,
    pub random_prompt_tokens: Vec<u32>,
    pub no_hint_prompt_tokens: Vec<u32>,
    pub prompt_sha256: String,
    pub random_prompt_sha256: String,
    pub no_hint_prompt_sha256: String,
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn value(seed: &str) -> String {
    Sha256::digest(seed.as_bytes())
        .iter()
        .take(8)
        .map(|byte| char::from(b'a' + byte % 26))
        .collect()
}

fn spaced(value: &str) -> String {
    value
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

fn prompt(value: &str) -> String {
    format!(
        "supplied value is {}.\nQuestion: what is the supplied value?\nAnswer: ",
        spaced(value)
    )
}

fn query() -> &'static str {
    "Question: what is the supplied value?\nAnswer: "
}

pub fn build_manifest(tokenizer: &ByteBpeTokenizer) -> Result<CopyManifest, String> {
    let mut rows = Vec::new();
    for index in 0..40 {
        let exact_value = value(&format!("{STAGE0_SEED}|value-{index}"));
        let exact_tokens = tokenizer.encode(&exact_value);
        let exact_prompt = prompt(&exact_value);
        let exact_prompt_tokens = tokenizer.encode(&exact_prompt);
        let mut random_hint = String::new();
        let mut random_tokens = Vec::new();
        let mut matched = false;
        for candidate in 0..1024 {
            let hint = value(&format!("{STAGE0_SEED}|random-{index}-{candidate}"));
            let tokens = tokenizer.encode(&hint);
            let candidate_prompt_tokens = tokenizer.encode(&prompt(&hint));
            if hint != exact_value
                && tokens.len() == exact_tokens.len()
                && candidate_prompt_tokens.len() == exact_prompt_tokens.len()
            {
                random_hint = hint;
                random_tokens = tokens;
                matched = true;
                break;
            }
        }
        if !matched {
            return Err(format!(
                "no token-length-matched random hint at row {index}"
            ));
        }
        let random_prompt = prompt(&random_hint);
        let no_hint_prompt = query().to_owned();
        let row = CopyRow {
            id: format!("copy-{index:03}"),
            value_tokens: exact_tokens,
            random_tokens,
            prompt_tokens: tokenizer.encode(&exact_prompt),
            random_prompt_tokens: tokenizer.encode(&random_prompt),
            no_hint_prompt_tokens: tokenizer.encode(&no_hint_prompt),
            prompt_sha256: sha(exact_prompt.as_bytes()),
            random_prompt_sha256: sha(random_prompt.as_bytes()),
            no_hint_prompt_sha256: sha(no_hint_prompt.as_bytes()),
            value: exact_value,
            random_hint,
            prompt: exact_prompt,
            random_prompt,
            no_hint_prompt,
        };
        if row.prompt_tokens.len() > 192 || row.random_prompt_tokens.len() > 192 {
            return Err(format!("prompt exceeds 192 tokens at row {index}"));
        }
        if row.value_tokens.len() != row.random_tokens.len() {
            return Err(format!("hint token lengths differ at row {index}"));
        }
        if row.prompt_tokens.len() != row.random_prompt_tokens.len() {
            return Err(format!("prompt token lengths differ at row {index}"));
        }
        rows.push(row);
    }
    let manifest = CopyManifest {
        schema: STAGE0_SCHEMA.to_owned(),
        seed: STAGE0_SEED.to_owned(),
        rows,
    };
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub fn validate_manifest(manifest: &CopyManifest) -> Result<(), String> {
    if manifest.schema != STAGE0_SCHEMA || manifest.seed != STAGE0_SEED {
        return Err("manifest schema or seed differs".into());
    }
    if manifest.rows.len() != 40 {
        return Err("copy probe needs exactly 40 rows".into());
    }
    let mut ids = std::collections::BTreeSet::new();
    let mut values = std::collections::BTreeSet::new();
    for row in &manifest.rows {
        if !ids.insert(&row.id) || !values.insert(&row.value) {
            return Err("duplicate row id or value".into());
        }
        if row.value.len() != 8
            || row.random_hint.len() != 8
            || !row.value.bytes().all(|b| b.is_ascii_lowercase())
            || !row.random_hint.bytes().all(|b| b.is_ascii_lowercase())
            || row.value == row.random_hint
        {
            return Err(format!("invalid value pair at {}", row.id));
        }
        if row.value_tokens.len() != row.random_tokens.len()
            || row.prompt_tokens.len() != row.random_prompt_tokens.len()
            || row.prompt_tokens.len() > 192
            || row.random_prompt_tokens.len() > 192
        {
            return Err(format!("token length mismatch at {}", row.id));
        }
        if row.prompt != prompt(&row.value)
            || row.random_prompt != prompt(&row.random_hint)
            || row.no_hint_prompt != query()
            || row.prompt_sha256 != sha(row.prompt.as_bytes())
            || row.random_prompt_sha256 != sha(row.random_prompt.as_bytes())
            || row.no_hint_prompt_sha256 != sha(row.no_hint_prompt.as_bytes())
        {
            return Err(format!("prompt or prompt hash mismatch at {}", row.id));
        }
        if row.no_hint_prompt.contains(&row.value) || row.random_prompt.contains(&row.value) {
            return Err(format!("answer leak at {}", row.id));
        }
    }
    Ok(())
}

pub fn values_missing_from_corpus(manifest: &CopyManifest, corpus: &[u32]) -> bool {
    fn contains(haystack: &[u32], needle: &[u32]) -> bool {
        !needle.is_empty()
            && haystack
                .windows(needle.len())
                .any(|window| window == needle)
    }
    manifest
        .rows
        .iter()
        .all(|row| !contains(corpus, &row.value_tokens))
}

pub fn normalize_emission(text: &str) -> Result<String, String> {
    let mut letters = text.chars().filter(|c| c.is_ascii_lowercase());
    let answer: String = letters.by_ref().take(8).collect();
    if answer.len() != 8 {
        return Err("emission has fewer than eight lowercase letters".into());
    }
    if letters.next().is_some() {
        return Err("emission has more than eight lowercase letters".into());
    }
    if text
        .chars()
        .any(|c| !(c.is_ascii_lowercase() || c.is_ascii_whitespace()))
    {
        return Err("emission contains punctuation or a non-ASCII character".into());
    }
    Ok(answer)
}
