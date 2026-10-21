//! Each run rule of `uor-harness loop gates` pinned against a temporary git repository with fake
//! gates. Every child process has every `GIT_*` variable removed, and each repository is checked
//! to sit inside its own temporary directory before anything writes to it.

use std::path::PathBuf;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd.env_remove("UOR_LAB").env_remove("CARGO")
}

struct Repo {
    _tmp: tempfile::TempDir,
    root: PathBuf,
}

impl Repo {
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

    fn run(&self, args: &[&str]) -> Output {
        clean(&mut Command::new(BIN))
            .args(["--repo", self.root.to_str().unwrap(), "loop", "gates"])
            .args(args)
            .env("UOR_LAB", "claude")
            .current_dir(&self.root)
            .output()
            .expect("uor-harness runs")
    }

    fn ledger(&self) -> String {
        std::fs::read_to_string(self.root.join("local/claude/gates.jsonl")).unwrap_or_default()
    }
}

/// `gates` is a list of (id, shell script, break append text, break expect text).
fn repo(gates: &[(&str, &str, &str, &str)]) -> Repo {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap().join("repo");
    assert!(root.starts_with(tmp.path().canonicalize().unwrap()));
    std::fs::create_dir_all(&root).unwrap();
    let r = Repo { _tmp: tmp, root };
    r.git(&["init", "-q", "-b", "main"]);
    r.git(&["config", "user.email", "t@example.invalid"]);
    r.git(&["config", "user.name", "t"]);
    r.write(".gitignore", "/local/\n");
    r.write(
        "docs/harness/labs.json",
        r#"{"labs":{"claude":{"vendor":"a","client":"b","headless":"c","open_pr_limit":1}}}"#,
    );
    r.write("src/input.txt", "fine\n");
    let entries: Vec<String> = gates
        .iter()
        .map(|(id, script, append, expect)| {
            serde_json::json!({
                "id": id, "proves": "a fake gate", "command": ["sh", "-c", script],
                "reads": ["src/**"], "quick": true,
                "break": {"file": "src/input.txt", "append": append, "expect": expect}
            })
            .to_string()
        })
        .collect();
    r.write(
        "docs/harness/gates.json",
        &format!(r#"{{"version":1,"gates":[{}]}}"#, entries.join(",")),
    );
    r.git(&["add", "-A"]);
    r.git(&["commit", "-q", "-m", "init"]);
    r
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn line<'a>(out: &'a str, id: &str) -> &'a str {
    out.lines()
        .find(|l| l.split_whitespace().next() == Some(id))
        .unwrap_or_else(|| panic!("no line for {id} in:\n{out}"))
}

const OK: (&str, &str, &str, &str) = ("ok", "exit 0", "x", "x");

