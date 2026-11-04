use std::net::TcpStream;
use std::path::PathBuf;

use uor_r4_desktop_client::sidecar::{NativeChatEvent, SidecarProcess};

fn paths() -> (PathBuf, PathBuf, PathBuf) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    (
        root.join("binaries/uor-chat-aarch64-apple-darwin"),
        root.join("runtime/studio/pkg/synthetic-stack.lut"),
        root.join("runtime/studio/pkg/tokenizer.json"),
    )
}

fn collect_chat(sidecar: &SidecarProcess, request_id: &str, request: &str) -> Result<Vec<NativeChatEvent>, String> {
    let mut events = Vec::new();
    sidecar.stream_chat(request_id, request, |event| events.push(event))?;
    Ok(events)
}

#[test]
fn sidecar_proxy_preserves_native_openai_surface_and_lifecycle() {
    let (binary, stack, tokenizer) = paths();
    let sidecar = SidecarProcess::start(&binary, &stack, &tokenizer).expect("start pinned uor-chat");
    let models = sidecar.models().expect("authenticated models");
    assert_eq!(models.status, 200);
    assert!(String::from_utf8_lossy(&models.body).contains("uor-chat"));

    let image = sidecar
        .decompose(1, 1, 1, 1, &[255, 0, 0, 255])
        .expect("decompose one pixel");
    assert_eq!(image.status, 200);
    assert_eq!(image.headers.get("x-uor-image-schema").map(String::as_str), Some("uor-r4.image-decomposition/1"));
    assert!(image.body.starts_with(b"R4IMG001"));

    let request = r#"{"model":"uor-chat","messages":[{"role":"user","content":"Hi"}],"stream":true,"stream_options":{"include_usage":true},"temperature":0,"max_tokens":16}"#;
    let events = collect_chat(&sidecar, "stream-1", request).expect("SSE chat");
    let wire = events
        .iter()
        .map(|event| String::from_utf8_lossy(&event.body_chunk).into_owned())
        .collect::<String>();
    assert!(wire.contains("data:"), "expected SSE data events");
    assert!(wire.contains("\r\n\r\n"), "SSE event delimiters must survive the proxy: {wire:?}");
    assert!(wire.contains("19/16/35") || wire.contains("\"total_tokens\":35"), "expected exact usage in {wire}");
    assert!(events.iter().any(|event| event.terminal));

    let json_request = r#"{"model":"uor-chat","messages":[{"role":"user","content":"Hi"}],"stream":false,"temperature":0,"max_tokens":1}"#;
    let json = collect_chat(&sidecar, "json-1", json_request).expect("JSON chat");
    let json_wire = json
        .iter()
        .map(|event| String::from_utf8_lossy(&event.body_chunk).into_owned())
        .collect::<String>();
    assert!(json_wire.contains("\"prompt_tokens\":19") && json_wire.contains("\"completion_tokens\":1") && json_wire.contains("\"total_tokens\":20"), "{json_wire}");
    assert_eq!(json.last().map(|event| event.status), Some(200));

    let long_prompt = format!(r#"{{"model":"uor-chat","messages":[{{"role":"user","content":"{}"}}],"stream":false,"max_tokens":16}}"#, "a".repeat(128));
    let refused = collect_chat(&sidecar, "context-1", &long_prompt).expect("context refusal");
    assert_eq!(refused.last().map(|event| event.status), Some(413));

    let port = sidecar.port();
    drop(sidecar);
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert!(TcpStream::connect(("127.0.0.1", port)).is_err(), "sidecar must stop on drop");
}

#[test]
fn sidecar_cancellation_is_explicit() {
    let (binary, stack, tokenizer) = paths();
    let sidecar = SidecarProcess::start(&binary, &stack, &tokenizer).expect("start pinned uor-chat");
    sidecar.cancel("cancel-me").expect("register cancellation");
    let request = r#"{"model":"uor-chat","messages":[{"role":"user","content":"Hi"}],"stream":true,"max_tokens":8}"#;
    let events = collect_chat(&sidecar, "cancel-me", request).expect("cancelled stream");
    assert!(events.iter().any(|event| event.error.as_deref().is_some_and(|message| message.contains("cancelled"))));
}

#[test]
fn sidecar_emits_sse_before_the_stream_completes() {
    let (binary, stack, tokenizer) = paths();
    let sidecar = SidecarProcess::start(&binary, &stack, &tokenizer).expect("start pinned uor-chat");
    let request = r#"{"model":"uor-chat","messages":[{"role":"user","content":"Hi"}],"stream":true,"max_tokens":16}"#;
    let mut events = Vec::new();
    let mut cancelled_on_first_chunk = false;
    sidecar.stream_chat("incremental-1", request, |event| {
        if !cancelled_on_first_chunk && !event.body_chunk.is_empty() {
            sidecar.cancel("incremental-1").expect("cancel from first live chunk");
            cancelled_on_first_chunk = true;
        }
        events.push(event);
    }).expect("incremental stream");
    assert!(cancelled_on_first_chunk, "no live body event was emitted");
    assert!(events.iter().any(|event| event.error.as_deref().is_some_and(|message| message.contains("cancelled"))), "cancellation took effect only after buffering: {events:?}");
}

#[cfg(feature = "gui")]
#[test]
fn tauri_state_owns_one_live_sidecar() {
    use uor_r4_desktop_client::tauri_gui::TauriDesktopState;
    let (binary, stack, tokenizer) = paths();
    let state = TauriDesktopState::with_sidecar(&binary, &stack, &tokenizer).expect("state with sidecar");
    let first = state.sidecar().expect("first sidecar");
    let second = state.sidecar().expect("same sidecar");
    assert_eq!(first.port(), second.port());
    assert_eq!(first.models().expect("models through owned sidecar").status, 200);
}
