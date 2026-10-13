//! Fixed murmuration qualification controller; Rust orchestration, never model math.
//! Build with rustc --edition=2021 on the admitted pod. Requires a FIT sheet
//! and exact source pin. Preliminary inventory only: repair/composition control
//! names must be incorporated before prospective FIT. Never runs an optimizer.
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Child, Command, ExitCode, Stdio},
    thread,
    time::{Duration, Instant},
};

// Driver telemetry may remain in uninterruptible sleep after SIGKILL. Reap
// promptly when possible, but never block the controller waiting for it.
fn stop_telemetry(child: &mut Child) {
    let _ = child.kill();
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(2) {
        match child.try_wait() {
            Ok(Some(_)) | Err(_) => return,
            Ok(None) => thread::sleep(Duration::from_millis(25)),
        }
    }
}

// Only this owned telemetry child is stopped on completion/error. The samples
// are observed GPU memory use, not a claim of an unsampled absolute peak.
struct DeviceSamples(Child);
impl DeviceSamples {
    fn start(report: &Path) -> Result<Self, String> {
        let out =
            fs::File::create(report.join("gpu-memory-samples.csv")).map_err(|e| e.to_string())?;
        let err = fs::File::create(report.join("gpu-memory-samples.stderr.log"))
            .map_err(|e| e.to_string())?;
        Command::new("nvidia-smi")
            .args([
                "--query-gpu=timestamp,uuid,memory.used,utilization.gpu",
                "--format=csv",
                "--loop-ms=200",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::from(out))
            .stderr(Stdio::from(err))
            .spawn()
            .map(Self)
            .map_err(|e| e.to_string())
    }
}
impl Drop for DeviceSamples {
    fn drop(&mut self) {
        stop_telemetry(&mut self.0);
    }
}

const META: &[&str] = &[
    "starling_native::murmuration::tests::murm_table_matches_canonical_zphi",
    "starling_native::murmuration::tests::murm_refuses_cpu_and_oversized_flocks",
    "starling_native::murmuration::tests::murm_right_table_exact_and_cocycle",
    "starling_native::murmuration::tests::murm_odd_heads_refused",
    "starling_native::murmuration_tree::tests::murm_tree_level_counts_and_bounds",
    "starling_native::murmuration_tree::tests::murm_tree_plan_respects_documents",
    "starling_native::murmuration_tree::tests::murm_tree_refuses_cpu",
];
const GPU: &[&str] = &[
    "starling_native::murmuration::tests::murm_forward_independent_reference",
    "starling_native::murmuration::tests::murm_future_value_is_inert",
    "starling_native::murmuration::tests::murm_fixed_point_and_identity",
    "starling_native::murmuration::tests::murm_backward_secants",
    "starling_native::murmuration::tests::murm_address_margin_direction",
    "starling_native::murmuration::tests::murm_zero_coupling_wakes",
    "starling_native::murmuration::tests::murm_left_heads_unchanged",
    "starling_native::murmuration::tests::murm_right_forward_independent_reference",
    "starling_native::murmuration::tests::murm_right_backward_secants",
    "starling_native::murmuration_tree::tests::murm_tree_closure_known_answer",
    "starling_native::murmuration_tree::tests::murm_tree_strict_causality",
    "starling_native::murmuration_tree::tests::murm_tree_backward_secants",
    "starling_native::murmuration_tree::tests::murm_tree_null_parent_refused",
];
const MODULE: &[&str] = &[
    "compiled_sources_carry_no_banned_operation",
    "no_excluded_historical_kernel_is_compiled",
    "nothing_in_the_crate_calls_a_runtime_compiler",
    "quarantined_dense_contraction_modules_stay_out_of_the_build",
    "built_ptx_modules_carry_no_banned_instruction",
];

fn capture(program: &Path, args: &[&str], report: &Path, label: &str) -> Result<String, String> {
    let start = std::time::Instant::now();
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    let text = format!(
        "command: {} {:?}\nstatus: {}\nelapsed_s: {:.6}\nstdout:\n{}\nstderr:\n{}",
        program.display(),
        args,
        output.status,
        start.elapsed().as_secs_f64(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::write(report.join(format!("{label}.log")), &text).map_err(|e| e.to_string())?;
    if label.starts_with("control-")
        && (String::from_utf8_lossy(&output.stdout).contains("D29:")
            || String::from_utf8_lossy(&output.stderr).contains("D29:"))
    {
        return Err(format!(
            "{label} emitted a D29 refusal; preserved in {}",
            report.display()
        ));
    }
    if !output.status.success() {
        return Err(format!("{label} failed; preserved in {}", report.display()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

// Only the NVIDIA identity probe is bounded. Regular files preserve partial
// output without pipe EOF waits if the driver cannot retire the owned child.
// Numerical tests continue to use capture(), with no wall-time cancellation.
fn capture_identity(report: &Path) -> Result<(), String> {
    let args = [
        "--query-gpu=name,uuid,driver_version,memory.total",
        "--format=csv",
    ];
    let stdout_path = report.join("device.stdout.log");
    let stderr_path = report.join("device.stderr.log");
    let stdout = fs::File::create(&stdout_path).map_err(|e| e.to_string())?;
    let stderr = fs::File::create(&stderr_path).map_err(|e| e.to_string())?;
    let started = Instant::now();
    let mut child = Command::new("nvidia-smi")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|e| {
            format!(
                "device probe spawn failed; logs in {}: {e}",
                report.display()
            )
        })?;
    let pid = child.id();
    let result = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) => {}
            Err(error) => break Err(format!("device probe poll failed: {error}")),
        }
        if started.elapsed() >= Duration::from_secs(30) {
            break Err("device identity probe timed out; no controls executed".to_string());
        }
        thread::sleep(Duration::from_millis(25));
    };
    if result.is_err() {
        stop_telemetry(&mut child);
    }
    let status = match &result {
        Ok(status) => status.to_string(),
        Err(error) => error.clone(),
    };
    let stdout = fs::read(&stdout_path).map_err(|e| e.to_string())?;
    let stderr = fs::read(&stderr_path).map_err(|e| e.to_string())?;
    fs::write(
        report.join("device.log"),
        format!(
            "command: nvidia-smi {args:?}\nowned_pid: {pid}\nstatus: {status}\nelapsed_s: {:.6}\nstdout:\n{}\nstderr:\n{}",
            started.elapsed().as_secs_f64(),
            String::from_utf8_lossy(&stdout),
            String::from_utf8_lossy(&stderr),
        ),
    )
    .map_err(|e| e.to_string())?;
    match result {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => Err(format!(
            "device query failed; preserved in {}",
            report.display()
        )),
        Err(error) => Err(format!(
            "{error}; partial logs preserved in {}",
            report.display()
        )),
    }
}

fn sha256_pin(pin: &str) -> bool {
    pin.len() == 64
        && pin
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn run() -> Result<(), String> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.len() != 8 {
        return Err("usage: qualify-murmuration REPO SOURCE_SHA LIB_TEST REPORT_ROOT FIT_RECEIPT_URL MODULE_TEST LIB_SHA256 MODULE_SHA256".into());
    }
    if !cfg!(target_os = "linux")
        || env::var("CUDA_VISIBLE_DEVICES")
            .unwrap_or_default()
            .is_empty()
    {
        return Err(
            "qualification requires the admitted Linux CUDA pod and explicit CUDA_VISIBLE_DEVICES"
                .into(),
        );
    }
    let repo = Path::new(&args[0])
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if args[1].len() != 40 || !args[1].bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("SOURCE_SHA must be the full frozen git commit".into());
    }
    if !args[4].starts_with("https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-") {
        return Err("FIT receipt must identify the prospective milestone comment".into());
    }
    if !sha256_pin(&args[6]) || !sha256_pin(&args[7]) {
        return Err("expected library/module SHA256 pins must each be 64 lowercase hexadecimal characters from the exact-source setup manifest".into());
    }
    env::set_current_dir(&repo).map_err(|e| e.to_string())?;
    let sheet = fs::read_to_string(
        repo.join("docs/harness/prep/m4-murmuration-s1-s2-finite-2026-10-13.md"),
    )
    .map_err(|e| e.to_string())?;
    if !sheet.lines().any(|l| l.trim() == "TEST FITNESS: FIT") {
        return Err("finite sheet is not FIT; no controls executed".into());
    }
    let report = PathBuf::from(&args[3]);
    let basename = args[3].rsplit('/').next().unwrap_or_default();
    if !report.is_absolute() || matches!(basename, "" | "." | "..") {
        return Err("exclusive report needs an absolute path and a normal basename".into());
    }
    let boundary = Path::new("/workspace/uor-r4/jobs")
        .canonicalize()
        .map_err(|e| format!("report boundary: {e}"))?;
    let parent = report
        .parent()
        .ok_or("report has no parent")?
        .canonicalize()
        .map_err(|e| format!("report parent: {e}"))?;
    if !parent.starts_with(&boundary) {
        return Err(
            "exclusive report must be under /workspace/uor-r4/jobs for admitted preservation"
                .into(),
        );
    }
    let report = parent.join(basename);
    fs::create_dir(&report).map_err(|e| format!("exclusive report: {e}"))?;
    fs::write(
        report.join("scope.txt"),
        format!(
            "source={}\nFIT={}\nlibrary_sha256={}\nmodule_sha256={}\ntraining=HELD\noptimizer_updates=0\ncontrols={}\n",
            args[1],
            args[4],
            args[6],
            args[7],
            META.len() + GPU.len() + MODULE.len()
        ),
    )
    .map_err(|e| e.to_string())?;
    let head = capture(Path::new("git"), &["rev-parse", "HEAD"], &report, "source")?;
    if head.trim() != args[1] {
        return Err("source pin mismatch".into());
    }
    let dirty = capture(
        Path::new("git"),
        &["status", "--porcelain", "--untracked-files=no"],
        &report,
        "tracked-status",
    )?;
    if !dirty.is_empty() {
        return Err("tracked source is dirty".into());
    }
    capture_identity(&report)?;
    let binaries = [
        Path::new(&args[2])
            .canonicalize()
            .map_err(|e| e.to_string())?,
        Path::new(&args[5])
            .canonicalize()
            .map_err(|e| e.to_string())?,
    ];
    let target = binaries[0]
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or("library test does not have a Cargo target/debug/deps parent")?;
    if !target.starts_with("/workspace") || !binaries.iter().all(|p| p.starts_with(target)) {
        return Err("test binaries must share the owned /workspace target directory".into());
    }
    // Tests use this variable only as their artifact-output parent. Keep the
    // tiny round-trip artifacts with the archived report, not the build cache.
    let fixture_root = report.join("fixture-artifacts");
    fs::create_dir(&fixture_root).map_err(|e| e.to_string())?;
    env::set_var("CARGO_TARGET_DIR", &fixture_root);
    env::set_var("UOR_FINITE_REPORT_ROOT", &report);
    let actual_hashes = capture(
        Path::new("sha256sum"),
        &[
            binaries[0].to_str().ok_or("non-UTF8 lib path")?,
            binaries[1].to_str().ok_or("non-UTF8 module path")?,
        ],
        &report,
        "executables",
    )?;
    let expected = [&args[6], &args[7]];
    let lines: Vec<_> = actual_hashes.lines().collect();
    if lines.len() != expected.len()
        || lines
            .iter()
            .zip(expected)
            .any(|(line, pin)| line.split_whitespace().next() != Some(pin.as_str()))
    {
        return Err("executable SHA256 differs from the frozen exact-source setup manifest; no controls executed".into());
    }
    // Preflight every exact test before any numerical execution. A zero-test
    // success or ignored test is never accepted as qualification.
    let groups = [
        (&binaries[0], META, false),
        (&binaries[0], GPU, true),
        (&binaries[1], MODULE, false),
    ];
    for (i, (binary, names, _)) in groups.iter().enumerate() {
        let list = capture(binary, &["--list"], &report, &format!("list-{i}"))?;
        for name in *names {
            if list
                .lines()
                .filter(|line| *line == format!("{name}: test"))
                .count()
                != 1
            {
                return Err(format!("missing or ambiguous frozen control {name}"));
            }
        }
    }
    let _samples = DeviceSamples::start(&report)?;
    let mut passed = 0;
    let total = META.len() + GPU.len() + MODULE.len();
    for (binary, names, include_ignored) in groups {
        for name in names {
            let mut selected = vec![*name, "--exact", "--nocapture", "--test-threads=1"];
            // This opt-in applies only to the exact preflighted GPU name.
            // No unfiltered ignored suite or arbitrary user command is accepted.
            if include_ignored {
                selected.push("--include-ignored");
            }
            let out = capture(
                binary,
                &selected,
                &report,
                &format!("control-{:02}", passed + 1),
            )?;
            if !out.contains("test result: ok. 1 passed; 0 failed; 0 ignored;")
                || out.contains("D29:")
            {
                return Err(format!(
                    "control {name} did not execute exactly one passing nonignored test"
                ));
            }
            passed += 1;
            println!("PASS {passed}/{total} {name}");
        }
    }
    fs::write(
        report.join("RESULT.txt"),
        format!(
            "{passed}/{total} fixed controls PASS\nTraining HELD; no optimizer or corpus run\n"
        ),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("BLOCKER: {error}");
            ExitCode::FAILURE
        }
    }
}
