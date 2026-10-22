//! The git hooks (loop PR 12): install and refresh, the commit-message check, the pre-push check,
//! and both new hook scripts end to end. Every child process has every GIT_* variable removed, each
//! repository is asserted to sit inside its temporary directory, and no test touches the network.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_uor-harness");
const PRE_COMMIT: &str = include_str!("../rules/precommit-hook.sh");
const COMMIT_MSG: &str = include_str!("../rules/commit-msg-hook.sh");
const PRE_PUSH: &str = include_str!("../rules/prepush-hook.sh");
const ORDER: &str = "docs/loop/tasks/infra-loop-harness/deepseek-2/ORDER.md";

fn clean(cmd: &mut Command) -> &mut Command {
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(&k);
        }
    }
    cmd.env_remove("UOR_LAB").env_remove("CI")
}

struct Repo {
    _tmp: tempfile::TempDir,
    root: PathBuf,
}

impl Repo {
    fn new() -> Repo {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap().join("repo");
        assert!(root.starts_with(tmp.path().canonicalize().unwrap()));
        std::fs::create_dir_all(&root).unwrap();
        let r = Repo { _tmp: tmp, root };
        for args in [
            &["init", "-q", "-b", "main"][..],
            &["config", "user.email", "t@example.invalid"][..],
            &["config", "user.name", "t"][..],
        ] {
            r.git(args);
        }
        r.write("README.md", "# r\n");
        r.write(
            "docs/harness/labs.json",
            r#"{"schema":1,"labs":{"deepseek":{"vendor":"DeepSeek","client":"dsh","headless":"dsh headless","open_pr_limit":3}}}"#,
        );
        r.git(&["add", "-A"]);
        r.git(&["commit", "-q", "-m", "init"]);
        let remote = r._tmp.path().canonicalize().unwrap().join("remote.git");
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
        assert!(p.starts_with(&self.root));
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn run(&self, args: &[&str], stdin: Option<&str>) -> Output {
        let mut built = Command::new(BIN);
        let cmd = clean(&mut built);
        cmd.args(["--repo", self.root.to_str().unwrap()])
            .args(args)
            .env("UOR_LAB", "deepseek")
            .current_dir(&self.root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd.spawn().expect("uor-harness runs");
        if let Some(text) = stdin {
            use std::io::Write;
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(text.as_bytes())
                .unwrap();
        }
        child.wait_with_output().expect("wait")
    }

    fn hooks_dir(&self) -> PathBuf {
        let raw = self.git(&["rev-parse", "--git-path", "hooks"]);
        let p = self.root.join(raw);
        assert!(p.starts_with(&self.root));
        p
    }

    fn message(&self, name: &str, body: &str) -> PathBuf {
        let p = self.root.join(name);
        assert!(p.starts_with(&self.root));
        std::fs::write(&p, body).unwrap();
        p
    }
}

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

/// Run one installed hook script with the built harness first on PATH; stdin is passed through.
fn run_script(repo: &Repo, name: &str, stdin: &str) -> Output {
    let script = repo.hooks_dir().join(name);
    let bin = Path::new(BIN).parent().expect("bin dir");
    let mut shell = Command::new("/bin/sh");
    let cmd = clean(&mut shell);
    cmd.arg(&script)
        .arg(repo.root.join("msg.txt"))
        .current_dir(&repo.root)
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("hook runs");
    use std::io::Write;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().expect("wait")
}

#[test]
fn install_hooks_writes_three_executable_hooks() {
    let r = Repo::new();
    let o = r.run(&["git", "install-hooks"], None);
    let out = text(&o);
    assert_eq!(o.status.code(), Some(0), "{out}");
    for (name, body) in [
        ("pre-commit", PRE_COMMIT),
        ("commit-msg", COMMIT_MSG),
        ("pre-push", PRE_PUSH),
    ] {
        let p = r.hooks_dir().join(name);
        assert_eq!(
            std::fs::read_to_string(&p).expect("hook reads"),
            body,
            "{name} is byte-identical to the reference"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&p).unwrap().permissions().mode();
            assert_eq!(
                mode & 0o111,
                0o111,
                "{name} must be executable, mode {mode:o}"
            );
        }
    }
    assert_eq!(out.matches("installed:").count(), 3, "{out}");
    let again = r.run(&["git", "install-hooks"], None);
    let out2 = text(&again);
    assert_eq!(again.status.code(), Some(0), "{out2}");
    assert_eq!(out2.matches("unchanged:").count(), 3, "{out2}");
}

#[test]
fn a_foreign_commit_msg_hook_is_refused_and_left_alone() {
    let r = Repo::new();
    let dir = r.hooks_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let foreign = "#!/bin/sh\n# somebody else writes this hook\nexit 0\n";
    std::fs::write(dir.join("commit-msg"), foreign).unwrap();
    let o = r.run(&["git", "install-hooks"], None);
    let out = text(&o);
    assert_eq!(o.status.code(), Some(2), "{out}");
    assert!(out.contains("was not written by the harness"), "{out}");
    assert_eq!(
        std::fs::read_to_string(dir.join("commit-msg")).unwrap(),
        foreign,
        "the foreign hook is not clobbered"
    );
}

#[test]
fn a_commit_message_without_trailers_is_refused() {
    let r = Repo::new();
    let m = r.message("msg.txt", "a subject with no trailers\n");
    let o = r.run(&["git", "check-message", m.to_str().unwrap()], None);
    let out = text(&o);
    assert_eq!(o.status.code(), Some(1), "{out}");
    assert!(out.contains("no Lab:"), "{out}");
    assert!(out.contains("no Order:"), "{out}");
    assert!(out.contains("tags, not proof"), "{out}");
}

#[test]
fn a_commit_message_with_trailers_passes() {
    let r = Repo::new();
    let body = format!("a subject\n\nwhy\n\nLab: deepseek\nOrder: {ORDER}\n");
    let m = r.message("msg.txt", &body);
    let o = r.run(&["git", "check-message", m.to_str().unwrap()], None);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    assert!(
        text(&o).is_empty(),
        "a passing message is silent: {}",
        text(&o)
    );
}

#[test]
fn a_merge_message_is_allowed_without_trailers() {
    let r = Repo::new();
    let m = r.message("msg.txt", "Merge branch main into topic\n");
    let o = r.run(&["git", "check-message", m.to_str().unwrap()], None);
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
}

#[test]
fn an_order_trailer_must_name_an_order_path() {
    let r = Repo::new();
    let m = r.message(
        "msg.txt",
        "subject\n\nLab: deepseek\nOrder: somewhere else\n",
    );
    let o = r.run(&["git", "check-message", m.to_str().unwrap()], None);
    let out = text(&o);
    assert_eq!(o.status.code(), Some(1), "{out}");
    assert!(out.contains("must name a docs/loop/tasks"), "{out}");
}

#[test]
fn the_commit_msg_hook_blocks_a_message_without_trailers() {
    let r = Repo::new();
    assert_eq!(
        r.run(&["git", "install-hooks"], None).status.code(),
        Some(0)
    );
    r.message("msg.txt", "a subject with no trailers\n");
    let blocked = run_script(&r, "commit-msg", "");
    let out = text(&blocked);
    assert_eq!(blocked.status.code(), Some(1), "{out}");
    assert!(out.contains("commit refused"), "{out}");
    let body = format!("a subject\n\nLab: deepseek\nOrder: {ORDER}\n");
    r.message("msg.txt", &body);
    let allowed = run_script(&r, "commit-msg", "");
    assert_eq!(allowed.status.code(), Some(0), "{}", text(&allowed));
}

#[test]
fn push_check_skips_a_non_lab_ref() {
    let r = Repo::new();
    let sha = r.git(&["rev-parse", "HEAD"]);
    let zeros = "0".repeat(40);
    let stdin = format!("refs/heads/main {sha} refs/heads/main {sha}\n");
    let o = r.run(&["git", "push-check"], Some(&stdin));
    let out = text(&o);
    assert_eq!(o.status.code(), Some(0), "{out}");
    assert!(out.contains("not a <lab>/<topic> branch"), "{out}");
    let del = format!("(delete) {zeros} refs/heads/deepseek/gone {zeros}\n");
    let o = r.run(&["git", "push-check"], Some(&del));
    assert_eq!(o.status.code(), Some(0), "{}", text(&o));
    assert!(text(&o).contains("deleting a branch"), "{}", text(&o));
}

#[test]
fn push_check_runs_pr_check_on_a_pushed_lab_branch() {
    let r = Repo::new();
    let sha = r.git(&["rev-parse", "HEAD"]);
    let stdin = format!("refs/heads/deepseek/topic {sha} refs/heads/deepseek/topic {sha}\n");
    let o = r.run(&["git", "push-check"], Some(&stdin));
    let out = text(&o);
    assert_eq!(o.status.code(), Some(1), "{out}");
    assert!(out.contains("check deepseek/topic at"), "{out}");
    assert!(out.contains("FAIL"), "pr-check ran and reported: {out}");
}

#[test]
fn the_prepush_hook_blocks_when_a_pushed_lab_branch_fails() {
    let r = Repo::new();
    assert_eq!(
        r.run(&["git", "install-hooks"], None).status.code(),
        Some(0)
    );
    let sha = r.git(&["rev-parse", "HEAD"]);
    let stdin = format!("refs/heads/deepseek/topic {sha} refs/heads/deepseek/topic {sha}\n");
    let o = run_script(&r, "pre-push", &stdin);
    let out = text(&o);
    assert_eq!(o.status.code(), Some(1), "{out}");
    assert!(out.contains("push blocked"), "{out}");
}

#[test]
fn a_hook_without_uor_harness_on_path_allows_the_commit() {
    let r = Repo::new();
    assert_eq!(
        r.run(&["git", "install-hooks"], None).status.code(),
        Some(0)
    );
    r.message("msg.txt", "a subject with no trailers\n");
    let script = r.hooks_dir().join("commit-msg");
    let out = clean(&mut Command::new("/bin/sh"))
        .arg(&script)
        .arg(r.root.join("msg.txt"))
        .current_dir(&r.root)
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("hook runs");
    assert_eq!(out.status.code(), Some(0), "no harness must not block");
}
