//! Executable artifact/interface check. Usage: MODEL EXPECTED_CID OUTPUT_JSON.
//! Uses only the supplied artifact; no training, fixture model, or canned replies.
use serde_json::{json, Value};
use std::error::Error;
use std::path::Path;
use uor_r4_api::native_capability_api::{
    CompletionRequest, NativeApiError, NativeModel, NativeSession, SessionConfig,
};
use uor_r4_core::native_geometric::{Control, Model, Session, BOS, EOS};

type CheckResult<T> = Result<T, Box<dyn Error>>;
const LIMIT: usize = 96;
const QUIET_RIVER: &str = "the report says quiet river holds selra. Where is selra? Answer:";
const SUM_13: &str = "User: suri has 13 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:";
const SUM_14: &str = "User: suri has 14 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:";

fn record(checks: &mut Vec<Value>, name: &str, passed: bool, evidence: Value) -> CheckResult<()> {
    checks.push(
        json!({"name":name,"status":if passed { "PASS" } else { "FAIL" },"evidence":evidence}),
    );
    if passed {
        Ok(())
    } else {
        Err(format!("check failed: {name}").into())
    }
}

fn direct_turn(model: &Model, session: &mut Session, prompt: &str) -> CheckResult<Value> {
    if session.needs_input_boundary() {
        session.end_response(model)?;
    }
    if session.work.observed_tokens == 0 {
        session.observe(model, BOS)?;
    }
    for token in model.encode(prompt)? {
        session.observe(model, token)?;
    }
    session.begin_response(model)?;
    let mut tokens = Vec::new();
    let mut stopped_by = "length";
    for _ in 0..LIMIT {
        let token = session.predict(model)?.token;
        session.observe(model, token)?;
        if token == EOS {
            stopped_by = "eos";
            break;
        }
        tokens.push(token);
    }
    let bytes = model.decode(&tokens)?;
    let text = std::str::from_utf8(&bytes)?.to_owned();
    Ok(json!({"bytes":bytes,"text":text,"token_count":tokens.len(),"stopped_by":stopped_by}))
}

fn compare_turn(
    model: &Model,
    direct: &mut Session,
    api: &mut NativeSession,
    prompt: &str,
    target: &str,
    name: &str,
    checks: &mut Vec<Value>,
) -> CheckResult<()> {
    let expected = direct_turn(model, direct, prompt)?;
    let response = api.complete(CompletionRequest {
        prompt: prompt.into(),
        max_tokens: Some(LIMIT),
        temperature: Some(0.0),
        stop_sequences: vec![],
    })?;
    let parity = json!(response.text.as_bytes()) == expected["bytes"]
        && json!(response.token_count) == expected["token_count"]
        && json!(response.stopped_by) == expected["stopped_by"];
    record(
        checks,
        name,
        parity && response.text == target,
        json!({"prompt":prompt,"target":target,"direct":expected,"api":response,"byte_parity":parity}),
    )
}

fn rejection(bytes: &[u8]) -> (bool, Option<String>) {
    match Model::from_bytes(bytes) {
        Ok(_) => (false, None),
        Err(error) => (true, Some(error.to_string())),
    }
}

// The word witness restores lexical emission and separately enforces exact
// inherited lexical parameters. Existing legal bias mutations fail that guard;
// other component mutations remain in the reconstructed parent and fail there.
fn word_boundary<'a>(
    writer: bool,
    source_role: bool,
    present: bool,
    pointer: &str,
    nested: &'a str,
) -> &'a str {
    if writer {
        "writer lexical frozen parent differs"
    } else if source_role {
        "source role refinement frozen parent differs"
    } else if !present {
        nested
    } else if pointer.starts_with("/lexical_emission/") {
        "word emission frozen lexical parameters differ"
    } else {
        "word emission frozen parent differs"
    }
}

fn source_role_boundary(writer: bool, present: bool, nested: &str) -> &str {
    if writer {
        "writer lexical frozen parent differs"
    } else if present {
        "source role refinement frozen parent differs"
    } else {
        nested
    }
}

fn writer_boundary(present: bool, nested: &str) -> &str {
    if present {
        "writer lexical frozen parent differs"
    } else {
        nested
    }
}

