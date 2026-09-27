//! Train, evaluate and sample the geometric stack and its transformer control
//! (`uor_r4_training::geometric_stack`) on u16 token files, and prepare those
//! files with the lab's byte-level BPE.
//!
//! Modes:
//!
//! ```text
//! geometric-stack train train=TRAIN.u16[,MORE.u16] [train_weights=W1,W2] valid=VALID.u16 \
//!   out=NEW_REPORT_ROOT arch=transformer|geometric [pattern=rrarra] [read=lorentz|dot] [rotation=true|false] \
//!   [seed=1] [steps=7324] [batch=16] [lr=0.002] [warmup=200] [min_lr=0.1] [weight_decay=0.1] \
//!   [clip=1.0] [eval_every=250] [eval_windows=64] [final_windows=512] [lens=LENS.u16] \
//!   [merges=MERGES.txt] [checkpoint_every=250] [resume=OLD_ROOT/checkpoint] [max_seconds=inf] \
//!   [sample_tokens=128] [tokenizer=TOKENIZER.json] [width=288] [heads=6] [layers=6] [mlp=768] \
//!   [context=256]
//! geometric-stack sample model=ROOT/model valid=VALID.u16 merges=MERGES.txt|tokenizer=TOKENIZER.json \
//!   out=NEW_REPORT_ROOT [prompts=3] [prompt_tokens=64] [sample_tokens=128] [temperature=0.8] [top_k=40] \
//!   [seed=1]
//! geometric-stack evaluate model=ROOT/model tokens=DEV.u16 out=NEW_REPORT_ROOT [tune_blocks=64] \
//!   [lens=LENS.u16]
//! geometric-stack encode merges=MERGES.txt input=TEXT out=TOKENS.u16
//! geometric-stack corpus registry=CARGO_REGISTRY_SRC_INDEX out=TEXT [max_file_bytes=200000]
//! geometric-stack export model=ROOT/model out=NEW_REPORT_ROOT
//! geometric-stack lut-evaluate artifact=ROOT/model.lut valid=VALID.u16 out=NEW_REPORT_ROOT \
//!   [model=ROOT/model] [windows=64] [blocks=false] [tune_blocks=64] [threads=1] [lens=LENS.u16]
//! geometric-stack lut-sample artifact=ROOT/model.lut out=NEW_REPORT_ROOT (valid=VALID.u16 | prompt=TEXT) \
//!   [merges=MERGES.txt | tokenizer=TOKENIZER.json] [prompts=3] [prompt_tokens=64] [sample_tokens=128] \
//!   [temperature=0.8] [top_k=40] [top_p=1] [seed=1] [threads=1]
//! ```
//!
//! The shape options describe the transformer control (#1017's by default). A
//! geometric stack takes the same width, heads, depth and context, the
//! pattern `rra` repeated by default, and the MLP width that matches the
//! control's parameter count.
//!
//! `train` samples windows uniformly from TRAIN with a seeded generator, so two
//! arms with one seed see identical windows. Evaluation windows are evenly
//! spaced over VALID, the rule of `joint-read-geometry`, so `final_windows=512`
//! at context 256 scores the same 131,072 targets as that example. A stopped run
//! continues with `resume=OLD_ROOT/checkpoint` in a new root, with identical
//! settings and input contents (sizes and SHA-256 of every input file). Every
//! report root is claimed before loading and sealed at the end. Set
//! RAYON_NUM_THREADS to bound the threads.
//!
//! `evaluate` scores consecutive 256-input blocks with a fresh state per
//! block (257 stored ids, 256 targets), the retained evaluator's protocol
//! (`docs/integration/reference-evaluator-v2.json`): it reports the first
//! `tune_blocks` blocks, the remaining comparison blocks and all blocks
//! separately. On the retained TinyStories development store that is 976
//! blocks, 64 tune and 912 comparison (233,472 targets), where #1017 scores
//! 1.574024 nats. Samples decode with `merges=` (the lab BPE) or
//! `tokenizer=` (a `tokenizer.json`, such as the retained 4,096-token one).
//!
//! `export` writes the integer serving artifact of a trained model under owner
//! decision D10 (`uor_r4_training::stack_export`): a stack artifact for a
//! geometric stack, served by `uor_r4_lut::stack`, or a Llama artifact for the
//! transformer control, served by `uor_r4_lut::engine`. `lut-evaluate` runs
//! either engine position by position over the evenly spaced windows of
//! `train`'s evaluation (`windows=512` is the final evaluation's 131,072
//! targets), with a fresh session per window, and reports its NLL beside the
//! float model's on the same windows when `model=` is given, their top-1
//! agreement and the engine's tokens per second; `blocks=true` uses
//! `evaluate`'s consecutive blocks and tune/comparison split instead.
//! `lut-sample` writes greedy and sampled continuations from the integer
//! engine (the integer sampler of `uor_r4_lut::sampling`).

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_training::geometric_stack::{ReadScore, StackAdamW, StackArch, StackConfig, StackModel};
use uor_r4_training::lut_export::export_llama;
use uor_r4_training::stack_export::{control_checkpoint, export_stack};
use uor_r4_training::{sha256_file, Result, TrainingError};

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

fn lut(error: uor_r4_lut::LutError) -> TrainingError {
    invalid(error.to_string())
}

struct Args(BTreeMap<String, String>);

impl Args {
    fn parse(arguments: &[String], allowed: &[&str]) -> Result<Self> {
        let mut pairs = BTreeMap::new();
        for argument in arguments {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| invalid(format!("arguments are key=value, got {argument}")))?;
            if !allowed.contains(&key) {
                return Err(invalid(format!("unknown argument {key}=")));
            }
            pairs.insert(key.to_owned(), value.to_owned());
        }
        Ok(Self(pairs))
    }

    fn required(&self, key: &str) -> Result<String> {
        self.0
            .get(key)
            .cloned()
            .ok_or_else(|| invalid(format!("missing {key}=")))
    }

    fn optional(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    fn number<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T> {
        match self.0.get(key) {
            None => Ok(default),
            Some(value) => value
                .parse()
                .map_err(|_| invalid(format!("invalid {key}={value}"))),
        }
    }
}

/// SplitMix64: window sampling and categorical draws.
#[derive(Clone, Copy)]
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }

    fn uniform(&mut self) -> f64 {
        ((self.next() >> 11) as f64 + 0.5) / (1u64 << 53) as f64
    }
}

fn read_tokens(path: &Path, vocabulary: usize) -> Result<Vec<u32>> {
    let bytes = fs::read(path)?;
    if bytes.len() % 2 != 0 {
        return Err(invalid(format!(
            "{} is not a u16 token file",
            path.display()
        )));
    }
    let tokens: Vec<u32> = bytes
        .chunks_exact(2)
        .map(|pair| u32::from(u16::from_le_bytes([pair[0], pair[1]])))
        .collect();
    if tokens.iter().any(|&id| id as usize >= vocabulary) {
        return Err(invalid(format!(
            "{} has ids outside the vocabulary",
            path.display()
        )));
    }
    Ok(tokens)
}

fn identity(path: &Path) -> Result<Value> {
    Ok(json!({"path": path, "bytes": fs::metadata(path)?.len(), "sha256": sha256_file(path)?}))
}

// ---------------------------------------------------------------------------
// The lab's byte-level BPE (docs/evidence/native-lorentz-packet-2026-09-26/
// sources/c3-harness/src/bin/bpe.rs): identical pre-tokenization and merges.

