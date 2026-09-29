//! DRAFT / NOT_RUN: bounded Track B source-token preparation.
//!
//! This file has not been built, tested, or used on a corpus. Intended destination:
//! `uor-r4-training::track_b::source_data`. It needs a direct `blake3` dependency
//! (already in the workspace lockfile), in addition to existing serde/sha2/hex.
//! No model weights are opened. Research choices (population, window, limits) are
//! caller supplied and belong in the prospective work card, not library defaults.
//! Storage limits below count logical output-file bytes; the runner must also
//! enforce the shared physical-volume reserve, wall time, and RSS ceiling.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::mmap_corpus::{CorpusError, CorpusWriter};
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

pub const SOURCE_VOCAB: u32 = 49_152;
pub const DOCUMENT_EOS: u16 = 2;
pub const TOKENIZER_SHA256: &str =
    "9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c";
// Covers claim's argv sentinel and the small, fixed-file-set seal manifest.
const FRAMEWORK_RESERVE: u64 = 128 * 1024;

#[derive(Debug)]
pub enum PreparationError {
    Io(io::Error),
    Json(serde_json::Error),
    Corpus(CorpusError),
    Invalid(String),
    Limit(String),
}
impl std::fmt::Display for PreparationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "source preparation I/O: {e}"),
            Self::Json(e) => write!(f, "source preparation JSON: {e}"),
            Self::Corpus(e) => write!(f, "source preparation corpus: {e}"),
            Self::Invalid(e) => write!(f, "invalid source preparation: {e}"),
            Self::Limit(e) => write!(f, "source preparation limit: {e}"),
        }
    }
}
impl std::error::Error for PreparationError {}
impl From<io::Error> for PreparationError {
    fn from(e: io::Error) -> Self { Self::Io(e) }
}
impl From<serde_json::Error> for PreparationError {
    fn from(e: serde_json::Error) -> Self { Self::Json(e) }
}
impl From<CorpusError> for PreparationError {
    fn from(e: CorpusError) -> Self { Self::Corpus(e) }
}
pub type Result<T> = std::result::Result<T, PreparationError>;
fn invalid(message: impl Into<String>) -> PreparationError {
    PreparationError::Invalid(message.into())
}
fn limit(message: impl Into<String>) -> PreparationError {
    PreparationError::Limit(message.into())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Split { Construction, Development, Confirmation }
impl Split {
    fn index(self) -> usize {
        match self { Self::Construction => 0, Self::Development => 1, Self::Confirmation => 2 }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Construction => "construction",
            Self::Development => "development",
            Self::Confirmation => "confirmation",
        }
    }
}
const SPLITS: [Split; 3] = [Split::Construction, Split::Development, Split::Confirmation];

/// D3 article-ID split, subdividing its historically exposed held-out partition.
fn document_split(id: &str) -> Split {
    let digest = blake3::hash(id.as_bytes());
    if digest.as_bytes()[0] % 5 != 0 { Split::Construction }
    else if digest.as_bytes()[1] % 2 == 0 { Split::Development }
    else { Split::Confirmation }
}

#[derive(Debug, Serialize)]
pub struct PreparationLimits {
    pub max_source_bytes: u64,
    pub max_source_documents: usize,
    pub max_jsonl_line_bytes: usize,
    pub max_tokenizer_bytes: u64,
    pub max_config_bytes: u64,
    pub max_encoded_document_tokens: usize,
    pub max_output_tokens: u64,
    pub max_output_file_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct SourceTokenSpec {
    pub source_jsonl: PathBuf,
    /// Lowercase raw BLAKE3 hex, without `blake3:`. Must be fixed before execution.
    pub expected_source_blake3: String,
    pub teacher_dir: PathBuf,
    pub source_commit: String,
    pub executable_sha256: String,
    pub documents_per_split: [usize; 3],
    pub input_tokens_per_window: usize,
    pub max_windows_per_document: usize,
    pub limits: PreparationLimits,
}

#[derive(Deserialize)]
struct Article { id: String, text: String }

#[derive(Debug, Serialize)]
struct SourceIdentity { bytes: u64, documents: usize, sha256: String, blake3: String }

#[derive(Debug, Serialize)]
struct DocumentIdentity {
    source_row: usize,
    id: String,
    split: Split,
    text_sha256: String,
    text_utf8_bytes: usize,
}
struct DuplicateGroup { first_row: usize, split_mask: u8 }
#[derive(Default)]
struct DocumentIndex {
    rows: Vec<DocumentIdentity>,
    ids: BTreeSet<String>,
    by_text: BTreeMap<String, DuplicateGroup>,
}
impl DocumentIndex {
    fn add(&mut self, row: usize, article: &Article) -> Result<()> {
        if article.id.is_empty() || !self.ids.insert(article.id.clone()) {
            return Err(invalid("article IDs must be nonempty and unique"));
        }
        let split = document_split(&article.id);
        let digest = hex::encode(Sha256::digest(article.text.as_bytes()));
        let group = self.by_text.entry(digest.clone()).or_insert(DuplicateGroup {
            first_row: row, split_mask: 0,
        });
        group.split_mask |= 1 << split.index();
        self.rows.push(DocumentIdentity {
            source_row: row, id: article.id.clone(), split, text_sha256: digest,
            text_utf8_bytes: article.text.len(),
        });
        Ok(())
    }
    fn disposition(&self, row: &DocumentIdentity) -> Result<&'static str> {
        let group = self.by_text.get(&row.text_sha256)
            .ok_or_else(|| invalid("missing text group"))?;
        if group.split_mask.count_ones() != 1 { Ok("excluded_cross_split_duplicate") }
        else if group.first_row != row.source_row { Ok("excluded_same_split_duplicate") }
        else { Ok("candidate") }
    }
}

