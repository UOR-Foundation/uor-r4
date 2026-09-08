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

    let document: Value = serde_json::from_slice(&bytes)?;
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
            let boundary = if composed_parent {
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
                rejected && error.as_deref().is_some_and(|e| e.contains(boundary)),
                json!({"field":pointer,"old":old,"new":new,"expected_boundary":boundary,"error":error}),
            )?;
        }
        let parent_boundary = if composed_parent {
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
                rejected && error.as_deref().is_some_and(|e| e.contains(boundary)),
                json!({"field":pointer,"old":old,"new":new,"expected_boundary":boundary,"error":error}),
            )?;
        }
        let mut changed = document.clone();
        let wrong_parent = format!("blake3:{}", "0".repeat(64));
        changed["composed_output"]["parent_artifact"] = json!(wrong_parent);
        let (rejected, error) = rejection(&serde_json::to_vec(&changed)?);
        record(
            checks,
            "composed_wrong_parent_cid_rejected",
            rejected
                && error
                    .as_deref()
                    .is_some_and(|e| e.contains("composed output frozen parent differs")),
            json!({"old":witness["parent_artifact"],"new":wrong_parent,"expected_boundary":"composed output frozen parent differs","error":error}),
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
