//! The lab registry, and the check that refuses a lab it does not list.
//!
//! **Why this exists.** Every recording command attributes its record to `--lab` or `$UOR_LAB`, and
//! until this module nothing checked the name. `UOR_LAB=claud` wrote a heartbeat, a receipt and a
//! claim owned by a lab that does not exist, and no reader could tell it from a real one. The only
//! roster was the set of labs that had already published a presence file, which is a list of who
//! *has* written, not of who *may*.
//!
//! **Why a file and not a constant.** `onboard::KNOWN_LABS` was a roster in code, and it drifted
//! from the roster in the rules (four names against five) the day a fifth lab onboarded. The
//! registry is `docs/harness/labs.json`, which only the owner changes (`owner-paths.json`), so
//! adding a lab is an owner decision recorded in one place rather than a code change.
//!
//! **Why a missing registry refuses.** A check that cannot run is a refusal, not a pass. If
//! `labs.json` is absent or malformed, every name would otherwise be accepted, which is exactly the
//! silent failure this module exists to remove.
//!
//! Names are compared after `state::slug`, because the harness slugs a lab into every path it
//! writes: `DeepSeek` and `deepseek` land in one shard, so refusing on case would be nagging.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// One registered lab.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Lab {
    pub vendor: String,
    pub client: String,
    pub headless: String,
    pub open_pr_limit: u32,
    /// `pm` for the project manager; absent for every other lab.
    #[serde(default)]
    pub role: Option<String>,
}

