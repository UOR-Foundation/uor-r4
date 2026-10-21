//! `uor-harness loop render`: the loop's views, rendered from one commit (loop PR 3).
//!
//! **Why this exists.** The loop page says STATE, HANDOFF and LESSONS are rendered, never typed,
//! so they cannot drift from the files they summarise. Every input is read at a named commit with
//! `git show <commit>:<path>` and `git ls-tree`, never from the working tree, so a view is a pure
//! function of that commit. The renderer reads no clock and makes no network call.
//!
//! **The stamp.** The first line of every view is [`stamp`]: an HTML comment naming the 40-hex
//! input commit. `--check` reads it back, re-renders at that commit and needs a byte-exact match,
//! so a hand edit fails (exit 1) and a missing or garbled stamp is refused (exit 2).
//!
//! **Views built.** `STATE.md`, `HANDOFF.md` and `docs/harness/LESSONS.md`, rendered and checked.
//! The AGENTS.md top-ten block is rendered only between its markers
//! (`<!-- loop:lessons-top-ten:start -->` / `<!-- loop:lessons-top-ten:end -->`), which loop PR 8
//! adds; without them nothing is written to AGENTS.md and a note is printed, and `--check` compares
//! only the bytes between the markers, whose first line is the block's stamp.
//! `docs/roadmap/README.md` comes from today's renderer, `cargo run -q --locked -p xtask --
//! roadmap render [--check]`; it is skipped with a note under `--out`, because xtask writes into
//! the repository, and an unavailable cargo or xtask is REFUSED (exit 2), never a silent pass.
//! `docs/loop/baseline.json` is deferred to loop PR 5 (ORDER claude-14, call 5).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const STAMP_HEAD: &str = "<!-- rendered by uor-harness loop render from ";
const STAMP_TAIL: &str = "; do not edit -->";
const USAGE: &str = "usage: uor-harness loop render [--check] [--out <dir>] [--at <commit>]";

/// The first line of every view: names the commit its inputs were read at.
pub fn stamp(commit: &str) -> String {
    format!("{STAMP_HEAD}{commit}{STAMP_TAIL}")
}

/// The commit named by a stamp line, or `None` if the line is not exactly a stamp.
pub fn parse_stamp(line: &str) -> Option<&str> {
    let sha = line.strip_prefix(STAMP_HEAD)?.strip_suffix(STAMP_TAIL)?;
    let hex = sha.len() == 40
        && sha
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    hex.then_some(sha)
}

/// Reads repository files at one commit. Never touches the working tree.
pub struct AtCommit<'a> {
    root: &'a Path,
    pub sha: String,
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
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

impl<'a> AtCommit<'a> {
    /// Resolves `at` (a commit-ish) to its full 40-hex commit.
    pub fn resolve(root: &'a Path, at: &str) -> Result<Self, String> {
        let spec = format!("{at}^{{commit}}");
        let out = git(root, &["rev-parse", "--verify", "--quiet", &spec])
            .map_err(|_| format!("{at:?} does not name a commit"))?;
        let sha = String::from_utf8_lossy(&out).trim().to_string();
        Ok(AtCommit { root, sha })
    }

    /// The file's text at the commit.
    pub fn read(&self, path: &str) -> Result<String, String> {
        let out = git(self.root, &["show", &format!("{}:{path}", self.sha)])
            .map_err(|_| format!("{path} is missing at {}", self.sha))?;
        String::from_utf8(out).map_err(|_| format!("{path} is not UTF-8 at {}", self.sha))
    }

    /// Every file under `dir` at the commit, sorted.
    pub fn list(&self, dir: &str) -> Result<Vec<String>, String> {
        let out = git(
            self.root,
            &["ls-tree", "-r", "--name-only", &self.sha, "--", dir],
        )?;
        let mut v: Vec<String> = String::from_utf8_lossy(&out)
            .lines()
            .map(str::to_string)
            .collect();
        v.sort();
        Ok(v)
    }
}

/// Every view at the commit, as (repository-relative path, bytes), in a fixed order.
pub fn render_all(c: &AtCommit) -> Result<Vec<(String, String)>, String> {
    Ok(vec![
        ("STATE.md".to_string(), render_state(c)?),
        ("HANDOFF.md".to_string(), render_handoff(c)?),
        ("docs/harness/LESSONS.md".to_string(), render_lessons(c)?),
    ])
}

