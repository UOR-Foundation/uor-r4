use clap::Parser;
use uor_r4_wasm_router::chat::ChatEngine;
use uor_r4_wasm_router::model::default_model_reference;

/// Ask a question using the local transformerless library directly.
#[derive(Debug, Parser)]
#[command(name = "ask", version, about, long_about = None)]
struct Cli {
    /// Increase log verbosity (-v info, -vv debug, -vvv trace).
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// CID-addressed chat model name or blake3 CID.
    #[arg(long, env = "TLESS_MODEL")]
    model: Option<String>,

    /// Question to ask. Multiple unquoted words are accepted.
    #[arg(required = true, num_args = 1..)]
    question: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    uor_r4_wasm_router::telemetry::init(cli.verbose);
    let question = cli.question.join(" ");
    let model = cli.model.unwrap_or_else(default_model_reference);
    eprintln!("ask: loading local transformerless model '{model}'...");

    let result = ChatEngine::builder()
        .model(model)
        .build()
        .and_then(|mut chat| {
            eprintln!("ask: generating local answer...");
            chat.ask(&question)
        });
    match result {
        Ok(answer) => println!("{}", answer.text),
        Err(error) => {
            tracing::error!(%error, "ask failed");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn help_definition_is_valid() {
        Cli::command().debug_assert();
        let help = Cli::command().render_long_help().to_string();
        assert!(help.contains("Usage:"));
        assert!(help.contains("QUESTION"));
        assert!(help.contains("--model"));
    }

    #[test]
    fn parses_question_and_paths() {
        let cli = Cli::try_parse_from(["ask", "hello"]).unwrap();
        assert!(cli.model.is_none());

        let cli = Cli::try_parse_from([
            "ask", "--model", "qa-model", "why", "is", "the", "sky", "blue?",
        ])
        .unwrap();
        assert_eq!(cli.question.join(" "), "why is the sky blue?");
        assert_eq!(cli.model.as_deref(), Some("qa-model"));

        let cli = Cli::try_parse_from(["ask", "--model", "qa-model", "-vv", "hello"]).unwrap();
        assert_eq!(cli.verbose, 2);
    }
}
