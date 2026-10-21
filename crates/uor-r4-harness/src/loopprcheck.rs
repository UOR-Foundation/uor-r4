//! `uor-harness loop pr-check`: the rules a loop PR must pass, read from git alone.
//!
//! The PR is the diff from `git merge-base <base> <head>` to `<head>`. Registries, items and
//! budgets are read at the merge base, never at the head, so a PR cannot loosen its own check;
//! the files the PR adds or changes are read at the head. Nothing reads the working tree, GitHub
//! or the network. A rule is one function plus one entry in [`RULES`].

use std::path::Path;
use std::process::Command;

const USAGE: &str = "usage: uor-harness loop pr-check [--base <rev>] [--head <rev>] [--json]";

/// Every rule, in the order it runs and prints.
const RULES: &[(&str, Rule)] = &[("order", rule_order), ("scope", rule_scope)];

type Rule = fn(&Pr) -> RuleResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
    Warn,
    NotApplicable,
}

impl Verdict {
    fn text(self) -> &'static str {
        match self {
            Verdict::Pass => "PASS",
            Verdict::Fail => "FAIL",
            Verdict::Warn => "WARN",
            Verdict::NotApplicable => "N/A",
        }
    }
}

#[derive(Debug)]
pub struct RuleResult {
    pub rule: &'static str,
    pub verdict: Verdict,
    pub summary: String,
    pub details: Vec<String>,
}

/// One changed file: its `--name-status` letter and its paths (old then new for a rename or copy).
#[derive(Debug)]
pub struct Change {
    pub status: char,
    pub paths: Vec<String>,
}

/// The PR under check: the merge base, the head and what changed between them.
pub struct Pr<'a> {
    root: &'a Path,
    pub merge_base: String,
    pub head: String,
    pub changes: Vec<Change>,
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
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
        .args(["-c", "core.quotepath=false"])
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
    Ok(out.stdout)
}

fn resolve(root: &Path, rev: &str) -> Result<String, String> {
    let spec = format!("{rev}^{{commit}}");
    let out = git(root, &["rev-parse", "--verify", "--quiet", &spec])
        .map_err(|_| format!("{rev:?} does not name a commit"))?;
    Ok(String::from_utf8_lossy(&out).trim().to_string())
}

fn z_fields(bytes: &[u8]) -> Vec<String> {
    bytes
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect()
}

impl<'a> Pr<'a> {
    pub fn open(root: &'a Path, base: &str, head: &str) -> Result<Pr<'a>, String> {
        let base_sha = resolve(root, base)?;
        let head = resolve(root, head)?;
        let mb = git(root, &["merge-base", &base_sha, &head])?;
        let merge_base = String::from_utf8_lossy(&mb).trim().to_string();
        let raw = git(
            root,
            &["diff", "-z", "--name-status", "-M", &merge_base, &head],
        )?;
        let fields = z_fields(&raw);
        let mut changes = Vec::new();
        let mut i = 0;
        while i < fields.len() {
            let status = fields[i].chars().next().unwrap_or('?');
            let n = if matches!(status, 'R' | 'C') { 2 } else { 1 };
            let paths = fields
                .get(i + 1..i + 1 + n)
                .ok_or("git diff --name-status output is truncated")?
                .to_vec();
            changes.push(Change { status, paths });
            i += 1 + n;
        }
        Ok(Pr {
            root,
            merge_base,
            head,
            changes,
        })
    }

    /// A file's text at the merge base, `None` when it is not there.
    pub fn at_base(&self, path: &str) -> Option<String> {
        self.show(&self.merge_base, path)
    }

    /// A file's text at the head, `None` when it is not there.
    pub fn at_head(&self, path: &str) -> Option<String> {
        self.show(&self.head, path)
    }

    fn show(&self, rev: &str, path: &str) -> Option<String> {
        git(self.root, &["show", &format!("{rev}:{path}")])
            .ok()
            .map(|b| String::from_utf8_lossy(&b).into_owned())
    }

    /// Every path the PR touches, both sides of a rename or copy.
    pub fn changed_paths(&self) -> impl Iterator<Item = (&Change, &str)> {
        self.changes
            .iter()
            .flat_map(|c| c.paths.iter().map(move |p| (c, p.as_str())))
    }

    /// The ORDER files the PR adds (an add, or the new side of a rename or copy).
    pub fn added_orders(&self) -> Vec<&str> {
        self.changes
            .iter()
            .filter(|c| matches!(c.status, 'A' | 'R' | 'C'))
            .filter_map(|c| c.paths.last().map(String::as_str))
            .filter(|p| order_parts(p).is_some())
            .collect()
    }

    /// The single ORDER the PR adds, or the reason there is not exactly one.
    fn the_order(&self) -> Result<&str, String> {
        match self.added_orders().as_slice() {
            [one] => Ok(one),
            [] => Err("the PR adds no docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md".into()),
            many => Err(format!(
                "the PR adds {} ORDERs: {}",
                many.len(),
                many.join(", ")
            )),
        }
    }
}

/// `(folder, lab, n)` of `docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md`.
fn order_parts(path: &str) -> Option<(String, String, String)> {
    let re = regex::Regex::new(r"^docs/loop/tasks/([^/]+)/([A-Za-z0-9_-]+)-([0-9]+)/ORDER\.md$")
        .expect("a valid pattern");
    let c = re.captures(path)?;
    Some((c[1].to_string(), c[2].to_string(), c[3].to_string()))
}

fn result(rule: &'static str, details: Vec<String>, pass: &str, fail: &str) -> RuleResult {
    let verdict = if details.is_empty() {
        Verdict::Pass
    } else {
        Verdict::Fail
    };
    let summary = if details.is_empty() { pass } else { fail };
    RuleResult {
        rule,
        verdict,
        summary: summary.to_string(),
        details,
    }
}

fn fail(rule: &'static str, summary: String) -> RuleResult {
    RuleResult {
        rule,
        verdict: Verdict::Fail,
        summary,
        details: Vec::new(),
    }
}

/// The labs registered in `docs/harness/labs.json` at the merge base.
fn base_labs(pr: &Pr) -> Result<Vec<String>, String> {
    let text = pr
        .at_base("docs/harness/labs.json")
        .ok_or("docs/harness/labs.json is missing at the merge base")?;
    let v: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("docs/harness/labs.json at the merge base: {e}"))?;
    let labs = v
        .get("labs")
        .and_then(|l| l.as_object())
        .ok_or("docs/harness/labs.json at the merge base has no `labs` object")?;
    Ok(labs.keys().cloned().collect())
}

