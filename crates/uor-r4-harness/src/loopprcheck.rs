//! `uor-harness loop pr-check`: the rules a loop PR must pass, read from git alone.
//!
//! The PR is the diff from `git merge-base <base> <head>` to `<head>`. Registries, items and
//! budgets are read at the merge base, never at the head, so a PR cannot loosen its own check;
//! the files the PR adds or changes are read at the head. Nothing reads the working tree, GitHub
//! or the network. A rule is one function plus one entry in [`RULES`].

use std::path::Path;
use std::process::Command;

const USAGE: &str =
    "usage: uor-harness loop pr-check [--base <rev>] [--head <rev>] [--head-ref <branch>] [--json]";

/// Every rule, in the order it runs and prints.
const RULES: &[(&str, Rule)] = &[
    ("order", rule_order),
    ("scope", rule_scope),
    ("commit-order", rule_commit_order),
    ("report", rule_report),
    ("owner-paths", rule_owner_paths),
    ("rule-sources", rule_rule_sources),
    ("rendered-lock", rule_rendered_lock),
    ("read-path", rule_read_path),
    ("filemap", rule_filemap),
];

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
    /// The branch the head is on (`--head-ref`), `""` when unknown; only `rendered-lock` reads it.
    pub head_ref: String,
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

/// `--name-status -z` fields as changes: a rename or copy carries two paths.
fn name_status(fields: &[String]) -> Result<Vec<Change>, String> {
    let mut changes = Vec::new();
    let mut i = 0;
    while i < fields.len() {
        let status = fields[i].chars().next().unwrap_or('?');
        let n = if matches!(status, 'R' | 'C') { 2 } else { 1 };
        let paths = fields
            .get(i + 1..i + 1 + n)
            .ok_or("git --name-status output is truncated")?
            .to_vec();
        changes.push(Change { status, paths });
        i += 1 + n;
    }
    Ok(changes)
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
        let changes = name_status(&z_fields(&raw))?;
        Ok(Pr {
            root,
            merge_base,
            head,
            changes,
            head_ref: String::new(),
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

/// The non-merge commits of the PR, oldest first (`rev-list --reverse --topo-order`).
fn pr_commits(pr: &Pr) -> Result<Vec<String>, String> {
    let range = format!("{}..{}", pr.merge_base, pr.head);
    let out = git(
        pr.root,
        &["rev-list", "--reverse", "--topo-order", "--parents", &range],
    )?;
    Ok(String::from_utf8_lossy(&out)
        .lines()
        .filter_map(
            |l| match l.split_whitespace().collect::<Vec<_>>().as_slice() {
                [c] | [c, _] => Some(c.to_string()),
                _ => None,
            },
        )
        .collect())
}

/// What one non-merge commit changes against its parent.
fn commit_changes(pr: &Pr, sha: &str) -> Result<Vec<Change>, String> {
    let parent = format!("{sha}^");
    let raw = git(
        pr.root,
        &[
            "diff-tree",
            "--no-commit-id",
            "-r",
            "-M",
            "--name-status",
            "-z",
            &parent,
            sha,
        ],
    )?;
    name_status(&z_fields(&raw))
}

/// The first non-merge commit adds the ORDER, and that commit changes nothing outside its folder.
fn rule_commit_order(pr: &Pr) -> RuleResult {
    const R: &str = "commit-order";
    let order = match pr.the_order() {
        Ok(p) => p,
        Err(e) => return fail(R, format!("cannot check the commit order: {e}")),
    };
    let folder = order.trim_end_matches("ORDER.md");
    let commits = match pr_commits(pr) {
        Ok(c) => c,
        Err(e) => return fail(R, format!("cannot list the PR's commits: {e}")),
    };
    let mut details = Vec::new();
    for sha in &commits {
        let short = &sha[..sha.len().min(9)];
        let changes = match commit_changes(pr, sha) {
            Ok(c) => c,
            Err(e) => return fail(R, format!("cannot read commit {short}: {e}")),
        };
        let paths = || {
            changes
                .iter()
                .flat_map(|c| c.paths.iter().map(String::as_str))
        };
        let adds_order = changes
            .iter()
            .any(|c| c.status == 'A' && c.paths.last().map(String::as_str) == Some(order));
        if !adds_order {
            let p: Vec<&str> = paths().collect();
            if !p.is_empty() {
                details.push(format!(
                    "{short} changes {} before the commit that adds {order}",
                    p.join(", ")
                ));
            }
            continue;
        }
        let outside: Vec<&str> = paths().filter(|p| !p.starts_with(folder)).collect();
        if !outside.is_empty() {
            details.push(format!(
                "{short} adds {order} but also changes {}, outside {folder}",
                outside.join(", ")
            ));
        }
        return result(
            R,
            details,
            &format!("{short} adds {order} first, alone in its folder"),
            "the ORDER commit does not come first, alone",
        );
    }
    fail(R, format!("no non-merge commit adds {order}"))
}

/// The `## ` sections a REPORT needs, in this order (each heading starts with one, any case).
const REPORT_SECTIONS: [&str; 7] = [
    "What did not work",
    "What changed",
    "Measurements",
    "Checks run at the head",
    "Skills used",
    "Still open",
    "Plain summary",
];

/// What is wrong with a REPORT's sections; empty when it is valid.
pub fn report_problems(text: &str) -> Vec<String> {
    // (lower-cased heading, whether a non-blank line follows before the next `## `)
    let mut sections: Vec<(String, bool)> = Vec::new();
    for line in text.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            sections.push((h.trim().to_lowercase(), false));
        } else if !line.trim().is_empty() {
            if let Some(last) = sections.last_mut() {
                last.1 = true;
            }
        }
    }
    let mut out = Vec::new();
    let mut prev: Option<(usize, &str)> = None;
    for name in REPORT_SECTIONS {
        let lower = name.to_lowercase();
        let Some(i) = sections.iter().position(|(h, _)| h.starts_with(&lower)) else {
            out.push(format!("no `## {name}` heading"));
            continue;
        };
        if let Some((pi, pname)) = prev {
            if i < pi {
                out.push(format!(
                    "out of order: `## {name}` comes before `## {pname}`"
                ));
            }
        }
        if !sections[i].1 {
            out.push(format!("the `## {name}` section is empty"));
        }
        prev = Some((i, name));
    }
    out
}

/// `<ORDER folder>/REPORT.md` exists at the head with every section, in order, none empty.
fn rule_report(pr: &Pr) -> RuleResult {
    const R: &str = "report";
    let order = match pr.the_order() {
        Ok(p) => p,
        Err(e) => return fail(R, format!("cannot find the REPORT: {e}")),
    };
    let path = format!("{}REPORT.md", order.trim_end_matches("ORDER.md"));
    let Some(text) = pr.at_head(&path) else {
        return fail(R, format!("{path} is missing at the head"));
    };
    result(
        R,
        report_problems(&text),
        &path,
        &format!("{path} is not a valid REPORT"),
    )
}

/// An owner-paths entry covers `path`: `<prefix>/**` covers the prefix and everything below it.
fn owner_match(entry: &str, path: &str) -> bool {
    match entry.strip_suffix("/**") {
        Some(prefix) => path == prefix || path.starts_with(&format!("{prefix}/")),
        None => path == entry,
    }
}

/// What the ORDER's `## Owner approval` section(s) lack for `entries`; empty when approved.
pub fn owner_approval_missing(order_text: &str, entries: &[&str]) -> Vec<String> {
    let mut section = String::new();
    let mut inside = false;
    for line in order_text.lines() {
        if line.starts_with("## ") {
            inside = line.to_lowercase().starts_with("## owner approval");
        }
        if inside {
            section.push_str(line);
            section.push('\n');
        }
    }
    if section.is_empty() {
        return vec!["the ORDER has no `## Owner approval` section".into()];
    }
    let mut out = Vec::new();
    for entry in entries {
        let named = entry.strip_suffix("/**").unwrap_or(entry);
        if !section.contains(named) {
            out.push(format!("the owner approval section does not name {entry}"));
        }
    }
    let quote = regex::Regex::new(r#""[^"]+""#).expect("a valid pattern");
    if !quote.is_match(&section) {
        out.push("the owner approval section holds no double-quoted answer".into());
    }
    let time =
        regex::Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}(:\d{2})?Z").expect("a valid pattern");
    if !time.is_match(&section) {
        out.push(
            "the owner approval section holds no ISO UTC time (YYYY-MM-DDTHH:MM[:SS]Z)".into(),
        );
    }
    out
}

/// A change to a path in `docs/harness/owner-paths.json` (read at the base) carries the owner's
/// approval in the ORDER: the entry named, a quoted answer and an ISO UTC time.
fn rule_owner_paths(pr: &Pr) -> RuleResult {
    const R: &str = "owner-paths";
    const LIST: &str = "docs/harness/owner-paths.json";
    let Some(text) = pr.at_base(LIST) else {
        return fail(R, format!("{LIST} is missing at the merge base"));
    };
    let v: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => return fail(R, format!("{LIST} at the merge base: {e}")),
    };
    let Some(entries) = v.get("paths").and_then(|p| p.as_array()).and_then(|a| {
        a.iter()
            .map(|e| e.get("path").and_then(|p| p.as_str()))
            .collect::<Option<Vec<&str>>>()
    }) else {
        return fail(
            R,
            format!("{LIST} at the merge base has no `paths` list of {{\"path\": ..}} entries"),
        );
    };
    let mut details = Vec::new();
    let mut matched = Vec::new();
    for entry in entries {
        let mut hits: Vec<&str> = pr
            .changed_paths()
            .map(|(_, p)| p)
            .filter(|p| owner_match(entry, p))
            .collect();
        hits.dedup();
        if !hits.is_empty() {
            details.push(format!("{} changed (owner path {entry})", hits.join(", ")));
            matched.push(entry);
        }
    }
    if matched.is_empty() {
        return result(R, Vec::new(), "no owner path changed", "");
    }
    let missing = match pr.the_order() {
        Err(e) => vec![format!("no ORDER to carry the owner's approval: {e}")],
        Ok(order) => match pr.at_head(order) {
            None => vec![format!("{order} cannot be read at the head")],
            Some(t) => owner_approval_missing(&t, &matched),
        },
    };
    if missing.is_empty() {
        return RuleResult {
            rule: R,
            verdict: Verdict::Pass,
            summary: format!("owner approval recorded for {}", matched.join(", ")),
            details: Vec::new(),
        };
    }
    details.extend(missing);
    RuleResult {
        rule: R,
        verdict: Verdict::Fail,
        summary: "owner path(s) changed without the owner's approval in the ORDER".into(),
        details,
    }
}

