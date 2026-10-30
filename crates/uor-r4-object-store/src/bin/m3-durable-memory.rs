#[cfg(feature = "cuda")]
use std::fs::OpenOptions;
#[cfg(feature = "cuda")]
use std::io::Write;
#[cfg(feature = "cuda")]
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use uor_r4_object_store::m3_durable_memory::{verify_frozen_panel, PANEL_SHA256};
#[cfg(feature = "cuda")]
use uor_r4_object_store::m3_durable_memory::{
    evaluate_m3_panel_with_scorer, sha256_hex, ModelStepScore, FT_DEV_MASK_SHA256, FT_DEV_ROWS,
    FT_DEV_TOKENS_SHA256, M3_IDENTITY_SCHEMA, M3_REPORT_SCHEMA, PANEL_MANIFEST_VERSION,
    SEED1_ARTIFACT_SHA256, SEED1_EXPECTED_DEV_ADMITTED, SEED1_EXPECTED_DEV_TOP1,
    SEED1_EXPECTED_DEV_TOP8, SEED2_ARTIFACT_SHA256, SEED2_EXPECTED_DEV_ADMITTED,
    SEED2_EXPECTED_DEV_TOP1, SEED2_EXPECTED_DEV_TOP8, TOKENIZER_SHA256,
};
#[cfg(feature = "cuda")]
use uor_r4_tokenizer::ByteBpeTokenizer;

fn main() -> ExitCode {
    match run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("m3-durable-memory refused: {error}");
            ExitCode::from(1)
        }
    }
}

fn run_cli() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [cmd, panel_path] if cmd == "verify-panel" => {
            let panel_bytes = read_file(panel_path, "panel")?;
            let (manifest, items) = verify_frozen_panel(&panel_bytes)?;
            println!(
                "m3-durable-memory verify-panel: {} items, whole_sha256={}, manifest_version={}",
                items.len(),
                PANEL_SHA256,
                manifest
                    .version()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            );
            Ok(())
        }
        [cmd, s1_artifact, s2_artifact, dev_config, tokenizer_path, panel_path, out_dir]
            if cmd == "run" =>
        {
            #[cfg(feature = "cuda")]
            {
                return run_gpu(
                    s1_artifact,
                    s2_artifact,
                    dev_config,
                    tokenizer_path,
                    panel_path,
                    out_dir,
                );
            }
            #[cfg(not(feature = "cuda"))]
            {
                let _ = (
                    s1_artifact,
                    s2_artifact,
                    dev_config,
                    tokenizer_path,
                    panel_path,
                    out_dir,
                );
                return Err("m3-durable-memory run requires --features cuda".into());
            }
        }
        _ => Err(concat!(
            "usage: m3-durable-memory verify-panel PANEL_JSONL | ",
            "m3-durable-memory run SEED1_ARTIFACT SEED2_ARTIFACT DEV_CONFIG_JSON ",
            "TOKENIZER_JSON PANEL_JSONL OUT_DIR"
        )
        .into()),
    }
}

fn read_file(path: &str, label: &str) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| format!("{label} {path}: {e}"))
}

#[cfg(feature = "cuda")]
fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("{}: {e} (outputs are never overwritten)", path.display()))?;
    file.write_all(bytes)
        .map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(feature = "cuda")]
fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let mut encoded = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    encoded.push('\n');
    write_new(path, encoded.as_bytes())
}

#[cfg(feature = "cuda")]
fn cuda_driver_api_version() -> Result<i32, String> {
    let mut raw = 0;
    // SAFETY: cuDriverGetVersion writes one c_int to a valid stack pointer.
    let status = unsafe { cudarc::driver::sys::cuDriverGetVersion(&mut raw) };
    if status == cudarc::driver::sys::CUresult::CUDA_SUCCESS && raw > 0 {
        Ok(raw)
    } else {
        Err(format!(
            "cuDriverGetVersion returned {status:?} (raw {raw})"
        ))
    }
}

