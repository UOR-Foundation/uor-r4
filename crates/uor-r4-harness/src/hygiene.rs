//! `uor-harness hygiene` — keep the laptop lean (CHARTER section 5 item 9, `docs/labs/workspace.md`).
//!
//! The Rust port of the bash `scripts/storage/uor-hygiene` as #2811 left it. That script is now a
//! thin wrapper that runs this command, and keeps its path because the charter names it. A dry run is
//! the default; `--apply` does the work.
//!
//! What it removes, and the guard on each, in the order it runs:
//!
//! 1. **Worktrees** that are clean and merged — HEAD is in `origin/main`, or is the head of a merged
//!    PR, because main squash-merges — and their local branch. Never a protected branch
//!    (`docs/harness/branches.json`), never a branch with an open PR, never a locked or busy
//!    worktree, never one holding gitignored data other than `target/`.
//! 2. **Build folders** (`~/.cache/uor-*`, `<repo>/target/*`, each worktree's `target/`) that no file
//!    was written to for the idle window and that no running process uses.
//! 3. **Scratch** `<repo>/local/<lab>/<topic>/` holding an `UPLOADED` marker, idle and not busy: moved
//!    to the Trash. Scratch without the marker is only reported, after 7 idle days.
//! 4. **Stale temporary items**: direct children of `/private/tmp` (and `/tmp` if it differs) owned by
//!    you, folders or files over 50 MB, idle and not open anywhere: moved to the Trash.
//! 5. `git gc --auto`, and `uv cache prune` when uv is idle.
//! 6. **Report only:** `~/uor-r4-local`, second clones, worktrees outside `<repo>/.worktrees/`, and
//!    free space under 30 GB.
//!
//! It never deletes lab data (scratch and temp go to the Trash; only build folders are removed), never
//! empties the Trash, and never touches a folder holding a `.keep-local` marker.
//!
//! # Facts, decisions, actions
//!
//! Each decision is a function over facts, so a test states the facts and checks the verdict. Facts
//! about other processes (who has a file open, what GitHub says about a branch) come through
//! [`System`]; facts on disk (idle, size, markers, git state) are read directly and are tested over
//! temporary directories. The guards run in the bash order and stop at the first refusal, because the
//! late ones (`gh`, `lsof +D`) are slow.
//!
//! **Every `git` call removes the `GIT_*` variables.** The tool always names its repository by path,
//! and a hook's `GIT_DIR` would otherwise point `git -C <worktree> status` at another repository.

use std::collections::{BTreeSet, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::io::{BufRead, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::githelp::protected_refusal;

/// Hours without a write before a build folder, scratch or temp item is idle (`UOR_IDLE_HOURS`).
pub const DEFAULT_IDLE_HOURS: i64 = 24;
/// Scratch without an `UPLOADED` marker is reported once nothing in it changed for this many days.
pub const NOTE_AFTER_DAYS: i64 = 7;
/// Below this much free space the report warns.
pub const MIN_FREE_GB: u64 = 30;
/// Temp files at or under this size are left alone; temp folders are considered at any size.
pub const TEMP_FILE_MIN_BYTES: u64 = 50 * 1024 * 1024;
/// Indexers and backups touch every file briefly; an open file of theirs is not live work.
pub const INDEXERS: [&str; 9] = [
    "mdworker",
    "mds",
    "mdsync",
    "Spotlight",
    "fseventsd",
    "backupd",
    "revisiond",
    "bird",
    "cloudd",
];

const IN_USE: &str = "in use by a running process";
const KEEP_LOCAL: &str = ".keep-local marker";
const MINUTE: i64 = 60;
const DAY: i64 = 86_400;

/// Where the run looks, and when it started.
#[derive(Debug, Clone)]
pub struct Config {
    pub apply: bool,
    pub home: PathBuf,
    /// The one clone (`UOR_REPO`, default `~/uor-r4`).
    pub repo: PathBuf,
    /// The scratch root (`UOR_LOCAL`, default `<repo>/local`).
    pub local: PathBuf,
    pub idle_hours: i64,
    /// `~/.Trash/uor-hygiene-<stamp>`, created only when something is moved there.
    pub trash: PathBuf,
    pub tmp_roots: Vec<PathBuf>,
    /// Temp items are considered only when this user owns them.
    pub uid: u32,
    /// The running binary. Its build folder holds it, so that folder is never removed under it.
    pub exe: Option<PathBuf>,
    /// Seconds since the epoch when the run started; every idle test measures from here.
    pub now: i64,
}

impl Config {
    /// The configuration the bash script read: `HOME`, `UOR_REPO`, `UOR_LOCAL`, `UOR_IDLE_HOURS`.
    pub fn from_env(apply: bool, repo: Option<PathBuf>) -> Result<Config, String> {
        let home = env_set("HOME")
            .map(PathBuf::from)
            .ok_or("HOME is not set")?;
        let repo = repo
            .or_else(|| env_set("UOR_REPO").map(PathBuf::from))
            .unwrap_or_else(|| home.join("uor-r4"));
        let local = env_set("UOR_LOCAL")
            .map(PathBuf::from)
            .unwrap_or_else(|| repo.join("local"));
        let idle_hours = match env_set("UOR_IDLE_HOURS") {
            Some(v) => {
                let v = v.to_string_lossy().into_owned();
                v.trim()
                    .parse::<i64>()
                    .ok()
                    .filter(|h| *h >= 0)
                    .ok_or_else(|| format!("UOR_IDLE_HOURS={v:?} is not a whole number of hours"))?
            }
            None => DEFAULT_IDLE_HOURS,
        };
        let uid = stdout_of("id", ["-u"])
            .and_then(|s| s.trim().parse::<u32>().ok())
            .ok_or("`id -u` did not give a user id")?;
        let now = now_unix();
        let stamp = stdout_of("date", ["+%Y%m%d-%H%M%S"])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| now.to_string());
        Ok(Config {
            apply,
            trash: home.join(".Trash").join(format!("uor-hygiene-{stamp}")),
            home,
            repo,
            local,
            idle_hours,
            tmp_roots: tmp_roots(),
            uid,
            exe: std::env::current_exe()
                .ok()
                .and_then(|p| fs::canonicalize(p).ok()),
            now,
        })
    }
}

/// An environment variable that is set and not empty, as the script's `${VAR:-default}` read it.
fn env_set(name: &str) -> Option<std::ffi::OsString> {
    std::env::var_os(name).filter(|v| !v.is_empty())
}

/// `/private/tmp`, and `/tmp` when it is a different directory.
pub fn tmp_roots() -> Vec<PathBuf> {
    let private = PathBuf::from("/private/tmp");
    let tmp = Path::new("/tmp");
    let mut roots = vec![private.clone()];
    if tmp.is_dir() && fs::canonicalize(tmp).ok() != fs::canonicalize(&private).ok() {
        roots.push(tmp.to_path_buf());
    }
    roots
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------------------------
// What other processes and GitHub say.
// ---------------------------------------------------------------------------------------------

/// One pull request whose head is a given branch, as `gh pr list --json state,headRefOid` reports it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct PullRequest {
    /// `OPEN`, `MERGED` or `CLOSED`.
    pub state: String,
    #[serde(rename = "headRefOid")]
    pub head: String,
}

/// The facts that depend on other processes, so a test can state them.
pub trait System {
    /// A running cargo or rustc names `path` or runs inside it, or any other process (indexers
    /// excepted, and this run's own process and its parent) has a file open beneath it.
    fn busy(&self, path: &Path) -> bool;
    /// Any process has the temp item open (`lsof +D` for a folder, `lsof` for a file).
    fn in_use(&self, path: &Path) -> bool;
    /// Every PR whose head is `branch`. `None` when GitHub could not be asked.
    fn pull_requests(&self, branch: &str) -> Option<Vec<PullRequest>>;
    /// `df -h /`'s available column, for the header and the footer.
    fn free_human(&self) -> Option<String>;
    /// Whole gigabytes free on the volume holding `path`.
    fn free_gb(&self, path: &Path) -> Option<u64>;
    /// `uv cache prune`: `None` when uv is not installed, `Some(false)` when it is busy.
    fn prune_uv(&self) -> Option<bool>;
}

/// The live machine: `ps`, `pgrep`, `lsof`, `gh`, `df` and `uv`.
pub struct LiveSystem {
    /// `owner/name` of `origin` on GitHub.
    slug: Option<String>,
    /// This process and its parent: the script's own process tree, which never makes a path busy.
    own: [String; 2],
}

impl LiveSystem {
    pub fn new(repo: &Path) -> LiveSystem {
        LiveSystem {
            slug: git(repo, ["remote", "get-url", "origin"]).and_then(|u| github_slug(&u)),
            own: [
                std::process::id().to_string(),
                std::os::unix::process::parent_id().to_string(),
            ],
        }
    }
}

/// `owner/name` from an https or ssh GitHub remote, as the script's `sed` extracted it.
pub fn github_slug(url: &str) -> Option<String> {
    let at = url.rfind("github.com")?;
    let rest = &url[at + "github.com".len()..];
    let rest = rest.strip_prefix(':').or_else(|| rest.strip_prefix('/'))?;
    let slug = rest.strip_suffix(".git").unwrap_or(rest);
    (!slug.is_empty()).then(|| slug.to_string())
}

/// `comm` names cargo or rustc, by itself or as the last path component.
fn is_cargo_or_rustc(comm: &str) -> bool {
    ["cargo", "rustc"]
        .iter()
        .any(|n| comm == *n || comm.ends_with(&format!("/{n}")))
}

impl System for LiveSystem {
    fn busy(&self, path: &Path) -> bool {
        let p = path.to_string_lossy();
        // A running cargo/rustc that names the folder on its command line.
        let ps = stdout_of("ps", ["-axo", "pid=,comm=,args="]).unwrap_or_default();
        for line in ps.lines() {
            let comm = line.split_whitespace().nth(1).unwrap_or("");
            if is_cargo_or_rustc(comm) && line.contains(&*p) {
                return true;
            }
        }
        // ... or that runs from inside it.
        let mut builders = stdout_of("pgrep", ["-x", "cargo"]).unwrap_or_default();
        builders.push_str(&stdout_of("pgrep", ["-x", "rustc"]).unwrap_or_default());
        for pid in builders.split_whitespace() {
            let cwd = stdout_of("lsof", ["-a", "-p", pid, "-d", "cwd", "-Fn"]).unwrap_or_default();
            if cwd
                .lines()
                .filter_map(|l| l.strip_prefix('n'))
                .any(|c| c.starts_with(&*p))
            {
                return true;
            }
        }
        // Any other process with an open file or working directory inside it (a training run, a
        // shell, a copy), except this run's own process tree and the indexers.
        let holders = stdout_of(
            "lsof",
            [OsStr::new("-t"), OsStr::new("+D"), path.as_os_str()],
        )
        .unwrap_or_default();
        let pids: BTreeSet<&str> = holders.split_whitespace().collect();
        for pid in pids {
            if self.own.iter().any(|o| o == pid) {
                continue;
            }
            let comm = stdout_of("ps", ["-o", "comm=", "-p", pid]).unwrap_or_default();
            if INDEXERS.iter().any(|i| comm.contains(i)) {
                continue;
            }
            return true;
        }
        false
    }

    fn in_use(&self, path: &Path) -> bool {
        let dir = fs::symlink_metadata(path)
            .map(|m| m.is_dir())
            .unwrap_or(false);
        let out = if dir {
            stdout_of("lsof", [OsStr::new("+D"), path.as_os_str()])
        } else {
            stdout_of("lsof", [path.as_os_str()])
        };
        out.is_some_and(|s| !s.trim().is_empty())
    }

    fn pull_requests(&self, branch: &str) -> Option<Vec<PullRequest>> {
        let slug = self.slug.as_deref()?;
        let out = Command::new("gh")
            .args(["pr", "list", "-R", slug, "--head", branch, "--state", "all"])
            .args(["--json", "state,headRefOid", "--limit", "100"])
            .current_dir("/")
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        serde_json::from_slice(&out.stdout).ok()
    }

    fn free_human(&self) -> Option<String> {
        let df = stdout_of("df", ["-h", "/"])?;
        Some(df.lines().nth(1)?.split_whitespace().nth(3)?.to_string())
    }

    fn free_gb(&self, path: &Path) -> Option<u64> {
        let df = stdout_of("df", [OsStr::new("-Pk"), path.as_os_str()])?;
        let kb: u64 = df.lines().nth(1)?.split_whitespace().nth(3)?.parse().ok()?;
        Some(kb / 1024 / 1024)
    }

    fn prune_uv(&self) -> Option<bool> {
        let uv = on_path("uv")?;
        let status = Command::new(uv)
            .args(["cache", "prune"])
            .env("UV_LOCK_TIMEOUT", "5")
            .current_dir("/")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        Some(status.map(|s| s.success()).unwrap_or(false))
    }
}

/// A program's stdout whatever its exit status: `lsof` and `pgrep` exit 1 when they find nothing,
/// and `lsof +D` exits 1 on an unreadable subfolder while still listing what it found. Run from `/`,
/// so the helper's own working directory is never inside a path it is asked about.
fn stdout_of<I, S>(program: &str, args: I) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let out = Command::new(program)
        .args(args)
        .current_dir("/")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The first executable called `name` on `PATH`, as `command -v` finds it.
fn on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|d| d.join(name))
        .find(|c| {
            fs::metadata(c)
                .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        })
}