fn not_applicable(rule: &'static str, summary: &str) -> RuleResult {
    RuleResult {
        rule,
        verdict: Verdict::NotApplicable,
        summary: summary.to_string(),
        details: Vec::new(),
    }
}

/// Outside the files in `docs/harness/rule-sources.json` (read at the base; absent counts as
/// empty), `archive/`, `docs/loop/tasks/` and `docs/roadmap/items/`, the PR adds no `.md` file
/// named for a rule and no markdown heading containing the word `rule` or `rules`.
fn rule_rule_sources(pr: &Pr) -> RuleResult {
    const R: &str = "rule-sources";
    const LIST: &str = "docs/harness/rule-sources.json";
    let listed: Vec<String> = match pr.at_base(LIST) {
        None => Vec::new(),
        Some(text) => {
            let files = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| {
                    v.get("files")?
                        .as_array()?
                        .iter()
                        .map(|f| f.as_str().map(String::from))
                        .collect::<Option<Vec<String>>>()
                });
            match files {
                Some(f) => f,
                None => {
                    return fail(
                        R,
                        format!("{LIST} at the merge base has no `files` list of paths"),
                    )
                }
            }
        }
    };
    let exempt = |p: &str| {
        ["archive/", "docs/loop/tasks/", "docs/roadmap/items/"]
            .iter()
            .any(|d| p.starts_with(d))
            || listed.iter().any(|l| l == p)
    };
    let heading = regex::Regex::new(r"^#{1,6} ").expect("a valid pattern");
    let word = regex::Regex::new(r"(?i)\brules?\b").expect("a valid pattern");
    let mut details = Vec::new();
    for c in &pr.changes {
        let Some(path) = c.paths.last().map(String::as_str) else {
            continue;
        };
        if c.status == 'D' || !path.ends_with(".md") || exempt(path) {
            continue;
        }
        let name = path.rsplit('/').next().unwrap_or(path);
        if matches!(c.status, 'A' | 'R' | 'C') && name.to_lowercase().contains("rule") {
            details.push(format!("{path} adds a rule-named file outside {LIST}"));
        }
        let Some(head) = pr.at_head(path) else {
            continue;
        };
        let base = pr.at_base(&c.paths[0]).unwrap_or_default();
        let old: std::collections::HashSet<&str> = base.lines().collect();
        for line in head.lines() {
            if heading.is_match(line) && word.is_match(line) && !old.contains(line) {
                details.push(format!(
                    "{path} adds the rule heading `{line}` outside {LIST}"
                ));
            }
        }
    }
    result(
        R,
        details,
        &format!("no rule stated outside {LIST}"),
        &format!("a rule is stated outside {LIST}"),
    )
}

