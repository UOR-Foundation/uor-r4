//! `uor-harness loop close --session` against temporary git repositories: a bare `origin` whose
//! `main` holds the real REPORT and PICKUP templates and `docs/roadmap/issues.json`, a branch that
//! adds an ORDER and a REPORT, and `UOR_LOOP_CLOSE_GH_FAKE` in place of `gh`. Every child process has
//! every `GIT_*` variable removed, and each repository is checked to sit inside its temporary
//! directory. No test touches the network or the real `gh`.

use std::path::PathBuf;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");
const PICKUP_TEMPLATE: &str = include_str!("../../../docs/loop/templates/PICKUP.md");
const REPORT_TEMPLATE: &str = include_str!("../../../docs/loop/templates/REPORT.md");
const ORDER: &str = "docs/loop/tasks/item-a/claude-1/ORDER.md";
const REPORT: &str = "docs/loop/tasks/item-a/claude-1/REPORT.md";
const PICKUP: &str = "local/claude/PICKUP.md";
const STILL_OPEN: &str = "- The verify half, tracked in #77.";

fn valid_report() -> String {
    format!(
        "# REPORT: item-a, claude-1\n\nLab: claude.\n\n## What did not work, first\n\nNothing.\n\n## What changed\n\nThe close step.\n\n## Measurements\n\nNone.\n\n## Checks run at the head\n\n- tests\n\n## Skills used\n\nNone.\n\n## Still open\n\n{STILL_OPEN}\n\n## Plain summary\n\nDone.\n"
    )
}

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd.env_remove("UOR_LAB")
        .env_remove("UOR_LOOP_CLOSE_GH_FAKE")
}

struct Repo {
    _tmp: tempfile::TempDir,
    base: PathBuf,
    root: PathBuf,
}

