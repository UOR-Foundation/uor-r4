//! Focused typed-state checks. Deliberately set rows exercise causal mechanics;
//! the separately named fitting check uses only raw prompt/response examples.
use super::numeral::NUMERAL_CODEC;
use super::value_types::*;
use super::*;

fn baseline() -> Model {
    let catalog = [Document {
        id: "typed-value-test-catalog".into(),
        text: "left right total copy none 13 4 17 18 answer unknown result".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 32,
            candidate_limit: 16,
            max_lexical_pieces: 128,
            ..Config::default()
        },
        &catalog,
    )
    .unwrap();
    trainer.train_documents(&catalog).unwrap();
    trainer.compile().unwrap()
}

fn mechanical_model(action: ValueAction) -> Model {
    let mut model = baseline();
    model.values = Some(ValueModel {
        schema: VALUE_SCHEMA.into(),
        codec: NUMERAL_CODEC.into(),
        capacity: VALUES,
        rows: [ValueAction::Copy, ValueAction::Add]
            .into_iter()
            .enumerate()
            .map(|(index, candidate)| ValueRow {
                feature: ValueFeature {
                    kind: 0,
                    a: index as u64,
                    b: 0,
                },
                weight: if candidate == action { 4096 } else { -4096 },
            })
            .collect(),
        continuation_score: 4096,
        fit_config: [0; 4],
        training: Vec::new(),
    });
    model.refresh_identity().unwrap();
    model
}

fn prefix(model: &Model, prompt: &str, control: Control) -> Session {
    let mut session = model.session(control).unwrap();
    session.observe(model, BOS).unwrap();
    for token in model.encode(prompt).unwrap() {
        session.observe(model, token).unwrap();
    }
    session.begin_response(model).unwrap();
    session
}

#[test]
fn native_value_prediction_is_causal_and_mismatch_never_selects_a_target_source() {
    let model = mechanical_model(ValueAction::Add);
    let mut session = prefix(&model, "left = 13; right = 4; total:", Control::Full);
    let before = session.state();
    let first = session.predict(&model).unwrap();
    let decision = session.value_decision().unwrap();
    assert_eq!(decision.action, ValueAction::Add);
    assert_eq!(decision.value, 17);
    assert_ne!(decision.operands[0].id, decision.operands[1].id);
    assert_eq!(session.state(), before);
    assert_eq!(session.predict(&model).unwrap(), first);
    assert_eq!(session.value_decision(), Some(decision));
    assert_eq!(session.work.values.derived_writes, 0);
    assert_eq!(session.work.values.emission_commits, 0);

    let wrong = u32::from(b'9') + 2;
    session.observe(&model, wrong).unwrap();
    assert_eq!(session.work.values.derived_writes, 0);
    assert_eq!(session.work.values.emission_mismatches, 1);
    assert_eq!(session.value_decision(), None);
    assert!(session.values.as_ref().unwrap().emission.is_none());
    // Wrong observed bytes did not create a numeral or identify another pair.
    session.predict(&model).unwrap();
    let retry = session.value_decision().unwrap();
    assert_eq!(retry.value, decision.value);
    assert_eq!(retry.operands, decision.operands);
    assert_eq!(retry.write_id, decision.write_id);
    session.observe(&model, retry.token).unwrap();
    assert_eq!(session.work.values.derived_writes, 1);
    assert_eq!(session.work.values.emission_commits, 1);
    assert_eq!(session.state().values.unwrap().emission_cursor, Some(1));

    session.predict(&model).unwrap();
    assert_eq!(session.value_decision().unwrap().token, u32::from(b'7') + 2);
    session.observe(&model, wrong).unwrap();
    assert!(session.values.as_ref().unwrap().emission.is_none());
    assert_eq!(session.work.values.emission_commits, 1);
    assert_eq!(session.work.values.emission_mismatches, 2);
    session.predict(&model).unwrap();
    assert!(session.value_decision().is_none());
    let records = &session.values.as_ref().unwrap().records;
    assert_eq!(records.iter().filter(|record| record.derived).count(), 1);
    assert_eq!(records.last().unwrap().value, 17);
}

