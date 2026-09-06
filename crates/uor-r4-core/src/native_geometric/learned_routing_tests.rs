use super::*;

fn fixture() -> (Model, Vec<Document>) {
    let docs:Vec<_>=(0..8).map(|i| Document{id:format!("routing-fit-{i}"),text:format!(
        "Alice saved {} gems. Bob saved {} gems. Alice gave Bob one gem. Bob has {} gems.\nfn add(a:i32,b:i32)->i32{{a+b}}\n",
        i+2,i+3,i+4)}).collect();
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 16,
            ..Config::default()
        },
        &docs,
    )
    .unwrap();
    trainer.train_documents(&docs).unwrap();
    (trainer.compile().unwrap(), docs)
}

#[test]
fn learned_routing_hard_fit_reload_and_disabled_parent_preservation() {
    let (parent, mut docs) = fixture();
    for document in &mut docs {
        document.id.push_str("-routing");
        document.text.push('\n');
    }
    let original = parent.to_bytes().unwrap();
    let (model, report) = parent
        .fit_routing_block(
            &docs,
            RoutingFitConfig {
                max_positions: 128,
                learned_tokens: 4,
                passes: 1,
                ..RoutingFitConfig::default()
            },
        )
        .unwrap();
    assert!(!report.stopped_at_time_limit);
    assert!(report.proposals > 0);
    assert!(report
        .initial_conditional_nll
        .iter()
        .zip(&report.final_conditional_nll)
        .all(|(a, b)| b <= a));
    let loaded = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
    assert!(parent.evaluate(&docs[..1], Control::Full).is_ok());
    assert!(model.evaluate(&docs[..1], Control::Full).is_err());
    let score = model
        .evaluate(
            &[Document {
                id: "routing-development".into(),
                text: "Alice saved 9 gems.".into(),
            }],
            Control::Full,
        )
        .unwrap();
    assert_eq!(score.work.learned_routing.predictions, score.positions);
    let prompt = "Alice saved 4 gems. Bob saved 5 gems. Bob has";
    assert_eq!(
        model.generate(prompt, 8, Control::Full).unwrap(),
        loaded.generate(prompt, 8, Control::Full).unwrap()
    );
    let before = parent.generate(prompt, 8, Control::Full).unwrap();
    let disabled = model
        .generate(prompt, 8, Control::LearnedRoutingDisabled)
        .unwrap();
    assert_eq!(before.bytes, disabled.bytes);
    assert_eq!(before.work, disabled.work);
    assert_eq!(original, parent.to_bytes().unwrap());
    let mut malformed = model.clone();
    malformed.learned_routing.as_mut().unwrap().heads[0].keys[0] = 120;
    malformed.refresh_identity().unwrap();
    assert!(Model::from_bytes(&malformed.to_bytes().unwrap()).is_err());
}

#[test]
fn learned_routing_selects_exact_source_then_executes_query_conditioned_transport() {
    let (model, _) = fixture();
    let identity = model.geometry.identity;
    let opposite = (0..120)
        .find(|&i| model.geometry.anchors.rows[i].root_scaled_zphi[0] == [-2, 0])
        .unwrap() as u16;
    let rank: Vec<_> = model
        .geometry
        .anchors
        .rows
        .iter()
        .map(|r| u16::from(r.root_index == identity))
        .collect();
    let vocab = model.vocabulary_size();
    let mut head = learned_routing::Head {
        queries: vec![identity; vocab],
        keys: vec![opposite; vocab],
        values: vec![identity; vocab],
        operators: vec![opposite; 120],
        emissions: vec![],
    };
    head.keys[3] = identity;
    let mut work = RoutingWork::default();
    let chosen = head.route(
        &model,
        &rank,
        RoutingMode::Angular,
        &[2, 3, 4],
        Control::Full,
        &mut work,
    );
    assert_eq!(chosen.source_token, 3);
    assert_eq!(chosen.source_offset, 1);
    assert_eq!(chosen.output, opposite);
    let recent = head.route(
        &model,
        &rank,
        RoutingMode::Angular,
        &[2, 3, 4],
        Control::LearnedRoutingSelectionDisabled,
        &mut RoutingWork::default(),
    );
    assert_eq!(recent.source_token, 2);
    let unchanged = head.route(
        &model,
        &rank,
        RoutingMode::Angular,
        &[2, 3, 4],
        Control::LearnedRoutingTransformDisabled,
        &mut RoutingWork::default(),
    );
    assert_eq!(unchanged.output, identity);
    assert_eq!(
        (
            work.sources_examined,
            work.payload_gathers,
            work.operator_executions
        ),
        (3, 1, 1)
    );
    let mut saturated = RoutingWork {
        comparisons: u64::MAX,
        table_reads: u64::MAX,
        logical_bytes_read: u64::MAX,
        ..RoutingWork::default()
    };
    assert_eq!(
        chosen,
        head.route(
            &model,
            &rank,
            RoutingMode::Angular,
            &[2, 3, 4],
            Control::Full,
            &mut saturated
        )
    );
    assert_eq!(saturated.table_reads, u64::MAX);
}

