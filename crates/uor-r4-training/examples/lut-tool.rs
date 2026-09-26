//! Export a Llama checkpoint to the integer serving artifact of `uor-r4-lut`
//! and measure the integer engine against the float forward pass.
//!
//! ```text
//! lut-tool mode=export model=DIR out=FILE.lut [max_positions=2048]
//! lut-tool mode=dequantize model=DIR lut=FILE.lut out=NEW_DIR
//! lut-tool mode=bench lut=FILE.lut [tokens=64]
//! lut-tool mode=fidelity model=DIR lut=FILE.lut tokens=X.u16 out=NEW_ROOT
//!     [windows=8] [time=128] [device=cpu|metal]
//! ```
//!
//! `export` folds the RMSNorm gains into the following weights, quantizes every
//! weight matrix to 4 bits in groups of 32 with shift-add scales, seals the exp,
//! SiLU and RoPE tables, and writes `FILE.lut` plus `FILE.lut.report.json`
//! (per-matrix relative RMS quantization error). `dequantize` writes the
//! artifact's 4-bit weights back as a float checkpoint (gains folded, norm
//! weights 1, untied head), so fidelity against it isolates the integer
//! arithmetic from weight quantization. `fidelity` runs both engines
//! on the same held-out windows and reports next-token NLL (nats per token) of
//! each, KL(float || integer), top-1 agreement, the largest logit difference and
//! the integer engine's tokens per second. It claims and seals its report root.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use serde_json::json;
use uor_r4_core::report_output;
use uor_r4_lut::engine::Model;
use uor_r4_training::kappa_llama::{load_checkpoint, KappaLlama, ScoreKind, Trainable};
use uor_r4_training::lut_export::export_llama;
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

fn export(args: &Args) -> Result<()> {
    let model = PathBuf::from(args.required("model")?);
    let out = PathBuf::from(args.required("out")?);
    let max_positions: usize = args.number("max_positions", 2048)?;
    if out.exists() {
        return Err(invalid(format!(
            "{} exists; artifacts are never overwritten",
            out.display()
        )));
    }
    let started = Instant::now();
    let checkpoint = load_checkpoint(&model, &Device::Cpu)?;
    let source = json!({
        "exporter": "uor-r4-training lut_export 1",
        "checkpoint": model,
        "weights_sha256": checkpoint.weights_sha256,
        "quantizer": "4-bit groups of 32, per-group least-squares scale over (16 + m) 2^(e - 4), RMSNorm gains folded",
    });
    let (bytes, report) = export_llama(&checkpoint, max_positions, source)?;
    fs::write(&out, &bytes)?;
    let mut report_path = out.clone().into_os_string();
    report_path.push(".report.json");
    fs::write(&report_path, serde_json::to_vec_pretty(&report)?)?;
    let worst = report["relative_rms_error"]
        .as_object()
        .map(|m| m.values().filter_map(|v| v.as_f64()).fold(0.0, f64::max))
        .unwrap_or(0.0);
    eprintln!(
        "wrote {} ({} bytes) in {:.1}s; worst matrix relative RMS error {:.4}",
        out.display(),
        bytes.len(),
        started.elapsed().as_secs_f64(),
        worst
    );
    Ok(())
}

