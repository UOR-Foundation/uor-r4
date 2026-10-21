//! `uor-harness loop close` (loop PR 7b, ORDER call 4): loop step 9.
//!
//! **`--session`** ends a session on a task branch, in this order; a refusal before the first
//! GitHub write writes nothing:
//!
//! 1. the task folder: the one ORDER this branch adds (`loop report`'s search);
//! 2. its `REPORT.md` must pass `loop report --check`;
//! 3. the PR: `--pr <n>`, else the one open PR whose head branch is the current branch; its head sha
//!    must equal local `HEAD` ("push first" otherwise);
//! 4. the body: the `size: …` line `loop pr-check` prints for `merge-base origin/main HEAD..HEAD`, the
//!    REPORT without its `# ` title line, `References #<issue>` from `docs/roadmap/issues.json` when
//!    the ORDER's item has an issue, and the Claude Code footer when the lab is `claude`;
//! 5. the PICKUP for `local/<lab>/PICKUP.md`, from `docs/loop/templates/PICKUP.md`; 8,192 bytes or
//!    more refuses;
//! 6. writes: the PR body, then `gh pr ready` when the PR is a draft, then the PICKUP.
//!
//! `--dry-run` prints the body and the PICKUP and writes nothing. **`--verify`** is the next commit.
//! Exit codes: 0 done, 1 refused (a check failed; nothing written), 2 cannot run.

use serde::Deserialize;
use std::cell::RefCell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const USAGE: &str = "usage: uor-harness loop close (--session [--pr <n>] [--dry-run] | --verify)";
const PICKUP_TEMPLATE: &str = "docs/loop/templates/PICKUP.md";
const ISSUES: &str = "docs/roadmap/issues.json";
const FOOTER: &str = "🤖 Generated with [Claude Code](https://claude.com/claude-code)";

/// Points `loop close` at a JSON file standing in for `gh`: `prs` (each `number`, `headRefName`,
/// `isDraft`, `headRefOid`) and optional `failSetBody` / `failReady`. Every write that succeeds is
/// appended as one JSON line to `<file>.calls.jsonl`. Never set it outside tests.
pub const CLOSE_GH_FAKE_ENV: &str = "UOR_LOOP_CLOSE_GH_FAKE";

/// An open PR as `loop close` needs it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PrRef {
    pub number: u64,
    #[serde(rename = "isDraft")]
    pub is_draft: bool,
    #[serde(rename = "headRefOid")]
    pub head_sha: String,
}

/// What `loop close` reads from and writes to GitHub.
pub trait CloseGh {
    /// The open PRs whose head branch is `branch`.
    fn pr_for_branch(&self, branch: &str) -> Result<Vec<PrRef>, String>;
    fn pr(&self, number: u64) -> Result<PrRef, String>;
    fn set_body(&self, number: u64, body: &str) -> Result<(), String>;
    fn mark_ready(&self, number: u64) -> Result<(), String>;
}

/// `gh`, run from the repository root.
struct RealCloseGh {
    root: PathBuf,
}

impl CloseGh for RealCloseGh {
    fn pr_for_branch(&self, branch: &str) -> Result<Vec<PrRef>, String> {
        let out = crate::loopclaim::run_gh(
            &self.root,
            &[
                "pr",
                "list",
                "--state",
                "open",
                "--head",
                branch,
                "--json",
                "number,isDraft,headRefOid",
            ],
        )?;
        serde_json::from_str(&out).map_err(|e| format!("gh pr list printed unexpected JSON: {e}"))
    }

    fn pr(&self, number: u64) -> Result<PrRef, String> {
        let out = crate::loopclaim::run_gh(
            &self.root,
            &[
                "pr",
                "view",
                &number.to_string(),
                "--json",
                "number,isDraft,headRefOid",
            ],
        )?;
        serde_json::from_str(&out).map_err(|e| format!("gh pr view printed unexpected JSON: {e}"))
    }

    fn set_body(&self, number: u64, body: &str) -> Result<(), String> {
        let path =
            std::env::temp_dir().join(format!("uor-loop-close-{}-{number}.md", std::process::id()));
        std::fs::write(&path, body).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        let file = path.to_string_lossy().into_owned();
        let res = crate::loopclaim::run_gh(
            &self.root,
            &["pr", "edit", &number.to_string(), "--body-file", &file],
        );
        let _ = std::fs::remove_file(&path);
        res.map(|_| ())
    }

    fn mark_ready(&self, number: u64) -> Result<(), String> {
        crate::loopclaim::run_gh(&self.root, &["pr", "ready", &number.to_string()]).map(|_| ())
    }
}

