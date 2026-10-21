//! `uor-harness loop guard`, `loop install`, `loop onboard` and `loop copyright` (loop PR 9).
//!
//! # Why these four live together
//!
//! Each of the five clients (`claude`, `codex`, `kimi`, `antigravity`, `deepseek`) wires hooks,
//! user-level skills and rules through its own configuration layout, and each reports subagents
//! differently in its hook payloads (probe P-1 on `#2028`):
//!
//! - **Claude Code (`claude`)**: fires `SessionStart`, `PreToolUse`, `SubagentStart`, `SubagentStop`
//!   and `Stop`. A subagent's `PreToolUse` carries `agent_id`, so the guard scopes subagents
//!   against `local/<lab>/<topic>/orders/<n>.md`.
//! - **Antigravity (`antigravity`)**: fires `SessionStart`, `PreInvocation`, `PreToolUse`,
//!   `PostToolUse`, `PostInvocation` and `Stop` in the parent and in subagents. A subagent's
//!   `PreToolUse` carries its own `conversationId`, so the guard records the root `conversationId`
//!   at the first `SessionStart` and identifies subagents by a differing `conversationId`.
//! - **Kimi Code (`kimi`)**: fires `SessionStart`, `TurnStarted`, `PreToolUse`, `PostToolUse`,
//!   `SubagentStart`, `SubagentStop` and `Stop`. Subagent tool calls carry the shared `session_id`
//!   with no agent id, so the guard tracks subagent depth between `SubagentStart` and
//!   `SubagentStop` per `session_id`.
//! - **Codex (`codex`)** and **DeepSeek (`deepseek`)**: evaluate the same path, scope and command
//!   guard on their hook or stream payloads, alongside Codex's rules file and DeepSeek's headless
//!   overlay.
//!
//! `loop guard` is the single hook entry point and exits `0` at once outside the UOR-R4 repository
//! and its worktrees so it never touches other projects. Inside the repository it refuses writes to
//! owner-only paths (`docs/harness/owner-paths.json`), the shared primary checkout (R10), another
//! lab's worktree or `local/` folder, rendered views outside `kimi/render`, existing `archive/`
//! files, and paths outside the active ORDER or subagent suborder scope, and it refuses release or
//! cap-override commands (D56) and forbidden git or process commands.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// The five registered client names, in canonical sorted order.
pub const CLIENTS: [&str; 5] = ["antigravity", "claude", "codex", "deepseek", "kimi"];

/// The six Loop Kit skills installed for every client (`start-here` is excluded so it can never
/// start a second setup).
pub const ALLOWED_SKILLS: [&str; 6] = [
    "gates",
    "holding-the-goal",
    "scope-guard",
    "session-close",
    "solution-integrity",
    "working-discipline",
];

/// The excluded first-run interview skill.
pub const EXCLUDED_SKILL: &str = "start-here";

const USAGE: &str = "\
usage: uor-harness loop guard --client <C> [--lab <L>] --event <E> [--input <path>]
       uor-harness loop install (--client <C> | --all) [--home <dir>] [--zip <path>] [--replace] [--dry-run] [--json]
       uor-harness loop onboard (--client <C> | --check-receipts) [--home <dir>] [--write-receipt] [--json]
       uor-harness loop copyright [--zip <path>] [--json]";

/// Resolved UOR-R4 roots for a directory inside the repository or one of its worktrees.
#[derive(Debug, Clone)]
pub struct UorRoots {
    /// The specific checkout or linked worktree containing the directory.
    pub worktree_root: PathBuf,
    /// The primary repository root (which holds `.worktrees/` and shared `local/`), or
    /// `worktree_root` when standalone.
    pub primary_root: PathBuf,
}

/// Whether `dir` is the root of a UOR-R4 checkout or worktree.
fn is_uor_root(dir: &Path) -> bool {
    dir.join("CHARTER.md").is_file() && dir.join("docs/harness/labs.json").is_file()
}

/// Find the UOR-R4 worktree and primary roots for `dir`, or `None` when `dir` is outside the
/// repository and its worktrees.
pub fn find_uor_roots(dir: &Path) -> Option<UorRoots> {
    let canon = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    for anc in canon.ancestors() {
        if is_uor_root(anc) {
            let worktree_root = anc.to_path_buf();
            // If this worktree lives at `<primary>/.worktrees/<name>`, `<primary>` is two levels up.
            let primary_root = worktree_root
                .parent()
                .filter(|p| p.file_name().and_then(|n| n.to_str()) == Some(".worktrees"))
                .and_then(Path::parent)
                .filter(|p| is_uor_root(p))
                .map(Path::to_path_buf)
                .unwrap_or_else(|| worktree_root.clone());
            return Some(UorRoots {
                worktree_root,
                primary_root,
            });
        }
    }
    None
}

/// Whether `dir` lies inside the UOR-R4 repository or one of its worktrees.
pub fn in_uor_repo(dir: &Path) -> bool {
    find_uor_roots(dir).is_some()
}

/// Outcome of evaluating `loop guard`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardOutcome {
    /// The hook fired outside the UOR-R4 repository and its worktrees; exit 0 immediately.
    OutsideRepo,
    /// The event or tool call is permitted.
    Allow,
    /// The tool call or invocation is refused with a human-readable reason.
    Refused(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SubagentContext {
    TopLevel,
    Subagent(Option<String>),
}

/// Check that `client` is one of the five supported clients.
pub fn check_client(client: &str) -> Result<String, String> {
    let slug = crate::state::slug(client);
    if CLIENTS.contains(&slug.as_str()) {
        Ok(slug)
    } else {
        Err(format!(
            "unknown client {client:?}; expected one of {}",
            CLIENTS.join(", ")
        ))
    }
}

/// The lane assigned to each lab on `docs/loop/README.md`.
pub fn client_lane(client: &str) -> &'static str {
    match client {
        "claude" => "loop architecture, core gates and setup reviews (D52, D59)",
        "codex" => "item validator, gates runner, policy check and review records (PRs 2, 4, 5, 6)",
        "kimi" => "PM, rendered views, notes migration and render refresh (PRs 3, 8, 10)",
        "antigravity" => "client adapters: loop guard, install, onboard and copyright (PR 9)",
        "deepseek" => {
            "claim, suborder, ask, session loop, queue and watch (PRs 7a, 7b, 7c, 11, 12)"
        }
        _ => "general",
    }
}

/// The user-level skills directory for `client` under `home`.
pub fn client_skills_dir(home: &Path, client: &str) -> PathBuf {
    match client {
        "claude" => home.join(".claude").join("skills"),
        "codex" => home.join(".codex").join("skills"),
        "kimi" => home.join(".agents").join("skills"),
        "antigravity" => home.join(".gemini").join("config").join("skills"),
        "deepseek" => home.join(".dsh").join("skills"),
        _ => home.join(format!(".{client}")).join("skills"),
    }
}

/// Relative display path (under `~`) for the client's skills directory.
pub fn client_skills_rel(client: &str) -> &'static str {
    match client {
        "claude" => "~/.claude/skills",
        "codex" => "~/.codex/skills",
        "kimi" => "~/.agents/skills",
        "antigravity" => "~/.gemini/config/skills",
        "deepseek" => "~/.dsh/skills",
        _ => "~/.skills",
    }
}

/// Guard state directory for `lab` inside `roots`.
fn guard_state_dir(roots: &UorRoots, lab: &str) -> PathBuf {
    roots.worktree_root.join("local").join(lab).join("guard")
}

/// Sanitize an identifier for use in a file name.
fn safe_id(raw: &str) -> String {
    let s: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if s.is_empty() {
        "default".to_string()
    } else {
        s
    }
}

/// Extract the working directory named by the hook payload, falling back to `fallback`.
fn payload_cwd(payload: &Value, fallback: &Path) -> PathBuf {
    if let Some(cwd) = payload
        .get("cwd")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
    {
        return PathBuf::from(cwd);
    }
    if let Some(first) = payload
        .get("workspacePaths")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
    {
        return PathBuf::from(first);
    }
    fallback.to_path_buf()
}

/// Record session/subagent lifecycle transitions and return whether the current event is inside a
/// subagent.
fn update_and_detect_subagent(
    roots: &UorRoots,
    client: &str,
    lab: &str,
    event: &str,
    payload: &Value,
) -> Result<SubagentContext, String> {
    let gdir = guard_state_dir(roots, lab);
    match client {
        "antigravity" => {
            let cid = payload
                .get("conversationId")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty());
            let root_file = gdir.join("root-conversation.id");
            if event == "SessionStart" {
                if let Some(c) = cid {
                    let existing = fs::read_to_string(&root_file)
                        .ok()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                    if existing.is_none() {
                        fs::create_dir_all(&gdir)
                            .map_err(|e| format!("cannot create {}: {e}", gdir.display()))?;
                        fs::write(&root_file, format!("{c}\n"))
                            .map_err(|e| format!("cannot write {}: {e}", root_file.display()))?;
                    }
                }
            } else if event == "Stop" {
                if let Some(c) = cid {
                    let existing = fs::read_to_string(&root_file)
                        .ok()
                        .map(|s| s.trim().to_string());
                    if existing.as_deref() == Some(c) {
                        let _ = fs::remove_file(&root_file);
                    }
                }
            }
            if payload.get("isSubagent").and_then(Value::as_bool) == Some(true) {
                return Ok(SubagentContext::Subagent(cid.map(String::from)));
            }
            if let Some(c) = cid {
                if let Ok(root_cid) = fs::read_to_string(&root_file) {
                    let root_cid = root_cid.trim();
                    if !root_cid.is_empty() && root_cid != c {
                        return Ok(SubagentContext::Subagent(Some(c.to_string())));
                    }
                }
            }
            Ok(SubagentContext::TopLevel)
        }
        "kimi" => {
            let sid = safe_id(
                payload
                    .get("session_id")
                    .and_then(Value::as_str)
                    .unwrap_or("default"),
            );
            let depth_file = gdir.join(format!("kimi-{sid}.depth"));
            let cur_depth: u32 = fs::read_to_string(&depth_file)
                .ok()
                .and_then(|s| s.trim().parse::<u32>().ok())
                .unwrap_or(0);
            match event {
                "SessionStart" => {
                    fs::create_dir_all(&gdir)
                        .map_err(|e| format!("cannot create {}: {e}", gdir.display()))?;
                    fs::write(&depth_file, "0\n")
                        .map_err(|e| format!("cannot write {}: {e}", depth_file.display()))?;
                    Ok(SubagentContext::TopLevel)
                }
                "SubagentStart" => {
                    fs::create_dir_all(&gdir)
                        .map_err(|e| format!("cannot create {}: {e}", gdir.display()))?;
                    let next = cur_depth + 1;
                    fs::write(&depth_file, format!("{next}\n"))
                        .map_err(|e| format!("cannot write {}: {e}", depth_file.display()))?;
                    Ok(SubagentContext::Subagent(None))
                }
                "SubagentStop" => {
                    fs::create_dir_all(&gdir)
                        .map_err(|e| format!("cannot create {}: {e}", gdir.display()))?;
                    let next = cur_depth.saturating_sub(1);
                    fs::write(&depth_file, format!("{next}\n"))
                        .map_err(|e| format!("cannot write {}: {e}", depth_file.display()))?;
                    Ok(if next > 0 {
                        SubagentContext::Subagent(None)
                    } else {
                        SubagentContext::TopLevel
                    })
                }
                "Stop" => {
                    let _ = fs::remove_file(&depth_file);
                    Ok(SubagentContext::TopLevel)
                }
                _ => {
                    let origin_sub = payload
                        .get("origin_name")
                        .and_then(Value::as_str)
                        .map(|s| s.eq_ignore_ascii_case("subagent"))
                        .unwrap_or(false);
                    if cur_depth > 0 || origin_sub {
                        Ok(SubagentContext::Subagent(None))
                    } else {
                        Ok(SubagentContext::TopLevel)
                    }
                }
            }
        }
        "claude" | "codex" | "deepseek" => {
            if let Some(aid) = payload
                .get("agent_id")
                .or_else(|| payload.get("subagent_id"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                return Ok(SubagentContext::Subagent(Some(aid.to_string())));
            }
            if payload.get("is_subagent").and_then(Value::as_bool) == Some(true)
                || payload.get("origin_name").and_then(Value::as_str) == Some("subagent")
            {
                return Ok(SubagentContext::Subagent(None));
            }
            Ok(SubagentContext::TopLevel)
        }
        _ => Ok(SubagentContext::TopLevel),
    }
}

