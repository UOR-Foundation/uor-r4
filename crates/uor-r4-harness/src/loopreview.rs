//! `uor-review v1` review records and the review gate, as pure functions over strings.
//!
//! A review record is a PR comment whose first line is `[<lab>] uor-review v1`, followed by
//! `key: value` lines up to the first blank line, then optional free text (the findings):
//!
//! ```text
//! [<lab>] uor-review v1
//! pr: <number>
//! head: <40 lowercase hex>
//! lab: <lab>
//! model: <model id or "unknown">
//! launch: <launch id or "manual">
//! verdict: APPROVE | CHANGES
//! order: docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md
//! context: fresh            (optional)
//! done-when: host: <what was re-run and its result>  |  done-when: gpu: sha256:<64 hex>
//! ```
//!
//! [`gate`] decides whether a PR's records let it merge. A setup PR (one whose ORDER's
//! `Setup row:` names a row of the loop page's build-plan table, and whose item is
//! `infra-loop-harness` or that row's own item) counts only `claude` records while D59 is in
//! force. D59 ends only by an owner switch: the file [`D59_SWITCH`] reading `"d59": "ended"`,
//! with that exact path listed in `docs/harness/owner-paths.json` at the merge base. The
//! claude-18 ORDER proposes the switch file; the owner adds it and lists it. Until both hold,
//! [`d59_in_force`] answers `true`.
//!
//! Nothing here calls git or GitHub; callers read the inputs and pass them in.

use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The owner's D59 switch file. Proposed by the claude-18 ORDER; added and listed in
/// `docs/harness/owner-paths.json` by the owner.
pub const D59_SWITCH: &str = "docs/harness/d59.json";

/// The text every record's first line carries.
const MARK: &str = "uor-review v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Approve,
    Changes,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Approve => "APPROVE",
            Verdict::Changes => "CHANGES",
        }
    }
}

