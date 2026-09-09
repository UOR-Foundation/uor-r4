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
fn group(owner: &str, old: &str, new: &str, i: usize, split: &str) -> Result<Vec<Case>> {
    let mut out = Vec::new();
    for (s, instruction) in [
        "Name the owner first.",
        "State the owner first.",
        "",
        "Explain in a sentence.",
        "Explain the stop in a sentence.",
    ]
    .into_iter()
    .enumerate()
    {
        let facts = format!("Record: {owner} in {old}. {owner} now in {new}.");
        out.push(labelled(
            format!("{split}/{i}/revision/{s}"),
            facts,
            owner,
            new,
            2,
            1,
            instruction,
            "",
        )?);
    }
    for (s, facts) in [
        format!("Record: now in {new}."),
        format!("Record: {owner} in Blanu {owner}. now in {new}."),
    ]
    .into_iter()
    .enumerate()
    {
        out.push(labelled(
            format!("{split}/{i}/literal-now/{s}"),
            facts,
            "now",
            new,
            1,
            0,
            "Name the owner first.",
            "",
        )?);
    }
    // Preserve actual assertions, contradictions and unrelated-owner commits;
    // no answer-quality claim is inferred from this parent comparison.
    for (s, facts) in [
        format!("Record: {owner} in {old}. {owner} in {new}."),
        format!("Record: {owner} in {old}. Record: tilva in {new}."),
    ]
    .into_iter()
    .enumerate()
    {
        out.push(Case {
            example: WriterChoiceExample {
                id: format!("{split}/{i}/inherit/{s}"),
                prompt: format!("{facts} Where is {owner}? Answer:"),
                overrides: vec![],
            },
            expected: None,
            target_record: None,
            inherit: true,
            source: "Matched parent assertion/conflict/distractor preservation".into(),
        });
    }
    Ok(out)
}
fn prepare(prior: &Path, out: &Path, fresh: bool) -> Result<()> {
    fs::create_dir(out)?;
    let groups: Vec<(String, String, String)> = if fresh {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        (0..4)
            .map(|i| {
                let h = blake3::hash(format!("{time}:{i}").as_bytes());
                let b = h.as_bytes();
                let word = |offset: usize, n: usize| {
                    (offset..offset + n)
                        .map(|j| (b'a' + b[j] % 26) as char)
                        .collect::<String>()
                };
                (
                    word(0, 5),
                    format!("{} {}", word(5, 4), word(9, 4)),
                    format!("{} {}", word(13, 4), word(17, 4)),
                )
            })
            .collect()
    } else {
        [
            "selvi", "serin", "navri", "pelvi", "pexul", "domvi", "raxen", "vespu",
        ]
        .into_iter()
        .enumerate()
        .map(|(i, o)| {
            (
                o.into(),
                if i < 4 { "Dusk Ridge" } else { "Blanu Tesh" }.into(),
                if i % 2 == 0 {
                    "Copper Vale"
                } else {
                    "Franu Vesh"
                }
                .into(),
            )
        })
        .collect()
    };
    let mut cases = Vec::new();
    for (i, (owner, old, new)) in groups.iter().enumerate() {
        cases.extend(group(
            owner,
            old,
            new,
            i,
            if fresh { "fresh" } else { "construction" },
        )?);
    }
    save(&out.join("new-cases.json"), &cases)?;
    if !fresh {
        let old: Vec<Value> = serde_json::from_slice(&fs::read(prior)?)?;
        for row in old {
            cases.push(Case {
                example: WriterChoiceExample {
                    id: format!("prior/{}", row["id"].as_str().ok_or("prior id")?),
                    prompt: row["prompt"].as_str().ok_or("prior prompt")?.into(),
                    overrides: vec![],
                },
                expected: None,
                target_record: None,
                inherit: true,
                source: prior.display().to_string(),
            });
        }
    }
    if cases.len() > 1200 {
        return Err("case cap1200".into());
    }
    save(&out.join("cases.json"), &cases)?;
    save(
        &out.join("training.json"),
        &cases.iter().map(|c| &c.example).collect::<Vec<_>>(),
    )?;
    save(
        &out.join("receipt.json"),
        &json!({"groups":groups,"cases":cases.len(),"fresh_draw":fresh,"source":prior,"forms":"Authored revision, literal-now and inherited assertion/conflict/distractor controls; labels offline only"}),
    )?;
    Ok(())
}
fn stress(out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let mut cases = Vec::new();
    for (i, (facts, owner, value, action, previous)) in [
        (
            "Record: selvi in Dusk Ridge. selvi now in Copper Vale.",
            "selvi",
            "Copper Vale",
            2,
            1,
        ),
        (
            "Record: Dusk Ridge holds selvi. selvi now in Copper Vale.",
            "selvi",
            "Copper Vale",
            2,
            1,
        ),
        (
            "Record: selvi in Rome. selvi now in Paris.",
            "selvi",
            "Paris",
            2,
            1,
        ),
        (
            "Record: selvi in Dusk Ridge. Record: tilva in Oak Bay. selvi now in Copper Vale.",
            "selvi",
            "Copper Vale",
            2,
            1,
        ),
        (
            "Record: selvi in Dusk Ridge. selvi now in Copper Vale. selvi now in Oak Bay.",
            "selvi",
            "Oak Bay",
            2,
            2,
        ),
        (
            "Record: selvi in Dusk selvi. now in Copper Vale.",
            "now",
            "Copper Vale",
            1,
            0,
        ),
        ("Record: now in Copper Vale.", "now", "Copper Vale", 1, 0),
        (
            "Record: selvi in Copper Vale. selvi now in Copper Vale.",
            "selvi",
            "Copper Vale",
            2,
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        for (j, padding) in [String::new(), "oak ash elm ".repeat(40)]
            .into_iter()
            .enumerate()
        {
            cases.push(labelled(
                format!("stress/{i}/{j}"),
                facts.into(),
                owner,
                value,
                action,
                previous,
                "Name the owner first.",
                &padding,
            )?);
        }
    }
    save(&out.join("cases.json"), &cases)
}
fn retained(input: &Path, out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let old: Vec<Value> = serde_json::from_slice(&fs::read(input)?)?;
    let cases: Vec<Case> = old
        .iter()
        .map(|c| {
            Ok(Case {
                example: WriterChoiceExample {
                    id: c["example"]["id"].as_str().ok_or("retained id")?.into(),
                    prompt: c["example"]["prompt"]
                        .as_str()
                        .ok_or("retained prompt")?
                        .into(),
                    overrides: vec![],
                },
                expected: c["expected"].as_str().map(String::from),
                target_record: None,
                inherit: true,
                source: input.display().to_string(),
            })
        })
        .collect::<Result<_>>()?;
    save(&out.join("cases.json"), &cases)
}
fn augment(prior: &Path, stress: &Path, out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let mut cases: Vec<Case> = serde_json::from_slice(&fs::read(prior)?)?;
    let more: Vec<Case> = serde_json::from_slice(&fs::read(stress)?)?;
    for mut c in more {
        // These open writer failures are now construction; the independently
        // observed reverse-read failure is deliberately excluded from fitting.
        if !(c.example.id.starts_with("stress/4/") || c.example.id.starts_with("stress/5/")) {
            continue;
        }
        if c.example.id.starts_with("stress/4/") {
            let phrase = "selvi now in Copper";
            let start = c
                .example
                .prompt
                .find(phrase)
                .ok_or("first revision absent")?;
            c.example.overrides.push(WriterChoiceOverride {
                at_end_byte: (start + phrase.len() - 1) as u64,
                target: WriterChoiceTarget::Write {
                    owner_end_byte: (start + "selvi".len() - 1) as u64,
                    value_end_byte: (start + phrase.len() - 1) as u64,
                    action: 2,
                },
            });
        }
        c.source =
            "Open stress writer failure added to construction before final selection; not held out"
                .into();
        cases.push(c);
    }
    save(&out.join("cases.json"), &cases)?;
    save(
        &out.join("training.json"),
        &cases.iter().map(|c| &c.example).collect::<Vec<_>>(),
    )
}
fn preserve(prior: &Path, reports: &[&Path], out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let mut cases: Vec<Case> = serde_json::from_slice(&fs::read(prior)?)?;
    for report in reports {
        let old: Value = serde_json::from_slice(&fs::read(report)?)?;
        for row in old["cases"].as_array().ok_or("preservation cases absent")? {
            cases.push(Case {
                example: WriterChoiceExample {
                    id: format!("preserve/{}", row["id"].as_str().ok_or("preservation id")?),
                    prompt: row["prompt"].as_str().ok_or("preservation prompt")?.into(),
                    overrides: vec![],
                },
                expected: Some(
                    row["expected"]
                        .as_str()
                        .ok_or("preservation expected")?
                        .into(),
                ),
                target_record: None,
                inherit: true,
                source: report.display().to_string(),
            });
        }
    }
    save(&out.join("cases.json"), &cases)?;
    save(
        &out.join("training.json"),
        &cases.iter().map(|c| &c.example).collect::<Vec<_>>(),
    )
}
fn generate(m: &Model, prompt: &str, control: Control, checkpoints: bool) -> Result<Value> {
    let mut s = m.session(control)?;
    s.observe(m, BOS)?;
    for t in m.encode(prompt)? {
        s.observe(m, t)?;
    }
    s.begin_response(m)?;
    let initial: Value = serde_json::from_slice(&s.checkpoint()?)?;
    let mut tokens = Vec::new();
    let mut eos = false;
    let mut positions = 0;
    for _ in 0..96 {
        let mut restored = if checkpoints {
            Some(m.restore_session(&s.checkpoint()?)?)
        } else {
            None
        };
        let p = s.predict(m)?;
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
        json!({"text":String::from_utf8(m.decode(&tokens)?)?,"eos":eos,"initial_relations":initial["values"]["relations"],"final_relations":final_state["values"]["relations"],"checkpoint_positions":positions,"work":s.work.values.relations}),
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
    let parent = m.without_writer_choice()?;
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
                records_match(&actual["initial_relations"], t)
                    .map(|ok| ok && actual["initial_relations"] == actual["final_relations"])
            })
            .transpose()?;
        if let Some(ok) = record_correct {
            record_targets += 1;
            records += usize::from(ok)
        }
        let same = actual["text"] == prior["text"]
            && actual["eos"] == prior["eos"]
            && actual["initial_relations"] == prior["initial_relations"]
            && actual["final_relations"] == prior["final_relations"];
        if c.inherit {
            inherited += 1;
            preserved += usize::from(same)
        }
        let mut interventions = Vec::new();
        if controls && !c.inherit {
            for control in [
                Control::WriterChoiceDisabled,
                Control::WriterChoiceBoundaryDisabled,
            ] {
                interventions.push(json!({"control":control,"result":generate(m,&c.example.prompt,control,false)?}));
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
fn main() -> Result<()> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() == 6 && a[1] == "preserve" {
        return preserve(
            Path::new(&a[2]),
            &[Path::new(&a[3]), Path::new(&a[4])],
            Path::new(&a[5]),
        );
    }
    if a.len() == 5 && a[1] == "augment" {
        return augment(Path::new(&a[2]), Path::new(&a[3]), Path::new(&a[4]));
    }
    if a.len() == 3 && a[1] == "stress" {
        return stress(Path::new(&a[2]));
    }
    if a.len() == 4 && a[1] == "retained" {
        return retained(Path::new(&a[2]), Path::new(&a[3]));
    }
    if a.len() == 4 && a[1] == "prepare" {
        return prepare(Path::new(&a[2]), Path::new(&a[3]), false);
    }
    if a.len() == 3 && a[1] == "fresh" {
        return prepare(Path::new("post-selection draw"), Path::new(&a[2]), true);
    }
    if a.len() != 5 && !(a.len() > 5 && a[1] == "trace") {
        return Err("usage: native_owner_revision prepare PRIOR OUT | fresh OUT | fit/evaluate/controls/trace MODEL INPUT OUT".into());
    }
    let model_bytes = fs::read(&a[2])?;
    let m = Model::from_bytes(&model_bytes)?;
    if m.to_bytes()? != model_bytes {
        return Err("loaded artifact did not roundtrip byte-exactly".into());
    }
    let out = Path::new(&a[4]);
    fs::create_dir_all(out)?;
    save(
        &out.join("model-roundtrip.json"),
        &json!({"artifact":m.artifact_cid(),"byte_exact":true,"bytes":model_bytes.len(),"blake3":blake3::hash(&model_bytes).to_hex().to_string()}),
    )?;
    match a[1].as_str() {
        "fit" => {
            let docs: Vec<WriterChoiceExample> = serde_json::from_slice(&fs::read(&a[3])?)?;
            let (candidate, report) = m.fit_writer_choice(&docs, 32, 120)?;
            fs::write(out.join("model.json"), candidate.to_bytes()?)?;
            save(&out.join("fit.json"), &report)?;
            println!("{report}");
            Ok(())
        }
        "source-trace" => {
            let prompts: Vec<Case> = serde_json::from_slice(&fs::read(&a[3])?)?;
            save(
                &out.join("source-traces.json"),
                &prompts
                    .iter()
                    .map(|p| m.source_routing_trace(&p.example.prompt))
                    .collect::<std::result::Result<Vec<_>, _>>()?,
            )
        }
        "trace" => {
            let prompts: Vec<Case> = serde_json::from_slice(&fs::read(&a[3])?)?;
            save(
                &out.join("traces.json"),
                &prompts
                    .iter()
                    .filter(|p| a.len() == 5 || a[5..].contains(&p.example.id))
                    .map(|p| { if p.example.prompt.len() > 256 { Ok(json!({"id":p.example.id,"status":"NOT_RUN_LONG_PROMPT_TRACE_CAP","reason":"Bounded writer trace is limited to128completed-word rows; full free-generation stress was run separately"})) } else { m.relation_writer_trace(&p.example.prompt).map(|trace| json!({"id":p.example.id,"trace":trace})) } })
                    .collect::<std::result::Result<Vec<_>, _>>()?,
            )
        }
        _ => {
            let cases: Vec<Case> = serde_json::from_slice(&fs::read(&a[3])?)?;
            evaluate(&m, &cases, out, a[1] == "controls")
        }
    }
}
