//! Prepare the chat-v0 R1 dialogue corpus (R1a: data preparation only).
//!
//! Reads the permissively licensed chat/instruction JSONL on the external SSD
//! and emits, per split, a contiguous `u16`-little-endian token stream in the
//! canonical `UORT` format the native trainer loads ([`CorpusWriter`]) plus a
//! parallel `u8` response mask (1 = assistant token, 0 = prompt/separator).
//!
//! This binary **only** prepares data. It does not build, train, load or score
//! a model, and it does not change the tokenizer. It reuses the shared
//! `crates/uor-r4-tokenizer` byte-level BPE engine and the fixed 4096-entry
//! vocabulary; no new vocabulary or tokenizer change is introduced.
//!
//! # Template
//!
//! Each JSONL row's `messages` array is rendered as turns. Literal role
//! markers already representable in the fixed vocabulary are used:
//!
//! ```text
//! <|bos|>System: {s}\nUser: {u}\nAssistant: {a}<|eos|>\n... last assistant <|eos|>
//! ```
//!
//! - `System: `, `User: ` and `Assistant: ` are literal marker strings.
//! - Turns are joined by a single `\n` separator (placed *before* every turn
//!   after the first), so each role marker begins a line.
//! - Every assistant turn ends with the `<|eos|>` token.
//! - A document-terminal `<|eos|>` is appended only when the final emitted turn
//!   is not an assistant turn (for example a conversation whose last message is
//!   a user turn). The last token of every document is therefore `<|eos|>`.
//! - `messages` with an unknown role or empty/whitespace content are skipped and
//!   counted; `\r\n`/`\r` line endings are normalised to `\n` and content is
//!   trimmed. Interior whitespace is preserved.
//!
//! # Response mask
//!
//! `response_mask.u8` has exactly one byte per token:
//!
//! - `1` — every token of an assistant turn's response content **and its
//!   terminating `<|eos|>`**;
//! - `0` — `<|bos|>`, every role marker (`System: `, `User: `, `Assistant: `),
//!   turn separators, system/user content, and an unmasked document-terminal
//!   `<|eos|>`.
//!
//! The `Assistant: ` marker itself is prompt context, not response content, so
//! it is `0`; the first masked token is the first response token, whose loss is
//! conditioned on the marker.
//!
//! # Output
//!
//! ```text
//! <out-dir>/<split>/tokens.u16        # UORT header + u16 little-endian stream
//! <out-dir>/<split>/response_mask.u8  # raw u8, one byte per token
//! <out-dir>/<split>/manifest.json
//! <out-dir>/manifest.json             # both splits
//! ```
//!
//! The held-out split is written separately and is never mixed into train.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::mmap_corpus::{CorpusWriter, MmapCorpusReader};
use uor_r4_tokenizer::ByteBpeTokenizer;

const SCHEMA: &str = "uor-r4-chat-corpus/v1";
const MASK_SCHEMA: &str = "uor-r4-response-mask/u8/v1";
const DEFAULT_MAX_TOKENS: usize = 8192;
const DEFAULT_EXPECT_VOCAB: usize = 4096;

const MARKER_SYSTEM: &str = "System: ";
const MARKER_USER: &str = "User: ";
const MARKER_ASSISTANT: &str = "Assistant: ";
const SEPARATOR: &str = "\n";
const BOS_SURFACE: &str = "<|bos|>";
const EOS_SURFACE: &str = "<|eos|>";
const UNK_SURFACE: &str = "<|unk|>";

const TEMPLATE_RULE: &str = "<|bos|> then turns joined by a single '\\n' separator (placed before every turn after the first); a turn is '<marker><content>' with markers 'System: ', 'User: ', 'Assistant: '; every assistant turn ends with <|eos|>; a document-terminal <|eos|> is appended only when the final emitted turn is not assistant. Content is \\r\\n/\\r-normalised and trimmed; interior whitespace preserved.";
const MASK_RULE: &str = "1 = each token of an assistant turn's response content and its terminating <|eos|>; 0 = <|bos|>, all role markers, turn separators, system/user content, and an unmasked document-terminal <|eos|>.";

// ---------------------------------------------------------------------------
// JSONL schema (only the fields this pipeline needs; unknown fields ignored).
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct Envelope {
    row: RowPayload,
}

#[derive(Debug, Deserialize)]
struct RowPayload {
    #[serde(default)]
    messages: Option<Vec<Message>>,
}

#[derive(Debug, Clone, Deserialize)]
struct Message {
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    content: Option<String>,
}

// ---------------------------------------------------------------------------
// Per-file and per-split statistics.
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone, Serialize)]
struct FileStats {
    label: String,
    path: String,
    input_bytes: u64,
    sha256: String,
    rows_total: u64,
    rows_used: u64,
    rows_dropped_no_messages: u64,
    rows_dropped_empty: u64,
    rows_dropped_no_response: u64,
    rows_dropped_oversized: u64,
    rows_dropped_malformed: u64,
    messages_skipped_empty: u64,
    messages_skipped_unknown_role: u64,
    special_token_occurrences: u64,
    tokens: u64,
    response_tokens: u64,
}

#[derive(Debug, Default, Clone, Serialize)]
struct DropTotals {
    rows_dropped_no_messages: u64,
    rows_dropped_empty: u64,
    rows_dropped_no_response: u64,
    rows_dropped_oversized: u64,
    rows_dropped_malformed: u64,
    messages_skipped_empty: u64,
    messages_skipped_unknown_role: u64,
    special_token_occurrences: u64,
}