#[derive(Deserialize)]
struct FakePr {
    number: u64,
    #[serde(rename = "headRefName")]
    head_ref_name: String,
    #[serde(rename = "isDraft")]
    is_draft: bool,
    #[serde(rename = "headRefOid")]
    head_sha: String,
}

#[derive(Deserialize)]
struct FakeFile {
    #[serde(default)]
    prs: Vec<FakePr>,
    #[serde(default, rename = "failSetBody")]
    fail_set_body: bool,
    #[serde(default, rename = "failReady")]
    fail_ready: bool,
}

/// The in-memory stand-in for `gh`, read from the file `UOR_LOOP_CLOSE_GH_FAKE` names.
struct FakeCloseGh {
    file: FakeFile,
    calls: RefCell<Option<std::fs::File>>,
    calls_path: PathBuf,
}

impl FakeCloseGh {
    fn from_file(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            format!(
                "cannot read {CLOSE_GH_FAKE_ENV} file {}: {e}",
                path.display()
            )
        })?;
        let file = serde_json::from_str(&text).map_err(|e| {
            format!(
                "{CLOSE_GH_FAKE_ENV} file {} is malformed: {e}",
                path.display()
            )
        })?;
        let mut calls_path = path.as_os_str().to_owned();
        calls_path.push(".calls.jsonl");
        Ok(FakeCloseGh {
            file,
            calls: RefCell::new(None),
            calls_path: calls_path.into(),
        })
    }

    fn record(&self, call: serde_json::Value) -> Result<(), String> {
        let mut slot = self.calls.borrow_mut();
        if slot.is_none() {
            *slot = Some(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.calls_path)
                    .map_err(|e| format!("cannot open {}: {e}", self.calls_path.display()))?,
            );
        }
        let f = slot.as_mut().expect("opened above");
        writeln!(f, "{call}")
            .map_err(|e| format!("cannot write {}: {e}", self.calls_path.display()))
    }

    fn pr_ref(p: &FakePr) -> PrRef {
        PrRef {
            number: p.number,
            is_draft: p.is_draft,
            head_sha: p.head_sha.clone(),
        }
    }
}

impl CloseGh for FakeCloseGh {
    fn pr_for_branch(&self, branch: &str) -> Result<Vec<PrRef>, String> {
        Ok(self
            .file
            .prs
            .iter()
            .filter(|p| p.head_ref_name == branch)
            .map(Self::pr_ref)
            .collect())
    }

    fn pr(&self, number: u64) -> Result<PrRef, String> {
        self.file
            .prs
            .iter()
            .find(|p| p.number == number)
            .map(Self::pr_ref)
            .ok_or_else(|| format!("the fake has no PR #{number}"))
    }

    fn set_body(&self, number: u64, body: &str) -> Result<(), String> {
        if self.file.fail_set_body {
            return Err("the fake gh pr edit failed".into());
        }
        self.record(serde_json::json!({"call": "set_body", "number": number, "body": body}))
    }

    fn mark_ready(&self, number: u64) -> Result<(), String> {
        if self.file.fail_ready {
            return Err("the fake gh pr ready failed".into());
        }
        self.record(serde_json::json!({"call": "ready", "number": number}))
    }
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("git");
    // A caller's GIT_DIR, GIT_WORK_TREE or GIT_INDEX_FILE would point git at another repository.
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    let out = cmd
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// The lines of the REPORT's `## <name>` section, verbatim, without surrounding blank lines.
fn report_section(report: &str, name: &str) -> String {
    let mut inside = false;
    let mut lines = Vec::new();
    for (line, can_head) in crate::loopprcheck::heading_lines(report) {
        if can_head && line.starts_with("## ") {
            inside = line[3..].trim() == name;
            continue;
        }
        if inside {
            lines.push(line);
        }
    }
    lines.join("\n").trim_matches('\n').to_string()
}

/// The REPORT without its first `# ` title line, without leading or trailing blank lines.
fn report_without_title(report: &str) -> String {
    let mut dropped = false;
    let kept: Vec<&str> = report
        .lines()
        .filter(|l| {
            if !dropped && l.starts_with("# ") {
                dropped = true;
                return false;
            }
            true
        })
        .collect();
    kept.join("\n").trim_matches('\n').trim_end().to_string()
}

/// The issue number `docs/roadmap/issues.json` maps `item` to, `None` when there is none.
fn issue_for(root: &Path, item: Option<&str>) -> Result<Option<u64>, String> {
    let Some(item) = item else {
        return Ok(None);
    };
    let Ok(text) = std::fs::read_to_string(root.join(ISSUES)) else {
        return Ok(None);
    };
    let map: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("{ISSUES} is malformed: {e}"))?;
    Ok(map.get(item).and_then(serde_json::Value::as_u64))
}