#[test]
fn native_value_equal_first_bytes_preserve_complete_derivation_identity() {
    let model = mechanical_model(ValueAction::Add);
    let mut first = prefix(&model, "left = 13; right = 4; total:", Control::Full);
    let mut second = prefix(&model, "left = 13; right = 5; total:", Control::Full);
    assert_eq!(
        first.predict(&model).unwrap().token,
        second.predict(&model).unwrap().token
    );
    let a = first.value_decision().unwrap();
    let b = second.value_decision().unwrap();
    assert_eq!(a.token, u32::from(b'1') + 2);
    assert_eq!((a.value, b.value), (17, 18));
    assert_ne!(a.operands, b.operands);
    first.observe(&model, a.token).unwrap();
    second.observe(&model, b.token).unwrap();
    first.predict(&model).unwrap();
    second.predict(&model).unwrap();
    assert_eq!(first.value_decision().unwrap().token, u32::from(b'7') + 2);
    assert_eq!(second.value_decision().unwrap().token, u32::from(b'8') + 2);
    assert_eq!(
        first.values.as_ref().unwrap().records.last().unwrap().value,
        17
    );
    assert_eq!(
        second
            .values
            .as_ref()
            .unwrap()
            .records
            .last()
            .unwrap()
            .value,
        18
    );
}

#[test]
fn native_value_signed_extrema_use_ordinary_tokens_and_complete_once() {
    let model = mechanical_model(ValueAction::Copy);
    for value in [i64::MIN, i64::MAX, -17, 0, 17] {
        let expected = value.to_string();
        let mut session = prefix(&model, &format!("copy {value}"), Control::Full);
        let mut tokens = Vec::new();
        for _ in expected.bytes() {
            let token = session.predict(&model).unwrap().token;
            let decision = session.value_decision().unwrap();
            assert_eq!(decision.value, value);
            assert_eq!(decision.action, ValueAction::Copy);
            assert_eq!(session.candidates().first().unwrap().token, token);
            tokens.push(token);
            session.observe(&model, token).unwrap();
        }
        assert_eq!(model.decode(&tokens).unwrap(), expected.as_bytes());
        assert_eq!(session.work.values.derived_writes, 1);
        assert_eq!(session.work.values.emission_commits, expected.len() as u64);
        assert!(session.values.as_ref().unwrap().emission.is_none());
        session.predict(&model).unwrap();
        assert!(session.value_decision().is_none());
    }
    // The lexical codec and byte emission may have different token counts,
    // while their decoded bytes remain the same actual output interface.
    let lexical = model.encode(" 17").unwrap();
    assert_eq!(lexical.len(), 1);
    assert_eq!(
        model.decode(&lexical).unwrap(),
        model.decode(&[34, 51, 57]).unwrap()
    );
}

#[test]
fn native_value_overflow_and_disabled_control_offer_no_derived_candidate() {
    let model = mechanical_model(ValueAction::Add);
    let mut overflow = prefix(
        &model,
        "left = 9223372036854775807; right = 1; total:",
        Control::Full,
    );
    overflow.predict(&model).unwrap();
    assert_eq!(overflow.value_decision(), None);
    assert_eq!(overflow.work.values.overflow_rejections, 2);
    assert_eq!(overflow.work.values.derived_writes, 0);
    for control in [Control::ValuesDisabled, Control::MemoryDisabled] {
        let mut disabled = prefix(&model, "left = 13; right = 4; total:", control);
        disabled.predict(&model).unwrap();
        assert_eq!(disabled.value_decision(), None);
        assert_eq!(disabled.work.values.proposals, 0);
        assert_eq!(disabled.work.values.derived_writes, 0);
    }
}

#[test]
fn native_value_absence_preserves_legacy_serialized_fields_and_roundtrip() {
    let model = baseline();
    let bytes = model.to_bytes().unwrap();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire.get("values").is_none());
    assert_eq!(
        Model::from_bytes(&bytes).unwrap().to_bytes().unwrap(),
        bytes
    );
    let state = prefix(&model, "left = 13; right = 4; total:", Control::Full);
    assert!(state.values.is_none());
    assert!(serde_json::to_value(state.state())
        .unwrap()
        .get("values")
        .is_none());
    assert!(serde_json::to_value(state.work)
        .unwrap()
        .get("values")
        .is_none());
    let saved = state.checkpoint().unwrap();
    let checkpoint: serde_json::Value = serde_json::from_slice(&saved).unwrap();
    assert_eq!(checkpoint["schema"], "uor-r4.native-geometric-session/1");
    assert_eq!(
        model.restore_session(&saved).unwrap().checkpoint().unwrap(),
        saved
    );
}

