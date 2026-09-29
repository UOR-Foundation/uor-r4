//! Foreground daemon loop. launchd owns restarts (KeepAlive); this loop
//! polls the queue, admits jobs under the shared limits, enforces kill
//! criteria, and finalizes finished jobs. It is crash-consistent: on startup
//! a `running/<id>` whose pid is dead is finalized as `error`.

use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::admission::{admit, Load};
use crate::jobs::{self, ensure_layout, Finalization};
use crate::spec::{JobSpec, KillKind};
use crate::{utc_now_iso, Result};

pub struct DaemonConfig {
    pub root: PathBuf,
    pub ledger_dir: PathBuf,
    /// Queue poll interval, about 1 s in production.
    pub poll_interval: Duration,
    /// RSS/criterion sample interval, about 2 s in production.
    pub monitor_interval: Duration,
    /// When this path exists the loop exits cleanly (test hook).
    pub stop_file: Option<PathBuf>,
}

struct RunningJob {
    spec: JobSpec,
    child: Option<Child>,
    pid: u32,
    started: Instant,
    started_utc: String,
    peak_rss_kib: u64,
    log_offsets: [u64; 2],
    log_carry: String,
}

impl RunningJob {
    fn elapsed_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }

    fn dir(&self, root: &Path) -> PathBuf {
        jobs::running_dir(root).join(&self.spec.id)
    }
}

fn minimal_env() -> Vec<(String, String)> {
    let mut env = Vec::new();
    for key in ["PATH", "HOME", "TMPDIR", "USER", "LANG"] {
        if let Some(value) = std::env::var_os(key) {
            env.push((key.to_string(), value.to_string_lossy().into_owned()));
        }
    }
    if !env.iter().any(|(key, _)| key == "PATH") {
        env.push((
            "PATH".to_string(),
            "/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin".to_string(),
        ));
    }
    env
}

fn spawn_job(root: &Path, spec: &JobSpec) -> Result<RunningJob> {
    let dir = jobs::running_dir(root).join(&spec.id);
    let stdout = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dir.join("stdout.log"))?;
    let stderr = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dir.join("stderr.log"))?;
    let mut command = Command::new(&spec.argv[0]);
    command
        .args(&spec.argv[1..])
        .current_dir(&spec.cwd)
        .env_clear()
        .envs(minimal_env());
    if let Some(extra) = &spec.env {
        command.envs(extra);
    }
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()?;
    let pid = child.id();
    fs::write(dir.join("pid"), format!("{pid}\n"))?;
    let started_utc = utc_now_iso();
    fs::write(dir.join("started_utc"), format!("{started_utc}\n"))?;
    let started_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    fs::write(dir.join("started_ms"), format!("{started_ms}\n"))?;
    Ok(RunningJob {
        spec: spec.clone(),
        child: Some(child),
        pid,
        started: Instant::now(),
        started_utc,
        peak_rss_kib: 0,
        log_offsets: [0, 0],
        log_carry: String::new(),
    })
}

fn finalize_job(
    root: &Path,
    ledger_dir: &Path,
    job: &mut RunningJob,
    outcome: &str,
    exit_status: Option<i64>,
    reason: String,
) -> Result<()> {
    jobs::finalize(
        root,
        &job.spec,
        &Finalization {
            outcome: outcome.to_string(),
            exit_status,
            reason,
            peak_rss_kib: job.peak_rss_kib,
            started_utc: job.started_utc.clone(),
            elapsed_ms: job.elapsed_ms(),
        },
        ledger_dir,
    )
}

fn exit_status_code(status: &std::process::ExitStatus) -> i64 {
    match status.code() {
        Some(code) => code as i64,
        None => {
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                -(status.signal().unwrap_or(0) as i64)
            }
            #[cfg(not(unix))]
            {
                -1
            }
        }
    }
}

/// Kill the process tree, reap, and finalize.
fn enforce_kill(
    root: &Path,
    ledger_dir: &Path,
    job: &mut RunningJob,
    outcome: &str,
    reason: String,
) -> Result<()> {
    jobs::kill_tree(job.pid, Duration::from_secs(1));
    let exit_status = match job.child.as_mut() {
        Some(child) => child.wait().ok().map(|s| exit_status_code(&s)),
        None => None,
    };
    finalize_job(root, ledger_dir, job, outcome, exit_status, reason)
}