fn writer_choice_checks(
    api_model: &NativeModel,
    document: &Value,
    checks: &mut Vec<Value>,
) -> CheckResult<Option<Model>> {
    let Some(witness) = document.get("writer_choice").filter(|v| v.is_object()) else {
        checks.push(json!({"name":"writer_choice_checks","status":"NOT_APPLICABLE","reason":"supplied artifact has no writer_choice witness"}));
        return Ok(None);
    };
    let model = api_model.inner_model();
    let parent = model.without_writer_choice()?;
    let parent_bytes = parent.to_bytes()?;
    let parent_document: Value = serde_json::from_slice(&parent_bytes)?;
    let expected_parent = "blake3:d1b0985fb8af0528dae6a7c684e1c76be3634099c868a7f9a3b09454cb68d1c0";
    let mut candidate_base = document.clone();
    let mut parent_base = parent_document.clone();
    for key in ["writer_choice", "artifact_cid", "uor_model_address"] {
        candidate_base
            .as_object_mut()
            .ok_or("candidate is not an object")?
            .remove(key);
        parent_base
            .as_object_mut()
            .ok_or("parent is not an object")?
            .remove(key);
    }
    record(
        checks,
        "writer_choice_exact_parent_and_frozen_components",
        witness["parent_artifact"] == expected_parent
            && parent.artifact_cid() == expected_parent
            && candidate_base == parent_base,
        json!({"candidate_artifact":model.artifact_cid(),"parent_artifact":parent.artifact_cid(),
            "parent_bytes":parent_bytes.len(),"parent_bytes_blake3":blake3::hash(&parent_bytes).to_hex().to_string(),
            "boundary":"Only the optional writer-choice residual and derived identities differ; all inherited writer, cache, field-composition and geometric components match the recursively validated parent."}),
    )?;
    for (label, padding) in [
        ("recent", String::new()),
        ("evicted", "oak ash elm ".repeat(40)),
    ] {
        let prompt = format!("Record: selvi in Dusk Ridge. selvi now in Copper Vale. {padding}Where is selvi? Name the owner first. Answer:");
        let config = SessionConfig {
            session_id: format!("writer-choice-{label}"),
            ..SessionConfig::default()
        };
        let mut direct = model.session(Control::Full)?;
        let mut api = api_model.create_session(config.clone())?;
        compare_turn(
            &model,
            &mut direct,
            &mut api,
            &prompt,
            " selvi is in Copper Vale.\n",
            &format!("writer_choice_{label}_revision_api_direct_parity"),
            checks,
        )?;
        let checkpoint: Value = serde_json::from_slice(&direct.checkpoint()?)?;
        let relations = &checkpoint["values"]["relations"];
        let records = relations["records"]
            .as_array()
            .ok_or("revision records absent")?;
        let old = records
            .iter()
            .find(|r| r["id"] == 1)
            .ok_or("old revision record absent")?;
        let new = records
            .iter()
            .find(|r| r["id"] == 2)
            .ok_or("new revision record absent")?;
        let atom = |v: &Value| -> CheckResult<String> {
            let len = v["len"].as_u64().ok_or("atom length absent")? as usize;
            let all = v["bytes"].as_array().ok_or("atom bytes absent")?;
            let bytes = all
                .get(..len)
                .ok_or("atom length overflow")?
                .iter()
                .map(|b| {
                    b.as_u64()
                        .and_then(|n| u8::try_from(n).ok())
                        .ok_or("atom byte invalid")
                })
                .collect::<std::result::Result<Vec<_>, _>>()?;
            Ok(String::from_utf8(bytes)?)
        };
        let directory: Vec<_> = relations["directory"]
            .as_array()
            .ok_or("revision directory absent")?
            .iter()
            .filter_map(|id| id.as_u64().filter(|id| *id != 0))
            .collect();
        record(
            checks,
            &format!("writer_choice_{label}_exact_revision_records"),
            records
                .iter()
                .filter(|r| r["id"].as_u64().is_some_and(|id| id != 0))
                .count()
                == 2
                && atom(&old["owner"])? == "selvi"
                && atom(&old["span"])? == "Dusk Ridge"
                && old["action"] == 1
                && old["previous"] == 0
                && atom(&new["owner"])? == "selvi"
                && atom(&new["span"])? == "Copper Vale"
                && new["action"] == 2
                && new["previous"] == 1
                && directory == vec![2],
            json!({"relations":relations}),
        )?;
        let exported = api.export_state()?;
        let mut imported = api_model.create_session(config)?;
        imported.import_state(&exported)?;
        record(
            checks,
            &format!("writer_choice_{label}_checkpoint_import"),
            imported.identity_scope() == api.identity_scope(),
            json!({"checkpoint_bytes":exported.len(),"scope":imported.identity_scope()}),
        )?;
        compare_turn(
            &model,
            &mut direct,
            &mut imported,
            SUM_14,
            "18.\n",
            &format!("writer_choice_{label}_independent_next_sum"),
            checks,
        )?;
    }
    let mut direct = model.session(Control::Full)?;
    let mut api = api_model.create_session(SessionConfig::default())?;
    compare_turn(
        &model,
        &mut direct,
        &mut api,
        "Record: now in Copper Vale. Where is now? Name the owner first. Answer:",
        " now is in Copper Vale.\n",
        "writer_choice_literal_now_owner_api_direct_parity",
        checks,
    )?;
    let config = SessionConfig {
        session_id: "writer-choice-dependent-revision".into(),
        ..SessionConfig::default()
    };
    let mut direct = model.session(Control::Full)?;
    let mut api = api_model.create_session(config.clone())?;
    compare_turn(&model, &mut direct, &mut api,
        "casket in elvin. elvin in Bremen. Now elvin in Zurich. Question: Where is the location of casket? Answer:",
        " Zurich.\n", "writer_choice_dependent_revision_api_direct_parity", checks)?;
    let checkpoint: Value = serde_json::from_slice(&direct.checkpoint()?)?;
    let relations = &checkpoint["values"]["relations"];
    let atom = |v: &Value| -> CheckResult<String> {
        let len = v["len"].as_u64().ok_or("dependent atom length absent")? as usize;
        let all = v["bytes"].as_array().ok_or("dependent atom bytes absent")?;
        let bytes = all
            .get(..len)
            .ok_or("dependent atom length overflow")?
            .iter()
            .map(|b| {
                b.as_u64()
                    .and_then(|n| u8::try_from(n).ok())
                    .ok_or("dependent atom byte invalid")
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(String::from_utf8(bytes)?)
    };
    let mut actual_records = Vec::new();
    for r in relations["records"]
        .as_array()
        .ok_or("dependent records absent")?
    {
        if r["id"].as_u64().is_some_and(|id| id != 0) {
            actual_records.push(json!({"id":r["id"],"owner":atom(&r["owner"])?
                ,"value":atom(&r["value"])? ,"action":r["action"],"previous":r["previous"],"conflict":r["conflict"]}));
        }
    }
    let directory: Vec<_> = relations["directory"]
        .as_array()
        .ok_or("dependent directory absent")?
        .iter()
        .filter_map(|id| id.as_u64().filter(|id| *id != 0))
        .collect();
    record(
        checks,
        "writer_choice_dependent_revision_exact_records",
        json!(actual_records)
            == json!([
                {"id":1,"owner":"casket","value":"elvin","action":1,"previous":0,"conflict":false},
                {"id":2,"owner":"elvin","value":"Bremen","action":1,"previous":0,"conflict":false},
                {"id":3,"owner":"elvin","value":"Zurich","action":2,"previous":2,"conflict":false},
            ])
            && directory == vec![1, 3],
        json!({"records":actual_records,"directory":directory}),
    )?;
    let exported = api.export_state()?;
    let mut imported = api_model.create_session(config)?;
    imported.import_state(&exported)?;
    record(
        checks,
        "writer_choice_dependent_revision_checkpoint_import",
        imported.identity_scope() == api.identity_scope(),
        json!({"checkpoint_bytes":exported.len(),"scope":imported.identity_scope()}),
    )?;
    compare_turn(
        &model,
        &mut direct,
        &mut imported,
        SUM_14,
        "18.\n",
        "writer_choice_dependent_revision_independent_next_sum",
        checks,
    )?;
    let mut chunked = api_model.create_session(SessionConfig::default())?;
    chunked.ingest("Record: selvi in Dusk Ridge. selvi n")?;
    let checkpoint = chunked.export_state()?;
    chunked.import_state(&checkpoint)?;
    chunked.ingest("ow in Copper Vale. Where is selvi? Name the owner first. Answer:")?;
    let actual = chunked.complete(CompletionRequest::new(""))?;
    record(
        checks,
        "writer_choice_revision_chunked_prefill_import",
        actual.text == " selvi is in Copper Vale.\n" && actual.stopped_by == "eos",
        json!({"checkpoint_bytes":checkpoint.len(),"actual":actual}),
    )?;
    let mut changed = document.clone();
    for (name, pointer, new, boundary) in [
        (
            "writer_choice_positive_coefficient_rejected",
            "/writer_choice/rows/0/weight",
            json!(1),
            "invalid writer choice residual rows",
        ),
        (
            "writer_choice_invalid_feature_rejected",
            "/writer_choice/rows/0/feature/kind",
            json!(0),
            "invalid writer choice residual rows",
        ),
        (
            "writer_choice_unknown_prime_rejected",
            "/writer_choice/rows/0/feature/a",
            json!(u64::MAX),
            "invalid writer choice residual rows",
        ),
        (
            "writer_choice_invalid_configuration_rejected",
            "/writer_choice/max_seconds",
            json!(0),
            "invalid writer choice configuration",
        ),
        (
            "writer_choice_empty_receipts_rejected",
            "/writer_choice/training",
            json!([]),
            "invalid writer choice training receipts",
        ),
    ] {
        let mut changed = document.clone();
        let field = changed
            .pointer_mut(pointer)
            .ok_or("writer choice mutation field absent")?;
        let old = field.clone();
        *field = new.clone();
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            name,
            rejected && error.as_deref().is_some_and(|e| e.contains(boundary)),
            json!({"field":pointer,"old":old,"new":new,"error":error}),
        )?;
    }
    let gap = changed
        .pointer_mut("/writer_choice/rows/0/feature/b")
        .ok_or("writer choice feature gap absent")?;
    let old = gap
        .as_u64()
        .ok_or("writer choice feature gap not integer")?;
    let layout = witness
        .get("feature_layout")
        .and_then(Value::as_u64)
        .unwrap_or(1);
    let gap_shift = match layout {
        1 => 0,
        2 => 4,
        _ => return Err("unknown writer choice layout in supplied artifact".into()),
    };
    let new = (old & !(1023_u64 << gap_shift)) | (1023_u64 << gap_shift);
    *gap = json!(new);
    let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
    record(
        checks,
        "writer_choice_invalid_exterior_gap_rejected",
        rejected
            && error
                .as_deref()
                .is_some_and(|e| e.contains("invalid writer choice residual rows")),
        json!({"layout":layout,"old":old,"new":new,"error":error}),
    )?;
    if layout == 2 {
        for distance in [0_u64, 8] {
            let mut changed = document.clone();
            let field = changed
                .pointer_mut("/writer_choice/rows/0/feature/b")
                .ok_or("writer choice feature distance absent")?;
            let old = field.as_u64().ok_or("writer choice distance not integer")?;
            let new = (old & !15_u64) | distance;
            *field = json!(new);
            let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
            record(
                checks,
                &format!("writer_choice_invalid_distance_{distance}_rejected"),
                rejected
                    && error
                        .as_deref()
                        .is_some_and(|e| e.contains("invalid writer choice residual rows")),
                json!({"layout":layout,"old":old,"new":new,"distance":distance,"error":error}),
            )?;
        }
    } else {
        checks.push(json!({"name":"writer_choice_distance_checks","status":"NOT_APPLICABLE","reason":"layout1 has no distance field"}));
    }
    let mut changed = document.clone();
    changed["writer_choice"]["feature_layout"] = json!(3);
    let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
    record(
        checks,
        "writer_choice_unknown_layout_rejected",
        rejected,
        json!({"layout":3,"error":error}),
    )?;
    let mut changed = document.clone();
    let weight = changed
        .pointer_mut("/writer_choice/rows/0/weight")
        .ok_or("writer choice residual absent")?;
    let old = weight
        .as_i64()
        .ok_or("writer choice residual not integer")?;
    if !(-1_000_000..=0).contains(&old) {
        return Err("writer choice residual outside range".into());
    }
    let new = if old == -1_000_000 { old + 1 } else { old - 1 };
    *weight = json!(new);
    let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
    record(
        checks,
        "writer_choice_valid_residual_identity_rejected",
        rejected
            && error
                .as_deref()
                .is_some_and(|e| e.contains("writer choice identity differs")),
        json!({"old":old,"new":new,"error":error}),
    )?;
    let mut changed = document.clone();
    changed["writer_choice"]["parent_artifact"] = json!(format!("blake3:{}", "0".repeat(64)));
    let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
    record(
        checks,
        "writer_choice_wrong_parent_rejected",
        rejected,
        json!({"error":error}),
    )?;
    let mut changed = document.clone();
    changed["writer_choice"]["unexpected_witness_field"] = json!(true);
    let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
    record(
        checks,
        "writer_choice_unknown_witness_field_rejected",
        rejected,
        json!({"error":error}),
    )?;
    record(
        checks,
        "writer_choice_nested_field_corruption_target",
        true,
        json!({"candidate_artifact":model.artifact_cid(),"corruption_target_artifact":parent.artifact_cid(),
            "scope":"Historical field-composition corruption checks use the exact reconstructed field parent. All direct/API generated behavior continues to execute the supplied writer-choice candidate."}),
    )?;
    Ok(Some(parent))
}

fn field_composition_checks(
    api_model: &NativeModel,
    mechanical_model: &Model,
    document: &Value,
    checks: &mut Vec<Value>,
) -> CheckResult<Option<Model>> {
    let Some(witness) = document.get("field_composition").filter(|v| v.is_object()) else {
        checks.push(json!({"name":"field_composition_checks","status":"NOT_APPLICABLE","reason":"supplied artifact has no field_composition witness"}));
        return Ok(None);
    };
    let model = api_model.inner_model();
    let parent = mechanical_model.without_field_composition()?;
    let parent_bytes = parent.to_bytes()?;
    let parent_document: Value = serde_json::from_slice(&parent_bytes)?;
    let expected_parent = "blake3:169f23efd1babd314deed5cd523953179d94a8eb9dbf6e930892e57080a85828";
    let mut candidate_base = document.clone();
    let mut parent_base = parent_document.clone();
    for key in [
        "field_composition",
        "lexical_emission",
        "artifact_cid",
        "uor_model_address",
    ] {
        candidate_base
            .as_object_mut()
            .ok_or("candidate is not an object")?
            .remove(key);
        parent_base
            .as_object_mut()
            .ok_or("parent is not an object")?
            .remove(key);
    }
    record(
        checks,
        "field_composition_parent_and_frozen_components",
        witness["parent_artifact"] == expected_parent
            && parent.artifact_cid() == expected_parent
            && witness["previous_lexical"] == parent_document["lexical_emission"]
            && candidate_base == parent_base,
        json!({"candidate_artifact":model.artifact_cid(),"mechanical_artifact":mechanical_model.artifact_cid(),"parent_artifact":parent.artifact_cid(),
            "parent_bytes":parent_bytes.len(),"parent_bytes_blake3":blake3::hash(&parent_bytes).to_hex().to_string(),
            "boundary":"All mechanical field-artifact fields except extension, shared lexical emission and derived identities equal the recursively validated parent; previous lexical selector is restored exactly. Generation continues on the supplied candidate."}),
    )?;
    let config = SessionConfig {
        session_id: "field-composition-check".into(),
        ..SessionConfig::default()
    };
    let mut api = api_model.create_session(config.clone())?;
    let mut direct = model.session(Control::Full)?;
    compare_turn(
        &model,
        &mut direct,
        &mut api,
        "Record: selvi in Dusk Ridge. Where is selvi? Name the owner first. Answer:",
        " selvi is in Dusk Ridge.\n",
        "field_composition_owner_value_api_direct_parity",
        checks,
    )?;
    let checkpoint = api.export_state()?;
    let mut imported = api_model.create_session(config)?;
    imported.import_state(&checkpoint)?;
    record(
        checks,
        "field_composition_checkpoint_import",
        imported.identity_scope() == api.identity_scope(),
        json!({"checkpoint_bytes":checkpoint.len(),"scope":imported.identity_scope()}),
    )?;
    compare_turn(
        &model,
        &mut direct,
        &mut imported,
        SUM_14,
        "18.\n",
        "field_composition_checkpoint_next_independent_sum",
        checks,
    )?;

    // Prefill import crosses the owner spelling boundary and must preserve the
    // same selected relation and generated bytes as the joined prompt.
    let mut chunked = api_model.create_session(SessionConfig::default())?;
    chunked.ingest("Record: sel")?;
    let checkpoint = chunked.export_state()?;
    chunked.import_state(&checkpoint)?;
    chunked.ingest("vi in Dusk Ridge. Where is selvi? Name the owner first. Answer:")?;
    let actual = chunked.complete(CompletionRequest::new(""))?;
    record(
        checks,
        "field_composition_chunked_prefill_import",
        actual.text == " selvi is in Dusk Ridge.\n" && actual.stopped_by == "eos",
        json!({"checkpoint_bytes":checkpoint.len(),"actual":actual}),
    )?;

    for (name, pointer, new) in [
        (
            "field_composition_wrong_parent_rejected",
            "/field_composition/parent_artifact",
            json!(format!("blake3:{}", "0".repeat(64))),
        ),
        (
            "field_composition_dictionary_prime_rejected",
            "/field_composition/dictionary/0/prime",
            json!(0),
        ),
        (
            "field_composition_invalid_token_rejected",
            "/field_composition/tokens/0",
            json!(u32::MAX),
        ),
    ] {
        let mut changed = document.clone();
        let field = changed
            .pointer_mut(pointer)
            .ok_or("field composition mutation member absent")?;
        let old = field.clone();
        *field = new.clone();
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            name,
            rejected,
            json!({"artifact":mechanical_model.artifact_cid(),"field":pointer,"old":old,"new":new,"error":error}),
        )?;
    }
    let mut changed = document.clone();
    let pointer = "/field_composition/previous_lexical/router/biases/0";
    let field = changed
        .pointer_mut(pointer)
        .ok_or("previous lexical bias absent")?;
    let old = field.as_i64().ok_or("previous lexical bias not integer")?;
    let new = if old == 32 { old - 1 } else { old + 1 };
    *field = json!(new);
    let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
    record(
        checks,
        "field_composition_previous_selector_rejected",
        rejected,
        json!({"artifact":mechanical_model.artifact_cid(),"field":pointer,"old":old,"new":new,"error":error}),
    )?;
    let mut changed = document.clone();
    changed["field_composition"]["unexpected_witness_field"] = json!(true);
    let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
    record(
        checks,
        "field_composition_unknown_witness_field_rejected",
        rejected,
        json!({"artifact":mechanical_model.artifact_cid(),"error":error}),
    )?;
    record(
        checks,
        "historical_nested_corruption_target",
        true,
        json!({"candidate_artifact":model.artifact_cid(),"corruption_target_artifact":parent.artifact_cid(),
            "scope":"The following historical nested-witness corruption checks mutate the exact reconstructed writer parent. All API and direct generated behavior continues to execute the supplied candidate. Outer witnesses and candidate identity are checked separately above."}),
    )?;
    Ok(Some(parent))
}

