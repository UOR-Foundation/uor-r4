//! B3: E8 Lattice Weight Codec Evaluation on SmolLM2-360M MLP Layers.
//!
//! Evaluates E8 lattice vector quantization at 2, 3, and 4 bits per weight against
//! 4-bit round-to-nearest (RTN) on all 32 layers of SmolLM2-360M's MLP weights:
//! `gate_proj`, `up_proj`, `down_proj` (235,929,600 total weights).
//!
//! Pre-registered Arms:
//! - Arm 0: Float reference (SmolLM2-360M-Instruct baseline)
//! - Arm 1: RTN 4-bit (Group size 32: 16 code bytes + 1 scale byte per 32 weights = 4.2500 bpw)
//! - Arm 2: E8 2-bit (1-stage E8: 1 byte index + 1 byte scale per 8 weights = 2.0000 bpw)
//! - Arm 3: E8 3-bit (2-stage residual E8: 2 byte indices + 1 byte scale per 8 weights = 3.0000 bpw)
//! - Arm 4: E8 4-bit (2-stage residual E8: 2 byte indices + 2 byte scales per 8 weights = 4.0000 bpw)
//!
//! Invariants:
//! - Bits per weight derived strictly from serialized bytes: `8 * serialized_bytes / num_weights`.
//! - Codec round trip gate: verified bit-for-bit exact deserialization on every quantized matrix.
//! - Kill criterion: 3-bit E8 delta NLL > RTN 4-bit delta NLL + 0.05 nats.
//! - Sealed report root via `report_output::{claim, seal, verify}`.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{Device, Tensor};
use serde::{Deserialize, Serialize};
use serde_json::json;

use uor_r4_core::report_output;
use uor_r4_training::b3_e8_codecs::{
    E8FourBitMatrix, E8ThreeBitMatrix, E8TwoBitMatrix, Rtn4BitMatrix,
};
use uor_r4_training::kappa_llama::{
    distillation_kl, load_checkpoint, next_token_nll, KappaLlama, ScoreKind, Trainable,
};

type GenericResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn err(msg: impl Into<String>) -> Box<dyn std::error::Error> {
    msg.into().into()
}

#[derive(Clone, Debug)]
struct CliArgs {
    model_dir: PathBuf,
    tokens_path: PathBuf,
    windows: usize,
    time: usize,
    batch: usize,
    out: PathBuf,
    layers: usize,
}

fn parse_cli_args() -> GenericResult<CliArgs> {
    let mut model_dir =
        PathBuf::from("/Volumes/UOR-Workspace/uor-r4-models/sources/smollm2-360m-instruct");
    let mut tokens_path =
        PathBuf::from("/Users/casey.allard/uor-r4/.uor-models/research/issue-1017/tokens/dev.u16");
    let mut windows = 8;
    let mut time = 256;
    let mut batch = 1;
    let mut out = PathBuf::from("/Volumes/UOR-Workspace/uor-r4-lab/b3-e8-smollm2-mlp/attempt-1");
    let mut layers = 32;

    for arg in std::env::args().skip(1) {
        if let Some((k, v)) = arg.split_once('=') {
            match k {
                "model" => model_dir = PathBuf::from(v),
                "tokens" => tokens_path = PathBuf::from(v),
                "windows" => {
                    windows = v
                        .parse()
                        .map_err(|e| err(format!("invalid windows: {e}")))?
                }
                "time" => time = v.parse().map_err(|e| err(format!("invalid time: {e}")))?,
                "batch" => batch = v.parse().map_err(|e| err(format!("invalid batch: {e}")))?,
                "out" => out = PathBuf::from(v),
                "layers" => layers = v.parse().map_err(|e| err(format!("invalid layers: {e}")))?,
                _ => eprintln!("Warning: unknown argument '{arg}'"),
            }
        }
    }

    Ok(CliArgs {
        model_dir,
        tokens_path,
        windows,
        time,
        batch,
        out,
        layers,
    })
}