/// Startup recovery: any `running/<id>` with a dead pid is finalized as
/// `error`; a live pid is adopted and monitored to completion.
fn recover_or_adopt(root: &Path, ledger_dir: &Path) -> Result<Vec<RunningJob>> {
    let mut running = Vec::new();
    let dir = jobs::running_dir(root);
    if !dir.is_dir() {
        return Ok(running);
    }
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let job_dir = entry.path();
        let Some(spec) = fs::read(job_dir.join("spec.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<JobSpec>(&bytes).ok())
        else {
            continue;
        };
        let pid: Option<u32> = fs::read_to_string(job_dir.join("pid"))
            .ok()
            .and_then(|text| text.trim().parse().ok());
        let started_utc = fs::read_to_string(job_dir.join("started_utc"))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| utc_now_iso());
        let started_ms: Option<u64> = fs::read_to_string(job_dir.join("started_ms"))
            .ok()
            .and_then(|s| s.trim().parse().ok());
        let elapsed_so_far = started_ms
            .map(|start| {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                now.saturating_sub(start)
            })
            .unwrap_or(0);
        match pid {
            Some(pid) if jobs::pid_alive(pid) => {
                running.push(RunningJob {
                    spec,
                    child: None,
                    pid,
                    started: Instant::now()
                        - Duration::from_millis(elapsed_so_far.min(u32::MAX as u64)),
                    started_utc,
                    peak_rss_kib: 0,
                    log_offsets: [0, 0],
                    log_carry: String::new(),
                });
            }
            _ => {
                let mut job = RunningJob {
                    spec,
                    child: None,
                    pid: pid.unwrap_or(0),
                    started: Instant::now()
                        - Duration::from_millis(elapsed_so_far.min(u32::MAX as u64)),
                    started_utc,
                    peak_rss_kib: 0,
                    log_offsets: [0, 0],
                    log_carry: String::new(),
                };
                finalize_job(
                    root,
                    ledger_dir,
                    &mut job,
                    "error",
                    None,
                    "daemon restart found the job's pid dead".to_string(),
                )?;
            }
        }
    }
    Ok(running)
}

fn scan_new_log_bytes(job: &mut RunningJob, root: &Path) -> String {
    let dir = job.dir(root);
    let mut fresh = std::mem::take(&mut job.log_carry);
    for (index, name) in ["stdout.log", "stderr.log"].iter().enumerate() {
        let path = dir.join(name);
        let Ok(mut file) = fs::File::open(&path) else {
            continue;
        };
        let len = file.metadata().map(|m| m.len()).unwrap_or(0);
        if len < job.log_offsets[index] {
            job.log_offsets[index] = 0;
        }
        if file.seek(SeekFrom::Start(job.log_offsets[index])).is_err() {
            continue;
        }
        let mut buf = String::new();
        if file.read_to_string(&mut buf).is_err() {
            continue;
        }
        job.log_offsets[index] = len;
        fresh.push_str(&buf);
    }
    fresh
}