fn default_schema() -> u32 {
    1
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LabsFile {
    #[serde(default = "default_schema")]
    schema: u32,
    #[serde(default, rename = "note")]
    _note: String,
    labs: BTreeMap<String, Lab>,
}

fn skip_json_space(bytes: &[u8], mut i: usize) -> usize {
    while bytes
        .get(i)
        .is_some_and(|b| matches!(*b, b' ' | b'\n' | b'\r' | b'\t'))
    {
        i += 1;
    }
    i
}

fn json_string_end(bytes: &[u8], start: usize) -> Result<usize, String> {
    if bytes.get(start) != Some(&b'"') {
        return Err(format!("expected a JSON string at byte {start}"));
    }
    let mut i = start + 1;
    while let Some(byte) = bytes.get(i) {
        match byte {
            b'\\' => i += 2,
            b'"' => return Ok(i + 1),
            _ => i += 1,
        }
    }
    Err(format!("unterminated JSON string at byte {start}"))
}

fn json_container_end(bytes: &[u8], start: usize) -> Result<usize, String> {
    let mut stack = Vec::new();
    match bytes.get(start) {
        Some(b'{') => stack.push(b'}'),
        Some(b'[') => stack.push(b']'),
        _ => return Err(format!("expected a JSON container at byte {start}")),
    }
    let mut i = start + 1;
    while let Some(byte) = bytes.get(i) {
        match byte {
            b'"' => i = json_string_end(bytes, i)?,
            b'{' => {
                stack.push(b'}');
                i += 1;
            }
            b'[' => {
                stack.push(b']');
                i += 1;
            }
            b'}' | b']' => {
                if stack.pop() != Some(*byte) {
                    return Err(format!("mismatched JSON container at byte {i}"));
                }
                i += 1;
                if stack.is_empty() {
                    return Ok(i);
                }
            }
            _ => i += 1,
        }
    }
    Err(format!("unterminated JSON container at byte {start}"))
}

fn json_value_end(bytes: &[u8], start: usize) -> Result<usize, String> {
    let Some(&first) = bytes.get(start) else {
        return Err(format!("expected a JSON value at byte {start}"));
    };
    match first {
        b'"' => json_string_end(bytes, start),
        b'{' | b'[' => json_container_end(bytes, start),
        b'-' | b'0'..=b'9' => {
            let mut i = start + 1;
            while bytes
                .get(i)
                .is_some_and(|b| matches!(*b, b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-'))
            {
                i += 1;
            }
            Ok(i)
        }
        b't' if bytes.get(start..start + 4).is_some_and(|v| v == b"true") => Ok(start + 4),
        b'f' if bytes.get(start..start + 5).is_some_and(|v| v == b"false") => Ok(start + 5),
        b'n' if bytes.get(start..start + 4).is_some_and(|v| v == b"null") => Ok(start + 4),
        _ => Err(format!("expected a JSON value at byte {start}")),
    }
}

fn raw_json_array_values(text: &str) -> Result<Vec<&str>, String> {
    let bytes = text.as_bytes();
    let mut i = skip_json_space(bytes, 0);
    if bytes.get(i) != Some(&b'[') {
        return Err("expected one JSON array".into());
    }
    i += 1;
    let mut values = Vec::new();
    loop {
        i = skip_json_space(bytes, i);
        if bytes.get(i) == Some(&b']') {
            i = skip_json_space(bytes, i + 1);
            if i == bytes.len() {
                return Ok(values);
            }
            return Err("characters follow the JSON array".into());
        }
        let value_end = json_value_end(bytes, i)?;
        values.push(&text[i..value_end]);
        i = skip_json_space(bytes, value_end);
        match bytes.get(i) {
            Some(b',') => i += 1,
            Some(b']') => {}
            _ => return Err(format!("expected `,` or `]` after array item {i}")),
        }
    }
}

fn validate_json_value(value: &str) -> Result<(), String> {
    match value.as_bytes().first() {
        Some(b'{') => {
            raw_json_object_members(value)?;
        }
        Some(b'[') => {
            for item in raw_json_array_values(value)? {
                validate_json_value(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// The immediate members of a JSON object, preserving each value's exact source text.
/// Duplicate keys are refused after JSON-string decoding, so `x` and `\u0078` collide.
pub(crate) fn raw_json_object_members(text: &str) -> Result<BTreeMap<String, &str>, String> {
    let bytes = text.as_bytes();
    let mut i = skip_json_space(bytes, 0);
    if bytes.get(i) != Some(&b'{') {
        return Err("expected one JSON object".into());
    }
    i += 1;
    let mut members = BTreeMap::new();
    loop {
        i = skip_json_space(bytes, i);
        if bytes.get(i) == Some(&b'}') {
            i = skip_json_space(bytes, i + 1);
            if i == bytes.len() {
                return Ok(members);
            }
            return Err("characters follow the JSON object".into());
        }
        let key_start = i;
        let key_end = json_string_end(bytes, key_start)?;
        let key: String = serde_json::from_str(&text[key_start..key_end])
            .map_err(|e| format!("cannot decode a JSON object key: {e}"))?;
        i = skip_json_space(bytes, key_end);
        if bytes.get(i) != Some(&b':') {
            return Err(format!("expected `:` after JSON key {key:?}"));
        }
        let value_start = skip_json_space(bytes, i + 1);
        let value_end = json_value_end(bytes, value_start)?;
        let value = &text[value_start..value_end];
        validate_json_value(value)?;
        if members.insert(key.clone(), value).is_some() {
            return Err(format!("JSON object repeats key {key:?}"));
        }
        i = skip_json_space(bytes, value_end);
        match bytes.get(i) {
            Some(b',') => i += 1,
            Some(b'}') => {}
            _ => return Err(format!("expected `,` or `}}` after JSON key {key:?}")),
        }
    }
}

fn registry_in(root: &Path) -> PathBuf {
    root.join("docs").join("harness").join("labs.json")
}

/// `docs/harness/labs.json` for the repository containing `repo`.
///
/// `repo` is the working directory unless `--repo` was given, so it may be a subdirectory. Inside a
/// git work tree the registry is **only** the one at that tree's top level (`state::toplevel`, the
/// resolution the state paths use). The walk up the ancestors is for a checkout that is not a git
/// work tree, and it never runs inside one: a worktree under `.worktrees/` branched before the
/// registry existed would otherwise climb out of itself and borrow the primary checkout's file, so a
/// missing registry would pass instead of refusing (round-2 review of #2898). With neither, the path
/// under `repo` is returned so the error names where the file was expected.
pub fn labs_path(repo: &Path) -> PathBuf {
    if let Some(top) = crate::state::toplevel(repo) {
        return registry_in(&top);
    }
    repo.ancestors()
        .map(registry_in)
        .find(|p| p.is_file())
        .unwrap_or_else(|| registry_in(repo))
}

/// Every registered lab, keyed and sorted by name.
///
/// An unreadable or malformed file is an error naming the file, never an empty registry: an empty
/// registry would refuse everyone, and a lenient reader would accept everyone. Both hide the cause.
pub fn load_labs(repo: &Path) -> Result<BTreeMap<String, Lab>, String> {
    let path = labs_path(repo);
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read the lab registry {}: {e}", path.display()))?;
    parse_labs(&text).map_err(|e| format!("the lab registry {} is malformed: {e}", path.display()))
}

fn validate_labs(doc: LabsFile) -> Result<BTreeMap<String, Lab>, String> {
    if doc.schema != 1 {
        return Err(format!("schema {} is not supported", doc.schema));
    }
    if doc.labs.is_empty() {
        return Err("it lists no labs".to_string());
    }
    for lab in doc.labs.keys() {
        let slug = crate::state::slug(lab);
        if lab != &slug {
            return Err(format!("entry {lab:?} is not its canonical slug {slug:?}"));
        }
    }
    Ok(doc.labs)
}

/// The labs in registry `text` (the same checks as [`load_labs`], for text read from elsewhere,
/// such as `git show origin/main:docs/harness/labs.json`).
pub(crate) fn parse_labs(text: &str) -> Result<BTreeMap<String, Lab>, String> {
    raw_json_object_members(text).map_err(|e| format!("the lab registry is malformed: {e}"))?;
    let doc: LabsFile =
        serde_json::from_str(text).map_err(|e| format!("the lab registry is malformed: {e}"))?;
    validate_labs(doc).map_err(|e| format!("the lab registry is malformed: {e}"))
}

/// Accept `name` only if the registry lists it.
///
/// The refusal names the unknown lab and lists every registered one, so the fix is on screen.
pub fn check_lab(repo: &Path, name: &str) -> Result<(), String> {
    let labs = load_labs(repo).map_err(|e| format!("cannot check lab {name:?}: {e}"))?;
    let slug = crate::state::slug(name);
    if labs.contains_key(&slug) {
        return Ok(());
    }
    Err(format!(
        "lab {name:?} is not in docs/harness/labs.json; the registered labs are {}. \
         A new lab is an owner decision recorded in that file.",
        labs.keys().cloned().collect::<Vec<_>>().join(", ")
    ))
}

/// The one resolver every recording command goes through: `--lab` first, then `$UOR_LAB`.
///
/// No lab at all resolves to an empty string, so each command keeps its own "no lab identity"
/// handling (some warn and record unattributed, some refuse). A lab that IS given must be
/// registered; that is the case nothing checked before.
pub fn resolve_lab(
    repo: &Path,
    flag: Option<String>,
    env: Option<String>,
) -> Result<String, String> {
    let lab = flag.or(env).unwrap_or_default();
    if lab.trim().is_empty() {
        return Ok(lab);
    }
    check_lab(repo, &lab)?;
    Ok(lab)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIVE: [&str; 5] = ["antigravity", "claude", "codex", "deepseek", "kimi"];

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn with_registry(text: Option<&str>) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        if let Some(text) = text {
            let harness = dir.path().join("docs").join("harness");
            std::fs::create_dir_all(&harness).expect("mkdir");
            std::fs::write(harness.join("labs.json"), text).expect("write");
        }
        dir
    }

    /// The planted fault the loop ORDER requires: an unregistered lab is refused by the check and
    /// by the resolver every recording command uses, and the refusal lists the five.
    #[test]
    fn an_unregistered_lab_is_refused() {
        let err = check_lab(&repo(), "gemini").expect_err("gemini is not registered");
        assert!(err.contains("\"gemini\""), "{err}");
        assert!(
            err.contains("antigravity, claude, codex, deepseek, kimi"),
            "{err}"
        );
        let err = resolve_lab(&repo(), None, Some("gemini".into()))
            .expect_err("the resolver refuses it too");
        assert!(err.contains("\"gemini\""), "{err}");
        assert!(resolve_lab(&repo(), Some("gemini".into()), Some("claude".into())).is_err());
    }

    #[test]
    fn duplicate_decoded_registry_keys_are_refused() {
        let text = r#"{"schema":1,"labs":{
            "codex":{"vendor":"OpenAI","client":"Codex","headless":"codex exec","open_pr_limit":3},
            "\u0063odex":{"vendor":"Mallory","client":"Impostor","headless":"x","open_pr_limit":99}
        }}"#;
        let err = parse_labs(text).expect_err("the duplicate Codex key is refused");
        assert!(err.contains("repeats key"), "{err}");
    }

    #[test]
    fn each_registered_lab_is_accepted() {
        for lab in FIVE {
            check_lab(&repo(), lab).unwrap_or_else(|e| panic!("{lab}: {e}"));
            assert_eq!(
                resolve_lab(&repo(), Some(lab.into()), None).as_deref(),
                Ok(lab)
            );
        }
        // Slugged, as every path the harness writes is.
        assert!(check_lab(&repo(), "DeepSeek").is_ok());
    }

    /// No lab given is not this module's call: each command keeps its own warning or refusal.
    #[test]
    fn no_lab_resolves_empty_rather_than_refusing() {
        assert_eq!(resolve_lab(&repo(), None, None).as_deref(), Ok(""));
        assert_eq!(
            resolve_lab(&repo(), None, Some("  ".into())).as_deref(),
            Ok("  ")
        );
    }

    #[test]
    fn the_real_registry_lists_exactly_the_five() {
        let labs = load_labs(&repo()).expect("labs.json loads");
        assert_eq!(labs.keys().map(String::as_str).collect::<Vec<_>>(), FIVE);
        assert_eq!(labs["kimi"].role.as_deref(), Some("pm"));
        assert!(labs.values().filter(|l| l.role.is_some()).count() == 1);
    }

    /// A check that cannot run is a refusal, not a pass.
    #[test]
    fn a_missing_or_malformed_registry_refuses() {
        let missing = with_registry(None);
        assert!(load_labs(missing.path()).is_err());
        let err = check_lab(missing.path(), "claude").expect_err("missing registry refuses");
        assert!(err.contains("cannot read the lab registry"), "{err}");

        let malformed = with_registry(Some("{not json"));
        let err = check_lab(malformed.path(), "claude").expect_err("malformed refuses");
        assert!(err.contains("malformed"), "{err}");

        let empty = with_registry(Some(r#"{"schema": 1, "labs": {}}"#));
        assert!(check_lab(empty.path(), "claude").is_err());

        let unknown_top = with_registry(Some(
            r#"{"schema":1,"unexpected":true,"labs":{"claude":{"vendor":"Anthropic","client":"Claude Code","headless":"claude -p","open_pr_limit":3}}}"#,
        ));
        let err = load_labs(unknown_top.path()).expect_err("unknown top-level fields are refused");
        assert!(err.contains("unknown field"), "{err}");
    }

    #[test]
    fn registry_keys_must_be_their_own_slug() {
        for alias in ["Codex", "codex_", "codex-"] {
            let text = serde_json::json!({
                "schema": 1,
                "labs": {
                    "codex": {
                        "vendor": "OpenAI", "client": "Codex", "headless": "codex exec",
                        "open_pr_limit": 3
                    },
                    (alias): {
                        "vendor": "Other", "client": "Other", "headless": "other",
                        "open_pr_limit": 3
                    }
                }
            })
            .to_string();
            let err = parse_labs(&text).expect_err("an aliased registry key is refused");
            assert!(err.contains("canonical slug"), "{alias}: {err}");
        }
    }

    /// Every owner path that is not a glob exists, so the list cannot name a file that moved.
    #[test]
    fn every_owner_path_and_rule_source_exists() {
        let root = repo();
        let read = |name: &str| -> serde_json::Value {
            let p = root.join("docs/harness").join(name);
            serde_json::from_str(&std::fs::read_to_string(&p).expect("reads")).expect("parses")
        };
        let owner = read("owner-paths.json");
        let paths = owner["paths"].as_array().expect("paths list");
        assert!(!paths.is_empty());
        for p in paths {
            let path = p["path"].as_str().expect("path");
            assert!(
                !p["why"].as_str().unwrap_or("").is_empty(),
                "{path}: no why"
            );
            if path.contains('*') {
                let base = path.split('*').next().unwrap_or("");
                assert!(root.join(base).exists(), "{path}: glob base missing");
            } else {
                assert!(root.join(path).exists(), "{path} does not exist");
            }
        }
        for name in ["labs.json", "owner-paths.json", "rule-sources.json"] {
            let listed = format!("docs/harness/{name}");
            assert!(
                paths.iter().any(|p| p["path"] == listed.as_str()),
                "{listed} must be owner-only"
            );
        }
        let sources = read("rule-sources.json");
        let files = sources["files"].as_array().expect("files list");
        assert!(!files.is_empty());
        for f in files {
            let f = f.as_str().expect("a path");
            assert!(root.join(f).exists(), "rule source {f} does not exist");
        }
    }
}