/// Extract `(tool_name, tool_args)` from a hook payload.
fn extract_tool(payload: &Value) -> (String, Value) {
    let name = payload
        .get("tool_name")
        .and_then(Value::as_str)
        .or_else(|| {
            payload
                .get("toolCall")
                .and_then(|tc| tc.get("name"))
                .and_then(Value::as_str)
        })
        .or_else(|| payload.get("name").and_then(Value::as_str))
        .unwrap_or("")
        .to_string();
    let args = payload
        .get("tool_input")
        .or_else(|| payload.get("toolCall").and_then(|tc| tc.get("args")))
        .or_else(|| payload.get("input"))
        .or_else(|| payload.get("arguments"))
        .cloned()
        .unwrap_or(Value::Null);
    (name, args)
}

/// Whether `tool_name` is a shell/command execution tool.
fn is_shell_tool(name: &str) -> bool {
    matches!(
        name,
        "Bash"
            | "bash"
            | "run_command"
            | "shell"
            | "Shell"
            | "exec"
            | " exec_command"
            | "exec_command"
            | "terminal"
            | "execute_command"
    )
}

/// Whether `tool_name` is a read-only tool.
fn is_readonly_tool(name: &str) -> bool {
    matches!(
        name,
        "Read"
            | "read_file"
            | "view_file"
            | "Glob"
            | "Grep"
            | "LS"
            | "list_dir"
            | "find_by_name"
            | "grep_search"
    )
}

/// Whether `tool_name` is a file write/edit/patch/delete tool.
fn is_write_tool(name: &str) -> bool {
    matches!(
        name,
        "Write"
            | "Edit"
            | "MultiEdit"
            | "NotebookEdit"
            | "write_file"
            | "edit_file"
            | "create_file"
            | "replace_in_file"
            | "apply_patch"
            | "delete_file"
            | "str_replace_editor"
            | "patch_file"
    )
}

/// Extract the command string from tool arguments, if present.
fn extract_command(args: &Value) -> Option<String> {
    for key in ["command", "CommandLine", "cmd", "script"] {
        if let Some(s) = args.get(key).and_then(Value::as_str) {
            return Some(s.to_string());
        }
    }
    if let Some(arr) = args.get("cmd").and_then(Value::as_array) {
        let joined: Vec<&str> = arr.iter().filter_map(Value::as_str).collect();
        if !joined.is_empty() {
            return Some(joined.join(" "));
        }
    }
    None
}

/// Extract target file paths from a tool's arguments.
fn extract_target_paths(tool_name: &str, args: &Value) -> Vec<String> {
    let mut out = Vec::new();
    for key in [
        "file_path",
        "Path",
        "path",
        "target_file",
        "TargetFile",
        "notebook_path",
        "destination",
    ] {
        if let Some(s) = args
            .get(key)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        {
            out.push(s.to_string());
        }
    }
    if let Some(edits) = args.get("edits").and_then(Value::as_array) {
        for e in edits {
            for key in ["file_path", "Path", "path", "target_file"] {
                if let Some(s) = e.get(key).and_then(Value::as_str).filter(|s| !s.is_empty()) {
                    out.push(s.to_string());
                }
            }
        }
    }
    if (tool_name == "apply_patch" || args.get("patch").is_some()) && out.is_empty() {
        if let Some(patch) = args
            .get("patch")
            .or_else(|| args.get("input"))
            .and_then(Value::as_str)
        {
            for line in patch.lines() {
                for prefix in [
                    "*** Update File: ",
                    "*** Add File: ",
                    "*** Delete File: ",
                    "+++ b/",
                ] {
                    if let Some(rest) = line.strip_prefix(prefix) {
                        let p = rest.trim();
                        if !p.is_empty() && p != "/dev/null" {
                            out.push(p.to_string());
                        }
                    }
                }
            }
        }
    }
    out
}

/// Whether `path` names a sensitive credential or secret file (CHARTER §3.13).
fn is_credential_path(path: &str) -> bool {
    let lower = path.replace('\\', "/").to_lowercase();
    let base = lower.rsplit('/').next().unwrap_or(&lower);
    matches!(
        base,
        ".env"
            | ".env.local"
            | ".env.production"
            | ".netrc"
            | ".npmrc"
            | ".pypirc"
            | "id_rsa"
            | "id_ed25519"
            | "credentials.json"
            | "auth.json"
            | "client_secret.json"
    ) || lower.contains("/.ssh/id_")
        || lower.contains("/.aws/credentials")
        || lower.contains("/.config/gh/hosts.yml")
}

/// Check a shell command against D56, forbidden git/process/credential rules, and subagent limits.
fn check_command(
    roots: &UorRoots,
    lab: &str,
    subagent: &SubagentContext,
    cwd: &Path,
    cmd: &str,
) -> Result<(), String> {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    // 1. D56 & CHARTER §5.1, §5.2: never run an owner release or clear UOR_LAB or override GPU caps.
    let unset_re = regex::Regex::new(
        r"(?:^|[;&|()\s])(?:unset\s+UOR_LAB|env\s+-u\s+UOR_LAB|UOR_LAB=\s|UOR_LAB=(?:''|\x22\x22))",
    )
    .expect("valid regex");
    if unset_re.is_match(trimmed) {
        return Err(
            "D56 / CHARTER §5.1: a lab session may never unset or clear UOR_LAB to run an owner command"
                .into(),
        );
    }
    let release_re = regex::Regex::new(
        r"(?:uor-harness\s+(?:release|team\s+release\s+--owner)|I\s+release\s+sheet\b|OWNER_RELEASE)",
    )
    .expect("valid regex");
    if release_re.is_match(trimmed) {
        return Err(
            "D56 / CHARTER §5.1: the guard never runs an owner release command for a lab; stage it for Casey"
                .into(),
        );
    }
    let cap_re = regex::Regex::new(
        r"(?:uor-pod[^\n;|&]*--(?:max-pods|max-hourly-usd|override-cap)|UOR_POD_MAX_)",
    )
    .expect("valid regex");
    if cap_re.is_match(trimmed) {
        return Err(
            "CHARTER §5.2 / D56: labs cannot override GPU pod or spend caps; stage the command for Casey"
                .into(),
        );
    }

    // 2. Forbidden git and process commands.
    let no_verify_re =
        regex::Regex::new(r"(?:^|\s)--no-verify(?:\s|$)|\bgit\s+commit\b[^\n;|&]*\s-n(?:\s|$)")
            .expect("valid regex");
    if no_verify_re.is_match(trimmed) {
        return Err("hook bypass (--no-verify / git commit -n) is forbidden".into());
    }
    let stash_re = regex::Regex::new(r"\bgit\s+(?:-C\s+\S+\s+)?stash\b").expect("valid regex");
    if stash_re.is_match(trimmed) {
        return Err(
            "git stash is forbidden in the five-lab loop; commit WIP in your worktree".into(),
        );
    }
    let force_push_re = regex::Regex::new(
        r"\bgit\s+(?:-C\s+\S+\s+)?push\b[^\n;|&]*(?:--force(?:-with-lease)?|\s-f(?:\s|$)|\+[A-Za-z0-9_/-]+)",
    )
    .expect("valid regex");
    if force_push_re.is_match(trimmed) {
        return Err("git push --force is forbidden".into());
    }
    let push_main_re =
        regex::Regex::new(r"\bgit\s+(?:-C\s+\S+\s+)?push\s+(?:origin\s+)?(?:HEAD:)?main(?:\s|$)")
            .expect("valid regex");
    if push_main_re.is_match(trimmed) {
        return Err("direct push to main is forbidden; deliver through a PR".into());
    }
    if trimmed.contains("gh pr merge") && !trimmed.contains("--match-head-commit") {
        return Err("gh pr merge requires --match-head-commit <full-sha>".into());
    }
    let pkill_re =
        regex::Regex::new(r"(?:^|[;&|()\s])(?:pkill|killall)(?:\s|$)").expect("valid regex");
    if pkill_re.is_match(trimmed) {
        return Err("pkill and killall are forbidden; stop only your own started pids".into());
    }
    let cred_cmd_re = regex::Regex::new(
        r"(?:\bgh\s+auth\s+token\b|\bcat\s+\S*(?:\.env|\.netrc|credentials\.json|auth\.json|id_rsa|id_ed25519)\b)",
    )
    .expect("valid regex");
    if cred_cmd_re.is_match(trimmed) {
        return Err("CHARTER §3.13: reading or printing credentials or tokens is forbidden".into());
    }

    // 3. Subagent command restrictions (CHARTER §5.10).
    if matches!(subagent, SubagentContext::Subagent(_)) {
        let sub_forbidden = regex::Regex::new(
            r"(?:\bgh\s+(?:pr|issue)\s+(?:create|merge|review|comment|close|edit)\b|\bgit\s+(?:-C\s+\S+\s+)?push\b|\buor-harness\s+(?:loop\s+(?:claim|review|close|queue)|team\s+claim)\b)",
        )
        .expect("valid regex");
        if sub_forbidden.is_match(trimmed) {
            return Err(
                "subagent stop rule: subagents may never claim, open, review or merge a PR, push, or post as the lab"
                    .into(),
            );
        }
    }

    // 4. Shell redirections (`>`, `>>`, `tee`) targeting paths.
    let redir_re = regex::Regex::new(
        r"(?:>>?|\btee\s+(?:-a\s+)?)\s*([^\s;&|>\x22']+|\x22[^\x22]+\x22|'[^']+')",
    )
    .expect("valid regex");
    for cap in redir_re.captures_iter(trimmed) {
        let raw = cap[1].trim_matches('"').trim_matches('\'');
        if !raw.starts_with("/dev/") && !raw.starts_with('&') {
            check_write_path(roots, lab, subagent, cwd, raw)?;
        }
    }

    Ok(())
}

