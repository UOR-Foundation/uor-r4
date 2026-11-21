use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::Write,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const ARMS: [&str; 5] = ["lexical", "transported", "knockout", "permuted", "fallback"];

fn increment(map: &mut BTreeMap<String, usize>, key: impl Into<String>) {
    *map.entry(key.into()).or_default() += 1;
}
fn miss(expected: &[u32], reply: &[u32]) -> Option<usize> {
    (0..expected.len().max(reply.len())).find(|&i| expected.get(i) != reply.get(i))
}
fn movement(old: Option<usize>, new: Option<usize>) -> &'static str {
    match (old, new) {
        (None, None) => "both_exact",
        (Some(_), None) => "recovered",
        (None, Some(_)) => "regressed",
        (Some(a), Some(b)) if b > a => "later",
        (Some(a), Some(b)) if b < a => "earlier",
        _ => "same_first",
    }
}
fn main() -> Result<()> {
    let paths: Vec<_> = env::args().skip(1).collect();
    let [pred, gold, strata, out] = paths.as_slice() else {
        return Err("PRED REFERENCE STRATA NEW_OUT".into());
    };
    let read = |p: &str| -> Result<Vec<Value>> { Ok(serde_json::from_slice(&fs::read(p)?)?) };
    let (pred, gold, strata) = (read(pred)?, read(gold)?, read(strata)?);
    if (pred.len(), gold.len(), strata.len()) != (160, 160, 160) {
        return Err("panel coverage".into());
    }
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    let mut groups: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    for (i, ((p, g), s)) in pred.iter().zip(gold).zip(strata).enumerate() {
        let id = p["id"].as_str().ok_or("missing ID")?;
        if !seen.insert(id)
            || p["id"] != g["id"]
            || p["id"] != s["id"]
            || p["donor_id"] != pred[i ^ 1]["id"]
        {
            return Err("ID/donor mismatch".into());
        }
        let reference: Vec<u32> = serde_json::from_value(g["reference"].clone())?;
        if reference.last() != Some(&1) || reference[..reference.len() - 1].contains(&1) {
            return Err("invalid gold EOS".into());
        }
        let family = s["family"].as_u64().ok_or("family")?;
        let offset = s["value_offset"].as_u64().ok_or("value offset")? as usize;
        let mut row = json!({"id":id,"stratum":s,"reference":reference});
        let lexical: Vec<u32> = serde_json::from_value(p["lexical"]["reply"].clone())?;
        let donor = &pred[i ^ 1];
        if p["trunk_state_sha256"] != donor["donor_state_sha256"]
            || p["permuted"]["forms"] != donor["transported"]["forms"]
        {
            return Err("counterfactual donor differs".into());
        }
        let mut base = None;
        for arm in ARMS {
            let reply: Vec<u32> = serde_json::from_value(p[arm]["reply"].clone())?;
            let first = miss(&reference, &reply);
            if arm == "lexical" {
                base = first
            }
            let changed = movement(base, first);
            let label = first.map_or("exact".to_owned(), |v| v.to_string());
            let trace = p[arm]["writer_step_words"]
                .as_array()
                .ok_or("missing native step words")?;
            if trace.len() != 23 * reply.len()
                || reply
                    .iter()
                    .enumerate()
                    .any(|(i, &t)| trace[i * 23 + 22].as_u64() != Some(u64::from(t)))
            {
                return Err("writer tuple trajectory mismatch".into());
            }
            let at = first.filter(|j| *j < reply.len());
            row[arm] = json!({"forms":p[arm]["forms"],"first_source_word":at.map(|j| &trace[j*23+21]),"first_cursor_word":at.map(|j| &trace[j*23+14]),"reply":reply, "first":first,"expected":first.and_then(|j|reference.get(j)), "actual":first.and_then(|j|reply.get(j)),"movement":changed});
            for segment in [
                if family == 0 {
                    "known".into()
                } else {
                    "unseen".into()
                },
                format!("family_{family}"),
            ] {
                let counts = groups.entry(format!("{segment}/{arm}")).or_default();
                increment(counts, "rows");
                increment(counts, format!("first_{label}"));
                increment(counts, changed);
                if reply != lexical {
                    increment(counts, "changed_sequence");
                }
                if p[arm]["forms"] != p["lexical"]["forms"] {
                    increment(counts, "changed_forms");
                }
                if p[arm]["abstained"] == true {
                    increment(counts, "abstained");
                }
                if p["trunk_state"] != p["donor_state"] {
                    increment(counts, "role_swap_changed_vector");
                }
                if p["trunk_state"] != p["knockout_state"] {
                    increment(counts, "knockout_changed_vector");
                }
                if first.is_none() {
                    increment(counts, "exact_eos");
                }
                if reply.last() == Some(&1) {
                    increment(counts, "ended_eos");
                }
                if reply.starts_with(&reference[..offset]) {
                    increment(counts, "correct_opening");
                    if reply.get(offset) != reference.get(offset) {
                        increment(counts, "wrong_value_start");
                    }
                }
            }
        }
        rows.push(row);
    }
    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(out)?;
    serde_json::to_writer(&mut f, &json!({"groups":groups,"rows":rows}))?;
    f.write_all(b"\n")?;
    Ok(())
}
