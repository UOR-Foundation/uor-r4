//! Offline writer supervision and actual parent/candidate behavior comparison.
//! Suppression annotations never enter the serving session.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path};
use uor_r4_core::native_geometric::{Control, Model, Session, WriterLexicalExample, BOS, EOS};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Serialize, Deserialize)]
struct Case {
    id: String,
    prompt: String,
    suppress_end_bytes: Vec<u64>,
    source: String,
    #[serde(default)]
    expected: Option<String>,
}
fn save(path: &Path, value: &impl Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec(value)?)?;
    Ok(())
}
fn checksum(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}
fn checkpoint(s: &Session) -> Result<Value> {
    Ok(serde_json::from_slice(&s.checkpoint()?)?)
}
fn state_without_work(mut state: Value) -> Result<Value> {
    state
        .as_object_mut()
        .ok_or("checkpoint object absent")?
        .remove("work");
    Ok(state)
}
fn endpoints(text: &str, offset: usize) -> Vec<u64> {
    let bytes = text.as_bytes();
    bytes
        .iter()
        .enumerate()
        .filter_map(|(i, b)| {
            (b.is_ascii_alphabetic() && bytes.get(i + 1).is_none_or(|n| !n.is_ascii_alphabetic()))
                .then_some((offset + i) as u64)
        })
        .collect()
}
fn annotate_existing(prompt: &str) -> Vec<u64> {
    let mut ends = Vec::new();
    for phrase in ["Explain in a sentence.", "Explain the stop in a sentence."] {
        for (start, text) in prompt.match_indices(phrase) {
            ends.extend(endpoints(text, start));
        }
    }
    ends.sort_unstable();
    ends.dedup();
    ends
}
fn authored_group(group: usize, owner: &str, value: &str, prefix: &str) -> Vec<Case> {
    let instruction = "Explain in a sentence.";
    let stop = "Explain the stop in a sentence.";
    let collision_fact_owner = if prefix == "construction" && group == 0 {
        "velra"
    } else {
        owner
    };
    let mut result = Vec::new();
    let layouts = [
        (format!("Where is {owner}? {instruction} Answer:"), Some(instruction)),
        (format!("{instruction} Where is {owner}? Answer:"), Some(instruction)),
        (format!("Record: {owner} in {value}. Where is {owner}? {instruction} Answer:"), Some(instruction)),
        (format!("Record: {owner} in {value}. Where is {owner}? {stop} Answer:"), Some(stop)),
        (format!("Record: tilva in {value}. Where is {owner}? {instruction} Answer:"), Some(instruction)),
        (format!("{instruction} Record: {owner} in {value}. Where is {owner}? Answer:"), Some(instruction)),
        (format!("Record: tilva in Cedar Harbor. {instruction} Record: {owner} in {value}. Where is {owner}? Answer:"), Some(instruction)),
        (format!("Where is {owner}? {stop} Answer:"), Some(stop)),
        (format!("Where is {owner}? {collision_fact_owner} in {value}. Answer:"), None),
        (format!("Record: Explain in {value}. Where is Explain? Answer:"), None),
        (format!("Record: {owner} in a. Where is {owner}? Answer:"), None),
        (format!("Record: {owner} in sentence. Where is {owner}? Answer:"), None),
        (format!("Record: stop in {value}. Where is stop? Answer:"), None),
        ("Record: a in sentence. Where is a? Answer:".into(), None),
        ("Record: sentence in stop. Where is sentence? Answer:".into(), None),
        // Exact same Explain/a endpoint spellings as the instruction, explicitly
        // used as factual payloads. This occurrence receives no suppression.
        ("Record: Explain in a sentence. Where is Explain? Answer:".into(), None),
    ];
    for (index, (prompt, annotation)) in layouts.into_iter().enumerate() {
        let suppress_end_bytes = annotation.map_or_else(Vec::new, |phrase| {
            prompt
                .match_indices(phrase)
                .flat_map(|(at, text)| endpoints(text, at))
                .collect()
        });
        result.push(Case {
            id: format!("{prefix}/{group}/{index}"),
            prompt,
            suppress_end_bytes,
            source: "Authored balanced instruction/fact context; explicit annotation only".into(),
            expected: None,
        });
    }
    result
}
fn write_data(out: &Path, cases: &[Case], receipt: Value) -> Result<()> {
    if cases.is_empty() || cases.len() > 1024 {
        return Err("case bound1..1024".into());
    }
    if cases
        .iter()
        .any(|c| c.prompt.is_empty() || c.prompt.len() > 4096)
        || cases.iter().map(|c| c.prompt.len()).sum::<usize>() > 4 * 1024 * 1024
    {
        return Err("writer data exceeds4096 bytes per prompt or4MiB combined".into());
    }
    fs::create_dir(out)?;
    let docs: Vec<_> = cases
        .iter()
        .map(|c| WriterLexicalExample {
            id: c.id.clone(),
            prompt: c.prompt.clone(),
            suppress_end_bytes: c.suppress_end_bytes.clone(),
        })
        .collect();
    let data = serde_json::to_vec(&docs)?;
    fs::write(out.join("training.json"), &data)?;
    save(&out.join("cases.json"), &cases)?;
    let instructions: Vec<_> = cases
        .iter()
        .filter(|c| !c.suppress_end_bytes.is_empty())
        .collect();
    save(&out.join("instruction-subset.json"), &instructions)?;
    save(
        &out.join("receipt.json"),
        &json!({"provenance":receipt,"cases":cases.len(),"instruction_cases":instructions.len(),
        "training_blake3":checksum(&data),"scope":"All annotations are offline completed-word byte-end labels. Unlisted boundaries preserve the exact parent writer winner. Serving observes prompt tokens only. Factual cue-word occurrences carry no suppression."}),
    )?;
    Ok(())
}
fn prepare(input: &Path, out: &Path) -> Result<()> {
    let bytes = fs::read(input)?;
    let prior: Vec<Value> = serde_json::from_slice(&bytes)?;
    if prior.len() != 761 {
        return Err("expected761 retained prompt documents".into());
    }
    let mut cases = Vec::new();
    for row in prior {
        let id = row["id"].as_str().ok_or("prior id absent")?;
        let prompt = row["prompt"].as_str().ok_or("prior prompt absent")?;
        cases.push(Case {
            id: format!("prior/{id}"),
            prompt: prompt.into(),
            suppress_end_bytes: annotate_existing(prompt),
            source: input.display().to_string(),
            expected: None,
        });
    }
    let mut open = Vec::new();
    for (i, (owner, value)) in [
        ("selvi", "Dusk Ridge"),
        ("serin", "Copper Vale"),
        ("navri", "Amber Hill"),
        ("pelvi", "Cedar Bay"),
    ]
    .into_iter()
    .enumerate()
    {
        open.extend(authored_group(i, owner, value, "construction"));
    }
    cases.extend(open.clone());
    write_data(
        out,
        &cases,
        json!({"input":input,"input_blake3":checksum(&bytes),"retained":761,"authored":64}),
    )?;
    save(&out.join("open.json"), &open)?;
    let collision = vec![
        Case {
            id: "collision/instruction".into(),
            prompt: "Where is selvi? Explain in a sentence. Answer:".into(),
            suppress_end_bytes: annotate_existing("Where is selvi? Explain in a sentence. Answer:"),
            source: "Exact source collision diagnostic".into(),
            expected: None,
        },
        Case {
            id: "collision/fact".into(),
            prompt: "Where is selvi? velra in Dusk Ridge. Answer:".into(),
            suppress_end_bytes: Vec::new(),
            source: "Exact source collision diagnostic".into(),
            expected: None,
        },
    ];
    save(&out.join("collision.json"), &collision)?;
    Ok(())
}
fn fresh(out: &Path) -> Result<()> {
    let elapsed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
    let seed = elapsed.as_nanos() as u64;
    let mut rng = if seed == 0 { 1 } else { seed };
    let mut used = std::collections::BTreeSet::new();
    let mut draw = |title: bool| -> Result<String> {
        for _ in 0..1024 {
            let mut s = String::new();
            for _ in 0..5 {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                s.push((b'a' + (rng % 26) as u8) as char);
            }
            if used.insert(s.clone()) {
                if title {
                    s[..1].make_ascii_uppercase();
                }
                return Ok(s);
            }
        }
        Err("fresh draw exhausted".into())
    };
    let mut cases = Vec::new();
    let mut groups = Vec::new();
    for i in 0..2 {
        let owner = draw(false)?;
        let value = format!("{} {}", draw(true)?, draw(true)?);
        cases.extend(authored_group(i, &owner, &value, "fresh"));
        groups.push(json!({"owner":owner,"value":value}));
    }
    write_data(
        out,
        &cases,
        json!({"seed":seed,"unix_seconds":elapsed.as_secs(),"subsecond_nanos":elapsed.subsec_nanos(),"groups":groups,
        "scope":"Post-selection evaluation-only draw under frozen instruction/fact forms; fixed cue payloads repeated deliberately. No fit after draw, no global historical uniqueness claim."}),
    )?;
    Ok(())
}
fn relation(state: &Value) -> Result<Value> {
    let r = &state["values"]["relations"];
    if !r.is_object() {
        return Err("relation checkpoint metadata absent".into());
    }
    Ok(r.clone())
}
fn live_records(r: &Value) -> Result<Vec<Value>> {
    let mut rows: Vec<_> = r["records"]
        .as_array()
        .ok_or("records absent")?
        .iter()
        .filter(|v| v["id"].as_u64().is_some_and(|n| n > 0))
        .cloned()
        .collect();
    rows.sort_by_key(|r| r["id"].as_u64());
    Ok(rows)
}
fn generate(model: &Model, c: &Case, control: Control) -> Result<Value> {
    let mut s = model.session(control)?;
    s.observe(model, BOS)?;
    for token in model.encode(&c.prompt)? {
        s.observe(model, token)?;
    }
    s.begin_response(model)?;
    let initial_bytes = s.checkpoint()?;
    let initial: Value = serde_json::from_slice(&initial_bytes)?;
    let initial_relation = relation(&initial)?;
    let mut tokens = Vec::new();
    let mut trace = Vec::new();
    let mut eos = false;
    let mut error = None;
    let outcome = (|| -> Result<()> {
        for position in 0..96 {
            let mut restored = model.restore_session(&s.checkpoint()?)?;
            let p = s.predict(model)?;
            let word = s.word_copy_decision();
            let entry = s.response_entry_decision();
            let equal = restored.predict(model)? == p
                && restored.word_copy_decision() == word
                && restored.response_entry_decision() == entry
                && restored.value_decision() == s.value_decision()
                && restored.completion_decision() == s.completion_decision();
            s.observe(model, p.token)?;
            restored.observe(model, p.token)?;
            let state = checkpoint(&s)?;
            let parity =
                state_without_work(state.clone())? == state_without_work(checkpoint(&restored)?)?;
            trace.push(json!({"position":position,"token":p.token,"word":word,"entry":entry,"prediction_parity":equal,
                "checkpoint_parity_except_work":parity,"read_commit":state["word_copy"]["read_commit"]}));
            if p.token == EOS {
                eos = true;
            } else {
                tokens.push(p.token);
            }
            if !equal || !parity {
                return Err("checkpoint or prediction parity differs".into());
            }
            if eos {
                break;
            }
        }
        Ok(())
    })();
    if let Err(e) = outcome {
        error = Some(e.to_string());
    }
    let final_bytes = s.checkpoint()?;
    let final_state: Value = serde_json::from_slice(&final_bytes)?;
    let bytes = model.decode(&tokens)?;
    let decoded = String::from_utf8(bytes.clone());
    if let Err(e) = &decoded {
        error = Some(e.to_string());
    }
    Ok(
        json!({"text":decoded.ok(),"decoded_bytes":bytes,"tokens":tokens,"eos":eos,"output_bound_reached":!eos&&tokens.len()==96,
        "error":error,"trace":trace,"initial_relations":initial_relation,"final_relations":relation(&final_state)?,
        "initial_checkpoint_blake3":checksum(&initial_bytes),"final_checkpoint_blake3":checksum(&final_bytes),
        "prompt_writer_work":initial["work"]["values"]["relations"],"final_writer_work":s.work.values.relations,
        "scope":"Actual prompt observations and freely generated response; complete checkpoint-state parity excludes only cumulative diagnostic work counters."}),
    )
}
fn payload(atom: &Value) -> Result<Vec<u8>> {
    let n = atom["len"].as_u64().ok_or("payload length absent")? as usize;
    let values = atom["bytes"].as_array().ok_or("payload bytes absent")?;
    if n > values.len() {
        return Err("payload length exceeds storage".into());
    }
    values[..n]
        .iter()
        .map(|b| {
            b.as_u64()
                .and_then(|n| u8::try_from(n).ok())
                .ok_or_else(|| "invalid payload byte".into())
        })
        .collect()
}
fn value_payload(record: &Value) -> Result<Vec<u8>> {
    payload(if record["span"].is_object() {
        &record["span"]
    } else {
        &record["value"]
    })
}
fn instruction_reference(parent: &Value, c: &Case) -> Result<Value> {
    let original = &parent["initial_relations"];
    let rows = live_records(original)?;
    let next = original["next_id"]
        .as_u64()
        .ok_or("parent next id absent")?;
    if next != rows.len() as u64 + 1 || next > 17 {
        return Err("instruction reference requires complete un-evicted parent writes".into());
    }
    let mut retained: Vec<Value> = Vec::new();
    let mut directory = vec![0_u64; 16];
    let mut mapping = BTreeMap::new();
    let mut removed = Vec::new();
    for record in rows {
        let old = record["id"].as_u64().ok_or("record id absent")?;
        let owner_end = record["owner"]["byte_end"]
            .as_u64()
            .ok_or("owner endpoint absent")?;
        let value_end = record["value"]["byte_end"]
            .as_u64()
            .ok_or("value endpoint absent")?;
        if c.suppress_end_bytes.contains(&owner_end) && c.suppress_end_bytes.contains(&value_end) {
            removed.push(record);
            continue;
        }
        let owner = payload(&record["owner"])?;
        let previous = directory
            .iter()
            .enumerate()
            .filter(|(_, id)| **id > 0)
            .find_map(|(slot, id)| {
                let prior = &retained[(*id - 1) as usize];
                (payload(&prior["owner"]).ok().as_ref() == Some(&owner)).then_some((slot, prior))
            });
        let action = record["action"].as_u64().ok_or("record action absent")?;
        let conflict = action == 3
            || (action == 1
                && if let Some((_, prior)) = previous {
                    prior["conflict"] == true || value_payload(prior)? != value_payload(&record)?
                } else {
                    false
                });
        let previous_id = previous.map_or(0, |(_, r)| r["id"].as_u64().unwrap_or(0));
        let slot = previous
            .map(|(slot, _)| slot)
            .or_else(|| directory.iter().position(|id| *id == 0))
            .ok_or("reference directory full")?;
        let id = retained.len() as u64 + 1;
        let mut expected = record;
        expected["id"] = json!(id);
        expected["previous"] = json!(previous_id);
        expected["conflict"] = json!(conflict);
        mapping.insert(old, id);
        directory[slot] = id;
        retained.push(expected);
    }
    Ok(
        json!({"records":retained,"directory":directory,"next_id":retained.len()+1,"parent_to_candidate_ids":mapping,
        "removed":removed,"last_word_end":original["last_word_end"],"pending":original["pending"],
        "scope":"Offline replay of actual parent commits after removing only writes with both anchor endpoints explicitly within an annotated instruction; IDs, previous links, conflict state and directory recomputed under the retained write law."}),
    )
}
fn compare(c: &Case, parent: &Value, candidate: &Value, control: &Value) -> Result<Value> {
    let error_free = [parent, candidate, control]
        .iter()
        .all(|r| r.get("error") == Some(&Value::Null));
    let response_equal = if let Some(expected) = &c.expected {
        candidate["text"] == expected.as_str() && candidate["eos"] == true
    } else {
        candidate["text"] == parent["text"] && candidate["eos"] == parent["eos"]
    };
    let control_equal = control["text"] == parent["text"]
        && control["eos"] == parent["eos"]
        && control["initial_relations"] == parent["initial_relations"]
        && control["final_relations"] == parent["final_relations"];
    let reference = if c.suppress_end_bytes.is_empty() {
        Value::Null
    } else {
        instruction_reference(parent, c)?
    };
    let actual = &candidate["initial_relations"];
    let records_equal = if reference.is_null() {
        actual == &parent["initial_relations"]
    } else {
        json!(live_records(actual)?) == reference["records"]
            && actual["directory"] == reference["directory"]
            && actual["next_id"] == reference["next_id"]
            && actual["last_word_end"] == reference["last_word_end"]
            && actual["pending"] == reference["pending"]
    };
    let no_generation_writes = [parent, candidate, control]
        .iter()
        .all(|r| r["initial_relations"] == r["final_relations"]);
    Ok(
        json!({"pass":error_free&&response_equal&&control_equal&&records_equal&&no_generation_writes,
        "error_free":error_free,"response_matches_reference":response_equal,"writer_disabled_matches_restored_parent":control_equal,
        "records_match_reference":records_equal,"no_generation_relation_changes":no_generation_writes,"reference":reference,
        "parent_eos":parent["eos"],"candidate_eos":candidate["eos"],
        "qualification_scope":"Parent output equality preserves prior negatives, including non-EOS results; it does not qualify those outputs. Only an explicit expected field establishes an independent output target."}),
    )
}
fn evaluate(model: &Model, cases: &[Case], out: &Path) -> Result<()> {
    if cases.is_empty() || cases.len() > 1024 {
        return Err("evaluation case bound".into());
    }
    fs::create_dir(out)?;
    let parent = model.without_writer_lexical()?;
    let mut rows = Vec::new();
    for (index, c) in cases.iter().enumerate() {
        let outcome = (|| -> Result<Value> {
            let p = generate(&parent, c, Control::Full)?;
            let actual = generate(model, c, Control::Full)?;
            let disabled = generate(model, c, Control::WriterLexicalDisabled)?;
            let checks = compare(c, &p, &actual, &disabled)?;
            Ok(
                json!({"id":c.id,"case":c,"parent":p,"candidate":actual,"writer_disabled":disabled,"checks":checks}),
            )
        })();
        let row = match outcome {
            Ok(r) => r,
            Err(e) => json!({"id":c.id,"case":c,"error":e.to_string(),"checks":{"pass":false}}),
        };
        save(&out.join(format!("case-{index:04}.json")), &row)?;
        rows.push(json!({"id":c.id,"path":format!("case-{index:04}.json"),"checks":row["checks"],"error":row["error"]}));
        save(
            &out.join("result.json"),
            &json!({"artifact":model.artifact_cid(),"restored_parent":parent.artifact_cid(),
            "total":cases.len(),"completed":rows.len(),"passed":rows.iter().filter(|r|r["checks"]["pass"]==true).count(),"rows":rows}),
        )?;
    }
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str){
        Some("prepare") if args.len()==4=>prepare(Path::new(&args[2]),Path::new(&args[3])),
        Some("fresh") if args.len()==3=>fresh(Path::new(&args[2])),
        Some("fit") if args.len()==5||args.len()==7=>{
            let model=Model::from_bytes(&fs::read(&args[2])?)?;let data=fs::read(&args[3])?;
            let docs:Vec<WriterLexicalExample>=serde_json::from_slice(&data)?;
            let epochs=if args.len()==7{args[5].parse()?}else{8};let seconds=if args.len()==7{args[6].parse()?}else{120};
            let (candidate,report)=model.fit_writer_lexical(&docs,epochs,seconds)?;
            let out=Path::new(&args[4]);fs::create_dir(out)?;
            fs::write(out.join("model.json"),candidate.to_bytes()?)?;save(&out.join("fit.json"),&report)?;
            let restored=candidate.without_writer_lexical()?;
            save(&out.join("receipt.json"),&json!({"parent":model.artifact_cid(),"candidate":candidate.artifact_cid(),
                "restored_parent":restored.artifact_cid(),"parent_restoration_byte_exact":restored.to_bytes()?==model.to_bytes()?,
                "training_path":args[3],"training_blake3":checksum(&data),"epochs":epochs,"max_seconds":seconds}))?;Ok(())
        },
        Some("evaluate") if args.len()==5=>{let model=Model::from_bytes(&fs::read(&args[2])?)?;
            let cases:Vec<Case>=serde_json::from_slice(&fs::read(&args[3])?)?;evaluate(&model,&cases,Path::new(&args[4]))},
        Some("trace") if args.len()==5=>{let model=Model::from_bytes(&fs::read(&args[2])?)?;
            let cases:Vec<Case>=serde_json::from_slice(&fs::read(&args[3])?)?;
            if cases.len()>32{return Err("trace case cap32".into());}let out=Path::new(&args[4]);fs::create_dir(out)?;
            for (i,c) in cases.iter().enumerate(){save(&out.join(format!("trace-{i:02}.json")),&json!({"id":c.id,"trace":model.relation_writer_trace(&c.prompt)?}))?;}Ok(())},
        _=>Err("usage: native_writer_lexical prepare PRIOR_TRAINING_JSON OUT | fit MODEL TRAINING_JSON OUT [EPOCHS MAX_SECONDS] | evaluate MODEL CASES_JSON OUT | fresh OUT | trace MODEL CASES_JSON OUT".into())
    }
}
