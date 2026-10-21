//! `uor-harness loop render` on fixture repositories (loop PR 3): determinism, `--check` at the
//! stamp, the planted fault (a hand-edited STATE fails), and commit-pinned inputs. Every child
//! process has every `GIT_*` variable removed, and each repository is checked to sit inside its
//! own temporary directory before anything writes to it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    cmd
}

struct Repo {
    _tmp: tempfile::TempDir,
    root: PathBuf,
}

impl Repo {
    fn git(&self, args: &[&str]) -> String {
        let o = clean(&mut Command::new("git"))
            .arg("-C")
            .arg(&self.root)
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(o.status.success(), "git {args:?}: {o:?}");
        String::from_utf8(o.stdout).unwrap().trim().to_string()
    }

    fn write(&self, rel: &str, text: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn commit(&self, msg: &str) -> String {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", msg]);
        self.git(&["rev-parse", "HEAD"])
    }

    fn render(&self, args: &[&str]) -> Output {
        clean(&mut Command::new(BIN))
            .args(["--repo", self.root.to_str().unwrap(), "loop", "render"])
            .args(args)
            .env("UOR_LAB", "claude")
            .output()
            .unwrap()
    }

    fn out_dir(&self, name: &str) -> PathBuf {
        let d = self._tmp.path().join(name);
        assert!(d.starts_with(self._tmp.path()));
        d
    }
}

const LABS: &str = r#"{"schema":1,"labs":{"kimi":{"vendor":"Moonshot","client":"Kimi","headless":"kimi -p","open_pr_limit":3},"claude":{"vendor":"Anthropic","client":"Claude Code","headless":"claude -p","open_pr_limit":3}}}"#;

fn repo() -> Repo {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("repo");
    std::fs::create_dir_all(&root).unwrap();
    assert!(root.starts_with(tmp.path()));
    let r = Repo { _tmp: tmp, root };
    r.git(&["init", "-q", "-b", "main"]);
    r.write("docs/harness/labs.json", LABS);
    r.write(
        "docs/harness/milestones.json",
        r#"{"schema":1,"milestones":{"m2":{"id":"M2","title":"Two","status":"blocked-on-owner","headline":null,"training_hold":true},"m1":{"id":"M1","title":"One","status":"in-progress","headline":{"label":"reply acceptable","current":37,"target":116,"of":232,"unit":"replies"},"training_hold":false}}}"#,
    );
    r.write(
        "docs/roadmap/items/b.md",
        "---\nid: b\nstatus: open\n---\n\nbody\n",
    );
    r.write(
        "docs/roadmap/items/a.md",
        "---\nid: a\nstatus: held\n---\n\nbody\n",
    );
    r.write(
        "docs/research/notebook/line-a.md",
        "# Notebook\n\n| Date | What ran | Number | Bar | Verdict | Artifact |\n|---|---|---|---|---|---|\n| 2026-10-13 | first | 1 | 2 | MISS | x |\n| 2026-10-14 | second | 3 | 2 | PASS | y |\n",
    );
    r.write(
        "docs/research/notebook/line-b.md",
        "# Notebook\n\n| Date | What ran | Number | Bar | Verdict | Artifact |\n|---|---|---|---|---|---|\n| | *No result rows yet.* | | | | |\n",
    );
    r.write(
        "docs/mechanisms/README.md",
        "# Board\n\n## ACTIVE: on path (1)\n\n| Mechanism | x |\n|---|---|\n| [Zeta](z.md) | x |\n\n## PARKED: closed (1)\n\n| Mechanism | x |\n|---|---|\n| [Alpha](a.md) | x |\n\n## Other\n\n| [Ignored](i.md) | x |\n",
    );
    r.commit("fixture");
    r
}

fn state(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("STATE.md")).unwrap()
}