const MAX_PUNCT: usize = 8;
const MAX_SPACE: usize = 32;

fn class(byte: u8) -> u8 {
    match byte {
        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | 0x80..=0xff => 0,
        b' ' | b'\t' | b'\n' | b'\r' => 1,
        _ => 2,
    }
}

fn run_end(text: &[u8], from: usize, c: u8) -> usize {
    let cap = match c {
        0 => usize::MAX,
        1 => MAX_SPACE,
        _ => MAX_PUNCT,
    };
    let mut end = from;
    while end < text.len() && class(text[end]) == c && end - from < cap {
        end += 1;
    }
    end
}

fn chunks(text: &[u8]) -> Vec<&[u8]> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < text.len() {
        let c = class(text[i]);
        if c == 1 {
            if text[i] == b' ' && i + 1 < text.len() && class(text[i + 1]) != 1 {
                let end = run_end(text, i + 1, class(text[i + 1]));
                out.push(&text[i..end]);
                i = end;
                continue;
            }
            let mut end = run_end(text, i, 1);
            if end < text.len() && end - i > 1 && text[end - 1] == b' ' && class(text[end]) != 1 {
                end -= 1;
            }
            out.push(&text[i..end]);
            i = end;
            continue;
        }
        let end = run_end(text, i, c);
        out.push(&text[i..end]);
        i = end;
    }
    out
}

struct Bpe {
    rank: HashMap<(u32, u32), u32>,
    token_bytes: Vec<Vec<u8>>,
}

impl Bpe {
    fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)?;
        let mut rank = HashMap::new();
        let mut token_bytes: Vec<Vec<u8>> = (0..=255u8).map(|b| vec![b]).collect();
        for (r, line) in text.lines().enumerate() {
            let (a, b) = line
                .split_once(' ')
                .ok_or_else(|| invalid("merges lines are `a b`"))?;
            let (a, b): (u32, u32) = (
                a.parse().map_err(|_| invalid("merge id"))?,
                b.parse().map_err(|_| invalid("merge id"))?,
            );
            if a as usize >= token_bytes.len() || b as usize >= token_bytes.len() {
                return Err(invalid("merge refers to a later token"));
            }
            let joined = [
                token_bytes[a as usize].clone(),
                token_bytes[b as usize].clone(),
            ]
            .concat();
            token_bytes.push(joined);
            rank.insert((a, b), r as u32);
        }
        Ok(Self { rank, token_bytes })
    }

    fn encode_chunk(&self, chunk: &[u8]) -> Vec<u32> {
        let mut symbols: Vec<u32> = chunk.iter().map(|&b| u32::from(b)).collect();
        loop {
            let mut best: Option<(u32, (u32, u32))> = None;
            for pair in symbols.windows(2) {
                if let Some(&r) = self.rank.get(&(pair[0], pair[1])) {
                    if best.is_none_or(|(b, _)| r < b) {
                        best = Some((r, (pair[0], pair[1])));
                    }
                }
            }
            let Some((r, (a, b))) = best else { break };
            let merged = 256 + r;
            let mut next = Vec::with_capacity(symbols.len());
            let mut i = 0;
            while i < symbols.len() {
                if i + 1 < symbols.len() && symbols[i] == a && symbols[i + 1] == b {
                    next.push(merged);
                    i += 2;
                } else {
                    next.push(symbols[i]);
                    i += 1;
                }
            }
            symbols = next;
        }
        symbols
    }

    fn encode(&self, text: &[u8]) -> Vec<u16> {
        let mut cache: HashMap<&[u8], Vec<u32>> = HashMap::new();
        let mut out = Vec::with_capacity(text.len() / 3);
        for chunk in chunks(text) {
            let ids = cache
                .entry(chunk)
                .or_insert_with(|| self.encode_chunk(chunk));
            out.extend(ids.iter().map(|&id| id as u16));
        }
        out
    }

    fn decode(&self, ids: &[u32]) -> String {
        let bytes: Vec<u8> = ids
            .iter()
            .flat_map(|&id| {
                self.token_bytes
                    .get(id as usize)
                    .cloned()
                    .unwrap_or_default()
            })
            .collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

/// Decodes sample ids for the report.
enum Decoder {
    Lab(Bpe),
    Retained(Box<uor_r4_tokenizer::ByteBpeTokenizer>),
}

impl Decoder {
    fn load(merges: Option<&Path>, tokenizer: Option<&Path>) -> Result<Option<Self>> {
        match (merges, tokenizer) {
            (Some(_), Some(_)) => Err(invalid("give merges= or tokenizer=, not both")),
            (Some(path), None) => Ok(Some(Self::Lab(Bpe::load(path)?))),
            (None, Some(path)) => Ok(Some(Self::Retained(Box::new(
                uor_r4_tokenizer::ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(path)?)
                    .ok_or_else(|| invalid("unreadable tokenizer.json"))?,
            )))),
            (None, None) => Ok(None),
        }
    }

    fn decode(&self, ids: &[u32]) -> String {
        match self {
            Self::Lab(bpe) => bpe.decode(ids),
            Self::Retained(tokenizer) => tokenizer.decode(ids),
        }
    }

    fn encode(&self, text: &str) -> Vec<u32> {
        match self {
            Self::Lab(bpe) => bpe
                .encode(text.as_bytes())
                .into_iter()
                .map(u32::from)
                .collect(),
            Self::Retained(tokenizer) => tokenizer.encode(text),
        }
    }
}

fn encode_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(arguments, &["merges", "input", "out"])?;
    let out = PathBuf::from(args.required("out")?);
    if out.exists() {
        return Err(invalid("encode output exists"));
    }
    let bpe = Bpe::load(Path::new(&args.required("merges")?))?;
    let text = fs::read(args.required("input")?)?;
    let ids = bpe.encode(&text);
    let decoded: Vec<u8> = ids
        .iter()
        .flat_map(|&id| bpe.token_bytes[id as usize].iter().copied())
        .collect();
    if decoded != text {
        return Err(invalid("BPE round trip failed"));
    }
    let bytes: Vec<u8> = ids.iter().flat_map(|id| id.to_le_bytes()).collect();
    fs::write(&out, bytes)?;
    println!(
        "{}",
        json!({"input_bytes": text.len(), "tokens": ids.len(), "bytes_per_token": text.len() as f64 / ids.len() as f64, "round_trip": "exact", "sha256": sha256_file(&out)?})
    );
    Ok(())
}

