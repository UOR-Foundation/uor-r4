use clap::{Parser, Subcommand};
use std::fmt;
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use uor_r4_wasm_router::chat::{ChatAnswer, ChatEngine, ChatError};
use uor_r4_wasm_router::model::default_model_reference;
use uor_r4_wasm_router::server::{self, ServerConfig};
use uor_r4_wasm_router::tless_uor;

/// R⁴ Tangent Space Router server and local chat CLI.
#[derive(Parser, Debug, Clone)]
#[command(name = "server", version, about, long_about = None)]
struct Cli {
    /// Increase log verbosity (-v info, -vv debug, -vvv trace).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// CID-addressed chat model name or blake3 CID.
    #[arg(long, env = "TLESS_MODEL", global = true)]
    model: Option<String>,

    /// Host interface to bind.
    #[arg(long, env = "UOR_R4_HOST", default_value = "127.0.0.1", global = true)]
    host: String,

    /// Port to listen on.
    #[arg(long, env = "UOR_R4_PORT", default_value_t = 8000, global = true)]
    port: u16,

    /// Router manifold cache file.
    #[arg(
        long,
        env = "UOR_R4_MANIFOLD_CACHE",
        default_value = "manifold_cache_rust.json",
        global = true
    )]
    manifold_cache: String,

    /// TLA3 artifact container for the transformerless engine.
    #[arg(
        long,
        env = "TLESS_ARTIFACTS",
        default_value = "/tmp/tless_artifacts.bin",
        global = true
    )]
    tless_artifacts: String,

    /// TLS1 graded store for the transformerless engine.
    #[arg(
        long,
        env = "TLESS_STORE",
        default_value = "/tmp/tless_store.bin",
        global = true
    )]
    tless_store: String,

    /// llama2.c tokenizer used by transformerless generation.
    #[arg(
        long,
        env = "TLESS_TOKENIZER",
        default_value = "/tmp/ref/tokenizer.bin",
        global = true
    )]
    tless_tokenizer: String,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Run the HTTP server (the default).
    Serve,
    /// Ask one question using the transformerless library directly.
    Ask { question: String },
    /// Start an interactive, stateful local chat.
    Chat,
}

impl Cli {
    fn server_config(&self) -> ServerConfig {
        ServerConfig {
            host: self.host.clone(),
            port: self.port,
            manifold_cache: self.manifold_cache.clone(),
            tless_artifacts: self.tless_artifacts.clone(),
            tless_store: self.tless_store.clone(),
            tless_tokenizer: self.tless_tokenizer.clone(),
        }
    }

    fn configure_tless(&self) {
        tless_uor::configure_tless_paths(tless_uor::TlessPaths {
            artifacts: self.tless_artifacts.clone(),
            store: self.tless_store.clone(),
            tokenizer: self.tless_tokenizer.clone(),
        });
    }

    fn build_chat_engine(&self) -> Result<ChatEngine, ChatError> {
        ChatEngine::builder()
            .model(self.model.clone().unwrap_or_else(default_model_reference))
            .build()
    }
}

trait Chat {
    fn ask(&mut self, question: &str) -> Result<ChatAnswer, ChatError>;
}

impl Chat for ChatEngine {
    fn ask(&mut self, question: &str) -> Result<ChatAnswer, ChatError> {
        ChatEngine::ask(self, question)
    }
}

#[derive(Debug)]
enum RunError {
    Chat(ChatError),
    Io(io::Error),
}

impl fmt::Display for RunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Chat(error) => error.fmt(formatter),
            Self::Io(error) => error.fmt(formatter),
        }
    }
}

impl From<ChatError> for RunError {
    fn from(error: ChatError) -> Self {
        Self::Chat(error)
    }
}

impl From<io::Error> for RunError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

fn answer_once(
    chat: &mut impl Chat,
    question: &str,
    output: &mut impl Write,
) -> Result<(), RunError> {
    let answer = chat.ask(question)?;
    writeln!(output, "{}", answer.text)?;
    Ok(())
}

