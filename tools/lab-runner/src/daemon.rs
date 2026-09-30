//! Singleton host daemon. Launch is gated until durable process identity
//! exists; restart never treats a bare PID or absent exit code as success.
use crate::admission::{admit, Load};
use crate::host::{self, HostPolicy};
use crate::jobs::{self, Attempt, Finalization};
use crate::process::{self, Identity};
use crate::spec::{JobSpec, KillKind};
use crate::{invalid, Result};
use serde_json::json;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub struct DaemonConfig {
    pub root: PathBuf,
    pub ledger_dir: PathBuf,
    pub poll_interval: Duration,
    pub monitor_interval: Duration,
    pub stop_file: Option<PathBuf>,
    /// Production always true. False is an explicit temporary test harness.
    pub require_host_policy: bool,
}
struct RunningJob {
    spec: JobSpec,
    child: Option<Child>,
    identity: Identity,
    attempt: Attempt,
    started: Instant,
    elapsed_before_ms: u64,
    measurement: crate::ledger::Measurement,
    peak_rss_kib: u64,
    log_offsets: [u64; 2],
    log_carry: String,
    /// The owned supervisor is verifying inputs; payload.go remains closed.
    preflight_pending: bool,
}

fn preflight_or_record_failure(
    root: &Path,
    spec: &JobSpec,
    attempt: &Attempt,
    started: Instant,
    preflight: impl FnOnce() -> Result<()>,
) -> Result<()> {
    match preflight() {
        Ok(()) => Ok(()),
        Err(error) => {
            jobs::record_pre_spawn_failure(
                root,
                spec,
                attempt,
                started.elapsed().as_millis() as u64,
                error.to_string(),
            )?;
            Err(error)
        }
    }
}
impl RunningJob {
    fn elapsed_ms(&self) -> u64 {
        self.elapsed_before_ms
            .saturating_add(self.started.elapsed().as_millis() as u64)
    }
    fn dir(&self, root: &Path) -> PathBuf {
        jobs::running_dir(root).join(&self.spec.id)
    }
}

// Carry the same running set through the late, post-hashing check. Passing an
// empty set here would trigger the coordinator's missing-reservation fence.
fn recheck_resources(
    spec: &JobSpec,
    running: &[JobSpec],
    host_check: impl FnOnce(&[JobSpec]) -> Result<()>,
    budget_check: impl FnOnce(u64) -> Result<()>,
) -> Result<()> {
    host_check(running)?;
    let reserved = running.iter().try_fold(spec.reserved_ms()?, |sum, job| {
        sum.checked_add(job.reserved_ms()?)
            .ok_or_else(|| invalid("wall reservation overflow"))
    })?;
    budget_check(reserved)
}

fn spawn_job(
    root: &Path,
    spec: &JobSpec,
    test_mode: bool,
    ledger_dir: &Path,
    running: &[JobSpec],
) -> Result<RunningJob> {
    let verifier = if !test_mode && !running.is_empty() {
        Some(std::env::current_exe()?)
    } else {
        None
    };
    spawn_job_with_verifier(
        root,
        spec,
        test_mode,
        ledger_dir,
        running,
        verifier.as_deref(),
    )
}