/// Latest version of each crate in a Cargo registry source directory; every
/// `.rs` file up to `max_file_bytes`, in sorted path order, each followed by a
/// newline (the repository corpus's convention).
fn corpus_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(arguments, &["registry", "out", "max_file_bytes"])?;
    let registry = PathBuf::from(args.required("registry")?);
    let out = PathBuf::from(args.required("out")?);
    let max_file_bytes: u64 = args.number("max_file_bytes", 200_000)?;
    if out.exists() {
        return Err(invalid("corpus output exists"));
    }
    let mut latest: BTreeMap<String, (Vec<u64>, PathBuf)> = BTreeMap::new();
    for entry in fs::read_dir(&registry)? {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(split) = name
            .char_indices()
            .find(|&(i, c)| c == '-' && name[i + 1..].starts_with(|d: char| d.is_ascii_digit()))
            .map(|(i, _)| i)
        else {
            continue;
        };
        let (crate_name, version) = (&name[..split], &name[split + 1..]);
        let key: Vec<u64> = version
            .split(|c: char| !c.is_ascii_digit())
            .filter(|part| !part.is_empty())
            .map(|part| part.parse().unwrap_or(0))
            .collect();
        let replace = latest
            .get(crate_name)
            .is_none_or(|(existing, _)| key > *existing);
        if replace {
            latest.insert(crate_name.to_owned(), (key, path));
        }
    }
    let mut files = Vec::new();
    for (_, root) in latest.values() {
        let mut stack = vec![root.clone()];
        while let Some(directory) = stack.pop() {
            for entry in fs::read_dir(&directory)? {
                let path = entry?.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "rs")
                    && fs::metadata(&path)?.len() <= max_file_bytes
                {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    let mut text = Vec::new();
    for path in &files {
        text.extend(fs::read(path)?);
        text.push(b'\n');
    }
    fs::write(&out, &text)?;
    println!(
        "{}",
        json!({"crates": latest.len(), "files": files.len(), "bytes": text.len(), "sha256": sha256_file(&out)?})
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Training.

struct Settings {
    /// One or more training streams; a window's stream is drawn with
    /// probability proportional to `train_weights`, then its start uniformly.
    train: Vec<PathBuf>,
    train_weights: Vec<f64>,
    valid: PathBuf,
    lens: Option<PathBuf>,
    merges: Option<PathBuf>,
    tokenizer: Option<PathBuf>,
    config: StackConfig,
    steps: usize,
    batch: usize,
    lr: f64,
    warmup: usize,
    min_lr: f64,
    weight_decay: f64,
    clip: f64,
    eval_every: usize,
    eval_windows: usize,
    final_windows: usize,
    checkpoint_every: usize,
    resume: Option<PathBuf>,
    max_seconds: f64,
    sample_tokens: usize,
}

impl Settings {
    fn record(&self) -> Value {
        json!({
            "train": self.train, "train_weights": self.train_weights, "valid": self.valid,
            "lens": self.lens, "merges": self.merges, "tokenizer": self.tokenizer,
            "config": self.config, "steps": self.steps, "batch": self.batch, "lr": self.lr,
            "warmup": self.warmup, "min_lr": self.min_lr, "weight_decay": self.weight_decay,
            "clip": self.clip, "eval_every": self.eval_every, "eval_windows": self.eval_windows,
            "final_windows": self.final_windows, "checkpoint_every": self.checkpoint_every,
            "sample_tokens": self.sample_tokens,
        })
    }

    /// Settings and input contents a resumed run must share with its parent.
    fn lineage(&self, inputs: &Value) -> Value {
        json!({
            "config": self.config, "steps": self.steps, "batch": self.batch, "lr": self.lr,
            "warmup": self.warmup, "min_lr": self.min_lr, "weight_decay": self.weight_decay,
            "clip": self.clip, "eval_every": self.eval_every, "eval_windows": self.eval_windows,
            "train_weights": self.train_weights, "inputs": inputs,
        })
    }

    /// Content identities (size and SHA-256, not paths) of every file the run
    /// reads, so a resume must present the same bytes in the same order.
    fn input_contents(&self) -> Result<Value> {
        let content = |path: &Path| -> Result<Value> {
            Ok(json!({"bytes": fs::metadata(path)?.len(), "sha256": sha256_file(path)?}))
        };
        Ok(json!({
            "train": self.train.iter().map(|p| content(p)).collect::<Result<Vec<_>>>()?,
            "valid": content(&self.valid)?,
            "lens": self.lens.as_deref().map(content).transpose()?,
            "merges": self.merges.as_deref().map(content).transpose()?,
            "tokenizer": self.tokenizer.as_deref().map(content).transpose()?,
        }))
    }
}

fn train_settings(args: &Args) -> Result<Settings> {
    let seed: u64 = args.number("seed", 1)?;
    let read = match args.optional("read").as_deref() {
        None | Some("lorentz") => ReadScore::Lorentz,
        Some("dot") => ReadScore::Dot,
        Some(other) => return Err(invalid(format!("unknown read {other}"))),
    };
    let rotation = match args.optional("rotation").as_deref() {
        None | Some("true") => true,
        Some("false") => false,
        Some(other) => return Err(invalid(format!("invalid rotation={other}"))),
    };
    // The control's shape; #1017's by default. A geometric stack takes its
    // width, heads, depth and context and matches its parameter count.
    let control = StackConfig::transformer(
        args.number("width", 288)?,
        args.number("heads", 6)?,
        args.number("layers", 6)?,
        args.number("mlp", 768)?,
        args.number("context", 256)?,
        seed,
    )?;
    let config = match args.required("arch")?.as_str() {
        "transformer" => control,
        "geometric" => {
            let layers = control.layers();
            let default_pattern = "rra".repeat(layers / 3) + &"r".repeat(layers % 3);
            StackConfig::geometric_matched_to(
                &control,
                &args.optional("pattern").unwrap_or(default_pattern),
                read,
                rotation,
            )?
        }
        other => return Err(invalid(format!("unknown arch {other}"))),
    };
    let train: Vec<PathBuf> = args
        .required("train")?
        .split(',')
        .map(PathBuf::from)
        .collect();
    let train_weights: Vec<f64> = match args.optional("train_weights") {
        None => vec![1.0; train.len()],
        Some(text) => text
            .split(',')
            .map(|w| {
                w.parse()
                    .map_err(|_| invalid(format!("invalid train weight {w}")))
            })
            .collect::<Result<_>>()?,
    };
    if train_weights.len() != train.len()
        || train_weights
            .iter()
            .any(|w| w.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater))
    {
        return Err(invalid("one positive train weight per training stream"));
    }
    let settings = Settings {
        train,
        train_weights,
        valid: PathBuf::from(args.required("valid")?),
        lens: args.optional("lens").map(PathBuf::from),
        merges: args.optional("merges").map(PathBuf::from),
        tokenizer: args.optional("tokenizer").map(PathBuf::from),
        config,
        steps: args.number("steps", 7324)?,
        batch: args.number("batch", 16)?,
        lr: args.number("lr", 0.002)?,
        warmup: args.number("warmup", 200)?,
        min_lr: args.number("min_lr", 0.1)?,
        weight_decay: args.number("weight_decay", 0.1)?,
        clip: args.number("clip", 1.0)?,
        eval_every: args.number("eval_every", 250)?,
        eval_windows: args.number("eval_windows", 64)?,
        final_windows: args.number("final_windows", 512)?,
        checkpoint_every: args.number("checkpoint_every", 250)?,
        resume: args.optional("resume").map(PathBuf::from),
        max_seconds: args.number("max_seconds", f64::INFINITY)?,
        sample_tokens: args.number("sample_tokens", 128)?,
    };
    if settings.steps == 0
        || settings.batch == 0
        || settings.eval_every == 0
        || settings.eval_windows == 0
        || settings.final_windows == 0
        || settings.lr.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)
        || !(0.0..=1.0).contains(&settings.min_lr)
    {
        return Err(invalid(
            "steps, batch, evaluation counts and lr must be positive; min_lr in [0, 1]",
        ));
    }
    Ok(settings)
}

fn learning_rate(settings: &Settings, step: usize) -> f64 {
    if step < settings.warmup {
        return settings.lr * (step + 1) as f64 / settings.warmup as f64;
    }
    let span = (settings.steps - settings.warmup).max(1) as f64;
    let progress = ((step - settings.warmup) as f64 / span).min(1.0);
    let floor = settings.lr * settings.min_lr;
    floor + (settings.lr - floor) * 0.5 * (1.0 + (std::f64::consts::PI * progress).cos())
}

struct Evaluation {
    nll: f64,
    bits_per_byte: Option<f64>,
    targets: usize,
}

impl Evaluation {
    fn record(&self) -> Value {
        json!({"nll": self.nll, "bits_per_byte": self.bits_per_byte, "targets": self.targets})
    }
}

/// Evenly spaced windows over VALID (the `joint-read-geometry` rule).
fn evaluate(
    model: &StackModel,
    valid: &[u32],
    lens: Option<&[u32]>,
    windows: usize,
) -> Result<Evaluation> {
    let time = model.config.context;
    let stride = (valid.len() - time - 1) / windows;
    let starts: Vec<usize> = (0..windows).map(|window| window * stride).collect();
    let (mut nll, mut bytes, mut targets_seen) = (0f64, 0f64, 0usize);
    for group in starts.chunks(16) {
        let mut ids = Vec::with_capacity(group.len() * time);
        let mut targets = Vec::with_capacity(group.len() * time);
        for &start in group {
            ids.extend_from_slice(&valid[start..start + time]);
            targets.extend_from_slice(&valid[start + 1..start + time + 1]);
        }
        if let Some(lens) = lens {
            bytes += targets
                .iter()
                .map(|&id| f64::from(lens[id as usize]))
                .sum::<f64>();
        }
        nll += model
            .target_nll(&ids, &targets, group.len(), time)?
            .iter()
            .sum::<f64>();
        targets_seen += targets.len();
    }
    Ok(Evaluation {
        nll: nll / targets_seen as f64,
        bits_per_byte: lens.map(|_| nll / std::f64::consts::LN_2 / bytes),
        targets: targets_seen,
    })
}

/// Greedy (`temperature = 0`) or top-k sampled continuation of `prompt`.
fn generate(
    model: &StackModel,
    prompt: &[u32],
    new_tokens: usize,
    temperature: f64,
    top_k: usize,
    rng: &mut Rng,
) -> Result<Vec<u32>> {
    let context = model.config.context;
    let mut tokens = prompt.to_vec();
    for _ in 0..new_tokens {
        let window = &tokens[tokens.len().saturating_sub(context)..];
        let logits = model.forward(window, 1, window.len())?.detach();
        let last = logits.get(window.len() - 1)?.to_vec1::<f32>()?;
        let next = if temperature <= 0.0 {
            last.iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .map(|(i, _)| i)
                .ok_or_else(|| invalid("empty logits"))?
        } else {
            let mut ranked: Vec<(usize, f64)> = last
                .iter()
                .enumerate()
                .map(|(i, &l)| (i, f64::from(l) / temperature))
                .collect();
            ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
            ranked.truncate(top_k.max(1));
            let maximum = ranked[0].1;
            let total: f64 = ranked.iter().map(|(_, l)| (l - maximum).exp()).sum();
            let mut draw = rng.uniform() * total;
            let mut chosen = ranked[ranked.len() - 1].0;
            for &(i, l) in &ranked {
                draw -= (l - maximum).exp();
                if draw <= 0.0 {
                    chosen = i;
                    break;
                }
            }
            chosen
        };
        tokens.push(next as u32);
    }
    Ok(tokens[prompt.len()..].to_vec())
}

#[allow(clippy::too_many_arguments)]
fn samples(
    model: &StackModel,
    valid: &[u32],
    decoder: Option<&Decoder>,
    prompts: usize,
    prompt_tokens: usize,
    new_tokens: usize,
    temperature: f64,
    top_k: usize,
    seed: u64,
) -> Result<Value> {
    let mut rng = Rng(seed ^ 0x5341_4D50);
    let stride = (valid.len() - prompt_tokens) / prompts.max(1);
    let mut out = Vec::new();
    for index in 0..prompts {
        let prompt = &valid[index * stride..index * stride + prompt_tokens];
        let greedy = generate(model, prompt, new_tokens, 0.0, 1, &mut rng)?;
        let sampled = generate(model, prompt, new_tokens, temperature, top_k, &mut rng)?;
        let text = |ids: &[u32]| decoder.map(|decoder| decoder.decode(ids));
        out.push(json!({
            "prompt_start": index * stride, "prompt": text(prompt),
            "greedy": text(&greedy), "sampled": text(&sampled),
            "greedy_ids": greedy, "sampled_ids": sampled,
        }));
    }
    Ok(json!({"temperature": temperature, "top_k": top_k, "seed": seed, "rows": out}))
}

struct Progress {
    step: usize,
    rng: Rng,
    curve: Vec<Value>,
    train_seconds: f64,
}

fn save_checkpoint(
    out: &Path,
    model: &StackModel,
    optimizer: &StackAdamW,
    progress: &Progress,
    lineage: &Value,
) -> Result<()> {
    let staging = out.join("checkpoint.partial");
    let _ = fs::remove_dir_all(&staging);
    model.save(&staging.join("model"))?;
    optimizer.save(&staging.join("optimizer"))?;
    fs::write(
        staging.join("state.json"),
        serde_json::to_vec_pretty(&json!({
            "step": progress.step, "rng": progress.rng.0.to_string(), "curve": progress.curve,
            "train_seconds": progress.train_seconds, "lineage": lineage,
        }))?,
    )?;
    let final_path = out.join("checkpoint");
    let _ = fs::remove_dir_all(&final_path);
    fs::rename(&staging, &final_path)?;
    Ok(())
}

fn train(settings: &Settings, out: &Path) -> Result<()> {
    let device = Device::Cpu;
    let lineage = settings.lineage(&settings.input_contents()?);
    let vocabulary = settings.config.vocab_size;
    let train: Vec<Vec<u32>> = settings
        .train
        .iter()
        .map(|path| read_tokens(path, vocabulary))
        .collect::<Result<_>>()?;
    let weight_total: f64 = settings.train_weights.iter().sum();
    let valid = read_tokens(&settings.valid, vocabulary)?;
    let lens = settings
        .lens
        .as_ref()
        .map(|path| read_tokens(path, u16::MAX as usize + 1))
        .transpose()?;
    if lens.as_ref().is_some_and(|lens| lens.len() != vocabulary) {
        return Err(invalid("lens must hold one byte length per token id"));
    }
    let decoder = Decoder::load(settings.merges.as_deref(), settings.tokenizer.as_deref())?;
    let time = settings.config.context;
    if train.iter().any(|stream| stream.len() <= time + 1)
        || valid.len() <= time + settings.final_windows
    {
        return Err(invalid(
            "token files are too short for the context and windows",
        ));
    }
    let (model, mut optimizer, mut progress, resumed_from) = match &settings.resume {
        None => {
            let model = StackModel::new(settings.config.clone(), &device)?;
            let optimizer = StackAdamW::new(&model, settings.weight_decay, settings.clip)?;
            let progress = Progress {
                step: 0,
                rng: Rng(settings.config.seed ^ 0x5749_4E44_4F57),
                curve: Vec::new(),
                train_seconds: 0.0,
            };
            (model, optimizer, progress, None)
        }
        Some(checkpoint) => {
            let state: Value = serde_json::from_slice(&fs::read(checkpoint.join("state.json"))?)?;
            if state["lineage"] != lineage {
                return Err(invalid(
                    "resume settings or input contents differ from the checkpoint",
                ));
            }
            let model = StackModel::load(&checkpoint.join("model"), &device)?;
            let optimizer = StackAdamW::load(&checkpoint.join("optimizer"), &model)?;
            let progress = Progress {
                step: state["step"]
                    .as_u64()
                    .ok_or_else(|| invalid("checkpoint step"))? as usize,
                rng: Rng(state["rng"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .ok_or_else(|| invalid("checkpoint rng"))?),
                curve: state["curve"].as_array().cloned().unwrap_or_default(),
                train_seconds: state["train_seconds"].as_f64().unwrap_or(0.0),
            };
            if optimizer.step != progress.step {
                return Err(invalid("checkpoint optimizer and progress steps differ"));
            }
            (
                model,
                optimizer,
                progress,
                Some(identity(&checkpoint.join("state.json"))?),
            )
        }
    };
    let parameters = model.parameter_count();
    eprintln!(
        "{:?} pattern {} read {:?} rotation {}: {parameters} parameters, mlp {}",
        model.config.arch,
        model.config.pattern,
        model.config.read,
        model.config.rotation,
        model.config.mlp_hidden
    );
    let mut window_loss = (0f64, 0usize);
    let mut stopped_early = false;
    let started = Instant::now();
    while progress.step < settings.steps {
        let lr = learning_rate(settings, progress.step);
        let clock = Instant::now();
        let mut ids = Vec::with_capacity(settings.batch * time);
        let mut targets = Vec::with_capacity(settings.batch * time);
        for _ in 0..settings.batch {
            let mut draw = progress.rng.uniform() * weight_total;
            let mut stream = &train[train.len() - 1];
            for (candidate, weight) in train.iter().zip(&settings.train_weights) {
                draw -= weight;
                if draw <= 0.0 {
                    stream = candidate;
                    break;
                }
            }
            let start = progress.rng.below(stream.len() - time - 1);
            ids.extend_from_slice(&stream[start..start + time]);
            targets.extend_from_slice(&stream[start + 1..start + time + 1]);
        }
        let loss = model.loss(&ids, &targets, settings.batch, time)?;
        let value = f64::from(loss.to_scalar::<f32>()?);
        if !value.is_finite() {
            return Err(invalid(format!(
                "nonfinite training loss at step {}",
                progress.step
            )));
        }
        let grads = loss.backward()?;
        let grad_norm = optimizer.update(&model, &grads, lr)?;
        progress.train_seconds += clock.elapsed().as_secs_f64();
        progress.step += 1;
        window_loss.0 += value;
        window_loss.1 += 1;
        if progress.step % settings.eval_every == 0 || progress.step == settings.steps {
            let evaluation = evaluate(&model, &valid, lens.as_deref(), settings.eval_windows)?;
            let tokens = progress.step * settings.batch * time;
            let point = json!({
                "step": progress.step, "tokens": tokens, "lr": lr,
                "train_loss": window_loss.0 / window_loss.1.max(1) as f64,
                "dev": evaluation.record(), "grad_norm": grad_norm,
                "train_seconds": progress.train_seconds,
                "tokens_per_second": tokens as f64 / progress.train_seconds,
            });
            eprintln!("{point}");
            progress.curve.push(point);
            window_loss = (0.0, 0);
        }
        if settings.checkpoint_every > 0
            && progress.step % settings.checkpoint_every == 0
            && progress.step < settings.steps
        {
            save_checkpoint(out, &model, &optimizer, &progress, &lineage)?;
        }
        if started.elapsed().as_secs_f64() > settings.max_seconds {
            stopped_early = true;
            save_checkpoint(out, &model, &optimizer, &progress, &lineage)?;
            break;
        }
    }
    let final_evaluation = evaluate(&model, &valid, lens.as_deref(), settings.final_windows)?;
    eprintln!("final: {}", final_evaluation.record());
    model.save(&out.join("model"))?;
    if !stopped_early {
        // A completed run keeps only its final model; a stopped one keeps the
        // checkpoint that `resume=` continues from.
        let _ = fs::remove_dir_all(out.join("checkpoint"));
    }
    let sample_record = if settings.sample_tokens > 0 {
        samples(
            &model,
            &valid,
            decoder.as_ref(),
            3,
            64,
            settings.sample_tokens,
            0.8,
            40,
            settings.config.seed,
        )?
    } else {
        Value::Null
    };
    let executable = std::env::current_exe()?;
    let report = json!({
        "schema": "uor-r4.geometric-stack-run/1",
        "settings": settings.record(),
        "parameters": parameters,
        "completed_steps": progress.step,
        "stopped_early": stopped_early,
        "resumed_from": resumed_from,
        "target_visits": progress.step * settings.batch * time,
        "train_seconds": progress.train_seconds,
        "tokens_per_second": (progress.step * settings.batch * time) as f64 / progress.train_seconds.max(1e-9),
        "threads": std::env::var("RAYON_NUM_THREADS").ok(),
        "curve": progress.curve,
        "final": final_evaluation.record(),
        "samples": sample_record,
        "inputs": {
            "train": settings.train.iter().map(|p| identity(p)).collect::<Result<Vec<_>>>()?,
            "valid": identity(&settings.valid)?,
            "lens": settings.lens.as_ref().map(|p| identity(p)).transpose()?,
            "merges": settings.merges.as_ref().map(|p| identity(p)).transpose()?,
            "tokenizer": settings.tokenizer.as_ref().map(|p| identity(p)).transpose()?,
        },
        "executable": identity(&executable)?,
        "model_sha256": sha256_file(&out.join("model").join("model.safetensors"))?,
    });
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

fn sample_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(
        arguments,
        &[
            "model",
            "valid",
            "merges",
            "tokenizer",
            "out",
            "prompts",
            "prompt_tokens",
            "sample_tokens",
            "temperature",
            "top_k",
            "seed",
        ],
    )?;
    let model_dir = PathBuf::from(args.required("model")?);
    let valid_path = PathBuf::from(args.required("valid")?);
    let merges = args.optional("merges").map(PathBuf::from);
    let tokenizer = args.optional("tokenizer").map(PathBuf::from);
    if merges.is_none() && tokenizer.is_none() {
        return Err(invalid("sample needs merges= or tokenizer="));
    }
    let out = PathBuf::from(args.required("out")?);
    let prompts: usize = args.number("prompts", 3)?;
    let prompt_tokens: usize = args.number("prompt_tokens", 64)?;
    let sample_tokens: usize = args.number("sample_tokens", 128)?;
    let temperature: f64 = args.number("temperature", 0.8)?;
    let top_k: usize = args.number("top_k", 40)?;
    let seed: u64 = args.number("seed", 1)?;
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        let model = StackModel::load(&model_dir, &Device::Cpu)?;
        let valid = read_tokens(&valid_path, model.config.vocab_size)?;
        let decoder = Decoder::load(merges.as_deref(), tokenizer.as_deref())?;
        let record = samples(
            &model,
            &valid,
            decoder.as_ref(),
            prompts,
            prompt_tokens,
            sample_tokens,
            temperature,
            top_k,
            seed,
        )?;
        fs::write(
            out.join("samples.json"),
            serde_json::to_vec_pretty(&json!({
                "model": identity(&model_dir.join("model.safetensors"))?, "config": model.config,
                "samples": record,
            }))?,
        )?;
        Ok(())
    })();
    finish(&out, result)
}

/// Consecutive 256-input blocks, fresh state per block: the retained
/// evaluator's protocol. Reports tune, comparison and full means.
fn evaluate_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(
        arguments,
        &["model", "tokens", "out", "tune_blocks", "lens"],
    )?;
    let model_dir = PathBuf::from(args.required("model")?);
    let tokens_path = PathBuf::from(args.required("tokens")?);
    let lens_path = args.optional("lens").map(PathBuf::from);
    let out = PathBuf::from(args.required("out")?);
    let tune_blocks: usize = args.number("tune_blocks", 64)?;
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        let model = StackModel::load(&model_dir, &Device::Cpu)?;
        let time = model.config.context;
        let tokens = read_tokens(&tokens_path, model.config.vocab_size)?;
        let lens = lens_path
            .as_ref()
            .map(|path| read_tokens(path, u16::MAX as usize + 1))
            .transpose()?;
        let blocks = (tokens.len() - 1) / time;
        if blocks <= tune_blocks {
            return Err(invalid("fewer blocks than tune_blocks"));
        }
        let mut block_nll = Vec::with_capacity(blocks);
        let mut block_bytes = Vec::with_capacity(blocks);
        for group in (0..blocks).collect::<Vec<_>>().chunks(16) {
            let mut ids = Vec::with_capacity(group.len() * time);
            let mut targets = Vec::with_capacity(group.len() * time);
            for &block in group {
                ids.extend_from_slice(&tokens[block * time..(block + 1) * time]);
                targets.extend_from_slice(&tokens[block * time + 1..(block + 1) * time + 1]);
            }
            let nll = model.target_nll(&ids, &targets, group.len(), time)?;
            for (index, _) in group.iter().enumerate() {
                block_nll.push(nll[index * time..(index + 1) * time].iter().sum::<f64>());
                block_bytes.push(lens.as_ref().map(|lens| {
                    targets[index * time..(index + 1) * time]
                        .iter()
                        .map(|&id| f64::from(lens[id as usize]))
                        .sum::<f64>()
                }));
            }
        }
        let summary = |range: std::ops::Range<usize>| {
            let targets = range.len() * time;
            let nll: f64 = block_nll[range.clone()].iter().sum();
            let bytes: Option<f64> = block_bytes[range].iter().copied().sum();
            json!({
                "blocks": targets / time, "targets": targets, "nll": nll / targets as f64,
                "bits_per_byte": bytes.map(|bytes| nll / std::f64::consts::LN_2 / bytes),
            })
        };
        fs::write(
            out.join("evaluation.json"),
            serde_json::to_vec_pretty(&json!({
                "schema": "uor-r4.geometric-stack-evaluation/1",
                "protocol": "consecutive blocks of 256 inputs and 256 shifted targets, fresh state per block",
                "model": identity(&model_dir.join("model.safetensors"))?,
                "config": model.config,
                "tokens": identity(&tokens_path)?,
                "lens": lens_path.as_ref().map(|p| identity(p)).transpose()?,
                "tune": summary(0..tune_blocks),
                "comparison": summary(tune_blocks..blocks),
                "full": summary(0..blocks),
                "block_nll": block_nll,
            }))?,
        )?;
        Ok(())
    })();
    finish(&out, result)
}

