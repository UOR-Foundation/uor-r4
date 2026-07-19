//! transformerless — cross-compilation of a transformer LM into a
//! multiplication-free, table-native, certifiable inference artifact.
//!
//! Read docs/TRANSFORMERLESS.md (the extrapolation) and docs/PROOF.md (the
//! proof structure and the measured certificate) alongside this code.
//!
//! # Commands
//!
//!   setup            print the external prerequisite commands
//!   gen [secs] [target]   generate/extend the teacher-labeled corpus
//!                         (resumable; whole-story chunking keeps the
//!                         stream deterministic under any chunking)
//!   certify          compile the source, build the store, and print the
//!                    full equivalence certificate and op census
//!
//! # The claim, precisely
//!
//! COMPILER (offline, once): multiplication permitted; every output frozen
//! and blake3-κ-pinned. RUNTIME (per token): every arithmetic operation
//! goes through `OpKernel`, whose complete method set is add / shift / xor /
//! compare / table-read — multiplication is absent from the interface, and
//! the census printed by `certify` measures the ops actually used. The
//! CERTIFIER is instrumentation and may use anything; it never runs at
//! inference. The source-architecture interface is two surfaces (embedding
//! table + next-token oracle); this crate ships the llama-family adapter,
//! and qwen/phi-class sources differ only in that adapter.

use super::{
    certify, compare, compiler, runtime, scenarios,
    teacher::{HuggingFaceLlamaOracle, LlamaOracle},
};
use std::path::{Path, PathBuf};

const DEFAULT_CHECKPOINT: &str = "/tmp/ref/out/model.bin";
const STORE_PATH: &str = "/tmp/tless_store.bin";

#[derive(Debug, PartialEq, Eq)]
struct CompileOptions {
    model: Option<String>,
    revision: Option<String>,
    source: Option<PathBuf>,
    output: Option<PathBuf>,
    seconds: u64,
    target: usize,
    sequence_length: usize,
}

fn parse_compile_options(args: &[String]) -> Result<CompileOptions, String> {
    let mut options = CompileOptions {
        model: None,
        revision: None,
        source: None,
        output: None,
        seconds: 300,
        target: 20_000,
        sequence_length: 128,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--model" => options.model = Some(value.clone()),
            "--revision" => options.revision = Some(value.clone()),
            "--source" => options.source = Some(PathBuf::from(value)),
            "--output" => options.output = Some(PathBuf::from(value)),
            "--seconds" => {
                options.seconds = value
                    .parse()
                    .map_err(|_| format!("invalid --seconds value: {value}"))?;
            }
            "--target" => {
                options.target = value
                    .parse()
                    .map_err(|_| format!("invalid --target value: {value}"))?;
            }
            "--sequence-length" => {
                options.sequence_length = value
                    .parse()
                    .map_err(|_| format!("invalid --sequence-length value: {value}"))?;
                if options.sequence_length == 0 {
                    return Err("--sequence-length must be greater than zero".to_owned());
                }
            }
            _ => return Err(format!("unknown compile option: {flag}")),
        }
        index += 2;
    }
    if options.model.is_none() && options.source.is_none() {
        return Err("pass --model <HF_REPOSITORY> or --source <DIRECTORY>".to_owned());
    }
    if options.model.is_some() && options.revision.is_none() {
        return Err("--model requires an immutable --revision".to_owned());
    }
    Ok(options)
}

fn source_slug(options: &CompileOptions) -> String {
    let raw = options
        .model
        .as_deref()
        .and_then(|model| model.rsplit('/').next())
        .or_else(|| {
            options
                .source
                .as_deref()
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
        })
        .unwrap_or("model");
    let slug: String = raw
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    slug.trim_matches('-').to_owned()
}

fn setup() {
    println!(
        "\
external prerequisites (network domains: github.com, raw.githubusercontent.com):

# source checkpoint AND tokenizer: stories15M inside the APE zip payload of
# the trholding/llama2.c release asset (source κ must pin to blake3:0ae73395…;
# tokenizer.bin is required by `scenarios`)
curl -sL -o /tmp/run.com https://github.com/trholding/llama2.c/releases/download/experimental/run.com
cd /tmp && unzip -o run.com out/model.bin tokenizer.bin -d ref

# real out-of-domain text for the scenario suite (public domain)
curl -sL https://raw.githubusercontent.com/karpathy/char-rnn/master/data/tinyshakespeare/input.txt -o /tmp/corpus.txt

pipeline:
  transformerless gen 1500 150000    # repeat until 'done=1'
  transformerless certify            # compile + store + certificate + census
  transformerless compare            # runtime comparison (docs/COMPARISON.md)
  transformerless compare-report     # print the certified llama.cpp comparison (no artifacts needed)
  transformerless scenarios          # scenario suite (needs tokenizer + corpus.txt)"
    );
}

