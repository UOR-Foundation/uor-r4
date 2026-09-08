//! Mechanical identity/control checks; these fixtures make no learned capability claim.
use super::super as native;
use super::*;
use native::completion_types::{CompletionAction, CompletionDecision};
use native::operation_transition_tests::{first_completed, mechanical_model};
use native::value_types::{ValueDerivation, ValueFeature, ValueRecord, ValueWork};

fn dictionary_document() -> MixedOperatorExample {
    MixedOperatorExample {
        first: MixedOperatorTarget {
            action: ValueAction::Copy,
            operand_ids: [0, 0],
        },
        context: OperationTransitionExample {
            id: "mechanical-dictionary-extension".into(),
            history: vec![],
            query: "again latest".into(),
            first_response: String::new(),
            next: None,
        },
        targets: vec![],
    }
}
fn word_query(session: &mut Session) {
    let values = session.values.as_mut().unwrap();
    values.query_boundary = None;
    let words = values.lexemes.as_mut().unwrap();
    words.queries.fill(Default::default());
    words.queries[0].bytes[..4].copy_from_slice(b"copy");
    words.queries[0].len = 4;
    words.query_len = 1;
}
fn lexical_model() -> Model {
    let mut model = mechanical_model(ValueAction::Copy);
    let identity = model.geometry.identity;
    let nonidentity = (identity + 1) % 120;
    let block = model.operation_transition.as_mut().unwrap();
    block.router.codes.extend([
        SourceCode {
            feature: ValueFeature {
                kind: 4,
                a: 2,
                b: 0,
            },
            roots: [nonidentity, identity],
        },
        SourceCode {
            feature: ValueFeature {
                kind: 5,
                a: 0,
                b: 2,
            },
            roots: [identity, nonidentity],
        },
    ]);
    block.router.codes.sort_by_key(|c| c.feature);
    model.refresh_identity().unwrap();
    model.validate().unwrap();
    model
}
#[test]
fn native_mixed_prime_migration_preserves_encoded_old_word_and_scores() {
    let model = lexical_model();
    let old = model.operation_transition.as_ref().unwrap();
    let mut migrated = old.clone();
    extend_dictionary(&mut migrated, &[dictionary_document()]).unwrap();
    let copy = migrated
        .dictionary
        .iter()
        .find(|w| &w.bytes[..usize::from(w.len)] == b"copy")
        .unwrap();
    assert_ne!(copy.prime, 2);
    let mut session = first_completed(&model);
    word_query(&mut session);
    let values = session.values.as_ref().unwrap();
    for operands in [None, Some([values.records[1]; 2])] {
        let (before, n) = features(old, values, operands, &mut Default::default());
        let (after, m) = features(&migrated, values, operands, &mut Default::default());
        assert_eq!(n, m);
        let old_pose =
            old.router
                .encode(&model, &before[..n], Control::Full, &mut Default::default());
        let new_pose =
            migrated
                .router
                .encode(&model, &after[..m], Control::Full, &mut Default::default());
        assert_ne!(old_pose, [model.geometry.identity; 2]);
        assert_eq!(old_pose, new_pose);
        for action in 0..3 {
            assert_eq!(
                old.router
                    .score(&model, old_pose, action, &mut Default::default()),
                migrated
                    .router
                    .score(&model, new_pose, action, &mut Default::default())
            );
        }
    }
    // Every original code keeps its geometry; only dictionary prime addresses move.
    assert_eq!(old.router.codes.len(), migrated.router.codes.len());
    assert_eq!(old.router.landmarks, migrated.router.landmarks);
    assert_eq!(old.router.biases, migrated.router.biases);
}
fn offered(
    model: &Model,
    session: &Session,
    control: Control,
) -> (Option<Candidate>, Option<ValueDecision>, u64) {
    let mut values = session.values.clone().unwrap();
    values.pending = None;
    let mut completion = session.completion.clone().unwrap();
    completion.pending = Some(CompletionDecision {
        token: EOS,
        score: 1000,
        write_id: values.records.last().unwrap().id,
        step: completion.steps,
        at_seen: values.seen,
        action: CompletionAction::Stop,
    });
    let mut work = ValueWork::default();
    let result = native::operation_transition::offer(
        model,
        &mut values,
        &completion,
        Candidate {
            token: EOS,
            score: 1000,
        },
        control,
        &mut work,
    );
    (result, values.pending, work.routing.code_reads)
}
#[test]
fn native_mixed_disabled_controls_restore_dictionary_with_old_router() {
    let old_model = lexical_model();
    let mut session = first_completed(&old_model);
    word_query(&mut session);
    let previous = old_model.operation_transition.clone().unwrap();
    let mut model = old_model.clone();
    extend_dictionary(
        model.operation_transition.as_mut().unwrap(),
        &[dictionary_document()],
    )
    .unwrap();
    model.mixed_operators = Some(MixedOperators {
        parent_artifact: old_model.artifact_cid().into(),
        previous_operation: previous.clone(),
        previous_roles: previous.router.clone(),
    });
    // The older scoped control also has old prime-coded parameters. This fixture
    // isolates dispatch, not witness serialization, and does not publish a model.
    model.composed_output = Some(native::composed_output::ComposedOutput {
        parent_artifact: previous.router.parent_artifact.clone(),
        previous_operation: previous.router.clone(),
    });
    let expected = offered(&old_model, &session, Control::Full);
    assert!(expected.0.is_some());
    assert!(expected.2 > 0);
    assert_eq!(
        offered(&model, &session, Control::MixedOperatorsDisabled),
        expected
    );
    assert_eq!(
        offered(&model, &session, Control::ComposedOutputDisabled),
        expected
    );
    // Detect a wrong dictionary even if equal action landmarks happen to hide
    // its prediction change: old word code reads must remain exactly equal.
    let mut wrong = model.clone();
    wrong.mixed_operators = None;
    assert_ne!(
        offered(&wrong, &session, Control::ComposedOutputDisabled).2,
        expected.2
    );
}
#[test]
fn native_mixed_copy_targets_distinguish_equal_valued_earlier_and_latest_ids() {
    let model = mechanical_model(ValueAction::Copy);
    let mut session = first_completed(&model);
    let values = session.values.as_mut().unwrap();
    let prior = values.records[1];
    let mut latest = prior;
    latest.id = values.next_id;
    latest.derivation = Some(ValueDerivation {
        action: ValueAction::Copy,
        operand_ids: [prior.id; 2],
        operand_values: [prior.value; 2],
    });
    values.next_id += 1;
    values.records.push(latest);
    let block = model.operation_transition.as_ref().unwrap();
    for id in [prior.id, latest.id] {
        let target = MixedOperatorTarget {
            action: ValueAction::Copy,
            operand_ids: [id; 2],
        };
        let (frame, (action, pair)) = exact_frame(block, &session, &target).unwrap();
        assert_eq!(action, ValueAction::Copy);
        assert_eq!(pair.map(|r| r.id), [id; 2]);
        assert_eq!(pair.map(|r| r.value), [17; 2]);
        assert_eq!(frame.alternatives.iter().filter(|a| a.correct).count(), 1);
    }
    for target in [
        MixedOperatorTarget {
            action: ValueAction::Copy,
            operand_ids: [prior.id, latest.id],
        },
        MixedOperatorTarget {
            action: ValueAction::Copy,
            operand_ids: [999; 2],
        },
        MixedOperatorTarget {
            action: ValueAction::Add,
            operand_ids: [prior.id, latest.id],
        },
        MixedOperatorTarget {
            action: ValueAction::Sub,
            operand_ids: [prior.id, latest.id],
        },
    ] {
        assert!(exact_frame(block, &session, &target).is_err());
    }
}
#[test]
fn native_mixed_training_frames_exclude_overflowing_add_candidates() {
    let model = mechanical_model(ValueAction::Copy);
    let mut session = first_completed(&model);
    session.values.as_mut().unwrap().records = vec![
        ValueRecord {
            id: 10,
            value: i64::MAX,
            ..Default::default()
        },
        ValueRecord {
            id: 11,
            value: 1,
            ..Default::default()
        },
    ];
    let block = model.operation_transition.as_ref().unwrap();
    let target = MixedOperatorTarget {
        action: ValueAction::Copy,
        operand_ids: [10; 2],
    };
    let (exact, _) = exact_frame(block, &session, &target).unwrap();
    let inherited = frame(block, &session, None).unwrap().0;
    for frame in [exact, inherited] {
        assert_eq!(frame.alternatives.len(), 3); // Stop and the two legal Copies.
        assert!(frame.alternatives.iter().all(|a| a.action != 2));
    }
    assert!(exact_frame(
        block,
        &session,
        &MixedOperatorTarget {
            action: ValueAction::Add,
            operand_ids: [10, 11]
        }
    )
    .is_err());
}

#[test]
fn native_mixed_rejection_keeps_features_and_all_wrong_alternatives() {
    let make = |action, correct| Alternative {
        features: vec![ValueFeature {
            kind: 0,
            a: action,
            b: 3,
        }],
        codes: vec![],
        action: action as usize,
        correct,
    };
    let original = Frame {
        alternatives: vec![
            make(0, false),
            make(1, true),
            make(1, false),
            make(2, true),
            make(2, false),
        ],
    };
    let rejected = rejection_frame(&original).unwrap();
    assert_eq!(rejected.alternatives.len(), 3);
    for (actual, prior) in rejected.alternatives.iter().zip([0, 2, 4]) {
        assert_eq!(actual.features, original.alternatives[prior].features);
        assert_eq!(actual.action, original.alternatives[prior].action);
        assert_eq!(actual.correct, actual.action == 0);
    }
    assert!(rejection_frame(&rejected).is_none());
    assert_eq!(
        original.alternatives.iter().filter(|a| a.correct).count(),
        2
    );
}
