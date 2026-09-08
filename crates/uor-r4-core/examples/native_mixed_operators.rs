//! Authored, open diagnostic of plain numeric mixed operation continuation.
//! Exact operand labels are output as offline source data; serving sees prompts only.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{
    Control, Document, MixedOperatorExample, MixedOperatorTarget, Model,
    OperationTransitionExample, RoutingMode, Session, SourceRoutingConfig, TypedRoutingTurn,
    ValueAction, BOS, EOS,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExpectedWrite {
    id: u64,
    action: ValueAction,
    value: i64,
    operand_ids: [u64; 2],
    operand_values: [i64; 2],
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Case {
    input: MixedOperatorExample,
    expected: String,
    expected_writes: Vec<ExpectedWrite>,
    expected_followup: ExpectedWrite,
}

fn group(split: &str, a: i64, b: i64, extra: i64, names: [&str; 2], reverse: bool) -> Vec<Case> {
    // Literal records 0/1 precede the freely generated sum at record 2. The
    // query introduces extra at record 3 and its first result at record 4.
    // Even equal values retain distinct authored occurrence identities.
    let sum = a + b;
    let first = sum + extra;
    let [left, right] = names;
    let facts = if reverse {
        format!("{right} has {b} coins. {left} has {a} coins.")
    } else {
        format!("{left} has {a} coins. {right} has {b} coins.")
    };
    [
        ("add-only", "", None),
        ("add-copy-latest", " Copy the latest result.", Some((ValueAction::Copy, [4, 4], [first, first], first))),
        ("add-copy-original", " Copy the original total.", Some((ValueAction::Copy, [2, 2], [sum, sum], sum))),
        ("add-add-control", " Again.", Some((ValueAction::Add, [3, 4], [extra, first], first + extra))),
    ]
    .into_iter()
    .map(|(id, suffix, second)| {
        let mut expected = format!("{first}.\n");
        let mut expected_writes = vec![ExpectedWrite {
            id: 4, action: ValueAction::Add, value: first,
            operand_ids: [3, 2], operand_values: [extra, sum],
        }];
        let mut targets = Vec::new();
        if let Some((action, operand_ids, operand_values, value)) = second {
            targets.push(MixedOperatorTarget { action, operand_ids });
            expected_writes.push(ExpectedWrite { id: 5, action, value, operand_ids, operand_values });
            expected.push_str(&format!("{value}.\n"));
        }
        let next_literal = 4 + expected_writes.len() as u64;
        Case {
            expected_followup: ExpectedWrite { id: next_literal + 2, action: ValueAction::Add, value: sum, operand_ids: [next_literal + 1, next_literal], operand_values: if reverse { [a, b] } else { [b, a] } },
            input: MixedOperatorExample {
                first:MixedOperatorTarget{action:ValueAction::Add,operand_ids:[3,2]},
                context: OperationTransitionExample {
                    id: format!("{split}/{id}"),
                    history: vec![TypedRoutingTurn {
                        prompt: format!("User: {facts}\nUser: What is the sum of {left}'s and {right}'s coins?\nAssistant:"),
                        response: format!("{sum}.\n"),
                    }],
                    query: format!("User: There are {extra} extra coins. Add the extra coins to the original total.{suffix}\nAssistant:"),
                    first_response: format!("{first}.\n"),
                    next: None,
                },
                targets,
            },
            expected,
            expected_writes,
        }
    })
    .collect()
}
fn cases(mode: &str) -> Result<Vec<Case>> {
    if mode == "diagnostic" {
        return Ok(group("mixed-diagnostic", 13, 4, 3, ["suri", "orin"], false));
    }
    if mode == "diagnostic-placement" {
        let base = group("mixed-diagnostic", 13, 4, 3, ["suri", "orin"], false);
        let body = "There are 3 extra coins. Add the extra coins to the original total.";
        return Ok([
            (
                "prefix-after-latest",
                1,
                "After the addition, copy the latest result. ",
                "",
            ),
            (
                "prefix-copy-latest",
                1,
                "Copy the latest result after the addition. ",
                "",
            ),
            (
                "prefix-after-original",
                2,
                "After the addition, copy the original total. ",
                "",
            ),
            (
                "prefix-copy-original",
                2,
                "Copy the original total after the addition. ",
                "",
            ),
            (
                "suffix-repeat-latest",
                1,
                "",
                " Then repeat the latest result.",
            ),
            (
                "suffix-repeat-original",
                2,
                "",
                " Then repeat the original total.",
            ),
            ("suffix-copy-this", 1, "", " Then copy this result."),
            ("suffix-copy-prior", 2, "", " Then copy the prior total."),
        ]
        .into_iter()
        .map(|(label, index, before, after)| {
            let mut case = base[index].clone();
            case.input.context.id = format!("mixed-placement/{label}");
            case.input.context.query = format!("User: {before}{body}{after}\nAssistant:");
            case
        })
        .collect());
    }
    let (numbers, names, reverse): (&[(i64, i64, i64)], _, _) = match mode {
        "fit" => (
            &[(13, 4, 3), (14, 5, 4), (15, 6, 5)],
            ["suri", "orin"],
            false,
        ),
        "evaluate" => (&[(36, 7, 6), (37, 8, 7)], ["ada", "cyra"], false),
        "stress" => (&[(56, 9, 8), (57, 10, 0)], ["suri", "orin"], true),
        "fresh" => (&[(122, 11, 10), (-47, 12, 11)], ["mira", "lena"], false),
        _ => return Err("unknown mixed operator case population".into()),
    };
    Ok(numbers
        .iter()
        .enumerate()
        .flat_map(|(i, &(a, b, extra))| {
            group(&format!("mixed-{mode}-{i}"), a, b, extra, names, reverse)
        })
        .collect())
}
fn save(path: &Path, value: &impl Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn state(session: &Session) -> Result<Value> {
    Ok(serde_json::from_slice(&session.checkpoint()?)?)
}
fn generate(model: &Model, session: &mut Session) -> Result<Value> {
    let initial = state(session)?;
    let initial_writes = session.work.values.derived_writes;
    let mut tokens = Vec::new();
    let mut writes = Vec::new();
    let mut eos = false;
    let mut positions = 0;
    // Preserve partial output and a diagnostic error on failed verification.
    let outcome = (|| -> Result<()> {
        for _ in 0..96 {
            let mut restored = model.restore_session(&session.checkpoint()?)?;
            let prediction = session.predict(model)?;
            let completion = session.completion_decision();
            let value = session.value_decision();
            if session.predict(model)? != prediction
                || session.completion_decision() != completion
                || session.value_decision() != value
                || restored.predict(model)? != prediction
                || restored.completion_decision() != completion
                || restored.value_decision() != value
                || restored.predict(model)? != prediction
                || restored.completion_decision() != completion
                || restored.value_decision() != value
            {
                return Err("checkpoint or repeated prediction/decision differs".into());
            }
            if let Some(write) = value.filter(|w| w.cursor == 0) {
                writes.push(json!({"id":write.write_id,"action":write.action,"value":write.value,
                    "operand_ids":write.operands.map(|r|r.id),"operand_values":write.operands.map(|r|r.value)}));
            }
            session.observe(model, prediction.token)?;
            if prediction.token != EOS {
                tokens.push(prediction.token);
            }
            restored.observe(model, prediction.token)?;
            let actual = state(session)?;
            let restored_state = state(&restored)?;
            if actual["values"] != restored_state["values"]
                || actual["completion"] != restored_state["completion"]
            {
                return Err("checkpoint committed values/completion differs".into());
            }
            positions += 1;
            if prediction.token == EOS {
                eos = true;
                return Ok(());
            }
            for key in ["sources", "queries", "query_len", "query_boundary"] {
                if actual["values"][key] != initial["values"][key] {
                    return Err(format!("captured {key} changed within response").into());
                }
            }
            for key in ["queries", "query_len"] {
                if actual["values"]["lexemes"][key] != initial["values"]["lexemes"][key] {
                    return Err(format!("captured lexical {key} changed within response").into());
                }
            }
        }
        Err("response exceeded 96 tokens".into())
    })();
    let committed = state(session)?;
    let records: Vec<_> = committed["values"]["records"].as_array()
        .ok_or("committed records absent")?.iter().map(|r| {
            json!({"id":r["id"],"value":r["value"],"derived":r["derived"],"derivation":r["derivation"]})
        }).collect();
    let count = session
        .work
        .values
        .derived_writes
        .checked_sub(initial_writes)
        .ok_or("committed write counter decreased")?;
    let decoded = model.decode(&tokens)?;
    let text = String::from_utf8(decoded.clone());
    let error = outcome
        .err()
        .map(|e| e.to_string())
        .or_else(|| text.as_ref().err().map(|e| e.to_string()));
    Ok(
        json!({"text":text.ok(),"decoded_bytes":decoded,"tokens":tokens,"eos":eos,
        "writes":writes,"committed_records":records,"committed_write_count":count,
        "operations_committed":committed["values"]["operations_committed"].as_u64().unwrap_or(0),
        "checkpoint_positions":positions,"verification_error":error,
        "frozen_captured_input":error.is_none()}),
    )
}
fn check_writes(row: &Value, expected: &[ExpectedWrite]) -> Result<(bool, bool)> {
    let writes = row["writes"].as_array().ok_or("pending writes absent")?;
    let pending =
        writes.len() == expected.len() && row["writes"] == serde_json::to_value(expected)?;
    let records = row["committed_records"]
        .as_array()
        .ok_or("committed records absent")?;
    let committed = row["committed_write_count"] == expected.len()
        && row["operations_committed"] == expected.len()
        && expected.iter().all(|expected| {
            let mut found = records.iter().filter(|r| r["id"] == expected.id);
            let Some(record) = found.next() else {
                return false;
            };
            found.next().is_none()
                && record["value"] == expected.value
                && record["derived"] == true
                && record["derivation"]["action"] == json!(expected.action)
                && record["derivation"]["operand_ids"] == json!(expected.operand_ids)
                && record["derivation"]["operand_values"] == json!(expected.operand_values)
        });
    Ok((pending, committed))
}
fn evaluate_case(model: &Model, case: &Case, control: Control) -> Result<Value> {
    let mut session = model.session(Control::Full)?;
    session.observe(model, BOS)?;
    let mut history_rows = Vec::new();
    for turn in &case.input.context.history {
        for token in model.encode(&turn.prompt)? {
            session.observe(model, token)?;
        }
        session.begin_response(model)?;
        let actual = generate(model, &mut session)?;
        let valid = actual["text"] == turn.response
            && actual["eos"] == true
            && actual["verification_error"].is_null();
        history_rows.push(
            json!({"prompt":turn.prompt,"expected":turn.response,"actual":actual,"exact":valid}),
        );
        if !valid {
            return Ok(
                json!({"id":case.input.context.id,"history":history_rows,"exact":false,"status":"history_failed"}),
            );
        }
        session.end_response(model)?;
    }
    // Preserve actual Full history, then change only the declared intervention.
    if control != Control::Full {
        let mut checkpoint = state(&session)?;
        checkpoint["control"] = serde_json::to_value(control)?;
        session = model.restore_session(&serde_json::to_vec(&checkpoint)?)?;
    }
    for token in model.encode(&case.input.context.query)? {
        session.observe(model, token)?;
    }
    session.begin_response(model)?;
    let mut row = generate(model, &mut session)?;
    let (pending, committed) = check_writes(&row, &case.expected_writes)?;
    let exact = row["text"] == case.expected
        && row["eos"] == true
        && row["verification_error"].is_null()
        && pending
        && committed;
    row["id"] = case.input.context.id.clone().into();
    row["query"] = case.input.context.query.clone().into();
    row["expected"] = case.expected.clone().into();
    row["expected_writes"] = serde_json::to_value(&case.expected_writes)?;
    row["history"] = history_rows.into();
    row["pending_writes_exact"] = pending.into();
    row["committed_writes_exact"] = committed.into();
    row["exact"] = exact.into();
    if exact && control == Control::Full {
        session.end_response(model)?;
        let independent = case
            .input
            .context
            .history
            .first()
            .ok_or("independent followup source absent")?;
        for token in model.encode(&independent.prompt)? {
            session.observe(model, token)?;
        }
        session.begin_response(model)?;
        let mut followup = generate(model, &mut session)?;
        let (pending, committed) =
            check_writes(&followup, std::slice::from_ref(&case.expected_followup))?;
        let valid = followup["text"] == independent.response
            && followup["eos"] == true
            && followup["verification_error"].is_null()
            && pending
            && committed;
        followup["exact"] = valid.into();
        followup["expected"] = independent.response.clone().into();
        followup["expected_write"] = serde_json::to_value(&case.expected_followup)?;
        row["independent_followup"] = followup;
        row["exact"] = valid.into();
    }
    Ok(row)
}
fn evaluate(model: &Model, cases: &[Case], control: Control, path: &Path) -> Result<()> {
    let mut rows = Vec::new();
    for case in cases {
        rows.push(match evaluate_case(model, case, control) {
            Ok(row) => row,
            Err(error) => {
                json!({"id":case.input.context.id,"exact":false,"error":error.to_string()})
            }
        });
        save(
            path,
            &json!({"artifact":model.artifact_cid(),"control":control,
            "scope":"Authored plain numeric generation with exact occurrence checks; not formatted Copy or general reasoning evidence",
            "total":cases.len(),"completed":rows.len(),"exact":rows.iter().filter(|r|r["exact"]==true).count(),"rows":rows}),
        )?;
    }
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 4
        || args.len() > 9
        || ![
            "diagnostic",
            "diagnostic-placement",
            "fit",
            "evaluate",
            "stress",
            "fresh",
        ]
        .contains(&args[1].as_str())
        || (args.len() > 4 && args[1] != "fit")
        || (args[1] == "fit" && args.len() < 6)
    {
        return Err("usage: native_mixed_operators <diagnostic|diagnostic-placement|fit|evaluate|stress|fresh> MODEL OUT [PRESERVATION_CONTEXTS_JSON PRESERVATION_PROMPTS_JSON [LITERAL_STOP_REPORT_JSON [COMPOSED_PANEL_DIR [NUMERIC_REPORT_JSON]]]] (first two required for fit)".into());
    }
    let out = Path::new(&args[3]);
    fs::create_dir_all(out)?;
    let cases = cases(&args[1])?;
    let docs: Vec<_> = cases.iter().map(|c| c.input.clone()).collect();
    save(&out.join("cases.json"), &cases)?;
    save(&out.join("source.json"), &docs)?;
    let source_bytes = fs::read(out.join("source.json"))?;
    save(
        &out.join("source.receipt.json"),
        &json!({"mode":args[1],"cases":docs.len(),"blake3":blake3::hash(&source_bytes).to_hex().to_string(),"scope":"Authored labels for offline fitting and exact comparison only"}),
    )?;
    let model = Model::from_bytes(&fs::read(&args[2])?)?;
    if args[1] == "fit" {
        let mut preservation: Vec<OperationTransitionExample> = Vec::new();
        let mut sources = Vec::new();
        for input in args.get(4).into_iter() {
            let bytes = fs::read(input)?;
            let prior: Vec<OperationTransitionExample> = serde_json::from_slice(&bytes)?;
            sources.push(json!({"path":input,"bytes":bytes.len(),"blake3":blake3::hash(&bytes).to_hex().to_string(),"documents":prior.len()}));
            preservation.extend(prior);
        }
        if let Some(path) = args.get(6) {
            let bytes = fs::read(path)?;
            let report: Value = serde_json::from_slice(&bytes)?;
            let rows = report["cases"]
                .as_array()
                .ok_or("literal Stop rows absent")?;
            if report["artifact"] != model.artifact_cid()
                || report["exact"] != report["total"]
                || report["total"].as_u64() != Some(rows.len() as u64)
            {
                return Err("literal Stop baseline identity or qualification differs".into());
            }
            for row in rows {
                let d = &row["input"];
                if d["literal_only"] != true
                    || d["initial_prompt"] != ""
                    || row["exact"] != true
                    || row["terminated"] != true
                    || row["text"] != row["expected"]
                    || d["query"] != row["query"]
                {
                    return Err("literal Stop source is not a qualified independent prompt".into());
                }
                preservation.push(OperationTransitionExample {
                    id: format!(
                        "preserve-literal-stop/{}",
                        row["id"].as_str().ok_or("literal Stop ID absent")?
                    ),
                    history: vec![],
                    query: d["query"]
                        .as_str()
                        .ok_or("literal Stop query absent")?
                        .into(),
                    first_response: row["text"]
                        .as_str()
                        .ok_or("literal Stop text absent")?
                        .into(),
                    next: None,
                });
            }
            sources.push(json!({"path":path,"bytes":bytes.len(),"blake3":blake3::hash(&bytes).to_hex().to_string(),"documents":rows.len(),"scope":"Complete qualified parent literal-answer panel converted to Stop construction in Rust"}));
        }
        if let Some(path) = args.get(7) {
            let case_bytes = fs::read(Path::new(path).join("cases.json"))?;
            let report_bytes = fs::read(Path::new(path).join("result.json"))?;
            let cases: Vec<Value> = serde_json::from_slice(&case_bytes)?;
            let report: Value = serde_json::from_slice(&report_bytes)?;
            let rows = report["rows"]
                .as_array()
                .ok_or("composed preservation rows absent")?;
            if report["artifact"] != model.artifact_cid()
                || report["exact"] != report["total"]
                || report["total"].as_u64() != Some(cases.len() as u64)
                || rows.len() != cases.len()
            {
                return Err("composed preservation identity or population differs".into());
            }
            for case in &cases {
                let mut d: OperationTransitionExample =
                    serde_json::from_value(case["transition"].clone())?;
                let row = rows
                    .iter()
                    .find(|row| row["id"] == d.id)
                    .ok_or("composed preservation row absent")?;
                if row["exact"] != true
                    || row["eos"] != true
                    || row["text"] != case["expected"]
                    || row["query"] != d.query
                    || d.history.len() != 1
                {
                    return Err(
                        "composed preservation is not a qualified one-history response".into(),
                    );
                }
                let independent = d.history[0].clone();
                d.history.push(TypedRoutingTurn {
                    prompt: d.query.clone(),
                    response: row["text"]
                        .as_str()
                        .ok_or("composed response absent")?
                        .into(),
                });
                d.id = format!("preserve-followup/{}", d.id);
                d.query = independent.prompt;
                d.first_response = independent.response;
                d.next = None;
                preservation.push(d);
            }
            sources.push(json!({"path":path,"case_blake3":blake3::hash(&case_bytes).to_hex().to_string(),"report_blake3":blake3::hash(&report_bytes).to_hex().to_string(),"documents":cases.len(),"scope":"Complete prior composed panel with actual full history followed by its original independent sum; expected text checked after free generation only"}));
        }
        if let Some(path) = args.get(8) {
            let bytes = fs::read(path)?;
            let report: Value = serde_json::from_slice(&bytes)?;
            let rows = report["cases"]
                .as_array()
                .ok_or("numeric preservation rows absent")?;
            if report["artifact"] != model.artifact_cid()
                || report["exact"] != report["total"]
                || report["total"].as_u64() != Some(rows.len() as u64)
            {
                return Err("numeric preservation identity or qualification differs".into());
            }
            for row in rows {
                let d = &row["input"];
                if row["exact"] != true
                    || row["terminated"] != true
                    || row["text"] != row["expected"]
                    || row["text"] != d["response"]
                    || row["query"] != d["query"]
                {
                    return Err("numeric preservation row is not qualified".into());
                }
                preservation.push(OperationTransitionExample {
                    id: format!(
                        "preserve-numeric/{}",
                        row["id"].as_str().ok_or("numeric ID absent")?
                    ),
                    history: vec![TypedRoutingTurn {
                        prompt: d["initial_prompt"]
                            .as_str()
                            .ok_or("numeric history prompt absent")?
                            .into(),
                        response: d["initial_response"]
                            .as_str()
                            .ok_or("numeric history response absent")?
                            .into(),
                    }],
                    query: d["query"].as_str().ok_or("numeric query absent")?.into(),
                    first_response: row["text"]
                        .as_str()
                        .ok_or("numeric response absent")?
                        .into(),
                    next: None,
                });
            }
            sources.push(json!({"path":path,"blake3":blake3::hash(&bytes).to_hex().to_string(),"documents":rows.len(),"scope":"Complete qualified parent numeric panel with actual generated history; preserves prior Copy/Add selection"}));
        }
        save(&out.join("preservation.json"), &preservation)?;
        let preservation_bytes = fs::read(out.join("preservation.json"))?;
        let receipt = json!({"sources":sources,"documents":preservation.len(),"output_blake3":blake3::hash(&preservation_bytes).to_hex().to_string(),"scope":"Named already-open preservation inputs; offline fitting only"});
        save(&out.join("preservation.receipt.json"), &receipt)?;
        evaluate(
            &model,
            &cases,
            Control::Full,
            &out.join("parent-result.json"),
        )?;
        let prompt_path = args.get(5).ok_or("fit requires preserved prompt JSON")?;
        let prompt_bytes = fs::read(prompt_path)?;
        let prompts: Vec<Document> = serde_json::from_slice(&prompt_bytes)?;
        save(
            &out.join("initial-preservation.receipt.json"),
            &json!({"path":prompt_path,"blake3":blake3::hash(&prompt_bytes).to_hex().to_string(),"count":prompts.len()}),
        )?;
        let (candidate, mut fit) = model.fit_mixed_operators(
            &docs,
            &preservation,
            &prompts,
            SourceRoutingConfig {
                learned_features: 768,
                passes: 8,
                proposals: 24,
                max_seconds: 120,
                seed: 1140,
                mode: RoutingMode::Angular,
                ..Default::default()
            },
            |binding, report| {
                fs::write(out.join("binding-model.json"), binding.to_bytes()?)
                    .map_err(|e| uor_r4_core::native_geometric::Error(e.to_string()))?;
                fs::write(
                    out.join("binding-fit.json"),
                    serde_json::to_vec_pretty(report)
                        .map_err(|e| uor_r4_core::native_geometric::Error(e.to_string()))?,
                )
                .map_err(|e| uor_r4_core::native_geometric::Error(e.to_string()))
            },
        )?;
        fit["preservation_input"] = receipt;
        fs::write(out.join("model.json"), candidate.to_bytes()?)?;
        save(&out.join("fit.json"), &fit)?;
        evaluate(&candidate, &cases, Control::Full, &out.join("result.json"))?;
    } else {
        evaluate(&model, &cases, Control::Full, &out.join("result.json"))?;
        if args[1] == "evaluate" || args[1] == "fresh" {
            for (name, control) in [
                ("mixed-disabled", Control::MixedOperatorsDisabled),
                ("initial-disabled", Control::MixedInitialDisabled),
                ("transition-disabled", Control::MixedTransitionDisabled),
                ("operation-disabled", Control::OperationTransitionDisabled),
                (
                    "intermediate-disabled",
                    Control::OperationTransitionIntermediateDisabled,
                ),
            ] {
                evaluate(&model, &cases, control, &out.join(format!("{name}.json")))?;
            }
        }
    }
    Ok(())
}