/// The integer serving artifact of a trained model (owner decision D10).
fn export_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(arguments, &["model", "out"])?;
    let model_dir = PathBuf::from(args.required("model")?);
    let out = PathBuf::from(args.required("out")?);
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        let model = StackModel::load(&model_dir, &Device::Cpu)?;
        let weights = model_dir.join("model.safetensors");
        let source = json!({
            "exporter": "geometric-stack export",
            "model": identity(&weights)?,
            "config": model.config,
            "executable": identity(&std::env::current_exe()?)?,
        });
        let (bytes, report) = match model.config.arch {
            StackArch::Geometric => export_stack(&model, source.clone())?,
            StackArch::Transformer => {
                let checkpoint = control_checkpoint(&model, sha256_file(&weights)?)?;
                export_llama(
                    &checkpoint,
                    model.config.context,
                    source.clone(),
                    None,
                    None,
                )?
            }
        };
        let artifact = out.join("model.lut");
        fs::write(&artifact, &bytes)?;
        fs::write(
            out.join("export.json"),
            serde_json::to_vec_pretty(&json!({
                "schema": "uor-r4.geometric-stack-export/1",
                "source": source,
                "artifact": identity(&artifact)?,
                "quantization": report,
            }))?,
        )?;
        Ok(())
    })();
    finish(&out, result)
}