#[test]
fn a_gate_that_cannot_run_is_refused() {
    let r = repo(&[OK]);
    // Replace the command with a binary that does not exist.
    let g = std::fs::read_to_string(r.root.join("docs/harness/gates.json")).unwrap();
    r.write(
        "docs/harness/gates.json",
        &g.replace(r#"["sh","-c","exit 0"]"#, r#"["uor-no-such-binary-xyz"]"#),
    );
    let o = r.run(&[]);
    assert_eq!(o.status.code(), Some(1));
    assert!(
        line(&stdout(&o), "ok").contains("REFUSED"),
        "{}",
        stdout(&o)
    );
}

#[test]
fn later_gates_read_unknown_after_a_red() {
    let r = repo(&[("red", "exit 3", "x", "x"), OK]);
    let out = stdout(&r.run(&[]));
    assert!(line(&out, "red").contains("RED"), "{out}");
    assert!(line(&out, "ok").contains("UNKNOWN"), "{out}");
}

#[test]
fn keep_going_runs_the_gates_after_a_red() {
    let r = repo(&[("red", "exit 3", "x", "x"), OK]);
    let out = stdout(&r.run(&["--keep-going"]));
    assert!(line(&out, "ok").contains("GREEN"), "{out}");
}

#[test]
fn the_true_exit_code_is_kept_in_the_log_and_ledger() {
    let r = repo(&[("red", "echo boom; exit 3", "x", "x")]);
    let out = stdout(&r.run(&[]));
    assert!(line(&out, "red").contains("exit 3"), "{out}");
    let runs = r.root.join("local/claude/gates");
    let dir = std::fs::read_dir(&runs)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(std::fs::read_to_string(dir.join("red.exit")).unwrap(), "3");
    assert!(std::fs::read_to_string(dir.join("red.log"))
        .unwrap()
        .contains("boom"));
    assert!(r.ledger().contains(r#""exit":3"#), "{}", r.ledger());
}

#[test]
fn a_change_to_a_read_file_during_the_run_voids_the_gate() {
    let r = repo(&[("edits", "echo changed >> src/input.txt; exit 0", "x", "x")]);
    let out = stdout(&r.run(&[]));
    assert!(line(&out, "edits").contains("VOID"), "{out}");
}

#[test]
fn a_ledger_result_is_stale_after_an_uncommitted_edit() {
    let r = repo(&[OK]);
    assert!(r.run(&[]).status.success());
    let fresh = stdout(&r.run(&["--status"]));
    assert!(!line(&fresh, "ok").contains("STALE"), "{fresh}");
    r.write("src/input.txt", "edited\n");
    let stale = stdout(&r.run(&["--status"]));
    assert!(line(&stale, "ok").contains("STALE"), "{stale}");
}

/// A gate that fails when the break's line is present, with its own message.
const CHECKS: &str = "if grep -q PLANTED src/input.txt; then echo 'input is broken'; exit 1; fi";

fn worktrees_under_local(r: &Repo) -> usize {
    let o = clean(&mut Command::new("git"))
        .args(["worktree", "list"])
        .current_dir(&r.root)
        .output()
        .unwrap();
    stdout(&o).lines().filter(|l| l.contains("/local/")).count()
}

#[test]
fn a_break_red_for_its_own_reason_passes() {
    let r = repo(&[("checks", CHECKS, "PLANTED\n", "input is broken")]);
    let o = r.run(&["--break", "checks"]);
    assert!(
        line(&stdout(&o), "checks").contains("CAUGHT"),
        "{}",
        stdout(&o)
    );
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(worktrees_under_local(&r), 0);
}

#[test]
fn a_break_red_for_another_reason_is_refused() {
    let r = repo(&[("other", "exit 101", "PLANTED\n", "input is broken")]);
    let o = r.run(&["--break", "other"]);
    assert!(
        line(&stdout(&o), "other").contains("REFUSED"),
        "{}",
        stdout(&o)
    );
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(worktrees_under_local(&r), 0);
}

#[test]
fn a_break_that_stays_green_is_a_defect() {
    let r = repo(&[OK]);
    let o = r.run(&["--break", "ok"]);
    assert!(line(&stdout(&o), "ok").contains("DEFECT"), "{}", stdout(&o));
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn break_all_turns_each_gate_red_for_its_own_reason() {
    let second = "if grep -q PLANTED src/input.txt; then echo 'second saw it'; exit 2; fi";
    let r = repo(&[
        ("checks", CHECKS, "PLANTED\n", "input is broken"),
        ("second", second, "PLANTED\n", "second saw it"),
    ]);
    let o = r.run(&["--break", "all"]);
    let out = stdout(&o);
    assert!(line(&out, "checks").contains("CAUGHT"), "{out}");
    assert!(line(&out, "second").contains("CAUGHT"), "{out}");
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(worktrees_under_local(&r), 0);
    // The planted fault never reaches the real tree.
    let input = std::fs::read_to_string(r.root.join("src/input.txt")).unwrap();
    assert_eq!(input, "fine\n");
}

#[test]
fn watch_and_other_actions_are_refused() {
    let r = repo(&[OK]);
    assert_eq!(r.run(&["--watch"]).status.code(), Some(2));
    let o = clean(&mut Command::new(BIN))
        .args(["--repo", r.root.to_str().unwrap(), "loop", "next"])
        .env("UOR_LAB", "claude")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("not built yet"));
}
