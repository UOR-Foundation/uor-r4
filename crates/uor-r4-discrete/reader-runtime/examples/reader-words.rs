//! `reader-words READER_FILE PANEL_JSON LIMIT`: answers a panel from a saved
//! reader file with the portable build and prints the SHA-256 of the raw
//! step words (the stream `conjunction-probe reader-replay` hashes) and of
//! the reply ids. It builds for `wasm32-wasip1` as well as for the host, so
//! the same file can be read by two builds and the words compared.
use sha2::{Digest, Sha256};
use std::process::ExitCode;
use uor_r4_reader_runtime::conjunction::{
    saved::{replay, Reader},
    PanelRow,
};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn run(args: &[String]) -> Result<String, String> {
    let [reader_path, panel_path, limit] = args else {
        return Err("usage: reader-words READER_FILE PANEL_JSON LIMIT".into());
    };
    let limit: usize = limit.parse().map_err(|e| format!("LIMIT: {e}"))?;
    let read = |path: &String| std::fs::read(path).map_err(|e| format!("{path}: {e}"));
    let file = read(reader_path)?;
    let panel_bytes = read(panel_path)?;
    let reader = Reader::from_bytes(&file).map_err(|e| e.to_string())?;
    let panel: Vec<PanelRow> = serde_json::from_slice(&panel_bytes).map_err(|e| e.to_string())?;
    if panel
        .iter()
        .flat_map(|row| row.prompt.iter())
        .any(|&id| id >= reader.vocabulary)
    {
        return Err("a panel id is past the vocabulary".into());
    }
    let prompts = || panel.iter().map(|row| row.prompt.as_slice());
    // Both sides are the file's reader: the served stream is what is wanted.
    let found = replay(&reader, &reader, prompts(), limit);
    let mut replies = Sha256::new();
    for prompt in prompts() {
        let reply = reader.reply(prompt, limit);
        replies.update((reply.len() as u64).to_le_bytes());
        for id in reply {
            replies.update(id.to_le_bytes());
        }
    }
    Ok(serde_json::json!({
        "schema": "uor-r4.reader-runtime-words/1",
        "target": format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        "reader_sha256": hex(&Sha256::digest(&file)),
        "panel_sha256": hex(&Sha256::digest(&panel_bytes)),
        "prompts": found.prompts,
        "reply_ids": found.reply_ids,
        "steps": found.steps,
        "served_words_sha256": found.served_words_sha256,
        "forced_differs": found.prompts_forced_differs_from_served,
        "reply_ids_sha256": hex(&replies.finalize()),
    })
    .to_string())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