const AGENTS_START: &str = "<!-- loop:lessons-top-ten:start -->";
const AGENTS_END: &str = "<!-- loop:lessons-top-ten:end -->";

/// The bytes of AGENTS.md between the lessons markers; `None` when neither marker is there. A
/// lone or misordered marker yields the whole text, so it still counts as a view.
fn agents_lessons(text: &str) -> Option<String> {
    match (text.find(AGENTS_START), text.find(AGENTS_END)) {
        (None, None) => None,
        (Some(s), Some(e)) if e >= s + AGENTS_START.len() => {
            Some(text[s + AGENTS_START.len()..e].to_string())
        }
        _ => Some(format!("unpaired marker\n{text}")),
    }
}

/// The rendered views change only on the `kimi/render` branch: `STATE.md`, `HANDOFF.md`,
/// `docs/harness/LESSONS.md` and AGENTS.md between the lessons-top-ten markers. The roadmap
/// README and `docs/loop/baseline.json` join this lock with loop PR 8.
fn rule_rendered_lock(pr: &Pr) -> RuleResult {
    const R: &str = "rendered-lock";
    let mut views: Vec<(&str, Option<String>, Option<String>)> =
        ["STATE.md", "HANDOFF.md", "docs/harness/LESSONS.md"]
            .iter()
            .map(|p| (*p, pr.at_base(p), pr.at_head(p)))
            .collect();
    views.push((
        "AGENTS.md (lessons-top-ten markers)",
        pr.at_base("AGENTS.md").as_deref().and_then(agents_lessons),
        pr.at_head("AGENTS.md").as_deref().and_then(agents_lessons),
    ));
    if views.iter().all(|(_, b, h)| b.is_none() && h.is_none()) {
        return not_applicable(R, "no rendered view exists yet (loop PR 8 commits them)");
    }
    let changed: Vec<String> = views
        .iter()
        .filter(|(_, b, h)| b != h)
        .map(|(p, _, _)| format!("{p} changed"))
        .collect();
    if matches!(pr.head_ref.as_str(), "kimi/render" | "origin/kimi/render") {
        return result(
            R,
            Vec::new(),
            "head ref kimi/render may change the views",
            "",
        );
    }
    let shown = if pr.head_ref.is_empty() {
        "(none)"
    } else {
        pr.head_ref.as_str()
    };
    result(
        R,
        changed,
        "no rendered view changed",
        &format!("rendered views change only on kimi/render, not head ref {shown}"),
    )
}

