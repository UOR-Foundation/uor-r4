//! Interface checks use a tiny explicit test-only artifact. They do not qualify language capability.
use std::sync::{Arc, OnceLock};
use uor_r4_api::native_capability_api::*;
use uor_r4_core::native_geometric::{Config, Control, Document, Model, Trainer, BOS, EOS};

fn fixture() -> NativeModel {
    static MODEL: OnceLock<Arc<Model>> = OnceLock::new();
    NativeModel::from_model(
        MODEL
            .get_or_init(|| {
                let docs = vec![Document {
                    id: "interface-fixture".into(),
                    text: "hello river.\nhello forest.\nThe river flows.\n".into(),
                }];
                let mut trainer = Trainer::new(Config::default(), &docs).unwrap();
                trainer.train_documents(&docs).unwrap();
                Arc::new(trainer.compile().unwrap())
            })
            .clone(),
    )
    .unwrap()
}
fn direct(model: &Model, prompt: &str, limit: usize) -> (String, usize) {
    let mut session = model.session(Control::Full).unwrap();
    session.observe(model, BOS).unwrap();
    for token in model.encode(prompt).unwrap() {
        session.observe(model, token).unwrap();
    }
    session.begin_response(model).unwrap();
    let mut tokens = vec![];
    for _ in 0..limit {
        let token = session.predict(model).unwrap().token;
        session.observe(model, token).unwrap();
        if token == EOS {
            break;
        }
        tokens.push(token);
    }
    (
        String::from_utf8(model.decode(&tokens).unwrap()).unwrap(),
        tokens.len(),
    )
}
#[test]
fn explicit_artifact_required_and_identity_is_bound() {
    assert!(NativeModel::load_from_bytes(&[]).is_err());
    assert!(NativeModel::default_baseline().is_err());
    assert!(NativeModel::load_from_bytes(b"invalid").is_err());
    let model = fixture();
    let restored = NativeModel::load_from_bytes(&model.inner_model().to_bytes().unwrap()).unwrap();
    assert_eq!(restored.artifact_cid(), model.inner_model().artifact_cid());
    assert_eq!(
        restored.metadata().api_schema_version,
        "uor-r4.native-capability-api/2"
    );
    assert_eq!(
        restored.metadata().schema_version,
        uor_r4_core::native_geometric::SCHEMA
    );
    assert!(!restored.metadata().zero_heap_alloc_hot_path);
    assert!(restored
        .metadata()
        .truth_matrix
        .language_prose
        .starts_with("UNQUALIFIED"));
}
#[test]
fn direct_prediction_parity_without_keyword_responses() {
    let model = fixture();
    for prompt in [
        "hello",
        "Explain the manifold",
        "What is serving?",
        "total:",
    ] {
        let expected = direct(&model.inner_model(), prompt, 24);
        let mut session = model.create_session(SessionConfig::default()).unwrap();
        let mut streamed = String::new();
        let result = session
            .complete_stream(
                CompletionRequest {
                    prompt: prompt.into(),
                    max_tokens: Some(24),
                    temperature: None,
                    stop_sequences: vec![],
                },
                |piece| {
                    streamed.push_str(piece);
                    true
                },
            )
            .unwrap();
        assert_eq!(
            (result.text.clone(), result.token_count),
            expected,
            "{prompt}"
        );
        assert_eq!(streamed, result.text);
        assert_eq!(result.memory_facts_read, None);
    }
}
#[test]
fn request_limits_and_recoverable_errors() {
    let model = fixture();
    let mut session = model.create_session(SessionConfig::default()).unwrap();
    for count in [0, 4097] {
        let mut request = CompletionRequest::new("hello");
        request.max_tokens = Some(count);
        assert!(session.complete(request).is_err());
    }
    let mut request = CompletionRequest::new("hello");
    request.temperature = Some(0.7);
    assert!(session.complete(request).is_err());
    let mut request = CompletionRequest::new("hello");
    request.stop_sequences = vec![String::new()];
    assert!(session.complete(request).is_err());
    assert!(session.ingest(&"x".repeat(1024 * 1024 + 1)).is_err());
    let result = session.complete(CompletionRequest::new("hello")).unwrap();
    assert!(result.token_count <= 128);
}
#[test]
fn incremental_generation_and_cancellation() {
    let model = fixture();
    let expected = direct(&model.inner_model(), "hello", 24);
    let runtime = WasmModelRuntime::new(model);
    let handle = runtime
        .wasm_create_session("session", "user", "project")
        .unwrap();
    runtime.wasm_ingest(handle, "hello").unwrap();
    let mut text = String::new();
    let mut count = 0;
    for _ in 0..24 {
        let result: CompletionResponse =
            serde_json::from_str(&runtime.wasm_generate_step(handle, 1).unwrap()).unwrap();
        text.push_str(&result.text);
        count += result.token_count;
        if result.stopped_by != "length" {
            break;
        }
    }
    assert_eq!((text, count), expected);
    runtime.wasm_ingest(handle, "hello").unwrap();
    runtime.wasm_cancel(handle).unwrap();
    let cancelled: CompletionResponse =
        serde_json::from_str(&runtime.wasm_generate_step(handle, 1).unwrap()).unwrap();
    assert_eq!(cancelled.stopped_by, "cancelled");
    assert_eq!(cancelled.token_count, 0);
    runtime.wasm_ingest(handle, "hello").unwrap();
    let resumed: CompletionResponse =
        serde_json::from_str(&runtime.wasm_generate_step(handle, 1).unwrap()).unwrap();
    assert_ne!(resumed.stopped_by, "cancelled");
    runtime.wasm_free_session(handle);
    assert_eq!(
        runtime.wasm_ingest(handle, "hello").unwrap_err(),
        NativeApiError::SessionNotFound(handle)
    );
}
#[test]
fn checkpoint_scope_isolation_and_session_capacity() {
    let model = fixture();
    let config = SessionConfig::default();
    let mut a = model.create_session(config.clone()).unwrap();
    a.store_fact("ada", "Rome").unwrap();
    let bytes = a.export_state().unwrap();
    let mut b = model
        .create_session(SessionConfig {
            user_id: "another-user".into(),
            ..config.clone()
        })
        .unwrap();
    assert!(matches!(
        b.import_state(&bytes),
        Err(NativeApiError::IdentityViolation(_))
    ));
    assert_eq!(b.query_memory("ada").unwrap(), None);
    let mut same = model.create_session(config).unwrap();
    same.import_state(&bytes).unwrap();
    assert_eq!(same.query_memory("ada").unwrap().as_deref(), Some("Rome"));
    let runtime = WasmModelRuntime::new(model);
    for i in 0..32 {
        runtime
            .wasm_create_session(&format!("s{i}"), "u", "p")
            .unwrap();
    }
    assert!(matches!(
        runtime.wasm_create_session("overflow", "u", "p"),
        Err(NativeApiError::ResourceLimit(_))
    ));
    runtime.wasm_free_session(1);
    assert!(runtime.wasm_create_session("replacement", "u", "p").is_ok());
}
#[test]
fn stream_stop_sequence_and_callback_cancellation() {
    let model = fixture();
    let expected = direct(&model.inner_model(), "hello", 24).0;
    assert!(!expected.is_empty(), "fixture must generate output");
    let stop: String = expected.chars().take(2).collect();
    let mut session = model.create_session(SessionConfig::default()).unwrap();
    let mut chunks = String::new();
    let result = session
        .complete_stream(
            CompletionRequest {
                prompt: "hello".into(),
                max_tokens: Some(24),
                temperature: None,
                stop_sequences: vec![stop],
            },
            |piece| {
                chunks.push_str(piece);
                true
            },
        )
        .unwrap();
    assert_eq!(result.stopped_by, "stop_sequence");
    assert!(result.text.is_empty());
    assert!(chunks.is_empty());
    session.reset().unwrap();
    let result = session
        .complete_stream(CompletionRequest::new("hello"), |_| false)
        .unwrap();
    assert_eq!(result.stopped_by, "cancelled");
}
#[test]
#[ignore = "requires explicit retained artifact path; a missing artifact is not a pass"]
fn retained_artifact_api_parity() {
    let path = std::env::var("UOR_R4_MODEL").expect("UOR_R4_MODEL is required");
    let model = NativeModel::load_from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(
        model.artifact_cid(),
        "blake3:79710468e3b75905550c15fbea62079e4267df774dbf4bc90ea45114826e4bfd"
    );
    for prompt in [
        "the report says quiet river holds selra. Where is selra? Answer:",
        "hello",
        "Explain the manifold",
    ] {
        let expected = direct(&model.inner_model(), prompt, 24);
        let mut session = model.create_session(SessionConfig::default()).unwrap();
        let result = session
            .complete(CompletionRequest {
                prompt: prompt.into(),
                max_tokens: Some(24),
                temperature: None,
                stop_sequences: vec![],
            })
            .unwrap();
        assert_eq!((result.text, result.token_count), expected, "{prompt}");
    }
}