fn download_hf_source(repository: &str, revision: &str, destination: &Path) -> Result<(), String> {
    let mut repository_parts = repository.split('/');
    let valid_part = |part: &str| {
        !part.is_empty()
            && part.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
            })
    };
    if !matches!(
        (repository_parts.next(), repository_parts.next(), repository_parts.next()),
        (Some(owner), Some(model), None) if valid_part(owner) && valid_part(model)
    ) {
        return Err("--model must be a Hugging Face owner/repository name".to_owned());
    }
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("--revision must be a 40-character immutable commit SHA".to_owned());
    }
    eprintln!(
        "downloading {repository}@{revision} to {}...",
        destination.display()
    );
    std::fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    let status = std::process::Command::new("hf")
        .arg("download")
        .arg(repository)
        .arg("--revision")
        .arg(revision)
        .arg("--local-dir")
        .arg(destination)
        .args([
            "--include",
            "*.safetensors",
            "--include",
            "*.json",
            "--include",
            "merges.txt",
            "--include",
            "vocab.json",
        ])
        .status()
        .map_err(|error| format!("failed to run hf: {error}"))?;
    if !status.success() {
        return Err(format!("hf download failed with status {status}"));
    }
    eprintln!("download complete");
    Ok(())
}

pub fn compile_hugging_face(args: &[String]) -> Result<(), String> {
    #[cfg(debug_assertions)]
    eprintln!(
        "warning: debug builds make teacher generation much slower; use `cargo run --release -- compile ...`"
    );
    let options = parse_compile_options(args)?;
    let slug = source_slug(&options);
    let source = options
        .source
        .clone()
        .unwrap_or_else(|| PathBuf::from(".uor-models/sources").join(&slug));
    if let Some(repository) = options.model.as_deref() {
        download_hf_source(
            repository,
            options
                .revision
                .as_deref()
                .expect("validated model revision"),
            &source,
        )?;
    }
    let output = options
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from(".uor-models/compiled").join(&slug));
    eprintln!("compiler output: {}", output.display());
    std::fs::create_dir_all(&output).map_err(|error| error.to_string())?;
    let meta = output.join("corpus.meta");
    let records = output.join("corpus.records");
    let meta = meta
        .to_str()
        .ok_or_else(|| "corpus metadata path is not UTF-8".to_owned())?;
    let records = records
        .to_str()
        .ok_or_else(|| "corpus records path is not UTF-8".to_owned())?;
    let mut oracle =
        HuggingFaceLlamaOracle::load_with_sequence_length(&source, options.sequence_length)
            .map_err(|error| format!("failed to load Hugging Face model: {error}"))?;
    compiler::generate_to(&mut oracle, options.seconds, options.target, meta, records);
    let Some(corpus) = compiler::load_corpus_from(meta, records) else {
        println!(
            "corpus is not complete; rerun the same command to resume {}",
            output.display()
        );
        return Ok(());
    };
    eprintln!("teacher corpus complete; compiling table-native artifact...");
    let artifacts = compiler::compile(&oracle, &corpus);
    eprintln!("writing artifact...");
    std::fs::write(
        output.join("tless_artifacts.bin"),
        compiler::artifact_bytes(&artifacts),
    )
    .map_err(|error| error.to_string())?;
    eprintln!("building graded store...");
    let (store, _) = runtime::build_store(&artifacts, &corpus);
    std::fs::write(output.join("tless_store.bin"), runtime::store_bytes(&store))
        .map_err(|error| error.to_string())?;
    eprintln!("exporting tokenizer...");
    scenarios::export_hf_bytelevel_tokenizer(
        source.join("tokenizer.json"),
        output.join("tokenizer.bin"),
    )
    .map_err(|error| error.to_string())?;
    println!("compile complete: {}", output.display());
    println!(
        "bundle ready for local `ask`; use `cargo run -- import --help` to attach a quality attestation and persist a named manifest (name: {slug})"
    );
    Ok(())
}

