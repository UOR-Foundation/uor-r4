//! Mechanical feature/control/trainer contracts, not learned language evidence.
use super::action_emission::ActionEmission;
use super::completion_types::LexicalRead;
use super::lexical_emission::{features, offer};
use super::lexical_emission_tests::{copied, lexical_model};
use super::source_routing::SourceCode;
use super::source_routing_training::{learn_code_subset, Alternative, Frame};
use super::value_types::ValueFeature;
use super::*;
use std::time::Instant;

#[test]
fn native_action_emission_tags_preserve_legacy_add_and_exact_read_roles() {
    let model = lexical_model();
    let session = copied(&model, 17);
    let block = model.lexical_emission.as_ref().unwrap();
    let values = session.values.as_ref().unwrap();
    let source = values.records[0];
    let result = values.records[1];
    assert_eq!(source.value, result.value);
    assert_ne!(source.id, result.id);
    let mut states = Vec::new();
    for steps in [0, 1, 4] {
        let mut c = *session.completion.as_ref().unwrap();
        c.steps = steps;
        c.last = 65536 + 258 - 1;
        c.previous = u32::from(b'+') + 2;
        states.push(c);
    }
    for record in [source, result] {
        for after in [0, 1, 2] {
            let mut c = *session.completion.as_ref().unwrap();
            let numeral = numeral::Numeral::from_zphi(crate::prime_route_attention::ZPhi::new(
                record.value,
                0,
            ))
            .unwrap();
            c.steps = 4;
            c.lexical_read = Some(LexicalRead {
                record_id: record.id,
                start_at: 100,
                numeral,
                cursor: numeral.len,
            });
            c.seen = 100 + u64::from(numeral.len) + after;
            states.push(c);
        }
    }
    for mut c in states {
        for record in [None, Some(source), Some(result)] {
            for token in [BOS, EOS, u32::from(b'=') + 2] {
                c.anchor.as_mut().unwrap().action = ValueAction::Add;
                let add = features(block, &c, values, token, record, true);
                let legacy = features(block, &c, values, token, record, false);
                assert_eq!(add, legacy);
                c.anchor.as_mut().unwrap().action = ValueAction::Copy;
                let copy_legacy = features(block, &c, values, token, record, false);
                assert_eq!(copy_legacy, legacy);
                let (mut tagged, n) = features(block, &c, values, token, record, true);
                assert_eq!(n, legacy.1);
                assert_eq!(tagged[0].a ^ legacy.0[0].a, 1_u64 << 56);
                tagged[0].a &= (1_u64 << 56) - 1;
                assert_eq!(tagged, legacy.0);
            }
        }
        let (source_features, _) = features(block, &c, values, BOS, Some(source), true);
        let (result_features, _) = features(block, &c, values, BOS, Some(result), true);
        assert_ne!(source_features[0].b, result_features[0].b);
    }
}

#[test]
fn native_action_emission_erasure_changes_only_action_context_and_preserves_cursor() {
    let mut model = lexical_model();
    let session = copied(&model, 17);
    let original = model.lexical_emission.as_ref().unwrap().clone();
    let state = *session.completion.as_ref().unwrap();
    let values = session.values.as_ref().unwrap();
    let identity = model.geometry.identity;
    let ranks = &original.router.ranks;
    let root = (0..120_u16)
        .find(|r| {
            ranks[usize::from(model.geometry.inverses[usize::from(*r)])]
                < ranks[usize::from(identity)]
        })
        .unwrap();
    let feature = features(&original, &state, values, EOS, None, true).0[0];
    let active = model.lexical_emission.as_mut().unwrap();
    active.router.codes = vec![SourceCode {
        feature,
        roots: [root; 2],
    }];
    active.router.landmarks = vec![[identity; 2], [root; 2]];
    active.router.biases = vec![0, 1];
    // Direct offer fixture isolates the feature law. It is not serialized as a
    // qualified learned artifact and does not claim witness-chain acceptance.
    model.action_emission = Some(ActionEmission {
        parent_artifact: model.artifact_cid.clone(),
        previous_lexical: original,
        previous_roles: model.operation_transition.as_ref().unwrap().router.clone(),
        previous_operation: model.operation_transition.as_ref().unwrap().clone(),
        context_enabled: true,
    });
    let baseline = Candidate {
        token: BOS,
        score: 5,
    };
    let mut full = state;
    let selected = offer(
        &model,
        &mut full,
        values,
        baseline,
        Control::Full,
        &mut Default::default(),
    )
    .unwrap();
    assert_eq!(selected.token, EOS);
    let mut erased = state;
    assert!(offer(
        &model,
        &mut erased,
        values,
        baseline,
        Control::LexicalActionContextDisabled,
        &mut Default::default()
    )
    .is_none());
    let mut restored = state;
    assert!(offer(
        &model,
        &mut restored,
        values,
        baseline,
        Control::ActionEmissionDisabled,
        &mut Default::default()
    )
    .is_none());
    model.action_emission.as_mut().unwrap().context_enabled = false;
    let mut staged = state;
    assert!(offer(
        &model,
        &mut staged,
        values,
        baseline,
        Control::Full,
        &mut Default::default()
    )
    .is_none());
    let mut witness = model.action_emission.take().unwrap();
    witness.context_enabled = true;
    let mut old_artifact = state;
    assert!(offer(
        &model,
        &mut old_artifact,
        values,
        baseline,
        Control::Full,
        &mut Default::default()
    )
    .is_none());
    for s in [&full, &erased, &restored, &staged, &old_artifact] {
        assert_eq!(s.anchor, state.anchor);
        assert_eq!(s.lexical_read, state.lexical_read);
        assert_eq!(s.seen, state.seen);
        assert_eq!(s.steps, state.steps);
    }
    model.action_emission = Some(witness);
    // Once selected, an exact read resumes identically through action erasure.
    let source = values.records[0];
    let numeral =
        numeral::Numeral::from_zphi(crate::prime_route_attention::ZPhi::new(source.value, 0))
            .unwrap();
    let read = LexicalRead {
        record_id: source.id,
        start_at: state.seen,
        numeral,
        cursor: 1,
    };
    for control in [
        Control::Full,
        Control::LexicalActionContextDisabled,
        Control::ActionEmissionDisabled,
    ] {
        let mut c = state;
        c.lexical_read = Some(read);
        let selected = offer(
            &model,
            &mut c,
            values,
            baseline,
            control,
            &mut Default::default(),
        )
        .unwrap();
        assert_eq!(selected.token, numeral.tokens[1]);
        assert_eq!(c.lexical_read, Some(read));
        assert_eq!(c.pending_lexical_read, Some(read));
        assert_eq!(c.anchor, state.anchor);
    }
}