#[test]
#[ignore = "requires retained artifact; checks actual input capture across response turns"]
fn retained_artifact_multiple_turn_input_capture() {
    let path = std::env::var("UOR_R4_MODEL").expect("UOR_R4_MODEL is required");
    let model = NativeModel::load_from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(
        model.artifact_cid(),
        "blake3:79710468e3b75905550c15fbea62079e4267df774dbf4bc90ea45114826e4bfd"
    );
    let mut session = model.create_session(SessionConfig::default()).unwrap();
    // Exact initial_prompt/initial_response pairs from the retained construction
    // source typed-roles-source.json: roles-open/0/0 and roles-open/1/0.
    // These are independent questions across turns, not an Add→Add claim.
    for (prompt, target) in [
        ("User: suri has 13 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:", "17.\n"),
        ("User: suri has 14 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:", "18.\n"),
    ] {
        let response = session
            .complete(CompletionRequest {
                prompt: prompt.into(),
                max_tokens: Some(24),
                temperature: None,
                stop_sequences: vec![],
            })
            .unwrap();
        assert_eq!(
            response.text,
            target,
            "new input after previous response: {prompt}"
        );
    }
}

#[test]
fn incremental_host_finish_allows_checkpoint_after_budget_and_cancel() {
    let runtime = WasmModelRuntime::new(fixture());
    let handle = runtime
        .wasm_create_session("finish", "user", "project")
        .unwrap();
    runtime.wasm_ingest(handle, "hello").unwrap();
    let first: CompletionResponse =
        serde_json::from_str(&runtime.wasm_generate_step(handle, 1).unwrap()).unwrap();
    assert_eq!(first.stopped_by, "length");
    assert!(runtime.wasm_export_session(handle).is_err());
    let finish: CompletionResponse =
        serde_json::from_str(&runtime.wasm_finish_generation(handle).unwrap()).unwrap();
    assert_eq!(finish.token_count, 0);
    let checkpoint = runtime.wasm_export_session(handle).unwrap();
    runtime.wasm_import_session(handle, &checkpoint).unwrap();
    runtime.wasm_ingest(handle, "hello").unwrap();
    runtime.wasm_generate_step(handle, 1).unwrap();
    runtime.wasm_cancel(handle).unwrap();
    let finish: CompletionResponse =
        serde_json::from_str(&runtime.wasm_finish_generation(handle).unwrap()).unwrap();
    assert_eq!(finish.stopped_by, "cancelled");
    assert_eq!(finish.token_count, 0);
    assert!(runtime.wasm_export_session(handle).is_ok());
}

