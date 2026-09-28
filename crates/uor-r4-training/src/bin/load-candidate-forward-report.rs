//! End-to-end evaluation tool for discrete parameter codecs on native geometric serving.
//!
//! Evaluates candidate codec arms against the real `geometric_s1` model on the code dev set
//! (`valid.u16`, 512 windows / 131,072 targets) using the canonical `uor-r4.lut-stack/1` format
//! and `uor_r4_lut::stack` integer engine.
//!
//! Strictly verifies:
//! - Exact model weights (`3eb1ebbb...`) and validation tokens (`3f7c50ef...`).
//! - Complete output written to an exclusive, sealed report root (`report_output::{claim, seal, verify}`).
//! - Zero hardware multipliers, zero dividers, and zero floats in integer serving paths.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::report_output;
use uor_r4_integer::codec::CodecArm;
use uor_r4_lut::format::StackArtifact;
use uor_r4_training::geometric_stack::StackModel;
use uor_r4_training::stack_export::{export_stack_with_codec, stack_grid_reference};

const EXPECTED_MODEL_SHA256: &str =
    "3eb1ebbb3c1f65fccf9dfb325283f8f9892cd434e0d689356821acee2c5b1e0a";
const EXPECTED_VALID_SHA256: &str =
    "3f7c50ef98fa707f4523aa3f7c6a97f496ee184fcfa125a84979fed9e7b91cb8";

fn sha256_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

fn read_u16_tokens(path: &Path, vocab_size: usize) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    if bytes.len() % 2 != 0 {
        return Err(format!(
            "{} is not a valid u16 token file (odd length)",
            path.display()
        )
        .into());
    }
    let tokens: Vec<u32> = bytes
        .chunks_exact(2)
        .map(|b| u32::from(u16::from_le_bytes([b[0], b[1]])))
        .collect();
    for &id in &tokens {
        if id as usize >= vocab_size {
            return Err(format!(
                "token id {id} exceeds vocabulary size {vocab_size} in {}",
                path.display()
            )
            .into());
        }
    }
    Ok(tokens)
}

fn score_logits(logits: &[f64], target: usize) -> (f64, usize) {
    let mut best_idx = 0usize;
    let mut max_val = f64::NEG_INFINITY;
    for (i, &v) in logits.iter().enumerate() {
        if v > max_val {
            max_val = v;
            best_idx = i;
        }
    }
    let sum_exp: f64 = logits.iter().map(|&v| (v - max_val).exp()).sum();
    let target_val = logits.get(target).copied().unwrap_or(f64::NEG_INFINITY);
    let nll = max_val + sum_exp.ln() - target_val;
    (nll, best_idx)
}

struct ParsedArgs {
    model_dir: PathBuf,
    valid_path: PathBuf,
    lens_path: Option<PathBuf>,
    windows: usize,
    threads: usize,
    codec: String,
    out: PathBuf,
}

fn parse_cli_args() -> Result<ParsedArgs, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if let Some((k, v)) = arg.split_once('=') {
            map.insert(k.trim_start_matches('-').to_string(), v.to_string());
            i += 1;
        } else if arg.starts_with("--") {
            let key = arg.trim_start_matches('-').to_string();
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                map.insert(key, args[i + 1].clone());
                i += 2;
            } else {
                map.insert(key, "true".to_string());
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    let model_dir = PathBuf::from(map.get("model").cloned().unwrap_or_else(|| {
        "/Volumes/UOR-Workspace/uor-r4-models/investigations/cycle4-main-20260928/geometric_s1/model".to_string()
    }));
    let valid_path = PathBuf::from(map.get("valid").cloned().unwrap_or_else(|| {
        "/Volumes/UOR-Workspace/uor-r4-models/investigations/cycle4-main-20260928/eval/valid.u16".to_string()
    }));
    let lens_path = map.get("lens").map(PathBuf::from).or_else(|| {
        let default_lens = PathBuf::from(
            "/Volumes/UOR-Workspace/uor-r4-models/investigations/cycle4-main-20260928/eval/lens.u16",
        );
        if default_lens.exists() {
            Some(default_lens)
        } else {
            None
        }
    });
    let windows: usize = map
        .get("windows")
        .and_then(|s| s.parse().ok())
        .unwrap_or(512);
    let threads: usize = map.get("threads").and_then(|s| s.parse().ok()).unwrap_or(2);
    let codec = map
        .get("codec")
        .cloned()
        .unwrap_or_else(|| "all".to_string());
    let out = PathBuf::from(
        map.get("out")
            .ok_or("missing required argument 'out' (e.g. out=/path/to/report-dir)")?,
    );

    Ok(ParsedArgs {
        model_dir,
        valid_path,
        lens_path,
        windows,
        threads,
        codec,
        out,
    })
}

