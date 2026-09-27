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
//!   [sample_tokens=128]
//! geometric-stack sample model=ROOT/model valid=VALID.u16 merges=MERGES.txt out=NEW_REPORT_ROOT \
//!   [prompts=3] [prompt_tokens=64] [sample_tokens=128] [temperature=0.8] [top_k=40] [seed=1]
//! geometric-stack encode merges=MERGES.txt input=TEXT out=TOKENS.u16
//! geometric-stack corpus registry=CARGO_REGISTRY_SRC_INDEX out=TEXT [max_file_bytes=200000]
//! ```
//!
//! `train` samples windows uniformly from TRAIN with a seeded generator, so two
//! arms with one seed see identical windows. Evaluation windows are evenly
//! spaced over VALID, the rule of `joint-read-geometry`, so `final_windows=512`
//! at context 256 scores the same 131,072 targets as that example. A stopped run
//! continues with `resume=OLD_ROOT/checkpoint` in a new root, with identical
//! settings. Every report root is claimed before loading and sealed at the end.
//! Set RAYON_NUM_THREADS to bound the threads.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_training::geometric_stack::{ReadScore, StackAdamW, StackConfig, StackModel};
use uor_r4_training::{sha256_file, Result, TrainingError};

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
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
            "lens": self.lens, "merges": self.merges,
            "config": self.config, "steps": self.steps, "batch": self.batch, "lr": self.lr,
            "warmup": self.warmup, "min_lr": self.min_lr, "weight_decay": self.weight_decay,
            "clip": self.clip, "eval_every": self.eval_every, "eval_windows": self.eval_windows,
            "final_windows": self.final_windows, "checkpoint_every": self.checkpoint_every,
            "sample_tokens": self.sample_tokens,
        })
    }

    /// Settings a resumed run must share with its parent.
    fn lineage(&self) -> Value {
        json!({
            "config": self.config, "steps": self.steps, "batch": self.batch, "lr": self.lr,
            "warmup": self.warmup, "min_lr": self.min_lr, "weight_decay": self.weight_decay,
            "clip": self.clip, "eval_every": self.eval_every, "eval_windows": self.eval_windows,
            "train_weights": self.train_weights,
        })
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
    let config = match args.required("arch")?.as_str() {
        "transformer" => StackConfig::transformer_control(seed),
        "geometric" => StackConfig::geometric_matched(
            &args.optional("pattern").unwrap_or_else(|| "rrarra".into()),
            read,
            rotation,
            seed,
        )?,
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
    bpe: Option<&Bpe>,
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
        let text = |ids: &[u32]| bpe.map(|bpe| bpe.decode(ids));
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
    settings: &Settings,
) -> Result<()> {
    let staging = out.join("checkpoint.partial");
    let _ = fs::remove_dir_all(&staging);
    model.save(&staging.join("model"))?;
    optimizer.save(&staging.join("optimizer"))?;
    fs::write(
        staging.join("state.json"),
        serde_json::to_vec_pretty(&json!({
            "step": progress.step, "rng": progress.rng.0.to_string(), "curve": progress.curve,
            "train_seconds": progress.train_seconds, "lineage": settings.lineage(),
        }))?,
    )?;
    let final_path = out.join("checkpoint");
    let _ = fs::remove_dir_all(&final_path);
    fs::rename(&staging, &final_path)?;
    Ok(())
}

fn train(settings: &Settings, out: &Path) -> Result<()> {
    let device = Device::Cpu;
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
    let bpe = settings
        .merges
        .as_ref()
        .map(|path| Bpe::load(path))
        .transpose()?;
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
            if state["lineage"] != settings.lineage() {
                return Err(invalid("resume settings differ from the checkpoint"));
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
            save_checkpoint(out, &model, &optimizer, &progress, settings)?;
        }
        if started.elapsed().as_secs_f64() > settings.max_seconds {
            stopped_early = true;
            save_checkpoint(out, &model, &optimizer, &progress, settings)?;
            break;
        }
    }
    let final_evaluation = evaluate(&model, &valid, lens.as_deref(), settings.final_windows)?;
    eprintln!("final: {}", final_evaluation.record());
    model.save(&out.join("model"))?;
    let _ = fs::remove_dir_all(out.join("checkpoint"));
    let sample_record = if settings.sample_tokens > 0 {
        samples(
            &model,
            &valid,
            bpe.as_ref(),
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
    let merges = PathBuf::from(args.required("merges")?);
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
        let bpe = Bpe::load(&merges)?;
        let record = samples(
            &model,
            &valid,
            Some(&bpe),
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
            "usage: geometric-stack train|sample|encode|corpus key=value ...",
        ));
    };
    match mode.as_str() {
        "train" => {
            let args = Args::parse(
                rest,
                &[
                    "train",
                    "train_weights",
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
        "encode" => encode_mode(rest),
        "corpus" => corpus_mode(rest),
        other => Err(invalid(format!("unknown mode {other}"))),
    }
}