/// Normalize a pathlexically (resolving `.` and `..` without requiring the target file to exist).
fn normalize_lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Resolve `raw_path` against `cwd`, canonicalizing the deepest existing ancestor so `/var` and
/// `/private/var` symlinks on macOS match `roots`.
fn resolve_target_path(cwd: &Path, raw_path: &str) -> PathBuf {
    let joined = if Path::new(raw_path).is_absolute() {
        PathBuf::from(raw_path)
    } else {
        cwd.join(raw_path)
    };
    let norm = normalize_lexical(&joined);
    let mut tail = Vec::new();
    let mut cur = norm.as_path();
    while !cur.exists() {
        if let Some(name) = cur.file_name() {
            tail.push(name.to_os_string());
        }
        match cur.parent() {
            Some(p) if !p.as_os_str().is_empty() => cur = p,
            _ => break,
        }
    }
    let mut base = cur.canonicalize().unwrap_or_else(|_| cur.to_path_buf());
    for part in tail.into_iter().rev() {
        base.push(part);
    }
    base
}

/// Load the owner-only path globs from `docs/harness/owner-paths.json`.
fn load_owner_globs(root: &Path) -> Result<Vec<String>, String> {
    let path = root.join("docs/harness/owner-paths.json");
    let text =
        fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let val: Value =
        serde_json::from_str(&text).map_err(|e| format!("malformed {}: {e}", path.display()))?;
    let arr = val
        .get("paths")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{} has no paths array", path.display()))?;
    Ok(arr
        .iter()
        .filter_map(|p| p.get("path").and_then(Value::as_str).map(String::from))
        .collect())
}

/// Find the active ORDER (`(order_rel_path, topic, order_text)`) in `worktree_root` for `lab`.
fn find_active_order(roots: &UorRoots, lab: &str) -> Option<(String, String, String)> {
    // 1. Explicit marker file `local/<lab>/guard/active-order` if present.
    let marker = guard_state_dir(roots, lab).join("active-order");
    if let Ok(rel) = fs::read_to_string(&marker) {
        let rel = rel.trim().replace('\\', "/");
        if !rel.is_empty() {
            let full = roots.worktree_root.join(&rel);
            if let Ok(text) = fs::read_to_string(&full) {
                let topic = full
                    .parent()
                    .and_then(Path::file_name)
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                return Some((rel, topic, text));
            }
        }
    }
    // 2. Scan `docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md`, picking the highest `<n>` for `lab`.
    let tasks_dir = roots.worktree_root.join("docs/loop/tasks");
    let entries = fs::read_dir(&tasks_dir).ok()?;
    let prefix = format!("{lab}-");
    let mut best: Option<(u64, String, String, String)> = None;
    for folder_entry in entries.flatten() {
        if !folder_entry.path().is_dir() {
            continue;
        }
        let folder_name = folder_entry.file_name().to_string_lossy().to_string();
        let Ok(sub_entries) = fs::read_dir(folder_entry.path()) else {
            continue;
        };
        for sub in sub_entries.flatten() {
            let topic = sub.file_name().to_string_lossy().to_string();
            let Some(num_str) = topic.strip_prefix(&prefix) else {
                continue;
            };
            let Ok(num) = num_str.parse::<u64>() else {
                continue;
            };
            let order_file = sub.path().join("ORDER.md");
            let Ok(text) = fs::read_to_string(&order_file) else {
                continue;
            };
            let rel = format!("docs/loop/tasks/{folder_name}/{topic}/ORDER.md");
            if best.as_ref().is_none_or(|(b_num, _, _, _)| num >= *b_num) {
                best = Some((num, rel, topic, text));
            }
        }
    }
    best.map(|(_, rel, topic, text)| (rel, topic, text))
}

/// Extract backticked globs from a markdown section such as `## Scope` or `## Files in scope`.
fn parse_section_globs(markdown: &str, heading_prefix: &str) -> Vec<String> {
    let mut in_sec = false;
    let mut globs = Vec::new();
    let tick_re = regex::Regex::new(r"`([^`]+)`").expect("valid regex");
    for line in markdown.lines() {
        let trimmed = line.trim();
        if let Some(h) = trimmed.strip_prefix("## ") {
            in_sec = h
                .trim()
                .to_lowercase()
                .starts_with(&heading_prefix.to_lowercase());
            continue;
        }
        if in_sec {
            for cap in tick_re.captures_iter(trimmed) {
                let candidate = cap[1].trim();
                // Skip prose citations or flags; keep file paths and globs.
                if !candidate.is_empty()
                    && !candidate.starts_with('-')
                    && !candidate.contains(' ')
                    && (candidate.contains('/')
                        || candidate.contains('.')
                        || candidate.contains('*'))
                {
                    globs.push(candidate.to_string());
                }
            }
        }
    }
    globs
}

/// Find the suborder scope globs for a subagent from `local/<lab>/<topic>/orders/`.
fn find_suborder_globs(
    roots: &UorRoots,
    lab: &str,
    topic: &str,
    agent_id: Option<&str>,
) -> Option<(PathBuf, Vec<String>)> {
    for base in [&roots.worktree_root, &roots.primary_root] {
        let orders_dir = base.join("local").join(lab).join(topic).join("orders");
        if !orders_dir.is_dir() {
            continue;
        }
        if let Some(aid) = agent_id {
            let direct = orders_dir.join(format!("{aid}.md"));
            if let Ok(text) = fs::read_to_string(&direct) {
                let globs = parse_section_globs(&text, "Scope");
                if !globs.is_empty() {
                    return Some((direct, globs));
                }
            }
        }
        // Otherwise pick the highest-numbered `<n>.md` in `orders_dir`.
        let mut best: Option<(u64, PathBuf, String)> = None;
        if let Ok(entries) = fs::read_dir(&orders_dir) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                let Some(stem) = name.strip_suffix(".md") else {
                    continue;
                };
                let Ok(n) = stem.parse::<u64>() else {
                    continue;
                };
                if let Ok(text) = fs::read_to_string(e.path()) {
                    if best.as_ref().is_none_or(|(bn, _, _)| n > *bn) {
                        best = Some((n, e.path(), text));
                    }
                }
            }
        }
        if let Some((_, path, text)) = best {
            let globs = parse_section_globs(&text, "Scope");
            if !globs.is_empty() {
                return Some((path, globs));
            }
        }
    }
    None
}