/// The frontmatter block between the leading `---` lines, if any.
fn front_of(text: &str) -> Option<&str> {
    text.strip_prefix("---\n")
        .and_then(|r| r.split("\n---").next())
}

/// A frontmatter scalar: the trimmed text after `key:` on its own line.
fn front_get<'t>(front: &'t str, key: &str) -> Option<&'t str> {
    front.lines().find_map(|l| {
        l.strip_prefix(key)
            .and_then(|r| r.strip_prefix(':'))
            .map(str::trim)
    })
}

/// The first paragraph of a `## <name>` section, joined onto one line.
fn section_first_paragraph(text: &str, name: &str) -> Option<String> {
    let head = format!("## {name}");
    let mut lines = text.lines().skip_while(|l| l.trim_end() != head).skip(1);
    let para: Vec<&str> = lines
        .by_ref()
        .skip_while(|l| l.trim().is_empty())
        .take_while(|l| !l.trim().is_empty() && !l.starts_with("## "))
        .map(str::trim)
        .collect();
    (!para.is_empty()).then(|| para.join(" "))
}

/// The first sentence of a paragraph: up to and including the first `. `, or all of it.
fn first_sentence(p: &str) -> &str {
    p.find(". ").map_or(p, |i| &p[..=i])
}

struct OpenItem {
    id: String,
    title: String,
    status: String,
    open_to: String,
    depends_on: String,
    done_when: String,
    cancel_if: String,
    size: String,
}

/// Where a status sorts in HANDOFF: ready, decision, held, then any other open status by name.
fn handoff_rank(status: &str) -> usize {
    ["ready", "decision", "held"]
        .iter()
        .position(|s| *s == status)
        .unwrap_or(3)
}

/// `HANDOFF.md`: open items by status, the blocked table and the not-doing list.
pub fn render_handoff(c: &AtCommit) -> Result<String, String> {
    const NR: &str = "not recorded";
    let mut items: Vec<OpenItem> = Vec::new();
    for path in c.list("docs/roadmap/items")? {
        if !path.ends_with(".md") || path.ends_with("/README.md") {
            continue;
        }
        let text = c.read(&path)?;
        let front = front_of(&text).ok_or_else(|| format!("{path}: no frontmatter"))?;
        let get = |k: &str| front_get(front, k).unwrap_or("").to_string();
        let status = get("status");
        if status.is_empty() {
            return Err(format!("{path}: frontmatter has no status"));
        }
        if status == "done" {
            continue;
        }
        let stem = path.rsplit('/').next().unwrap_or(&path);
        let id = Some(get("id"))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| stem.trim_end_matches(".md").to_string());
        let deps = get("depends_on");
        let deps = deps.trim_start_matches('[').trim_end_matches(']').trim();
        let size = get("size");
        items.push(OpenItem {
            id,
            title: get("title"),
            open_to: get("open_to"),
            depends_on: deps.to_string(),
            done_when: section_first_paragraph(&text, "Done when")
                .map(|p| first_sentence(&p).to_string())
                .unwrap_or_else(|| NR.into()),
            cancel_if: section_first_paragraph(&text, "Cancel if")
                .map(|p| first_sentence(&p).to_string())
                .unwrap_or_else(|| NR.into()),
            size: if size.is_empty() { NR.into() } else { size },
            status,
        });
    }
    items.sort_by(|a, b| {
        (handoff_rank(&a.status), &a.status, &a.id).cmp(&(
            handoff_rank(&b.status),
            &b.status,
            &b.id,
        ))
    });

    let mut o = String::new();
    o.push_str(&stamp(&c.sha));
    o.push_str("\n# HANDOFF\n\nRendered from `docs/roadmap/items/*.md` at the commit in the stamp above. Each open-item row gives the first sentence of the item's `Done when` and `Cancel if` sections; blocked items are listed once, in the blocked table, with their owner; `not recorded` means the item has no such section or key yet.\n");

    let blocked = |i: &OpenItem| i.status == "held" || i.status == "decision";
    let open: Vec<&OpenItem> = items.iter().filter(|i| !blocked(i)).collect();
    o.push_str("\n## Open items\n\nItems not `held`, `decision` or `done`. Held and decision items are listed once, in the blocked table below.\n");
    let mut group: Option<&str> = None;
    for it in &open {
        if group != Some(it.status.as_str()) {
            group = Some(it.status.as_str());
            let n = open.iter().filter(|i| i.status == it.status).count();
            o.push_str(&format!(
                "\n### {} ({n})\n\n| Item | Title | Done when | Cancel if | Size | Stands |\n|---|---|---|---|---|---|\n",
                it.status
            ));
        }
        let stands = if it.depends_on.is_empty() {
            it.status.clone()
        } else {
            format!("{}; after {}", it.status, it.depends_on)
        };
        o.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            cell(&it.id),
            cell(&it.title),
            cell(&it.done_when),
            cell(&it.cancel_if),
            cell(&it.size),
            cell(&stands)
        ));
    }
    if open.is_empty() {
        o.push_str("\nnone\n");
    }

    o.push_str("\n## Blocked\n\nItems with status `held` or `decision`, each listed once. The owner is `owner` for a decision or for `open_to: owner`, otherwise the item's `open_to`.\n\n");
    o.push_str("| Item | Title | Status | Owner |\n|---|---|---|---|\n");
    for it in items.iter().filter(|i| blocked(i)) {
        let owner = if it.status == "decision" || it.open_to == "owner" {
            "owner"
        } else if it.open_to.is_empty() {
            NR
        } else {
            &it.open_to
        };
        o.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            cell(&it.id),
            cell(&it.title),
            it.status,
            cell(owner)
        ));
    }

    o.push_str("\n## Not doing\n\nnone recorded\n");
    Ok(o)
}