/// Either integer engine, behind one step function.
enum Engine {
    Stack(Box<uor_r4_lut::stack::StackModel>),
    Llama(Box<uor_r4_lut::engine::Model>),
}

/// An integer decoding session of either engine.
trait Stepper {
    /// Feed one id; the next-token logits at exponent -16.
    fn advance(&mut self, id: u32) -> Result<&[i32]>;
}

impl Stepper for uor_r4_lut::stack::StackSession<'_> {
    fn advance(&mut self, id: u32) -> Result<&[i32]> {
        self.step(id).map_err(lut)
    }
}

impl Stepper for uor_r4_lut::engine::Session<'_> {
    fn advance(&mut self, id: u32) -> Result<&[i32]> {
        self.step(id).map_err(lut)
    }
}

/// Feed `prompt`, then draw `new_tokens` ids with `sampler`.
fn continue_prompt(
    session: &mut impl Stepper,
    prompt: &[u32],
    new_tokens: usize,
    sampler: &mut uor_r4_lut::sampling::Sampler,
    exp: (&[u32], i32),
) -> Result<Vec<u32>> {
    let mut seen = prompt.to_vec();
    let mut logits = Vec::new();
    for &id in prompt {
        logits = session.advance(id)?.to_vec();
    }
    let mut generated = Vec::with_capacity(new_tokens);
    for step in 0..new_tokens {
        let next = sampler.sample(&logits, &seen, exp.0, exp.1).map_err(lut)?;
        generated.push(next);
        seen.push(next);
        if step + 1 < new_tokens {
            logits = session.advance(next)?.to_vec();
        }
    }
    Ok(generated)
}