#[test]
#[ignore = "requires retained artifact; input chunks must not reset a partial numeral"]
fn retained_artifact_chunked_prefill_preserves_literal() {
    let path = std::env::var("UOR_R4_MODEL").expect("UOR_R4_MODEL is required");
    let model = NativeModel::load_from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(
        model.artifact_cid(),
        "blake3:79710468e3b75905550c15fbea62079e4267df774dbf4bc90ea45114826e4bfd"
    );
    let chunks = [
        "User: suri has 1",
        "3 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:",
    ];
    let mut joined = model.create_session(SessionConfig::default()).unwrap();
    let expected = joined
        .complete(CompletionRequest::new(chunks.concat()))
        .unwrap();
    assert_eq!(expected.text, "17.\n");
    let mut chunked = model.create_session(SessionConfig::default()).unwrap();
    chunked.ingest(chunks[0]).unwrap();
    // A prefill checkpoint must preserve the unfinished literal scanner, too.
    let checkpoint = chunked.export_state().unwrap();
    chunked.import_state(&checkpoint).unwrap();
    chunked.ingest(chunks[1]).unwrap();
    let actual = chunked.complete(CompletionRequest::new("")).unwrap();
    assert_eq!(actual.text, expected.text);
    assert_eq!(actual.token_count, expected.token_count);
}