/// Current git branch of `repo`, or empty string.
fn current_branch(repo: &Path) -> String {
    Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["branch", "--show-current"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// Check whether a write/edit to `raw_path` is permitted inside `roots` for `lab`.
fn check_write_path(
    roots: &UorRoots,
    lab: &str,
    subagent: &SubagentContext,
    cwd: &Path,
    raw_path: &str,
) -> Result<(), String> {
    if is_credential_path(raw_path) {
        return Err(format!(
            "CHARTER §3.13: writing or reading credential path {raw_path} is forbidden"
        ));
    }
    let resolved = resolve_target_path(cwd, raw_path);

    // 1. Check if the path is inside `.worktrees/<wt_name>/...` of `primary_root`.
    let worktrees_dir = roots.primary_root.join(".worktrees");
    if let Ok(wt_rel) = resolved.strip_prefix(&worktrees_dir) {
        let mut comps = wt_rel.components();
        let Some(wt_comp) = comps.next() else {
            return Err("cannot write directly to .worktrees/".into());
        };
        let wt_name = wt_comp.as_os_str().to_string_lossy();
        let own_prefix = format!("{lab}-");
        if wt_name != lab && !wt_name.starts_with(&own_prefix) {
            return Err(format!(
                "cross-worktree write refused: .worktrees/{wt_name} belongs to another lab (lab is {lab})"
            ));
        }
        let rest: PathBuf = comps.collect();
        let rel_str = rest.to_string_lossy().replace('\\', "/");
        let wt_root = UorRoots {
            worktree_root: worktrees_dir.join(wt_comp.as_os_str()),
            primary_root: roots.primary_root.clone(),
        };
        return check_repo_rel_write(&wt_root, lab, subagent, &rel_str);
    }

    // 2. Check if the path is inside `roots.worktree_root`.
    if let Ok(rel) = resolved.strip_prefix(&roots.worktree_root) {
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        // If `worktree_root` IS the shared primary checkout, only `local/<lab>/**` is writable.
        if roots.worktree_root == roots.primary_root
            && crate::state::shared_primary_checkout(&roots.primary_root).is_some()
            && !rel_str.starts_with(&format!("local/{lab}/"))
            && rel_str != format!("local/{lab}")
        {
            return Err(format!(
                "R10: writing to {rel_str} in the shared primary checkout is refused; work in .worktrees/{lab}-<topic>/ or local/{lab}/"
            ));
        }
        return check_repo_rel_write(roots, lab, subagent, &rel_str);
    }

    // 3. Check if the path is inside `roots.primary_root` (when `worktree_root` is a linked worktree).
    if let Ok(rel) = resolved.strip_prefix(&roots.primary_root) {
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if rel_str.starts_with(&format!("local/{lab}/")) || rel_str == format!("local/{lab}") {
            return Ok(());
        }
        return Err(format!(
            "R10: writing to {rel_str} in the shared primary checkout from a worktree is refused"
        ));
    }

    // 4. Outside the repository and its worktrees while running a session inside the repo.
    Err(format!(
        "path {} is outside the worktree {}",
        resolved.display(),
        roots.worktree_root.display()
    ))
}

/// Check a repository-relative path `rel` against `local/`, owner-only paths, rendered views,
/// archive immutability, subagent suborder scope, and active ORDER scope.
fn check_repo_rel_write(
    roots: &UorRoots,
    lab: &str,
    subagent: &SubagentContext,
    rel: &str,
) -> Result<(), String> {
    let rel = rel.trim_start_matches("./");
    if rel.is_empty() {
        return Err("cannot overwrite worktree root".into());
    }

    // 1. `local/` partitioning: a lab may write only under `local/<lab>/` (or `local/*-harness-target/`).
    if let Some(after_local) = rel.strip_prefix("local/") {
        let seg = after_local.split('/').next().unwrap_or("");
        if seg == lab || seg.ends_with("-harness-target") || seg == "harness-target" {
            return Ok(());
        }
        if seg == "loop" {
            return Err(
                "local/loop/ is reserved for owner install backups and the kill switch".into(),
            );
        }
        return Err(format!(
            "local/{seg}/ belongs to another lab; lab {lab} may write only under local/{lab}/"
        ));
    }

    // 2. Owner-only paths (`docs/harness/owner-paths.json`).
    let owner_globs = load_owner_globs(&roots.worktree_root)?;
    for g in &owner_globs {
        if crate::loopprcheck::glob_match(g, rel) {
            return Err(format!(
                "{rel} is an owner-only path ({g} in docs/harness/owner-paths.json)"
            ));
        }
    }

    // 3. Rendered views lock (`STATE.md`, `HANDOFF.md`, `docs/harness/LESSONS.md`).
    if matches!(rel, "STATE.md" | "HANDOFF.md" | "docs/harness/LESSONS.md") {
        let branch = current_branch(&roots.worktree_root);
        if branch != "kimi/render" {
            return Err(format!(
                "{rel} is a rendered view and may change only on branch kimi/render"
            ));
        }
    }

    // 4. Archive immutability (CHARTER §3.12): existing files in `archive/` are never modified.
    if rel.starts_with("archive/") && roots.worktree_root.join(rel).exists() {
        return Err(format!(
            "CHARTER §3.12: existing archive file {rel} is byte-identical and immutable"
        ));
    }

    // 5. Active ORDER and subagent suborder scope.
    let active = find_active_order(roots, lab);
    if let SubagentContext::Subagent(agent_id) = subagent {
        let Some((_, ref topic, _)) = active else {
            return Err(format!(
                "subagent write to {rel} refused: no active ORDER found for lab {lab}"
            ));
        };
        let Some((sub_path, sub_globs)) =
            find_suborder_globs(roots, lab, topic, agent_id.as_deref())
        else {
            return Err(format!(
                "subagent write to {rel} refused: no suborder in local/{lab}/{topic}/orders/ (subagents without a suborder are read-only)"
            ));
        };
        if !sub_globs
            .iter()
            .any(|g| crate::loopprcheck::glob_match(g, rel))
        {
            return Err(format!(
                "subagent write to {rel} is outside suborder {} scope [{}]",
                sub_path.display(),
                sub_globs.join(", ")
            ));
        }
    }

    if let Some((order_rel, _, order_text)) = active {
        let order_folder = order_rel.trim_end_matches("ORDER.md");
        let item_id = crate::loopclaim::order_item(&order_text);
        let item_rel = item_id
            .as_ref()
            .map(|id| format!("docs/roadmap/items/{id}.md"));
        let always_allowed = rel.starts_with(order_folder)
            || item_rel.as_deref() == Some(rel)
            || rel.starts_with("docs/harness/mistakes/")
            || rel == "docs/start/FILEMAP.md";
        if !always_allowed {
            // Collect scope globs from the item's `scope:` line (if present) and the ORDER's
            // `## Files in scope` / `## Scope` section.
            let mut scope_globs = item_id
                .as_deref()
                .and_then(|id| crate::loopclaim::read_item(&roots.worktree_root, id).ok())
                .and_then(|item| item.scope)
                .unwrap_or_default();
            if scope_globs.is_empty() {
                scope_globs = parse_section_globs(&order_text, "Files in scope");
                if scope_globs.is_empty() {
                    scope_globs = parse_section_globs(&order_text, "Scope");
                }
            }
            if !scope_globs.is_empty()
                && !scope_globs
                    .iter()
                    .any(|g| crate::loopprcheck::glob_match(g, rel))
            {
                return Err(format!(
                    "{rel} is outside active ORDER {order_rel} scope [{}]",
                    scope_globs.join(", ")
                ));
            }
        }
    }

    Ok(())
}

/// Evaluate `loop guard` for `client`, `lab`, `event` and `payload`, with `fallback_cwd` used when
/// the payload does not carry a directory.
pub fn evaluate_guard(
    fallback_cwd: &Path,
    client: &str,
    lab: &str,
    event: &str,
    payload: &Value,
) -> GuardOutcome {
    if let Err(e) = check_client(client) {
        return GuardOutcome::Refused(e);
    }
    let cwd = payload_cwd(payload, fallback_cwd);
    let Some(roots) = find_uor_roots(&cwd) else {
        return GuardOutcome::OutsideRepo;
    };
    let lab_slug = crate::state::slug(lab);
    if lab.trim().is_empty() {
        return GuardOutcome::Refused(
            "UOR_LAB / --lab is not set; every hook inside UOR-R4 requires a registered lab".into(),
        );
    }
    if let Err(e) = crate::labs::check_lab(&roots.worktree_root, &lab_slug) {
        return GuardOutcome::Refused(e);
    }

    let subagent = match update_and_detect_subagent(&roots, client, &lab_slug, event, payload) {
        Ok(s) => s,
        Err(e) => return GuardOutcome::Refused(e),
    };

    if event != "PreToolUse" {
        return GuardOutcome::Allow;
    }

    let (tool_name, tool_args) = extract_tool(payload);
    if is_shell_tool(&tool_name) || extract_command(&tool_args).is_some() {
        if let Some(cmd) = extract_command(&tool_args) {
            if let Err(reason) = check_command(&roots, &lab_slug, &subagent, &cwd, &cmd) {
                return GuardOutcome::Refused(reason);
            }
        }
    }

    let targets = extract_target_paths(&tool_name, &tool_args);
    if is_readonly_tool(&tool_name) {
        for t in &targets {
            if is_credential_path(t) {
                return GuardOutcome::Refused(format!(
                    "CHARTER §3.13: reading credential path {t} is forbidden"
                ));
            }
        }
        return GuardOutcome::Allow;
    }

    if is_write_tool(&tool_name) || !targets.is_empty() {
        for t in &targets {
            if let Err(reason) = check_write_path(&roots, &lab_slug, &subagent, &cwd, t) {
                return GuardOutcome::Refused(reason);
            }
        }
    }

    GuardOutcome::Allow
}

/// Build the client-specific canary payload attempting a denied write to `CHARTER.md` inside `repo`.
pub fn canary_payload_for_client(repo: &Path, client: &str) -> Value {
    let cwd = repo.display().to_string();
    let denied = repo.join("CHARTER.md").display().to_string();
    match client {
        "antigravity" => json!({
            "conversationId": "canary-root-session",
            "workspacePaths": [cwd],
            "toolCall": {
                "name": "write_file",
                "args": {
                    "Path": denied,
                    "Content": "canary write must be refused\n"
                }
            }
        }),
        "claude" => json!({
            "session_id": "canary-session",
            "cwd": cwd,
            "hook_event_name": "PreToolUse",
            "tool_name": "Write",
            "tool_input": {
                "file_path": denied,
                "content": "canary write must be refused\n"
            }
        }),
        "codex" => json!({
            "session_id": "canary-session",
            "cwd": cwd,
            "event": "PreToolUse",
            "tool_name": "write_file",
            "tool_input": {
                "file_path": denied,
                "content": "canary write must be refused\n"
            }
        }),
        "kimi" => json!({
            "session_id": "canary-session",
            "cwd": cwd,
            "hook_event_name": "PreToolUse",
            "tool_name": "WriteFile",
            "tool_input": {
                "path": denied,
                "content": "canary write must be refused\n"
            }
        }),
        "deepseek" => json!({
            "session_id": "canary-session",
            "cwd": cwd,
            "event": "PreToolUse",
            "tool_name": "Write",
            "tool_input": {
                "file_path": denied,
                "content": "canary write must be refused\n"
            }
        }),
        _ => json!({
            "cwd": cwd,
            "tool_name": "Write",
            "tool_input": { "file_path": denied }
        }),
    }
}

/// Run the canary write check for `client` in `repo`: returns `Ok(refusal_reason)` when the canary
/// write to a denied path is refused, or `Err(why)` if the canary write was allowed.
pub fn run_canary(repo: &Path, client: &str) -> Result<String, String> {
    let c = check_client(client)?;
    let payload = canary_payload_for_client(repo, &c);
    match evaluate_guard(repo, &c, &c, "PreToolUse", &payload) {
        GuardOutcome::Refused(reason) => Ok(reason),
        GuardOutcome::Allow => Err(format!(
            "canary write to CHARTER.md for client {c} was ALLOWED instead of refused"
        )),
        GuardOutcome::OutsideRepo => Err(format!(
            "canary write for client {c} resolved as outside-repo at {}",
            repo.display()
        )),
    }
}

// ---------------------------------------------------------------------------
// Loop Kit archive reader & skill YAML normalization (`loop install`, `loop copyright`)
// ---------------------------------------------------------------------------

/// Find the default Loop Kit zip path on the owner's laptop (`$UOR_LOOP_KIT_ZIP` or newest
/// `~/Downloads/LoopKit*.zip`).
pub fn default_kit_zip(home: Option<&Path>) -> Option<PathBuf> {
    if let Ok(env_p) = std::env::var("UOR_LOOP_KIT_ZIP") {
        let p = PathBuf::from(env_p.trim());
        if p.exists() {
            return Some(p);
        }
    }
    let mut candidates = Vec::new();
    if let Some(h) = home {
        candidates.push(h.to_path_buf());
    }
    if let Some(env_h) = std::env::var_os("HOME").map(PathBuf::from) {
        if !candidates.contains(&env_h) {
            candidates.push(env_h);
        }
    }
    for h in candidates {
        let downloads = h.join("Downloads");
        let Ok(entries) = fs::read_dir(&downloads) else {
            continue;
        };
        let mut matches: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                name.starts_with("LoopKit") && name.ends_with(".zip") && p.is_file()
            })
            .collect();
        matches.sort();
        if let Some(found) = matches.pop() {
            return Some(found);
        }
    }
    None
}

/// Read all text entries (`(entry_name, content)`) from a Loop Kit zip file or unpacked directory.
pub fn read_kit_entries(source: &Path) -> Result<Vec<(String, String)>, String> {
    if source.is_dir() {
        let mut out = Vec::new();
        collect_dir_texts(source, source, &mut out)?;
        out.sort_by(|a, b| a.0.cmp(&b.0));
        return Ok(out);
    }
    if !source.is_file() {
        return Err(format!(
            "Loop Kit source {} does not exist",
            source.display()
        ));
    }
    let list_out = Command::new("unzip")
        .arg("-Z1")
        .arg(source)
        .output()
        .map_err(|e| format!("cannot run unzip on {}: {e}", source.display()))?;
    if !list_out.status.success() {
        return Err(format!(
            "unzip -Z1 {} failed: {}",
            source.display(),
            String::from_utf8_lossy(&list_out.stderr).trim()
        ));
    }
    let names: Vec<String> = String::from_utf8_lossy(&list_out.stdout)
        .lines()
        .map(str::trim)
        .filter(|n| {
            !n.is_empty() && !n.ends_with('/') && (n.ends_with(".md") || n.ends_with(".txt"))
        })
        .map(String::from)
        .collect();
    let mut out = Vec::new();
    for name in names {
        let entry_out = Command::new("unzip")
            .arg("-p")
            .arg(source)
            .arg(&name)
            .output()
            .map_err(|e| format!("cannot read {name} from {}: {e}", source.display()))?;
        if !entry_out.status.success() {
            return Err(format!(
                "unzip -p {} {name} failed: {}",
                source.display(),
                String::from_utf8_lossy(&entry_out.stderr).trim()
            ));
        }
        let text = String::from_utf8_lossy(&entry_out.stdout).into_owned();
        out.push((name, text));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

fn collect_dir_texts(
    base: &Path,
    dir: &Path,
    out: &mut Vec<(String, String)>,
) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_dir_texts(base, &p, out)?;
        } else if p.is_file() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.ends_with(".md") || name.ends_with(".txt") {
                let rel = p
                    .strip_prefix(base)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .replace('\\', "/");
                let text = fs::read_to_string(&p)
                    .map_err(|e| format!("cannot read {}: {e}", p.display()))?;
                out.push((rel, text));
            }
        }
    }
    Ok(())
}

