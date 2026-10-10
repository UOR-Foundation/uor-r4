//! Structural controls for known-literal prospective role/bank composition panels.
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn invalid(s: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, s)
}
fn literal(w: &Value) -> Result<String> {
    let text = w["text"]
        .as_str()
        .ok_or_else(|| invalid("diversity text absent"))?;
    let template = w["template"]
        .as_str()
        .ok_or_else(|| invalid("diversity template absent"))?;
    let (pre, post) = template
        .split_once("{v}")
        .ok_or_else(|| invalid("diversity slot absent"))?;
    if post.contains("{v}") {
        return Err(invalid("diversity multiple slots").into());
    }
    Ok(text
        .strip_prefix(pre)
        .and_then(|s| s.strip_suffix(post))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| invalid("diversity literal mismatch"))?
        .to_owned())
}
/// Checks counts, whole-bank family blocks and lexical role/position balance.
/// No model scores, geometric addresses or answer labels enter this control.
pub(super) fn validate(plan: &Value) -> Result<Value> {
    let split = plan["split"]
        .as_str()
        .ok_or_else(|| invalid("diversity split absent"))?;
    let expected = match split {
        "development" => 256,
        "fresh" => 64,
        _ => return Err(invalid("diversity split differs").into()),
    };
    let histories = plan["histories"]
        .as_array()
        .ok_or_else(|| invalid("diversity histories absent"))?;
    if histories.len() != expected {
        return Err(invalid("diversity fixed count differs").into());
    }
    let mut banks =
        BTreeMap::<(String, String), BTreeMap<(String, String), Vec<(String, Vec<String>)>>>::new();
    let mut role_counts = BTreeMap::<(String, String), usize>::new();
    let mut position_counts = BTreeMap::<(String, usize), usize>::new();
    let mut stratum_counts = BTreeMap::<String, usize>::new();
    let mut query_counts = BTreeMap::<(String, String), usize>::new();
    for h in histories {
        let turns = h["turns"]
            .as_array()
            .ok_or_else(|| invalid("diversity turns absent"))?;
        let queries = h["queries"]
            .as_array()
            .ok_or_else(|| invalid("diversity queries absent"))?;
        let stratum = h["stratum"]
            .as_str()
            .and_then(|s| s.split("/q").next())
            .ok_or_else(|| invalid("diversity stratum absent"))?;
        *stratum_counts.entry(stratum.to_owned()).or_default() += 1;
        if turns.len() < 2 || turns.len() > 8 {
            return Err(invalid("diversity turn bound differs").into());
        }
        if queries.len() != 2 {
            return Err(invalid("diversity requires both queries").into());
        }
        let mut current = BTreeMap::new();
        let mut chronology = Vec::new();
        for (position, w) in turns.iter().enumerate() {
            let role = w["relation"]
                .as_str()
                .filter(|r| matches!(*r, "job" | "home"))
                .ok_or_else(|| invalid("diversity role differs"))?;
            let value = literal(w)?;
            current.insert(role, value.clone());
            *role_counts
                .entry((value.clone(), role.to_owned()))
                .or_default() += 1;
            if position < 2 {
                *position_counts
                    .entry((value.clone(), position))
                    .or_default() += 1;
            }
            chronology.push(format!(
                "{role}/{}/{value}",
                w["act"]
                    .as_str()
                    .ok_or_else(|| invalid("diversity action absent"))?
            ));
        }
        let j = current
            .get("job")
            .ok_or_else(|| invalid("diversity job absent"))?
            .clone();
        let home = current
            .get("home")
            .ok_or_else(|| invalid("diversity home absent"))?
            .clone();
        if j == home {
            return Err(invalid("diversity equal role values").into());
        }
        let unordered = if j < home {
            (j.clone(), home.clone())
        } else {
            (home.clone(), j.clone())
        };
        let mut wording = BTreeMap::new();
        for q in queries {
            let role = q["relation"]
                .as_str()
                .ok_or_else(|| invalid("diversity query role absent"))?;
            let text = q["text"]
                .as_str()
                .ok_or_else(|| invalid("diversity query text absent"))?;
            if wording.insert(role, text).is_some() {
                return Err(invalid("diversity duplicate query role").into());
            }
            *query_counts
                .entry((role.to_owned(), text.to_owned()))
                .or_default() += 1;
        }
        if wording.keys().copied().collect::<BTreeSet<_>>() != BTreeSet::from(["job", "home"]) {
            return Err(invalid("diversity query roles differ").into());
        }
        let wordkey = serde_json::to_string(&wording)?;
        banks
            .entry(unordered)
            .or_default()
            .entry((j, home))
            .or_default()
            .push((wordkey, chronology));
    }
    let multiplier = if split == "development" { 4 } else { 1 };
    if stratum_counts
        != BTreeMap::from([
            ("length2".to_owned(), 16 * multiplier),
            ("length4".to_owned(), 16 * multiplier),
            ("length8".to_owned(), 16 * multiplier),
            ("update".to_owned(), 8 * multiplier),
            ("reassert".to_owned(), 8 * multiplier),
        ])
    {
        return Err(invalid("diversity declared stratum quotas differ").into());
    }
    for variants in banks.values() {
        if variants.len() != 2 || variants.values().any(|v| v.len() != 4) {
            return Err(invalid(
                "diversity requires both role assignments and four history variants each",
            )
            .into());
        }
        let mut role_wordsets = Vec::new();
        for histories in variants.values() {
            let mut groups = BTreeMap::<&str, Vec<&Vec<String>>>::new();
            for (q, t) in histories {
                groups.entry(q).or_default().push(t);
            }
            if groups.len() != 2 || groups.values().any(|v| v.len() != 2 || v[0][0] == v[1][0]) {
                return Err(invalid("diversity bank lacks crossed wording/order").into());
            }
            role_wordsets.push(groups.keys().copied().collect::<BTreeSet<_>>());
        }
        if role_wordsets[0] != role_wordsets[1] {
            return Err(invalid("diversity wording correlated with role assignment").into());
        }
    }
    for ((value, role), count) in &role_counts {
        let other = if role == "job" { "home" } else { "job" };
        if role_counts.get(&(value.clone(), other.to_owned())) != Some(count) {
            return Err(invalid("diversity literal role imbalance").into());
        }
    }
    for ((value, position), count) in &position_counts {
        if position_counts.get(&(value.clone(), 1 - position)) != Some(count) {
            return Err(invalid("diversity literal position imbalance").into());
        }
    }
    for role in ["job", "home"] {
        let counts = query_counts
            .iter()
            .filter(|((r, _), _)| r == role)
            .map(|(_, n)| *n)
            .collect::<Vec<_>>();
        if counts.len() != 6
            || counts.iter().max().copied().unwrap_or(0) - counts.iter().min().copied().unwrap_or(0)
                > 4
        {
            return Err(invalid("diversity six familiar query forms unbalanced").into());
        }
    }
    Ok(
        json!({"shared_literal_vertex_caps":{"length2":10,"length4":10,"length8":10,"update":10,"reassert":8},"candidate_order":"deterministic round-robin circle schedule over sorted capped shared exact donor literals; public exclusions only", "histories":histories.len(),"rows":histories.len()*2,"unordered_literal_bank_blocks":banks.len(),"ordered_role_value_banks":banks.len()*2,"wording_pairs_per_bank":2,"role_assignments_per_block":2,"chronology_orders_per_wording":2,"query_counts":query_counts.iter().map(|((role,text),count)|json!({"role":role,"text":text,"histories":count})).collect::<Vec<_>>(),"literal_role_balance":true,"literal_initial_position_balance":true,"scope":"known retained full literals; heldout bank combinations with familiar wording; no unseen-word/literal/paraphrase claim; no model or geometry selection"}),
    )
}