/// The lesson id from an `id: L-NNN` line, as its number.
fn lesson_id(front: &str) -> Option<(u32, String)> {
    front.lines().find_map(|l| {
        let v = l.strip_prefix("id:")?.trim();
        let n = v.strip_prefix("L-")?;
        if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        Some((n.parse().ok()?, v.to_string()))
    })
}

/// Lessons with an id as (number, id, title, file name), sorted by id; the rest as (title, name).
type Lessons = (Vec<(u32, String, String, String)>, Vec<(String, String)>);

fn lessons(c: &AtCommit) -> Result<Lessons, String> {
    let mut with_id: Vec<(u32, String, String, String)> = Vec::new();
    let mut without: Vec<(String, String)> = Vec::new();
    for path in c.list("docs/harness/mistakes")? {
        let Some(name) = path.strip_prefix("docs/harness/mistakes/") else {
            continue;
        };
        if !name.ends_with(".md") || name.contains('/') || name == "README.md" {
            continue;
        }
        let text = c.read(&path)?;
        let front = front_of(&text).unwrap_or("");
        let title = text
            .lines()
            .find_map(|l| l.strip_prefix("# "))
            .or_else(|| front_get(front, "title"))
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .unwrap_or(name.trim_end_matches(".md"))
            .to_string();
        match lesson_id(front) {
            Some((n, id)) => with_id.push((n, id, title, name.to_string())),
            None => without.push((title, name.to_string())),
        }
    }
    with_id.sort_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
    Ok((with_id, without))
}

/// `docs/harness/LESSONS.md`: the mistakes registry, by lesson id where one is recorded.
pub fn render_lessons(c: &AtCommit) -> Result<String, String> {
    let (with_id, without) = lessons(c)?;
    let link = |name: &str| format!("[`{name}`](mistakes/{name})");
    let mut o = String::new();
    o.push_str(&stamp(&c.sha));
    o.push_str("\n# LESSONS\n\nRendered from `docs/harness/mistakes/*.md` at the commit in the stamp above. A lesson is listed under its `id: L-NNN` line; ids are never invented.\n");
    o.push_str("\n## By lesson id\n\n");
    for (_, id, title, name) in &with_id {
        o.push_str(&format!("- **{id}**: {title} ({})\n", link(name)));
    }
    if with_id.is_empty() {
        o.push_str("none yet\n");
    }
    o.push_str("\n## Without a lesson id yet (assigned in loop PR 8)\n\n");
    for (title, name) in &without {
        o.push_str(&format!("- {title} ({})\n", link(name)));
    }
    if without.is_empty() {
        o.push_str("none\n");
    }
    Ok(o)
}