fn spawn_job_with_verifier(
    root: &Path,
    spec: &JobSpec,
    test_mode: bool,
    ledger_dir: &Path,
    running: &[JobSpec],
    verifier: Option<&Path>,
) -> Result<RunningJob> {
    let started = Instant::now();
    let dir = jobs::running_dir(root).join(&spec.id);
    if fs::symlink_metadata(dir.join("preflight-failure.json")).is_ok() {
        return Err(invalid(
            "terminal preflight proof forbids relaunch of this job",
        ));
    }
    let attempt = jobs::prepare_reserved_attempt(
        &dir,
        &spec.id,
        spec.coordination.as_ref().map(|c| c.attempt_id.as_str()),
    )?;
    if !test_mode && verifier.is_none() {
        preflight_or_record_failure(root, spec, &attempt, started, || {
            spec.verify_provenance()?;
            // Hashing can outlive ownership. Refresh authority/resources here.
            let policy = HostPolicy::load(root)?;
            recheck_resources(
                spec,
                running,
                |peers| host::check_admission(root, &policy, spec, peers),
                |reserved| crate::ledger::check_budget(ledger_dir, reserved).map(|_| ()),
            )?;
            if started.elapsed() >= Duration::from_secs(spec.wall_s) {
                return Err(invalid("preflight exhausted the reserved job wall time"));
            }
            Ok(())
        })?;
        // A failure while consuming an existing/uncertain tombstone cannot
        // assert globally that this attempt never ran; retain the normal hold.
        jobs::consume_attempt(ledger_dir, root, spec)?;
    }
    let spec_digest = crate::coord::digest(&serde_json::to_vec(spec)?);
    if verifier.is_some() {
        crate::write_json_atomic(
            &dir.join("verification-required.json"),
            &json!({"schema":"uor-r4.input-verification/1", "spec_sha256":spec_digest,
                "state":"pending", "payload_started":false}),
        )?;
    }
    let stdout = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join("stdout.log"))?;
    let stderr = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join("stderr.log"))?;
    let hold_fifo = dir.join("supervisor-hold.fifo");
    let fifo = Command::new("/usr/bin/mkfifo").arg(&hold_fifo).status()?;
    if !fifo.success() {
        return Err(invalid("could not create supervisor hold FIFO"));
    }
    // Positional arguments avoid shell interpolation of job input. Before
    // launch.go exists this wrapper cannot execute the requested workload.
    let script = "shift; gate=$1; result=$2; hold=$3; verifier=$4; spec=$5; digest=$6; verified=$7; payload_gate=$8; shift 8; n=0; while [ ! -f \"$gate\" ]; do n=$((n+1)); [ \"$n\" -lt 200 ] || exit 125; /bin/sleep 0.05; done; status=0; if [ -n \"$verifier\" ]; then \"$verifier\" verify-inputs \"$spec\" \"$digest\" > \"$verified.stdout\" 2> \"$verified.stderr\" & child=$!; wait \"$child\"; status=$?; printf '%s\\n' \"$status\" > \"$verified\"; while [ \"$status\" -eq 0 ] && [ ! -f \"$payload_gate\" ]; do /bin/sleep 0.05; done; fi; if [ \"$status\" -eq 0 ]; then \"$@\" & child=$!; wait \"$child\"; status=$?; fi; printf '%s\\n' \"$status\" > \"$result\"; exec 3<> \"$hold\"; IFS= read -r ack <&3; exit \"$status\"";
    let mut argv = spec.argv.clone();
    if test_mode {
        argv[0] = match Path::new(&argv[0]).file_name().and_then(|s| s.to_str()) {
            Some("sh") => "/bin/sh",
            Some("sleep") => "/bin/sleep",
            Some("true") => "/usr/bin/true",
            Some("false") => "/usr/bin/false",
            _ => return Err(invalid("non-fixture test executable")),
        }
        .to_string();
    }
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", script, process::SUPERVISOR_MARKER])
        .arg(&attempt.process_token)
        .arg(dir.join("launch.go"))
        .arg(dir.join("payload.exit"))
        .arg(&hold_fifo)
        .arg(verifier.unwrap_or_else(|| Path::new("")))
        .arg(dir.join("spec.json"))
        .arg(&spec_digest)
        .arg(dir.join("verification.exit"))
        .arg(dir.join("payload.go"))
        .args(&argv)
        .current_dir(&spec.cwd)
        .env_clear();
    for key in ["PATH", "HOME", "TMPDIR", "USER", "LANG"] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    if let Some(env) = &spec.env {
        command.envs(env);
    }
    command.env("UOR_RUNNER_PROCESS_TOKEN", &attempt.process_token);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()?;
    let capture = process::capture(child.id(), &attempt.process_token);
    let identity = match capture {
        Ok(i) => i,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
    };
    let persist = (|| -> Result<()> {
        crate::atomic_write(
            &dir.join("process.json"),
            &serde_json::to_vec_pretty(&identity)?,
        )?;
        crate::atomic_write(&dir.join("pid"), child.id().to_string().as_bytes())?;
        crate::atomic_write(&dir.join("launch.go"), attempt.attempt_id.as_bytes())?;
        Ok(())
    })();
    if let Err(error) = persist {
        let _ = process::terminate(&identity, Duration::from_millis(spec.stop_grace_ms));
        let _ = child.wait();
        return Err(error);
    }
    Ok(RunningJob {
        spec: spec.clone(),
        child: Some(child),
        identity,
        attempt,
        started,
        elapsed_before_ms: 0,
        measurement: crate::ledger::Measurement::Measured,
        peak_rss_kib: 0,
        log_offsets: [0, 0],
        log_carry: String::new(),
        preflight_pending: verifier.is_some(),
    })
}
fn finalize_job(
    root: &Path,
    ledger: &Path,
    job: &RunningJob,
    outcome: &str,
    code: Option<i64>,
    reason: String,
) -> Result<()> {
    jobs::finalize(
        root,
        &job.spec,
        &Finalization {
            outcome: outcome.into(),
            exit_status: code,
            reason: if job.preflight_pending {
                format!("input verification phase; payload launch not confirmed: {reason}")
            } else {
                reason
            },
            peak_rss_kib: job.peak_rss_kib,
            started_utc: job.attempt.started_utc.clone(),
            elapsed_ms: job.elapsed_ms(),
            measurement: job.measurement,
        },
        ledger,
    )
}
fn code(status: std::process::ExitStatus) -> i64 {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        status
            .code()
            .map(i64::from)
            .unwrap_or_else(|| -i64::from(status.signal().unwrap_or(0)))
    }
    #[cfg(not(unix))]
    {
        status.code().map(i64::from).unwrap_or(-1)
    }
}
fn stop_job(
    root: &Path,
    ledger: &Path,
    job: &mut RunningJob,
    outcome: &str,
    reason: String,
) -> Result<()> {
    if process::matches(&job.identity) {
        jobs::record_stop_request(&job.dir(root), job.spec.stop_grace_ms, &reason)?;
        process::terminate(&job.identity, Duration::from_millis(job.spec.stop_grace_ms))?;
    } else if !process::owned_members(&job.identity).is_ok_and(|members| members.is_empty()) {
        host::hold(
            root,
            "process identity mismatch during stop; manual reconciliation required",
        )?;
        return finalize_job(
            root,
            ledger,
            job,
            "unknown",
            None,
            "identity mismatch; no signal sent".into(),
        );
    }
    let status = job
        .child
        .as_mut()
        .and_then(|c| c.try_wait().ok().flatten())
        .map(code);
    finalize_job(root, ledger, job, outcome, status, reason)
}

/// Finalization may only have persisted an UNKNOWN receipt. Keep both the
/// runtime paths and an unreaped Child alive in the monitor until stopped
/// reconciliation has archived the directory and the known child is reaped.
fn retain_tracking(root: &Path, job: &mut RunningJob) -> Result<bool> {
    if let Some(child) = job.child.as_mut() {
        if child.try_wait()?.is_some() {
            job.child = None;
        }
    }
    Ok(job.dir(root).is_dir() || job.child.is_some())
}

