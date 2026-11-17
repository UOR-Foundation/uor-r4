//! Prepare a literal-role dialogue split (`uor-r4-chat-corpus/v1`) from JSONL whose rows carry a
//! `messages` array of `{role, content}` objects — the donor bank's own format.
//!
//! ```text
//! prepare-chat-jsonl out=NEW_DIR tokenizer=TOKENIZER.json \
//!   source=LABEL:LICENCE:FILE[|FILE...] [source=...] \
//!   [protocol=2] [version=2] [max_tokens=8192] [vocab_size=32768]
//! ```
//!
//! It is `prepare-chat-parquet` with a line reader in place of a parquet reader: the encoding,
//! the response mask, the batching, the drop rules and the manifest are the same, so a corpus
//! built here is indistinguishable downstream from one built from parquet.

use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use rayon::prelude::*;
use serde_json::json;
use uor_r4_core::native_geometric::mmap_corpus::CorpusWriter;
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, String>;

const BATCH_ROWS: usize = 4096;

struct Source {
    label: String,
    licence: String,
    files: Vec<PathBuf>,
}

fn parse_source(spec: &str) -> Result<Source> {
    let mut parts = spec.splitn(3, ':');
    let (Some(label), Some(licence), Some(files)) = (parts.next(), parts.next(), parts.next())
    else {
        return Err(format!(
            "source={spec}: expected LABEL:LICENCE:FILE[|FILE...]"
        ));
    };
    let files: Vec<PathBuf> = files.split('|').map(PathBuf::from).collect();
    if label.is_empty() || licence.is_empty() || files.iter().any(|f| f.as_os_str().is_empty()) {
        return Err(format!("source={spec}: empty label, licence or file"));
    }
    Ok(Source {
        label: label.to_owned(),
        licence: licence.to_owned(),
        files,
    })
}

/// The `(role, content)` pairs of a row's `messages` array.
fn messages(line: &str) -> Result<Vec<(String, String)>> {
    let value: serde_json::Value =
        serde_json::from_str(line).map_err(|e| format!("a row is not JSON: {e}"))?;
    let list = value
        .get("messages")
        .ok_or("a row has no messages field")?
        .as_array()
        .ok_or("the messages field is not an array")?;
    let mut out = Vec::with_capacity(list.len());
    for message in list {
        let role = message
            .get("role")
            .and_then(|v| v.as_str())
            .ok_or("a message has no role")?;
        let content = message
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        out.push((role.to_owned(), content.to_owned()));
    }
    Ok(out)
}

enum Outcome {
    Kept(Vec<u16>, Vec<u8>),
    Empty,
    NoResponse,
    Oversized,
    Special,
    Defect,
}

fn encode_row(
    encoder: &DialogueEncoder<'_>,
    row: &[(String, String)],
    max_tokens: usize,
) -> (Outcome, usize, usize) {
    let messages: Vec<Message<'_>> = row
        .iter()
        .map(|(role, content)| Message {
            role: role.as_str(),
            content: content.as_str(),
        })
        .collect();
    let encoded = encoder.encode_document(&messages);
    let skipped = (encoded.skipped_empty, encoded.skipped_unknown_role);
    let outcome = if encoded.emitted_turns == 0 {
        Outcome::Empty
    } else if encoded.special_token_occurrences > 0 {
        Outcome::Special
    } else if !encoded.response_mask.contains(&1) {
        Outcome::NoResponse
    } else if encoded.tokens.len() > max_tokens {
        Outcome::Oversized
    } else {
        match encoded
            .tokens
            .iter()
            .map(|&t| u16::try_from(t))
            .collect::<std::result::Result<Vec<u16>, _>>()
        {
            Ok(ids) if ids.len() == encoded.response_mask.len() => {
                Outcome::Kept(ids, encoded.response_mask)
            }
            _ => Outcome::Defect,
        }
    };
    (outcome, skipped.0, skipped.1)
}

#[derive(Default)]
struct Counts {
    rows_total: usize,
    rows_used: usize,
    rows_dropped_empty: usize,
    rows_dropped_no_response: usize,
    rows_dropped_oversized: usize,
    rows_dropped_special: usize,
    messages_skipped_empty: usize,
    messages_skipped_unknown_role: usize,
    tokens: usize,
    response_tokens: usize,
}

