//! Mechanical fixtures exercise transition contracts, not learned capability.
use super::completion_types::*;
use super::operation_transition::{self, OperationTransition};
use super::source_routing::{SourceCode, SourceRouting};
use super::value_types::*;
use super::*;

pub(super) fn mechanical_model(next: ValueAction) -> Model {
    let documents = [Document {
        id: "operation-transition-mechanical-catalog".into(),
        text: "source query copy total 17 34 answer ; : .\n".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 64,
            candidate_limit: 16,
            max_lexical_pieces: 128,
            ..Config::default()
        },
        &documents,
    )
    .unwrap();
    trainer.train_documents(&documents).unwrap();
    let mut model = trainer.compile().unwrap();
    // The first operator is a mechanical Copy; these are unit fixtures only.
    model.values = Some(ValueModel {
        schema: LEXEME_VALUE_SCHEMA.into(),
        codec: numeral::NUMERAL_CODEC.into(),
        capacity: VALUES,
        rows: vec![ValueRow {
            feature: ValueFeature {
                kind: 0,
                a: 0,
                b: 0,
            },
            weight: 4096,
        }],
        continuation_score: 4096,
        fit_config: [0; 4],
        training: vec![],
    });
    model.refresh_identity().unwrap();
    let baseline_artifact = model.artifact_cid.clone();
    model.completion = Some(CompletionModel {
        schema: COMPLETION_SCHEMA.into(),
        baseline_artifact,
        rows: vec![ScoreRow {
            feature: Feature { kind: 0, value: 0 },
            default_score: 0,
            scores: vec![TokenScore {
                token: EOS,
                score: 1000,
            }],
            postings: vec![EOS],
        }],
        global_postings: vec![EOS],
        fit_config: [0; 4],
        fit_positions: 0,
        training: vec![],
    });
    model.refresh_identity().unwrap();
    let mut bytes = [0; 32];
    bytes[..4].copy_from_slice(b"copy");
    model.operation_transition = Some(OperationTransition {
        dictionary: vec![word_copy_types::WordCopyAddress {
            bytes,
            len: 4,
            prime: 2,
        }],
        max_operations: 3,
        router: SourceRouting {
            schema: "uor-r4.geometric-source-routing/1".into(),
            parent_artifact: model.artifact_cid.clone(),
            codes: vec![SourceCode {
                feature: ValueFeature {
                    kind: 0,
                    a: 1,
                    b: 1,
                },
                roots: [model.geometry.identity; 2],
            }],
            landmarks: vec![[model.geometry.identity; 2]; 3],
            biases: if next == ValueAction::Copy {
                vec![0, 1, -1]
            } else {
                vec![0, -1, 1]
            },
            ranks: learned_routing_training::ranks(&model),
            training: vec![DocumentReceipt {
                id: "mechanical-test-only".into(),
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

pub(super) fn first_completed(model: &Model) -> Session {
    let mut session = model.session(Control::Full).unwrap();
    session.observe(model, BOS).unwrap();
    for token in model.encode("source 17; query:").unwrap() {
        session.observe(model, token).unwrap();
    }
    session.begin_response(model).unwrap();
    for byte in b"17" {
        let prediction = session.predict(model).unwrap();
        assert_eq!(prediction.token, u32::from(*byte) + 2);
        session.observe(model, prediction.token).unwrap();
    }
    assert_eq!(session.values.as_ref().unwrap().operations_committed, 1);
    assert!(session.values.as_ref().unwrap().emission.is_none());
    session
}

fn semantic_checkpoint(session: &Session) -> serde_json::Value {
    let mut value: serde_json::Value =
        serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
    value.as_object_mut().unwrap().remove("work");
    value
}

fn direct_offer(model: &Model, session: &mut Session) -> Option<Candidate> {
    let values = session.values.as_mut().unwrap();
    values.pending = None;
    let latest = values.records.iter().rev().find(|r| r.derived).unwrap().id;
    let mut completion = session.completion.clone().unwrap();
    completion.pending = Some(CompletionDecision {
        token: EOS,
        score: 1000,
        write_id: latest,
        step: completion.steps,
        at_seen: values.seen,
        action: CompletionAction::Stop,
    });
    operation_transition::offer(
        model,
        values,
        &completion,
        Candidate {
            token: EOS,
            score: 1000,
        },
        Control::Full,
        &mut session.work.values,
    )
}

#[test]
fn native_operation_transition_excludes_copy_alias_add_but_keeps_equal_distinct_literals() {
    let model = mechanical_model(ValueAction::Add);
    let mut session = first_completed(&model);
    // Literal 17 and its actual Copy 17 are one origin; Add must not double it.
    assert!(direct_offer(&model, &mut session).is_none());
    let values = session.values.as_mut().unwrap();
    let mut independent = values.records[0];
    independent.id = values.next_id;
    values.next_id += 1;
    values.records.push(independent);
    assert!(direct_offer(&model, &mut session).is_some());
    let decision = session.value_decision().unwrap();
    assert_eq!(decision.action, ValueAction::Add);
    assert_eq!(decision.value, 34);
    assert!(decision.operands.iter().any(|r| r.id == independent.id));
}

#[test]
fn native_operation_transition_exhausted_write_identity_never_offers_an_operator() {
    let model = mechanical_model(ValueAction::Copy);
    let mut session = first_completed(&model);
    session.values.as_mut().unwrap().next_id = u64::MAX;
    assert!(direct_offer(&model, &mut session).is_none());
    assert!(session.value_decision().is_none());
}

#[test]
fn native_operation_transition_prediction_is_idempotent_and_mismatch_does_not_write() {
    let model = mechanical_model(ValueAction::Copy);
    let mut session = first_completed(&model);
    let before = semantic_checkpoint(&session);
    let first = session.predict(&model).unwrap();
    let decision = session.value_decision().unwrap();
    assert_eq!(decision.cursor, 0);
    assert_eq!(session.predict(&model).unwrap(), first);
    assert_eq!(session.value_decision(), Some(decision));
    assert_eq!(semantic_checkpoint(&session), before);
    session.observe(&model, u32::from(b'9') + 2).unwrap();
    assert_eq!(session.work.values.derived_writes, 1);
    assert_eq!(session.values.as_ref().unwrap().operations_committed, 1);
    assert_eq!(session.values.as_ref().unwrap().records.len(), 2);
}

#[test]
fn native_operation_transition_checkpoint_and_cap_survive_repeated_operations() {
    let model = mechanical_model(ValueAction::Copy);
    let mut session = first_completed(&model);
    for expected in 2..=3 {
        for byte in b"17" {
            let checkpoint = session.checkpoint().unwrap();
            let mut restored = model.restore_session(&checkpoint).unwrap();
            let prediction = session.predict(&model).unwrap();
            assert_eq!(prediction.token, u32::from(*byte) + 2);
            assert_eq!(restored.predict(&model).unwrap(), prediction);
            restored.observe(&model, prediction.token).unwrap();
            session.observe(&model, prediction.token).unwrap();
            assert_eq!(
                semantic_checkpoint(&session),
                semantic_checkpoint(&restored)
            );
        }
        assert_eq!(
            session.values.as_ref().unwrap().operations_committed,
            expected
        );
    }
    assert_eq!(session.predict(&model).unwrap().token, EOS);
    assert!(session.value_decision().is_none());
    let checkpoint = session.checkpoint().unwrap();
    for field in ["max_operations", "operations_committed"] {
        let mut forged: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
        forged["values"][field] = serde_json::json!(9);
        assert!(model
            .restore_session(&serde_json::to_vec(&forged).unwrap())
            .is_err());
    }
}

#[test]
fn native_operation_transition_artifact_rejects_frozen_parent_and_shape_mutations() {
    let model = mechanical_model(ValueAction::Copy);
    let bytes = serde_json::to_vec(&model).unwrap();
    let restored: Model = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    let mut changed_parent = model.clone();
    changed_parent.values.as_mut().unwrap().continuation_score += 1;
    changed_parent.refresh_identity().unwrap();
    assert!(changed_parent.validate().is_err());
    let mut changed_cap = model.clone();
    changed_cap
        .operation_transition
        .as_mut()
        .unwrap()
        .max_operations = 4;
    changed_cap.refresh_identity().unwrap();
    assert!(changed_cap.validate().is_err());
    let mut changed_root = model;
    changed_root
        .operation_transition
        .as_mut()
        .unwrap()
        .router
        .codes[0]
        .roots[0] = 120;
    changed_root.refresh_identity().unwrap();
    assert!(changed_root.validate().is_err());
}

#[test]
fn native_operation_transition_completed_checkpoint_requires_boundary_but_prefill_does_not() {
    let model = mechanical_model(ValueAction::Copy);
    let mut session = first_completed(&model);
    session.observe(&model, EOS).unwrap();
    assert!(!session.is_response_active());
    assert!(session.needs_input_boundary());
    let mut restored = model
        .restore_session(&session.checkpoint().unwrap())
        .unwrap();
    assert!(restored.needs_input_boundary());
    restored.end_response(&model).unwrap();
    assert!(!restored.needs_input_boundary());
    for token in model.encode("source 1").unwrap() {
        restored.observe(&model, token).unwrap();
    }
    assert!(!restored.needs_input_boundary());
    let restored = model
        .restore_session(&restored.checkpoint().unwrap())
        .unwrap();
    assert!(!restored.needs_input_boundary());
}
