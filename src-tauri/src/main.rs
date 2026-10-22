//! Native desktop client CLI / JSON-RPC host runner for UOR-R4 (`uor-desktop-client`).
//!
//! Supports:
//! - `uor-desktop-client --self-test`: runs an end-to-end verification of the synthetic stand-in
//!   and durable memory store and prints a JSON receipt.
//! - `uor-desktop-client --stdio`: runs a newline-delimited JSON-RPC loop driving [`DesktopBackend`]
//!   commands (`native_list_local_models`, `native_load_model`, `native_load_synthetic_standin`,
//!   `native_step_tokens`, `native_generate_prompt`, `native_export_session`,
//!   `native_import_session`, `native_reset_session`, `native_save_session_durable`,
//!   `native_restore_session_durable`, `native_grant_folder`, `native_read_file`,
//!   `native_write_file`, `native_list_directory`, `native_git_status`,
//!   `native_set_terminal_enabled`, `native_run_terminal_command`).
#![forbid(unsafe_code)]

use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use serde_json::{json, Value};
use uor_r4_desktop_client::{
    build_synthetic_stack_artifact, sha256_bytes, DesktopBackend, PinRegistry,
    TerminalCommandRequest, SYNTHETIC_STANDIN_FILE_NAME, SYNTHETIC_STANDIN_SEED,
};

