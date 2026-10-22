//! `loop drive`: the supervisor for the five-lab loop (loop PR 10).
//!
//! # What it does
//!
//! One invocation runs one or more **fresh sessions**. Before each session the loop's own
//! [`loopnext::run`] decides go or stop: the kill switch (`local/loop/STOP` or the `loop:pause`
//! label on #2028), held work, an empty queue and the rest of the loop page's stops are **not
//! re-implemented here** - a session starts only when `loop next` says `Go`. The headless client
//! is launched from the caller's explicit argv with one `{prompt}` placeholder, in its own process
//! group, with its pid, pgid, client, model, prompt hash and start time recorded under
//! `local/loop/drive/<lab>/`. The supervisor only ever signals that recorded pid/group, and it
//! waits on the tracked child, so a client's after-stop summary process is never mistaken for the
//! lab session.
//!
//! # The soft 2-hour check-in (D53)
//!
//! At [`CHECK_IN_SECONDS`] on a session the supervisor classifies progress from the repository:
//! commits, gate-run activity, and working-tree changes are real progress; no movement is stalled,
//! and a second consecutive stalled session is spiraled. Real progress continues **with a note on
//! the owning issue**; a stalled or spiraled session is closed with WIP state recorded from git and
//! a card to the PM. Leased GPU runs are exempt: [`watch_until_exit`] keeps renewing the lease and
//! never cancels the run.
//!
//! # Why the process layer is thin
//!
//! The planner ([`classify`], [`check_in`], [`compose_prompt`], [`launch_argv`]) is pure and tested
//! with a fake clock; the process layer is [`start_session`], [`wait_session`], [`session_alive`]
//! and [`terminate_session`], so the risky part is small and the tests mostly avoid pids.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// The soft task limit of D53: two hours on a session before the check-in runs.
pub const CHECK_IN_SECONDS: u64 = 7_200;
/// Where session records live, relative to the main checkout.
pub const DRIVE_DIR: &str = "local/loop/drive";
/// How many sessions one invocation may start by default.
pub const DEFAULT_MAX_SESSIONS: usize = 1;

/// Why a drive command refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveError {
    /// The launch argv does not carry exactly one `{prompt}` placeholder.
    PromptPlaceholder(usize),
    /// The client or model flag is missing.
    MissingFlag(&'static str),
    /// The client name is not a registered adapter under `docs/harness/onboarding/`.
    UnknownClient(String),
    /// A prompt was passed twice.
    TwoPrompts,
    /// The session could not be launched or its record written.
    Io(String),
}

impl fmt::Display for DriveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PromptPlaceholder(found) => write!(
                formatter,
                "the launch command must contain exactly one {{prompt}} placeholder, found {found}"
            ),
            Self::MissingFlag(flag) => write!(formatter, "loop drive needs {flag}"),
            Self::UnknownClient(client) => write!(
                formatter,
                "no client adapter docs/harness/onboarding/{client}.json"
            ),
            Self::TwoPrompts => formatter.write_str("pass --prompt or --prompt-file, not both"),
            Self::Io(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for DriveError {}

type Result<T> = std::result::Result<T, DriveError>;

/// One observation of the repository a check-in classifies progress from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    /// `git rev-parse HEAD`.
    pub head: String,
    /// `git rev-list --count HEAD`.
    pub commits: u64,
    /// Lines in the lab's gate ledger; gate runs are activity even without a commit.
    pub gate_runs: u64,
    /// `git status --porcelain` entries: edits in progress are activity too.
    pub changed_paths: usize,
}

impl Observation {
    /// The observation from `root` and `lab`; read-only, no network.
    pub fn take(root: &Path, lab: &str) -> Observation {
        let git = |args: &[&str]| -> String {
            Command::new("git")
                .args(args)
                .current_dir(root)
                .output()
                .ok()
                .filter(|out| out.status.success())
                .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
                .unwrap_or_default()
        };
        let head = git(&["rev-parse", "HEAD"]);
        let commits = git(&["rev-list", "--count", "HEAD"]).parse().unwrap_or(0);
        let changed = git(&["status", "--porcelain"]);
        let changed_paths = changed
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count();
        let ledger = root
            .join("local")
            .join(crate::state::slug(lab))
            .join("gates.jsonl");
        let gate_runs = std::fs::read_to_string(ledger)
            .map(|text| text.lines().count() as u64)
            .unwrap_or(0);
        Observation {
            head,
            commits,
            gate_runs,
            changed_paths,
        }
    }
}

