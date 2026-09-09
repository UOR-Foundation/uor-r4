//! Offline exact writer labels and independently checked free generation.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{
    Control, Model, WriterChoiceExample, WriterChoiceOverride, WriterChoiceTarget, BOS, EOS,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Serialize, Deserialize)]
struct Case {
    example: WriterChoiceExample,
    expected: Option<String>,
    target_record: Option<TargetRecord>,
    inherit: bool,
    source: String,
}
#[derive(Serialize, Deserialize)]
struct TargetRecord {
    records: usize,
    owner: String,
    value: String,
    action: u8,
    previous: u64,
}
fn save(p: &Path, v: &impl Serialize) -> Result<()> {
    fs::write(p, serde_json::to_vec(v)?)?;
    Ok(())
}
fn labelled(
    id: String,
    facts: String,
    owner: &str,
    value: &str,
    action: u8,
    previous: u64,
    instruction: &str,
    padding: &str,
) -> Result<Case> {
    let phrase = if action == 2 {
        format!("{owner} now in {value}")
    } else {
        format!("{owner} in {value}")
    };
    let start = facts.rfind(&phrase).ok_or("authored owner phrase absent")?;
    let first = value.split_ascii_whitespace().next().ok_or("empty value")?;
    let owner_end = (start + owner.len() - 1) as u64;
    let value_start = facts.rfind(value).ok_or("authored value absent")?;
    let value_end = (value_start + first.len() - 1) as u64;
    let suffix = if instruction.contains("owner first") {
        format!(" {owner} is in {value}.\n")
    } else if instruction == "Explain in a sentence." {
        format!(" {value} is the place.\n")
    } else if instruction == "Explain the stop in a sentence." {
        format!(" {value} is the stop.\n")
    } else {
        format!(" {value}.\n")
    };
    Ok(Case {
        example: WriterChoiceExample {
            id,
            prompt: format!("{facts} {padding}Where is {owner}? {instruction} Answer:"),
            overrides: vec![WriterChoiceOverride {
                at_end_byte: value_end,
                target: WriterChoiceTarget::Write {
                    owner_end_byte: owner_end,
                    value_end_byte: value_end,
                    action,
                },
            }],
        },
        expected: Some(suffix),
        target_record: Some(TargetRecord {
            owner: owner.into(),
            value: value.into(),
            action,
            previous,
            records: facts.matches(" in ").count() + facts.matches(" holds ").count(),
        }),
        inherit: false,
        source: "Authored exact endpoint/action labels offline; generation receives prompt only"
            .into(),
    })
}
fn generate(m: &Model, prompt: &str, control: Control, checkpoints: bool) -> Result<Value> {
    let input = m.encode(prompt)?;
    if prompt.len() > 65536
        || input.len() > 8192
        || (checkpoints && (prompt.len() > 4096 || input.len() > 512))
    {
        return Err("configured input bound".into());
    }
    let mut s = m.session(control)?;
    s.observe(m, BOS)?;
    for t in input {
        s.observe(m, t)?;
    }
    s.begin_response(m)?;
    let initial: Value = serde_json::from_slice(&s.checkpoint()?)?;
    let mut tokens = Vec::new();
    let mut eos = false;
    let mut positions = 0;
    let mut first = Value::Null;
    for _ in 0..96 {
        let mut restored = if checkpoints {
            Some(m.restore_session(&s.checkpoint()?)?)
        } else {
            None
        };
        let p = s.predict(m)?;
        if tokens.is_empty() {
            first =
                json!({"word_copy":s.word_copy_decision(),"field":s.field_composition_decision()});
        }
        if checkpoints && s.predict(m)? != p {
            return Err("repeated prediction differs".into());
        }
        if let Some(r) = &mut restored {
            if r.predict(m)? != p {
                return Err("restored prediction differs".into());
            }
        }
        s.observe(m, p.token)?;
        if let Some(r) = &mut restored {
            r.observe(m, p.token)?;
            let a: Value = serde_json::from_slice(&s.checkpoint()?)?;
            let b: Value = serde_json::from_slice(&r.checkpoint()?)?;
            for k in [
                "values",
                "word_copy",
                "field_composition",
                "response_entry",
                "completion",
            ] {
                if a[k] != b[k] {
                    return Err(format!("restored {k} differs").into());
                }
            }
            positions += 1;
        }
        if p.token == EOS {
            eos = true;
            break;
        }
        tokens.push(p.token);
    }
    let final_state: Value = serde_json::from_slice(&s.checkpoint()?)?;
    Ok(
        json!({"text":String::from_utf8(m.decode(&tokens)?)?,"eos":eos,"initial_relations":initial["values"]["relations"],"final_relations":final_state["values"]["relations"],"checkpoint_positions":positions,"first_decision":first,"work":s.work.values.relations}),
    )
}
fn atom_text(v: &Value) -> Result<String> {
    let len = v["len"].as_u64().ok_or("atom len")? as usize;
    let bytes = v["bytes"].as_array().ok_or("atom bytes")?;
    Ok(String::from_utf8(
        bytes
            .iter()
            .take(len)
            .map(|v| v.as_u64().ok_or("byte").map(|n| n as u8))
            .collect::<std::result::Result<Vec<_>, _>>()?,
    )?)
}
fn records_match(r: &Value, t: &TargetRecord) -> Result<bool> {
    let records = r["records"].as_array().ok_or("records")?;
    let directory = r["directory"].as_array().ok_or("directory")?;
    if records.iter().filter(|r| r["id"] != 0).count() != t.records {
        return Ok(false);
    }
    for record in records {
        if record["id"] == 0 {
            continue;
        }
        let value = if record["span"].is_object() {
            atom_text(&record["span"])?
        } else {
            atom_text(&record["value"])?
        };
        if atom_text(&record["owner"])? == t.owner
            && value == t.value
            && record["action"] == t.action
            && record["previous"] == t.previous
            && record["conflict"] == false
            && directory.contains(&record["id"])
            && (t.previous == 0
                || (!directory.contains(&json!(t.previous))
                    && records.iter().any(|old| {
                        old["id"] == t.previous
                            && atom_text(&old["owner"]).ok().as_deref() == Some(t.owner.as_str())
                    })))
        {
            return Ok(true);
        }
    }
    Ok(false)
}
fn evaluate(m: &Model, cases: &[Case], out: &Path, controls: bool) -> Result<()> {
    fs::create_dir_all(out)?;
    let parent = m.without_writer_role()?;
    save(
        &out.join("lineage.json"),
        &json!({"candidate":m.artifact_cid(),"parent":parent.artifact_cid(),"parent_bytes_blake3":blake3::hash(&parent.to_bytes()?).to_hex().to_string()}),
    )?;
    let mut rows = Vec::new();
    let (mut exact, mut labelled, mut records, mut record_targets, mut inherited, mut preserved) =
        (0, 0, 0, 0, 0, 0);
    for c in cases {
        let actual = generate(m, &c.example.prompt, Control::Full, !c.inherit)?;
        let prior = generate(&parent, &c.example.prompt, Control::Full, false)?;
        let correct = c
            .expected
            .as_ref()
            .map(|e| actual["text"] == *e && actual["eos"] == true);
        if let Some(ok) = correct {
            labelled += 1;
            exact += usize::from(ok)
        }
        let record_correct = c
            .target_record
            .as_ref()
            .map(|t| {
                records_match(&actual["initial_relations"], t).map(|ok| {
                    ok && actual["initial_relations"] == actual["final_relations"]
                        && (!(c.example.id.starts_with("construction/")
                            || c.example.id.starts_with("fresh/"))
                            || (actual["initial_relations"]["directory"]
                                .as_array()
                                .is_some_and(|ds| {
                                    ds.iter().filter(|v| **v != 0).collect::<Vec<_>>()
                                        == vec![&json!(3)]
                                })
                                && actual["initial_relations"]["records"]
                                    .as_array()
                                    .is_some_and(|rs| {
                                        (1..=3).all(|id| {
                                            rs.iter().any(|r| {
                                                r["id"] == id
                                                    && r["action"] == if id == 1 { 1 } else { 2 }
                                                    && r["previous"] == id - 1
                                                    && r["conflict"] == false
                                                    && atom_text(&r["owner"]).ok().as_deref()
                                                        == Some(t.owner.as_str())
                                            })
                                        })
                                    })))
                })
            })
            .transpose()?;
        if let Some(ok) = record_correct {
            record_targets += 1;
            records += usize::from(ok)
        }
        let same = actual["text"] == prior["text"]
            && actual["eos"] == prior["eos"]
            && actual["initial_relations"] == prior["initial_relations"]
            && actual["final_relations"] == prior["final_relations"]
            && first_identity(&actual) == first_identity(&prior);
        if c.inherit {
            inherited += 1;
            preserved += usize::from(same)
        }
        let mut interventions = Vec::new();
        if controls && !c.inherit {
            for control in [
                Control::WriterRoleDisabled,
                Control::WriterRoleContextDisabled,
            ] {
                let result = generate(m, &c.example.prompt, control, false)?;
                let parent_equal = result["text"] == prior["text"]
                    && result["eos"] == prior["eos"]
                    && result["initial_relations"] == prior["initial_relations"]
                    && result["final_relations"] == prior["final_relations"]
                    && first_identity(&result) == first_identity(&prior);
                interventions
                    .push(json!({"control":control,"result":result,"parent_equal":parent_equal}));
            }
        }
        rows.push(json!({"id":c.example.id,"prompt":c.example.prompt,"expected":c.expected,"target_record":c.target_record,"inherit":c.inherit,"actual":actual,"parent":prior,"correct":correct,"record_correct":record_correct,"parent_equal":same,"controls":interventions}));
    }
    save(
        &out.join("result.json"),
        &json!({"artifact":m.artifact_cid(),"total":cases.len(),"labelled":labelled,"exact":exact,"record_targets":record_targets,"records_correct":records,"inherited":inherited,"preserved":preserved,"rows":rows,"scope":"Actual committed records and free generation; inherited comparisons are not independent language correctness"}),
    )?;
    println!(
        "{}",
        json!({"total":cases.len(),"labelled":labelled,"exact":exact,"records_correct":records,"record_targets":record_targets,"inherited":inherited,"preserved":preserved})
    );
    Ok(())
}