#[test]
fn native_value_fit_uses_raw_responses_and_emits_an_absent_sum() {
    let baseline = baseline();
    let source = (0..12)
        .flat_map(|index| {
            let a = 11 + index;
            let b = 3 + index;
            [
                ValueExample {
                    id: format!("typed-sum-{index}"),
                    prompt: format!("left = {a}; right = {b}; total:"),
                    response: (a + b).to_string(),
                },
                ValueExample {
                    id: format!("typed-no-write-{index}"),
                    prompt: format!("left = {a}; right = {b}; none:"),
                    response: "unknown".into(),
                },
            ]
        })
        .collect::<Vec<_>>();
    let (learned, report) = baseline
        .fit_values(
            &source,
            ValueFitConfig {
                epochs: 96,
                learning_rate: 0.25,
                max_features: 4096,
            },
        )
        .unwrap();
    assert_eq!(report.numeric_targets, 12);
    assert_eq!(report.reachable_numeric_targets, 12);
    assert_eq!(report.no_write_targets, 12);
    assert!(report.learned_features > 0);
    assert!(report.continuation_positions > 0);
    assert!(baseline.values.is_none());
    let learned = Model::from_bytes(&learned.to_bytes().unwrap()).unwrap();
    let mut correct_sums = 0;
    let mut correct_no_writes = 0;
    for example in &source {
        let mut session = prefix(&learned, &example.prompt, Control::Full);
        session.predict(&learned).unwrap();
        if example.response == "unknown" {
            correct_no_writes += usize::from(session.value_decision().is_none());
            continue;
        }
        let mut output = Vec::new();
        let mut decisions = Vec::new();
        for _ in example.response.bytes() {
            let prediction = session.predict(&learned).unwrap();
            let Some(decision) = session.value_decision() else {
                break;
            };
            decisions.push(decision);
            output.push(prediction.token);
            session.observe(&learned, prediction.token).unwrap();
        }
        if learned.decode(&output).unwrap() == example.response.as_bytes() {
            correct_sums += 1;
            assert!(decisions
                .iter()
                .all(|decision| decision.action == ValueAction::Add));
            assert_eq!(session.work.values.derived_writes, 1);
        }
    }
    // This is a finite construction-fitting integration smoke, not held-out
    // capability evidence or a claim of a geometric contribution.
    assert_eq!(correct_sums, 12);
    assert_eq!(correct_no_writes, 12);
}

#[test]
fn native_value_fit_preserves_canonical_spelling_and_terminal_punctuation() {
    let baseline = baseline();
    let examples = [
        ("canonical", "17"),
        ("plus", "+17"),
        ("zero", "017"),
        ("period", "17."),
    ]
    .into_iter()
    .map(|(id, response)| ValueExample {
        id: format!("typed-spelling-{id}"),
        prompt: format!("{id}: left = 13; right = 4; total:"),
        response: response.into(),
    })
    .collect::<Vec<_>>();
    let (_, report) = baseline
        .fit_values(
            &examples,
            ValueFitConfig {
                epochs: 4,
                learning_rate: 0.25,
                max_features: 4096,
            },
        )
        .unwrap();
    assert_eq!(report.numeric_targets, 4);
    // 17 is the generated numeral prefix; the terminal period remains an
    // ordinary-token responsibility. +17 and 017 are different output bytes.
    assert_eq!(report.reachable_numeric_targets, 2);
    assert_eq!(report.no_write_targets, 0);
}

#[test]
fn native_value_document_boundaries_do_not_join_numerals() {
    let model = mechanical_model(ValueAction::Copy);
    let mut session = model.session(Control::Full).unwrap();
    session.observe(&model, BOS).unwrap();
    session.observe(&model, u32::from(b'1') + 2).unwrap();
    session.observe(&model, EOS).unwrap();
    session.observe(&model, u32::from(b'2') + 2).unwrap();
    session.begin_response(&model).unwrap();
    let values: Vec<_> = session
        .values
        .as_ref()
        .unwrap()
        .records
        .iter()
        .map(|r| r.value)
        .collect();
    assert_eq!(values, vec![1, 2]);
}

