//! `uor-harness loop pr-check` against temporary git repositories: a base commit holding the
//! registries and items, and one head per case built from it. Every child process has every
//! `GIT_*` variable removed, and each repository is checked to sit inside its temporary directory.

use std::path::PathBuf;
use std::process::{Command, Output};
use uor_r4_harness::loopprcheck::glob_match;

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");

/// One change applied to the working tree before a head is committed.
type Edit = Box<dyn FnOnce(&Repo)>;

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd.env_remove("UOR_LAB").env_remove("UOR_LOOP_GH_FAKE")
}

const ITEM_A: &str = "---\nid: item-a\nstatus: ready\nscope: [crates/a/**, docs/a.md]\nnotebook: docs/research/notebook/a.md\n---\n\n## Why\n\nBecause.\n";
const NOTEBOOK: &str = "# A notebook\n\n| row | result |\n|---|---|\n| 1 | first |\n";
const FILEMAP: &str = "# FILEMAP\n\n| path | what |\n|---|---|\n| crates/a | the a crate |\n";

struct Repo {
    _tmp: tempfile::TempDir,
    root: PathBuf,
    base: String,
}

/// A lab registry naming `names`, with the fields the binary's own lab check needs.
fn labs(names: &[&str]) -> String {
    let labs: serde_json::Map<String, serde_json::Value> = names
        .iter()
        .map(|n| {
            let v = serde_json::json!({"vendor": "a", "client": "b", "headless": "c", "open_pr_limit": 2});
            (n.to_string(), v)
        })
        .collect();
    serde_json::json!({ "labs": labs }).to_string()
}

fn order(lab: &str, id: &str, extra: &str) -> String {
    format!("# ORDER\n\nLab: {lab}. Item: [{id}](../../../../roadmap/items/{id}.md) (#1).\n{extra}")
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
        r.write("docs/harness/labs.json", &labs(&["claude", "codex"]));
        r.write("docs/roadmap/items/item-a.md", ITEM_A);
        r.write(
            "docs/roadmap/items/item-noscope.md",
            "---\nid: item-noscope\nstatus: ready\n---\n\n## Why\n",
        );
        r.write(
            "docs/roadmap/items/infra-loop-pr9.md",
            "---\nid: infra-loop-pr9\nstatus: ready\nscope: [crates/a/**]\n---\n",
        );
        r.write("docs/research/notebook/a.md", NOTEBOOK);
        r.write("docs/start/FILEMAP.md", FILEMAP);
        r.write("crates/a/src/lib.rs", "pub fn a() {}\n");
        r.write("docs/a.md", "# a\n");
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

    /// A head commit built from the base by `edit`; returns its sha.
    fn head(&self, edit: impl FnOnce(&Repo)) -> String {
        self.git(&["checkout", "-q", "--detach", &self.base]);
        edit(self);
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "--allow-empty", "-m", "head"]);
        self.out(&["rev-parse", "HEAD"])
    }

    /// The ORDER for `item-a` by claude at `docs/loop/tasks/item-a/claude-1/ORDER.md`.
    fn order_a(&self) {
        self.write(
            "docs/loop/tasks/item-a/claude-1/ORDER.md",
            &order("claude", "item-a", ""),
        );
    }

    fn check(&self, head: &str, extra: &[&str]) -> Output {
        clean(&mut Command::new(BIN))
            .args(["--repo", self.root.to_str().unwrap(), "loop", "pr-check"])
            .args(["--base", &self.base, "--head", head])
            .args(extra)
            .env("UOR_LAB", "claude")
            .current_dir(&self.root)
            .output()
            .expect("uor-harness runs")
    }
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// Exit `code`, a `<verdict> <rule>` line, and `text` somewhere in the output.
fn expect(o: &Output, code: i32, line: &str, text: &str) {
    let out = stdout(o);
    assert_eq!(o.status.code(), Some(code), "{out}\n{o:?}");
    assert!(
        out.lines().any(|l| l.starts_with(&format!("{line} "))),
        "no `{line}` line:\n{out}"
    );
    assert!(out.contains(text), "no `{text}`:\n{out}");
}

#[test]
fn a_green_pr_passes_order_and_scope() {
    let r = Repo::new();
    let h = r.head(|r| {
        r.order_a();
        r.write("crates/a/src/lib.rs", "pub fn a() { let _ = 1; }\n");
        r.write("crates/a/src/new.rs", "\n");
    });
    let o = r.check(&h, &[]);
    expect(&o, 0, "PASS order", "verdict: PASS");
    expect(&o, 0, "PASS scope", "size: tests 0 file(s)");
}