fn recovery_required(root: &Path, dir: &Path, reason: &str) -> Result<()> {
    crate::write_json_atomic(
        &dir.join("recovery-required.json"),
        &json!({"schema":"uor-r4.job-recovery/1","outcome":"unknown","reason":reason,"at":crate::utc_now_iso()}),
    )?;
    host::hold(root, reason)
}
fn recover(root: &Path, ledger: &Path, production: bool) -> Result<Vec<RunningJob>> {
    let mut running = vec![];
    for entry in fs::read_dir(jobs::running_dir(root))? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let dir = entry.path();
        if dir.join("exit.json").is_file() {
            jobs::replay_finalization(root, &dir, ledger)?;
            if !dir.is_dir() {
                continue;
            }
        }
        let spec = match jobs::read_spec(&dir) {
            Ok(s) => s,
            Err(_) => {
                recovery_required(root, &dir, "unreadable running spec")?;
                continue;
            }
        };
        let attempt = match jobs::read_attempt(&dir) {
            Ok(a) => a,
            Err(_) => {
                recovery_required(
                    root,
                    &dir,
                    "legacy running job lacks launch receipt; no automatic retry or signal",
                )?;
                continue;
            }
        };
        let identity = match jobs::read_identity(&dir) {
            Ok(i) => i,
            Err(_) => {
                recovery_required(root,&dir,"launch receipt has no durable process identity; gated process expires without executing")?;
                continue;
            }
        };
        let elapsed = jobs::now_ms().saturating_sub(attempt.started_ms);
        let mut job = RunningJob {
            spec,
            child: None,
            identity,
            attempt,
            started: Instant::now(),
            elapsed_before_ms: elapsed,
            measurement: crate::ledger::Measurement::Estimated,
            peak_rss_kib: 0,
            log_offsets: [0, 0],
            log_carry: String::new(),
            preflight_pending: dir.join("verification-required.json").is_file()
                && !dir.join("payload.go").is_file(),
        };
        if dir.join("exit.json").is_file() || job.identity.host.is_empty() {
            recovery_required(root, &dir, "unresolved receipt or legacy host identity; preserve runtime paths for stopped reconciliation")?;
            running.push(job);
            continue;
        }
        if !process::matches(&job.identity) {
            if process::owned_members(&job.identity).is_ok_and(|members| !members.is_empty()) {
                jobs::record_stop_request(&job.dir(root), job.spec.stop_grace_ms, "recovery stop")?;
                process::terminate(&job.identity, Duration::from_millis(job.spec.stop_grace_ms))?;
            }
            host::hold(
                root,
                "recovered job lost process identity; review unknown outcome",
            )?;
            finalize_job(
                root,
                ledger,
                &job,
                "unknown",
                None,
                "process absent or identity changed; no exit status available".into(),
            )?;
        } else if production {
            host::hold(root, "supervisor restart requires checkpoint/process reconciliation; no wall-clock adoption")?;
            stop_job(
                root,
                ledger,
                &mut job,
                "interrupted",
                "supervisor restarted; verified process stopped without resetting wall budget"
                    .into(),
            )?;
        } else if host::validate_test_job(root, &job.spec).is_err() {
            host::hold(
                root,
                "non-fixture running job cannot be adopted in test mode",
            )?;
            stop_job(
                root,
                ledger,
                &mut job,
                "interrupted",
                "test mode refuses non-fixture recovery".into(),
            )?;
        } else if !dir.join("launch.go").is_file() {
            stop_job(
                root,
                ledger,
                &mut job,
                "interrupted",
                "launch gate was never released".into(),
            )?;
        }
        if retain_tracking(root, &mut job)? {
            running.push(job);
        }
    }
    Ok(running)
}
/// Finish the one pending verification only after the normal monitor has run.
/// The verifier shares the candidate's owned supervisor, resource bounds and
/// durable attempt. It cannot launch the workload before this second gate.
fn advance_preflight(config: &DaemonConfig, running: &mut [RunningJob]) -> Result<()> {
    let Some(index) = running.iter().position(|job| job.preflight_pending) else {
        return Ok(());
    };
    let dir = running[index].dir(&config.root);
    let Ok(bytes) = fs::read(dir.join("verification.exit")) else {
        return Ok(());
    };
    // A partial/nonzero record never authorizes the workload. The ordinary
    // monitor records verifier failure, wall exhaustion or unknown identity.
    if bytes != b"0\n" {
        return Ok(());
    }
    let peers: Vec<_> = running
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != index)
        .map(|(_, job)| job.spec.clone())
        .collect();
    let job = &mut running[index];
    let checked = (|| -> Result<()> {
        if dir.join("exit.json").exists() || dir.join("recovery-required.json").exists() {
            return Err(invalid("verification has an unresolved execution receipt"));
        }
        if config.stop_file.as_ref().is_some_and(|path| path.exists())
            || config.root.join("admissions-held.json").exists()
        {
            return Err(invalid("admission held during input verification"));
        }
        if !process::matches(&job.identity) {
            return Err(invalid("verifier supervisor identity unavailable"));
        }
        let saved = jobs::read_spec(&dir)?;
        if crate::coord::digest(&serde_json::to_vec(&saved)?)
            != crate::coord::digest(&serde_json::to_vec(&job.spec)?)
        {
            return Err(invalid("spec changed during input verification"));
        }
        if config.require_host_policy {
            let policy = HostPolicy::load(&config.root)?;
            recheck_resources(
                &job.spec,
                &peers,
                |others| host::check_admission(&config.root, &policy, &job.spec, others),
                |reserved| crate::ledger::check_budget(&config.ledger_dir, reserved).map(|_| ()),
            )?;
        }
        if job.elapsed_ms() >= job.spec.wall_s.saturating_mul(1000) {
            return Err(invalid("input verification exhausted reserved wall time"));
        }
        // Source/executable/input verification has completed successfully.
        // Preserve the original consume-before-payload ordering, not before hashing.
        if config.require_host_policy {
            jobs::consume_attempt(&config.ledger_dir, &config.root, &job.spec)?;
        }
        crate::atomic_write(&dir.join("payload.go"), job.attempt.attempt_id.as_bytes())?;
        Ok(())
    })();
    match checked {
        Ok(()) => job.preflight_pending = false,
        Err(error) => {
            host::hold(
                &config.root,
                &format!("input verification admission failed: {error}"),
            )?;
            stop_job(
                &config.root,
                &config.ledger_dir,
                job,
                "failed",
                format!("late admission failed: {error}"),
            )?;
        }
    }
    Ok(())
}

