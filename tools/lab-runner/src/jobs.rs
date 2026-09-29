//! Queue operations shared by the CLI and the daemon: submit, status, tail,
//! cancel, and the finalize path that writes `exit.json`, moves the job to
//! `done/` and charges the ledger.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::spec::JobSpec;
use crate::{invalid, ledger, utc_now_iso, Result};

pub const EXIT_SCHEMA: &str = "uor-r4.lab-runner-exit/1";

pub fn queue_dir(root: &Path) -> PathBuf {
    root.join("queue")
}
pub fn running_dir(root: &Path) -> PathBuf {
    root.join("running")
}
pub fn done_dir(root: &Path) -> PathBuf {
    root.join("done")
}

pub fn ensure_layout(root: &Path) -> Result<()> {
    fs::create_dir_all(queue_dir(root))?;
    fs::create_dir_all(running_dir(root))?;
    fs::create_dir_all(done_dir(root))?;
    Ok(())
}

/// Locate a job id in any of the three queue directories.
pub fn locate(root: &Path, id: &str) -> Option<(&'static str, PathBuf)> {
    for (stage, dir) in [
        ("queue", queue_dir(root)),
        ("running", running_dir(root)),
        ("done", done_dir(root)),
    ] {
        let path = dir.join(id);
        if path.is_dir() {
            return Some((stage, path));
        }
    }
    None
}

/// Validate `spec_path` and atomically place it in the queue. Fails if the
/// id already exists in any stage.
pub fn submit(root: &Path, spec_path: &Path) -> Result<String> {
    let spec = JobSpec::parse(&fs::read(spec_path)?)?;
    ensure_layout(root)?;
    if locate(root, &spec.id).is_some() {
        return Err(invalid(format!("job id {} already exists", spec.id)));
    }
    let dir = queue_dir(root).join(&spec.id);
    // create_dir is the atomic claim on the id.
    fs::create_dir(&dir)?;
    let write_result =
        crate::atomic_write(&dir.join("spec.json"), &serde_json::to_vec_pretty(&spec)?);
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&dir);
        return Err(error);
    }
    Ok(spec.id)
}

fn read_spec(dir: &Path) -> Option<JobSpec> {
    let bytes = fs::read(dir.join("spec.json")).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn read_pid(dir: &Path) -> Option<u32> {
    fs::read_to_string(dir.join("pid"))
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// `ps -o pid= -p <pid>` liveness check (stdlib + ps only).
pub fn pid_alive(pid: u32) -> bool {
    Command::new("ps")
        .args(["-o", "pid=", "-p"])
        .arg(pid.to_string())
        .output()
        .map(|output| {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout)
                    .split_whitespace()
                    .any(|field| field == pid.to_string())
        })
        .unwrap_or(false)
}

/// One `ps -Ao pid=,ppid=,rss=` snapshot: (pid, ppid, rss_kib) rows.
fn ps_table() -> Vec<(u32, u32, u64)> {
    let output = match Command::new("ps").args(["-Ao", "pid=,ppid=,rss="]).output() {
        Ok(output) if output.status.success() => output,
        _ => return Vec::new(),
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let pid = fields.next()?.parse().ok()?;
            let ppid = fields.next()?.parse().ok()?;
            let rss = fields.next()?.parse().ok()?;
            Some((pid, ppid, rss))
        })
        .collect()
}

/// All pids in `root_pid`'s process tree, root first.
pub fn process_tree(root_pid: u32) -> Vec<u32> {
    let table = ps_table();
    let mut tree = vec![root_pid];
    let mut cursor = 0;
    while cursor < tree.len() {
        let parent = tree[cursor];
        for (pid, ppid, _) in &table {
            if *ppid == parent && !tree.contains(pid) {
                tree.push(*pid);
            }
        }
        cursor += 1;
    }
    tree
}

/// Total RSS in KiB of `root_pid` and its descendants; `None` when the root
/// pid is gone.
pub fn tree_rss_kib(root_pid: u32) -> Option<u64> {
    let table = ps_table();
    if !table.iter().any(|(pid, _, _)| *pid == root_pid) {
        return None;
    }
    let tree = process_tree(root_pid);
    Some(
        table
            .iter()
            .filter(|(pid, _, _)| tree.contains(pid))
            .map(|(_, _, rss)| rss)
            .sum(),
    )
}

fn kill_tree_signal(root_pid: u32, signal: &str) {
    let tree = process_tree(root_pid);
    if tree.is_empty() {
        return;
    }
    let mut command = Command::new("/bin/kill");
    command.arg(format!("-{signal}"));
    for pid in tree {
        command.arg(pid.to_string());
    }
    let _ = command.status();
}

/// SIGTERM the tree, wait `grace`, then SIGKILL whatever remains.
pub fn kill_tree(root_pid: u32, grace: Duration) {
    kill_tree_signal(root_pid, "TERM");
    std::thread::sleep(grace);
    kill_tree_signal(root_pid, "KILL");
}

pub struct Finalization {
    pub outcome: String,
    pub exit_status: Option<i64>,
    pub reason: String,
    pub peak_rss_kib: u64,
    pub started_utc: String,
    pub elapsed_ms: u64,
}