/// Ordered role/value identity ignores query wording and chronological order.
pub(super) fn bank_keys(plan: &Value) -> Result<BTreeSet<(String, String)>> {
    let mut keys = BTreeSet::new();
    for h in plan["histories"]
        .as_array()
        .ok_or_else(|| invalid("bank histories absent"))?
    {
        let mut current = BTreeMap::new();
        for w in h["turns"]
            .as_array()
            .ok_or_else(|| invalid("bank turns absent"))?
        {
            let role = w["relation"]
                .as_str()
                .ok_or_else(|| invalid("bank role absent"))?;
            current.insert(role, literal(w)?);
        }
        keys.insert((
            current
                .get("job")
                .ok_or_else(|| invalid("bank job absent"))?
                .clone(),
            current
                .get("home")
                .ok_or_else(|| invalid("bank home absent"))?
                .clone(),
        ));
    }
    Ok(keys)
}

/// Assertion context plus both current source records for every query.
pub(super) fn validate_all_sources(packet: &Value) -> Result<()> {
    let segments = packet["segments"]
        .as_array()
        .ok_or_else(|| invalid("diversity runtime segments absent"))?;
    if segments.len() != 4
        || segments[0]["kind"] != "Context"
        || segments[1]["kind"] != "Source"
        || segments[2]["kind"] != "Context"
        || segments[3]["kind"] != "Source"
    {
        return Err(invalid("diversity all-bank packet lost context/source candidate").into());
    }
    let relations = segments
        .iter()
        .filter(|s| s["kind"] == "Source")
        .filter_map(|s| s["relation"].as_u64())
        .collect::<BTreeSet<_>>();
    if relations != BTreeSet::from([1, 2]) {
        return Err(invalid("diversity all-bank packet filtered query role").into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allbank_retains_distractor_for_both_query_roles() -> Result<()> {
        let mut p = json!({"segments":[{"kind":"Context"},{"kind":"Source","relation":1},{"kind":"Context"},{"kind":"Source","relation":2}],"query_ids":[10]});
        validate_all_sources(&p)?;
        p["query_ids"] = json!([20]);
        validate_all_sources(&p)?;
        p["segments"]
            .as_array_mut()
            .ok_or_else(|| invalid("fixture segments absent"))?
            .truncate(2);
        assert!(validate_all_sources(&p).is_err());
        Ok(())
    }
}

/// Circle schedule enumerates every unordered edge once, spreading vertex degrees
/// within each round instead of creating a lexicographic first-literal hub.
pub(super) fn round_robin_edges(count: usize) -> Vec<(usize, usize)> {
    if count < 2 {
        return Vec::new();
    }
    let size = count + count % 2;
    let mut vertices = (0..size).collect::<Vec<_>>();
    let mut edges = Vec::new();
    for _ in 0..size - 1 {
        for i in 0..size / 2 {
            let (a, b) = (vertices[i], vertices[size - 1 - i]);
            if a < count && b < count {
                edges.push((a.min(b), a.max(b)));
            }
        }
        vertices[1..].rotate_right(1);
    }
    edges
}
/// Prospective evaluation holds out combinations, not entire full literals.
/// Reject an insufficient schedule explicitly; never redraw for model scores.
pub(super) fn validate_literal_coverage(development: &Value, fresh: &Value) -> Result<()> {
    fn values(plan: &Value) -> Result<BTreeSet<String>> {
        let mut values = BTreeSet::new();
        for h in plan["histories"]
            .as_array()
            .ok_or_else(|| invalid("coverage histories absent"))?
        {
            for w in h["turns"]
                .as_array()
                .ok_or_else(|| invalid("coverage turns absent"))?
            {
                values.insert(literal(w)?);
            }
        }
        Ok(values)
    }
    let training = values(development)?;
    let evaluation = values(fresh)?;
    if !evaluation.is_subset(&training) {
        return Err(invalid("diversity insufficient training full-literal coverage; fresh would confound unseen literals with new combinations; no redraw").into());
    }
    Ok(())
}
#[cfg(test)]
mod schedule_tests {
    use super::*;
    #[test]
    fn round_robin_spreads_first_round_and_exhausts_unique_edges() {
        for n in [8usize, 10] {
            let edges = round_robin_edges(n);
            assert_eq!(edges.len(), n * (n - 1) / 2);
            assert_eq!(
                edges.iter().copied().collect::<BTreeSet<_>>().len(),
                edges.len()
            );
            let mut degree = vec![0; n];
            for (a, b) in &edges[..n / 2] {
                degree[*a] += 1;
                degree[*b] += 1;
            }
            assert!(degree.iter().all(|d| *d == 1));
            for (a, b) in &edges[n / 2..n] {
                degree[*a] += 1;
                degree[*b] += 1;
            }
            assert!(degree.iter().all(|d| *d == 2));
        }
        assert_eq!(round_robin_edges(5).len(), 10);
    }
    #[test]
    fn fresh_full_literal_must_have_training_witness() -> Result<()> {
        let training = json!({"histories":[{"turns":[{"text":"Old amber willow.","template":"Old {v}."},{"text":"Old copper cedar.","template":"Old {v}."}]}]});
        let fresh = json!({"histories":[{"turns":[{"text":"New copper cedar.","template":"New {v}."},{"text":"New amber willow.","template":"New {v}."}]}]});
        validate_literal_coverage(&training, &fresh)?;
        let mut missing = fresh.clone();
        missing["histories"][0]["turns"][0]["text"] = json!("New previously absent.");
        assert!(validate_literal_coverage(&training, &missing).is_err());
        Ok(())
    }
}

/// Only the twelve retained, already-exposed frozen-input bundles are admitted.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct ExpandedDonorRoot {
    pub root: std::path::PathBuf,
    pub manifest_sha256: String,
}
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExpandedDonorConfig {
    pub roots: Vec<ExpandedDonorRoot>,
}
const EXPANDED_MANIFESTS: [&str; 12] = [
    "14e96299b2d256d0e1729eefb95507d8f1f1052b7e3144ce67d3b0facd709140",
    "8a8b7618f67117086e24cc482c82ce407eb46ef56427420bf2b54d376631679c",
    "1bbacf4c8e327b48a002f98a8efcca6f9eaea84fb34058a6b180d20f63b02500",
    "ccaace26245ff07c148200ce6c37fa1193e56bc477a44ee173bfc97c932cc446",
    "92e38610348cdbc79522e98c0a6f019219365c48ce302fb52525f24939d4c16d",
    "709927f0600f9e200f4088ba9d7733198765a1abadc261be560728a01dd6d1e3",
    "76c5596a337870a2d09667572a8dcde705f916d7b7e1a7bd04d44c4cb7baf45f",
    "e4d4e824c16e671089868dd6572b1ccef6f501f9f7beb6f3d29b5fc8bb6bed1e",
    "88334d971cb35f525605a4b76c90455780abde5bb99592f2b9db60eb856ae73a",
    "df659bef365bdd135e75e36fb0f5669f25892c085a7789404ac4bd39c7d0b603",
    "39380230b40965ff7b89fc544a8b0847b318a0432c7497091cd021253ab6acb0",
    "2b46d23ad7e0103d9701520577a33712e9f6d88c6d2263b18f9a18d508505a64",
];

pub(super) struct ExpandedDonors {
    pub wires: Vec<Value>,
    pub receipt: Value,
    pub bank_keys: BTreeSet<(String, String)>,
    pub history_fingerprints: BTreeSet<String>,
}
impl ExpandedDonorConfig {
    pub fn validate(&self, exposed: &[(std::path::PathBuf, String)]) -> Result<()> {
        let hashes = self
            .roots
            .iter()
            .map(|r| r.manifest_sha256.as_str())
            .collect::<BTreeSet<_>>();
        if self.roots.len() != 12
            || hashes != EXPANDED_MANIFESTS.into_iter().collect()
            || self
                .roots
                .iter()
                .map(|r| &r.root)
                .collect::<BTreeSet<_>>()
                .len()
                != 12
            || self.roots.iter().any(|r| {
                !exposed
                    .iter()
                    .any(|(p, h)| p == &r.root && h == &r.manifest_sha256)
            })
        {
            return Err(invalid("expanded donors require exactly twelve pinned frozen bundles also bound in the exposure census").into());
        }
        Ok(())
    }
}
fn concrete_wire(value: &Value) -> Option<Value> {
    if !matches!(value["act"].as_str(), Some("assert" | "update"))
        || !matches!(value["relation"].as_str(), Some("job" | "home"))
        || literal(value).ok().is_none_or(|s| s.trim().is_empty())
    {
        return None;
    }
    Some(
        json!({"text":value["text"],"relation":value["relation"],"act":value["act"],"template":value["template"]}),
    )
}
fn collect_concrete(
    value: &Value,
    pointer: &str,
    source: &ExpandedDonorRoot,
    donors: &mut BTreeMap<String, Value>,
) -> Result<()> {
    if let Some(wire) = concrete_wire(value) {
        let key = serde_json::to_string(&wire)?;
        let source_sha = uor_r4_training::sha256_bytes(&serde_json::to_vec(value)?);
        donors.entry(key.clone()).or_insert_with(|| {
            json!({
            "root":source.root,"manifest_sha256":source.manifest_sha256,
            "source_file":"frozen-inputs.json","json_pointer":pointer,
            "source_wire_sha256":source_sha,
            "wire_sha256":uor_r4_training::sha256_bytes(key.as_bytes()),"wire":wire})
        });
    }
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if key != "reference_templates_control_only" {
                    let escaped = key.replace('~', "~0").replace('/', "~1");
                    collect_concrete(child, &format!("{pointer}/{escaped}"), source, donors)?;
                }
            }
        }
        Value::Array(items) => {
            for (i, child) in items.iter().enumerate() {
                collect_concrete(child, &format!("{pointer}/{i}"), source, donors)?;
            }
        }
        _ => {}
    }
    Ok(())
}
/// Record each observed query boundary and final state, not only the last write.
fn collect_episode_exposures(
    value: &Value,
    banks: &mut BTreeSet<(String, String)>,
    histories: &mut BTreeSet<String>,
) -> Result<()> {
    if let Some(turns) = value.get("turns") {
        let turns = turns
            .as_array()
            .ok_or_else(|| invalid("frozen episode turns invalid"))?;
        let mut current = BTreeMap::<String, String>::new();
        let mut writes = Vec::<Value>::new();
        let record = |current: &BTreeMap<String, String>,
                      writes: &[Value],
                      banks: &mut BTreeSet<(String, String)>,
                      histories: &mut BTreeSet<String>|
         -> Result<()> {
            if let (Some(job), Some(home)) = (current.get("job"), current.get("home")) {
                banks.insert((job.clone(), home.clone()));
            }
            if !writes.is_empty() {
                histories.insert(uor_r4_training::sha256_bytes(&serde_json::to_vec(writes)?));
            }
            Ok(())
        };
        for w in turns {
            if w["act"] == "query" {
                record(&current, &writes, banks, histories)?;
            } else {
                let wire = concrete_wire(w)
                    .ok_or_else(|| invalid("invalid concrete frozen episode write"))?;
                let role = wire["relation"]
                    .as_str()
                    .ok_or_else(|| invalid("episode role absent"))?;
                let v = literal(&wire)?;
                current.insert(role.to_owned(), v.clone());
                writes.push(json!({"role":role,"act":wire["act"],"literal":v}));
            }
        }
        record(&current, &writes, banks, histories)?;
    }
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if key != "reference_templates_control_only" && key != "turns" {
                    collect_episode_exposures(child, banks, histories)?;
                }
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_episode_exposures(child, banks, histories)?;
            }
        }
        _ => {}
    }
    Ok(())
}
pub(super) fn load_expanded_donors(
    config: &ExpandedDonorConfig,
    exposed: &[(std::path::PathBuf, String)],
) -> Result<ExpandedDonors> {
    config.validate(exposed)?;
    let mut roots = config.roots.clone();
    roots.sort_by(|a, b| a.manifest_sha256.cmp(&b.manifest_sha256));
    let mut donors = BTreeMap::new();
    let mut banks = BTreeSet::new();
    let mut histories = BTreeSet::new();
    for source in &roots {
        uor_r4_core::report_output::verify(&source.root)?;
        if uor_r4_training::sha256_file(&source.root.join("manifest.json"))?
            != source.manifest_sha256
        {
            return Err(invalid("expanded donor manifest pin differs").into());
        }
        let frozen: Value =
            serde_json::from_slice(&std::fs::read(source.root.join("frozen-inputs.json"))?)?;
        if frozen["training"].as_array().map(Vec::len) != Some(512)
            || frozen["development"].as_array().map(Vec::len) != Some(128)
        {
            return Err(invalid("expanded frozen-input bundle cardinality differs").into());
        }
        collect_concrete(&frozen, "", source, &mut donors)?;
        collect_episode_exposures(&frozen, &mut banks, &mut histories)?;
    }
    let witnesses = donors.into_values().collect::<Vec<_>>();
    let wires = witnesses
        .iter()
        .map(|r| r["wire"].clone())
        .collect::<Vec<_>>();
    let receipt = json!({"schema":"uor-r4.expanded-concrete-exposed-donors/1",
        "policy":"exact12-pinned-frozen-bundles/concrete-assert-update-only/no-registry-or-query/first-witness-by-manifest-and-structural-traversal/1",
        "roots":roots,"unique_wires":wires.len(),"witnesses":witnesses,
        "frozen_episode_banks":banks,"frozen_episode_history_fingerprints":histories});
    Ok(ExpandedDonors {
        wires,
        receipt,
        bank_keys: banks,
        history_fingerprints: histories,
    })
}

