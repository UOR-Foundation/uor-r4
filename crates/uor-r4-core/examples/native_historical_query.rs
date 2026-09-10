//! Prompt-only generation, fitting and controls for contextual historical selection.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{
    Control, HistoricalQueryExample, Model, RoutingMode, SourceRoutingConfig, BOS, EOS,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
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

fn diagnose() -> Result<()> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() != 4 || !["diagnose", "intent-diagnose", "current-diagnose"].contains(&a[1].as_str())
    {
        return Err("usage: diagnose MODEL OUT".into());
    }
    let model = Model::from_bytes(&fs::read(&a[2])?)?;
    let mut rows = Vec::new();
    let facts =
        if a[1] == "intent-diagnose" {
            ["Record: selvi in Dusk Ridge. selvi now in Copper Vale. selvi now in Amber Field.",
         "Record: moss dale holds selvi. selvi now in Copper Vale. selvi now in Amber Field."]
        } else {
            [
                "Record: selvi in Dusk Ridge. selvi now in Copper Vale.",
                "Record: moss dale holds selvi. selvi now in Copper Vale.",
            ]
        };
    let queries = if a[1] == "current-diagnose" {
        [
            "Where is selvi?",
            "What is the current location of selvi?",
            "What was the previous location of selvi?",
        ]
    } else if a[1] == "intent-diagnose" {
        [
            "What was the previous location of selvi?",
            "What was the initial location of selvi?",
            "Where is selvi?",
        ]
    } else {
        [
            "Where was selvi before?",
            "What was the previous location of selvi?",
            "Where is selvi?",
        ]
    };
    for (layout, fact) in facts.iter().enumerate() {
        for (query, q) in queries.iter().enumerate() {
            for (style, s) in ["", "Name the owner first.", "State the owner first."]
                .iter()
                .enumerate()
            {
                let prompt = format!("{fact} {q} {s} Answer:");
                rows.push(json!({"id":format!("{layout}/{query}/{style}"),"prompt":prompt,"actual":generate(&model,&prompt,Control::Full,false,false)?,"trace":model.historical_query_trace(&prompt)?,"source_trace":if a[1] == "current-diagnose" { Some(model.source_routing_trace(&prompt)?) } else { None }}));
            }
        }
    }
    save(
        Path::new(&a[3]),
        &json!({"artifact":model.artifact_cid(),"rows":rows}),
    )?;
    Ok(())
}
#[derive(Clone, Serialize, Deserialize)]
struct Case {
    id: String,
    prompt: String,
    expected: Option<String>,
    #[serde(default)]
    current_record: Option<u64>,
    #[serde(default)]
    owner: String,
    #[serde(default)]
    value: String,
}
fn owner_cases(owners: &[(String, String, String)]) -> Vec<Case> {
    let mut cases = Vec::new();
    let absent = owners.get(2).map_or("merli", |o| o.0.as_str());
    for swap in [false, true] {
        let order = if swap { [1, 0] } else { [0, 1] };
        let mut facts = String::from("Record:");
        for i in order {
            let (owner, old, new) = &owners[i];
            facts.push_str(&format!(" {old} holds {owner}. {owner} now in {new}."));
        }
        for evicted in [false, true] {
            let padding = if evicted {
                "oak ash elm ".repeat(12)
            } else {
                String::new()
            };
            for i in 0..3 {
                let (owner, old) = if i < 2 {
                    (owners[i].0.as_str(), owners[i].1.as_str())
                } else {
                    (absent, "")
                };
                for (style, instruction) in ["Name the owner first.", "State the owner first."]
                    .iter()
                    .enumerate()
                {
                    let current_record = (i < 2).then(|| if i == order[0] { 2 } else { 4 });
                    cases.push(Case {
                        id: format!("owner/{swap}/{evicted}/{i}/{style}"),
                        prompt: format!("{facts} {padding}What was the previous location of {owner}? {instruction} Answer:"),
                        expected: current_record.map(|_| format!(" {owner} was in {old}.\n")),
                        current_record, owner: owner.into(), value: old.into(),
                    });
                }
                for (style, query) in [
                    format!("Where was {owner} before? Name the owner first."),
                    format!("Where was {owner} before? State the owner first."),
                    format!("What was the previous location of {owner}?"),
                    format!("Where is {owner}? Name the owner first."),
                ]
                .iter()
                .enumerate()
                {
                    cases.push(Case {
                        id: format!("owner-preserve/{swap}/{evicted}/{i}/{style}"),
                        prompt: format!("{facts} {padding}{query} Answer:"),
                        expected: None,
                        current_record: None,
                        owner: owner.into(),
                        value: old.into(),
                    });
                }
            }
        }
    }
    cases
}
fn prepare(out: &Path, fresh: bool) -> Result<()> {
    fs::create_dir(out)?;
    let draw = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let owners: Vec<(String, String, String)> = if fresh {
        (0..3)
            .map(|i| {
                let h = blake3::hash(format!("historical-query:{draw}:{i}").as_bytes());
                let b = h.as_bytes();
                let word = |a: usize, n: usize| {
                    (a..a + n)
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
        vec![
            ("selvi".into(), "Dusk Ridge".into(), "Copper Vale".into()),
            ("tilva".into(), "moss dale".into(), "Birch Grove".into()),
        ]
    };
    let mut cases = Vec::new();
    for (i, (owner, old, new)) in owners.iter().enumerate() {
        for reverse in [false, true] {
            for evicted in [false, true] {
                for third in [false, true] {
                    let fact = if reverse {
                        format!("{old} holds {owner}.")
                    } else {
                        format!("{owner} in {old}.")
                    };
                    let third_fact = if third {
                        format!(" {owner} now in Amber Field.")
                    } else {
                        String::new()
                    };
                    let padding = if evicted {
                        "oak ash elm ".repeat(12)
                    } else {
                        String::new()
                    };
                    let facts =
                        format!("Record: {fact} {owner} now in {new}.{third_fact} {padding}");
                    let previous = if third { new } else { old };
                    let current = if third { 3 } else { 2 };
                    for (style, instruction) in ["Name the owner first.", "State the owner first."]
                        .iter()
                        .enumerate()
                    {
                        cases.push(Case {
                            id: format!("target/{i}/{reverse}/{evicted}/{third}/{style}"),
                            prompt: format!(
                                "{facts}What was the previous location of {owner}? {instruction} Answer:"
                            ),
                            expected: Some(format!(" {owner} was in {previous}.\n")),
                            current_record: Some(current),
                            owner: owner.clone(),
                            value: previous.clone(),
                        });
                    }
                    for (style, q) in [
                        format!("Where was {owner} before?"),
                        format!("Where was {owner} before? Explain in a sentence."),
                        format!("What was the previous location of {owner}?"),
                        format!("Where is {owner}? Name the owner first."),
                        format!("Where was {owner} before? Name the owner first."),
                        format!("Where was {owner} before? State the owner first."),
                        format!("What is the current location of {owner}? Name the owner first."),
                    ]
                    .iter()
                    .enumerate()
                    {
                        cases.push(Case {
                            id: format!("preserve/{i}/{reverse}/{evicted}/{third}/{style}"),
                            prompt: format!("{facts}{q} Answer:"),
                            expected: None,
                            current_record: None,
                            owner: owner.clone(),
                            value: previous.clone(),
                        });
                    }
                }
            }
        }
    }
    for (i,prompt) in ["Where was merli before? Name the owner first. Answer:","Record: selvi in Oak Bay. Where was selvi before? Name the owner first. Answer:","Record: selvi in Oak Bay. Record: selvi in Pine Cove. Where was selvi before? Name the owner first. Answer:"].iter().enumerate() {
        cases.push(Case{id:format!("unsupported/{i}"),prompt:(*prompt).into(),expected:None,current_record:None,owner:String::new(),value:String::new()});
    }
    cases.extend(owner_cases(&owners));
    let mut docs: Vec<_> = cases
        .iter()
        .map(|c| HistoricalQueryExample {
            id: c.id.clone(),
            prompt: c.prompt.clone(),
            current_record: c.current_record,
            inherit: c.current_record.is_none(),
        })
        .collect();
    let mut excluded = Vec::new();
    if !fresh {
        for (group,path) in [
            ("retained", "/Users/casey.allard/Documents/Codex/2026-09-09/uor-r4-reverse-start-transfer/construction/cases.json"),
            ("owner", "/Users/casey.allard/Documents/Codex/2026-09-09/uor-r4-query-owner-selection/fresh-data/cases.json"),
            ("history", "/Users/casey.allard/Documents/Codex/2026-09-09/uor-r4-reverse-start-transfer/fresh-data/cases.json"),
            ("fields", "/Users/casey.allard/Documents/Codex/2026-09-09/uor-r4-historical-field-composition/construction/cases.json"),
            ("fields-fresh", "/Users/casey.allard/Documents/Codex/2026-09-09/uor-r4-historical-field-composition/fresh-data/cases.json"),
        ] {
            let prior:Vec<Case>=serde_json::from_slice(&fs::read(path)?)?;
            for c in prior {
                // Earlier long owner-first negatives are the explicitly declared repair target.
                // Exact original files remain preserved; all other prior prompts constrain parent choice.
                if ["fields", "fields-fresh"].contains(&group) && c.prompt.contains("previous location") && (c.prompt.contains("Name the owner first.") || c.prompt.contains("State the owner first.")) { excluded.push(json!({"group":group,"id":c.id,"prompt":c.prompt,"reason":"declared long historical owner-first repair target","original_source":path})); continue; }
                docs.push(HistoricalQueryExample{id:format!("{group}/{}",c.id),prompt:c.prompt,current_record:None,inherit:true});
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        docs.retain(|d| seen.insert(d.prompt.clone()));
    }
    save(&out.join("excluded-prior-negatives.json"), &excluded)?;
    save(&out.join("cases.json"), &cases)?;
    if !fresh {
        save(&out.join("training.json"), &docs)?;
    }
    save(
        &out.join("receipt.json"),
        &json!({"at":format!("{:?}",std::time::SystemTime::now()),"draw":draw.to_string(),"fresh":fresh,"groups":owners,"cases":cases.len(),"targets":cases.iter().filter(|c|c.current_record.is_some()).count(),"fixed_authored_forms":true,"labels_offline_only":true}),
    )?;
    Ok(())
}
fn evaluate(
    model: &Model,
    parent: Option<&Model>,
    cases: &[Case],
    out: &Path,
    controls: bool,
    evaluation_control: Control,
) -> Result<()> {
    fs::create_dir_all(out)?;
    let mut rows = Vec::new();
    let (mut exact, mut targets, mut preserved, mut inherited, mut disabled_equal, mut selected) =
        (0, 0, 0, 0, 0, 0);
    for c in cases {
        let target = c.current_record.is_some();
        let actual = generate(model, &c.prompt, evaluation_control, target, !target)?;
        let reference = parent
            .map(|m| generate(m, &c.prompt, Control::Full, false, !target))
            .transpose()?;
        let records_unchanged = actual["initial_relations"] == actual["final_relations"];
        let correct = target.then(|| {
            c.expected.as_ref().is_some_and(|e| actual["text"] == *e)
                && actual["eos"] == true
                && records_unchanged
        });
        if target {
            targets += 1;
            exact += usize::from(correct == Some(true));
        }
        let same = reference
            .as_ref()
            .map(|r| comparable(&actual) == comparable(r));
        if !target && reference.is_some() {
            inherited += 1;
            preserved += usize::from(same == Some(true));
        }
        let mut interventions = Vec::new();
        if target {
            let anchor = &actual["first_decision"]["field"]["anchor"];
            let records = actual["initial_relations"]["records"]
                .as_array()
                .ok_or("records")?;
            let current = records
                .iter()
                .find(|r| r["id"].as_u64() == c.current_record);
            if current.is_some_and(|r| {
                anchor["relation_id"] == r["previous"] && anchor["current_revision"] == r["id"]
            }) {
                selected += 1;
            }
            if controls {
                for control in [
                    Control::HistoricalQueryContextDisabled,
                    Control::HistoricalQueryWindowDisabled,
                    Control::HistoricalReadDisabled,
                    Control::LearnedRoutingTransformDisabled,
                    Control::FieldCompositionReadDisabled,
                ] {
                    let result = generate(model, &c.prompt, control, false, !target)?;
                    if control == Control::HistoricalQueryContextDisabled
                        && reference
                            .as_ref()
                            .is_some_and(|r| comparable(&result) == comparable(r))
                    {
                        disabled_equal += 1;
                    }
                    interventions.push(json!({"control":control,"result":result}));
                }
            }
        }
        rows.push(json!({"id":c.id,"prompt":c.prompt,"expected":c.expected,"current_record":c.current_record,"actual":actual,"parent":reference,"correct":correct,"records_unchanged":records_unchanged,"parent_equal":same,"controls":interventions}));
    }
    let summary = json!({"artifact":model.artifact_cid(),"total":cases.len(),"targets":targets,"exact":exact,"selected":selected,"inherited":inherited,"preserved":preserved,"disabled_parent_equal":disabled_equal});
    let mut result = summary.clone();
    result["rows"] = json!(rows);
    save(&out.join("result.json"), &result)?;
    println!("{summary}");
    Ok(())
}
fn main() -> Result<()> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() == 4 && a[1] == "owner-diagnose" {
        let model = Model::from_bytes(&fs::read(&a[2])?)?;
        let mut rows = Vec::new();
        for reverse in [false, true] {
            let chains = [
                "Dusk Ridge holds selvi. selvi now in Copper Vale.",
                "moss dale holds tilva. tilva now in Birch Grove.",
            ];
            let facts = if reverse {
                format!("{} {}", chains[1], chains[0])
            } else {
                format!("{} {}", chains[0], chains[1])
            };
            for owner in ["selvi", "tilva", "merli"] {
                for style in ["Name", "State"] {
                    let prompt = format!("Record: {facts} What was the previous location of {owner}? {style} the owner first. Answer:");
                    rows.push(json!({"reverse":reverse,"owner":owner,"style":style,"prompt":prompt,"actual":generate(&model,&prompt,Control::Full,false,false)?,"trace":model.historical_query_trace(&prompt)?}));
                }
            }
        }
        return save(
            Path::new(&a[3]),
            &json!({"artifact":model.artifact_cid(),"rows":rows}),
        );
    }
    if a.len() == 3 && ["prepare", "fresh"].contains(&a[1].as_str()) {
        return prepare(Path::new(&a[2]), a[1] == "fresh");
    }
    if a.len() == 4 && ["diagnose", "intent-diagnose", "current-diagnose"].contains(&a[1].as_str())
    {
        return diagnose();
    }
    if a.len() != 5
        || ![
            "expose",
            "fit",
            "repair-prior",
            "preserve",
            "baseline",
            "evaluate",
            "controls",
            "window",
        ]
        .contains(&a[1].as_str())
    {
        return Err("usage: prepare/fresh OUT | diagnose/intent-diagnose/current-diagnose/owner-diagnose MODEL OUT | expose/fit/repair-prior/preserve/baseline/evaluate/controls/window MODEL CASES OUT".into());
    }
    let bytes = fs::read(&a[2])?;
    let model = Model::from_bytes(&bytes)?;
    let out = Path::new(&a[4]);
    fs::create_dir_all(out)?;
    if a[1] == "expose" {
        let candidate = model.with_historical_query_context()?;
        fs::write(out.join("model.json"), candidate.to_bytes()?)?;
        save(
            &out.join("exposure.json"),
            &json!({"parent":model.artifact_cid(),"candidate":candidate.artifact_cid(),"no_parameter_fit":true}),
        )?;
        return Ok(());
    }
    if a[1] == "fit" {
        let docs: Vec<HistoricalQueryExample> = serde_json::from_slice(&fs::read(&a[3])?)?;
        let config = SourceRoutingConfig {
            learned_features: 768,
            passes: 8,
            proposals: 120,
            max_seconds: 120,
            mode: RoutingMode::Angular,
            seed: 973,
            role_context_only: false,
        };
        let (candidate, report) = model.fit_historical_query_context(&docs, config)?;
        fs::write(out.join("model.json"), candidate.to_bytes()?)?;
        save(&out.join("fit.json"), &report)?;
        println!("{report}");
        return Ok(());
    }
    let mut cases: Vec<Case> = serde_json::from_slice(&fs::read(&a[3])?)?;
    if a[1] == "repair-prior" {
        // Reuse the original historical-field panel's exact group labels.
        // This classification is offline evaluation only; generation sees prompt bytes.
        let group = |id: &str| -> Result<String> {
            let (_, tail) = id.split_once('/').ok_or("prior group prefix absent")?;
            let (key, _) = tail.rsplit_once('/').ok_or("prior group style absent")?;
            Ok(key.to_owned())
        };
        let mut labels = std::collections::BTreeMap::new();
        for c in cases.iter().filter(|c| c.current_record.is_some()) {
            let label = (
                c.current_record,
                c.expected.clone(),
                c.owner.clone(),
                c.value.clone(),
            );
            if labels
                .insert(group(&c.id)?, label.clone())
                .is_some_and(|old| old != label)
            {
                return Err("prior historical group has inconsistent exact labels".into());
            }
        }
        for c in &mut cases {
            if c.prompt.contains("previous location")
                && (c.prompt.contains("Name the owner first.")
                    || c.prompt.contains("State the owner first."))
            {
                let label = labels
                    .get(&group(&c.id)?)
                    .ok_or("prior repair group unavailable")?;
                c.current_record = label.0;
                c.expected = label.1.clone();
                c.owner = label.2.clone();
                c.value = label.3.clone();
            } else {
                c.current_record = None;
            }
        }
        save(&out.join("classified-cases.json"), &cases)?;
    }
    if a[1] == "preserve" {
        for c in &mut cases {
            c.current_record = None;
        }
    }
    if a[1] == "baseline" {
        return evaluate(&model, None, &cases, out, false, Control::Full);
    }
    let parent = model.without_historical_query_context()?;
    save(
        &out.join("lineage.json"),
        &json!({"parent":parent.artifact_cid(),"candidate":model.artifact_cid(),"roundtrip":model.to_bytes()?==bytes,"parent_bytes_blake3":blake3::hash(&parent.to_bytes()?).to_hex().to_string()}),
    )?;
    evaluate(
        &model,
        Some(&parent),
        &cases,
        out,
        a[1] == "controls",
        if a[1] == "window" {
            Control::HistoricalQueryWindowDisabled
        } else {
            Control::Full
        },
    )
}