/// A read-path budget: name, (base, head) value, over budget at a value, the budget, its unit.
type Budget = (
    &'static str,
    (u64, u64),
    fn(u64) -> bool,
    &'static str,
    &'static str,
);

/// The read-path budgets, measured on the head and compared with the base. An item over budget
/// fails when it grew, and warns when it did not.
fn rule_read_path(pr: &Pr) -> RuleResult {
    const R: &str = "read-path";
    let bytes = |t: Option<String>| t.map_or(0, |t| t.len() as u64);
    let lines = |t: Option<String>| t.map_or(0, |t| t.lines().count() as u64);
    let order = pr.the_order().ok();
    let pair = |p: &str| (bytes(pr.at_base(p)), bytes(pr.at_head(p)));
    let agents_lines = (
        lines(pr.at_base("AGENTS.md")),
        lines(pr.at_head("AGENTS.md")),
    );
    let (agents, charter, state, handoff) = (
        pair("AGENTS.md"),
        pair("CHARTER.md"),
        pair("STATE.md"),
        pair("HANDOFF.md"),
    );
    let ord = order.map_or((0, 0), pair);
    let total = (
        agents.0 + charter.0 + state.0 + handoff.0 + ord.0,
        agents.1 + charter.1 + state.1 + handoff.1 + ord.1,
    );
    // (name, (base, head), over budget at this value, the budget as words, unit)
    let items: [Budget; 5] = [
        ("AGENTS.md", agents_lines, |v| v > 80, "at most 80", "lines"),
        ("STATE.md", state, |v| v >= 24_576, "under 24576", "B"),
        ("HANDOFF.md", handoff, |v| v >= 24_576, "under 24576", "B"),
        ("the ORDER", ord, |v| v >= 4_096, "under 4096", "B"),
        (
            "the read path",
            total,
            |v| v > 128_000,
            "stop above 128000",
            "B",
        ),
    ];
    let (mut fails, mut warns) = (Vec::new(), Vec::new());
    for (name, (b, h), over, budget, unit) in items {
        if over(h) {
            let line = format!("{name} is {h} {unit} (base {b}), over its budget ({budget})");
            if h > b {
                fails.push(format!("{line} and grew"));
            } else {
                warns.push(format!("{line} but did not grow"));
            }
        }
    }
    if total.1 > 76_800 && total.1 <= 128_000 {
        warns.push(format!(
            "the read path is {} B (base {}), above the 76800 B warning line",
            total.1, total.0
        ));
    }
    let summary = format!(
        "AGENTS.md {} lines, STATE.md {} B, HANDOFF.md {} B, ORDER {} B, read path {} B (AGENTS + CHARTER + STATE + HANDOFF + ORDER)",
        agents_lines.1, state.1, handoff.1, ord.1, total.1
    );
    let verdict = if !fails.is_empty() {
        Verdict::Fail
    } else if !warns.is_empty() {
        Verdict::Warn
    } else {
        Verdict::Pass
    };
    fails.extend(warns);
    RuleResult {
        rule: R,
        verdict,
        summary,
        details: fails,
    }
}