impl Engine {
    fn load(bytes: Vec<u8>, threads: usize) -> Result<Self> {
        use uor_r4_lut::format::{schema_of, Artifact, StackArtifact, SCHEMA, STACK_SCHEMA};
        let schema = schema_of(&bytes).map_err(lut)?;
        Ok(if schema == STACK_SCHEMA {
            let mut model = uor_r4_lut::stack::StackModel::from_artifact(
                StackArtifact::parse(bytes).map_err(lut)?,
            )
            .map_err(lut)?;
            model.set_threads(threads).map_err(lut)?;
            Self::Stack(Box::new(model))
        } else if schema == SCHEMA {
            let mut model =
                uor_r4_lut::engine::Model::from_artifact(Artifact::parse(bytes).map_err(lut)?)
                    .map_err(lut)?;
            model.set_threads(threads).map_err(lut)?;
            Self::Llama(Box::new(model))
        } else {
            return Err(invalid(format!("unknown artifact schema {schema}")));
        })
    }

    fn vocabulary(&self) -> usize {
        match self {
            Self::Stack(model) => model.shape().vocab,
            Self::Llama(model) => model.shape().vocab,
        }
    }

    fn context(&self) -> usize {
        match self {
            Self::Stack(model) => model.shape().context,
            Self::Llama(model) => model.shape().max_positions,
        }
    }

