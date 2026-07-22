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
    certify, compare, compiler, convert_r4g1, runtime, scenarios,
    teacher::{BehaviorSource, HuggingFaceLlamaOracle, LlamaOracle, TeacherOracle},
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

const DEFAULT_CHECKPOINT: &str = "/tmp/ref/out/model.bin";
const STORE_PATH: &str = "/tmp/tless_store.bin";
const DEFAULT_HF_SOURCE_PATH: &str = ".uor-models/sources/smollm2-135m-instruct";
const DEFAULT_HF_COMPILED_PATH: &str = ".uor-models/compiled/smollm2-135m-instruct";
const DEFAULT_HF_EVALUATION_REPORT: &str = "instruction-eval.json";

#[derive(Debug, PartialEq, Eq)]
struct CompileOptions {
    model: Option<String>,
    revision: Option<String>,
    source: Option<PathBuf>,
    output: Option<PathBuf>,
    seconds: u64,
    target: usize,
    sequence_length: usize,
    r4_attention: bool,
}

#[derive(Debug, PartialEq, Eq)]
struct EvaluateReportOptions {
    source: PathBuf,
    compiled: PathBuf,
    report: Option<PathBuf>,
    sequence_length: usize,
}

#[derive(Debug, Serialize)]
struct EvaluationReport {
    schema: u32,
    distribution: EvaluationDistribution,
    source: EvaluationSource,
    artifacts: EvaluationArtifacts,
    metrics: EvaluationMetrics,
}

#[derive(Debug, Serialize)]
struct EvaluationReportEnvelope {
    report: EvaluationReport,
    report_cid_of_report_bytes: String,
}

#[derive(Debug, Serialize)]
struct EvaluationDistribution {
    name: String,
    split: String,
    held_out_tokens: usize,
}

#[derive(Debug, Serialize)]
struct EvaluationSource {
    directory: String,
    cid: String,
    sequence_length: usize,
}

#[derive(Debug, Serialize)]
struct EvaluationArtifacts {
    directory: String,
    artifacts_cid: String,
    store_cid: String,
    tokenizer_cid: String,
    corpus_meta_cid: String,
    corpus_records_cid: String,
}

#[derive(Debug, Serialize)]
struct EvaluationMetrics {
    top1_accuracy_pct: f64,
    teacher_argmax_agreement_pct: f64,
    bits_per_token: f64,
    teacher_floor_bits_per_token: f64,
    bits_over_teacher_floor: f64,
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
        r4_attention: false,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        if flag == "--r4-attention" {
            options.r4_attention = true;
            index += 1;
            continue;
        }
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

fn parse_evaluate_report_options(args: &[String]) -> Result<EvaluateReportOptions, String> {
    let mut options = EvaluateReportOptions {
        source: PathBuf::from(DEFAULT_HF_SOURCE_PATH),
        compiled: PathBuf::from(DEFAULT_HF_COMPILED_PATH),
        report: None,
        sequence_length: 128,
    };
    let mut index = 0usize;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--source" => options.source = PathBuf::from(value),
            "--compiled" => options.compiled = PathBuf::from(value),
            "--report" => options.report = Some(PathBuf::from(value)),
            "--sequence-length" => {
                options.sequence_length = value
                    .parse()
                    .map_err(|_| format!("invalid --sequence-length value: {value}"))?;
                if options.sequence_length == 0 {
                    return Err("--sequence-length must be greater than zero".to_owned());
                }
            }
            _ => return Err(format!("unknown evaluate-report option: {flag}")),
        }
        index += 2;
    }
    Ok(options)
}