fn run(
    path: &Path,
    expected_cid: &str,
    checks: &mut Vec<Value>,
    identity: &mut Value,
) -> CheckResult<()> {
    let bytes = std::fs::read(path)?;
    let api_model = NativeModel::load_from_bytes(&bytes)?;
    *identity = json!({
        "path":path,"artifact_cid":api_model.artifact_cid(),
        "expected_cid":expected_cid,"input_bytes":bytes.len(),
        "input_bytes_blake3":blake3::hash(&bytes).to_hex().to_string(),
        "metadata":api_model.metadata(),
    });
    record(
        checks,
        "expected_artifact_identity",
        api_model.artifact_cid() == expected_cid,
        json!({"actual":api_model.artifact_cid(),"expected":expected_cid}),
    )?;
    let model = api_model.inner_model();
    let encoded = model.to_bytes()?;
    record(
        checks,
        "artifact_serialization_byte_exact",
        encoded == bytes,
        json!({"input_bytes":bytes.len(),"serialized_bytes":encoded.len()}),
    )?;
    let restored = Model::from_bytes(&encoded)?;
    record(
        checks,
        "artifact_load_save_roundtrip",
        restored.to_bytes()? == bytes && restored.artifact_cid() == expected_cid,
        json!({"restored_cid":restored.artifact_cid()}),
    )?;
    drop(restored);

    // The phrase and arithmetic targets are previously supported narrow panels.
    // The two arithmetic questions share a session and contain independent inputs.
    let mut direct = model.session(Control::Full)?;
    let mut api = api_model.create_session(SessionConfig::default())?;
    compare_turn(
        &model,
        &mut direct,
        &mut api,
        QUIET_RIVER,
        " quiet river.\n",
        "quiet_river_api_direct_parity",
        checks,
    )?;
    let mut direct = model.session(Control::Full)?;
    let config = SessionConfig {
        session_id: "recovery-check".into(),
        user_id: "recovery-user".into(),
        project_id: "recovery-project".into(),
        ..SessionConfig::default()
    };
    let mut api = api_model.create_session(config.clone())?;
    compare_turn(
        &model,
        &mut direct,
        &mut api,
        SUM_13,
        "17.\n",
        "first_turn_sum_api_direct_parity",
        checks,
    )?;
    compare_turn(
        &model,
        &mut direct,
        &mut api,
        SUM_14,
        "18.\n",
        "second_turn_new_inputs_api_direct_parity",
        checks,
    )?;

    let checkpoint = api.export_state()?;
    let mut same = api_model.create_session(config.clone())?;
    same.import_state(&checkpoint)?;
    record(
        checks,
        "checkpoint_same_identity_import",
        same.identity_scope() == api.identity_scope(),
        json!({"checkpoint_bytes":checkpoint.len(),"scope":same.identity_scope()}),
    )?;
    let original_next = api.complete(CompletionRequest::new(SUM_13))?;
    let restored_next = same.complete(CompletionRequest::new(SUM_13))?;
    record(
        checks,
        "checkpoint_next_response_byte_parity",
        original_next.text == "17.\n"
            && original_next.text == restored_next.text
            && original_next.token_count == restored_next.token_count
            && original_next.stopped_by == restored_next.stopped_by,
        json!({"prompt":SUM_13,"original":original_next,"restored":restored_next}),
    )?;
    let mut cross = api_model.create_session(SessionConfig {
        user_id: "different-user".into(),
        ..config.clone()
    })?;
    let cross_result = cross.import_state(&checkpoint);
    let cross_rejected = matches!(&cross_result, Err(NativeApiError::IdentityViolation(_)));
    record(
        checks,
        "checkpoint_cross_identity_rejected",
        cross_rejected,
        json!({"error":cross_result.err().map(|e|e.to_string()),"target_scope":cross.identity_scope()}),
    )?;

    let split = [
        "User: suri has 1",
        "3 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:",
    ];
    let mut chunked = api_model.create_session(config)?;
    chunked.ingest(split[0])?;
    let prefill = chunked.export_state()?;
    chunked.import_state(&prefill)?;
    chunked.ingest(split[1])?;
    let actual = chunked.complete(CompletionRequest::new(""))?;
    let mut joined = api_model.create_session(SessionConfig::default())?;
    let expected = joined.complete(CompletionRequest::new(split.concat()))?;
    record(
        checks,
        "chunked_prefill_checkpoint_preserves_literal",
        actual.text == "17.\n"
            && actual.text == expected.text
            && actual.token_count == expected.token_count,
        json!({"chunks":split,"prefill_checkpoint_bytes":prefill.len(),"chunked":actual,"joined":expected}),
    )?;

    let candidate_document: Value = serde_json::from_slice(&bytes)?;
    let field_parent = writer_choice_checks(&api_model, &candidate_document, checks)?;
    let field_document: Value = if let Some(parent) = field_parent.as_ref() {
        serde_json::from_slice(&parent.to_bytes()?)?
    } else {
        candidate_document
    };
    let mechanical_model = field_parent.as_ref().unwrap_or(&model);
    let nested_parent =
        field_composition_checks(&api_model, mechanical_model, &field_document, checks)?;
    let document: Value = if let Some(parent) = nested_parent.as_ref() {
        serde_json::from_slice(&parent.to_bytes()?)?
    } else {
        field_document
    };
    let writer_parent = document.get("writer_lexical").is_some_and(Value::is_object);
    let source_role_parent = document
        .get("source_role_refinement")
        .is_some_and(Value::is_object);
    let word_parent = document.get("word_emission").is_some_and(Value::is_object);
    let shared_parent = document
        .get("shared_operator_refinement")
        .is_some_and(Value::is_object);
    let action_parent = document
        .get("action_emission")
        .is_some_and(Value::is_object);
    let mixed_parent = document
        .get("mixed_operators")
        .is_some_and(Value::is_object);
    if document
        .get("operation_transition")
        .is_some_and(|v| !v.is_null())
    {
        let mut direct = model.session(Control::Full)?;
        let mut api = api_model.create_session(SessionConfig::default())?;
        compare_turn(
            &model,
            &mut direct,
            &mut api,
            SUM_13,
            "17.\n",
            "transition_history_api",
            checks,
        )?;
        let before = api.export_state()?;
        api.import_state(&before)?;
        compare_turn(&model, &mut direct, &mut api, "User: There are 3 extra coins. Add the extra coins to the original total. Again.\nAssistant:", "20.\n23.\n", "same_query_add_add_api_after_checkpoint", checks)?;
    }
    if document
        .get("lexical_emission")
        .is_some_and(|v| !v.is_null())
    {
        // Open development interface cases. Expected text is checked only here;
        // the direct and API sessions generate entirely from the supplied artifact.
        for (label, request, target) in [
            ("sentence", "sentence. ", "17 is 4 plus 13.\n"),
            ("rust", "Rust. ", "17 == 4 + 13\n"),
        ] {
            let prompt = format!("User: suri has 13 coins. orin has 4 coins.\nUser: {request}What is the sum of suri's and orin's coins?\nAssistant:");
            let config = SessionConfig {
                session_id: format!("lexical-emission-{label}"),
                ..SessionConfig::default()
            };
            let mut direct = model.session(Control::Full)?;
            let mut api = api_model.create_session(config.clone())?;
            compare_turn(
                &model,
                &mut direct,
                &mut api,
                &prompt,
                target,
                &format!("lexical_{label}_api_direct_parity"),
                checks,
            )?;
            let exported = api.export_state()?;
            let mut imported = api_model.create_session(config)?;
            imported.import_state(&exported)?;
            direct = model.restore_session(&direct.checkpoint()?)?;
            record(
                checks,
                &format!("lexical_{label}_checkpoint_import"),
                imported.identity_scope() == api.identity_scope(),
                json!({"checkpoint_bytes":exported.len(),"scope":imported.identity_scope(),"development_case":true}),
            )?;
            compare_turn(
                &model,
                &mut direct,
                &mut imported,
                SUM_13,
                "17.\n",
                &format!("lexical_{label}_checkpoint_next_independent_sum"),
                checks,
            )?;
        }
    }
    if let Some(witness) = document.get("instruction_binding").filter(|v| !v.is_null()) {
        // Both successful loads above run the outer validator: it restores the
        // two witnessed routers, checks the exact parent CID and recursively
        // validates that parent before validating the current artifact identity.
        // Reuse those loads instead of adding a redundant full reconstruction.
        record(
            checks,
            "instruction_binding_parent_reconstruction_and_roundtrip",
            witness
                .get("parent_artifact")
                .and_then(Value::as_str)
                .is_some(),
            json!({"parent_artifact":witness["parent_artifact"],"current_artifact":expected_cid,
                "validated_by":["NativeModel::load_from_bytes","artifact_load_save_roundtrip"],
                "boundary":"The loader reconstructs and validates the full frozen parent; this check does not extract a separate parent artifact."}),
        )?;
        // These are authored open development cases. Operand order is fixed to
        // the retained parent's exact ordered source IDs [1, 0], independently
        // of the response currently produced by either interface.
        for (label, before, after, target) in [
            (
                "sentence_prefix",
                "Explain in a sentence. ",
                "",
                "17 is 4 plus 13.\n",
            ),
            (
                "sentence_suffix",
                "",
                " Explain in a sentence.",
                "17 is 4 plus 13.\n",
            ),
            (
                "rust_prefix",
                "Write a Rust equality. ",
                "",
                "17 == 4 + 13\n",
            ),
            (
                "rust_suffix",
                "",
                " Write a Rust equality.",
                "17 == 4 + 13\n",
            ),
        ] {
            let prompt = format!("User: suri has 13 coins. orin has 4 coins.\nUser: {before}What is the sum of suri's and orin's coins?{after}\nAssistant:");
            let config = SessionConfig {
                session_id: format!("instruction-binding-{label}"),
                ..SessionConfig::default()
            };
            let mut direct = model.session(Control::Full)?;
            let mut api = api_model.create_session(config.clone())?;
            compare_turn(
                &model,
                &mut direct,
                &mut api,
                &prompt,
                target,
                &format!("instruction_{label}_api_direct_parity"),
                checks,
            )?;
            let checkpoint = api.export_state()?;
            let mut imported = api_model.create_session(config)?;
            imported.import_state(&checkpoint)?;
            direct = model.restore_session(&direct.checkpoint()?)?;
            record(
                checks,
                &format!("instruction_{label}_checkpoint_import"),
                imported.identity_scope() == api.identity_scope(),
                json!({"checkpoint_bytes":checkpoint.len(),"scope":imported.identity_scope(),"development_case":true}),
            )?;
            compare_turn(
                &model,
                &mut direct,
                &mut imported,
                SUM_13,
                "17.\n",
                &format!("instruction_{label}_checkpoint_next_independent_sum"),
                checks,
            )?;
        }
        let composed_parent = document
            .get("composed_output")
            .is_some_and(Value::is_object);
        // Legal scalar mutations isolate identity boundaries from shape errors.
        // Witness changes must fail at parent reconstruction, while a changed
        // current router must preserve that parent and fail at current identity.
        for (name, pointer, boundary) in [
            (
                "instruction_previous_literals_frozen_parent_rejected",
                "/instruction_binding/previous_literals/biases/0",
                "instruction binding frozen parent differs",
            ),
            (
                "instruction_previous_admission_frozen_parent_rejected",
                "/instruction_binding/previous_admission/biases/0",
                "instruction binding frozen parent differs",
            ),
            (
                "instruction_current_literals_identity_rejected",
                "/typed_literals/router/biases/0",
                "instruction binding identity differs",
            ),
        ] {
            let boundary = if shared_parent {
                if pointer.starts_with("/operation_transition/")
                    || pointer.starts_with("/typed_roles/")
                {
                    "shared operator identity differs"
                } else {
                    "shared operator frozen parent differs"
                }
            } else if action_parent {
                if pointer.starts_with("/operation_transition/")
                    || pointer.starts_with("/typed_roles/")
                {
                    "action emission identity differs"
                } else {
                    "action emission frozen parent differs"
                }
            } else if mixed_parent {
                "mixed operators frozen parent differs"
            } else if composed_parent {
                "composed output frozen parent differs"
            } else {
                boundary
            };
            let mut changed = document.clone();
            let bias = changed
                .pointer_mut(pointer)
                .ok_or("instruction router bias absent")?;
            let old = bias
                .as_i64()
                .ok_or("instruction router bias is not an integer")?;
            if !(-32..=32).contains(&old) {
                return Err("instruction router bias lies outside documented range".into());
            }
            let new = if old == 32 { old - 1 } else { old + 1 };
            *bias = json!(new);
            let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
            record(
                checks,
                name,
                rejected
                    && error.as_deref().is_some_and(|e| {
                        e.contains(word_boundary(
                            writer_parent,
                            source_role_parent,
                            word_parent,
                            pointer,
                            boundary,
                        ))
                    }),
                json!({"field":pointer,"old":old,"new":new,"expected_boundary":word_boundary(writer_parent, source_role_parent, word_parent, pointer, boundary),"error":error}),
            )?;
        }
        let parent_boundary = if writer_parent {
            "writer lexical frozen parent differs"
        } else if source_role_parent {
            "source role refinement frozen parent differs"
        } else if word_parent {
            "word emission frozen parent differs"
        } else if shared_parent {
            "shared operator frozen parent differs"
        } else if action_parent {
            "action emission frozen parent differs"
        } else if mixed_parent {
            "mixed operators frozen parent differs"
        } else if composed_parent {
            "composed output frozen parent differs"
        } else {
            "instruction binding frozen parent differs"
        };
        let mut changed = document.clone();
        let wrong_parent = format!("blake3:{}", "0".repeat(64));
        changed["instruction_binding"]["parent_artifact"] = json!(wrong_parent);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "instruction_wrong_parent_cid_rejected",
            rejected
                && error
                    .as_deref()
                    .is_some_and(|e| e.contains(parent_boundary)),
            json!({"old":witness["parent_artifact"],"new":wrong_parent,"expected_boundary":parent_boundary,"error":error}),
        )?;
        let mut changed = document.clone();
        changed["instruction_binding"]["unexpected_witness_field"] = json!(true);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "instruction_unknown_witness_field_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains("unknown field") && e.contains("unexpected_witness_field")
                }),
            json!({"expected_boundary":"witness deserialization denies unknown fields","error":error}),
        )?;
    } else {
        checks.push(json!({"name":"instruction_binding_checks","status":"NOT_APPLICABLE","reason":"supplied artifact has no instruction_binding witness"}));
    }
    if let Some(witness) = document.get("composed_output").filter(|v| !v.is_null()) {
        for (label, request, target) in [
            (
                "sentence",
                " Explain in a sentence.",
                "20 is 3 plus 17.\n23 is 3 plus 20.\n",
            ),
            (
                "rust",
                " Write a Rust equality.",
                "20 == 3 + 17\n23 == 3 + 20\n",
            ),
        ] {
            let config = SessionConfig {
                session_id: format!("composed-output-{label}"),
                ..SessionConfig::default()
            };
            let mut direct = model.session(Control::Full)?;
            let mut api = api_model.create_session(config.clone())?;
            compare_turn(
                &model,
                &mut direct,
                &mut api,
                SUM_13,
                "17.\n",
                &format!("composed_{label}_actual_history"),
                checks,
            )?;
            let history = api.export_state()?;
            api.import_state(&history)?;
            direct = model.restore_session(&direct.checkpoint()?)?;
            let prompt = format!("User: There are 3 extra coins. Add the extra coins to the original total. Again.{request}\nAssistant:");
            compare_turn(
                &model,
                &mut direct,
                &mut api,
                &prompt,
                target,
                &format!("composed_{label}_api_direct_parity_after_history_checkpoint"),
                checks,
            )?;
            let exported = api.export_state()?;
            let mut imported = api_model.create_session(config)?;
            imported.import_state(&exported)?;
            direct = model.restore_session(&direct.checkpoint()?)?;
            record(
                checks,
                &format!("composed_{label}_checkpoint_import"),
                imported.identity_scope() == api.identity_scope(),
                json!({"checkpoint_bytes":exported.len(),"scope":imported.identity_scope(),"development_case":true}),
            )?;
            compare_turn(
                &model,
                &mut direct,
                &mut imported,
                SUM_13,
                "17.\n",
                &format!("composed_{label}_checkpoint_next_independent_sum"),
                checks,
            )?;
        }
        // Legal parameter changes must fail at the stated reconstruction layer.
        // The successful loads above already validate the complete parent chain.
        for (name, pointer, boundary) in [
            (
                "composed_previous_operation_frozen_parent_rejected",
                "/composed_output/previous_operation/biases/0",
                "composed output frozen parent differs",
            ),
            (
                "composed_current_operation_identity_rejected",
                "/operation_transition/router/biases/0",
                "composed output identity differs",
            ),
        ] {
            let boundary = if shared_parent {
                if pointer.starts_with("/operation_transition/")
                    || pointer.starts_with("/typed_roles/")
                {
                    "shared operator identity differs"
                } else {
                    "shared operator frozen parent differs"
                }
            } else if action_parent {
                if pointer.starts_with("/operation_transition/")
                    || pointer.starts_with("/typed_roles/")
                {
                    "action emission identity differs"
                } else {
                    "action emission frozen parent differs"
                }
            } else if mixed_parent {
                if pointer == "/operation_transition/router/biases/0" {
                    "mixed operators identity differs"
                } else {
                    "mixed operators frozen parent differs"
                }
            } else {
                boundary
            };
            let mut changed = document.clone();
            let bias = changed
                .pointer_mut(pointer)
                .ok_or("composed router bias absent")?;
            let old = bias
                .as_i64()
                .ok_or("composed router bias is not an integer")?;
            if !(-32..=32).contains(&old) {
                return Err("composed router bias outside documented range".into());
            }
            let new = if old == 32 { old - 1 } else { old + 1 };
            *bias = json!(new);
            let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
            record(
                checks,
                name,
                rejected
                    && error.as_deref().is_some_and(|e| {
                        e.contains(word_boundary(
                            writer_parent,
                            source_role_parent,
                            word_parent,
                            pointer,
                            boundary,
                        ))
                    }),
                json!({"field":pointer,"old":old,"new":new,"expected_boundary":word_boundary(writer_parent, source_role_parent, word_parent, pointer, boundary),"error":error}),
            )?;
        }
        let mut changed = document.clone();
        let wrong_parent = format!("blake3:{}", "0".repeat(64));
        changed["composed_output"]["parent_artifact"] = json!(wrong_parent);
        let parent_boundary = if writer_parent {
            "writer lexical frozen parent differs"
        } else if source_role_parent {
            "source role refinement frozen parent differs"
        } else if word_parent {
            "word emission frozen parent differs"
        } else if shared_parent {
            "shared operator frozen parent differs"
        } else if action_parent {
            "action emission frozen parent differs"
        } else if mixed_parent {
            "mixed operators frozen parent differs"
        } else {
            "composed output frozen parent differs"
        };
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "composed_wrong_parent_cid_rejected",
            rejected
                && error
                    .as_deref()
                    .is_some_and(|e| e.contains(parent_boundary)),
            json!({"old":witness["parent_artifact"],"new":wrong_parent,"expected_boundary":parent_boundary,"error":error}),
        )?;
        let mut changed = document.clone();
        changed["composed_output"]["unexpected_witness_field"] = json!(true);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "composed_unknown_witness_field_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains("unknown field") && e.contains("unexpected_witness_field")
                }),
            json!({"expected_boundary":"witness deserialization denies unknown fields","error":error}),
        )?;
    } else {
        checks.push(json!({"name":"composed_output_checks","status":"NOT_APPLICABLE","reason":"supplied artifact has no composed_output witness"}));
    }
    if let Some(witness) = document.get("mixed_operators").filter(|v| !v.is_null()) {
        // Authored open development sequences. Both interfaces must produce
        // the full independently specified response, including EOS, from the
        // actual generated history; parity alone cannot satisfy correctness.
        for (label, suffix, target) in [
            ("copy_latest", " Copy the latest result.", "20.\n20.\n"),
            ("copy_original", " Copy the original total.", "20.\n17.\n"),
        ] {
            let config = SessionConfig {
                session_id: format!("mixed-operators-{label}"),
                ..SessionConfig::default()
            };
            let mut direct = model.session(Control::Full)?;
            let mut api = api_model.create_session(config.clone())?;
            compare_turn(
                &model,
                &mut direct,
                &mut api,
                SUM_13,
                "17.\n",
                &format!("mixed_{label}_actual_history"),
                checks,
            )?;
            let history = api.export_state()?;
            api.import_state(&history)?;
            direct = model.restore_session(&direct.checkpoint()?)?;
            let prompt = format!("User: There are 3 extra coins. Add the extra coins to the original total.{suffix}\nAssistant:");
            compare_turn(
                &model,
                &mut direct,
                &mut api,
                &prompt,
                target,
                &format!("mixed_{label}_api_direct_parity_after_history_checkpoint"),
                checks,
            )?;
            let exported = api.export_state()?;
            let mut imported = api_model.create_session(config)?;
            imported.import_state(&exported)?;
            direct = model.restore_session(&direct.checkpoint()?)?;
            record(
                checks,
                &format!("mixed_{label}_checkpoint_import"),
                imported.identity_scope() == api.identity_scope(),
                json!({"checkpoint_bytes":exported.len(),"scope":imported.identity_scope(),"development_case":true}),
            )?;
            compare_turn(
                &model,
                &mut direct,
                &mut imported,
                SUM_13,
                "17.\n",
                &format!("mixed_{label}_checkpoint_next_independent_sum"),
                checks,
            )?;
        }
        // Successful loads above reconstruct the entire previous operation
        // component (router, dictionary, and limit) and previous shared roles,
        // then validate the full parent chain. Do not substitute the current
        // expanded operation dictionary or current role parameters.
        let expected_parent =
            "blake3:866cb92de4ad5c130da811d3f2fe8828aed1fe9b2b39b9c9b3c7ad3dbf7dee62";
        record(
            checks,
            "mixed_operators_parent_reconstruction_and_roundtrip",
            witness.get("parent_artifact").and_then(Value::as_str) == Some(expected_parent),
            json!({"parent_artifact":witness["parent_artifact"],"expected_parent":expected_parent,
                "current_artifact":expected_cid,
                "validated_by":["NativeModel::load_from_bytes","artifact_load_save_roundtrip"],
                "boundary":"The loader restores the full previous OperationTransition and previous shared role router and validates the frozen parent recursively; no separate extracted parent is claimed."}),
        )?;
        for (name, pointer, boundary) in [
            (
                "mixed_previous_operation_frozen_parent_rejected",
                "/mixed_operators/previous_operation/router/biases/0",
                "mixed operators frozen parent differs",
            ),
            (
                "mixed_current_operation_identity_rejected",
                "/operation_transition/router/biases/0",
                "mixed operators identity differs",
            ),
            (
                "mixed_previous_roles_frozen_parent_rejected",
                "/mixed_operators/previous_roles/biases/0",
                "mixed operators frozen parent differs",
            ),
            (
                "mixed_current_roles_identity_rejected",
                "/typed_roles/router/biases/0",
                "mixed operators identity differs",
            ),
        ] {
            let boundary = if shared_parent {
                if pointer.starts_with("/operation_transition/")
                    || pointer.starts_with("/typed_roles/")
                {
                    "shared operator identity differs"
                } else {
                    "shared operator frozen parent differs"
                }
            } else if action_parent {
                if pointer.starts_with("/operation_transition/")
                    || pointer.starts_with("/typed_roles/")
                {
                    "action emission identity differs"
                } else {
                    "action emission frozen parent differs"
                }
            } else {
                boundary
            };
            let mut changed = document.clone();
            let bias = changed
                .pointer_mut(pointer)
                .ok_or("mixed router bias absent")?;
            let old = bias.as_i64().ok_or("mixed router bias is not an integer")?;
            if !(-32..=32).contains(&old) {
                return Err("mixed router bias outside documented range".into());
            }
            let new = if old == 32 { old - 1 } else { old + 1 };
            *bias = json!(new);
            let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
            record(
                checks,
                name,
                rejected
                    && error.as_deref().is_some_and(|e| {
                        e.contains(word_boundary(
                            writer_parent,
                            source_role_parent,
                            word_parent,
                            pointer,
                            boundary,
                        ))
                    }),
                json!({"field":pointer,"old":old,"new":new,"expected_boundary":word_boundary(writer_parent, source_role_parent, word_parent, pointer, boundary),"error":error}),
            )?;
        }
        let mut changed = document.clone();
        let wrong_parent = format!("blake3:{}", "0".repeat(64));
        let parent_boundary = if writer_parent {
            "writer lexical frozen parent differs"
        } else if source_role_parent {
            "source role refinement frozen parent differs"
        } else if word_parent {
            "word emission frozen parent differs"
        } else if shared_parent {
            "shared operator frozen parent differs"
        } else if action_parent {
            "action emission frozen parent differs"
        } else {
            "mixed operators frozen parent differs"
        };
        changed["mixed_operators"]["parent_artifact"] = json!(wrong_parent);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "mixed_wrong_parent_cid_rejected",
            rejected
                && error
                    .as_deref()
                    .is_some_and(|e| e.contains(parent_boundary)),
            json!({"old":witness["parent_artifact"],"new":wrong_parent,"expected_boundary":parent_boundary,"error":error}),
        )?;
        let mut changed = document.clone();
        changed["mixed_operators"]["unexpected_witness_field"] = json!(true);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "mixed_unknown_witness_field_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains("unknown field") && e.contains("unexpected_witness_field")
                }),
            json!({"expected_boundary":"witness deserialization denies unknown fields","error":error}),
        )?;
    } else {
        checks.push(json!({"name":"mixed_operator_checks","status":"NOT_APPLICABLE","reason":"supplied artifact has no mixed_operators witness"}));
    }
    if let Some(witness) = document.get("action_emission").filter(|v| v.is_object()) {
        if witness.get("context_enabled").and_then(Value::as_bool) == Some(true) {
            for (label, suffix, target) in [
                (
                    "sentence_copy_latest",
                    " Copy the latest result. Explain in a sentence.",
                    "20 is 3 plus 17.\n20 is 20.\n",
                ),
                (
                    "sentence_copy_original",
                    " Copy the original total. Explain in a sentence.",
                    "20 is 3 plus 17.\n17 is 17.\n",
                ),
                (
                    "rust_copy_latest",
                    " Copy the latest result. Write a Rust equality.",
                    "20 == 3 + 17\n20 == 20\n",
                ),
                (
                    "rust_copy_original",
                    " Copy the original total. Write a Rust equality.",
                    "20 == 3 + 17\n17 == 17\n",
                ),
            ] {
                let config = SessionConfig {
                    session_id: format!("action-emission-{label}"),
                    ..SessionConfig::default()
                };
                let mut direct = model.session(Control::Full)?;
                let mut api = api_model.create_session(config.clone())?;
                compare_turn(
                    &model,
                    &mut direct,
                    &mut api,
                    SUM_13,
                    "17.\n",
                    &format!("action_{label}_actual_history"),
                    checks,
                )?;
                api.import_state(&api.export_state()?)?;
                direct = model.restore_session(&direct.checkpoint()?)?;
                let prompt = format!("User: There are 3 extra coins. Add the extra coins to the original total.{suffix}\nAssistant:");
                compare_turn(
                    &model,
                    &mut direct,
                    &mut api,
                    &prompt,
                    target,
                    &format!("action_{label}_api_direct_parity_after_history_checkpoint"),
                    checks,
                )?;
                let exported = api.export_state()?;
                let mut imported = api_model.create_session(config)?;
                imported.import_state(&exported)?;
                direct = model.restore_session(&direct.checkpoint()?)?;
                record(
                    checks,
                    &format!("action_{label}_checkpoint_import"),
                    imported.identity_scope() == api.identity_scope(),
                    json!({"checkpoint_bytes":exported.len(),"scope":imported.identity_scope(),"development_case":true}),
                )?;
                compare_turn(
                    &model,
                    &mut direct,
                    &mut imported,
                    SUM_13,
                    "17.\n",
                    &format!("action_{label}_checkpoint_next_independent_sum"),
                    checks,
                )?;
            }
        } else {
            checks.push(json!({"name":"action_emission_behavior","status":"NOT_APPLICABLE","reason":"staged artifact has context_enabled false"}));
        }
        let expected_parent =
            "blake3:9ab64902d4811f4e23e2119bdee74f16a6675eee7446e85e21666dfccfabea59";
        record(
            checks,
            "action_emission_parent_reconstruction_and_roundtrip",
            witness.get("parent_artifact").and_then(Value::as_str) == Some(expected_parent),
            json!({"parent_artifact":witness["parent_artifact"],"expected_parent":expected_parent,"current_artifact":expected_cid,
                "validated_by":["NativeModel::load_from_bytes","artifact_load_save_roundtrip"],
                "boundary":"Loader restores full previous lexical emission, operation transition, and shared role router, and validates the frozen parent recursively; no separate extracted parent is claimed."}),
        )?;
        for (name, pointer, boundary) in [
            (
                "action_previous_lexical_frozen_parent_rejected",
                "/action_emission/previous_lexical/router/biases/0",
                "action emission frozen parent differs",
            ),
            (
                "action_current_lexical_identity_rejected",
                "/lexical_emission/router/biases/0",
                "action emission identity differs",
            ),
            (
                "action_previous_roles_frozen_parent_rejected",
                "/action_emission/previous_roles/biases/0",
                "action emission frozen parent differs",
            ),
            (
                "action_current_roles_identity_rejected",
                "/typed_roles/router/biases/0",
                "action emission identity differs",
            ),
            (
                "action_previous_operation_frozen_parent_rejected",
                "/action_emission/previous_operation/router/biases/0",
                "action emission frozen parent differs",
            ),
            (
                "action_current_operation_identity_rejected",
                "/operation_transition/router/biases/0",
                "action emission identity differs",
            ),
        ] {
            let boundary = if shared_parent {
                if pointer.starts_with("/operation_transition/")
                    || pointer.starts_with("/typed_roles/")
                {
                    "shared operator identity differs"
                } else {
                    "shared operator frozen parent differs"
                }
            } else {
                boundary
            };
            let mut changed = document.clone();
            let bias = changed
                .pointer_mut(pointer)
                .ok_or("action router bias absent")?;
            let old = bias
                .as_i64()
                .ok_or("action router bias is not an integer")?;
            if !(-32..=32).contains(&old) {
                return Err("action router bias outside documented range".into());
            }
            let new = if old == 32 { old - 1 } else { old + 1 };
            *bias = json!(new);
            let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
            record(
                checks,
                name,
                rejected
                    && error.as_deref().is_some_and(|e| {
                        e.contains(word_boundary(
                            writer_parent,
                            source_role_parent,
                            word_parent,
                            pointer,
                            boundary,
                        ))
                    }),
                json!({"field":pointer,"old":old,"new":new,"expected_boundary":word_boundary(writer_parent, source_role_parent, word_parent, pointer, boundary),"error":error}),
            )?;
        }
        let mut changed = document.clone();
        changed["action_emission"]["parent_artifact"] = json!(format!("blake3:{}", "0".repeat(64)));
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "action_wrong_parent_cid_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains(if writer_parent {
                        "writer lexical frozen parent differs"
                    } else if source_role_parent {
                        "source role refinement frozen parent differs"
                    } else if word_parent {
                        "word emission frozen parent differs"
                    } else if shared_parent {
                        "shared operator frozen parent differs"
                    } else {
                        "action emission frozen parent differs"
                    })
                }),
            json!({"error":error}),
        )?;
        let mut changed = document.clone();
        changed["action_emission"]["context_enabled"] = json!(!witness["context_enabled"]
            .as_bool()
            .ok_or("action context flag absent")?);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "action_context_flag_identity_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains(if writer_parent {
                        "writer lexical frozen parent differs"
                    } else if source_role_parent {
                        "source role refinement frozen parent differs"
                    } else if word_parent {
                        "word emission frozen parent differs"
                    } else if shared_parent {
                        "shared operator frozen parent differs"
                    } else {
                        "action emission identity differs"
                    })
                }),
            json!({"error":error}),
        )?;
        let mut changed = document.clone();
        changed["action_emission"]["unexpected_witness_field"] = json!(true);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "action_unknown_witness_field_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains("unknown field") && e.contains("unexpected_witness_field")
                }),
            json!({"error":error}),
        )?;
    } else {
        checks.push(json!({"name":"action_emission_checks","status":"NOT_APPLICABLE","reason":"supplied artifact has no action_emission witness"}));
    }
    if let Some(witness) = document
        .get("shared_operator_refinement")
        .filter(|v| v.is_object())
    {
        // The stage flag records completed fitting, never behavior quality.
        // Every independently specified target below must still be generated.
        if witness.get("continuation_fitted").and_then(Value::as_bool) == Some(true) {
            for (label, request, target) in [
                ("sentence_original_add_extra", "Copy the original total. Add the extra to the copied result. Explain in a sentence.", "17 is 17.\n20 is 3 plus 17.\n"),
                ("sentence_extra_add_original", "Copy the extra. Add the original total to the copied result. Explain in a sentence.", "3 is 3.\n20 is 17 plus 3.\n"),
                ("rust_original_add_extra", "Copy the original total. Add the extra to the copied result. Write a Rust equality.", "17 == 17\n20 == 3 + 17\n"),
                ("rust_extra_add_original", "Copy the extra. Add the original total to the copied result. Write a Rust equality.", "3 == 3\n20 == 17 + 3\n"),
            ] {
                let config = SessionConfig { session_id: format!("copy-add-{label}"), ..SessionConfig::default() };
                let mut direct = model.session(Control::Full)?;
                let mut api = api_model.create_session(config.clone())?;
                compare_turn(&model, &mut direct, &mut api, SUM_13, "17.\n", &format!("copy_add_{label}_actual_history"), checks)?;
                api.import_state(&api.export_state()?)?;
                direct = model.restore_session(&direct.checkpoint()?)?;
                let prompt = format!("User: There are 3 extra coins. {request}\nAssistant:");
                compare_turn(&model, &mut direct, &mut api, &prompt, target, &format!("copy_add_{label}_api_direct_parity_after_history_checkpoint"), checks)?;
                let exported = api.export_state()?;
                let mut imported = api_model.create_session(config)?;
                imported.import_state(&exported)?;
                direct = model.restore_session(&direct.checkpoint()?)?;
                record(checks, &format!("copy_add_{label}_checkpoint_import"), imported.identity_scope() == api.identity_scope(),
                    json!({"checkpoint_bytes":exported.len(),"scope":imported.identity_scope(),"development_case":true}))?;
                compare_turn(&model, &mut direct, &mut imported, SUM_13, "17.\n", &format!("copy_add_{label}_checkpoint_next_independent_sum"), checks)?;
            }
        } else {
            checks.push(json!({"name":"copy_add_behavior","status":"NOT_APPLICABLE","reason":"continuation_fitted is false; staged binding checkpoint has no fitted continuation"}));
        }
        let expected_parent =
            "blake3:5ed24f4e7487b6f9cb7fb762fc8bcc9092152f40cb756d864a82880d08ca867d";
        record(
            checks,
            "shared_operator_parent_reconstruction_and_roundtrip",
            witness.get("parent_artifact").and_then(Value::as_str) == Some(expected_parent),
            json!({"parent_artifact":witness["parent_artifact"],"expected_parent":expected_parent,"current_artifact":expected_cid,
                "validated_by":["NativeModel::load_from_bytes","artifact_load_save_roundtrip"],
                "boundary":"Loader restores full previous operation transition, including dictionary and limit, and shared role router, then validates the frozen parent recursively; no separate extracted parent is claimed."}),
        )?;
        for (name, pointer, boundary) in [
            (
                "shared_previous_roles_frozen_parent_rejected",
                "/shared_operator_refinement/previous_roles/biases/0",
                "shared operator frozen parent differs",
            ),
            (
                "shared_current_roles_identity_rejected",
                "/typed_roles/router/biases/0",
                "shared operator identity differs",
            ),
            (
                "shared_previous_operation_frozen_parent_rejected",
                "/shared_operator_refinement/previous_operation/router/biases/0",
                "shared operator frozen parent differs",
            ),
            (
                "shared_current_operation_identity_rejected",
                "/operation_transition/router/biases/0",
                "shared operator identity differs",
            ),
        ] {
            let mut changed = document.clone();
            let bias = changed
                .pointer_mut(pointer)
                .ok_or("shared router bias absent")?;
            let old = bias
                .as_i64()
                .ok_or("shared router bias is not an integer")?;
            if !(-32..=32).contains(&old) {
                return Err("shared router bias outside documented range".into());
            }
            let new = if old == 32 { old - 1 } else { old + 1 };
            *bias = json!(new);
            let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
            record(
                checks,
                name,
                rejected
                    && error.as_deref().is_some_and(|e| {
                        e.contains(word_boundary(
                            writer_parent,
                            source_role_parent,
                            word_parent,
                            pointer,
                            boundary,
                        ))
                    }),
                json!({"field":pointer,"old":old,"new":new,"expected_boundary":word_boundary(writer_parent, source_role_parent, word_parent, pointer, boundary),"error":error}),
            )?;
        }
        for (name, pointer, boundary) in [
            (
                "shared_previous_dictionary_frozen_parent_rejected",
                "/shared_operator_refinement/previous_operation/dictionary/0/prime",
                "shared operator frozen parent differs",
            ),
            (
                "shared_current_dictionary_shape_rejected",
                "/operation_transition/dictionary/0/prime",
                "invalid operation transition dictionary",
            ),
        ] {
            let mut changed = document.clone();
            let prime = changed
                .pointer_mut(pointer)
                .ok_or("shared operation prime absent")?;
            let old = prime
                .as_u64()
                .ok_or("shared operation prime is not unsigned")?;
            *prime = json!(old.checked_add(1).ok_or("prime overflow")?);
            let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
            record(
                checks,
                name,
                rejected
                    && error.as_deref().is_some_and(|e| {
                        e.contains(word_boundary(
                            writer_parent,
                            source_role_parent,
                            word_parent,
                            pointer,
                            boundary,
                        ))
                    }),
                json!({"field":pointer,"old":old,"expected_boundary":word_boundary(writer_parent, source_role_parent, word_parent, pointer, boundary),"error":error}),
            )?;
        }
        let mut changed = document.clone();
        changed["shared_operator_refinement"]["parent_artifact"] =
            json!(format!("blake3:{}", "0".repeat(64)));
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "shared_wrong_parent_cid_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains(word_boundary(
                        writer_parent,
                        source_role_parent,
                        word_parent,
                        "/shared_operator_refinement/parent_artifact",
                        "shared operator frozen parent differs",
                    ))
                }),
            json!({"error":error}),
        )?;
        let mut changed = document.clone();
        changed["shared_operator_refinement"]["continuation_fitted"] = json!(!witness
            ["continuation_fitted"]
            .as_bool()
            .ok_or("shared continuation stage flag absent")?);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "shared_stage_flag_identity_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains(word_boundary(
                        writer_parent,
                        source_role_parent,
                        word_parent,
                        "/shared_operator_refinement/continuation_fitted",
                        "shared operator identity differs",
                    ))
                }),
            json!({"error":error}),
        )?;
        let mut changed = document.clone();
        changed["shared_operator_refinement"]["unexpected_witness_field"] = json!(true);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "shared_unknown_witness_field_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains("unknown field") && e.contains("unexpected_witness_field")
                }),
            json!({"error":error}),
        )?;
    } else {
        checks.push(json!({"name":"shared_operator_checks","status":"NOT_APPLICABLE","reason":"supplied artifact has no shared_operator_refinement witness"}));
    }
    if let Some(witness) = document.get("word_emission").filter(|v| v.is_object()) {
        for (label, prompt, target) in [
            ("span_place", "User: tilva lives in Ash Court.\nUser: Where is tilva?\nExplain in a sentence. Assistant:", " Ash Court is the place.\n"),
            ("span_stop", "User: tilva lives in Ash Court.\nUser: Where is tilva?\nExplain the stop in a sentence. Assistant:", " Ash Court is the stop.\n"),
            ("word_place", "Record: velra in Lodov. Where is velra? Explain in a sentence. Answer:", " Lodov is the place.\n"),
        ] {
            let config = SessionConfig { session_id: format!("word-emission-{label}"), ..SessionConfig::default() };
            let mut direct = model.session(Control::Full)?;
            let mut api = api_model.create_session(config.clone())?;
            compare_turn(&model, &mut direct, &mut api, prompt, target, &format!("word_emission_{label}_api_direct_parity"), checks)?;
            let exported = api.export_state()?;
            let mut imported = api_model.create_session(config)?;
            imported.import_state(&exported)?;
            direct = model.restore_session(&direct.checkpoint()?)?;
            record(checks, &format!("word_emission_{label}_checkpoint_import"), imported.identity_scope() == api.identity_scope(),
                json!({"checkpoint_bytes":exported.len(),"scope":imported.identity_scope(),"development_case":true}))?;
            compare_turn(&model, &mut direct, &mut imported, SUM_13, "17.\n", &format!("word_emission_{label}_checkpoint_next_independent_sum"), checks)?;
        }
        let expected_parent =
            "blake3:2dc63b2c7b7073155fd3c5a8c743eaaace66f409483d86dd390966d4f9fc03f6";
        record(
            checks,
            "word_emission_parent_reconstruction_and_roundtrip",
            witness.get("parent_artifact").and_then(Value::as_str) == Some(expected_parent),
            json!({"parent_artifact":witness["parent_artifact"],"expected_parent":expected_parent,"current_artifact":expected_cid,
                "validated_by":["NativeModel::load_from_bytes","artifact_load_save_roundtrip"],
                "boundary":"Loader restores the complete previous lexical component and validates the frozen parent recursively; no separate extracted parent is claimed."}),
        )?;
        for (name, pointer, boundary) in [
            (
                "word_previous_lexical_parameters_rejected",
                "/word_emission/previous_lexical/router/biases/0",
                "word emission frozen lexical parameters differ",
            ),
            (
                "word_current_lexical_frozen_parameters_rejected",
                "/lexical_emission/router/biases/0",
                "word emission frozen lexical parameters differ",
            ),
        ] {
            let mut changed = document.clone();
            let bias = changed
                .pointer_mut(pointer)
                .ok_or("word-emission router bias absent")?;
            let old = bias
                .as_i64()
                .ok_or("word-emission bias is not an integer")?;
            if !(-32..=32).contains(&old) {
                return Err("word-emission bias outside documented range".into());
            }
            let new = if old == 32 { old - 1 } else { old + 1 };
            *bias = json!(new);
            let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
            record(
                checks,
                name,
                rejected
                    && error.as_deref().is_some_and(|e| {
                        e.contains(source_role_boundary(
                            writer_parent,
                            source_role_parent,
                            boundary,
                        ))
                    }),
                json!({"field":pointer,"old":old,"new":new,"expected_boundary":source_role_boundary(writer_parent, source_role_parent, boundary),"error":error}),
            )?;
        }
        // Change a legal new word-only root, keeping inherited roots exact,
        // to isolate current identity from the frozen-parameter guard.
        let mut changed = document.clone();
        let codes = changed
            .pointer_mut("/lexical_emission/router/codes")
            .and_then(Value::as_array_mut)
            .ok_or("lexical codes absent")?;
        let (index, code) = codes
            .iter_mut()
            .enumerate()
            .find(|(_, code)| {
                code["feature"]["kind"]
                    .as_u64()
                    .is_some_and(|k| matches!(k, 0 | 3))
                    && code["feature"]["a"].as_u64().is_some_and(|a| a >> 56 == 16)
            })
            .ok_or("new word-only lexical code absent")?;
        let root = code
            .pointer_mut("/roots/0")
            .ok_or("word-only root absent")?;
        let old = root.as_u64().ok_or("word-only root is not unsigned")?;
        if old >= 120 {
            return Err("word-only root outside H4 range".into());
        }
        let new = if old == 119 { 0 } else { old + 1 };
        *root = json!(new);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "word_new_root_identity_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains(source_role_boundary(
                        writer_parent,
                        source_role_parent,
                        "word emission identity differs",
                    ))
                }),
            json!({"code_index":index,"old":old,"new":new,"expected_boundary":source_role_boundary(writer_parent, source_role_parent, "word emission identity differs"),"error":error}),
        )?;
        let mut changed = document.clone();
        let tokens = changed["word_emission"]["tokens"]
            .as_array_mut()
            .ok_or("word-emission tokens absent")?;
        tokens.push(
            tokens
                .first()
                .ok_or("word-emission token population empty")?
                .clone(),
        );
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "word_duplicate_token_shape_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains(source_role_boundary(
                        writer_parent,
                        source_role_parent,
                        "invalid word emission tokens",
                    ))
                }),
            json!({"error":error}),
        )?;
        let mut changed = document.clone();
        let prime = changed
            .pointer_mut("/word_emission/dictionary/0/prime")
            .ok_or("word-emission dictionary prime absent")?;
        let old = prime
            .as_u64()
            .ok_or("word-emission dictionary prime is not unsigned")?;
        *prime = json!(old.checked_add(1).ok_or("word-emission prime overflow")?);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "word_dictionary_prime_shape_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains(source_role_boundary(
                        writer_parent,
                        source_role_parent,
                        "invalid word emission dictionary",
                    ))
                }),
            json!({"old":old,"error":error}),
        )?;
        let mut changed = document.clone();
        changed["word_emission"]["parent_artifact"] = json!(format!("blake3:{}", "0".repeat(64)));
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "word_wrong_parent_cid_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains(source_role_boundary(
                        writer_parent,
                        source_role_parent,
                        "word emission frozen parent differs",
                    ))
                }),
            json!({"error":error}),
        )?;
        let mut changed = document.clone();
        changed["word_emission"]["unexpected_witness_field"] = json!(true);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "word_unknown_witness_field_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains("unknown field") && e.contains("unexpected_witness_field")
                }),
            json!({"error":error}),
        )?;
    } else {
        checks.push(json!({"name":"word_emission_checks","status":"NOT_APPLICABLE","reason":"supplied artifact has no word_emission witness"}));
    }
    if let Some(witness) = document
        .get("source_role_refinement")
        .filter(|v| v.is_object())
    {
        for value in ["Lodov", "Talven"] {
            for prefix in [true, false] {
                for present in [false, true] {
                    let owner = if present { "velra" } else { "tovin" };
                    let prompt = if prefix {
                        format!("Record: {owner} in {value}. Explain the stop in a sentence. Where is velra? Answer:")
                    } else {
                        format!("Record: {owner} in {value}. Where is velra? Explain the stop in a sentence. Answer:")
                    };
                    let target = if present {
                        format!(" {value} is the stop.\n")
                    } else {
                        " Unknown.\n".into()
                    };
                    let label = format!(
                        "{}-{}-{}",
                        value.to_ascii_lowercase(),
                        if prefix { "prefix" } else { "suffix" },
                        if present { "present" } else { "absent" }
                    );
                    let config = SessionConfig {
                        session_id: format!("source-role-{label}"),
                        ..SessionConfig::default()
                    };
                    let mut direct = model.session(Control::Full)?;
                    let mut api = api_model.create_session(config.clone())?;
                    compare_turn(
                        &model,
                        &mut direct,
                        &mut api,
                        &prompt,
                        &target,
                        &format!("source_role_{label}_api_direct_parity"),
                        checks,
                    )?;
                    let exported = api.export_state()?;
                    let mut imported = api_model.create_session(config)?;
                    imported.import_state(&exported)?;
                    direct = model.restore_session(&direct.checkpoint()?)?;
                    record(
                        checks,
                        &format!("source_role_{label}_checkpoint_import"),
                        imported.identity_scope() == api.identity_scope(),
                        json!({"checkpoint_bytes":exported.len(),"scope":imported.identity_scope(),"development_case":true,"owner_present":present}),
                    )?;
                    compare_turn(
                        &model,
                        &mut direct,
                        &mut imported,
                        SUM_13,
                        "17.\n",
                        &format!("source_role_{label}_checkpoint_next_independent_sum"),
                        checks,
                    )?;
                }
            }
        }
        let expected_parent =
            "blake3:91ede422c51a2e80aa0dc4f579a005d94a60e8023a0960d7d891f98cc7e7db9b";
        record(
            checks,
            "source_role_parent_reconstruction_and_roundtrip",
            witness.get("parent_artifact").and_then(Value::as_str) == Some(expected_parent),
            json!({"parent_artifact":witness["parent_artifact"],"expected_parent":expected_parent,"current_artifact":expected_cid,
                "validated_by":["NativeModel::load_from_bytes","artifact_load_save_roundtrip"],
                "boundary":"Loader restores the complete previous source router and validates the original word-emission parent recursively; no separate extracted parent is claimed."}),
        )?;
        for (name, pointer, boundary) in [
            (
                "source_role_previous_router_parent_rejected",
                "/source_role_refinement/previous/biases/0",
                "source role refinement frozen parent differs",
            ),
            (
                "source_role_current_bias_frozen_rejected",
                "/source_routing/biases/0",
                "source role refinement frozen parameters differ",
            ),
        ] {
            let mut changed = document.clone();
            let bias = changed
                .pointer_mut(pointer)
                .ok_or("source-role router bias absent")?;
            let old = bias.as_i64().ok_or("source-role bias is not integer")?;
            if !(-32..=32).contains(&old) {
                return Err("source-role bias outside range".into());
            }
            let new = if old == 32 { old - 1 } else { old + 1 };
            *bias = json!(new);
            let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
            record(
                checks,
                name,
                rejected
                    && error
                        .as_deref()
                        .is_some_and(|e| e.contains(writer_boundary(writer_parent, boundary))),
                json!({"field":pointer,"old":old,"new":new,"expected_boundary":writer_boundary(writer_parent, boundary),"error":error}),
            )?;
        }
        let mut changed = document.clone();
        let codes = changed
            .pointer_mut("/source_routing/codes")
            .and_then(Value::as_array_mut)
            .ok_or("source routing codes absent")?;
        let identity = document
            .pointer("/geometry/identity")
            .and_then(Value::as_u64)
            .ok_or("geometry identity absent")?;
        let previous = witness["previous"]["codes"]
            .as_array()
            .ok_or("previous source codes absent")?;
        let (index, code) = codes
            .iter_mut()
            .enumerate()
            .find(|(i, c)| {
                c["feature"]["kind"] == 6
                    && previous
                        .get(*i)
                        .and_then(|p| p["roots"].as_array())
                        .is_some_and(|roots| roots.iter().any(|r| r.as_u64() != Some(identity)))
            })
            .ok_or("mutable source recency code absent")?;
        let root = code
            .pointer_mut("/roots/0")
            .ok_or("source recency root absent")?;
        let old = root.as_u64().ok_or("source recency root is not unsigned")?;
        if old >= 120 {
            return Err("source recency root outside H4 range".into());
        }
        let new = if old == 119 { 0 } else { old + 1 };
        *root = json!(new);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "source_role_recency_root_identity_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains(writer_boundary(
                        writer_parent,
                        "source role refinement identity differs",
                    ))
                }),
            json!({"code_index":index,"old":old,"new":new,"error":error}),
        )?;
        let mut changed = document.clone();
        changed["source_role_refinement"]["parent_artifact"] =
            json!(format!("blake3:{}", "0".repeat(64)));
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "source_role_wrong_parent_cid_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains(writer_boundary(
                        writer_parent,
                        "source role refinement frozen parent differs",
                    ))
                }),
            json!({"error":error}),
        )?;
        let mut changed = document.clone();
        changed["source_role_refinement"]["config"]["max_seconds"] = json!(0);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "source_role_invalid_configuration_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains(writer_boundary(
                        writer_parent,
                        "invalid source role refinement configuration",
                    ))
                }),
            json!({"error":error}),
        )?;
        let mut changed = document.clone();
        changed["source_role_refinement"]["training"] = json!([]);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "source_role_empty_receipts_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains(writer_boundary(
                        writer_parent,
                        "invalid source role refinement receipts",
                    ))
                }),
            json!({"error":error}),
        )?;
        let mut changed = document.clone();
        changed["source_role_refinement"]["unexpected_witness_field"] = json!(true);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "source_role_unknown_witness_field_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains("unknown field") && e.contains("unexpected_witness_field")
                }),
            json!({"error":error}),
        )?;
    } else {
        checks.push(json!({"name":"source_role_refinement_checks","status":"NOT_APPLICABLE","reason":"supplied artifact has no source_role_refinement witness"}));
    }
    if let Some(witness) = document.get("writer_lexical").filter(|v| v.is_object()) {
        let expected_parent =
            "blake3:82662f856908e548a9cccbb78809daf4821d243781ad497b661f5ee6d94c2032";
        let restored_parent = if let Some(parent) = nested_parent.as_ref() {
            parent.without_writer_lexical()?
        } else {
            model.without_writer_lexical()?
        };
        let parent_document: Value = serde_json::from_slice(&restored_parent.to_bytes()?)?;
        record(
            checks,
            "writer_lexical_exact_parent_and_cache_restoration",
            witness["parent_artifact"] == expected_parent
                && restored_parent.artifact_cid() == expected_parent
                && parent_document["relation_writer"] == document["relation_writer"]
                && parent_document["relation_writer_refinement"]
                    == document["relation_writer_refinement"],
            json!({"expected_parent":expected_parent,"restored_parent":restored_parent.artifact_cid(),
                "boundary":"The full reconstructed parent validates recursively; original writer and its cache remain unchanged and certified at that exact parent."}),
        )?;
        let mut changed = document.clone();
        let coefficient = changed
            .pointer_mut("/writer_lexical/rows/0/weight")
            .ok_or("writer lexical learned row absent")?;
        let old = coefficient
            .as_i64()
            .ok_or("writer lexical weight is not integer")?;
        if !(-1000000..=0).contains(&old) {
            return Err("writer lexical coefficient outside declared range".into());
        }
        let new = if old == -1000000 { old + 1 } else { old - 1 };
        *coefficient = json!(new);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "writer_lexical_valid_residual_identity_rejected",
            rejected
                && error
                    .as_deref()
                    .is_some_and(|e| e.contains("writer lexical identity differs")),
            json!({"old":old,"new":new,"error":error}),
        )?;
        for (name, pointer, new, boundary) in [
            (
                "writer_lexical_positive_coefficient_rejected",
                "/writer_lexical/rows/0/weight",
                1_u64,
                "invalid writer lexical residual rows",
            ),
            (
                "writer_lexical_unsupported_row_rejected",
                "/writer_lexical/rows/0/feature/kind",
                0,
                "invalid writer lexical residual rows",
            ),
            (
                "writer_lexical_unknown_row_prime_rejected",
                "/writer_lexical/rows/0/feature/a",
                u64::MAX,
                "invalid writer lexical residual rows",
            ),
            (
                "writer_lexical_dictionary_prime_rejected",
                "/writer_lexical/dictionary/0/prime",
                0,
                "invalid writer lexical dictionary",
            ),
            (
                "writer_lexical_configuration_rejected",
                "/writer_lexical/max_seconds",
                0,
                "invalid writer lexical configuration",
            ),
        ] {
            let mut changed = document.clone();
            let field = changed
                .pointer_mut(pointer)
                .ok_or("writer lexical mutation field absent")?;
            let old = field.clone();
            *field = json!(new);
            let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
            record(
                checks,
                name,
                rejected && error.as_deref().is_some_and(|e| e.contains(boundary)),
                json!({"field":pointer,"old":old,"new":new,"expected_boundary":boundary,"error":error}),
            )?;
        }
        let mut changed = document.clone();
        changed["writer_lexical"]["parent_artifact"] = json!(format!("blake3:{}", "0".repeat(64)));
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "writer_lexical_wrong_parent_rejected",
            rejected
                && error
                    .as_deref()
                    .is_some_and(|e| e.contains("writer lexical frozen parent differs")),
            json!({"error":error}),
        )?;
        let mut changed = document.clone();
        changed["writer_lexical"]["training"] = json!([]);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "writer_lexical_empty_receipts_rejected",
            rejected
                && error
                    .as_deref()
                    .is_some_and(|e| e.contains("invalid writer lexical receipts")),
            json!({"error":error}),
        )?;
        let mut changed = document.clone();
        changed["writer_lexical"]["unexpected_witness_field"] = json!(true);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "writer_lexical_unknown_field_rejected",
            rejected
                && error.as_deref().is_some_and(|e| {
                    e.contains("unknown field") && e.contains("unexpected_witness_field")
                }),
            json!({"error":error}),
        )?;
    } else {
        checks.push(json!({"name":"writer_lexical_checks","status":"NOT_APPLICABLE","reason":"supplied artifact has no writer_lexical witness"}));
    }
    if document
        .get("typed_role_refinement")
        .is_some_and(|v| !v.is_null())
    {
        // Stay inside the legal bias range: rejection must protect the frozen
        // parent binding, rather than merely reject a malformed scalar range.
        let mut changed = document.clone();
        let bias = changed
            .pointer_mut("/typed_role_refinement/previous/router/biases/0")
            .ok_or("refinement witness has no previous router bias")?;
        let old = bias
            .as_i64()
            .ok_or("previous router bias is not an integer")?;
        if !(-32..=32).contains(&old) {
            return Err("previous router bias lies outside documented range".into());
        }
        let new = if old == 32 { old - 1 } else { old + 1 };
        *bias = json!(new);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "changed_previous_router_frozen_parent_rejected",
            rejected
                && error
                    .as_deref()
                    .is_some_and(|e| e.contains("frozen parent differs")),
            json!({"field":"typed_role_refinement.previous.router.biases[0]","old":old,"new":new,"error":error}),
        )?;
        drop(changed);
        let mut changed = document;
        let wrong_cid = format!("blake3:{}", "0".repeat(64));
        let parent = changed
            .pointer_mut("/typed_role_refinement/parent_artifact")
            .ok_or("refinement parent absent")?;
        let old_parent = parent.clone();
        *parent = json!(wrong_cid);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "wrong_refinement_parent_cid_rejected",
            rejected,
            json!({"field":"typed_role_refinement.parent_artifact","old":old_parent,"new":wrong_cid,"error":error}),
        )?;
    } else {
        checks.push(json!({"name":"typed_role_refinement_corruption_checks","status":"NOT_APPLICABLE","reason":"supplied artifact has no typed_role_refinement witness"}));
    }
    Ok(())
}

fn main() -> CheckResult<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: native_artifact_recovery_check MODEL EXPECTED_CID OUTPUT_JSON".into());
    }
    let mut checks = Vec::new();
    let mut identity = Value::Null;
    let result = run(Path::new(&args[0]), &args[1], &mut checks, &mut identity);
    let report = json!({
        "schema":"uor-r4.native-artifact-recovery-check/1",
        "status":if result.is_ok() {"PASS"} else {"FAIL"},
        "scope":"Artifact integrity and actual narrow interface behavior only; no general capability, alpha, energy, or performance qualification.",
        "identity":identity,"checks":checks,
        "error":result.as_ref().err().map(|e|e.to_string()),
    });
    std::fs::write(&args[2], serde_json::to_vec_pretty(&report)?)?;
    println!(
        "{}: {}",
        report["status"].as_str().unwrap_or("FAIL"),
        args[2]
    );
    result
}
