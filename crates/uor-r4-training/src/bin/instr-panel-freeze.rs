//! Freeze an instruction panel whose answers are checkable without a model (codex-48).
//!
//! ```text
//! instr-panel-freeze out=PANEL.json tokenizer=TOKENIZER.json \
//!   source=FILE[|FILE...] [n=200] [max-items=40000]
//! ```
//!
//! Items come from the held-out donor shards only. Each needs a deterministic oracle:
//! - `boxed`: the answer's final `\boxed{…}` (numina, openthoughts, gsm8k);
//! - `number`: the last number the answer states (gsm8k fallback);
//! - `copy`: the exact string a "say exactly this" prompt asks to repeat (wildchat).
//!
//! Items are ordered by the sha256 of their id before sampling, so the draw is deterministic and a
//! frozen panel can be re-derived. The census records what was filtered and why, before any model
//! is scored.

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use serde_json::json;
use sha2::{Digest, Sha256};
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone)]
struct Item {
    id: String,
    source: String,
    kind: &'static str,
    prompt: String,
    expected: String,
}

/// The balanced `{…}` after the last `\boxed` in `text`.
fn last_boxed(text: &str) -> Option<String> {
    let at = text.rfind("\\boxed")?;
    let open = text[at..].find('{')? + at;
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    for i in open..bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(text[open + 1..i].trim().to_owned());
                }
            }
            _ => {}
        }
    }
    None
}

/// The last number in `text`.
fn last_number(text: &str) -> Option<String> {
    let mut end = None;
    let mut start = None;
    for (i, c) in text.char_indices().rev() {
        if c.is_ascii_digit() {
            if end.is_none() {
                end = Some(i + c.len_utf8());
            }
            start = Some(i);
        } else if matches!(c, '.' | ',' | '-') && end.is_some() {
            start = Some(i);
        } else if end.is_some() {
            break;
        }
    }
    let (start, end) = (start?, end?);
    Some(text[start..end].trim_matches(|c: char| c == ',' || c == '.').to_owned())
}

/// The target of a copy instruction, if the prompt is one.
fn copy_target(prompt: &str) -> Option<String> {
    let lower = prompt.to_ascii_lowercase();
    let at = lower
        .find("say exactly this:")
        .or_else(|| lower.find("repeat exactly:"))
        .or_else(|| lower.find("say exactly:"))?;
    let rest = &prompt[at..];
    let colon = rest.find(':')?;
    let target = rest[colon + 1..].lines().next()?.trim();
    if target.is_empty() || target.len() > 200 {
        return None;
    }
    Some(target.to_owned())
}

fn sort_key(id: &str) -> String {
    hex::encode(Sha256::digest(id.as_bytes()))
}

fn main() {
    if let Err(error) = run() {
        eprintln!("instr-panel-freeze: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = |key: &str| {
        args.iter()
            .find_map(|a| a.strip_prefix(&format!("{key}=")))
            .map(str::to_owned)
    };
    let out = PathBuf::from(value("out").ok_or("out= is required")?);
    let tokenizer_path = PathBuf::from(value("tokenizer").ok_or("tokenizer= is required")?);
    let sources: Vec<PathBuf> = value("source")
        .ok_or("source= is required")?
        .split('|')
        .map(PathBuf::from)
        .collect();
    let want: usize = match value("n") {
        Some(v) => v.parse().map_err(|e: std::num::ParseIntError| e.to_string())?,
        None => 200,
    };
    let max_items: usize = match value("max-items") {
        Some(v) => v.parse().map_err(|e: std::num::ParseIntError| e.to_string())?,
        None => 40_000,
    };

    let tokenizer_json = std::fs::read(&tokenizer_path).map_err(|e| e.to_string())?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json)
        .ok_or("unreadable tokenizer.json")?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).map_err(|e| e.to_string())?;
    let encoder: DialogueEncoder<'_> = protocol.bind(&tokenizer).map_err(|e| e.to_string())?;

    let mut seen = 0usize;
    let mut no_answer = 0usize;
    let mut no_oracle = 0usize;
    let mut items: Vec<Item> = Vec::new();
    for path in &sources {
        let reader = BufReader::new(File::open(path).map_err(|e| e.to_string())?);
        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            if line.trim().is_empty() || items.len() >= max_items {
                continue;
            }
            seen += 1;
            let Ok(record) = serde_json::from_str::<serde_json::Value>(&line) else {
                continue;
            };
            let Some(id) = record.get("id").and_then(|v| v.as_str()) else {
                continue;
            };
            let source = record
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_owned();
            let Some(prompt) = record
                .get("messages")
                .and_then(|v| v.as_array())
                .and_then(|m| m.first())
                .and_then(|m| m.get("content"))
                .and_then(|v| v.as_str())
            else {
                no_answer += 1;
                continue;
            };
            let answer = record
                .get("turns")
                .and_then(|v| v.as_array())
                .and_then(|t| t.first())
                .and_then(|t| t.get("answer"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if answer.trim().is_empty() {
                no_answer += 1;
                continue;
            }
            let (kind, expected) = if let Some(target) = copy_target(prompt) {
                ("copy", target)
            } else if let Some(boxed) = last_boxed(answer) {
                ("boxed", boxed)
            } else if matches!(source.as_str(), "gsm8k" | "numina" | "openthoughts") {
                match last_number(answer) {
                    Some(number) => ("number", number),
                    None => {
                        no_oracle += 1;
                        continue;
                    }
                }
            } else {
                no_oracle += 1;
                continue;
            };
            if expected.is_empty() || expected.len() > 64 {
                no_oracle += 1;
                continue;
            }
            items.push(Item {
                id: id.to_owned(),
                source,
                kind,
                prompt: prompt.to_owned(),
                expected,
            });
        }
    }

    items.sort_by_key(|item| sort_key(&item.id));
    items.truncate(want);

    let mut rows = Vec::with_capacity(items.len());
    let mut per_kind: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for item in &items {
        let encoded = encoder.encode_assistant_prefix(&[Message {
            role: "user",
            content: &item.prompt,
        }]);
        if encoded.tokens.len() > 512 {
            continue;
        }
        *per_kind.entry(item.kind).or_default() += 1;
        rows.push(json!({
            "id": item.id,
            "source": item.source,
            "kind": item.kind,
            "prompt": item.prompt,
            "expected": item.expected,
            "tokens": encoded.tokens,
        }));
    }

    let panel = json!({
        "schema": "uor-r4.instr-panel/1",
        "tokenizer_sha256": uor_r4_training::sha256_bytes(&tokenizer_json),
        "n": rows.len(),
        "census": {
            "rows_seen": seen,
            "items_with_a_checkable_oracle": items.len(),
            "rows_without_answer": no_answer,
            "rows_without_oracle": no_oracle,
        },
        "per_kind": per_kind,
        "items": rows,
    });
    let mut file = File::create(&out).map_err(|e| e.to_string())?;
    file.write_all(&serde_json::to_vec_pretty(&panel).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    println!(
        "froze {} items ({} seen, {} without answer, {} without oracle) into {}",
        panel["n"],
        seen,
        no_answer,
        no_oracle,
        out.display()
    );
    Ok(())
}
