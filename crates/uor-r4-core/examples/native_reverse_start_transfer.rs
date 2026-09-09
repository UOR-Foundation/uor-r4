//! Offline exact-occurrence labels and real generation for reverse-start transfer.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{
    Control, Model, RelationStartExample, RelationStartOverride, SourceRoutingConfig, BOS, EOS,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Clone, Serialize, Deserialize)]
struct Target {
    owner: String,
    payload: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Case {
    id: String,
    prompt: String,
    expected: Option<String>,
    #[serde(default)]
    current_record: Option<u64>,
    #[serde(default)]
    targets: Vec<Target>,
}
fn save(path: &Path, v: &impl Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec(v)?)?;
    Ok(())
}
fn generate(
    model: &Model,
    prompt: &str,
    control: Control,
    verify: bool,
    inherited: bool,
) -> Result<Value> {
    let tokens = model.encode(prompt)?;
    if prompt.len() > if inherited { 65536 } else { 4096 }
        || tokens.len() > if inherited { 8192 } else { 512 }
    {
        return Err(format!(
            "configured input bound: {} bytes, {} tokens",
            prompt.len(),
            tokens.len()
        )
        .into());
    }
    let mut session = model.session(control)?;
    session.observe(model, BOS)?;
    for t in tokens {
        session.observe(model, t)?;
    }
    session.begin_response(model)?;
    let initial: Value = serde_json::from_slice(&session.checkpoint()?)?;
    let mut generated = Vec::new();
    let mut first = Value::Null;
    let mut eos = false;
    let mut positions = 0;
    for i in 0..96 {
        let mut restored = if verify {
            Some(model.restore_session(&session.checkpoint()?)?)
        } else {
            None
        };
        let p = session.predict(model)?;
        if verify && session.predict(model)? != p {
            return Err("repeated prediction differs".into());
        }
        if let Some(s) = &mut restored {
            if s.predict(model)? != p {
                return Err("restored prediction differs".into());
            }
        }
        if i == 0 {
            first = json!({"word_copy":session.word_copy_decision(),"field":session.field_composition_decision()});
        }
        session.observe(model, p.token)?;
        if let Some(s) = &mut restored {
            s.observe(model, p.token)?;
            let a: Value = serde_json::from_slice(&session.checkpoint()?)?;
            let b: Value = serde_json::from_slice(&s.checkpoint()?)?;
            for key in [
                "values",
                "word_copy",
                "field_composition",
                "response_entry",
                "completion",
            ] {
                if a[key] != b[key] {
                    return Err(format!("restored {key} differs").into());
                }
            }
            positions += 1;
        }
        if p.token == EOS {
            eos = true;
            break;
        }
        generated.push(p.token);
    }
    let final_state: Value = serde_json::from_slice(&session.checkpoint()?)?;
    Ok(
        json!({"text":String::from_utf8(model.decode(&generated)?)?,"eos":eos,"first_decision":first,"initial_relations":initial["values"]["relations"],"final_relations":final_state["values"]["relations"],"checkpoint_positions":positions}),
    )
}
fn comparable(v: &Value) -> Value {
    let mut first = v["first_decision"].clone();
    for key in ["field", "word_copy"] {
        if let Some(o) = first[key].as_object_mut() {
            o.remove("score");
        }
    }
    json!({"text":v["text"],"eos":v["eos"],"first_decision":first,"initial_relations":v["initial_relations"],"final_relations":v["final_relations"]})
}
fn atom(v: &Value) -> Result<String> {
    let bytes = v["bytes"].as_array().ok_or("atom bytes")?;
    let n = v["len"].as_u64().ok_or("atom length")? as usize;
    let b = bytes[..n]
        .iter()
        .map(|x| {
            x.as_u64()
                .and_then(|n| u8::try_from(n).ok())
                .ok_or("atom byte")
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(String::from_utf8(b)?)
}
fn number(v: &Value, k: &str) -> Result<u64> {
    Ok(v[k].as_u64().ok_or_else(|| format!("missing {k}"))?)
}
fn balanced() -> Vec<Case> {
    let mut out = Vec::new();
    for (owner, words) in [
        ("selvi", ["moss", "dale", "bank"]),
        ("tilva", ["pine", "cove", "bank"]),
        ("venri", ["elm", "hill", "bank"]),
    ]
    .iter()
    {
        for n in 1..=3 {
            let payload = words[..n].join(" ");
            for (i, prefix) in ["Record: ", "", "notes say ", "report says "]
                .iter()
                .enumerate()
            {
                out.push(Case {
                    id: format!("reverse/{owner}/{n}/{i}"),
                    prompt: format!("{prefix}{payload} holds {owner}. Where is {owner}? Answer:"),
                    expected: Some(format!(" {payload}.\n")),
                    current_record: None,
                    targets: vec![Target {
                        owner: (*owner).into(),
                        payload: payload.clone(),
                    }],
                });
            }
        }
    }
    out
}
fn prepare(model: &Model, out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let mut cases:Vec<Case>=serde_json::from_slice(&fs::read("/Users/casey.allard/Documents/Codex/2026-09-09/uor-r4-historical-selection/construction/cases.json")?)?;
    for c in &mut cases {
        c.id = format!("preserve/{}", c.id);
    }
    cases.extend(balanced());
    let mut docs = Vec::new();
    let mut traces = Vec::new();
    for c in &cases {
        let mut overrides = Vec::new();
        if !c.targets.is_empty() {
            let trace = model.relation_start_trace(&c.prompt)?;
            for target in &c.targets {
                let events = trace["events"].as_array().ok_or("events")?;
                let matching = events
                    .iter()
                    .filter(|e| atom(&e["owner"]).ok().as_deref() == Some(target.owner.as_str()))
                    .collect::<Vec<_>>();
                if matching.len() != 1 {
                    return Err(
                        format!("unique writer event absent {}: {}", c.id, matching.len()).into(),
                    );
                }
                let e = matching[0];
                let candidates = e["candidates"].as_array().ok_or("candidates")?;
                let choices = candidates
                    .iter()
                    .filter(|a| a["payload"] == target.payload)
                    .collect::<Vec<_>>();
                if choices.len() != 1 {
                    return Err(format!(
                        "complete target not uniquely admitted {}: {}",
                        c.id, target.payload
                    )
                    .into());
                }
                let a = choices[0];
                overrides.push(RelationStartOverride {
                    record_id: number(e, "record_id")?,
                    owner_end: number(&e["owner"], "end")?,
                    owner_byte_end: number(&e["owner"], "byte_end")?,
                    endpoint_end: number(&e["endpoint"], "end")?,
                    endpoint_byte_end: number(&e["endpoint"], "byte_end")?,
                    start_end: number(a, "start_end")?,
                    start_byte_end: number(a, "start_byte_end")?,
                });
            }
            traces.push(json!({"id":c.id,"trace":trace}));
        }
        docs.push(RelationStartExample {
            id: c.id.clone(),
            prompt: c.prompt.clone(),
            overrides,
        });
    }
    save(&out.join("cases.json"), &cases)?;
    save(&out.join("training.json"), &docs)?;
    save(&out.join("target-traces.json"), &traces)?;
    save(
        &out.join("receipt.json"),
        &json!({"parent":model.artifact_cid(),"cases":cases.len(),"target_documents":36,"preservation_documents":cases.len()-36,"scope":"Only exact observed owner/endpoint/start occurrence labels enter fitting. Expected output strings are evaluation-only. No previous fresh spellings are training inputs."}),
    )
}
fn fresh(out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let draw = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let mut cases = Vec::new();
    for i in 0..3 {
        let h = blake3::hash(format!("reverse-start:{draw}:{i}").as_bytes());
        let b = h.as_bytes();
        let word = |s: usize, n: usize| {
            (s..s + n)
                .map(|j| (b'a' + b[j] % 26) as char)
                .collect::<String>()
        };
        let owner = word(0, 5);
        let words = [word(5, 4), word(9, 4), word(13, 4)];
        let current = format!("{} {}", word(17, 4), word(21, 4));
        for n in 1..=3 {
            let old = words[..n].join(" ");
            for (p, prefix) in ["Record: ", "", "notes say ", "report says "]
                .iter()
                .enumerate()
            {
                for evicted in [false, true] {
                    let padding = if evicted {
                        "oak ash elm ".repeat(12)
                    } else {
                        String::new()
                    };
                    let fact =
                        format!("{prefix}{old} holds {owner}. {owner} now in {current}. {padding}");
                    for form in 0..3 {
                        let question = match form {
                            0 => format!("Where was {owner} before? Answer:"),
                            1 => format!("What was the previous location of {owner}? Answer:"),
                            _ => format!("Where is {owner}? Answer:"),
                        };
                        cases.push(Case {
                            id: format!("fresh/{i}/{n}/{p}/{evicted}/{form}"),
                            prompt: format!("{fact}{question}"),
                            expected: Some(format!(
                                " {}.\n",
                                if form == 2 { &current } else { &old }
                            )),
                            current_record: if form == 2 { None } else { Some(2) },
                            targets: vec![Target {
                                owner: owner.clone(),
                                payload: old.clone(),
                            }],
                        });
                    }
                }
            }
        }
    }
    save(&out.join("cases.json"), &cases)?;
    save(
        &out.join("receipt.json"),
        &json!({"draw":draw.to_string(),"cases":cases.len(),"scope":"New full owner/value spellings after selection; balanced one/two/three-word reverse values, prefixes, eviction and current/previous questions. No fit."}),
    )
}
fn record_identities(v: &Value) -> Value {
    let mut x = v.clone();
    if let Some(records) = x["records"].as_array_mut() {
        for r in records {
            r.as_object_mut().map(|o| o.remove("span"));
        }
    }
    x
}
fn evaluate(model: &Model, cases: &[Case], out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let parent = model.without_relation_start_refinement()?;
    let mut rows = Vec::new();
    let (mut targets, mut exact, mut preserved, mut inherited, mut writer_exact, mut selections) =
        (0, 0, 0, 0, 0, 0);
    for c in cases {
        let target = !c.targets.is_empty() || c.current_record.is_some();
        let inherited_window = c.targets.is_empty();
        let a = generate(
            model,
            &c.prompt,
            Control::Full,
            !c.targets.is_empty(),
            inherited_window,
        )?;
        let p = generate(&parent, &c.prompt, Control::Full, false, inherited_window)?;
        let d = if target {
            Some(generate(
                model,
                &c.prompt,
                Control::RelationStartRefinementDisabled,
                false,
                inherited_window,
            )?)
        } else {
            None
        };
        let same = comparable(&a) == comparable(&p);
        let control_equal = d.as_ref().map(|v| comparable(v) == comparable(&p));
        if record_identities(&a["initial_relations"]) != record_identities(&p["initial_relations"])
        {
            return Err(format!("writer occurrence/version identity changed {}", c.id).into());
        }
        if a["initial_relations"] != a["final_relations"] {
            return Err(format!("generation mutated relation records {}", c.id).into());
        }
        let mut writer_ok = true;
        for t in &c.targets {
            let records = a["initial_relations"]["records"]
                .as_array()
                .ok_or("records")?;
            let old = records
                .iter()
                .filter(|r| {
                    r["previous"] == 0
                        && atom(&r["owner"]).ok().as_deref() == Some(t.owner.as_str())
                })
                .collect::<Vec<_>>();
            writer_ok &= old.len() == 1
                && atom(if old[0]["span"].is_null() {
                    &old[0]["value"]
                } else {
                    &old[0]["span"]
                })? == t.payload;
        }
        let correct = c
            .expected
            .as_ref()
            .map(|s| a["text"] == *s && a["eos"] == true);
        let mut selected = None;
        if let Some(id) = c.current_record {
            let records = a["initial_relations"]["records"]
                .as_array()
                .ok_or("records")?;
            let current = records
                .iter()
                .find(|r| r["id"] == id)
                .ok_or("current record")?;
            let old = records
                .iter()
                .find(|r| r["id"] == current["previous"])
                .ok_or("previous record")?;
            let ok = a["first_decision"]["word_copy"]["word_index"]
                == 32 + ((number(old, "id")? - 1) & 15)
                && a["first_decision"]["word_copy"]["source_end"] == old["value"]["end"]
                && a["first_decision"]["word_copy"]["source_byte_end"] == old["value"]["byte_end"];
            selected = Some(ok);
            selections += usize::from(ok);
        }
        if target {
            targets += 1;
            exact += usize::from(correct == Some(true));
            writer_exact += usize::from(writer_ok);
        }
        // Historical targets are also inherited writer controls. Their complete
        // outputs and spans must remain exact, independently of answer scoring.
        if c.targets.is_empty() {
            inherited += 1;
            preserved += usize::from(same);
        }
        rows.push(json!({"id":c.id,"prompt":c.prompt,"expected":c.expected,"target":target,"current_record":c.current_record,"correct":correct,"writer_correct":writer_ok,"selected_previous":selected,"parent_equal":same,"control_parent_equal":control_equal,"actual":a,"parent":p}));
    }
    save(
        &out.join("result.json"),
        &json!({"artifact":model.artifact_cid(),"parent":parent.artifact_cid(),"total":cases.len(),"targets":targets,"exact":exact,"writer_exact":writer_exact,"historical_targets":cases.iter().filter(|c|c.current_record.is_some()).count(),"selections":selections,"inherited":inherited,"preserved":preserved,"controls_parent_equal":rows.iter().all(|r|r["control_parent_equal"].as_bool().unwrap_or(true)),"rows":rows}),
    )
}
fn main() -> Result<()> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() == 3 && a[1] == "fresh" {
        return fresh(Path::new(&a[2]));
    }
    if a.len() < 4 {
        return Err("usage: native_reverse_start_transfer diagnose/prepare MODEL OUT | fit/evaluate MODEL DATA OUT | fresh OUT".into());
    }
    let bytes = fs::read(&a[2])?;
    let model = Model::from_bytes(&bytes)?;
    if model.to_bytes()? != bytes {
        return Err("artifact roundtrip differs".into());
    }
    if a[1] == "diagnose" {
        fs::create_dir(&a[3])?;
        let mut traces = Vec::new();
        for prompt in["Record: eoqn rgoo holds eddwg. eddwg now in bbew ezye. Where was eddwg before? Answer:","Record: moss dale holds selvi. Where is selvi? Answer:","Record: moss holds selvi. Where is selvi? Answer:","notes say moss dale holds selvi. Where is selvi? Answer:"]{traces.push(model.relation_start_trace(prompt)?);}
        return save(&Path::new(&a[3]).join("trace.json"), &traces);
    }
    if a[1] == "prepare" {
        return prepare(&model, Path::new(&a[3]));
    }
    if a.len() != 5 {
        return Err("five arguments required".into());
    }
    if a[1] == "fit" {
        fs::create_dir(&a[4])?;
        let docs: Vec<RelationStartExample> = serde_json::from_slice(&fs::read(&a[3])?)?;
        let config = SourceRoutingConfig {
            seed: 973,
            passes: 4,
            proposals: 32,
            max_seconds: 120,
            learned_features: 256,
            role_context_only: true,
            ..Default::default()
        };
        let (candidate, report) = model.fit_relation_start_refinement(&docs, config)?;
        fs::write(Path::new(&a[4]).join("model.json"), candidate.to_bytes()?)?;
        return save(&Path::new(&a[4]).join("fit.json"), &report);
    }
    if a[1] == "evaluate" {
        let cases: Vec<Case> = serde_json::from_slice(&fs::read(&a[3])?)?;
        return evaluate(&model, &cases, Path::new(&a[4]));
    }
    Err("unknown mode".into())
}