    fn sha256(&self) -> &str {
        match self {
            Self::Stack(model) => model.artifact_sha256(),
            Self::Llama(model) => model.artifact_sha256(),
        }
    }

    fn backend(&self) -> &'static str {
        match self {
            Self::Stack(model) => model.backend().name(),
            Self::Llama(model) => model.backend().name(),
        }
    }

    /// Logits (exponent -16) after each id of one window, from a fresh session.
    fn window(&self, ids: &[u32], mut visit: impl FnMut(usize, &[i32])) -> Result<()> {
        fn run(
            session: &mut impl Stepper,
            ids: &[u32],
            visit: &mut impl FnMut(usize, &[i32]),
        ) -> Result<()> {
            for (t, &id) in ids.iter().enumerate() {
                visit(t, session.advance(id)?);
            }
            Ok(())
        }
        match self {
            Self::Stack(model) => run(&mut model.session(), ids, &mut visit),
            Self::Llama(model) => run(&mut model.session(), ids, &mut visit),
        }
    }

    /// A continuation of `prompt` from a fresh session.
    fn generate(
        &self,
        prompt: &[u32],
        new_tokens: usize,
        sampler: &mut uor_r4_lut::sampling::Sampler,
    ) -> Result<Vec<u32>> {
        match self {
            Self::Stack(model) => continue_prompt(
                &mut model.session(),
                prompt,
                new_tokens,
                sampler,
                model.exp_table(),
            ),
            Self::Llama(model) => continue_prompt(
                &mut model.session(),
                prompt,
                new_tokens,
                sampler,
                model.exp_table(),
            ),
        }
    }
}

/// `-log softmax(logits)[target]` and the argmax, in f64 (evaluation only).
fn score_row(logits: impl Iterator<Item = f64> + Clone, target: usize) -> (f64, usize) {
    let (mut best, mut max) = (0usize, f64::NEG_INFINITY);
    for (i, v) in logits.clone().enumerate() {
        if v > max {
            (best, max) = (i, v);
        }
    }
    let total: f64 = logits.clone().map(|v| (v - max).exp()).sum();
    let at = logits.clone().nth(target).unwrap_or(f64::NEG_INFINITY);
    (max + total.ln() - at, best)
}

/// Integer serving on development windows, beside the float model on the same
/// windows when `model=` is given: evenly spaced windows (`train`'s rule), or
/// with `blocks=true` the retained evaluator's consecutive blocks with its
/// tune/comparison split.
fn lut_evaluate_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(
        arguments,
        &[
            "artifact",
            "valid",
            "model",
            "windows",
            "blocks",
            "tune_blocks",
            "threads",
            "lens",
            "out",
        ],
    )?;
    let artifact_path = PathBuf::from(args.required("artifact")?);
    let valid_path = PathBuf::from(args.required("valid")?);
    let model_dir = args.optional("model").map(PathBuf::from);
    let lens_path = args.optional("lens").map(PathBuf::from);
    let windows: usize = args.number("windows", 64)?;
    let blocks: bool = args.number("blocks", false)?;
    let tune_blocks: usize = args.number("tune_blocks", 64)?;
    let threads: usize = args.number("threads", 1)?;
    let out = PathBuf::from(args.required("out")?);
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        let engine = Engine::load(fs::read(&artifact_path)?, threads)?;
        let time = engine.context();
        let valid = read_tokens(&valid_path, engine.vocabulary())?;
        let lens = lens_path
            .as_ref()
            .map(|path| read_tokens(path, u16::MAX as usize + 1))
            .transpose()?;
        let float = model_dir
            .as_ref()
            .map(|dir| StackModel::load(dir, &Device::Cpu))
            .transpose()?;
        if let Some(model) = &float {
            if model.config.vocab_size != engine.vocabulary() || model.config.context != time {
                return Err(invalid("the float model and the artifact differ in shape"));
            }
        }
        let starts: Vec<usize> = if blocks {
            let count = (valid.len() - 1) / time;
            if count <= tune_blocks {
                return Err(invalid("fewer blocks than tune_blocks"));
            }
            (0..count).map(|block| block * time).collect()
        } else {
            if windows == 0 || valid.len() <= time + windows {
                return Err(invalid("too few development tokens for the windows"));
            }
            let stride = (valid.len() - time - 1) / windows;
            (0..windows).map(|window| window * stride).collect()
        };
        // Per window: integer NLL, float NLL, target bytes (sums over targets).
        let mut sums: Vec<(f64, f64, f64)> = Vec::with_capacity(starts.len());
        let (mut agree, mut integer_seconds) = (0usize, 0f64);
        for &start in &starts {
            let ids = &valid[start..start + time];
            let next = &valid[start + 1..start + time + 1];
            let mut window_nll = 0f64;
            let mut integer_top = vec![0usize; time];
            let clock = Instant::now();
            engine.window(ids, |t, logits| {
                let (nll, top) = score_row(
                    logits.iter().map(|&v| f64::from(v) / 65536.0),
                    next[t] as usize,
                );
                window_nll += nll;
                integer_top[t] = top;
            })?;
            integer_seconds += clock.elapsed().as_secs_f64();
            let mut window_float = 0f64;
            if let Some(model) = &float {
                let logits = model.forward(ids, 1, time)?.to_vec2::<f32>()?;
                for (t, row) in logits.iter().enumerate() {
                    let (nll, top) = score_row(row.iter().map(|&v| f64::from(v)), next[t] as usize);
                    window_float += nll;
                    agree += usize::from(top == integer_top[t]);
                }
            }
            let bytes = lens.as_ref().map_or(0.0, |lens| {
                next.iter()
                    .map(|&id| f64::from(lens[id as usize]))
                    .sum::<f64>()
            });
            sums.push((window_nll, window_float, bytes));
        }
        let summary = |range: std::ops::Range<usize>| -> Value {
            let part = &sums[range];
            let targets = (part.len() * time) as f64;
            let (integer, float_sum, bytes) = part
                .iter()
                .fold((0.0, 0.0, 0.0), |a, s| (a.0 + s.0, a.1 + s.1, a.2 + s.2));
            let bits = |nll: f64| lens.as_ref().map(|_| nll / std::f64::consts::LN_2 / bytes);
            json!({
                "windows": part.len(), "targets": part.len() * time,
                "integer": {"nll": integer / targets, "bits_per_byte": bits(integer)},
                "float": float.as_ref().map(|_| json!({"nll": float_sum / targets, "bits_per_byte": bits(float_sum)})),
                "integer_minus_float_nll": float.as_ref().map(|_| (integer - float_sum) / targets),
            })
        };
        let targets = starts.len() * time;
        let mut record = json!({
            "schema": "uor-r4.geometric-stack-lut-evaluation/2",
            "protocol": if blocks {
                "consecutive blocks of 256 inputs and 256 shifted targets (the retained evaluator's), one fresh integer session per block, position by position"
            } else {
                "evenly spaced windows of the development tokens (train's evaluation rule), one fresh integer session per window, position by position"
            },
            "artifact": identity(&artifact_path)?,
            "artifact_sha256": engine.sha256(),
            "float_model": model_dir.as_ref().map(|dir| identity(&dir.join("model.safetensors"))).transpose()?,
            "valid": identity(&valid_path)?,
            "windows": starts.len(),
            "targets": targets,
            "all": summary(0..starts.len()),
            "engine": {
                "seconds": integer_seconds,
                "tokens_per_second": targets as f64 / integer_seconds,
                "threads": threads,
                "backend": engine.backend(),
            },
            "top1_agreement": float.as_ref().map(|_| agree as f64 / targets as f64),
            "per_window": starts.iter().zip(&sums).map(|(start, s)| json!({
                "start": start, "integer_nll": s.0 / time as f64,
                "float_nll": float.as_ref().map(|_| s.1 / time as f64),
            })).collect::<Vec<_>>(),
        });
        if blocks {
            record["tune"] = summary(0..tune_blocks);
            record["comparison"] = summary(tune_blocks..starts.len());
        }
        fs::write(
            out.join("evaluation.json"),
            serde_json::to_vec_pretty(&record)?,
        )?;
        Ok(())
    })();
    finish(&out, result)
}