impl DropTotals {
    fn total_rows_dropped(&self) -> u64 {
        self.rows_dropped_no_messages
            + self.rows_dropped_empty
            + self.rows_dropped_no_response
            + self.rows_dropped_oversized
            + self.rows_dropped_malformed
    }
}

#[derive(Debug, Serialize)]
struct SplitManifest {
    schema: String,
    mask_schema: String,
    split: String,
    template_rule: String,
    mask_rule: String,
    tokenizer: TokenizerInfo,
    max_tokens: usize,
    inputs: Vec<InputInfo>,
    rows_used: u64,
    rows_total: u64,
    tokens: u64,
    response_tokens: u64,
    response_fraction: f64,
    tokens_bytes: u64,
    mask_bytes: u64,
    tokens_sha256: String,
    mask_sha256: String,
    drops: DropTotals,
    files: Vec<FileStats>,
}

#[derive(Debug, Clone, Serialize)]
struct TokenizerInfo {
    path: String,
    sha256: String,
    tokenizer_cid: String,
    vocab_size: usize,
    bos_id: u32,
    eos_id: u32,
    unk_id: u32,
}

#[derive(Debug, Clone, Serialize)]
struct InputInfo {
    label: String,
    path: String,
}

// ---------------------------------------------------------------------------
// Encoding.
// ---------------------------------------------------------------------------

struct Encoded {
    tokens: Vec<u16>,
    mask: Vec<u8>,
    emitted_turns: u64,
    response_tokens: u64,
    skipped_empty: u64,
    skipped_unknown_role: u64,
    special_token_occurrences: u64,
}

/// Normalise line endings and trim only the ends; interior whitespace is kept.
fn normalize_content(raw: &str) -> String {
    raw.replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim()
        .to_string()
}

/// Append the encoding of `text`, tagging every produced token with `mask_value`
/// and counting occurrences of the added special tokens.
fn append_text(
    tokenizer: &ByteBpeTokenizer,
    tokens: &mut Vec<u16>,
    mask: &mut Vec<u8>,
    text: &str,
    mask_value: u8,
    special_ids: (u32, u32, u32),
    special_count: &mut u64,
) {
    for id in tokenizer.encode(text) {
        if id == special_ids.0 || id == special_ids.1 || id == special_ids.2 {
            *special_count += 1;
        }
        tokens.push(id as u16);
        mask.push(mask_value);
    }
}

/// Render one conversation. The returned mask is byte-for-byte aligned with the
/// token stream; `emitted_turns == 0` means the row has no usable content.
fn encode_conversation(
    messages: &[Message],
    tokenizer: &ByteBpeTokenizer,
    bos: u32,
    eos: u32,
    unk: u32,
) -> Encoded {
    let special_ids = (bos, eos, unk);
    let mut out = Encoded {
        tokens: Vec::new(),
        mask: Vec::new(),
        emitted_turns: 0,
        response_tokens: 0,
        skipped_empty: 0,
        skipped_unknown_role: 0,
        special_token_occurrences: 0,
    };
    out.tokens.push(bos as u16);
    out.mask.push(0);

    let mut last_is_assistant = false;
    for message in messages {
        let role = message.role.as_deref().unwrap_or("");
        let content = normalize_content(message.content.as_deref().unwrap_or(""));
        if content.is_empty() {
            out.skipped_empty += 1;
            continue;
        }
        let (marker, is_assistant) = match role {
            "assistant" => (MARKER_ASSISTANT, true),
            "user" => (MARKER_USER, false),
            "system" => (MARKER_SYSTEM, false),
            _ => {
                out.skipped_unknown_role += 1;
                continue;
            }
        };

        if out.emitted_turns > 0 {
            append_text(
                tokenizer,
                &mut out.tokens,
                &mut out.mask,
                SEPARATOR,
                0,
                special_ids,
                &mut out.special_token_occurrences,
            );
        }
        append_text(
            tokenizer,
            &mut out.tokens,
            &mut out.mask,
            marker,
            0,
            special_ids,
            &mut out.special_token_occurrences,
        );
        append_text(
            tokenizer,
            &mut out.tokens,
            &mut out.mask,
            &content,
            if is_assistant { 1 } else { 0 },
            special_ids,
            &mut out.special_token_occurrences,
        );
        if is_assistant {
            out.tokens.push(eos as u16);
            out.mask.push(1);
        }

        out.emitted_turns += 1;
        last_is_assistant = is_assistant;
    }

    // Document terminator when the final emitted turn is not an assistant turn.
    if out.emitted_turns > 0 && !last_is_assistant {
        out.tokens.push(eos as u16);
        out.mask.push(0);
    }

    out.response_tokens = out.mask.iter().filter(|&&m| m == 1).count() as u64;
    out
}

