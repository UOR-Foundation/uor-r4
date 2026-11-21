use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, fs};
use uor_r4_tokenizer::ByteBpeTokenizer;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    let [tok, train, manifest] = args.as_slice() else {
        return Err("TOKENIZER TRAIN MANIFEST".into());
    };
    let data = fs::read(train)?;
    if format!("{:x}", Sha256::digest(&data))
        != "bfbf581256a1ef332c005134cfaeefc3bcd0d9653281fa248cab84c0bc0bfc24"
        || &data[..4] != b"UORT"
    {
        return Err("TRAIN mismatch".into());
    }
    let ids: Vec<u32> = data[64..]
        .chunks_exact(2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]) as u32)
        .collect();
    if u64::from_le_bytes(data[8..16].try_into()?) != ids.len() as u64 {
        return Err("count mismatch".into());
    }
    let tb = fs::read(tok)?;
    if format!("{:x}", Sha256::digest(&tb))
        != "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377"
    {
        return Err("tokenizer mismatch".into());
    }
    let codec = ByteBpeTokenizer::from_tokenizer_json_bytes(&tb).ok_or("bad tokenizer")?;
    let rows: Vec<Value> = serde_json::from_slice(&fs::read(manifest)?)?;
    let mut texts = BTreeMap::new();
    for row in &rows {
        for val in row["turns"].as_array().ok_or("missing turn texts")? {
            let text = val.as_str().ok_or("invalid turn")?;
            texts.entry(text).or_insert_with(|| {
                let count = |s: &str| {
                    let q = codec.encode(s);
                    ids.windows(q.len()).filter(|&w| w == q.as_slice()).count()
                };
                json!({"bare":count(text),"leading_space":count(&format!(" {text}"))})
            });
        }
    }
    let mut summary = BTreeMap::<String, [usize; 2]>::new();
    for row in &rows {
        let a = &texts[row["turns"][0].as_str().ok_or("first")?];
        let b = &texts[row["turns"][1].as_str().ok_or("second")?];
        let novel = |v: &Value| v["bare"] == 0 && v["leading_space"] == 0;
        let x = summary
            .entry(row["cohort"].as_str().ok_or("cohort")?.to_owned())
            .or_default();
        x[0] += usize::from(novel(a));
        x[1] += usize::from(novel(b));
    }
    println!(
        "{}",
        json!({"new_full_utterances_per_row_first_then_second":summary,"turn_occurrences":texts})
    );
    Ok(())
}
