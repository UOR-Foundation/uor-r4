//! Actual-output Copy-to-Add diagnostic with exact occurrence and read checks.
//! Unary Copy text targets are authored evaluation labels, never serving templates.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{
    ActionEmissionExample, ActionEmissionSegment, Control, Document, EmissionPiece,
    MixedOperatorExample, MixedOperatorTarget, Model, OperationTransitionExample, RoutingMode,
    Session, SourceRoutingConfig, TypedRoutingExample, TypedRoutingTurn, ValueAction, BOS, EOS,
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
    let added = sum.checked_add(extra).ok_or("Add result overflow")?;
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
        for (label, copy_instruction, continuation, source_id, copied, second) in [
            (
                "copy-original-only",
                "Copy the original total.",
                "",
                2,
                sum,
                None,
            ),
            (
                "copy-original-add-extra",
                "Copy the original total.",
                " Add the extra to the copied result.",
                2,
                sum,
                Some(([3, 4], [extra, sum])),
            ),
            ("copy-extra-only", "Copy the extra.", "", 3, extra, None),
            (
                "copy-extra-add-original",
                "Copy the extra.",
                " Add the original total to the copied result.",
                3,
                extra,
                Some(([2, 4], [sum, extra])),
            ),
        ] {
            let mut expected_writes = vec![ExpectedWrite {
                id: 4,
                action: ValueAction::Copy,
                value: copied,
                operand_ids: [source_id, source_id],
                operand_values: [copied, copied],
            }];
            if let Some((operand_ids, operand_values)) = second {
                expected_writes.push(ExpectedWrite {
                    id: 5,
                    action: ValueAction::Add,
                    value: added,
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
                query: format!("User: There are {extra} extra coins. {copy_instruction}{continuation}{format_request}\nAssistant:"),
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
        return group("copy-add-diagnostic", 13, 4, 3, ["suri", "orin"], false);
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
            "construction" | "fit-binding" | "fit-transition" => (
                vec![[13, 4, 3], [14, 5, 4], [15, 6, 5]],
                ["suri", "orin"],
                false,
            ),
            "evaluate" => (vec![[36, 7, 6], [37, 8, 7]], ["ada", "cyra"], false),
            "stress" => (vec![[56, 9, 8], [57, 10, 67]], ["suri", "orin"], true),
            _ => return Err("unknown case population".into()),
        };
        (groups, names.map(str::to_owned), reverse)
    };
    let mut rows = Vec::new();
    for (i, [a, b, extra]) in numbers.into_iter().enumerate() {
        rows.extend(group(
            &format!("copy-add-{mode}-{i}"),
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
    // Compact report JSON retains all checkpoint fields and byte values. Model
    // artifact serialization still uses Model::to_bytes and is unchanged.
    fs::write(path, serde_json::to_vec(value)?)?;
    Ok(())
}
fn state(session: &Session) -> Result<Value> {
    Ok(serde_json::from_slice(&session.checkpoint()?)?)
}
fn generate(model: &Model, session: &mut Session, capture_checkpoints: bool) -> Result<Value> {
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
            let mut step = json!({"position":positions,"token":prediction.token,
                "anchor":actual["completion"]["anchor"],"lexical_read":actual["completion"]["lexical_read"]});
            if capture_checkpoints {
                step["checkpoint"] = actual.clone();
            }
            trace.push(step);
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
        "writes":writes,"reads":reads,"trace":trace,"initial_checkpoint":initial,"final_checkpoint":committed,"committed_records":records,"committed_write_count":count,
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
fn evaluate_case(
    model: &Model,
    case: &Case,
    control: Control,
    capture_checkpoints: bool,
) -> Result<Value> {
    let mut session = model.session(Control::Full)?;
    session.observe(model, BOS)?;
    for token in model.encode(&case.history_prompt)? {
        session.observe(model, token)?;
    }
    session.begin_response(model)?;
    let history = generate(model, &mut session, capture_checkpoints)?;
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
    let actual = generate(model, &mut session, capture_checkpoints)?;
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
    if control == Control::OperationTransitionIntermediateDisabled {
        let expected = case
            .expected_writes
            .first()
            .ok_or("first Copy target absent")?;
        let (pending_first, committed_first) =
            check_writes(&row["actual"], std::slice::from_ref(expected))?;
        let expected_reads: Vec<_> = case
            .expected_reads
            .iter()
            .copied()
            .filter(|(anchor, _)| *anchor == expected.id)
            .collect();
        let expected_text = rendered(&case.style, expected);
        let copied_record_retained =
            row["actual"]["committed_records"]
                .as_array()
                .is_some_and(|records| {
                    records
                        .iter()
                        .any(|r| r["id"] == expected.id && r["value"] == expected.value)
                });
        let intervention_exact = pending_first
            && committed_first
            && reads == expected_reads
            && row["actual"]["text"] == expected_text
            && row["actual"]["eos"] == true
            && row["actual"]["verification_error"].is_null()
            && copied_record_retained;
        row["intervention"] = json!({"expected_text":expected_text,"expected_writes":[expected],"expected_reads":expected_reads,"copied_record_retained":copied_record_retained,"exact":intervention_exact,"scope":"Exclude the copied result from next-operation proposals; preserve the first Copy and Stop. This is candidate-availability robustness, not memory erasure."});
    }
    if exact && control == Control::Full {
        session.end_response(model)?;
        for token in model.encode(&case.history_prompt)? {
            session.observe(model, token)?;
        }
        session.begin_response(model)?;
        let mut followup = generate(model, &mut session, capture_checkpoints)?;
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
fn evaluate(
    model: &Model,
    cases: &[Case],
    control: Control,
    path: &Path,
    capture_checkpoints: bool,
) -> Result<()> {
    let mut rows = Vec::new();
    for case in cases {
        rows.push(
            match evaluate_case(model, case, control, capture_checkpoints) {
                Ok(row) => row,
                Err(error) => json!({"id":case.id,"error":error.to_string(),"exact":false}),
            },
        );
        save(
            path,
            &json!({"artifact":model.artifact_cid(),"control":control,
            "total":cases.len(),"completed":rows.len(),"exact":rows.iter().filter(|r|r["exact"] == true).count(),"intervention_exact":if control == Control::OperationTransitionIntermediateDisabled {Some(rows.iter().filter(|r|r["intervention"]["exact"] == true).count())} else {None},"rows":rows,
            "scope":"Authored Copy/Add formatting with exact writes, reads and independent next-turn records; not general prose or program synthesis"}),
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
fn prepare_preservation(prior: &str, action_report: &str, output: &Path) -> Result<()> {
    let prior_bytes = fs::read(prior)?;
    let report_bytes = fs::read(action_report)?;
    let mut contexts: Vec<OperationTransitionExample> = serde_json::from_slice(&prior_bytes)?;
    if contexts.len() != 459 {
        return Err("expected complete 459-context preservation input".into());
    }
    let report: Value = serde_json::from_slice(&report_bytes)?;
    let rows = report["rows"]
        .as_array()
        .ok_or("prior action rows absent")?;
    if report["control"] != json!(Control::Full)
        || report["artifact"]
            != "blake3:5ed24f4e7487b6f9cb7fb762fc8bcc9092152f40cb756d864a82880d08ca867d"
        || rows.len() != 36
        || report["total"] != 36
        || report["completed"] != 36
        || report["exact"] != 36
    {
        return Err(
            "prior action construction report is incomplete or has wrong artifact/control".into(),
        );
    }
    let mut ids = std::collections::BTreeSet::new();
    if contexts
        .iter()
        .any(|d| d.id.trim().is_empty() || !ids.insert(d.id.clone()))
    {
        return Err("duplicate or empty preservation ID".into());
    }
    for row in rows {
        let case: Case = serde_json::from_value(row["case"].clone())?;
        let actual = &row["actual"];
        if row["exact"] != true
            || row["id"] != case.id
            || actual["eos"] != true
            || !actual["verification_error"].is_null()
            || case
                .expected_text
                .as_ref()
                .is_none_or(|t| actual["text"] != *t)
            || row["history"]["text"] != case.history_expected
            || row["history"]["eos"] != true
            || !row["history"]["verification_error"].is_null()
            || row["independent_followup"]["exact"] != true
        {
            return Err(
                "prior action row is not a qualified freely generated response/history".into(),
            );
        }
        let (pending, committed) = check_writes(actual, &case.expected_writes)?;
        if !pending || !committed || row["reads_exact"] != true {
            return Err("prior action exact record qualification differs".into());
        }
        let id = format!("prior-action/{}", case.id);
        if !ids.insert(id.clone()) {
            return Err("duplicate prior action preservation ID".into());
        }
        contexts.push(OperationTransitionExample {
            id,
            history: vec![TypedRoutingTurn {
                prompt: case.history_prompt,
                response: case.history_expected,
            }],
            query: case.query,
            first_response: actual["text"]
                .as_str()
                .ok_or("prior action generated text absent")?
                .into(),
            next: None,
        });
    }
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    save(output, &contexts)?;
    let output_bytes = fs::read(output)?;
    save(
        &output.with_extension("receipt.json"),
        &json!({
            "prior_input":{"path":prior,"bytes":prior_bytes.len(),"blake3":blake3::hash(&prior_bytes).to_hex().to_string(),"contexts":459},
            "prior_action_input":{"path":action_report,"bytes":report_bytes.len(),"blake3":blake3::hash(&report_bytes).to_hex().to_string(),"artifact":report["artifact"],"contexts":36},
            "output":{"path":output,"bytes":output_bytes.len(),"blake3":blake3::hash(&output_bytes).to_hex().to_string(),"contexts":contexts.len()},
            "scope":"Rust typed preparation preserves all previous context values/order and appends complete qualified freely generated action responses; no new Copy/Add histories injected"
        }),
    )
}

fn extend_preservation(prior: &str, chains: &str, prior_chains: &str, output: &Path) -> Result<()> {
    const PARENT: &str = "blake3:5ed24f4e7487b6f9cb7fb762fc8bcc9092152f40cb756d864a82880d08ca867d";
    let prior_bytes = fs::read(prior)?;
    let mut contexts: Vec<OperationTransitionExample> = serde_json::from_slice(&prior_bytes)?;
    if contexts.len() != 495 {
        return Err("expected complete 495-context preservation input".into());
    }
    let mut ids = std::collections::BTreeSet::new();
    if contexts
        .iter()
        .any(|d| d.id.trim().is_empty() || !ids.insert(d.id.clone()))
    {
        return Err("duplicate or empty previous preservation ID".into());
    }
    let mut source_ids = std::collections::BTreeSet::new();
    let mut sources = Vec::new();
    for path in [chains, prior_chains] {
        let bytes = fs::read(path)?;
        let report: Value = serde_json::from_slice(&bytes)?;
        let rows = report["cases"]
            .as_array()
            .ok_or("chain report cases absent")?;
        if report["artifact"] != PARENT
            || report["exact"] != 16
            || report["total"] != 16
            || rows.len() != 16
            || report["remove_intermediate_control"] != false
        {
            return Err(
                "chain report is incomplete, intervened, or belongs to wrong parent".into(),
            );
        }
        for row in rows {
            let input: TypedRoutingExample = serde_json::from_value(row["input"].clone())?;
            let expected_operands = serde_json::to_value(input.operands)?;
            // These historical numeric panels qualified commutative Add values.
            // Capture actual parent routing later; do not force their label order.
            let operands_match = row["actual_operands"] == expected_operands
                || (input.action == Some(ValueAction::Add)
                    && input
                        .operands
                        .is_some_and(|v| row["actual_operands"] == json!([v[1], v[0]])));
            if input.id.trim().is_empty()
                || !source_ids.insert(input.id.clone())
                || input.literal_only
                || input.initial_prompt.is_empty()
                || input.initial_response.is_empty()
                || input.query.is_empty()
                || input.response.is_empty()
                || !matches!(input.action, Some(ValueAction::Copy | ValueAction::Add))
                || input.operands.is_none()
                || input.target_intermediate > 1
                || (input.target_intermediate == 1 && input.continuation.is_none())
                || row["id"] != input.id
                || row["query"] != input.query
                || row["exact"] != true
                || row["terminated"] != true
                || row["text"] != input.response
                || row["expected"] != input.response
                || row["actual_action"] != serde_json::to_value(input.action)?
                || row["expected_action"] != serde_json::to_value(input.action)?
                || !operands_match
                || row["expected_operands"] != serde_json::to_value(input.operands)?
            {
                return Err("chain source row identity, labels or actual result differs".into());
            }
            let mut history = vec![TypedRoutingTurn {
                prompt: input.initial_prompt,
                response: input.initial_response,
            }];
            history.extend(input.continuation);
            history.extend(input.refresh);
            if history.len() > 4
                || history.iter().any(|h| {
                    h.prompt.is_empty()
                        || h.prompt.len() > 4096
                        || h.response.is_empty()
                        || h.response.len() > 128
                })
                || input.query.len() > 4096
                || input.response.len() > 128
            {
                return Err("chain context bounds differ".into());
            }
            let id = format!("preserved-chain/{}", input.id);
            if !ids.insert(id.clone()) {
                return Err("duplicate extended preservation ID".into());
            }
            contexts.push(OperationTransitionExample {
                id,
                history,
                query: input.query,
                first_response: input.response,
                next: None,
            });
        }
        sources.push(json!({"path":path,"bytes":bytes.len(),"blake3":blake3::hash(&bytes).to_hex().to_string(),"artifact":report["artifact"],"contexts":rows.len()}));
    }
    if contexts.len() != 527 {
        return Err("extended preservation count differs".into());
    }
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    save(output, &contexts)?;
    let output_bytes = fs::read(output)?;
    save(
        &output.with_extension("receipt.json"),
        &json!({
            "prior_input":{"path":prior,"bytes":prior_bytes.len(),"blake3":blake3::hash(&prior_bytes).to_hex().to_string(),"contexts":495},
            "chain_inputs":sources,
            "output":{"path":output,"bytes":output_bytes.len(),"blake3":blake3::hash(&output_bytes).to_hex().to_string(),"contexts":contexts.len()},
            "scope":"Rust typed preparation preserves previous context values/order and appends all32 qualified original-parent chain contexts including initial, continuation and refresh history. Report JSON is compact without omitted fields; model serialization is unchanged."
        }),
    )
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).is_some_and(|m| m == "extend-preservation") {
        if args.len() != 6 {
            return Err("usage: native_copy_add extend-preservation PRIOR_CONTEXTS_JSON CHAINS_REPORT_JSON PRIOR_CHAINS_REPORT_JSON OUTPUT_JSON".into());
        }
        return extend_preservation(&args[2], &args[3], &args[4], Path::new(&args[5]));
    }
    if args.get(1).is_some_and(|m| m == "prepare-preservation") {
        if args.len() != 5 {
            return Err("usage: native_copy_add prepare-preservation PRIOR_PRESERVATION_JSON PRIOR_ACTION_FULL_RESULT_JSON OUTPUT_JSON".into());
        }
        return prepare_preservation(&args[2], &args[3], Path::new(&args[4]));
    }
    if args.len() < 4 {
        return Err("usage: native_copy_add <diagnostic|evaluate|stress|fresh|fit-binding|fit-transition|rust-source> MODEL_OR_REPORT OUT [FRESH_INPUT_JSON|PRESERVATION_JSON [PROMPTS_JSON]]".into());
    }
    let mode = args[1].as_str();
    let expected_args = match mode {
        "fit-binding" => 6,
        "fresh" | "fit-transition" => 5,
        "diagnostic" | "evaluate" | "stress" | "rust-source" => 4,
        _ => return Err("unknown Copy/Add mode".into()),
    };
    if args.len() != expected_args {
        return Err("incorrect Copy/Add arguments".into());
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
        "scope":"Authored labels for offline typed binding/transition fitting and comparison; lexical suffixes are never forced by these fitting modes; free generation is separate acceptance"}),
    )?;
    let model = Model::from_bytes(&fs::read(&args[2])?)?;
    let model = if matches!(mode, "fit-binding" | "fit-transition") {
        let preservation: Vec<OperationTransitionExample> =
            serde_json::from_slice(&fs::read(&args[4])?)?;
        save(&out.join("preservation-training.json"), &preservation)?;
        let config = SourceRoutingConfig {
            learned_features: 768,
            passes: 8,
            proposals: 120,
            max_seconds: 120,
            mode: RoutingMode::Angular,
            seed: std::env::var("R4_COPY_ADD_FIT_SEED")
                .ok()
                .map(|s| s.parse::<u64>())
                .transpose()?
                .unwrap_or(1140),
            role_context_only: false,
        };
        let (candidate, fit) = if mode == "fit-binding" {
            let prompts: Vec<Document> = serde_json::from_slice(&fs::read(&args[5])?)?;
            model.fit_shared_operator_binding(&binding_docs, &preservation, &prompts, config)?
        } else {
            model.refine_shared_operators(&binding_docs, &preservation, config)?
        };
        // Seal the completed stage before any diagnostic or acceptance evaluation.
        fs::write(out.join("model.json"), candidate.to_bytes()?)?;
        save(&out.join("fit.json"), &fit)?;
        candidate
    } else {
        model
    };
    evaluate(
        &model,
        &cases,
        Control::Full,
        &out.join("result.json"),
        mode == "diagnostic",
    )?;
    if matches!(
        mode,
        "evaluate" | "stress" | "fresh" | "fit-binding" | "fit-transition"
    ) {
        for (name, control) in [
            (
                "shared-binding-disabled",
                Control::SharedOperatorBindingDisabled,
            ),
            (
                "shared-transition-disabled",
                Control::SharedOperatorTransitionDisabled,
            ),
            (
                "copied-result-excluded",
                Control::OperationTransitionIntermediateDisabled,
            ),
            (
                "action-context-disabled",
                Control::LexicalActionContextDisabled,
            ),
            ("record-read-disabled", Control::LexicalRecordReadDisabled),
            (
                "emission-geometry-disabled",
                Control::LexicalEmissionGeometryDisabled,
            ),
        ] {
            evaluate(
                &model,
                &cases,
                control,
                &out.join(format!("{name}.json")),
                false,
            )?;
        }
    }
    Ok(())
}
