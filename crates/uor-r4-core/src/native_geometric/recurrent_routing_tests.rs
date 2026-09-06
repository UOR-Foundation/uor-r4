use super::learned_routing::{Emission, Head, RoutingBlock};
use super::recurrent_routing::{JointOutput, JointRow};
use super::*;

#[test]
fn recurrent_routing_first_value_changes_second_source_under_equal_scan_work() {
    let (model, _) = learned_routing_tests::fixture();
    let identity = model.geometry.identity;
    let opposite = model
        .geometry
        .anchors
        .rows
        .iter()
        .find(|r| r.root_scaled_zphi[0] == [-2, 0])
        .unwrap()
        .root_index;
    let mut head = Head {
        queries: vec![identity; model.vocabulary_size()],
        keys: vec![opposite; model.vocabulary_size()],
        values: vec![identity; model.vocabulary_size()],
        operators: vec![identity; 120],
        emissions: vec![],
    };
    head.keys[3] = identity;
    let second = head.clone();
    head.values[3] = opposite;
    let block = RoutingBlock {
        schema: recurrent_routing::SCHEMA.into(),
        parent_artifact: model.artifact_cid().into(),
        mode: RoutingMode::Angular,
        heads: vec![head, second],
        angular_rank: learned_routing_training::ranks(&model),
        training: vec![],
        fit_config: RoutingFitConfig::default(),
        joint: Some(JointOutput {
            additive_scores: false,
            rows: vec![],
            priors: vec![],
            positions: 1,
            response_examples: 1,
        }),
    };
    let mut full_work = RoutingWork::default();
    let full = block.route(&model, &[2, 3, 4], Control::Full, &mut full_work);
    let mut disabled_work = RoutingWork::default();
    let disabled = block.route(
        &model,
        &[2, 3, 4],
        Control::LearnedRoutingChainDisabled,
        &mut disabled_work,
    );
    assert_eq!(full.heads[0], disabled.heads[0]);
    assert_eq!(
        (full.heads[0].source_token, full.heads[1].source_token),
        (3, 2)
    );
    assert_eq!(disabled.heads[1].source_token, 3);
    assert_eq!(full_work, disabled_work);
    assert_eq!(
        (
            full_work.sources_examined,
            full_work.payload_gathers,
            full_work.operator_executions
        ),
        (6, 2, 2)
    );
}

#[test]
fn recurrent_routing_joint_fit_preserves_parent_and_reloads_causal_state() {
    let (parent, documents) = learned_routing_tests::fixture();
    let responses = vec![
        ValueExample {
            id: "joint-response-a".into(),
            prompt: "fn add(a:i32,b:i32)->i32{".into(),
            response: "a+b}".into(),
        },
        ValueExample {
            id: "joint-response-b".into(),
            prompt: "Alice saved".into(),
            response: " three gems.".into(),
        },
    ];
    let (model, report) = parent
        .fit_recurrent_routing(
            &documents,
            &responses,
            RoutingFitConfig {
                max_positions: 64,
                learned_tokens: 2,
                passes: 1,
                max_seconds: 30,
                ..RoutingFitConfig::default()
            },
        )
        .unwrap();
    assert!(!report.stopped_at_time_limit);
    assert!(report.response_positions > 0);
    assert!(report.final_correct >= report.initial_correct);
    let bytes = model.to_bytes().unwrap();
    let loaded = Model::from_bytes(&bytes).unwrap();
    let prompt = "Alice saved three gems.";
    let full = model.generate(prompt, 8, Control::Full).unwrap();
    assert_eq!(full, loaded.generate(prompt, 8, Control::Full).unwrap());
    let old = parent.generate(prompt, 8, Control::Full).unwrap();
    let disabled = loaded
        .generate(prompt, 8, Control::LearnedRoutingDisabled)
        .unwrap();
    assert_eq!(old.bytes, disabled.bytes);
    assert_eq!(old.work, disabled.work);
    let mut session = loaded.session(Control::Full).unwrap();
    session.observe(&loaded, BOS).unwrap();
    for t in loaded.encode(prompt).unwrap() {
        session.observe(&loaded, t).unwrap();
    }
    session.begin_response(&loaded).unwrap();
    let first = session.predict(&loaded).unwrap();
    let route = session.routing_decision();
    assert_eq!(first, session.predict(&loaded).unwrap());
    assert_eq!(route, session.routing_decision());
    session.observe(&loaded, first.token).unwrap();
    let snapshot = session.checkpoint().unwrap();
    let mut restored = loaded.restore_session(&snapshot).unwrap();
    assert_eq!(
        session.predict(&loaded).unwrap(),
        restored.predict(&loaded).unwrap()
    );
    let mut malformed = loaded.clone();
    malformed
        .learned_routing
        .as_mut()
        .unwrap()
        .joint
        .as_mut()
        .unwrap()
        .rows[0]
        .key = 0x90000;
    malformed.refresh_identity().unwrap();
    assert!(Model::from_bytes(&malformed.to_bytes().unwrap()).is_err());
}

#[test]
fn recurrent_routing_output_can_learn_eos_and_base_without_emitting_bos() {
    let output = JointOutput {
        additive_scores: false,
        positions: 2,
        response_examples: 1,
        priors: vec![0; 4],
        rows: vec![JointRow {
            key: 0,
            emission: Emission {
                default_score: -32,
                scores: vec![
                    TokenScore {
                        token: BOS,
                        score: -32,
                    },
                    TokenScore {
                        token: EOS,
                        score: 0,
                    },
                ],
                postings: vec![EOS, BOS],
            },
        }],
    };
    output.validate(4).unwrap();
    let (action, score, base) = output.choose([0, 1, 2, 3, 4], 2, &mut RoutingWork::default());
    assert_eq!(action, EOS);
    assert!(score > base);
    assert_eq!(
        output
            .choose([1, 2, 3, 4, 5], 2, &mut RoutingWork::default())
            .0,
        BOS
    );
    // An already-correct EOS is represented by Base; it is not a duplicate class.
    assert_eq!(
        output
            .choose([0, 1, 2, 3, 4], EOS, &mut RoutingWork::default())
            .0,
        BOS
    );
    let mut additive = output;
    additive.additive_scores = true;
    additive.rows[0].emission.default_score = 0;
    additive.rows[0].emission.scores[1].score = 64;
    additive.validate(4).unwrap();
    assert_eq!(
        additive
            .choose([0, 1, 2, 3, 4], 2, &mut RoutingWork::default())
            .0,
        EOS
    );
}