/// Continuations generated by an integer engine, greedy and sampled, from
/// evenly spaced development prompts or from `prompt=` text.
fn lut_sample_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(
        arguments,
        &[
            "artifact",
            "valid",
            "merges",
            "tokenizer",
            "prompt",
            "out",
            "prompts",
            "prompt_tokens",
            "sample_tokens",
            "temperature",
            "top_k",
            "top_p",
            "seed",
            "threads",
        ],
    )?;
    let artifact_path = PathBuf::from(args.required("artifact")?);
    let valid_path = args.optional("valid").map(PathBuf::from);
    let merges = args.optional("merges").map(PathBuf::from);
    let tokenizer = args.optional("tokenizer").map(PathBuf::from);
    let prompt_text = args.optional("prompt");
    let out = PathBuf::from(args.required("out")?);
    let prompts: usize = args.number("prompts", 3)?;
    let prompt_tokens: usize = args.number("prompt_tokens", 64)?;
    let sample_tokens: usize = args.number("sample_tokens", 128)?;
    let temperature: f64 = args.number("temperature", 0.8)?;
    let top_k: usize = args.number("top_k", 40)?;
    let top_p: f64 = args.number("top_p", 1.0)?;
    let seed: u64 = args.number("seed", 1)?;
    let threads: usize = args.number("threads", 1)?;
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        use uor_r4_lut::sampling::{Sampler, SamplingSettings};
        let engine = Engine::load(fs::read(&artifact_path)?, threads)?;
        let decoder = Decoder::load(merges.as_deref(), tokenizer.as_deref())?;
        let prompt_ids: Vec<(Option<usize>, Vec<u32>)> = match (&prompt_text, &valid_path) {
            (Some(text), _) => {
                let decoder = decoder
                    .as_ref()
                    .ok_or_else(|| invalid("a text prompt needs merges= or tokenizer="))?;
                vec![(None, decoder.encode(text))]
            }
            (None, Some(path)) => {
                let valid = read_tokens(path, engine.vocabulary())?;
                if prompts == 0 || valid.len() < prompt_tokens + prompts {
                    return Err(invalid("too few development tokens for the prompts"));
                }
                let stride = (valid.len() - prompt_tokens) / prompts;
                (0..prompts)
                    .map(|index| {
                        let start = index * stride;
                        (Some(start), valid[start..start + prompt_tokens].to_vec())
                    })
                    .collect()
            }
            (None, None) => return Err(invalid("give prompt= text or valid= prompts")),
        };
        let greedy_settings = SamplingSettings::default();
        let sampled_settings =
            SamplingSettings::from_decimal(temperature, top_k, top_p, 0.0).map_err(lut)?;
        let text = |ids: &[u32]| decoder.as_ref().map(|decoder| decoder.decode(ids));
        let (mut rows, mut generated, mut seconds) = (Vec::new(), 0usize, 0f64);
        for (index, (start, prompt)) in prompt_ids.iter().enumerate() {
            if prompt.is_empty() || prompt.len() + sample_tokens > engine.context() {
                return Err(invalid("the prompt and continuation exceed the context"));
            }
            let clock = Instant::now();
            let greedy = engine.generate(
                prompt,
                sample_tokens,
                &mut Sampler::new(greedy_settings, seed),
            )?;
            let sampled = engine.generate(
                prompt,
                sample_tokens,
                &mut Sampler::new(sampled_settings, seed.wrapping_add(index as u64)),
            )?;
            seconds += clock.elapsed().as_secs_f64();
            generated += 2 * (prompt.len() + sample_tokens);
            rows.push(json!({
                "prompt_start": start, "prompt": text(prompt), "prompt_ids": prompt,
                "greedy": text(&greedy), "sampled": text(&sampled),
                "greedy_ids": greedy, "sampled_ids": sampled,
            }));
        }
        fs::write(
            out.join("samples.json"),
            serde_json::to_vec_pretty(&json!({
                "schema": "uor-r4.geometric-stack-lut-samples/1",
                "artifact": identity(&artifact_path)?,
                "artifact_sha256": engine.sha256(),
                "settings": {"temperature": temperature, "top_k": top_k, "top_p": top_p, "seed": seed},
                "engine": {"positions": generated, "seconds": seconds,
                    "positions_per_second": generated as f64 / seconds,
                    "threads": threads, "backend": engine.backend()},
                "rows": rows,
            }))?,
        )?;
        Ok(())
    })();
    finish(&out, result)
}

fn finish(out: &Path, result: Result<()>) -> Result<()> {
    if let Err(error) = &result {
        fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error": error.to_string()}))?,
        )?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    result
}

fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some((mode, rest)) = arguments.split_first() else {
        return Err(invalid(
            "usage: geometric-stack train|sample|evaluate|encode|corpus|export|lut-evaluate|lut-sample key=value ...",
        ));
    };
    match mode.as_str() {
        "train" => {
            let args = Args::parse(
                rest,
                &[
                    "train",
                    "train_weights",
                    "width",
                    "heads",
                    "layers",
                    "mlp",
                    "context",
                    "valid",
                    "out",
                    "arch",
                    "pattern",
                    "read",
                    "rotation",
                    "seed",
                    "steps",
                    "batch",
                    "lr",
                    "warmup",
                    "min_lr",
                    "weight_decay",
                    "clip",
                    "eval_every",
                    "eval_windows",
                    "final_windows",
                    "lens",
                    "merges",
                    "tokenizer",
                    "checkpoint_every",
                    "resume",
                    "max_seconds",
                    "sample_tokens",
                ],
            )?;
            let settings = train_settings(&args)?;
            let out = PathBuf::from(args.required("out")?);
            report_output::claim(&out)?;
            let result = train(&settings, &out);
            finish(&out, result)
        }
        "sample" => sample_mode(rest),
        "evaluate" => evaluate_mode(rest),
        "encode" => encode_mode(rest),
        "corpus" => corpus_mode(rest),
        "export" => export_mode(rest),
        "lut-evaluate" => lut_evaluate_mode(rest),
        "lut-sample" => lut_sample_mode(rest),
        other => Err(invalid(format!("unknown mode {other}"))),
    }
}