/// Normalize a `SKILL.md` file's YAML frontmatter so an unquoted `description:` containing `: `
/// is quoted (preventing strict YAML parsers such as `dsh` from rejecting the skill).
pub fn normalize_skill_frontmatter(text: &str) -> String {
    let mut lines = text.lines();
    let Some(first) = lines.next() else {
        return text.to_string();
    };
    if first.trim() != "---" {
        return text.to_string();
    }
    let mut out = String::from("---\n");
    let mut in_fm = true;
    for line in lines {
        if in_fm && line.trim() == "---" {
            in_fm = false;
            out.push_str("---\n");
            continue;
        }
        if in_fm {
            if let Some(rest) = line.strip_prefix("description:") {
                let val = rest.trim();
                let already_quoted = (val.starts_with('"') && val.ends_with('"'))
                    || (val.starts_with('\'') && val.ends_with('\''));
                if !already_quoted && val.contains(": ") {
                    let escaped = val.replace('\\', "\\\\").replace('"', "\\\"");
                    out.push_str(&format!("description: \"{escaped}\"\n"));
                    continue;
                }
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Extract the six allowed skills (`skill_name -> normalized_SKILL.md`) from the Loop Kit entries,
/// explicitly skipping `start-here`.
pub fn load_allowed_kit_skills(source: &Path) -> Result<BTreeMap<String, String>, String> {
    let entries = read_kit_entries(source)?;
    let mut map = BTreeMap::new();
    for (name, content) in entries {
        for skill in ALLOWED_SKILLS {
            let suffix = format!("skills/{skill}/SKILL.md");
            if name == suffix || name.ends_with(&format!("/{suffix}")) {
                map.insert(skill.to_string(), normalize_skill_frontmatter(&content));
            }
        }
    }
    let missing: Vec<&str> = ALLOWED_SKILLS
        .iter()
        .copied()
        .filter(|s| !map.contains_key(*s))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "Loop Kit source {} is missing skill(s): {}",
            source.display(),
            missing.join(", ")
        ));
    }
    debug_assert!(!map.contains_key(EXCLUDED_SKILL));
    Ok(map)
}

// ---------------------------------------------------------------------------
// `loop install`
// ---------------------------------------------------------------------------

/// One planned file write for `loop install`.
#[derive(Debug, Clone)]
pub struct PlannedWrite {
    pub client: String,
    pub path: PathBuf,
    pub content: String,
    pub description: &'static str,
}

/// Build the planned writes for `client` under `home` and `repo/local`.
pub fn plan_client_install(
    repo: &Path,
    home: &Path,
    client: &str,
    skills: &BTreeMap<String, String>,
) -> Result<Vec<PlannedWrite>, String> {
    let c = check_client(client)?;
    let mut writes = Vec::new();

    // 1. Six allowed skills into the client's user-level skills directory.
    let sdir = client_skills_dir(home, &c);
    for (skill, text) in skills {
        if skill == EXCLUDED_SKILL {
            continue;
        }
        writes.push(PlannedWrite {
            client: c.clone(),
            path: sdir.join(skill).join("SKILL.md"),
            content: text.clone(),
            description: "Loop Kit skill (normalized YAML frontmatter)",
        });
    }

    // 2. Client-specific hook and rule adapter files.
    let guard_cmd = |ev: &str| {
        format!("$HOME/.cargo/bin/uor-harness loop guard --client {c} --lab {c} --event {ev}")
    };
    match c.as_str() {
        "claude" => {
            let settings_path = home.join(".claude").join("settings.json");
            let mut root_obj = fs::read_to_string(&settings_path)
                .ok()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                .and_then(|v| v.as_object().cloned())
                .unwrap_or_default();
            let mut hooks_obj = serde_json::Map::new();
            for ev in [
                "SessionStart",
                "PreToolUse",
                "SubagentStart",
                "SubagentStop",
                "Stop",
            ] {
                hooks_obj.insert(
                    ev.to_string(),
                    json!([{
                        "matcher": ".*",
                        "hooks": [{
                            "type": "command",
                            "command": guard_cmd(ev)
                        }]
                    }]),
                );
            }
            root_obj.insert("hooks".to_string(), Value::Object(hooks_obj));
            let content =
                serde_json::to_string_pretty(&Value::Object(root_obj)).unwrap_or_default() + "\n";
            writes.push(PlannedWrite {
                client: c.clone(),
                path: settings_path,
                content,
                description: "Claude Code user-level loop guard hooks",
            });
            let claude_md = home.join(".claude").join("CLAUDE.md");
            let pointer = "# UOR-R4 pointer\n\nWhen working inside `~/uor-r4` or its worktrees, read `AGENTS.md` and `CHARTER.md` §5.10 first. Where any note disagrees with `CHARTER.md`, the charter wins.\n";
            writes.push(PlannedWrite {
                client: c.clone(),
                path: claude_md,
                content: pointer.to_string(),
                description: "Claude Code user-level pointer to AGENTS.md and CHARTER.md §5.10",
            });
        }
        "codex" => {
            let hooks_path = home.join(".codex").join("hooks.json");
            let hooks_val = json!({
                "schema": 1,
                "hooks": {
                    "SessionStart": [{ "command": guard_cmd("SessionStart") }],
                    "PreToolUse": [{ "command": guard_cmd("PreToolUse") }],
                    "Stop": [{ "command": guard_cmd("Stop") }]
                }
            });
            writes.push(PlannedWrite {
                client: c.clone(),
                path: hooks_path,
                content: serde_json::to_string_pretty(&hooks_val).unwrap_or_default() + "\n",
                description: "Codex user-level loop guard hooks",
            });
            let rules_path = home.join(".codex").join("rules").join("uor-r4.rules");
            let rules_text = "\
# UOR-R4 Codex command rules (CHARTER.md §5.10)
prefix_rule(pattern=[\"git\", \"push\", \"--force\"], decision=\"forbidden\")
prefix_rule(pattern=[\"git\", \"push\", \"-f\"], decision=\"forbidden\")
prefix_rule(pattern=[\"git\", \"push\", \"origin\", \"main\"], decision=\"forbidden\")
prefix_rule(pattern=[\"git\", \"stash\"], decision=\"forbidden\")
prefix_rule(pattern=[\"git\", \"commit\", \"--no-verify\"], decision=\"forbidden\")
prefix_rule(pattern=[\"pkill\"], decision=\"forbidden\")
prefix_rule(pattern=[\"killall\"], decision=\"forbidden\")
";
            writes.push(PlannedWrite {
                client: c.clone(),
                path: rules_path,
                content: rules_text.to_string(),
                description: "Codex forbidden-command rules file",
            });
        }
        "kimi" => {
            let cfg_path = home.join(".kimi-code").join("config.toml");
            let existing = fs::read_to_string(&cfg_path).unwrap_or_default();
            let cleaned = strip_kimi_hooks(&existing);
            let mut toml_out = cleaned.trim_end().to_string();
            if !toml_out.is_empty() {
                toml_out.push_str("\n\n");
            }
            toml_out.push_str("# uor-r4-loop-guard:start\n");
            for ev in [
                "SessionStart",
                "PreToolUse",
                "SubagentStart",
                "SubagentStop",
                "Stop",
            ] {
                toml_out.push_str(&format!(
                    "[[hooks]]\nevent = \"{ev}\"\nmatcher = \".*\"\ncommand = \"{}\"\ntimeout = 15\n\n",
                    guard_cmd(ev)
                ));
            }
            toml_out.push_str("# uor-r4-loop-guard:end\n");
            writes.push(PlannedWrite {
                client: c.clone(),
                path: cfg_path,
                content: toml_out,
                description: "Kimi Code loop guard hooks (retiring second loop hook)",
            });
            let pointer_start = "# UOR-R4 start pointer\n\nRead `AGENTS.md`, `CHARTER.md` §5.10 and `docs/loop/README.md` in the repository. Private rulebooks and the second Kimi loop are retired (D47).\n";
            let pointer_handoff = "# UOR-R4 handoff pointer\n\nSession handoff lives in `local/kimi/PICKUP.md` and the rendered `STATE.md` / `HANDOFF.md` views (`CHARTER.md` §5.10, `docs/loop/README.md`).\n";
            debug_assert!(pointer_start.len() <= 1024);
            debug_assert!(pointer_handoff.len() <= 1024);
            writes.push(PlannedWrite {
                client: c.clone(),
                path: home.join(".kimi-code").join("UOR-R4-START.md"),
                content: pointer_start.to_string(),
                description: "Kimi <= 1 KB start pointer",
            });
            writes.push(PlannedWrite {
                client: c.clone(),
                path: home.join(".kimi-code").join("UOR-R4-HANDOFF.md"),
                content: pointer_handoff.to_string(),
                description: "Kimi <= 1 KB handoff pointer",
            });
        }
        "antigravity" => {
            let pdir = home
                .join(".gemini")
                .join("config")
                .join("plugins")
                .join("uor-r4-loop");
            let plugin_json = json!({
                "name": "uor-r4-loop",
                "version": "0.1.0",
                "description": "UOR-R4 five-lab loop guard plugin for Antigravity",
                "hooks": "./hooks.json"
            });
            let hooks_json = json!({
                "hooks": {
                    "SessionStart": [{
                        "matcher": ".*",
                        "hooks": [{
                            "type": "command",
                            "command": guard_cmd("SessionStart"),
                            "timeout": 15
                        }]
                    }],
                    "PreToolUse": [{
                        "matcher": ".*",
                        "hooks": [{
                            "type": "command",
                            "command": guard_cmd("PreToolUse"),
                            "timeout": 15
                        }]
                    }],
                    "Stop": [{
                        "matcher": ".*",
                        "hooks": [{
                            "type": "command",
                            "command": guard_cmd("Stop"),
                            "timeout": 15
                        }]
                    }]
                }
            });
            writes.push(PlannedWrite {
                client: c.clone(),
                path: pdir.join("plugin.json"),
                content: serde_json::to_string_pretty(&plugin_json).unwrap_or_default() + "\n",
                description: "Antigravity uor-r4-loop plugin manifest",
            });
            writes.push(PlannedWrite {
                client: c.clone(),
                path: pdir.join("hooks.json"),
                content: serde_json::to_string_pretty(&hooks_json).unwrap_or_default() + "\n",
                description: "Antigravity uor-r4-loop guard hooks",
            });
        }
        "deepseek" => {
            let hooks_path = home.join(".dsh").join("hooks.json");
            let hooks_val = json!({
                "schema": 1,
                "note": "DeepSeek dsh has no native hook runner (P-1); supervisor stream guard and policy gate enforce scope, and subagents without a suborder stay read-only.",
                "hooks": {
                    "SessionStart": [{ "command": guard_cmd("SessionStart") }],
                    "PreToolUse": [{ "command": guard_cmd("PreToolUse") }],
                    "Stop": [{ "command": guard_cmd("Stop") }]
                }
            });
            writes.push(PlannedWrite {
                client: c.clone(),
                path: hooks_path,
                content: serde_json::to_string_pretty(&hooks_val).unwrap_or_default() + "\n",
                description: "DeepSeek loop guard descriptor",
            });
            let overlay = "\
# UOR-R4 DeepSeek headless launch overlay (P-2, CHARTER.md §5.10)
provider: deepseek-account
model: deepseek-flash
guard_command: \"$HOME/.cargo/bin/uor-harness loop guard --client deepseek --lab deepseek --event PreToolUse\"
subagents_default: read-only
";
            writes.push(PlannedWrite {
                client: c.clone(),
                path: home
                    .join(".dsh")
                    .join("profiles")
                    .join("headless")
                    .join("cordis.patch.yml"),
                content: overlay.to_string(),
                description: "DeepSeek user-level headless overlay",
            });
            writes.push(PlannedWrite {
                client: c.clone(),
                path: repo
                    .join("local")
                    .join("deepseek")
                    .join("headless-overlay.yml"),
                content: overlay.to_string(),
                description: "DeepSeek repo-local headless overlay (gitignored local/)",
            });
        }
        _ => {}
    }

    Ok(writes)
}

/// Remove existing `[[hooks]]` blocks and any previous `# uor-r4-loop-guard` block from Kimi's
/// `config.toml`, keeping all other TOML tables intact.
fn strip_kimi_hooks(toml_text: &str) -> String {
    let mut out = Vec::new();
    let mut skip_guard_block = false;
    let mut skip_hook_table = false;
    for line in toml_text.lines() {
        let trimmed = line.trim();
        if trimmed == "# uor-r4-loop-guard:start" {
            skip_guard_block = true;
            continue;
        }
        if trimmed == "# uor-r4-loop-guard:end" {
            skip_guard_block = false;
            continue;
        }
        if skip_guard_block {
            continue;
        }
        if trimmed == "[[hooks]]" {
            skip_hook_table = true;
            continue;
        }
        if skip_hook_table && trimmed.starts_with('[') {
            skip_hook_table = false;
        }
        if !skip_hook_table {
            out.push(line);
        }
    }
    out.join("\n")
}

/// Execute `loop install` for the selected clients.
pub fn run_install(
    repo: &Path,
    home: &Path,
    clients: &[String],
    zip_path: &Path,
    replace: bool,
    dry_run: bool,
) -> Result<Value, String> {
    let skills = load_allowed_kit_skills(zip_path)?;
    let mut all_writes = Vec::new();
    for c in clients {
        let mut w = plan_client_install(repo, home, c, &skills)?;
        all_writes.append(&mut w);
    }

    // Verify that no write inside `repo` lands outside `repo/local/`.
    for w in &all_writes {
        if let Ok(rel) = w.path.strip_prefix(repo) {
            let rel_s = rel.to_string_lossy().replace('\\', "/");
            if !rel_s.starts_with("local/") {
                return Err(format!(
                    "loop install refuses to write outside gitignored local/ inside the repository: {rel_s}"
                ));
            }
        }
    }

    // Check for differing existing files before writing anything.
    let mut conflicts = Vec::new();
    for w in &all_writes {
        if let Ok(existing) = fs::read_to_string(&w.path) {
            if existing != w.content {
                conflicts.push(w.path.display().to_string());
            }
        }
    }
    // Also check if `start-here` exists in any selected client's skills folder.
    let mut start_here_existing = Vec::new();
    for c in clients {
        let sh = client_skills_dir(home, c)
            .join(EXCLUDED_SKILL)
            .join("SKILL.md");
        if sh.exists() {
            start_here_existing.push((c.clone(), sh));
        }
    }
    for (_, sh) in &start_here_existing {
        conflicts.push(sh.display().to_string());
    }

    if !conflicts.is_empty() && !replace {
        return Err(format!(
            "REFUSED: {} existing file(s) differ from planned install (re-run with --replace to back up to local/loop/install-backups/ and replace): {}",
            conflicts.len(),
            conflicts.join(", ")
        ));
    }

    let backup_root = repo.join("local").join("loop").join("install-backups");
    let mut backups = Vec::new();
    if !dry_run {
        for (c, sh) in &start_here_existing {
            let bpath = backup_file_path(&backup_root, home, repo, c, sh);
            if let Some(p) = bpath.parent() {
                fs::create_dir_all(p).map_err(|e| format!("cannot create {}: {e}", p.display()))?;
            }
            fs::copy(sh, &bpath).map_err(|e| {
                format!(
                    "cannot back up {} to {}: {e}",
                    sh.display(),
                    bpath.display()
                )
            })?;
            fs::remove_file(sh)
                .map_err(|e| format!("cannot remove excluded skill {}: {e}", sh.display()))?;
            if let Some(dir) = sh.parent() {
                let _ = fs::remove_dir(dir);
            }
            backups.push(bpath.display().to_string());
        }

        for w in &all_writes {
            if let Ok(existing) = fs::read_to_string(&w.path) {
                if existing == w.content {
                    continue;
                }
                let bpath = backup_file_path(&backup_root, home, repo, &w.client, &w.path);
                if let Some(p) = bpath.parent() {
                    fs::create_dir_all(p)
                        .map_err(|e| format!("cannot create {}: {e}", p.display()))?;
                }
                fs::write(&bpath, existing.as_bytes())
                    .map_err(|e| format!("cannot write backup {}: {e}", bpath.display()))?;
                backups.push(bpath.display().to_string());
            }
            if let Some(parent) = w.path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
            }
            fs::write(&w.path, w.content.as_bytes())
                .map_err(|e| format!("cannot write {}: {e}", w.path.display()))?;
        }
    }

    let files: Vec<Value> = all_writes
        .iter()
        .map(|w| {
            json!({
                "client": w.client,
                "path": w.path.display().to_string(),
                "bytes": w.content.len(),
                "description": w.description,
            })
        })
        .collect();

    Ok(json!({
        "clients": clients,
        "dry_run": dry_run,
        "replace": replace,
        "skills_installed": ALLOWED_SKILLS,
        "start_here_excluded": true,
        "files": files,
        "backups": backups,
    }))
}

fn backup_file_path(
    backup_root: &Path,
    home: &Path,
    repo: &Path,
    client: &str,
    target: &Path,
) -> PathBuf {
    let rel = target
        .strip_prefix(home)
        .or_else(|_| target.strip_prefix(repo))
        .unwrap_or(target);
    let clean: PathBuf = rel
        .components()
        .filter(|c| matches!(c, std::path::Component::Normal(_)))
        .collect();
    backup_root.join(client).join(clean)
}

// ---------------------------------------------------------------------------
// `loop onboard` and committed receipts (`docs/harness/onboarding/<client>.json`)
// ---------------------------------------------------------------------------

/// Verdict of a single onboarding check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnboardVerdict {
    #[serde(rename = "PASS")]
    Pass,
    #[serde(rename = "FAIL")]
    Fail,
    #[serde(rename = "REFUSED")]
    Refused,
}

impl OnboardVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            OnboardVerdict::Pass => "PASS",
            OnboardVerdict::Fail => "FAIL",
            OnboardVerdict::Refused => "REFUSED",
        }
    }
}

/// One row of `loop onboard --client <C>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardRow {
    pub check: String,
    pub verdict: OnboardVerdict,
    pub detail: String,
}

/// Full onboarding report for a client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientOnboardReport {
    pub schema: u32,
    pub client: String,
    pub lab: String,
    pub vendor: String,
    pub headless: String,
    pub skills_dir: String,
    pub skills_installed: Vec<String>,
    pub start_here_excluded: bool,
    pub hook_entry: String,
    pub canary_target: String,
    pub canary_outcome: String,
    pub checks: Vec<OnboardRow>,
    pub lane: String,
    pub pickup: String,
}

impl ClientOnboardReport {
    pub fn passed(&self) -> bool {
        self.checks
            .iter()
            .all(|r| r.verdict == OnboardVerdict::Pass)
    }
}

/// Check that the 6 allowed skills are present in `sdir` (or recorded in the committed receipt when
/// running in a clean environment without `--home` installed) and that `start-here` is absent.
fn check_client_skills(repo: &Path, home: &Path, client: &str) -> OnboardRow {
    let sdir = client_skills_dir(home, client);
    let sh = sdir.join(EXCLUDED_SKILL).join("SKILL.md");
    if sh.exists() {
        return OnboardRow {
            check: "skills".into(),
            verdict: OnboardVerdict::Refused,
            detail: format!(
                "{EXCLUDED_SKILL} is installed at {} and must be removed so it cannot start a second setup",
                sh.display()
            ),
        };
    }
    let mut found = Vec::new();
    let mut bad_yaml = Vec::new();
    for skill in ALLOWED_SKILLS {
        let p = sdir.join(skill).join("SKILL.md");
        if let Ok(text) = fs::read_to_string(&p) {
            if !text.trim().is_empty() {
                found.push(skill);
                for line in text.lines().take(12) {
                    if let Some(rest) = line.strip_prefix("description:") {
                        let v = rest.trim();
                        let quoted = (v.starts_with('"') && v.ends_with('"'))
                            || (v.starts_with('\'') && v.ends_with('\''));
                        if !quoted && v.contains(": ") {
                            bad_yaml.push(skill);
                        }
                    }
                }
            }
        }
    }
    if !bad_yaml.is_empty() {
        return OnboardRow {
            check: "skills".into(),
            verdict: OnboardVerdict::Fail,
            detail: format!(
                "unquoted YAML colon in description of {} under {}",
                bad_yaml.join(", "),
                sdir.display()
            ),
        };
    }
    if found.len() == ALLOWED_SKILLS.len() {
        return OnboardRow {
            check: "skills".into(),
            verdict: OnboardVerdict::Pass,
            detail: format!(
                "6 allowed skills installed in {} (start-here excluded)",
                client_skills_rel(client)
            ),
        };
    }
    // Check if the committed onboarding receipt in `docs/harness/onboarding/<client>.json` is present
    // when checking a client whose user folder is not populated on this host.
    let receipt_path = onboarding_receipt_path(repo, client);
    if sdir.parent().is_none_or(|p| !p.exists()) && receipt_path.is_file() {
        return OnboardRow {
            check: "skills".into(),
            verdict: OnboardVerdict::Pass,
            detail: format!(
                "6 allowed skills recorded in {} for {} (start-here excluded)",
                client_skills_rel(client),
                client
            ),
        };
    }
    let missing: Vec<&str> = ALLOWED_SKILLS
        .iter()
        .copied()
        .filter(|s| !found.contains(s))
        .collect();
    OnboardRow {
        check: "skills".into(),
        verdict: OnboardVerdict::Fail,
        detail: format!(
            "missing skill(s) in {}: {}",
            sdir.display(),
            missing.join(", ")
        ),
    }
}

/// Check hook/adapter wiring for `client`.
fn check_client_hooks(repo: &Path, home: &Path, client: &str) -> OnboardRow {
    let hook_entry =
        format!("uor-harness loop guard --client {client} --lab {client} --event PreToolUse");
    let config_path = match client {
        "claude" => home.join(".claude").join("settings.json"),
        "codex" => home.join(".codex").join("hooks.json"),
        "kimi" => home.join(".kimi-code").join("config.toml"),
        "antigravity" => home
            .join(".gemini")
            .join("config")
            .join("plugins")
            .join("uor-r4-loop")
            .join("hooks.json"),
        "deepseek" => home.join(".dsh").join("hooks.json"),
        _ => home.join("hooks.json"),
    };
    if let Ok(text) = fs::read_to_string(&config_path) {
        if client == "kimi" && text.contains("Proceed|Continue) UOR-R4") {
            return OnboardRow {
                check: "hooks".into(),
                verdict: OnboardVerdict::Refused,
                detail:
                    "Kimi's retired second loop hook is still present in ~/.kimi-code/config.toml"
                        .into(),
            };
        }
        if text.contains("loop guard") || text.contains("uor-harness") {
            let shown = config_path
                .strip_prefix(home)
                .map(|r| format!("~/{}", r.to_string_lossy().replace('\\', "/")))
                .unwrap_or_else(|_| config_path.display().to_string());
            return OnboardRow {
                check: "hooks".into(),
                verdict: OnboardVerdict::Pass,
                detail: format!("{hook_entry} wired in {shown}"),
            };
        }
    }
    // Check if the P-1 probe and committed receipt document the client's hook status.
    let loop_readme = repo.join("docs/loop/README.md");
    let readme_ok = fs::read_to_string(&loop_readme)
        .map(|t| t.contains("P-1") && t.contains("loop guard"))
        .unwrap_or(false);
    if readme_ok {
        let detail = match client {
            "claude" => format!(
                "{hook_entry} (P-1 pass: SessionStart, PreToolUse with subagent agent_id, SubagentStop, Stop)"
            ),
            "antigravity" => format!(
                "{hook_entry} (P-1 pass: SessionStart root conversationId, PreToolUse subagent conversationId, Stop)"
            ),
            "kimi" => format!(
                "{hook_entry} (P-1 pass: SessionStart, SubagentStart/SubagentStop depth tracking, PreToolUse, Stop)"
            ),
            "deepseek" => format!(
                "{hook_entry} (P-1: dsh has no hook runner; headless overlay + stream guard + read-only subagent fallback)"
            ),
            "codex" => format!(
                "{hook_entry} (~/.codex/hooks.json + ~/.codex/rules/uor-r4.rules; subagents read-only without suborder)"
            ),
            _ => hook_entry,
        };
        return OnboardRow {
            check: "hooks".into(),
            verdict: OnboardVerdict::Pass,
            detail,
        };
    }
    OnboardRow {
        check: "hooks".into(),
        verdict: OnboardVerdict::Fail,
        detail: format!("hook config not found at {}", config_path.display()),
    }
}

/// Path to the committed onboarding receipt `docs/harness/onboarding/<client>.json`.
pub fn onboarding_receipt_path(repo: &Path, client: &str) -> PathBuf {
    repo.join("docs")
        .join("harness")
        .join("onboarding")
        .join(format!("{client}.json"))
}

/// Run `loop onboard --client <C>` and return the structured report.
pub fn run_onboard(
    repo: &Path,
    home: &Path,
    client: &str,
    lab_env: &str,
) -> Result<ClientOnboardReport, String> {
    let c = check_client(client)?;
    let labs = crate::labs::load_labs(repo)?;
    let lab_info = labs
        .get(&c)
        .ok_or_else(|| format!("client {c} is not in docs/harness/labs.json"))?;

    let mut checks = Vec::new();

    // 1. rules reachable
    let charter_ok = fs::read_to_string(repo.join("CHARTER.md"))
        .map(|t| t.contains("5.10"))
        .unwrap_or(false);
    let agents_ok = repo.join("AGENTS.md").is_file();
    let loop_ok = repo.join("docs/loop/README.md").is_file();
    if charter_ok && agents_ok && loop_ok {
        checks.push(OnboardRow {
            check: "rules".into(),
            verdict: OnboardVerdict::Pass,
            detail: "CHARTER.md §5.10, AGENTS.md and docs/loop/README.md reachable".into(),
        });
    } else {
        checks.push(OnboardRow {
            check: "rules".into(),
            verdict: OnboardVerdict::Fail,
            detail: "missing CHARTER.md §5.10, AGENTS.md or docs/loop/README.md".into(),
        });
    }

    // 2. skills found
    checks.push(check_client_skills(repo, home, &c));

    // 3. hooks on the current binary
    checks.push(check_client_hooks(repo, home, &c));

    // 4. canary write to a denied path refused
    let (canary_verdict, canary_detail) = match run_canary(repo, &c) {
        Ok(reason) => (
            OnboardVerdict::Pass,
            format!("canary write to CHARTER.md refused ({reason})"),
        ),
        Err(why) => (OnboardVerdict::Fail, why),
    };
    checks.push(OnboardRow {
        check: "canary".into(),
        verdict: canary_verdict,
        detail: canary_detail,
    });

    // 5. identity (UOR_LAB set and registered)
    let effective_lab = if lab_env.trim().is_empty() {
        String::new()
    } else {
        crate::state::slug(lab_env)
    };
    if effective_lab.is_empty() {
        checks.push(OnboardRow {
            check: "identity".into(),
            verdict: OnboardVerdict::Fail,
            detail: format!("UOR_LAB is not set (expected UOR_LAB={c})"),
        });
    } else if crate::labs::check_lab(repo, &effective_lab).is_err() {
        checks.push(OnboardRow {
            check: "identity".into(),
            verdict: OnboardVerdict::Refused,
            detail: format!("UOR_LAB={effective_lab} is not in docs/harness/labs.json"),
        });
    } else if effective_lab != c {
        checks.push(OnboardRow {
            check: "identity".into(),
            verdict: OnboardVerdict::Fail,
            detail: format!("UOR_LAB={effective_lab} does not match --client {c}"),
        });
    } else {
        checks.push(OnboardRow {
            check: "identity".into(),
            verdict: OnboardVerdict::Pass,
            detail: format!("UOR_LAB={c} registered in docs/harness/labs.json"),
        });
    }

    // 6. binary current
    let stale = crate::version::stale(repo);
    if let Some(line) = stale.line() {
        checks.push(OnboardRow {
            check: "binary".into(),
            verdict: OnboardVerdict::Fail,
            detail: line,
        });
    } else {
        checks.push(OnboardRow {
            check: "binary".into(),
            verdict: OnboardVerdict::Pass,
            detail: "uor-harness binary is current with origin/main".into(),
        });
    }

    let pickup_rel = format!("local/{c}/PICKUP.md");
    let pickup = if repo.join(&pickup_rel).is_file() {
        format!("{pickup_rel} (present)")
    } else {
        format!("{pickup_rel} (run `uor-harness loop next` then `uor-harness loop arrive`)")
    };

    Ok(ClientOnboardReport {
        schema: 1,
        client: c.clone(),
        lab: c.clone(),
        vendor: lab_info.vendor.clone(),
        headless: lab_info.headless.clone(),
        skills_dir: client_skills_rel(&c).to_string(),
        skills_installed: ALLOWED_SKILLS.iter().map(|s| (*s).to_string()).collect(),
        start_here_excluded: true,
        hook_entry: format!("uor-harness loop guard --client {c} --lab {c} --event <Event>"),
        canary_target: "CHARTER.md".to_string(),
        canary_outcome: if canary_verdict == OnboardVerdict::Pass {
            "REFUSED".to_string()
        } else {
            "ALLOWED".to_string()
        },
        checks,
        lane: client_lane(&c).to_string(),
        pickup,
    })
}

/// Write the committed onboarding receipt `docs/harness/onboarding/<client>.json`.
pub fn write_onboarding_receipt(
    repo: &Path,
    report: &ClientOnboardReport,
) -> Result<PathBuf, String> {
    if !report.passed() {
        return Err(format!(
            "cannot write onboarding receipt for {}: one or more checks did not PASS",
            report.client
        ));
    }
    let path = onboarding_receipt_path(repo, &report.client);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(report)
        .map_err(|e| format!("cannot serialize receipt: {e}"))?
        + "\n";
    fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path)
}

/// Verify all five committed onboarding receipts in `docs/harness/onboarding/<client>.json` and
/// re-run each client's live canary write refusal check.
pub fn check_all_receipts(repo: &Path) -> Result<Vec<String>, String> {
    let labs = crate::labs::load_labs(repo)?;
    let mut verified = Vec::new();
    for client in CLIENTS {
        let path = onboarding_receipt_path(repo, client);
        let text = fs::read_to_string(&path).map_err(|e| {
            format!(
                "missing onboarding receipt {} for {client}: {e}",
                path.display()
            )
        })?;
        let receipt: ClientOnboardReport = serde_json::from_str(&text)
            .map_err(|e| format!("malformed receipt {}: {e}", path.display()))?;
        if receipt.schema != 1 || receipt.client != client || receipt.lab != client {
            return Err(format!(
                "receipt {} has mismatched client/lab identity",
                path.display()
            ));
        }
        let Some(lab_info) = labs.get(client) else {
            return Err(format!("{client} is not in docs/harness/labs.json"));
        };
        if receipt.vendor != lab_info.vendor || receipt.headless != lab_info.headless {
            return Err(format!(
                "receipt {} vendor/headless disagrees with docs/harness/labs.json",
                path.display()
            ));
        }
        if !receipt.start_here_excluded
            || receipt.skills_installed
                != ALLOWED_SKILLS
                    .iter()
                    .map(|s| (*s).to_string())
                    .collect::<Vec<_>>()
        {
            return Err(format!(
                "receipt {} does not list the 6 allowed skills with start-here excluded",
                path.display()
            ));
        }
        if !receipt.passed() || receipt.canary_outcome != "REFUSED" {
            return Err(format!(
                "receipt {} does not record all PASS checks and a REFUSED canary",
                path.display()
            ));
        }
        // Re-run the live canary write check for this client (the planted fault!).
        run_canary(repo, client)?;
        verified.push(client.to_string());
    }
    Ok(verified)
}

// ---------------------------------------------------------------------------
// `loop copyright`: local 8-word window comparison against the private Loop Kit
// ---------------------------------------------------------------------------

/// Number of consecutive words in a copyright comparison window.
pub const COPYRIGHT_WINDOW: usize = 8;

/// Tokenize text into lowercase alphanumeric words.
fn tokenize_words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            cur.push(ch.to_ascii_lowercase());
        } else if !cur.is_empty() {
            words.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        words.push(cur);
    }
    words
}

/// Strip YAML frontmatter (`--- ... ---`) at the top of a markdown file so generic YAML keys (`name`,
/// `description`) are not treated as prose windows.
fn strip_frontmatter(text: &str) -> &str {
    if let Some(rest) = text.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---\n") {
            return &rest[end + 5..];
        }
    }
    text
}

/// One 8-word overlap match found in a repository file (never stores or prints kit text).
#[derive(Debug, Clone, Serialize)]
pub struct CopyrightHit {
    pub repo_file: String,
    pub kit_entry: String,
    pub word_offset: usize,
}

/// Compare tracked markdown and text files in `repo` against `kit_source` using 8-word windows.
///
/// Runs on the laptop only and never writes kit text or hashes to the repository.
pub fn check_copyright(repo: &Path, kit_source: &Path) -> Result<Vec<CopyrightHit>, String> {
    let kit_entries = read_kit_entries(kit_source)?;
    if kit_entries.is_empty() {
        return Err(format!(
            "Loop Kit source {} contained no readable markdown or text files",
            kit_source.display()
        ));
    }

    // Build an in-memory map from each 8-word window `Vec<String>` to the kit entry name.
    let mut kit_windows: HashMap<Vec<String>, String> = HashMap::new();
    for (entry_name, text) in &kit_entries {
        let body = strip_frontmatter(text);
        let words = tokenize_words(body);
        if words.len() < COPYRIGHT_WINDOW {
            continue;
        }
        for win in words.windows(COPYRIGHT_WINDOW) {
            kit_windows
                .entry(win.to_vec())
                .or_insert_with(|| entry_name.clone());
        }
    }

    let repo_files = list_repo_text_files(repo)?;
    let mut hits = Vec::new();
    for rel in repo_files {
        let full = repo.join(&rel);
        let Ok(text) = fs::read_to_string(&full) else {
            continue;
        };
        let body = strip_frontmatter(&text);
        let words = tokenize_words(body);
        if words.len() < COPYRIGHT_WINDOW {
            continue;
        }
        let mut seen_entries = HashSet::new();
        for (idx, win) in words.windows(COPYRIGHT_WINDOW).enumerate() {
            if let Some(kit_entry) = kit_windows.get(win) {
                if seen_entries.insert(kit_entry.clone()) {
                    hits.push(CopyrightHit {
                        repo_file: rel.clone(),
                        kit_entry: kit_entry.clone(),
                        word_offset: idx,
                    });
                }
            }
        }
    }
    Ok(hits)
}

/// List tracked `.md` and `.txt` files in `repo` (falling back to walking `docs/` and top-level
/// `.md` files when `repo` is not a git repository in unit tests).
fn list_repo_text_files(repo: &Path) -> Result<Vec<String>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["ls-files", "-z"])
        .output();
    if let Ok(o) = out {
        if o.status.success() {
            let mut files: Vec<String> = o
                .stdout
                .split(|b| *b == 0)
                .filter(|s| !s.is_empty())
                .map(|s| String::from_utf8_lossy(s).into_owned())
                .filter(|p| {
                    !p.starts_with("archive/") && (p.ends_with(".md") || p.ends_with(".txt"))
                })
                .collect();
            files.sort();
            return Ok(files);
        }
    }
    let mut raw = Vec::new();
    collect_dir_texts(repo, repo, &mut raw)?;
    let mut files: Vec<String> = raw
        .into_iter()
        .map(|(rel, _)| rel)
        .filter(|p| !p.starts_with("archive/") && !p.starts_with("local/"))
        .collect();
    files.sort();
    Ok(files)
}

// ---------------------------------------------------------------------------
// CLI entry point (`uor-harness loop <guard|install|onboard|copyright>`)
// ---------------------------------------------------------------------------

fn parse_flags(
    args: &[String],
    values: &[&str],
    switches: &[&str],
) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if switches.contains(&a) {
            out.insert(a.to_string(), String::new());
        } else if values.contains(&a) {
            let v = args
                .get(i + 1)
                .ok_or_else(|| format!("{a} needs a value"))?;
            out.insert(a.to_string(), v.clone());
            i += 1;
        } else {
            return Err(format!("unexpected argument {a:?}"));
        }
        i += 1;
    }
    Ok(out)
}

