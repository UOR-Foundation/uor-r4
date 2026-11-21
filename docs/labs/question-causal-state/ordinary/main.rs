use serde_json::{json, Value};
use std::{env, fs};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths: Vec<_> = env::args().skip(1).collect();
    let [artifact, transcript] = paths.as_slice() else {
        return Err("ARTIFACT TRANSCRIPT".into());
    };
    let model = IntegerDiscreteModel::from_bytes(&fs::read(artifact)?)?;
    let transcript: Value = serde_json::from_slice(&fs::read(transcript)?)?;
    if transcript["artifact_sha256"] != model.artifact_sha256() {
        return Err("artifact mismatch".into());
    }
    let reader = model.question_reader().ok_or("missing reader")?;
    let mut turns = Vec::new();
    for turn in transcript["turns"].as_array().ok_or("missing turns")? {
        let prompt: Vec<u32> = serde_json::from_value(turn["prompt_ids"].clone())?;
        let (reply, words, retry) = reader.asked.reply_words(&reader.ladder, &prompt, 48);
        if json!(reply) != turn["reply_ids"] || words.len() != 23 * reply.len() {
            return Err("ordinary transcript mismatch".into());
        }
        let mut prefix = prompt;
        let mut steps = Vec::new();
        for (i, step) in words.chunks_exact(23).enumerate() {
            if step[22] != reply[i] {
                return Err("step/output mismatch".into());
            }
            let at = (step[21] != u32::MAX).then_some(step[21]);
            let copied = at
                .map(|p| {
                    prefix
                        .get(p as usize)
                        .copied()
                        .ok_or("invalid source position")
                })
                .transpose()?;
            if copied.is_some_and(|token| token != step[22]) {
                return Err("copy source disagrees with output".into());
            }
            steps.push(json!({"index":i,"kind":step[0],"form":step[2],"entry":step[16..21].to_vec(),"cursor":(step[14]!=u32::MAX).then_some(step[14]),"source":at,"emitted":step[22]}));
            prefix.push(step[22]);
        }
        turns.push(json!({"user":turn["user"],"reply":turn["reply_text"],"empty_retry":retry,"steps":steps}));
    }
    println!("{}", serde_json::to_string_pretty(&turns)?);
    Ok(())
}