struct PickupFields<'a> {
    item: &'a str,
    lab: &'a str,
    task: &'a str,
    branch: &'a str,
    head: &'a str,
    folder: &'a str,
    still_open: &'a str,
    next_step: &'a str,
}

/// The PICKUP from the template: placeholders filled, `## Open items` holding the REPORT's
/// "Still open" verbatim, `## Next step` the next step, and `## Standing references` left empty
/// (the template's line there is an instruction, and `loop arrive` fails an uncited line).
fn pickup_text(template: &str, f: &PickupFields) -> String {
    let mut out = String::new();
    let mut section = String::new();
    for (i, line) in template.lines().enumerate() {
        if i == 0 && line.starts_with("<!--") {
            continue;
        }
        if let Some(h) = line.strip_prefix("## ") {
            section = h.trim().to_string();
            out.push_str(line);
            out.push_str("\n\n");
            match section.as_str() {
                "Open items" => out.push_str(&format!("{}\n\n", f.still_open)),
                "Next step" => out.push_str(&format!("{}\n\n", f.next_step)),
                _ => {}
            }
            continue;
        }
        if matches!(
            section.as_str(),
            "Open items" | "Next step" | "Standing references"
        ) {
            continue;
        }
        let filled = line
            .replace(
                "docs/loop/tasks/<folder>/<lab>-<n>/",
                &format!("{}/", f.folder),
            )
            .replace("<lab>-<n>", f.task)
            .replace("<item id>", f.item)
            .replace("<lab>", f.lab)
            .replace("<branch>", f.branch)
            .replace("<full sha of the last commit>", f.head);
        out.push_str(&filled);
        out.push('\n');
    }
    format!("{}\n", out.trim_end())
}

/// `loop close ...`; `args` follow the action name.
pub fn cli(root: &Path, lab: &str, args: &[String]) -> u8 {
    let (mut session, mut verify, mut dry_run, mut pr_arg) = (false, false, false, None);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--session" => session = true,
            "--verify" => verify = true,
            "--dry-run" => dry_run = true,
            "--pr" => match args.get(i + 1).and_then(|v| v.parse::<u64>().ok()) {
                Some(n) => {
                    pr_arg = Some(n);
                    i += 1;
                }
                None => {
                    eprintln!("uor-harness: loop close: --pr needs a PR number\n{USAGE}");
                    return 2;
                }
            },
            other => {
                eprintln!("uor-harness: loop close: unknown argument {other:?}\n{USAGE}");
                return 2;
            }
        }
        i += 1;
    }
    if session == verify {
        eprintln!("uor-harness: loop close: give exactly one of --session, --verify\n{USAGE}");
        return 2;
    }
    if verify {
        eprintln!("uor-harness: loop close --verify: not built yet (the next commit)");
        return 2;
    }
    if lab.is_empty() {
        eprintln!("uor-harness: loop close needs a lab (UOR_LAB or --lab)");
        return 2;
    }
    match session_close(root, lab, pr_arg, dry_run) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("uor-harness: loop close: {e}");
            2
        }
    }
}