fn dispatch_command(
    backend: &mut DesktopBackend,
    cmd: &str,
    args: &Value,
) -> Result<Value, String> {
    match cmd {
        "native_list_local_models" => {
            if let Some(dir) = args.get("modelDir").and_then(Value::as_str) {
                backend.add_model_search_dir(PathBuf::from(dir));
            }
            let models = backend
                .native_list_local_models()
                .map_err(|e| e.to_string())?;
            serde_json::to_value(models).map_err(|e| e.to_string())
        }
        "native_load_synthetic_standin" => {
            let bytes = build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED);
            let sha = sha256_bytes(&bytes);
            let caps = backend
                .native_load_model_bytes(SYNTHETIC_STANDIN_FILE_NAME, &bytes, Some(&sha))
                .map_err(|e| e.to_string())?;
            serde_json::to_value(caps).map_err(|e| e.to_string())
        }
        "native_load_model" => {
            if let Some(raw_bytes) = args.get("bytes").and_then(Value::as_array) {
                let mut buf = Vec::with_capacity(raw_bytes.len());
                for v in raw_bytes {
                    let b = v
                        .as_u64()
                        .and_then(|n| u8::try_from(n).ok())
                        .ok_or_else(|| "invalid byte in bytes array".to_string())?;
                    buf.push(b);
                }
                let file_name = args
                    .get("fileName")
                    .and_then(Value::as_str)
                    .unwrap_or(SYNTHETIC_STANDIN_FILE_NAME);
                let expected = args.get("expectedSha256").and_then(Value::as_str);
                let caps = backend
                    .native_load_model_bytes(file_name, &buf, expected)
                    .map_err(|e| e.to_string())?;
                return serde_json::to_value(caps).map_err(|e| e.to_string());
            }
            let file_path = args
                .get("filePath")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing filePath or bytes".to_string())?;
            let expected = args.get("expectedSha256").and_then(Value::as_str);
            let caps = backend
                .native_load_model(std::path::Path::new(file_path), expected)
                .map_err(|e| e.to_string())?;
            serde_json::to_value(caps).map_err(|e| e.to_string())
        }
        "native_step_tokens" => {
            let conv_id = args
                .get("conversationId")
                .and_then(Value::as_str)
                .unwrap_or("default");
            let max_new = args
                .get("maxNewTokens")
                .and_then(Value::as_u64)
                .and_then(|n| usize::try_from(n).ok())
                .unwrap_or(32);
            let mut prompt_tokens = Vec::new();
            if let Some(arr) = args.get("tokenIds").and_then(Value::as_array) {
                for v in arr {
                    let t = v
                        .as_u64()
                        .and_then(|n| u32::try_from(n).ok())
                        .ok_or_else(|| "invalid token id".to_string())?;
                    prompt_tokens.push(t);
                }
            }
            let mut events = Vec::new();
            let receipt = backend
                .native_step_tokens(conv_id, &prompt_tokens, max_new, |ev| events.push(ev))
                .map_err(|e| e.to_string())?;
            Ok(json!({
                "receipt": receipt,
                "events": events,
            }))
        }
        "native_generate_prompt" => {
            let conv_id = args
                .get("conversationId")
                .and_then(Value::as_str)
                .unwrap_or("default");
            let prompt_text = args
                .get("promptText")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing promptText".to_string())?;
            let max_new = args
                .get("maxNewTokens")
                .and_then(Value::as_u64)
                .and_then(|n| usize::try_from(n).ok())
                .unwrap_or(32);
            let mut events = Vec::new();
            let receipt = backend
                .native_generate_prompt(conv_id, prompt_text, max_new, |ev| events.push(ev))
                .map_err(|e| e.to_string())?;
            Ok(json!({
                "receipt": receipt,
                "events": events,
            }))
        }
        "native_export_session" => {
            let conv_id = args
                .get("conversationId")
                .and_then(Value::as_str)
                .unwrap_or("default");
            let (bytes, last_token) = backend
                .native_export_session(conv_id)
                .map_err(|e| e.to_string())?;
            Ok(json!({
                "conversationId": conv_id,
                "bytes": bytes,
                "lastToken": last_token,
            }))
        }
        "native_import_session" => {
            let conv_id = args
                .get("conversationId")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing conversationId".to_string())?;
            let arr = args
                .get("bytes")
                .and_then(Value::as_array)
                .ok_or_else(|| "missing bytes".to_string())?;
            let mut buf = Vec::with_capacity(arr.len());
            for v in arr {
                let b = v
                    .as_u64()
                    .and_then(|n| u8::try_from(n).ok())
                    .ok_or_else(|| "invalid byte".to_string())?;
                buf.push(b);
            }
            let last_token = args
                .get("lastToken")
                .and_then(Value::as_u64)
                .and_then(|n| u32::try_from(n).ok());
            let verified_len = backend
                .native_import_session(conv_id, &buf, last_token)
                .map_err(|e| e.to_string())?;
            Ok(json!({
                "conversationId": conv_id,
                "bytes": verified_len,
            }))
        }
        "native_reset_session" => {
            let conv_id = args
                .get("conversationId")
                .and_then(Value::as_str)
                .unwrap_or("default");
            backend
                .native_reset_session(conv_id)
                .map_err(|e| e.to_string())?;
            Ok(json!({ "conversationId": conv_id, "reset": true }))
        }
        "native_save_session_durable" => {
            let user_id = args
                .get("userId")
                .and_then(Value::as_str)
                .unwrap_or("local-user");
            let project_id = args
                .get("projectId")
                .and_then(Value::as_str)
                .unwrap_or("studio");
            let conv_id = args
                .get("conversationId")
                .and_then(Value::as_str)
                .unwrap_or("default");
            if let Some(dir) = args.get("storeDir").and_then(Value::as_str) {
                backend
                    .set_durable_store_dir(dir)
                    .map_err(|e| e.to_string())?;
            }
            let receipt = backend
                .native_save_session_durable(user_id, project_id, conv_id)
                .map_err(|e| e.to_string())?;
            serde_json::to_value(receipt).map_err(|e| e.to_string())
        }
        "native_restore_session_durable" => {
            let user_id = args
                .get("userId")
                .and_then(Value::as_str)
                .unwrap_or("local-user");
            let project_id = args
                .get("projectId")
                .and_then(Value::as_str)
                .unwrap_or("studio");
            let conv_id = args
                .get("conversationId")
                .and_then(Value::as_str)
                .unwrap_or("default");
            if let Some(dir) = args.get("storeDir").and_then(Value::as_str) {
                backend
                    .set_durable_store_dir(dir)
                    .map_err(|e| e.to_string())?;
            }
            let manifest_vec: Option<Vec<u8>> =
                match args.get("manifestBytes").and_then(Value::as_array) {
                    Some(arr) => {
                        let mut buf = Vec::with_capacity(arr.len());
                        for v in arr {
                            let b = v
                                .as_u64()
                                .and_then(|n| u8::try_from(n).ok())
                                .ok_or_else(|| "invalid manifest byte".to_string())?;
                            buf.push(b);
                        }
                        Some(buf)
                    }
                    None => None,
                };
            let receipt = backend
                .native_restore_session_durable(
                    user_id,
                    project_id,
                    conv_id,
                    manifest_vec.as_deref(),
                )
                .map_err(|e| e.to_string())?;
            serde_json::to_value(receipt).map_err(|e| e.to_string())
        }
        "native_grant_folder" => {
            let folder = args
                .get("folderPath")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing folderPath".to_string())?;
            let canonical = backend
                .native_grant_folder(std::path::Path::new(folder))
                .map_err(|e| e.to_string())?;
            Ok(json!({ "granted": canonical.to_string_lossy() }))
        }
        "native_read_file" => {
            let file_path = args
                .get("filePath")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing filePath".to_string())?;
            let text = backend
                .native_read_file(std::path::Path::new(file_path))
                .map_err(|e| e.to_string())?;
            Ok(Value::String(text))
        }
        "native_write_file" => {
            let file_path = args
                .get("filePath")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing filePath".to_string())?;
            let content = args
                .get("content")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing content".to_string())?;
            backend
                .native_write_file(std::path::Path::new(file_path), content)
                .map_err(|e| e.to_string())?;
            Ok(json!({ "written": true }))
        }
        "native_list_directory" => {
            let folder_path = args
                .get("folderPath")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing folderPath".to_string())?;
            let max_depth = args
                .get("maxDepth")
                .and_then(Value::as_u64)
                .and_then(|n| usize::try_from(n).ok())
                .unwrap_or(4);
            let files = backend
                .native_list_directory(std::path::Path::new(folder_path), max_depth)
                .map_err(|e| e.to_string())?;
            serde_json::to_value(files).map_err(|e| e.to_string())
        }
        "native_git_status" => {
            let repo_path = args
                .get("repoPath")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing repoPath".to_string())?;
            let report = backend
                .native_git_status(std::path::Path::new(repo_path))
                .map_err(|e| e.to_string())?;
            serde_json::to_value(report).map_err(|e| e.to_string())
        }
        "native_set_terminal_enabled" => {
            let enabled = args
                .get("enabled")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            backend.native_set_terminal_enabled(enabled);
            Ok(json!({ "terminalEnabled": backend.terminal_enabled() }))
        }
        "native_run_terminal_command" => {
            let cwd = args
                .get("cwd")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing cwd".to_string())?;
            let command = args
                .get("command")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing command".to_string())?;
            let user_confirmed = args
                .get("userConfirmed")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let confirmed_command = args
                .get("confirmedCommand")
                .and_then(Value::as_str)
                .unwrap_or("");
            let req = TerminalCommandRequest {
                cwd: PathBuf::from(cwd),
                command: command.to_string(),
                user_confirmed,
                confirmed_command: confirmed_command.to_string(),
            };
            let output = backend
                .native_run_terminal_command(&req)
                .map_err(|e| e.to_string())?;
            Ok(Value::String(output))
        }
        other => Err(format!("unknown desktop command: {other}")),
    }
}