/// The PR adds exactly one ORDER, by a registered lab, naming an item that exists at the base.
fn rule_order(pr: &Pr) -> RuleResult {
    const R: &str = "order";
    let path = match pr.the_order() {
        Ok(p) => p,
        Err(e) => return fail(R, e),
    };
    let (_, lab, _) = order_parts(path).expect("an added ORDER path parses");
    let Some(text) = pr.at_head(path) else {
        return fail(R, format!("{path} cannot be read at the head"));
    };
    let mut details = Vec::new();
    match base_labs(pr) {
        Ok(labs) if labs.contains(&lab) => {}
        Ok(_) => details.push(format!(
            "lab {lab} (from the folder) is not registered in docs/harness/labs.json at the merge base"
        )),
        Err(e) => details.push(e),
    }
    let lab_re = regex::Regex::new(r"\bLab:\s*([A-Za-z0-9_-]+)").expect("a valid pattern");
    match lab_re.captures(&text) {
        Some(c) if c[1] == lab => {}
        Some(c) => details.push(format!(
            "the ORDER says `Lab: {}` but its folder is lab {lab}",
            &c[1]
        )),
        None => details.push("the ORDER has no `Lab: <lab>` line".into()),
    }
    match crate::loopclaim::order_item(&text) {
        None => {
            details.push("the ORDER names no item (`Item: [..](roadmap/items/<id>.md)`)".into())
        }
        Some(id) => {
            if pr.at_base(&format!("docs/roadmap/items/{id}.md")).is_none() {
                details.push(format!(
                    "item {id} does not exist at the merge base (docs/roadmap/items/{id}.md)"
                ));
            }
            let setup_re = regex::Regex::new(r"Setup row:\s*\S").expect("a valid pattern");
            if id.starts_with("infra-loop-pr") && !setup_re.is_match(&text) {
                details.push(format!(
                    "item {id} is a loop setup PR, so the ORDER needs a `Setup row: <x>` line"
                ));
            }
        }
    }
    result(R, details, path, &format!("{path} is not a valid ORDER"))
}