/// `--session`. `Err` is a cannot-run (exit 2); a refusal prints its reasons and returns 1.
fn session_close(root: &Path, lab: &str, pr_arg: Option<u64>, dry_run: bool) -> Result<u8, String> {
    // 1. The task folder.
    let order_rel = crate::loopreport::find_order(root)?;
    let order_path = root.join(&order_rel);
    let order_text = std::fs::read_to_string(&order_path)
        .map_err(|e| format!("cannot read {order_rel}: {e}"))?;
    let folder = order_path.parent().unwrap_or(root).to_path_buf();
    let shown = folder
        .strip_prefix(root)
        .unwrap_or(&folder)
        .display()
        .to_string();

    // 2. The REPORT passes `loop report --check`.
    let problems = crate::loopreport::report_file_problems(&folder, &shown);
    if !problems.is_empty() {
        println!("REFUSED close: {shown}/REPORT.md fails `loop report --check`");
        for p in problems {
            println!("  {p}");
        }
        return Ok(1);
    }
    let report = std::fs::read_to_string(folder.join("REPORT.md"))
        .map_err(|e| format!("cannot read {shown}/REPORT.md: {e}"))?;

    // 3. The PR, at HEAD.
    let branch = git(root, &["symbolic-ref", "--short", "HEAD"])
        .map_err(|e| format!("not on a branch: {e}"))?;
    let head = git(root, &["rev-parse", "HEAD"])?;
    let gh: Box<dyn CloseGh> = match std::env::var_os(CLOSE_GH_FAKE_ENV) {
        Some(p) => Box::new(FakeCloseGh::from_file(Path::new(&p))?),
        None => Box::new(RealCloseGh {
            root: root.to_path_buf(),
        }),
    };
    let pr = match pr_arg {
        Some(n) => gh.pr(n)?,
        None => {
            let mut prs = gh.pr_for_branch(&branch)?;
            match prs.len() {
                1 => prs.remove(0),
                0 => {
                    return Err(format!(
                        "no open PR has head branch {branch}; pass --pr <n>"
                    ))
                }
                n => {
                    let nums: Vec<String> = prs.iter().map(|p| format!("#{}", p.number)).collect();
                    return Err(format!(
                        "{n} open PRs have head branch {branch} ({}); pass --pr <n>",
                        nums.join(", ")
                    ));
                }
            }
        }
    };
    if pr.head_sha != head {
        println!(
            "REFUSED close: push first: the PR head is {}, HEAD is {head}",
            pr.head_sha
        );
        return Ok(1);
    }

    // 4. The body.
    let merge_base = git(root, &["merge-base", "origin/main", "HEAD"])?;
    let size = crate::loopprcheck::size_line(root, &merge_base, &head)?;
    let item = crate::loopclaim::order_item(&order_text);
    let mut body = format!("{size}\n\n{}\n", report_without_title(&report));
    if let Some(issue) = issue_for(root, item.as_deref())? {
        body.push_str(&format!("\nReferences #{issue}\n"));
    }
    if lab == "claude" {
        body.push_str(&format!("\n{FOOTER}\n"));
    }

    // 5. The PICKUP, checked before any write.
    let template = std::fs::read_to_string(root.join(PICKUP_TEMPLATE))
        .map_err(|e| format!("cannot read {PICKUP_TEMPLATE}: {e}"))?;
    let task = folder
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let item_shown = item.clone().unwrap_or_else(|| {
        folder
            .parent()
            .and_then(Path::file_name)
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    });
    let still_open = report_section(&report, "Still open");
    let next_step = format!(
        "Await the exact-head review of PR #{} at `{head}`.",
        pr.number
    );
    let pickup = pickup_text(
        &template,
        &PickupFields {
            item: &item_shown,
            lab,
            task: &task,
            branch: &branch,
            head: &head,
            folder: &shown,
            still_open: &still_open,
            next_step: &next_step,
        },
    );
    let max = crate::looparrive::PICKUP_MAX;
    if pickup.len() >= max {
        println!(
            "REFUSED close: the PICKUP would be {} bytes; a PICKUP must stay under {max} \
             (shorten the REPORT's \"Still open\")",
            pickup.len()
        );
        return Ok(1);
    }
    let pickup_rel = format!("local/{lab}/PICKUP.md");

    if dry_run {
        println!("--- PR #{} body ---", pr.number);
        print!("{body}");
        println!("--- {pickup_rel} ---");
        print!("{pickup}");
        println!("--- dry run: nothing written ---");
        return Ok(0);
    }

    // 6. Writes: body, ready, PICKUP.
    gh.set_body(pr.number, &body).map_err(|e| {
        format!(
            "setting the body of PR #{} failed: {e}; nothing written",
            pr.number
        )
    })?;
    if pr.is_draft {
        gh.mark_ready(pr.number).map_err(|e| {
            format!(
                "the body of PR #{} was already set, but marking it ready failed: {e}; \
                 no PICKUP written",
                pr.number
            )
        })?;
    }
    let pickup_path = root.join(&pickup_rel);
    if let Some(dir) = pickup_path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    }
    std::fs::write(&pickup_path, &pickup).map_err(|e| format!("cannot write {pickup_rel}: {e}"))?;
    println!(
        "close: set the body of PR #{} ({} bytes); {}; wrote {pickup_rel} ({} bytes)",
        pr.number,
        body.len(),
        if pr.is_draft {
            "marked it ready"
        } else {
            "it was already ready"
        },
        pickup.len()
    );
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pickup_fills_the_template() {
        let t = "<!-- c -->\n# PICKUP: <item id>, <lab>-<n>\n\n- Branch: `<branch>`\n- Head: `<full sha of the last commit>`\n- Task folder: `docs/loop/tasks/<folder>/<lab>-<n>/`\n\n## Open items\n\n- x\n\n## Next step\n\ny\n\n## Standing references\n\nz\n";
        let p = pickup_text(
            t,
            &PickupFields {
                item: "it",
                lab: "claude",
                task: "claude-1",
                branch: "b",
                head: "h",
                folder: "docs/loop/tasks/it/claude-1",
                still_open: "- open",
                next_step: "go",
            },
        );
        assert_eq!(
            p,
            "# PICKUP: it, claude-1\n\n- Branch: `b`\n- Head: `h`\n- Task folder: `docs/loop/tasks/it/claude-1/`\n\n## Open items\n\n- open\n\n## Next step\n\ngo\n\n## Standing references\n"
        );
    }
}
