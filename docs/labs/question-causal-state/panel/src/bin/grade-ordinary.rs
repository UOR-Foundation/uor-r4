use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::Path,
};
use uor_r4_tokenizer::ByteBpeTokenizer;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type Counts = BTreeMap<String, usize>;
fn count(c: &mut Counts, k: &str) {
    *c.entry(k.to_owned()).or_default() += 1;
}
fn ids(v: &Value) -> Result<Vec<u32>> {
    Ok(serde_json::from_value(v.clone())?)
}
fn load(p: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
fn position(v: &Value) -> Result<Option<usize>> {
    if v.is_null() {
        return Ok(None);
    }
    Ok(Some(usize::try_from(v.as_u64().ok_or("bad occurrence")?)?))
}
fn movement(b: Option<usize>, v: Option<usize>) -> &'static str {
    match (b, v) {
        (None, None) => "both_exact",
        (Some(_), None) => "recovered",
        (None, Some(_)) => "regressed",
        (Some(x), Some(y)) if y > x => "later",
        (Some(x), Some(y)) if y < x => "earlier",
        _ => "same_error_offset",
    }
}
fn origin(at: usize, first_prompt: usize, first_reply: usize, prompt: usize) -> &'static str {
    if at < first_prompt {
        "first_user_or_frame"
    } else if at < first_prompt + first_reply {
        "genuine_prior_assistant"
    } else if at < prompt {
        "question_or_frame"
    } else {
        "current_answer"
    }
}
fn main() -> Result<()> {
    let a: Vec<_> = env::args().skip(1).collect();
    let [tok, manifest, transcripts, pred, out] = a.as_slice() else {
        return Err("TOKENIZER MANIFEST TRANSCRIPTS PRED OUT".into());
    };
    let tb = fs::read(tok)?;
    if format!("{:x}", Sha256::digest(&tb))
        != "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377"
    {
        return Err("wrong client tokenizer".into());
    }
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tb).ok_or("invalid tokenizer")?;
    let cases = load(Path::new(manifest))?;
    let predictions = load(Path::new(pred))?;
    let mut by_id = BTreeMap::new();
    for row in predictions.as_array().ok_or("predictions shape")? {
        if by_id
            .insert(row["id"].as_str().ok_or("prediction ID")?, row)
            .is_some()
        {
            return Err("duplicate prediction".into());
        }
    }
    let mut seen = BTreeSet::new();
    let mut graded = Vec::new();
    let mut stat: BTreeMap<String, Counts> = BTreeMap::new();
    for case in cases.as_array().ok_or("manifest shape")? {
        let id = case["id"].as_str().ok_or("case ID")?;
        if !seen.insert(id) {
            return Err("duplicate case".into());
        }
        let role = case["role"].as_str().ok_or("role")?;
        let cohort = case["cohort"].as_str().ok_or("cohort")?;
        let groups = [
            "all".to_string(),
            format!("role/{role}"),
            format!("cohort/{cohort}"),
            format!("cohort/{cohort}/{role}"),
            format!("wording/{}", case["wording"]),
        ];
        let t = load(&Path::new(transcripts).join(format!("{id}.json")))?;
        if t["artifact_sha256"]
            != "2e16a8ea27b316cce01dac5744c6b02da38e0ee6c0abc3fcb610e381b1c85302"
            || t["tokenizer_sha256"]
                != "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377"
        {
            return Err("unbound transcript".into());
        }
        let turns = t["turns"].as_array().ok_or("turns")?;
        if turns.is_empty() || turns.len() > 2 || turns[0]["user"] != case["turns"][0] {
            return Err("history mismatch".into());
        }
        for group in &groups {
            count(stat.entry(group.clone()).or_default(), "attempted");
        }
        if turns.len() == 1 {
            if by_id.contains_key(id) || turns[0]["stop"] == "eos" {
                return Err("unexpected query access/missing query".into());
            }
            graded.push(json!({"id":id,"role":role,"cohort":cohort,"reached":false,"stop":turns[0]["stop"]}));
            continue;
        }
        let p = by_id.remove(id).ok_or("missing reached prediction")?;
        let prompt = ids(&turns[1]["prompt_ids"])?;
        let previous = ids(&turns[0]["reply_ids"])?;
        let first = ids(&turns[0]["prompt_ids"])?;
        let mut prefix = first.clone();
        prefix.extend(&previous);
        if turns[1]["user"] != case["turns"][1]
            || turns[0]["stop"] != "eos"
            || previous.last() != Some(&1)
            || !prompt.starts_with(&prefix)
            || prompt != ids(&p["prompt"])?
        {
            return Err("not genuine paired history".into());
        }
        let answer = case["answer"].as_str().ok_or("reference")?;
        let mut gold = tokenizer.encode(answer);
        if tokenizer.decode(&gold) != answer || gold.contains(&1) {
            return Err("unrepresentable reference".into());
        }
        gold.push(1);
        let mut result = json!({"id":id,"cohort":cohort,"role":role,"wording":case["wording"],"value":case["value"],"answer":answer,"gold":gold,"reached":true});
        let mut baseline = None;
        for arm in ["lexical", "oracle", "promoted"] {
            let ar = &p[arm];
            let reply = ids(&ar["reply"])?;
            if arm == "lexical" && reply != ids(&turns[1]["reply_ids"])? {
                return Err("lexical client diverged".into());
            }
            let err = (0..reply.len().max(gold.len())).find(|&i| reply.get(i) != gold.get(i));
            if arm == "lexical" {
                baseline = err;
            }
            let steps = ar["steps"].as_array().ok_or("missing steps")?;
            if steps.len() != reply.len() {
                return Err("trace length".into());
            }
            let mut context = prompt.clone();
            let mut repeats = Vec::new();
            let mut first_trace = Value::Null;
            for (i, s) in steps.iter().enumerate() {
                let cursor = position(s.get("cursor").ok_or("cursor field absent")?)?;
                let source = position(s.get("source").ok_or("source field absent")?)?;
                if s["emitted"] != reply[i]
                    || source.is_some_and(|at| context.get(at) != Some(&reply[i]))
                {
                    return Err("recorded read mismatch".into());
                }
                let event = json!({"i":i,"entering_cursor":cursor,"source":source,"origin":source.map(|v|origin(v,first.len(),previous.len(),prompt.len())),"output":reply[i],"expected":gold.get(i),"next_source_token":source.and_then(|v|context.get(v+1))});
                if source.is_some() && cursor == source {
                    repeats.push(event.clone());
                }
                if Some(i) == err {
                    first_trace = event;
                }
                context.push(reply[i]);
            }
            let r_at_err = repeats
                .iter()
                .any(|r| Some(r["i"].as_u64().unwrap() as usize) == err);
            let advancing_matches = r_at_err
                && !first_trace["expected"].is_null()
                && first_trace["expected"] == first_trace["next_source_token"];
            let shift = movement(baseline, err);
            let ended = reply.last() == Some(&1);
            result[arm] = json!({"reply":reply,"text":tokenizer.decode(reply.strip_suffix(&[1]).unwrap_or(&reply)),
                "first":err,"expected":err.and_then(|i|gold.get(i)),"actual":err.and_then(|i|reply.get(i)),"first_trace":first_trace,
                "ended_eos":ended,"exact":err.is_none(),"movement":shift,"repeats":repeats,"repeat_at_error":r_at_err,"advance_matches":advancing_matches});
            for group in &groups {
                let counts = stat.entry(format!("{group}/{arm}")).or_default();
                count(counts, "reached");
                count(counts, shift);
                count(
                    counts,
                    &err.map_or("exact".to_owned(), |v| format!("wrong_{v}")),
                );
                if ended {
                    count(counts, "ended_eos");
                }
                if !repeats.is_empty() {
                    count(counts, "repeat_any");
                }
                if r_at_err {
                    count(counts, "repeat_at_first_wrong");
                }
                if advancing_matches {
                    count(counts, "advance_matches");
                }
            }
        }
        graded.push(result);
    }
    if !by_id.is_empty() {
        return Err("unknown prediction IDs".into());
    }
    let f = fs::File::options().create_new(true).write(true).open(out)?;
    serde_json::to_writer(f, &json!({"counts":stat,"rows":graded}))?;
    Ok(())
}
