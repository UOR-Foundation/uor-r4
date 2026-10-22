//! Native desktop client CLI / JSON-RPC & loopback Studio host runner (`uor-desktop-client`).
//!
//! Supports:
//! - `uor-desktop-client --self-test`: runs an end-to-end verification of the synthetic stand-in
//!   and durable memory store and prints a JSON receipt.
//! - `uor-desktop-client --serve-studio <127.0.0.1:port>`: serves `web/studio/` over loopback HTTP
//!   and answers `POST /__tauri_invoke` (`window.__TAURI__.core.invoke`) against [`DesktopBackend`].
//! - `uor-desktop-client [--stdio]`: runs a newline-delimited JSON-RPC loop driving [`DesktopBackend`].
#![forbid(unsafe_code)]

use std::io::{self, BufRead, Read, Write};
use std::net::TcpListener;
use std::path::Path;

use uor_r4_desktop_client::{
    handle_rpc_line, handle_studio_http_request, run_self_test_receipt, DesktopBackend, PinRegistry,
};

fn run_serve_studio(addr: &str, studio_root: &Path) -> Result<(), String> {
    let listener = TcpListener::bind(addr).map_err(|e| format!("bind {addr}: {e}"))?;
    let local_addr = listener.local_addr().map_err(|e| e.to_string())?;
    println!("UOR-R4 Desktop Studio loopback host listening on http://{local_addr}/");
    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let mut buf = [0u8; 65536];
        let Ok(n) = stream.read(&mut buf) else {
            continue;
        };
        if n == 0 {
            continue;
        }
        let raw = &buf[..n];
        let header_end = raw
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .map(|p| p + 4)
            .unwrap_or(n);
        let head_str = String::from_utf8_lossy(&raw[..header_end]);
        let mut lines = head_str.lines();
        let req_line = lines.next().unwrap_or("GET / HTTP/1.1");
        let mut parts = req_line.split_whitespace();
        let method = parts.next().unwrap_or("GET");
        let path = parts.next().unwrap_or("/");
        let body = &raw[header_end..];
        let (status, content_type, resp_body) =
            handle_studio_http_request(&mut backend, studio_root, method, path, body);
        let status_text = match status {
            200 => "OK",
            403 => "Forbidden",
            404 => "Not Found",
            _ => "Method Not Allowed",
        };
        let header = format!(
            "HTTP/1.1 {status} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            resp_body.len()
        );
        let _ = stream.write_all(header.as_bytes());
        let _ = stream.write_all(&resp_body);
        let _ = stream.flush();
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--self-test") {
        match run_self_test_receipt() {
            Ok(receipt) => println!("{receipt}"),
            Err(err) => {
                eprintln!("self-test error: {err}");
                std::process::exit(1);
            }
        }
        return;
    }

    if let Some(pos) = args.iter().position(|a| a == "--serve-studio") {
        let addr = args
            .get(pos + 1)
            .map(String::as_str)
            .unwrap_or("127.0.0.1:1420");
        let studio_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../web/studio");
        if let Err(err) = run_serve_studio(addr, &studio_root) {
            eprintln!("serve-studio error: {err}");
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
        let resp = handle_rpc_line(&mut backend, &line);
        let _ = writeln!(stdout, "{resp}");
        let _ = stdout.flush();
    }
}
