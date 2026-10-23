//! Exclusive report directories for experiment drivers.
//!
//! A report root is claimed once, with exclusive creation semantics, before any
//! model work; an existing path fails instead of being reused or overwritten.
//! A completed attempt is sealed with a manifest naming every file and its
//! BLAKE3 digest, so retries and interrupted attempts keep distinct directories,
//! logs and charges. This is a small executable guard, not an evidence framework.
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Sentinel written into a freshly claimed report directory.
pub const ATTEMPT_FILE: &str = "attempt.json";
/// Manifest written once when an attempt is sealed.
pub const MANIFEST_FILE: &str = "manifest.json";

/// Create `path` exclusively as a new report directory and write its attempt
/// sentinel. Parents are created as containers; the leaf must not exist. An
/// existing leaf, including an existing completed report, fails before any
/// caller work and is left byte-identical.
pub fn claim(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "report directory {} was not created exclusively ({e}); choose a new attempt path instead of reusing or overwriting an existing report",
                path.display()
            ),
        )
    })?;
    let argv: Vec<String> = std::env::args().collect();
    let sentinel = serde_json::json!({
        "schema": "uor-r4.report-attempt/1",
        "claimed_at": humantime_now(),
        "pid": std::process::id(),
        "argv": argv,
        "rule": "exclusively created before model work; never reused for a retry"
    });
    let mut file = fs::File::create_new(path.join(ATTEMPT_FILE))?;
    io::Write::write_all(
        &mut file,
        serde_json::to_string_pretty(&sentinel)?.as_bytes(),
    )?;
    Ok(())
}

/// Seal a completed attempt: write `manifest.json` (exclusively) listing every
/// regular file below `path` with its size and BLAKE3 digest. Sealing twice fails.
pub fn seal(path: &Path) -> io::Result<PathBuf> {
    let manifest_path = path.join(MANIFEST_FILE);
    if manifest_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{} is already sealed", path.display()),
        ));
    }
    let mut files = Vec::new();
    collect(path, path, &mut files)?;
    files.sort();
    let mut entries = Vec::with_capacity(files.len());
    for relative in files {
        let bytes = fs::read(path.join(&relative))?;
        entries.push(serde_json::json!({
            "path": relative,
            "bytes": bytes.len(),
            "blake3": blake3::hash(&bytes).to_hex().to_string(),
        }));
    }
    let manifest = serde_json::json!({
        "schema": "uor-r4.report-manifest/1",
        "sealed_at": humantime_now(),
        "root": path,
        "files": entries,
    });
    let mut file = fs::File::create_new(&manifest_path)?;
    io::Write::write_all(
        &mut file,
        serde_json::to_string_pretty(&manifest)?.as_bytes(),
    )?;
    Ok(manifest_path)
}

/// Verify a sealed attempt: every listed file must exist with its recorded size and
/// BLAKE3 digest, and the directory must hold no unlisted regular file besides the
/// manifest. Returns the sorted list of unlisted files on success (empty) or an error
/// naming the first listed mismatch; unlisted files are reported as an error too, so
/// a complete file set is checked, not only the listed hashes.
pub fn verify(path: &Path) -> io::Result<Vec<String>> {
    let manifest: serde_json::Value = serde_json::from_slice(&fs::read(path.join(MANIFEST_FILE))?)?;
    let listed = manifest["files"]
        .as_array()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "manifest without files"))?;
    let mut names = std::collections::BTreeSet::new();
    for entry in listed {
        let relative = entry["path"].as_str().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "manifest entry without path")
        })?;
        let bytes = fs::read(path.join(relative))?;
        if entry["bytes"].as_u64() != Some(bytes.len() as u64)
            || entry["blake3"].as_str() != Some(blake3::hash(&bytes).to_hex().as_str())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("sealed file changed: {relative}"),
            ));
        }
        names.insert(relative.to_owned());
    }
    let mut actual = Vec::new();
    collect(path, path, &mut actual)?;
    let unlisted: Vec<String> = actual
        .into_iter()
        .filter(|f| f != MANIFEST_FILE && !names.contains(f))
        .collect();
    if !unlisted.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "sealed directory holds unlisted files: {}",
                unlisted.join(", ")
            ),
        ));
    }
    Ok(unlisted)
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<String>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_dir() {
            collect(root, &path, out)?;
        } else if kind.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| io::Error::other(e.to_string()))?;
            out.push(relative.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

fn humantime_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{now}")
}

