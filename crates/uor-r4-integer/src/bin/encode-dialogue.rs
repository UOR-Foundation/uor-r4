//! Encode a dialogue as the served protocol's token ids, for the panel and trace probes.
//!
//! `usage: encode-dialogue --tokenizer PATH [--protocol 1|2] --turn ROLE=TEXT ... [--out FILE]`
//!
//! The prompt is `encode_assistant_prefix` of the turns: everything up to the final assistant
//! marker, so its last token is the row at which the reply's first token is read. `--reply TEXT`
//! also prints that text's own ids (the expected reply) beside the prompt.

use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

fn main() {
    if let Err(error) = run() {
        eprintln!("encode-dialogue: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut turns: Vec<(String, String)> = Vec::new();
    let (mut tokenizer, mut protocol, mut out, mut reply) = (None, 2u8, None, None);
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--tokenizer" => tokenizer = Some(value()?),
            "--protocol" => protocol = value()?.parse().map_err(|e| format!("--protocol: {e}"))?,
            "--turn" => {
                let raw = value()?;
                let (role, text) = raw
                    .split_once('=')
                    .ok_or("--turn wants ROLE=TEXT")?;
                turns.push((role.to_owned(), text.to_owned()));
            }
            "--reply" => reply = Some(value()?),
            "--out" => out = Some(value()?),
            other => return Err(format!("unknown flag {other}")),
        }
    }
    let path = tokenizer.ok_or("--tokenizer is required")?;
    if turns.is_empty() {
        return Err("at least one --turn is required".into());
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("{path}: {e}"))?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or("the tokenizer is not a byte-level BPE tokenizer")?;
    let protocol = match protocol {
        1 => DialogueProtocol::literal_roles_v1(&tokenizer),
        _ => DialogueProtocol::literal_roles_v2(&tokenizer),
    }
    .map_err(|e| format!("the dialogue protocol: {e}"))?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|e| format!("the dialogue encoder: {e}"))?;
    let messages: Vec<Message<'_>> = turns
        .iter()
        .map(|(role, content)| Message {
            role: role.as_str(),
            content: content.as_str(),
        })
        .collect();
    let encoded = encoder.encode_assistant_prefix(&messages);
    let reply_ids: Option<Vec<u32>> = reply.as_ref().map(|text| {
        let mut ids = tokenizer.encode(text);
        ids.push(protocol.eos_id);
        ids
    });
    let value_first: Option<Vec<usize>> = reply.as_ref().map(|text| {
        let needle = tokenizer.encode(text);
        encoded
            .tokens
            .windows(needle.len())
            .enumerate()
            .filter(|(_, window)| *window == needle.as_slice())
            .map(|(at, _)| at)
            .collect()
    });
    let record = serde_json::json!({
        "prompt": encoded.tokens,
        "position": encoded.tokens.len().saturating_sub(1),
        "emitted_turns": encoded.emitted_turns,
        "special_token_occurrences": encoded.special_token_occurrences,
        "reply": reply_ids,
        "reply_starts": value_first,
        "bos": protocol.bos_id,
        "eos": protocol.eos_id,
    });
    let text = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?;
    match out {
        Some(path) => std::fs::write(&path, text).map_err(|e| format!("{path}: {e}"))?,
        None => println!("{text}"),
    }
    Ok(())
}
