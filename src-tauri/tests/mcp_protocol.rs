#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};
use uor_r4_desktop_client::mcp::{McpLimits, McpManager, McpRemoteSession, McpStdioSession};

#[derive(Clone, Default)]
struct FixtureOptions {
    sse: bool,
    delay_ms: u64,
    large: bool,
}

#[derive(Debug)]
struct RecordedRequest {
    method: String,
    headers: BTreeMap<String, String>,
    body: Value,
}

struct HttpFixture {
    url: String,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl HttpFixture {
    fn start(options: FixtureOptions) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind HTTP fixture");
        listener.set_nonblocking(true).expect("nonblocking fixture");
        let address = listener.local_addr().expect("fixture address");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let recorded = requests.clone();
        let stopping = stop.clone();
        let thread = std::thread::spawn(move || {
            while !stopping.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream.set_read_timeout(Some(Duration::from_secs(1))).expect("fixture read timeout");
                        let mut raw = Vec::new();
                        let mut buffer = [0u8; 4096];
                        let mut header_end = None;
                        let mut content_length = 0usize;
                        for _ in 0..128 {
                            let Ok(read) = stream.read(&mut buffer) else { break };
                            if read == 0 { break; }
                            raw.extend_from_slice(&buffer[..read]);
                            if let Some(position) = raw.windows(4).position(|window| window == b"\r\n\r\n") {
                                header_end = Some(position + 4);
                                let head = String::from_utf8_lossy(&raw[..position]);
                                content_length = head.lines().find_map(|line| {
                                    let (name, value) = line.split_once(':')?;
                                    name.eq_ignore_ascii_case("content-length").then(|| value.trim().parse().ok()).flatten()
                                }).unwrap_or(0);
                                if raw.len() >= position + 4 + content_length { break; }
                            }
                        }
                        let split = header_end.unwrap_or(raw.len());
                        let head = String::from_utf8_lossy(&raw[..split]).into_owned();
                        let body = &raw[split..split + content_length.min(raw.len().saturating_sub(split))];
                        let method = head.split_whitespace().next().unwrap_or("").to_string();
                        if method == "OPTIONS" {
                            let response = b"HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: http://127.0.0.1:8765\r\nAccess-Control-Allow-Headers: Content-Type, Accept, Authorization, Mcp-Session-Id, MCP-Protocol-Version\r\nAccess-Control-Allow-Methods: POST, OPTIONS\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                            stream.write_all(response).expect("write preflight");
                            continue;
                        }
                        let message: Value = serde_json::from_slice(body).unwrap_or(Value::Null);
                        let headers = head.lines().filter_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            Some((name.trim().to_ascii_lowercase(), value.trim().to_string()))
                        }).collect::<BTreeMap<_, _>>();
                        recorded.lock().expect("record request").push(RecordedRequest { method, headers: headers.clone(), body: message.clone() });
                        if options.delay_ms > 0 { std::thread::sleep(Duration::from_millis(options.delay_ms)); }
                        if headers.get("authorization").map(String::as_str) != Some("Bearer secret") {
                            write_json(&mut stream, 401, "application/json", b"{\"error\":{\"message\":\"bad credential\"}}", false);
                            continue;
                        }
                        if message["method"] == "notifications/initialized" {
                            stream.write_all(b"HTTP/1.1 202 Accepted\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").expect("write initialized");
                            continue;
                        }
                        let id = message["id"].as_u64().unwrap_or(0);
                        let result = match message["method"].as_str().unwrap_or("") {
                            "initialize" => json!({"protocolVersion":"2025-03-26","capabilities":{"tools":{}},"serverInfo":{"name":"fixture","version":"1"}}),
                            "tools/list" if options.large => json!({"tools":[{"name":"x".repeat(4096)}]}),
                            "tools/list" => json!({"tools":[{"name":"read_file","description":"json/sse","inputSchema":{"type":"object"}}]}),
                            "tools/call" => json!({"content":[{"type":"text","text":"remote result"}],"isError":false}),
                            _ => json!({"code":-32601,"message":"unknown"}),
                        };
                        let payload = if message["method"] == "unknown" {
                            json!({"jsonrpc":"2.0","id":id,"error":result})
                        } else {
                            json!({"jsonrpc":"2.0","id":id,"result":result})
                        };
                        let text = payload.to_string();
                        write_json(&mut stream, 200, if options.sse { "text/event-stream" } else { "application/json" }, text.as_bytes(), options.sse);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(5)),
                    Err(_) => break,
                }
            }
        });
        Self { url: format!("http://{address}/mcp"), requests, stop, thread: Some(thread) }
    }
}

fn write_json(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8], sse: bool) {
    let body = if sse {
        let text = String::from_utf8_lossy(body);
        format!("data: {text}\r\n\r\n").into_bytes()
    } else {
        body.to_vec()
    };
    let head = format!("HTTP/1.1 {status} OK\r\nContent-Type: {content_type}\r\nMcp-Session-Id: session-test\r\nAccess-Control-Allow-Origin: http://127.0.0.1:8765\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
    stream.write_all(head.as_bytes()).expect("write response head");
    stream.write_all(&body).expect("write response body");
}

impl Drop for HttpFixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() { let _ = thread.join(); }
    }
}