/// Returns at most `maximum` adjacent input windows, each with one extra target
/// token, entirely inside one document. Adjacent windows share the boundary token
/// as target/input, but never duplicate a scored target position.
fn document_windows(tokens: usize, input: usize, maximum: usize) -> Result<Vec<(usize, usize)>> {
    if input == 0 || maximum == 0 { return Err(invalid("window sizes must be nonzero")); }
    let count = tokens.saturating_sub(1).checked_div(input)
        .ok_or_else(|| invalid("zero window width"))?.min(maximum);
    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        let start = i.checked_mul(input).ok_or_else(|| limit("window offset overflow"))?;
        let end = start.checked_add(input).and_then(|n| n.checked_add(1))
            .ok_or_else(|| limit("window end overflow"))?;
        if end > tokens { return Err(invalid("window crosses document boundary")); }
        result.push((start, end));
    }
    Ok(result)
}

/// Bounded line reader: unlike read_until, a maliciously long line cannot grow
/// the allocation past the declared cap. It includes the newline in file hashes.
fn bounded_line(reader: &mut impl BufRead, max: usize) -> Result<Vec<u8>> {
    let mut line = Vec::with_capacity(max.min(64 * 1024));
    loop {
        let buffer = reader.fill_buf()?;
        if buffer.is_empty() { return Ok(line); }
        let count = buffer.iter().position(|&b| b == b'\n').map_or(buffer.len(), |n| n + 1);
        if count > max.saturating_sub(line.len()) { return Err(limit("JSONL line byte cap")); }
        let complete = buffer[count - 1] == b'\n';
        line.extend_from_slice(&buffer[..count]);
        reader.consume(count);
        if complete { return Ok(line); }
    }
}

fn scan_source(
    path: &Path, limits: &PreparationLimits,
    mut visit: impl FnMut(usize, &Article) -> Result<()>,
) -> Result<SourceIdentity> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > limits.max_source_bytes {
        return Err(limit("source must be a regular file within source byte cap"));
    }
    // One extra byte is the oversize sentinel if the file grows after metadata.
    let read_cap = limits.max_source_bytes.checked_add(1)
        .ok_or_else(|| limit("source read cap overflow"))?;
    let mut reader = BufReader::new(file.take(read_cap));
    let mut sha = Sha256::new();
    let mut b3 = blake3::Hasher::new();
    let mut bytes = 0u64;
    let mut documents = 0usize;
    loop {
        let line = bounded_line(&mut reader, limits.max_jsonl_line_bytes)?;
        if line.is_empty() { break; }
        bytes = bytes.checked_add(line.len() as u64).ok_or_else(|| limit("source byte overflow"))?;
        if bytes > limits.max_source_bytes || documents >= limits.max_source_documents {
            return Err(limit("source byte/document cap"));
        }
        sha.update(&line);
        b3.update(&line);
        let article: Article = serde_json::from_slice(&line)?;
        visit(documents, &article)?;
        documents += 1;
    }
    Ok(SourceIdentity {
        bytes, documents, sha256: hex::encode(sha.finalize()), blake3: b3.finalize().to_hex().to_string(),
    })
}