fn read_u16_tokens(path: &Path, vocab: usize) -> GenericResult<Vec<u32>> {
    let bytes = fs::read(path)?;
    if bytes.len() % 2 != 0 {
        return Err(err(format!("odd token file length {}", bytes.len())));
    }
    let tokens: Vec<u32> = bytes
        .chunks_exact(2)
        .map(|chunk| u32::from(u16::from_le_bytes([chunk[0], chunk[1]])))
        .collect();
    if tokens.iter().any(|&t| t as usize >= vocab) {
        return Err(err(format!(
            "{} has a token outside the vocabulary ({vocab})",
            path.display()
        )));
    }
    Ok(tokens)
}

fn evenly_spaced(len: usize, time: usize, count: usize) -> GenericResult<Vec<usize>> {
    if len < time + 2 || count == 0 {
        return Err(err("token file shorter than one window"));
    }
    let span = len - time - 1;
    Ok((0..count).map(|i| i * span / count).collect())
}

type Chunk = (Vec<u32>, Vec<u32>, usize);

fn build_chunks(tokens: &[u32], starts: &[usize], time: usize, batch: usize) -> Vec<Chunk> {
    starts
        .chunks(batch)
        .map(|chunk| {
            let mut inputs = Vec::with_capacity(chunk.len() * time);
            let mut targets = Vec::with_capacity(chunk.len() * time);
            for &start in chunk {
                inputs.extend_from_slice(&tokens[start..start + time]);
                targets.extend_from_slice(&tokens[start + 1..start + time + 1]);
            }
            (inputs, targets, chunk.len())
        })
        .collect()
}

