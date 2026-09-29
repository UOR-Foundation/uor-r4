//! Fixed all-logit parity gate for the offline Track B conversion teacher.
//! cargo run --release -p uor-r4-training --features metal --example track-b-parity -- MODEL NEW_REPORT
//! Every window resets both caches. These synthetic tokens test numerical fidelity,
//! not language quality, and the reported elapsed rates are not serving benchmarks.
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
use uor_r4_training::{sha256_file, track_b::conversion::CandleLlamaTeacher};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const TOLERANCE: f64 = 1e-4;
const VOCAB: usize = 49152;
const WALL_SECONDS: u64 = 600;

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
            "schema": "uor-r4.track-b-parity-inputs/1", "windows": inputs,
            "model": model, "sha256": hashes, "executable_sha256": sha256_file(&exe)?,
        "source_revision": source_revision,
        "source_diff_sha256": source_diff,
            "candle_version": "0.9.2", "dtype": "F32", "stored_weights": "BF16",
            "canonical_math_env": "0", "exact_scalar_env": null, "tolerance": TOLERANCE,
            "wall_seconds": WALL_SECONDS,
            "scope": "fixed synthetic-token numerical fidelity; no language-quality or timing claim",
            "modes": ["singleton_every_position", "fresh_full_prefill_final", "fresh_half_prefix_then_singletons_final"]
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
    let mut backends = Vec::new();
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
        let pass = over == 0 && rows.len() == 53;
        overall_pass &= pass;
        backends.push(json!({
            "backend": name, "pass": pass, "maximum_absolute_error": worst,
            "cells_over_1e_4": over, "compared_cells": cells,
            "expected_rows": 53, "completed_rows": rows.len(),
            "load_seconds": load_seconds, "evaluation_seconds": eval_start.elapsed().as_secs_f64(),
            "rows": rows
        }));
    }
    check_wall(start)?;
    Ok(json!({
        "schema": "uor-r4.track-b-parity-result/1",
        "status": if overall_pass { "PASS" } else { "FAIL_NUMERICAL_GATE" },
        "pass": overall_pass, "tolerance_max_absolute": TOLERANCE,
        "reference_backend": backend, "reference_execution": execution,
        "reference_seconds": reference_seconds, "backends": backends,
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
    // A hung load/kernel may not return to the cooperative deadline. On timeout
    // preserve the partial root unsealed; it is never a successful result.
    let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || {
        if done_rx
            .recv_timeout(std::time::Duration::from_secs(WALL_SECONDS))
            .is_err()
        {
            eprintln!("UNSEALED_TIMEOUT: fixed 600-second smoke bound reached");
            std::process::exit(124);
        }
    });
    let result = run(&model, &out, start);
    let (summary, code) = match result {
        Ok(summary) => {
            let pass = summary["pass"] == true;
            (summary, if pass { 0 } else { 2 })
        }
        Err(error) => (
            json!({"schema":"uor-r4.track-b-parity-result/1", "status":"UNAVAILABLE",
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
}
