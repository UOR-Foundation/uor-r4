//! M1 bar (f), `d11-scorer panel-score` (claude-211): the gate, the reserved panels it refuses,
//! and the two-seed evaluation.

use m1_d11_artifact_scorer_instrument as inst;
use serde_json::json;
use uor_r4_discrete::corpus::{CorpusSource, FilePin, Split};

use inst::fresh_transfer::{
    evaluate_panel_two_seeds, refuse_reserved_panel, validate_decision_receipt,
    validate_fresh_transfer_gate, FreshSeedScore, DEV_TOKENS_SHA256, FRESH_V2_MASK_SHA256,
    FRESH_V2_TOKENS_SHA256, P_DEV_TOKENS_SHA256, TEMPLATE_SHARED_PANEL_TOKENS_SHA256,
    TRAIN_TOKENS_SHA256,
};

/// p-f, the bar (f) panel drawn on 30 Oct (docs/labs/m1-heldout-phrasing-transfer/README.md).
const P_F_TOKENS: &str = "e6abbdf0dd2f2fff1810d5e1fd95f6cfa20582c57c98e990a1d9c852e5e43fef";
const P_F_MASK: &str = "c63c625a9897b4ee42043028addb3f9f831d7ecc28976cae8223ba0050e5c910";
const S1_ART: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
const S2_ART: &str = "1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de";

fn source(tokens: &str, mask: &str, payload: u64, eligible: u64) -> CorpusSource {
    CorpusSource {
        split: Split::Dev,
        tokens: FilePin { path: "/nonexistent/tokens.u16".into(), sha256: tokens.into() },
        mask: FilePin { path: "/nonexistent/response_mask.u8".into(), sha256: mask.into() },
        expected_payload_tokens: payload,
        expected_eligible_targets: eligible,
    }
}

fn receipt() -> serde_json::Value {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let path = root.join(
        "docs/labs/m1-fresh-panel-transfer/evidence/trunk-joint-pointer-v1-d11-fresh-gate-receipt.json",
    );
    serde_json::from_slice(&std::fs::read(path).expect("bar (e)'s committed gate receipt"))
        .expect("parse the gate receipt")
}

/// The receipt binds the two saved artifacts on its own, for any panel; the Fresh V2 gate
/// still refuses every source but Fresh V2.
#[test]
fn the_receipt_binds_the_saved_artifacts_and_the_fresh_gate_keeps_its_pins() {
    let receipt = receipt();
    validate_decision_receipt(&receipt, S1_ART).expect("seed 1's artifact is bound");
    validate_decision_receipt(&receipt, S2_ART).expect("seed 2's artifact is bound");
    assert!(validate_decision_receipt(&receipt, &"cc".repeat(32)).is_err(), "an unbound artifact");
    let p_f = source(P_F_TOKENS, P_F_MASK, 50_061, 15_953);
    let err = validate_fresh_transfer_gate(&receipt, S1_ART, &p_f).unwrap_err();
    assert!(err.contains("does not match pinned Fresh V2"), "{err}");
}

/// Bar (f)'s panel is never a sealed fresh panel, the reserved P-dev, DEV or TRAIN.
#[test]
fn panel_score_refuses_the_reserved_panels() {
    refuse_reserved_panel(&source(P_F_TOKENS, P_F_MASK, 50_061, 15_953)).expect("p-f is admitted");
    for tokens in [P_DEV_TOKENS_SHA256, DEV_TOKENS_SHA256, TRAIN_TOKENS_SHA256, FRESH_V2_TOKENS_SHA256] {
        assert!(refuse_reserved_panel(&source(tokens, P_F_MASK, 1, 1)).is_err(), "{tokens}");
    }
    // A panel that shares TRAIN's prompt templates is no held-out-phrasing panel.
    for tokens in TEMPLATE_SHARED_PANEL_TOKENS_SHA256 {
        assert!(refuse_reserved_panel(&source(tokens, P_F_MASK, 1, 1)).is_err(), "{tokens}");
    }
    assert!(refuse_reserved_panel(&source(P_F_TOKENS, FRESH_V2_MASK_SHA256, 1, 1)).is_err());
    assert!(refuse_reserved_panel(&source("not-hex", P_F_MASK, 1, 1)).is_err());
}

fn seed(seed: u64, model: u64, t2: u64, b_t2: u64, c_t2: u64, psucc_model: u64) -> FreshSeedScore {
    FreshSeedScore {
        seed,
        artifact_sha256: if seed == 1 { S1_ART } else { S2_ART }.into(),
        rows: 15_953,
        psucc_rows: 3_000,
        refused_rows: 0,
        reference_mismatches: 0,
        b4_pass: true,
        model_top1: model,
        model_top8: model,
        unigram_top1: 1_300,
        t2_top1: t2,
        b_vs_unigram: model - 1_000,
        c_vs_unigram: 300,
        b_vs_t2: b_t2,
        c_vs_t2: c_t2,
        psucc_model_top1: psucc_model,
        psucc_unigram_top1: 100,
        psucc_t2_top1: 1_700,
        psucc_b_vs_t2: 10,
        psucc_c_vs_t2: 10,
    }
}

/// Bar (f) is bar (e)'s two full-panel conditions with its band; the copy-successor condition
/// is reported, not required (D82 names the unigram and T2).
#[test]
fn bar_f_needs_both_seeds_past_the_unigram_and_t2_by_bar_es_band() {
    // b + c = 1,000 vs T2: floor ⌈2·√1000⌉ = 64; spread |7,300 − 7,310| = 10. Model 7,300 vs
    // T2 6,600 is past it. P_succ 1,000 vs T2's 1,700 fails bar (e)'s third condition.
    let s1 = seed(1, 7_300, 6_600, 800, 200, 1_000);
    let s2 = seed(2, 7_310, 6_600, 810, 190, 1_000);
    let out = evaluate_panel_two_seeds(&s1, &s2, P_F_TOKENS, P_F_MASK).unwrap();
    assert_eq!(out["outcome"], json!("PASS_BAR_F_HELDOUT_PHRASING"));
    assert_eq!(out["bar_f_pass"], json!(true));
    assert_eq!(out["seed_1"]["beats_psucc_t2"], json!(false), "reported, not required");
    assert_eq!(out["panel_tokens_sha256"], json!(P_F_TOKENS));
    assert!(out.get("bar_e_pass").is_none() && out.get("fresh_tokens_sha256").is_none());
    // One seed within T2's band fails the bar: model 6,650 is only 50 past T2, under 64.
    let near = seed(2, 6_650, 6_600, 525, 475, 1_000);
    let out = evaluate_panel_two_seeds(&s1, &near, P_F_TOKENS, P_F_MASK).unwrap();
    assert_eq!(out["outcome"], json!("FAIL_BAR_F_CLOSE_CONFIGURATION"));
    assert_eq!(out["seed_2"]["bar_f_seed_pass"], json!(false));
    // A parity fault fails it too.
    let mut faulted = s2.clone();
    faulted.reference_mismatches = 1;
    let out = evaluate_panel_two_seeds(&s1, &faulted, P_F_TOKENS, P_F_MASK).unwrap();
    assert_eq!(out["bar_f_pass"], json!(false));
}