// ---------------------------------------------------------------------------
// Preparation.
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn process_file(
    label: &str,
    path: &Path,
    tokenizer: &ByteBpeTokenizer,
    ids: (u32, u32, u32),
    max_tokens: usize,
    limit: Option<usize>,
    writer: &mut CorpusWriter,
    mask_writer: &mut BufWriter<File>,
) -> Result<FileStats, Box<dyn Error>> {
    let (sha256, input_bytes) = sha256_file(path)?;
    let mut stats = FileStats {
        label: label.to_string(),
        path: path.to_string_lossy().into_owned(),
        input_bytes,
        sha256,
        ..FileStats::default()
    };

    let file = File::open(path)?;
    let reader = BufReader::with_capacity(1 << 20, file);
    for (line_index, line) in reader.lines().enumerate() {
        if let Some(limit) = limit {
            if line_index >= limit {
                break;
            }
        }
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        stats.rows_total += 1;

        let envelope: Envelope = match serde_json::from_str(&line) {
            Ok(envelope) => envelope,
            Err(_) => {
                stats.rows_dropped_malformed += 1;
                continue;
            }
        };
        let Some(messages) = envelope.row.messages else {
            stats.rows_dropped_no_messages += 1;
            continue;
        };
        if messages.is_empty() {
            stats.rows_dropped_no_messages += 1;
            continue;
        }

        let encoded = encode_conversation(&messages, tokenizer, ids.0, ids.1, ids.2);
        stats.messages_skipped_empty += encoded.skipped_empty;
        stats.messages_skipped_unknown_role += encoded.skipped_unknown_role;
        stats.special_token_occurrences += encoded.special_token_occurrences;

        if encoded.emitted_turns == 0 {
            stats.rows_dropped_empty += 1;
            continue;
        }
        if encoded.response_tokens == 0 {
            stats.rows_dropped_no_response += 1;
            continue;
        }
        if encoded.tokens.len() > max_tokens {
            stats.rows_dropped_oversized += 1;
            continue;
        }

        writer.write_tokens_unchecked(&encoded.tokens)?;
        mask_writer.write_all(&encoded.mask)?;
        stats.rows_used += 1;
        stats.tokens += encoded.tokens.len() as u64;
        stats.response_tokens += encoded.response_tokens;
    }

    Ok(stats)
}

#[allow(clippy::too_many_arguments)]
fn prepare_split(
    split: &str,
    inputs: &[InputInfo],
    tokenizer: &ByteBpeTokenizer,
    ids: (u32, u32, u32),
    out_root: &Path,
    max_tokens: usize,
    limit: Option<usize>,
    tokenizer_info: &TokenizerInfo,
) -> Result<SplitManifest, Box<dyn Error>> {
    let dir = out_root.join(split);
    fs::create_dir_all(&dir)?;
    let tokens_path = dir.join("tokens.u16");
    let mask_path = dir.join("response_mask.u8");

    let mut writer = CorpusWriter::create(&tokens_path, tokenizer.vocab_size() as u32)?;
    let mut mask_writer = BufWriter::with_capacity(1 << 20, File::create(&mask_path)?);

    let mut files: Vec<FileStats> = Vec::with_capacity(inputs.len());
    for input in inputs {
        let stats = process_file(
            &input.label,
            Path::new(&input.path),
            tokenizer,
            ids,
            max_tokens,
            limit,
            &mut writer,
            &mut mask_writer,
        )?;
        files.push(stats);
    }

    let total_tokens = writer.finish()?;
    mask_writer.flush()?;
    drop(mask_writer);

    let mask_bytes = fs::metadata(&mask_path)?.len();
    if mask_bytes != total_tokens {
        return Err(format!(
            "mask/token length mismatch in split '{split}': {mask_bytes} mask bytes vs {total_tokens} tokens"
        )
        .into());
    }

    let rows_used: u64 = files.iter().map(|f| f.rows_used).sum();
    let rows_total: u64 = files.iter().map(|f| f.rows_total).sum();
    let response_tokens: u64 = files.iter().map(|f| f.response_tokens).sum();
    let mut drops = DropTotals::default();
    for file in &files {
        drops.rows_dropped_no_messages += file.rows_dropped_no_messages;
        drops.rows_dropped_empty += file.rows_dropped_empty;
        drops.rows_dropped_no_response += file.rows_dropped_no_response;
        drops.rows_dropped_oversized += file.rows_dropped_oversized;
        drops.rows_dropped_malformed += file.rows_dropped_malformed;
        drops.messages_skipped_empty += file.messages_skipped_empty;
        drops.messages_skipped_unknown_role += file.messages_skipped_unknown_role;
        drops.special_token_occurrences += file.special_token_occurrences;
    }
    let (tokens_sha256, tokens_bytes) = sha256_file(&tokens_path)?;
    let (mask_sha256, _) = sha256_file(&mask_path)?;

    let manifest = SplitManifest {
        schema: SCHEMA.to_string(),
        mask_schema: MASK_SCHEMA.to_string(),
        split: split.to_string(),
        template_rule: TEMPLATE_RULE.to_string(),
        mask_rule: MASK_RULE.to_string(),
        tokenizer: tokenizer_info.clone(),
        max_tokens,
        inputs: inputs.to_vec(),
        rows_used,
        rows_total,
        tokens: total_tokens,
        response_tokens,
        response_fraction: if total_tokens > 0 {
            response_tokens as f64 / total_tokens as f64
        } else {
            0.0
        },
        tokens_bytes,
        mask_bytes,
        tokens_sha256,
        mask_sha256,
        drops,
        files,
    };
    write_json(&dir.join("manifest.json"), &manifest)?;
    Ok(manifest)
}

// ---------------------------------------------------------------------------
// Hashing helpers.
// ---------------------------------------------------------------------------

fn sha256_file(path: &Path) -> Result<(String, u64), Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1 << 20];
    let mut total = 0u64;
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        total += read as u64;
    }
    Ok((hex::encode(hasher.finalize()), total))
}

/// Resolve a required, single-id added special token by its literal surface.
fn resolve_special(
    tokenizer: &ByteBpeTokenizer,
    surface: &str,
    role: &str,
) -> Result<u32, Box<dyn Error>> {
    let ids = tokenizer.encode(surface);
    match ids.as_slice() {
        [id] => Ok(*id),
        _ => Err(format!(
            "tokenizer has no single-token added special for {role} ('{surface}'): encoded to {ids:?}"
        )
        .into()),
    }
}

// ---------------------------------------------------------------------------
// Report writing.
// ---------------------------------------------------------------------------

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, value)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// CLI.
// ---------------------------------------------------------------------------