#[test]
fn native_value_selected_execution_runs_one_of_256_scored_choices() {
    let model = mechanical_model(ValueAction::Add);
    let mut session = prefix(
        &model,
        "1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 total:",
        Control::Full,
    );
    session.predict(&model).unwrap();
    let d = session.value_decision().unwrap();
    assert_eq!(d.value, 3);
    assert_eq!(d.operands.map(|v| v.value), [2, 1]);
    assert_eq!(session.work.values.proposals, 256);
    assert_eq!(session.work.values.operator_executions, 1);
    assert_eq!(session.work.values.additions, 1);
    assert_eq!(session.work.values.selection_comparisons, 256);
    assert_eq!(session.work.values.selection_passes, 1);
    assert_eq!(session.work.values.derived_writes, 0);
}

#[test]
fn native_typed_routing_preserves_role_information_without_encoding_values() {
    let model = mechanical_model(ValueAction::Add);
    let session = prefix(&model, "13 4 total:", Control::Full);
    let values = session.values.as_ref().unwrap();
    let mut a = values.sources[0];
    let b = values.sources[1];
    let mut addr = [0; 16];
    addr[0] = 2;
    addr[1] = 3;
    let first =
        super::typed_routing::features(values, Some((a, b)), &addr, &mut Default::default());
    a.value = 999;
    assert_eq!(
        first,
        super::typed_routing::features(values, Some((a, b)), &addr, &mut Default::default())
    );
    a.derived = true;
    assert_ne!(
        first,
        super::typed_routing::features(values, Some((a, b)), &addr, &mut Default::default())
    );
    a.derived = false;
    addr.swap(0, 1);
    assert_ne!(
        first,
        super::typed_routing::features(values, Some((a, b)), &addr, &mut Default::default())
    );
}

#[test]
fn native_typed_routing_case_fold_changes_metadata_only() {
    use super::source_routing::SourceRouting;
    use super::typed_routing::TypedRouting;
    use super::word_copy_types::WordCopyAddress;
    let model = mechanical_model(ValueAction::Add);
    let mut session = prefix(&model, "13 4 total:", Control::Full);
    let values = session.values.as_mut().unwrap();
    let mut bytes = [0; 32];
    bytes[..4].copy_from_slice(b"copy");
    let mut block = TypedRouting {
        fold_ascii_case: true,
        canonical_copy_aliases: false,
        local_query: false,
        dictionary: vec![WordCopyAddress {
            bytes,
            len: 4,
            prime: 2,
        }],
        router: SourceRouting {
            schema: String::new(),
            parent_artifact: String::new(),
            codes: Vec::new(),
            landmarks: Vec::new(),
            biases: Vec::new(),
            ranks: Vec::new(),
            training: Vec::new(),
            config: SourceRoutingConfig::default(),
        },
    };
    for spelling in [b"copy", b"Copy", b"COPY"] {
        let mut words = super::value_lexemes::LexemeState {
            query_len: 1,
            ..Default::default()
        };
        words.queries[0].len = 4;
        words.queries[0].bytes[..4].copy_from_slice(spelling);
        values.lexemes = Some(words);
        block.fold_ascii_case = true;
        assert_eq!(
            super::typed_routing::addresses(&block, values, &mut Default::default())[0],
            2
        );
        assert_eq!(values.lexemes, Some(words));
        block.local_query = true;
        values.query_boundary = Some(1);
        assert_eq!(
            super::typed_routing::addresses(&block, values, &mut Default::default())[0],
            0
        );
        values.lexemes.as_mut().unwrap().queries[0].end = 1;
        assert_eq!(
            super::typed_routing::addresses(&block, values, &mut Default::default())[0],
            2
        );
        values.lexemes = Some(words);
        values.query_boundary = None;
        block.local_query = false;
        block.fold_ascii_case = false;
        assert_eq!(
            super::typed_routing::addresses(&block, values, &mut Default::default())[0],
            if spelling == b"copy" { 2 } else { 0 }
        );
    }
}