#[test]
fn learned_routing_preserves_frozen_nested_parent_validation() {
    let (mut parent, docs) = fixture();
    parent.values = Some(value_types::ValueModel {
        schema: value_types::LEXEME_VALUE_SCHEMA.into(),
        codec: numeral::NUMERAL_CODEC.into(),
        capacity: value_types::VALUES,
        rows: Vec::new(),
        continuation_score: 0,
        fit_config: [0; 4],
        training: Vec::new(),
    });
    parent.refresh_identity().unwrap();
    parent.completion = Some(completion_types::CompletionModel {
        schema: completion_types::COMPLETION_SCHEMA.into(),
        baseline_artifact: parent.artifact_cid.clone(),
        rows: Vec::new(),
        global_postings: Vec::new(),
        fit_config: [0; 4],
        fit_positions: 0,
        training: Vec::new(),
    });
    parent.refresh_identity().unwrap();
    let (model, _) = parent
        .fit_routing_block(
            &docs,
            RoutingFitConfig {
                max_positions: 32,
                learned_tokens: 1,
                passes: 1,
                ..RoutingFitConfig::default()
            },
        )
        .unwrap();
    let loaded = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
    assert_eq!(loaded.completion, parent.completion);
    let mut malformed = model.clone();
    malformed.completion.as_mut().unwrap().baseline_artifact = "wrong".into();
    let mut changed_parent = malformed.clone();
    changed_parent.learned_routing = None;
    changed_parent.refresh_identity().unwrap();
    malformed.learned_routing.as_mut().unwrap().parent_artifact = changed_parent.artifact_cid;
    malformed.refresh_identity().unwrap();
    assert!(Model::from_bytes(&malformed.to_bytes().unwrap()).is_err());
}

#[test]
fn learned_routing_uses_ordered_retained_context_and_snapshot_replay() {
    let (parent, docs) = fixture();
    let (model, _) = parent
        .fit_routing_block(
            &docs,
            RoutingFitConfig {
                max_positions: 64,
                learned_tokens: 2,
                passes: 1,
                ..RoutingFitConfig::default()
            },
        )
        .unwrap();
    let tokens = model.encode(&docs[0].text).unwrap();
    let mut session = model.session(Control::Full).unwrap();
    for &t in &tokens {
        session.observe(&model, t).unwrap();
    }
    let expected = session.predict(&model).unwrap();
    let decision = session.routing_decision().unwrap();
    let mut restored = model
        .restore_session(&session.checkpoint().unwrap())
        .unwrap();
    assert_eq!(expected, restored.predict(&model).unwrap());
    assert_eq!(decision, restored.routing_decision().unwrap());
    for head in decision.heads {
        assert_eq!(
            head.source_token,
            tokens[tokens.len() - 1 - head.source_offset]
        );
        assert!(head.source_offset < 8);
    }
    let mut tail = model.session(Control::Full).unwrap();
    for &t in &tokens[tokens.len() - 16..] {
        tail.observe(&model, t).unwrap();
    }
    // The parent's previous-window feature can differ after eviction. The
    // new routing block itself must depend only on the retained recent tail.
    tail.predict(&model).unwrap();
    assert_eq!(decision, tail.routing_decision().unwrap());
}