/// What the repository says about progress between two observations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Progress {
    /// Real progress: commits, gate runs or working-tree edits.
    Progressing(&'static str),
    /// No movement at all.
    Stalled,
}

/// Classify progress. The classification is deliberately conservative: **any** of a new commit,
/// another gate run or a changed working tree counts as progress, because D53 says real progress
/// continues and only a session with nothing to show is stalled.
pub fn classify(previous: &Observation, current: &Observation) -> Progress {
    if !current.head.is_empty() && current.head != previous.head {
        return Progress::Progressing("new HEAD");
    }
    if current.commits > previous.commits {
        return Progress::Progressing("new commits");
    }
    if current.gate_runs > previous.gate_runs {
        return Progress::Progressing("new gate runs");
    }
    if current.changed_paths != previous.changed_paths {
        return Progress::Progressing("working tree changed");
    }
    Progress::Stalled
}

/// What the 2-hour check-in decides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckIn {
    /// Under two hours, or real progress: continue, with a note on the owning issue.
    Continue {
        /// The reason printed or posted.
        note: String,
    },
    /// Two hours with no progress (and a repeat): close the session with WIP state and a card.
    Close {
        /// The reason printed or carded.
        reason: String,
    },
}

/// The check-in decision. Under [`CHECK_IN_SECONDS`] it always continues; past it, a progressing
/// session continues with a note (D53) and a stalled one closes - and when the same stall repeats it
/// is reported as spiraled. The caller counts the repeats.
pub fn check_in(elapsed_seconds: u64, progress: &Progress, stalled_repeats: u32) -> CheckIn {
    if elapsed_seconds < CHECK_IN_SECONDS {
        return CheckIn::Continue {
            note: format!("session under the 2-hour soft check-in ({elapsed_seconds}s)"),
        };
    }
    match progress {
        Progress::Progressing(why) => CheckIn::Continue {
            note: format!("2-hour check-in: real progress ({why}); continuing"),
        },
        Progress::Stalled if stalled_repeats >= 2 => CheckIn::Close {
            reason: format!(
                "spiraled: two consecutive sessions with no commits, gate runs or edits ({elapsed_seconds}s)"
            ),
        },
        Progress::Stalled => CheckIn::Close {
            reason: format!(
                "stalled: no commits, gate runs or edits for {elapsed_seconds}s of the 2-hour task"
            ),
        },
    }
}

/// The argv to launch: the caller's command with its single `{prompt}` element replaced by the
/// composed prompt. A placeholder anywhere else in an argument is left alone, so a client flag that
/// legitimately contains the word is not rewritten.
pub fn launch_argv(launch: &[String], prompt: &str) -> Result<Vec<String>> {
    let mut found = 0usize;
    let mut argv = Vec::with_capacity(launch.len());
    for argument in launch {
        if argument == "{prompt}" {
            found += 1;
            argv.push(prompt.to_string());
        } else {
            argv.push(argument.clone());
        }
    }
    if found != 1 {
        return Err(DriveError::PromptPlaceholder(found));
    }
    Ok(argv)
}

/// The session prompt: the caller's prompt, or a default composed from the `loop next` decision.
pub fn compose_prompt(caller_prompt: Option<&str>, decision: &str) -> String {
    if let Some(prompt) = caller_prompt {
        if !prompt.trim().is_empty() {
            return prompt.to_string();
        }
    }
    format!(
        "You are one fresh session of the UOR-R4 five-lab loop. `loop next` decided: {decision}.\n\nRun one unit of work per the loop cycle: read local/claude/onboarding/run-<lab>.txt, the bench inbox (main checkout local/bench/INBOX.md) and your PICKUP; claim, work inside the ORDER's scope, run gates, write the REPORT, push and request the exact-head review; stop cleanly at the end."
    )
}

/// One running (or finished) session, as recorded on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecord {
    /// The client adapter name.
    pub client: String,
    /// The pinned model id the supervisor recorded from its own launch command.
    pub model: String,
    /// The child pid.
    pub pid: u32,
    /// The child process group, which is what the supervisor signals.
    pub pgid: u32,
    /// Unix start time.
    pub started_unix: u64,
    /// SHA-256 of the prompt, so a session can be tied to what it was asked.
    pub prompt_sha256: String,
}