#[test]
fn order_fails_for_each_broken_condition() {
    let r = Repo::new();
    let cases: Vec<(&str, Edit)> = vec![
        (
            "adds no docs/loop/tasks",
            Box::new(|r: &Repo| r.write("crates/a/src/lib.rs", "//\n")),
        ),
        (
            "says `Lab: codex` but its folder is lab claude",
            Box::new(|r: &Repo| {
                r.write(
                    "docs/loop/tasks/item-a/claude-1/ORDER.md",
                    &order("codex", "item-a", ""),
                )
            }),
        ),
        (
            "item item-zz does not exist at the merge base",
            Box::new(|r: &Repo| {
                r.write(
                    "docs/loop/tasks/item-zz/claude-1/ORDER.md",
                    &order("claude", "item-zz", ""),
                )
            }),
        ),
        (
            "lab deepseek (from the folder) is not registered",
            Box::new(|r: &Repo| {
                r.write(
                    "docs/loop/tasks/item-a/deepseek-1/ORDER.md",
                    &order("deepseek", "item-a", ""),
                )
            }),
        ),
        (
            "needs a `Setup row: <x>` line",
            Box::new(|r: &Repo| {
                r.write(
                    "docs/loop/tasks/infra-loop-pr9/claude-1/ORDER.md",
                    &order("claude", "infra-loop-pr9", ""),
                )
            }),
        ),
        (
            "the PR adds 2 ORDERs",
            Box::new(|r: &Repo| {
                r.order_a();
                r.write(
                    "docs/loop/tasks/item-a/claude-2/ORDER.md",
                    &order("claude", "item-a", ""),
                );
            }),
        ),
    ];
    for (why, edit) in cases {
        let h = r.head(edit);
        expect(&r.check(&h, &[]), 1, "FAIL order", why);
    }
    // The setup row satisfies the setup-PR condition.
    let h = r.head(|r| {
        r.write(
            "docs/loop/tasks/infra-loop-pr9/claude-1/ORDER.md",
            &order("claude", "infra-loop-pr9", "Setup row: 5\n"),
        )
    });
    expect(&r.check(&h, &[]), 0, "PASS order", "verdict: PASS");
}

#[test]
fn an_out_of_scope_change_fails_scope() {
    let r = Repo::new();
    let h = r.head(|r| {
        r.order_a();
        r.write("crates/b/x.rs", "\n");
    });
    expect(
        &r.check(&h, &[]),
        1,
        "FAIL scope",
        "crates/b/x.rs is outside item item-a's scope",
    );
}

#[test]
fn scope_refuses_a_scopeless_item_a_crossing_glob_and_a_rename_out() {
    let r = Repo::new();
    let h = r.head(|r| {
        r.write(
            "docs/loop/tasks/item-noscope/claude-1/ORDER.md",
            &order("claude", "item-noscope", ""),
        )
    });
    expect(
        &r.check(&h, &[]),
        1,
        "FAIL scope",
        "item item-noscope has no scope: at the base",
    );

    let h = r.head(|r| {
        r.order_a();
        r.write("crates/ab/x.rs", "\n");
    });
    expect(
        &r.check(&h, &[]),
        1,
        "FAIL scope",
        "crates/ab/x.rs is outside",
    );

    let h = r.head(|r| {
        r.order_a();
        r.write("crates/b/lib.rs", "");
        std::fs::remove_file(r.root.join("crates/b/lib.rs")).unwrap();
        r.git(&["mv", "crates/a/src/lib.rs", "crates/b/lib.rs"]);
    });
    let o = r.check(&h, &[]);
    expect(&o, 1, "FAIL scope", "crates/b/lib.rs is outside");
    assert!(!stdout(&o).contains("crates/a/src/lib.rs is outside"));
}

