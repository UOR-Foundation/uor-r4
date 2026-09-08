//! Mechanical read-state contracts; these fixtures establish no learned ability.
use super::completion_types::{CompletionAction, CompletionDecision, LexicalRead};
use super::lexical_emission::LexicalEmission;
use super::operation_transition_tests::mechanical_model;
use super::source_routing::{SourceCode, SourceRouting};
use super::value_types::ValueFeature;
use super::*;

fn lexical_model() -> Model {
    let mut model = mechanical_model(ValueAction::Add);
    let mut bytes = [0; 32];
    bytes[..4].copy_from_slice(b"copy");
    model.lexical_emission = Some(LexicalEmission {
        dictionary: vec![word_copy_types::WordCopyAddress {
            bytes,
            len: 4,
            prime: 2,
        }],
        tokens: vec![EOS],
        router: SourceRouting {
            schema: "uor-r4.geometric-source-routing/1".into(),
            parent_artifact: model.artifact_cid.clone(),
            codes: vec![SourceCode {
                feature: ValueFeature {
                    kind: 2,
                    a: 1,
                    b: 1026,
                },
                roots: [model.geometry.identity; 2],
            }],
            landmarks: vec![[model.geometry.identity; 2]; 2],
            // Base wins freely selected starts. Tests explicitly install a
            // pending read to isolate its causal state and exact byte cursor.
            biases: vec![1, 0],
            ranks: learned_routing_training::ranks(&model),
            training: vec![DocumentReceipt {
                id: "lexical-read-mechanical-only".into(),
                bytes: 1,
                text_cid: format!("blake3:{}", blake3::hash(b"x")),
            }],
            config: SourceRoutingConfig::default(),
        },
    });
    model.refresh_identity().unwrap();
    model.validate().unwrap();
    model
}

fn copied(model: &Model, value: i64) -> Session {
    let mut session = model.session(Control::Full).unwrap();
    session.observe(model, BOS).unwrap();
    for token in model.encode(&format!("source {value}; query:")).unwrap() {
        session.observe(model, token).unwrap();
    }
    session.begin_response(model).unwrap();
    for byte in value.to_string().bytes() {
        let prediction = session.predict(model).unwrap();
        assert_eq!(prediction.token, u32::from(byte) + 2);
        session.observe(model, prediction.token).unwrap();
    }
    assert_eq!(session.work.values.derived_writes, 1);
    assert!(session.completion.as_ref().unwrap().active);
    session
}

fn semantic_checkpoint(session: &Session) -> serde_json::Value {
    let mut wire: serde_json::Value =
        serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
    wire.as_object_mut().unwrap().remove("work");
    wire
}

fn selected_byte(session: &mut Session, token: u32, read: Option<LexicalRead>) {
    let state = session.completion.as_mut().unwrap();
    state.pending = Some(CompletionDecision {
        token,
        score: 1,
        write_id: state.anchor.unwrap().write_id,
        step: state.steps,
        at_seen: state.seen,
        action: CompletionAction::Emit,
    });
    state.pending_lexical_read = read;
}

fn start_read(model: &Model, session: &mut Session, record_id: u64) -> LexicalRead {
    let record = session
        .values
        .as_ref()
        .unwrap()
        .records
        .iter()
        .find(|r| r.id == record_id)
        .unwrap();
    let read = LexicalRead {
        record_id,
        start_at: session.completion.as_ref().unwrap().seen,
        numeral: numeral::Numeral::from_zphi(crate::prime_route_attention::ZPhi::new(
            record.value,
            0,
        ))
        .unwrap(),
        cursor: 0,
    };
    selected_byte(session, read.numeral.tokens[0], Some(read));
    session.observe(model, read.numeral.tokens[0]).unwrap();
    read
}

#[test]
fn native_lexical_read_signed_multidigit_cursor_restores_without_value_writes() {
    let model = lexical_model();
    for value in [-123, 17, 0, i64::MIN] {
        let mut session = copied(&model, value);
        let source = session.values.as_ref().unwrap().records[0].id;
        let read = start_read(&model, &mut session, source);
        for cursor in 1..=read.numeral.len {
            let checkpoint = session.checkpoint().unwrap();
            let mut restored = model.restore_session(&checkpoint).unwrap();
            let committed = session.completion.as_ref().unwrap().lexical_read.unwrap();
            assert_eq!(committed.record_id, source);
            assert_eq!(committed.cursor, cursor);
            assert_eq!(
                restored.completion.as_ref().unwrap().lexical_read,
                Some(committed)
            );
            if cursor < read.numeral.len {
                let expected = read.numeral.tokens[usize::from(cursor)];
                let prediction = session.predict(&model).unwrap();
                assert_eq!(prediction.token, expected);
                assert_eq!(restored.predict(&model).unwrap(), prediction);
                assert_eq!(
                    restored.completion.as_ref().unwrap().pending_lexical_read,
                    session.completion.as_ref().unwrap().pending_lexical_read
                );
                session.observe(&model, expected).unwrap();
                restored.observe(&model, expected).unwrap();
                assert_eq!(
                    semantic_checkpoint(&restored),
                    semantic_checkpoint(&session)
                );
            }
        }
        assert_eq!(session.work.values.derived_writes, 1);
        assert_eq!(session.values.as_ref().unwrap().records.len(), 2);
        let complete = session.completion.as_ref().unwrap().lexical_read;
        let space = u32::from(b' ') + 2;
        selected_byte(&mut session, space, None);
        session.observe(&model, space).unwrap();
        assert_eq!(session.completion.as_ref().unwrap().lexical_read, complete);
        model
            .restore_session(&session.checkpoint().unwrap())
            .unwrap();
    }
}

