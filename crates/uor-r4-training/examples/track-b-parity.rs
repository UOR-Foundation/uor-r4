//! Fixed all-logit parity gate for both offline Track B Llama implementations.
//! cargo run --release -p uor-r4-training --features metal --example track-b-parity -- MODEL NEW_REPORT
//! Every window resets oracle/stock caches; the shared model receives fresh full
//! prefixes. These synthetic tokens test numerical fidelity, not language
//! quality, and the reported elapsed rates are not serving benchmarks.
use candle_core::Device;
use serde_json::{json, Value};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::report_output;
use uor_r4_model_source::{BehaviorSource, HuggingFaceLlamaOracle, TeacherExecutionConfig};
use uor_r4_training::{
    sha256_file,
    track_b::{conversion::CandleLlamaTeacher, model::TrackBModel},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const TOLERANCE: f64 = 1e-4;
const VOCAB: usize = 49152;
const WALL_SECONDS: u64 = 600;
const SHARED_STORAGE_PATH: &str = "/Volumes/UOR-Workspace";
// Prospective correction: the plan's 30 GiB is a Track B trace allocation,
// not a free-space floor. This conservative lab guard is recorded separately.
const STORAGE_RESERVE_BYTES: u64 = 24 * 1024 * 1024 * 1024;
const STORAGE_STOP_MARGIN_BYTES: u64 = 128 * 1024 * 1024;
const REPORT_ALLOWANCE_BYTES: u64 = 96 * 1024 * 1024;
const STOCK_ID: &str = "candle-transformers/0.9.2::models::llama::Llama";
const SHARED_ID: &str = "uor-r4-training::track_b::model::TrackBModel+DenseAttention";
const STOCK_MODES: &[(&str, usize)] = &[
    ("singleton_every_position", 45),
    ("fresh_full_prefill_final", 4),
    ("fresh_half_prefix_then_singletons_final", 4),
];
const SHARED_MODES: &[(&str, usize)] = &[
    ("fresh_full_prefix_every_position", 45),
    ("fresh_batch2_right_padding_row0_every_position", 45),
    ("fresh_batch2_right_padding_row1_position0", 4),
];

fn windows() -> Vec<Vec<u32>> {
    vec![
        vec![1],
        vec![1, 42, 17, 314],
        (1..=8).collect(),
        (0..32).map(|i| 1 + ((9973 * i + 17) % 49151)).collect(),
    ]
}
fn logsumexp(xs: &[f32]) -> f64 {
    let m = xs
        .iter()
        .map(|&v| f64::from(v))
        .fold(f64::NEG_INFINITY, f64::max);
    m + xs
        .iter()
        .map(|&v| (f64::from(v) - m).exp())
        .sum::<f64>()
        .ln()
}
fn compare(reference: &[f32], candidate: &[f32], target: Option<u32>) -> Result<Value> {
    if reference.len() != VOCAB
        || candidate.len() != VOCAB
        || reference.iter().chain(candidate).any(|x| !x.is_finite())
    {
        return Err("logit shape mismatch or nonfinite value".into());
    }
    let mut max_abs = 0_f64;
    let mut max_id = 0;
    let mut squared = 0_f64;
    let mut signed = 0_f64;
    let mut over = 0_u64;
    for (i, (&a, &b)) in reference.iter().zip(candidate).enumerate() {
        let d = f64::from(b) - f64::from(a);
        if d.abs() > max_abs {
            max_abs = d.abs();
            max_id = i;
        }
        squared += d * d;
        signed += d;
        over += u64::from(d.abs() > TOLERANCE);
    }
    let rz = logsumexp(reference);
    let cz = logsumexp(candidate);
    let kl: f64 = reference
        .iter()
        .zip(candidate)
        .map(|(&r, &c)| {
            let lr = f64::from(r) - rz;
            lr.exp() * (lr - (f64::from(c) - cz))
        })
        .sum();
    Ok(json!({
        "cells": VOCAB, "finite": true, "max_abs": max_abs,
        "max_abs_vocab_id": max_id, "reference_at_max": reference[max_id],
        "candidate_at_max": candidate[max_id], "rms": (squared/VOCAB as f64).sqrt(),
        "mean_signed_delta": signed/VOCAB as f64, "cells_over_1e_4": over,
        "kl_reference_to_candidate_nats": kl,
        "target_id": target, "reference_nll": target.map(|t| rz-f64::from(reference[t as usize])),
        "candidate_nll": target.map(|t| cz-f64::from(candidate[t as usize])),
        "pass": max_abs <= TOLERANCE
    }))
}
fn write_logits(file: &mut fs::File, logits: &[f32]) -> Result<()> {
    let bytes: Vec<u8> = logits.iter().flat_map(|v| v.to_le_bytes()).collect();
    file.write_all(&bytes)?;
    Ok(())
}
fn write_json(path: &Path, value: &Value) -> Result<()> {
    let mut file = fs::File::create_new(path)?;
    file.write_all(&serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn check_wall(start: Instant) -> Result<()> {
    if start.elapsed().as_secs() >= WALL_SECONDS {
        Err("preregistered wall bound reached".into())
    } else {
        Ok(())
    }
}

fn parse_available_bytes(df: &str) -> Result<u64> {
    let mut lines = df.lines().filter(|line| !line.trim().is_empty());
    if !lines
        .next()
        .is_some_and(|header| header.starts_with("Filesystem"))
    {
        return Err("unrecognized POSIX df header".into());
    }
    let line = lines.next().ok_or("missing POSIX df filesystem row")?;
    if lines.next().is_some() {
        return Err("expected exactly one filesystem from POSIX df".into());
    }
    let available_kib = line
        .split_whitespace()
        .nth(3)
        .ok_or("missing POSIX df available-block field")?
        .parse::<u64>()?;
    available_kib
        .checked_mul(1024)
        .ok_or_else(|| "available-byte overflow".into())
}

fn available_storage_bytes() -> Result<u64> {
    // POSIX format keeps this one-filesystem report on one row. This driver is
    // the source-bound M1/Metal smoke; its prospective reserve names this SSD.
    let output = std::process::Command::new("/bin/df")
        .env("LC_ALL", "C")
        .args(["-Pk", SHARED_STORAGE_PATH])
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "storage observation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    parse_available_bytes(std::str::from_utf8(&output.stdout)?)
}

fn admit_storage() -> Result<u64> {
    let available = available_storage_bytes()?;
    let required = STORAGE_RESERVE_BYTES + STORAGE_STOP_MARGIN_BYTES + REPORT_ALLOWANCE_BYTES;
    if available < required {
        return Err(format!(
            "storage admission denied: available={available}, required={required} bytes"
        )
        .into());
    }
    Ok(available)
}

fn record_watchdog_stop(start: Instant, status: &str, observation: Value) -> ! {
    let marker = json!({
        "schema": "uor-r4.track-b-parity-interruption/1", "status": status,
        "pass": false, "elapsed_seconds": start.elapsed().as_secs_f64(),
        "observation": observation, "completion": "NOT_CONFIRMED",
        "scope": "execution interruption; no numerical or model-quality decision"
    });
    // Job stderr must be retained outside the claimed root while it is live.
    // Never race a background marker write against the main thread's seal.
    eprintln!("{marker}");
    std::process::exit(if status == "UNSEALED_TIMEOUT" {
        124
    } else {
        125
    });
}

fn mode_counts(rows: &[Value]) -> std::collections::BTreeMap<String, usize> {
    let mut counts = std::collections::BTreeMap::new();
    for row in rows {
        let mode = row["mode"].as_str().unwrap_or("INVALID_MODE");
        *counts.entry(mode.to_owned()).or_default() += 1;
    }
    counts
}

fn complete_coverage(rows: &[Value], modes: &[(&str, usize)]) -> bool {
    let actual = mode_counts(rows);
    rows.len() == modes.iter().map(|(_, count)| *count).sum::<usize>()
        && actual.len() == modes.len()
        && modes
            .iter()
            .all(|(mode, count)| actual.get(*mode) == Some(count))
}

fn coverage_json(modes: &[(&str, usize)]) -> Value {
    json!(modes
        .iter()
        .copied()
        .collect::<std::collections::BTreeMap<_, _>>())
}

fn set_raw_location(row: &mut Value, filename: &str, raw_row: usize) {
    row["raw_file"] = json!(filename);
    row["raw_row"] = json!(raw_row);
    row["raw_byte_offset"] = json!(raw_row as u64 * VOCAB as u64 * 4);
}

fn run(model: &Path, out: &Path, start: Instant) -> Result<Value> {
    let source_revision = option_env!("TRACK_B_SOURCE_REVISION")
        .ok_or("admitted builds require TRACK_B_SOURCE_REVISION")?;
    let source_diff = option_env!("TRACK_B_SOURCE_DIFF_SHA256")
        .ok_or("admitted builds require TRACK_B_SOURCE_DIFF_SHA256")?;
    if std::env::var("TLESS_CANONICAL_DETERMINISTIC").as_deref() != Ok("0")
        || std::env::var_os("TLESS_EXACT_SCALAR").is_some()
    {
        return Err("requires TLESS_CANONICAL_DETERMINISTIC=0 and TLESS_EXACT_SCALAR unset".into());
    }
    let inputs = windows();
    let mut hashes = serde_json::Map::new();
    for name in [
        "config.json",
        "model.safetensors",
        "tokenizer.json",
        "tokenizer_config.json",
    ] {
        hashes.insert(name.into(), json!(sha256_file(&model.join(name))?));
    }
    let exe = std::env::current_exe()?;
    write_json(
        &out.join("inputs.json"),
        &json!({
            "schema": "uor-r4.track-b-parity-inputs/2", "windows": inputs,
            "model": model, "sha256": hashes, "executable_sha256": sha256_file(&exe)?,
        "source_revision": source_revision,
        "source_diff_sha256": source_diff,
            "candle_version": "0.9.2", "dtype": "F32", "stored_weights": "BF16",
            "canonical_math_env": "0", "exact_scalar_env": null, "tolerance": TOLERANCE,
            "wall_seconds": WALL_SECONDS,
            "scope": "fixed synthetic-token numerical fidelity; no language-quality or timing claim",
            "reference_rows": 45, "reference_workers": 2,
            "backends": ["cpu", "metal"],
            "expected_compared_rows_per_backend": 147,
            "expected_compared_rows_total": 294,
            "implementations": [
                {"identity": STOCK_ID, "expected_rows_per_backend": 53,
                 "expected_mode_counts": coverage_json(STOCK_MODES)},
                {"identity": SHARED_ID, "expected_rows_per_backend": 94,
                 "expected_mode_counts": coverage_json(SHARED_MODES),
                 "full_prefix_raw_rows_per_backend": 45,
                 "batch2_raw_rows_per_backend": 90,
                 "batch2": {"batch":2, "row0":"fixed window, every position compared",
                     "row1":"[1, 0, ..., 0], only position zero compared to reference window zero position zero",
                     "padding_token":0, "valid_lengths":"[window length, 1]",
                     "padding_semantics":"right padding; future pad tokens are causally excluded from valid positions",
                     "raw_order":"per window, batch row, position; padding outputs are retained but not scored"}}
            ],
            "raw_storage": {"dtype":"F32", "endianness":"little", "vocab":VOCAB,
                "all_raw_rows_including_reference_and_unscored_padding":421,
                "all_raw_bytes":421_u64 * VOCAB as u64 * 4,
                "row_metadata":"raw_file, raw_row, raw_byte_offset"}
        }),
    )?;
    check_wall(start)?;
    let ref_start = Instant::now();
    let workers = std::num::NonZeroUsize::new(2).ok_or("zero exact workers")?;
    let mut oracle = HuggingFaceLlamaOracle::load_with_sequence_length_and_execution(
        model,
        32,
        TeacherExecutionConfig::fixed_workers(workers),
    )
    .map_err(|e| format!("reference load: {e}"))?;
    let backend = oracle.exact_backend_report();
    if backend.arithmetic_owner != "uor-matmul exact GEMM" {
        return Err(format!(
            "unexpected reference arithmetic: {}",
            backend.arithmetic_owner
        )
        .into());
    }
    let mut reference_file = fs::File::create_new(out.join("reference-logits.f32le"))?;
    let mut reference = Vec::new();
    for (w, tokens) in inputs.iter().enumerate() {
        oracle.reset();
        let mut rows = Vec::new();
        for (position, &token) in tokens.iter().enumerate() {
            check_wall(start)?;
            let mut logits = vec![0_f32; VOCAB];
            oracle.step(token as usize, position, &mut logits);
            if logits.iter().any(|x| !x.is_finite()) {
                return Err("nonfinite exact logits".into());
            }
            write_logits(&mut reference_file, &logits)?;
            rows.push(logits);
            eprintln!(
                "reference window={w} position={position} elapsed_s={:.3}",
                start.elapsed().as_secs_f64()
            );
        }
        reference.push(rows);
    }
    let reference_seconds = ref_start.elapsed().as_secs_f64();
    let execution = oracle.execution_snapshot();
    drop(oracle);
    let mut stock_backends = Vec::new();
    let mut overall_pass = true;
    for name in ["cpu", "metal"] {
        check_wall(start)?;
        let device = match name {
            "cpu" => Device::Cpu,
            _ => Device::new_metal(0)?,
        };
        let load_start = Instant::now();
        let candidate = CandleLlamaTeacher::load(model, &device)?;
        if candidate.config().vocab_size != VOCAB {
            return Err("unexpected vocabulary".into());
        }
        let load_seconds = load_start.elapsed().as_secs_f64();
        let eval_start = Instant::now();
        let mut logits_file = fs::File::create_new(out.join(format!("{name}-logits.f32le")))?;
        let mut endpoints_file =
            fs::File::create_new(out.join(format!("{name}-prefill-logits.f32le")))?;
        let mut row_file = fs::File::create_new(out.join(format!("{name}-rows.jsonl")))?;
        let mut rows = Vec::new();
        let mut worst = 0_f64;
        let mut cells = 0_u64;
        let mut over = 0_u64;
        let mut singleton_raw_row = 0_usize;
        let mut endpoint_raw_row = 0_usize;
        for (w, tokens) in inputs.iter().enumerate() {
            check_wall(start)?;
            let actual = candidate.logits(tokens)?;
            if actual.len() != tokens.len() {
                return Err("candidate incomplete position coverage".into());
            }
            for (position, (r, c)) in reference[w].iter().zip(&actual).enumerate() {
                write_logits(&mut logits_file, c)?;
                let mut row = compare(r, c, tokens.get(position + 1).copied())?;
                row["window"] = json!(w);
                row["position"] = json!(position);
                row["mode"] = json!("singleton_every_position");
                row["implementation"] = json!(STOCK_ID);
                row["backend"] = json!(name);
                set_raw_location(&mut row, &format!("{name}-logits.f32le"), singleton_raw_row);
                singleton_raw_row += 1;
                worst = worst.max(row["max_abs"].as_f64().ok_or("missing max")?);
                cells += VOCAB as u64;
                over += row["cells_over_1e_4"].as_u64().ok_or("missing count")?;
                writeln!(row_file, "{}", serde_json::to_string(&row)?)?;
                rows.push(row);
            }
            for (mode, split) in [
                ("fresh_full_prefill_final", None),
                (
                    "fresh_half_prefix_then_singletons_final",
                    Some((tokens.len() / 2).max(1)),
                ),
            ] {
                check_wall(start)?;
                let endpoint = candidate.logits_prefill(tokens, split)?;
                write_logits(&mut endpoints_file, &endpoint)?;
                let r = reference[w].last().ok_or("empty reference")?;
                let mut row = compare(r, &endpoint, None)?;
                row["window"] = json!(w);
                row["position"] = json!(tokens.len() - 1);
                row["mode"] = json!(mode);
                row["implementation"] = json!(STOCK_ID);
                row["backend"] = json!(name);
                set_raw_location(
                    &mut row,
                    &format!("{name}-prefill-logits.f32le"),
                    endpoint_raw_row,
                );
                endpoint_raw_row += 1;
                worst = worst.max(row["max_abs"].as_f64().ok_or("missing max")?);
                cells += VOCAB as u64;
                over += row["cells_over_1e_4"].as_u64().ok_or("missing count")?;
                writeln!(row_file, "{}", serde_json::to_string(&row)?)?;
                rows.push(row);
            }
            eprintln!(
                "{name} window={w} max_abs={worst:.9} elapsed_s={:.3}",
                start.elapsed().as_secs_f64()
            );
        }
        let coverage_pass = complete_coverage(&rows, STOCK_MODES);
        let pass = over == 0 && coverage_pass;
        overall_pass &= pass;
        stock_backends.push(json!({
            "implementation": STOCK_ID,
            "backend": name, "pass": pass, "maximum_absolute_error": worst,
            "cells_over_1e_4": over, "compared_cells": cells,
            "expected_rows": 53, "completed_rows": rows.len(),
            "expected_mode_counts": coverage_json(STOCK_MODES),
            "completed_mode_counts": mode_counts(&rows), "coverage_pass": coverage_pass,
            "load_seconds": load_seconds, "evaluation_seconds": eval_start.elapsed().as_secs_f64(),
            "rows": rows
        }));
        // Release stock weights before any shared model is loaded. Both stock
        // backends are evaluated first, preserving the original 53-row gate.
        drop(candidate);
    }
    let mut shared_backends = Vec::new();
    for name in ["cpu", "metal"] {
        check_wall(start)?;
        let device = match name {
            "cpu" => Device::Cpu,
            _ => Device::new_metal(0)?,
        };
        let load_start = Instant::now();
        let candidate = TrackBModel::load(model, &device)?;
        if candidate.shape().vocab != VOCAB {
            return Err("unexpected shared-model vocabulary".into());
        }
        let load_seconds = load_start.elapsed().as_secs_f64();
        let eval_start = Instant::now();
        let full_filename = format!("{name}-shared-full-logits.f32le");
        let batch_filename = format!("{name}-shared-batch2-logits.f32le");
        let mut full_file = fs::File::create_new(out.join(&full_filename))?;
        let mut batch_file = fs::File::create_new(out.join(&batch_filename))?;
        let mut row_file = fs::File::create_new(out.join(format!("{name}-shared-rows.jsonl")))?;
        let mut rows = Vec::new();
        let mut worst = 0_f64;
        let mut cells = 0_u64;
        let mut over = 0_u64;
        let mut full_raw_row = 0_usize;
        let mut batch_raw_row = 0_usize;
        for (w, tokens) in inputs.iter().enumerate() {
            check_wall(start)?;
            let full = candidate.forward(tokens, 1, tokens.len())?;
            if full.dims() != [1, tokens.len(), VOCAB] {
                return Err("shared full-prefix logit shape mismatch".into());
            }
            let full = full.squeeze(0)?.to_vec2::<f32>()?;
            for (position, (r, c)) in reference[w].iter().zip(&full).enumerate() {
                write_logits(&mut full_file, c)?;
                let mut row = compare(r, c, tokens.get(position + 1).copied())?;
                row["implementation"] = json!(SHARED_ID);
                row["backend"] = json!(name);
                row["window"] = json!(w);
                row["batch_row"] = json!(0);
                row["position"] = json!(position);
                row["reference_window"] = json!(w);
                row["reference_position"] = json!(position);
                row["mode"] = json!("fresh_full_prefix_every_position");
                set_raw_location(&mut row, &full_filename, full_raw_row);
                full_raw_row += 1;
                worst = worst.max(row["max_abs"].as_f64().ok_or("missing shared max")?);
                cells += VOCAB as u64;
                over += row["cells_over_1e_4"]
                    .as_u64()
                    .ok_or("missing shared count")?;
                writeln!(row_file, "{}", serde_json::to_string(&row)?)?;
                rows.push(row);
            }
            check_wall(start)?;
            let mut batch_tokens = tokens.clone();
            batch_tokens.resize(tokens.len() * 2, 0);
            batch_tokens[tokens.len()] = 1;
            let batched = candidate.forward(&batch_tokens, 2, tokens.len())?;
            if batched.dims() != [2, tokens.len(), VOCAB] {
                return Err("shared batch-two logit shape mismatch".into());
            }
            let batched = batched.to_vec3::<f32>()?;
            for (batch_index, output_rows) in batched.iter().enumerate() {
                for (position, c) in output_rows.iter().enumerate() {
                    // Preserve even unscored padding outputs, while keeping
                    // the independent parity gate restricted to valid tokens.
                    write_logits(&mut batch_file, c)?;
                    let raw_row = batch_raw_row;
                    batch_raw_row += 1;
                    if batch_index == 1 && position > 0 {
                        continue;
                    }
                    let (r, reference_window, reference_position, target, mode) =
                        if batch_index == 0 {
                            (
                                &reference[w][position],
                                w,
                                position,
                                tokens.get(position + 1).copied(),
                                "fresh_batch2_right_padding_row0_every_position",
                            )
                        } else {
                            (
                                &reference[0][0],
                                0,
                                0,
                                None,
                                "fresh_batch2_right_padding_row1_position0",
                            )
                        };
                    let mut row = compare(r, c, target)?;
                    row["implementation"] = json!(SHARED_ID);
                    row["backend"] = json!(name);
                    row["window"] = json!(w);
                    row["batch_row"] = json!(batch_index);
                    row["position"] = json!(position);
                    row["reference_window"] = json!(reference_window);
                    row["reference_position"] = json!(reference_position);
                    row["mode"] = json!(mode);
                    set_raw_location(&mut row, &batch_filename, raw_row);
                    worst = worst.max(row["max_abs"].as_f64().ok_or("missing shared max")?);
                    cells += VOCAB as u64;
                    over += row["cells_over_1e_4"]
                        .as_u64()
                        .ok_or("missing shared count")?;
                    writeln!(row_file, "{}", serde_json::to_string(&row)?)?;
                    rows.push(row);
                }
            }
            eprintln!(
                "shared {name} window={w} max_abs={worst:.9} elapsed_s={:.3}",
                start.elapsed().as_secs_f64()
            );
        }
        let coverage_pass =
            complete_coverage(&rows, SHARED_MODES) && full_raw_row == 45 && batch_raw_row == 90;
        let pass = over == 0 && coverage_pass;
        overall_pass &= pass;
        shared_backends.push(json!({
            "implementation": SHARED_ID, "backend":name, "pass":pass,
            "maximum_absolute_error":worst, "cells_over_1e_4":over, "compared_cells":cells,
            "expected_rows":94, "completed_rows":rows.len(),
            "expected_mode_counts":coverage_json(SHARED_MODES),
            "completed_mode_counts":mode_counts(&rows), "coverage_pass":coverage_pass,
            "raw_full_rows":full_raw_row, "raw_batch_rows":batch_raw_row,
            "unscored_padding_raw_rows":batch_raw_row - 49,
            "load_seconds":load_seconds, "evaluation_seconds":eval_start.elapsed().as_secs_f64(),
            "rows":rows
        }));
        drop(candidate);
    }
    check_wall(start)?;
    let completed_rows: u64 = stock_backends
        .iter()
        .chain(&shared_backends)
        .map(|backend| {
            backend["completed_rows"]
                .as_u64()
                .ok_or("missing final row count")
        })
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .sum();
    overall_pass &=
        completed_rows == 294 && stock_backends.len() == 2 && shared_backends.len() == 2;
    Ok(json!({
        "schema": "uor-r4.track-b-parity-result/2",
        "status": if overall_pass { "PASS" } else { "FAIL_NUMERICAL_GATE" },
        "pass": overall_pass, "tolerance_max_absolute": TOLERANCE,
        "reference_backend": backend, "reference_execution": execution,
        "reference_seconds": reference_seconds, "reference_rows":45,
        "expected_compared_rows_per_backend":147, "expected_compared_rows_total":294,
        "completed_compared_rows_total":completed_rows,
        "implementations":[
            {"identity":STOCK_ID,"expected_rows_per_backend":53,"backends":stock_backends},
            {"identity":SHARED_ID,"expected_rows_per_backend":94,"backends":shared_backends}
        ],
        "total_seconds": start.elapsed().as_secs_f64(), "training_steps": 0,
        "b2_authorized_by_this_result": overall_pass,
        "language_quality": "NOT_EVALUATED", "serving_cost": "NOT_EVALUATED"
    }))
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: track-b-parity MODEL_DIR NEW_REPORT_DIR".into());
    }
    let model = PathBuf::from(&args[0]);
    let out = PathBuf::from(&args[1]);
    if !model.is_dir() {
        return Err("model directory missing".into());
    }
    report_output::claim(&out)?;
    let start = Instant::now();
    // Refuse before any model load. Once admitted, enforce the actual failure
    // seen in parity-3 automatically, even if a kernel has not returned. The
    // margin remains reserved throughout; the report allowance is preflight.
    let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
    // Keep the hard deadline independent of filesystem observation: a stalled
    // df command must not suspend the wall-time watchdog.
    std::thread::spawn(move || {
        if done_rx
            .recv_timeout(std::time::Duration::from_secs(WALL_SECONDS))
            .is_err()
        {
            eprintln!("UNSEALED_TIMEOUT: fixed 600-second smoke bound reached");
            std::process::exit(124);
        }
    });
    let stop_observation = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut observation_thread = None;
    let mut result = match admit_storage() {
        Err(error) => Err(error),
        Ok(initial_available) => {
            write_json(
                &out.join("resource-admission.json"),
                &json!({
                    "schema": "uor-r4.track-b-parity-admission/1",
                    "storage_path": SHARED_STORAGE_PATH,
                    "available_bytes": initial_available,
                    "reserve_bytes": STORAGE_RESERVE_BYTES,
                    "stop_margin_bytes": STORAGE_STOP_MARGIN_BYTES,
                    "report_allowance_bytes": REPORT_ALLOWANCE_BYTES,
                    "watchdog_interval_ms": 1000, "wall_seconds": WALL_SECONDS
                }),
            )?;
            let observed_stop = stop_observation.clone();
            observation_thread = Some(std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                if observed_stop.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                match available_storage_bytes() {
                    Ok(available)
                        if available < STORAGE_RESERVE_BYTES + STORAGE_STOP_MARGIN_BYTES =>
                    {
                        record_watchdog_stop(
                            start,
                            "UNSEALED_STORAGE_STOP",
                            json!({
                                "available_bytes":available,
                                "required_bytes":STORAGE_RESERVE_BYTES+STORAGE_STOP_MARGIN_BYTES
                            }),
                        );
                    }
                    Err(error) => record_watchdog_stop(
                        start,
                        "UNSEALED_STORAGE_OBSERVATION",
                        json!({"error":error.to_string()}),
                    ),
                    Ok(_) => {}
                }
            }));
            run(&model, &out, start)
        }
    };
    stop_observation.store(true, std::sync::atomic::Ordering::Relaxed);
    if let Some(observer) = observation_thread {
        if observer.join().is_err() {
            result = Err("storage observer panicked; execution qualification unavailable".into());
        }
        if result.is_ok() {
            result = match available_storage_bytes() {
                Ok(available) if available >= STORAGE_RESERVE_BYTES + STORAGE_STOP_MARGIN_BYTES => {
                    result
                }
                Ok(available) => {
                    Err(format!("storage below reserve at completion: {available} bytes").into())
                }
                Err(error) => Err(error),
            };
        }
    }
    let (summary, code) = match result {
        Ok(summary) => {
            let pass = summary["pass"] == true;
            (summary, if pass { 0 } else { 2 })
        }
        Err(error) => (
            json!({"schema":"uor-r4.track-b-parity-result/2", "status":"UNAVAILABLE",
            "pass":false, "error":error.to_string(), "total_seconds":start.elapsed().as_secs_f64()}),
            1,
        ),
    };
    write_json(&out.join("result.json"), &summary)?;
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    eprintln!(
        "{}; sealed and verified {}",
        summary["status"],
        out.display()
    );
    let _ = done_tx.send(());
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn storage_parser_uses_available_blocks_and_fails_closed() -> Result<()> {
        let sample = "Filesystem 1024-blocks Used Available Capacity Mounted on\n/dev/disk6s1 209715160 183277876 26306004 88% /Volumes/UOR-Workspace\n";
        assert_eq!(parse_available_bytes(sample)?, 26_306_004 * 1024);
        for invalid in [
            "",
            "bad header\n/dev/disk6s1 1 2 3\n",
            "Filesystem\n",
            "Filesystem\n/dev/disk6s1 1 2 -1\n",
            "Filesystem\n/dev/disk6s1 1 2 18446744073709551615\n",
            "Filesystem\n/dev/a 1 2 3\n/dev/b 1 2 3\n",
        ] {
            assert!(parse_available_bytes(invalid).is_err());
        }
        Ok(())
    }
    #[test]
    fn gate_checks_every_cell_and_rejects_nonfinite() {
        let a = vec![0_f32; VOCAB];
        let mut b = a.clone();
        assert_eq!(compare(&a, &b, None).unwrap()["pass"], true);
        b[VOCAB - 1] = 0.0002;
        let r = compare(&a, &b, Some(1)).unwrap();
        assert_eq!(r["pass"], false);
        assert_eq!(r["cells_over_1e_4"], 1);
        assert_eq!(r["max_abs_vocab_id"], VOCAB - 1);
        b[0] = f32::NAN;
        assert!(compare(&a, &b, None).is_err());
        assert!(compare(&a[..3], &a, None).is_err());
    }

    #[test]
    fn complete_coverage_requires_every_registered_mode_and_count() {
        for modes in [STOCK_MODES, SHARED_MODES] {
            let mut rows = Vec::new();
            for (mode, count) in modes {
                rows.extend((0..*count).map(|_| json!({"mode":mode})));
            }
            assert!(complete_coverage(&rows, modes));
            rows.pop();
            assert!(!complete_coverage(&rows, modes));
            // The expected total by itself is insufficient: missing a batch
            // isolation row cannot be replaced with another full-prefix row.
            rows.push(json!({"mode":"singleton_every_position"}));
            assert!(!complete_coverage(&rows, modes));
        }
    }
}
