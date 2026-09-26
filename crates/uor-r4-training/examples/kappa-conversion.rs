//! Curvature-homotopy conversion of a Llama checkpoint (for example
//! SmolLM2-135M/360M-Instruct) to hyperbolic (Lorentz) attention.
//!
//! Offline float tool; see `uor_r4_training::kappa_llama` for the score family.
//! Modes (arguments are key=value):
//!
//! ```text
//! kappa-conversion mode=tokenize model=DIR text=IN.txt out=OUT.u16
//! kappa-conversion mode=probe model=DIR tokens=X.u16 out=NEW_ROOT
//!     [score=intrinsic|key_norm] [batch=2] [time=128] [log_eps=-12,-10,-8,-6,-5,-4,-3,-2,-1,0]
//! kappa-conversion mode=train student=DIR train=X.u16 valid=Y.u16 out=NEW_ROOT
//!     [teacher=DIR] [score=intrinsic|key_norm|dot] [init_log_eps=-4.6] [trainable=scalars|query_key]
//!     [steps=500] [batch=4] [time=256] [lr=1e-5] [curv_lr=1e-2] [eval_every=100]
//!     [eval_windows=16] [seed=1] [max_seconds=0] [save_model=true]
//!     [anneal_to=LOG_EPS] [anneal_steps=steps/2] [anneal_hold=true]
//! kappa-conversion mode=sample model=DIR prompt=TEXT [variables=ROOT/model/variables.safetensors]
//!     [score=dot] [trainable=scalars|query_key] [tokens=64]
//! ```
//!
//! Every mode also accepts `device=cpu|metal` (metal needs `--features metal`).
//! `train` minimizes KL(teacher || student) when `teacher=` is given (the
//! teacher runs the Dot score) and next-token loss on `train=` otherwise. A
//! `score=dot` run is the matched control: same data order, same per-head
//! temperatures and trainable weights, no curvature. An exact flat-limit start
//! can stay in the flat basin, so `anneal_to=` raises a floor on every head's
//! `log_eps` linearly from `init_log_eps` to `anneal_to` over `anneal_steps`,
//! then holds it (or releases it with `anneal_hold=false`).

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{Device, Tensor};
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;
use uor_r4_training::joint_optimizer::{AdamConfig, NamedAdamW};
use uor_r4_training::kappa_llama::{
    distillation_kl, load_checkpoint, max_abs_difference, next_token_nll, Checkpoint, KappaLlama,
    ScoreKind, Trainable, LOG_BETA, LOG_EPS,
};
use uor_r4_training::{Result, TrainingError};

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

struct Args(BTreeMap<String, String>);
impl Args {
    fn text(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }
    fn required(&self, key: &str) -> Result<&str> {
        self.text(key)
            .ok_or_else(|| invalid(format!("missing {key}=")))
    }
    fn number<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T> {
        match self.text(key) {
            None => Ok(default),
            Some(value) => value
                .parse()
                .map_err(|_| invalid(format!("invalid {key}={value}"))),
        }
    }
}