fn usage(program: &str) {
    eprintln!(
        "Usage:\n  {program} prepare --tokenizer <tokenizer.json> --out-dir <dir> \\\n          [--train <file.jsonl>]... [--heldout <file.jsonl>]... \\\n          [--max-tokens N] [--limit N] [--expect-vocab N] [--no-verify]\n\
         \n  {program} verify --tokenizer <tokenizer.json> --dir <prepared-split-dir>... [--preview N]\n\
         \nArguments:\n\
         --tokenizer <path>  Shared 4096-entry byte-BPE tokenizer.json (no tokenizer change is made).\n\
         --out-dir <dir>     Base directory; outputs land in <dir>/train and <dir>/heldout.\n\
         --train <path>      Training JSONL (repeatable). Never mixed with heldout.\n\
         --heldout <path>    Held-out JSONL (repeatable). Written as a separate split.\n\
         --max-tokens <n>    Drop rows whose rendered length exceeds n (default {DEFAULT_MAX_TOKENS}).\n\
         --limit <n>         Optional per-file row cap (smoke runs only).\n\
         --expect-vocab <n>  Fail unless the tokenizer has exactly this many id slots (default {DEFAULT_EXPECT_VOCAB}).\n\
         --no-verify         Skip the post-prepare self-check.\n\
         --preview <n>       verify: decode this many leading tokens (default 64).\n\
         --help              Show this help."
    );
}

struct PrepareArgs {
    tokenizer: PathBuf,
    out_dir: PathBuf,
    train: Vec<PathBuf>,
    heldout: Vec<PathBuf>,
    max_tokens: usize,
    limit: Option<usize>,
    expect_vocab: usize,
    verify: bool,
}

fn parse_prepare(rest: &[String]) -> Result<PrepareArgs, Box<dyn Error>> {
    let mut args = PrepareArgs {
        tokenizer: PathBuf::new(),
        out_dir: PathBuf::new(),
        train: Vec::new(),
        heldout: Vec::new(),
        max_tokens: DEFAULT_MAX_TOKENS,
        limit: None,
        expect_vocab: DEFAULT_EXPECT_VOCAB,
        verify: true,
    };
    let mut i = 0;
    while i < rest.len() {
        let key = rest[i].as_str();
        let value = |i: usize| -> Result<&String, Box<dyn Error>> {
            rest.get(i + 1)
                .ok_or_else(|| format!("option {key} requires a value").into())
        };
        match key {
            "--tokenizer" => {
                args.tokenizer = PathBuf::from(value(i)?);
                i += 2;
            }
            "--out-dir" => {
                args.out_dir = PathBuf::from(value(i)?);
                i += 2;
            }
            "--train" => {
                args.train.push(PathBuf::from(value(i)?));
                i += 2;
            }
            "--heldout" => {
                args.heldout.push(PathBuf::from(value(i)?));
                i += 2;
            }
            "--max-tokens" => {
                args.max_tokens = value(i)?.parse()?;
                i += 2;
            }
            "--limit" => {
                args.limit = Some(value(i)?.parse()?);
                i += 2;
            }
            "--expect-vocab" => {
                args.expect_vocab = value(i)?.parse()?;
                i += 2;
            }
            "--no-verify" => {
                args.verify = false;
                i += 1;
            }
            other => return Err(format!("unknown prepare option: {other}").into()),
        }
    }
    if args.tokenizer.as_os_str().is_empty() {
        return Err("prepare requires --tokenizer".into());
    }
    if args.out_dir.as_os_str().is_empty() {
        return Err("prepare requires --out-dir".into());
    }
    if args.train.is_empty() && args.heldout.is_empty() {
        return Err("prepare requires at least one --train or --heldout input".into());
    }
    Ok(args)
}

fn label_for(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

fn load_tokenizer(
    path: &Path,
    expect_vocab: usize,
) -> Result<(ByteBpeTokenizer, TokenizerInfo), Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or("failed to parse byte-level BPE tokenizer from tokenizer.json")?;
    let vocab_size = tokenizer.vocab_size();
    if vocab_size != expect_vocab {
        return Err(format!(
            "tokenizer vocabulary is {vocab_size} id slots, expected {expect_vocab}; refusing to emit a corpus under the wrong identity"
        )
        .into());
    }
    let bos = resolve_special(&tokenizer, BOS_SURFACE, "BOS")?;
    let eos = resolve_special(&tokenizer, EOS_SURFACE, "EOS")?;
    let unk = resolve_special(&tokenizer, UNK_SURFACE, "UNK")?;
    let (sha256, _) = sha256_file(path)?;
    let info = TokenizerInfo {
        path: path.to_string_lossy().into_owned(),
        sha256,
        tokenizer_cid: tokenizer.address(),
        vocab_size,
        bos_id: bos,
        eos_id: eos,
        unk_id: unk,
    };
    Ok((tokenizer, info))
}

