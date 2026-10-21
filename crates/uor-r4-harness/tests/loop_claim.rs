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
        let remote = r.tmp.path().canonicalize().unwrap().join("remote.git");
        let st = clean(&mut Command::new("git"))
            .args([
                "init",
                "-q",
                "--bare",
                "-b",
                "main",
                remote.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(st.status.success(), "{st:?}");
        r.git(&["remote", "add", "origin", remote.to_str().unwrap()]);
        r.git(&["push", "-q", "origin", "main"]);
        r.git(&["fetch", "-q", "origin"]);
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
    r.git(&["push", "-q", "origin", "claude/t:main"]);
    r.git(&["fetch", "-q", "origin"]);
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

impl Repo {
    fn out(&self, dir: &std::path::Path, args: &[&str]) -> Output {
        assert!(dir.starts_with(&self.root), "git runs inside the temp repo");
        clean(&mut Command::new("git"))
            .args(args)
            .current_dir(dir)
            .output()
            .expect("git runs")
    }

    fn has_branch(&self, b: &str) -> bool {
        let r = format!("refs/heads/{b}");
        self.out(&self.root, &["rev-parse", "--verify", "--quiet", &r])
            .status
            .success()
    }

    /// The fake's write log, one JSON value per line; empty when nothing was written.
    fn calls(&self) -> Vec<serde_json::Value> {
        let p = self.tmp.path().join("gh-fake.json.calls.jsonl");
        std::fs::read_to_string(p)
            .unwrap_or_default()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    /// Commit an ORDER naming `id` on a new worktree `.worktrees/claude-<name>`.
    fn order_worktree(&self, name: &str, order_rel: &str, id: &str) -> PathBuf {
        let wt = format!(".worktrees/claude-{name}");
        self.git(&[
            "worktree",
            "add",
            "-q",
            "-b",
            &format!("claude/{name}"),
            &wt,
        ]);
        let wt = self.root.join(wt);
        let o = wt.join(order_rel);
        std::fs::create_dir_all(o.parent().unwrap()).unwrap();
        std::fs::write(&o, order(id)).unwrap();
        assert!(self.out(&wt, &["add", "-A"]).status.success());
        assert!(self
            .out(&wt, &["commit", "-q", "-m", "order"])
            .status
            .success());
        wt
    }
}

fn nothing_created(r: &Repo, o: &Output) {
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(!r.has_branch("claude/ready-a"), "no branch was created");
    assert!(r.calls().is_empty(), "no GitHub write: {:?}", r.calls());
}

#[test]
fn a_clean_claim_creates_the_worktree_order_commit_and_draft_pr() {
    let r = Repo::new();
    let o = r.claim(
        "ready-a",
        serde_json::json!({"prs": [], "nextPrNumber": 77}),
    );
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("draft PR: #77"),
        "{o:?}"
    );
    let wt = r.root.join(".worktrees/claude-ready-a");
    let head = r.out(&wt, &["rev-parse", "--abbrev-ref", "HEAD"]);
    assert_eq!(
        String::from_utf8_lossy(&head.stdout).trim(),
        "claude/ready-a"
    );
    let shown = r.out(
        &wt,
        &["show", "HEAD:docs/loop/tasks/ready-a/claude-1/ORDER.md"],
    );
    assert!(shown.status.success(), "the ORDER is committed: {shown:?}");
    let text = String::from_utf8_lossy(&shown.stdout).into_owned();
    let item_line = text.lines().find(|l| l.contains("Item:")).unwrap();
    assert!(
        item_line.contains("(../../../../roadmap/items/ready-a.md)"),
        "{text}"
    );
    assert!(text.contains("- `crates/a/**`"), "{text}");
    let calls = r.calls();
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert_eq!(calls[0]["call"], "create_draft_pr");
    assert_eq!(calls[0]["branch"], "claude/ready-a");
}

#[test]
fn a_lost_race_closes_our_pr_and_removes_only_what_we_created() {
    let r = Repo::new();
    r.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "claude/keep",
        ".worktrees/claude-keep",
    ]);
    let fake = serde_json::json!({
        "prs": [], "nextPrNumber": 50,
        "afterCreate": [pr_with_order(40, "codex/a", "ready-a")]
    });
    let o = r.claim("ready-a", fake);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("lost the race to #40"),
        "{o:?}"
    );
    let calls = r.calls();
    assert_eq!(calls.len(), 3, "{calls:?}");
    assert_eq!(
        calls[1],
        serde_json::json!({"call": "close_pr", "number": 50})
    );
    assert_eq!(
        calls[2],
        serde_json::json!({"call": "delete_remote_branch", "branch": "claude/ready-a"}),
        "only our pushed branch is deleted from origin"
    );
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("deleted claude/ready-a from origin"),
        "{o:?}"
    );
    assert!(!r.root.join(".worktrees/claude-ready-a").exists());
    assert!(!r.has_branch("claude/ready-a"));
    assert!(
        r.root.join(".worktrees/claude-keep").is_dir(),
        "the unrelated worktree survives"
    );
    assert!(r.has_branch("claude/keep"));
}

