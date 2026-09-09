//! Authored historical-version construction and actual generation; labels stay offline.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{
    Control, HistoricalReadExample, Model, SourceRoutingConfig, BOS, EOS,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Serialize, Deserialize)]
struct Case {
    id: String,
    prompt: String,
    expected: Option<String>,
    #[serde(default)]
    current_record: Option<u64>,
}
fn save(path: &Path, value: &impl Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec(value)?)?;
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
fn query(owner: &str, form: usize) -> String {
    if form == 0 {
        format!("Where was {owner} before? Answer:")
    } else {
        format!("What was the previous location of {owner}? Answer:")
    }
}
fn prepare(base: Option<&Path>, out: &Path, fresh: bool) -> Result<()> {
    fs::create_dir(out)?;
    let mut cases: Vec<Case> = if let Some(p) = base {
        serde_json::from_slice(&fs::read(p)?)?
    } else {
        Vec::new()
    };
    // Replace five explicitly named earlier unqualified preservation cases with exact old-version labels.
    for c in &mut cases {
        let pair = if c.id.starts_with("open/0/history-raw-absent/") {
            Some(("Dusk Ridge", 2))
        } else if c.id.starts_with("open/1/history-raw-absent/") {
            Some(("Silver Cove", 2))
        } else if c.id == "writer-stress/stress/control/0" {
            Some(("Dusk Ridge", 2))
        } else {
            None
        };
        if let Some((value, id)) = pair {
            if c.id.ends_with("/0") || c.id.ends_with("/1") {
                c.current_record = Some(id);
                c.expected = Some(format!(" {value}.\n"));
            }
        }
    }
    let draw = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let mut owners = vec![
        (
            "selvi".to_owned(),
            "Dusk Ridge".to_owned(),
            "Copper Vale".to_owned(),
        ),
        (
            "tilva".to_owned(),
            "Silver Cove".to_owned(),
            "Birch Grove".to_owned(),
        ),
    ];
    if fresh {
        owners.clear();
        for i in 0..3 {
            let h = blake3::hash(format!("historical:{draw}:{i}").as_bytes());
            let b = h.as_bytes();
            let word = |s: usize, n: usize| {
                (s..s + n)
                    .map(|j| (b'a' + b[j] % 26) as char)
                    .collect::<String>()
            };
            owners.push((
                word(0, 5),
                format!("{} {}", word(5, 4), word(9, 4)),
                format!("{} {}", word(13, 4), word(17, 4)),
            ));
        }
    }
    for (i, (owner, old, new)) in owners.iter().enumerate() {
        for reverse in [false, true] {
            for evicted in [false, true] {
                for third in [false, true] {
                    let first = if reverse {
                        format!("{old} holds {owner}.")
                    } else {
                        format!("{owner} in {old}.")
                    };
                    let facts = format!(
                        "Record: {first} {owner} now in {new}.{}",
                        if third {
                            format!(" {owner} now in Amber Field.")
                        } else {
                            String::new()
                        }
                    );
                    let padding = if evicted {
                        "oak ash elm ".repeat(12)
                    } else {
                        String::new()
                    };
                    for form in 0..2 {
                        cases.push(Case {
                            id: format!("history/{i}/{reverse}/{evicted}/{third}/{form}"),
                            prompt: format!("{facts} {padding}{}", query(owner, form)),
                            expected: Some(format!(" {}.\n", if third { new } else { old })),
                            current_record: Some(if third { 3 } else { 2 }),
                        });
                    }
                    cases.push(Case {
                        id: format!("current/{i}/{reverse}/{evicted}/{third}"),
                        prompt: format!("{facts} {padding}Where is {owner}? Answer:"),
                        expected: None,
                        current_record: None,
                    });
                }
            }
        }
    }
    for order in 0..2 {
        let fact = |i: usize| {
            let (o, a, b) = &owners[i];
            format!("{a} holds {o}. {o} now in {b}.")
        };
        let facts = format!("Record: {} {}", fact(order), fact(1 - order));
        for q in 0..2 {
            for evicted in [false, true] {
                let padding = if evicted {
                    "oak ash elm ".repeat(12)
                } else {
                    String::new()
                };
                for form in 0..2 {
                    cases.push(Case {
                        id: format!("competing/{order}/{q}/{evicted}/{form}"),
                        prompt: format!("{facts} {padding}{}", query(&owners[q].0, form)),
                        expected: Some(format!(" {}.\n", owners[q].1)),
                        current_record: Some(if q == order { 2 } else { 4 }),
                    });
                }
            }
        }
        for form in 0..2 {
            cases.push(Case {
                id: format!("absent/{order}/{form}"),
                prompt: format!("{facts} {}", query("merli", form)),
                expected: None,
                current_record: None,
            });
        }
    }
    if cases.len() > 1500 {
        return Err("historical population cap".into());
    }
    let docs: Vec<_> = cases
        .iter()
        .map(|c| HistoricalReadExample {
            id: c.id.clone(),
            prompt: c.prompt.clone(),
            current_record: c.current_record,
        })
        .collect();
    save(&out.join("cases.json"), &cases)?;
    save(&out.join("training.json"), &docs)?;
    save(
        &out.join("receipt.json"),
        &json!({"base":base,"base_blake3":base.map(fs::read).transpose()?.map(|b|blake3::hash(&b).to_hex().to_string()),"fresh":fresh,"draw":draw,"owners":owners,"documents":cases.len(),"targets":cases.iter().filter(|c|c.current_record.is_some()).count(),"scope":"Offline authored immediate-previous record labels; expected response bytes excluded from training. All untargeted cases retain parent behavior."}),
    )
}
fn evaluate(model: &Model, cases: &[Case], out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let parent = model.without_historical_read()?;
    let mut rows = Vec::new();
    let (mut targets, mut exact, mut inherited, mut preserved, mut selections) = (0, 0, 0, 0, 0);
    for c in cases {
        let p = generate(&parent, &c.prompt, Control::Full, false, true)?;
        let a = generate(
            model,
            &c.prompt,
            Control::Full,
            c.current_record.is_some(),
            true,
        )?;
        let disabled = if c.current_record.is_some() {
            Some(generate(
                model,
                &c.prompt,
                Control::HistoricalReadDisabled,
                false,
                true,
            )?)
        } else {
            None
        };
        if a["initial_relations"] != p["initial_relations"]
            || a["final_relations"] != p["final_relations"]
        {
            return Err(format!("historical reader changed relation state: {}", c.id).into());
        }
        let equal = comparable(&a) == comparable(&p);
        let correct = c
            .expected
            .as_ref()
            .map(|e| a["text"] == *e && a["eos"] == true);
        let mut selected = None;
        if let Some(id) = c.current_record {
            targets += 1;
            exact += usize::from(correct == Some(true));
            let records = a["initial_relations"]["records"]
                .as_array()
                .ok_or("records absent")?;
            let current = records
                .iter()
                .find(|r| r["id"] == id)
                .ok_or("labelled current absent")?;
            let previous = records
                .iter()
                .find(|r| r["id"] == current["previous"])
                .ok_or("labelled previous absent")?;
            let ok = a["first_decision"]["word_copy"]["word_index"]
                == 32 + ((previous["id"].as_u64().ok_or("record id")? - 1) & 15)
                && a["first_decision"]["word_copy"]["source_end"] == previous["value"]["end"]
                && a["first_decision"]["word_copy"]["source_byte_end"]
                    == previous["value"]["byte_end"];
            selections += usize::from(ok);
            selected = Some(ok);
        } else {
            inherited += 1;
            preserved += usize::from(equal);
        }
        let control_equal = disabled.as_ref().map(|d| comparable(d) == comparable(&p));
        rows.push(json!({"id":c.id,"prompt":c.prompt,"expected":c.expected,"current_record":c.current_record,"correct":correct,"selected_previous":selected,"parent_equal":equal,"control_parent_equal":control_equal,"parent":p,"actual":a}));
    }
    save(
        &out.join("lineage.json"),
        &json!({"candidate":model.artifact_cid(),"parent":parent.artifact_cid(),"parent_bytes_blake3":blake3::hash(&parent.to_bytes()?).to_hex().to_string()}),
    )?;
    save(
        &out.join("result.json"),
        &json!({"artifact":model.artifact_cid(),"total":cases.len(),"targets":targets,"exact":exact,"selections":selections,"inherited":inherited,"preserved":preserved,"controls_parent_equal":rows.iter().all(|r|r["control_parent_equal"].as_bool().unwrap_or(true)),"rows":rows}),
    )
}
fn main() -> Result<()> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() == 4 && a[1] == "prepare" {
        return prepare(Some(Path::new(&a[2])), Path::new(&a[3]), false);
    }
    if a.len() == 3 && a[1] == "fresh" {
        return prepare(None, Path::new(&a[2]), true);
    }
    if a.len() != 5 {
        return Err("usage: native_historical_read prepare BASE OUT | fresh OUT | fit/evaluate MODEL DATA OUT".into());
    }
    let bytes = fs::read(&a[2])?;
    let model = Model::from_bytes(&bytes)?;
    if model.to_bytes()? != bytes {
        return Err("artifact roundtrip differs".into());
    }
    if a[1] == "fit" {
        fs::create_dir(&a[4])?;
        let docs: Vec<HistoricalReadExample> = serde_json::from_slice(&fs::read(&a[3])?)?;
        let config = SourceRoutingConfig {
            seed: 973,
            passes: 4,
            proposals: 32,
            max_seconds: 120,
            learned_features: 768,
            ..Default::default()
        };
        let (candidate, report) = model.fit_historical_read(&docs, config)?;
        fs::write(Path::new(&a[4]).join("model.json"), candidate.to_bytes()?)?;
        save(&Path::new(&a[4]).join("fit.json"), &report)?;
        return Ok(());
    }
    if a[1] == "evaluate" {
        let cases: Vec<Case> = serde_json::from_slice(&fs::read(&a[3])?)?;
        return evaluate(&model, &cases, Path::new(&a[4]));
    }
    Err("unknown mode".into())
}