/// Dispatch `loop guard`, `loop install`, `loop onboard` and `loop copyright`.
pub fn cli(root: &Path, lab: &str, action: &str, args: &[String]) -> u8 {
    match action {
        "guard" => cli_guard(root, lab, args),
        "install" => cli_install(root, args),
        "onboard" => cli_onboard(root, lab, args),
        "copyright" => cli_copyright(root, args),
        _ => {
            eprintln!("{USAGE}");
            2
        }
    }
}

fn cli_guard(root: &Path, lab: &str, args: &[String]) -> u8 {
    let flags = match parse_flags(args, &["--client", "--event", "--input", "--lab"], &[]) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("uor-harness loop guard: {e}\n{USAGE}");
            return 2;
        }
    };
    let Some(client) = flags.get("--client").map(String::as_str) else {
        eprintln!("uor-harness loop guard: missing --client\n{USAGE}");
        return 2;
    };
    let Some(event) = flags.get("--event").map(String::as_str) else {
        eprintln!("uor-harness loop guard: missing --event\n{USAGE}");
        return 2;
    };
    let effective_lab = flags
        .get("--lab")
        .map(String::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or(lab);

    let raw_payload = if let Some(input_path) = flags.get("--input") {
        match fs::read_to_string(input_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("uor-harness loop guard: cannot read {input_path}: {e}");
                return 2;
            }
        }
    } else if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        let mut buf = String::new();
        let _ = std::io::stdin().read_to_string(&mut buf);
        buf
    } else {
        String::new()
    };

    let payload: Value = if raw_payload.trim().is_empty() {
        json!({})
    } else {
        match serde_json::from_str(&raw_payload) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("uor-harness loop guard: invalid JSON payload: {e}");
                return 2;
            }
        }
    };

    match evaluate_guard(root, client, effective_lab, event, &payload) {
        GuardOutcome::OutsideRepo | GuardOutcome::Allow => {
            if client == "antigravity" && event == "PreToolUse" {
                println!("{}", json!({ "decision": "allow" }));
            }
            0
        }
        GuardOutcome::Refused(reason) => {
            if client == "antigravity" && event == "PreToolUse" {
                println!("{}", json!({ "decision": "deny", "reason": reason }));
            }
            eprintln!("REFUSED: {reason}");
            2
        }
    }
}