#[test]
fn native_typed_roles_query_boundary_tracks_end_and_roundtrips() {
    let model = mechanical_model(ValueAction::Add);
    let mut session = prefix(&model, "13 4 total:", Control::Full);
    let v = session.values.as_mut().unwrap();
    v.query_boundary = Some(0);
    let end = v.seen;
    v.end();
    assert_eq!(v.query_boundary, Some(end));
    let saved = serde_json::to_vec(v).unwrap();
    let restored: ValueState = serde_json::from_slice(&saved).unwrap();
    assert_eq!(restored.query_boundary, Some(end));
    // Old state has no boundary field and remains boundary-free.
    let mut wire = serde_json::to_value(&restored).unwrap();
    wire.as_object_mut().unwrap().remove("query_boundary");
    let old: ValueState = serde_json::from_value(wire).unwrap();
    assert_eq!(old.query_boundary, None);
}

#[test]
fn native_value_selected_execution_preserves_overflow_fallback_order() {
    let model = mechanical_model(ValueAction::Add);
    let mut session = prefix(&model, "9223372036854775807 1 -2 total:", Control::Full);
    session.predict(&model).unwrap();
    let d = session.value_decision().unwrap();
    assert_eq!(d.value, i64::MAX - 2);
    assert_eq!(d.operands.map(|v| v.value), [-2, i64::MAX]);
    assert_eq!(session.work.values.operator_executions, 2);
    assert_eq!(session.work.values.overflow_rejections, 1);
    assert_eq!(session.work.values.selection_passes, 2);
    assert_eq!(session.work.values.proposals, 18);
}

#[test]
fn native_value_selected_execution_abstention_does_no_arithmetic() {
    let mut model = mechanical_model(ValueAction::Add);
    for row in &mut model.values.as_mut().unwrap().rows {
        row.weight = 0;
    }
    model.refresh_identity().unwrap();
    let mut session = prefix(&model, "13 4 total:", Control::Full);
    session.predict(&model).unwrap();
    assert!(session.value_decision().is_none());
    assert_eq!(session.work.values.proposals, 4);
    assert_eq!(session.work.values.operator_executions, 0);
    assert_eq!(session.work.values.additions, 0);
}

#[test]
fn native_value_selected_execution_reuses_only_committed_derived_values() {
    // Mechanical role weights isolate causal state reuse, not language learning.
    let mut model = mechanical_model(ValueAction::Add);
    model.values.as_mut().unwrap().rows.extend([
        ValueRow {
            feature: ValueFeature {
                kind: 1,
                a: 1,
                b: 256,
            },
            weight: 16384,
        },
        ValueRow {
            feature: ValueFeature {
                kind: 2,
                a: 1,
                b: 1,
            },
            weight: 8192,
        },
    ]);
    model
        .values
        .as_mut()
        .unwrap()
        .rows
        .sort_by_key(|r| r.feature);
    model.refresh_identity().unwrap();
    for committed in [false, true] {
        let mut session = prefix(&model, "left = 13; right = 4; total:", Control::Full);
        session.predict(&model).unwrap();
        assert_eq!(session.value_decision().unwrap().value, 17);
        if committed {
            for _ in 0..2 {
                let token = session.predict(&model).unwrap().token;
                session.observe(&model, token).unwrap();
            }
        }
        session.end_response(&model).unwrap();
        for token in model.encode("next = 5; total:").unwrap() {
            session.observe(&model, token).unwrap();
        }
        session.begin_response(&model).unwrap();
        let saved = session.checkpoint().unwrap();
        let mut restored = model.restore_session(&saved).unwrap();
        session.predict(&model).unwrap();
        restored.predict(&model).unwrap();
        assert_eq!(session.value_decision(), restored.value_decision());
        let decision = session.value_decision().unwrap();
        assert_eq!(decision.value, if committed { 22 } else { 9 });
        assert_eq!(decision.operands[0].derived, committed);
        assert_eq!(decision.operands[1].value, 5);
        let mut emitted = Vec::new();
        for _ in 0..if committed { 2 } else { 1 } {
            let token = session.predict(&model).unwrap().token;
            emitted.push(token);
            session.observe(&model, token).unwrap();
        }
        assert_eq!(
            model.decode(&emitted).unwrap(),
            if committed {
                b"22".as_slice()
            } else {
                b"9".as_slice()
            }
        );
        let last = session.values.as_ref().unwrap().records.last().unwrap();
        assert_eq!(last.value, decision.value);
        assert_eq!(
            last.derivation.unwrap().operand_ids,
            decision.operands.map(|v| v.id)
        );
    }
}