// ---------------------------------------------------------------------------------------------
// git, always with the GIT_* variables removed.
// ---------------------------------------------------------------------------------------------

/// `git -C <dir> <args>` with every `GIT_*` variable removed from its environment.
fn git_command<I, S>(dir: &Path, args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(dir).args(args).stdin(Stdio::null());
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(&k);
        }
    }
    cmd
}

/// Stdout of a successful git command, trailing whitespace trimmed; `None` when it failed.
fn git<I, S>(dir: &Path, args: I) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let out = git_command(dir, args).stderr(Stdio::null()).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim_end().to_string())
}

/// A git command that changes something: `Err` carries git's own message.
fn git_run<I, S>(dir: &Path, args: I) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let out = git_command(dir, args)
        .output()
        .map_err(|e| format!("git did not run: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

// ---------------------------------------------------------------------------------------------
// Facts on disk.
// ---------------------------------------------------------------------------------------------

/// Visit `root` and everything beneath it without following symbolic links, as `find` and `du` do
/// by default. `visit` returns false to stop. Unreadable entries are skipped, as `2>/dev/null` did.
fn walk(root: &Path, mut visit: impl FnMut(&fs::Metadata) -> bool) {
    let Ok(md) = fs::symlink_metadata(root) else {
        return;
    };
    if !visit(&md) || !md.is_dir() {
        return;
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            // `DirEntry::metadata` does not follow a symbolic link on Unix.
            let Ok(md) = entry.metadata() else {
                continue;
            };
            if !visit(&md) {
                return;
            }
            if md.is_dir() {
                stack.push(entry.path());
            }
        }
    }
}

/// BSD `find`'s `-mmin -n` and `-mtime -n`: the age, rounded UP to whole periods, is under `n`. A
/// file dated in the future counts as recent.
pub fn within(mtime: i64, now: i64, period: i64, n: i64) -> bool {
    let age = now - mtime;
    age <= 0 || (age + period - 1) / period < n
}

/// Some regular file at or beneath `path` was modified within `n` periods.
pub fn has_recent_file(path: &Path, now: i64, period: i64, n: i64) -> bool {
    let mut found = false;
    walk(path, |md| {
        if md.is_file() && within(md.mtime(), now, period, n) {
            found = true;
            return false;
        }
        true
    });
    found
}

/// Nothing beneath `path` was modified in the last `idle_hours`.
pub fn idle(path: &Path, now: i64, idle_hours: i64) -> bool {
    !has_recent_file(path, now, MINUTE, idle_hours * 60)
}

/// `du -sk`: kilobytes allocated at and beneath `path`, a hard-linked file counted once.
pub fn disk_kb(path: &Path) -> u64 {
    let mut seen = HashSet::new();
    let mut blocks: u64 = 0;
    walk(path, |md| {
        if md.nlink() > 1 && !md.is_dir() && !seen.insert((md.dev(), md.ino())) {
            return true;
        }
        blocks += md.blocks();
        true
    });
    blocks.div_ceil(2)
}

fn keep_local(path: &Path) -> bool {
    path.join(".keep-local").exists()
}

/// The first line of a file, as `head -1` prints it; empty when it cannot be read.
fn first_line(path: &Path) -> String {
    let mut line = String::new();
    if let Ok(f) = fs::File::open(path) {
        let _ = std::io::BufReader::new(f).read_line(&mut line);
    }
    line.trim_end_matches(['\n', '\r']).to_string()
}

/// The entries of `dir` whose names pass `keep`, sorted as a shell glob lists them.
fn children(dir: &Path, keep: impl Fn(&str) -> bool) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| keep(&e.file_name().to_string_lossy()))
        .map(|e| e.path())
        .collect();
    found.sort();
    found
}

fn not_hidden(name: &str) -> bool {
    !name.starts_with('.')
}

/// What a guard decided for one candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Not a candidate: nothing is said about it.
    Skip,
    /// A guard refused; the reason is printed.
    Keep(String),
    /// Every guard passed.
    Act,
}

// ---------------------------------------------------------------------------------------------
// 1. Merged, clean worktrees.
// ---------------------------------------------------------------------------------------------

/// One record of `git worktree list --porcelain`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Worktree {
    pub path: PathBuf,
    pub head: String,
    /// The branch without `refs/heads/`; `None` when HEAD is detached.
    pub branch: Option<String>,
    pub locked: bool,
    pub bare: bool,
    /// The first record: the main working tree, which is never removed.
    pub main: bool,
}

