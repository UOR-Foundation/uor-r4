use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_HEADER_BYTES: usize = 64 * 1024;
const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
const IO_TIMEOUT: Duration = Duration::from_millis(100);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeProxyResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeChatEvent {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body_chunk: Vec<u8>,
    pub terminal: bool,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct SidecarProcess {
    port: u16,
    token: String,
    child: Arc<Mutex<Child>>,
    active: Arc<Mutex<BTreeMap<String, Arc<AtomicBool>>>>,
}

impl SidecarProcess {
    pub fn start(binary: &Path, stack: &Path, tokenizer: &Path) -> Result<Self, String> {
        for path in [binary, stack, tokenizer] {
            if !path.is_file() {
                return Err(format!("required sidecar input is unavailable: {}", path.display()));
            }
        }
        let token = generate_token();
        let mut child = Command::new(binary)
            .arg("--stack")
            .arg(stack)
            .arg("--tokenizer")
            .arg(tokenizer)
            .arg("--serve")
            .arg("127.0.0.1:0")
            .arg("--token-stdin")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("could not launch uor-chat: {error}"))?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| "uor-chat stdin was unavailable".to_string())?;
        writeln!(stdin, "{token}").map_err(|error| format!("could not pass token to uor-chat: {error}"))?;
        drop(stdin);
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "uor-chat stdout was unavailable".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "uor-chat stderr was unavailable".to_string())?;
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut sink = std::io::sink();
            let _ = std::io::copy(&mut reader, &mut sink);
        });
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        let deadline = Instant::now() + STARTUP_TIMEOUT;
        let mut port = None;
        while port.is_none() {
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Err("uor-chat did not report its loopback port before timeout".into());
            }
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(line) => {
                    if let Some(address) = line.strip_prefix("uor-chat OpenAI loopback listening on http://127.0.0.1:") {
                        let parsed = address
                            .split_whitespace()
                            .next()
                            .unwrap_or_default()
                            .parse::<u16>()
                            .map_err(|_| format!("invalid uor-chat port in {line}"))?;
                        port = Some(parsed);
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err("uor-chat exited before reporting its port".into());
                }
            }
        }
        let sidecar = Self {
            port: port.ok_or_else(|| "uor-chat did not report a port".to_string())?,
            token,
            child: Arc::new(Mutex::new(child)),
            active: Arc::new(Mutex::new(BTreeMap::new())),
        };
        let readiness_deadline = Instant::now() + STARTUP_TIMEOUT;
        loop {
            match sidecar.models() {
                Ok(_) => return Ok(sidecar),
                Err(error) => {
                    if Instant::now() >= readiness_deadline {
                        let _ = sidecar.child.lock().map_err(|_| "sidecar mutex poisoned".to_string())?.kill();
                        let _ = sidecar.child.lock().map_err(|_| "sidecar mutex poisoned".to_string())?.wait();
                        return Err(format!("uor-chat was not ready: {error}"));
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn models(&self) -> Result<NativeProxyResponse, String> {
        let (reader, status, headers) = self.request("GET", "/v1/models", &[], "application/json")?;
        let body = read_bounded(reader)?;
        Ok(NativeProxyResponse { status, headers, body })
    }

    pub fn decompose(
        &self,
        width: u32,
        height: u32,
        columns: u32,
        rows: u32,
        rgba: &[u8],
    ) -> Result<NativeProxyResponse, String> {
        if rgba.len() != width as usize * height as usize * 4 {
            return Err("RGBA byte length does not match dimensions".into());
        }
        let path = format!("/v1/images/decompose?width={width}&height={height}&columns={columns}&rows={rows}");
        let (reader, status, headers) = self.request("POST", &path, rgba, "application/octet-stream")?;
        let body = read_bounded(reader)?;
        Ok(NativeProxyResponse { status, headers, body })
    }

    pub fn stream_chat<F>(&self, request_id: &str, body: &str, emit: F) -> Result<(), String>
    where
        F: FnMut(NativeChatEvent),
    {
        if request_id.trim().is_empty() {
            return Err("request ID is required".into());
        }
        let cancel = self
            .active
            .lock()
            .map_err(|_| "request registry mutex poisoned".to_string())?
            .get(request_id)
            .cloned()
            .unwrap_or_else(|| Arc::new(AtomicBool::new(false)));
        self.active
            .lock()
            .map_err(|_| "request registry mutex poisoned".to_string())?
            .insert(request_id.to_string(), cancel.clone());
        let result = self.stream_chat_inner(request_id, body, cancel, emit);
        self.active
            .lock()
            .map_err(|_| "request registry mutex poisoned".to_string())?
            .remove(request_id);
        result
    }

    fn stream_chat_inner<F>(
        &self,
        request_id: &str,
        body: &str,
        cancel: Arc<AtomicBool>,
        mut emit: F,
    ) -> Result<(), String>
    where
        F: FnMut(NativeChatEvent),
    {
        let (reader, status, headers) = self.request("POST", "/v1/chat/completions", body.as_bytes(), "application/json")?;
        let is_sse = headers
            .get("content-type")
            .is_some_and(|value| value.to_ascii_lowercase().contains("text/event-stream"));
        if !is_sse {
            let body_chunk = read_bounded(reader)?;
            emit(NativeChatEvent {
                status,
                headers,
                body_chunk,
                terminal: true,
                error: None,
            });
            return Ok(());
        }
        let mut reader = reader;
        let mut block = Vec::new();
        loop {
            if cancel.load(Ordering::Relaxed) {
                let _ = reader.get_ref().shutdown(Shutdown::Both);
                emit(NativeChatEvent {
                    status,
                    headers,
                    body_chunk: Vec::new(),
                    terminal: true,
                    error: Some(format!("request {request_id} cancelled")),
                });
                return Ok(());
            }
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if line == "\r\n" || line == "\n" {
                        if !block.is_empty() {
                            block.extend_from_slice(line.as_bytes());
                            emit(NativeChatEvent {
                                status,
                                headers: headers.clone(),
                                body_chunk: std::mem::take(&mut block),
                                terminal: false,
                                error: None,
                            });
                        }
                    } else {
                        block.extend_from_slice(line.as_bytes());
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(error) => return Err(format!("sidecar stream failed: {error}")),
            }
        }
        if !block.is_empty() {
            emit(NativeChatEvent {
                status,
                headers: headers.clone(),
                body_chunk: block,
                terminal: false,
                error: None,
            });
        }
        emit(NativeChatEvent {
            status,
            headers,
            body_chunk: Vec::new(),
            terminal: true,
            error: None,
        });
        Ok(())
    }

    pub fn cancel(&self, request_id: &str) -> Result<(), String> {
        if request_id.trim().is_empty() {
            return Err("request ID is required for cancellation".into());
        }
        self.active
            .lock()
            .map_err(|_| "request registry mutex poisoned".to_string())?
            .entry(request_id.to_string())
            .or_insert_with(|| Arc::new(AtomicBool::new(false)))
            .store(true, Ordering::Relaxed);
        Ok(())
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        body: &[u8],
        content_type: &str,
    ) -> Result<(BufReader<TcpStream>, u16, BTreeMap<String, String>), String> {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port))
            .map_err(|error| format!("could not connect to uor-chat: {error}"))?;
        stream
            .set_read_timeout(Some(IO_TIMEOUT))
            .map_err(|error| error.to_string())?;
        stream
            .set_write_timeout(Some(IO_TIMEOUT))
            .map_err(|error| error.to_string())?;
        let head = format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {token}\r\nContent-Type: {content_type}\r\nAccept: application/json, text/event-stream, application/octet-stream\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n",
            port = self.port,
            token = self.token,
            length = body.len()
        );
        stream
            .write_all(head.as_bytes())
            .and_then(|_| stream.write_all(body))
            .map_err(|error| format!("could not write sidecar request: {error}"))?;
        let mut reader = BufReader::new(stream);
        let status_line = read_header_line(&mut reader)?;
        let status = status_line
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| format!("invalid sidecar status line: {status_line}"))?
            .parse::<u16>()
            .map_err(|_| format!("invalid sidecar status: {status_line}"))?;
        let mut headers = BTreeMap::new();
        let mut consumed = status_line.len();
        loop {
            let line = read_header_line(&mut reader)?;
            consumed += line.len();
            if consumed > MAX_HEADER_BYTES {
                return Err("sidecar headers exceed 64 KiB".into());
            }
            if line == "\r\n" || line == "\n" {
                break;
            }
            if let Some((name, value)) = line.split_once(':') {
                headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
            }
        }
        Ok((reader, status, headers))
    }
}

