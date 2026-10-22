//! A repair chain (loop step 11) keeps one item, lab and setup row: `loop pr-check`'s `order`
//! rule compares every ORDER of the chain with the active one. Each case runs against a
//! temporary git repository; every child process has every `GIT_*` variable removed, and the
//! repository is checked to sit inside its temporary directory.

use std::path::PathBuf;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");
const LABS: &str = r#"{"labs":{"claude":{"vendor":"a","client":"b","headless":"c","open_pr_limit":2},"codex":{"vendor":"d","client":"b","headless":"c","open_pr_limit":2}}}"#;
const ORDER_1: &str = "docs/loop/tasks/item-a/claude-1/ORDER.md";
const ORDER_2: &str = "docs/loop/tasks/item-a/claude-2/ORDER.md";

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd.env_remove("UOR_LAB")
        .env_remove("UOR_LOOP_GH_FAKE")
        .env_remove("UOR_PRCHECK_GH_FAKE")
}

fn order(lab: &str, id: &str, extra: &str) -> String {
    format!("# ORDER\n\nLab: {lab}. Item: [{id}](../../../../roadmap/items/{id}.md) (#1).\n{extra}")
}

struct Repo {
    _tmp: tempfile::TempDir,
    root: PathBuf,
    base: String,
}

impl Repo {
    fn new() -> Repo {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap().join("repo");
        assert!(root.starts_with(tmp.path().canonicalize().unwrap()));
        std::fs::create_dir_all(&root).unwrap();
        let mut r = Repo {
            _tmp: tmp,
            root,
            base: String::new(),
        };
        r.git(&["init", "-q", "-b", "main"]);
        r.write("docs/harness/labs.json", LABS);
        for id in ["item-a", "item-b"] {
            r.write(
                &format!("docs/roadmap/items/{id}.md"),
                &format!("---\nid: {id}\nstatus: ready\nscope: [crates/a/**]\n---\n"),
            );
        }
        r.write("crates/a/src/lib.rs", "pub fn a() {}\n");
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "base"]);
        r.base = r.out(&["rev-parse", "HEAD"]);
        r
    }

    fn cmd_git(&self, args: &[&str]) -> Output {
        clean(&mut Command::new("git"))
            .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("git runs")
    }

    fn git(&self, args: &[&str]) {
        let st = self.cmd_git(args);
        assert!(st.status.success(), "git {args:?}: {st:?}");
    }

    fn out(&self, args: &[&str]) -> String {
        let st = self.cmd_git(args);
        assert!(st.status.success(), "git {args:?}: {st:?}");
        String::from_utf8_lossy(&st.stdout).trim().to_string()
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

    /// A repair PR: `first` as `claude-1` alone, work, then `second` as `claude-2` alone.
    fn repair(&self, first: &str, second: &str) -> String {
        self.git(&["checkout", "-q", "--detach", &self.base]);
        for (path, text) in [
            (ORDER_1, first),
            ("crates/a/src/lib.rs", "//\n"),
            (ORDER_2, second),
        ] {
            self.write(path, text);
            self.git(&["add", "-A"]);
            self.git(&["commit", "-q", "-m", "step"]);
        }
        self.out(&["rev-parse", "HEAD"])
    }

    /// The `order` rule's line and the whole output of `loop pr-check --local` at `head`.
    fn order_rule(&self, head: &str) -> (String, String) {
        let o = clean(&mut Command::new(BIN))
            .args(["--repo", self.root.to_str().unwrap(), "loop", "pr-check"])
            .args(["--base", &self.base, "--head", head, "--local"])
            .env("UOR_LAB", "claude")
            .current_dir(&self.root)
            .output()
            .expect("uor-harness runs");
        let out = String::from_utf8_lossy(&o.stdout).into_owned();
        let line = out
            .lines()
            .find(|l| l.starts_with("PASS order ") || l.starts_with("FAIL order "))
            .unwrap_or_else(|| panic!("no order line:\n{out}"))
            .to_string();
        (line, out)
    }
}

#[test]
fn a_clean_chain_passes() {
    let r = Repo::new();
    let row = "Setup row: 7a.\n";
    let h = r.repair(
        &order("claude", "item-a", row),
        &order("claude", "item-a", row),
    );
    let (line, out) = r.order_rule(&h);
    assert!(line.starts_with("PASS order"), "{out}");
}

#[test]
fn a_repair_order_naming_another_item_fails() {
    let r = Repo::new();
    let h = r.repair(
        &order("claude", "item-a", ""),
        &order("claude", "item-b", ""),
    );
    let (line, out) = r.order_rule(&h);
    assert!(line.starts_with("FAIL order"), "{out}");
    assert!(
        out.contains(&format!(
            "{ORDER_1} has `Item: item-a` but the active ORDER {ORDER_2} has `Item: item-b`"
        )),
        "{out}"
    );
}

#[test]
fn an_earlier_order_naming_another_lab_fails() {
    let r = Repo::new();
    let h = r.repair(
        &order("codex", "item-a", ""),
        &order("claude", "item-a", ""),
    );
    let (line, out) = r.order_rule(&h);
    assert!(line.starts_with("FAIL order"), "{out}");
    assert!(
        out.contains(&format!(
            "{ORDER_1} says `Lab: codex` but its folder is lab claude"
        )),
        "{out}"
    );
}

#[test]
fn a_repair_order_dropping_the_setup_row_fails() {
    let r = Repo::new();
    let h = r.repair(
        &order("claude", "item-a", "Setup row: 7a.\n"),
        &order("claude", "item-a", ""),
    );
    let (line, out) = r.order_rule(&h);
    assert!(line.starts_with("FAIL order"), "{out}");
    assert!(
        out.contains(&format!(
            "{ORDER_1} has `Setup row: 7a` but the active ORDER {ORDER_2} has `Setup row: (none)`"
        )),
        "{out}"
    );
}