/// The session record file for a lab.
pub fn record_path(root: &Path, lab: &str) -> PathBuf {
    root.join(DRIVE_DIR)
        .join(crate::state::slug(lab))
        .join("last-session.json")
}

/// Write the record atomically (temporary file then rename), so a supervisor restart never reads a
/// half-written session.
pub fn write_record(root: &Path, lab: &str, record: &SessionRecord) -> Result<()> {
    let path = record_path(root, lab);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            DriveError::Io(format!("cannot create {}: {error}", parent.display()))
        })?;
    }
    let text = serde_json::json!({
        "client": record.client,
        "model": record.model,
        "pid": record.pid,
        "pgid": record.pgid,
        "started_unix": record.started_unix,
        "prompt_sha256": record.prompt_sha256,
    })
    .to_string();
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, format!("{text}\n"))
        .map_err(|error| DriveError::Io(format!("cannot write {}: {error}", tmp.display())))?;
    std::fs::rename(&tmp, &path)
        .map_err(|error| DriveError::Io(format!("cannot record {}: {error}", path.display())))
}

/// Read the last session record, if any.
pub fn read_record(root: &Path, lab: &str) -> Option<SessionRecord> {
    let text = std::fs::read_to_string(record_path(root, lab)).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    Some(SessionRecord {
        client: value.get("client")?.as_str()?.to_string(),
        model: value.get("model")?.as_str()?.to_string(),
        pid: value.get("pid")?.as_u64()? as u32,
        pgid: value.get("pgid")?.as_u64()? as u32,
        started_unix: value.get("started_unix")?.as_u64()?,
        prompt_sha256: value.get("prompt_sha256")?.as_str()?.to_string(),
    })
}