#[cfg(feature = "cuda")]
fn run_gpu(
    s1_artifact: &str,
    s2_artifact: &str,
    dev_config_path: &str,
    tokenizer_path: &str,
    panel_path: &str,
    out_dir: &str,
) -> Result<(), String> {
    use m1_d11_artifact_scorer_instrument::{
        artifact::ArtifactTables,
        dev_rows,
        device::{cubin_sha256, ptx_sha256, DeviceScorer, IMAGE_TARGET, RESOURCES},
        kernel::{pack_rows, pack_rows_with_targets},
        score_summary,
    };
    use serde_json::json;
    use uor_r4_discrete::device::compiled_identity;
    use uor_r4_discrete::training::{driver_source_sha256, DriverConfig};

    let head = std::env::var("UOR_STAGE0_HEAD")
        .map_err(|_| "UOR_STAGE0_HEAD must be set to the full commit SHA".to_string())?;
    if head.len() != 40 || !head.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("UOR_STAGE0_HEAD {head:?} is not a 40-hex commit SHA"));
    }

    let s1_bytes = read_file(s1_artifact, "seed1 artifact")?;
    let s1_sha = sha256_hex(&s1_bytes);
    if s1_sha != SEED1_ARTIFACT_SHA256 {
        return Err(format!(
            "seed1 artifact sha256 {s1_sha} differs from pinned {SEED1_ARTIFACT_SHA256}"
        ));
    }
    let s2_bytes = read_file(s2_artifact, "seed2 artifact")?;
    let s2_sha = sha256_hex(&s2_bytes);
    if s2_sha != SEED2_ARTIFACT_SHA256 {
        return Err(format!(
            "seed2 artifact sha256 {s2_sha} differs from pinned {SEED2_ARTIFACT_SHA256}"
        ));
    }

    let tok_bytes = read_file(tokenizer_path, "tokenizer")?;
    let tok_sha = sha256_hex(&tok_bytes);
    if tok_sha != TOKENIZER_SHA256 {
        return Err(format!(
            "tokenizer sha256 {tok_sha} differs from pinned {TOKENIZER_SHA256}"
        ));
    }
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tok_bytes)
        .ok_or_else(|| "invalid pinned tokenizer JSON".to_string())?;

    let panel_bytes = read_file(panel_path, "panel")?;
    verify_frozen_panel(&panel_bytes)?;

    let cfg_bytes = read_file(dev_config_path, "dev config")?;
    let cfg_sha = sha256_hex(&cfg_bytes);
    let driver_config: DriverConfig =
        serde_json::from_slice(&cfg_bytes).map_err(|e| format!("driver config: {e}"))?;
    let dev_tok_sha = sha256_hex(
        &driver_config
            .dev
            .tokens
            .read()
            .map_err(|e| format!("read ft/dev tokens: {e}"))?,
    );
    let dev_mask_sha = sha256_hex(
        &driver_config
            .dev
            .mask
            .read()
            .map_err(|e| format!("read ft/dev mask: {e}"))?,
    );
    if dev_tok_sha != FT_DEV_TOKENS_SHA256 || dev_mask_sha != FT_DEV_MASK_SHA256 {
        return Err(format!(
            "ft/dev corpus digests ({dev_tok_sha}, {dev_mask_sha}) differ from pinned ({FT_DEV_TOKENS_SHA256}, {FT_DEV_MASK_SHA256})"
        ));
    }

    let (s1_tables, _) = ArtifactTables::from_bytes(&s1_bytes)?;
    let (s2_tables, _) = ArtifactTables::from_bytes(&s2_bytes)?;
    let s1_scorer =
        DeviceScorer::new(0, &s1_tables).map_err(|e| format!("seed1 DeviceScorer: {e}"))?;
    let s2_scorer =
        DeviceScorer::new(0, &s2_tables).map_err(|e| format!("seed2 DeviceScorer: {e}"))?;

    // Gate R: verify ft/dev 14,287-row parity on GPU for both seeds before running M3
    let dev_rows_vec = dev_rows(&driver_config)?;
    if dev_rows_vec.len() != FT_DEV_ROWS {
        return Err(format!(
            "ft/dev row count {} differs from expected {FT_DEV_ROWS}",
            dev_rows_vec.len()
        ));
    }
    let dev_views: Vec<&[u32]> = dev_rows_vec.iter().map(|r| r.prefix.as_slice()).collect();
    let dev_targets: Vec<u32> = dev_rows_vec.iter().map(|r| r.target_token).collect();
    let dev_packed = pack_rows_with_targets(&dev_views, &dev_targets)?;

    let s1_dev_scores = s1_scorer.score(&dev_packed)?;
    let (s1_dev_summary, _) = score_summary(&s1_tables, &dev_rows_vec, &s1_dev_scores)?;
    let s1_top1 = s1_dev_summary["top1"].as_u64().unwrap_or(0);
    let s1_top8 = s1_dev_summary["top8"].as_u64().unwrap_or(0);
    let s1_adm = s1_dev_summary["admitted"].as_u64().unwrap_or(0);
    let s1_refused = s1_dev_summary["refused"].as_u64().unwrap_or(1);
    if s1_top1 != SEED1_EXPECTED_DEV_TOP1
        || s1_top8 != SEED1_EXPECTED_DEV_TOP8
        || s1_adm != SEED1_EXPECTED_DEV_ADMITTED
        || s1_refused != 0
    {
        return Err(format!(
            "Seed 1 Gate R ft/dev parity mismatch: got top1={s1_top1}, top8={s1_top8}, admitted={s1_adm}, refused={s1_refused}"
        ));
    }

    let s2_dev_scores = s2_scorer.score(&dev_packed)?;
    let (s2_dev_summary, _) = score_summary(&s2_tables, &dev_rows_vec, &s2_dev_scores)?;
    let s2_top1 = s2_dev_summary["top1"].as_u64().unwrap_or(0);
    let s2_top8 = s2_dev_summary["top8"].as_u64().unwrap_or(0);
    let s2_adm = s2_dev_summary["admitted"].as_u64().unwrap_or(0);
    let s2_refused = s2_dev_summary["refused"].as_u64().unwrap_or(1);
    if s2_top1 != SEED2_EXPECTED_DEV_TOP1
        || s2_top8 != SEED2_EXPECTED_DEV_TOP8
        || s2_adm != SEED2_EXPECTED_DEV_ADMITTED
        || s2_refused != 0
    {
        return Err(format!(
            "Seed 2 Gate R ft/dev parity mismatch: got top1={s2_top1}, top8={s2_top8}, admitted={s2_adm}, refused={s2_refused}"
        ));
    }

    let context_limit = s1_tables.shape[2] as usize;
    let s1_eval = evaluate_m3_panel_with_scorer(
        &panel_bytes,
        &tokenizer,
        SEED1_ARTIFACT_SHA256,
        context_limit,
        |prefix: &[u32]| {
            let packed = pack_rows(&[prefix])?;
            let scores = s1_scorer.score(&packed)?;
            if scores.len() != 1 {
                return Err("seed1 DeviceScorer returned unexpected row count".into());
            }
            Ok(ModelStepScore {
                status: scores[0].status,
                prediction: scores[0].prediction,
                top8: scores[0].top8.clone(),
            })
        },
    )?;

    let s2_eval = evaluate_m3_panel_with_scorer(
        &panel_bytes,
        &tokenizer,
        SEED2_ARTIFACT_SHA256,
        context_limit,
        |prefix: &[u32]| {
            let packed = pack_rows(&[prefix])?;
            let scores = s2_scorer.score(&packed)?;
            if scores.len() != 1 {
                return Err("seed2 DeviceScorer returned unexpected row count".into());
            }
            Ok(ModelStepScore {
                status: scores[0].status,
                prediction: scores[0].prediction,
                top8: scores[0].top8.clone(),
            })
        },
    )?;

    let out = PathBuf::from(out_dir);
    std::fs::create_dir_all(&out).map_err(|e| format!("create_dir_all {}: {e}", out.display()))?;

    let executable = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let executable_sha256 =
        sha256_hex(&std::fs::read(&executable).map_err(|e| format!("read executable: {e}"))?);
    let display_driver = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=driver_version", "--format=csv,noheader"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_else(|| "UNVERIFIED".into());
    let gpu_name = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=name", "--format=csv,noheader"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_else(|| "UNVERIFIED".into());

    let identity = json!({
        "schema": M3_IDENTITY_SCHEMA,
        "HEAD": head,
        "executable_sha256": executable_sha256,
        "cubin_sha256": cubin_sha256(),
        "ptx_sha256": ptx_sha256(),
        "driver_source_sha256": driver_source_sha256(),
        "compiled_identity": compiled_identity(),
        "image_target": IMAGE_TARGET.trim(),
        "resources": RESOURCES.trim(),
        "device_capability": [s1_scorer.capability.0, s1_scorer.capability.1],
        "cuda_driver_api": cuda_driver_api_version()?,
        "nvidia_display_driver": display_driver,
        "gpu_name": gpu_name,
        "seed1_artifact_sha256": SEED1_ARTIFACT_SHA256,
        "seed2_artifact_sha256": SEED2_ARTIFACT_SHA256,
        "tokenizer_sha256": TOKENIZER_SHA256,
        "panel_sha256": PANEL_SHA256,
        "panel_manifest_version": PANEL_MANIFEST_VERSION,
        "dev_config_sha256": cfg_sha,
        "ft_dev_tokens_sha256": FT_DEV_TOKENS_SHA256,
        "ft_dev_mask_sha256": FT_DEV_MASK_SHA256,
    });
    write_json(&out.join("m3-identity.json"), &identity)?;

    let s1_val = serde_json::to_value(&s1_eval).map_err(|e| e.to_string())?;
    let s2_val = serde_json::to_value(&s2_eval).map_err(|e| e.to_string())?;
    write_json(&out.join("s1-m3-evaluation.json"), &s1_val)?;
    write_json(&out.join("s2-m3-evaluation.json"), &s2_val)?;

    let both_pass = s1_eval.bars_pass && s2_eval.bars_pass;
    let summary = json!({
        "schema": M3_REPORT_SCHEMA,
        "HEAD": head,
        "panel_sha256": PANEL_SHA256,
        "panel_manifest_version": PANEL_MANIFEST_VERSION,
        "gate_r_ft_dev": {
            "rows": FT_DEV_ROWS,
            "seed1": {
                "artifact_sha256": SEED1_ARTIFACT_SHA256,
                "top1": s1_top1,
                "top8": s1_top8,
                "admitted": s1_adm,
                "refused": s1_refused,
                "parity_pass": true,
            },
            "seed2": {
                "artifact_sha256": SEED2_ARTIFACT_SHA256,
                "top1": s2_top1,
                "top8": s2_top8,
                "admitted": s2_adm,
                "refused": s2_refused,
                "parity_pass": true,
            }
        },
        "seed1": {
            "artifact_sha256": s1_eval.artifact_sha256,
            "decode_tokens_per_second_milli": s1_eval.decode_tokens_per_second_milli,
            "total_model_scored_steps": s1_eval.total_model_scored_steps,
            "total_elapsed_ns": s1_eval.total_elapsed_ns,
            "provenance_hits": s1_eval.provenance_hits,
            "provenance_no_memory_hits": s1_eval.provenance_no_memory_hits,
            "provenance_unassisted_model_hits": s1_eval.provenance_unassisted_model_hits,
            "correction_current_hits": s1_eval.correction_current_hits,
            "correction_prior_hits": s1_eval.correction_prior_hits,
            "correction_no_memory_hits": s1_eval.correction_no_memory_hits,
            "correction_unassisted_model_hits": s1_eval.correction_unassisted_model_hits,
            "abstention_refusals": s1_eval.abstention_refusals,
            "abstention_total_queries": s1_eval.abstention_total_queries,
            "abstention_zero_fetch_queries": s1_eval.abstention_zero_fetch_queries,
            "non_memory_prose_hits": s1_eval.non_memory_prose_hits,
            "non_memory_prose_no_memory_hits": s1_eval.non_memory_prose_no_memory_hits,
            "reset_isolation_hits": s1_eval.reset_isolation_hits,
            "restart_restore_hits": s1_eval.restart_restore_hits,
            "restart_restore_matched_tokens": s1_eval.restart_restore_matched_tokens,
            "restart_restore_total_tokens": s1_eval.restart_restore_total_tokens,
            "restart_restore_no_memory_hits": s1_eval.restart_restore_no_memory_hits,
            "r14": s1_eval.r14,
            "controls": s1_eval.controls,
            "bars_pass": s1_eval.bars_pass,
        },
        "seed2": {
            "artifact_sha256": s2_eval.artifact_sha256,
            "decode_tokens_per_second_milli": s2_eval.decode_tokens_per_second_milli,
            "total_model_scored_steps": s2_eval.total_model_scored_steps,
            "total_elapsed_ns": s2_eval.total_elapsed_ns,
            "provenance_hits": s2_eval.provenance_hits,
            "provenance_no_memory_hits": s2_eval.provenance_no_memory_hits,
            "provenance_unassisted_model_hits": s2_eval.provenance_unassisted_model_hits,
            "correction_current_hits": s2_eval.correction_current_hits,
            "correction_prior_hits": s2_eval.correction_prior_hits,
            "correction_no_memory_hits": s2_eval.correction_no_memory_hits,
            "correction_unassisted_model_hits": s2_eval.correction_unassisted_model_hits,
            "abstention_refusals": s2_eval.abstention_refusals,
            "abstention_total_queries": s2_eval.abstention_total_queries,
            "abstention_zero_fetch_queries": s2_eval.abstention_zero_fetch_queries,
            "non_memory_prose_hits": s2_eval.non_memory_prose_hits,
            "non_memory_prose_no_memory_hits": s2_eval.non_memory_prose_no_memory_hits,
            "reset_isolation_hits": s2_eval.reset_isolation_hits,
            "restart_restore_hits": s2_eval.restart_restore_hits,
            "restart_restore_matched_tokens": s2_eval.restart_restore_matched_tokens,
            "restart_restore_total_tokens": s2_eval.restart_restore_total_tokens,
            "restart_restore_no_memory_hits": s2_eval.restart_restore_no_memory_hits,
            "r14": s2_eval.r14,
            "controls": s2_eval.controls,
            "bars_pass": s2_eval.bars_pass,
        },
        "all_bars_pass": both_pass,
        "decision": if both_pass {
            "PASS_M3_DURABLE_MEMORY_TWO_SEEDS"
        } else {
            "FAIL_M3_DURABLE_MEMORY"
        },
    });
    write_json(&out.join("m3-summary.json"), &summary)?;

    if !both_pass {
        return Err("one or more M3 durable-memory bars failed".into());
    }
    println!(
        "m3-durable-memory: PASS_M3_DURABLE_MEMORY_TWO_SEEDS; outputs in {}",
        out.display()
    );
    Ok(())
}