fn run(args: &[String]) -> Result<()> {
    let value = |key: &str| args.iter().find_map(|a| a.strip_prefix(&format!("{key}=")));
    let number = |key: &str, default: u64| -> Result<u64> {
        value(key).map_or(Ok(default), |v| {
            v.parse().map_err(|_| format!("{key}= must be a number"))
        })
    };
    let out = PathBuf::from(value("out").ok_or("out= is required")?);
    let tokenizer_path = PathBuf::from(value("tokenizer").ok_or("tokenizer= is required")?);
    let protocol_version = u8::try_from(number("protocol", 2)?).map_err(|_| "protocol= too big")?;
    let version = u8::try_from(number("version", 2)?).map_err(|_| "version= too big")?;
    let max_tokens = usize::try_from(number("max_tokens", 8192)?).map_err(|_| "max_tokens")?;
    let vocab_size = usize::try_from(number("vocab_size", 32768)?).map_err(|_| "vocab_size")?;
    let sources: Vec<Source> = args
        .iter()
        .filter_map(|a| a.strip_prefix("source="))
        .map(parse_source)
        .collect::<Result<Vec<_>>>()?;
    if sources.is_empty() {
        return Err("at least one source= is required".into());
    }

    let tokenizer_json = fs::read(&tokenizer_path).map_err(|e| e.to_string())?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json)
        .ok_or("unreadable tokenizer.json")?;
    let protocol = DialogueProtocol::literal_roles_version(&tokenizer, version)
        .map_err(|e| e.to_string())?;
    let encoder: DialogueEncoder<'_> = protocol.bind(&tokenizer).map_err(|e| e.to_string())?;
    let _ = protocol_version;

    report_output::claim(&out).map_err(|e| e.to_string())?;
    let tokens_path = out.join("tokens.u16");
    let mask_path = out.join("response_mask.u8");
    let mut writer = CorpusWriter::create(&tokens_path, vocab_size as u32).map_err(|e| e.to_string())?;
    let mut mask = BufWriter::new(File::create(&mask_path).map_err(|e| e.to_string())?);

    let mut all = Counts::default();
    let mut file_entries = Vec::new();
    for source in &sources {
        let mut counts = Counts::default();
        let mut input_files = Vec::new();
        let mut batch: Vec<Vec<(String, String)>> = Vec::with_capacity(BATCH_ROWS);
        let flush = |batch: &mut Vec<Vec<(String, String)>>,
                     counts: &mut Counts,
                     writer: &mut CorpusWriter,
                     mask: &mut BufWriter<File>|
         -> Result<()> {
            let encoded: Vec<(Outcome, usize, usize)> = batch
                .par_iter()
                .map(|row| encode_row(&encoder, row, max_tokens))
                .collect();
            for (outcome, empty, unknown) in encoded {
                counts.messages_skipped_empty += empty;
                counts.messages_skipped_unknown_role += unknown;
                match outcome {
                    Outcome::Kept(ids, row_mask) => {
                        writer.write_tokens(&ids).map_err(|e| e.to_string())?;
                        mask.write_all(&row_mask).map_err(|e| e.to_string())?;
                        counts.rows_used += 1;
                        counts.tokens += ids.len();
                        counts.response_tokens += row_mask.iter().filter(|&&m| m == 1).count();
                    }
                    Outcome::Empty => counts.rows_dropped_empty += 1,
                    Outcome::NoResponse => counts.rows_dropped_no_response += 1,
                    Outcome::Oversized => counts.rows_dropped_oversized += 1,
                    Outcome::Special => counts.rows_dropped_special += 1,
                    Outcome::Defect => {
                        return Err("the encoder produced a token id above u16 or a token/mask \
                                    length mismatch"
                            .into())
                    }
                }
            }
            batch.clear();
            Ok(())
        };
        for file in &source.files {
            let reader = BufReader::new(File::open(file).map_err(|e| e.to_string())?);
            input_files.push(json!({
                "path": file.display().to_string(),
                "bytes": fs::metadata(file).map_err(|e| e.to_string())?.len(),
                "sha256": uor_r4_training::sha256_file(file).map_err(|e| e.to_string())?,
            }));
            for line in reader.lines() {
                let line = line.map_err(|e| format!("{}: {e}", file.display()))?;
                if line.trim().is_empty() {
                    continue;
                }
                counts.rows_total += 1;
                batch.push(messages(&line)?);
                if batch.len() == BATCH_ROWS {
                    flush(&mut batch, &mut counts, &mut writer, &mut mask)?;
                }
            }
        }
        flush(&mut batch, &mut counts, &mut writer, &mut mask)?;
        println!(
            "{}: {} rows read, {} used, {} tokens ({} response); dropped empty {}, no response \
             {}, oversized {}, special {}",
            source.label,
            counts.rows_total,
            counts.rows_used,
            counts.tokens,
            counts.response_tokens,
            counts.rows_dropped_empty,
            counts.rows_dropped_no_response,
            counts.rows_dropped_oversized,
            counts.rows_dropped_special
        );
        file_entries.push(json!({
            "label": source.label,
            "licence": source.licence,
            "input_files": input_files,
            "tokens": counts.tokens,
            "response_tokens": counts.response_tokens,
            "rows_total": counts.rows_total,
            "rows_used": counts.rows_used,
            "rows_dropped_empty": counts.rows_dropped_empty,
            "rows_dropped_no_response": counts.rows_dropped_no_response,
            "rows_dropped_oversized": counts.rows_dropped_oversized,
            "rows_dropped_special_text": counts.rows_dropped_special,
            "messages_skipped_empty": counts.messages_skipped_empty,
            "messages_skipped_unknown_role": counts.messages_skipped_unknown_role,
        }));
        all.rows_total += counts.rows_total;
        all.rows_used += counts.rows_used;
        all.rows_dropped_empty += counts.rows_dropped_empty;
        all.rows_dropped_no_response += counts.rows_dropped_no_response;
        all.rows_dropped_oversized += counts.rows_dropped_oversized;
        all.rows_dropped_special += counts.rows_dropped_special;
        all.messages_skipped_empty += counts.messages_skipped_empty;
        all.messages_skipped_unknown_role += counts.messages_skipped_unknown_role;
        all.tokens += counts.tokens;
        all.response_tokens += counts.response_tokens;
    }
    let written = writer.finish().map_err(|e| e.to_string())?;
    mask.flush().map_err(|e| e.to_string())?;
    drop(mask);
    let mask_bytes = fs::metadata(&mask_path).map_err(|e| e.to_string())?.len();
    if written as usize != all.tokens || mask_bytes as usize != all.tokens || all.tokens == 0 {
        return Err(format!(
            "wrote {written} tokens and {mask_bytes} mask bytes, expected {} of each (non-zero)",
            all.tokens
        ));
    }
    let tokens_sha = uor_r4_training::sha256_file(&tokens_path).map_err(|e| e.to_string())?;
    let mask_sha = uor_r4_training::sha256_file(&mask_path).map_err(|e| e.to_string())?;
    let tokenizer_sha = uor_r4_training::sha256_bytes(&tokenizer_json);
    let manifest = json!({
        "schema": "uor-r4-chat-corpus/v1",
        "files": file_entries,
        "rows_total": all.rows_total,
        "rows_used": all.rows_used,
        "tokens": all.tokens,
        "response_tokens": all.response_tokens,
        "tokens_sha256": tokens_sha,
        "mask_sha256": mask_sha,
        "mask_schema": "uor-r4-response-mask/u8/v1",
        "dialogue_protocol": format!("uor-r4.literal-role-dialogue/{version}"),
        "tokenizer": {"path": tokenizer_path.display().to_string(), "sha256": tokenizer_sha},
        "vocab_size": vocab_size,
        "max_tokens": max_tokens,
    });
    fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!(
        "wrote {} tokens ({} response) to {}; tokens {} mask {}",
        all.tokens,
        all.response_tokens,
        out.display(),
        &tokens_sha[..16],
        &mask_sha[..16]
    );
    Ok(())
}

fn main() {
    if let Err(error) = run(&std::env::args().skip(1).collect::<Vec<_>>()) {
        eprintln!("prepare-chat-jsonl: {error}");
        std::process::exit(1);
    }
}