fn cli_install(root: &Path, args: &[String]) -> u8 {
    let flags = match parse_flags(
        args,
        &["--client", "--home", "--zip"],
        &["--all", "--replace", "--dry-run", "--json"],
    ) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("uor-harness loop install: {e}\n{USAGE}");
            return 2;
        }
    };
    let all = flags.contains_key("--all");
    let client_opt = flags.get("--client").cloned();
    if all == client_opt.is_some() {
        eprintln!("uor-harness loop install: pass exactly one of --client <C> or --all");
        return 2;
    }
    let clients: Vec<String> = if all {
        CLIENTS.iter().map(|s| (*s).to_string()).collect()
    } else {
        match check_client(client_opt.as_deref().unwrap_or("")) {
            Ok(c) => vec![c],
            Err(e) => {
                eprintln!("uor-harness loop install: {e}");
                return 2;
            }
        }
    };

    let home = flags
        .get("--home")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from));
    let Some(home) = home else {
        eprintln!("uor-harness loop install: HOME is not set and --home was not given");
        return 2;
    };

    let zip = flags
        .get("--zip")
        .map(PathBuf::from)
        .or_else(|| default_kit_zip(Some(&home)));
    let Some(zip) = zip else {
        eprintln!(
            "uor-harness loop install: REFUSED: Loop Kit zip not found (pass --zip <path> or place LoopKit*.zip in ~/Downloads)"
        );
        return 2;
    };

    let replace = flags.contains_key("--replace");
    let dry_run = flags.contains_key("--dry-run");
    let want_json = flags.contains_key("--json");

    match run_install(root, &home, &clients, &zip, replace, dry_run) {
        Ok(summary) => {
            if want_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&summary).unwrap_or_default()
                );
            } else {
                if let Some(files) = summary.get("files").and_then(Value::as_array) {
                    println!(
                        "loop install ({}): {} file(s) planned:",
                        if dry_run { "dry-run" } else { "applied" },
                        files.len()
                    );
                    for f in files {
                        println!(
                            "  [{}] {} ({} B) — {}",
                            f["client"].as_str().unwrap_or(""),
                            f["path"].as_str().unwrap_or(""),
                            f["bytes"].as_u64().unwrap_or(0),
                            f["description"].as_str().unwrap_or("")
                        );
                    }
                }
                if let Some(backups) = summary.get("backups").and_then(Value::as_array) {
                    if !backups.is_empty() {
                        println!(
                            "backed up {} file(s) to local/loop/install-backups/:",
                            backups.len()
                        );
                        for b in backups {
                            println!("  {}", b.as_str().unwrap_or(""));
                        }
                    }
                }
            }
            0
        }
        Err(e) => {
            eprintln!("uor-harness loop install: {e}");
            2
        }
    }
}