/// Every changed path lies inside the ORDER item's `scope:` (read at the base) or is always allowed.
fn rule_scope(pr: &Pr) -> RuleResult {
    const R: &str = "scope";
    let order = match pr.the_order() {
        Ok(p) => p,
        Err(e) => return fail(R, format!("cannot check scope: {e}")),
    };
    let Some(order_text) = pr.at_head(order) else {
        return fail(
            R,
            format!("cannot check scope: {order} cannot be read at the head"),
        );
    };
    let Some(id) = crate::loopclaim::order_item(&order_text) else {
        return fail(R, format!("cannot check scope: {order} names no item"));
    };
    let item_path = format!("docs/roadmap/items/{id}.md");
    let Some(item_text) = pr.at_base(&item_path) else {
        return fail(
            R,
            format!("cannot check scope: item {id} does not exist at the merge base"),
        );
    };
    let item = match crate::loopclaim::parse_item(&item_text) {
        Ok(i) => i,
        Err(e) => {
            return fail(
                R,
                format!("cannot check scope: item {id} at the merge base {e}"),
            )
        }
    };
    let Some(scope) = item.scope else {
        return fail(R, format!("item {id} has no scope: at the base"));
    };
    let bad_globs: Vec<String> = scope
        .iter()
        .filter_map(|g| crate::loopclaim::scope_glob_ok(g).err())
        .collect();
    if !bad_globs.is_empty() {
        return RuleResult {
            rule: R,
            verdict: Verdict::Fail,
            summary: format!("item {id} has an invalid scope glob at the base"),
            details: bad_globs,
        };
    }
    let folder = order.trim_end_matches("ORDER.md");
    let mut details = Vec::new();
    for (change, path) in pr.changed_paths() {
        let allowed = scope.iter().any(|g| glob_match(g, path))
            || path.starts_with(folder)
            || (path == item_path && status_only_edit(pr, path))
            || (item.notebook.as_deref() == Some(path) && append_only(pr, path))
            || (change.status == 'A' && path.starts_with("docs/harness/mistakes/"))
            || (path == "docs/start/FILEMAP.md" && keeps_base_lines(pr, path));
        if !allowed {
            details.push(format!("{path} is outside item {id}'s scope"));
        }
    }
    details.dedup();
    let n = pr.changed_paths().count();
    result(
        R,
        details,
        &format!(
            "{n} changed path(s) inside item {id}'s scope [{}]",
            scope.join(", ")
        ),
        &format!("changes outside item {id}'s scope [{}]", scope.join(", ")),
    )
}

/// Base and head differ only on `status:` lines.
fn status_only_edit(pr: &Pr, path: &str) -> bool {
    let (Some(b), Some(h)) = (pr.at_base(path), pr.at_head(path)) else {
        return false;
    };
    let (b, h): (Vec<&str>, Vec<&str>) = (b.lines().collect(), h.lines().collect());
    b.len() == h.len()
        && b.iter()
            .zip(&h)
            .all(|(x, y)| x == y || (x.starts_with("status:") && y.starts_with("status:")))
}

/// The head text starts with the base text; a page absent at the base counts as empty.
fn append_only(pr: &Pr, path: &str) -> bool {
    let Some(h) = pr.at_head(path) else {
        return false;
    };
    h.starts_with(&pr.at_base(path).unwrap_or_default())
}

/// Every base line is still present at the head, in order (rows added only).
fn keeps_base_lines(pr: &Pr, path: &str) -> bool {
    let Some(h) = pr.at_head(path) else {
        return false;
    };
    let b = pr.at_base(path).unwrap_or_default();
    let mut head = h.lines();
    b.lines().all(|bl| head.any(|hl| hl == bl))
}

/// Separator-aware glob match: `*` and `?` never match `/`; a segment that is exactly `**`
/// matches zero or more whole segments.
pub fn glob_match(glob: &str, path: &str) -> bool {
    let g: Vec<&str> = glob.split('/').collect();
    let p: Vec<&str> = path.split('/').collect();
    match_segments(&g, &p)
}

fn match_segments(g: &[&str], p: &[&str]) -> bool {
    match g.split_first() {
        None => p.is_empty(),
        Some((&"**", rest)) => (0..=p.len()).any(|i| match_segments(rest, &p[i..])),
        Some((seg, rest)) => {
            !p.is_empty()
                && segment_match(seg.as_bytes(), p[0].as_bytes())
                && match_segments(rest, &p[1..])
        }
    }
}

fn segment_match(pat: &[u8], s: &[u8]) -> bool {
    match pat.split_first() {
        None => s.is_empty(),
        Some((b'*', rest)) => (0..=s.len()).any(|i| segment_match(rest, &s[i..])),
        Some((b'?', rest)) => !s.is_empty() && segment_match(rest, &s[1..]),
        Some((c, rest)) => s.first() == Some(c) && segment_match(rest, &s[1..]),
    }
}

/// The size class of a changed path.
fn size_class(path: &str) -> &'static str {
    let name = path.rsplit('/').next().unwrap_or(path);
    if path.split('/').any(|s| s == "tests")
        || name.ends_with("_test.rs")
        || name.ends_with("tests.rs")
    {
        "tests"
    } else if path == "Cargo.lock" || path == "docs/roadmap/README.md" {
        "generated"
    } else if [".rs", ".toml", ".yml", ".yaml", ".sh"]
        .iter()
        .any(|e| name.ends_with(e))
    {
        "code"
    } else if name.ends_with(".md") {
        "docs"
    } else {
        "other"
    }
}