/// Write `exit.json` inside the job's current directory, rename the directory
/// to `done/<id>/`, then charge the ledger. Safe against a concurrent
/// finalizer: if the job directory has already moved, this is a no-op.
pub fn finalize(root: &Path, spec: &JobSpec, fin: &Finalization, ledger_dir: &Path) -> Result<()> {
    let running = running_dir(root).join(&spec.id);
    if !running.is_dir() {
        return Ok(());
    }
    let exit = json!({
        "schema": EXIT_SCHEMA,
        "id": spec.id,
        "lab": spec.lab,
        "argv": spec.argv,
        "started_utc": fin.started_utc,
        "ended_utc": utc_now_iso(),
        "elapsed_ms": fin.elapsed_ms,
        "wall_s": spec.wall_s,
        "outcome": fin.outcome,
        "exit_status": fin.exit_status,
        "peak_rss_kib": fin.peak_rss_kib,
        "reason": fin.reason,
    });
    match crate::write_json_atomic(&running.join("exit.json"), &exit) {
        Ok(()) => {}
        Err(crate::RunnerError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(());
        }
        Err(error) => return Err(error),
    }
    let done = done_dir(root).join(&spec.id);
    // A concurrent finalizer may win the rename after the exit write; treat
    // every NotFound from here on as "the other finalizer completed first".
    match fs::rename(&running, &done) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    }
    ledger::record_charge(
        ledger_dir,
        &spec.lab,
        &spec.id,
        fin.elapsed_ms,
        spec.wall_s,
        &fin.outcome,
    )?;
    Ok(())
}

/// JSON summary of one job directory.
fn summarize(root: &Path, stage: &str, dir: &Path) -> Value {
    let spec = read_spec(dir);
    let pid = read_pid(dir);
    let alive = pid.map(pid_alive);
    json!({
        "id": dir.file_name().map(|n| n.to_string_lossy().into_owned()),
        "stage": stage,
        "lab": spec.as_ref().map(|s| s.lab.clone()),
        "pid": pid,
        "pid_alive": alive,
        "has_exit": dir.join("exit.json").is_file(),
        "root": root,
    })
}

/// `status [id]`: reads the queue directories only; the daemon need not run.
pub fn status(root: &Path, id: Option<&str>) -> Result<Value> {
    ensure_layout(root)?;
    if let Some(id) = id {
        let (stage, dir) =
            locate(root, id).ok_or_else(|| invalid(format!("no job with id {id}")))?;
        return Ok(summarize(root, stage, &dir));
    }
    let mut jobs = Vec::new();
    for (stage, dir) in [
        ("queue", queue_dir(root)),
        ("running", running_dir(root)),
        ("done", done_dir(root)),
    ] {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                jobs.push(summarize(root, stage, &entry.path()));
            }
        }
    }
    Ok(json!({"schema": "uor-r4.lab-runner-status/1", "jobs": jobs}))
}

fn tail_lines(path: &Path, count: usize) -> Vec<String> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    lines
        .into_iter()
        .rev()
        .take(count)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

/// `tail <id>`: last ~50 lines of each log, wherever the job currently is.
pub fn tail(root: &Path, id: &str, count: usize) -> Result<String> {
    let (_, dir) = locate(root, id).ok_or_else(|| invalid(format!("no job with id {id}")))?;
    let mut out = String::new();
    for name in ["stdout.log", "stderr.log"] {
        out.push_str(&format!("== {name} ==\n"));
        for line in tail_lines(&dir.join(name), count) {
            out.push_str(&line);
            out.push('\n');
        }
    }
    Ok(out)
}

/// `cancel <id>`: remove a queued job, or SIGTERM/SIGKILL a running one and
/// finalize it as `cancelled`.
pub fn cancel(root: &Path, id: &str, ledger_dir: &Path) -> Result<()> {
    let (stage, dir) = locate(root, id).ok_or_else(|| invalid(format!("no job with id {id}")))?;
    match stage {
        "queue" => {
            fs::remove_dir_all(&dir)?;
            Ok(())
        }
        "done" => Err(invalid(format!("job {id} is already done"))),
        _ => {
            let spec = read_spec(&dir).ok_or_else(|| invalid(format!("job {id} has no spec")))?;
            // Claim the outcome first: the daemon honors the marker when it
            // next monitors the job, so the race between the two finalizers
            // always converges on "cancelled".
            crate::write_json_atomic(
                &dir.join("cancel.json"),
                &json!({
                    "schema": "uor-r4.lab-runner-cancel/1",
                    "recorded_utc": utc_now_iso(),
                    "reason": "cancelled by lab-runner cancel",
                }),
            )?;
            let started_utc = fs::read_to_string(dir.join("started_utc"))
                .unwrap_or_else(|_| utc_now_iso())
                .trim()
                .to_string();
            let elapsed_ms = fs::read_to_string(dir.join("started_ms"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
                .map(|start| {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);
                    now.saturating_sub(start)
                })
                .unwrap_or(0);
            if let Some(pid) = read_pid(&dir) {
                if pid_alive(pid) {
                    kill_tree_signal(pid, "TERM");
                    let deadline = std::time::Instant::now() + Duration::from_secs(5);
                    while pid_alive(pid) && std::time::Instant::now() < deadline {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                    if pid_alive(pid) {
                        kill_tree_signal(pid, "KILL");
                    }
                }
            }
            let peak = read_pid(&dir).and_then(tree_rss_kib).unwrap_or(0);
            finalize(
                root,
                &spec,
                &Finalization {
                    outcome: "cancelled".to_string(),
                    exit_status: None,
                    reason: "cancelled by lab-runner cancel".to_string(),
                    peak_rss_kib: peak,
                    started_utc,
                    elapsed_ms,
                },
                ledger_dir,
            )
        }
    }
}