fn file_cid(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn collect_file_entries(
    root: &Path,
    directory: &Path,
    entries: &mut Vec<(PathBuf, String)>,
) -> Result<(), String> {
    let mut children = Vec::new();
    for child in std::fs::read_dir(directory).map_err(|error| error.to_string())? {
        children.push(child.map_err(|error| error.to_string())?.path());
    }
    children.sort();
    for child in children {
        if child.is_dir() {
            collect_file_entries(root, &child, entries)?;
            continue;
        }
        let relative = child
            .strip_prefix(root)
            .map_err(|error| error.to_string())?
            .to_path_buf();
        entries.push((relative, file_cid(&child)?));
    }
    Ok(())
}

fn directory_cid(path: &Path) -> Result<String, String> {
    let mut entries = Vec::new();
    collect_file_entries(path, path, &mut entries)?;
    let mut hasher = blake3::Hasher::new();
    for (relative, cid) in entries {
        hasher.update(relative.to_string_lossy().as_bytes());
        hasher.update(b"\n");
        hasher.update(cid.as_bytes());
        hasher.update(b"\n");
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn argmax_token(distribution: &BTreeMap<u32, u32>) -> u32 {
    let mut best_token = 0u32;
    let mut best_count = 0u32;
    for (&token, &count) in distribution {
        if count > best_count {
            best_count = count;
            best_token = token;
        }
    }
    best_token
}

fn deepest_argmax(store: &runtime::Store, code: &[u8; compiler::STAGES]) -> Option<u32> {
    for depth in (0..=compiler::STAGES).rev() {
        let key = code[..depth].to_vec();
        if let Some(distribution) = store[depth].get(&key) {
            return Some(argmax_token(distribution));
        }
    }
    None
}

fn witten_bell_probability(
    store: &runtime::Store,
    code: &[u8; compiler::STAGES],
    next: u32,
) -> f64 {
    let mut levels: Vec<(f64, &BTreeMap<u32, u32>, u32)> = Vec::new();
    for (depth, level) in store.iter().enumerate().take(compiler::STAGES + 1) {
        let key = code[..depth].to_vec();
        if let Some(distribution) = level.get(&key) {
            let total: u32 = distribution.values().sum();
            let lambda = total as f64 / (total as f64 + distribution.len() as f64);
            levels.push((lambda, distribution, total));
        }
    }
    let mut remaining = 1.0f64;
    let mut probability = 0.0f64;
    for index in (0..levels.len()).rev() {
        let weight = remaining * levels[index].0;
        remaining *= 1.0 - levels[index].0;
        if let Some(&count) = levels[index].1.get(&next) {
            probability += weight * count as f64 / levels[index].2 as f64;
        }
    }
    (probability + remaining / compiler::V as f64).max(1e-30)
}

fn evaluate_report(args: &[String]) -> Result<(), String> {
    let options = parse_evaluate_report_options(args)?;
    let report_path = options
        .report
        .clone()
        .unwrap_or_else(|| options.compiled.join(DEFAULT_HF_EVALUATION_REPORT));
    let source_cid = directory_cid(&options.source)?;
    let artifacts_path = options.compiled.join("tless_artifacts.bin");
    let store_path = options.compiled.join("tless_store.bin");
    let tokenizer_path = options.compiled.join("tokenizer.bin");
    let corpus_meta_path = options.compiled.join("corpus.meta");
    let corpus_records_path = options.compiled.join("corpus.records");
    let artifacts_cid = file_cid(&artifacts_path)?;
    let store_cid = file_cid(&store_path)?;
    let tokenizer_cid = file_cid(&tokenizer_path)?;
    let corpus_meta_cid = file_cid(&corpus_meta_path)?;
    let corpus_records_cid = file_cid(&corpus_records_path)?;

    let corpus_meta = corpus_meta_path
        .to_str()
        .ok_or_else(|| "corpus metadata path is not UTF-8".to_owned())?;
    let corpus_records = corpus_records_path
        .to_str()
        .ok_or_else(|| "corpus records path is not UTF-8".to_owned())?;
    let corpus = compiler::load_corpus_from(corpus_meta, corpus_records).ok_or_else(|| {
        format!(
            "corpus is incomplete at {}; rerun compile until it is complete",
            options.compiled.display()
        )
    })?;
    let held_out_cut = compiler::train_cut(&corpus);
    let mut oracle =
        HuggingFaceLlamaOracle::load_with_sequence_length(&options.source, options.sequence_length)
            .map_err(|error| format!("failed to load Hugging Face model: {error}"))?;
    let mut teacher_logits = vec![0f32; oracle.vocab()];
    let artifacts_bytes = std::fs::read(&artifacts_path).map_err(|error| error.to_string())?;
    let artifacts = compiler::parse_artifacts(&artifacts_bytes)
        .ok_or_else(|| "invalid compiled artifact container".to_owned())?;
    let store_bytes = std::fs::read(&store_path).map_err(|error| error.to_string())?;
    let store = runtime::parse_store(&store_bytes).ok_or_else(|| "invalid store".to_owned())?;
    let rotations = compiler::derive_rotations();

    let mut held_out_tokens = 0usize;
    let mut top1_hits = 0u64;
    let mut argmax_hits = 0u64;
    let mut teacher_floor_bits_total = 0f64;
    let mut bits = 0f64;
    let mut current_story = None;
    let mut story_position = 0usize;
    for index in 0..corpus.n {
        if current_story != Some(corpus.story[index]) {
            current_story = Some(corpus.story[index]);
            story_position = 0;
            oracle.reset();
        }
        if corpus.story[index] < held_out_cut {
            continue;
        }
        oracle.step(
            corpus.input[index] as usize,
            story_position,
            &mut teacher_logits,
        );
        story_position += 1;
        held_out_tokens += 1;
        let code = runtime::code_plain(&artifacts, &rotations, &corpus, index);
        let prediction = deepest_argmax(&store, &code).ok_or_else(|| {
            format!("store has no populated backoff class for held-out position {index}")
        })?;
        if prediction == corpus.next[index] {
            top1_hits += 1;
        }
        let teacher_argmax = teacher_logits
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(token, _)| token as u32)
            .ok_or_else(|| "teacher produced empty logits".to_owned())?;
        if prediction == teacher_argmax {
            argmax_hits += 1;
        }
        let next_token = corpus.next[index] as usize;
        if next_token >= teacher_logits.len() {
            return Err(format!(
                "next token {} is outside teacher vocab {}",
                corpus.next[index],
                teacher_logits.len()
            ));
        }
        let max_logit = teacher_logits
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        let mut denominator = 0f64;
        for logit in &teacher_logits {
            denominator += ((*logit - max_logit) as f64).exp();
        }
        let next_probability =
            ((teacher_logits[next_token] - max_logit) as f64).exp() / denominator.max(1e-30);
        teacher_floor_bits_total += -next_probability.max(1e-30).log2();
        bits += -witten_bell_probability(&store, &code, corpus.next[index]).log2();
    }
    if held_out_tokens == 0 {
        return Err("held-out split is empty; cannot evaluate".to_owned());
    }
    let top1_accuracy_pct = 100.0 * top1_hits as f64 / held_out_tokens as f64;
    let teacher_argmax_agreement_pct = 100.0 * argmax_hits as f64 / held_out_tokens as f64;
    let bits_per_token = bits / held_out_tokens as f64;
    let teacher_floor_bits_per_token = teacher_floor_bits_total / held_out_tokens as f64;
    let bits_over_teacher_floor = bits_per_token - teacher_floor_bits_per_token;

    let report = EvaluationReport {
        schema: 1,
        distribution: EvaluationDistribution {
            name: "D3-held-out".to_owned(),
            split: "compiler::train_cut 80/20 by story id".to_owned(),
            held_out_tokens,
        },
        source: EvaluationSource {
            directory: options.source.display().to_string(),
            cid: source_cid,
            sequence_length: options.sequence_length,
        },
        artifacts: EvaluationArtifacts {
            directory: options.compiled.display().to_string(),
            artifacts_cid,
            store_cid,
            tokenizer_cid,
            corpus_meta_cid,
            corpus_records_cid,
        },
        metrics: EvaluationMetrics {
            top1_accuracy_pct,
            teacher_argmax_agreement_pct,
            bits_per_token,
            teacher_floor_bits_per_token,
            bits_over_teacher_floor,
        },
    };
    if let Some(parent) = report_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let report_json = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    let report_cid = format!("blake3:{}", blake3::hash(&report_json).to_hex());
    let envelope = EvaluationReportEnvelope {
        report,
        report_cid_of_report_bytes: report_cid.clone(),
    };
    let envelope_json =
        serde_json::to_string_pretty(&envelope).map_err(|error| error.to_string())?;
    std::fs::write(&report_path, envelope_json).map_err(|error| error.to_string())?;

    println!(
        "evaluation report written: {} ({})",
        report_path.display(),
        report_cid
    );
    println!(
        "held-out D3 metrics: top1 {:.1}% | agreement {:.1}% | WB {:.4} bits/token (teacher floor {:.4}, +{:.4})",
        top1_accuracy_pct,
        teacher_argmax_agreement_pct,
        bits_per_token,
        teacher_floor_bits_per_token,
        bits_over_teacher_floor
    );
    Ok(())
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
    if options.r4_attention {
        oracle.set_r4_attention(true);
    }
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
    eprintln!("calibrating masked-hamming region radii...");
    let calibration = compiler::calibrate_hamming_regions(&artifacts, &corpus);
    let calibration_json =
        serde_json::to_string_pretty(&calibration).map_err(|error| error.to_string())?;
    std::fs::write(output.join("hamming_calibration.json"), calibration_json)
        .map_err(|error| error.to_string())?;
    eprintln!("inducing hierarchical codes...");
    let hc = compiler::induce_hierarchical_codes(&artifacts.token_codes, oracle.vocab(), &corpus);
    let hc_json = serde_json::to_string_pretty(&hc).map_err(|error| error.to_string())?;
    std::fs::write(output.join("hierarchical_codes.json"), hc_json)
        .map_err(|error| error.to_string())?;
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

    // Helper to calculate Blake3 hash CID
    let calculate_file_hash = |path: &Path| -> Result<String, String> {
        let content = std::fs::read(path).map_err(|error| error.to_string())?;
        let hash = blake3::hash(&content);
        Ok(format!("blake3:{}", hash.to_hex()))
    };

    let artifacts_path = output.join("tless_artifacts.bin");
    let store_path = output.join("tless_store.bin");

    let artifacts_cid = calculate_file_hash(&artifacts_path)?;
    let store_cid = calculate_file_hash(&store_path)?;
    let corpus_cid = calculate_file_hash(Path::new(&meta))?;

    let origin = if let Some(model_name) = options.model.clone() {
        Some(crate::semantic::LearningOrigin {
            kind: "teacher-distillation".to_string(),
            teacher_model: Some(model_name),
            teacher_revision: options.revision.clone(),
        })
    } else {
        Some(crate::semantic::LearningOrigin {
            kind: "native-corpus".to_string(),
            teacher_model: None,
            teacher_revision: None,
        })
    };

    let manifest = crate::semantic::SemanticSpaceManifestV1 {
        space_name: slug.clone(),
        parent_space_cid: None,
        schema_roots: vec!["blake3:schema_root_r4_v1".to_string()],
        axis_definitions: vec![
            "blake3:axis_type".to_string(),
            "blake3:axis_entity".to_string(),
            "blake3:axis_relation".to_string(),
        ],
        codebook_cids: vec![artifacts_cid],
        threshold_cids: vec![store_cid],
        metric_cids: vec!["blake3:metric_hamming_1024".to_string()],
        operator_registry_cid: "blake3:operator_registry_r4_v1".to_string(),
        corpus_root_cids: vec![corpus_cid],
        compiler_cid: "blake3:compiler_r4_v0.1.0".to_string(),
        quality_certificate_cid: "blake3:quality_certificate_r4_v1".to_string(),
        epoch: 1,
        learning_origin: origin,
    };

    let manifest_json =
        serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())?;
    std::fs::write(output.join("space_manifest.json"), manifest_json)
        .map_err(|error| error.to_string())?;
    eprintln!("space manifest generated: space_manifest.json");

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
        Some("evaluate-report") => evaluate_report(&args[1..])?,
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
        Some("convert-r4g1") => convert_r4g1::run(&args[1..])?,
        _ => {
            println!(
                "R4 transformerless — cross-compile a transformer into a mul-free table artifact\n\
<<<<<<< HEAD
                 commands: setup | gen [secs] [target] | compile [--model REPO --revision SHA | --source DIR] [--output DIR] [--seconds N] [--target N] [--sequence-length N] | store | certify | compare | compare-report | scenarios | teacher-kappa\n\
                 hf evaluation: evaluate-report [--source DIR] [--compiled DIR] [--report PATH] [--sequence-length N]\n\
=======
                 commands: setup | gen [secs] [target] | compile [--model REPO --revision SHA | --source DIR] [--output DIR] [--seconds N] [--target N] [--sequence-length N] | store | certify | compare | compare-report | scenarios | teacher-kappa | convert-r4g1 --artifacts <TLA> --store <TLS1> [--calibration <hamming_calibration.json>] --out <R4G1>\n\
>>>>>>> 810573d (Add TLA/TLS1 to R4G1 migration converter and fuzz targets (Phase 1 exit))
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

    #[test]
    fn evaluate_report_defaults_target_smollm2_paths() {
        let options = parse_evaluate_report_options(&[]).expect("defaults");
        assert_eq!(options.source, PathBuf::from(DEFAULT_HF_SOURCE_PATH));
        assert_eq!(options.compiled, PathBuf::from(DEFAULT_HF_COMPILED_PATH));
        assert_eq!(options.sequence_length, 128);
        assert_eq!(options.report, None);
    }

    #[test]
    fn evaluate_report_parses_overrides() {
        let args = [
            "--source",
            "/tmp/source",
            "--compiled",
            "/tmp/compiled",
            "--report",
            "/tmp/out.json",
            "--sequence-length",
            "256",
        ]
        .map(str::to_owned);
        let options = parse_evaluate_report_options(&args).expect("valid options");
        assert_eq!(options.source, PathBuf::from("/tmp/source"));
        assert_eq!(options.compiled, PathBuf::from("/tmp/compiled"));
        assert_eq!(options.report, Some(PathBuf::from("/tmp/out.json")));
        assert_eq!(options.sequence_length, 256);
    }
}
