//! Open read-dispatch contrasts; labels never enter inference.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{
    Control, CurrentSourceExample, CurrentSourceTarget, Model, SourceRoutingConfig, BOS, EOS,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Serialize, Deserialize)]
struct Case {
    id: String,
    prompt: String,
    expected: Option<String>,
    #[serde(default)]
    target_current: bool,
    #[serde(default)]
    inherit: bool,
}
fn save(path: &Path, value: &impl Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec(value)?)?;
    Ok(())
}
fn prepare(out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let mut cases = Vec::new();
    for (i, (owner, old, new)) in [
        ("selvi", "Dusk Ridge", "Copper Vale"),
        ("tilva", "Silver Cove", "Birch Grove"),
    ]
    .into_iter()
    .enumerate()
    {
        for (kind, facts, value) in [
            (
                "revision-forward",
                format!("Record: {owner} in {old}. {owner} now in {new}."),
                Some(new),
            ),
            (
                "revision-reverse",
                format!("Record: {old} holds {owner}. {owner} now in {new}."),
                Some(new),
            ),
            (
                "reverse-only",
                format!("Record: {old} holds {owner}."),
                Some(old),
            ),
            (
                "unrelated",
                format!("Record: {old} holds {owner}. merli in {new}."),
                Some(old),
            ),
            (
                "conflict",
                format!("Record: {owner} in {old}. {owner} in {new}."),
                None,
            ),
            (
                "equal-value",
                format!("Record: {owner} in {new}. merli in {new}."),
                Some(new),
            ),
        ] {
            for (evicted, padding) in [false, true].into_iter().map(|e| {
                (
                    e,
                    if e {
                        "oak ash elm ".repeat(12)
                    } else {
                        String::new()
                    },
                )
            }) {
                for (form, query, expected) in [
                    (
                        "plain",
                        format!("Where is {owner}? Answer:"),
                        value.map(|v| format!(" {v}.\n")),
                    ),
                    (
                        "owner",
                        format!("Where is {owner}? Name the owner first. Answer:"),
                        value.map(|v| format!(" {owner} is in {v}.\n")),
                    ),
                ] {
                    cases.push(Case {
                        id: format!("open/{i}/{kind}/{form}/{evicted}"),
                        prompt: format!("{facts} {padding}{query}"),
                        expected,
                        target_current: false,
                        inherit: false,
                    });
                }
            }
        }
        let facts = format!("Record: {old} holds {owner}. {owner} now in {new}.");
        for (j, query) in [
            format!("Where was {owner} before? Answer:"),
            format!("What was the previous location of {owner}? Answer:"),
            format!("Repeat the first location stated for {owner}. Answer:"),
            format!("Copy {old}. Answer:"),
            format!("Where is unknown? Answer:"),
        ]
        .into_iter()
        .enumerate()
        {
            cases.push(Case {
                id: format!("open/{i}/history-raw-absent/{j}"),
                prompt: format!("{facts} {query}"),
                expected: None,
                target_current: false,
                inherit: false,
            });
        }
    }
    save(&out.join("cases.json"), &cases)
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
fn training(prior: &Path, open: &Path, out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let mut cases = Vec::new();
    for row in serde_json::from_slice::<Vec<Value>>(&fs::read(prior)?)? {
        cases.push(Case {
            id: format!(
                "retained/{}",
                row["example"]["id"].as_str().ok_or("prior id")?
            ),
            prompt: row["example"]["prompt"]
                .as_str()
                .ok_or("prior prompt")?
                .into(),
            expected: None,
            target_current: false,
            inherit: true,
        });
    }
    for mut c in serde_json::from_slice::<Vec<Case>>(&fs::read(open)?)? {
        c.target_current = c.id.contains("/revision-reverse/") && c.id.ends_with("/false");
        c.inherit = !c.target_current;
        cases.push(c);
    }
    for (i, (owner, old, new)) in [
        ("selvi", "Dusk Ridge", "Copper Vale"),
        ("tilva", "Silver Cove", "Birch Grove"),
    ]
    .into_iter()
    .enumerate()
    {
        cases.push(Case{id:format!("construction/state/{i}"),prompt:format!("Record: {old} holds {owner}. {owner} now in {new}. Where is {owner}? State the owner first. Answer:"),expected:Some(format!(" {owner} is in {new}.\n")),target_current:true,inherit:false});
    }
    let docs: Vec<_> = cases
        .iter()
        .map(|c| CurrentSourceExample {
            id: c.id.clone(),
            prompt: c.prompt.clone(),
            target: if c.target_current {
                CurrentSourceTarget::CurrentRevision
            } else {
                CurrentSourceTarget::Preserve
            },
        })
        .collect();
    save(&out.join("training.json"), &docs)?;
    save(&out.join("cases.json"), &cases)?;
    save(
        &out.join("receipt.json"),
        &json!({"prior":prior,"prior_blake3":blake3::hash(&fs::read(prior)?).to_hex().to_string(),"open":open,"open_blake3":blake3::hash(&fs::read(open)?).to_hex().to_string(),"documents":cases.len(),"current_revision_targets":cases.iter().filter(|c|c.target_current).count(),"scope":"Exact parent source/action preservation plus authored current-revision source labels. No output bytes enter fitting. Open historical controls remain preservation, not historical-language qualification."}),
    )
}
fn fresh(out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let mut cases = Vec::new();
    let mut drawn = Vec::new();
    for i in 0..3 {
        let h = blake3::hash(format!("current-source:{time}:{i}").as_bytes());
        let b = h.as_bytes();
        let word = |start: usize, len: usize| {
            (start..start + len)
                .map(|j| (b'a' + b[j] % 26) as char)
                .collect::<String>()
        };
        let owner = word(0, 5);
        let old = format!("{} {}", word(5, 4), word(9, 4));
        let new = format!("{} {}", word(13, 4), word(17, 4));
        drawn.push(json!({"owner":owner,"old":old,"new":new}));
        for (evicted, padding) in [(false, String::new()), (true, "oak ash elm ".repeat(12))] {
            for (form, instruction) in ["", "Name the owner first.", "State the owner first."]
                .into_iter()
                .enumerate()
            {
                let prompt=format!("Record: {old} holds {owner}. {owner} now in {new}. {padding}Where is {owner}? {instruction} Answer:");
                cases.push(Case {
                    id: format!("fresh/{i}/{form}/{evicted}"),
                    prompt,
                    expected: Some(if form == 0 {
                        format!(" {new}.\n")
                    } else {
                        format!(" {owner} is in {new}.\n")
                    }),
                    target_current: !evicted,
                    inherit: evicted,
                });
            }
        }
        for (j, q) in [
            format!("Where was {owner} before? Answer:"),
            format!("Copy {old}. Answer:"),
        ]
        .into_iter()
        .enumerate()
        {
            cases.push(Case {
                id: format!("fresh/{i}/history-raw/{j}"),
                prompt: format!("Record: {old} holds {owner}. {owner} now in {new}. {q}"),
                expected: None,
                target_current: false,
                inherit: true,
            });
        }
    }
    save(&out.join("cases.json"), &cases)?;
    save(
        &out.join("receipt.json"),
        &json!({"draw_unix_ns":time,"drawn":drawn,"cases":cases.len(),"scope":"New complete spellings in fixed forms, generated only after selection. Historical/raw cases preserve parent; no refit."}),
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
fn stress(out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let mut cases = Vec::new();
    for (kind, facts, owner, value) in [
        ("chain", "Record: Dusk Ridge holds selvi. selvi now in Copper Vale. selvi now in Amber Field.", "selvi", "Amber Field"),
        ("two-owners-first", "Record: Dusk Ridge holds selvi. selvi now in Copper Vale. Silver Cove holds tilva. tilva now in Birch Grove.", "selvi", "Copper Vale"),
        ("two-owners-second", "Record: Dusk Ridge holds selvi. selvi now in Copper Vale. Silver Cove holds tilva. tilva now in Birch Grove.", "tilva", "Birch Grove"),
        ("unbound-equal-spelling", "Record: Dusk Ridge holds selvi. selvi now in Copper Vale. Words: Copper Vale.", "selvi", "Copper Vale"),
    ] {
        for evicted in [false, true] {
            let padding = if evicted { "oak ash elm ".repeat(12) } else { String::new() };
            for (form, instruction) in ["", "Name the owner first."].into_iter().enumerate() {
                cases.push(Case {
                    id: format!("stress/{kind}/{form}/{evicted}"),
                    prompt: format!("{facts} {padding}Where is {owner}? {instruction} Answer:"),
                    expected: Some(if form == 0 { format!(" {value}.\n") } else { format!(" {owner} is in {value}.\n") }),
                    target_current: false,
                    inherit: false,
                });
            }
        }
    }
    save(&out.join("cases.json"), &cases)
}
fn current_selected(v: &Value) -> bool {
    let state = &v["initial_relations"];
    let Some(records) = state["records"].as_array() else {
        return false;
    };
    let Some(directory) = state["directory"].as_array() else {
        return false;
    };
    records.iter().any(|r| {
        r["id"] != 0
            && directory.contains(&r["id"])
            && r["action"] == 2
            && r["previous"] != 0
            && r["conflict"] == false
            && (v["first_decision"]["field"]["anchor"]["relation_id"] == r["id"]
                || (v["first_decision"]["word_copy"]["source_end"] == r["value"]["end"]
                    && v["first_decision"]["word_copy"]["source_byte_end"]
                        == r["value"]["byte_end"]))
    })
}
fn evaluate(model: &Model, cases: &[Case], out: &Path) -> Result<()> {
    let parent = model.without_current_source()?;
    save(
        &out.join("lineage.json"),
        &json!({"candidate":model.artifact_cid(),"parent":parent.artifact_cid(),"parent_bytes_blake3":blake3::hash(&parent.to_bytes()?).to_hex().to_string()}),
    )?;
    let mut rows = Vec::new();
    let (
        mut labelled,
        mut exact,
        mut targets,
        mut target_exact,
        mut inherited,
        mut preserved,
        mut current_selections,
    ) = (0, 0, 0, 0, 0, 0, 0);
    for c in cases {
        let p = generate(&parent, &c.prompt, Control::Full, false, c.inherit)?;
        let actual = generate(model, &c.prompt, Control::Full, c.target_current, c.inherit)?;
        let equal = comparable(&actual) == comparable(&p);
        if actual["initial_relations"] != p["initial_relations"]
            || actual["final_relations"] != p["final_relations"]
        {
            return Err(format!("read refinement changed stored records: {}", c.id).into());
        }
        let selected_current = current_selected(&actual);
        let correct = c
            .expected
            .as_ref()
            .map(|e| actual["text"] == *e && actual["eos"] == true);
        if let Some(ok) = correct {
            labelled += 1;
            exact += usize::from(ok);
        }
        if c.target_current {
            targets += 1;
            target_exact += usize::from(correct == Some(true));
            current_selections += usize::from(selected_current);
        }
        if c.inherit {
            inherited += 1;
            preserved += usize::from(equal);
        }
        let mut controls = Vec::new();
        if c.target_current {
            for control in [
                Control::CurrentSourceDisabled,
                Control::CurrentSourceVersionDisabled,
            ] {
                let value = generate(model, &c.prompt, control, false, false)?;
                controls.push(json!({"control":control,"parent_equal":comparable(&value)==comparable(&p),"result":value}));
            }
        }
        rows.push(json!({"id":c.id,"prompt":c.prompt,"expected":c.expected,"target_current":c.target_current,"inherit":c.inherit,"parent":p,"actual":actual,"correct":correct,"selected_current_revision":selected_current,"parent_equal":equal,"controls":controls}));
    }
    let report = json!({"artifact":model.artifact_cid(),"total":cases.len(),"labelled":labelled,"exact":exact,"current_targets":targets,"current_exact":target_exact,"current_selections":current_selections,"inherited":inherited,"preserved":preserved,"rows":rows});
    save(&out.join("result.json"), &report)?;
    println!(
        "{}",
        json!({"total":cases.len(),"labelled":labelled,"exact":exact,"current_targets":targets,"current_exact":target_exact,"inherited":inherited,"preserved":preserved})
    );
    Ok(())
}
fn extend_preservation(base: &Path, writer: &Path, out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let mut cases: Vec<Case> = serde_json::from_slice(&fs::read(base)?)?;
    let prior: Vec<Value> = serde_json::from_slice(&fs::read(writer)?)?;
    for row in prior {
        cases.push(Case {
            id: format!(
                "writer-stress/{}",
                row["example"]["id"].as_str().ok_or("writer id")?
            ),
            prompt: row["example"]["prompt"]
                .as_str()
                .ok_or("writer prompt")?
                .into(),
            expected: row["expected"].as_str().map(String::from),
            target_current: false,
            inherit: true,
        });
    }
    if cases.len() > 1200 {
        return Err("extended population exceeds bound".into());
    }
    let docs: Vec<_> = cases
        .iter()
        .map(|c| CurrentSourceExample {
            id: c.id.clone(),
            prompt: c.prompt.clone(),
            target: if c.target_current {
                CurrentSourceTarget::CurrentRevision
            } else {
                CurrentSourceTarget::Preserve
            },
        })
        .collect();
    save(&out.join("cases.json"), &cases)?;
    save(&out.join("training.json"), &docs)?;
    save(
        &out.join("receipt.json"),
        &json!({"base":base,"base_blake3":blake3::hash(&fs::read(base)?).to_hex().to_string(),"writer":writer,"writer_blake3":blake3::hash(&fs::read(writer)?).to_hex().to_string(),"documents":cases.len(),"scope":"Retained writer stress added as exact parent source/action preservation; no answer bytes enter fitting."}),
    )
}

fn main() -> Result<()> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() == 5 && a[1] == "extend-preservation" {
        return extend_preservation(Path::new(&a[2]), Path::new(&a[3]), Path::new(&a[4]));
    }
    if a.len() == 3 && a[1] == "prepare" {
        return prepare(Path::new(&a[2]));
    }
    if a.len() == 3 && a[1] == "fresh" {
        return fresh(Path::new(&a[2]));
    }
    if a.len() == 3 && a[1] == "stress" {
        return stress(Path::new(&a[2]));
    }
    if a.len() == 5 && a[1] == "training" {
        return training(Path::new(&a[2]), Path::new(&a[3]), Path::new(&a[4]));
    }
    if a.len() != 5 {
        return Err(
            "usage: native_current_version_read prepare OUT | diagnose MODEL CASES OUT".into(),
        );
    }
    fs::create_dir(&a[4])?;
    let bytes = fs::read(&a[2])?;
    let model = Model::from_bytes(&bytes)?;
    if model.to_bytes()? != bytes {
        return Err("artifact roundtrip differs".into());
    }
    if a[1] == "fit" {
        let docs: Vec<CurrentSourceExample> = serde_json::from_slice(&fs::read(&a[3])?)?;
        let wire: Value = serde_json::from_slice(&bytes)?;
        let mut config: SourceRoutingConfig =
            serde_json::from_value(wire["source_routing"]["config"].clone())?;
        config.learned_features = 4096;
        config.passes = 4;
        config.proposals = 32;
        config.max_seconds = 120;
        config.seed = 973;
        let (candidate, report) = model.fit_current_source(&docs, config)?;
        fs::write(Path::new(&a[4]).join("model.json"), candidate.to_bytes()?)?;
        save(&Path::new(&a[4]).join("fit.json"), &report)?;
        println!("{report}");
        return Ok(());
    }
    let cases: Vec<Case> = serde_json::from_slice(&fs::read(&a[3])?)?;
    if a[1] == "evaluate" {
        return evaluate(&model, &cases, Path::new(&a[4]));
    }
    if a[1] == "trace" {
        return save(
            &Path::new(&a[4]).join("source-traces.json"),
            &cases
                .iter()
                .map(|c| model.source_routing_trace(&c.prompt))
                .collect::<std::result::Result<Vec<_>, _>>()?,
        );
    }
    if a[1] != "diagnose" {
        return Err("unknown mode".into());
    }
    let mut rows = Vec::new();
    for c in cases {
        let parent = generate(&model, &c.prompt, Control::Full, false, false)?;
        let all_current = generate(
            &model,
            &c.prompt,
            Control::CurrentRelationReadAll,
            false,
            false,
        )?;
        let trace = model.source_routing_trace(&c.prompt)?;
        rows.push(json!({"id":c.id,"prompt":c.prompt,"expected":c.expected,"parent":parent,"including_recent":all_current,"trace":trace}));
    }
    save(
        &Path::new(&a[4]).join("diagnostic.json"),
        &json!({"artifact":model.artifact_cid(),"total":rows.len(),"rows":rows,"scope":"Open reachability intervention; not a selected or newly learned artifact. Historic/raw questions are unqualified controls."}),
    )
}
