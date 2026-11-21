//! Carry a saved question reader in a D11 artifact, and answer a panel from it.
//!
//! `question-reader-d11 pack PARENT_D11 READER_FILE OUT_D11` writes the
//! carrier (it never overwrites OUT_D11).
//! `question-reader-d11 words D11 PANEL_JSON LIMIT` loads the carrier with
//! the integer client, answers every prompt through
//! `IntegerDiscreteModel::question_reply`, and prints the SHA-256 of the
//! reply ids and of the raw step words, and counts the replies that are
//! token for token the panel row's reference. The two hashes are the streams
//! `conjunction-probe reader-replay` and `reader-words` hash, so a reply
//! served from the artifact can be held equal to the trained reader's.
use sha2::{Digest, Sha256};
use std::{fs, io::Write, process::ExitCode};
use uor_r4_integer::discrete_d11::{pack_question_reader, IntegerDiscreteModel};
use uor_r4_reader_runtime::conjunction::{saved::replay, PanelRow};

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn read(path: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("{path}: {e}"))
}

fn pack(parent_path: &str, reader_path: &str, out_path: &str) -> Result<String, String> {
    let parent = read(parent_path)?;
    let reader = read(reader_path)?;
    let bytes = pack_question_reader(&parent, &reader).map_err(|e| e.to_string())?;
    let mut out = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out_path)
        .map_err(|e| format!("{out_path}: {e}"))?;
    out.write_all(&bytes)
        .and_then(|()| out.sync_all())
        .map_err(|e| format!("{out_path}: {e}"))?;
    Ok(serde_json::json!({
        "schema": "uor-r4.question-reader-d11-pack/1",
        "parent_sha256": sha(&parent),
        "parent_bytes": parent.len(),
        "reader_sha256": sha(&reader),
        "reader_bytes": reader.len(),
        "artifact_sha256": sha(&bytes),
        "artifact_bytes": bytes.len(),
    })
    .to_string())
}

fn words(artifact_path: &str, panel_path: &str, limit: &str) -> Result<String, String> {
    let limit: usize = limit.parse().map_err(|e| format!("LIMIT: {e}"))?;
    let artifact = read(artifact_path)?;
    let panel_bytes = read(panel_path)?;
    let model = IntegerDiscreteModel::from_bytes(&artifact).map_err(|e| e.to_string())?;
    let panel: Vec<PanelRow> = serde_json::from_slice(&panel_bytes).map_err(|e| e.to_string())?;
    let mut replies = Sha256::new();
    let mut reply_ids = 0u64;
    let mut exact = 0u64;
    for row in &panel {
        let reply = model
            .question_reply(&row.prompt, limit)
            .map_err(|e| e.to_string())?;
        reply_ids += reply.len() as u64;
        exact += u64::from(reply == row.reference);
        replies.update((reply.len() as u64).to_le_bytes());
        for id in reply {
            replies.update(id.to_le_bytes());
        }
    }
    let reader = model
        .question_reader()
        .ok_or("this artifact carries no question reader")?;
    let prompts = panel.iter().map(|row| row.prompt.as_slice());
    let found = replay(reader, reader, prompts, limit);
    if found.reply_ids != reply_ids {
        return Err("the word stream and the replies disagree on length".into());
    }
    Ok(serde_json::json!({
        "schema": "uor-r4.question-reader-d11-words/1",
        "target": format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        "artifact_schema": model.artifact_schema(),
        "artifact_sha256": model.artifact_sha256(),
        "reader_sha256": model.question_reader_sha256(),
        "panel_sha256": sha(&panel_bytes),
        "prompts": found.prompts,
        "reply_ids": reply_ids,
        "replies_equal_to_reference": exact,
        "steps": found.steps,
        "served_words_sha256": found.served_words_sha256,
        "forced_differs": found.prompts_forced_differs_from_served,
        "reply_ids_sha256": hex::encode(replies.finalize()),
    })
    .to_string())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let done = match args.as_slice() {
        ["pack", parent, reader, out] => pack(parent, reader, out),
        ["words", artifact, panel, limit] => words(artifact, panel, limit),
        _ => Err("usage: question-reader-d11 pack PARENT_D11 READER_FILE OUT_D11 | words D11 PANEL_JSON LIMIT".into()),
    };
    match done {
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
