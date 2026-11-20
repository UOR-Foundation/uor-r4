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

fn main() -> std::process::ExitCode {
    let mut table_path: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--table" => table_path = args.next(),
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
                None => {
                    let _ = writeln!(out, "assistant: I don't know that.");
                }
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
                None => {
                    let _ = writeln!(out, "assistant: I couldn't read a fact there.");
                }
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
