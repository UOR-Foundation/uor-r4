use uor_r4_api::native_capability_api::*;
use uor_r4_core::native_geometric::durable_memory::IdentityScope;
use uor_r4_core::native_geometric::SCHEMA;

#[test]
fn test_native_model_load_and_metadata() {
    let model = NativeModel::load_from_bytes(&[]).expect("Default model load should succeed");
    let meta = model.metadata();

    assert_eq!(meta.schema_version, SCHEMA);
    assert_eq!(meta.canonical_uor_address, "uor:native-geometric/r4/1");
    assert_eq!(meta.backend_name, BACKEND_IDENTIFIER);
    assert!(meta.is_provider_free);
    assert!(meta.zero_matmul_serving);
    assert!(meta.zero_heap_alloc_hot_path);
    assert_eq!(meta.supported_modalities.len(), 5);
}

#[test]
fn test_capability_truth_matrix() {
    let model = NativeModel::load_from_bytes(&[]).expect("Default model load should succeed");
    let truth = &model.metadata().truth_matrix;

    assert!(truth.language_prose.contains("#973"));
    assert!(truth.causal_attention.contains("#1139"));
    assert!(truth.multi_step_reasoning.contains("#955"));
    assert!(truth.executable_coding.contains("#1088"));
    assert!(truth.durable_memory.contains("#962"));
    assert!(truth.serving_guarantees.contains("#964"));
    assert!(truth.m1_performance_profile.contains("#963"));
    assert!(truth.general_ai_disavowal.contains("unproven"));
}

#[test]
fn test_native_session_ingest_and_complete() {
    let model = NativeModel::load_from_bytes(&[]).expect("Model load should succeed");
    let config = SessionConfig {
        session_id: "test-sess-1".into(),
        user_id: "user-alpha".into(),
        project_id: "proj-1".into(),
        ..Default::default()
    };

    let mut session = model
        .create_session(config)
        .expect("Session creation should succeed");

    let receipt = session
        .ingest("The quiet river flows gently through the forest.")
        .expect("Ingest should succeed");
    assert!(receipt.ingested_bytes > 0);
    assert!(!receipt.text_cid.is_empty());

    let req = CompletionRequest {
        prompt: String::new(),
        max_tokens: Some(10),
        temperature: Some(0.0),
        stop_sequences: Vec::new(),
    };

    let resp = session.complete(req).expect("Completion should succeed");
    assert!(resp.stopped_by == "eos" || resp.stopped_by == "length");
}

#[test]
fn test_native_session_streaming_and_cancellation() {
    let model = NativeModel::load_from_bytes(&[]).expect("Model load should succeed");
    let config = SessionConfig::default();
    let mut session = model
        .create_session(config)
        .expect("Session creation should succeed");

    session
        .ingest("In the quiet woods")
        .expect("Ingest should succeed");

    let mut chunk_count = 0;
    let req = CompletionRequest {
        prompt: String::new(),
        max_tokens: Some(20),
        temperature: Some(0.0),
        stop_sequences: Vec::new(),
    };

    let resp = session
        .complete_stream(req, |_chunk| {
            chunk_count += 1;
            // Early cancel after 2 chunks
            chunk_count < 2
        })
        .expect("Streaming should complete");

    assert!(resp.stopped_by == "cancelled" || resp.stopped_by == "eos");
}

#[test]
fn test_identity_scoped_memory_isolation() {
    let model = NativeModel::load_from_bytes(&[]).expect("Model load should succeed");

    // Session A (User 1, Project 1)
    let config_a = SessionConfig {
        session_id: "session-a".into(),
        user_id: "user-1".into(),
        project_id: "proj-1".into(),
        ..Default::default()
    };
    let mut session_a = model.create_session(config_a).expect("Session A created");

    // Session B (User 2, Project 2)
    let config_b = SessionConfig {
        session_id: "session-b".into(),
        user_id: "user-2".into(),
        project_id: "proj-2".into(),
        ..Default::default()
    };
    let session_b = model.create_session(config_b).expect("Session B created");

    session_a
        .store_fact("alice", "paris")
        .expect("Fact stored in session A");

    // Session A can query fact
    let retrieved_a = session_a.query_memory("alice").expect("Query session A");
    assert_eq!(retrieved_a.as_deref(), Some("paris"));

    let record_a = session_a
        .query_memory_record("alice")
        .expect("Record query");
    assert!(record_a.is_some());
    assert_eq!(record_a.unwrap().value, "paris");

    // Session B CANNOT see fact from Session A (strict user/project isolation!)
    let retrieved_b = session_b.query_memory("alice").expect("Query session B");
    assert!(retrieved_b.is_none());
}

#[test]
fn test_session_export_import_roundtrip() {
    let model = NativeModel::load_from_bytes(&[]).expect("Model load should succeed");

    let config = SessionConfig {
        session_id: "session-persist".into(),
        user_id: "user-test".into(),
        project_id: "proj-test".into(),
        ..Default::default()
    };
    let mut session_1 = model
        .create_session(config.clone())
        .expect("Session 1 created");

    session_1
        .store_fact("server", "online")
        .expect("Fact stored");

    // Export state
    let state_bytes = session_1.export_state().expect("State exported");
    assert!(!state_bytes.is_empty());

    // Create fresh session with same identity scope and import state
    let mut session_2 = model.create_session(config).expect("Session 2 created");
    assert!(session_2.query_memory("server").unwrap().is_none());

    session_2
        .import_state(&state_bytes)
        .expect("State imported");
    let recovered = session_2.query_memory("server").unwrap();
    assert_eq!(recovered.as_deref(), Some("online"));
}

#[test]
fn test_wasm_runtime_bridge_parity() {
    let model = NativeModel::load_from_bytes(&[]).expect("Model load should succeed");
    let runtime = WasmModelRuntime::new(model);

    let caps_json = runtime.wasm_get_capabilities();
    assert!(caps_json.contains(BACKEND_IDENTIFIER));
    assert!(caps_json.contains("uor:native-geometric/r4/1"));

    let handle = runtime
        .wasm_create_session("wasm-session-01", "user-web", "project-studio")
        .expect("WASM session created");
    assert_eq!(handle, 1);

    let receipt_json = runtime
        .wasm_ingest(handle, "Hello native geometric language")
        .expect("WASM ingest should succeed");
    assert!(receipt_json.contains("ingested_bytes"));

    let step_json = runtime
        .wasm_generate_step(handle, 5)
        .expect("WASM step should succeed");
    assert!(step_json.contains("token_count"));

    // State export and import in WASM
    let exported = runtime
        .wasm_export_session(handle)
        .expect("WASM export should succeed");
    assert!(runtime.wasm_import_session(handle, &exported).is_ok());

    // Cancellation
    assert!(runtime.wasm_cancel(handle).is_ok());

    // Free session
    runtime.wasm_free_session(handle);

    // Call on freed session must return SessionNotFound
    let err = runtime.wasm_ingest(handle, "test").unwrap_err();
    assert_eq!(err, NativeApiError::SessionNotFound(handle));
}

#[test]
fn test_native_api_error_handling() {
    // Empty identity fields
    let err = IdentityScope::new("", "proj", "sess").unwrap_err();
    assert!(err.0.contains("empty"));

    // Error formatting check
    let err_load = NativeApiError::ModelLoad("corrupted bytes".into());
    assert!(format!("{err_load}").contains("corrupted bytes"));

    let err_cancel = NativeApiError::Cancelled;
    assert_eq!(format!("{err_cancel}"), "Operation was cancelled by caller");
}