const CLASSES: [&str; 5] = ["tests", "code", "docs", "generated", "other"];

/// `(files, added, removed)` per class, from `git diff --numstat` (binary files count 0 lines).
fn size(pr: &Pr) -> Result<Vec<(&'static str, u64, u64, u64)>, String> {
    let raw = git(
        pr.root,
        &["diff", "-z", "--numstat", "-M", &pr.merge_base, &pr.head],
    )?;
    let fields = z_fields(&raw);
    let mut out: Vec<(&'static str, u64, u64, u64)> =
        CLASSES.iter().map(|c| (*c, 0, 0, 0)).collect();
    let mut i = 0;
    while i < fields.len() {
        let mut cols = fields[i].splitn(3, '\t');
        let add = cols.next().unwrap_or("-").parse::<u64>().unwrap_or(0);
        let del = cols.next().unwrap_or("-").parse::<u64>().unwrap_or(0);
        let path = cols.next().unwrap_or("");
        // A rename prints an empty path, then the old and the new path as their own fields.
        let (path, step) = if path.is_empty() {
            (fields.get(i + 2).map(String::as_str).unwrap_or(""), 3)
        } else {
            (path, 1)
        };
        let class = size_class(path);
        let row = out
            .iter_mut()
            .find(|r| r.0 == class)
            .expect("a known class");
        row.1 += 1;
        row.2 += add;
        row.3 += del;
        i += step;
    }
    Ok(out)
}

/// `loop pr-check [--base <rev>] [--head <rev>] [--json]`. Exit 0 every rule passes, 1 a rule
/// failed, 2 the check could not run.
pub fn cli(root: &Path, args: &[String]) -> u8 {
    let (mut base, mut head, mut json) = ("origin/main".to_string(), "HEAD".to_string(), false);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            flag @ ("--base" | "--head") => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("{USAGE}");
                    return 2;
                };
                if flag == "--base" {
                    base = v.clone();
                } else {
                    head = v.clone();
                }
                i += 1;
            }
            _ => {
                eprintln!("{USAGE}");
                return 2;
            }
        }
        i += 1;
    }
    let pr = match Pr::open(root, &base, &head) {
        Ok(pr) => pr,
        Err(e) => {
            eprintln!("uor-harness: loop pr-check: {e}");
            return 2;
        }
    };
    let size = match size(&pr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("uor-harness: loop pr-check: {e}");
            return 2;
        }
    };
    let results: Vec<RuleResult> = RULES
        .iter()
        .map(|(name, rule)| {
            let r = rule(&pr);
            debug_assert_eq!(&r.rule, name);
            r
        })
        .collect();
    let failed = results.iter().any(|r| r.verdict == Verdict::Fail);
    if json {
        let size_obj: serde_json::Map<String, serde_json::Value> = size
            .iter()
            .map(|(c, f, a, d)| {
                (
                    c.to_string(),
                    serde_json::json!({"files": f, "added": a, "removed": d}),
                )
            })
            .collect();
        let rules: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "rule": r.rule,
                    "verdict": r.verdict.text().to_lowercase(),
                    "summary": r.summary,
                    "details": r.details,
                })
            })
            .collect();
        let out = serde_json::json!({
            "base": base,
            "merge_base": pr.merge_base,
            "head": pr.head,
            "verdict": if failed { "fail" } else { "pass" },
            "size": size_obj,
            "rules": rules,
        });
        println!("{out}");
    } else {
        let parts: Vec<String> = size
            .iter()
            .map(|(c, f, a, d)| format!("{c} {f} file(s) +{a}/-{d}"))
            .collect();
        println!("size: {}", parts.join(", "));
        for r in &results {
            println!("{} {} {}", r.verdict.text(), r.rule, r.summary);
            for d in &r.details {
                println!("  {d}");
            }
        }
        println!("verdict: {}", if failed { "FAIL" } else { "PASS" });
    }
    u8::from(failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_classes() {
        assert_eq!(size_class("crates/h/tests/x.rs"), "tests");
        assert_eq!(size_class("crates/h/src/foo_test.rs"), "tests");
        assert_eq!(size_class("crates/h/src/lib.rs"), "code");
        assert_eq!(size_class("Cargo.lock"), "generated");
        assert_eq!(size_class("docs/roadmap/README.md"), "generated");
        assert_eq!(size_class("docs/a.md"), "docs");
        assert_eq!(size_class("docs/a.json"), "other");
    }
}