/// What the reviewer re-ran: a host check with its result, or a GPU evidence digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoneWhen {
    /// The text after `host: `.
    Host(String),
    /// The 64 lowercase hex digits after `gpu: sha256:`.
    Gpu(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub lab: String,
    pub pr: u64,
    pub head: String,
    pub model: String,
    pub launch: String,
    pub verdict: Verdict,
    pub order: String,
    pub fresh_context: bool,
    pub done_when: DoneWhen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parsed {
    /// The first line does not contain `uor-review v1`.
    NotARecord,
    /// It claims to be a record but is wrong; the reason says how.
    Malformed(String),
    Record(Record),
}

/// The record as comment text, which [`parse`] reads back to the same record.
pub fn render(r: &Record) -> String {
    let mut s = format!(
        "[{lab}] {MARK}\npr: {pr}\nhead: {head}\nlab: {lab}\nmodel: {model}\nlaunch: {launch}\nverdict: {verdict}\norder: {order}\n",
        lab = r.lab,
        pr = r.pr,
        head = r.head,
        model = r.model,
        launch = r.launch,
        verdict = r.verdict.as_str(),
        order = r.order,
    );
    if r.fresh_context {
        s.push_str("context: fresh\n");
    }
    match &r.done_when {
        DoneWhen::Host(t) => s.push_str(&format!("done-when: host: {t}\n")),
        DoneWhen::Gpu(h) => s.push_str(&format!("done-when: gpu: sha256:{h}\n")),
    }
    s
}

fn re(cell: &'static OnceLock<regex::Regex>, pattern: &str) -> &'static regex::Regex {
    cell.get_or_init(|| regex::Regex::new(pattern).expect("a valid pattern"))
}

fn is_hex(s: &str, len: usize) -> bool {
    s.len() == len
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

const KEYS: &[&str] = &[
    "pr",
    "head",
    "lab",
    "model",
    "launch",
    "verdict",
    "order",
    "context",
    "done-when",
];

/// Reads a comment body. See [`Parsed`] for the three outcomes.
pub fn parse(body: &str) -> Parsed {
    static TAG: OnceLock<regex::Regex> = OnceLock::new();
    static ORDER: OnceLock<regex::Regex> = OnceLock::new();
    let mut lines = body.lines().map(|l| l.strip_suffix('\r').unwrap_or(l));
    let first = lines.next().unwrap_or("");
    if !first.contains(MARK) {
        return Parsed::NotARecord;
    }
    let bad = |why: String| Parsed::Malformed(why);
    let Some(tag) = re(&TAG, r"^\[([a-z0-9-]+)\] uor-review v1$")
        .captures(first)
        .map(|c| c[1].to_string())
    else {
        return bad(format!(
            "untagged: the first line must be exactly `[<lab>] {MARK}`, got {first:?}"
        ));
    };
    let mut fields: BTreeMap<&str, String> = BTreeMap::new();
    for line in lines {
        if line.trim().is_empty() {
            break;
        }
        let Some((key, value)) = line.split_once(':') else {
            return bad(format!("field line {line:?} is not `key: value`"));
        };
        let key = key.trim();
        let Some(&known) = KEYS.iter().find(|k| **k == key) else {
            return bad(format!("unknown field {key:?}"));
        };
        if fields.insert(known, value.trim().to_string()).is_some() {
            return bad(format!("duplicated field {key:?}"));
        }
    }
    for key in KEYS.iter().filter(|k| **k != "context") {
        if !fields.contains_key(key) {
            return bad(format!("missing field {key:?}"));
        }
    }
    let lab = fields["lab"].clone();
    if lab != tag {
        return bad(format!(
            "the tag [{tag}] differs from the lab: field {lab:?}"
        ));
    }
    let Ok(pr) = fields["pr"].parse::<u64>() else {
        return bad(format!("bad pr {:?}: not a PR number", fields["pr"]));
    };
    let head = fields["head"].clone();
    if !is_hex(&head, 40) {
        return bad(format!("bad head {head:?}: not 40 lowercase hex"));
    }
    let verdict = match fields["verdict"].as_str() {
        "APPROVE" => Verdict::Approve,
        "CHANGES" => Verdict::Changes,
        other => return bad(format!("bad verdict {other:?}: not APPROVE or CHANGES")),
    };
    for key in ["model", "launch"] {
        if fields[key].is_empty() {
            return bad(format!("empty field {key:?}"));
        }
    }
    let order = fields["order"].clone();
    if !re(
        &ORDER,
        r"^docs/loop/tasks/[^/\s]+/[a-z0-9-]+-[0-9]+/ORDER\.md$",
    )
    .is_match(&order)
    {
        return bad(format!(
            "bad order {order:?}: not docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md"
        ));
    }
    let fresh_context = match fields.get("context").map(String::as_str) {
        None => false,
        Some("fresh") => true,
        Some(other) => return bad(format!("bad context {other:?}: only `fresh` is allowed")),
    };
    let dw = fields["done-when"].as_str();
    let done_when = if let Some(t) = dw.strip_prefix("host:").map(str::trim) {
        if t.is_empty() {
            return bad("bad done-when: `host:` names nothing".to_string());
        }
        DoneWhen::Host(t.to_string())
    } else if let Some(h) = dw
        .strip_prefix("gpu:")
        .map(str::trim)
        .and_then(|g| g.strip_prefix("sha256:"))
        .filter(|h| is_hex(h, 64))
    {
        DoneWhen::Gpu(h.to_string())
    } else {
        return bad(format!(
            "bad done-when {dw:?}: not `host: <what ran>` or `gpu: sha256:<64 hex>`"
        ));
    };
    Parsed::Record(Record {
        lab,
        pr,
        head,
        model: fields["model"].clone(),
        launch: fields["launch"].clone(),
        verdict,
        order,
        fresh_context,
        done_when,
    })
}

/// The first column of the loop page's build-plan table: the table under `## Build plan`
/// whose header row starts `| PR | What it builds |`.
pub fn build_rows(loop_page: &str) -> Vec<String> {
    let mut lines = loop_page
        .lines()
        .skip_while(|l| l.trim_end() != "## Build plan")
        .skip(1);
    let mut rows = Vec::new();
    for line in lines.by_ref() {
        if line.starts_with("## ") {
            return rows;
        }
        if line.starts_with("| PR | What it builds |") {
            break;
        }
    }
    for line in lines {
        let Some(rest) = line.strip_prefix('|') else {
            break;
        };
        let cell = rest.split('|').next().unwrap_or("").trim();
        if cell.is_empty() || cell.bytes().all(|b| b == b'-' || b == b':') {
            continue;
        }
        rows.push(cell.to_string());
    }
    rows
}

/// The item-id prefix of a build-plan row: `6` → `infra-loop-pr06-`, `7a` →
/// `infra-loop-pr07a-`. Rows `A` and `1` have no item of their own (PR 1 made the items for
/// PRs 2–12), so they, and anything not digits with an optional lowercase letter, map to `None`.
pub fn row_item_prefix(row: &str) -> Option<String> {
    static ROW: OnceLock<regex::Regex> = OnceLock::new();
    let c = re(&ROW, r"^([0-9]+)([a-z]?)$").captures(row)?;
    let n: u32 = c[1].parse().ok()?;
    (n >= 2).then(|| format!("infra-loop-pr{n:02}{}-", &c[2]))
}

/// The value of an ORDER's `Setup row:` line, up to whitespace or `.`.
pub fn setup_row(order_text: &str) -> Option<String> {
    static SETUP: OnceLock<regex::Regex> = OnceLock::new();
    re(&SETUP, r"Setup row:[ \t]*([^\s.]+)")
        .captures(order_text)
        .map(|c| c[1].to_string())
}

/// A setup PR: the ORDER's `Setup row:` names one of `rows`, and its item is
/// `infra-loop-harness` or starts with that row's item prefix.
pub fn is_setup_pr(setup_row: Option<&str>, rows: &[String], order_item: &str) -> bool {
    let Some(row) = setup_row else {
        return false;
    };
    if !rows.iter().any(|r| r == row) {
        return false;
    }
    order_item == "infra-loop-harness"
        || row_item_prefix(row).is_some_and(|p| order_item.starts_with(&p))
}

/// D59 is in force unless both: `owner_paths_json` lists exactly [`D59_SWITCH`], and
/// `switch_text` is JSON with `"d59": "ended"`. Anything missing or unreadable keeps it in force.
pub fn d59_in_force(owner_paths_json: &str, switch_text: Option<&str>) -> bool {
    let listed = serde_json::from_str::<serde_json::Value>(owner_paths_json)
        .ok()
        .and_then(|v| v.get("paths")?.as_array().cloned())
        .is_some_and(|paths| {
            paths
                .iter()
                .any(|p| p.get("path").and_then(|s| s.as_str()) == Some(D59_SWITCH))
        });
    if !listed {
        return true;
    }
    let ended = switch_text
        .and_then(|t| serde_json::from_str::<serde_json::Value>(t).ok())
        .is_some_and(|v| v.get("d59").and_then(|s| s.as_str()) == Some("ended"));
    !ended
}

/// Everything the gate reads. `vendors` maps each registered lab to its vendor; `comments` are
/// the PR's comment bodies in posting order.
pub struct GateInput<'a> {
    pub pr: u64,
    pub head: &'a str,
    pub order_lab: &'a str,
    pub order_item: &'a str,
    pub setup_row: Option<&'a str>,
    pub rows: &'a [String],
    pub vendors: &'a BTreeMap<String, String>,
    pub d59_in_force: bool,
    pub doc_only: bool,
    pub comments: &'a [String],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateOutcome {
    pub pass: bool,
    pub setup_pr: bool,
    pub details: Vec<String>,
}

/// Why a well-formed record does not count, or `None` when it does.
fn refusal(input: &GateInput, setup_pr: bool, r: &Record) -> Option<String> {
    if r.pr != input.pr {
        return Some(format!("it reviews PR {}, not PR {}", r.pr, input.pr));
    }
    if r.head != input.head {
        return Some(format!("not the current head {}", input.head));
    }
    let Some(lab_vendor) = input.vendors.get(&r.lab) else {
        return Some(format!("lab {} is not a registered lab", r.lab));
    };
    if setup_pr && input.d59_in_force {
        return (r.lab != "claude").then(|| {
            format!(
                "a setup PR under D59 counts only claude records, not {}",
                r.lab
            )
        });
    }
    let order_vendor = input
        .vendors
        .get(input.order_lab)
        .map(String::as_str)
        .unwrap_or("");
    let other_vendor =
        !lab_vendor.is_empty() && !order_vendor.is_empty() && lab_vendor != order_vendor;
    if other_vendor {
        return None;
    }
    if input.doc_only && r.lab == input.order_lab {
        return (!r.fresh_context)
            .then(|| "a same-lab record on a doc-only PR needs `context: fresh`".to_string());
    }
    Some(format!(
        "same vendor: lab {} (vendor {lab_vendor:?}) is not of another vendor than the ORDER lab {} (vendor {order_vendor:?}) (D46)",
        r.lab, input.order_lab
    ))
}

/// Passes when a counted record is an APPROVE and the last counted record is an APPROVE.
/// `details` names every record, malformed ones included, and why it counted or not.
pub fn gate(input: &GateInput) -> GateOutcome {
    let setup_pr = is_setup_pr(input.setup_row, input.rows, input.order_item);
    let mut details = Vec::new();
    let mut last: Option<Verdict> = None;
    for (i, body) in input.comments.iter().enumerate() {
        let n = i + 1;
        match parse(body) {
            Parsed::NotARecord => {}
            Parsed::Malformed(why) => {
                details.push(format!("comment {n}: refused, malformed record: {why}"))
            }
            Parsed::Record(r) => {
                let seen = format!(
                    "comment {n}: [{}] {} at {} on PR {}",
                    r.lab,
                    r.verdict.as_str(),
                    r.head,
                    r.pr
                );
                match refusal(input, setup_pr, &r) {
                    Some(why) => details.push(format!("{seen}: not counted, {why}")),
                    None => {
                        details.push(format!("{seen}: counted"));
                        last = Some(r.verdict);
                    }
                }
            }
        }
    }
    let pass = last == Some(Verdict::Approve);
    details.push(match last {
        Some(Verdict::Approve) => format!("pass: a counted APPROVE at {} stands", input.head),
        Some(Verdict::Changes) => "fail: the last counted record is CHANGES".to_string(),
        None => format!("fail: no counted record at {}", input.head),
    });
    GateOutcome {
        pass,
        setup_pr,
        details,
    }
}