pub fn run(args: &[String]) -> Result<(), String> {
    match args.first().map(|s| s.as_str()) {
        Some("setup") => setup(),
        Some("gen") => {
            let secs: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(300);
            let target: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(150_000);
            let mut oracle = LlamaOracle::load(DEFAULT_CHECKPOINT);
            compiler::generate(&mut oracle, secs, target);
        }
        Some("certify") => {
            let oracle = LlamaOracle::load(DEFAULT_CHECKPOINT);
            certify::certify(&oracle);
        }
        Some("compile") => {
            if args.len() == 1 {
                let c = compiler::load_corpus().expect("corpus incomplete: run gen first");
                let oracle = LlamaOracle::load(DEFAULT_CHECKPOINT);
                let art = compiler::compile(&oracle, &c);
                compiler::save_artifacts(&art);
            } else {
                compile_hugging_face(&args[1..])?;
            }
        }
        Some("store") => {
            let c = compiler::load_corpus()
                .expect("corpus incomplete: run `transformerless gen` first");
            let art =
                compiler::load_artifacts().expect("run `cargo run --release -- compile` first");
            let (store, _) = runtime::build_store(&art, &c);
            let bytes = runtime::store_bytes(&store);
            std::fs::write(STORE_PATH, &bytes).unwrap();
            println!(
                "store saved: {} ({} bytes, κ {})",
                STORE_PATH,
                bytes.len(),
                runtime::store_kappa(&store)
            );
        }
        Some("compare") => {
            let mut oracle = LlamaOracle::load(DEFAULT_CHECKPOINT);
            compare::compare(&mut oracle);
        }
        Some("compare-report") => compare::report(),
        Some("scenarios") => {
            let mut oracle = LlamaOracle::load(DEFAULT_CHECKPOINT);
            scenarios::scenarios(&mut oracle);
        }
        Some("teacher-kappa") => match std::fs::read(DEFAULT_CHECKPOINT) {
            Ok(b) => println!(
                "source κ: blake3:{} ({} bytes)",
                blake3::hash(&b).to_hex(),
                b.len()
            ),
            Err(_) => println!("source checkpoint not found; see `setup`"),
        },
        _ => {
            println!(
                "R4 transformerless — cross-compile a transformer into a mul-free table artifact\n\
                 commands: setup | gen [secs] [target] | compile [--model REPO --revision SHA | --source DIR] [--output DIR] [--seconds N] [--target N] [--sequence-length N] | store | certify | compare | compare-report | scenarios | teacher-kappa\n\
                 docs: docs/TRANSFORMERLESS.md (extrapolation), docs/PROOF.md (proof + certificate)"
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_parametric_hugging_face_compile() {
        let args = [
            "--model",
            "HuggingFaceTB/SmolLM2-135M-Instruct",
            "--revision",
            "7e27bd9f95328f0f3b08261d1252705110c806f8",
            "--output",
            "/tmp/compiled",
            "--seconds",
            "60",
            "--target",
            "1000",
            "--sequence-length",
            "64",
        ]
        .map(str::to_owned);
        let options = parse_compile_options(&args).expect("valid options");
        assert_eq!(
            options.model.as_deref(),
            Some("HuggingFaceTB/SmolLM2-135M-Instruct")
        );
        assert_eq!(options.output, Some(PathBuf::from("/tmp/compiled")));
        assert_eq!(options.seconds, 60);
        assert_eq!(options.target, 1000);
        assert_eq!(options.sequence_length, 64);
        assert_eq!(source_slug(&options), "smollm2-135m-instruct");
    }

    #[test]
    fn local_source_does_not_require_hugging_face_revision() {
        let args = ["--source", "/models/local", "--target", "10"].map(str::to_owned);
        let options = parse_compile_options(&args).expect("valid local source");
        assert_eq!(options.source, Some(PathBuf::from("/models/local")));
        assert_eq!(options.target, 10);
        assert_eq!(options.sequence_length, 128);
        assert_eq!(source_slug(&options), "local");
    }

    #[test]
    fn hugging_face_compile_defaults_are_bounded() {
        let args = ["--source", "/models/local"].map(str::to_owned);
        let options = parse_compile_options(&args).expect("valid local source");
        assert_eq!(options.target, 20_000);
        assert_eq!(options.sequence_length, 128);
    }

    #[test]
    fn remote_model_requires_pinned_revision() {
        let args = ["--model", "org/model"].map(str::to_owned);
        assert_eq!(
            parse_compile_options(&args),
            Err("--model requires an immutable --revision".to_owned())
        );
    }
}
