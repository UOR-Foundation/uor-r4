use serde_json::{json, Value};
use std::{env, fs};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;
type R<T> = Result<T, Box<dyn std::error::Error>>;
fn arm(
    reader: &uor_r4_reader_runtime::conjunction::saved::Reader,
    prompt: &[u32],
    forms: Option<&[u32]>,
) -> R<Value> {
    let (reply, words, retry) = match forms {
        Some(f) => reader
            .asked
            .reply_words_for_forms(&reader.ladder, prompt, 48, f),
        None => reader.asked.reply_words(&reader.ladder, prompt, 48),
    };
    if words.len() != reply.len() * 23 {
        return Err("step count mismatch".into());
    }
    let mut prefix = prompt.to_vec();
    let mut steps = Vec::new();
    for (i, s) in words.chunks_exact(23).enumerate() {
        if s[22] != reply[i] {
            return Err("emitted mismatch".into());
        }
        let opt = |v| (v != u32::MAX).then_some(v);
        let source = opt(s[21]);
        if let Some(p) = source {
            if prefix.get(p as usize) != Some(&s[22]) {
                return Err("copy mismatch".into());
            }
        }
        steps.push(json!({"index":i,"kind":s[0],"form_index":opt(s[1]),"form":opt(s[2]),"empty_pass":s[3],"empty_row":s[4],"cursor":opt(s[14]),"source":source,"entry":s[16..21],"emitted":s[22],"fallback":s[0]>=2,"raw":s}));
        prefix.push(s[22]);
    }
    Ok(json!({"reply":reply,"steps":steps,"empty_read_retry":retry}))
}
fn main() -> R<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 4 {
        return Err("usage: supplied-form-trace ARTIFACT dev|predict INPUT".into());
    }
    let model = IntegerDiscreteModel::from_bytes(&fs::read(&args[1])?)?;
    let reader = model.question_reader().ok_or("missing reader")?;
    let input: Value = serde_json::from_slice(&fs::read(&args[3])?)?;
    let mut out = Vec::new();
    if args[2] == "dev" {
        for row in input.as_array().ok_or("array")? {
            if row["artifact_sha256"] != model.artifact_sha256() {
                return Err("artifact mismatch".into());
            }
            let prompt: Vec<u32> = serde_json::from_value(row["prompt_ids"].clone())?;
            let normal = arm(&reader, &prompt, None)?;
            if normal["reply"] != row["generated_ids"] {
                return Err("dev replay mismatch".into());
            }
            let mut forms: Vec<u32> = normal["steps"]
                .as_array()
                .ok_or("steps")?
                .iter()
                .filter_map(|s| s["form"].as_u64().map(|x| x as u32))
                .collect();
            forms.sort_unstable();
            forms.dedup();
            let mut singles = Vec::new();
            for f in forms {
                singles.push(json!({"form":f,"arm":arm(&reader,&prompt,Some(&[f]))?}));
            }
            out.push(json!({"id":row["id"],"prompt":prompt,"lexical":normal,"singletons":singles}));
        }
    } else if args[2] == "semantics" {
        let path = input["tokenizer"].as_str().ok_or("tokenizer path")?;
        let tok = uor_r4_tokenizer::ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(path)?)
            .ok_or("bad tokenizer")?;
        for row in input["rows"].as_array().ok_or("rows")? {
            let prompt: Vec<u32> = serde_json::from_value(row["prompt"].clone())?;
            let reply: Vec<u32> = serde_json::from_value(row["lexical"]["reply"].clone())?;
            let p = tok.decode(&prompt);
            let r = tok.decode(&reply);
            let job = row["id"] == "development-0" || row["id"] == "development-2";
            let q = if job {
                "What is my current job?"
            } else {
                "Where do I currently live?"
            };
            let a = if job { "Your job is" } else { "You live in" };
            let expected_form = if job { 1223 } else { 1553 };
            if row["singletons"][0]["form"] != expected_form || !p.contains(q) || !r.contains(a) {
                return Err(format!(
                    "role semantic assertion failed: {} prompt={p:?} reply={r:?}",
                    row["id"]
                )
                .into());
            }
            out.push(json!({"id":row["id"],"role":if job {"job"} else {"home"},"decoded_prompt":p,"decoded_reply":r}));
        }
    } else if args[2] == "identities" {
        let forms = reader.asked.forms_by_id();
        for id in [1223u32, 1553u32] {
            out.push(json!({"id":id,"words":forms.get(&id).ok_or("missing form")?,"artifact_sha256":model.artifact_sha256(),"reader_sha256":model.question_reader_sha256()}));
        }
    } else if args[2] == "predict" {
        let job = u32::try_from(input["job_form"].as_u64().ok_or("job form")?)?;
        let home = u32::try_from(input["home_form"].as_u64().ok_or("home form")?)?;
        if job != 1223
            || home != 1553
            || model.artifact_sha256()
                != "2e16a8ea27b316cce01dac5744c6b02da38e0ee6c0abc3fcb610e381b1c85302"
            || model.question_reader_sha256()
                != Some("9db7e8348bb0bdadee7502a35716d39dacb8e9c25e57f76338fe3fea91e1227b")
        {
            return Err("fixed form or artifact binding mismatch".into());
        }
        for row in input["rows"].as_array().ok_or("rows")? {
            let prompt: Vec<u32> = serde_json::from_value(row["prompt"].clone())?;
            let form = match row["role"].as_str() {
                Some("job") => job,
                Some("home") => home,
                _ => return Err("unknown role".into()),
            };
            let lexical = arm(&reader, &prompt, None)?;
            if lexical["reply"] != row["ordinary_reply"] {
                return Err(format!("ordinary reply mismatch {}", row["id"]).into());
            }
            let mut promoted = vec![form];
            promoted.extend(
                reader
                    .asked
                    .form_ids(&prompt)
                    .into_iter()
                    .filter(|&f| f != form),
            );
            out.push(json!({"id":row["id"],"prompt":prompt,"lexical":lexical,"oracle":arm(&reader,&prompt,Some(&[form]))?,"promoted_forms":promoted,"promoted":arm(&reader,&prompt,Some(&promoted))?}));
        }
    } else {
        return Err("unknown mode".into());
    }
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