fn first_identity(value: &Value) -> Value {
    let mut v = value["first_decision"].clone();
    for k in ["word_copy", "field"] {
        if let Some(o) = v[k].as_object_mut() {
            o.remove("score");
        }
    }
    v
}
fn diagnose(model: &Model, out: &Path) -> Result<()> {
    let mut rows = Vec::new();
    for (i, value) in ["Amber Field", "Copper Vale", "sovek Field"]
        .into_iter()
        .enumerate()
    {
        for chain in [false, true] {
            let prefix = if chain {
                "Record: Dusk Ridge holds selvi. selvi now in Copper Vale."
            } else {
                "Record: Dusk Ridge holds selvi."
            };
            let prompt = format!(
                "{prefix} selvi now in {value}. Where is selvi? Name the owner first. Answer:"
            );
            rows.push(json!({"id":format!("diagnostic/{i}/{chain}"),"prompt":prompt,"trace":model.relation_writer_trace(&prompt)?,"actual":generate(model,&prompt,Control::Full,false)?}));
        }
    }
    save(
        &out.join("result.json"),
        &json!({"artifact":model.artifact_cid(),"rows":rows,"scope":"Actual observation/proposal traces and free generation; open diagnostic, no fitting or transfer qualification."}),
    )
}
fn write_cases(out: &Path, cases: &[Case]) -> Result<()> {
    fs::create_dir(out)?;
    save(&out.join("cases.json"), &cases)?;
    save(
        &out.join("training.json"),
        &cases.iter().map(|c| &c.example).collect::<Vec<_>>(),
    )
}
fn prepare(prior: &Path, out: &Path) -> Result<()> {
    let mut cases: Vec<Case> = serde_json::from_slice(&fs::read(prior)?)?;
    for c in &mut cases {
        c.example.id = format!("retained/{}", c.example.id);
        c.example.overrides.clear();
        c.inherit = true;
    }
    for (i, owner) in ["selvi", "tilva", "zanvo"].into_iter().enumerate() {
        for (j, value) in ["Amber Field", "Birch Grove"].into_iter().enumerate() {
            for (k, instruction) in ["", "Name the owner first.", "State the owner first."]
                .into_iter()
                .enumerate()
            {
                for evicted in [false, true] {
                    let facts=format!("Record: Dusk Ridge holds {owner}. {owner} now in Copper Vale. {owner} now in {value}.");
                    let padding = if evicted {
                        "oak ash elm ".repeat(12)
                    } else {
                        String::new()
                    };
                    cases.push(labelled(
                        format!("construction/{i}/{j}/{k}/{evicted}"),
                        facts,
                        owner,
                        value,
                        2,
                        2,
                        instruction,
                        &padding,
                    )?);
                }
            }
        }
    }
    write_cases(out, &cases)?;
    save(
        &out.join("receipt.json"),
        &json!({"prior":prior,"prior_bytes_blake3":blake3::hash(&fs::read(prior)?).to_hex().to_string(),"cases":cases.len(),"new_targets":36,"scope":"Old completed-word choices preserved; only final exact owner/value Revise proposals overridden. Three owners crossed with two values and three response forms before/after eviction. Labels remain offline."}),
    )
}
fn stress(out: &Path) -> Result<()> {
    let mut cases = Vec::new();
    for (i, (facts, owner, value, action, previous)) in [
        (
            "Record: Dusk Ridge holds selvi. selvi now in Copper Vale. selvi now in Mist Bay.",
            "selvi",
            "Mist Bay",
            2,
            2,
        ),
        (
            "Record: tilva in Dusk Ridge. tilva now in Dover Court.",
            "tilva",
            "Dover Court",
            2,
            1,
        ),
        ("Record: now in Amber Field.", "now", "Amber Field", 1, 0),
        (
            "Record: selvi in Dusk Ridge. Now selvi in Amber Field.",
            "selvi",
            "Amber Field",
            2,
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        for evicted in [false, true] {
            for (j, instruction) in ["", "Name the owner first."].into_iter().enumerate() {
                let padding = if evicted {
                    "oak ash elm ".repeat(12)
                } else {
                    String::new()
                };
                // This stress population is never fitted. The leading-Now form
                // deliberately differs from the construction phrase helper.
                let prompt = format!("{facts} {padding}Where is {owner}? {instruction} Answer:");
                cases.push(Case {
                    example: WriterChoiceExample {
                        id: format!("stress/{i}/{j}/{evicted}"),
                        prompt,
                        overrides: vec![],
                    },
                    expected: Some(if j == 0 {
                        format!(" {value}.\n")
                    } else {
                        format!(" {owner} is in {value}.\n")
                    }),
                    target_record: Some(TargetRecord {
                        records: facts.matches(" in ").count() + facts.matches(" holds ").count(),
                        owner: owner.into(),
                        value: value.into(),
                        action,
                        previous,
                    }),
                    inherit: false,
                    source: "Open structural stress; no fitting".into(),
                });
            }
        }
    }
    for (i, q) in [
        "Where was selvi before? Answer:",
        "Copy Dusk Ridge. Answer:",
        "Where is unknown? Answer:",
    ]
    .into_iter()
    .enumerate()
    {
        cases.push(Case {
            example: WriterChoiceExample {
                id: format!("stress/control/{i}"),
                prompt: format!("Record: selvi in Dusk Ridge. selvi now in Copper Vale. {q}"),
                overrides: vec![],
            },
            expected: None,
            target_record: None,
            inherit: true,
            source: "Preserve parent historical/raw/absent behavior; not correctness".into(),
        });
    }
    write_cases(out, &cases)
}
fn fresh(model: &Model, construction: &Path, out: &Path) -> Result<()> {
    let wire: Value = serde_json::from_slice(&model.to_bytes()?)?;
    let dict = wire["writer_lexical"]["dictionary"]
        .as_array()
        .ok_or("dictionary absent")?;
    let pool: [(&str, &str); 3] = [("Ash", "Court"), ("Cedar", "Gate"), ("Cloud", "Cove")];
    let mut identities = Vec::new();
    for (a, b) in pool {
        let d = dict
            .iter()
            .find(|d| atom_text(d).ok().as_deref() == Some(a))
            .ok_or("fresh known value is not in inherited dictionary")?;
        identities.push(json!({"word":a,"prime":d["prime"],"tail":b}));
    }
    let prior: Vec<Case> = serde_json::from_slice(&fs::read(construction)?)?;
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let mut cases = Vec::new();
    let mut drawn = Vec::new();
    for i in 0..3 {
        let hash = blake3::hash(format!("writer-role-fresh/{time}/{i}").as_bytes());
        let word = |start: usize| {
            (start..start + 5)
                .map(|j| (b'a' + hash.as_bytes()[j] % 26) as char)
                .collect::<String>()
        };
        let owner = word(0);
        let unknown = word(6);
        if dict.iter().any(|d| {
            atom_text(d).ok().as_deref() == Some(owner.as_str())
                || atom_text(d).ok().as_deref() == Some(unknown.as_str())
        }) {
            return Err("fresh draw unexpectedly known".into());
        }
        let value = format!("{} {}", pool[i].0, pool[i].1);
        drawn.push(json!({"owner":owner,"known_value":value,"unknown_value":unknown}));
        for (j, value) in [value, format!("{unknown} Field")].into_iter().enumerate() {
            for (k, instruction) in ["", "Name the owner first."].into_iter().enumerate() {
                for evicted in [false, true] {
                    let facts=format!("Record: Dusk Ridge holds {owner}. {owner} now in Copper Vale. {owner} now in {value}.");
                    let padding = if evicted {
                        "oak ash elm ".repeat(12)
                    } else {
                        String::new()
                    };
                    let c = labelled(
                        format!("fresh/{i}/{j}/{k}/{evicted}"),
                        facts,
                        &owner,
                        &value,
                        2,
                        2,
                        instruction,
                        &padding,
                    )?;
                    if prior.iter().any(|p| p.example.prompt == c.example.prompt) {
                        return Err("fresh prompt present in construction".into());
                    }
                    cases.push(c);
                }
            }
        }
    }
    write_cases(out, &cases)?;
    save(
        &out.join("receipt.json"),
        &json!({"draw_unix_ns":time,"artifact":model.artifact_cid(),"known_value_primes":identities,"drawn":drawn,"cases":cases.len(),"scope":"Post-selection new whole prompts/owner spellings with both inherited nonzero value primes and unseen values. Familiar forms; no general language claim; do not refit."}),
    )
}
fn main() -> Result<()> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() == 4 && a[1] == "source-cases" {
        let old: Vec<Value> = serde_json::from_slice(&fs::read(&a[2])?)?;
        let cases = old
            .iter()
            .map(|r| {
                Ok(Case {
                    example: WriterChoiceExample {
                        id: format!("source-probe/{}", r["id"].as_str().ok_or("source case id")?),
                        prompt: r["prompt"].as_str().ok_or("source prompt")?.into(),
                        overrides: vec![],
                    },
                    expected: r["expected"].as_str().map(String::from),
                    target_record: None,
                    inherit: r["expected"].is_null(),
                    source: "Previously opened source panel, evaluated only; no new writer fit"
                        .into(),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        return write_cases(Path::new(&a[3]), &cases);
    }
    if a.len() == 4 && a[1] == "prepare" {
        return prepare(Path::new(&a[2]), Path::new(&a[3]));
    }
    if a.len() == 3 && a[1] == "stress" {
        return stress(Path::new(&a[2]));
    }
    if a.len() != 4 && a.len() != 5 {
        return Err("usage: native_writer_role_transfer diagnose MODEL OUT | prepare PRIOR OUT | fit/evaluate/controls/trace MODEL INPUT OUT | fresh MODEL CONSTRUCTION OUT".into());
    }
    let bytes = fs::read(&a[2])?;
    let m = Model::from_bytes(&bytes)?;
    if m.to_bytes()? != bytes {
        return Err("artifact roundtrip differs".into());
    }
    if a[1] == "fresh" {
        return fresh(&m, Path::new(&a[3]), Path::new(&a[4]));
    }
    let out = Path::new(a.last().ok_or("output absent")?);
    fs::create_dir(out)?;
    match a[1].as_str() {
        "diagnose" => diagnose(&m, out),
        "fit" => {
            let docs: Vec<WriterChoiceExample> = serde_json::from_slice(&fs::read(&a[3])?)?;
            let (candidate, report) = m.fit_writer_role(&docs, 32, 180)?;
            fs::write(out.join("model.json"), candidate.to_bytes()?)?;
            save(&out.join("fit.json"), &report)
        }
        "trace" => {
            let cases: Vec<Case> = serde_json::from_slice(&fs::read(&a[3])?)?;
            save(
                &out.join("traces.json"),
                &cases
                    .iter()
                    .filter(|c| !c.inherit && c.example.prompt.len() < 256)
                    .take(16)
                    .map(|c| m.relation_writer_trace(&c.example.prompt))
                    .collect::<std::result::Result<Vec<_>, _>>()?,
            )
        }
        "evaluate" | "controls" => {
            let cases: Vec<Case> = serde_json::from_slice(&fs::read(&a[3])?)?;
            evaluate(&m, &cases, out, true)
        }
        _ => Err("unknown mode".into()),
    }
}