/// `docs/start/FILEMAP.md` at the head has a row (a table line whose first cell is a backticked
/// path) for every depth-1 and depth-2 directory, and every row names a directory at the head.
fn rule_filemap(pr: &Pr) -> RuleResult {
    const R: &str = "filemap";
    const MAP: &str = "docs/start/FILEMAP.md";
    let Some(text) = pr.at_head(MAP) else {
        return not_applicable(
            R,
            "docs/start/FILEMAP.md does not exist yet (loop PR 8 creates it)",
        );
    };
    let raw = match git(
        pr.root,
        &["ls-tree", "-d", "-r", "--name-only", "-z", &pr.head],
    ) {
        Ok(r) => r,
        Err(e) => return fail(R, e),
    };
    let dirs = z_fields(&raw);
    let row = regex::Regex::new(r"^\|\s*`([^`]+)`\s*\|").expect("a valid pattern");
    let rows: Vec<String> = text
        .lines()
        .filter_map(|l| row.captures(l))
        .map(|c| c[1].trim().trim_end_matches('/').to_string())
        .collect();
    let mut details = Vec::new();
    for d in &dirs {
        let depth = d.split('/').count();
        if depth > 2 || d.split('/').any(|s| s.starts_with('.')) || d.starts_with("archive/") {
            continue;
        }
        if !rows.contains(d) {
            details.push(format!("missing row: {d}"));
        }
    }
    for r in &rows {
        if !dirs.contains(r) {
            details.push(format!("stale row: {r} is not a directory at the head"));
        }
    }
    result(
        R,
        details,
        &format!("{MAP} has a row for every depth-1 and depth-2 directory"),
        &format!("{MAP} has missing or stale rows"),
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

/// The branch `head` names (`refs/heads/<x>` or `refs/remotes/<x>`), else the current branch,
/// else `""`.
fn default_head_ref(root: &Path, head: &str) -> String {
    for prefix in ["refs/heads/", "refs/remotes/"] {
        let r = format!("{prefix}{head}");
        if git(root, &["rev-parse", "--verify", "--quiet", &r]).is_ok() {
            return head.to_string();
        }
    }
    git(root, &["symbolic-ref", "--short", "-q", "HEAD"])
        .map(|b| String::from_utf8_lossy(&b).trim().to_string())
        .unwrap_or_default()
}

/// `loop pr-check [--base <rev>] [--head <rev>] [--head-ref <branch>] [--json]`. Exit 0 every rule passes, 1 a rule
/// failed, 2 the check could not run.
pub fn cli(root: &Path, args: &[String]) -> u8 {
    let (mut base, mut head, mut json) = ("origin/main".to_string(), "HEAD".to_string(), false);
    let mut head_ref: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            flag @ ("--base" | "--head" | "--head-ref") => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("{USAGE}");
                    return 2;
                };
                match flag {
                    "--base" => base = v.clone(),
                    "--head" => head = v.clone(),
                    _ => head_ref = Some(v.clone()),
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
    let mut pr = match Pr::open(root, &base, &head) {
        Ok(pr) => pr,
        Err(e) => {
            eprintln!("uor-harness: loop pr-check: {e}");
            return 2;
        }
    };
    pr.head_ref = head_ref.unwrap_or_else(|| default_head_ref(root, &head));
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
