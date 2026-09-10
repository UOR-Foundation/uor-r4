//! Small actual-output diagnostic for mixed operations under existing format requests.
//! Unary Copy text targets are authored evaluation labels, never serving templates.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{
    ActionEmissionExample, ActionEmissionSegment, Control, Document, EmissionPiece,
    MixedOperatorExample, MixedOperatorTarget, Model, OperationTransitionExample, RoutingMode,
    Session, SourceRoutingConfig, TypedRoutingTurn, ValueAction, BOS, EOS,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Clone, Serialize, Deserialize)]
struct ExpectedWrite {
    id: u64,
    action: ValueAction,
    value: i64,
    operand_ids: [u64; 2],
    operand_values: [i64; 2],
}
#[derive(Clone, Serialize, Deserialize)]
struct Case {
    id: String,
    history_prompt: String,
    history_expected: String,
    query: String,
    style: String,
    expected_text: Option<String>,
    expected_writes: Vec<ExpectedWrite>,
    expected_reads: Vec<(u64, u64)>,
    expected_followup: ExpectedWrite,
}
fn rendered(style: &str, write: &ExpectedWrite) -> String {
    let ExpectedWrite {
        action,
        value,
        operand_values: [left, right],
        ..
    } = *write;
    match (style, action) {
        ("sentence", ValueAction::Add) => format!("{value} is {left} plus {right}.\n"),
        ("rust", ValueAction::Add) => format!("{value} == {left} + {right}\n"),
        ("sentence", ValueAction::Copy) => format!("{value} is {left}.\n"),
        ("rust", ValueAction::Copy) => format!("{value} == {left}\n"),
        _ => format!("{value}.\n"),
    }
}
fn group(
    split: &str,
    a: i64,
    b: i64,
    extra: i64,
    names: [&str; 2],
    reverse: bool,
) -> Result<Vec<Case>> {
    let sum = a.checked_add(b).ok_or("sum overflow")?;
    let first = sum.checked_add(extra).ok_or("first result overflow")?;
    let again = first.checked_add(extra).ok_or("second result overflow")?;
    let [left, right] = names;
    let facts = if reverse {
        format!("{right} has {b} coins. {left} has {a} coins.")
    } else {
        format!("{left} has {a} coins. {right} has {b} coins.")
    };
    let mut rows = Vec::new();
    for (style, format_request) in [
        ("plain", ""),
        ("sentence", " Explain in a sentence."),
        ("rust", " Write a Rust equality."),
    ] {
        for (label, suffix, second) in [
            ("add-only", "", None),
            (
                "copy-latest",
                " Copy the latest result.",
                Some((ValueAction::Copy, first, [4, 4], [first, first])),
            ),
            (
                "copy-original",
                " Copy the original total.",
                Some((ValueAction::Copy, sum, [2, 2], [sum, sum])),
            ),
            (
                "add-again",
                " Again.",
                Some((ValueAction::Add, again, [3, 4], [extra, first])),
            ),
        ] {
            let mut expected_writes = vec![ExpectedWrite {
                id: 4,
                action: ValueAction::Add,
                value: first,
                operand_ids: [3, 2],
                operand_values: [extra, sum],
            }];
            if let Some((action, value, operand_ids, operand_values)) = second {
                expected_writes.push(ExpectedWrite {
                    id: 5,
                    action,
                    value,
                    operand_ids,
                    operand_values,
                });
            }
            let expected_text = Some(expected_writes.iter().map(|w| rendered(style, w)).collect());
            let mut expected_reads = Vec::new();
            if style != "plain" {
                for w in &expected_writes {
                    expected_reads.push((w.id, w.operand_ids[0]));
                    if w.action == ValueAction::Add {
                        expected_reads.push((w.id, w.operand_ids[1]));
                    }
                }
            }
            let literal = 4 + expected_writes.len() as u64;
            rows.push(Case {
                id: format!("{split}/{style}/{label}"),
                history_prompt: format!("User: {facts}\nUser: What is the sum of {left}'s and {right}'s coins?\nAssistant:"),
                history_expected: format!("{sum}.\n"),
                query: format!("User: There are {extra} extra coins. Add the extra coins to the original total.{suffix}{format_request}\nAssistant:"),
                style: style.into(), expected_text, expected_writes, expected_reads,
                expected_followup: ExpectedWrite {
                    id: literal + 2, action: ValueAction::Add, value: sum,
                    operand_ids: [literal + 1, literal],
                    operand_values: if reverse { [a, b] } else { [b, a] },
                },
            });
        }
    }
    Ok(rows)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FreshInput {
    groups: Vec<[i64; 3]>,
    names: [String; 2],
    reverse: bool,
}
fn cases(mode: &str, fresh: Option<&str>) -> Result<Vec<Case>> {
    if mode == "diagnostic" {
        return group(
            "action-emission-diagnostic",
            13,
            4,
            3,
            ["suri", "orin"],
            false,
        );
    }
    let (numbers, names, reverse) = if mode == "fresh" {
        let input: FreshInput =
            serde_json::from_slice(&fs::read(fresh.ok_or("fresh input required")?)?)?;
        if input.groups.len() != 2
            || input
                .names
                .iter()
                .any(|n| n.is_empty() || n.len() > 32 || !n.bytes().all(|b| b.is_ascii_lowercase()))
        {
            return Err("fresh population bounds".into());
        }
        (input.groups, input.names, input.reverse)
    } else {
        let (groups, names, reverse) = match mode {
            "fit-binding" | "fit-emission" => (
                vec![[13, 4, 3], [14, 5, 4], [15, 6, 5]],
                ["suri", "orin"],
                false,
            ),
            "evaluate" | "binding-diagnostic" => {
                (vec![[36, 7, 6], [37, 8, 7]], ["ada", "cyra"], false)
            }
            "stress" => (vec![[56, 9, 8], [57, 10, 0]], ["suri", "orin"], true),
            _ => return Err("unknown case population".into()),
        };
        (groups, names.map(str::to_owned), reverse)
    };
    let mut rows = Vec::new();
    for (i, [a, b, extra]) in numbers.into_iter().enumerate() {
        rows.extend(group(
            &format!("action-emission-{mode}-{i}"),
            a,
            b,
            extra,
            [&names[0], &names[1]],
            reverse,
        )?);
    }
    Ok(rows)
}
fn context(case: &Case) -> OperationTransitionExample {
    OperationTransitionExample {
        id: case.id.clone(),
        history: vec![TypedRoutingTurn {
            prompt: case.history_prompt.clone(),
            response: case.history_expected.clone(),
        }],
        query: case.query.clone(),
        first_response: rendered(&case.style, &case.expected_writes[0]),
        next: None,
    }
}
fn binding_document(case: &Case) -> MixedOperatorExample {
    let target = |w: &ExpectedWrite| MixedOperatorTarget {
        action: w.action,
        operand_ids: w.operand_ids,
    };
    MixedOperatorExample {
        context: context(case),
        first: target(&case.expected_writes[0]),
        targets: case.expected_writes[1..].iter().map(target).collect(),
    }
}
fn emission_document(case: &Case) -> ActionEmissionExample {
    ActionEmissionExample {
        id: case.id.clone(),
        history: context(case).history,
        query: case.query.clone(),
        segments: case
            .expected_writes
            .iter()
            .map(|write| {
                let suffix = match case.style.as_str() {
                    "plain" => None,
                    style => {
                        let mut pieces = vec![
                            EmissionPiece::Text(
                                if style == "sentence" { " is " } else { " == " }.into(),
                            ),
                            EmissionPiece::Operand(0),
                        ];
                        if write.action == ValueAction::Add {
                            pieces.extend([
                                EmissionPiece::Text(
                                    if style == "sentence" { " plus " } else { " + " }.into(),
                                ),
                                EmissionPiece::Operand(1),
                            ]);
                        }
                        pieces.push(EmissionPiece::Text(
                            if style == "sentence" { ".\n" } else { "\n" }.into(),
                        ));
                        Some(pieces)
                    }
                };
                ActionEmissionSegment {
                    action: write.action,
                    operand_ids: write.operand_ids,
                    prefix: write.value.to_string(),
                    suffix,
                }
            })
            .collect(),
    }
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
    let mut reads = Vec::new();
    let mut trace = Vec::new();
    let mut last_read_start = None;
    let mut eos = false;
    let mut positions = 0;
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
            trace.push(json!({"position":positions,"token":prediction.token,
                "anchor":actual["completion"]["anchor"],"lexical_read":actual["completion"]["lexical_read"]}));
            if let Some(read) = actual["completion"]
                .get("lexical_read")
                .filter(|r| r.is_object())
            {
                let at = read["start_at"].as_u64().ok_or("read start absent")?;
                if last_read_start != Some(at) {
                    reads.push(
                        json!({"write_id":actual["completion"]["anchor"]["write_id"],
                        "record_id":read["record_id"],"start_at":at,"cursor":read["cursor"]}),
                    );
                    last_read_start = Some(at);
                }
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
        "writes":writes,"reads":reads,"trace":trace,"committed_records":records,"committed_write_count":count,
        "operations_committed":committed["values"]["operations_committed"].as_u64().unwrap_or(0),
        "checkpoint_positions":positions,"verification_error":error,"frozen_captured_input":error.is_none()}),
    )
}
fn check_writes(actual: &Value, expected: &[ExpectedWrite]) -> Result<(bool, bool)> {
    let pending = actual["writes"] == serde_json::to_value(expected)?;
    let records = actual["committed_records"]
        .as_array()
        .ok_or("committed records absent")?;
    let committed = actual["committed_write_count"] == expected.len()
        && actual["operations_committed"] == expected.len()
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
    for token in model.encode(&case.history_prompt)? {
        session.observe(model, token)?;
    }
    session.begin_response(model)?;
    let history = generate(model, &mut session)?;
    if history["text"] != case.history_expected
        || history["eos"] != true
        || !history["verification_error"].is_null()
    {
        return Ok(json!({"id":case.id,"history":history,"status":"history_failed","exact":false}));
    }
    session.end_response(model)?;
    if control != Control::Full {
        let mut checkpoint = state(&session)?;
        checkpoint["control"] = serde_json::to_value(control)?;
        session = model.restore_session(&serde_json::to_vec(&checkpoint)?)?;
    }
    for token in model.encode(&case.query)? {
        session.observe(model, token)?;
    }
    session.begin_response(model)?;
    let actual = generate(model, &mut session)?;
    let (pending, committed) = check_writes(&actual, &case.expected_writes)?;
    let first = actual["writes"].as_array().and_then(|w| w.first());
    let first_exact = match (first, case.expected_writes.first()) {
        (Some(actual), Some(expected)) => actual == &serde_json::to_value(expected)?,
        _ => false,
    };
    let text_exact = case
        .expected_text
        .as_ref()
        .map(|expected| actual["text"] == *expected);
    let reads = actual["reads"]
        .as_array()
        .ok_or("reads absent")?
        .iter()
        .map(|r| {
            Ok((
                r["write_id"].as_u64().ok_or("read anchor absent")?,
                r["record_id"].as_u64().ok_or("read record absent")?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let reads_exact = reads == case.expected_reads;
    let exact = pending
        && committed
        && reads_exact
        && text_exact == Some(true)
        && actual["eos"] == true
        && actual["verification_error"].is_null();
    let mut row = json!({"id":case.id,"case":case,"history":history,"actual":actual,
        "first_operation_exact":first_exact,"pending_writes_exact":pending,
        "committed_writes_exact":committed,"known_text_exact":text_exact,"reads_exact":reads_exact,"exact":exact});
    if exact && control == Control::Full {
        session.end_response(model)?;
        for token in model.encode(&case.history_prompt)? {
            session.observe(model, token)?;
        }
        session.begin_response(model)?;
        let mut followup = generate(model, &mut session)?;
        let (pending, committed) =
            check_writes(&followup, std::slice::from_ref(&case.expected_followup))?;
        let valid = pending
            && committed
            && followup["text"] == case.history_expected
            && followup["eos"] == true
            && followup["verification_error"].is_null();
        followup["expected"] = case.history_expected.clone().into();
        followup["expected_write"] = serde_json::to_value(&case.expected_followup)?;
        followup["exact"] = valid.into();
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
            Err(error) => json!({"id":case.id,"error":error.to_string(),"exact":false}),
        });
        save(
            path,
            &json!({"artifact":model.artifact_cid(),"control":control,
            "total":cases.len(),"completed":rows.len(),"exact":rows.iter().filter(|r|r["exact"] == true).count(),"rows":rows,
            "scope":"Authored mixed Add/Copy formatting with exact writes, reads and independent next-turn records; not general prose or program synthesis"}),
        )?;
    }
    Ok(())
}
fn rust_source(input: &str, out: &Path) -> Result<()> {
    let bytes = fs::read(input)?;
    let report: Value = serde_json::from_slice(&bytes)?;
    if report["control"] != json!(Control::Full) || report["completed"] != report["total"] {
        return Err("Rust source requires a completed Full report".into());
    }
    let mut count = 0;
    let mut program = String::from("fn main() {\n");
    for row in report["rows"].as_array().ok_or("result rows absent")? {
        if row["case"]["style"] != "rust" {
            continue;
        }
        if row["exact"] != true
            || row["actual"]["eos"] != true
            || !row["actual"]["verification_error"].is_null()
        {
            return Err("Rust source row is not qualified".into());
        }
        let text = row["actual"]["text"]
            .as_str()
            .ok_or("generated text absent")?;
        for line in text.lines() {
            if line.is_empty()
                || !line
                    .bytes()
                    .all(|b| b.is_ascii_digit() || b" -=+".contains(&b))
                || line.matches("==").count() != 1
            {
                return Err("generated line is not a bounded numeric equality".into());
            }
            program.push_str("    assert!(");
            program.push_str(line);
            program.push_str(");\n");
            count += 1;
        }
    }
    if count == 0 {
        return Err("no generated Rust equalities".into());
    }
    program.push_str("}\n");
    fs::write(out.join("generated_checks.rs"), program)?;
    save(
        &out.join("source-receipt.json"),
        &json!({"artifact":report["artifact"],"expressions":count,
        "source_result":input,"source_result_blake3":blake3::hash(&bytes).to_hex().to_string(),
        "scope":"Actual generated numeric equality lines embedded unchanged in Rust assert harness; compile and execute separately"}),
    )
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 4 {
        return Err("usage: native_action_emission MODE MODEL_OR_REPORT OUT [PRESERVATION_JSON PROMPTS_JSON MIXED_SOURCE_JSON|FRESH_INPUT_JSON]".into());
    }
    let mode = args[1].as_str();
    let expected_args = match mode {
        "fit-binding" => 7,
        "fit-emission" | "fresh" => 5,
        "diagnostic" | "binding-diagnostic" | "evaluate" | "stress" | "rust-source" => 4,
        _ => return Err("unknown action emission mode".into()),
    };
    if args.len() != expected_args {
        return Err("incorrect action emission arguments".into());
    }
    let out = Path::new(&args[3]);
    uor_r4_core::report_output::claim(out)?;
    if mode == "rust-source" {
        return rust_source(&args[2], out);
    }
    let cases = cases(mode, args.get(4).map(String::as_str))?;
    save(&out.join("cases.json"), &cases)?;
    let bytes = fs::read(out.join("cases.json"))?;
    let binding_docs: Vec<_> = cases.iter().map(binding_document).collect();
    let emission_docs: Vec<_> = cases.iter().map(emission_document).collect();
    save(&out.join("binding-source.json"), &binding_docs)?;
    save(&out.join("emission-source.json"), &emission_docs)?;
    let mut sources = Vec::new();
    for path in args.iter().skip(4) {
        let bytes = fs::read(path)?;
        sources.push(json!({"path":path,"bytes":bytes.len(),"blake3":blake3::hash(&bytes).to_hex().to_string()}));
    }
    save(
        &out.join("source.receipt.json"),
        &json!({"mode":mode,"cases":cases.len(),
        "blake3":blake3::hash(&bytes).to_hex().to_string(),"additional_sources":sources,
        "scope":"Authored labels for offline fitting and comparison only; actual history and first results must be generated"}),
    )?;
    let model = Model::from_bytes(&fs::read(&args[2])?)?;
    let config = SourceRoutingConfig {
        learned_features: 768,
        passes: 8,
        proposals: 120,
        max_seconds: 120,
        mode: RoutingMode::Angular,
        seed: std::env::var("R4_ACTION_FIT_SEED")
            .ok()
            .map(|s| s.parse::<u64>())
            .transpose()?
            .unwrap_or(1140),
        role_context_only: false,
    };
    let model = if mode == "fit-binding" || mode == "fit-emission" {
        let mut preservation: Vec<OperationTransitionExample> =
            serde_json::from_slice(&fs::read(&args[4])?)?;
        let (candidate, fit) = if mode == "fit-binding" {
            let prompts: Vec<Document> = serde_json::from_slice(&fs::read(&args[5])?)?;
            let mixed: Vec<MixedOperatorExample> = serde_json::from_slice(&fs::read(&args[6])?)?;
            if mixed.len() != 12 {
                return Err("expected complete 12-case retained mixed construction source".into());
            }
            preservation.extend(mixed.into_iter().map(|d| d.context));
            save(&out.join("preservation-training.json"), &preservation)?;
            model.fit_action_binding(&binding_docs, &preservation, &prompts, config)?
        } else {
            model.refine_action_emission(&emission_docs, &preservation, config)?
        };
        fs::write(out.join("model.json"), candidate.to_bytes()?)?;
        save(&out.join("fit.json"), &fit)?;
        candidate
    } else {
        model
    };
    evaluate(&model, &cases, Control::Full, &out.join("result.json"))?;
    if matches!(mode, "evaluate" | "stress" | "fresh" | "fit-emission") {
        for (name, control) in [
            ("action-binding-disabled", Control::ActionBindingDisabled),
            (
                "action-transition-disabled",
                Control::ActionTransitionDisabled,
            ),
            (
                "action-context-disabled",
                Control::LexicalActionContextDisabled,
            ),
            ("action-emission-disabled", Control::ActionEmissionDisabled),
            ("record-read-disabled", Control::LexicalRecordReadDisabled),
            (
                "emission-geometry-disabled",
                Control::LexicalEmissionGeometryDisabled,
            ),
        ] {
            evaluate(&model, &cases, control, &out.join(format!("{name}.json")))?;
        }
    }
    Ok(())
}