fn compute_top1_agreement(logits: &Tensor, ref_logits: &Tensor) -> GenericResult<f64> {
    let (b, t, v) = logits.dims3()?;
    let pred = logits.reshape((b * t, v))?.argmax(1)?;
    let ref_pred = ref_logits.reshape((b * t, v))?.argmax(1)?;
    let eq = pred.eq(&ref_pred)?;
    let count: u32 = eq
        .to_dtype(candle_core::DType::U32)?
        .sum_all()?
        .to_scalar()?;
    Ok((count as f64) / ((b * t) as f64))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ArmMetrics {
    arm_id: usize,
    name: String,
    title: String,
    target_bpw: f64,
    payload_bpw: f64,
    total_serialized_bytes: usize,
    quantized_weights_count: usize,
    mean_nll: f64,
    delta_nll: f64,
    mean_kl: f64,
    top1_agreement_pct: f64,
    window_nlls: Vec<f64>,
    codec_roundtrip_gate_passed: bool,
}

fn main() -> GenericResult<()> {
    let args = parse_cli_args()?;
    println!("=== B3: E8 Lattice Weight Coding on SmolLM2-360M MLP ===");
    println!("Model dir:   {}", args.model_dir.display());
    println!("Tokens path: {}", args.tokens_path.display());
    println!(
        "Windows:     {} (time: {}, batch: {})",
        args.windows, args.time, args.batch
    );
    println!("Layers:      {}", args.layers);
    println!("Output dir:  {}", args.out.display());

    // 1. Claim exclusive report root
    report_output::claim(&args.out)?;

    // 2. Select compute device (CPU default for reproducible arithmetic)
    let device = Device::Cpu;

    // 3. Load SmolLM2-360M Checkpoint
    println!("\nLoading SmolLM2-360M checkpoint...");
    let load_start = Instant::now();
    let checkpoint = load_checkpoint(&args.model_dir, &device)?;
    println!(
        "Loaded in {:.2}s (SHA-256: {}, layers: {}, hidden: {}, ffn: {})",
        load_start.elapsed().as_secs_f64(),
        checkpoint.weights_sha256,
        checkpoint.shape.layers,
        checkpoint.shape.width,
        checkpoint.shape.ffn
    );

    // 4. Load Evaluation Tokens
    let tokens = read_u16_tokens(&args.tokens_path, checkpoint.shape.vocab)?;
    println!(
        "Loaded {} tokens from {}",
        tokens.len(),
        args.tokens_path.display()
    );
    let starts = evenly_spaced(tokens.len(), args.time, args.windows)?;
    let chunks = build_chunks(&tokens, &starts, args.time, args.batch);
    let total_eval_tokens = args.windows * args.time;
    println!(
        "Prepared {} windows (total {} tokens)",
        args.windows, total_eval_tokens
    );

    // 5. Instantiate Model and Arm 0 (Float Reference)
    let mut model = KappaLlama::new(
        checkpoint.clone(),
        ScoreKind::Dot,
        0.0,
        Trainable::Scalars,
        &device,
    )?;

    println!("\n[Arm 0] Evaluating Float Reference Baseline...");
    let mut ref_nlls = Vec::with_capacity(chunks.len());
    let mut ref_logits_list = Vec::with_capacity(chunks.len());
    let float_eval_start = Instant::now();

    for (inputs, targets, n) in &chunks {
        let logits = model.forward(inputs, *n, args.time, true)?;
        let loss = next_token_nll(&logits, targets)?;
        let nll_val = f64::from(loss.to_scalar::<f32>()?);
        ref_nlls.push(nll_val);
        ref_logits_list.push(logits);
    }

    let float_mean_nll: f64 = ref_nlls.iter().sum::<f64>() / (ref_nlls.len() as f64);
    println!(
        "  Float NLL: {:.6} nats (evaluated in {:.2}s)",
        float_mean_nll,
        float_eval_start.elapsed().as_secs_f64()
    );

    let arm0 = ArmMetrics {
        arm_id: 0,
        name: "float".to_string(),
        title: "Float Reference Baseline (SmolLM2-360M unquantized)".to_string(),
        target_bpw: 32.0,
        payload_bpw: 32.0,
        total_serialized_bytes: args.layers
            * 3
            * (checkpoint.shape.width * checkpoint.shape.ffn)
            * 4,
        quantized_weights_count: args.layers * 3 * (checkpoint.shape.width * checkpoint.shape.ffn),
        mean_nll: float_mean_nll,
        delta_nll: 0.0,
        mean_kl: 0.0,
        top1_agreement_pct: 100.0,
        window_nlls: ref_nlls.clone(),
        codec_roundtrip_gate_passed: true,
    };

    let mlp_projections = ["gate_proj", "up_proj", "down_proj"];
    let num_mlp_weights_per_layer = 3 * (checkpoint.shape.width * checkpoint.shape.ffn);
    let total_mlp_weights = args.layers * num_mlp_weights_per_layer;

    // Helper closure to evaluate an arm
    let evaluate_current_model = |model: &KappaLlama| -> GenericResult<(f64, f64, f64, Vec<f64>)> {
        let mut nlls = Vec::with_capacity(chunks.len());
        let mut kls = Vec::with_capacity(chunks.len());
        let mut agreements = Vec::with_capacity(chunks.len());

        for (i, (inputs, targets, n)) in chunks.iter().enumerate() {
            let logits = model.forward(inputs, *n, args.time, true)?;
            let loss = next_token_nll(&logits, targets)?;
            let nll_val = f64::from(loss.to_scalar::<f32>()?);
            nlls.push(nll_val);

            let kl_tensor = distillation_kl(&logits, &ref_logits_list[i])?;
            let kl_val = f64::from(kl_tensor.to_scalar::<f32>()?);
            kls.push(kl_val);

            let agr = compute_top1_agreement(&logits, &ref_logits_list[i])?;
            agreements.push(agr);
        }

        let mean_nll = nlls.iter().sum::<f64>() / (nlls.len() as f64);
        let mean_kl = kls.iter().sum::<f64>() / (kls.len() as f64);
        let mean_agr = agreements.iter().sum::<f64>() / (agreements.len() as f64) * 100.0;
        Ok((mean_nll, mean_kl, mean_agr, nlls))
    };

    // -----------------------------------------------------------------------
    // Arm 1: RTN 4-bit (Group size 32, 4.2500 bpw)
    // -----------------------------------------------------------------------
    println!("\n[Arm 1] Quantizing MLP layers with RTN 4-bit (group size 32)...");
    let rtn_start = Instant::now();
    let mut rtn_bytes = 0usize;
    let mut rtn_roundtrip_ok = true;

    for l in 0..args.layers {
        for proj in &mlp_projections {
            let name = format!("model.layers.{l}.mlp.{proj}.weight");
            let orig_tensor = checkpoint
                .tensors
                .get(&name)
                .ok_or_else(|| err(format!("tensor {name} not found")))?;
            let dims = orig_tensor.dims();
            let (rows, cols) = (dims[0], dims[1]);
            let weights: Vec<f32> = orig_tensor.flatten_all()?.to_vec1()?;

            let matrix = Rtn4BitMatrix::encode(&weights, rows, cols)?;
            let bytes = matrix.to_bytes();
            rtn_bytes += bytes.len();

            let deserialized = Rtn4BitMatrix::from_bytes(&bytes)?;
            if matrix != deserialized {
                rtn_roundtrip_ok = false;
            }

            let deq = deserialized.dequantize()?;
            let new_tensor = Tensor::from_vec(deq, (rows, cols), &device)?;
            model.set_tensor(&name, new_tensor)?;
        }
    }

    let rtn_bpw = (rtn_bytes as f64 * 8.0) / (total_mlp_weights as f64);
    println!(
        "  RTN 4-bit encoded in {:.2}s (bytes: {}, bpw: {:.4}, roundtrip gate: {})",
        rtn_start.elapsed().as_secs_f64(),
        rtn_bytes,
        rtn_bpw,
        if rtn_roundtrip_ok { "PASSED" } else { "FAILED" }
    );

    let (rtn_nll, rtn_kl, rtn_agr, rtn_nlls) = evaluate_current_model(&model)?;
    let rtn_delta = rtn_nll - float_mean_nll;
    println!(
        "  RTN 4-bit NLL: {:.6} (Delta: {:+.6} nats, KL: {:.6}, Agr: {:.2}%)",
        rtn_nll, rtn_delta, rtn_kl, rtn_agr
    );

    let arm1 = ArmMetrics {
        arm_id: 1,
        name: "rtn_4bit".to_string(),
        title: "Round-to-Nearest 4-bit (Group size 32, ~4.25 bpw)".to_string(),
        target_bpw: 4.25,
        payload_bpw: rtn_bpw,
        total_serialized_bytes: rtn_bytes,
        quantized_weights_count: total_mlp_weights,
        mean_nll: rtn_nll,
        delta_nll: rtn_delta,
        mean_kl: rtn_kl,
        top1_agreement_pct: rtn_agr,
        window_nlls: rtn_nlls,
        codec_roundtrip_gate_passed: rtn_roundtrip_ok,
    };

    // -----------------------------------------------------------------------
    // Arm 2: E8 2-bit (1-stage E8, 2.0000 bpw)
    // -----------------------------------------------------------------------
    println!("\n[Arm 2] Quantizing MLP layers with E8 2-bit (1-stage E8)...");
    let e8_2bit_start = Instant::now();
    let mut e8_2bit_bytes = 0usize;
    let mut e8_2bit_roundtrip_ok = true;

    for l in 0..args.layers {
        for proj in &mlp_projections {
            let name = format!("model.layers.{l}.mlp.{proj}.weight");
            let orig_tensor = checkpoint
                .tensors
                .get(&name)
                .ok_or_else(|| err(format!("tensor {name} not found")))?;
            let dims = orig_tensor.dims();
            let (rows, cols) = (dims[0], dims[1]);
            let weights: Vec<f32> = orig_tensor.flatten_all()?.to_vec1()?;

            let matrix = E8TwoBitMatrix::encode(&weights, rows, cols)?;
            let bytes = matrix.to_bytes();
            e8_2bit_bytes += bytes.len();

            let deserialized = E8TwoBitMatrix::from_bytes(&bytes)?;
            if matrix != deserialized {
                e8_2bit_roundtrip_ok = false;
            }

            let deq = deserialized.dequantize()?;
            let new_tensor = Tensor::from_vec(deq, (rows, cols), &device)?;
            model.set_tensor(&name, new_tensor)?;
        }
    }

    let e8_2bit_bpw = (e8_2bit_bytes as f64 * 8.0) / (total_mlp_weights as f64);
    println!(
        "  E8 2-bit encoded in {:.2}s (bytes: {}, bpw: {:.4}, roundtrip gate: {})",
        e8_2bit_start.elapsed().as_secs_f64(),
        e8_2bit_bytes,
        e8_2bit_bpw,
        if e8_2bit_roundtrip_ok {
            "PASSED"
        } else {
            "FAILED"
        }
    );

    let (e8_2bit_nll, e8_2bit_kl, e8_2bit_agr, e8_2bit_nlls) = evaluate_current_model(&model)?;
    let e8_2bit_delta = e8_2bit_nll - float_mean_nll;
    println!(
        "  E8 2-bit NLL: {:.6} (Delta: {:+.6} nats, KL: {:.6}, Agr: {:.2}%)",
        e8_2bit_nll, e8_2bit_delta, e8_2bit_kl, e8_2bit_agr
    );

    let arm2 = ArmMetrics {
        arm_id: 2,
        name: "e8_2bit".to_string(),
        title: "E8 Lattice 2-bit (1-stage, 2.0 bpw)".to_string(),
        target_bpw: 2.0,
        payload_bpw: e8_2bit_bpw,
        total_serialized_bytes: e8_2bit_bytes,
        quantized_weights_count: total_mlp_weights,
        mean_nll: e8_2bit_nll,
        delta_nll: e8_2bit_delta,
        mean_kl: e8_2bit_kl,
        top1_agreement_pct: e8_2bit_agr,
        window_nlls: e8_2bit_nlls,
        codec_roundtrip_gate_passed: e8_2bit_roundtrip_ok,
    };

    // -----------------------------------------------------------------------
    // Arm 3: E8 3-bit (2-stage residual E8 with single block scale, 3.0000 bpw)
    // -----------------------------------------------------------------------
    println!("\n[Arm 3] Quantizing MLP layers with E8 3-bit (2-stage residual E8)...");
    let e8_3bit_start = Instant::now();
    let mut e8_3bit_bytes = 0usize;
    let mut e8_3bit_roundtrip_ok = true;

    for l in 0..args.layers {
        for proj in &mlp_projections {
            let name = format!("model.layers.{l}.mlp.{proj}.weight");
            let orig_tensor = checkpoint
                .tensors
                .get(&name)
                .ok_or_else(|| err(format!("tensor {name} not found")))?;
            let dims = orig_tensor.dims();
            let (rows, cols) = (dims[0], dims[1]);
            let weights: Vec<f32> = orig_tensor.flatten_all()?.to_vec1()?;

            let matrix = E8ThreeBitMatrix::encode(&weights, rows, cols)?;
            let bytes = matrix.to_bytes();
            e8_3bit_bytes += bytes.len();

            let deserialized = E8ThreeBitMatrix::from_bytes(&bytes)?;
            if matrix != deserialized {
                e8_3bit_roundtrip_ok = false;
            }

            let deq = deserialized.dequantize()?;
            let new_tensor = Tensor::from_vec(deq, (rows, cols), &device)?;
            model.set_tensor(&name, new_tensor)?;
        }
    }

    let e8_3bit_bpw = (e8_3bit_bytes as f64 * 8.0) / (total_mlp_weights as f64);
    println!(
        "  E8 3-bit encoded in {:.2}s (bytes: {}, bpw: {:.4}, roundtrip gate: {})",
        e8_3bit_start.elapsed().as_secs_f64(),
        e8_3bit_bytes,
        e8_3bit_bpw,
        if e8_3bit_roundtrip_ok {
            "PASSED"
        } else {
            "FAILED"
        }
    );

    let (e8_3bit_nll, e8_3bit_kl, e8_3bit_agr, e8_3bit_nlls) = evaluate_current_model(&model)?;
    let e8_3bit_delta = e8_3bit_nll - float_mean_nll;
    println!(
        "  E8 3-bit NLL: {:.6} (Delta: {:+.6} nats, KL: {:.6}, Agr: {:.2}%)",
        e8_3bit_nll, e8_3bit_delta, e8_3bit_kl, e8_3bit_agr
    );

    // Evaluate Kill Criterion: 3-bit E8 > RTN 4-bit + 0.05 nats
    let excess_vs_rtn = e8_3bit_delta - rtn_delta;
    let kill_triggered = excess_vs_rtn > 0.05;
    println!(
        "  Kill check: E8 3-bit Delta ({:+.6}) vs RTN 4-bit Delta ({:+.6}) -> Excess: {:+.6} nats (Threshold: +0.05 nats)",
        e8_3bit_delta, rtn_delta, excess_vs_rtn
    );
    if kill_triggered {
        println!(
            "  >>> KILL CRITERION TRIGGERED: E8 3-bit is > 0.05 nats worse than RTN 4-bit! <<<"
        );
    } else {
        println!(
            "  >>> Kill criterion NOT triggered: E8 3-bit is within +0.05 nats of RTN 4-bit. <<<"
        );
    }

    let arm3 = ArmMetrics {
        arm_id: 3,
        name: "e8_3bit".to_string(),
        title: "E8 Lattice 3-bit (2-stage residual, 3.0 bpw)".to_string(),
        target_bpw: 3.0,
        payload_bpw: e8_3bit_bpw,
        total_serialized_bytes: e8_3bit_bytes,
        quantized_weights_count: total_mlp_weights,
        mean_nll: e8_3bit_nll,
        delta_nll: e8_3bit_delta,
        mean_kl: e8_3bit_kl,
        top1_agreement_pct: e8_3bit_agr,
        window_nlls: e8_3bit_nlls,
        codec_roundtrip_gate_passed: e8_3bit_roundtrip_ok,
    };

    // -----------------------------------------------------------------------
    // Arm 4: E8 4-bit (2-stage residual E8 with two scales, 4.0000 bpw)
    // -----------------------------------------------------------------------
    println!("\n[Arm 4] Quantizing MLP layers with E8 4-bit (2-stage residual E8, 2 scales)...");
    let e8_4bit_start = Instant::now();
    let mut e8_4bit_bytes = 0usize;
    let mut e8_4bit_roundtrip_ok = true;

    for l in 0..args.layers {
        for proj in &mlp_projections {
            let name = format!("model.layers.{l}.mlp.{proj}.weight");
            let orig_tensor = checkpoint
                .tensors
                .get(&name)
                .ok_or_else(|| err(format!("tensor {name} not found")))?;
            let dims = orig_tensor.dims();
            let (rows, cols) = (dims[0], dims[1]);
            let weights: Vec<f32> = orig_tensor.flatten_all()?.to_vec1()?;

            let matrix = E8FourBitMatrix::encode(&weights, rows, cols)?;
            let bytes = matrix.to_bytes();
            e8_4bit_bytes += bytes.len();

            let deserialized = E8FourBitMatrix::from_bytes(&bytes)?;
            if matrix != deserialized {
                e8_4bit_roundtrip_ok = false;
            }

            let deq = deserialized.dequantize()?;
            let new_tensor = Tensor::from_vec(deq, (rows, cols), &device)?;
            model.set_tensor(&name, new_tensor)?;
        }
    }

    let e8_4bit_bpw = (e8_4bit_bytes as f64 * 8.0) / (total_mlp_weights as f64);
    println!(
        "  E8 4-bit encoded in {:.2}s (bytes: {}, bpw: {:.4}, roundtrip gate: {})",
        e8_4bit_start.elapsed().as_secs_f64(),
        e8_4bit_bytes,
        e8_4bit_bpw,
        if e8_4bit_roundtrip_ok {
            "PASSED"
        } else {
            "FAILED"
        }
    );

    let (e8_4bit_nll, e8_4bit_kl, e8_4bit_agr, e8_4bit_nlls) = evaluate_current_model(&model)?;
    let e8_4bit_delta = e8_4bit_nll - float_mean_nll;
    println!(
        "  E8 4-bit NLL: {:.6} (Delta: {:+.6} nats, KL: {:.6}, Agr: {:.2}%)",
        e8_4bit_nll, e8_4bit_delta, e8_4bit_kl, e8_4bit_agr
    );

    let arm4 = ArmMetrics {
        arm_id: 4,
        name: "e8_4bit".to_string(),
        title: "E8 Lattice 4-bit (2-stage residual, 4.0 bpw)".to_string(),
        target_bpw: 4.0,
        payload_bpw: e8_4bit_bpw,
        total_serialized_bytes: e8_4bit_bytes,
        quantized_weights_count: total_mlp_weights,
        mean_nll: e8_4bit_nll,
        delta_nll: e8_4bit_delta,
        mean_kl: e8_4bit_kl,
        top1_agreement_pct: e8_4bit_agr,
        window_nlls: e8_4bit_nlls,
        codec_roundtrip_gate_passed: e8_4bit_roundtrip_ok,
    };

    // -----------------------------------------------------------------------
    // Generate Report Artifacts
    // -----------------------------------------------------------------------
    println!(
        "\nGenerating sealed report artifacts in {}...",
        args.out.display()
    );
    let all_arms = vec![arm0, arm1, arm2, arm3, arm4];

    let summary = json!({
        "schema": "uor-r4.b3-e8-smollm2-mlp/1",
        "model": {
            "path": args.model_dir.display().to_string(),
            "weights_sha256": checkpoint.weights_sha256,
            "architecture": "LlamaForCausalLM",
            "layers": args.layers,
            "hidden_size": checkpoint.shape.width,
            "intermediate_size": checkpoint.shape.ffn,
            "vocab_size": checkpoint.shape.vocab,
        },
        "evaluation": {
            "tokens_path": args.tokens_path.display().to_string(),
            "windows": args.windows,
            "time": args.time,
            "batch": args.batch,
            "total_tokens": total_eval_tokens,
        },
        "quantization_scope": {
            "target": "mlp_layers",
            "matrices_per_layer": ["gate_proj", "up_proj", "down_proj"],
            "total_matrices_quantized": args.layers * 3,
            "total_weights_quantized": total_mlp_weights,
        },
        "kill_criterion": {
            "definition": "Delta NLL(E8 3-bit) - Delta NLL(RTN 4-bit) > 0.05 nats",
            "excess_vs_rtn_nats": excess_vs_rtn,
            "threshold_nats": 0.05,
            "triggered": kill_triggered,
            "verdict": if kill_triggered { "KILL (negative preserved)" } else { "PASS (viable candidate)" },
        },
        "arms": all_arms,
    });

    fs::write(
        args.out.join("evaluation_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;

    report_output::seal(&args.out)?;
    report_output::verify(&args.out)?;

    println!(
        "\n=== B3 E8 Study Complete. Report root sealed and verified: {} ===",
        args.out.display()
    );

    Ok(())
}
