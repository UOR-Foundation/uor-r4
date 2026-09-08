//! Offline composition construction and freely generated exact-ID evaluation.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{
    Control, Model, OperationTransitionExample, RoutingMode, Session, SourceRoutingConfig,
    TypedRoutingExample, TypedRoutingTurn, ValueAction, BOS, EOS,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExpectedWrite {
    id: u64,
    value: i64,
    operands: [u64; 2],
    operand_values: [i64; 2],
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Case {
    transition: OperationTransitionExample,
    style: String,
    expected: String,
    expected_writes: Vec<ExpectedWrite>,
    expected_reads: Vec<(u64, u64)>,
}
fn line(style: &str, value: i64, left: i64, right: i64) -> String {
    match style {
        "sentence" => format!("{value} is {left} plus {right}.\n"),
        "rust" => format!("{value} == {left} + {right}\n"),
        _ => format!("{value}.\n"),
    }
}
fn cases(split: &str, groups: &[(i64, i64, i64)], names: [&str; 2], reverse: bool) -> Vec<Case> {
    let mut rows = Vec::new();
    for (g, &(a, b, extra)) in groups.iter().enumerate() {
        let [name_a, name_b] = names;
        let facts = if reverse {
            format!("{name_b} has {b} coins. {name_a} has {a} coins.")
        } else {
            format!("{name_a} has {a} coins. {name_b} has {b} coins.")
        };
        let sum = a + b;
        let first = sum + extra;
        let second = first + extra;
        let history = format!(
            "User: {facts}\nUser: What is the sum of {name_a}'s and {name_b}'s coins?\nAssistant:"
        );
        for (label, before, after, style) in [
            ("plain", "", "", "plain"),
            ("sentence-prefix", "sentence. ", "", "sentence"),
            ("rust-prefix", "Rust. ", "", "rust"),
            ("sentence-suffix", "", " sentence.", "sentence"),
            ("rust-suffix", "", " Rust.", "rust"),
            (
                "ordinary-sentence",
                "",
                " Explain in a sentence.",
                "sentence",
            ),
            ("ordinary-rust", "", " Write a Rust equality.", "rust"),
        ] {
            for again in [false, true] {
                let first_response = line(style, first, extra, sum);
                let mut expected = first_response.clone();
                let mut writes = vec![ExpectedWrite {
                    id: 4,
                    value: first,
                    operands: [3, 2],
                    operand_values: [extra, sum],
                }];
                let mut reads = if style == "plain" {
                    vec![]
                } else {
                    vec![(4, 3), (4, 2)]
                };
                if again {
                    expected.push_str(&line(style, second, extra, first));
                    writes.push(ExpectedWrite {
                        id: 5,
                        value: second,
                        operands: [3, 4],
                        operand_values: [extra, first],
                    });
                    if style != "plain" {
                        reads.extend([(5, 3), (5, 4)]);
                    }
                }
                rows.push(Case {
                    transition: OperationTransitionExample {
                        id: format!("{split}-{g}-{label}-{again}"),
                        history: vec![TypedRoutingTurn { prompt: history.clone(), response: format!("{sum}.\n") }],
                        query: format!("User: {before}There are {extra} extra coins. Add the extra coins to the original total.{}{after}\nAssistant:", if again { " Again." } else { "" }),
                        first_response,
                        next: again.then_some((ValueAction::Add, extra)),
                    },
                    style: style.into(), expected, expected_writes: writes, expected_reads: reads,
                });
            }
        }
    }
    rows
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
    let initial_derived_writes = session.work.values.derived_writes;
    let mut tokens = Vec::new();
    let mut writes = Vec::new();
    let mut reads = Vec::new();
    let mut last_read_start = None;
    let mut eos = false;
    let mut positions = 0;
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
            return Err("checkpoint/repeated prediction or pending decision differs".into());
        }
        if let Some(write) = value.filter(|w| w.cursor == 0) {
            writes.push(
                json!({"action":write.action,"write_id":write.write_id,"value":write.value,
                "operands":write.operands.map(|r| json!({"id":r.id,"value":r.value}))}),
            );
        }
        session.observe(model, prediction.token)?;
        restored.observe(model, prediction.token)?;
        let actual = state(session)?;
        let restored_state = state(&restored)?;
        if actual["values"] != restored_state["values"]
            || actual["completion"] != restored_state["completion"]
        {
            return Err("checkpoint committed values/completion differs".into());
        }
        positions += 1;
        if let Some(read) = actual["completion"]
            .get("lexical_read")
            .filter(|r| r.is_object())
        {
            let at = read["start_at"].as_u64().ok_or("read start absent")?;
            if last_read_start != Some(at) {
                if read["cursor"] != 1 {
                    return Err("new observed read cursor differs".into());
                }
                reads.push((
                    actual["completion"]["anchor"]["write_id"]
                        .as_u64()
                        .ok_or("read anchor absent")?,
                    read["record_id"].as_u64().ok_or("read identity absent")?,
                ));
                last_read_start = Some(at);
            }
        }
        if prediction.token == EOS {
            eos = true;
            break;
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
        tokens.push(prediction.token);
    }
    let committed = state(session)?;
    let records: Vec<_> = committed["values"]["records"]
        .as_array()
        .ok_or("committed value records absent")?
        .iter()
        .map(|record| {
            json!({"id":record["id"],"value":record["value"],
                "derived":record["derived"],"derivation":record["derivation"]})
        })
        .collect();
    let committed_write_count = session
        .work
        .values
        .derived_writes
        .checked_sub(initial_derived_writes)
        .ok_or("committed derived write counter decreased")?;
    // ValueState omits this field when zero; EOS retains the response count.
    let operations_committed = committed["values"]["operations_committed"]
        .as_u64()
        .unwrap_or(0);
    Ok(
        json!({"text":String::from_utf8(model.decode(&tokens)?)?, "eos":eos,"writes":writes,"reads":reads,"checkpoint_positions":positions,"frozen_captured_input":true,
            "committed_records":records,"committed_write_count":committed_write_count,
            "operations_committed":operations_committed}),
    )
}
fn evaluate(model: &Model, cases: &[Case], control: Control) -> Result<Value> {
    let mut rows = Vec::new();
    for case in cases {
        let mut session = model.session(Control::Full)?;
        session.observe(model, BOS)?;
        for turn in &case.transition.history {
            for token in model.encode(&turn.prompt)? {
                session.observe(model, token)?;
            }
            session.begin_response(model)?;
            let history = generate(model, &mut session)?;
            if history["text"] != turn.response || history["eos"] != true {
                return Err(format!(
                    "actual history failed {}: {}",
                    case.transition.id, history["text"]
                )
                .into());
            }
            session.end_response(model)?;
        }
        // Intervention changes only the declared control after freely generated
        // history. It never injects expected response bytes or record values.
        if control != Control::Full {
            let mut checkpoint = state(&session)?;
            checkpoint["control"] = serde_json::to_value(control)?;
            session = model.restore_session(&serde_json::to_vec(&checkpoint)?)?;
        }
        for token in model.encode(&case.transition.query)? {
            session.observe(model, token)?;
        }
        session.begin_response(model)?;
        let mut row = generate(model, &mut session)?;
        let writes = row["writes"].as_array().ok_or("writes absent")?;
        let ordered = writes.len() == case.expected_writes.len()
            && writes
                .iter()
                .zip(&case.expected_writes)
                .all(|(actual, expected)| {
                    actual["action"] == "add"
                        && actual["write_id"] == expected.id
                        && actual["value"] == expected.value
                        && actual["operands"][0]["id"] == expected.operands[0]
                        && actual["operands"][1]["id"] == expected.operands[1]
                        && actual["operands"][0]["value"] == expected.operand_values[0]
                        && actual["operands"][1]["value"] == expected.operand_values[1]
                });
        let records = row["committed_records"]
            .as_array()
            .ok_or("committed records absent")?;
        let committed_exact = row["committed_write_count"] == case.expected_writes.len()
            && row["operations_committed"] == case.expected_writes.len()
            && case.expected_writes.iter().all(|expected| {
                let mut matches = records.iter().filter(|record| record["id"] == expected.id);
                let Some(record) = matches.next() else {
                    return false;
                };
                matches.next().is_none()
                    && record["value"] == expected.value
                    && record["derived"] == true
                    && record["derivation"]["action"] == "add"
                    && record["derivation"]["operand_ids"][0] == expected.operands[0]
                    && record["derivation"]["operand_ids"][1] == expected.operands[1]
                    && record["derivation"]["operand_values"][0] == expected.operand_values[0]
                    && record["derivation"]["operand_values"][1] == expected.operand_values[1]
            });
        let exact = row["text"] == case.expected
            && row["eos"] == true
            && ordered
            && committed_exact
            && row["reads"] == serde_json::to_value(&case.expected_reads)?;
        row["id"] = case.transition.id.clone().into();
        row["query"] = case.transition.query.clone().into();
        row["style"] = case.style.clone().into();
        row["again"] = case.transition.next.is_some().into();
        row["expected"] = case.expected.clone().into();
        row["expected_writes"] = serde_json::to_value(&case.expected_writes)?;
        row["expected_reads"] = serde_json::to_value(&case.expected_reads)?;
        row["ordered_writes_exact"] = ordered.into();
        row["committed_writes_exact"] = committed_exact.into();
        row["exact"] = exact.into();
        rows.push(row);
    }
    Ok(
        json!({"artifact":model.artifact_cid(),"control":control,"total":rows.len(),"exact":rows.iter().filter(|r|r["exact"]==true).count(),"rows":rows}),
    )
}
fn rust_source(input: &Path, out: &Path) -> Result<()> {
    let bytes = fs::read(input)?;
    let report: Value = serde_json::from_slice(&bytes)?;
    let mut program = String::from("fn main() {\n");
    let mut count = 0;
    for row in report["rows"].as_array().ok_or("result rows absent")? {
        if row["style"] != "rust" {
            continue;
        }
        if row["eos"] != true {
            return Err("generated Rust response lacks EOS".into());
        }
        let text = row["text"].as_str().ok_or("generated text absent")?;
        if text.is_empty()
            || !text.ends_with('\n')
            || !text
                .bytes()
                .all(|b| b.is_ascii_digit() || b" +-\n=".contains(&b))
        {
            return Err("generated text outside numeric equality scope".into());
        }
        let id = row["id"].as_str().ok_or("case id absent")?;
        for (index, line) in text.lines().enumerate() {
            if line.is_empty() || line.matches("==").count() != 1 {
                return Err("generated equality line invalid".into());
            }
            program.push_str(&format!(
                "assert!({line}, {:?});\n",
                format!("{id}/{index}")
            ));
            count += 1;
        }
    }
    if count == 0 {
        return Err("no generated Rust equality lines".into());
    }
    program.push_str("}\n");
    fs::create_dir_all(out)?;
    fs::write(out.join("generated_checks.rs"), program)?;
    save(
        &out.join("source-receipt.json"),
        &json!({"source_result":input,"source_result_blake3":blake3::hash(&bytes).to_hex().to_string(),"artifact":report["artifact"],"expressions":count,"scope":"Actual generated equality lines embedded unchanged in Rust assert harness; compile and execute separately"}),
    )
}
fn preserve_training(root: &Path, out: &Path) -> Result<()> {
    let mut sources = Vec::new();
    let mut read = |path: &Path| -> Result<Value> {
        let bytes = fs::read(path)?;
        sources.push(json!({"path":path,"bytes":bytes.len(),"blake3":blake3::hash(&bytes).to_hex().to_string()}));
        Ok(serde_json::from_slice(&bytes)?)
    };
    let mut docs = Vec::new();
    for (label, path, count) in [
        (
            "prior-operations",
            root.join("prior-operations/cases.json"),
            24,
        ),
        (
            "operation-construction",
            root.parent()
                .ok_or("instruction root has no parent")?
                .join("uor-r4-operation-transition/fit-random/cases.json"),
            36,
        ),
    ] {
        let mut prior: Vec<OperationTransitionExample> = serde_json::from_value(read(&path)?)?;
        if prior.len() != count {
            return Err(format!("unexpected {label} population").into());
        }
        for d in &mut prior {
            d.id = format!("preserve/{label}/{}", d.id);
        }
        docs.extend(prior);
    }
    let lexical: Vec<uor_r4_core::native_geometric::EmissionExample> =
        serde_json::from_value(read(&root.join("fit-final/cases.json"))?)?;
    let result = read(&root.join("fit-final/result.json"))?;
    let parent = result["artifact"]
        .as_str()
        .ok_or("preserved parent identity absent")?
        .to_string();
    let rows = result["rows"]
        .as_array()
        .ok_or("prior instruction results absent")?;
    if lexical.len() != 72 || rows.len() != 72 {
        return Err("unexpected prior instruction population".into());
    }
    let mut indexed = std::collections::BTreeMap::new();
    for row in rows {
        let id = row["id"].as_str().ok_or("prior result id absent")?;
        if indexed.insert(id, row).is_some() {
            return Err("duplicate prior result id".into());
        }
    }
    for d in lexical {
        let row = indexed
            .get(d.id.as_str())
            .ok_or("prior instruction id unmatched")?;
        if row["query"] != d.query || row["exact"] != true || row["eos"] != true {
            return Err("prior instruction receipt differs or failed".into());
        }
        docs.push(OperationTransitionExample {
            id: format!("preserve/instruction/{}", d.id),
            history: d.history,
            query: d.query,
            first_response: row["text"]
                .as_str()
                .ok_or("prior actual text absent")?
                .to_string(),
            next: None,
        });
    }
    for (label, count) in [("prior-current-evaluate", 48), ("prior-current-stress", 16)] {
        let prior: Vec<TypedRoutingExample> =
            serde_json::from_value(read(&root.join(label).join("source.json"))?)?;
        if prior.len() != count {
            return Err(format!("unexpected {label} population").into());
        }
        for d in prior {
            let mut history = Vec::new();
            if !d.literal_only {
                history.push(TypedRoutingTurn {
                    prompt: d.initial_prompt,
                    response: d.initial_response,
                });
            }
            history.extend(d.continuation);
            history.extend(d.refresh);
            if history.len() > 4 {
                return Err("preserved history exceeds four turns".into());
            }
            docs.push(OperationTransitionExample {
                id: format!("preserve/{label}/{}", d.id),
                history,
                query: d.query,
                first_response: d.response,
                next: None,
            });
        }
    }
    if docs.len() != 196 {
        return Err("unexpected pre-computed preservation population".into());
    }
    let mut expected_write_counts = std::collections::BTreeMap::new();
    for d in &docs {
        if expected_write_counts
            .insert(d.id.clone(), 1 + usize::from(d.next.is_some()))
            .is_some()
        {
            return Err("duplicate preservation count identity".into());
        }
    }
    let computed = read(&root.join("preserve-final/preservation/computed-construction.json"))?;
    if computed["artifact"] != parent || computed["exact"] != 58 || computed["total"] != 58 {
        return Err("prior computed construction artifact or result differs".into());
    }
    for row in computed["cases"]
        .as_array()
        .ok_or("computed cases absent")?
    {
        let d: TypedRoutingExample = serde_json::from_value(row["input"].clone())?;
        if row["exact"] != true || row["terminated"] != true || row["text"] != d.response {
            return Err("prior computed response differs".into());
        }
        let expected_action: Option<ValueAction> = serde_json::from_value(
            row.get("expected_action")
                .ok_or("computed expected action absent")?
                .clone(),
        )?;
        if expected_action != d.action || row["actual_action"] != row["expected_action"] {
            return Err("computed action provenance differs".into());
        }
        let id = format!("preserve/computed/{}", d.id);
        if expected_write_counts
            .insert(id.clone(), usize::from(expected_action.is_some()))
            .is_some()
        {
            return Err("duplicate computed preservation count identity".into());
        }
        let mut history = Vec::new();
        if !d.literal_only {
            history.push(TypedRoutingTurn {
                prompt: d.initial_prompt,
                response: d.initial_response,
            });
        }
        history.extend(d.continuation);
        history.extend(d.refresh);
        docs.push(OperationTransitionExample {
            id,
            history,
            query: d.query,
            first_response: d.response,
            next: None,
        });
    }
    let mut ids = std::collections::BTreeSet::new();
    if docs.len() != 254 || docs.iter().any(|d| !ids.insert(&d.id)) {
        return Err("preservation count or identity differs".into());
    }
    save(out, &docs)?;
    let output_blake3 = blake3::hash(&fs::read(out)?).to_hex().to_string();
    save(
        &out.with_extension("receipt.json"),
        &json!({"instruction_root":root,"parent_artifact":parent,"sources":sources,"documents":docs.len(),"output_blake3":output_blake3,"expected_write_counts":expected_write_counts,"scope":"Already-open parent panels, offline targets only; fitting must freely regenerate and check each history/first response. Write counts come from declared operation continuations and computed baseline expected_action, never response text."}),
    )
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if !(args.len() == 4 || args.len() == 5)
        || ![
            "fit",
            "evaluate",
            "stress",
            "fresh",
            "rust-source",
            "preserve-training",
            "replay-preservation",
        ]
        .contains(&args[1].as_str())
        || (args.len() == 5 && !["fit", "replay-preservation"].contains(&args[1].as_str()))
    {
        return Err("usage: native_composed_output <fit|evaluate|stress|fresh|rust-source> MODEL_OR_RESULT OUT [PRESERVE_CASES for fit]; preserve-training INSTRUCTION_ROOT OUT_JSON".into());
    }
    let out = Path::new(&args[3]);
    if args[1] == "rust-source" {
        return rust_source(Path::new(&args[2]), out);
    }
    if args[1] == "preserve-training" {
        return preserve_training(Path::new(&args[2]), out);
    }
    let model = Model::from_bytes(&fs::read(&args[2])?)?;
    fs::create_dir_all(out)?;
    if args[1] == "replay-preservation" {
        let input = args
            .get(4)
            .ok_or("replay-preservation requires input examples")?;
        let bytes = fs::read(input)?;
        let docs: Vec<OperationTransitionExample> = serde_json::from_slice(&bytes)?;
        if docs.is_empty() || docs.len() > 384 {
            return Err("invalid preservation population".into());
        }
        let receipt_path = Path::new(input).with_extension("receipt.json");
        let receipt_bytes = fs::read(&receipt_path)?;
        let receipt: Value = serde_json::from_slice(&receipt_bytes)?;
        if receipt["output_blake3"] != blake3::hash(&bytes).to_hex().to_string()
            || receipt["documents"] != docs.len()
        {
            return Err("preservation receipt is not bound to input documents".into());
        }
        let counts = receipt["expected_write_counts"]
            .as_object()
            .ok_or("declared preservation write counts absent")?;
        let mut ids = std::collections::BTreeSet::new();
        if counts.len() != docs.len() {
            return Err("preservation write count population differs".into());
        }
        for d in &docs {
            let count = counts
                .get(&d.id)
                .and_then(Value::as_u64)
                .ok_or("preservation write count identity absent or invalid")?;
            if !ids.insert(d.id.as_str()) || count > 2 || (count == 2) != d.next.is_some() {
                return Err("preservation write count identity or continuation differs".into());
            }
        }
        save(&out.join("cases.json"), &docs)?;
        let mut result = model.evaluate_operation_transition(&docs, Control::Full)?;
        result["legacy_exact"] = result["exact"].clone();
        let rows = result["rows"]
            .as_array_mut()
            .ok_or("preservation rows absent")?;
        let mut replayed = std::collections::BTreeSet::new();
        for row in rows.iter_mut() {
            let id = row["id"]
                .as_str()
                .ok_or("preservation row identity absent")?
                .to_string();
            let count = counts
                .get(&id)
                .and_then(Value::as_u64)
                .ok_or("preservation row identity undeclared")?;
            if !replayed.insert(id) {
                return Err("duplicate preservation replay row".into());
            }
            let decisions = row["decisions"]
                .as_array()
                .ok_or("preservation decisions absent")?;
            let count_exact = decisions.len() as u64 == count;
            let dependency = count_exact && (count != 2 || row["dependency"] == true);
            let exact = row["response"] == row["expected"] && row["eos"] == true && dependency;
            row["legacy_exact"] = row["exact"].clone();
            row["legacy_dependency"] = row["dependency"].clone();
            row["expected_write_count"] = count.into();
            row["write_count_exact"] = count_exact.into();
            row["dependency"] = dependency.into();
            row["exact"] = exact.into();
        }
        if replayed.len() != docs.len() {
            return Err("preservation replay population differs".into());
        }
        let exact = rows.iter().filter(|row| row["exact"] == true).count();
        result["exact"] = exact.into();
        result["preservation_receipt"] = json!({"path":receipt_path,"blake3":blake3::hash(&receipt_bytes).to_hex().to_string(),"documents_blake3":receipt["output_blake3"],"scope":"Exact response and EOS, declared baseline write count, and legacy dependency for two-write continuations"});
        save(&out.join("result.json"), &result)?;
        return Ok(());
    }
    let cases = match args[1].as_str() {
        "fit" => cases(
            "construction",
            &[(13, 4, 3), (14, 5, 4), (15, 6, 5)],
            ["suri", "orin"],
            false,
        ),
        "evaluate" => cases("open", &[(36, 7, 6), (37, 8, 7)], ["ada", "cyra"], false),
        "stress" => cases(
            "stress-reversed",
            &[(56, 9, 8), (57, 10, 9)],
            ["suri", "orin"],
            true,
        ),
        "fresh" => cases(
            "fresh",
            &[(122, 11, 10), (-47, 12, 11)],
            ["mira", "lena"],
            false,
        ),
        _ => unreachable!(),
    };
    save(&out.join("cases.json"), &cases)?;
    if args[1] == "fit" {
        save(
            &out.join("parent-result.json"),
            &evaluate(&model, &cases, Control::Full)?,
        )?;
        let mut construction: Vec<_> = cases.iter().map(|c| c.transition.clone()).collect();
        let mut preservation = Value::Null;
        if args.len() == 5 {
            let bytes = fs::read(&args[4])?;
            let prior: Vec<OperationTransitionExample> = serde_json::from_slice(&bytes)?;
            preservation = json!({"path":args[4],"count":prior.len(),"blake3":blake3::hash(&bytes).to_hex().to_string(),"scope":"Explicit already-open preservation construction, not fresh evaluation"});
            construction.extend(prior);
        }
        save(&out.join("construction.json"), &construction)?;
        let (candidate, mut fit) = model.fit_composed_output(
            &construction,
            SourceRoutingConfig {
                learned_features: 768,
                passes: 8,
                proposals: 24,
                max_seconds: 120,
                mode: RoutingMode::Angular,
                seed: 1140,
                ..Default::default()
            },
        )?;
        fit["preservation_input"] = preservation;
        fs::write(out.join("model.json"), candidate.to_bytes()?)?;
        save(&out.join("fit.json"), &fit)?;
        save(
            &out.join("result.json"),
            &evaluate(&candidate, &cases, Control::Full)?,
        )?;
    } else {
        save(
            &out.join("result.json"),
            &evaluate(&model, &cases, Control::Full)?,
        )?;
        if args[1] == "evaluate" {
            for (label, control) in [
                ("composed-disabled", Control::ComposedOutputDisabled),
                ("operation-disabled", Control::OperationTransitionDisabled),
                (
                    "intermediate-disabled",
                    Control::OperationTransitionIntermediateDisabled,
                ),
                ("lexical-disabled", Control::LexicalEmissionDisabled),
                ("read-disabled", Control::LexicalRecordReadDisabled),
                (
                    "lexical-geometry-disabled",
                    Control::LexicalEmissionGeometryDisabled,
                ),
            ] {
                save(
                    &out.join(format!("{label}.json")),
                    &evaluate(&model, &cases, control)?,
                )?;
            }
        }
    }
    Ok(())
}