struct OutputBudget { used: u64, cap: u64 }
impl OutputBudget {
    fn reserve(&mut self, bytes: u64) -> Result<()> {
        let next = self.used.checked_add(bytes).ok_or_else(|| limit("output byte overflow"))?;
        if next > self.cap { return Err(limit("output-file byte cap")); }
        self.used = next;
        Ok(())
    }
    fn json_line(&mut self, out: &mut File, value: &impl Serialize) -> Result<()> {
        let mut bytes = serde_json::to_vec(value)?;
        bytes.push(b'\n');
        self.reserve(bytes.len() as u64)?;
        out.write_all(&bytes)?;
        Ok(())
    }
}

fn read_bounded(path: &Path, cap: u64) -> Result<Vec<u8>> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > cap { return Err(limit("source metadata file cap")); }
    let mut bytes = Vec::new();
    file.take(cap.checked_add(1).ok_or_else(|| limit("read cap overflow"))?).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > cap { return Err(limit("source metadata file grew beyond cap")); }
    Ok(bytes)
}

#[derive(Debug, Serialize)]
pub struct PreparedSourceTokens {
    pub root: PathBuf,
    pub documents_per_split: [usize; 3],
    pub tokens_per_split: [u64; 3],
    pub windows_per_split: [usize; 3],
}

/// Public entry point. Existing roots fail before source/tokenizer reads. Any
/// error retains a distinct unsealed attempt; it must not be consumed as ready.
pub fn prepare_source_tokens(spec: &SourceTokenSpec, fresh_root: &Path) -> Result<PreparedSourceTokens> {
    validate_spec(spec, fresh_root)?;
    report_output::claim(fresh_root)?;
    let tokenizer_bytes = read_bounded(&spec.teacher_dir.join("tokenizer.json"), spec.limits.max_tokenizer_bytes)?;
    if hex::encode(Sha256::digest(&tokenizer_bytes)) != TOKENIZER_SHA256 {
        return Err(invalid("source tokenizer SHA-256 differs from pinned SmolLM2 tokenizer"));
    }
    let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| invalid("pinned source tokenizer did not parse"))?;
    if tokenizer.vocab_size() != SOURCE_VOCAB as usize { return Err(invalid("source tokenizer vocabulary differs")); }
    let config_bytes = read_bounded(&spec.teacher_dir.join("config.json"), spec.limits.max_config_bytes)?;
    let config: serde_json::Value = serde_json::from_slice(&config_bytes)?;
    if config["vocab_size"].as_u64() != Some(SOURCE_VOCAB as u64)
        || config["eos_token_id"].as_u64() != Some(DOCUMENT_EOS as u64) {
        return Err(invalid("teacher config vocabulary/EOS differs"));
    }
    prepare_with_encoder(spec, fresh_root, tokenizer.address(), hex::encode(Sha256::digest(&config_bytes)), |s| tokenizer.encode(s))
}

fn validate_spec(spec: &SourceTokenSpec, root: &Path) -> Result<()> {
    let l = &spec.limits;
    if spec.input_tokens_per_window == 0 || spec.max_windows_per_document == 0
        || l.max_source_bytes == 0 || l.max_source_documents == 0 || l.max_jsonl_line_bytes == 0
        || l.max_tokenizer_bytes == 0 || l.max_config_bytes == 0 || l.max_output_tokens == 0
        || l.max_encoded_document_tokens == 0 || l.max_output_file_bytes <= FRAMEWORK_RESERVE + 192
        || spec.documents_per_split.iter().all(|&n| n == 0) {
        return Err(invalid("zero or insufficient preparation limits/population"));
    }
    for digest in [&spec.expected_source_blake3, &spec.executable_sha256] {
        if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
            return Err(invalid("expected lowercase 64-character digest"));
        }
    }
    if spec.source_commit.is_empty() { return Err(invalid("missing source commit")); }
    validate_claim_metadata(root, std::env::args_os())
}

