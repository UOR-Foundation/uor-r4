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
const LIMIT: usize = 24;
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
    if session.is_response_active() {
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
        original_next.text == restored_next.text
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

    let document: Value = serde_json::from_slice(&bytes)?;
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