#[test]
#[ignore = "requires an admitted local model allowance and R4_TYPED_ADMISSION_MODEL artifact"]
fn native_value_selected_execution_learned_chain_diagnostic() {
    // OPEN evaluation only: this records the frozen learned model's behavior,
    // separately from the deliberately weighted causal tests above.
    fn generate(model: &Model, session: &mut Session) -> (Option<ValueDecision>, Vec<u8>, bool) {
        let mut decision = None;
        let mut tokens = Vec::new();
        for _ in 0..32 {
            let token = session.predict(model).unwrap().token;
            if let Some(d) = session.value_decision().filter(|d| d.cursor == 0) {
                decision = Some(d);
            }
            session.observe(model, token).unwrap();
            if token == EOS {
                return (decision, model.decode(&tokens).unwrap(), true);
            }
            tokens.push(token);
        }
        (decision, model.decode(&tokens).unwrap(), false)
    }

    let path = std::env::var("R4_TYPED_ADMISSION_MODEL").unwrap();
    let model = Model::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let mut exact_chains = 0;
    // The first two worlds change one input under identical query wording.
    // Each world's follow-ups share state and delta but require Copy versus Add.
    for (a, b, delta) in [(13, 4, 5), (14, 4, 5), (-3, 8, 4)] {
        let prompt = format!("User: suri has {a} coins. orin has {b} coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:");
        let mut session = prefix(&model, &prompt, Control::Full);
        let (first, first_bytes, first_eos) = generate(&model, &mut session);
        session.end_response(&model).unwrap();
        let first_expected = format!("{}.\n", a + b);
        let first_correct = first.is_some_and(|d| d.value == a + b && d.action == ValueAction::Add)
            && first_bytes == first_expected.as_bytes()
            && first_eos;
        let saved = session.checkpoint().unwrap();
        for action in [ValueAction::Copy, ValueAction::Add] {
            let question = match action {
                ValueAction::Copy => "Repeat the previous total without adding the new coins.",
                ValueAction::Add => "Add the new coins to the previous total.",
            };
            let query = format!("User: There are {delta} new coins. {question}\nAssistant:");
            let expected = a + b + if action == ValueAction::Add { delta } else { 0 };
            let expected_text = format!("{expected}.\n");
            let mut full_correct = false;
            let mut control_changes_output = false;
            let mut full_bytes = Vec::new();
            for remove_intermediate in [false, true] {
                let mut branch = model.restore_session(&saved).unwrap();
                // Evaluator-only deletion of the one actual derived record.
                // Keep transcript/geometric state and all other records intact.
                // No supplied value, answer injection or serving rule is added.
                let removed = if remove_intermediate {
                    first.is_some_and(|d| {
                        let values = branch.values.as_mut().unwrap();
                        let before = values.records.len();
                        values.records.retain(|r| r.id != d.write_id);
                        before != values.records.len()
                    })
                } else {
                    false
                };
                for token in model.encode(&query).unwrap() {
                    branch.observe(&model, token).unwrap();
                }
                branch.begin_response(&model).unwrap();
                let source_records = branch.values.as_ref().unwrap().sources.clone();
                let (second, bytes, eos) = generate(&model, &mut branch);
                let uses_intermediate = first.zip(second).is_some_and(|(x, y)| {
                    y.operands.iter().any(|v| v.derived && v.id == x.write_id)
                });
                let second_correct = second
                    .is_some_and(|d| d.value == expected && d.action == action)
                    && bytes == expected_text.as_bytes()
                    && eos;
                if remove_intermediate {
                    control_changes_output = removed && bytes != full_bytes;
                } else {
                    full_correct = first_correct && second_correct && uses_intermediate;
                    full_bytes = bytes.clone();
                }
                println!(
                    "{}",
                    serde_json::json!({"artifact_cid":model.artifact_cid,
                    "operands":[a,b,delta],"prompt":prompt,"query":query,"required_action":action,
                    "first":first,"second":second,"first_expected":first_expected,
                    "first_text":String::from_utf8_lossy(&first_bytes),
                    "expected":expected_text,"second_text":String::from_utf8_lossy(&bytes),
                    "first_correct":first_correct,"second_correct":second_correct,"terminated":eos,
                    "source_records":source_records,"uses_intermediate":uses_intermediate,
                    "remove_intermediate_control":remove_intermediate,"record_removed":removed,
                    "work":branch.work,"shared_prefix_work":session.work,
                    "scope":"OPEN frozen learned artifact, untrained follow-up wording; no refit or supplied derived value. Branch work includes shared prefix work; subtract it before summing physical work."})
                );
            }
            exact_chains += usize::from(full_correct && control_changes_output);
        }
    }
    assert_eq!(
        exact_chains, 6,
        "learned generated composition remains unqualified; inspect reachability, selection, emission and intermediate control separately"
    );
}

