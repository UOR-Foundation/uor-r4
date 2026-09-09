use super::*;
use std::sync::OnceLock;

fn documents() -> Vec<ValueExample> {
    ["alpha", "bravo", "cedar", "delta"]
        .into_iter()
        .enumerate()
        .flat_map(|(i, name)| {
            [
                ValueExample {
                    id: format!("refine-copy-{i}"),
                    prompt: format!("holder in {name}. Where is holder? Answer:"),
                    response: format!(" {name}.\n"),
                },
                ValueExample {
                    id: format!("refine-none-{i}"),
                    prompt: format!("{name} in city. Where is missing? Answer:"),
                    response: " Unknown.\n".into(),
                },
            ]
        })
        .collect()
}
fn config(features: usize) -> SourceRoutingConfig {
    SourceRoutingConfig {
        learned_features: features,
        passes: 1,
        proposals: 2,
        role_context_only: true,
        ..SourceRoutingConfig::default()
    }
}
fn pair() -> &'static (Model, Model, serde_json::Value) {
    static PAIR: OnceLock<(Model, Model, serde_json::Value)> = OnceLock::new();
    PAIR.get_or_init(|| {
        let (parent, _) = role_read_tests::fitted()
            .fit_source_routing(&documents(), config(16))
            .unwrap();
        let (refined, report) = parent
            .refine_source_routing(&documents(), config(32))
            .unwrap();
        (parent, refined, report)
    })
}
#[test]
fn source_refinement_reconstructs_exact_parent_and_reloads() {
    let (parent, refined, report) = pair();
    assert_eq!(report["warm_initialization_preserved"], true);
    assert_eq!(report["warm_features"], 16);
    assert!(report["added_features"].as_u64().unwrap() > 0);
    let witness = refined.source_routing_refinement.as_ref().unwrap();
    assert_eq!(&witness.previous, parent.source_routing.as_ref().unwrap());
    let mut reconstructed = refined.clone();
    reconstructed.source_routing_refinement = None;
    reconstructed.source_routing = Some(witness.previous.clone());
    reconstructed.refresh_identity().unwrap();
    assert_eq!(&reconstructed, parent);
    assert_eq!(refined.response_entry, parent.response_entry);
    let loaded = Model::from_bytes(&refined.to_bytes().unwrap()).unwrap();
    let prompt = &documents()[0].prompt;
    assert_eq!(
        loaded.generate(prompt, 32, Control::Full).unwrap(),
        refined.generate(prompt, 32, Control::Full).unwrap()
    );
}
#[test]
fn source_refinement_rejects_tampered_parent_and_inherited_parameters() {
    let (_, refined, _) = pair();
    for field in 0..4 {
        let mut bad = refined.clone();
        match field {
            0 => {
                bad.source_routing_refinement
                    .as_mut()
                    .unwrap()
                    .parent_artifact = "different".into()
            }
            1 => {
                bad.source_routing_refinement
                    .as_mut()
                    .unwrap()
                    .previous
                    .biases[0] += 1
            }
            2 => bad.prior_scores[0] += 1,
            _ => bad.source_routing.as_mut().unwrap().codes[0].roots[0] = 120,
        }
        bad.refresh_identity().unwrap();
        assert!(Model::from_bytes(&bad.to_bytes().unwrap()).is_err());
    }
}
#[test]
fn source_refinement_bounds_and_single_revision_are_explicit() {
    let (parent, refined, _) = pair();
    assert!(refined
        .refine_source_routing(&documents(), config(32))
        .is_err());
    assert!(parent
        .refine_source_routing(&documents(), config(8))
        .is_err());
    let mut changed_context = config(32);
    changed_context.role_context_only = false;
    assert!(parent
        .refine_source_routing(&documents(), changed_context)
        .is_err());
    assert!(parent.refine_source_routing(&[], config(32)).is_err());
    assert!(parent.source_routing_trace(&"x".repeat(65537)).is_err());
}
#[test]
fn source_refinement_trace_scores_match_selected_direct_choice() {
    let (_, refined, _) = pair();
    let trace = refined
        .source_routing_trace(&documents()[0].prompt)
        .unwrap();
    assert_eq!(trace["direct_source_dispatch"], true);
    let selected = trace["direct_choice"].as_array().unwrap();
    let source = selected[0].as_u64().unwrap();
    let action = selected[1].as_u64().unwrap();
    let score = selected[2].as_i64().unwrap();
    let mut first_best = None;
    for candidate in trace["candidates"].as_array().unwrap() {
        assert!(candidate["current_source_hints"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(
            candidate["pre_current_source_hint_state"],
            candidate["state"]
        );
        assert_eq!(
            candidate["pre_current_source_hint_scores"],
            candidate["scores"]
        );
        assert_eq!(
            candidate["pre_current_source_hint_features"],
            candidate["features"]
        );
        assert_eq!(
            candidate["pre_current_source_hint_feature_count"]
                .as_u64()
                .unwrap(),
            candidate["features"].as_array().unwrap().len() as u64
        );
        for scored in candidate["scores"].as_array().unwrap() {
            let value = scored["score"].as_i64().unwrap();
            if first_best.is_none_or(|(_, _, prior)| value > prior) {
                first_best = Some((
                    candidate["source"].as_u64().unwrap(),
                    scored["action"].as_u64().unwrap(),
                    value,
                ));
            }
        }
    }
    assert_eq!(first_best, Some((source, action, score)));
}
