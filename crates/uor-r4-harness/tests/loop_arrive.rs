//! `uor-harness loop arrive` against temporary git repositories with a bare local `origin`. Every
//! child process has every `GIT_*` variable removed, each repository is checked to sit inside its
//! temporary directory, and no test touches the network.

use std::path::PathBuf;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");
const HOOK: &str = include_str!("../rules/precommit-hook.sh");
const LABS: &str = r#"{"schema":1,"labs":{"claude":{"vendor":"Anthropic","client":"Claude Code","headless":"claude -p","open_pr_limit":3}}}"#;

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd.env_remove("UOR_LAB")
}

struct Repo {
    tmp: tempfile::TempDir,
    root: PathBuf,
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
        r.write(".gitignore", "/local/\n");
        r.write("docs/harness/labs.json", LABS);
        r.write(
            "docs/harness/milestones.json",
            r#"{"schema":1,"milestones":{"m1":{"id":"M1","title":"One","status":"in-progress","headline":null,"training_hold":false}}}"#,
        );
        r.write(
            "docs/roadmap/items/a.md",
            "---\nid: a\nstatus: open\n---\n\nbody\n",
        );
        r.write(
            "docs/mechanisms/README.md",
            "# Board\n\n## ACTIVE: on path (1)\n\n| Mechanism | x |\n|---|---|\n| [Zeta](z.md) | x |\n",
        );
        r.write("AGENTS.md", "# AGENTS\n");
        r.write("CHARTER.md", "# CHARTER\n");
        r.commit("init");
        let remote = r.tmp.path().canonicalize().unwrap().join("remote.git");
        let st = clean(&mut Command::new("git"))
            .args(["init", "-q", "--bare", "-b", "main"])
            .arg(&remote)
            .output()
            .unwrap();
        assert!(st.status.success(), "{st:?}");
        r.git(&["remote", "add", "origin", remote.to_str().unwrap()]);
        r.git(&["push", "-q", "origin", "main"]);
        r.git(&["fetch", "-q", "origin"]);
        r
    }

    fn git(&self, args: &[&str]) -> String {
        let st = clean(&mut Command::new("git"))
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("git runs");
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

    fn commit(&self, msg: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", msg]);
    }

    /// Commits `rel` and pushes it, so it is at origin/main.
    fn publish(&self, rel: &str, text: &str) {
        self.write(rel, text);
        self.commit(rel);
        self.git(&["push", "-q", "origin", "main"]);
    }

    fn install_hook(&self) {
        let dir = self.git(&["rev-parse", "--git-path", "hooks"]);
        let dir = self.root.join(dir);
        assert!(dir.starts_with(&self.root));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("pre-commit"), HOOK).unwrap();
    }

    fn arrive(&self, extra: &[&str], lab: Option<&str>) -> Output {
        let mut cmd = Command::new(BIN);
        clean(&mut cmd)
            .args(["--repo", self.root.to_str().unwrap()])
            .args(extra)
            .args(["loop", "arrive"])
            .current_dir(&self.root);
        if let Some(l) = lab {
            cmd.env("UOR_LAB", l);
        }
        cmd.output().expect("uor-harness runs")
    }
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

fn has_line(out: &str, prefix: &str) -> bool {
    out.lines().any(|l| l.starts_with(prefix))
}

#[test]
fn a_clean_fixture_with_the_hook_passes() {
    let r = Repo::new();
    r.install_hook();
    let o = r.arrive(&[], Some("claude"));
    let out = stdout(&o);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    for p in [
        "PASS fetch",
        "PASS hooks",
        "N/A client-hooks",
        "N/A pickup",
        "PASS views",
        "PASS read-path",
    ] {
        assert!(has_line(&out, p), "{p}: {out}");
    }
    assert!(out.contains("no view is committed yet"), "{out}");
    assert!(out.trim_end().ends_with("arrive: OK"), "{out}");
    let views = r.root.join("local/claude/views");
    for v in ["STATE.md", "HANDOFF.md", "docs/harness/LESSONS.md"] {
        assert!(views.join(v).is_file(), "{v}");
    }
}

#[test]
fn a_missing_hook_warns_and_still_exits_zero() {
    let r = Repo::new();
    let o = r.arrive(&[], Some("claude"));
    let out = stdout(&o);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(
        has_line(&out, "WARN hooks run `uor-harness git install-hooks`"),
        "{out}"
    );
}

#[test]
fn an_oversized_pickup_fails() {
    let r = Repo::new();
    r.write("local/claude/PICKUP.md", &"x".repeat(8_192));
    let o = r.arrive(&[], Some("claude"));
    let out = stdout(&o);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    assert!(has_line(&out, "FAIL pickup"), "{out}");
    assert!(out.contains("8192 bytes"), "{out}");
}