/// Two-sided 95% Student-$t$ critical value $t_{0.975, \text{df}}$ for degrees of freedom $\text{df} = n - 1$.
///
/// Exact tabulated values for $1 \le \text{df} \le 30$; second-order Cornish-Fisher expansion ($O(\nu^{-2})$)
/// around $z_{0.975} = 1.959963984540054$ for $\text{df} > 30$.
pub fn student_t_0975(df: usize) -> f64 {
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
        11 => 2.200_985_160_08,
        12 => 2.178_812_829_66,
        13 => 2.160_368_656_46,
        14 => 2.144_786_687_92,
        15 => 2.131_449_545_56,
        16 => 2.119_905_299_22,
        17 => 2.109_815_577_83,
        18 => 2.100_922_040_24,
        19 => 2.093_024_054_41,
        20 => 2.085_963_447_27,
        21 => 2.079_613_844_73,
        22 => 2.073_873_067_9,
        23 => 2.068_657_610_42,
        24 => 2.063_898_561_63,
        25 => 2.059_538_552_75,
        26 => 2.055_529_438_64,
        27 => 2.051_830_516_48,
        28 => 2.048_407_141_8,
        29 => 2.045_229_642_13,
        30 => 2.042_272_456_3,
        _ => {
            let z = 1.959_963_984_540_054_f64;
            let d = df as f64;
            z + (z.powi(3) + z) / (4.0 * d)
                + (5.0 * z.powi(5) + 16.0 * z.powi(3) + 3.0 * z) / (96.0 * d * d)
        }
    }
}

/// Alias for [`student_t_0975`].
pub fn student_t_975(df: usize) -> f64 {
    student_t_0975(df)
}

/// Unbiased sample mean, sample standard deviation ($n - 1$ denominator), and two-sided
/// 95% Student-$t$ confidence interval half-width ($t_{0.975, n-1} \cdot s / \sqrt{n}$).
/// Returns `None` when $n < 2$.
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
    let ci95 = student_t_0975(n - 1) * sem;
    Some((mean, sd, ci95))
}