/// Start one session: the argv runs in its own process group, stdout/stderr go to `log`, and the
/// record is written before the caller waits. Returns the child and its record.
pub fn start_session(
    root: &Path,
    lab: &str,
    client: &str,
    model: &str,
    argv: &[String],
    prompt_sha256: &str,
    log: &Path,
) -> Result<(Child, SessionRecord)> {
    let (program, arguments) = argv
        .split_first()
        .ok_or(DriveError::MissingFlag("a launch command after --"))?;
    if let Some(parent) = log.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            DriveError::Io(format!("cannot create {}: {error}", parent.display()))
        })?;
    }
    let out = std::fs::File::create(log)
        .map_err(|error| DriveError::Io(format!("cannot create {}: {error}", log.display())))?;
    let err = out
        .try_clone()
        .map_err(|error| DriveError::Io(format!("cannot clone the log handle: {error}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let mut command = Command::new(program);
        command
            .args(arguments)
            .current_dir(root)
            .stdin(Stdio::null())
            .stdout(Stdio::from(out))
            .stderr(Stdio::from(err))
            .process_group(0);
        let child = command
            .spawn()
            .map_err(|error| DriveError::Io(format!("cannot launch {program}: {error}")))?;
        let pid = child.id();
        let record = SessionRecord {
            client: client.to_string(),
            model: model.to_string(),
            pid,
            pgid: pid,
            started_unix: unix_now(),
            prompt_sha256: prompt_sha256.to_string(),
        };
        write_record(root, lab, &record)?;
        Ok((child, record))
    }
    #[cfg(not(unix))]
    {
        let _ = (
            program,
            arguments,
            out,
            err,
            client,
            model,
            prompt_sha256,
            log,
        );
        Err(DriveError::Io(
            "loop drive signals process groups and is unix-only".to_string(),
        ))
    }
}

/// `kill -0` on the recorded pid/group: is the tracked session still the one running?
pub fn session_alive(record: &SessionRecord) -> bool {
    Command::new("kill")
        .args(["-0", &format!("-{}", record.pgid)])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Terminate the recorded session: SIGTERM to its own process group, poll for a short grace, then
/// SIGKILL to the same group. Nothing else is ever signalled - not another lab's worktree, not a
/// `pkill`-style pattern, and not a pid that was not recorded.
pub fn terminate_session(record: &SessionRecord) -> Result<()> {
    if !session_alive(record) {
        return Ok(());
    }
    let group = format!("-{}", record.pgid);
    let _ = Command::new("kill")
        .args(["-TERM", &group])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    for _ in 0..20 {
        if !session_alive(record) {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let _ = Command::new("kill")
        .args(["-KILL", &group])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    Ok(())
}

/// The watcher for a leased GPU run: while the tracked session pid is alive, run the caller's renew
/// command every `interval_seconds` and note it; when the session ends, the watcher ends. It never
/// signals the run and never edits a lease it does not hold. `ticks` bounds the loop for tests;
/// the caller passes `None` in production.
pub fn watch_until_exit(
    record: &SessionRecord,
    renew: &[String],
    interval_seconds: u64,
    ticks: Option<usize>,
) -> Result<Vec<String>> {
    if renew.is_empty() {
        return Err(DriveError::MissingFlag("a renew command after --"));
    }
    let mut notes = Vec::new();
    let mut rounds = 0usize;
    while session_alive(record) {
        if let Some(limit) = ticks {
            if rounds >= limit {
                notes.push(format!("watch stopped after {limit} tick(s) (test bound)"));
                break;
            }
        }
        let (program, arguments) = (&renew[0], &renew[1..]);
        let out = Command::new(program)
            .args(arguments)
            .output()
            .map_err(|error| DriveError::Io(format!("cannot run the renew command: {error}")))?;
        notes.push(format!(
            "renewed at {}: {} {}",
            unix_now(),
            out.status,
            String::from_utf8_lossy(&out.stdout).trim()
        ));
        rounds += 1;
        std::thread::sleep(std::time::Duration::from_secs(interval_seconds));
    }
    notes.push("session ended; watcher exits without cancelling anything".to_string());
    Ok(notes)
}

/// A leased GPU run's watcher: it **outlives the lab session**. Started as its own detached
/// process, it renews the lease every `interval_seconds` while the renew command succeeds and stops
/// when that command reports the lease is gone (a non-zero exit is the lease-ended signal the caller's
/// command defines). It never signals the run, never edits a lease it does not hold, and never cancels
/// a healthy run; `ticks` bounds the loop for tests.
pub fn watch_lease_until_renew_fails(
    renew: &[String],
    interval_seconds: u64,
    ticks: Option<usize>,
) -> Result<Vec<String>> {
    if renew.is_empty() {
        return Err(DriveError::MissingFlag("a renew command after --"));
    }
    let mut notes = Vec::new();
    let mut rounds = 0usize;
    loop {
        if let Some(limit) = ticks {
            if rounds >= limit {
                notes.push(format!(
                    "lease watch stopped after {limit} tick(s) (test bound)"
                ));
                return Ok(notes);
            }
        }
        let (program, arguments) = (&renew[0], &renew[1..]);
        let out = Command::new(program)
            .args(arguments)
            .output()
            .map_err(|error| DriveError::Io(format!("cannot run the renew command: {error}")))?;
        if !out.status.success() {
            notes.push(format!(
                "lease ended at {}: renew exited {}; watcher stops without cancelling the run",
                unix_now(),
                out.status
            ));
            return Ok(notes);
        }
        notes.push(format!(
            "renewed at {}: {}",
            unix_now(),
            String::from_utf8_lossy(&out.stdout).trim()
        ));
        rounds += 1;
        std::thread::sleep(std::time::Duration::from_secs(interval_seconds));
    }
}

/// The PM card for a closed session: what the PM reads to prompt the lab. Pure so the wording is
/// tested; the CLI posts it on #2028.
pub fn pm_card_text(lab: &str, reason: &str, head: &str, commits: u64) -> String {
    format!(
        "[{lab}] drive closed a session at the soft 2-hour check-in ({reason}).\n\n- git HEAD: {head}\n- commits: {commits}\nThe REPORT stub and PICKUP are under local/loop/drive/{lab}/close-<time>/. The lab needs a prompt."
    )
}

/// Post the PM card on #2028 through `gh`. Live-only: the tests test the wording and the stub files,
/// never the network.
pub fn post_pm_card(root: &Path, lab: &str, reason: &str) -> Result<()> {
    let observation = Observation::take(root, lab);
    let text = pm_card_text(lab, reason, &observation.head, observation.commits);
    let path = root
        .join("local/loop/drive")
        .join(crate::state::slug(lab))
        .join("pm-card.md");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            DriveError::Io(format!("cannot create {}: {error}", parent.display()))
        })?;
    }
    std::fs::write(&path, format!("{text}\n"))
        .map_err(|error| DriveError::Io(format!("cannot write {}: {error}", path.display())))?;
    let status = Command::new("gh")
        .args(["issue", "comment", "2028", "--body-file"])
        .arg(&path)
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| DriveError::Io(format!("cannot run gh: {error}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(DriveError::Io(format!(
            "gh issue comment 2028 exited {status}"
        )))
    }
}