fn run_self_test() -> Result<(), String> {
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    let bytes = build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED);
    let sha = sha256_bytes(&bytes);
    let caps = backend
        .native_load_model_bytes(SYNTHETIC_STANDIN_FILE_NAME, &bytes, Some(&sha))
        .map_err(|e| e.to_string())?;
    let mut streamed = Vec::new();
    let receipt = backend
        .native_step_tokens("self-test", &[0], 32, |ev| streamed.push(ev.token_id))
        .map_err(|e| e.to_string())?;
    let durable = backend
        .native_save_session_durable("local-user", "studio", "self-test")
        .map_err(|e| e.to_string())?;
    println!(
        "{}",
        json!({
            "verdict": "PASS",
            "engine": caps.engine,
            "artifactBytes": caps.artifact_bytes,
            "artifactSha256": caps.artifact_sha256,
            "weightsPerToken": caps.weights_per_token,
            "tokens32": receipt.token_ids,
            "streamedCount": streamed.len(),
            "durableScope": durable.scope_key,
            "objectVersion": durable.object_version_hex,
            "ur4sess1Bytes": durable.ur4sess1_bytes_len,
            "storeRecords": durable.store_record_count,
        })
    );
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--self-test") {
        if let Err(err) = run_self_test() {
            eprintln!("self-test error: {err}");
            std::process::exit(1);
        }
        return;
    }

    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let parsed: Result<Value, _> = serde_json::from_str(&line);
        let resp = match parsed {
            Ok(req) => {
                let id = req.get("id").cloned().unwrap_or(Value::Null);
                let cmd = req.get("command").and_then(Value::as_str).unwrap_or("");
                let payload = req.get("args").cloned().unwrap_or_else(|| json!({}));
                match dispatch_command(&mut backend, cmd, &payload) {
                    Ok(result) => json!({ "id": id, "ok": true, "result": result }),
                    Err(error) => json!({ "id": id, "ok": false, "error": error }),
                }
            }
            Err(e) => json!({ "id": Value::Null, "ok": false, "error": e.to_string() }),
        };
        let _ = writeln!(stdout, "{resp}");
        let _ = stdout.flush();
    }
}