/// Validation failure returned by [`validate_protocol_v3_energy_receipt`].
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolV3ValidationError {
    /// `work_rate_w` disagrees with `gross_j_per_tok * tokens_per_s` by more than 2%.
    PowerThroughputInconsistency {
        reported_work_rate_w: f64,
        implied_work_rate_w: f64,
        relative_error: f64,
    },
    /// Reported `ci95` disagrees with Student's $t_{0.975, n-1} \cdot s / \sqrt{n}$ by more than 2%.
    InvalidStudentTCi95 {
        metric: String,
        reported_ci95: f64,
        expected_t_ci95: f64,
        relative_error: f64,
    },
    /// Sample count $n < 2$, so unbiased sample variance and Student-$t$ degrees of freedom $\nu = n - 1$ are undefined.
    InvalidSampleCount { metric: String, count: usize },
    /// `work_rate_w <= idle_rate_w` in strict qualification mode (zero or negative net power above idle).
    NonPositiveNetPower { work_rate_w: f64, idle_rate_w: f64 },
    /// One or more external inference daemons (`ollama`, `llama-server`, `vllm`, etc.) were running.
    ExternalInferenceDaemonsDetected(Vec<String>),
    /// Required Protocol v3 full-path field is missing from the receipt.
    MissingRequiredV3Field(&'static str),
}

/// Validate an energy + full-path cost receipt against strict Protocol v3 qualification requirements
/// (`allow_provisional = false`).
pub fn validate_protocol_v3_energy_receipt(
    receipt: &serde_json::Value,
) -> Vec<ProtocolV3ValidationError> {
    validate_protocol_v3_energy_receipt_with_mode(receipt, false)
}

/// Validate an energy + full-path cost receipt against Protocol v3 requirements, with an explicit
/// `allow_provisional` toggle for non-quiescent host rehearsals.
pub fn validate_protocol_v3_energy_receipt_with_mode(
    receipt: &serde_json::Value,
    allow_provisional: bool,
) -> Vec<ProtocolV3ValidationError> {
    use serde_json::Value;
    let mut errors = Vec::new();

    let work_rate_w = receipt
        .pointer("/power/work_rate_w")
        .or_else(|| receipt.pointer("/serving/energy/work_rate_w"))
        .and_then(Value::as_f64);
    let idle_rate_w = receipt
        .pointer("/power/idle_rate_w")
        .or_else(|| receipt.pointer("/serving/energy/idle_power_w"))
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

    if !allow_provisional {
        if let (Some(work_w), Some(idle_w)) = (work_rate_w, idle_rate_w) {
            if work_w <= idle_w {
                errors.push(ProtocolV3ValidationError::NonPositiveNetPower {
                    work_rate_w: work_w,
                    idle_rate_w: idle_w,
                });
            }
        }
    }

    let top_num_runs = receipt
        .pointer("/num_runs")
        .or_else(|| receipt.pointer("/serving/num_runs"))
        .or_else(|| receipt.pointer("/serving/energy/num_runs"))
        .and_then(Value::as_u64)
        .map(|n| n as usize);
    if let Some(n) = top_num_runs {
        if n < 2 {
            errors.push(ProtocolV3ValidationError::InvalidSampleCount {
                metric: "num_runs".to_string(),
                count: n,
            });
        }
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
            let reported_sd = block.get("sd").and_then(Value::as_f64);
            let block_n = block
                .get("n")
                .and_then(Value::as_u64)
                .map(|n| n as usize)
                .or(top_num_runs);

            if let Some(ref s) = samples {
                if s.len() < 2 {
                    errors.push(ProtocolV3ValidationError::InvalidSampleCount {
                        metric: name.to_string(),
                        count: s.len(),
                    });
                } else if let (Some((_mean, _sd, expected_ci95)), Some(reported)) =
                    (sample_mean_sd_t_ci95(s), reported_ci95)
                {
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
            } else if let (Some(sd), Some(n), Some(reported)) =
                (reported_sd, block_n, reported_ci95)
            {
                if n >= 2 {
                    let expected_ci95 = student_t_0975(n - 1) * sd / (n as f64).sqrt();
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

    // Check `/serving/energy/net_j_per_tok/ci95`:
    // 1. If `/serving/energy/net_j_per_tok/samples` is present, validate for any n >= 2.
    // 2. Else if `/serving/energy/net_j_per_tok/sd` and `num_runs >= 2` are present, validate
    //    `ci95 = t_{0.975, n-1} * sd / sqrt(n)` for any n >= 2.
    // 3. Else ONLY when `num_runs == 3` (or `num_runs` is omitted in a 3-run v2 legacy receipt such
    //    as `archive/ops/measurement_report_rnd.json`, where `gross_j_per_tok` records
    //    `min = 0.003135, mean = 0.003172, max = 0.003199` J/tok and `net_j_per_tok` records
    //    `mean = 0.00153, ci95 = 0.000037` J/tok), check the 3-run Student-t_{0.975, 2} CI95.
    if let Some(net_ci95_j) = receipt
        .pointer("/serving/energy/net_j_per_tok/ci95")
        .and_then(Value::as_f64)
    {
        if let Some(samples_j) = receipt
            .pointer("/serving/energy/net_j_per_tok/samples")
            .and_then(Value::as_array)
            .map(|arr| arr.iter().filter_map(Value::as_f64).collect::<Vec<f64>>())
        {
            if samples_j.len() < 2 {
                errors.push(ProtocolV3ValidationError::InvalidSampleCount {
                    metric: "serving.energy.net_j_per_tok".to_string(),
                    count: samples_j.len(),
                });
            } else if let Some((_m, _sd, expected_t_ci95_j)) = sample_mean_sd_t_ci95(&samples_j) {
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
        } else if let (Some(sd_j), Some(n)) = (
            receipt
                .pointer("/serving/energy/net_j_per_tok/sd")
                .and_then(Value::as_f64),
            top_num_runs,
        ) {
            if n >= 2 {
                let expected_t_ci95_j = student_t_0975(n - 1) * sd_j / (n as f64).sqrt();
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
        } else if top_num_runs.unwrap_or(3) == 3 {
            let legacy_3run = receipt
                .pointer("/serving/energy/net_j_per_tok/min")
                .and_then(Value::as_f64)
                .zip(
                    receipt
                        .pointer("/serving/energy/net_j_per_tok/mean")
                        .and_then(Value::as_f64),
                )
                .zip(
                    receipt
                        .pointer("/serving/energy/net_j_per_tok/max")
                        .and_then(Value::as_f64),
                )
                .or_else(|| {
                    receipt
                        .pointer("/serving/energy/gross_j_per_tok/min")
                        .and_then(Value::as_f64)
                        .zip(
                            receipt
                                .pointer("/serving/energy/gross_j_per_tok/mean")
                                .and_then(Value::as_f64),
                        )
                        .zip(
                            receipt
                                .pointer("/serving/energy/gross_j_per_tok/max")
                                .and_then(Value::as_f64),
                        )
                });
            if let Some(((min_j, mean_j), max_j)) = legacy_3run {
                let samples_j = [min_j, mean_j, max_j];
                if let Some((_m, _sd, expected_t_ci95_j)) = sample_mean_sd_t_ci95(&samples_j) {
                    let rel_err =
                        (net_ci95_j - expected_t_ci95_j).abs() / expected_t_ci95_j.max(1e-12);
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

    if !allow_provisional {
        if let Some(arr) = receipt
            .pointer("/machine/external_inference_daemons_detected")
            .and_then(Value::as_array)
        {
            let daemons: Vec<String> = arr
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect();
            if !daemons.is_empty() {
                errors.push(ProtocolV3ValidationError::ExternalInferenceDaemonsDetected(
                    daemons,
                ));
            }
        }
    }

    errors
}

/// All 43 `uor_r4_integer::stack` kernel and mixer symbol suffixes in the `step` call graph
/// (matching `STACK_MANDATORY_SYMBOLS` in `scripts/audit_zero_matmul_serving.py`).
const STACK_CALL_GRAPH_KERNELS: &[&str] = &[
    "stack_step",
    "stack_map_pairs",
    "stack_map_nibbles",
    "stack_step_call",
    "stack_map_task",
    "stack_head_task",
    "stack_row_task",
    "stack_heads",
    "stack_lanes",
    "stack_recurrence",
    "stack_rotation",
    "stack_read",
    "stack_pointer",
    "stack_swiglu",
    "stack_gemv",
    "stack_gemv_pairs",
    "stack_pair_tables",
    "stack_activation_tables",
    "stack_dequant_row",
    "stack_rms_norm",
    "stack_quantize16",
    "stack_max_abs_i32",
    "stack_max_abs_i64",
    "stack_exp_neg",
    "stack_sigmoid_q31",
    "stack_activation",
    "stack_arcosh1p_q24",
    "stack_lift",
    "stack_lorentz_distance",
    "stack_l2_distance",
    "stack_query_tables",
    "stack_dot",
    "stack_mix_row",
    "stack_hamilton",
    "stack_mul_u128",
    "stack_mul_u64",
    "stack_square",
    "stack_div_u64",
    "stack_div_u128",
    "stack_isqrt",
    "stack_argmax",
    "stack_pointer_mixture",
    "stack_snap_select",
    "stack_snap_rotation",
];

fn is_stack_serving_symbol(sym: &str) -> bool {
    if sym.contains("measure_step_phases") {
        return false;
    }
    let is_step = (sym.contains("IntegerStackSession")
        && (sym.ends_with("::step") || sym.contains("::step::")))
        || (sym.contains("19IntegerStackSession") && sym.contains("4step"));
    if is_step {
        return true;
    }
    let in_stack_mod = sym.contains("stack::") || sym.contains("5stack");
    if !in_stack_mod {
        return false;
    }
    for &kernel in STACK_CALL_GRAPH_KERNELS {
        let demangled_exact = format!("::{kernel}");
        let demangled_closure = format!("::{kernel}::");
        let mangled_exact = format!("{}{kernel}", kernel.len());
        if sym.ends_with(&demangled_exact)
            || sym.contains(&demangled_closure)
            || sym.ends_with(&mangled_exact)
        {
            return true;
        }
    }
    false
}

/// Verify that disassembled text from `objdump --demangle -d` or `otool -tV` contains
/// `IntegerStackSession::step` and its `uor_r4_integer::stack` mixer and kernel callees
/// (`stack_step`, `stack_recurrence`, `stack_read`, `stack_gemv_pairs`, `stack_rms_norm`,
/// `stack_dot`, `stack_mix_row`, etc., plus their closures) with zero Class I (multiply),
/// Class II (divide), or Class III (floating-point / SIMD transfer) instructions.
///
/// When `require_full_call_graph` is true, requires `IntegerStackSession::step`, `stack_step`,
/// and at least 20 emitted stack serving symbols across `stack::session` and `stack::kernels`.
pub fn verify_step_disassembly_text(
    disasm_text: &str,
    require_full_call_graph: bool,
) -> Result<(usize, usize), String> {
    const FORBIDDEN_MNEMONICS: &[&str] = &[
        "mul", "madd", "msub", "mneg", "smull", "umull", "smaddl", "umaddl", "smsubl", "umsubl",
        "smulh", "umulh", "sqdmulh", "sqrdmulh", "mla", "mls", "pmul", "pmull", "sdot", "udot",
        "sdiv", "udiv", "imul", "idiv", "fmul", "fmov", "fadd", "fsub", "fdiv", "fmadd", "fmsub",
        "fnmadd", "fnmsub", "fnmul", "fsqrt", "fcmp", "fcmpe", "scvtf", "ucvtf", "fneg", "fabs",
        "fmla", "fmls", "fmax", "fmin", "fcsel", "vfmadd", "vmulss", "vmulsd", "vaddps", "vmulps",
    ];

    let mut current_target_sym: Option<String> = None;
    let mut symbols_checked = 0usize;
    let mut saw_step = false;
    let mut saw_stack_step = false;
    let mut instructions_checked = 0usize;
    let mut violations = Vec::new();

    for raw_line in disasm_text.lines() {
        let line = raw_line.trim();
        if line.ends_with(">:") {
            if let Some(start) = line.find('<') {
                let sym = &line[start + 1..line.len() - 2];
                if is_stack_serving_symbol(sym) {
                    if sym.contains("IntegerStackSession") && sym.ends_with("::step") {
                        saw_step = true;
                    }
                    if sym.ends_with("::stack_step") {
                        saw_stack_step = true;
                    }
                    current_target_sym = Some(sym.to_string());
                    symbols_checked += 1;
                } else {
                    current_target_sym = None;
                }
                continue;
            }
        } else if line.ends_with(':') && !line.contains('\t') && !line.contains(' ') {
            let sym = &line[..line.len() - 1];
            if is_stack_serving_symbol(sym) {
                if sym.contains("19IntegerStackSession") && sym.ends_with("4step") {
                    saw_step = true;
                }
                if sym.ends_with("10stack_step") {
                    saw_stack_step = true;
                }
                current_target_sym = Some(sym.to_string());
                symbols_checked += 1;
            } else {
                current_target_sym = None;
            }
            continue;
        }

        if let Some(ref sym) = current_target_sym {
            let mut ws = line.split_whitespace();
            let Some(first_tok) = ws.next() else {
                continue;
            };
            let addr_clean = first_tok.trim_end_matches(':');
            if addr_clean.is_empty() || !addr_clean.chars().all(|c| c.is_ascii_hexdigit()) {
                continue;
            }
            let mnemonic_opt = if line.contains('\t') {
                line.split('\t')
                    .skip(1)
                    .filter_map(|col| col.split_whitespace().next())
                    .find(|tok| !tok.chars().all(|c| c.is_ascii_hexdigit()))
            } else {
                ws.find(|tok| !tok.chars().all(|c| c.is_ascii_hexdigit()))
            };
            if let Some(raw_mnemonic) = mnemonic_opt {
                let token = raw_mnemonic.to_ascii_lowercase();
                instructions_checked += 1;
                let base_mnemonic = token.split('.').next().unwrap_or(&token);
                if FORBIDDEN_MNEMONICS.contains(&base_mnemonic) {
                    violations.push(format!("{sym}: {token} in `{line}`"));
                }
            }
        }
    }

    if !saw_step || symbols_checked == 0 || instructions_checked == 0 {
        return Err(format!(
            "IntegerStackSession::step symbol or instructions not found in disassembly (saw_step={saw_step}, symbols={symbols_checked}, instructions={instructions_checked})"
        ));
    }
    if require_full_call_graph && (!saw_stack_step || symbols_checked < 20) {
        return Err(format!(
            "expected IntegerStackSession::step, stack_step, and >=20 stack kernel/mixer symbols in full call-graph disassembly, found symbols={symbols_checked} (saw_stack_step={saw_stack_step})"
        ));
    }
    if !violations.is_empty() {
        return Err(format!(
            "forbidden multiply/divide/float instructions in stack serving disassembly: {}",
            violations.join("; ")
        ));
    }
    Ok((symbols_checked, instructions_checked))
}

/// Run `objdump --demangle -d` (falling back to `otool -tV`) on `binary_path` and verify that
/// `IntegerStackSession::step`, `stack_step`, and all emitted `uor_r4_integer::stack` mixer and
/// kernel symbols contain zero multiply, divide, or floating-point instructions.
#[cfg(not(target_arch = "wasm32"))]
pub fn verify_step_objdump_zero_mul_zero_float(
    binary_path: &Path,
) -> Result<(usize, usize), String> {
    let disasm = std::process::Command::new("objdump")
        .args(["--demangle", "-d"])
        .arg(binary_path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .or_else(|| {
            std::process::Command::new("otool")
                .args(["-tV"])
                .arg(binary_path)
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
        })
        .ok_or_else(|| {
            format!(
                "failed to disassemble {} with objdump or otool",
                binary_path.display()
            )
        })?;
    verify_step_disassembly_text(&disasm, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "uor-r4-report-output-{}-{}-{name}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn report_output_new_path_is_claimed_and_sealed_normally() {
        let base = scratch("new");
        let out = base.join("candidate-a").join("evaluate").join("attempt-1");
        claim(&out).unwrap();
        assert!(out.join(ATTEMPT_FILE).is_file());
        fs::write(out.join("result.json"), b"{\"exact\":1}").unwrap();
        fs::create_dir(out.join("nested")).unwrap();
        fs::write(out.join("nested").join("rows.jsonl"), b"{}\n").unwrap();
        let manifest_path = seal(&out).unwrap();
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        let files = manifest["files"].as_array().unwrap();
        let names: Vec<_> = files.iter().map(|f| f["path"].as_str().unwrap()).collect();
        assert_eq!(
            names,
            vec![ATTEMPT_FILE, "nested/rows.jsonl", "result.json"]
        );
        assert_eq!(
            files[2]["blake3"],
            blake3::hash(b"{\"exact\":1}").to_hex().to_string()
        );
        assert!(seal(&out).is_err(), "sealing twice must fail");
        assert!(
            verify(&out).unwrap().is_empty(),
            "a complete sealed set verifies"
        );
        // A file added beneath a sealed directory is detected as unlisted.
        fs::write(out.join("derived-input.json"), b"[]").unwrap();
        let error = verify(&out).unwrap_err().to_string();
        assert!(
            error.contains("unlisted") && error.contains("derived-input.json"),
            "{error}"
        );
        fs::remove_file(out.join("derived-input.json")).unwrap();
        // A changed listed file is detected before the file set is judged.
        fs::write(out.join("result.json"), b"{\"exact\":0}").unwrap();
        assert!(verify(&out)
            .unwrap_err()
            .to_string()
            .contains("result.json"));
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn report_output_existing_report_fails_before_work_and_stays_byte_identical() {
        let base = scratch("collision");
        let out = base.join("evaluate").join("attempt-1");
        claim(&out).unwrap();
        let report = out.join("result.json");
        fs::write(&report, b"original rejected-candidate rows").unwrap();
        let sentinel_before = fs::read(out.join(ATTEMPT_FILE)).unwrap();
        let error = claim(&out).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert!(error.to_string().contains("new attempt path"));
        assert_eq!(
            fs::read(&report).unwrap(),
            b"original rejected-candidate rows"
        );
        assert_eq!(fs::read(out.join(ATTEMPT_FILE)).unwrap(), sentinel_before);
        // A plain pre-existing directory without a sentinel is also refused.
        let plain = base.join("plain");
        fs::create_dir(&plain).unwrap();
        fs::write(plain.join("result.json"), b"keep").unwrap();
        assert_eq!(
            claim(&plain).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(fs::read(plain.join("result.json")).unwrap(), b"keep");
        // A distinct attempt path beside it completes normally.
        let next = base.join("evaluate").join("attempt-2");
        claim(&next).unwrap();
        assert!(next.join(ATTEMPT_FILE).is_file());
        fs::remove_dir_all(base).unwrap();
    }
}