impl Drop for SidecarProcess {
    fn drop(&mut self) {
        if Arc::strong_count(&self.child) == 1 {
            if let Ok(mut child) = self.child.lock() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

fn read_header_line(reader: &mut impl BufRead) -> Result<String, String> {
    let mut line = String::new();
    let read = reader
        .read_line(&mut line)
        .map_err(|error| format!("could not read sidecar headers: {error}"))?;
    if read == 0 {
        return Err("sidecar closed before completing headers".into());
    }
    if line.len() > MAX_HEADER_BYTES {
        return Err("sidecar header line exceeds 64 KiB".into());
    }
    Ok(line)
}

fn read_bounded(reader: impl Read) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take((MAX_RESPONSE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("could not read sidecar response: {error}"))?;
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err("sidecar response exceeds 1 MiB".into());
    }
    Ok(bytes)
}

fn generate_token() -> String {
    use sha2::{Digest, Sha256};
    let mut seed = [0u8; 32];
    let mut filled = false;
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        filled = file.read_exact(&mut seed).is_ok();
    }
    let mut hasher = Sha256::new();
    hasher.update(b"UOR-TAURI-SIDECAR-TOKEN-V1");
    hasher.update(seed);
    hasher.update(std::process::id().to_le_bytes());
    if let Ok(duration) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        hasher.update(duration.as_nanos().to_le_bytes());
    }
    if !filled {
        hasher.update(b"fallback-entropy");
    }
    hex::encode(hasher.finalize())
}