#[test]
fn native_lexical_read_prediction_is_idempotent_and_mismatch_clears_cursor() {
    let model = lexical_model();
    let mut session = copied(&model, 17);
    let id = session.values.as_ref().unwrap().records[0].id;
    start_read(&model, &mut session, id);
    let before = semantic_checkpoint(&session);
    let first = session.predict(&model).unwrap();
    let read = session.completion.as_ref().unwrap().pending_lexical_read;
    assert_eq!(session.predict(&model).unwrap(), first);
    assert_eq!(
        session.completion.as_ref().unwrap().pending_lexical_read,
        read
    );
    assert_eq!(semantic_checkpoint(&session), before);
    session.observe(&model, u32::from(b'9') + 2).unwrap();
    assert!(session.completion.as_ref().unwrap().lexical_read.is_none());
    assert!(session
        .completion
        .as_ref()
        .unwrap()
        .pending_lexical_read
        .is_none());
    assert_eq!(session.work.values.derived_writes, 1);
    assert_eq!(session.values.as_ref().unwrap().records.len(), 2);
}

#[test]
fn native_lexical_read_preserves_selected_identity_among_equal_values() {
    let model = lexical_model();
    let mut source_read = copied(&model, 17);
    let mut result_read = model
        .restore_session(&source_read.checkpoint().unwrap())
        .unwrap();
    let records = &source_read.values.as_ref().unwrap().records;
    let source_id = records[0].id;
    let result_id = records[1].id;
    assert_eq!(records[0].value, records[1].value);
    assert_ne!(source_id, result_id);
    start_read(&model, &mut source_read, source_id);
    start_read(&model, &mut result_read, result_id);
    assert_eq!(
        source_read.completion.as_ref().unwrap().last,
        result_read.completion.as_ref().unwrap().last
    );
    for (session, id) in [(&source_read, source_id), (&result_read, result_id)] {
        let restored = model
            .restore_session(&session.checkpoint().unwrap())
            .unwrap();
        assert_eq!(
            restored
                .completion
                .as_ref()
                .unwrap()
                .lexical_read
                .unwrap()
                .record_id,
            id
        );
    }
    assert_ne!(
        source_read.completion.as_ref().unwrap().lexical_read,
        result_read.completion.as_ref().unwrap().lexical_read
    );
}

#[test]
fn native_lexical_read_checkpoint_rejects_forged_payload_cursor_and_transients() {
    let model = lexical_model();
    let mut session = copied(&model, 17);
    let id = session.values.as_ref().unwrap().records[0].id;
    start_read(&model, &mut session, id);
    let wire: serde_json::Value = serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
    for (field, value) in [
        ("record_id", u64::MAX),
        ("cursor", 0),
        ("cursor", 3),
        ("start_at", u64::MAX),
    ] {
        let mut forged = wire.clone();
        forged["completion"]["lexical_read"][field] = serde_json::json!(value);
        assert!(model
            .restore_session(&serde_json::to_vec(&forged).unwrap())
            .is_err());
    }
    let mut payload = wire.clone();
    payload["completion"]["lexical_read"]["numeral"]["tokens"][0] =
        serde_json::json!(u32::from(b'9') + 2);
    assert!(model
        .restore_session(&serde_json::to_vec(&payload).unwrap())
        .is_err());
    let mut transient = wire;
    transient["completion"]["pending_lexical_read"] = serde_json::Value::Null;
    assert!(model
        .restore_session(&serde_json::to_vec(&transient).unwrap())
        .is_err());
}

#[test]
fn native_lexical_read_old_artifact_rejects_even_null_read_field() {
    let model = mechanical_model(ValueAction::Add);
    let session = copied(&model, 17);
    let bytes = session.checkpoint().unwrap();
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire["completion"].get("lexical_read").is_none());
    model.restore_session(&bytes).unwrap();
    wire["completion"]["lexical_read"] = serde_json::Value::Null;
    assert!(model
        .restore_session(&serde_json::to_vec(&wire).unwrap())
        .is_err());
}

#[test]
fn native_lexical_read_disabled_control_stops_a_restored_partial_read() {
    let model = lexical_model();
    let mut session = copied(&model, 17);
    let id = session.values.as_ref().unwrap().records[0].id;
    start_read(&model, &mut session, id);
    let committed = session.completion.as_ref().unwrap().lexical_read.unwrap();
    assert_eq!(committed.cursor, 1);
    assert!(committed.cursor < committed.numeral.len);

    // Apply the control through the public checkpoint format while retaining
    // the actual partial-read evidence. This is not just a fresh-start ablation.
    let mut wire: serde_json::Value =
        serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
    wire["control"] = serde_json::to_value(Control::LexicalRecordReadDisabled).unwrap();
    let mut restored = model
        .restore_session(&serde_json::to_vec(&wire).unwrap())
        .unwrap();
    assert_eq!(
        restored.completion.as_ref().unwrap().lexical_read,
        Some(committed)
    );
    let prediction = restored.predict(&model).unwrap();
    assert_eq!(prediction.token, EOS); // Mechanical parent completion is Stop.
    assert!(restored
        .completion
        .as_ref()
        .unwrap()
        .pending_lexical_read
        .is_none());
    assert!(restored.value_decision().is_none());
    restored.observe(&model, prediction.token).unwrap();
    assert!(restored.completion.as_ref().unwrap().lexical_read.is_none());
    assert_eq!(restored.work.values.derived_writes, 1);
}