#[test]
fn an_uncited_standing_reference_fails_and_names_the_line() {
    let r = Repo::new();
    r.write(
        "local/claude/PICKUP.md",
        "# PICKUP\n\n## Standing references\n\n- reviews come from another lab (D59)\n- always use small commits\n\n## Next\n\n- uncited text here is fine\n",
    );
    let o = r.arrive(&[], Some("claude"));
    let out = stdout(&o);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    assert!(has_line(&out, "FAIL pickup"), "{out}");
    assert!(out.contains("always use small commits"), "{out}");
    assert!(!out.contains("(D59)"), "the cited line passes: {out}");
    assert!(
        !out.contains("uncited text here"),
        "only the standing section is read: {out}"
    );
}

#[test]
fn a_cited_pickup_passes() {
    let r = Repo::new();
    r.write(
        "local/claude/PICKUP.md",
        "# PICKUP\n\n## Standing references\n\n- reviews come from another lab (D59)\n- see §5.10 and L-012\n",
    );
    let o = r.arrive(&[], Some("claude"));
    let out = stdout(&o);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(has_line(&out, "PASS pickup"), "{out}");
}

#[test]
fn an_agents_md_of_81_lines_fails_the_read_path() {
    let r = Repo::new();
    r.publish("AGENTS.md", &"line\n".repeat(81));
    let o = r.arrive(&[], Some("claude"));
    let out = stdout(&o);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    assert!(has_line(&out, "FAIL read-path"), "{out}");
    assert!(out.contains("AGENTS.md has 81 lines"), "{out}");
}

#[test]
fn a_read_path_between_the_bounds_warns() {
    let r = Repo::new();
    r.publish("CHARTER.md", &"c".repeat(80_000));
    let o = r.arrive(&[], Some("claude"));
    let out = stdout(&o);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(has_line(&out, "WARN read-path"), "{out}");
}

#[test]
fn a_committed_view_that_differs_from_its_render_fails() {
    let r = Repo::new();
    r.publish("STATE.md", "<!-- stamp -->\nhand-written state\n");
    let o = r.arrive(&[], Some("claude"));
    let out = stdout(&o);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    assert!(has_line(&out, "FAIL views"), "{out}");
    assert!(
        out.contains("the committed STATE.md disagrees with its inputs at origin/main; the repository's inputs win"),
        "{out}"
    );
}

#[test]
fn a_committed_view_equal_to_its_render_passes() {
    let r = Repo::new();
    // Render once, then commit the render: the stamp line names the previous commit and the body
    // still agrees with the inputs.
    assert_eq!(r.arrive(&[], Some("claude")).status.code(), Some(0));
    let text = std::fs::read_to_string(r.root.join("local/claude/views/STATE.md")).unwrap();
    r.publish("STATE.md", &text);
    let o = r.arrive(&[], Some("claude"));
    let out = stdout(&o);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(out.contains("the committed STATE.md agree"), "{out}");
}

#[test]
fn an_unreachable_origin_fails_fetch_and_runs_the_rest() {
    let r = Repo::new();
    let missing = r.tmp.path().canonicalize().unwrap().join("missing.git");
    r.git(&["remote", "set-url", "origin", missing.to_str().unwrap()]);
    let o = r.arrive(&[], Some("claude"));
    let out = stdout(&o);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    assert!(has_line(&out, "FAIL fetch"), "{out}");
    for p in [
        "checkout",
        "binary",
        "hooks",
        "views",
        "pickup",
        "read-path",
    ] {
        assert!(
            out.lines().any(|l| l.split(' ').nth(1) == Some(p)),
            "{p}: {out}"
        );
    }
    assert!(out.trim_end().ends_with("arrive: FAIL"), "{out}");
}

#[test]
fn no_lab_exits_two() {
    let r = Repo::new();
    let o = r.arrive(&[], None);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(String::from_utf8_lossy(&o.stderr).contains("set UOR_LAB or pass --lab"));
}

#[test]
fn json_parses_and_carries_ok() {
    let r = Repo::new();
    let o = r.arrive(&["--json"], Some("claude"));
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("one JSON object");
    assert_eq!(v["ok"], true);
    assert_eq!(v["lab"], "claude");
    let checks = v["checks"].as_array().unwrap();
    assert_eq!(checks[0]["check"], "fetch");
    assert!(checks
        .iter()
        .all(|c| c["verdict"].is_string() && c["details"].is_array()));
}