/// Commit and push the WIP of a closed session, best effort: the session's changes are staged and
/// committed with the lab trailer, then pushed. A failure is returned, never silently swallowed; the
/// supervisor logs it and continues, so nothing is ever force-pushed or deleted.
pub fn commit_and_push_wip(root: &Path, lab: &str, reason: &str) -> Result<()> {
    let run = |args: &[&str]| -> Result<()> {
        let out = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .map_err(|error| DriveError::Io(format!("cannot run git {:?}: {error}", args)))?;
        if out.status.success() {
            Ok(())
        } else {
            Err(DriveError::Io(format!(
                "git {:?} exited {}: {}",
                args,
                out.status,
                String::from_utf8_lossy(&out.stderr).trim()
            )))
        }
    };
    run(&["add", "-A"])?;
    run(&[
        "commit",
        "-m",
        &format!("[{lab}] WIP: drive closed a session at the 2-hour check-in\n\n- reason: {reason}\n\nLab: {lab}"),
    ])?;
    run(&["push"])
}

/// Seconds since the Unix epoch.
pub fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

/// Dispatch `loop drive`. Returns the exit code, 0 on a clean stop or a completed session, 1 when a
/// session ran and its client failed, 2 for usage and refusal errors.
pub fn cli(root: &Path, lab: &str, args: &[String]) -> u8 {
    let mut client: Option<String> = None;
    let mut model: Option<String> = None;
    let mut prompt: Option<String> = None;
    let mut max_sessions: Option<usize> = None;
    let mut watch = false;
    let mut lease = false;
    let mut pid: Option<u32> = None;
    let mut interval: u64 = 60;
    let mut launch: Vec<String> = Vec::new();
    let mut index = 0;
    let mut after_separator = false;
    while index < args.len() {
        let argument = args[index].as_str();
        if after_separator {
            launch.push(argument.to_string());
            index += 1;
            continue;
        }
        match argument {
            "--" => after_separator = true,
            "--watch" => watch = true,
            "--lease" => lease = true,
            "--client" | "--model" | "--prompt" | "--max-sessions" | "--pid" | "--interval" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("loop drive: {argument} needs a value");
                    return 2;
                };
                match argument {
                    "--client" => client = Some(value.clone()),
                    "--model" => model = Some(value.clone()),
                    "--prompt" => prompt = Some(value.clone()),
                    "--pid" => pid = value.parse().ok(),
                    "--interval" => interval = value.parse().unwrap_or(60),
                    _ => max_sessions = value.parse().ok(),
                }
                index += 2;
                continue;
            }
            other => {
                eprintln!(
                    "loop drive: unknown argument {other:?} (the client command goes after --)"
                );
                return 2;
            }
        }
        index += 1;
    }
    if watch {
        if lease {
            return match watch_lease_until_renew_fails(&launch, interval, None) {
                Ok(notes) => {
                    for note in notes {
                        println!("{note}");
                    }
                    0
                }
                Err(error) => {
                    eprintln!("loop drive --watch --lease: {error}");
                    2
                }
            };
        }
        let Some(pid) = pid else {
            eprintln!("loop drive --watch needs --pid <the tracked session pid>");
            return 2;
        };
        let record = session_by_pid(root, lab, pid).or_else(|| read_record(root, lab));
        let Some(record) = record.filter(|record| record.pid == pid) else {
            eprintln!("loop drive --watch: no recorded session with pid {pid}");
            return 2;
        };
        return match watch_until_exit(&record, &launch, interval, None) {
            Ok(notes) => {
                for note in notes {
                    println!("{note}");
                }
                0
            }
            Err(error) => {
                eprintln!("loop drive --watch: {error}");
                2
            }
        };
    }
    let Some(client) = client else {
        eprintln!("loop drive needs --client <name>");
        return 2;
    };
    let Some(model) = model else {
        eprintln!("loop drive needs --model <id>");
        return 2;
    };
    if !root
        .join("docs/harness/onboarding")
        .join(format!("{client}.json"))
        .is_file()
    {
        eprintln!("loop drive: no client adapter docs/harness/onboarding/{client}.json");
        return 2;
    }
    if launch.is_empty() {
        eprintln!("loop drive needs the client command after --, with one {{prompt}} placeholder");
        return 2;
    }
    if launch_argv(&launch, "prompt").is_err() {
        eprintln!("loop drive: the launch command must contain exactly one {{prompt}} placeholder");
        return 2;
    }
    // The loops's own stop decision, not a re-implementation of it.
    let next_gh = crate::loopnext::RealNextGh {
        root: root.to_path_buf(),
    };
    let claim_gh = crate::loopclaim::RealGh {
        root: root.to_path_buf(),
    };
    let decision = crate::loopnext::run(root, lab, &next_gh, &claim_gh);
    let decision_text = match decision {
        Ok((_, crate::loopnext::Decision::Stop(reason))) => {
            println!("STOP {reason}");
            return 0;
        }
        Ok((
            _,
            crate::loopnext::Decision::Go {
                kind,
                target,
                reason,
            },
        )) => {
            format!("GO {kind} {target} - {reason}")
        }
        Err(error) => {
            eprintln!("loop drive: cannot decide with loop next: {error}");
            return 2;
        }
    };
    let prompt = compose_prompt(prompt.as_deref(), &decision_text);
    let argv = match launch_argv(&launch, &prompt) {
        Ok(argv) => argv,
        Err(error) => {
            eprintln!("loop drive: {error}");
            return 2;
        }
    };
    let limit = max_sessions.unwrap_or(DEFAULT_MAX_SESSIONS).max(1);
    let mut last_code = 0;
    for session in 0..limit {
        if session > 0 {
            // Re-decide between sessions: the kill switch and the rest of the stops apply again.
            match crate::loopnext::run(root, lab, &next_gh, &claim_gh) {
                Ok((_, crate::loopnext::Decision::Stop(reason))) => {
                    println!("STOP {reason}");
                    return 0;
                }
                Ok(_) => {}
                Err(error) => {
                    eprintln!("loop drive: cannot decide with loop next: {error}");
                    return 2;
                }
            }
        }
        let directory = root.join(DRIVE_DIR).join(crate::state::slug(lab));
        let log = directory.join(format!("session-{}-{session}.log", unix_now()));
        let digest = crate::attention_reference::sha256_hex(prompt.as_bytes());
        let (mut child, record) =
            match start_session(root, lab, &client, &model, &argv, &digest, &log) {
                Ok(started) => started,
                Err(error) => {
                    eprintln!("loop drive: {error}");
                    return 2;
                }
            };
        println!(
            "started session {session} pid {} pgid {} client {client} model {model} log {}",
            record.pid,
            record.pgid,
            log.display()
        );
        let status = match child.wait() {
            Ok(status) => status,
            Err(error) => {
                eprintln!("loop drive: cannot wait on pid {}: {error}", record.pid);
                return 2;
            }
        };
        let elapsed = unix_now().saturating_sub(record.started_unix);
        let before = Observation::take(root, lab);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let after = Observation::take(root, lab);
        let progress = classify(&before, &after);
        match check_in(elapsed, &progress, 0) {
            CheckIn::Continue { note } => println!("{note}"),
            CheckIn::Close { reason } => {
                println!("CHECK-IN CLOSE: {reason}");
                if let Err(error) = close_stalled_session(root, lab, &record, &reason) {
                    eprintln!("loop drive: cannot record the stalled session: {error}");
                }
                if let Err(error) = commit_and_push_wip(root, lab, &reason) {
                    eprintln!("loop drive: cannot commit/push the WIP: {error}");
                }
                if let Err(error) = post_pm_card(root, lab, &reason) {
                    eprintln!("loop drive: cannot post the PM card: {error}");
                }
            }
        }
        last_code = status.code().unwrap_or(1);
    }
    if last_code == 0 {
        0
    } else {
        1
    }
}