pub fn parse_worktrees(porcelain: &str) -> Vec<Worktree> {
    let mut found: Vec<Worktree> = Vec::new();
    let mut current: Option<Worktree> = None;
    for line in porcelain.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            found.extend(current.take());
            current = Some(Worktree {
                path: PathBuf::from(path),
                main: found.is_empty(),
                ..Worktree::default()
            });
        } else if let Some(wt) = current.as_mut() {
            if let Some(head) = line.strip_prefix("HEAD ") {
                wt.head = head.to_string();
            } else if let Some(branch) = line.strip_prefix("branch ") {
                wt.branch = Some(
                    branch
                        .strip_prefix("refs/heads/")
                        .unwrap_or(branch)
                        .to_string(),
                );
            } else if line == "locked" || line.starts_with("locked ") {
                wt.locked = true;
            } else if line == "bare" {
                wt.bare = true;
            }
        }
    }
    found.extend(current);
    found
}

/// The repository's worktrees as git lists them now.
pub fn worktrees(repo: &Path) -> Vec<Worktree> {
    parse_worktrees(&git(repo, ["worktree", "list", "--porcelain"]).unwrap_or_default())
}

/// Gitignored entries other than `target/` (local-only work data). `None` when git failed.
pub fn ignored_local_data(worktree: &Path) -> Option<bool> {
    let status = git(
        worktree,
        [
            "status",
            "--porcelain",
            "--ignored",
            "--untracked-files=normal",
        ],
    )?;
    Some(
        status
            .lines()
            .filter_map(|l| l.strip_prefix("!! "))
            .any(|p| !p.starts_with("target/")),
    )
}

/// The guards on one worktree, in the bash order.
pub fn decide_worktree(
    wt: &Worktree,
    repo: &Path,
    main_worktree: &Path,
    sys: &dyn System,
) -> Verdict {
    if wt.main || wt.bare || wt.path == main_worktree || !wt.path.is_dir() {
        return Verdict::Skip;
    }
    let branch = wt.branch.as_deref();
    if let Some(br) = branch {
        if protected_refusal(repo, br).is_some() {
            return Verdict::Keep(format!("{br} is a protected branch"));
        }
    }
    if wt.locked {
        return Verdict::Keep("locked".to_string());
    }
    if keep_local(&wt.path) {
        return Verdict::Keep(KEEP_LOCAL.to_string());
    }
    match git(
        &wt.path,
        ["status", "--porcelain", "--untracked-files=normal"],
    ) {
        None => return Verdict::Keep("git status failed".to_string()),
        Some(s) if !s.trim().is_empty() => {
            return Verdict::Keep("uncommitted or untracked files".to_string())
        }
        Some(_) => {}
    }
    // Gitignored local-only data (e.g. local/) is where the rules keep work data; never delete it
    // with the worktree. Build output under target/ is regenerable and does not count.
    match ignored_local_data(&wt.path) {
        None => return Verdict::Keep("git status --ignored failed".to_string()),
        Some(true) => {
            return Verdict::Keep(
                "gitignored local data present; upload it to the HF store first".to_string(),
            )
        }
        Some(false) => {}
    }
    // Main squash-merges, so a merged branch's HEAD is not an ancestor of main: accept either
    // ancestry or a merged PR whose head is exactly this HEAD.
    let ancestor = !wt.head.is_empty()
        && git(
            repo,
            [
                "merge-base",
                "--is-ancestor",
                wt.head.as_str(),
                "origin/main",
            ],
        )
        .is_some();
    let prs = branch.map(|br| sys.pull_requests(br));
    let merged_pr = matches!(&prs, Some(Some(list))
        if list.iter().any(|p| p.state == "MERGED" && p.head == wt.head));
    if !ancestor && !merged_pr {
        return Verdict::Keep(format!(
            "{} not merged into origin/main",
            branch.unwrap_or("detached")
        ));
    }
    // A merged HEAD can still carry an open PR (the branch moved on elsewhere); and a question
    // GitHub did not answer is not a "no".
    if let (Some(br), Some(answer)) = (branch, &prs) {
        match answer {
            None => {
                return Verdict::Keep(format!("could not ask GitHub whether {br} has an open PR"))
            }
            Some(list) if list.iter().any(|p| p.state == "OPEN") => {
                return Verdict::Keep(format!("{br} has an open PR"))
            }
            Some(_) => {}
        }
    }
    if sys.busy(&wt.path) {
        return Verdict::Keep(IN_USE.to_string());
    }
    Verdict::Act
}