#[test]
fn an_existing_target_worktree_or_branch_is_refused_and_nothing_is_created() {
    let r = Repo::new();
    let dir = r.root.join(".worktrees/claude-ready-a");
    std::fs::create_dir_all(&dir).unwrap();
    let o = r.claim("ready-a", serde_json::json!([]));
    nothing_created(&r, &o);
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("already exists"),
        "{o:?}"
    );
    std::fs::remove_dir(&dir).unwrap();
    r.git(&["branch", "claude/ready-a"]);
    let o = r.claim("ready-a", serde_json::json!([]));
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(String::from_utf8_lossy(&o.stderr).contains("branch claude/ready-a already exists"));
    assert!(!dir.exists(), "no worktree was created");
    assert!(r.calls().is_empty());
}

#[test]
fn a_squash_merged_order_no_longer_blocks_a_claim() {
    let r = Repo::new();
    let rel = "docs/loop/tasks/t/claude-1/ORDER.md";
    r.order_worktree("t", rel, "ready-a");
    refused(
        &r.claim("ready-a", serde_json::json!([])),
        "already claimed by worktree",
    );
    // Squash-merge: the same ORDER bytes land on main in a new commit; claude/t is no ancestor.
    r.write(rel, &order("ready-a"));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "squash"]);
    r.git(&["push", "-q", "origin", "main"]);
    r.git(&["fetch", "-q", "origin"]);
    let o = r.claim("ready-a", serde_json::json!([]));
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    // The next number counts the squashed claude-1 folder.
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("docs/loop/tasks/ready-a/claude-2/ORDER.md")
    );
}

#[test]
fn a_failed_fetch_exits_2_and_creates_nothing() {
    let r = Repo::new();
    r.git(&["remote", "remove", "origin"]);
    let o = r.claim("ready-a", serde_json::json!([]));
    nothing_created(&r, &o);
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("fetch"),
        "{o:?}"
    );
    assert!(!r.root.join(".worktrees").exists());
}

const PARENT: &str = "docs/loop/tasks/ready-a/claude-1/ORDER.md";

impl Repo {
    /// Run `uor-harness loop <args>` as claude, with no GitHub at all.
    fn loop_cmd(&self, args: &[&str]) -> Output {
        clean(&mut Command::new(BIN))
            .args(["--repo", self.root.to_str().unwrap(), "loop"])
            .args(args)
            .env("UOR_LAB", "claude")
            .current_dir(&self.root)
            .output()
            .expect("uor-harness runs")
    }

    /// A parent ORDER naming `ready-a`, whose scope is `[crates/a/**, docs/research/notebook/a.md]`.
    fn with_parent(self) -> Repo {
        self.write(PARENT, &order("ready-a"));
        self
    }