fn limits() -> McpLimits {
    McpLimits::default()
}

fn temp_dir(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("uor-mcp-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create MCP test dir");
    path
}

fn compile_stdio_fixture(root: &Path) -> PathBuf {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mcp_stdio_fixture.rs");
    let output = root.join("mcp-stdio-fixture");
    let status = Command::new("rustc").arg("--edition=2021").arg(source).arg("-o").arg(&output).status().expect("compile stdio fixture");
    assert!(status.success());
    output
}

#[test]
fn remote_json_and_sse_share_session_protocol_and_cors() {
    for sse in [false, true] {
        let fixture = HttpFixture::start(FixtureOptions { sse, ..FixtureOptions::default() });
        let session = McpRemoteSession::connect(&fixture.url, "secret", limits()).expect("connect remote");
        assert_eq!(session.list_tools().expect("list tools")[0]["name"], "read_file");
        assert_eq!(session.call_tool("read_file", &json!({})).expect("call tool")["content"][0]["text"], "remote result");
        let requests = fixture.requests.lock().expect("read requests");
        assert_eq!(requests.len(), 4);
        assert_eq!(requests[0].method, "POST");
        assert_eq!(requests[0].body["method"], "initialize");
        assert_eq!(requests[1].body["method"], "notifications/initialized");
        assert_eq!(requests[0].headers.get("mcp-protocol-version").map(String::as_str), Some("2025-03-26"));
        assert_eq!(requests[2].headers.get("mcp-session-id").map(String::as_str), Some("session-test"));
        drop(requests);
        let preflight = reqwest::blocking::Client::new().request(reqwest::Method::OPTIONS, &fixture.url).header("Origin", "http://127.0.0.1:8765").send().expect("preflight");
        assert_eq!(preflight.status(), 204);
        assert_eq!(preflight.headers().get("access-control-allow-origin").and_then(|value| value.to_str().ok()), Some("http://127.0.0.1:8765"));
    }
}

#[test]
fn remote_auth_timeout_and_size_caps_fail_explicitly() {
    let unauthorized = HttpFixture::start(FixtureOptions::default());
    let error = McpRemoteSession::connect(&unauthorized.url, "wrong", limits()).expect_err("bad auth must fail");
    assert!(error.contains("401"), "{error}");
    drop(unauthorized);

    let slow = HttpFixture::start(FixtureOptions { delay_ms: 80, ..FixtureOptions::default() });
    let short = McpLimits { timeout: Duration::from_millis(15), ..McpLimits::default() };
    let error = McpRemoteSession::connect(&slow.url, "secret", short).expect_err("timeout must fail");
    assert!(error.contains("timed out"), "{error}");
    drop(slow);

    let large = HttpFixture::start(FixtureOptions { large: true, ..FixtureOptions::default() });
    let small = McpLimits { max_response_bytes: 1024, ..McpLimits::default() };
    let session = McpRemoteSession::connect(&large.url, "secret", small).expect("small initialize");
    let error = session.list_tools().expect_err("large list must fail");
    assert!(error.contains("exceeds"), "{error}");
}

#[test]
fn stdio_handles_approval_paths_frames_stderr_timeout_and_cleanup() {
    let root = temp_dir("stdio");
    let fixture = compile_stdio_fixture(&root);
    let cwd = root.join("cwd");
    fs::create_dir_all(&cwd).expect("create stdio cwd");
    let approved = cwd.join("mcp-stdio-fixture");
    fs::copy(&fixture, &approved).expect("copy approved fixture");
    let error = McpStdioSession::spawn(&fixture, &[], &cwd, limits()).expect_err("outside-cwd executable must fail");
    assert!(error.contains("outside"), "{error}");

    let session = McpStdioSession::spawn(&approved, &[], &cwd, limits()).expect("approved stdio");
    assert_eq!(session.list_tools().expect("stdio tools")[0]["name"], "read_file");
    assert_eq!(session.call_tool("read_file", &json!({})).expect("stdio call")["content"][0]["text"], "stdio result");
    let short = McpLimits { timeout: Duration::from_millis(25), ..McpLimits::default() };
    let hanging = McpStdioSession::spawn(&approved, &[], &cwd, short).expect("hanging stdio");
    let error = hanging.call_tool("hang", &json!({})).expect_err("hanging call must time out");
    assert!(error.contains("timed out"), "{error}");
    assert!(!hanging.is_running(), "timed-out child must be killed and reaped");
    drop(session);
    drop(hanging);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn manager_bounds_process_count_and_reuses_matching_surface() {
    let root = temp_dir("manager");
    let fixture = compile_stdio_fixture(&root);
    let manager = McpManager::default();
    for _ in 0..4 {
        manager.connect_stdio(&fixture, &[], &root, limits()).expect("bounded stdio connection");
    }
    let error = manager.connect_stdio(&fixture, &[], &root, limits()).expect_err("fifth child must be refused");
    assert!(error.contains("cap"), "{error}");
    drop(manager);
    let _ = fs::remove_dir_all(&root);
}
