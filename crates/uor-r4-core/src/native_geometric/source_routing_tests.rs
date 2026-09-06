use super::*;
fn docs() -> Vec<ValueExample> {
    ["alpha", "bravo", "cedar", "delta"]
        .iter()
        .enumerate()
        .flat_map(|(i, n)| {
            [
                ValueExample {
                    id: format!("route-copy-{i}"),
                    prompt: format!("holder in {n}. Where is holder? Answer:"),
                    response: format!(" {n}.\n"),
                },
                ValueExample {
                    id: format!("route-none-{i}"),
                    prompt: format!("{n} in city. Where is missing? Answer:"),
                    response: " Unknown.\n".into(),
                },
            ]
        })
        .collect()
}
fn fitted() -> Model {
    let parent = role_read_tests::fitted();
    let (m, r) = parent
        .fit_source_routing(
            &docs(),
            SourceRoutingConfig {
                learned_features: 16,
                passes: 1,
                proposals: 2,
                role_context_only: true,
                ..SourceRoutingConfig::default()
            },
        )
        .unwrap();
    assert!(!r["stopped_at_time_limit"].as_bool().unwrap());
    assert!(r["frames"].as_u64().unwrap() > 0);
    assert!(r["final_correct"].as_u64().unwrap() >= r["initial_correct"].as_u64().unwrap());
    assert!(m
        .source_routing
        .as_ref()
        .unwrap()
        .codes
        .iter()
        .all(|c| c.feature.kind < 20));
    m
}
#[test]
fn source_routing_fit_reloads_and_disabled_path_preserves_complete_parent() {
    let parent = role_read_tests::fitted();
    let m = fitted();
    let loaded = Model::from_bytes(&m.to_bytes().unwrap()).unwrap();
    for d in docs() {
        assert_eq!(
            m.generate(&d.prompt, 32, Control::Full).unwrap(),
            loaded.generate(&d.prompt, 32, Control::Full).unwrap()
        );
        let before = parent.generate(&d.prompt, 32, Control::Full).unwrap();
        let mut disabled = loaded
            .generate(&d.prompt, 32, Control::LearnedRoutingDisabled)
            .unwrap();
        disabled.state.control = Control::Full;
        assert_eq!(before, disabled);
    }
    for field in 0..2 {
        let mut bad = m.clone();
        let b = bad.source_routing.as_mut().unwrap();
        if field == 0 {
            b.codes[0].roots[0] = 120;
        } else {
            b.parent_artifact = "different".into();
        }
        bad.refresh_identity().unwrap();
        assert!(Model::from_bytes(&bad.to_bytes().unwrap()).is_err());
    }
}
#[test]
fn source_routing_selected_exact_source_commits_only_after_observation() {
    let mut m = fitted();
    let actions = &role_read::head(&m).unwrap().actions;
    let copy = actions.iter().position(|a| a.copy).unwrap();
    let b = m.source_routing.as_mut().unwrap();
    b.biases.fill(-32);
    b.biases[copy] = 32;
    m.refresh_identity().unwrap();
    let prompt = "holder in alpha. Where is holder? Answer:";
    let mut s = m.session(Control::Full).unwrap();
    s.observe(&m, BOS).unwrap();
    for t in m.encode(prompt).unwrap() {
        s.observe(&m, t).unwrap();
    }
    s.begin_response(&m).unwrap();
    let prior = s.checkpoint().unwrap();
    let first = s.predict(&m).unwrap();
    let decision = s.word_copy_decision().unwrap();
    assert!(matches!(
        decision.action,
        WordCopyAction::Read | WordCopyAction::Prepare
    ));
    assert!(s.word_copy.as_ref().unwrap().read_commit.is_none());
    let mut mismatch = m.restore_session(&prior).unwrap();
    mismatch.predict(&m).unwrap();
    mismatch.observe(&m, u32::from(b'?') + 2).unwrap();
    assert!(mismatch.word_copy.as_ref().unwrap().read_commit.is_none());
    s.observe(&m, first.token).unwrap();
    let mut restored = m.restore_session(&s.checkpoint().unwrap()).unwrap();
    assert!(s.word_copy.as_ref().unwrap().read_commit.is_some());
    for _ in 0..4 {
        let p = s.predict(&m).unwrap();
        assert_eq!(p, restored.predict(&m).unwrap());
        assert_eq!(s.word_copy_decision(), restored.word_copy_decision());
        s.observe(&m, p.token).unwrap();
        restored.observe(&m, p.token).unwrap();
        if p.token == EOS {
            break;
        }
    }
}
