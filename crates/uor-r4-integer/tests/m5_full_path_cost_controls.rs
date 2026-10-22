//! Finite host controls for `m5-laptop-full-path-measurement` (Issue #2862 / Milestone M5 #2033).
//!
//! Verifies:
//! 1. Dynamic parameter and four-phase breakdown (`ingestion`, `routing`, `memory_lookup`, `emission`)
//!    on `mk-synthetic-stack` (`seed = 5`), replacing the hardcoded `7,238,304` string in `uor-r4-stack cost`.
//! 2. Bit-identical logits and session state between `IntegerStackSession::step` and
//!    `IntegerStackSession::measure_step_phases` (which stays outside the audited `step` call graph).
//! 3. End-to-end `uor-r4-stack cost` JSON receipt fields (`parameter_breakdown`, `phase_latency_us`,
//!    `phase_instrumentation_logits_match`, `peak_rss_mb`, `external_inference_daemons_detected`).
//! 4. Protocol v3 energy-receipt validation: deterministic rejection of `archive/ops/measurement_report_rnd.json`
//!    for Defect 1 (`work_rate_w = 40.5 W` vs `0.003172 * 17,190 = 54.53 W`) and Defect 2
//!    (`ci95 = 0.037 mJ` computed with normal `z = 1.96` instead of Student's `t_{0.975, 2} = 4.302653`
//!    giving `0.0798 mJ`), and acceptance of a valid Protocol v3 receipt.
#![forbid(unsafe_code)]

use serde_json::Value;
use std::path::PathBuf;

/// Two-sided 95% Student-t critical value t_{0.975, df} for degrees of freedom df = n - 1.
pub fn student_t_975(df: usize) -> f64 {
    match df {
        0 => f64::INFINITY,
        1 => 12.706_204_736_4,
        2 => 4.302_652_729_91,
        3 => 3.182_446_305_28,
        4 => 2.776_445_105_2,
        5 => 2.570_581_836_61,
        6 => 2.446_911_851_14,
        7 => 2.364_624_251_6,
        8 => 2.306_004_135_2,
        9 => 2.262_157_162_85,
        10 => 2.228_138_851_96,
        11..=15 => 2.131_449_545_56,
        16..=20 => 2.085_963_447_27,
        21..=30 => 2.042_272_456_3,
        _ => 1.959_963_984_54,
    }
}