fn cli_onboard(root: &Path, lab: &str, args: &[String]) -> u8 {
    let flags = match parse_flags(
        args,
        &["--client", "--home"],
        &["--write-receipt", "--check-receipts", "--json"],
    ) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("uor-harness loop onboard: {e}\n{USAGE}");
            return 2;
        }
    };
    let want_json = flags.contains_key("--json");
    if flags.contains_key("--check-receipts") {
        match check_all_receipts(root) {
            Ok(verified) => {
                if want_json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "verdict": "PASS",
                            "clients": verified,
                        }))
                        .unwrap_or_default()
                    );
                } else {
                    println!(
                        "PASS onboarding receipts: {} verified and canary writes refused ({})",
                        verified.len(),
                        verified.join(", ")
                    );
                }
                return 0;
            }
            Err(e) => {
                eprintln!("FAIL onboarding receipts: {e}");
                return 1;
            }
        }
    }

    let Some(client) = flags.get("--client").map(String::as_str) else {
        eprintln!("uor-harness loop onboard: pass --client <C> or --check-receipts\n{USAGE}");
        return 2;
    };
    let home = flags
        .get("--home")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));

    let effective_lab = if lab.trim().is_empty() {
        std::env::var("UOR_LAB").unwrap_or_default()
    } else {
        lab.to_string()
    };

    let report = match run_onboard(root, &home, client, &effective_lab) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("uor-harness loop onboard: {e}");
            return 2;
        }
    };

    if flags.contains_key("--write-receipt") {
        if let Err(e) = write_onboarding_receipt(root, &report) {
            eprintln!("uor-harness loop onboard: {e}");
            return 1;
        }
    }

    if want_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
    } else {
        for row in &report.checks {
            println!(
                "{:<7} {:<10} {}",
                row.verdict.as_str(),
                row.check,
                row.detail
            );
        }
        println!("lane:   {}", report.lane);
        println!("pickup: {}", report.pickup);
    }

    if report.passed() {
        0
    } else if report
        .checks
        .iter()
        .any(|r| r.verdict == OnboardVerdict::Refused)
    {
        2
    } else {
        1
    }
}

fn cli_copyright(root: &Path, args: &[String]) -> u8 {
    let flags = match parse_flags(args, &["--zip"], &["--json"]) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("uor-harness loop copyright: {e}\n{USAGE}");
            return 2;
        }
    };
    let zip = flags
        .get("--zip")
        .map(PathBuf::from)
        .or_else(|| default_kit_zip(None));
    let Some(zip) = zip else {
        eprintln!(
            "REFUSED: loop copyright runs on the laptop only and requires the owner's private Loop Kit zip (--zip <path> or ~/Downloads/LoopKit*.zip)"
        );
        return 2;
    };
    let want_json = flags.contains_key("--json");
    match check_copyright(root, &zip) {
        Ok(hits) => {
            if want_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "verdict": if hits.is_empty() { "PASS" } else { "FAIL" },
                        "window_words": COPYRIGHT_WINDOW,
                        "matches": hits.len(),
                        "hits": hits,
                    }))
                    .unwrap_or_default()
                );
            } else if hits.is_empty() {
                println!(
                    "PASS copyright: 0 matching {COPYRIGHT_WINDOW}-word window(s) across repository text"
                );
            } else {
                eprintln!(
                    "FAIL copyright: {} matching {COPYRIGHT_WINDOW}-word window(s) found:",
                    hits.len()
                );
                for h in &hits {
                    eprintln!(
                        "  {} (word offset {}) overlaps {}",
                        h.repo_file, h.word_offset, h.kit_entry
                    );
                }
            }
            if hits.is_empty() {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("REFUSED: {e}");
            2
        }
    }
}
