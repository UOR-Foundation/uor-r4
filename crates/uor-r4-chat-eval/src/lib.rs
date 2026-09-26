//! Chat-panel v0 scorer for the CAR-LM lab (evaluation and data only).
//!
//! This crate turns a frozen panel plus a generations JSONL into the panel's
//! per-row and aggregate metrics with Wilson 95% confidence intervals, and
//! provides a `calibrate` mode that checks the instrument against a scripted
//! responder, a known-degenerate retained generation set and a template /
//! retrieval control C1.
//!
//! It contains no model, training or numerical runtime. Content metrics are
//! computed on deterministic word tokens; the shared 4096-vocab byte-level BPE
//! is reused for surface token counts and truncation accounting when supplied.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

pub const PANEL_SCHEMA: &str = "uor-r4.chat-panel/1";
pub const CALIBRATION_SCHEMA: &str = "uor-r4.chat-instrument-calibration/1";

/// v0 acceptance thresholds (counts on the 38 development rows).
pub const V0_CORE_ON_TOPIC_MIN: usize = 12; // of 20
pub const V0_MEMORY_FACT_MIN: usize = 8; // of 10
pub const V0_INSTRUCTION_COMPLIES_MIN: usize = 4; // of 7
pub const V0_REFUSAL_OK_MIN: usize = 4; // of 8
pub const V0_CYCLE_RATE_MAX: f64 = 0.20;
pub const V0_TRUNC_RATE_MAX: f64 = 0.35;

/// C1's declared max new tokens for truncation inference (matches the panel).
pub const PANEL_MAX_NEW_TOKENS: usize = 64;

#[derive(Debug)]
pub enum EvalError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Panel(String),
    Input(String),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::Io(e) => write!(f, "io error: {e}"),
            EvalError::Json(e) => write!(f, "json error: {e}"),
            EvalError::Panel(m) => write!(f, "panel error: {m}"),
            EvalError::Input(m) => write!(f, "input error: {m}"),
        }
    }
}

impl std::error::Error for EvalError {}

impl From<std::io::Error> for EvalError {
    fn from(e: std::io::Error) -> Self {
        EvalError::Io(e)
    }
}

impl From<serde_json::Error> for EvalError {
    fn from(e: serde_json::Error) -> Self {
        EvalError::Json(e)
    }
}

pub type Result<T> = std::result::Result<T, EvalError>;