/// `git worktree remove`, then `git branch -D` of its branch. The protected-branch refusal is
/// checked again here, before anything is removed, so no caller can delete a protected branch.
pub fn remove_worktree(repo: &Path, wt: &Worktree) -> Result<(), String> {
    if let Some(refusal) = wt
        .branch
        .as_deref()
        .and_then(|b| protected_refusal(repo, b))
    {
        return Err(refusal);
    }
    git_run(
        repo,
        [
            OsStr::new("worktree"),
            OsStr::new("remove"),
            wt.path.as_os_str(),
        ],
    )?;
    if let Some(br) = wt.branch.as_deref() {
        git_run(repo, ["branch", "-D", "-q", br])?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// 2. Idle build folders.
// ---------------------------------------------------------------------------------------------

/// `~/.cache/uor-*`, `<repo>/target/*`, then `target/` inside every worktree (the main one too), in
/// the order the script's `for` loop listed them.
pub fn build_candidates(home: &Path, repo: &Path, worktree_paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut found = children(&home.join(".cache"), |n| n.starts_with("uor-"));
    found.extend(children(&repo.join("target"), not_hidden));
    found.extend(worktree_paths.iter().map(|w| w.join("target")));
    found
}

pub fn decide_build(path: &Path, cfg: &Config, sys: &dyn System) -> Verdict {
    if !path.is_dir() {
        return Verdict::Skip;
    }
    if keep_local(path) {
        return Verdict::Keep(KEEP_LOCAL.to_string());
    }
    // The running binary's own build folder: removing it would only force a rebuild next run.
    if cfg.exe.as_deref().is_some_and(|e| e.starts_with(path)) {
        return Verdict::Keep("holds the running uor-harness".to_string());
    }
    if sys.busy(path) {
        return Verdict::Keep(IN_USE.to_string());
    }
    if !idle(path, cfg.now, cfg.idle_hours) {
        return Verdict::Keep(format!("written within {} h", cfg.idle_hours));
    }
    Verdict::Act
}

// ---------------------------------------------------------------------------------------------
// 3. Lab scratch already uploaded.
// ---------------------------------------------------------------------------------------------

/// `<local>/<lab>/<topic>/` folders, as the glob `"$LOCAL"/*/*/` listed them.
pub fn scratch_candidates(local: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for lab in children(local, not_hidden) {
        if lab.is_dir() {
            found.extend(
                children(&lab, not_hidden)
                    .into_iter()
                    .filter(|t| t.is_dir()),
            );
        }
    }
    found
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScratchVerdict {
    Keep(String),
    /// Reported only: no `UPLOADED` marker and nothing written for 7 days.
    Note(String),
    /// No marker, still in use within 7 days: nothing to say.
    Quiet,
    /// Every guard passed: move it to the Trash. Carries the marker's first line.
    Move {
        uploaded: String,
    },
}

pub fn decide_scratch(path: &Path, cfg: &Config, sys: &dyn System) -> ScratchVerdict {
    if keep_local(path) {
        return ScratchVerdict::Keep(KEEP_LOCAL.to_string());
    }
    // The lab writes UPLOADED after uploading to the HF store and reading the upload back.
    let marker = path.join("UPLOADED");
    if !marker.exists() {
        return if has_recent_file(path, cfg.now, DAY, NOTE_AFTER_DAYS) {
            ScratchVerdict::Quiet
        } else {
            ScratchVerdict::Note(format!(
                "idle {NOTE_AFTER_DAYS} days, no UPLOADED marker: upload what is worth keeping, \
                 then remove it"
            ))
        };
    }
    if sys.busy(path) {
        return ScratchVerdict::Keep(IN_USE.to_string());
    }
    if !idle(path, cfg.now, cfg.idle_hours) {
        return ScratchVerdict::Keep(format!("written within {} h", cfg.idle_hours));
    }
    ScratchVerdict::Move {
        uploaded: first_line(&marker),
    }
}

/// Move `path` into `trash`, never over something already there: a name that is taken gets `-2`,
/// `-3`, ... Nothing is deleted; across volumes it is `mv`'s copy-then-remove.
pub fn move_to_trash(trash: &Path, path: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(trash).map_err(|e| format!("cannot create {}: {e}", trash.display()))?;
    let name = path
        .file_name()
        .ok_or_else(|| format!("{} has no file name", path.display()))?
        .to_string_lossy()
        .into_owned();
    let mut dest = trash.join(&name);
    let mut n = 2;
    while fs::symlink_metadata(&dest).is_ok() {
        dest = trash.join(format!("{name}-{n}"));
        n += 1;
    }
    match fs::rename(path, &dest) {
        Ok(()) => Ok(dest),
        // EXDEV: the Trash is on another volume.
        Err(e) if e.raw_os_error() == Some(18) => {
            let out = Command::new("mv")
                .arg(path)
                .arg(&dest)
                .output()
                .map_err(|e| format!("mv did not run: {e}"))?;
            if out.status.success() {
                Ok(dest)
            } else {
                Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

// ---------------------------------------------------------------------------------------------
// 4. Stale temporary items.
// ---------------------------------------------------------------------------------------------

/// Names never touched: session folders, system sockets, and (from the shell original) any name
/// with a single quote in it.
pub fn temp_excluded(name: &str) -> bool {
    name.starts_with("claude-")
        || name.starts_with("com.apple.")
        || name == ".X11-unix"
        || name == ".font-unix"
        || name == "powerlog"
        || name.ends_with(".sock")
        || name.contains('\'')
}

/// Direct children of `root` owned by `uid` that are folders, or files over 50 MB. Symbolic links
/// are neither, as `find -type d` / `-type f` saw them.
pub fn temp_candidates(root: &Path, uid: u32) -> Vec<PathBuf> {
    children(root, |_| true)
        .into_iter()
        .filter(|p| {
            fs::symlink_metadata(p).is_ok_and(|m| {
                m.uid() == uid && (m.is_dir() || (m.is_file() && m.len() > TEMP_FILE_MIN_BYTES))
            })
        })
        .collect()
}

pub fn decide_temp(path: &Path, cfg: &Config, sys: &dyn System) -> Verdict {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    if temp_excluded(&name) {
        return Verdict::Skip;
    }
    if keep_local(path) {
        return Verdict::Keep(KEEP_LOCAL.to_string());
    }
    if sys.in_use(path) {
        return Verdict::Keep(IN_USE.to_string());
    }
    if !idle(path, cfg.now, cfg.idle_hours) {
        return Verdict::Keep(format!("written within {} h", cfg.idle_hours));
    }
    Verdict::Act
}

// ---------------------------------------------------------------------------------------------
// 6. Report only.
// ---------------------------------------------------------------------------------------------

/// Folders holding a `.git` directory at most three levels below `home`, outside the one clone,
/// `~/Library` and `~/.Trash` (the script's `find -maxdepth 3 -name .git -type d`).
pub fn git_dirs_under(home: &Path, repo: &Path) -> Vec<PathBuf> {
    let skip = [
        repo.to_path_buf(),
        home.join("Library"),
        home.join(".Trash"),
    ];
    let mut found = Vec::new();
    let mut stack = vec![(home.to_path_buf(), 1usize)];
    while let Some((dir, depth)) = stack.pop() {
        for entry in children(&dir, |_| true) {
            if skip.iter().any(|s| entry.starts_with(s)) {
                continue;
            }
            let Ok(md) = fs::symlink_metadata(&entry) else {
                continue;
            };
            if !md.is_dir() {
                continue;
            }
            if entry.file_name() == Some(OsStr::new(".git")) {
                found.push(dir.clone());
            } else if depth < 3 {
                stack.push((entry, depth + 1));
            }
        }
    }
    found.sort();
    found
}

/// Project material outside the one-clone layout.
pub fn layout_notes(
    cfg: &Config,
    main_worktree: &Path,
    origin_url: Option<&str>,
    worktree_paths: &[PathBuf],
) -> Vec<String> {
    let mut notes = Vec::new();
    let retired = cfg.home.join("uor-r4-local");
    if retired.is_dir() {
        notes.push(format!(
            "{} exists (retired 20 Oct 2026: upload what is worth keeping, then remove it)",
            retired.display()
        ));
    }
    if let Some(url) = origin_url.filter(|u| !u.is_empty()) {
        for clone in git_dirs_under(&cfg.home, &cfg.repo) {
            if git(&clone, ["remote", "get-url", "origin"]).as_deref() == Some(url) {
                notes.push(format!(
                    "second clone of the repository at {} (the layout has one clone, {})",
                    clone.display(),
                    cfg.repo.display()
                ));
            }
        }
    }
    let inside = cfg.repo.join(".worktrees");
    for wt in worktree_paths {
        let main = wt == &cfg.repo || wt == main_worktree;
        if !main && !(wt.starts_with(&inside) && wt != &inside) {
            notes.push(format!(
                "worktree outside {}: {}",
                inside.display(),
                wt.display()
            ));
        }
    }
    notes
}

pub fn low_space_warning(free_gb: Option<u64>) -> Option<String> {
    match free_gb {
        Some(gb) if gb < MIN_FREE_GB => {
            Some(format!("only {gb} GB free; keep at least {MIN_FREE_GB} GB"))
        }
        Some(_) => None,
        None => Some(format!(
            "could not read free space; keep at least {MIN_FREE_GB} GB"
        )),
    }
}

// ---------------------------------------------------------------------------------------------
// The run.
// ---------------------------------------------------------------------------------------------

/// One line of the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// header, worktree, build, scratch, temp, layout, caches or footer.
    pub section: &'static str,
    /// keep, would, done, FAILED, note, WARN or skip; empty for a header, footer or warning line.
    pub verb: &'static str,
    pub text: String,
}

impl Line {
    pub fn render(&self) -> String {
        if self.verb.is_empty() {
            self.text.clone()
        } else {
            format!("{:<6} {}", self.verb, self.text)
        }
    }
}

pub struct Report {
    pub apply: bool,
    pub lines: Vec<Line>,
    /// Kilobytes removed (apply) or that would be removed (dry run).
    pub freed_kb: u64,
    /// The Trash folder, when anything was moved there.
    pub trash: Option<PathBuf>,
    echo: bool,
}

impl Report {
    fn push(&mut self, section: &'static str, verb: &'static str, text: impl Into<String>) {
        let line = Line {
            section,
            verb,
            text: text.into(),
        };
        if self.echo {
            println!("{}", line.render());
            let _ = std::io::stdout().flush();
        }
        self.lines.push(line);
    }

    /// `would <desc>` in a dry run; in apply, run `work` and say `done` or `FAILED`.
    fn act(
        &mut self,
        section: &'static str,
        kb: u64,
        desc: String,
        work: impl FnOnce() -> Result<(), String>,
    ) {
        if !self.apply {
            self.freed_kb += kb;
            self.push(section, "would", desc);
            return;
        }
        match work() {
            Ok(()) => {
                self.freed_kb += kb;
                self.push(section, "done", desc);
            }
            Err(e) => self.push(section, "FAILED", format!("{desc}: {e}")),
        }
    }

    pub fn text(&self) -> String {
        self.lines.iter().map(|l| l.render() + "\n").collect()
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "mode": if self.apply { "apply" } else { "dry-run" },
            "freed_kb": self.freed_kb,
            "trash": self.trash.as_ref().map(|t| t.display().to_string()),
            "lines": self.lines.iter().map(|l| serde_json::json!({
                "section": l.section,
                "verb": l.verb,
                "text": l.text,
            })).collect::<Vec<_>>(),
        })
    }
}

/// Run every section in the script's order (6 before 5) and return the report. `echo` prints each
/// line as it is decided. Only a repository that is not a git checkout is an error.
pub fn run(cfg: &Config, sys: &dyn System, echo: bool) -> Result<Report, String> {
    let main_worktree = git(&cfg.repo, ["rev-parse", "--show-toplevel"])
        .map(PathBuf::from)
        .ok_or_else(|| {
            format!(
                "{} is not a git checkout (set UOR_REPO to the one clone)",
                cfg.repo.display()
            )
        })?;
    let mut r = Report {
        apply: cfg.apply,
        lines: Vec::new(),
        freed_kb: 0,
        trash: None,
        echo,
    };
    let free = || sys.free_human().unwrap_or_else(|| "?".to_string());
    let mode = if cfg.apply { "apply" } else { "dry-run" };
    r.push(
        "header",
        "",
        format!("== uor-hygiene {mode}  free before: {}", free()),
    );
    if git(&cfg.repo, ["fetch", "-q", "--prune", "origin"]).is_none() {
        r.push(
            "header",
            "",
            "warn: fetch failed; worktree checks use the last fetched origin/main",
        );
    }

    // 1. Merged, clean worktrees.
    for wt in worktrees(&cfg.repo) {
        match decide_worktree(&wt, &cfg.repo, &main_worktree, sys) {
            Verdict::Skip => {}
            Verdict::Keep(why) => r.push(
                "worktree",
                "keep",
                format!("worktree {} ({why})", wt.path.display()),
            ),
            Verdict::Act => {
                let kb = disk_kb(&wt.path);
                let desc = format!(
                    "remove worktree {} ({} MB, {}, in origin/main)",
                    wt.path.display(),
                    kb / 1024,
                    wt.branch.as_deref().unwrap_or("detached")
                );
                r.act("worktree", kb, desc, || remove_worktree(&cfg.repo, &wt));
            }
        }
    }
    if cfg.apply {
        let _ = git(&cfg.repo, ["worktree", "prune"]);
    }

    // 2. Idle build folders.
    let paths: Vec<PathBuf> = worktrees(&cfg.repo).into_iter().map(|w| w.path).collect();
    for d in build_candidates(&cfg.home, &cfg.repo, &paths) {
        match decide_build(&d, cfg, sys) {
            Verdict::Skip => {}
            Verdict::Keep(why) => r.push("build", "keep", format!("build {} ({why})", d.display())),
            Verdict::Act => {
                let kb = disk_kb(&d);
                let desc = format!(
                    "remove build folder {} ({} MB, regenerated by the next build)",
                    d.display(),
                    kb / 1024
                );
                r.act("build", kb, desc, || {
                    fs::remove_dir_all(&d).map_err(|e| e.to_string())
                });
            }
        }
    }

    // 3. Lab scratch already uploaded.
    for d in scratch_candidates(&cfg.local) {
        match decide_scratch(&d, cfg, sys) {
            ScratchVerdict::Quiet => {}
            ScratchVerdict::Keep(why) => {
                r.push("scratch", "keep", format!("{} ({why})", d.display()))
            }
            ScratchVerdict::Note(why) => {
                r.push("scratch", "note", format!("{} ({why})", d.display()))
            }
            ScratchVerdict::Move { uploaded } => {
                let kb = disk_kb(&d);
                let desc = format!(
                    "move {} to the Trash ({} MB, uploaded: {uploaded})",
                    d.display(),
                    kb / 1024
                );
                r.act("scratch", kb, desc, || {
                    move_to_trash(&cfg.trash, &d).map(|_| ())
                });
            }
        }
    }

    // 4. Stale temporary items.
    for root in &cfg.tmp_roots {
        for p in temp_candidates(root, cfg.uid) {
            match decide_temp(&p, cfg, sys) {
                Verdict::Skip => {}
                Verdict::Keep(why) => {
                    r.push("temp", "keep", format!("temp {} ({why})", p.display()))
                }
                Verdict::Act => {
                    let kb = disk_kb(&p);
                    let desc = format!(
                        "move stale temp {} to the Trash ({} MB, idle > {} h)",
                        p.display(),
                        kb / 1024,
                        cfg.idle_hours
                    );
                    r.act("temp", kb, desc, || {
                        move_to_trash(&cfg.trash, &p).map(|_| ())
                    });
                }
            }
        }
    }

    // 6. Report only: project material outside the one-clone layout, and low free space.
    let origin = git(&cfg.repo, ["remote", "get-url", "origin"]);
    let paths: Vec<PathBuf> = worktrees(&cfg.repo).into_iter().map(|w| w.path).collect();
    for note in layout_notes(cfg, &main_worktree, origin.as_deref(), &paths) {
        r.push("layout", "note", note);
    }
    if let Some(warning) = low_space_warning(sys.free_gb(&cfg.repo)) {
        r.push("layout", "WARN", warning);
    }

    // 5. Git and package caches.
    if cfg.apply {
        if git(&cfg.repo, ["gc", "--auto", "--quiet"]).is_some() {
            r.push("caches", "done", "git gc --auto");
        }
        match sys.prune_uv() {
            Some(true) => r.push("caches", "done", "uv cache prune"),
            Some(false) => r.push("caches", "skip", "uv cache prune (uv busy)"),
            None => {}
        }
    } else {
        r.push("caches", "would", "git gc --auto; uv cache prune");
    }

    let verb = if cfg.apply { "freed" } else { "would free" };
    let gb = r.freed_kb / 1024 / 1024;
    r.push(
        "footer",
        "",
        format!("== {verb} about {gb} GB; free now: {}", free()),
    );
    if cfg.apply && cfg.trash.is_dir() {
        r.trash = Some(cfg.trash.clone());
        r.push(
            "footer",
            "",
            format!(
                "   moved to the Trash: {} (empty the Trash to reclaim it; until then it can be \
                 restored from there)",
                cfg.trash.display()
            ),
        );
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::os::unix::fs::MetadataExt;
    use std::time::{Duration, SystemTime};

    /// What other processes and GitHub would say, as the test states it.
    #[derive(Default)]
    struct Fake {
        busy: HashSet<PathBuf>,
        in_use: HashSet<PathBuf>,
        /// Branch to PRs; a branch not listed has none. `None` is a question GitHub did not answer.
        prs: HashMap<String, Option<Vec<PullRequest>>>,
        asked: RefCell<Vec<String>>,
    }

    impl System for Fake {
        fn busy(&self, path: &Path) -> bool {
            self.busy.contains(path)
        }
        fn in_use(&self, path: &Path) -> bool {
            self.in_use.contains(path)
        }
        fn pull_requests(&self, branch: &str) -> Option<Vec<PullRequest>> {
            self.asked.borrow_mut().push(branch.to_string());
            self.prs.get(branch).cloned().unwrap_or(Some(Vec::new()))
        }
        fn free_human(&self) -> Option<String> {
            Some("99Gi".to_string())
        }
        fn free_gb(&self, _: &Path) -> Option<u64> {
            Some(99)
        }
        fn prune_uv(&self) -> Option<bool> {
            None
        }
    }

    fn pr(state: &str, head: &str) -> PullRequest {
        PullRequest {
            state: state.to_string(),
            head: head.to_string(),
        }
    }

    fn keep(why: &str) -> Verdict {
        Verdict::Keep(why.to_string())
    }

    /// Test-setup git through [`git_command`], so every `GIT_*` variable is removed; a failure fails
    /// the test.
    fn tgit(dir: &Path, args: &[&str]) -> String {
        let out = git_command(dir, args).output().expect("git runs");
        assert!(
            out.status.success(),
            "git {args:?} in {}: {}",
            dir.display(),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn s(p: &Path) -> &str {
        p.to_str().expect("temp paths are UTF-8")
    }

    fn has_branch(repo: &Path, name: &str) -> bool {
        let r = format!("refs/heads/{name}");
        git(repo, ["rev-parse", "--verify", "--quiet", r.as_str()]).is_some()
    }

    /// A temp HOME holding the one clone `~/uor-r4` (on `main`, one commit) and its `origin`, a bare
    /// repository beside HOME, fetched so `origin/main` exists.
    struct Fixture {
        _tmp: tempfile::TempDir,
        root: PathBuf,
        home: PathBuf,
        repo: PathBuf,
    }

    fn fixture() -> Fixture {
        let tmp = tempfile::tempdir().expect("tempdir");
        // Canonical, because git reports canonical paths (/var is /private/var on macOS).
        let root = fs::canonicalize(tmp.path()).expect("canonical temp dir");
        let home = root.join("home");
        let repo = home.join("uor-r4");
        let hooks = root.join("hooks");
        fs::create_dir_all(&repo).expect("mkdir repo");
        fs::create_dir_all(&hooks).expect("mkdir hooks");
        tgit(&repo, &["init", "-q", "-b", "main"]);
        let top = PathBuf::from(tgit(&repo, &["rev-parse", "--show-toplevel"]));
        assert!(
            top.starts_with(&root),
            "git resolved {top:?} outside the temp dir {root:?}"
        );
        for (k, v) in [
            ("user.email", "t@t"),
            ("user.name", "t"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", s(&hooks)),
        ] {
            tgit(&repo, &["config", k, v]);
        }
        fs::write(repo.join(".gitignore"), "target/\nlocal/\n.worktrees/\n").expect("write");
        fs::write(repo.join("README.md"), "x\n").expect("write");
        tgit(&repo, &["add", "-A"]);
        tgit(&repo, &["commit", "-qm", "init"]);
        let origin = root.join("origin.git");
        tgit(&root, &["clone", "-q", "--bare", s(&repo), s(&origin)]);
        tgit(&repo, &["remote", "add", "origin", s(&origin)]);
        tgit(&repo, &["fetch", "-q", "origin"]);
        Fixture {
            _tmp: tmp,
            root,
            home,
            repo,
        }
    }

    fn config(f: &Fixture, apply: bool) -> Config {
        let tmp = f.root.join("tmp");
        fs::create_dir_all(&tmp).expect("mkdir tmp");
        Config {
            apply,
            home: f.home.clone(),
            repo: f.repo.clone(),
            local: f.repo.join("local"),
            idle_hours: 24,
            trash: f.home.join(".Trash").join("uor-hygiene-test"),
            tmp_roots: vec![tmp],
            uid: fs::metadata(&f.root).expect("stat").uid(),
            exe: None,
            now: now_unix(),
        }
    }

    /// Write `path` (and its parents) and date it `hours` hours ago.
    fn write_aged(path: &Path, contents: &str, hours: u64) {
        fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        fs::write(path, contents).expect("write");
        let f = fs::File::options().write(true).open(path).expect("open");
        f.set_modified(SystemTime::now() - Duration::from_secs(hours * 3600))
            .expect("set mtime");
    }

    /// A linked worktree at `<repo>/.worktrees/<name>` on a new branch `claude/<name>` from main.
    fn add_worktree(f: &Fixture, name: &str) -> Worktree {
        let path = f.repo.join(".worktrees").join(name);
        let branch = format!("claude/{name}");
        tgit(
            &f.repo,
            &["worktree", "add", "-q", "-b", &branch, s(&path), "main"],
        );
        assert!(
            path.starts_with(&f.root),
            "the worktree must live in the temp dir"
        );
        find_worktree(f, &path)
    }

    fn find_worktree(f: &Fixture, path: &Path) -> Worktree {
        worktrees(&f.repo)
            .into_iter()
            .find(|w| w.path == path)
            .expect("git lists the worktree")
    }

    fn commit_in(dir: &Path, file: &str) {
        fs::write(dir.join(file), file).expect("write");
        tgit(dir, &["add", "-A"]);
        tgit(dir, &["commit", "-qm", file]);
    }

    // ---- 1. worktrees ----------------------------------------------------------------------

    #[test]
    fn porcelain_records_keep_spaces_locks_and_detached_heads() {
        let text = "worktree /r\nHEAD aaa\nbranch refs/heads/main\n\n\
                    worktree /r/.worktrees/a b\nHEAD bbb\nbranch refs/heads/claude/a\nlocked why\n\n\
                    worktree /r/.worktrees/d\nHEAD ccc\ndetached\n";
        let w = parse_worktrees(text);
        assert_eq!(w.len(), 3);
        assert!(
            w[0].main && !w[1].main && !w[2].main,
            "only the first record is main"
        );
        assert_eq!(
            w[1].path,
            PathBuf::from("/r/.worktrees/a b"),
            "a space stays in the path"
        );
        assert_eq!(w[1].branch.as_deref(), Some("claude/a"));
        assert!(w[1].locked);
        assert_eq!((w[2].branch.as_deref(), w[2].head.as_str()), (None, "ccc"));
    }

    #[test]
    fn the_main_worktree_and_a_vanished_path_are_not_candidates() {
        let f = fixture();
        let sys = Fake::default();
        let all = worktrees(&f.repo);
        assert_eq!(
            decide_worktree(&all[0], &f.repo, &f.repo, &sys),
            Verdict::Skip
        );
        let gone = Worktree {
            path: f.root.join("nowhere"),
            head: "x".to_string(),
            branch: Some("claude/x".to_string()),
            ..Worktree::default()
        };
        assert_eq!(
            decide_worktree(&gone, &f.repo, &f.repo, &sys),
            Verdict::Skip
        );
    }

    #[test]
    fn a_clean_merged_worktree_is_removed_with_its_branch() {
        let f = fixture();
        let wt = add_worktree(&f, "done");
        let sys = Fake::default();
        assert_eq!(decide_worktree(&wt, &f.repo, &f.repo, &sys), Verdict::Act);
        assert_eq!(
            *sys.asked.borrow(),
            ["claude/done"],
            "GitHub was asked about open PRs"
        );
        remove_worktree(&f.repo, &wt).expect("removes");
        assert!(!wt.path.exists());
        assert!(!has_branch(&f.repo, "claude/done"));
    }

    #[test]
    fn a_protected_branch_is_never_removed() {
        let f = fixture();
        let path = f.repo.join(".worktrees").join("pages");
        tgit(
            &f.repo,
            &["worktree", "add", "-q", "-b", "gh-pages", s(&path), "main"],
        );
        let wt = find_worktree(&f, &path);
        let sys = Fake::default();
        assert_eq!(
            decide_worktree(&wt, &f.repo, &f.repo, &sys),
            keep("gh-pages is a protected branch")
        );
        // The action refuses on its own, before it removes anything.
        let err = remove_worktree(&f.repo, &wt).expect_err("refused");
        assert!(err.contains("protected branch"), "{err}");
        assert!(path.exists() && has_branch(&f.repo, "gh-pages"));
    }

    #[test]
    fn uncommitted_or_untracked_files_keep_a_worktree() {
        let f = fixture();
        let wt = add_worktree(&f, "dirty");
        fs::write(wt.path.join("new.txt"), "x").expect("write");
        let dirty = keep("uncommitted or untracked files");
        assert_eq!(
            decide_worktree(&wt, &f.repo, &f.repo, &Fake::default()),
            dirty
        );
        fs::remove_file(wt.path.join("new.txt")).expect("rm");
        fs::write(wt.path.join("README.md"), "changed\n").expect("write");
        assert_eq!(
            decide_worktree(&wt, &f.repo, &f.repo, &Fake::default()),
            dirty
        );
    }

    #[test]
    fn gitignored_data_keeps_a_worktree_but_target_does_not() {
        let f = fixture();
        let wt = add_worktree(&f, "data");
        write_aged(&wt.path.join("target/debug/x"), "x", 0);
        assert_eq!(
            ignored_local_data(&wt.path),
            Some(false),
            "build output is regenerable"
        );
        assert_eq!(
            decide_worktree(&wt, &f.repo, &f.repo, &Fake::default()),
            Verdict::Act
        );
        write_aged(&wt.path.join("local/claude/run/out.bin"), "x", 0);
        assert_eq!(ignored_local_data(&wt.path), Some(true));
        assert_eq!(
            decide_worktree(&wt, &f.repo, &f.repo, &Fake::default()),
            keep("gitignored local data present; upload it to the HF store first")
        );
    }

    #[test]
    fn an_unmerged_branch_is_kept() {
        let f = fixture();
        let path = add_worktree(&f, "wip").path;
        commit_in(&path, "work.txt");
        let wt = find_worktree(&f, &path);
        assert_eq!(
            decide_worktree(&wt, &f.repo, &f.repo, &Fake::default()),
            keep("claude/wip not merged into origin/main")
        );
    }

    #[test]
    fn a_squash_merged_branch_is_removed_only_when_the_merged_head_is_this_head() {
        let f = fixture();
        let path = add_worktree(&f, "sq").path;
        commit_in(&path, "work.txt");
        let wt = find_worktree(&f, &path);
        let mut sys = Fake::default();
        sys.prs
            .insert("claude/sq".to_string(), Some(vec![pr("MERGED", "0000")]));
        assert_eq!(
            decide_worktree(&wt, &f.repo, &f.repo, &sys),
            keep("claude/sq not merged into origin/main"),
            "a PR merged at another head proves nothing about this one"
        );
        sys.prs
            .insert("claude/sq".to_string(), Some(vec![pr("MERGED", &wt.head)]));
        assert_eq!(decide_worktree(&wt, &f.repo, &f.repo, &sys), Verdict::Act);
    }

    #[test]
    fn an_open_pr_keeps_a_merged_worktree() {
        let f = fixture();
        let wt = add_worktree(&f, "open");
        let mut sys = Fake::default();
        sys.prs.insert(
            "claude/open".to_string(),
            Some(vec![pr("MERGED", &wt.head), pr("OPEN", "1111")]),
        );
        assert_eq!(
            decide_worktree(&wt, &f.repo, &f.repo, &sys),
            keep("claude/open has an open PR")
        );
    }

    #[test]
    fn an_unanswered_github_question_keeps_a_branch() {
        let f = fixture();
        let wt = add_worktree(&f, "offline");
        let mut sys = Fake::default();
        sys.prs.insert("claude/offline".to_string(), None);
        assert_eq!(
            decide_worktree(&wt, &f.repo, &f.repo, &sys),
            keep("could not ask GitHub whether claude/offline has an open PR")
        );
        // Unmerged and unanswered reads as unmerged, as the shell script read it.
        commit_in(&wt.path, "w.txt");
        let wt = find_worktree(&f, &wt.path);
        assert_eq!(
            decide_worktree(&wt, &f.repo, &f.repo, &sys),
            keep("claude/offline not merged into origin/main")
        );
    }

    #[test]
    fn a_busy_locked_or_marked_worktree_is_kept() {
        let f = fixture();
        let busy = add_worktree(&f, "busy");
        let mut sys = Fake::default();
        sys.busy.insert(busy.path.clone());
        assert_eq!(decide_worktree(&busy, &f.repo, &f.repo, &sys), keep(IN_USE));

        let path = add_worktree(&f, "locked").path;
        tgit(&f.repo, &["worktree", "lock", s(&path)]);
        let locked = find_worktree(&f, &path);
        assert!(locked.locked);
        assert_eq!(
            decide_worktree(&locked, &f.repo, &f.repo, &Fake::default()),
            keep("locked")
        );

        let marked = add_worktree(&f, "marked");
        fs::write(marked.path.join(".keep-local"), "").expect("write");
        assert_eq!(
            decide_worktree(&marked, &f.repo, &f.repo, &Fake::default()),
            keep(KEEP_LOCAL)
        );
    }

    #[test]
    fn a_detached_merged_worktree_is_removed_without_asking_github() {
        let f = fixture();
        let path = f.repo.join(".worktrees").join("detached");
        tgit(
            &f.repo,
            &["worktree", "add", "-q", "--detach", s(&path), "main"],
        );
        let wt = find_worktree(&f, &path);
        assert_eq!(wt.branch, None);
        let sys = Fake::default();
        assert_eq!(decide_worktree(&wt, &f.repo, &f.repo, &sys), Verdict::Act);
        assert!(
            sys.asked.borrow().is_empty(),
            "a detached HEAD has no PR to ask about"
        );
        remove_worktree(&f.repo, &wt).expect("removes");
        assert!(!path.exists());
    }

    // ---- 2. build folders ------------------------------------------------------------------

    #[test]
    fn build_candidates_follow_the_scripts_globs() {
        let f = fixture();
        for d in [
            "home/.cache/uor-a",
            "home/.cache/other",
            "home/uor-r4/target/claude",
            "home/uor-r4/target/.hidden",
        ] {
            fs::create_dir_all(f.root.join(d)).expect("mkdir");
        }
        let wts = [f.repo.clone(), f.repo.join(".worktrees/x")];
        assert_eq!(
            build_candidates(&f.home, &f.repo, &wts),
            [
                f.home.join(".cache/uor-a"),
                f.repo.join("target/claude"),
                f.repo.join("target"),
                f.repo.join(".worktrees/x/target"),
            ]
        );
    }

    #[test]
    fn a_build_folder_must_be_idle_and_unused() {
        let f = fixture();
        let cfg = config(&f, false);
        let d = f.repo.join("target/claude");
        assert_eq!(
            decide_build(&d, &cfg, &Fake::default()),
            Verdict::Skip,
            "not there"
        );
        write_aged(&d.join("debug/old.rlib"), "x", 48);
        assert_eq!(decide_build(&d, &cfg, &Fake::default()), Verdict::Act);
        let mut sys = Fake::default();
        sys.busy.insert(d.clone());
        assert_eq!(decide_build(&d, &cfg, &sys), keep(IN_USE));
        write_aged(&d.join("debug/new.rlib"), "x", 1);
        assert_eq!(
            decide_build(&d, &cfg, &Fake::default()),
            keep("written within 24 h")
        );
    }

    #[test]
    fn a_marked_build_folder_and_the_running_binarys_own_are_kept() {
        let f = fixture();
        let mut cfg = config(&f, false);
        let d = f.repo.join("target/claude");
        write_aged(&d.join("debug/uor-harness"), "x", 48);
        cfg.exe = Some(d.join("debug/uor-harness"));
        assert_eq!(
            decide_build(&d, &cfg, &Fake::default()),
            keep("holds the running uor-harness")
        );
        cfg.exe = None;
        write_aged(&d.join(".keep-local"), "", 48);
        assert_eq!(decide_build(&d, &cfg, &Fake::default()), keep(KEEP_LOCAL));
    }

    // ---- 3. scratch ------------------------------------------------------------------------

    #[test]
    fn scratch_candidates_are_lab_topic_folders() {
        let f = fixture();
        let local = f.repo.join("local");
        write_aged(&local.join("claude/run-a/out"), "x", 0);
        write_aged(&local.join("claude/stray-file"), "x", 0);
        write_aged(&local.join("kimi/run-b/out"), "x", 0);
        fs::create_dir_all(local.join(".hidden/x")).expect("mkdir");
        assert_eq!(
            scratch_candidates(&local),
            [local.join("claude/run-a"), local.join("kimi/run-b")]
        );
    }

    #[test]
    fn scratch_moves_only_with_an_uploaded_marker_when_idle_and_unused() {
        let f = fixture();
        let cfg = config(&f, false);
        let d = f.repo.join("local/claude/run");
        write_aged(&d.join("out.bin"), "x", 48);
        assert_eq!(
            decide_scratch(&d, &cfg, &Fake::default()),
            ScratchVerdict::Quiet,
            "no marker and touched this week: nothing to say"
        );
        write_aged(
            &d.join("UPLOADED"),
            "results/claude/run sha256 ok\nmore\n",
            48,
        );
        assert_eq!(
            decide_scratch(&d, &cfg, &Fake::default()),
            ScratchVerdict::Move {
                uploaded: "results/claude/run sha256 ok".to_string()
            }
        );
        let mut sys = Fake::default();
        sys.busy.insert(d.clone());
        assert_eq!(
            decide_scratch(&d, &cfg, &sys),
            ScratchVerdict::Keep(IN_USE.to_string())
        );
        write_aged(&d.join("late.log"), "x", 1);
        assert_eq!(
            decide_scratch(&d, &cfg, &Fake::default()),
            ScratchVerdict::Keep("written within 24 h".to_string())
        );
        write_aged(&d.join(".keep-local"), "", 48);
        assert_eq!(
            decide_scratch(&d, &cfg, &Fake::default()),
            ScratchVerdict::Keep(KEEP_LOCAL.to_string())
        );
    }

    #[test]
    fn scratch_without_a_marker_is_noted_after_seven_idle_days() {
        let f = fixture();
        let cfg = config(&f, false);
        let d = f.repo.join("local/kimi/old");
        write_aged(&d.join("out"), "x", 8 * 24);
        assert_eq!(
            decide_scratch(&d, &cfg, &Fake::default()),
            ScratchVerdict::Note(
                "idle 7 days, no UPLOADED marker: upload what is worth keeping, then remove it"
                    .to_string()
            )
        );
        assert!(d.join("out").exists(), "a note changes nothing");
    }

    #[test]
    fn the_trash_is_never_overwritten_and_nothing_is_deleted() {
        let f = fixture();
        let trash = f.home.join(".Trash/uor-hygiene-t");
        let a = f.repo.join("local/claude/run");
        let b = f.repo.join("local/kimi/run");
        write_aged(&a.join("x"), "a", 0);
        write_aged(&b.join("x"), "b", 0);
        let into_a = move_to_trash(&trash, &a).expect("moves");
        let into_b = move_to_trash(&trash, &b).expect("moves");
        assert_eq!(
            (into_a.clone(), into_b.clone()),
            (trash.join("run"), trash.join("run-2"))
        );
        assert_eq!(fs::read_to_string(into_a.join("x")).expect("read"), "a");
        assert_eq!(fs::read_to_string(into_b.join("x")).expect("read"), "b");
        assert!(!a.exists() && !b.exists());
    }

    // ---- 4. temp ---------------------------------------------------------------------------

    #[test]
    fn temp_names_that_are_never_touched() {
        for n in [
            "claude-501",
            "com.apple.launchd.x",
            ".X11-unix",
            ".font-unix",
            "powerlog",
            "agent.sock",
            "it's",
        ] {
            assert!(temp_excluded(n), "{n}");
        }
        for n in ["x", "node-compile-cache", "claude", "sock"] {
            assert!(!temp_excluded(n), "{n}");
        }
    }

    #[test]
    fn temp_candidates_are_your_folders_and_files_over_50_mb() {
        let f = fixture();
        let tmp = f.root.join("tmp");
        fs::create_dir_all(tmp.join("dir")).expect("mkdir");
        fs::write(tmp.join("small"), "x").expect("write");
        // Sparse, so the test costs no disk.
        let big = fs::File::create(tmp.join("big")).expect("create");
        big.set_len(TEMP_FILE_MIN_BYTES + 1).expect("set_len");
        let exact = fs::File::create(tmp.join("exact")).expect("create");
        exact.set_len(TEMP_FILE_MIN_BYTES).expect("set_len");
        std::os::unix::fs::symlink(tmp.join("dir"), tmp.join("link")).expect("symlink");
        let uid = fs::metadata(&tmp).expect("stat").uid();
        assert_eq!(
            temp_candidates(&tmp, uid),
            [tmp.join("big"), tmp.join("dir")]
        );
        assert!(
            temp_candidates(&tmp, uid.wrapping_add(1)).is_empty(),
            "another user's items are never candidates"
        );
    }

    #[test]
    fn a_temp_item_must_be_idle_and_not_open() {
        let f = fixture();
        let cfg = config(&f, false);
        let d = cfg.tmp_roots[0].join("stale");
        write_aged(&d.join("f"), "x", 48);
        assert_eq!(decide_temp(&d, &cfg, &Fake::default()), Verdict::Act);
        let mut sys = Fake::default();
        sys.in_use.insert(d.clone());
        assert_eq!(decide_temp(&d, &cfg, &sys), keep(IN_USE));
        write_aged(&d.join("g"), "x", 1);
        assert_eq!(
            decide_temp(&d, &cfg, &Fake::default()),
            keep("written within 24 h")
        );
        let session = cfg.tmp_roots[0].join("claude-501");
        write_aged(&session.join("f"), "x", 48);
        assert_eq!(decide_temp(&session, &cfg, &Fake::default()), Verdict::Skip);
    }

    // ---- 6. report only --------------------------------------------------------------------

    #[test]
    fn layout_notes_name_retired_folders_second_clones_and_stray_worktrees() {
        let f = fixture();
        let cfg = config(&f, false);
        let origin = tgit(&f.repo, &["remote", "get-url", "origin"]);
        assert!(
            layout_notes(&cfg, &f.repo, Some(&origin), std::slice::from_ref(&f.repo)).is_empty(),
            "the layout as written has nothing to report"
        );
        fs::create_dir_all(f.home.join("uor-r4-local")).expect("mkdir");
        let origin_dir = f.root.join("origin.git");
        // ~/code/uor-r4/.git is three levels down: found. ~/a/b/uor-r4/.git is four: not.
        let clone = f.home.join("code/uor-r4");
        let deep = f.home.join("a/b/uor-r4");
        for dir in [&clone, &deep] {
            tgit(&f.root, &["clone", "-q", s(&origin_dir), s(dir)]);
            assert!(dir.starts_with(&f.root), "clones live in the temp dir");
        }
        let unrelated = f.home.join("code/other");
        fs::create_dir_all(&unrelated).expect("mkdir");
        tgit(&unrelated, &["init", "-q"]);
        let stray = f.root.join("elsewhere/wt");
        let notes = layout_notes(
            &cfg,
            &f.repo,
            Some(&origin),
            &[f.repo.clone(), f.repo.join(".worktrees/ok"), stray.clone()],
        );
        assert_eq!(
            notes,
            [
                format!(
                    "{} exists (retired 20 Oct 2026: upload what is worth keeping, then remove it)",
                    f.home.join("uor-r4-local").display()
                ),
                format!(
                    "second clone of the repository at {} (the layout has one clone, {})",
                    clone.display(),
                    f.repo.display()
                ),
                format!(
                    "worktree outside {}: {}",
                    f.repo.join(".worktrees").display(),
                    stray.display()
                ),
            ]
        );
    }

    #[test]
    fn free_space_under_thirty_gb_warns() {
        assert_eq!(
            low_space_warning(Some(29)).as_deref(),
            Some("only 29 GB free; keep at least 30 GB")
        );
        assert_eq!(low_space_warning(Some(30)), None);
        assert!(
            low_space_warning(None).is_some(),
            "an unread disk is not a healthy one"
        );
    }

    // ---- helpers ---------------------------------------------------------------------------

    #[test]
    fn ages_round_up_to_whole_periods_as_bsd_find_does() {
        let now = 1_000_000;
        // `-mmin -1440`: 23 h 59 min is recent; one second more rounds up to 1440 minutes.
        assert!(within(now - 1439 * 60, now, MINUTE, 1440));
        assert!(!within(now - 1439 * 60 - 1, now, MINUTE, 1440));
        // `-mtime -7`: six days is recent; six and a half rounds up to seven.
        assert!(within(now - 6 * DAY, now, DAY, 7));
        assert!(!within(now - 6 * DAY - DAY / 2, now, DAY, 7));
        assert!(
            within(now + 10, now, MINUTE, 1),
            "a file dated in the future is recent"
        );
    }

    #[test]
    fn disk_use_counts_a_hard_linked_file_once() {
        let f = fixture();
        let d = f.root.join("du");
        fs::create_dir_all(&d).expect("mkdir");
        fs::write(d.join("a"), vec![1u8; 256 * 1024]).expect("write");
        let once = disk_kb(&d);
        assert!(once >= 256, "{once} KB for a 256 KB file");
        fs::hard_link(d.join("a"), d.join("b")).expect("link");
        assert_eq!(disk_kb(&d), once, "the second name adds no blocks");
    }

    #[test]
    fn the_github_slug_comes_from_https_and_ssh_remotes() {
        for url in [
            "https://github.com/UOR-Foundation/uor-r4.git",
            "git@github.com:UOR-Foundation/uor-r4.git",
            "https://github.com/UOR-Foundation/uor-r4",
        ] {
            assert_eq!(
                github_slug(url).as_deref(),
                Some("UOR-Foundation/uor-r4"),
                "{url}"
            );
        }
        assert_eq!(github_slug("/tmp/origin.git"), None);
    }

    #[test]
    fn only_cargo_and_rustc_count_as_builders() {
        for comm in [
            "cargo",
            "rustc",
            "/Users/x/.cargo/bin/cargo",
            "/t/bin/rustc",
        ] {
            assert!(is_cargo_or_rustc(comm), "{comm}");
        }
        for comm in [
            "cargo-watch",
            "/t/bin/cargo-clippy",
            "/usr/bin/xcargo",
            "rustc-wrapper",
        ] {
            assert!(!is_cargo_or_rustc(comm), "{comm}");
        }
    }

    // ---- the whole run ---------------------------------------------------------------------

    /// A dry run reports what apply then does and changes nothing; apply removes what is merged
    /// and idle, moves scratch and temp to the Trash, and leaves what a guard protects.
    #[test]
    fn a_dry_run_changes_nothing_and_apply_does_what_it_reported() {
        let f = fixture();
        let done = add_worktree(&f, "done");
        let wip = add_worktree(&f, "wip");
        commit_in(&wip.path, "w.txt");
        let cache = f.home.join(".cache/uor-old");
        write_aged(&cache.join("blob"), "x", 48);
        let fresh = f.repo.join("target/claude");
        write_aged(&fresh.join("x"), "x", 1);
        let uploaded = f.repo.join("local/claude/run");
        write_aged(&uploaded.join("out"), "x", 48);
        write_aged(
            &uploaded.join("UPLOADED"),
            "results/claude/run sha256 ok\n",
            48,
        );
        let unmarked = f.repo.join("local/kimi/old");
        write_aged(&unmarked.join("out"), "x", 10 * 24);
        let dry_cfg = config(&f, false);
        let stale = dry_cfg.tmp_roots[0].join("stale");
        write_aged(&stale.join("f"), "x", 48);
        let session = dry_cfg.tmp_roots[0].join("claude-501");
        write_aged(&session.join("f"), "x", 48);

        let sys = Fake::default();
        let dry = run(&dry_cfg, &sys, false).expect("dry run");
        let text = dry.text();
        for want in [
            "== uor-hygiene dry-run  free before: 99Gi".to_string(),
            format!("would  remove worktree {} (", done.path.display()),
            format!(
                "keep   worktree {} (claude/wip not merged into origin/main)",
                wip.path.display()
            ),
            format!("would  remove build folder {} (", cache.display()),
            format!("keep   build {} (written within 24 h)", fresh.display()),
            format!(
                "would  move {} to the Trash (0 MB, uploaded: results/claude/run sha256 ok)",
                uploaded.display()
            ),
            format!(
                "note   {} (idle 7 days, no UPLOADED marker",
                unmarked.display()
            ),
            format!("would  move stale temp {} to the Trash (", stale.display()),
            "would  git gc --auto; uv cache prune".to_string(),
            "== would free about 0 GB; free now: 99Gi".to_string(),
        ] {
            assert!(text.contains(&want), "missing {want:?} in\n{text}");
        }
        assert!(
            !text.contains("claude-501"),
            "a session folder is never mentioned:\n{text}"
        );
        assert!(done.path.exists() && has_branch(&f.repo, "claude/done"));
        assert!(cache.exists() && uploaded.exists() && stale.exists());
        assert!(!f.home.join(".Trash").exists(), "a dry run moves nothing");
        assert_eq!(dry.to_json()["mode"], "dry-run");
        assert_eq!(
            dry.to_json()["lines"].as_array().map(Vec::len),
            Some(dry.lines.len())
        );

        let cfg = config(&f, true);
        let applied = run(&cfg, &sys, false).expect("apply");
        let text = applied.text();
        assert!(
            text.contains(&format!("done   remove worktree {}", done.path.display())),
            "{text}"
        );
        assert!(!done.path.exists() && !has_branch(&f.repo, "claude/done"));
        assert!(wip.path.exists() && has_branch(&f.repo, "claude/wip"));
        assert!(!cache.exists(), "an idle build folder is removed");
        assert!(fresh.exists(), "a fresh one is not");
        assert!(!uploaded.exists() && !stale.exists());
        assert_eq!(
            fs::read_to_string(cfg.trash.join("run/UPLOADED")).expect("in the Trash"),
            "results/claude/run sha256 ok\n"
        );
        assert!(
            cfg.trash.join("stale/f").exists(),
            "temp goes to the Trash, not away"
        );
        assert!(unmarked.exists() && session.exists());
        assert_eq!(applied.trash.as_deref(), Some(cfg.trash.as_path()));
        assert!(text.contains("   moved to the Trash: "), "{text}");
        assert!(!text.contains("FAILED"), "{text}");
    }
}