fn run_prepare(rest: &[String]) -> Result<(), Box<dyn Error>> {
    let args = parse_prepare(rest)?;
    let (tokenizer, tokenizer_info) = load_tokenizer(&args.tokenizer, args.expect_vocab)?;
    let ids = (
        tokenizer_info.bos_id,
        tokenizer_info.eos_id,
        tokenizer_info.unk_id,
    );

    println!("prepare-chat-corpus");
    println!("  tokenizer:     {}", tokenizer_info.path);
    println!("  tokenizer_cid: {}", tokenizer_info.tokenizer_cid);
    println!(
        "  vocab:         {} (bos={} eos={} unk={})",
        tokenizer_info.vocab_size, ids.0, ids.1, ids.2
    );
    println!("  out-dir:       {}", args.out_dir.display());
    println!(
        "  max-tokens:    {}  limit: {}",
        args.max_tokens,
        args.limit.map_or("none".to_string(), |n| n.to_string())
    );

    let mut splits: BTreeMap<String, SplitManifest> = BTreeMap::new();

    let train_inputs: Vec<InputInfo> = args
        .train
        .iter()
        .map(|path| InputInfo {
            label: label_for(path),
            path: path.to_string_lossy().into_owned(),
        })
        .collect();
    if !train_inputs.is_empty() {
        let manifest = prepare_split(
            "train",
            &train_inputs,
            &tokenizer,
            ids,
            &args.out_dir,
            args.max_tokens,
            args.limit,
            &tokenizer_info,
        )?;
        print_split_summary("train", &manifest);
        splits.insert("train".to_string(), manifest);
    }

    let heldout_inputs: Vec<InputInfo> = args
        .heldout
        .iter()
        .map(|path| InputInfo {
            label: label_for(path),
            path: path.to_string_lossy().into_owned(),
        })
        .collect();
    if !heldout_inputs.is_empty() {
        let manifest = prepare_split(
            "heldout",
            &heldout_inputs,
            &tokenizer,
            ids,
            &args.out_dir,
            args.max_tokens,
            args.limit,
            &tokenizer_info,
        )?;
        print_split_summary("heldout", &manifest);
        splits.insert("heldout".to_string(), manifest);
    }

    write_json(&args.out_dir.join("manifest.json"), &splits)?;
    println!("wrote {}", args.out_dir.join("manifest.json").display());

    if args.verify {
        for (split, manifest) in &splits {
            let dir = args.out_dir.join(split);
            verify_split(&tokenizer, &dir, 64)?;
            if manifest.rows_used == 0 {
                return Err(format!("split '{split}' produced zero usable rows").into());
            }
        }
    }
    Ok(())
}

fn print_split_summary(split: &str, manifest: &SplitManifest) {
    let dropped = manifest.drops.total_rows_dropped();
    println!(
        "  [{split}] rows {} / {} used, dropped {dropped}; tokens {} ({} response, {:.2}%); tokens_sha256 {}",
        manifest.rows_used,
        manifest.rows_total,
        manifest.tokens,
        manifest.response_tokens,
        manifest.response_fraction * 100.0,
        manifest.tokens_sha256
    );
}

fn verify_split(
    tokenizer: &ByteBpeTokenizer,
    dir: &Path,
    preview: usize,
) -> Result<(), Box<dyn Error>> {
    let tokens_path = dir.join("tokens.u16");
    let mask_path = dir.join("response_mask.u8");
    let reader = MmapCorpusReader::open(&tokens_path)?;
    let mask = fs::read(&mask_path)?;
    if mask.len() != reader.len() {
        return Err(format!(
            "{}: mask length {} != token length {}",
            dir.display(),
            mask.len(),
            reader.len()
        )
        .into());
    }
    let ones = mask.iter().filter(|&&m| m == 1).count();
    let bos = tokenizer.encode(BOS_SURFACE);
    let eos = tokenizer.encode(EOS_SURFACE);
    if bos.len() != 1 || eos.len() != 1 {
        return Err(format!("{}: tokenizer lacks single-id BOS/EOS", dir.display()).into());
    }
    let bos_id = bos[0] as u16;
    let eos_id = eos[0] as u16;
    let marker = tokenizer.encode(MARKER_ASSISTANT);
    let marker_len = marker.len();
    let total = reader.len();
    if total == 0 {
        return Err(format!("{}: empty token stream", dir.display()).into());
    }
    if reader[0] != bos_id || reader[total - 1] != eos_id {
        return Err(format!("{}: stream does not start/end with BOS/EOS", dir.display()).into());
    }

    let mut documents = 1u64;
    let mut doc_boundary_violations = 0u64;
    let mut masked_runs = 0u64;
    let mut run_marker_violations = 0u64;
    let mut run_eos_violations = 0u64;
    let mut masked_bos_violations = 0u64;
    let mut in_run = false;
    for index in 0..total {
        let token = reader[index];
        if index > 0 && token == bos_id {
            documents += 1;
            if reader[index - 1] != eos_id {
                doc_boundary_violations += 1;
            }
        }
        if mask[index] == 1 {
            if token == bos_id {
                masked_bos_violations += 1;
            }
            if !in_run {
                in_run = true;
                masked_runs += 1;
                let preceded_by_marker = index >= marker_len
                    && (0..marker_len).all(|k| reader[index - marker_len + k] as u32 == marker[k]);
                if !preceded_by_marker {
                    run_marker_violations += 1;
                }
            }
        } else if in_run {
            in_run = false;
            if reader[index - 1] != eos_id {
                run_eos_violations += 1;
            }
        }
    }
    if in_run && reader[total - 1] != eos_id {
        run_eos_violations += 1;
    }

    let violations = doc_boundary_violations
        + run_marker_violations
        + run_eos_violations
        + masked_bos_violations;
    let n = preview.min(total);
    let ids: Vec<u32> = (0..n).map(|i| reader[i] as u32).collect();
    let decoded = tokenizer.decode(&ids);
    println!(
        "  verify {}: tokens={} mask_ones={} ({:.2}%) docs={} masked_runs={} violations={}",
        dir.display(),
        total,
        ones,
        100.0 * ones as f64 / total as f64,
        documents,
        masked_runs,
        violations
    );
    if violations > 0 {
        return Err(format!(
            "{}: structural violations doc_boundary={doc_boundary_violations} run_marker={run_marker_violations} run_eos={run_eos_violations} masked_bos={masked_bos_violations}",
            dir.display()
        )
        .into());
    }
    println!("    preview[0..{n}] mask={:?}", &mask[..n]);
    println!("    preview[0..{n}] text={decoded:?}");
    Ok(())
}