fn validate_claim_metadata(
    root: &Path, arguments: impl IntoIterator<Item = std::ffi::OsString>,
) -> Result<()> {
    // JSON escaping takes at most six bytes per input byte. Each pretty-printed
    // argv item also needs four indent spaces, two quotes, a comma and a newline
    // (eight bytes even for an empty argument). Leave half the reserve for seal.
    let mut metadata_bound = root.to_string_lossy().len().checked_mul(6)
        .and_then(|n| n.checked_add(8192))
        .ok_or_else(|| limit("framework metadata overflow"))?;
    if metadata_bound as u64 > FRAMEWORK_RESERVE / 2 { return Err(limit("framework metadata reserve")); }
    for argument in arguments {
        // claim() uses env::args(), which panics on non-UTF-8. Reject it here,
        // before creating an attempt, without exposing the argument's contents.
        let argument = argument.into_string()
            .map_err(|_| invalid("report claim arguments must be UTF-8"))?;
        let argument_bound = argument.len().checked_mul(6).and_then(|n| n.checked_add(8))
            .ok_or_else(|| limit("argv metadata overflow"))?;
        metadata_bound = metadata_bound.checked_add(argument_bound)
            .ok_or_else(|| limit("framework metadata overflow"))?;
        if metadata_bound as u64 > FRAMEWORK_RESERVE / 2 { return Err(limit("framework metadata reserve")); }
    }
    Ok(())
}