fn new_log_bytes(job: &mut RunningJob, root: &Path) -> String {
    let mut fresh = std::mem::take(&mut job.log_carry);
    for (index, name) in ["stdout.log", "stderr.log"].iter().enumerate() {
        let Ok(mut file) = fs::File::open(job.dir(root).join(name)) else {
            continue;
        };
        let Ok(meta) = file.metadata() else {
            continue;
        };
        if meta.len() < job.log_offsets[index] {
            job.log_offsets[index] = 0;
        }
        if file.seek(SeekFrom::Start(job.log_offsets[index])).is_err() {
            continue;
        }
        let mut bytes = Vec::new();
        if file.take(65536).read_to_end(&mut bytes).is_err() {
            continue;
        }
        job.log_offsets[index] = job.log_offsets[index].saturating_add(bytes.len() as u64);
        fresh.push_str(&String::from_utf8_lossy(&bytes));
    }
    fresh
}
fn monitor(
    root: &Path,
    ledger: &Path,
    job: &mut RunningJob,
    sample: bool,
    host_failure: Option<&str>,
) -> Result<bool> {
    if !job.dir(root).is_dir() {
        // A CLI finalizer can move the directory while the daemon still owns
        // the Child handle; reap it before forgetting the job.
        return retain_tracking(root, job);
    }
    if job.dir(root).join("exit.json").is_file() {
        jobs::replay_finalization(root, &job.dir(root), ledger)?;
        // A transient failed observation must not disable the original wall
        // and resource bounds forever. Once full current ownership is again
        // verified, stop the uncertain attempt rather than silently adopt it.
        // Legacy identities cannot satisfy matches and are never signalled.
        if job.dir(root).is_dir() && process::matches(&job.identity) {
            jobs::record_stop_request(
                &job.dir(root),
                job.spec.stop_grace_ms,
                "uncertain attempt ownership reverified; bounded stop",
            )?;
            process::terminate(&job.identity, Duration::from_millis(job.spec.stop_grace_ms))?;
        }
        return retain_tracking(root, job);
    }
    if job.dir(root).join("cancel.json").is_file() {
        stop_job(
            root,
            ledger,
            job,
            "cancelled",
            "cancel marker received".into(),
        )?;
        return retain_tracking(root, job);
    }
    if let Ok(bytes) = fs::read(job.dir(root).join("payload.exit")) {
        // A complete short line is the shell wait result. The supervisor
        // remains blocked on its private FIFO, anchoring all descendants.
        if bytes.len() <= 5 && bytes.last() == Some(&b'\n') {
            if let Ok(status) = String::from_utf8_lossy(&bytes).trim().parse::<u8>() {
                let members = process::owned_members(&job.identity)?;
                let background = members.iter().any(|member| member.pid != job.identity.pid);
                if background {
                    jobs::record_stop_request(
                        &job.dir(root),
                        job.spec.stop_grace_ms,
                        "background descendant stop",
                    )?;
                }
                process::terminate(&job.identity, Duration::from_millis(job.spec.stop_grace_ms))?;
                if let Some(child) = job.child.as_mut() {
                    let _ = child.try_wait()?;
                }
                finalize_job(
                    root,
                    ledger,
                    job,
                    if status == 0 && !background {
                        "completed"
                    } else {
                        "failed"
                    },
                    Some(i64::from(status)),
                    if background {
                        "payload exited with background descendants; verified descendants stopped"
                            .into()
                    } else {
                        format!("payload exited with status {status}; supervisor stopped")
                    },
                )?;
                return retain_tracking(root, job);
            }
        }
    }
    if let Some(child) = job.child.as_mut() {
        if let Some(status) = child.try_wait()? {
            let status = code(status);
            if !process::owned_members(&job.identity)?.is_empty() {
                jobs::record_stop_request(
                    &job.dir(root),
                    job.spec.stop_grace_ms,
                    "background descendant stop",
                )?;
                process::terminate(&job.identity, Duration::from_millis(job.spec.stop_grace_ms))?;
                finalize_job(
                    root,
                    ledger,
                    job,
                    "failed",
                    Some(status),
                    "job parent exited with live background children; children terminated".into(),
                )?;
                return retain_tracking(root, job);
            }
            finalize_job(
                root,
                ledger,
                job,
                if status == 0 { "completed" } else { "failed" },
                Some(status),
                format!("process exited with status {status}"),
            )?;
            return retain_tracking(root, job);
        }
    } else if !process::matches(&job.identity) {
        if process::owned_members(&job.identity).is_ok_and(|members| !members.is_empty()) {
            jobs::record_stop_request(
                &job.dir(root),
                job.spec.stop_grace_ms,
                "adopted process identity lost",
            )?;
            process::terminate(&job.identity, Duration::from_millis(job.spec.stop_grace_ms))?;
        }
        host::hold(
            root,
            "adopted process ended without recoverable exit status",
        )?;
        finalize_job(
            root,
            ledger,
            job,
            "unknown",
            None,
            "adopted process ended or identity changed; success unverified".into(),
        )?;
        return retain_tracking(root, job);
    }
    if job.elapsed_ms() >= job.spec.wall_s.saturating_mul(1000) {
        stop_job(
            root,
            ledger,
            job,
            "wall_killed",
            "wall bound reached".into(),
        )?;
        return retain_tracking(root, job);
    }
    if !sample {
        return Ok(true);
    }
    if let Some(reason) = host_failure {
        stop_job(root, ledger, job, "resource_killed", reason.into())?;
        return retain_tracking(root, job);
    }
    if !process::matches(&job.identity) {
        host::hold(root, "running process lost identity")?;
        finalize_job(
            root,
            ledger,
            job,
            "unknown",
            None,
            "identity lost; no signal sent".into(),
        )?;
        return retain_tracking(root, job);
    }
    match jobs::tree_rss_kib(job.identity.pid) {
        Some(rss) => {
            job.peak_rss_kib = job.peak_rss_kib.max(rss);
            if rss as f64 > job.spec.rss_gib * 1024.0 * 1024.0 {
                stop_job(
                    root,
                    ledger,
                    job,
                    "resource_killed",
                    format!("RSS {rss} KiB exceeded declared limit"),
                )?;
                return retain_tracking(root, job);
            }
        }
        None => {
            stop_job(
                root,
                ledger,
                job,
                "resource_killed",
                "RSS observation unavailable".into(),
            )?;
            return retain_tracking(root, job);
        }
    }
    match job.spec.kill_criterion.kind {
        KillKind::Wall => {}
        KillKind::LogMatch => {
            let needle = job.spec.kill_criterion.value.clone();
            let fresh = new_log_bytes(job, root);
            if fresh.contains(&needle) {
                stop_job(
                    root,
                    ledger,
                    job,
                    "criterion_killed",
                    "literal log criterion observed".into(),
                )?;
                return retain_tracking(root, job);
            }
            let keep = needle.chars().count().saturating_sub(1);
            let n = fresh.chars().count();
            job.log_carry = fresh.chars().skip(n.saturating_sub(keep)).collect();
        }
        KillKind::Command => {
            // Bounded monitor child; an unbounded shell would stall all wall limits.
            let mut check = Command::new("/bin/sh")
                .args(["-c", &job.spec.kill_criterion.value])
                .env("UOR_RUNNER_PROCESS_TOKEN", &job.attempt.process_token)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
            let identity = process::capture(check.id(), &job.attempt.process_token).ok();
            let deadline = Instant::now() + Duration::from_secs(1);
            let ok = loop {
                if let Some(status) = check.try_wait()? {
                    break status.success();
                }
                if Instant::now() >= deadline {
                    if let Some(identity) = &identity {
                        let _ = process::terminate(identity, Duration::from_millis(50));
                    } else {
                        let _ = check.kill();
                    }
                    let _ = check.wait();
                    break false;
                }
                std::thread::sleep(Duration::from_millis(20));
            };
            if !ok {
                stop_job(
                    root,
                    ledger,
                    job,
                    "criterion_killed",
                    "command criterion failed or exceeded one second".into(),
                )?;
                return retain_tracking(root, job);
            }
        }
    }
    Ok(true)
}
fn blocked(root: &Path, reason: &str) -> Result<()> {
    let path = root.join("admission-blocked.json");
    if let Ok(bytes) = fs::read(&path) {
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if value["reason"].as_str() == Some(reason) {
                return Ok(());
            }
        }
    }
    crate::write_json_atomic(&path, &json!({"reason":reason,"at":crate::utc_now_iso()}))
}
fn admit_pending(
    config: &DaemonConfig,
    policy: Option<&HostPolicy>,
    running: &mut Vec<RunningJob>,
) -> Result<()> {
    let root = &config.root;
    if root.join("admissions-held.json").exists() || running.iter().any(|job| job.preflight_pending)
    {
        return Ok(());
    }
    if config.require_host_policy && policy.is_none() {
        return blocked(root, "valid host policy required");
    }
    let mut entries = fs::read_dir(jobs::queue_dir(root))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect::<Vec<_>>();
    entries.sort();
    for dir in entries {
        let spec = match jobs::read_spec(&dir) {
            Ok(s) => s,
            Err(error) => {
                blocked(root, &format!("invalid queued spec: {error}"))?;
                continue;
            }
        };
        if let Err(error) = spec.validate() {
            blocked(root, &error.to_string())?;
            continue;
        }
        let specs: Vec<_> = running.iter().map(|j| j.spec.clone()).collect();
        if let Err(reason) = admit(&Load::of(&specs), &spec) {
            blocked(root, &reason)?;
            continue;
        }
        if !config.require_host_policy {
            if let Err(error) = host::validate_test_job(root, &spec) {
                blocked(root, &error.to_string())?;
                continue;
            }
        }
        if let Some(policy) = policy {
            if let Err(error) = host::check_admission(root, policy, &spec, &specs) {
                blocked(root, &error.to_string())?;
                continue;
            }
        }
        let reservation =
            running
                .iter()
                .try_fold(spec.reserved_ms()?, |sum, j| -> Result<u64> {
                    sum.checked_add(j.spec.reserved_ms()?)
                        .ok_or_else(|| invalid("wall reservation overflow"))
                })?;
        if let Err(error) = crate::ledger::check_budget(&config.ledger_dir, reservation) {
            blocked(root, &error.to_string())?;
            continue;
        }
        let guard = match jobs::job_lock(root, &spec.id) {
            Ok(g) => g,
            Err(_) => continue,
        };
        if !dir.exists() {
            continue;
        }
        jobs::durable_rename(&dir, &jobs::running_dir(root).join(&spec.id))?;
        let launch_started = Instant::now();
        let launched = spawn_job(
            root,
            &spec,
            !config.require_host_policy,
            &config.ledger_dir,
            &specs,
        );
        drop(guard);
        match launched {
            Ok(job) => {
                let verifying = job.preflight_pending;
                running.push(job);
                let _ = fs::remove_file(root.join("admission-blocked.json"));
                if verifying {
                    break;
                }
            }
            Err(error) => {
                let running_dir = jobs::running_dir(root).join(&spec.id);
                if let Ok(attempt) = jobs::read_attempt(&running_dir) {
                    // Hashing/preflight and interrupted launches consume real
                    // time even when no payload ran. Charge their stable
                    // attempt once; UNKNOWN retains the shared reservation.
                    jobs::finalize(
                        root,
                        &spec,
                        &Finalization {
                            outcome: "unknown".into(),
                            exit_status: None,
                            reason: format!(
                                "launch/preflight interrupted before successful admission: {error}"
                            ),
                            peak_rss_kib: 0,
                            started_utc: attempt.started_utc,
                            elapsed_ms: launch_started.elapsed().as_millis() as u64,
                            measurement: crate::ledger::Measurement::Measured,
                        },
                        &config.ledger_dir,
                    )?;
                    host::hold(
                        root,
                        &format!(
                            "launch interrupted; shared attempt requires reconciliation: {error}"
                        ),
                    )?;
                } else {
                    recovery_required(root, &running_dir, &format!("launch interrupted: {error}"))?;
                }
                break;
            }
        }
    }
    Ok(())
}

