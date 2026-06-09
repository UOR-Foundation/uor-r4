use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

fn main() {
    // Attempt to run wasm-pack build --target web programmatically to verify
    // or compile changes. If it is not found, continue.
    println!("Checking for wasm-pack build update...");
    match std::process::Command::new("wasm-pack")
        .args(["build", "--target", "web"])
        .status()
    {
        Ok(status) => {
            if status.success() {
                println!("Successfully compiled/updated WebAssembly components.");
            } else {
                println!("Warning: wasm-pack failed to build, serving existing package files.");
            }
        }
        Err(_) => {
            println!("Note: wasm-pack command not found on local path. Serving precompiled files.");
        }
    }

    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    println!("Local server running at http://127.0.0.1:8000/");
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            handle_connection(stream);
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = match buf_reader.lines().next() {
        Some(Ok(line)) => line,
        _ => return,
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let path_str = parts[1];
    let clean_path = path_str.split('?').next().unwrap().split('#').next().unwrap();
    
    let mut relative_path = clean_path.trim_start_matches('/');
    if relative_path.is_empty() {
        relative_path = "index.html";
    }

    let file_path = Path::new(relative_path);
    if !file_path.exists() || file_path.is_dir() {
        let response = "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 0\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    let contents = match fs::read(file_path) {
        Ok(c) => c,
        Err(_) => {
            let response = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\nContent-Length: 0\r\n\r\n";
            let _ = stream.write_all(response.as_bytes());
            return;
        }
    };

    let mime_type = match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        mime_type,
        contents.len()
    );

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.write_all(&contents);
}