pub const TOP_TEN_START: &str = "<!-- loop:lessons-top-ten:start -->";
pub const TOP_TEN_END: &str = "<!-- loop:lessons-top-ten:end -->";
const NO_MARKERS: &str = "AGENTS.md: no markers; top-ten block not rendered";

/// The byte ranges of the text strictly between the AGENTS.md markers, if both are present in order.
fn between_markers(text: &str) -> Option<(usize, usize)> {
    let s = text.find(TOP_TEN_START)? + TOP_TEN_START.len();
    let e = s + text[s..].find(TOP_TEN_END)?;
    Some((s, e))
}

/// The AGENTS.md top-ten block: the bytes that go strictly between the markers. Its stamp is the
/// block's first line. No PM-owned `top_ten:` list exists yet, so it is the first ten lessons by id.
pub fn render_top_ten(c: &AtCommit) -> Result<String, String> {
    let (with_id, _) = lessons(c)?;
    let mut o = format!("\n{}\n", stamp(&c.sha));
    for (_, id, title, name) in with_id.iter().take(10) {
        o.push_str(&format!(
            "- **{id}**: {title} ([`{name}`](docs/harness/mistakes/{name}))\n"
        ));
    }
    if with_id.is_empty() {
        o.push_str("no lessons with an id yet\n");
    }
    Ok(o)
}

/// AGENTS.md at the commit with the block re-rendered, or `None` when the markers are absent.
fn render_agents(c: &AtCommit) -> Result<Option<String>, String> {
    let Ok(text) = c.read("AGENTS.md") else {
        return Ok(None);
    };
    let Some((s, e)) = between_markers(&text) else {
        return Ok(None);
    };
    Ok(Some(format!(
        "{}{}{}",
        &text[..s],
        render_top_ten(c)?,
        &text[e..]
    )))
}

/// `--check` of the AGENTS block on disk: only the bytes between the markers, only when they exist.
fn check_agents(root: &Path, file: &Path) -> u8 {
    let Ok(text) = fs::read_to_string(file) else {
        println!("{NO_MARKERS}");
        return 0;
    };
    let Some((s, e)) = between_markers(&text) else {
        println!("{NO_MARKERS}");
        return 0;
    };
    let block = &text[s..e];
    let Some(sha) = block
        .strip_prefix('\n')
        .and_then(|b| b.lines().next())
        .and_then(parse_stamp)
    else {
        eprintln!("uor-harness: loop render --check: the AGENTS.md top-ten block has no valid stamp on its first line");
        return 2;
    };
    let fresh = match AtCommit::resolve(root, sha).and_then(|c| render_top_ten(&c)) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("uor-harness: loop render --check: AGENTS.md: {e}");
            return 2;
        }
    };
    if fresh == block {
        println!("AGENTS.md top-ten block: matches its render at {sha}");
        0
    } else {
        println!("AGENTS.md top-ten block: differs from its render at {sha}");
        1
    }
}

const ROADMAP: &str = "docs/roadmap/README.md";

/// The roadmap view through today's renderer: `cargo run -q --locked -p xtask -- roadmap render
/// [--check]`, cargo resolved as the gates resolve it. 0 written or matching, 1 differs, 2 refused.
fn roadmap(root: &Path, check: bool) -> u8 {
    let mut cmd = Command::new(crate::loopgates::resolve_program("cargo"));
    cmd.args([
        "run", "-q", "--locked", "-p", "xtask", "--", "roadmap", "render",
    ]);
    if check {
        cmd.arg("--check");
    }
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    let out = match cmd.current_dir(root).output() {
        Ok(o) => o,
        Err(e) => {
            println!("{ROADMAP}: REFUSED: cannot run cargo: {e}");
            return 2;
        }
    };
    match out.status.code() {
        Some(0) => {
            println!(
                "{ROADMAP}: {} through xtask",
                if check {
                    "matches its render"
                } else {
                    "rendered"
                }
            );
            0
        }
        // cargo's own failure code: no workspace, no xtask package, a build error or a panic.
        Some(101) | None => {
            println!(
                "{ROADMAP}: REFUSED: cargo or xtask unavailable: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
            2
        }
        Some(n) => {
            println!(
                "{ROADMAP}: differs from its xtask render (exit {n}): {}",
                String::from_utf8_lossy(&out.stdout).trim()
            );
            1
        }
    }
}

fn value_text(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => "none".to_string(),
        other => other.to_string(),
    }
}

