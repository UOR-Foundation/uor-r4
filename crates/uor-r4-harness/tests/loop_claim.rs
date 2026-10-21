//! Each refusal of `uor-harness loop claim` pinned against a temporary git repository, with GitHub
//! replaced by the test-only `UOR_LOOP_GH_FAKE` file. Every child process has every `GIT_*`
//! variable removed, and each repository is checked to sit inside its temporary directory.

use std::path::PathBuf;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd.env_remove("UOR_LAB").env_remove("UOR_LOOP_GH_FAKE")
}

struct Repo {
    tmp: tempfile::TempDir,
    root: PathBuf,
}

fn item(status: &str, scope: Option<&str>) -> String {
    let scope = scope.map(|s| format!("scope: {s}\n")).unwrap_or_default();
    format!("---\nid: x\nstatus: {status}\n{scope}---\n\n## Why\n")
}

fn order(id: &str) -> String {
    format!("# ORDER\n\nLab: codex. Item: [{id}](../../../../roadmap/items/{id}.md) (#1).\n")
}

impl Repo {
    fn new() -> Repo {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap().join("repo");
        assert!(root.starts_with(tmp.path().canonicalize().unwrap()));
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo { tmp, root };
        r.git(&["init", "-q", "-b", "main"]);
        r.git(&["config", "user.email", "t@example.invalid"]);
        r.git(&["config", "user.name", "t"]);
        r.write(".gitignore", "/local/\n/.worktrees/\n");
        r.write(
            "docs/harness/labs.json",
            r#"{"labs":{"claude":{"vendor":"a","client":"b","headless":"c","open_pr_limit":2}}}"#,
        );
        let items = [
            (
                "ready-a",
                item("ready", Some("[crates/a/**, docs/research/notebook/a.md]")),
            ),
            ("draft-b", item("draft", Some("[crates/b/**]"))),
            ("noscope-c", item("ready", None)),
            ("other-d", item("ready", Some("[crates/a/src/*.rs]"))),
            ("nb-e", item("ready", Some("[docs/research/notebook/a.md]"))),
        ];
        for (id, text) in items {
            r.write(&format!("docs/roadmap/items/{id}.md"), &text);
        }
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "init"]);
        r.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        r
    }

    fn git(&self, args: &[&str]) {
        let st = clean(&mut Command::new("git"))
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("git runs");
        assert!(st.status.success(), "git {args:?}: {st:?}");
    }

    fn write(&self, rel: &str, text: &str) {
        let p = self.root.join(rel);
        assert!(
            p.starts_with(&self.root),
            "write stays inside the temp repo"
        );
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    /// Run `loop claim <item>` as claude with `prs` as the fake `gh pr list` output.
    fn claim(&self, item: &str, prs: serde_json::Value) -> Output {
        let fake = self.tmp.path().join("gh-fake.json");
        std::fs::write(&fake, prs.to_string()).unwrap();
        clean(&mut Command::new(BIN))
            .args(["--repo", self.root.to_str().unwrap(), "loop", "claim", item])
            .env("UOR_LAB", "claude")
            .env("UOR_LOOP_GH_FAKE", &fake)
            .current_dir(&self.root)
            .output()
            .expect("uor-harness runs")
    }
}

fn pr_with_order(number: u64, branch: &str, id: &str) -> serde_json::Value {
    let path = format!("docs/loop/tasks/t/codex-{number}/ORDER.md");
    serde_json::json!({
        "number": number, "headRefName": branch, "isDraft": true,
        "files": [{"path": path}, {"path": "crates/z/x.rs"}],
        "orderTexts": {(path.clone()): order(id)}
    })
}

fn refused(o: &Output, why: &str) {
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(err.contains("refused") && err.contains(why), "{err}");
}

#[test]
fn a_missing_or_unready_item_is_refused() {
    let r = Repo::new();
    refused(
        &r.claim("no-such-item", serde_json::json!([])),
        "does not exist",
    );
    refused(&r.claim("draft-b", serde_json::json!([])), "not \"ready\"");
}

#[test]
fn an_item_named_by_an_open_order_is_refused() {
    let r = Repo::new();
    // An open PR's ORDER names the item.
    let prs = serde_json::json!([pr_with_order(7, "codex/a", "ready-a")]);
    refused(&r.claim("ready-a", prs), "already claimed by PR #7");
    // An unmerged worktree of this lab names it.
    r.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "claude/t",
        ".worktrees/claude-t",
    ]);
    let wt = r.root.join(".worktrees/claude-t");
    assert!(wt.starts_with(&r.root));
    let o = wt.join("docs/loop/tasks/t/claude-1/ORDER.md");
    std::fs::create_dir_all(o.parent().unwrap()).unwrap();
    std::fs::write(&o, order("ready-a")).unwrap();
    let wgit = |args: &[&str]| {
        let st = clean(&mut Command::new("git"))
            .args(args)
            .current_dir(&wt)
            .output()
            .unwrap();
        assert!(st.status.success(), "{st:?}");
    };
    wgit(&["add", "-A"]);
    wgit(&["commit", "-q", "-m", "order"]);
    refused(
        &r.claim("ready-a", serde_json::json!([])),
        "already claimed by worktree",
    );
    // Once its commit is in origin/main, the worktree's order is no longer open.
    r.git(&["update-ref", "refs/remotes/origin/main", "claude/t"]);
    let ok = r.claim("ready-a", serde_json::json!([]));
    assert_eq!(ok.status.code(), Some(0), "{ok:?}");
}

#[test]
fn an_item_without_scope_is_refused() {
    let r = Repo::new();
    refused(
        &r.claim("noscope-c", serde_json::json!([])),
        "has no scope: line",
    );
}

#[test]
fn an_overlapping_claim_is_refused() {
    let r = Repo::new();
    let prs = serde_json::json!([pr_with_order(9, "codex/d", "other-d")]);
    refused(
        &r.claim("ready-a", prs),
        "scope crates/a/** overlaps crates/a/src/*.rs",
    );
}

#[test]
fn a_notebook_page_overlap_is_allowed() {
    let r = Repo::new();
    let prs = serde_json::json!([pr_with_order(9, "codex/e", "nb-e")]);
    let o = r.claim("ready-a", prs);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
}

#[test]
fn a_lab_at_its_open_pr_limit_is_refused() {
    let r = Repo::new();
    let pr = |n: u64, b: &str| serde_json::json!({"number": n, "headRefName": b, "isDraft": false, "files": []});
    let prs = serde_json::json!([pr(1, "claude/x"), pr(2, "claude/y"), pr(3, "codex/z")]);
    refused(&r.claim("ready-a", prs), "at its open_pr_limit of 2");
    let under = serde_json::json!([pr(1, "claude/x"), pr(3, "codex/z")]);
    assert_eq!(r.claim("ready-a", under).status.code(), Some(0));
}

#[test]
fn a_clean_claim_passes_and_writes_nothing() {
    let r = Repo::new();
    let o = r.claim("ready-a", serde_json::json!([]));
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(String::from_utf8_lossy(&o.stdout)
        .contains("claim checks passed; writes come in the next commit"));
    let st = clean(&mut Command::new("git"))
        .args(["status", "--porcelain", "--ignored"])
        .current_dir(&r.root)
        .output()
        .unwrap();
    assert!(
        st.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&st.stdout)
    );
}
