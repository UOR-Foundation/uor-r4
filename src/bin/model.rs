use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use uor_r4_wasm_router::model::{
    download_source, ModelCapability, ModelManifest, ModelStore, QualityAttestation, SourceDownload,
};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Capability {
    Continuation,
    InstructionChat,
}

impl From<Capability> for ModelCapability {
    fn from(capability: Capability) -> Self {
        match capability {
            Capability::Continuation => Self::Continuation,
            Capability::InstructionChat => Self::InstructionChat,
        }
    }
}

/// Manage local transformerless model sources and CID bundles.
#[derive(Debug, Parser)]
#[command(name = "model", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Download pinned open weights for offline transformerless compilation.
    Download(DownloadArgs),
    /// Import compiled and evaluated files into the UOR CID store.
    Import(ImportArgs),
}

#[derive(Debug, Args)]
struct DownloadArgs {
    /// Hugging Face repository, for example HuggingFaceTB/SmolLM2-135M-Instruct.
    #[arg(long)]
    repository: String,
    /// Immutable commit revision. Floating branches such as `main` are discouraged.
    #[arg(long)]
    revision: String,
    /// Portable local source name containing letters, digits, '-' or '_'.
    #[arg(long)]
    name: String,
    /// Download destination directory [default: .uor-models/sources/<name>].
    #[arg(long, value_name = "DIRECTORY")]
    output: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct ImportArgs {
    #[arg(long)]
    name: String,
    #[arg(long)]
    source_model: String,
    #[arg(long, value_enum)]
    capability: Capability,
    #[arg(long)]
    artifacts: PathBuf,
    #[arg(long)]
    store: PathBuf,
    #[arg(long)]
    tokenizer: PathBuf,
    /// Machine-readable instruction/grounding evaluation report.
    #[arg(long)]
    evaluation_report: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    instruction_eval_passed: bool,
    #[arg(long, default_value_t = 0.0)]
    grounded_answer_rate: f32,
    #[arg(long, default_value_t = 1.0)]
    repetition_rate: f32,
}

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("model command failed: {error}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Command::Download(args) => download(args),
        Command::Import(args) => import(args),
    }
}

fn download(args: DownloadArgs) -> Result<(), Box<dyn std::error::Error>> {
    let path = download_source(&SourceDownload {
        repository: args.repository,
        revision: args.revision,
        name: args.name,
        output: args.output,
    })?;
    println!("{}", path.display());
    Ok(())
}

fn import(args: ImportArgs) -> Result<(), Box<dyn std::error::Error>> {
    let model_store = ModelStore::from_env();
    let artifacts = model_store.put(&std::fs::read(args.artifacts)?)?;
    let store = model_store.put(&std::fs::read(args.store)?)?;
    let tokenizer = model_store.put(&std::fs::read(args.tokenizer)?)?;
    let evaluation_report = args
        .evaluation_report
        .map(std::fs::read)
        .transpose()?
        .map(|bytes| model_store.put(&bytes))
        .transpose()?;
    let manifest = ModelManifest {
        schema: 1,
        name: args.name,
        source_model: args.source_model,
        capability: args.capability.into(),
        artifacts,
        store,
        tokenizer,
        evaluation_report,
        quality: QualityAttestation {
            instruction_eval_passed: args.instruction_eval_passed,
            grounded_answer_rate: args.grounded_answer_rate,
            repetition_rate: args.repetition_rate,
        },
    };
    manifest.validate_for_chat().or_else(|error| {
        if manifest.capability == ModelCapability::Continuation {
            Ok(())
        } else {
            Err(error)
        }
    })?;
    let cid = model_store.write_manifest(&manifest)?;
    println!("{cid}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn help_and_download_command_are_valid() {
        Cli::command().debug_assert();
        let cli = Cli::try_parse_from([
            "model",
            "download",
            "--repository",
            "org/model",
            "--revision",
            "abc123",
            "--name",
            "local-model",
        ])
        .unwrap();
        assert!(matches!(cli.command, Command::Download(_)));

        let cli = Cli::try_parse_from([
            "model",
            "download",
            "--repository",
            "org/model",
            "--revision",
            "abc123",
            "--name",
            "local-model",
            "--output",
            "/tmp/models/local-model",
        ])
        .unwrap();
        let Command::Download(args) = cli.command else {
            panic!("expected download command");
        };
        assert_eq!(args.output, Some(PathBuf::from("/tmp/models/local-model")));
    }
}