fn acquire_host_lock() -> Result<fs::File> {
    let host_root = process::host_state_dir()?;
    fs::create_dir_all(&host_root)?;
    let file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(host_root.join("host-runner.lock"))?;
    file.try_lock()
        .map_err(|e| invalid(format!("host already has a production runner: {e}")))?;
    Ok(file)
}

pub fn run(config: &DaemonConfig) -> Result<()> {
    jobs::ensure_layout(&config.root)?;
    if !config.require_host_policy
        && !fs::canonicalize(&config.root)?.starts_with(fs::canonicalize(std::env::temp_dir())?)
    {
        return Err(invalid(
            "test mode requires a temporary isolated runner root",
        ));
    }
    let singleton = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(config.root.join("daemon.lock"))?;
    singleton
        .try_lock()
        .map_err(|e| invalid(format!("another daemon owns this host queue: {e}")))?;
    // An empty held queue with no policy does not need or mutate the real
    // host lock (also allows isolated missing-policy integration checks).
    let needs_host_lock = config.require_host_policy
        && (HostPolicy::load(&config.root).is_ok()
            || fs::read_dir(jobs::running_dir(&config.root))?
                .next()
                .is_some());
    let mut host_singleton = if needs_host_lock {
        Some(acquire_host_lock()?)
    } else {
        None
    };
    crate::write_json_atomic(
        &config.root.join("daemon.json"),
        &json!({"pid":std::process::id(),"started":crate::utc_now_iso(),"production":config.require_host_policy}),
    )?;
    let mut running = recover(&config.root, &config.ledger_dir, config.require_host_policy)?;
    let mut last_monitor = Instant::now()
        .checked_sub(config.monitor_interval)
        .unwrap_or_else(Instant::now);
    loop {
        if config.stop_file.as_ref().is_some_and(|p| p.exists()) {
            return Ok(());
        }
        let policy = if config.require_host_policy {
            match HostPolicy::load(&config.root) {
                Ok(p) => Some(p),
                Err(error) => {
                    blocked(&config.root, &error.to_string())?;
                    None
                }
            }
        } else {
            None
        };
        if config.require_host_policy && policy.is_some() && host_singleton.is_none() {
            host_singleton = Some(acquire_host_lock()?);
        }
        let sample = last_monitor.elapsed() >= config.monitor_interval;
        if sample {
            last_monitor = Instant::now();
        }
        let specs: Vec<_> = running.iter().map(|j| j.spec.clone()).collect();
        let host_failure = if sample && config.require_host_policy {
            match &policy {
                Some(p) => host::check_storage(p, &specs, false)
                    .and_then(|()| {
                        if host::memory_pressure()? >= 4 {
                            Err(invalid("critical host memory pressure"))
                        } else {
                            Ok(())
                        }
                    })
                    .and_then(|()| {
                        let actual: u64 = running
                            .iter()
                            .map(|j| jobs::tree_rss_kib(j.identity.pid).unwrap_or(0))
                            .sum();
                        if actual as f64 > p.max_rss_gib * 1024.0 * 1024.0 {
                            Err(invalid("aggregate observed RSS exceeds host ceiling"))
                        } else {
                            Ok(())
                        }
                    })
                    .and_then(|()| crate::ledger::check_budget(&config.ledger_dir, 0).map(|_| ()))
                    .err()
                    .map(|e| e.to_string()),
                None => Some("host policy unavailable during run".into()),
            }
        } else {
            None
        };
        if let Some(reason) = &host_failure {
            host::hold(&config.root, reason)?;
        }
        let mut still = vec![];
        for mut job in running {
            match monitor(
                &config.root,
                &config.ledger_dir,
                &mut job,
                sample,
                host_failure.as_deref(),
            ) {
                Ok(true) => still.push(job),
                Ok(false) => {}
                Err(error) => {
                    host::hold(
                        &config.root,
                        &format!("job finalization/monitor needs reconciliation: {error}"),
                    )?;
                    still.push(job);
                }
            }
        }
        running = still;
        if let Err(error) = advance_preflight(config, &mut running) {
            host::hold(
                &config.root,
                &format!("input verification needs reconciliation: {error}"),
            )?;
        }
        admit_pending(config, policy.as_ref(), &mut running)?;
        std::thread::sleep(config.poll_interval.min(Duration::from_secs(1)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::{self, Measurement};

    struct RunningFixture {
        job: RunningJob,
        identity: Identity,
    }

    impl Drop for RunningFixture {
        fn drop(&mut self) {
            // A failing assertion must not strand the fixture's FIFO anchor.
            if process::matches(&self.identity) {
                let _ = process::terminate(&self.identity, Duration::from_millis(20));
            }
            if let Some(child) = self.job.child.as_mut() {
                let _ = child.try_wait();
            }
        }
    }

    fn fixture(label: &str) -> (PathBuf, PathBuf, JobSpec, Attempt) {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "uor-preflight-{}-{unique}-{label}",
            std::process::id()
        ));
        jobs::ensure_layout(&root).unwrap();
        let ledger = root.join("ledger");
        fs::create_dir(&ledger).unwrap();
        ledger::initialize_empty(
            &ledger,
            if label == "budget" { 1 } else { 100_000 },
            "fixture",
        )
        .unwrap();
        let input = root.join("input.json");
        fs::write(&input,serde_json::to_vec(&json!({"id":format!("preflight-{unique}-{label}"),"lab":"test",
            "cwd":root,"argv":["/usr/bin/true"],"threads":1,"rss_gib":0.1,"wall_s":5,"kill_criterion":{"kind":"wall"}})).unwrap()).unwrap();
        let id = jobs::submit(&root, &input).unwrap();
        let dir = jobs::running_dir(&root).join(&id);
        jobs::durable_rename(&jobs::queue_dir(&root).join(&id), &dir).unwrap();
        let spec = jobs::read_spec(&dir).unwrap();
        let _guard = jobs::job_lock(&root, &id).unwrap();
        let attempt = jobs::prepare_attempt(&dir, &id).unwrap();
        (root, ledger, spec, attempt)
    }

    fn verification_fixture(
        label: &str,
        verifier_body: &str,
        wall_s: u64,
    ) -> (PathBuf, PathBuf, RunningFixture, PathBuf) {
        use std::os::unix::fs::PermissionsExt;
        let (root, ledger, mut spec, _) = fixture(label);
        let dir = jobs::running_dir(&root).join(&spec.id);
        fs::remove_file(dir.join("attempt.json")).unwrap();
        let marker = root.join("payload-marker");
        spec.argv = vec![
            "/bin/sh".into(),
            "-c".into(),
            "printf x >> \"$1\"".into(),
            "payload".into(),
            marker.to_string_lossy().into_owned(),
        ];
        spec.wall_s = wall_s;
        spec.stop_grace_ms = 20;
        fs::write(dir.join("spec.json"), serde_json::to_vec(&spec).unwrap()).unwrap();
        let verifier = root.join("verifier-fixture");
        fs::write(&verifier, format!("#!/bin/sh\n{verifier_body}\n")).unwrap();
        fs::set_permissions(&verifier, fs::Permissions::from_mode(0o700)).unwrap();
        let job =
            spawn_job_with_verifier(&root, &spec, true, &ledger, &[], Some(&verifier)).unwrap();
        let identity = job.identity.clone();
        (root, ledger, RunningFixture { job, identity }, marker)
    }

    fn test_config(root: &Path, ledger: &Path) -> DaemonConfig {
        DaemonConfig {
            root: root.to_path_buf(),
            ledger_dir: ledger.to_path_buf(),
            poll_interval: Duration::from_millis(20),
            monitor_interval: Duration::from_millis(20),
            stop_file: None,
            require_host_policy: false,
        }
    }

    #[test]
    fn pending_verifier_keeps_partner_monitored_and_opens_payload_once() {
        let (root, ledger, mut pending, marker) =
            verification_fixture("pending-partner", "/bin/sleep 2; exit 0", 8);
        let dir = pending.job.dir(&root);
        let mut peer_spec = pending.job.spec.clone();
        peer_spec.id.push_str("-peer");
        peer_spec.argv = vec!["/bin/sleep".into(), "10".into()];
        peer_spec.wall_s = 1;
        let peer_input = root.join("peer-spec.json");
        fs::write(&peer_input, serde_json::to_vec(&peer_spec).unwrap()).unwrap();
        jobs::submit(&root, &peer_input).unwrap();
        jobs::durable_rename(
            &jobs::queue_dir(&root).join(&peer_spec.id),
            &jobs::running_dir(&root).join(&peer_spec.id),
        )
        .unwrap();
        let peer = spawn_job(&root, &peer_spec, true, &ledger, &[]).unwrap();
        let mut peer = RunningFixture {
            identity: peer.identity.clone(),
            job: peer,
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        while !dir.join("verification.exit").exists() && Instant::now() < deadline {
            let _ = monitor(&root, &ledger, &mut peer.job, true, None).unwrap();
            assert!(monitor(&root, &ledger, &mut pending.job, true, None).unwrap());
            assert!(!marker.exists());
            std::thread::sleep(Duration::from_millis(20));
        }
        let peer_exit: serde_json::Value = serde_json::from_slice(
            &fs::read(jobs::done_dir(&root).join(&peer_spec.id).join("exit.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(peer_exit["outcome"], "wall_killed");
        assert!(process::owned_members(&peer.identity).unwrap().is_empty());
        assert_eq!(fs::read(dir.join("verification.exit")).unwrap(), b"0\n");
        assert!(!dir.join("payload.go").exists());
        assert!(!marker.exists());
        let config = test_config(&root, &ledger);
        advance_preflight(&config, std::slice::from_mut(&mut pending.job)).unwrap();
        assert!(!pending.job.preflight_pending);
        advance_preflight(&config, std::slice::from_mut(&mut pending.job)).unwrap();
        while monitor(&root, &ledger, &mut pending.job, true, None).unwrap()
            && Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(fs::read(&marker).unwrap(), b"x");
        assert!(process::owned_members(&pending.identity)
            .unwrap()
            .is_empty());
        drop(pending);
        drop(peer);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_or_timed_out_verifier_never_opens_payload_and_preserves_receipt() {
        for (label, body, wall, outcome) in [
            ("verify-failed", "exit 7", 5, "failed"),
            ("verify-timeout", "/bin/sleep 20", 1, "wall_killed"),
        ] {
            let (root, ledger, mut pending, marker) = verification_fixture(label, body, wall);
            let id = pending.job.spec.id.clone();
            let deadline = Instant::now() + Duration::from_secs(5);
            while monitor(&root, &ledger, &mut pending.job, true, None).unwrap()
                && Instant::now() < deadline
            {
                std::thread::sleep(Duration::from_millis(20));
            }
            let done = jobs::done_dir(&root).join(id);
            let receipt: serde_json::Value =
                serde_json::from_slice(&fs::read(done.join("exit.json")).unwrap()).unwrap();
            assert_eq!(receipt["outcome"], outcome);
            assert!(receipt["reason"]
                .as_str()
                .unwrap()
                .contains("input verification"));
            assert!(!done.join("payload.go").exists());
            assert!(!marker.exists());
            assert!(process::owned_members(&pending.identity)
                .unwrap()
                .is_empty());
            drop(pending);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn late_admission_keeps_partner_and_combined_budget() {
        let (root, _, spec, _) = fixture("two-lane-preflight");
        let mut peer = spec.clone();
        peer.id = "running-peer".into();
        peer.wall_s = 17;
        let expected = spec.reserved_ms().unwrap() + peer.reserved_ms().unwrap();
        recheck_resources(
            &spec,
            &[peer],
            |running| {
                if running.len() != 1 || running[0].id != "running-peer" {
                    return Err(invalid("running partner lost at late admission"));
                }
                Ok(())
            },
            |reserved| {
                assert_eq!(reserved, expected);
                Ok(())
            },
        )
        .unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_hash_late_policy_and_budget_have_positive_unstarted_proofs() {
        for failure in ["hash", "policy", "budget"] {
            let (root, ledger, spec, attempt) = fixture(failure);
            let dir = jobs::running_dir(&root).join(&spec.id);
            let guard = jobs::job_lock(&root, &spec.id).unwrap();
            let data = root.join("data");
            fs::write(&data, b"changed").unwrap();
            let result = preflight_or_record_failure(
                &root,
                &spec,
                &attempt,
                Instant::now(),
                || match failure {
                    "hash" => crate::spec::FileIdentity {
                        path: data.clone(),
                        sha256: crate::coord::digest(b"original"),
                    }
                    .verify(),
                    "policy" => HostPolicy::load(&root).map(|_| ()),
                    _ => ledger::check_budget(&ledger, spec.reserved_ms()?).map(|_| ()),
                },
            );
            assert!(result.is_err());
            let proof = jobs::validate_pre_spawn_failure(&root, &dir, &spec, &attempt).unwrap();
            assert!(!dir.join("process.json").exists());
            drop(guard);
            jobs::finalize(
                &root,
                &spec,
                &Finalization {
                    outcome: "unknown".into(),
                    exit_status: None,
                    reason: "failed preflight fixture".into(),
                    peak_rss_kib: 0,
                    started_utc: attempt.started_utc,
                    elapsed_ms: proof.elapsed_ms,
                    measurement: Measurement::Measured,
                },
                &ledger,
            )
            .unwrap();
            let done = jobs::done_dir(&root).join(&spec.id);
            let original = fs::read(dir.join("exit.json")).unwrap();
            assert!(dir.is_dir());
            assert!(!done.exists());
            // A marker does not override contradictory launch evidence.
            fs::write(dir.join("launch.go"), b"contradiction").unwrap();
            assert!(jobs::reconcile_stopped(&root, &spec.id, &ledger).is_err());
            fs::remove_file(dir.join("launch.go")).unwrap();
            let before = ledger::rebuild(&ledger).unwrap().cumulative_ms;
            let receipt = jobs::reconcile_stopped(&root, &spec.id, &ledger).unwrap();
            let value: serde_json::Value =
                serde_json::from_slice(&fs::read(receipt).unwrap()).unwrap();
            assert_eq!(value["process_state"], "confirmed_not_started");
            assert_eq!(value["additional_charged_ms"], 0);
            assert_eq!(value["measurement"], "measured");
            assert_eq!(fs::read(done.join("exit.json")).unwrap(), original);
            assert_eq!(ledger::rebuild(&ledger).unwrap().cumulative_ms, before);
            jobs::reconcile_stopped(&root, &spec.id, &ledger).unwrap();
            fs::remove_file(done.join("preflight-failure.json")).unwrap();
            assert!(jobs::reconcile_stopped(&root, &spec.id, &ledger).is_err());
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn unknown_monitor_preserves_paths_child_and_charge_until_stopped_reconciliation() {
        for reverified in [false, true] {
            let (root, ledger, mut spec, _) = fixture(if reverified {
                "reverified"
            } else {
                "unknown-live"
            });
            let dir = jobs::running_dir(&root).join(&spec.id);
            // This fixture creates the actual attempt through the gated spawner.
            fs::remove_file(dir.join("attempt.json")).unwrap();
            spec.argv = vec![
                "/bin/sleep".into(),
                if reverified { "10" } else { "2" }.into(),
            ];
            spec.stop_grace_ms = 20;
            fs::write(dir.join("spec.json"), serde_json::to_vec(&spec).unwrap()).unwrap();
            let spawned = spawn_job(&root, &spec, true, &ledger, &[]).unwrap();
            let mut fixture = RunningFixture {
                identity: spawned.identity.clone(),
                job: spawned,
            };
            let job = &mut fixture.job;
            let original_identity = job.identity.clone();
            let child_pid = job.child.as_ref().unwrap().id();
            // Inject a failed identity observation; a malformed boot token can
            // neither establish reboot nor authorize a stop of the live group.
            job.identity.boot = "unverified observation".into();
            assert!(monitor(&root, &ledger, job, true, None).unwrap());
            let original_exit = fs::read(dir.join("exit.json")).unwrap();
            let charged = ledger::rebuild(&ledger).unwrap().cumulative_ms;
            assert!(dir.join("supervisor-hold.fifo").exists());
            assert_eq!(job.child.as_ref().unwrap().id(), child_pid);
            assert!(root.join("admissions-held.json").exists());
            assert!(monitor(&root, &ledger, job, true, None).unwrap());
            assert_eq!(job.child.as_ref().unwrap().id(), child_pid);
            assert_eq!(fs::read(dir.join("exit.json")).unwrap(), original_exit);
            assert_eq!(ledger::rebuild(&ledger).unwrap().cumulative_ms, charged);
            if reverified {
                // Recovery of the full identity must enforce a bounded stop,
                // not skip wall/resource enforcement forever because exit.json exists.
                job.identity = original_identity;
                assert!(monitor(&root, &ledger, job, true, None).unwrap());
                assert!(process::owned_members(&job.identity).unwrap().is_empty());
            } else {
                let deadline = Instant::now() + Duration::from_secs(3);
                while !dir.join("payload.exit").exists() && Instant::now() < deadline {
                    std::thread::sleep(Duration::from_millis(10));
                }
                assert_eq!(fs::read(dir.join("payload.exit")).unwrap(), b"0\n");
                assert!(job.child.as_mut().unwrap().try_wait().unwrap().is_none());
                // The known fixture owner stops its supervisor only after the
                // original payload/FIFO paths have demonstrably survived.
                process::terminate(&original_identity, Duration::from_millis(20)).unwrap();
            }
            let deadline = Instant::now() + Duration::from_secs(2);
            while job.child.is_some() && Instant::now() < deadline {
                assert!(monitor(&root, &ledger, job, false, None).unwrap());
                std::thread::sleep(Duration::from_millis(10));
            }
            assert!(job.child.is_none());
            assert!(dir.is_dir());
            let receipt = jobs::reconcile_stopped(&root, &spec.id, &ledger).unwrap();
            let done = jobs::done_dir(&root).join(&spec.id);
            assert!(!dir.exists());
            assert_eq!(fs::read(done.join("exit.json")).unwrap(), original_exit);
            let exit: serde_json::Value = serde_json::from_slice(&original_exit).unwrap();
            assert_eq!(exit["outcome"], "unknown");
            assert!(exit["exit_status"].is_null());
            let after = ledger::rebuild(&ledger).unwrap().cumulative_ms;
            let proof = fs::read(&receipt).unwrap();
            jobs::reconcile_stopped(&root, &spec.id, &ledger).unwrap();
            assert_eq!(ledger::rebuild(&ledger).unwrap().cumulative_ms, after);
            assert_eq!(fs::read(receipt).unwrap(), proof);
            assert!(!monitor(&root, &ledger, job, true, None).unwrap());
            drop(fixture);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn recovered_wall_clock_is_estimated_in_both_exit_and_charge() {
        let (root, ledger, spec, attempt) = fixture("recover-estimate");
        let dir = jobs::running_dir(&root).join(&spec.id);
        let identity = Identity {
            pid: u32::MAX,
            pgid: u32::MAX,
            started: "fixture".into(),
            boot: "legacy-unverified-fixture".into(),
            host: process::host_id().unwrap(),
            token: attempt.process_token,
            supervisor_started: "fixture".into(),
        };
        fs::write(
            dir.join("process.json"),
            serde_json::to_vec(&identity).unwrap(),
        )
        .unwrap();
        assert_eq!(recover(&root, &ledger, true).unwrap().len(), 1);
        assert!(dir.join("exit.json").exists());
        jobs::reconcile_stopped(&root, &spec.id, &ledger).unwrap();
        let receipt: serde_json::Value = serde_json::from_slice(
            &fs::read(jobs::done_dir(&root).join(&spec.id).join("exit.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(receipt["measurement"], "estimated");
        for entry in fs::read_dir(&ledger).unwrap().map(|e| e.unwrap()) {
            if entry
                .file_name()
                .to_string_lossy()
                .starts_with("charge-v2-")
            {
                let charge: ledger::ChargeRecord =
                    serde_json::from_slice(&fs::read(entry.path()).unwrap()).unwrap();
                assert_eq!(
                    charge.accounting.unwrap().measurement,
                    Measurement::Estimated
                );
            }
        }
        fs::remove_dir_all(root).unwrap();
    }
}