fn dequantize(args: &Args) -> Result<()> {
    use std::collections::HashMap;
    use uor_r4_lut::format::Artifact;
    let model = PathBuf::from(args.required("model")?);
    let lut = PathBuf::from(args.required("lut")?);
    let out = PathBuf::from(args.required("out")?);
    if out.exists() {
        return Err(invalid(format!("{} exists", out.display())));
    }
    let artifact = Artifact::load(&lut).map_err(|e| invalid(e.to_string()))?;
    let shape = artifact.header.shape.clone();
    let matrix = |name: &str| -> Result<(Vec<f32>, usize, usize)> {
        let spec = artifact.matrix(name).map_err(|e| invalid(e.to_string()))?;
        let values = uor_r4_training::lut_export::dequantize_matrix(
            spec.rows,
            spec.cols,
            spec.exp_base,
            artifact.section(spec.nibbles),
            artifact.section(spec.scales),
        )?;
        Ok((values, spec.rows, spec.cols))
    };
    let device = Device::Cpu;
    let mut tensors: HashMap<String, candle_core::Tensor> = HashMap::new();
    let mut put = |name: String, (values, rows, cols): (Vec<f32>, usize, usize)| -> Result<()> {
        tensors.insert(
            name,
            candle_core::Tensor::from_vec(values, (rows, cols), &device)?,
        );
        Ok(())
    };
    put("model.embed_tokens.weight".into(), matrix("embed")?)?;
    put("lm_head.weight".into(), matrix("head")?)?;
    for l in 0..shape.layers {
        let p = format!("model.layers.{l}");
        for (short, name) in [
            ("q", "self_attn.q_proj"),
            ("k", "self_attn.k_proj"),
            ("v", "self_attn.v_proj"),
            ("o", "self_attn.o_proj"),
            ("gate", "mlp.gate_proj"),
            ("up", "mlp.up_proj"),
            ("down", "mlp.down_proj"),
        ] {
            put(
                format!("{p}.{name}.weight"),
                matrix(&format!("l{l}.{short}"))?,
            )?;
        }
    }
    let ones = |n: usize| candle_core::Tensor::ones(n, candle_core::DType::F32, &device);
    for l in 0..shape.layers {
        for name in ["input_layernorm", "post_attention_layernorm"] {
            tensors.insert(
                format!("model.layers.{l}.{name}.weight"),
                ones(shape.width)?,
            );
        }
    }
    tensors.insert("model.norm.weight".into(), ones(shape.width)?);
    fs::create_dir(&out)?;
    candle_core::safetensors::save(&tensors, out.join("model.safetensors"))?;
    let mut config: serde_json::Value =
        serde_json::from_slice(&fs::read(model.join("config.json"))?)?;
    config["tie_word_embeddings"] = json!(false);
    fs::write(out.join("config.json"), serde_json::to_vec_pretty(&config)?)?;
    if model.join("tokenizer.json").exists() {
        fs::copy(model.join("tokenizer.json"), out.join("tokenizer.json"))?;
    }
    eprintln!("wrote dequantized float checkpoint {}", out.display());
    Ok(())
}

fn bench(args: &Args) -> Result<()> {
    let lut = PathBuf::from(args.required("lut")?);
    let tokens: usize = args.number("tokens", 64)?;
    let model = Model::load(&lut).map_err(|e| invalid(e.to_string()))?;
    let vocab = model.shape().vocab as u32;
    if tokens == 0 || tokens > model.shape().max_positions {
        return Err(invalid("tokens must be positive and fit max_positions"));
    }
    let mut session = model.session();
    let started = Instant::now();
    let mut next = 1u32;
    for i in 0..tokens {
        let logits = session.step(next).map_err(|e| invalid(e.to_string()))?;
        next = (uor_r4_lut::kernels::argmax(logits) as u32 + i as u32) % vocab;
    }
    let seconds = started.elapsed().as_secs_f64();
    let report = json!({
        "lut": lut, "lut_sha256": model.artifact_sha256(), "tokens": tokens,
        "seconds": seconds, "tokens_per_second": tokens as f64 / seconds,
        "threads": std::env::var("RAYON_NUM_THREADS").unwrap_or_else(|_| "default".into()),
    });
    println!("{report}");
    Ok(())
}

fn read_tokens(path: &Path, vocab: usize) -> Result<Vec<u32>> {
    let bytes = fs::read(path)?;
    let tokens: Vec<u32> = bytes
        .chunks_exact(2)
        .map(|b| u32::from(u16::from_le_bytes([b[0], b[1]])))
        .collect();
    if bytes.len() % 2 != 0 || tokens.iter().any(|&t| t as usize >= vocab) {
        return Err(invalid(format!(
            "{} is not a valid u16 token file",
            path.display()
        )));
    }
    Ok(tokens)
}

/// log-sum-exp and log-probabilities of one logit row.
fn log_softmax(row: &[f64]) -> Vec<f64> {
    let max = row.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let sum: f64 = row.iter().map(|v| (v - max).exp()).sum();
    let lse = max + sum.ln();
    row.iter().map(|v| v - lse).collect()
}

fn argmax(row: &[f64]) -> usize {
    let mut best = 0;
    for (i, v) in row.iter().enumerate() {
        if *v > row[best] {
            best = i;
        }
    }
    best
}

struct FidelitySettings {
    model: PathBuf,
    lut: PathBuf,
    tokens: PathBuf,
    windows: usize,
    time: usize,
    device: Device,
}

fn fidelity_settings(args: &Args) -> Result<FidelitySettings> {
    let settings = FidelitySettings {
        model: PathBuf::from(args.required("model")?),
        lut: PathBuf::from(args.required("lut")?),
        tokens: PathBuf::from(args.required("tokens")?),
        windows: args.number("windows", 8)?,
        time: args.number("time", 128)?,
        device: match args.text("device").unwrap_or("cpu") {
            "cpu" => Device::Cpu,
            "metal" => Device::new_metal(0)?,
            other => return Err(invalid(format!("device must be cpu or metal, not {other}"))),
        },
    };
    if settings.windows == 0 || settings.time < 2 {
        return Err(invalid("windows must be positive and time >= 2"));
    }
    Ok(settings)
}