#[test]
fn scope_always_allows_status_notebook_append_mistakes_and_filemap_rows() {
    let r = Repo::new();
    let pass: Vec<Edit> = vec![
        Box::new(|r: &Repo| {
            r.write(
                "docs/roadmap/items/item-a.md",
                &ITEM_A.replace("status: ready", "status: done"),
            )
        }),
        Box::new(|r: &Repo| {
            r.write(
                "docs/research/notebook/a.md",
                &format!("{NOTEBOOK}| 2 | second |\n"),
            )
        }),
        Box::new(|r: &Repo| r.write("docs/harness/mistakes/2026-10-21-x.md", "# x\n")),
        Box::new(|r: &Repo| {
            r.write(
                "docs/start/FILEMAP.md",
                &format!("{FILEMAP}| crates/a/src/new.rs | new |\n"),
            )
        }),
    ];
    for edit in pass {
        let h = r.head(|r| {
            r.order_a();
            edit(r);
        });
        expect(&r.check(&h, &[]), 0, "PASS scope", "verdict: PASS");
    }
    let fail: Vec<(&str, Edit)> = vec![
        (
            "docs/roadmap/items/item-a.md is outside",
            Box::new(|r: &Repo| {
                r.write(
                    "docs/roadmap/items/item-a.md",
                    &ITEM_A.replace("Because.", "Because of x."),
                )
            }),
        ),
        (
            "docs/research/notebook/a.md is outside",
            Box::new(|r: &Repo| {
                r.write(
                    "docs/research/notebook/a.md",
                    &NOTEBOOK.replace("first", "rewritten"),
                )
            }),
        ),
        (
            "docs/start/FILEMAP.md is outside",
            Box::new(|r: &Repo| {
                r.write(
                    "docs/start/FILEMAP.md",
                    &FILEMAP.replace("| crates/a | the a crate |\n", ""),
                )
            }),
        ),
    ];
    for (why, edit) in fail {
        let h = r.head(|r| {
            r.order_a();
            edit(r);
        });
        expect(&r.check(&h, &[]), 1, "FAIL scope", why);
    }
}

#[test]
fn registries_are_read_at_the_merge_base() {
    let r = Repo::new();
    // Removing codex in the same PR does not unregister it for this PR.
    let h = r.head(|r| {
        r.write("docs/harness/labs.json", &labs(&["claude"]));
        r.write(
            "docs/loop/tasks/item-a/codex-1/ORDER.md",
            &order("codex", "item-a", ""),
        );
    });
    let o = r.check(&h, &[]);
    expect(
        &o,
        1,
        "PASS order",
        "docs/loop/tasks/item-a/codex-1/ORDER.md",
    );
    // Registering a new lab in the same PR does not let it use the new name.
    let h = r.head(|r| {
        r.write(
            "docs/harness/labs.json",
            &labs(&["claude", "codex", "kimi"]),
        );
        r.write(
            "docs/loop/tasks/item-a/kimi-1/ORDER.md",
            &order("kimi", "item-a", ""),
        );
    });
    expect(
        &r.check(&h, &[]),
        1,
        "FAIL order",
        "lab kimi (from the folder) is not registered",
    );
}

#[test]
fn the_matcher_never_crosses_a_slash_except_by_double_star() {
    assert!(glob_match("crates/a/*.rs", "crates/a/x.rs"));
    assert!(!glob_match("crates/a/*.rs", "crates/a/src/x.rs"));
    assert!(!glob_match("crates/a*", "crates/ab/x.rs"));
    assert!(glob_match("crates/a*", "crates/ab"));
    assert!(glob_match("docs/?.md", "docs/a.md"));
    assert!(!glob_match("docs/?.md", "docs/ab.md"));
    assert!(!glob_match("docs?a.md", "docs/a.md"));
    assert!(glob_match("crates/a/**", "crates/a"));
    assert!(glob_match("crates/a/**", "crates/a/x.rs"));
    assert!(glob_match("crates/a/**", "crates/a/src/deep/x.rs"));
    assert!(!glob_match("crates/a/**", "crates/ab/x.rs"));
    assert!(glob_match("crates/**/x.rs", "crates/x.rs"));
    assert!(glob_match("crates/**/x.rs", "crates/a/b/x.rs"));
    assert!(!glob_match("crates/**/x.rs", "crates/a/b/y.rs"));
    assert!(!glob_match("crates/a**", "crates/a/x.rs"));
}

#[test]
fn a_bad_rev_exits_2_and_json_carries_the_verdict() {
    let r = Repo::new();
    let o = clean(&mut Command::new(BIN))
        .args(["--repo", r.root.to_str().unwrap(), "loop", "pr-check"])
        .args(["--base", "no-such-rev", "--head", "HEAD"])
        .env("UOR_LAB", "claude")
        .current_dir(&r.root)
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(String::from_utf8_lossy(&o.stderr).contains("no-such-rev"));

    let h = r.head(|r| {
        r.order_a();
        r.write("crates/a/src/lib.rs", "//\n");
    });
    let o = r.check(&h, &["--json"]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).expect("one JSON object");
    assert_eq!(v["verdict"], "pass");
    assert_eq!(v["merge_base"], r.base.as_str());
    assert_eq!(v["head"], h.as_str());
    assert_eq!(v["size"]["code"]["files"], 1);
    assert_eq!(v["size"]["docs"]["files"], 1);
    assert_eq!(v["rules"][1]["rule"], "scope");
    assert_eq!(v["rules"][1]["verdict"], "pass");

    let h = r.head(|r| {
        r.order_a();
        r.write("crates/b/x.rs", "\n");
    });
    let o = r.check(&h, &["--json"]);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    assert_eq!(v["verdict"], "fail");
}