/// Encoder injection is private and exists only for tiny fixtures. Production
/// reaches it after exact tokenizer/config checks and exclusive root creation.
fn prepare_with_encoder(
    spec: &SourceTokenSpec, root: &Path, tokenizer_cid: String, config_sha256: String,
    encode: impl Fn(&str) -> Vec<u32>,
) -> Result<PreparedSourceTokens> {
    let mut index = DocumentIndex::default();
    let source = scan_source(&spec.source_jsonl, &spec.limits, |row, a| index.add(row, a))?;
    if source.blake3 != spec.expected_source_blake3 { return Err(invalid("source corpus hash differs")); }
    let mut budget = OutputBudget { used: FRAMEWORK_RESERVE, cap: spec.limits.max_output_file_bytes };
    let mut source_index = File::create_new(root.join("source-index.jsonl"))?;
    for row in &index.rows {
        budget.json_line(&mut source_index, &serde_json::json!({
            "document": row, "dedup_disposition": index.disposition(row)?,
        }))?;
    }
    source_index.flush()?;
    let mut documents_file = File::create_new(root.join("documents.jsonl"))?;
    let mut windows_file = File::create_new(root.join("windows.jsonl"))?;
    let mut writers = Vec::with_capacity(3);
    for split in SPLITS {
        budget.reserve(64)?;
        writers.push(CorpusWriter::create(root.join(format!("{}.uort", split.name())), SOURCE_VOCAB)?);
    }
    let mut result = PreparedSourceTokens {
        root: root.to_path_buf(), documents_per_split: [0; 3], tokens_per_split: [0; 3], windows_per_split: [0; 3],
    };
    let mut total_tokens = 0u64;
    let repeated_source = scan_source(&spec.source_jsonl, &spec.limits, |row_number, article| {
        let row = index.rows.get(row_number).ok_or_else(|| invalid("source grew between passes"))?;
        if row.id != article.id || row.text_sha256 != hex::encode(Sha256::digest(article.text.as_bytes())) {
            return Err(invalid("source document changed between passes"));
        }
        let s = row.split.index();
        if index.disposition(row)? != "candidate" || result.documents_per_split[s] == spec.documents_per_split[s] {
            return Ok(());
        }
        let tokens = encode(&article.text);
        let n = tokens.len().checked_add(1).ok_or_else(|| limit("document token count overflow"))?;
        if n > spec.limits.max_encoded_document_tokens { return Err(limit("encoded document token cap")); }
        let mut tokens = tokens.into_iter().map(|t| {
            if t >= SOURCE_VOCAB { Err(invalid("encoder produced token outside source vocabulary")) }
            else { u16::try_from(t).map_err(|_| invalid("source token does not fit u16")) }
        }).collect::<Result<Vec<_>>>()?;
        tokens.push(DOCUMENT_EOS);
        let windows = document_windows(tokens.len(), spec.input_tokens_per_window, spec.max_windows_per_document)?;
        if windows.is_empty() { return Ok(()); }
        let next = total_tokens.checked_add(tokens.len() as u64).ok_or_else(|| limit("total token count overflow"))?;
        if next > spec.limits.max_output_tokens { return Err(limit("output token cap")); }
        let bytes = (tokens.len() as u64).checked_mul(2).ok_or_else(|| limit("token byte overflow"))?;
        budget.reserve(bytes)?;
        let offset = result.tokens_per_split[s];
        writers[s].write_tokens(&tokens)?;
        let mut digest = Sha256::new();
        for token in &tokens { digest.update(token.to_le_bytes()); }
        budget.json_line(&mut documents_file, &serde_json::json!({
            "document": row, "token_file": format!("{}.uort", row.split.name()),
            "token_offset": offset, "token_count": tokens.len(), "eos_appended": 1,
            "token_sha256_le_u16": hex::encode(digest.finalize()),
        }))?;
        for (start, end) in windows.iter().copied() {
            let mut digest = Sha256::new();
            for token in &tokens[start..end] { digest.update(token.to_le_bytes()); }
            budget.json_line(&mut windows_file, &serde_json::json!({
                "document_id": row.id, "source_row": row.source_row, "split": row.split,
                "document_token_start": start, "token_offset": offset + start as u64,
                "input_tokens": spec.input_tokens_per_window, "target_tokens": spec.input_tokens_per_window,
                "retained_tokens": end - start, "token_sha256_le_u16": hex::encode(digest.finalize()),
                "context_reset": true, "position_start": 0,
            }))?;
        }
        total_tokens = next;
        result.tokens_per_split[s] += tokens.len() as u64;
        result.documents_per_split[s] += 1;
        result.windows_per_split[s] += windows.len();
        Ok(())
    })?;
    if repeated_source.sha256 != source.sha256 || repeated_source.blake3 != source.blake3
        || repeated_source.bytes != source.bytes || repeated_source.documents != source.documents {
        return Err(invalid("complete source changed between passes"));
    }
    if result.documents_per_split != spec.documents_per_split {
        return Err(invalid(format!("insufficient eligible documents: requested {:?}, prepared {:?}", spec.documents_per_split, result.documents_per_split)));
    }
    for (s, writer) in writers.into_iter().enumerate() {
        if writer.finish()? != result.tokens_per_split[s] { return Err(invalid("writer token count differs")); }
    }
    documents_file.flush()?;
    windows_file.flush()?;
    let mut metadata = File::create_new(root.join("prepared.json"))?;
    budget.json_line(&mut metadata, &serde_json::json!({
        "schema": "uor-r4.track-b-source-tokens/1", "format": "UORT/1",
        "spec": spec, "source": source, "result": result,
        "tokenizer_sha256": TOKENIZER_SHA256, "tokenizer_cid": tokenizer_cid,
        "config_sha256": config_sha256, "vocabulary": SOURCE_VOCAB,
        "driver_binding": "source_commit and executable_sha256 are caller supplied; the runner must bind them to its actual build",
        "split_policy": "BLAKE3(id UTF-8)[0]%5 != 0 construction; else second-byte parity: even development, odd confirmation; decided before encoding",
        "text_policy": "exact parsed JSON text UTF-8; no trimming, normalization, BOS, or chat template; append EOS 2 once per document",
        "duplicates": "SHA-256 exact-text groups crossing split classes all excluded; otherwise retain first source row",
        "windows": "first eligible documents in source order; adjacent within-document input windows; one shifted-label lookahead token; reset context per window",
        "exposure": "D3 held-out partition was historically evaluated; confirmation is reserved within this B2 cycle, not globally fresh",
        "temporary_files": 0,
    }))?;
    metadata.flush()?;
    report_output::seal(root)?;
    report_output::verify(root)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture_id(split: Split) -> String {
        (0..10_000).map(|n| format!("fixture-{n}"))
            .find(|id| document_split(id) == split).expect("small split fixture exists")
    }

    #[test]
    fn later_cross_split_duplicate_excludes_earlier_document_before_encoding() {
        let mut index = DocumentIndex::default();
        for (row, split) in [Split::Construction, Split::Development].into_iter().enumerate() {
            index.add(row, &Article { id: fixture_id(split), text: "same exact text".into() }).unwrap();
        }
        assert!(index.rows.iter().all(|r| index.disposition(r).unwrap() == "excluded_cross_split_duplicate"));
    }

    #[test]
    fn duplicate_representative_is_first_source_row_and_whitespace_is_preserved() {
        let first = fixture_id(Split::Construction);
        let second = (10_000..20_000).map(|n| format!("fixture-{n}"))
            .find(|id| document_split(id) == Split::Construction).unwrap();
        let mut index = DocumentIndex::default();
        index.add(0, &Article { id: first, text: "text".into() }).unwrap();
        index.add(1, &Article { id: second, text: "text".into() }).unwrap();
        index.add(2, &Article { id: fixture_id(Split::Development), text: " text".into() }).unwrap();
        assert_eq!(index.disposition(&index.rows[0]).unwrap(), "candidate");
        assert_eq!(index.disposition(&index.rows[1]).unwrap(), "excluded_same_split_duplicate");
        assert_eq!(index.disposition(&index.rows[2]).unwrap(), "candidate");
    }

    #[test]
    fn windows_require_lookahead_and_never_cross_a_document() {
        assert!(document_windows(4, 4, 2).unwrap().is_empty());
        assert_eq!(document_windows(5, 4, 2).unwrap(), vec![(0, 5)]);
        assert_eq!(document_windows(9, 4, 2).unwrap(), vec![(0, 5), (4, 9)]);
        assert_eq!(document_windows(10, 4, 1).unwrap(), vec![(0, 5)]);
        assert!(document_windows(10, 0, 1).is_err());
    }

    #[test]
    fn line_cap_is_enforced_before_unbounded_allocation() {
        let mut input = io::Cursor::new(b"123456789\n");
        assert!(matches!(bounded_line(&mut input, 4), Err(PreparationError::Limit(_))));
        let mut input = io::Cursor::new(b"abcd\nnext");
        assert_eq!(bounded_line(&mut input, 5).unwrap(), b"abcd\n");
        assert_eq!(bounded_line(&mut input, 5).unwrap(), b"next");
    }

    #[test]
    fn claim_metadata_counts_each_empty_or_short_argument() {
        assert!(validate_claim_metadata(Path::new("attempt"), ["program".into(), "arg".into()]).is_ok());
        for argument in ["", "x"] {
            let arguments = std::iter::repeat(std::ffi::OsString::from(argument)).take(10_000);
            assert!(matches!(validate_claim_metadata(Path::new("attempt"), arguments), Err(PreparationError::Limit(_))));
        }
    }

    #[cfg(unix)]
    #[test]
    fn claim_metadata_rejects_non_utf8_arguments() {
        use std::os::unix::ffi::OsStringExt;
        let arguments = [std::ffi::OsString::from_vec(vec![0xff])];
        assert!(matches!(validate_claim_metadata(Path::new("attempt"), arguments), Err(PreparationError::Invalid(_))));
    }

    #[test]
    fn fixture_encoder_writes_uort_eos_and_a_document_bounded_window() {
        // This test deliberately bypasses production tokenizer identity, using
        // only a three-token fake document. It performs no model computation.
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_nanos();
        let container = std::env::temp_dir().join(format!("uor-track-b-source-test-{}-{nonce}", std::process::id()));
        fs::create_dir(&container).unwrap();
        let source_path = container.join("source.jsonl");
        let raw = serde_json::to_vec(&serde_json::json!({
            "id": fixture_id(Split::Construction), "text": "abc",
        })).unwrap();
        fs::write(&source_path, &raw).unwrap();
        let spec = SourceTokenSpec {
            source_jsonl: source_path, expected_source_blake3: blake3::hash(&raw).to_hex().to_string(),
            teacher_dir: PathBuf::new(), source_commit: "fixture".into(), executable_sha256: "0".repeat(64),
            documents_per_split: [1, 0, 0], input_tokens_per_window: 3, max_windows_per_document: 1,
            limits: PreparationLimits {
                max_source_bytes: 4096, max_source_documents: 1, max_jsonl_line_bytes: 4096,
                max_tokenizer_bytes: 4096, max_config_bytes: 4096, max_encoded_document_tokens: 4,
                max_output_tokens: 4, max_output_file_bytes: 1024 * 1024,
            },
        };
        let root = container.join("attempt");
        report_output::claim(&root).unwrap();
        let result = prepare_with_encoder(&spec, &root, "fixture".into(), "fixture".into(), |_| vec![10, 11, 12]).unwrap();
        assert_eq!(result.tokens_per_split, [4, 0, 0]);
        assert_eq!(result.windows_per_split, [1, 0, 0]);
        let bytes = fs::read(root.join("construction.uort")).unwrap();
        let header = uor_r4_core::native_geometric::mmap_corpus::CorpusHeader::from_bytes(&bytes).unwrap();
        assert_eq!(header.vocab_size, SOURCE_VOCAB);
        assert_eq!(header.total_tokens, 4);
        assert_eq!(&bytes[64..], &[10, 0, 11, 0, 12, 0, 2, 0]);
        let window: serde_json::Value = serde_json::from_slice(&fs::read(root.join("windows.jsonl")).unwrap()).unwrap();
        assert_eq!(window["token_offset"], 0);
        assert_eq!(window["retained_tokens"], 4);
        report_output::verify(&root).unwrap();
        fs::remove_dir_all(container).unwrap();
    }
}