    fn card(&self, rel: &str) -> Option<String> {
        let p = self.root.join("local/claude/claude-1").join(rel);
        assert!(p.starts_with(&self.root));
        std::fs::read_to_string(p).ok()
    }
}

#[test]
fn suborder_accepts_a_narrower_scope_and_writes_the_order() {
    let r = Repo::new().with_parent();
    let o = r.loop_cmd(&[
        "suborder",
        "--parent",
        PARENT,
        "--scope",
        "crates/a/src/**,crates/a/x.rs",
    ]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(
        out.contains(PARENT) && out.contains("Stop and ask on any file outside this scope"),
        "{out}"
    );
    let card = r.card("orders/1.md").expect("orders/1.md is written");
    assert!(
        card.contains("- `crates/a/src/**`") && card.contains("- `crates/a/x.rs`"),
        "{card}"
    );
    let again = r.loop_cmd(&["suborder", "--parent", PARENT, "--scope", "crates/a/y.rs"]);
    assert_eq!(again.status.code(), Some(0), "{again:?}");
    assert!(
        r.card("orders/2.md").is_some(),
        "the next number is one more"
    );
}

#[test]
fn suborder_refuses_a_glob_outside_the_parent_scope() {
    let r = Repo::new().with_parent();
    let o = r.loop_cmd(&[
        "suborder",
        "--parent",
        PARENT,
        "--scope",
        "crates/a/x.rs,crates/b/y.rs",
    ]);
    refused(&o, "scope crates/b/y.rs is outside the parent's scope");
    assert!(r.card("orders/1.md").is_none(), "nothing is written");
}

#[test]
fn suborder_refuses_a_sibling_whose_prefix_only_partly_matches() {
    let r = Repo::new().with_parent();
    let o = r.loop_cmd(&["suborder", "--parent", PARENT, "--scope", "crates/ab/x.rs"]);
    refused(&o, "scope crates/ab/x.rs is outside");
    assert!(r.card("orders/1.md").is_none());
}

#[test]
fn ask_scope_writes_a_card() {
    let r = Repo::new().with_parent();
    let o = r.loop_cmd(&[
        "ask",
        "--scope",
        "--order",
        PARENT,
        "--file",
        "crates/b/lib.rs",
        "--reason",
        "the helper lives there",
        "--lines",
        "12",
        "--own-code",
        "no",
    ]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let card = r.card("asks/1.md").expect("asks/1.md is written");
    for want in [
        "`crates/b/lib.rs`",
        "the helper lives there",
        "roughly: 12",
        "instead: no",
    ] {
        assert!(card.contains(want), "{want} in {card}");
    }
}

#[test]
fn ask_blocked_writes_a_card() {
    let r = Repo::new().with_parent();
    let o = r.loop_cmd(&[
        "ask",
        "--blocked",
        "--order",
        PARENT,
        "--limit",
        "4 pods",
        "--measured",
        "uor-pod status count",
        "--command",
        "scripts/pod/uor-pod status",
        "--state",
        "not-reached",
    ]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let card = r.card("asks/1.md").expect("asks/1.md is written");
    for want in [
        "Limit: 4 pods",
        "uor-pod status count",
        "`scripts/pod/uor-pod status`",
        "State: not-reached",
    ] {
        assert!(card.contains(want), "{want} in {card}");
    }
}

#[test]
fn ask_refuses_a_missing_field() {
    let r = Repo::new().with_parent();
    let o = r.loop_cmd(&[
        "ask", "--scope", "--order", PARENT, "--file", "x.rs", "--reason", "r", "--lines", "3",
    ]);
    refused(&o, "missing --own-code");
    let o = r.loop_cmd(&[
        "ask",
        "--blocked",
        "--order",
        PARENT,
        "--limit",
        "l",
        "--measured",
        "m",
        "--state",
        "refused",
    ]);
    refused(&o, "missing --command");
    assert!(r.card("asks/1.md").is_none(), "nothing is written");
}