/// Sample mean, sample standard deviation (n - 1 denominator), and Student-t 95% CI half-width.
pub fn sample_mean_sd_t_ci95(samples: &[f64]) -> Option<(f64, f64, f64)> {
    let n = samples.len();
    if n < 2 {
        return None;
    }
    let nf = n as f64;
    let mean = samples.iter().sum::<f64>() / nf;
    let var = samples.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / ((n - 1) as f64);
    let sd = var.sqrt();
    let sem = sd / nf.sqrt();
    let ci95 = student_t_975(n - 1) * sem;
    Some((mean, sd, ci95))
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolV3ValidationError {
    PowerThroughputInconsistency {
        reported_work_rate_w: f64,
        implied_work_rate_w: f64,
        relative_error: f64,
    },
    InvalidStudentTCi95 {
        metric: String,
        reported_ci95: f64,
        expected_t_ci95: f64,
        relative_error: f64,
    },
    MissingRequiredV3Field(&'static str),
}

/// Validate an energy + full-path cost receipt against Protocol v3 requirements:
/// 1. `work_rate_w` must agree with `gross_j_per_tok * tokens_per_s` within 2%.
/// 2. `gross_mj_per_tok.ci95` and `net_mj_per_tok.ci95` must use Student's `t_{0.975, n-1}` within 2%.
/// 3. Protocol v3 full-path fields (`parameter_breakdown`, `phase_latency_us`, `logits_sha256`,
///    `external_inference_daemons_detected`, `kernel_peak_footprint_bytes`) must be present.
pub fn validate_protocol_v3_energy_receipt(receipt: &Value) -> Vec<ProtocolV3ValidationError> {
    let mut errors = Vec::new();

    let work_rate_w = receipt
        .pointer("/power/work_rate_w")
        .or_else(|| receipt.pointer("/serving/energy/work_rate_w"))
        .and_then(Value::as_f64);
    let gross_j = receipt
        .pointer("/energy_per_token/gross_j_per_tok")
        .or_else(|| receipt.pointer("/serving/energy/gross_j_per_tok/mean"))
        .and_then(Value::as_f64);
    let tok_s = receipt
        .pointer("/throughput/tokens_per_s")
        .or_else(|| receipt.pointer("/serving/tokens_per_s/mean"))
        .and_then(Value::as_f64);

    if let (Some(work_rate_w), Some(gross_j), Some(tok_s)) = (work_rate_w, gross_j, tok_s) {
        let implied = gross_j * tok_s;
        let rel_err = (work_rate_w - implied).abs() / implied.max(1e-12);
        if rel_err > 0.02 {
            errors.push(ProtocolV3ValidationError::PowerThroughputInconsistency {
                reported_work_rate_w: work_rate_w,
                implied_work_rate_w: implied,
                relative_error: rel_err,
            });
        }
    } else {
        errors.push(ProtocolV3ValidationError::MissingRequiredV3Field(
            "power.work_rate_w / energy_per_token.gross_j_per_tok / throughput.tokens_per_s",
        ));
    }

    for (name, ptr) in [
        ("gross_mj_per_tok", "/energy_per_token/gross_mj_per_tok"),
        ("net_mj_per_tok", "/energy_per_token/net_mj_per_tok"),
    ] {
        if let Some(block) = receipt.pointer(ptr) {
            let samples: Option<Vec<f64>> = block
                .get("samples")
                .and_then(Value::as_array)
                .map(|arr| arr.iter().filter_map(Value::as_f64).collect());
            let reported_ci95 = block.get("ci95").and_then(Value::as_f64);
            if let (Some(samples), Some(reported)) = (samples, reported_ci95) {
                if let Some((_mean, _sd, expected_ci95)) = sample_mean_sd_t_ci95(&samples) {
                    let rel_err = (reported - expected_ci95).abs() / expected_ci95.max(1e-12);
                    if rel_err > 0.02 {
                        errors.push(ProtocolV3ValidationError::InvalidStudentTCi95 {
                            metric: name.to_string(),
                            reported_ci95: reported,
                            expected_t_ci95: expected_ci95,
                            relative_error: rel_err,
                        });
                    }
                }
            }
        }
    }

    // Also check legacy /serving/energy/net_j_per_tok/ci95 when min/mean/max of gross_j_per_tok are present
    // (as in `archive/ops/measurement_report_rnd.json`, where n = 3 runs have gross [0.003135, 0.003172, 0.003199] J/tok
    // and net_j_per_tok.ci95 = 3.7e-5 J/tok = 0.037 mJ/tok was computed with z = 1.96 instead of t_{0.975, 2} = 4.302653).
    if let (Some(min_j), Some(mean_j), Some(max_j), Some(net_ci95_j)) = (
        receipt
            .pointer("/serving/energy/gross_j_per_tok/min")
            .and_then(Value::as_f64),
        receipt
            .pointer("/serving/energy/gross_j_per_tok/mean")
            .and_then(Value::as_f64),
        receipt
            .pointer("/serving/energy/gross_j_per_tok/max")
            .and_then(Value::as_f64),
        receipt
            .pointer("/serving/energy/net_j_per_tok/ci95")
            .and_then(Value::as_f64),
    ) {
        let samples_j = [min_j, mean_j, max_j];
        if let Some((_m, _sd, expected_t_ci95_j)) = sample_mean_sd_t_ci95(&samples_j) {
            let rel_err = (net_ci95_j - expected_t_ci95_j).abs() / expected_t_ci95_j.max(1e-12);
            if rel_err > 0.02 {
                errors.push(ProtocolV3ValidationError::InvalidStudentTCi95 {
                    metric: "serving.energy.net_j_per_tok".to_string(),
                    reported_ci95: net_ci95_j * 1000.0,
                    expected_t_ci95: expected_t_ci95_j * 1000.0,
                    relative_error: rel_err,
                });
            }
        }
    }

    for required in [
        "/memory/parameter_breakdown",
        "/step_latency/phase_latency_us",
        "/tokens/logits_sha256",
        "/machine/external_inference_daemons_detected",
        "/memory/kernel_peak_footprint_bytes",
    ] {
        if receipt.pointer(required).is_none() {
            errors.push(ProtocolV3ValidationError::MissingRequiredV3Field(required));
        }
    }

    errors
}

#[test]
fn test_protocol_v3_validator_rejects_protocol_v2_rnd_receipt_defects() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let rnd_path = repo_root.join("archive/ops/measurement_report_rnd.json");
    let rnd_bytes = std::fs::read(&rnd_path).expect("read archive/ops/measurement_report_rnd.json");
    let rnd_json: Value = serde_json::from_slice(&rnd_bytes).expect("parse rnd json");

    let errors = validate_protocol_v3_energy_receipt(&rnd_json);

    // 1. Verify Defect 1 (power/throughput inconsistency: 40.5 W vs 54.52668 W, ~25.7% discrepancy) is caught.
    let power_err = errors.iter().find_map(|e| match e {
        ProtocolV3ValidationError::PowerThroughputInconsistency {
            reported_work_rate_w,
            implied_work_rate_w,
            relative_error,
        } => Some((*reported_work_rate_w, *implied_work_rate_w, *relative_error)),
        _ => None,
    });
    let (reported_w, implied_w, rel_w) = power_err
        .expect("validator must catch work_rate_w vs gross_j_per_tok * tokens_per_s mismatch");
    assert!((reported_w - 40.5).abs() < 1e-6);
    assert!((implied_w - 54.526_68).abs() < 1e-4);
    assert!(rel_w > 0.25, "relative error should be ~25.7%, got {rel_w}");

    // 2. Verify Defect 2 (normal z = 1.96 CI95 = 0.037 mJ instead of Student-t_{0.975, 2} = 4.302653 CI95 = 0.079815 mJ) is caught.
    let gross_ci_err = errors.iter().find_map(|e| match e {
        ProtocolV3ValidationError::InvalidStudentTCi95 {
            metric,
            reported_ci95,
            expected_t_ci95,
            relative_error,
        } if metric == "serving.energy.net_j_per_tok" => {
            Some((*reported_ci95, *expected_t_ci95, *relative_error))
        }
        _ => None,
    });
    let (reported_ci, expected_ci, rel_ci) =
        gross_ci_err.expect("validator must catch z-based ci95 on n=3 samples");
    assert!((reported_ci - 0.037).abs() < 1e-6);
    assert!((expected_ci - 0.079_815).abs() < 1e-4);
    assert!(
        rel_ci > 0.50,
        "z vs t_2 error should be ~53.6%, got {rel_ci}"
    );

    // 3. Verify that a compliant Protocol v3 receipt passes with zero errors.
    let gross_samples = [3.135_f64, 3.172_f64, 3.199_f64];
    let net_samples = [3.128_f64, 3.165_f64, 3.192_f64];
    let (gross_mean, gross_sd, gross_t_ci95) = sample_mean_sd_t_ci95(&gross_samples).unwrap();
    let (net_mean, net_sd, net_t_ci95) = sample_mean_sd_t_ci95(&net_samples).unwrap();
    let tok_s = 17_190.0_f64;
    let gross_j = gross_mean / 1000.0;
    let consistent_work_rate_w = gross_j * tok_s;

    let valid_v3 = serde_json::json!({
        "power": {
            "work_rate_w": consistent_work_rate_w,
        },
        "throughput": {
            "tokens_per_s": tok_s,
        },
        "energy_per_token": {
            "gross_j_per_tok": gross_j,
            "gross_mj_per_tok": {
                "samples": gross_samples,
                "mean": gross_mean,
                "sd": gross_sd,
                "ci95": gross_t_ci95,
            },
            "net_mj_per_tok": {
                "samples": net_samples,
                "mean": net_mean,
                "sd": net_sd,
                "ci95": net_t_ci95,
            },
        },
        "memory": {
            "parameter_breakdown": {},
            "kernel_peak_footprint_bytes": 4_194_304,
        },
        "step_latency": {
            "phase_latency_us": {},
        },
        "tokens": {
            "logits_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
        },
        "machine": {
            "external_inference_daemons_detected": [],
        },
    });
    let v3_errors = validate_protocol_v3_energy_receipt(&valid_v3);
    assert!(
        v3_errors.is_empty(),
        "valid Protocol v3 receipt must pass with 0 errors, got {v3_errors:?}"
    );

    // 4. Verify the sealed provisional Apple M1 rehearsal receipt in `docs/labs/m5-laptop-full-path-measurement/provisional-synthetic-stack-receipt.json`.
    let prov_path = repo_root
        .join("docs/labs/m5-laptop-full-path-measurement/provisional-synthetic-stack-receipt.json");
    let prov_bytes =
        std::fs::read(&prov_path).expect("read provisional-synthetic-stack-receipt.json");
    let prov_json: Value = serde_json::from_slice(&prov_bytes).expect("parse provisional json");
    let prov_errors = validate_protocol_v3_energy_receipt(&prov_json);
    assert!(
        prov_errors.is_empty(),
        "provisional Protocol v3 receipt must pass validation with 0 errors, got {prov_errors:?}"
    );
}

#[cfg(feature = "serving-wasm")]
mod serving_controls {
    use super::*;
    use std::process::Command;
    use uor_r4_integer::stack::IntegerStackModel;

    fn make_synthetic_stack_artifact(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "uor-r4-m5-cost-controls-{}-{tag}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let artifact_path = dir.join("synthetic-stack.lut");
        let status = Command::new(env!("CARGO_BIN_EXE_mk-synthetic-stack"))
            .arg("5")
            .arg(&artifact_path)
            .status()
            .expect("run mk-synthetic-stack");
        assert!(status.success(), "mk-synthetic-stack must succeed");
        artifact_path
    }

    #[test]
    fn test_parameter_breakdown_and_phase_timing_on_synthetic_stack() {
        let artifact_path = make_synthetic_stack_artifact("breakdown");
        let model = IntegerStackModel::load(&artifact_path).expect("load synthetic stack");
        let pb = model.parameter_breakdown();

        // Phase 1 (ingestion): 1 row of 32 4-bit weights (17 B) active out of 40 * 32 = 1,280 total (680 B).
        assert_eq!(pb.ingestion.active_4bit_weights_per_token, 32);
        assert_eq!(pb.ingestion.total_4bit_weights, 1_280);
        assert_eq!(
            pb.ingestion.active_4bit_weight_and_scale_bytes_per_token,
            17
        );
        assert_eq!(pb.ingestion.total_4bit_weight_and_scale_bytes, 680);
        assert_eq!(pb.ingestion.learned_scalars_total, 0);

        // Phase 2 (routing, l0 `r`): rec_in (2048) + rec_gate (1280) + rec_out (1024) + gate/up/down (3072) = 7,424.
        assert_eq!(pb.routing.active_4bit_weights_per_token, 7_424);
        assert_eq!(pb.routing.total_4bit_weights, 7_424);
        assert_eq!(
            pb.routing.active_4bit_weight_and_scale_bytes_per_token,
            3_944
        );
        assert_eq!(pb.routing.learned_scalars_total, 208);
        assert_eq!(pb.routing.learned_scalar_bytes_total, 560);

        // Phase 3 (memory_lookup, l1 `a`): query/key/value/out (4096) + null (64) + gate/up/down (3072) = 7,232.
        assert_eq!(pb.memory_lookup.active_4bit_weights_per_token, 7_232);
        assert_eq!(pb.memory_lookup.total_4bit_weights, 7_232);
        assert_eq!(
            pb.memory_lookup
                .active_4bit_weight_and_scale_bytes_per_token,
            3_842
        );
        assert_eq!(pb.memory_lookup.learned_scalars_total, 134);
        assert_eq!(pb.memory_lookup.learned_scalar_bytes_total, 532);

        // Phase 4 (emission, `head`): 40 * 32 = 1,280.
        assert_eq!(pb.emission.active_4bit_weights_per_token, 1_280);
        assert_eq!(pb.emission.total_4bit_weights, 1_280);
        assert_eq!(
            pb.emission.active_4bit_weight_and_scale_bytes_per_token,
            680
        );
        assert_eq!(pb.emission.learned_scalars_total, 0);

        // Totals across all four phases:
        assert_eq!(pb.weights_per_token, 15_968);
        assert_eq!(pb.weights_per_token, model.weights_per_token());
        assert_eq!(pb.total_4bit_weights, 17_216);
        assert_eq!(pb.active_4bit_weight_and_scale_bytes_per_token, 8_483);
        assert_eq!(pb.total_4bit_weight_and_scale_bytes, 9_146);
        assert_eq!(pb.total_learned_scalars, 342);
        assert_eq!(pb.total_learned_scalar_bytes, 1_092);
        assert_eq!(pb.total_learned_parameters, 17_558);
        assert_eq!(pb.active_learned_parameters_per_token, 16_310);

        // Verify `measure_step_phases` produces bit-identical logits and session state to `step`.
        let mut session_std = model.session();
        let mut session_phased = model.session();
        let mut sum_ing = 0u64;
        let mut sum_rout = 0u64;
        let mut sum_mem = 0u64;
        let mut sum_em = 0u64;

        for step_idx in 0..model.shape().context {
            let tok = ((step_idx * 7 + 3) % model.shape().vocab) as u32;
            let logits_std = session_std.step(tok).expect("step").to_vec();
            let timing = session_phased
                .measure_step_phases(tok)
                .expect("measure_step_phases");
            assert_eq!(
                logits_std,
                session_phased.logits(),
                "logits mismatch at step {step_idx}"
            );
            sum_ing += timing.ingestion_ns;
            sum_rout += timing.routing_ns;
            sum_mem += timing.memory_lookup_ns;
            sum_em += timing.emission_ns;
            assert!(
                timing.total_ns
                    >= timing.ingestion_ns
                        + timing.routing_ns
                        + timing.memory_lookup_ns
                        + timing.emission_ns
            );
        }
        assert_eq!(session_std.save_state(), session_phased.save_state());
        assert!(sum_ing > 0 && sum_rout > 0 && sum_mem > 0 && sum_em > 0);

        let _ = std::fs::remove_dir_all(artifact_path.parent().unwrap());
    }

    #[test]
    fn test_uor_r4_stack_cost_cli_emits_dynamic_breakdown_and_phase_latency() {
        let artifact_path = make_synthetic_stack_artifact("cli");
        let dir = artifact_path.parent().unwrap().to_path_buf();
        let tokens_path = dir.join("tokens.u16");
        let tokens_bytes: Vec<u8> = (0..256u16).flat_map(|i| (i % 40).to_le_bytes()).collect();
        std::fs::write(&tokens_path, &tokens_bytes).expect("write tokens.u16");

        let output = Command::new(env!("CARGO_BIN_EXE_uor-r4-stack"))
            .args([
                "cost",
                artifact_path.to_str().unwrap(),
                tokens_path.to_str().unwrap(),
                "2",
                "16",
            ])
            .output()
            .expect("run uor-r4-stack cost");
        assert!(
            output.status.success(),
            "uor-r4-stack cost failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let receipt: Value =
            serde_json::from_slice(&output.stdout).expect("parse uor-r4-stack cost JSON");

        assert_eq!(
            receipt.pointer("/tokens/phase_instrumentation_logits_match"),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            receipt
                .pointer("/memory/weights_read_per_token")
                .and_then(Value::as_u64),
            Some(15_968)
        );
        assert_eq!(
            receipt
                .pointer("/memory/total_4bit_weights")
                .and_then(Value::as_u64),
            Some(17_216)
        );
        assert_eq!(
            receipt
                .pointer("/memory/total_learned_scalars")
                .and_then(Value::as_u64),
            Some(342)
        );
        assert_eq!(
            receipt
                .pointer("/memory/total_learned_parameters")
                .and_then(Value::as_u64),
            Some(17_558)
        );
        assert_eq!(
            receipt
                .pointer("/memory/active_learned_parameters_per_token")
                .and_then(Value::as_u64),
            Some(16_310)
        );
        let notes = receipt
            .pointer("/memory/weights_notes")
            .and_then(Value::as_str)
            .unwrap_or("");
        assert!(
            !notes.contains("7,238,304"),
            "weights_notes must not contain hardcoded 7,238,304: {notes}"
        );
        assert!(notes.contains("15968") && notes.contains("17216"));

        let share_sum: f64 = ["ingestion", "routing", "memory_lookup", "emission"]
            .iter()
            .map(|phase| {
                receipt
                    .pointer(&format!(
                        "/step_latency/phase_latency_us/{phase}/share_of_step"
                    ))
                    .and_then(Value::as_f64)
                    .expect("phase share_of_step")
            })
            .sum();
        assert!(
            (share_sum - 1.0).abs() < 1e-6,
            "four phase shares must sum to 1.0, got {share_sum}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
