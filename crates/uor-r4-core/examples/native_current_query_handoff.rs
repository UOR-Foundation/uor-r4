//! Offline labels and actual prompt-only checks for current-query handoff.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, io::Read, path::Path};
use uor_r4_core::native_geometric::{
    Control, CurrentQueryExample, Model, RoutingMode, SourceRoutingConfig, BOS, EOS,
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
            let mut a = a;
            let mut b = b;
            a.as_object_mut().ok_or("checkpoint object")?.remove("work");
            b.as_object_mut().ok_or("checkpoint object")?.remove("work");
            if a != b {
                return Err("complete restored causal state differs".into());
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

#[derive(Clone, Serialize, Deserialize)]
struct Case {
    id: String,
    prompt: String,
    #[serde(default)]
    expected: Option<String>,
    #[serde(default)]
    current_record: Option<u64>,
    #[serde(default)]
    owner: String,
    #[serde(default)]
    value: String,
}
type Owner = (String, String, String, String);
fn push_context(
    cases: &mut Vec<Case>,
    id: &str,
    facts: &str,
    owner: &str,
    value: &str,
    current: Option<u64>,
) {
    let facts = facts.trim_end();
    for (style, instruction) in ["", "Name the owner first.", "State the owner first."]
        .iter()
        .enumerate()
    {
        let suffix = if instruction.is_empty() {
            String::new()
        } else {
            format!(" {instruction}")
        };
        cases.push(Case {
            id: format!("target/{id}/{style}"),
            prompt: format!("{facts} What is the current location of {owner}?{suffix} Answer:"),
            expected: current.map(|_| {
                if style == 0 {
                    format!(" {value}.\n")
                } else {
                    format!(" {owner} is in {value}.\n")
                }
            }),
            current_record: current,
            owner: owner.into(),
            value: value.into(),
        });
    }
    // Explicitly preserve existing short current and historical response paths.
    for (style, question) in [
        format!("Where is {owner}?"),
        format!("Where is {owner}? Name the owner first."),
        format!("Where was {owner} before?"),
        format!("What was the previous location of {owner}? Name the owner first."),
        format!("What was the previous location of {owner}? State the owner first."),
    ]
    .iter()
    .enumerate()
    {
        cases.push(Case {
            id: format!("preserve/{id}/{style}"),
            prompt: format!("{facts} {question} Answer:"),
            expected: None,
            current_record: None,
            owner: owner.into(),
            value: value.into(),
        });
    }
}
fn chain(owner: &Owner, reverse: bool, consecutive: bool) -> String {
    let first = if reverse {
        format!("{} holds {}.", owner.1, owner.0)
    } else {
        format!("{} in {}.", owner.0, owner.1)
    };
    let last = if consecutive {
        format!(" {} now in {}.", owner.0, owner.3)
    } else {
        String::new()
    };
    format!("{first} {} now in {}.{last}", owner.0, owner.2)
}
fn authored_cases(owners: &[Owner]) -> Vec<Case> {
    let mut cases = Vec::new();
    for i in 0..2 {
        for reverse in [false, true] {
            for consecutive in [false, true] {
                for evicted in [false, true] {
                    let padding = if evicted {
                        "oak ash elm ".repeat(12)
                    } else {
                        String::new()
                    };
                    let facts = format!(
                        "Record: {} {padding}",
                        chain(&owners[i], reverse, consecutive)
                    );
                    let value = if consecutive {
                        &owners[i].3
                    } else {
                        &owners[i].2
                    };
                    push_context(
                        &mut cases,
                        &format!("single/{i}/{reverse}/{consecutive}/{evicted}"),
                        &facts,
                        &owners[i].0,
                        value,
                        Some(if consecutive { 3 } else { 2 }),
                    );
                }
            }
        }
    }
    // Both queried owners in both insertion orders, with homogeneous and mixed layouts.
    for (layout, reverse) in [[false, false], [true, true], [false, true]]
        .iter()
        .enumerate()
    {
        for swap in [false, true] {
            let order = if swap { [1, 0] } else { [0, 1] };
            for consecutive in [false, true] {
                let versions = if consecutive { 3 } else { 2 };
                for evicted in [false, true] {
                    let padding = if evicted {
                        "oak ash elm ".repeat(12)
                    } else {
                        String::new()
                    };
                    let facts = format!(
                        "Record: {} {} {padding}",
                        chain(&owners[order[0]], reverse[order[0]], consecutive),
                        chain(&owners[order[1]], reverse[order[1]], consecutive)
                    );
                    for i in 0..3 {
                        let current = (i < 2).then(|| {
                            if i == order[0] {
                                versions
                            } else {
                                versions * 2
                            }
                        });
                        let value = if i == 2 {
                            ""
                        } else if consecutive {
                            &owners[i].3
                        } else {
                            &owners[i].2
                        };
                        push_context(
                            &mut cases,
                            &format!("owners/{layout}/{swap}/{consecutive}/{evicted}/{i}"),
                            &facts,
                            &owners[i].0,
                            value,
                            current,
                        );
                    }
                }
            }
        }
    }
    cases
}
fn prepare(out: &Path, fresh: bool) -> Result<()> {
    // Fail on an existing destination: a fresh draw is never silently overwritten.
    uor_r4_core::report_output::claim(out)?;
    let owners: Vec<Owner> = if fresh {
        let mut entropy = [0_u8; 96];
        fs::File::open("/dev/urandom")?.read_exact(&mut entropy)?;
        let hex: String = entropy.iter().map(|b| format!("{b:02x}")).collect();
        save(
            &out.join("draw.json"),
            &json!({"source":"/dev/urandom","bytes":96,"entropy_hex":hex,"blake3":blake3::hash(&entropy).to_hex().to_string(),"at":format!("{:?}",std::time::SystemTime::now()),"redraw":false,"scope":"Fresh spelling draw; caller must invoke only after artifact selection. No fit is performed."}),
        )?;
        let mut result = Vec::new();
        for bytes in entropy.chunks_exact(32) {
            let word = |start: usize, n: usize| {
                bytes[start..start + n]
                    .iter()
                    .map(|b| (b'a' + b % 26) as char)
                    .collect::<String>()
            };
            result.push((
                word(0, 5),
                format!("{} {}", word(5, 4), word(9, 4)),
                format!("{} {}", word(13, 4), word(17, 4)),
                format!("{} {}", word(21, 4), word(25, 4)),
            ));
        }
        if result[0].0 == result[1].0 || result[0].0 == result[2].0 || result[1].0 == result[2].0 {
            return Err(
                "recorded fresh owner collision; retain draw and do not silently redraw".into(),
            );
        }
        result
    } else {
        vec![
            (
                "selvi".into(),
                "Dusk Ridge".into(),
                "Copper Vale".into(),
                "Amber Field".into(),
            ),
            (
                "tilva".into(),
                "moss dale".into(),
                "Birch Grove".into(),
                "Silver Cove".into(),
            ),
            ("merli".into(), "".into(), "".into(), "".into()),
        ]
    };
    let mut authored = authored_cases(&owners);
    if !fresh {
        // Retain the exact previously authored tilva/Amber Field revision variant
        // as a current repair target, independently of any model prediction.
        let prior_owner = (
            "tilva".into(),
            "moss dale".into(),
            "Birch Grove".into(),
            "Amber Field".into(),
        );
        for reverse in [false, true] {
            for evicted in [false, true] {
                let padding = if evicted {
                    "oak ash elm ".repeat(12)
                } else {
                    String::new()
                };
                let facts = format!("Record: {} {padding}", chain(&prior_owner, reverse, true));
                push_context(
                    &mut authored,
                    &format!("prior-current/{reverse}/{evicted}"),
                    &facts,
                    "tilva",
                    "Amber Field",
                    Some(3),
                );
            }
        }
    }
    for (id, prompt) in [
        ("dependent", "casket in elvin. elvin in Bremen. Question: Where is the location of casket? Answer:"),
        ("dependent-revision", "casket in elvin. elvin in Bremen. Now elvin in Zurich. Question: Where is the location of casket? Answer:"),
        ("dependent-missing", "casket in elvin. Question: Where is the location of casket? Answer:"),
        ("dependent-owner-missing", "casket in elvin. elvin in Bremen. Question: Where is the location of missing? Answer:"),
    ] {
        authored.push(Case { id: format!("dependency-preserve/{id}"), prompt: prompt.into(), expected: None, current_record: None, owner: String::new(), value: String::new() });
    }
    let mut fresh_contrasts = Vec::new();
    if fresh {
        let old = owners[2]
            .1
            .split_whitespace()
            .next()
            .ok_or("drawn dependency old value absent")?;
        let new = owners[2]
            .2
            .split_whitespace()
            .next()
            .ok_or("drawn dependency new value absent")?;
        for mut c in contrast_cases() {
            for key in ["prompt", "semantic_expected", "owner", "value"] {
                let text = c[key].as_str().ok_or("contrast text absent")?;
                c[key] = json!(text
                    .replace("casket", &owners[0].0)
                    .replace("elvin", &owners[1].0)
                    .replace("Bremen", old)
                    .replace("Zurich", new)
                    .replace("missing", &owners[2].0));
            }
            c["id"] = json!(c["id"].as_str().ok_or("contrast id absent")?.replacen(
                "open/",
                "fresh-contrast/",
                1
            ));
            c["fresh_spellings"] = json!(true);
            if c["id"]
                .as_str()
                .ok_or("fresh contrast id absent")?
                .starts_with("fresh-contrast/direct-current/")
            {
                c["current_record"] = json!(2);
                c["expected"] = c["semantic_expected"].clone();
                c["comparison"] = json!("Exact current-record2 positive target, with authored owner/value/answer; no prediction supplies this label.");
            } else {
                c["comparison"] = json!("Strict frozen-parent comparison; authored semantics are assessed independently, including any parent writer failure.");
            }
            authored.push(serde_json::from_value::<Case>(c.clone())?);
            fresh_contrasts.push(c);
        }
        let roles = current_role_contrasts("fresh-current-role", &owners[0].0, &owners[1].0, old);
        for c in &roles {
            authored.push(serde_json::from_value::<Case>(c.clone())?);
            fresh_contrasts.push(c.clone());
        }
        save(&out.join("current-role-contrasts.json"), &roles)?;
        save(&out.join("dependency-contrasts.json"), &fresh_contrasts)?;
        let semantics: Vec<_> = fresh_contrasts.iter().map(|c| json!({"id":c["id"],"prompt":c["prompt"],"semantic_expected":c["semantic_expected"],"contrast":c["contrast"],"scope":"Fresh spelling semantic label; not an assertion of frozen-parent correctness."})).collect();
        save(&out.join("semantic-expectations.json"), &semantics)?;
    }
    let mut cases = authored.clone();
    let mut sources = Vec::new();
    let mut duplicate_receipts = Vec::new();
    let mut excluded = Vec::new();
    if !fresh {
        let mut seen: BTreeMap<String, usize> = cases
            .iter()
            .enumerate()
            .map(|(i, c)| (c.prompt.clone(), i))
            .collect();
        for (group, path) in [
            ("historical-balanced", "/Users/casey.allard/Documents/Codex/2026-09-09/uor-r4-historical-query-context/balanced-construction/training.json"),
            ("retained1300", "/Users/casey.allard/Documents/Codex/2026-09-09/uor-r4-reverse-start-transfer/construction/cases.json"),
        ] {
            let bytes = fs::read(path)?;
            let prior: Vec<Case> = serde_json::from_slice(&bytes)?;
            sources.push(json!({"group":group,"path":path,"bytes":bytes.len(),"blake3":blake3::hash(&bytes).to_hex().to_string(),"rows":prior.len(),"treatment":"defer to complete frozen parent; target labels from prior tasks are not imported"}));
            for c in prior {
                if let Some(&index) = seen.get(&c.prompt) {
                    duplicate_receipts.push(json!({"source":path,"id":c.id,"kept":cases[index].id,"explicit_current_target":cases[index].current_record.is_some()}));
                    continue;
                }
                if c.prompt.contains("What is the current location of ") {
                    excluded.push(json!({"source":path,"id":c.id,"prompt":c.prompt,"reason":"prior long-current negative belongs to declared repair scope; exact label unavailable in this source; not a parent-preservation constraint","source_current_record":c.current_record}));
                    continue;
                }
                seen.insert(c.prompt.clone(), cases.len());
                cases.push(Case { id: format!("{group}/{}",c.id), prompt:c.prompt, expected:None, current_record:None, owner:String::new(), value:String::new() });
            }
        }
    }
    let docs: Vec<_> = cases
        .iter()
        .map(|c| CurrentQueryExample {
            id: c.id.clone(),
            prompt: c.prompt.clone(),
            current_record: c.current_record,
        })
        .collect();
    save(&out.join("excluded-prior-current.json"), &excluded)?;
    save(&out.join("cases.json"), &cases)?;
    save(&out.join("authored-cases.json"), &authored)?;
    if !fresh {
        save(&out.join("training.json"), &docs)?;
    }
    save(
        &out.join("receipt.json"),
        &json!({"schema":"uor-r4.current-query-preparation/1","fresh":fresh,"owners":owners,"cases":cases.len(),"authored":authored.len(),"targets":cases.iter().filter(|c|c.current_record.is_some()).count(),"inherited":cases.iter().filter(|c|c.current_record.is_none()).count(),"sources":sources,"duplicate_receipts":duplicate_receipts,"excluded_prior_current":excluded.len(),"labels_offline_only":true,"training_written":!fresh,"fixed_dependency_sentinels":4,"fresh_drawn_contrasts":fresh_contrasts.len(),"literal_current_role_contrasts":if fresh {16}else{0},"scope":"Authored exact current IDs; fresh spellings in unchanged authored forms do not establish general prose."}),
    )?;
    Ok(())
}
fn atom(v: &Value) -> Option<String> {
    let n = usize::try_from(v.get("len")?.as_u64()?).ok()?;
    let bytes = v
        .get("bytes")?
        .as_array()?
        .get(..n)?
        .iter()
        .map(|b| u8::try_from(b.as_u64()?).ok())
        .collect::<Option<Vec<_>>>()?;
    String::from_utf8(bytes).ok()
}
fn target_record<'a>(c: &Case, actual: &'a Value) -> Option<&'a Value> {
    let id = c.current_record?;
    let relations = &actual["initial_relations"];
    let record = relations["records"]
        .as_array()?
        .iter()
        .find(|r| r["id"].as_u64() == Some(id))?;
    let value = if record["span"].is_object() {
        &record["span"]
    } else {
        &record["value"]
    };
    (relations["directory"].as_array()?.contains(&json!(id))
        && record["conflict"] == false
        && record["action"] == 2
        && record["previous"].as_u64() == id.checked_sub(1)
        && atom(&record["owner"])? == c.owner
        && atom(value)? == c.value)
        .then_some(record)
}
fn selected(c: &Case, actual: &Value) -> bool {
    let Some(record) = target_record(c, actual) else {
        return false;
    };
    let Some(id) = c.current_record else {
        return false;
    };
    let source = 32 + ((id - 1) & 15);
    let field = &actual["first_decision"]["field"];
    let copy = &actual["first_decision"]["word_copy"];
    let endpoint = |d: &Value| {
        d["source_end"] == record["value"]["end"]
            && d["source_byte_end"] == record["value"]["byte_end"]
    };
    if field.is_object() {
        let a = &field["anchor"];
        a["relation_id"].as_u64() == Some(id)
            && a["source"].as_u64() == Some(source)
            && a["current_revision"].is_null()
            && endpoint(a)
    } else {
        copy["word_index"].as_u64() == Some(source)
            && copy["dependency"].is_null()
            && endpoint(copy)
    }
}
fn exact(c: &Case, actual: &Value) -> bool {
    c.expected.as_ref().is_some_and(|e| actual["text"] == *e)
        && actual["eos"] == true
        && actual["initial_relations"] == actual["final_relations"]
}
fn trace_summary(model: &Model, c: &Case, actual: &Value) -> Result<Value> {
    let trace = model.source_routing_trace(&c.prompt)?;
    let expected_source = c.current_record.map(|id| 32 + ((id - 1) & 15));
    let frozen_choice = &trace["current_relation_choice_including_recent"];
    let record_valid = target_record(c, actual).is_some();
    let candidate_matches =
        c.current_record.is_some() && record_valid && frozen_choice[0].as_u64() == expected_source;
    let candidates: Vec<_>=trace["current_relation_candidates"].as_array().into_iter().flatten().map(|v|json!({"id":v["record"]["id"],"owner":atom(&v["record"]["owner"]),"value":atom(if v["record"]["span"].is_object(){&v["record"]["span"]}else{&v["record"]["value"]}),"conflict":v["record"]["conflict"],"previous":v["record"]["previous"],"exact_value_recent":v["exact_value_recent"],"features":v["features"],"scores":v["scores"]})).collect();
    Ok(
        json!({"expected_current_record":c.current_record,"expected_source":expected_source,"expected_record_valid":record_valid,"candidate_matches":candidate_matches,"current_relation_choice_including_recent":frozen_choice,"candidates":candidates,"historical_choice":trace["historical_choice"],"dependent_choice":trace["dependent_choice"],"persistent_choice":trace["persistent_choice"],"direct_choice":trace["direct_choice"],"word_copy_eligible":trace["word_copy_eligible"],"direct_source_dispatch":trace["direct_source_dispatch"],"current_query_handoff_choice":trace["current_query_handoff_choice"]}),
    )
}
fn diagnose(model: &Model, cases: &[Case], out: &Path) -> Result<()> {
    uor_r4_core::report_output::claim(out)?;
    let mut rows = Vec::new();
    let mut targets = 0;
    let mut matched = 0;
    for c in cases {
        if c.current_record.is_none() {
            continue;
        }
        let actual = generate(model, &c.prompt, Control::Full, false, false)?;
        let trace = trace_summary(model, c, &actual)?;
        targets += 1;
        matched += usize::from(trace["candidate_matches"] == true);
        rows.push(json!({"id":c.id,"prompt":c.prompt,"expected":c.expected,"actual":actual,"trace":trace}));
    }
    save(
        &out.join("diagnostic.json"),
        &json!({"artifact":model.artifact_cid(),"targets":targets,"frozen_candidate_matches":matched,"rows":rows}),
    )?;
    println!(
        "{}",
        json!({"artifact":model.artifact_cid(),"targets":targets,"frozen_candidate_matches":matched,"scope":"No labels selected from predictions; mismatches remain explicit negatives before fitting."})
    );
    Ok(())
}
fn evaluate(model: &Model, parent: &Model, cases: &[Case], out: &Path) -> Result<()> {
    uor_r4_core::report_output::claim(out)?;
    let mut rows = Vec::new();
    let (
        mut targets,
        mut correct,
        mut selected_count,
        mut inherited,
        mut preserved,
        mut disabled_equal,
        mut erased_exact,
        mut positions,
        mut scope_exact,
        mut scope_preserved,
    ) = (0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    for c in cases {
        let target = c.current_record.is_some();
        let actual = generate(model, &c.prompt, Control::Full, target, !target)?;
        let reference = generate(parent, &c.prompt, Control::Full, false, !target)?;
        let same = comparable(&actual) == comparable(&reference);
        let selection = target && selected(c, &actual);
        let target_correct = target && exact(c, &actual) && selection;
        if target {
            targets += 1;
            correct += usize::from(target_correct);
            selected_count += usize::from(selection);
        } else {
            inherited += 1;
            preserved += usize::from(same);
        }
        positions += actual["checkpoint_positions"].as_u64().unwrap_or(0);
        let mut interventions = Vec::new();
        if target {
            for control in [
                Control::CurrentQueryHandoffDisabled,
                Control::CurrentQueryHandoffTransformDisabled,
            ] {
                let result = generate(model, &c.prompt, control, false, false)?;
                if control == Control::CurrentQueryHandoffDisabled {
                    disabled_equal += usize::from(comparable(&result) == comparable(&reference));
                } else {
                    erased_exact += usize::from(exact(c, &result));
                }
                interventions.push(json!({"control":control,"result":result,"exact":exact(c,&result),"selected":selected(c,&result)}));
            }
        }
        let scope = generate(
            model,
            &c.prompt,
            Control::CurrentQueryHandoffScopeDisabled,
            false,
            !target,
        )?;
        let scope_same = comparable(&scope) == comparable(&reference);
        let scope_correct = target && exact(c, &scope) && selected(c, &scope);
        scope_exact += usize::from(scope_correct);
        scope_preserved += usize::from(!target && scope_same);
        rows.push(json!({"id":c.id,"prompt":c.prompt,"expected":c.expected,"current_record":c.current_record,"owner":c.owner,"value":c.value,"actual":actual,"parent":reference,"correct":if target{Some(target_correct)}else{None},"selected":if target{Some(selection)}else{None},"parent_equal":same,"records_unchanged":actual["initial_relations"]==actual["final_relations"],"controls":interventions,"scope_disabled":{"text":scope["text"],"eos":scope["eos"],"parent_equal":scope_same,"exact_and_selected":scope_correct}}));
    }
    let summary = json!({"artifact":model.artifact_cid(),"parent":parent.artifact_cid(),"total":cases.len(),"targets":targets,"exact_and_selected":correct,"selected":selected_count,"inherited":inherited,"preserved":preserved,"disabled_parent_equal":disabled_equal,"transform_disabled_exact":erased_exact,"checkpoint_positions":positions,"scope_disabled_exact_and_selected":scope_exact,"scope_disabled_preserved":scope_preserved});
    let mut result = summary.clone();
    result["rows"] = json!(rows);
    save(&out.join("result.json"), &result)?;
    println!("{summary}");
    Ok(())
}
/// Literal `current` in data roles with one or two observed occurrences.
fn current_role_contrasts(prefix: &str, owner: &str, middle: &str, value: &str) -> Vec<Value> {
    let mut rows = Vec::new();
    for (role, facts, query_owner, answer) in [
        (
            "queried-owner",
            format!("current in {middle}. {middle} in {value}."),
            "current",
            format!(" {value}.\n"),
        ),
        (
            "intermediate-owner-value",
            format!("{owner} in current. current in {value}."),
            owner,
            format!(" {value}.\n"),
        ),
        (
            "missing-intermediate",
            format!("{owner} in current."),
            owner,
            " Unknown.\n".into(),
        ),
        (
            "unrelated-distractor",
            format!("{owner} in {middle}. {middle} in {value}. note in current."),
            owner,
            format!(" {value}.\n"),
        ),
    ] {
        for interrogative in ["Where", "What"] {
            for question_prefix in [false, true] {
                let question = if question_prefix { "Question: " } else { "" };
                rows.push(json!({"id":format!("{prefix}/{role}/{interrogative}/{question_prefix}"),"prompt":format!("{facts} {question}{interrogative} is the location of {query_owner}? Answer:"),"expected":null,"current_record":null,"owner":query_owner,"value":"","semantic_expected":answer,"contrast":"literal current word in data role","comparison":"Strict frozen-parent comparison; semantic dependency expectation does not assume parent correctness."}));
            }
        }
    }
    rows
}

/// Open development contrasts. Every row remains a strict parent comparison;
/// semantic expectations are authored separately and do not relabel training.
fn contrast_cases() -> Vec<Value> {
    let mut cases = Vec::new();
    for (kind, facts, answer) in [
        (
            "complete",
            "casket in elvin. elvin in Bremen.",
            " Bremen.\n",
        ),
        (
            "revised",
            "casket in elvin. elvin in Bremen. Now elvin in Zurich.",
            " Zurich.\n",
        ),
        ("missing", "casket in elvin.", " Unknown.\n"),
        (
            "conflicting",
            "casket in elvin. elvin in Bremen. elvin in Zurich.",
            " Unknown.\n",
        ),
    ] {
        for record_prefix in [false, true] {
            for question_prefix in [false, true] {
                let record = if record_prefix { "Record: " } else { "" };
                let question = if question_prefix { "Question: " } else { "" };
                for interrogative in ["Where", "What"] {
                    let prompt = format!("{record}{facts} {question}{interrogative} is the location of casket? Answer:");
                    cases.push(json!({"id":format!("open/dependency/{kind}/{record_prefix}/{question_prefix}/{interrogative}"),"prompt":prompt,"expected":null,"current_record":null,"owner":"casket","value":"","semantic_expected":answer,"contrast":"genuine dependency without explicit current cue","comparison":"complete frozen-parent behavior; semantic correctness assessed separately"}));
                }
            }
        }
    }
    for record_prefix in [false, true] {
        let record = if record_prefix { "Record: " } else { "" };
        for interrogative in ["Where", "What"] {
            cases.push(json!({"id":format!("open/absent/{record_prefix}/{interrogative}"),"prompt":format!("{record}casket in elvin. elvin in Bremen. Question: {interrogative} is the location of missing? Answer:"),"expected":null,"current_record":null,"owner":"missing","value":"","semantic_expected":" Unknown.\n","contrast":"absent queried owner","comparison":"complete frozen-parent behavior; semantic correctness assessed separately"}));
        }
        for owner_first in [false, true] {
            let instruction = if owner_first {
                " Name the owner first."
            } else {
                ""
            };
            let answer = if owner_first {
                " elvin is in Zurich.\n"
            } else {
                " Zurich.\n"
            };
            cases.push(json!({"id":format!("open/direct-current/{record_prefix}/{owner_first}"),"prompt":format!("{record}elvin in Bremen. Now elvin in Zurich. Question: What is the current location of elvin?{instruction} Answer:"),"expected":null,"current_record":null,"owner":"elvin","value":"Zurich","semantic_expected":answer,"contrast":"explicit direct current request","comparison":"complete frozen-parent behavior; an intended repair may differ and must be identified separately"}));
        }
    }
    cases
}
fn open_contrasts(out: &Path) -> Result<()> {
    uor_r4_core::report_output::claim(out)?;
    let cases = contrast_cases();
    if cases.len() != 40 {
        return Err("open contrast population differs from the authored 40 rows".into());
    }
    let semantics: Vec<_> = cases.iter().map(|c| json!({"id":c["id"],"prompt":c["prompt"],"semantic_expected":c["semantic_expected"],"contrast":c["contrast"],"scope":"Offline semantic expectation, never an assumption that the parent answered correctly."})).collect();
    save(&out.join("cases.json"), &cases)?;
    save(&out.join("semantic-expectations.json"), &semantics)?;
    let receipt = json!({"schema":"uor-r4.current-query-open-contrasts/1","at":format!("{:?}",std::time::SystemTime::now()),"cases":cases.len(),"dependency":32,"absent_owner":4,"explicit_direct_current":4,"record_question_prefixes":"all four combinations for dependency cases","interrogatives":["Where","What"],"training_written":false,"fresh_draw":false,"strict_parent_comparison_rows":40,"scope":"Open development panel authored after inspecting candidate roots; report all rows, with parent agreement separate from semantic correctness. No fitting or model generation occurs in open mode."});
    save(&out.join("receipt.json"), &receipt)?;
    println!("{receipt}");
    Ok(())
}

/// Incorporates every opened contrast only after recording its development status.
fn balanced(out: &Path) -> Result<()> {
    uor_r4_core::report_output::claim(out)?;
    let base =
        Path::new("/Users/casey.allard/Documents/Codex/2026-09-09/uor-r4-current-query-handoff");
    let cases_bytes = fs::read(base.join("construction/cases.json"))?;
    let train_bytes = fs::read(base.join("construction/training.json"))?;
    let open_bytes = fs::read(base.join("open/cases.json"))?;
    let mut cases: Vec<Case> = serde_json::from_slice(&cases_bytes)?;
    let mut docs: Vec<CurrentQueryExample> = serde_json::from_slice(&train_bytes)?;
    let opened: Vec<Value> = serde_json::from_slice(&open_bytes)?;
    if cases.len() != docs.len()
        || cases.iter().zip(&docs).any(|(c, d)| {
            c.id != d.id || c.prompt != d.prompt || c.current_record != d.current_record
        })
    {
        return Err("original construction cases/training correspondence differs".into());
    }
    let original_count = cases.len();
    let original_targets = cases.iter().filter(|c| c.current_record.is_some()).count();
    if original_targets != 204 || opened.len() != 40 {
        return Err("balanced source population is not original204 targets plus opened40".into());
    }
    let mut opened_rows = Vec::new();
    let mut direct = 0;
    for row in &opened {
        let mut c: Case = serde_json::from_value(row.clone())?;
        if c.current_record.is_some() || c.expected.is_some() {
            return Err("opened source labels were unexpectedly changed".into());
        }
        if c.id.starts_with("open/direct-current/") {
            if c.owner != "elvin" || c.value != "Zurich" {
                return Err("opened direct current semantic metadata differs".into());
            }
            c.current_record = Some(2);
            c.expected = Some(
                row["semantic_expected"]
                    .as_str()
                    .ok_or("opened direct semantic answer absent")?
                    .to_owned(),
            );
            direct += 1;
        }
        if docs
            .iter()
            .any(|d| d.id == c.id || (d.prompt == c.prompt && d.current_record != c.current_record))
        {
            return Err(format!("opened/original label conflict: {}", c.id).into());
        }
        docs.push(CurrentQueryExample {
            id: c.id.clone(),
            prompt: c.prompt.clone(),
            current_record: c.current_record,
        });
        opened_rows.push(json!({"id":c.id,"prompt":c.prompt,"current_record":c.current_record,"expected":c.expected,"semantic_expected":row["semantic_expected"],"contrast":row["contrast"],"training_status":"opened development data after540dfa71 dependency regression; not held-out"}));
        cases.push(c);
    }
    if direct != 4 {
        return Err("balanced opened direct-target count differs".into());
    }
    let roles = current_role_contrasts("construction-current-role", "casket", "elvin", "Bremen");
    if roles.len() != 16 {
        return Err("literal-current role population differs".into());
    }
    for row in &roles {
        let c: Case = serde_json::from_value(row.clone())?;
        if docs
            .iter()
            .any(|d| d.id == c.id || (d.prompt == c.prompt && d.current_record.is_some()))
        {
            return Err(format!("literal-current role contrast label conflict: {}", c.id).into());
        }
        docs.push(CurrentQueryExample {
            id: c.id.clone(),
            prompt: c.prompt.clone(),
            current_record: None,
        });
        cases.push(c);
    }
    save(&out.join("cases.json"), &cases)?;
    save(&out.join("training.json"), &docs)?;
    save(&out.join("opened-cases.json"), &opened_rows)?;
    save(&out.join("current-role-contrasts.json"), &roles)?;
    save(
        &out.join("current-role-preparation.json"),
        &json!({"cases":16,"targets":0,"source":"Rust authored data-role contrasts prepared before lexical fit selection","previously_observed_open_panel":false,"roles":["queried-owner","intermediate-owner-value","missing-intermediate","unrelated-distractor"],"interrogatives":["Where","What"],"question_prefix":[false,true],"scope":"All sixteen defer to frozen parent; both single and double cue occurrences are covered. Literal current remains data; authored semantic expectations are separate from parent agreement."}),
    )?;
    let sources: Vec<_> = [("construction/cases.json",&cases_bytes),("construction/training.json",&train_bytes),("open/cases.json",&open_bytes)].into_iter().map(|(path,bytes)|json!({"path":base.join(path),"bytes":bytes.len(),"blake3":blake3::hash(bytes).to_hex().to_string()})).collect();
    let receipt = json!({"schema":"uor-r4.current-query-balanced/1","original_cases":original_count,"original_targets":original_targets,"opened_rows":40,"opened_direct_targets":4,"opened_defer":36,"new_construction_role_defer":16,"cases":cases.len(),"targets":original_targets+direct,"sources":sources,"training_written":true,"scope":"All original rows retained; all40 opened rows and16 newly authored data-role contrasts included. Opened contrasts are development training data, not fresh qualification. None labels preserve complete frozen parent dispatch."});
    save(&out.join("receipt.json"), &receipt)?;
    println!("{receipt}");
    Ok(())
}

fn main() -> Result<()> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() == 3 && a[1] == "balanced" {
        return balanced(Path::new(&a[2]));
    }
    if a.len() == 3 && a[1] == "open" {
        return open_contrasts(Path::new(&a[2]));
    }
    if a.len() == 3 && ["prepare", "fresh"].contains(&a[1].as_str()) {
        return prepare(Path::new(&a[2]), a[1] == "fresh");
    }
    if a.len() != 5
        || ![
            "diagnose",
            "context-trace",
            "fit",
            "fit-lexical",
            "fit-scoped",
            "evaluate",
            "preserve",
        ]
        .contains(&a[1].as_str())
    {
        return Err("usage: prepare/fresh/open/balanced OUT | diagnose/context-trace MODEL CASES OUT | fit/fit-lexical/fit-scoped MODEL TRAIN OUT | evaluate/preserve MODEL CASES OUT".into());
    }
    let bytes = fs::read(&a[2])?;
    let model = Model::from_bytes(&bytes)?;
    if model.to_bytes()? != bytes {
        return Err("supplied artifact byte roundtrip differs".into());
    }
    let out = Path::new(&a[4]);
    if ["fit", "fit-lexical", "fit-scoped"].contains(&a[1].as_str()) {
        let docs: Vec<CurrentQueryExample> = serde_json::from_slice(&fs::read(&a[3])?)?;
        uor_r4_core::report_output::claim(out)?;
        let config = SourceRoutingConfig {
            learned_features: 768,
            passes: 8,
            proposals: 120,
            max_seconds: 120,
            mode: RoutingMode::Angular,
            seed: 973,
            role_context_only: false,
        };
        let (candidate, report) = if a[1] == "fit-scoped" {
            model.fit_current_query_scoped_handoff(&docs, config)?
        } else if a[1] == "fit-lexical" {
            model.fit_current_query_lexical_handoff(&docs, config)?
        } else {
            model.fit_current_query_handoff(&docs, config)?
        };
        fs::write(out.join("model.json"), candidate.to_bytes()?)?;
        save(&out.join("fit.json"), &report)?;
        println!("{report}");
        return Ok(());
    }
    let mut cases: Vec<Case> = serde_json::from_slice(&fs::read(&a[3])?)?;
    if a[1] == "context-trace" {
        uor_r4_core::report_output::claim(out)?;
        let mut rows = Vec::new();
        for c in &cases {
            rows.push(
                json!({"id":c.id,"prompt":c.prompt,"trace":model.current_query_trace(&c.prompt)?}),
            );
        }
        return save(&out.join("context-trace.json"), &rows);
    }
    if a[1] == "preserve" {
        for c in &mut cases {
            c.current_record = None;
            c.expected = None;
        }
    }
    if a[1] == "diagnose" {
        return diagnose(&model, &cases, out);
    }
    let parent = model.without_current_query_handoff()?;
    if parent.artifact_cid() == model.artifact_cid() {
        return Err("evaluate requires actual current-query outer witness".into());
    }
    evaluate(&model, &parent, &cases, out)?;
    save(
        &out.join("lineage.json"),
        &json!({"parent":parent.artifact_cid(),"artifact":model.artifact_cid(),"candidate_roundtrip":true,"parent_bytes_blake3":blake3::hash(&parent.to_bytes()?).to_hex().to_string()}),
    )?;
    Ok(())
}