/// Record a stalled session from git state: a REPORT stub and a PICKUP under the drive directory, so
/// the next session (and the PM card) can see what was happening. Nothing is deleted and no other
/// lab's file is touched.
pub fn close_stalled_session(
    root: &Path,
    lab: &str,
    record: &SessionRecord,
    reason: &str,
) -> Result<PathBuf> {
    let directory = root
        .join(DRIVE_DIR)
        .join(crate::state::slug(lab))
        .join(format!("close-{}", unix_now()));
    std::fs::create_dir_all(&directory).map_err(|error| {
        DriveError::Io(format!("cannot create {}: {error}", directory.display()))
    })?;
    let observation = Observation::take(root, lab);
    let stub = format!(
        "# REPORT stub: session closed by the soft 2-hour check-in\n\n\
         - lab: {lab}\n- client: {}\n- model: {}\n- pid: {}\n- started_unix: {}\n- reason: {reason}\n\
         - git HEAD: {}\n- commits: {}\n- changed paths: {}\n\n\
         Written from git state only; no result is claimed.\n",
        record.client,
        record.model,
        record.pid,
        record.started_unix,
        observation.head,
        observation.commits,
        observation.changed_paths
    );
    std::fs::write(directory.join("REPORT-stub.md"), stub)
        .map_err(|error| DriveError::Io(format!("cannot write the REPORT stub: {error}")))?;
    let pickup = format!(
        "# PICKUP (written by loop drive from git state)\n\n\
         - closed: {reason}\n- HEAD: {}\n- commits: {}\n\n\
         Resume by reading the active ORDER and running `uor-harness loop next`.\n",
        observation.head, observation.commits
    );
    std::fs::write(directory.join("PICKUP.md"), pickup)
        .map_err(|error| DriveError::Io(format!("cannot write the PICKUP: {error}")))?;
    Ok(directory)
}