#[cfg(test)]
mod expanded_tests {
    use super::*;
    fn wire(role: &str, act: &str, value: &str) -> Value {
        json!({"relation":role,"act":act,"template":"Remember {v}.","text":format!("Remember {value}."),"gold_span":[9,9+value.len()]})
    }
    #[test]
    fn expanded_concrete_witnesses_bind_paths_and_skip_registry_query_invalid() -> Result<()> {
        let real = wire("job", "assert", "azure orchard");
        let query = json!({"relation":"job","act":"query","template":"What is my job?","text":"What is my job?"});
        let fixture = json!({"a/b":[real.clone(),real.clone(),query,
            {"relation":"job","act":"assert","template":"Remember {v}.","text":"mismatch"}],
            "reference_templates_control_only":{"fake":wire("home","assert","unwitnessed value")}});
        let source = ExpandedDonorRoot {
            root: "fixture".into(),
            manifest_sha256: EXPANDED_MANIFESTS[0].into(),
        };
        let mut donors = BTreeMap::new();
        collect_concrete(&fixture, "", &source, &mut donors)?;
        assert_eq!(donors.len(), 1);
        let receipt = donors
            .values()
            .next()
            .ok_or_else(|| invalid("test witness absent"))?;
        assert_eq!(receipt["json_pointer"], "/a~1b/0");
        assert_eq!(receipt["manifest_sha256"], source.manifest_sha256);
        let pointer = receipt["json_pointer"]
            .as_str()
            .ok_or_else(|| invalid("test pointer"))?;
        assert_eq!(fixture.pointer(pointer), Some(&real));
        assert_eq!(
            receipt["source_wire_sha256"],
            uor_r4_training::sha256_bytes(&serde_json::to_vec(&real)?)
        );
        assert_eq!(
            receipt["wire_sha256"],
            uor_r4_training::sha256_bytes(&serde_json::to_vec(&receipt["wire"])?)
        );
        assert!(concrete_wire(
            &json!({"act":"assert","relation":"other","template":"{v}","text":"x"})
        )
        .is_none());
        Ok(())
    }
    #[test]
    fn expanded_donor_pin_set_and_exposure_membership_fail_closed() -> Result<()> {
        let mut config = ExpandedDonorConfig {
            roots: EXPANDED_MANIFESTS
                .iter()
                .enumerate()
                .map(|(i, h)| ExpandedDonorRoot {
                    root: format!("bundle-{i}").into(),
                    manifest_sha256: (*h).into(),
                })
                .collect(),
        };
        let exposed = config
            .roots
            .iter()
            .map(|r| (r.root.clone(), r.manifest_sha256.clone()))
            .collect::<Vec<_>>();
        config.validate(&exposed)?;
        assert!(config.validate(&exposed[1..]).is_err());
        config.roots[0].manifest_sha256 = "0".repeat(64);
        assert!(config.validate(&exposed).is_err());
        config.roots[0] = config.roots[1].clone();
        assert!(config.validate(&exposed).is_err());
        config.roots.pop();
        assert!(config.validate(&exposed).is_err());
        Ok(())
    }
    #[test]
    fn expanded_episode_exclusions_cover_intermediate_queries_and_final_bank() -> Result<()> {
        let fixture = json!({"episodes":[{"turns":[wire("job","assert","old job"),wire("home","assert","old home"),
            {"act":"query","relation":"job"},wire("job","update","new job"),
            {"act":"query","relation":"home"},wire("home","update","new home")]}],
            "training":[wire("job","assert","donor only")],
            "reference_templates_control_only":{"turns":[wire("job","assert","fake job"),wire("home","assert","fake home")]}});
        let mut banks = BTreeSet::new();
        let mut histories = BTreeSet::new();
        collect_episode_exposures(&fixture, &mut banks, &mut histories)?;
        assert_eq!(
            banks,
            BTreeSet::from([
                ("old job".into(), "old home".into()),
                ("new job".into(), "old home".into()),
                ("new job".into(), "new home".into())
            ])
        );
        assert_eq!(histories.len(), 3);
        assert!(collect_episode_exposures(
            &json!({"turns":[{"act":"assert"}]}),
            &mut banks,
            &mut histories
        )
        .is_err());
        Ok(())
    }
}
