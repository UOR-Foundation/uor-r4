//! End-to-end regression: the committed frozen panel must calibrate.
//!
//! Uses the scripted responder and an all-repetition degenerate set built in
//! memory, so the test needs no tokenizer file or retained artifact.

use std::path::PathBuf;

use uor_r4_chat_eval::{calibrate, load_panel, Generation, GenerationRow};

fn panel_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/integration/chat-panel-v0-2026-09-25.json")
}

fn repetition_generations(panel: &uor_r4_chat_eval::Panel) -> Vec<GenerationRow> {
    panel
        .development
        .iter()
        .map(|row| GenerationRow {
            id: row.id.clone(),
            generations: vec![Generation {
                sample: "greedy".to_string(),
                seed: None,
                text: "the the the the the the the the the the".to_string(),
                stop: Some("maximum_new_tokens".to_string()),
                truncated: None,
            }],
            text: None,
        })
        .collect()
}

#[test]
fn committed_panel_calibrates() {
    let panel = load_panel(&panel_path()).expect("load committed panel");
    assert_eq!(panel.development.len(), 38);
    assert_eq!(panel.fresh.len(), 24);

    let degenerate = repetition_generations(&panel);
    let calibration = calibrate(&panel, &degenerate, None);

    assert!(
        calibration.scripted.aggregate.passes_v0,
        "scripted responder must pass v0: {:?}",
        calibration.scripted.aggregate.thresholds
    );
    assert!(
        calibration.degenerate.aggregate.core_on_topic.k <= 4,
        "degenerate on_topic must be <= 4/20, got {}",
        calibration.degenerate.aggregate.core_on_topic.k
    );
    assert!(
        calibration.degenerate.aggregate.cycle_rate.rate >= 0.50,
        "degenerate cycle_rate must be >= 0.50, got {}",
        calibration.degenerate.aggregate.cycle_rate.rate
    );
    assert!(
        !calibration.c1_retrieval.aggregate.passes_v0,
        "C1 retrieval control must not pass v0"
    );
    assert!(
        calibration.instrument_valid,
        "instrument must be valid: {:?}",
        calibration.checks
    );
}
