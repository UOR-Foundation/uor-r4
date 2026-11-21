//! `memory-chat`: a conversation against the exact slot memory.
//!
//! Statements are written to the store; questions are answered from it by a formed address, or
//! abstained. Nothing is generated: a returned value was stated earlier and is copied out verbatim,
//! which is the mechanism measured in `slot-address-check` and `slot-table-build`.
//!
//! Usage: `memory-chat --table TABLE.json` then one utterance per line on stdin.
//! A line ending in `?` is a question; anything else is a statement.

use std::io::{BufRead, Write};

use uor_r4_integer::slot_memory::{SlotMemory, SlotTagger, ATTRIBUTES};
use uor_r4_integer::stack::{IntegerStackModel, StackChat};

fn main() -> std::process::ExitCode {
    let mut table_path: Option<String> = None;
    let mut artifact: Option<String> = None;
    let mut tokenizer: Option<String> = None;
    let mut protocol: u8 = 2;
    let mut reply_tokens: usize = 24;
    let mut batch: Option<(String, String)> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--table" => table_path = args.next(),
            "--artifact" => artifact = args.next(),
            "--tokenizer" => tokenizer = args.next(),
            "--protocol" => protocol = args.next().and_then(|v| v.parse().ok()).unwrap_or(2),
            "--reply-tokens" => reply_tokens = args.next().and_then(|v| v.parse().ok()).unwrap_or(24),
            "--batch" => {
                let input = args.next();
                let output = args.next();
                match (input, output) {
                    (Some(i), Some(o)) => batch = Some((i, o)),
                    _ => {
                        eprintln!("--batch wants INPUT OUT");
                        return std::process::ExitCode::FAILURE;
                    }
                }
            }
            other => {
                eprintln!("unknown argument {other}");
                return std::process::ExitCode::FAILURE;
            }
        }
    }
    let Some(path) = table_path else {
        eprintln!("--table PATH is required (the slot-table-build output)");
        return std::process::ExitCode::FAILURE;
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("{path}: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let tagger = match SlotTagger::from_json(&text) {
        Ok(tagger) => tagger,
        Err(e) => {
            eprintln!("{e}");
            return std::process::ExitCode::FAILURE;
        }
    };

    // The trunk half, when an artifact is given: memory answers first, the trunk answers the rest.
    let model = match (&artifact, &tokenizer) {
        (Some(artifact), Some(_)) => match IntegerStackModel::load(std::path::Path::new(artifact)) {
            Ok(model) => Some(model),
            Err(e) => {
                eprintln!("{artifact}: {e}");
                return std::process::ExitCode::FAILURE;
            }
        },
        _ => None,
    };
    let mut chat = match (&model, &tokenizer) {
        (Some(model), Some(tokenizer)) => match StackChat::from_tokenizer_path(
            model,
            std::path::Path::new(tokenizer),
            protocol,
        ) {
            Ok(chat) => Some(chat),
            Err(e) => {
                eprintln!("tokenizer {tokenizer}: {e}");
                return std::process::ExitCode::FAILURE;
            }
        },
        _ => None,
    };

    // Batch mode: generate one reply per prompt into JSONL, with the reference beside it, so the
    // chat gate can be graded offline. The memory is not consulted here — this measures the trunk.
    if let (Some((input, output)), Some(chat)) = (&batch, chat.as_mut()) {
        let reader = match std::fs::File::open(input) {
            Ok(file) => std::io::BufReader::new(file),
            Err(e) => {
                eprintln!("{input}: {e}");
                return std::process::ExitCode::FAILURE;
            }
        };
        let mut out = match std::fs::File::create(output) {
            Ok(file) => std::io::BufWriter::new(file),
            Err(e) => {
                eprintln!("{output}: {e}");
                return std::process::ExitCode::FAILURE;
            }
        };
        for line in std::io::BufRead::lines(reader) {
            let Ok(line) = line else { break };
            let Ok(row) = serde_json::from_str::<serde_json::Value>(&line) else { continue };
            let Some(prompt) = row.get("prompt").and_then(|p| p.as_str()) else { continue };
            let reference = row.get("reference").and_then(|r| r.as_str()).unwrap_or("");
            match chat.reply(prompt, reply_tokens) {
                Ok(reply) => {
                    let _ = writeln!(
                        out,
                        "{}",
                        serde_json::json!({"prompt": prompt, "model": reply.text.trim(), "reference": reference})
                    );
                }
                Err(e) => {
                    let _ = writeln!(
                        out,
                        "{}",
                        serde_json::json!({"prompt": prompt, "model": "", "reference": reference, "error": e.to_string()})
                    );
                }
            }
        }
        let _ = out.flush();
        return std::process::ExitCode::SUCCESS;
    }

    let mut memory = SlotMemory::default();
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let _ = writeln!(out, "user: {line}");
        if line.ends_with('?') {
            match memory.answer(&tagger, line) {
                Some((entity, value)) => {
                    let _ = writeln!(out, "assistant: {value}");
                    let _ = writeln!(out, "  [{entity} / {}]", attribute_name(&tagger, line));
                }
                None => match chat.as_mut() {
                    Some(chat) => match chat.reply(line, reply_tokens) {
                        Ok(reply) => {
                            let _ = writeln!(out, "assistant: {}", reply.text.trim());
                        }
                        Err(e) => {
                            let _ = writeln!(out, "assistant: (trunk error: {e})");
                        }
                    },
                    None => {
                        let _ = writeln!(out, "assistant: I don't know that.");
                    }
                },
            }
        } else {
            match memory.write_statement(&tagger, line) {
                Some((entity, attribute)) => {
                    let _ = writeln!(
                        out,
                        "assistant: noted ({entity} / {})",
                        ATTRIBUTES[attribute as usize]
                    );
                }
                None => match chat.as_mut() {
                    // Not a fact: the trunk answers, so a declarative turn is not dropped.
                    Some(chat) => match chat.reply(line, reply_tokens) {
                        Ok(reply) => {
                            let _ = writeln!(out, "assistant: {}", reply.text.trim());
                        }
                        Err(e) => {
                            let _ = writeln!(out, "assistant: (trunk error: {e})");
                        }
                    },
                    None => {
                        let _ = writeln!(out, "assistant: I couldn't read a fact there.");
                    }
                },
            }
        }
    }
    std::process::ExitCode::SUCCESS
}

fn attribute_name(tagger: &SlotTagger, question: &str) -> &'static str {
    tagger
        .attribute_of(question)
        .map(|a| ATTRIBUTES[a as usize])
        .unwrap_or("unknown")
}