#[test]
fn native_action_emission_subset_fit_rejects_mask_and_freezes_inherited_parameters() {
    let model = lexical_model();
    let mut block = model.lexical_emission.as_ref().unwrap().router.clone();
    let identity = model.geometry.identity;
    let root = (0..120_u16)
        .find(|r| block.ranks[usize::from(*r)] < block.ranks[usize::from(identity)])
        .unwrap();
    let inherited = ValueFeature {
        kind: 0,
        a: 0,
        b: EOS.into(),
    };
    let added = ValueFeature {
        kind: 0,
        a: 1_u64 << 56,
        b: EOS.into(),
    };
    block.codes = vec![
        SourceCode {
            feature: inherited,
            roots: [identity; 2],
        },
        SourceCode {
            feature: added,
            roots: [root; 2],
        },
        SourceCode {
            feature: ValueFeature {
                kind: 3,
                a: 2,
                b: 0,
            },
            roots: [root; 2],
        },
    ];
    block.landmarks = vec![[identity; 2]; 2];
    block.biases = vec![0, 1];
    block.config.passes = 1;
    block.config.proposals = 1;
    let before = block.clone();
    let mut frames = vec![Frame {
        alternatives: vec![
            Alternative {
                features: vec![],
                codes: vec![],
                action: 0,
                correct: false,
            },
            Alternative {
                features: vec![added],
                codes: vec![],
                action: 1,
                correct: true,
            },
        ],
    }];
    assert!(learn_code_subset(&model, &mut block, &mut frames, Instant::now(), &[true]).is_err());
    assert_eq!(block, before);
    let no_updates = learn_code_subset(
        &model,
        &mut block,
        &mut frames,
        Instant::now(),
        &[false, false, false],
    )
    .unwrap();
    assert_eq!(no_updates["proposals"], 0);
    assert_eq!(block, before);
    let report = learn_code_subset(
        &model,
        &mut block,
        &mut frames,
        Instant::now(),
        &[false, true, false],
    )
    .unwrap();
    assert_eq!(report["final_correct"], 1);
    assert!(report["accepted"][0].as_u64().unwrap() > 0);
    assert_eq!(report["accepted"][1], 0);
    assert_eq!(report["accepted"][2], 0);
    assert_eq!(block.codes[0], before.codes[0]);
    assert_ne!(block.codes[1].roots, before.codes[1].roots);
    let mut restored = block.clone();
    restored.codes[1].roots = before.codes[1].roots;
    assert_eq!(restored, before);
}

#[test]
fn native_action_emission_conflict_check_keeps_jointly_acceptable_alternatives() {
    let make = |positive: [bool; 2]| Frame {
        alternatives: positive
            .into_iter()
            .enumerate()
            .map(|(i, correct)| Alternative {
                features: vec![ValueFeature {
                    kind: 0,
                    a: i as u64,
                    b: 0,
                }],
                codes: vec![],
                action: i,
                correct,
            })
            .collect(),
    };
    let compatible = super::action_emission::operation_frame_conflicts(&[
        make([true, true]),
        make([false, true]),
    ])
    .unwrap();
    assert_eq!(compatible["count"], 0);
    let contradictory = super::action_emission::operation_frame_conflicts(&[
        make([true, false]),
        make([false, true]),
    ])
    .unwrap();
    assert_eq!(contradictory["count"], 1);
}