fn interactive_chat(
    chat: &mut impl Chat,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<(), io::Error> {
    writeln!(output, "R⁴ Router — interactive transformerless chat")?;
    writeln!(output, "type 'exit' or Ctrl-D to quit\n")?;
    loop {
        write!(output, "you> ")?;
        output.flush()?;
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            break;
        }
        let question = line.trim();
        if matches!(question, "exit" | "quit") {
            break;
        }
        if question.is_empty() {
            continue;
        }
        match chat.ask(question) {
            Ok(answer) => writeln!(output, "r4> {}\n", answer.text)?,
            Err(error) => tracing::error!(%error, "chat response failed"),
        }
    }
    Ok(())
}

fn run_ask(cli: &Cli, question: &str) -> Result<(), RunError> {
    let mut chat = cli.build_chat_engine()?;
    answer_once(&mut chat, question, &mut io::stdout().lock())
}

fn run_chat(cli: &Cli) -> Result<(), RunError> {
    let mut chat = cli.build_chat_engine()?;
    interactive_chat(&mut chat, &mut io::stdin().lock(), &mut io::stdout().lock())?;
    Ok(())
}

fn run(cli: &Cli) -> Result<(), RunError> {
    cli.configure_tless();
    match cli.command.as_ref() {
        Some(Command::Ask { question }) => run_ask(cli, question),
        Some(Command::Chat) => run_chat(cli),
        Some(Command::Serve) | None => {
            server::run_server(Arc::new(cli.server_config()));
            Ok(())
        }
    }
}

fn main() {
    let cli = Cli::parse();
    uor_r4_wasm_router::telemetry::init(cli.verbose);
    if let Err(error) = run(&cli) {
        tracing::error!(%error, "command failed");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use std::collections::VecDeque;

    struct FakeChat {
        answers: VecDeque<String>,
        questions: Vec<String>,
    }

    impl FakeChat {
        fn with_answers(answers: &[&str]) -> Self {
            Self {
                answers: answers.iter().map(ToString::to_string).collect(),
                questions: Vec::new(),
            }
        }
    }

    impl Chat for FakeChat {
        fn ask(&mut self, question: &str) -> Result<ChatAnswer, ChatError> {
            self.questions.push(question.to_owned());
            Ok(ChatAnswer {
                text: self.answers.pop_front().unwrap_or_default(),
                generated_tokens: 1,
            })
        }
    }

    #[test]
    fn help_definition_is_valid() {
        Cli::command().debug_assert();
        let help = Cli::command().render_long_help().to_string();
        assert!(help.contains("Usage:"));
        assert!(help.contains("serve"));
        assert!(help.contains("ask"));
        assert!(help.contains("chat"));
    }

    #[test]
    fn parses_defaults_flags_and_subcommands() {
        let cli = Cli::try_parse_from(["server"]).unwrap();
        assert_eq!(cli.host, "127.0.0.1");
        assert_eq!(cli.port, 8000);
        assert!(cli.model.is_none());
        assert!(cli.command.is_none());

        let cli = Cli::try_parse_from(["server", "ask", "hello world", "--port", "9001", "-vvv"])
            .unwrap();
        assert_eq!(cli.port, 9001);
        assert_eq!(cli.verbose, 3);
        assert!(matches!(cli.command, Some(Command::Ask { .. })));
    }

    #[test]
    fn one_shot_writes_only_the_answer() {
        let mut chat = FakeChat::with_answers(&["Because of Rayleigh scattering."]);
        let mut output = Vec::new();
        answer_once(&mut chat, "Why is the sky blue?", &mut output).unwrap();
        assert_eq!(chat.questions, ["Why is the sky blue?"]);
        assert_eq!(output, b"Because of Rayleigh scattering.\n");
    }

    #[test]
    fn repl_skips_blanks_retains_turns_and_exits() {
        let mut chat = FakeChat::with_answers(&["first", "second"]);
        let mut input = io::Cursor::new("\nhello\nnext\nexit\nignored\n");
        let mut output = Vec::new();
        interactive_chat(&mut chat, &mut input, &mut output).unwrap();
        assert_eq!(chat.questions, ["hello", "next"]);
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("r4> first"));
        assert!(output.contains("r4> second"));
        assert!(!output.contains("ignored"));
    }
}
