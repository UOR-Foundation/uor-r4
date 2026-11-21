use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::Path,
};
use uor_r4_tokenizer::ByteBpeTokenizer;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    let [tokenizer, capture, dispatch, expected_count] = args.as_slice() else {
        return Err("TOKENIZER CAPTURE DISPATCH EXPECTED_COUNT".into());
    };
    let t = fs::read(tokenizer)?;
    if format!("{:x}", Sha256::digest(&t))
        != "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377"
    {
        return Err("unbound tokenizer".into());
    }
    let codec = ByteBpeTokenizer::from_tokenizer_json_bytes(&t).ok_or("bad tokenizer")?;
    let root = Path::new(capture);
    let rows: Vec<Value> = serde_json::from_slice(&fs::read(root.join("manifest.json"))?)?;
    let spans: Vec<Value> = serde_json::from_slice(&fs::read(root.join("source-spans.json"))?)?;
    let packet: Value = serde_json::from_slice(&fs::read(dispatch)?)?;
    let mut inputs = BTreeMap::new();
    for r in packet["rows"].as_array().ok_or("dispatch rows")? {
        if inputs
            .insert(r["id"].as_str().ok_or("input id")?, r)
            .is_some()
        {
            return Err("duplicate dispatch id".into());
        }
    }
    if rows.is_empty() || rows.len() != expected_count.parse::<usize>()? || spans.len() != rows.len() {
        return Err("panel coverage".into());
    }
    let mut seen = BTreeSet::new();
    let mut reached = 0;
    for (row, s) in rows.iter().zip(&spans) {
        let id = row["id"].as_str().ok_or("row id")?;
        if row["id"] != s["id"] || !seen.insert(id) {
            return Err("span row join".into());
        }
        let tr: Value = serde_json::from_slice(&fs::read(
            root.join("transcripts").join(format!("{id}.json")),
        )?)?;
        let first = &tr["turns"][0];
        let prompt: Vec<u32> = serde_json::from_value(first["prompt_ids"].clone())?;
        let at = usize::try_from(s["job_value_start"].as_u64().ok_or("missing start")?)?;
        let end = usize::try_from(s["job_value_end"].as_u64().ok_or("missing end")?)?;
        let text = first["user"].as_str().ok_or("user text")?;
        let (_, value_tail) = text.split_once("My current job is ").ok_or("no job fact")?;
        let (value, _) = value_tail.split_once('.').ok_or("no end of job fact")?;
        let value_bound = if end < prompt.len() && at < end {
            let left = codec.decode(&prompt[..at]);
            let slice = codec.decode(&prompt[at..end]);
            // The BPE can encode the separating space independently or with the first letters.
            (left.ends_with("My current job is ") && slice == value)
                || (left.ends_with("My current job is") && slice == format!(" {value}"))
        } else {
            false
        };
        if first["user"] != row["turns"][0]
            || s["first_prompt_ids"] != first["prompt_ids"]
            || at + 1 >= end
            || end >= prompt.len()
            || !value_bound
            || !codec.decode(&prompt[end..]).starts_with('.')
        {
            return Err(
                format!("source span does not decode to the original job fact: {id}").into(),
            );
        }
        match tr["turns"].as_array().ok_or("turns")?.len() {
            1 if first["stop"] != "eos" && !inputs.contains_key(id) => {}
            2 => {
                let input = inputs.remove(id).ok_or("reached row absent")?;
                let second = &tr["turns"][1];
                if first["stop"] != "eos"
                    || input["job_value_start"] != s["job_value_start"]
                    || input["role"] != row["role"]
                    || input["prompt"] != second["prompt_ids"]
                    || input["ordinary_reply"] != second["reply_ids"]
                {
                    return Err("dispatch differs from captured case".into());
                }
                reached += 1;
            }
            _ => return Err("unreachable/reached disposition".into()),
        }
    }
    if !inputs.is_empty() {
        return Err("extra inputs".into());
    }
    println!(
        "{}",
        json!({"attempted":rows.len(),"original_user_spans_verified":rows.len(),"reached_dispatches_verified":reached})
    );
    Ok(())
}