impl Repo {
    /// `main` (pushed to a bare origin) holds the templates and the issue map; `feat` adds an
    /// ORDER and the REPORT text `report`.
    fn new(report: &str) -> Repo {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().canonicalize().unwrap();
        let root = base.join("repo");
        let origin = base.join("origin.git");
        assert!(root.starts_with(&base) && origin.starts_with(&base));
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo {
            _tmp: tmp,
            base,
            root,
        };
        r.git(&["init", "-q", "--bare", origin.to_str().unwrap()]);
        r.git(&["init", "-q", "-b", "main"]);
        r.write("docs/loop/templates/PICKUP.md", PICKUP_TEMPLATE);
        r.write("docs/loop/templates/REPORT.md", REPORT_TEMPLATE);
        r.write(
            "docs/harness/labs.json",
            r#"{"labs": {"claude": {"vendor": "a", "client": "b", "headless": "c", "open_pr_limit": 2}}}"#,
        );
        r.write("docs/roadmap/issues.json", "{\"item-a\": 4242}\n");
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "base"]);
        r.git(&["remote", "add", "origin", origin.to_str().unwrap()]);
        r.git(&["push", "-q", "origin", "main"]);
        r.git(&["fetch", "-q", "origin"]);
        r.git(&["checkout", "-q", "-b", "feat"]);
        r.write("crates/x.rs", "// work\n");
        r.write(
            ORDER,
            "# ORDER: item-a, claude-1 (t)\n\nLab: claude. Item: [item-a](../../../../roadmap/items/item-a.md) (#1).\n",
        );
        r.write(REPORT, report);
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "order"]);
        r
    }

    fn git(&self, args: &[&str]) -> String {
        let o = clean(&mut Command::new("git"))
            .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("git runs");
        assert!(
            o.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    }

    fn write(&self, rel: &str, text: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn head(&self) -> String {
        self.git(&["rev-parse", "HEAD"])
    }

    fn fake_path(&self) -> PathBuf {
        self.base.join("gh.json")
    }

    /// One PR for `feat` at `head`, plus `extra` top-level fields.
    fn fake(&self, head: &str, draft: bool, extra: &str) {
        std::fs::write(
            self.fake_path(),
            format!(
                "{{\"prs\": [{{\"number\": 7, \"headRefName\": \"feat\", \"isDraft\": {draft}, \"headRefOid\": \"{head}\"}}]{extra}}}"
            ),
        )
        .unwrap();
    }

    fn calls(&self) -> Vec<serde_json::Value> {
        let mut p = self.fake_path().into_os_string();
        p.push(".calls.jsonl");
        std::fs::read_to_string(PathBuf::from(p))
            .unwrap_or_default()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn pickup(&self) -> Option<String> {
        std::fs::read_to_string(self.root.join(PICKUP)).ok()
    }

    fn close(&self, lab: Option<&str>, args: &[&str]) -> (i32, String) {
        let mut cmd = Command::new(BIN);
        clean(&mut cmd);
        if let Some(l) = lab {
            cmd.env("UOR_LAB", l);
        }
        cmd.env("UOR_LOOP_CLOSE_GH_FAKE", self.fake_path());
        let o: Output = cmd
            .args(["--repo", self.root.to_str().unwrap(), "loop", "close"])
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("the binary runs");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        );
        (o.status.code().unwrap_or(-1), text)
    }
}

#[test]
fn a_complete_report_on_a_draft_pr_at_head_sets_the_body_marks_ready_and_writes_pickup() {
    let r = Repo::new(&valid_report());
    let head = r.head();
    r.fake(&head, true, "");
    let (code, out) = r.close(Some("claude"), &["--session"]);
    assert_eq!(code, 0, "{out}");
    let calls = r.calls();
    let kinds: Vec<&str> = calls.iter().map(|c| c["call"].as_str().unwrap()).collect();
    assert_eq!(kinds, ["set_body", "ready"], "{out}");
    assert_eq!(calls[1]["number"], 7);
    let body = calls[0]["body"].as_str().unwrap();
    assert!(body.lines().next().unwrap().starts_with("size:"), "{body}");
    assert!(body.contains("## Still open"), "{body}");
    assert!(body.contains("The close step."), "{body}");
    assert!(!body.contains("# REPORT: item-a"), "{body}");
    assert!(body.contains("References #4242"), "{body}");
    assert!(
        body.trim_end()
            .ends_with("🤖 Generated with [Claude Code](https://claude.com/claude-code)"),
        "{body}"
    );
    let pickup = r.pickup().expect("PICKUP written");
    assert!(pickup.contains("`feat`"), "{pickup}");
    assert!(pickup.contains(&head), "{pickup}");
    assert!(pickup.contains(STILL_OPEN), "{pickup}");
    assert!(pickup.contains("exact-head review"), "{pickup}");
    assert!(
        pickup.contains("docs/loop/tasks/item-a/claude-1/"),
        "{pickup}"
    );
}

#[test]
fn a_report_missing_a_section_is_refused_and_nothing_is_written() {
    let r = Repo::new(&valid_report().replace("## Skills used\n\nNone.\n\n", ""));
    r.fake(&r.head(), true, "");
    let (code, out) = r.close(Some("claude"), &["--session"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("Skills used"), "{out}");
    assert!(r.calls().is_empty(), "{out}");
    assert!(r.pickup().is_none());
}

#[test]
fn a_pr_head_other_than_head_says_push_first() {
    let r = Repo::new(&valid_report());
    r.fake("0123456789012345678901234567890123456789", true, "");
    let (code, out) = r.close(Some("claude"), &["--session"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("push first"), "{out}");
    assert!(r.calls().is_empty(), "{out}");
    assert!(r.pickup().is_none());
}

#[test]
fn a_ready_pr_gets_the_body_only() {
    let r = Repo::new(&valid_report());
    r.fake(&r.head(), false, "");
    let (code, out) = r.close(Some("claude"), &["--session", "--pr", "7"]);
    assert_eq!(code, 0, "{out}");
    let kinds: Vec<String> = r
        .calls()
        .iter()
        .map(|c| c["call"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(kinds, ["set_body"], "{out}");
    assert!(r.pickup().is_some());
}

#[test]
fn dry_run_prints_and_writes_nothing() {
    let r = Repo::new(&valid_report());
    r.fake(&r.head(), true, "");
    let (code, out) = r.close(Some("claude"), &["--session", "--dry-run"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("size:"), "{out}");
    assert!(out.contains("References #4242"), "{out}");
    assert!(out.contains(STILL_OPEN), "{out}");
    assert!(r.calls().is_empty(), "{out}");
    assert!(r.pickup().is_none());
}

#[test]
fn a_pickup_of_8192_bytes_or_more_is_refused_before_any_write() {
    let huge = format!("- {}\n", "x".repeat(9000));
    let r = Repo::new(&valid_report().replace(STILL_OPEN, &huge));
    r.fake(&r.head(), true, "");
    let (code, out) = r.close(Some("claude"), &["--session"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("8192"), "{out}");
    assert!(r.calls().is_empty(), "{out}");
    assert!(r.pickup().is_none());
}

#[test]
fn a_failed_body_write_stops_before_ready_and_pickup() {
    let r = Repo::new(&valid_report());
    r.fake(&r.head(), true, ", \"failSetBody\": true");
    let (code, out) = r.close(Some("claude"), &["--session"]);
    assert_eq!(code, 2, "{out}");
    assert!(r.calls().is_empty(), "{out}");
    assert!(r.pickup().is_none());
}

#[test]
fn a_failed_ready_says_the_body_was_already_set() {
    let r = Repo::new(&valid_report());
    r.fake(&r.head(), true, ", \"failReady\": true");
    let (code, out) = r.close(Some("claude"), &["--session"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("already set"), "{out}");
    let kinds: Vec<String> = r
        .calls()
        .iter()
        .map(|c| c["call"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(kinds, ["set_body"], "{out}");
    assert!(r.pickup().is_none());
}

#[test]
fn no_lab_no_pr_and_verify_cannot_run() {
    let r = Repo::new(&valid_report());
    r.fake(&r.head(), true, "");
    let (code, out) = r.close(None, &["--session"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("needs a lab"), "{out}");

    std::fs::write(r.fake_path(), "{\"prs\": []}").unwrap();
    let (code, out) = r.close(Some("claude"), &["--session"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("no open PR"), "{out}");

    let (code, out) = r.close(Some("claude"), &["--verify"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("not built yet"), "{out}");
    assert!(r.calls().is_empty(), "{out}");
    assert!(r.pickup().is_none());
}