/// Escapes a value for a Markdown table cell.
fn cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

fn json(c: &AtCommit, path: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(&c.read(path)?).map_err(|e| format!("{path}: {e}"))
}

/// The cells of a Markdown table row, honouring `\|` escapes.
fn row_cells(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    let mut cells = vec![String::new()];
    let mut chars = inner.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' && chars.peek() == Some(&'|') {
            cells.last_mut().unwrap().push_str("\\|");
            chars.next();
        } else if ch == '|' {
            cells.push(String::new());
        } else {
            cells.last_mut().unwrap().push(ch);
        }
    }
    cells.iter().map(|c| c.trim().to_string()).collect()
}

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b.iter().enumerate().all(|(i, ch)| {
            if i == 4 || i == 7 {
                *ch == b'-'
            } else {
                ch.is_ascii_digit()
            }
        })
}

/// `STATE.md`: bars and holds, item counts, newest notebook rows, mechanisms, labs.
pub fn render_state(c: &AtCommit) -> Result<String, String> {
    let mut o = String::new();
    o.push_str(&stamp(&c.sha));
    o.push_str("\n# STATE\n\nRendered from committed inputs at the commit in the stamp above.\n");

    // 1. Milestone bars and holds.
    o.push_str("\n## Milestone bars and holds\n\nSource: `docs/harness/milestones.json`.\n\n");
    o.push_str("| Milestone | Status | Headline bar | Training hold |\n|---|---|---|---|\n");
    let ms = json(c, "docs/harness/milestones.json")?;
    let ms = ms
        .get("milestones")
        .and_then(|m| m.as_object())
        .ok_or("docs/harness/milestones.json: no \"milestones\" object")?;
    let mut keys: Vec<&String> = ms.keys().collect();
    keys.sort();
    for k in keys {
        let m = &ms[k];
        let field = |f: &str| m.get(f).map(value_text).unwrap_or_else(|| "none".into());
        let bar = match m.get("headline") {
            Some(h) if h.is_object() => {
                let h = |f: &str| h.get(f).map(value_text).unwrap_or_else(|| "none".into());
                format!(
                    "{}: {} of target {} (out of {} {})",
                    h("label"),
                    h("current"),
                    h("target"),
                    h("of"),
                    h("unit")
                )
            }
            _ => "none".into(),
        };
        let hold = match m.get("training_hold") {
            Some(serde_json::Value::Bool(true)) => "HELD",
            Some(serde_json::Value::Bool(false)) => "released",
            _ => "not recorded",
        };
        o.push_str(&format!(
            "| {} {} | {} | {} | {} |\n",
            cell(&field("id")),
            cell(&field("title")),
            cell(&field("status")),
            cell(&bar),
            hold
        ));
    }

    // 2. Roadmap item counts by status.
    o.push_str(
        "\n## Roadmap items by status\n\nSource: `docs/roadmap/items/*.md` frontmatter.\n\n",
    );
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut total = 0;
    for path in c.list("docs/roadmap/items")? {
        if !path.ends_with(".md") || path.ends_with("/README.md") {
            continue;
        }
        let text = c.read(&path)?;
        let Some(front) = text
            .strip_prefix("---\n")
            .and_then(|r| r.split("\n---").next())
        else {
            return Err(format!("{path}: no frontmatter"));
        };
        let status = front
            .lines()
            .find_map(|l| l.strip_prefix("status:"))
            .map(|s| s.trim().to_string())
            .ok_or_else(|| format!("{path}: frontmatter has no status"))?;
        *counts.entry(status).or_default() += 1;
        total += 1;
    }
    o.push_str("| Status | Items |\n|---|---|\n");
    for (s, n) in &counts {
        o.push_str(&format!("| {} | {n} |\n", cell(s)));
    }
    o.push_str(&format!("| **total** | {total} |\n"));

    // 3. Newest notebook row per research line.
    o.push_str("\n## Newest notebook row per research line\n\nSource: `docs/research/notebook/*.md`; the row with the latest date, the last such row on a tie.\n\n");
    o.push_str("| Line | Date | Number | Bar | Verdict |\n|---|---|---|---|---|\n");
    for path in c.list("docs/research/notebook")? {
        if !path.ends_with(".md") || path.ends_with("/README.md") {
            continue;
        }
        let line_name = path
            .rsplit('/')
            .next()
            .unwrap_or(&path)
            .trim_end_matches(".md");
        let text = c.read(&path)?;
        let mut header: Option<Vec<String>> = None;
        let mut newest: Option<Vec<String>> = None;
        for l in text.lines() {
            if !l.trim_start().starts_with('|') {
                continue;
            }
            let cells = row_cells(l);
            if cells.first().map(String::as_str) == Some("Date") {
                header = Some(cells);
                continue;
            }
            let Some(d) = cells.first().filter(|d| is_date(d)) else {
                continue;
            };
            if newest.as_ref().is_none_or(|n| d.as_str() >= n[0].as_str()) {
                newest = Some(cells);
            }
        }
        let pick = |row: &[String], name: &str| -> String {
            header
                .as_ref()
                .and_then(|h| h.iter().position(|c| c == name))
                .and_then(|i| row.get(i).cloned())
                .unwrap_or_default()
        };
        match (&header, newest) {
            (None, _) => {
                return Err(format!(
                    "{path}: no results table (a row starting `| Date |`)"
                ))
            }
            (Some(_), None) => o.push_str(&format!("| {line_name} | no rows yet | | | |\n")),
            (Some(_), Some(r)) => o.push_str(&format!(
                "| {line_name} | {} | {} | {} | {} |\n",
                r[0],
                pick(&r, "Number"),
                pick(&r, "Bar"),
                pick(&r, "Verdict")
            )),
        }
    }

    // 4. Mechanism status lines.
    o.push_str("\n## Mechanisms\n\nSource: the ACTIVE / READY / PARKED tables in `docs/mechanisms/README.md`.\n\n");
    let board = c.read("docs/mechanisms/README.md")?;
    let order = ["ACTIVE", "READY", "PARKED"];
    let mut mech: Vec<(usize, String)> = Vec::new();
    let mut current: Option<usize> = None;
    for l in board.lines() {
        if let Some(h) = l.strip_prefix("## ") {
            current = order.iter().position(|s| h.starts_with(&format!("{s}:")));
            continue;
        }
        if let (Some(s), Some(rest)) = (current, l.strip_prefix("| [")) {
            if let Some(end) = rest.find("](") {
                mech.push((s, rest[..end].to_string()));
            }
        }
    }
    mech.sort();
    for (s, name) in &mech {
        o.push_str(&format!("- {}: {name}\n", order[*s]));
    }
    if mech.is_empty() {
        o.push_str("- no mechanism rows found\n");
    }

    // 5. Lab registry.
    o.push_str("\n## Lab registry\n\nSource: `docs/harness/labs.json`.\n\n");
    o.push_str("| Lab | Vendor | Client | Open PR limit |\n|---|---|---|---|\n");
    let labs = json(c, "docs/harness/labs.json")?;
    let labs = labs
        .get("labs")
        .and_then(|l| l.as_object())
        .ok_or("docs/harness/labs.json: no \"labs\" object")?;
    let mut names: Vec<&String> = labs.keys().collect();
    names.sort();
    for n in names {
        let f = |k: &str| {
            labs[n]
                .get(k)
                .map(value_text)
                .unwrap_or_else(|| "none".into())
        };
        o.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            cell(n),
            cell(&f("vendor")),
            cell(&f("client")),
            cell(&f("open_pr_limit"))
        ));
    }
    Ok(o)
}