fn fidelity(settings: &FidelitySettings, out: &Path) -> Result<()> {
    let checkpoint = load_checkpoint(&settings.model, &settings.device)?;
    let weights_sha256 = checkpoint.weights_sha256.clone();
    let vocab = checkpoint.shape.vocab;
    let float = KappaLlama::new(
        checkpoint,
        ScoreKind::Dot,
        0.0,
        Trainable::Scalars,
        &settings.device,
    )?;
    let model = Model::load(&settings.lut).map_err(|e| invalid(e.to_string()))?;
    if model.shape().vocab != vocab || model.shape().max_positions < settings.time {
        return Err(invalid(
            "artifact and checkpoint disagree, or the artifact is too short",
        ));
    }
    let tokens = read_tokens(&settings.tokens, vocab)?;
    let (time, windows) = (settings.time, settings.windows);
    if tokens.len() < time + 2 {
        return Err(invalid("token file shorter than one window"));
    }
    let span = tokens.len() - time - 1;
    let (mut nll_float, mut nll_int, mut kl, mut agree, mut max_diff) =
        (0.0, 0.0, 0.0, 0usize, 0f64);
    let (mut float_seconds, mut int_seconds, mut count) = (0.0, 0.0, 0usize);
    let mut session = model.session();
    for w in 0..windows {
        let start = w * span / windows;
        let ids = &tokens[start..start + time];
        let targets = &tokens[start + 1..start + time + 1];
        let t0 = Instant::now();
        let logits = float.forward(ids, 1, time, true)?.to_vec3::<f32>()?;
        float_seconds += t0.elapsed().as_secs_f64();
        session.reset();
        let t1 = Instant::now();
        let mut integer = Vec::with_capacity(time);
        for &id in ids {
            let row = session.step(id).map_err(|e| invalid(e.to_string()))?;
            integer.push(
                row.iter()
                    .map(|v| f64::from(*v) * 2f64.powi(-16))
                    .collect::<Vec<f64>>(),
            );
        }
        int_seconds += t1.elapsed().as_secs_f64();
        for (t, int_row) in integer.iter().enumerate() {
            let float_row: Vec<f64> = logits[0][t].iter().map(|v| f64::from(*v)).collect();
            let (lf, li) = (log_softmax(&float_row), log_softmax(int_row));
            let target = targets[t] as usize;
            nll_float -= lf[target];
            nll_int -= li[target];
            kl += lf
                .iter()
                .zip(&li)
                .map(|(a, b)| a.exp() * (a - b))
                .sum::<f64>();
            agree += usize::from(argmax(&float_row) == argmax(int_row));
            let shift_f = lf[0] - float_row[0];
            let shift_i = li[0] - int_row[0];
            max_diff = float_row
                .iter()
                .zip(int_row)
                .map(|(a, b)| ((a + shift_f) - (b + shift_i)).abs())
                .fold(max_diff, f64::max);
            count += 1;
        }
    }
    let n = count as f64;
    let report = json!({
        "schema": "uor-r4.lut-fidelity/1",
        "identity": {"model": settings.model, "weights_sha256": weights_sha256,
                     "lut": settings.lut, "lut_sha256": model.artifact_sha256(),
                     "tokens": settings.tokens, "tokens_sha256": uor_r4_training::sha256_file(&settings.tokens)?},
        "settings": {"windows": windows, "time": time},
        "positions": count,
        "nll_float": nll_float / n,
        "nll_integer": nll_int / n,
        "nll_change": (nll_int - nll_float) / n,
        "kl_float_to_integer": kl / n,
        "top1_agreement": agree as f64 / n,
        "max_abs_log_probability_difference": max_diff,
        "integer_tokens_per_second": n / int_seconds.max(1e-9),
        "float_tokens_per_second": n / float_seconds.max(1e-9),
        "units": "nats per token; integer engine single-threaded portable kernels",
    });
    eprintln!("{report}");
    fs::write(
        out.join("fidelity.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
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
    match args.required("mode")? {
        "export" => export(&args),
        "dequantize" => dequantize(&args),
        "bench" => bench(&args),
        "fidelity" => {
            let settings = fidelity_settings(&args)?;
            let out = PathBuf::from(args.required("out")?);
            report_output::claim(&out)?;
            let result = fidelity(&settings, &out);
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
        other => Err(invalid(format!("unknown mode {other}"))),
    }
}