fn parse_verify(rest: &[String]) -> Result<(PathBuf, Vec<PathBuf>, usize), Box<dyn Error>> {
    let mut tokenizer = PathBuf::new();
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut preview = 64usize;
    let mut i = 0;
    while i < rest.len() {
        let key = rest[i].as_str();
        let value = |i: usize| -> Result<&String, Box<dyn Error>> {
            rest.get(i + 1)
                .ok_or_else(|| format!("option {key} requires a value").into())
        };
        match key {
            "--tokenizer" => {
                tokenizer = PathBuf::from(value(i)?);
                i += 2;
            }
            "--dir" => {
                dirs.push(PathBuf::from(value(i)?));
                i += 2;
            }
            "--preview" => {
                preview = value(i)?.parse()?;
                i += 2;
            }
            other => return Err(format!("unknown verify option: {other}").into()),
        }
    }
    if tokenizer.as_os_str().is_empty() || dirs.is_empty() {
        return Err("verify requires --tokenizer and at least one --dir".into());
    }
    Ok((tokenizer, dirs, preview))
}

fn run_verify(rest: &[String]) -> Result<(), Box<dyn Error>> {
    let (tokenizer_path, dirs, preview) = parse_verify(rest)?;
    let (tokenizer, info) = load_tokenizer(&tokenizer_path, DEFAULT_EXPECT_VOCAB)?;
    println!("verify-chat-corpus (tokenizer_cid {})", info.tokenizer_cid);
    for dir in &dirs {
        verify_split(&tokenizer, dir, preview)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let argv: Vec<String> = std::env::args().collect();
    let program = argv
        .first()
        .map(String::as_str)
        .unwrap_or("prepare-chat-corpus");
    match argv.get(1).map(String::as_str) {
        Some("prepare") => run_prepare(&argv[2..]),
        Some("verify") => run_verify(&argv[2..]),
        Some("--help") | Some("-h") | None => {
            usage(program);
            Ok(())
        }
        Some(other) => {
            eprintln!("unknown subcommand: {other}");
            usage(program);
            Err("unknown subcommand".into())
        }
    }
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// The standard GPT-2 byte-to-unicode table, reproduced here so the test
    /// fixture is a true byte-level tokenizer with one token per input byte.
    fn byte_encoder() -> Vec<char> {
        let mut table = ['\0'; 256];
        let mut assigned = [false; 256];
        for byte in (b'!'..=b'~').chain(0xA1..=0xAC).chain(0xAE..=0xFF) {
            table[byte as usize] = char::from_u32(u32::from(byte)).expect("valid codepoint");
            assigned[byte as usize] = true;
        }
        let mut extra = 0u32;
        for byte in 0..256usize {
            if !assigned[byte] {
                table[byte] = char::from_u32(256 + extra).expect("valid codepoint");
                extra += 1;
            }
        }
        table.to_vec()
    }

    /// Build a deterministic single-byte-per-token fixture tokenizer: ids 0..2
    /// are bos/eos/unk, ids 3..259 are the 256 byte symbols, and there are no
    /// merges. `encode(s)` therefore yields exactly one token per input byte.
    fn byte_fixture_tokenizer() -> ByteBpeTokenizer {
        let encoder = byte_encoder();
        let mut vocab = serde_json::Map::new();
        vocab.insert(BOS_SURFACE.to_string(), json!(0));
        vocab.insert(EOS_SURFACE.to_string(), json!(1));
        vocab.insert(UNK_SURFACE.to_string(), json!(2));
        for (byte, ch) in encoder.iter().enumerate() {
            vocab.insert(ch.to_string(), json!(3 + byte));
        }
        let fixture = json!({
            "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
            "added_tokens": [
                {"id": 0, "content": BOS_SURFACE},
                {"id": 1, "content": EOS_SURFACE},
                {"id": 2, "content": UNK_SURFACE},
            ],
            "model": {"type": "BPE", "vocab": vocab, "merges": []},
        });
        ByteBpeTokenizer::from_tokenizer_json_bytes(fixture.to_string().as_bytes())
            .expect("byte-level fixture parses")
    }

    fn message(role: &str, content: &str) -> Message {
        Message {
            role: Some(role.to_string()),
            content: Some(content.to_string()),
        }
    }

    fn fixture_ids(tokenizer: &ByteBpeTokenizer) -> (u32, u32, u32) {
        (
            resolve_special(tokenizer, BOS_SURFACE, "BOS").unwrap(),
            resolve_special(tokenizer, EOS_SURFACE, "EOS").unwrap(),
            resolve_special(tokenizer, UNK_SURFACE, "UNK").unwrap(),
        )
    }

    fn decode_conversation(encoded: &Encoded, tokenizer: &ByteBpeTokenizer) -> String {
        let ids: Vec<u32> = encoded.tokens.iter().map(|&t| t as u32).collect();
        tokenizer.decode(&ids)
    }

    fn unique_temp_dir(tag: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "uor-prepare-chat-corpus-{}-{}-{}",
            std::process::id(),
            tag,
            n
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn single_turn_exact_mask_and_decode() {
        let tokenizer = byte_fixture_tokenizer();
        let ids = fixture_ids(&tokenizer);
        let messages = vec![message("user", "Hi"), message("assistant", "Hello")];
        let encoded = encode_conversation(&messages, &tokenizer, ids.0, ids.1, ids.2);

        assert_eq!(
            decode_conversation(&encoded, &tokenizer),
            "<|bos|>User: Hi\nAssistant: Hello<|eos|>"
        );
        // One token per byte: BOS(1) + "User: "(6) + "Hi"(2) + "\n"(1) +
        // "Assistant: "(11) + "Hello"(5) + EOS(1).
        let mut expected = vec![0u8; 1 + 6 + 2 + 1 + 11];
        expected.extend_from_slice(&[1u8; 5]);
        expected.push(1);
        assert_eq!(encoded.mask, expected);
        assert_eq!(encoded.response_tokens, 6);
        assert_eq!(encoded.tokens.len(), encoded.mask.len());
    }

    #[test]
    fn multi_turn_boundaries_exact_mask_and_decode() {
        let tokenizer = byte_fixture_tokenizer();
        let ids = fixture_ids(&tokenizer);
        let messages = vec![
            message("user", "A"),
            message("assistant", "B"),
            message("user", "C"),
            message("assistant", "D"),
        ];
        let encoded = encode_conversation(&messages, &tokenizer, ids.0, ids.1, ids.2);

        assert_eq!(
            decode_conversation(&encoded, &tokenizer),
            "<|bos|>User: A\nAssistant: B<|eos|>\nUser: C\nAssistant: D<|eos|>"
        );
        // 0: BOS, "User: "(6), "A"(1), "\n"(1), "Assistant: "(11) = 20 zeros.
        // 1: "B"(1) + EOS(1) = 2 ones.
        // 0: "\n"(1), "User: "(6), "C"(1), "\n"(1), "Assistant: "(11) = 20 zeros.
        // 1: "D"(1) + EOS(1) = 2 ones.
        let mut expected = vec![0u8; 20];
        expected.extend_from_slice(&[1, 1]);
        expected.extend_from_slice(&[0u8; 20]);
        expected.extend_from_slice(&[1, 1]);
        assert_eq!(encoded.mask, expected);
        assert_eq!(encoded.response_tokens, 4);
    }

    #[test]
    fn trailing_user_gets_unmasked_terminal_eos() {
        let tokenizer = byte_fixture_tokenizer();
        let ids = fixture_ids(&tokenizer);
        let messages = vec![
            message("user", "A"),
            message("assistant", "B"),
            message("user", "C"),
        ];
        let encoded = encode_conversation(&messages, &tokenizer, ids.0, ids.1, ids.2);

        assert_eq!(
            decode_conversation(&encoded, &tokenizer),
            "<|bos|>User: A\nAssistant: B<|eos|>\nUser: C<|eos|>"
        );
        assert_eq!(*encoded.mask.last().unwrap(), 0, "terminal EOS is unmasked");
        assert_eq!(encoded.response_tokens, 2);
        // Index 20 is response "B" and index 21 is its masked terminating EOS.
        assert_eq!(encoded.mask[20], 1);
        assert_eq!(encoded.mask[21], 1);
    }

    #[test]
    fn system_turn_is_prompt_context() {
        let tokenizer = byte_fixture_tokenizer();
        let ids = fixture_ids(&tokenizer);
        let messages = vec![
            message("system", "S"),
            message("user", "A"),
            message("assistant", "B"),
        ];
        let encoded = encode_conversation(&messages, &tokenizer, ids.0, ids.1, ids.2);

        assert_eq!(
            decode_conversation(&encoded, &tokenizer),
            "<|bos|>System: S\nUser: A\nAssistant: B<|eos|>"
        );
        // Only "B" and the assistant EOS are masked.
        assert_eq!(encoded.response_tokens, 2);
        assert_eq!(encoded.mask[encoded.mask.len() - 2], 1);
        assert_eq!(encoded.mask[encoded.mask.len() - 1], 1);
    }

    #[test]
    fn empty_content_is_skipped_and_row_has_no_response() {
        let tokenizer = byte_fixture_tokenizer();
        let ids = fixture_ids(&tokenizer);
        let messages = vec![
            message("user", "Hi"),
            message("assistant", "   "),
            message("user", "Again?"),
        ];
        let encoded = encode_conversation(&messages, &tokenizer, ids.0, ids.1, ids.2);
        // The empty assistant message is not emitted, so there is no response.
        assert_eq!(encoded.skipped_empty, 1);
        assert_eq!(encoded.response_tokens, 0);
        assert_eq!(encoded.emitted_turns, 2);
        assert_eq!(
            decode_conversation(&encoded, &tokenizer),
            "<|bos|>User: Hi\nUser: Again?<|eos|>"
        );
    }

    #[test]
    fn unknown_role_is_skipped_and_counted() {
        let tokenizer = byte_fixture_tokenizer();
        let ids = fixture_ids(&tokenizer);
        let messages = vec![
            message("tool", "secret"),
            message("user", "Hi"),
            message("assistant", "Hello"),
        ];
        let encoded = encode_conversation(&messages, &tokenizer, ids.0, ids.1, ids.2);
        assert_eq!(encoded.skipped_unknown_role, 1);
        assert_eq!(
            decode_conversation(&encoded, &tokenizer),
            "<|bos|>User: Hi\nAssistant: Hello<|eos|>"
        );
    }

    fn write_jsonl(path: &Path, lines: &[serde_json::Value]) {
        let mut text = String::new();
        for line in lines {
            text.push_str(&line.to_string());
            text.push('\n');
        }
        fs::write(path, text).expect("write jsonl");
    }

    #[test]
    fn drop_counts_and_split_files_are_written() {
        let tokenizer = byte_fixture_tokenizer();
        let ids = fixture_ids(&tokenizer);
        let info = TokenizerInfo {
            path: "fixture".into(),
            sha256: "fixture".into(),
            tokenizer_cid: tokenizer.address(),
            vocab_size: tokenizer.vocab_size(),
            bos_id: ids.0,
            eos_id: ids.1,
            unk_id: ids.2,
        };
        let root = unique_temp_dir("drops");
        let input = root.join("synth.jsonl");
        let lines = [
            json!({"row_idx": 0, "row": {"messages": []}}),
            json!({"row_idx": 1, "row": {"messages": [
                {"role": "user", "content": "   "},
                {"role": "assistant", "content": "\n"}
            ]}}),
            json!({"row_idx": 2, "row": {"messages": [
                {"role": "user", "content": "A"},
                {"role": "assistant", "content": "B"}
            ]}}),
            json!({"row_idx": 3, "row": {"messages": [
                {"role": "user", "content": "no assistant here"}
            ]}}),
            json!({"row_idx": 4, "row": {"other": 1}}),
        ];
        let mut text = String::new();
        for line in &lines {
            text.push_str(&line.to_string());
            text.push('\n');
        }
        text.push_str("{not valid json\n");
        fs::write(&input, text).expect("write jsonl");

        let inputs = vec![InputInfo {
            label: "synth".into(),
            path: input.to_string_lossy().into_owned(),
        }];
        let manifest = prepare_split(
            "train",
            &inputs,
            &tokenizer,
            ids,
            &root,
            DEFAULT_MAX_TOKENS,
            None,
            &info,
        )
        .expect("prepare");

        assert_eq!(manifest.rows_used, 1);
        assert_eq!(manifest.drops.rows_dropped_no_messages, 2);
        assert_eq!(manifest.drops.rows_dropped_empty, 1);
        assert_eq!(manifest.drops.rows_dropped_no_response, 1);
        assert_eq!(manifest.drops.rows_dropped_malformed, 1);
        assert_eq!(manifest.tokens, 22);
        assert_eq!(manifest.response_tokens, 2);
        assert_eq!(manifest.mask_bytes, manifest.tokens);

        let tokens = fs::read(root.join("train/tokens.u16")).expect("tokens");
        assert_eq!(tokens.len() as u64, 64 + manifest.tokens * 2);
        let mask = fs::read(root.join("train/response_mask.u8")).expect("mask");
        assert_eq!(mask.len() as u64, manifest.tokens);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn oversized_rows_are_dropped_with_counts() {
        let tokenizer = byte_fixture_tokenizer();
        let ids = fixture_ids(&tokenizer);
        let info = TokenizerInfo {
            path: "fixture".into(),
            sha256: "fixture".into(),
            tokenizer_cid: tokenizer.address(),
            vocab_size: tokenizer.vocab_size(),
            bos_id: ids.0,
            eos_id: ids.1,
            unk_id: ids.2,
        };
        let root = unique_temp_dir("oversized");
        let input = root.join("synth.jsonl");
        write_jsonl(
            &input,
            &[json!({"row": {"messages": [
                {"role": "user", "content": "hi"},
                {"role": "assistant", "content": "there"}
            ]}})],
        );
        let inputs = vec![InputInfo {
            label: "synth".into(),
            path: input.to_string_lossy().into_owned(),
        }];
        // Max tokens 4 is smaller than the rendered length, so the row drops.
        let manifest = prepare_split("train", &inputs, &tokenizer, ids, &root, 4, None, &info)
            .expect("prepare");
        assert_eq!(manifest.rows_used, 0);
        assert_eq!(manifest.drops.rows_dropped_oversized, 1);
        assert_eq!(manifest.tokens, 0);
        let mask = fs::read(root.join("train/response_mask.u8")).expect("mask");
        assert!(mask.is_empty());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn two_runs_are_byte_identical() {
        let tokenizer = byte_fixture_tokenizer();
        let ids = fixture_ids(&tokenizer);
        let info = TokenizerInfo {
            path: "fixture".into(),
            sha256: "fixture".into(),
            tokenizer_cid: tokenizer.address(),
            vocab_size: tokenizer.vocab_size(),
            bos_id: ids.0,
            eos_id: ids.1,
            unk_id: ids.2,
        };
        let root_a = unique_temp_dir("det-a");
        let root_b = unique_temp_dir("det-b");
        let input_root = unique_temp_dir("det-input");
        let input = input_root.join("synth.jsonl");
        let rows = [
            json!({"row": {"messages": [
                {"role": "user", "content": "Hi"},
                {"role": "assistant", "content": "Hello there"}
            ]}}),
            json!({"row": {"messages": [
                {"role": "system", "content": "Be brief."},
                {"role": "user", "content": "Name a color."},
                {"role": "assistant", "content": "Blue."}
            ]}}),
        ];
        write_jsonl(&input, &rows);

        let run = |root: &Path| {
            let inputs = vec![InputInfo {
                label: "synth".into(),
                path: input.to_string_lossy().into_owned(),
            }];
            prepare_split(
                "train",
                &inputs,
                &tokenizer,
                ids,
                root,
                DEFAULT_MAX_TOKENS,
                None,
                &info,
            )
            .expect("prepare")
        };
        run(&root_a);
        run(&root_b);

        for artifact in [
            "train/tokens.u16",
            "train/response_mask.u8",
            "train/manifest.json",
        ] {
            let a = fs::read(root_a.join(artifact)).expect("a");
            let b = fs::read(root_b.join(artifact)).expect("b");
            assert_eq!(a, b, "{artifact} differs between runs");
        }
        let _ = fs::remove_dir_all(&root_a);
        let _ = fs::remove_dir_all(&root_b);
        let _ = fs::remove_dir_all(&input_root);
    }
}