/// `--check` over one view file: 0 match, 1 differs, 2 missing or garbled stamp.
fn check_one(root: &Path, file: &Path, view: &str) -> u8 {
    let text = match fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!(
                "uor-harness: loop render --check: cannot read {}: {e}",
                file.display()
            );
            return 2;
        }
    };
    let Some(sha) = text.lines().next().and_then(parse_stamp) else {
        eprintln!(
            "uor-harness: loop render --check: {} has no valid stamp on its first line",
            file.display()
        );
        return 2;
    };
    let c = match AtCommit::resolve(root, sha) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("uor-harness: loop render --check: {}: {e}", file.display());
            return 2;
        }
    };
    let fresh = match render_all(&c) {
        Ok(v) => v.into_iter().find(|(p, _)| p == view).map(|(_, b)| b),
        Err(e) => {
            eprintln!("uor-harness: loop render --check: {e}");
            return 2;
        }
    };
    let fresh = fresh.unwrap_or_default();
    if fresh.as_bytes() == text.as_bytes() {
        println!("{view}: matches its render at {sha}");
        return 0;
    }
    let (mut a, mut b) = (fresh.split_inclusive('\n'), text.split_inclusive('\n'));
    let mut n = 1;
    loop {
        let (x, y) = (a.next(), b.next());
        if x != y {
            println!("{view}: differs from its render at {sha}, first at line {n}");
            println!("  rendered: {:?}", x.unwrap_or("<end of file>"));
            println!("  on disk:  {:?}", y.unwrap_or("<end of file>"));
            return 1;
        }
        n += 1;
    }
}