#[test]
fn native_typed_roles_lineage_is_order_and_copy_invariant() {
    let model = mechanical_model(ValueAction::Add);
    let mut session = prefix(&model, "13 4 total:", Control::Full);
    let v = session.values.as_mut().unwrap();
    let literal = |id, value| ValueRecord {
        id,
        value,
        ..Default::default()
    };
    let derived = |id, value, action, ids, nums| ValueRecord {
        id,
        value,
        derived: true,
        derivation: Some(ValueDerivation {
            action,
            operand_ids: ids,
            operand_values: nums,
        }),
        ..Default::default()
    };
    v.sources = vec![
        literal(0, 13),
        literal(1, 4),
        derived(2, 17, ValueAction::Add, [0, 1], [13, 4]),
        literal(3, 5),
        derived(4, 22, ValueAction::Add, [2, 3], [17, 5]),
        derived(5, 17, ValueAction::Copy, [2, 2], [17, 17]),
    ];
    let d = super::typed_routing::lineage_depths(v, &mut Default::default());
    assert_eq!(&d[..6], &[0, 0, 1, 0, 2, 1]);
    let a = v.sources[2];
    let b = v.sources[4];
    let features = super::typed_routing::features_with_depths(
        v,
        Some((a, b)),
        &[0; 16],
        Some(&d),
        &mut Default::default(),
    );
    v.sources.reverse();
    let reversed = super::typed_routing::lineage_depths(v, &mut Default::default());
    assert_eq!(&reversed[..6], &[1, 2, 0, 1, 0, 0]);
    assert_eq!(
        features,
        super::typed_routing::features_with_depths(
            v,
            Some((a, b)),
            &[0; 16],
            Some(&reversed),
            &mut Default::default()
        )
    );
    // Numeric values are payloads, not lineage metadata.
    for r in &mut v.sources {
        r.value = -999;
    }
    assert_eq!(
        reversed,
        super::typed_routing::lineage_depths(v, &mut Default::default())
    );
    // Missing ancestors are explicit unknown, never silently depth zero.
    v.sources.retain(|r| r.id != 0);
    let missing = super::typed_routing::lineage_depths(v, &mut Default::default());
    for (i, r) in v.sources.iter().enumerate() {
        if r.derived {
            assert_eq!(missing[i], 255);
        }
    }
}

#[test]
fn native_typed_roles_alias_admission_uses_identity_not_numeric_equality() {
    let model = mechanical_model(ValueAction::Add);
    let mut session = prefix(&model, "13 4 total:", Control::Full);
    let v = session.values.as_mut().unwrap();
    let original = ValueRecord {
        id: 2,
        value: 17,
        derived: true,
        derivation: Some(ValueDerivation {
            action: ValueAction::Add,
            operand_ids: [0, 1],
            operand_values: [13, 4],
        }),
        ..Default::default()
    };
    let alias = ValueRecord {
        id: 3,
        derivation: Some(ValueDerivation {
            action: ValueAction::Copy,
            operand_ids: [2, 2],
            operand_values: [17, 17],
        }),
        ..original
    };
    let other = ValueRecord { id: 4, ..original };
    v.sources.extend([original, alias, other]);
    let origins = super::typed_routing::copy_origins(v, &mut Default::default());
    assert!(super::typed_routing::alias_self_add(
        v,
        original,
        alias,
        Some(&origins),
        &mut Default::default()
    ));
    assert!(!super::typed_routing::alias_self_add(
        v,
        original,
        other,
        Some(&origins),
        &mut Default::default()
    ));
    assert!(!super::typed_routing::alias_self_add(
        v,
        original,
        alias,
        None,
        &mut Default::default()
    ));
    v.sources.reverse();
    let reversed = super::typed_routing::copy_origins(v, &mut Default::default());
    assert!(super::typed_routing::alias_self_add(
        v,
        alias,
        original,
        Some(&reversed),
        &mut Default::default()
    ));
}
