//! Terminal adapter for an explicitly supplied native artifact.
use std::io::{self, Write};
use uor_r4_api::native_capability_api::{CompletionRequest, NativeModel, SessionConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut model_path = std::env::var("UOR_R4_MODEL").ok();
    let mut prompt = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--model" => model_path = Some(args.next().ok_or("--model requires a path")?),
            "--help" | "-h" => {
                println!("r4-native-chat --model PATH [PROMPT]\nUOR_R4_MODEL can supply PATH. No default model or fallback replies.\nInteractive commands: /reset, /stats, /exit");
                return Ok(());
            }
            "--" => {
                prompt.extend(args);
                break;
            }
            _ if arg.starts_with("--") => return Err(format!("unknown option: {arg}").into()),
            _ => prompt.push(arg),
        }
    }
    let path = model_path
        .ok_or("supply --model PATH or UOR_R4_MODEL; serving requires a trained artifact")?;
    let model = NativeModel::load_from_bytes(&std::fs::read(&path)?)?;
    let mut session = model.create_session(SessionConfig {
        user_id: "cli-user".into(),
        project_id: "terminal".into(),
        session_id: format!("cli-{}", std::process::id()),
        ..Default::default()
    })?;
    eprintln!(
        "UOR-R4 pre-alpha | artifact {} | context {}",
        model.artifact_cid(),
        model.metadata().max_context_tokens
    );
    if !prompt.is_empty() {
        let response =
            session.complete_stream(CompletionRequest::new(prompt.join(" ")), |piece| {
                print!("{piece}");
                let _ = io::stdout().flush();
                true
            })?;
        eprintln!(
            "\n[{} model tokens; {}]",
            response.token_count, response.stopped_by
        );
        return Ok(());
    }
    let mut line = String::new();
    loop {
        print!("uor> ");
        io::stdout().flush()?;
        line.clear();
        if io::stdin().read_line(&mut line)? == 0 {
            break;
        }
        match line.trim() {
            "/exit" | "/quit" => break,
            "/reset" => {
                session.reset()?;
                eprintln!("Context and memory cleared.");
                continue;
            }
            "/stats" => {
                println!("{}", serde_json::to_string_pretty(model.metadata())?);
                continue;
            }
            "" => continue,
            _ => {}
        }
        // Preserve the newline separating successive user turns.
        let result = session.complete_stream(CompletionRequest::new(line.clone()), |piece| {
            print!("{piece}");
            let _ = io::stdout().flush();
            true
        });
        match result {
            Ok(response) => eprintln!(
                "\n[{} model tokens; {}]",
                response.token_count, response.stopped_by
            ),
            Err(error) => eprintln!("\n[error: {error}]"),
        }
    }
    Ok(())
}