#[test]
fn two_renders_of_one_commit_are_byte_identical() {
    let r = repo();
    let (a, b) = (r.out_dir("a"), r.out_dir("b"));
    assert!(r.render(&["--out", a.to_str().unwrap()]).status.success());
    assert!(r.render(&["--out", b.to_str().unwrap()]).status.success());
    let s = state(&a);
    assert_eq!(s, state(&b));
    let head = r.git(&["rev-parse", "HEAD"]);
    assert_eq!(
        s.lines().next().unwrap(),
        format!("<!-- rendered by uor-harness loop render from {head}; do not edit -->")
    );
    // Every section, in order, with sorted rows.
    let order = [
        "## Milestone bars and holds",
        "| M1 One | in-progress | reply acceptable: 37 of target 116 (out of 232 replies) | released |",
        "| M2 Two | blocked-on-owner | none | HELD |",
        "## Roadmap items by status",
        "| held | 1 |",
        "| open | 1 |",
        "## Newest notebook row per research line",
        "| line-a | 2026-10-14 | 3 | 2 | PASS |",
        "| line-b | no rows yet | | | |",
        "## Mechanisms",
        "- ACTIVE: Zeta",
        "- PARKED: Alpha",
        "## Lab registry",
        "| claude | Anthropic | Claude Code | 3 |",
        "| kimi | Moonshot | Kimi | 3 |",
    ];
    let mut at = 0;
    for want in order {
        let pos = s[at..]
            .find(want)
            .unwrap_or_else(|| panic!("{want:?} in\n{s}"));
        at += pos + want.len();
    }
    assert!(!s.contains("Ignored"), "{s}");
}

#[test]
fn check_passes_on_a_fresh_render() {
    let r = repo();
    assert!(r.render(&[]).status.success());
    assert!(r.root.join("STATE.md").exists());
    let o = r.render(&["--check"]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
}

/// The planted fault: a hand edit to STATE.md must turn `--check` red.
#[test]
fn a_hand_edited_state_fails_check() {
    let r = repo();
    assert!(r.render(&[]).status.success());
    let p = r.root.join("STATE.md");
    let edited = std::fs::read_to_string(&p)
        .unwrap()
        .replace("| HELD |", "| released |");
    std::fs::write(&p, edited).unwrap();
    let o = r.render(&["--check"]);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains("first at line"), "{out}");
    assert!(out.contains("| M2 Two |"), "{out}");
}

#[test]
fn a_garbled_or_missing_stamp_exits_2() {
    let r = repo();
    let o = r.render(&["--check"]);
    assert_eq!(o.status.code(), Some(2), "missing file: {o:?}");
    assert!(r.render(&[]).status.success());
    let p = r.root.join("STATE.md");
    let text = std::fs::read_to_string(&p).unwrap();
    std::fs::write(&p, text.replacen("from ", "from x", 1)).unwrap();
    let o = r.render(&["--check"]);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("stamp"),
        "{o:?}"
    );
}

#[test]
fn a_later_input_change_leaves_the_earlier_render_alone() {
    let r = repo();
    let first = r.git(&["rev-parse", "HEAD"]);
    let before = r.out_dir("before");
    assert!(r
        .render(&["--out", before.to_str().unwrap()])
        .status
        .success());
    r.write(
        "docs/harness/labs.json",
        &LABS.replace("Moonshot", "Changed"),
    );
    r.commit("change a lab");
    // An uncommitted edit is not an input either.
    r.write("docs/roadmap/items/c.md", "---\nid: c\nstatus: open\n---\n");
    let again = r.out_dir("again");
    let o = r.render(&["--out", again.to_str().unwrap(), "--at", &first]);
    assert!(o.status.success(), "{o:?}");
    assert_eq!(state(&before), state(&again));
    let o = r.render(&["--check", "--out", before.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let head = r.out_dir("head");
    assert!(r
        .render(&["--out", head.to_str().unwrap()])
        .status
        .success());
    assert!(state(&head).contains("| kimi | Changed |"));
    assert!(
        !state(&head).contains("| open | 2 |"),
        "working tree was read"
    );
}

#[test]
fn at_renders_an_older_commit() {
    let r = repo();
    let first = r.git(&["rev-parse", "HEAD"]);
    r.write("docs/roadmap/items/c.md", "---\nid: c\nstatus: open\n---\n");
    r.commit("add an item");
    let old = r.out_dir("old");
    assert!(r
        .render(&["--out", old.to_str().unwrap(), "--at", "HEAD~1"])
        .status
        .success());
    let s = state(&old);
    assert!(s.starts_with(&format!(
        "<!-- rendered by uor-harness loop render from {first};"
    )));
    assert!(s.contains("| open | 1 |"), "{s}");
    let o = r.render(&["--at", "no-such-ref", "--out", old.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(2), "{o:?}");
}