/// `loop render ...` (arguments after `render`). Returns the process exit code.
pub fn cli(root: &Path, args: &[String]) -> u8 {
    let (mut check, mut out, mut at): (bool, Option<PathBuf>, Option<String>) = (false, None, None);
    let mut i = 0;
    while i < args.len() {
        match (args[i].as_str(), args.get(i + 1)) {
            ("--check", _) => check = true,
            ("--out", Some(v)) => {
                out = Some(PathBuf::from(v));
                i += 1;
            }
            ("--at", Some(v)) => {
                at = Some(v.clone());
                i += 1;
            }
            (other, _) => {
                eprintln!("uor-harness: loop render: bad argument {other:?}\n{USAGE}");
                return 2;
            }
        }
        i += 1;
    }
    let out_given = out.is_some();
    let dir = out.unwrap_or_else(|| root.to_path_buf());
    if check {
        if at.is_some() {
            eprintln!(
                "uor-harness: loop render --check reads the commit from the stamp; drop --at"
            );
            return 2;
        }
        let names = match AtCommit::resolve(root, "HEAD").and_then(|c| render_all(&c)) {
            Ok(v) => v.into_iter().map(|(p, _)| p).collect::<Vec<_>>(),
            Err(e) => {
                eprintln!("uor-harness: loop render --check: {e}");
                return 2;
            }
        };
        let mut code = names
            .iter()
            .map(|p| check_one(root, &dir.join(p), p))
            .max()
            .unwrap_or(0);
        code = code.max(check_agents(root, &dir.join("AGENTS.md")));
        code = code.max(roadmap_or_skip(root, out_given, true));
        return code;
    }
    let c = match AtCommit::resolve(root, at.as_deref().unwrap_or("HEAD")) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("uor-harness: loop render: {e}");
            return 2;
        }
    };
    let mut views = match render_all(&c) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("uor-harness: loop render: {e}");
            return 2;
        }
    };
    match render_agents(&c) {
        Ok(Some(text)) => views.push(("AGENTS.md".to_string(), text)),
        Ok(None) => println!("{NO_MARKERS}"),
        Err(e) => {
            eprintln!("uor-harness: loop render: {e}");
            return 2;
        }
    }
    for (p, bytes) in views {
        let path = dir.join(&p);
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!(
                    "uor-harness: loop render: cannot create {}: {e}",
                    parent.display()
                );
                return 2;
            }
        }
        if let Err(e) = fs::write(&path, bytes) {
            eprintln!(
                "uor-harness: loop render: cannot write {}: {e}",
                path.display()
            );
            return 2;
        }
        println!("wrote {} from {}", path.display(), c.sha);
    }
    roadmap_or_skip(root, out_given, false)
}

/// The roadmap view, or a printed skip under `--out`: xtask writes into the repository itself.
fn roadmap_or_skip(root: &Path, out_given: bool, check: bool) -> u8 {
    if out_given {
        println!("{ROADMAP}: skipped with --out; xtask renders it inside the repository");
        return 0;
    }
    roadmap(root, check)
}