/// Counter-based window sampler (SplitMix64).
struct Windows(u64);
impl Windows {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

fn device(args: &Args) -> Result<Device> {
    match args.text("device").unwrap_or("cpu") {
        "cpu" => Ok(Device::Cpu),
        "metal" => Ok(Device::new_metal(0)?),
        other => Err(invalid(format!("device must be cpu or metal, not {other}"))),
    }
}

fn read_tokens(path: &Path, vocab: usize) -> Result<Vec<u32>> {
    let bytes = fs::read(path)?;
    if bytes.len() % 2 != 0 {
        return Err(invalid(format!(
            "{} is not a u16 token file",
            path.display()
        )));
    }
    let tokens: Vec<u32> = bytes
        .chunks_exact(2)
        .map(|b| u32::from(u16::from_le_bytes([b[0], b[1]])))
        .collect();
    if tokens.iter().any(|&t| t as usize >= vocab) {
        return Err(invalid(format!(
            "{} has a token outside the vocabulary",
            path.display()
        )));
    }
    Ok(tokens)
}

/// `batch` windows of `time + 1` tokens: inputs and next-token targets.
fn windows(tokens: &[u32], starts: &[usize], time: usize) -> (Vec<u32>, Vec<u32>) {
    let mut inputs = Vec::with_capacity(starts.len() * time);
    let mut targets = Vec::with_capacity(starts.len() * time);
    for &start in starts {
        inputs.extend_from_slice(&tokens[start..start + time]);
        targets.extend_from_slice(&tokens[start + 1..start + time + 1]);
    }
    (inputs, targets)
}

fn evenly_spaced(len: usize, time: usize, count: usize) -> Result<Vec<usize>> {
    if len < time + 2 || count == 0 {
        return Err(invalid("token file shorter than one window"));
    }
    let span = len - time - 1;
    Ok((0..count).map(|i| i * span / count).collect())
}

fn scalar(tensor: &Tensor) -> Result<f64> {
    Ok(f64::from(tensor.to_scalar::<f32>()?))
}

fn summary(grid: &[Vec<f32>]) -> Value {
    let mut values: Vec<f32> = grid.iter().flatten().copied().collect();
    if values.is_empty() {
        return Value::Null;
    }
    values.sort_by(f32::total_cmp);
    let n = values.len();
    let above = |threshold: f32| values.iter().filter(|&&v| v > threshold).count();
    json!({
        "heads": n,
        "min": values[0],
        "median": values[n / 2],
        "max": values[n - 1],
        "above_1e-3": above(1e-3),
        "above_1e-2": above(1e-2),
        "above_1e-1": above(1e-1),
    })
}

fn tokenize(args: &Args) -> Result<()> {
    let model = PathBuf::from(args.required("model")?);
    let text = fs::read_to_string(args.required("text")?)?;
    let out = PathBuf::from(args.required("out")?);
    if out.exists() {
        return Err(invalid(format!(
            "{} exists; token files are never overwritten",
            out.display()
        )));
    }
    let tokenizer =
        HfBpeTokenizer::from_dir(&model).map_err(|error| invalid(format!("tokenizer: {error}")))?;
    if tokenizer.vocab_size() > usize::from(u16::MAX) + 1 {
        return Err(invalid("vocabulary does not fit u16 token files"));
    }
    let tokens = tokenizer.encode(&text);
    let mut bytes = Vec::with_capacity(tokens.len() * 2);
    for token in &tokens {
        let token = u16::try_from(*token).map_err(|_| invalid("token id exceeds u16"))?;
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    fs::write(&out, bytes)?;
    eprintln!(
        "tokenized {} bytes into {} tokens ({:.3} bytes/token)",
        text.len(),
        tokens.len(),
        text.len() as f64 / tokens.len().max(1) as f64
    );
    Ok(())
}

struct ProbeSettings {
    device: Device,
    model: PathBuf,
    tokens: PathBuf,
    score: ScoreKind,
    batch: usize,
    time: usize,
    grid: Vec<f32>,
}

fn probe_settings(args: &Args) -> Result<ProbeSettings> {
    let score = ScoreKind::parse(args.text("score").unwrap_or("intrinsic"))?;
    if !score.is_curved() {
        return Err(invalid("probe compares a curved score with dot"));
    }
    let grid: Vec<f32> = args
        .text("log_eps")
        .unwrap_or("-12,-10,-8,-6,-5,-4,-3,-2,-1,0")
        .split(',')
        .map(|v| {
            v.parse()
                .map_err(|_| invalid(format!("invalid log_eps value {v}")))
        })
        .collect::<Result<_>>()?;
    if grid.is_empty()
        || grid
            .iter()
            .any(|v| !v.is_finite() || !(-20.0..=5.0).contains(v))
    {
        return Err(invalid("log_eps values must be finite and in [-20, 5]"));
    }
    let settings = ProbeSettings {
        device: device(args)?,
        model: PathBuf::from(args.required("model")?),
        tokens: PathBuf::from(args.required("tokens")?),
        score,
        batch: args.number("batch", 2)?,
        time: args.number("time", 128)?,
        grid,
    };
    if settings.batch == 0 || settings.time < 2 {
        return Err(invalid("batch must be positive and time >= 2"));
    }
    Ok(settings)
}

fn probe(settings: &ProbeSettings, out: &Path) -> Result<()> {
    let started = Instant::now();
    let ProbeSettings {
        device,
        model: model_dir,
        score,
        batch,
        time,
        grid,
        ..
    } = settings;
    let (device, score, batch, time) = (device.clone(), *score, *batch, *time);
    let checkpoint = load_checkpoint(model_dir, &device)?;
    let identity = json!({"model": model_dir, "weights_sha256": checkpoint.weights_sha256, "shape": checkpoint.shape});
    let tokens = read_tokens(&settings.tokens, checkpoint.shape.vocab)?;
    let starts = evenly_spaced(tokens.len(), time, batch)?;
    let (inputs, targets) = windows(&tokens, &starts, time);
    let dot = KappaLlama::new(
        clone_checkpoint(&checkpoint),
        ScoreKind::Dot,
        0.0,
        Trainable::Scalars,
        &device,
    )?;
    let reference = dot.forward(&inputs, batch, time, true)?;
    let reference_nll = scalar(&next_token_nll(&reference, &targets)?)?;
    let curved = KappaLlama::new(checkpoint, score, grid[0], Trainable::Scalars, &device)?;
    let reference_top = reference.argmax(2)?;
    let mut rows = Vec::new();
    for &log_eps in grid {
        curved.set_log_eps(log_eps)?;
        let logits = curved.forward(&inputs, batch, time, true)?;
        let agree = logits
            .argmax(2)?
            .eq(&reference_top)?
            .to_dtype(candle_core::DType::F32)?
            .mean_all()?;
        let row = json!({
            "log_eps": log_eps,
            "kappa": (2.0 * f64::from(log_eps)).exp(),
            "max_abs_logit_difference": max_abs_difference(&logits, &reference)?,
            "mean_kl_dot_to_curved": scalar(&distillation_kl(&logits, &reference)?)?,
            "top1_agreement": scalar(&agree)?,
            "nll": scalar(&next_token_nll(&logits, &targets)?)?,
        });
        eprintln!("{row}");
        rows.push(row);
    }
    let report = json!({
        "schema": "uor-r4.kappa-conversion-probe/1",
        "identity": identity,
        "score": score,
        "batch": batch,
        "time": time,
        "window_starts": starts,
        "dot_nll": reference_nll,
        "rows": rows,
        "seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(out.join("probe.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

fn clone_checkpoint(checkpoint: &Checkpoint) -> Checkpoint {
    Checkpoint {
        shape: checkpoint.shape.clone(),
        tensors: checkpoint.tensors.clone(),
        weights_sha256: checkpoint.weights_sha256.clone(),
    }
}

struct Evaluation {
    nll: f64,
    kl: Option<f64>,
}

fn evaluate(
    student: &KappaLlama,
    teacher: Option<&KappaLlama>,
    tokens: &[u32],
    starts: &[usize],
    time: usize,
    batch: usize,
) -> Result<Evaluation> {
    let mut nll = 0.0;
    let mut kl = 0.0;
    let mut chunks = 0usize;
    for chunk in starts.chunks(batch) {
        let (inputs, targets) = windows(tokens, chunk, time);
        let logits = student.forward(&inputs, chunk.len(), time, true)?;
        nll += scalar(&next_token_nll(&logits, &targets)?)? * chunk.len() as f64;
        if let Some(teacher) = teacher {
            let target = teacher.forward(&inputs, chunk.len(), time, true)?;
            kl += scalar(&distillation_kl(&logits, &target)?)? * chunk.len() as f64;
        }
        chunks += chunk.len();
    }
    Ok(Evaluation {
        nll: nll / chunks as f64,
        kl: teacher.map(|_| kl / chunks as f64),
    })
}

struct TrainSettings {
    device: Device,
    student: PathBuf,
    teacher: Option<PathBuf>,
    train: PathBuf,
    valid: PathBuf,
    score: ScoreKind,
    trainable: Trainable,
    init_log_eps: f32,
    steps: usize,
    batch: usize,
    time: usize,
    lr: f64,
    curv_lr: f64,
    eval_every: usize,
    eval_windows: usize,
    seed: u64,
    max_seconds: f64,
    save_model: bool,
    anneal: Option<Anneal>,
}

/// Rising lower bound on every head's `log_eps` (curvature annealing).
#[derive(Clone, Copy, Debug)]
struct Anneal {
    to: f32,
    steps: usize,
    hold: bool,
}

impl Anneal {
    /// The floor after `step` optimizer steps, if one applies.
    fn floor(&self, init: f32, step: usize) -> Option<f32> {
        if step > self.steps && !self.hold {
            return None;
        }
        let fraction = (step as f32 / self.steps as f32).min(1.0);
        Some(init + (self.to - init) * fraction)
    }
}

fn train_settings(args: &Args) -> Result<TrainSettings> {
    let settings = TrainSettings {
        device: device(args)?,
        student: PathBuf::from(args.required("student")?),
        teacher: args.text("teacher").map(PathBuf::from),
        train: PathBuf::from(args.required("train")?),
        valid: PathBuf::from(args.required("valid")?),
        score: ScoreKind::parse(args.text("score").unwrap_or("intrinsic"))?,
        trainable: Trainable::parse(args.text("trainable").unwrap_or("scalars"))?,
        init_log_eps: args.number("init_log_eps", -4.6)?,
        steps: args.number("steps", 500)?,
        batch: args.number("batch", 4)?,
        time: args.number("time", 256)?,
        lr: args.number("lr", 1e-5)?,
        curv_lr: args.number("curv_lr", 1e-2)?,
        eval_every: args.number("eval_every", 100)?,
        eval_windows: args.number("eval_windows", 16)?,
        seed: args.number("seed", 1)?,
        max_seconds: args.number("max_seconds", 0.0)?,
        save_model: match args.text("save_model").unwrap_or("true") {
            "true" => true,
            "false" => false,
            other => {
                return Err(invalid(format!(
                    "save_model must be true or false, not {other}"
                )))
            }
        },
        anneal: None,
    };
    let mut settings = settings;
    if let Some(text) = args.text("anneal_to") {
        let to: f32 = text
            .parse()
            .map_err(|_| invalid(format!("invalid anneal_to={text}")))?;
        let hold = match args.text("anneal_hold").unwrap_or("true") {
            "true" => true,
            "false" => false,
            other => {
                return Err(invalid(format!(
                    "anneal_hold must be true or false, not {other}"
                )))
            }
        };
        let steps = args.number("anneal_steps", settings.steps.div_ceil(2))?;
        if !settings.score.is_curved()
            || !to.is_finite()
            || !(-20.0..=5.0).contains(&to)
            || to < settings.init_log_eps
            || steps == 0
        {
            return Err(invalid(
                "anneal needs a curved score, anneal_to in [init_log_eps, 5] and anneal_steps > 0",
            ));
        }
        settings.anneal = Some(Anneal { to, steps, hold });
    }
    if settings.steps == 0
        || settings.batch == 0
        || settings.time < 2
        || settings.eval_every == 0
        || settings.eval_windows == 0
        || !(settings.lr > 0.0 && settings.curv_lr > 0.0)
        || !settings.max_seconds.is_finite()
        || settings.max_seconds < 0.0
    {
        return Err(invalid(
            "steps, batch, eval_every, eval_windows, lr and curv_lr must be positive; time >= 2",
        ));
    }
    Ok(settings)
}

fn train(settings: &TrainSettings, out: &Path) -> Result<()> {
    let started = Instant::now();
    let TrainSettings {
        device,
        student: student_dir,
        score,
        trainable,
        init_log_eps,
        steps,
        batch,
        time,
        lr,
        curv_lr,
        eval_every,
        eval_windows,
        seed,
        max_seconds,
        save_model,
        anneal,
        ..
    } = settings;
    let anneal = *anneal;
    let (score, trainable, init_log_eps, steps, batch, time) =
        (*score, *trainable, *init_log_eps, *steps, *batch, *time);
    let (lr, curv_lr, eval_every, eval_windows, seed, max_seconds, save_model) = (
        *lr,
        *curv_lr,
        *eval_every,
        *eval_windows,
        *seed,
        *max_seconds,
        *save_model,
    );
    let device = device.clone();
    let checkpoint = load_checkpoint(student_dir, &device)?;
    let vocab = checkpoint.shape.vocab;
    let mut identity = json!({
        "student": student_dir, "student_sha256": checkpoint.weights_sha256, "shape": checkpoint.shape,
    });
    let teacher = match &settings.teacher {
        None => None,
        Some(dir) => {
            let loaded = load_checkpoint(dir, &device)?;
            if loaded.shape.vocab != vocab {
                return Err(invalid("teacher and student vocabularies differ"));
            }
            identity["teacher"] = json!(dir);
            identity["teacher_sha256"] = json!(loaded.weights_sha256);
            Some(KappaLlama::new(
                loaded,
                ScoreKind::Dot,
                0.0,
                Trainable::Scalars,
                &device,
            )?)
        }
    };
    let student = KappaLlama::new(checkpoint, score, init_log_eps, trainable, &device)?;
    let train_tokens = read_tokens(&settings.train, vocab)?;
    let valid_tokens = read_tokens(&settings.valid, vocab)?;
    identity["train_sha256"] = json!(uor_r4_training::sha256_file(&settings.train)?);
    identity["valid_sha256"] = json!(uor_r4_training::sha256_file(&settings.valid)?);
    if train_tokens.len() < time + 2 {
        return Err(invalid("train file shorter than one window"));
    }
    let valid_starts = evenly_spaced(valid_tokens.len(), time, eval_windows)?;

    // Curvature/temperature scalars and checkpoint weights get separate step sizes
    // and no weight decay (decay would pull log_eps toward kappa = 1).
    let split = |scalars: bool| -> BTreeMap<String, candle_core::Var> {
        student
            .variables()
            .iter()
            .filter(|(name, _)| (name.as_str() == LOG_EPS || name.as_str() == LOG_BETA) == scalars)
            .map(|(name, var)| (name.clone(), var.clone()))
            .collect()
    };
    let scalar_vars = split(true);
    let weight_vars = split(false);
    let config = |learning_rate: f64| AdamConfig {
        learning_rate,
        weight_decay: 0.0,
        ..AdamConfig::default()
    };
    let mut scalar_opt = NamedAdamW::new(&scalar_vars, config(curv_lr))?;
    let mut weight_opt = if weight_vars.is_empty() {
        None
    } else {
        Some(NamedAdamW::new(&weight_vars, config(lr))?)
    };

    let mut progress = Vec::new();
    let record = |step: usize, recent: Option<f64>, model: &KappaLlama| -> Result<Value> {
        let eval = evaluate(
            model,
            teacher.as_ref(),
            &valid_tokens,
            &valid_starts,
            time,
            batch,
        )?;
        let row = json!({
            "step": step,
            "train_objective_recent": recent,
            "valid_nll": eval.nll,
            "valid_kl_to_teacher": eval.kl,
            "curvature": summary(&model.curvature()?),
            "curvature_floor_log_eps": anneal.and_then(|a| a.floor(init_log_eps, step)),
            "temperature": summary(&model.temperature()?),
            "wall": started.elapsed().as_secs_f64(),
        });
        eprintln!("{row}");
        Ok(row)
    };
    progress.push(record(0, None, &student)?);
    let mut sampler = Windows(seed);
    let mut recent = Vec::new();
    let mut completed = 0usize;
    for step in 1..=steps {
        if max_seconds > 0.0 && started.elapsed().as_secs_f64() > max_seconds {
            break;
        }
        let starts: Vec<usize> = (0..batch)
            .map(|_| (sampler.next() % (train_tokens.len() - time - 1) as u64) as usize)
            .collect();
        let (inputs, targets) = windows(&train_tokens, &starts, time);
        let logits = student.forward(&inputs, batch, time, false)?;
        let loss = match &teacher {
            Some(teacher) => {
                distillation_kl(&logits, &teacher.forward(&inputs, batch, time, true)?)?
            }
            None => next_token_nll(&logits, &targets)?,
        };
        recent.push(scalar(&loss)?);
        let gradients = loss.backward()?;
        scalar_opt.step(&scalar_vars, &gradients)?;
        if let Some(optimizer) = weight_opt.as_mut() {
            optimizer.step(&weight_vars, &gradients)?;
        }
        if let Some(floor) = anneal.and_then(|a| a.floor(init_log_eps, step)) {
            student.floor_log_eps(floor)?;
        }
        completed = step;
        if step % eval_every == 0 || step == steps {
            let window = recent.len().min(eval_every);
            let mean = recent[recent.len() - window..].iter().sum::<f64>() / window as f64;
            progress.push(record(step, Some(mean), &student)?);
            fs::write(
                out.join("progress.json"),
                serde_json::to_vec_pretty(&progress)?,
            )?;
        }
    }
    if progress.last().and_then(|row| row["step"].as_u64()) != Some(completed as u64) {
        let window = recent.len().clamp(1, eval_every);
        let mean = recent.iter().rev().take(window).sum::<f64>() / window as f64;
        progress.push(record(completed, Some(mean), &student)?);
    }
    if save_model {
        let directory = out.join("model");
        fs::create_dir(&directory)?;
        let tensors: HashMap<String, Tensor> = student
            .variables()
            .iter()
            .map(|(name, var)| (name.clone(), var.as_tensor().clone()))
            .collect();
        candle_core::safetensors::save(&tensors, directory.join("variables.safetensors"))?;
    }
    let report = json!({
        "schema": "uor-r4.kappa-conversion-train/1",
        "identity": identity,
        "settings": {
            "score": score, "trainable": trainable, "init_log_eps": init_log_eps, "steps": steps,
            "completed_steps": completed, "batch": batch, "time": time, "lr": lr, "curv_lr": curv_lr,
            "eval_every": eval_every, "eval_windows": eval_windows, "seed": seed,
            "anneal": anneal.map(|a| json!({"to": a.to, "steps": a.steps, "hold": a.hold})),
            "objective": if teacher.is_some() { "kl_to_dot_teacher" } else { "next_token_nll" },
        },
        "curvature_final": student.curvature()?,
        "temperature_final": student.temperature()?,
        "progress": progress,
        "seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

fn sample(args: &Args) -> Result<()> {
    let device = device(args)?;
    let model_dir = PathBuf::from(args.required("model")?);
    let score = ScoreKind::parse(args.text("score").unwrap_or("dot"))?;
    let count: usize = args.number("tokens", 64)?;
    let checkpoint = load_checkpoint(&model_dir, &device)?;
    let tokenizer = HfBpeTokenizer::from_dir(&model_dir)
        .map_err(|error| invalid(format!("tokenizer: {error}")))?;
    let trainable = Trainable::parse(args.text("trainable").unwrap_or("scalars"))?;
    let model = KappaLlama::new(checkpoint, score, -4.6, trainable, &device)?;
    if let Some(path) = args.text("variables") {
        let saved = candle_core::safetensors::load(path, &device)?;
        for (name, var) in model.variables() {
            let tensor = saved
                .get(name)
                .ok_or_else(|| invalid(format!("{path} lacks {name}")))?;
            var.set(tensor)?;
        }
    }
    let prompt = args.required("prompt")?;
    let chat = format!("<|im_start|>user\n{prompt}<|im_end|>\n<|im_start|>assistant\n");
    let mut ids = tokenizer.encode(&chat);
    let stop = tokenizer.encode("<|im_end|>");
    let prompt_len = ids.len();
    for _ in 0..count {
        let logits = model.forward(&ids, 1, ids.len(), true)?;
        let last = logits.narrow(1, ids.len() - 1, 1)?.flatten_all()?;
        let next = last.argmax(0)?.to_scalar::<u32>()?;
        ids.push(next);
        if stop.len() == 1 && next == stop[0] {
            break;
        }
    }
    println!("{}", tokenizer.decode(&ids[prompt_len..]));
    Ok(())
}

fn main() -> Result<()> {
    let mut pairs = BTreeMap::new();
    for argument in std::env::args().skip(1) {
        let (key, value) = argument
            .split_once('=')
            .ok_or_else(|| invalid("arguments are key=value"))?;
        pairs.insert(key.to_string(), value.to_string());
    }
    let args = Args(pairs);
    let mode = args.required("mode")?.to_owned();
    match mode.as_str() {
        "tokenize" => return tokenize(&args),
        "sample" => return sample(&args),
        "probe" | "train" => {}
        other => return Err(invalid(format!("unknown mode {other}"))),
    }
    let out = PathBuf::from(args.required("out")?);
    let probe_args = if mode == "probe" {
        Some(probe_settings(&args)?)
    } else {
        None
    };
    let train_args = if mode == "train" {
        Some(train_settings(&args)?)
    } else {
        None
    };
    report_output::claim(&out)?;
    let result = match (&probe_args, &train_args) {
        (Some(settings), _) => probe(settings, &out),
        (_, Some(settings)) => train(settings, &out),
        _ => Err(invalid("unreachable mode")),
    };
    if let Err(error) = &result {
        fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error": error.to_string()}))?,
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result
}