fn monitor_job(
    root: &Path,
    ledger_dir: &Path,
    job: &mut RunningJob,
    sample_resources: bool,
) -> Result<bool> {
    // Returns Ok(true) while the job is still running.
    if !job.dir(root).is_dir() {
        // Finalized externally (e.g. `lab-runner cancel`).
        return Ok(false);
    }
    let cancel_marker = job.dir(root).join("cancel.json");
    if cancel_marker.is_file() {
        let reason = fs::read_to_string(&cancel_marker)
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
            .and_then(|v| v.get("reason").and_then(|r| r.as_str().map(String::from)))
            .unwrap_or_else(|| "cancelled (marker present)".to_string());
        enforce_kill(root, ledger_dir, job, "cancelled", reason)?;
        return Ok(false);
    }
    if let Some(child) = job.child.as_mut() {
        match child.try_wait() {
            Ok(Some(status)) => {
                let code = exit_status_code(&status);
                finalize_job(
                    root,
                    ledger_dir,
                    job,
                    "completed",
                    Some(code),
                    format!("process exited with status {code}"),
                )?;
                return Ok(false);
            }
            Ok(None) => {}
            Err(error) => {
                finalize_job(
                    root,
                    ledger_dir,
                    job,
                    "error",
                    None,
                    format!("could not poll child process: {error}"),
                )?;
                return Ok(false);
            }
        }
    } else if !jobs::pid_alive(job.pid) {
        finalize_job(
            root,
            ledger_dir,
            job,
            "completed",
            None,
            "adopted process exited (exit status unavailable after restart)".to_string(),
        )?;
        return Ok(false);
    }

    if job.elapsed_ms() >= job.spec.wall_s * 1000 {
        enforce_kill(
            root,
            ledger_dir,
            job,
            "wall_killed",
            format!("wall clock limit of {} s reached", job.spec.wall_s),
        )?;
        return Ok(false);
    }

    if !sample_resources {
        return Ok(true);
    }

    if let Some(rss) = jobs::tree_rss_kib(job.pid) {
        job.peak_rss_kib = job.peak_rss_kib.max(rss);
    }

    match job.spec.kill_criterion.kind {
        KillKind::Wall => {}
        KillKind::LogMatch => {
            let needle = job.spec.kill_criterion.value.clone();
            let fresh = scan_new_log_bytes(job, root);
            if fresh.contains(&needle) {
                enforce_kill(
                    root,
                    ledger_dir,
                    job,
                    "criterion_killed",
                    format!("log_match criterion {:?} observed", needle),
                )?;
                return Ok(false);
            }
            let keep = needle.len().saturating_sub(1);
            let total = fresh.chars().count();
            job.log_carry = fresh.chars().skip(total.saturating_sub(keep)).collect();
        }
        KillKind::Command => {
            let check = job.spec.kill_criterion.value.clone();
            let status = Command::new("sh")
                .arg("-c")
                .arg(&check)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            match status {
                Ok(status) if status.success() => {}
                Ok(status) => {
                    enforce_kill(
                        root,
                        ledger_dir,
                        job,
                        "criterion_killed",
                        format!("command criterion exited with {status}"),
                    )?;
                    return Ok(false);
                }
                Err(error) => {
                    enforce_kill(
                        root,
                        ledger_dir,
                        job,
                        "error",
                        format!("command criterion could not run: {error}"),
                    )?;
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}

fn admit_pending(root: &Path, ledger_dir: &Path, running: &mut Vec<RunningJob>) -> Result<()> {
    let queue = jobs::queue_dir(root);
    let mut entries: Vec<PathBuf> = Vec::new();
    if queue.is_dir() {
        for entry in fs::read_dir(&queue)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                entries.push(entry.path());
            }
        }
    }
    entries.sort();
    for dir in entries {
        let spec: JobSpec = match fs::read(dir.join("spec.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        {
            Some(spec) => spec,
            None => continue,
        };
        let load = Load::of(
            &running
                .iter()
                .map(|job| job.spec.clone())
                .collect::<Vec<_>>(),
        );
        if admit(&load, &spec).is_err() {
            continue;
        }
        let target = jobs::running_dir(root).join(&spec.id);
        if fs::rename(&dir, &target).is_err() {
            continue;
        }
        match spawn_job(root, &spec) {
            Ok(job) => running.push(job),
            Err(error) => {
                let mut stub = RunningJob {
                    spec: spec.clone(),
                    child: None,
                    pid: 0,
                    started: Instant::now(),
                    started_utc: utc_now_iso(),
                    peak_rss_kib: 0,
                    log_offsets: [0, 0],
                    log_carry: String::new(),
                };
                let _ = finalize_job(
                    root,
                    ledger_dir,
                    &mut stub,
                    "error",
                    None,
                    format!("spawn failed: {error}"),
                );
            }
        }
    }
    Ok(())
}

/// Run the daemon loop until `stop_file` appears (or forever).
pub fn run(config: &DaemonConfig) -> Result<()> {
    ensure_layout(&config.root)?;
    let mut running = recover_or_adopt(&config.root, &config.ledger_dir)?;
    let mut last_monitor = Instant::now() - config.monitor_interval;
    loop {
        if let Some(stop) = &config.stop_file {
            if stop.exists() {
                return Ok(());
            }
        }
        admit_pending(&config.root, &config.ledger_dir, &mut running)?;
        let sample = last_monitor.elapsed() >= config.monitor_interval;
        if sample {
            last_monitor = Instant::now();
        }
        let mut still = Vec::with_capacity(running.len());
        for mut job in running {
            match monitor_job(&config.root, &config.ledger_dir, &mut job, sample) {
                Ok(true) => still.push(job),
                Ok(false) => {}
                Err(error) => {
                    eprintln!(
                        "lab-runner daemon: monitor error for {}: {error}",
                        job.spec.id
                    );
                    still.push(job);
                }
            }
        }
        running = still;
        std::thread::sleep(config.poll_interval);
    }
}