/// The session record for a pid, if `last-session.json` names it.
fn session_by_pid(root: &Path, lab: &str, pid: u32) -> Option<SessionRecord> {
    read_record(root, lab).filter(|record| record.pid == pid)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation(head: &str, commits: u64, gate_runs: u64, changed: usize) -> Observation {
        Observation {
            head: head.to_string(),
            commits,
            gate_runs,
            changed_paths: changed,
        }
    }

    #[test]
    fn commits_gate_runs_and_edits_are_progress() {
        let before = observation("a", 1, 0, 0);
        assert_eq!(
            classify(&before, &observation("b", 1, 0, 0)),
            Progress::Progressing("new HEAD")
        );
        assert_eq!(
            classify(&before, &observation("a", 2, 0, 0)),
            Progress::Progressing("new commits")
        );
        assert_eq!(
            classify(&before, &observation("a", 1, 1, 0)),
            Progress::Progressing("new gate runs")
        );
        assert_eq!(
            classify(&before, &observation("a", 1, 0, 2)),
            Progress::Progressing("working tree changed")
        );
        assert_eq!(classify(&before, &before), Progress::Stalled);
    }

    #[test]
    fn the_check_in_is_soft_until_two_hours_then_progress_continues() {
        assert!(matches!(
            check_in(60, &Progress::Stalled, 0),
            CheckIn::Continue { .. }
        ));
        assert!(matches!(
            check_in(CHECK_IN_SECONDS, &Progress::Progressing("new commits"), 0),
            CheckIn::Continue { .. }
        ));
    }

    #[test]
    fn a_stalled_session_past_two_hours_closes_and_a_repeat_is_spiraled() {
        match check_in(CHECK_IN_SECONDS, &Progress::Stalled, 0) {
            CheckIn::Close { reason } => assert!(reason.starts_with("stalled:"), "{reason}"),
            other => panic!("expected a close, got {other:?}"),
        }
        match check_in(CHECK_IN_SECONDS, &Progress::Stalled, 2) {
            CheckIn::Close { reason } => assert!(reason.starts_with("spiraled:"), "{reason}"),
            other => panic!("expected a spiraled close, got {other:?}"),
        }
    }

    #[test]
    fn the_launch_command_needs_exactly_one_prompt_placeholder() {
        let argv = vec!["dsh".to_string(), "{prompt}".to_string()];
        assert_eq!(
            launch_argv(&argv, "hi").unwrap(),
            vec!["dsh".to_string(), "hi".to_string()]
        );
        assert_eq!(
            launch_argv(&["dsh".to_string()], "hi").unwrap_err(),
            DriveError::PromptPlaceholder(0)
        );
        assert_eq!(
            launch_argv(&["{prompt}".to_string(), "{prompt}".to_string()], "hi").unwrap_err(),
            DriveError::PromptPlaceholder(2)
        );
    }

    #[test]
    fn the_default_prompt_carries_the_decision_and_a_caller_prompt_wins() {
        let default = compose_prompt(None, "GO item x - ready");
        assert!(default.contains("GO item x - ready"), "{default}");
        assert_eq!(
            compose_prompt(Some("my prompt"), "GO item x"),
            "my prompt".to_string()
        );
    }
}