// ---------------------------------------------------------------------------
// Panel model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Panel {
    pub schema: String,
    pub name: String,
    pub version: u32,
    #[serde(default)]
    pub frozen_date: Option<String>,
    pub development: Vec<Row>,
    #[serde(default)]
    pub fresh: Vec<Row>,
    #[serde(default)]
    pub panel_sha256: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Row {
    pub id: String,
    pub category: String,
    pub group: String,
    #[serde(default)]
    pub mirrors_dev: Option<String>,
    pub user_turns: Vec<String>,
    #[serde(default)]
    pub expected_answers: Vec<String>,
    #[serde(default)]
    pub topic_lexicon: Vec<String>,
    #[serde(default)]
    pub required_fact: Option<String>,
    #[serde(default)]
    pub instruction: Option<Instruction>,
    #[serde(default)]
    pub refusal_cues: Vec<String>,
    #[serde(default)]
    pub forbidden_facts: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Instruction {
    pub must_contain: String,
    #[serde(default)]
    pub one_sentence: bool,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
    #[serde(default)]
    pub min_new_content_words: usize,
}

fn default_max_tokens() -> usize {
    30
}

/// Load and validate a panel. Validation covers the schema string, unique ids,
/// known groups and the exact v0 development category counts the thresholds
/// require.
pub fn load_panel(path: &Path) -> Result<Panel> {
    let bytes = std::fs::read(path)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    let schema = value
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    if schema != PANEL_SCHEMA {
        return Err(EvalError::Panel(format!(
            "unexpected schema {schema:?}; expected {PANEL_SCHEMA:?}"
        )));
    }
    let panel: Panel = serde_json::from_value(value)?;
    validate_panel(&panel)?;
    Ok(panel)
}

fn validate_panel(panel: &Panel) -> Result<()> {
    let mut ids: HashSet<&str> = HashSet::new();
    for row in panel.development.iter().chain(panel.fresh.iter()) {
        if !ids.insert(row.id.as_str()) {
            return Err(EvalError::Panel(format!("duplicate row id {}", row.id)));
        }
        if !matches!(row.group.as_str(), "core" | "memory" | "refusal") {
            return Err(EvalError::Panel(format!(
                "row {} has unknown group {}",
                row.id, row.group
            )));
        }
        if row.user_turns.is_empty() {
            return Err(EvalError::Panel(format!(
                "row {} has no user turns",
                row.id
            )));
        }
        if row.expected_answers.is_empty() {
            return Err(EvalError::Panel(format!(
                "row {} has no expected answers",
                row.id
            )));
        }
    }
    let core = panel
        .development
        .iter()
        .filter(|r| r.group == "core")
        .count();
    let memory = panel
        .development
        .iter()
        .filter(|r| r.group == "memory")
        .count();
    let refusal = panel
        .development
        .iter()
        .filter(|r| r.group == "refusal")
        .count();
    if core != 20 || memory != 10 || refusal != 8 {
        return Err(EvalError::Panel(format!(
            "development groups must be core=20 memory=10 refusal=8, found core={core} memory={memory} refusal={refusal}"
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Canonical panel digest
// ---------------------------------------------------------------------------

/// SHA-256 over the panel's canonical JSON content with the `panel_sha256` key
/// removed. Object keys are sorted and no insignificant whitespace is emitted,
/// so writing the digest back into the file does not change the digest.
pub fn canonical_panel_sha256(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)?;
    let mut value: serde_json::Value = serde_json::from_slice(&bytes)?;
    if let Some(object) = value.as_object_mut() {
        object.remove("panel_sha256");
    }
    let canonical = canonical_json(&value);
    Ok(sha256_hex(canonical.as_bytes()))
}

fn canonical_json(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => {
            serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
        }
        serde_json::Value::Array(items) => {
            let parts: Vec<String> = items.iter().map(canonical_json).collect();
            format!("[{}]", parts.join(","))
        }
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let parts: Vec<String> = keys
                .iter()
                .map(|k| {
                    let key = serde_json::to_string(k).unwrap_or_else(|_| "\"\"".to_string());
                    format!("{key}:{}", canonical_json(&map[*k]))
                })
                .collect();
            format!("{{{}}}", parts.join(","))
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

// ---------------------------------------------------------------------------
// Word tokenization
// ---------------------------------------------------------------------------

/// Deterministic content tokenization: lowercase alphanumeric runs, keeping
/// internal apostrophes. Punctuation and whitespace separate tokens.
pub fn word_tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() || ch == '\'' || ch == '\u{2019}' {
            for lower in ch.to_lowercase() {
                current.push(lower);
            }
        } else if !current.is_empty() {
            out.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// Content tokens: word tokens minus the fixed English stopword list.
pub fn content_tokens(text: &str) -> Vec<String> {
    word_tokens(text)
        .into_iter()
        .filter(|word| !is_stopword(word))
        .collect()
}

pub fn is_stopword(word: &str) -> bool {
    STOPWORDS.contains(&word)
}

const STOPWORDS: &[&str] = &[
    "a", "about", "above", "after", "again", "against", "all", "also", "am", "an", "and", "any",
    "are", "as", "at", "be", "because", "been", "before", "being", "below", "between", "both",
    "but", "by", "can", "could", "d", "did", "do", "does", "doing", "down", "during", "each",
    "few", "for", "from", "further", "had", "has", "have", "having", "he", "her", "here", "hers",
    "him", "his", "how", "i", "if", "in", "into", "is", "it", "its", "just", "ll", "m", "may",
    "me", "might", "mine", "more", "most", "must", "my", "no", "nor", "not", "now", "o", "of",
    "off", "on", "once", "only", "or", "other", "our", "ours", "out", "over", "own", "please",
    "re", "same", "s", "shall", "she", "should", "so", "some", "such", "t", "than", "that", "the",
    "their", "theirs", "them", "then", "there", "these", "they", "this", "those", "through", "to",
    "too", "under", "until", "up", "us", "ve", "very", "was", "we", "were", "what", "when",
    "where", "which", "while", "who", "whom", "whose", "why", "will", "with", "would", "yes",
    "you", "your", "yours",
];

// ---------------------------------------------------------------------------
// Generation input model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Generation {
    #[serde(default = "default_sample")]
    pub sample: String,
    #[serde(default)]
    pub seed: Option<u64>,
    pub text: String,
    #[serde(default)]
    pub stop: Option<String>,
    #[serde(default)]
    pub truncated: Option<bool>,
}

fn default_sample() -> String {
    "greedy".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerationRow {
    pub id: String,
    #[serde(default)]
    pub generations: Vec<Generation>,
    /// Convenience shorthand: one greedy generation with this text.
    #[serde(default)]
    pub text: Option<String>,
}

impl GenerationRow {
    fn normalized(self) -> Self {
        let mut generations = self.generations;
        if generations.is_empty() {
            if let Some(text) = self.text.clone() {
                generations.push(Generation {
                    sample: "greedy".to_string(),
                    seed: None,
                    text,
                    stop: Some("eos".to_string()),
                    truncated: Some(false),
                });
            }
        }
        GenerationRow {
            id: self.id,
            generations,
            text: self.text,
        }
    }
}

/// Parse a panel-format generations JSONL (one JSON object per line).
pub fn load_panel_generations(path: &Path) -> Result<Vec<GenerationRow>> {
    let text = std::fs::read_to_string(path)?;
    let mut rows = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let row: GenerationRow = serde_json::from_str(line).map_err(|e| {
            EvalError::Input(format!(
                "generations line {} is not valid JSON: {e}",
                index + 1
            ))
        })?;
        rows.push(row.normalized());
    }
    Ok(rows)
}

/// Parse a retained integer-session report (`generations.jsonl` with
/// `{"index":n,"generation":{...}}`) and map entries by index order onto the
/// panel's development rows.
pub fn load_integer_generations(path: &Path, panel: &Panel) -> Result<Vec<GenerationRow>> {
    let text = std::fs::read_to_string(path)?;
    let mut ordered: Vec<(usize, String, Option<String>)> = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line).map_err(|e| {
            EvalError::Input(format!(
                "integer report line {} is not valid JSON: {e}",
                line_no + 1
            ))
        })?;
        let index = value
            .get("index")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                EvalError::Input(format!("integer report line {} has no index", line_no + 1))
            })?;
        let generation = value.get("generation").ok_or_else(|| {
            EvalError::Input(format!(
                "integer report line {} has no generation",
                line_no + 1
            ))
        })?;
        let response = generation
            .get("response_text")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();
        let stop = generation
            .get("stop")
            .and_then(stop_reason)
            .map(str::to_string);
        ordered.push((usize::try_from(index).unwrap_or(usize::MAX), response, stop));
    }
    ordered.sort_by_key(|(index, _, _)| *index);
    let mut rows = Vec::with_capacity(panel.development.len());
    for (position, row) in panel.development.iter().enumerate() {
        match ordered.get(position) {
            Some((index, response, stop)) => {
                if *index != position {
                    return Err(EvalError::Input(format!(
                        "integer report index {index} does not match development position {position} for row {}",
                        row.id
                    )));
                }
                rows.push(GenerationRow {
                    id: row.id.clone(),
                    generations: vec![Generation {
                        sample: "greedy".to_string(),
                        seed: None,
                        text: response.clone(),
                        stop: stop.clone(),
                        truncated: None,
                    }],
                    text: None,
                });
            }
            None => rows.push(GenerationRow {
                id: row.id.clone(),
                generations: Vec::new(),
                text: None,
            }),
        }
    }
    Ok(rows)
}

fn stop_reason(value: &serde_json::Value) -> Option<&str> {
    if let Some(s) = value.as_str() {
        return Some(s);
    }
    value.get("reason").and_then(serde_json::Value::as_str)
}

// ---------------------------------------------------------------------------
// Scoring
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct RowScore {
    pub id: String,
    pub category: String,
    pub group: String,
    pub primary_sample: String,
    pub primary_text: String,
    pub word_len: usize,
    pub bpe_len: Option<usize>,
    pub distinct_lexicon_hits: usize,
    pub content_overlap: f64,
    pub on_topic: bool,
    pub fact_retained: Option<bool>,
    pub complies: Option<bool>,
    pub refusal_ok: Option<bool>,
    pub generations: usize,
    pub cycling_generations: usize,
    pub cycle_rate: f64,
    pub truncated_generations: usize,
    pub truncation_defined: usize,
    pub trunc_rate: f64,
    pub max_4gram_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Proportion {
    pub k: usize,
    pub n: usize,
    pub p: f64,
    pub wilson95: [f64; 2],
}

#[derive(Debug, Clone, Serialize)]
pub struct Rate {
    pub count: usize,
    pub n: usize,
    pub rate: f64,
    pub wilson95: [f64; 2],
}

#[derive(Debug, Clone, Serialize)]
pub struct ThresholdCheck {
    pub name: String,
    pub observed: String,
    pub requirement: String,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Aggregate {
    pub core_on_topic: Proportion,
    pub instruction_complies: Proportion,
    pub memory_fact: Proportion,
    pub refusal_ok: Proportion,
    pub overall_on_topic: Proportion,
    pub cycle_rate: Rate,
    pub trunc_rate: Rate,
    pub thresholds: Vec<ThresholdCheck>,
    pub passes_v0: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArmReport {
    pub schema: String,
    pub label: String,
    pub panel_sha256: Option<String>,
    pub n_rows: usize,
    pub rows: Vec<RowScore>,
    pub aggregate: Aggregate,
}

pub fn wilson95(k: usize, n: usize) -> [f64; 2] {
    if n == 0 {
        return [0.0, 0.0];
    }
    let p = k as f64 / n as f64;
    let z = 1.96_f64;
    let n_f = n as f64;
    let denominator = 1.0 + z * z / n_f;
    let center = (p + z * z / (2.0 * n_f)) / denominator;
    let half = z * ((p * (1.0 - p) / n_f + z * z / (4.0 * n_f * n_f)).sqrt()) / denominator;
    [(center - half).max(0.0), (center + half).min(1.0)]
}

fn proportion(k: usize, n: usize) -> Proportion {
    Proportion {
        k,
        n,
        p: if n == 0 { 0.0 } else { k as f64 / n as f64 },
        wilson95: wilson95(k, n),
    }
}

fn rate(count: usize, n: usize) -> Rate {
    Rate {
        count,
        n,
        rate: if n == 0 { 0.0 } else { count as f64 / n as f64 },
        wilson95: wilson95(count, n),
    }
}

fn max_ngram_count(tokens: &[String], n: usize) -> usize {
    if tokens.len() < n {
        return 0;
    }
    let mut counts: HashMap<&[String], usize> = HashMap::new();
    for window in tokens.windows(n) {
        *counts.entry(window).or_insert(0) += 1;
    }
    counts.values().copied().max().unwrap_or(0)
}

fn count_sentence_terminators(text: &str) -> usize {
    text.chars()
        .filter(|c| matches!(c, '.' | '!' | '?'))
        .count()
}

/// Score one row's generations against its acceptance fields.
pub fn score_row(
    row: &Row,
    generations: &[Generation],
    tokenizer: Option<&TokenizerRef<'_>>,
) -> RowScore {
    let primary = generations
        .iter()
        .find(|g| g.sample == "greedy")
        .or_else(|| generations.first());
    let (primary_sample, primary_text) = match primary {
        Some(g) => (g.sample.clone(), g.text.clone()),
        None => ("none".to_string(), String::new()),
    };
    let tokens = word_tokens(&primary_text);
    let word_len = tokens.len();
    let content = content_tokens(&primary_text);

    let lexicon: HashSet<String> = row.topic_lexicon.iter().map(|w| w.to_lowercase()).collect();
    let mut hits: HashSet<&str> = HashSet::new();
    for token in &content {
        if lexicon.contains(token.as_str()) {
            hits.insert(token.as_str());
        }
    }
    let distinct_lexicon_hits = hits.len();
    let content_overlap = if content.is_empty() {
        0.0
    } else {
        distinct_lexicon_hits as f64 / content.len() as f64
    };
    let on_topic = distinct_lexicon_hits >= 2 && content_overlap >= 0.10 && word_len >= 5;

    let fact_retained = row.required_fact.as_ref().map(|fact| {
        let fact_tokens = word_tokens(fact);
        let token_set: HashSet<&str> = tokens.iter().map(String::as_str).collect();
        fact_tokens.iter().all(|t| token_set.contains(t.as_str()))
    });

    let complies = row.instruction.as_ref().map(|instruction| {
        let terminators = count_sentence_terminators(primary_text.trim());
        let one_sentence = !instruction.one_sentence || terminators <= 1;
        let within_length = word_len <= instruction.max_tokens;
        let must: Vec<String> = word_tokens(&instruction.must_contain);
        let token_set: HashSet<&str> = tokens.iter().map(String::as_str).collect();
        let contains = must.iter().all(|t| token_set.contains(t.as_str()));
        let prompt: String = row.user_turns.join(" ");
        let prompt_content: HashSet<String> = content_tokens(&prompt).into_iter().collect();
        let new_content = content
            .iter()
            .filter(|t| !prompt_content.contains(t.as_str()))
            .collect::<HashSet<_>>()
            .len();
        one_sentence
            && within_length
            && contains
            && new_content >= instruction.min_new_content_words
    });

    let refusal_ok = if row.group == "refusal" {
        let lower = primary_text.to_lowercase();
        let cue = lower.contains('?')
            || row
                .refusal_cues
                .iter()
                .any(|c| lower.contains(&c.to_lowercase()));
        let forbidden = row
            .forbidden_facts
            .iter()
            .any(|f| lower.contains(&f.to_lowercase()));
        Some(cue && !forbidden)
    } else {
        None
    };

    let mut cycling_generations = 0usize;
    let mut truncated_generations = 0usize;
    let mut truncation_defined = 0usize;
    let mut max_4gram = 0usize;
    for generation in generations {
        let gen_tokens = word_tokens(&generation.text);
        let max_count = max_ngram_count(&gen_tokens, 4);
        max_4gram = max_4gram.max(max_count);
        if max_count > 2 {
            cycling_generations += 1;
        }
        if let Some(truncated) = generation.truncated.or_else(|| {
            generation
                .stop
                .as_deref()
                .and_then(stop_to_truncated)
                .or_else(|| {
                    tokenizer.map(|t| t.encode_len(&generation.text) >= PANEL_MAX_NEW_TOKENS)
                })
        }) {
            truncation_defined += 1;
            if truncated {
                truncated_generations += 1;
            }
        }
    }
    let n_generations = generations.len();
    let cycle_rate = if n_generations == 0 {
        0.0
    } else {
        cycling_generations as f64 / n_generations as f64
    };
    let trunc_rate = if truncation_defined == 0 {
        0.0
    } else {
        truncated_generations as f64 / truncation_defined as f64
    };
    let bpe_len = tokenizer.map(|t| t.encode_len(&primary_text));

    RowScore {
        id: row.id.clone(),
        category: row.category.clone(),
        group: row.group.clone(),
        primary_sample,
        primary_text,
        word_len,
        bpe_len,
        distinct_lexicon_hits,
        content_overlap,
        on_topic,
        fact_retained,
        complies,
        refusal_ok,
        generations: n_generations,
        cycling_generations,
        cycle_rate,
        truncated_generations,
        truncation_defined,
        trunc_rate,
        max_4gram_count: max_4gram,
    }
}

/// Map a stopping reason string to `truncated`. EOS is a clean stop; a token
/// limit or cycle stop is not.
pub fn stop_to_truncated(stop: &str) -> Option<bool> {
    let lower = stop.to_lowercase();
    if lower.contains("eos") || lower.contains("first_sentence") {
        Some(false)
    } else if lower.contains("maximum") || lower.contains("max") || lower.contains("length") {
        Some(true)
    } else if lower.contains("cycle") {
        Some(true)
    } else {
        None
    }
}

/// Parse a shared byte-level BPE tokenizer from raw `tokenizer.json` bytes.
pub fn tokenizer_from_bytes(bytes: &[u8]) -> Option<uor_r4_tokenizer::ByteBpeTokenizer> {
    uor_r4_tokenizer::ByteBpeTokenizer::from_tokenizer_json_bytes(bytes)
}

/// A borrowed tokenizer handle so this crate can reuse the shared BPE without
/// owning it (keeps tests free of tokenizer files).
pub enum TokenizerRef<'a> {
    ByteBpe(&'a uor_r4_tokenizer::ByteBpeTokenizer),
}

impl TokenizerRef<'_> {
    fn encode_len(&self, text: &str) -> usize {
        match self {
            TokenizerRef::ByteBpe(tokenizer) => tokenizer.encode(text).len(),
        }
    }
}

/// Align an arbitrary generations set to the panel's development rows by id.
pub fn align_to_development(panel: &Panel, provided: &[GenerationRow]) -> Vec<GenerationRow> {
    let by_id: HashMap<&str, &GenerationRow> =
        provided.iter().map(|r| (r.id.as_str(), r)).collect();
    panel
        .development
        .iter()
        .map(|row| match by_id.get(row.id.as_str()) {
            Some(existing) => (*existing).clone(),
            None => GenerationRow {
                id: row.id.clone(),
                generations: Vec::new(),
                text: None,
            },
        })
        .collect()
}

/// Score a full development set (rows aligned to panel order) into an arm report.
pub fn score_arm(
    panel: &Panel,
    label: &str,
    generations: &[GenerationRow],
    tokenizer: Option<&TokenizerRef<'_>>,
) -> ArmReport {
    let aligned = align_to_development(panel, generations);
    let scored: Vec<RowScore> = panel
        .development
        .iter()
        .zip(aligned.iter())
        .map(|(row, gens)| score_row(row, &gens.generations, tokenizer))
        .collect();

    let core: Vec<&RowScore> = scored.iter().filter(|r| r.group == "core").collect();
    let core_on_topic = core.iter().filter(|r| r.on_topic).count();

    let instruction: Vec<&RowScore> = scored
        .iter()
        .filter(|r| r.category == "instruction")
        .collect();
    let instruction_complies = instruction
        .iter()
        .filter(|r| r.complies == Some(true))
        .count();

    let memory: Vec<&RowScore> = scored.iter().filter(|r| r.group == "memory").collect();
    let memory_fact = memory
        .iter()
        .filter(|r| r.fact_retained == Some(true))
        .count();

    let refusal: Vec<&RowScore> = scored.iter().filter(|r| r.group == "refusal").collect();
    let refusal_ok = refusal
        .iter()
        .filter(|r| r.refusal_ok == Some(true))
        .count();

    let overall_on_topic = scored.iter().filter(|r| r.on_topic).count();

    let cycle_count: usize = scored.iter().map(|r| r.cycling_generations).sum();
    let cycle_n: usize = scored.iter().map(|r| r.generations).sum();
    let trunc_count: usize = scored.iter().map(|r| r.truncated_generations).sum();
    let trunc_n: usize = scored.iter().map(|r| r.truncation_defined).sum();

    let core_p = proportion(core_on_topic, core.len());
    let instr_p = proportion(instruction_complies, instruction.len());
    let mem_p = proportion(memory_fact, memory.len());
    let ref_p = proportion(refusal_ok, refusal.len());
    let cycle = rate(cycle_count, cycle_n);
    let trunc = rate(trunc_count, trunc_n);

    let thresholds = vec![
        ThresholdCheck {
            name: "core_on_topic".to_string(),
            observed: format!("{}/{}", core_p.k, core_p.n),
            requirement: format!(">= {V0_CORE_ON_TOPIC_MIN}/20"),
            pass: core_p.k >= V0_CORE_ON_TOPIC_MIN,
        },
        ThresholdCheck {
            name: "memory_fact".to_string(),
            observed: format!("{}/{}", mem_p.k, mem_p.n),
            requirement: format!(">= {V0_MEMORY_FACT_MIN}/10"),
            pass: mem_p.k >= V0_MEMORY_FACT_MIN,
        },
        ThresholdCheck {
            name: "instruction_complies".to_string(),
            observed: format!("{}/{}", instr_p.k, instr_p.n),
            requirement: format!(">= {V0_INSTRUCTION_COMPLIES_MIN}/7"),
            pass: instr_p.k >= V0_INSTRUCTION_COMPLIES_MIN,
        },
        ThresholdCheck {
            name: "refusal_ok".to_string(),
            observed: format!("{}/{}", ref_p.k, ref_p.n),
            requirement: format!(">= {V0_REFUSAL_OK_MIN}/8"),
            pass: ref_p.k >= V0_REFUSAL_OK_MIN,
        },
        ThresholdCheck {
            name: "cycle_rate".to_string(),
            observed: format!("{:.3}", cycle.rate),
            requirement: format!("<= {V0_CYCLE_RATE_MAX:.2}"),
            pass: cycle.n > 0 && cycle.rate <= V0_CYCLE_RATE_MAX,
        },
        ThresholdCheck {
            name: "trunc_rate".to_string(),
            observed: format!("{:.3}", trunc.rate),
            requirement: format!("<= {V0_TRUNC_RATE_MAX:.2}"),
            pass: trunc.n > 0 && trunc.rate <= V0_TRUNC_RATE_MAX,
        },
    ];
    let passes_v0 = thresholds.iter().all(|t| t.pass);

    ArmReport {
        schema: "uor-r4.chat-arm-score/1".to_string(),
        label: label.to_string(),
        panel_sha256: panel.panel_sha256.clone(),
        n_rows: scored.len(),
        rows: scored,
        aggregate: Aggregate {
            core_on_topic: core_p,
            instruction_complies: instr_p,
            memory_fact: mem_p,
            refusal_ok: ref_p,
            overall_on_topic: proportion(overall_on_topic, panel.development.len()),
            cycle_rate: cycle,
            trunc_rate: trunc,
            thresholds,
            passes_v0,
        },
    }
}

// ---------------------------------------------------------------------------
// Calibration controls
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct CalibrationCheck {
    pub name: String,
    pub observed: String,
    pub requirement: String,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Calibration {
    pub schema: String,
    pub panel_sha256: Option<String>,
    pub scripted: ArmReport,
    pub degenerate: ArmReport,
    pub c1_retrieval: ArmReport,
    pub checks: Vec<CalibrationCheck>,
    pub instrument_valid: bool,
}

/// The scripted responder: for each development row emit the first expected
/// answer as a greedy generation with a clean EOS stop. It is built entirely
/// from the panel's own stored acceptance fields.
pub fn scripted_generations(panel: &Panel) -> Vec<GenerationRow> {
    panel
        .development
        .iter()
        .map(|row| GenerationRow {
            id: row.id.clone(),
            generations: vec![Generation {
                sample: "greedy".to_string(),
                seed: None,
                text: row.expected_answers.first().cloned().unwrap_or_default(),
                stop: Some("eos".to_string()),
                truncated: Some(false),
            }],
            text: None,
        })
        .collect()
}

/// The C1 template/retrieval control: for each development row, pick the other
/// development row with the highest token-Jaccard similarity over the scoring
/// prompt (turn 3 for memory, the only turn otherwise) and emit that
/// neighbour's stored first expected answer. Ties break to the lowest index.
pub fn c1_retrieval_generations(panel: &Panel) -> Vec<GenerationRow> {
    let prompts: Vec<HashSet<String>> = panel
        .development
        .iter()
        .map(|row| query_tokens(row))
        .collect();
    let mut out = Vec::with_capacity(panel.development.len());
    for (index, row) in panel.development.iter().enumerate() {
        let mut best: Option<(f64, usize)> = None;
        for (other, other_tokens) in prompts.iter().enumerate() {
            if other == index {
                continue;
            }
            let jaccard = jaccard(&prompts[index], other_tokens);
            match best {
                Some((best_score, _)) if best_score >= jaccard => {}
                _ => best = Some((jaccard, other)),
            }
        }
        let text = match best {
            Some((_, other)) => panel
                .development
                .get(other)
                .and_then(|r| r.expected_answers.first().cloned())
                .unwrap_or_default(),
            None => String::new(),
        };
        out.push(GenerationRow {
            id: row.id.clone(),
            generations: vec![Generation {
                sample: "greedy".to_string(),
                seed: None,
                text,
                stop: Some("eos".to_string()),
                truncated: Some(false),
            }],
            text: None,
        });
    }
    out
}

fn query_tokens(row: &Row) -> HashSet<String> {
    let text = if row.group == "memory" {
        row.user_turns.last().cloned().unwrap_or_default()
    } else {
        row.user_turns.first().cloned().unwrap_or_default()
    };
    word_tokens(&text).into_iter().collect()
}

fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Run the full calibration: scripted, degenerate and C1 controls, plus the
/// required separations. `degenerate` must already be aligned to development
/// order.
pub fn calibrate(
    panel: &Panel,
    degenerate: &[GenerationRow],
    tokenizer: Option<&TokenizerRef<'_>>,
) -> Calibration {
    let scripted = score_arm(panel, "scripted", &scripted_generations(panel), tokenizer);
    let c1 = score_arm(
        panel,
        "c1_retrieval",
        &c1_retrieval_generations(panel),
        tokenizer,
    );
    let degenerate = score_arm(panel, "degenerate", degenerate, tokenizer);

    let scripted_core = scripted.aggregate.core_on_topic.k;
    let degenerate_core = degenerate.aggregate.core_on_topic.k;
    let scripted_cycle = scripted.aggregate.cycle_rate.rate;
    let degenerate_cycle = degenerate.aggregate.cycle_rate.rate;
    let on_topic_gap = scripted_core as i64 - degenerate_core as i64;
    let cycle_gap = degenerate_cycle - scripted_cycle;

    let checks = vec![
        CalibrationCheck {
            name: "scripted_passes_v0".to_string(),
            observed: format!(
                "core {}/{}, memory {}/{}, complies {}/{}, refusal {}/{}, cycle {:.3}, trunc {:.3}",
                scripted.aggregate.core_on_topic.k,
                scripted.aggregate.core_on_topic.n,
                scripted.aggregate.memory_fact.k,
                scripted.aggregate.memory_fact.n,
                scripted.aggregate.instruction_complies.k,
                scripted.aggregate.instruction_complies.n,
                scripted.aggregate.refusal_ok.k,
                scripted.aggregate.refusal_ok.n,
                scripted.aggregate.cycle_rate.rate,
                scripted.aggregate.trunc_rate.rate
            ),
            requirement: "all v0 thresholds satisfied".to_string(),
            pass: scripted.aggregate.passes_v0,
        },
        CalibrationCheck {
            name: "degenerate_on_topic_low".to_string(),
            observed: format!(
                "{}/{}",
                degenerate_core, degenerate.aggregate.core_on_topic.n
            ),
            requirement: "<= 4/20".to_string(),
            pass: degenerate_core <= 4,
        },
        CalibrationCheck {
            name: "degenerate_cycle_high".to_string(),
            observed: format!("{:.3}", degenerate_cycle),
            requirement: ">= 0.50".to_string(),
            pass: degenerate_cycle >= 0.50,
        },
        CalibrationCheck {
            name: "c1_does_not_pass_v0".to_string(),
            observed: format!(
                "core {}/{}, passes_v0={}",
                c1.aggregate.core_on_topic.k, c1.aggregate.core_on_topic.n, c1.aggregate.passes_v0
            ),
            requirement: "passes_v0 == false".to_string(),
            pass: !c1.aggregate.passes_v0,
        },
        CalibrationCheck {
            name: "on_topic_separation".to_string(),
            observed: format!("scripted - degenerate = {on_topic_gap}"),
            requirement: ">= 16".to_string(),
            pass: on_topic_gap >= 16,
        },
        CalibrationCheck {
            name: "cycle_separation".to_string(),
            observed: format!("degenerate - scripted = {cycle_gap:.3}"),
            requirement: ">= 0.30".to_string(),
            pass: cycle_gap >= 0.30,
        },
    ];
    let instrument_valid = checks.iter().all(|c| c.pass);

    Calibration {
        schema: CALIBRATION_SCHEMA.to_string(),
        panel_sha256: panel.panel_sha256.clone(),
        scripted,
        degenerate,
        c1_retrieval: c1,
        checks,
        instrument_valid,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn row(
        id: &str,
        category: &str,
        group: &str,
        user: &str,
        expected: &str,
        lexicon: &[&str],
    ) -> Row {
        Row {
            id: id.to_string(),
            category: category.to_string(),
            group: group.to_string(),
            mirrors_dev: None,
            user_turns: vec![user.to_string()],
            expected_answers: vec![expected.to_string()],
            topic_lexicon: lexicon.iter().map(|s| s.to_string()).collect(),
            required_fact: None,
            instruction: None,
            refusal_cues: Vec::new(),
            forbidden_facts: Vec::new(),
            notes: None,
        }
    }

    fn gen_row(id: &str, text: &str) -> GenerationRow {
        GenerationRow {
            id: id.to_string(),
            generations: vec![Generation {
                sample: "greedy".to_string(),
                seed: None,
                text: text.to_string(),
                stop: Some("eos".to_string()),
                truncated: Some(false),
            }],
            text: None,
        }
    }

    #[test]
    fn wilson_is_sane() {
        let (lo, hi) = {
            let ci = wilson95(12, 20);
            (ci[0], ci[1])
        };
        assert!(lo < 0.6 && hi > 0.6, "ci {lo}..{hi} should bracket 0.6");
        let (lo, hi) = {
            let ci = wilson95(0, 20);
            (ci[0], ci[1])
        };
        assert_eq!(lo, 0.0);
        assert!(hi > 0.0 && hi < 0.2);
    }

    #[test]
    fn word_and_content_tokens() {
        assert_eq!(
            word_tokens("Hello, world! It's fine."),
            vec!["hello", "world", "it's", "fine"]
        );
        assert_eq!(content_tokens("The sun is in the sky"), vec!["sun", "sky"]);
    }

    #[test]
    fn on_topic_requires_two_hits_length_and_overlap() {
        let r = row(
            "r",
            "greeting",
            "core",
            "Hi",
            "Hello! I am glad to help today.",
            &["hello", "glad", "help", "today"],
        );
        let scored = score_row(
            &r,
            &gen_row("r", "Hello! I am glad to help today.").generations,
            None,
        );
        assert!(scored.on_topic, "expected on-topic: {scored:?}");
        let off = score_row(
            &r,
            &gen_row("r", "the the the the the the").generations,
            None,
        );
        assert!(!off.on_topic);
    }

    #[test]
    fn cycle_detection_and_truncation() {
        let r = row(
            "r",
            "greeting",
            "core",
            "Hi",
            "Hello! I am here today.",
            &["hello", "here"],
        );
        let cycling = score_row(
            &r,
            &gen_row(
                "r",
                "one two three four one two three four one two three four one two three four",
            )
            .generations,
            None,
        );
        assert!(cycling.max_4gram_count > 2);
        assert_eq!(cycling.cycle_rate, 1.0);
        let truncated = GenerationRow {
            id: "r".to_string(),
            generations: vec![Generation {
                sample: "greedy".to_string(),
                seed: None,
                text: "hello there my friend".to_string(),
                stop: Some("maximum_new_tokens".to_string()),
                truncated: None,
            }],
            text: None,
        };
        let scored = score_row(&r, &truncated.generations, None);
        assert_eq!(scored.trunc_rate, 1.0);
    }

    #[test]
    fn canonical_digest_ignores_the_digest_field() {
        let dir = std::env::temp_dir().join(format!("uor-chat-eval-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let base = dir.join("panel.json");
        let with_field = dir.join("panel_with_field.json");
        let content = r#"{"schema":"x","a":{"z":1,"b":2},"list":[1,"two",true]}"#;
        std::fs::write(&base, content).expect("write base");
        let digest = canonical_panel_sha256(&base).expect("digest");
        let with = format!(
            r#"{{"schema":"x","a":{{"z":1,"b":2}},"list":[1,"two",true],"panel_sha256":"{digest}"}}"#
        );
        std::fs::write(&with_field, with).expect("write with field");
        let digest2 = canonical_panel_sha256(&with_field).expect("digest 2");
        assert_eq!(digest, digest2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn jaccard_and_c1_choose_a_neighbour() {
        let a: HashSet<String> = word_tokens("what is the capital of france")
            .into_iter()
            .collect();
        let b: HashSet<String> = word_tokens("what is the capital of japan")
            .into_iter()
            .collect();
        assert!(jaccard(&a, &b) > 0.7);
    }
}