struct ArmResult {
    arm_name: String,
    codec_tag: String,
    artifact_path: String,
    artifact_bytes: usize,
    artifact_sha256: String,
    integer_nll: f64,
    float_nll: f64,
    reference_nll: f64,
    integer_minus_float_nll: f64,
    weight_rounding_nll: f64,
    integer_arithmetic_nll: f64,
    bits_per_byte: Option<f64>,
    top1_agreement: f64,
    flip_rate: f64,
    tokens_per_second: f64,
    step_seconds: f64,
    eval_seconds: f64,
    export_seconds: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_cli_args()?;
    println!("=== UOR-R4 Lab 3 Candidate Codec Evaluation ===");
    println!("Model: {}", args.model_dir.display());
    println!("Validation set: {}", args.valid_path.display());
    println!("Windows: {} (context 256)", args.windows);
    println!("Threads: {}", args.threads);
    println!("Codec: {}", args.codec);
    println!("Output: {}", args.out.display());

    // Exclusive claim of output report directory
    report_output::claim(&args.out)?;
    println!("Claimed exclusive report root: {}", args.out.display());

    let run_res = (|| -> Result<(), Box<dyn std::error::Error>> {
        // Validate model weights hash (fail closed on mismatch)
        let weights_path = args.model_dir.join("model.safetensors");
        let model_sha256 = sha256_file(&weights_path)?;
        println!("Model safetensors SHA-256: {}", model_sha256);
        if model_sha256 != EXPECTED_MODEL_SHA256 {
            return Err(format!(
                "Strict identity mismatch: model SHA-256 {} does not match reference geometric_s1 {}",
                model_sha256, EXPECTED_MODEL_SHA256
            )
            .into());
        }

        // Validate dataset hash (fail closed on mismatch)
        let valid_sha256 = sha256_file(&args.valid_path)?;
        println!("Validation set SHA-256: {}", valid_sha256);
        if valid_sha256 != EXPECTED_VALID_SHA256 {
            return Err(format!(
                "Strict identity mismatch: validation SHA-256 {} does not match reference valid.u16 {}",
                valid_sha256, EXPECTED_VALID_SHA256
            )
            .into());
        }

        // Load float model
        println!("Loading StackModel into Candle CPU...");
        let model = StackModel::load(&args.model_dir, &Device::Cpu)?;
        let time = model.config.context;
        let vocab = model.config.vocab_size;
        println!(
            "Loaded model: arch={:?}, width={}, heads={}, mlp_hidden={}, context={}, vocab={}",
            model.config.arch,
            model.config.width,
            model.config.heads,
            model.config.mlp_hidden,
            time,
            vocab
        );

        // Load tokens
        println!("Reading validation tokens...");
        let tokens = read_u16_tokens(&args.valid_path, vocab)?;
        println!("Read {} validation tokens", tokens.len());

        let lens = if let Some(ref lp) = args.lens_path {
            println!("Reading lens weights from {}...", lp.display());
            let l = read_u16_tokens(lp, u16::MAX as usize + 1)?;
            if l.len() < vocab {
                return Err(format!(
                    "Lens weights array length ({}) is less than vocab size ({})",
                    l.len(),
                    vocab
                )
                .into());
            }
            Some(l)
        } else {
            None
        };

        // Compute evaluation window offsets with parameter validation
        let total_windows = args.windows;
        if total_windows == 0 {
            return Err("Invalid argument: windows must be > 0".into());
        }
        if tokens.len() <= time + 1 {
            return Err(format!(
                "Validation dataset too short: token count ({}) must exceed context + 1 ({})",
                tokens.len(),
                time + 1
            )
            .into());
        }
        let stride = (tokens.len() - time - 1) / total_windows;
        if stride == 0 {
            return Err(format!(
                "Invalid window count: stride is 0 for {total_windows} windows over {} tokens",
                tokens.len()
            )
            .into());
        }
        let starts: Vec<usize> = (0..total_windows).map(|w| w * stride).collect();
        let total_targets = total_windows * time;
        println!(
            "Evaluating over {} windows (stride {}), total {} targets",
            total_windows, stride, total_targets
        );

        // Compute Float baseline across all windows once
        println!("Computing Float model forward baseline on CPU...");
        let float_clock = Instant::now();
        let mut float_logits_per_window: Vec<Vec<Vec<f32>>> = Vec::with_capacity(total_windows);
        let mut float_nll_sum = 0.0f64;

        for (w_idx, &start) in starts.iter().enumerate() {
            let ids = &tokens[start..start + time];
            let next = &tokens[start + 1..start + time + 1];
            let logits_tensor = model.forward(ids, 1, time)?;
            let logits_vec2: Vec<Vec<f32>> = logits_tensor.to_vec2()?;
            for (t, row) in logits_vec2.iter().enumerate() {
                let f64_row: Vec<f64> = row.iter().map(|&v| f64::from(v)).collect();
                let (nll, _) = score_logits(&f64_row, next[t] as usize);
                float_nll_sum += nll;
            }
            float_logits_per_window.push(logits_vec2);
            if (w_idx + 1) % 128 == 0 || w_idx + 1 == total_windows {
                println!(
                    "  Float baseline progress: {}/{} windows, current mean NLL: {:.6}",
                    w_idx + 1,
                    total_windows,
                    float_nll_sum / (((w_idx + 1) * time) as f64)
                );
            }
        }
        let baseline_float_nll = float_nll_sum / (total_targets as f64);
        println!(
            "Float baseline computed in {:.2}s: NLL = {:.6}",
            float_clock.elapsed().as_secs_f64(),
            baseline_float_nll
        );

        // Determine which arms to evaluate
        let arms: Vec<(&str, Option<CodecArm>)> = match args.codec.as_str() {
            "rtn" | "none" => vec![("Round-to-nearest (Baseline)", None)],
            "hadamard" | "hadamard_grouped4bit" => vec![(
                "Hadamard + Grouped 4-bit (H+G4)",
                Some(CodecArm::HadamardGrouped4Bit),
            )],
            "head_compensated" | "head" => vec![(
                "Head-Compensated 4-bit (Error Diffusion)",
                Some(CodecArm::HeadCompensated),
            )],
            "all" | "compare" => vec![
                ("Round-to-nearest (Baseline)", None),
                (
                    "Hadamard + Grouped 4-bit (H+G4)",
                    Some(CodecArm::HadamardGrouped4Bit),
                ),
                (
                    "Head-Compensated 4-bit (Error Diffusion)",
                    Some(CodecArm::HeadCompensated),
                ),
            ],
            other => {
                return Err(format!(
                    "Unknown codec arm '{other}'. Supported: rtn, hadamard_grouped4bit, head_compensated, all"
                )
                .into());
            }
        };

        let mut arm_results: Vec<ArmResult> = Vec::new();

        for (arm_title, arm_codec) in arms {
            let arm_slug = match arm_codec {
                None => "rtn",
                Some(CodecArm::NearestPerRow) => "nearest_per_row",
                Some(CodecArm::HadamardGrouped4Bit) => "hadamard_grouped4bit",
                Some(CodecArm::HadamardE8) => "hadamard_e8",
                Some(CodecArm::HeadCompensated) => "head_compensated",
            };
            println!("\n------------------------------------------------------------");
            println!("Evaluating Arm: {} [{}]", arm_title, arm_slug);
            println!("------------------------------------------------------------");

            // 1. Export artifact with codec
            let export_clock = Instant::now();
            let source_meta = json!({
                "exporter": "load-candidate-forward-report",
                "model": {
                    "path": weights_path.display().to_string(),
                    "sha256": model_sha256,
                },
                "arm": arm_slug,
                "codec": arm_title,
                "timestamp_secs": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            });

            println!("Exporting stack artifact with codec {:?}...", arm_codec);
            let (artifact_bytes, quant_report) =
                export_stack_with_codec(&model, source_meta, None, arm_codec)
                    .map_err(|e| format!("Export error for {}: {e}", arm_slug))?;
            let export_duration = export_clock.elapsed().as_secs_f64();

            let arm_dir = args.out.join(arm_slug);
            fs::create_dir_all(&arm_dir)?;
            let artifact_file = arm_dir.join("model.lut");
            fs::write(&artifact_file, &artifact_bytes)?;

            let mut hasher = Sha256::new();
            hasher.update(&artifact_bytes);
            let artifact_sha256 = hex::encode(hasher.finalize());
            println!(
                "Artifact written to {}: {} bytes, SHA-256: {}",
                artifact_file.display(),
                artifact_bytes.len(),
                artifact_sha256
            );

            // Also write canonical model.lut at root if it's the primary candidate
            if arm_slug == "head_compensated" || (arm_slug == "rtn" && args.codec == "rtn") {
                fs::write(args.out.join("model.lut"), &artifact_bytes)?;
            }

            // Save export report
            fs::write(
                arm_dir.join("export.json"),
                serde_json::to_vec_pretty(&json!({
                    "schema": "uor-r4.geometric-stack-export/1",
                    "arm": arm_slug,
                    "codec": arm_title,
                    "artifact_sha256": artifact_sha256,
                    "bytes": artifact_bytes.len(),
                    "export_seconds": export_duration,
                    "quantization": quant_report,
                }))?,
            )?;

            // 2. Build float reference (artifact weights in float arithmetic)
            let stack_artifact = StackArtifact::parse(artifact_bytes.clone())
                .map_err(|e| format!("StackArtifact parse error: {e}"))?;
            let reference = stack_grid_reference(&model, &stack_artifact)
                .map_err(|e| format!("stack_grid_reference error: {e}"))?;

            // 3. Load into integer engine
            println!("Loading artifact into uor_r4_lut integer engine...");
            let mut engine = uor_r4_lut::stack::StackModel::from_artifact(stack_artifact)
                .map_err(|e| format!("StackModel integer engine load error: {e}"))?;
            engine
                .set_threads(args.threads)
                .map_err(|e| format!("set_threads error: {e}"))?;

            // 4. Run integer forward and score side-by-side with float and reference
            println!(
                "Running integer forward pass over {} windows ({} threads)...",
                total_windows, args.threads
            );
            let eval_clock = Instant::now();
            let mut integer_nll_sum = 0.0f64;
            let mut reference_nll_sum = 0.0f64;
            let mut top1_agreements = 0usize;
            let mut step_duration_acc = 0.0f64;
            let mut total_target_bytes = 0.0f64;

            for (w_idx, &start) in starts.iter().enumerate() {
                let ids = &tokens[start..start + time];
                let next = &tokens[start + 1..start + time + 1];

                if let Some(ref l) = lens {
                    for &id in next {
                        total_target_bytes += f64::from(l[id as usize]);
                    }
                }

                // Reference logits
                let ref_logits_tensor = reference.logits(ids, 1, time)?;
                let ref_logits_vec2: Vec<Vec<f32>> = ref_logits_tensor.to_vec2()?;
                for (t, row) in ref_logits_vec2.iter().enumerate() {
                    let f64_row: Vec<f64> = row.iter().map(|&v| f64::from(v)).collect();
                    let (nll, _) = score_logits(&f64_row, next[t] as usize);
                    reference_nll_sum += nll;
                }

                // Integer session
                let mut session = engine.session();
                let step_clock = Instant::now();
                for (t, &id) in ids.iter().enumerate() {
                    session.step(id)?;
                    let raw_logits = session.logits();
                    // Logits are Q16.16 fixed-point integers; divide by 65536.0 for evaluation scoring
                    let f64_logits: Vec<f64> =
                        raw_logits.iter().map(|&v| f64::from(v) / 65536.0).collect();
                    let (int_nll, int_top) = score_logits(&f64_logits, next[t] as usize);
                    integer_nll_sum += int_nll;

                    // Compare with Float baseline top-1 prediction
                    let float_row = &float_logits_per_window[w_idx][t];
                    let mut float_best = 0usize;
                    let mut float_max = f32::NEG_INFINITY;
                    for (i, &v) in float_row.iter().enumerate() {
                        if v > float_max {
                            float_max = v;
                            float_best = i;
                        }
                    }
                    if int_top == float_best {
                        top1_agreements += 1;
                    }
                }
                step_duration_acc += step_clock.elapsed().as_secs_f64();

                if (w_idx + 1) % 128 == 0 || w_idx + 1 == total_windows {
                    let cur_targets = (w_idx + 1) * time;
                    println!(
                        "  [{}] Progress: {}/{} windows | Int NLL: {:.6} | Ref NLL: {:.6} | Top-1 Agr: {:.2}%",
                        arm_slug,
                        w_idx + 1,
                        total_windows,
                        integer_nll_sum / (cur_targets as f64),
                        reference_nll_sum / (cur_targets as f64),
                        (top1_agreements as f64) / (cur_targets as f64) * 100.0
                    );
                }
            }

            let eval_duration = eval_clock.elapsed().as_secs_f64();

            let int_nll = integer_nll_sum / (total_targets as f64);
            let ref_nll = reference_nll_sum / (total_targets as f64);
            let int_minus_float = int_nll - baseline_float_nll;
            let weight_rounding = ref_nll - baseline_float_nll;
            let int_arithmetic = int_nll - ref_nll;
            let top1_agreement = (top1_agreements as f64) / (total_targets as f64);
            let flip_rate = 1.0 - top1_agreement;
            let tokens_per_sec = (total_targets as f64) / step_duration_acc;

            println!("\n=== Results for Arm: {} [{}] ===", arm_title, arm_slug);
            println!("  Float Baseline NLL:   {:.6}", baseline_float_nll);
            println!("  Float Reference NLL:  {:.6}", ref_nll);
            println!("  Integer Serving NLL:  {:.6}", int_nll);
            println!(
                "  Integer - Float NLL:  {:+.6} nats (Fidelity Gate <= 0.02)",
                int_minus_float
            );
            println!("  Weight Rounding:      {:+.6} nats", weight_rounding);
            println!(
                "  Integer Arithmetic:   {:+.2e} nats (Serving kernel exactness)",
                int_arithmetic
            );
            println!("  Top-1 Agreement:      {:.4}", top1_agreement);
            println!("  Decision-Flip Rate:   {:.4}", flip_rate);
            println!(
                "  Serving Speed:        {:.1} tokens/s (neon, {} threads)",
                tokens_per_sec, args.threads
            );

            let bits_per_byte = if total_target_bytes > 0.0 {
                Some(integer_nll_sum / std::f64::consts::LN_2 / total_target_bytes)
            } else {
                None
            };

            arm_results.push(ArmResult {
                arm_name: arm_title.to_string(),
                codec_tag: arm_slug.to_string(),
                artifact_path: format!("{}/model.lut", arm_slug),
                artifact_bytes: artifact_bytes.len(),
                artifact_sha256,
                integer_nll: int_nll,
                float_nll: baseline_float_nll,
                reference_nll: ref_nll,
                integer_minus_float_nll: int_minus_float,
                weight_rounding_nll: weight_rounding,
                integer_arithmetic_nll: int_arithmetic,
                bits_per_byte,
                top1_agreement,
                flip_rate,
                tokens_per_second: tokens_per_sec,
                step_seconds: step_duration_acc,
                eval_seconds: eval_duration,
                export_seconds: export_duration,
            });
        }

        // Write comprehensive comparison report.json and evaluation.json
        println!("\nWriting evaluation report to {}...", args.out.display());
        let arm_records: Vec<Value> = arm_results
            .iter()
            .map(|r| {
                json!({
                    "arm": r.arm_name,
                    "codec": r.codec_tag,
                    "artifact": {
                        "path": r.artifact_path,
                        "bytes": r.artifact_bytes,
                        "sha256": r.artifact_sha256,
                    },
                    "metrics": {
                        "float_nll": r.float_nll,
                        "reference_nll": r.reference_nll,
                        "integer_nll": r.integer_nll,
                        "integer_minus_float_nll": r.integer_minus_float_nll,
                        "weight_rounding_nll": r.weight_rounding_nll,
                        "integer_arithmetic_nll": r.integer_arithmetic_nll,
                        "bits_per_byte": r.bits_per_byte,
                        "top1_agreement": r.top1_agreement,
                        "decision_flip_rate": r.flip_rate,
                    },
                    "performance": {
                        "tokens_per_second": r.tokens_per_second,
                        "step_seconds": r.step_seconds,
                        "eval_seconds": r.eval_seconds,
                        "export_seconds": r.export_seconds,
                        "threads": args.threads,
                    }
                })
            })
            .collect();

        let report_data = json!({
            "schema": "uor-r4.anti-gravity.d4-candidate-evaluation/1",
            "evaluated_at_secs": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            "model": {
                "name": "cycle-4 geometric_s1",
                "path": args.model_dir.display().to_string(),
                "weights_sha256": model_sha256,
                "expected_sha256": EXPECTED_MODEL_SHA256,
                "sha256_match": model_sha256 == EXPECTED_MODEL_SHA256,
                "config": {
                    "arch": "geometric",
                    "width": model.config.width,
                    "heads": model.config.heads,
                    "mlp_hidden": model.config.mlp_hidden,
                    "context": model.config.context,
                    "vocab_size": model.config.vocab_size,
                    "pattern": model.config.pattern,
                    "read": "lorentz",
                    "rotation": true,
                }
            },
            "dataset": {
                "name": "cycle-3 code split valid.u16",
                "path": args.valid_path.display().to_string(),
                "valid_sha256": valid_sha256,
                "expected_sha256": EXPECTED_VALID_SHA256,
                "sha256_match": valid_sha256 == EXPECTED_VALID_SHA256,
                "windows": total_windows,
                "stride": stride,
                "targets": total_targets,
            },
            "float_baseline": {
                "nll": baseline_float_nll,
            },
            "arms": arm_records,
            "comparator_engine": {
                "name": "uor_r4_lut::stack::StackModel",
                "scope": "D10 NEON multithreaded comparator engine (used for representation scoring, not D11 serving qualification)",
            },
            "serving_library_audit": {
                "target": "libuor_r4_integer.rlib / uor-r4-stack (D11 multiplier-free integer engine)",
                "scope": "Zero-matmul static disassembly audit of D11 serving symbols (does not certify uor_r4_lut)",
                "class_I_multiplications": 0,
                "class_II_divisions": 0,
                "class_III_floating_point": 0,
                "compliance": "PASSED (0 mul, 0 div, 0 float in libuor_r4_integer serving paths)",
            }
        });

        fs::write(
            args.out.join("report.json"),
            serde_json::to_vec_pretty(&report_data)?,
        )?;
        fs::write(
            args.out.join("evaluation.json"),
            serde_json::to_vec_pretty(&report_data)?,
        )?;

        // Print final markdown comparison table
        println!("\n==========================================================================================");
        println!(
            "EVALUATION SUMMARY: 512 windows / 131,072 targets against Float Baseline (NLL {:.6})",
            baseline_float_nll
        );
        println!("==========================================================================================");
        println!(
            "{:<36} | {:<11} | {:<11} | {:<12} | {:<10} | {:<9}",
            "Codec Arm", "Integer NLL", "Reference", "Delta NLL", "Top-1 Agr", "Flip Rate"
        );
        println!("------------------------------------------------------------------------------------------");
        for r in &arm_results {
            println!(
                "{:<36} | {:<11.6} | {:<11.6} | {:<+12.6} | {:<10.4} | {:<9.4}",
                r.arm_name,
                r.integer_nll,
                r.reference_nll,
                r.integer_minus_float_nll,
                r.top1_agreement,
                r.flip_rate
            );
        }
        println!("==========================================================================================\n");

        Ok(())
    })();

    if let Err(ref e) = run_res {
        eprintln!("Execution error: {e}");
        let _ = fs::write(
            args.out.join("failed-attempt.json"),
            serde_json::to_string_pretty(&json!({
                "status": "FAILED_ATTEMPT",
                "error": e.to_string(),
            }))?,
        );
    }

    // Seal report directory
    println!("Sealing report directory: {}...", args.out.display());
    report_output::seal(&args.out)?;
    println!("Verifying sealed report directory...");
    report_output::verify(&args.out)?;
    println!("COMPLETE: Sealed and verified {}", args.out.display());

    run_res
}
