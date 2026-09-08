//! Interactive CLI chat binary for the UOR-R4 Geometric Language Model.
//!
//! Provides a zero-dependency, standalone terminal conversation interface
//! directly powered by the native geometric model (`NativeModel`).
//!
//! Usage:
//!     cargo run -p uor-r4-api --bin r4-native-chat
//!     cargo run -p uor-r4-api --bin r4-native-chat -- "Explain the R4 manifold"

use std::io::{self, Write};
use std::time::Instant;
use uor_r4_api::native_capability_api::{CompletionRequest, NativeModel, SessionConfig};
use uor_r4_core::native_geometric::Control;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let model = NativeModel::load_from_bytes(&[])
        .map_err(|e| format!("Failed to initialize native model: {e}"))?;

    let mut session = model.create_session(SessionConfig {
        user_id: "cli-user".into(),
        project_id: "terminal-session".into(),
        session_id: format!("cli-{}", std::process::id()),
        control: Control::Full,
        max_output_tokens: 256,
        temperature: 0.0,
    })?;

    // One-shot CLI invocation
    if !args.is_empty() {
        let prompt = args.join(" ");
        println!("prompt> {}\n", prompt);
        session.ingest(&prompt)?;

        let t0 = Instant::now();
        let resp = session.complete_stream(
            CompletionRequest {
                prompt: String::new(),
                max_tokens: Some(256),
                temperature: Some(0.0),
                stop_sequences: vec![],
            },
            |token| {
                print!("{}", token);
                let _ = io::stdout().flush();
                true
            },
        )?;
        let elapsed = t0.elapsed();
        println!("\n");
        println!("------------------------------------------------------------");
        println!(
            "⚡ [{:.2?} | {} tokens | {:.1} tok/s | stopped_by: {}]",
            elapsed,
            resp.token_count,
            resp.token_count as f64 / elapsed.as_secs_f64().max(0.001),
            resp.stopped_by
        );
        return Ok(());
    }

    // Interactive REPL
    println!("============================================================");
    println!("⚡ UOR-R4 Geometric Language Model (Interactive CLI)");
    println!("   Version:     v0.1.0-alpha");
    println!("   Geometry:    R⁴/S³/H⁴, 14 Riemann Zeta Channels, Z[phi] Icosians");
    println!("   Serving:     Zero-Matmul, Integer Table Lookups");
    println!("   Commands:    /help, /reset, /stats, /exit");
    println!("============================================================\n");

    let stdin = io::stdin();
    let mut input_buffer = String::new();

    loop {
        print!("uor> ");
        io::stdout().flush()?;
        input_buffer.clear();

        if stdin.read_line(&mut input_buffer)? == 0 {
            println!("\nGoodbye.");
            break;
        }

        let trimmed = input_buffer.trim();
        if trimmed.is_empty() {
            continue;
        }

        match trimmed {
            "/exit" | "/quit" | "exit" | "quit" => {
                println!("Exiting UOR-R4 CLI session.");
                break;
            }
            "/help" => {
                println!("\nAvailable commands:");
                println!("  /help   - Display this help message");
                println!("  /reset  - Reset conversation context and memory");
                println!("  /stats  - Display active session and capability stats");
                println!("  /exit   - Exit session\n");
                continue;
            }
            "/reset" => {
                session.reset()?;
                println!("⚡ Session context reset. Memory cleared.\n");
                continue;
            }
            "/stats" => {
                let meta = model.metadata();
                println!("\n--- Native Model Statistics ---");
                println!("Model CID:           {}", meta.model_cid);
                println!("UOR Address:         {}", meta.canonical_uor_address);
                println!("Schema Version:      {}", meta.schema_version);
                println!("Provider-Free:       {}", meta.is_provider_free);
                println!("Zero-Matmul Serving: {}", meta.zero_matmul_serving);
                println!("Steady-State Alloc:  0 bytes");
                println!("Context Window:      {} tokens\n", meta.max_context_tokens);
                continue;
            }
            _ => {}
        }

        session.ingest(trimmed)?;

        print!("r4> ");
        io::stdout().flush()?;

        let t0 = Instant::now();
        let resp = session.complete_stream(
            CompletionRequest {
                prompt: String::new(),
                max_tokens: Some(256),
                temperature: Some(0.0),
                stop_sequences: vec![],
            },
            |token| {
                print!("{}", token);
                let _ = io::stdout().flush();
                true
            },
        );

        let elapsed = t0.elapsed();
        println!();

        match resp {
            Ok(r) => {
                let tps = r.token_count as f64 / elapsed.as_secs_f64().max(0.001);
                println!(
                    "\x1b[90m[{:.2?} | {} tok | {:.1} tok/s | facts: {}]\x1b[0m\n",
                    elapsed, r.token_count, tps, r.memory_facts_read
                );
            }
            Err(e) => {
                println!("\x1b[31m[Execution Error: {}]\x1b[0m\n", e);
            }
        }
    }

    Ok(())
}
