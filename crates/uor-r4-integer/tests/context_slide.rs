//! A read-free model may slide its token window at the context bound; a model whose reads name
//! positions may not.
//!
//! The recurrence and the native Q31 state carry the history; the token vector is only a read's
//! window. With no Hamming/place read and no turn memory, dropping the oldest token changes nothing
//! the model computes. This test steps past the bound, checks the window, checks determinism, and
//! checks that a read-bearing model refuses the policy.

#![cfg(all(feature = "serving-wasm", feature = "fbs-synthetic"))]

use uor_r4_integer::stack::synthetic::{
    synthetic_fbs_trunk_with, synthetic_hamming, FbsSynth, FBS_SYNTH_SMALL,
};
use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, IntegerStackModel, StackError};

fn ids() -> Vec<u32> {
    (0..16).map(|step| (step * 13 + 1) % 64).collect()
}

#[test]
fn a_read_free_model_slides_at_the_bound_and_a_read_bearing_one_refuses() {
    let shape = FbsSynth {
        context: 8,
        ..FBS_SYNTH_SMALL
    };
    let snap = synthetic_fbs_trunk_with(20_261_500, shape);
    let bytes = export_fbs_trunk_uorlut01(&snap).expect("export a read-free trunk");
    let model = IntegerStackModel::parse(&bytes).expect("parse");
    assert!(model.can_slide_tokens(), "the trunk is read-free");

    // Without the policy, the ninth step refuses.
    let mut refusing = model.session();
    let refused = {
        let mut result = Ok(());
        for &id in ids().iter().take(9) {
            result = refusing.step_native(id).map(|_| ());
            if result.is_err() {
                break;
            }
        }
        result
    };
    assert!(
        matches!(refused, Err(StackError::ContextFull { .. })),
        "a full context without the policy must refuse"
    );

    // With the policy, all sixteen steps serve, and the window holds the last eight ids.
    let mut sliding = model.session();
    sliding.set_context_slide(true).expect("a read-free model slides");
    let served: Vec<u32> = ids()
        .iter()
        .map(|&id| sliding.step_native(id).expect("slid step").token)
        .collect();
    assert_eq!(served.len(), 16);
    assert_eq!(
        sliding.tokens(),
        ids()[8..].to_vec(),
        "the window is the last context ids"
    );

    // Two runs of the same policy serve the same tokens.
    let mut again = model.session();
    again.set_context_slide(true).expect("slide");
    let repeated: Vec<u32> = ids()
        .iter()
        .map(|&id| again.step_native(id).expect("slid step").token)
        .collect();
    assert_eq!(served, repeated, "the slide is deterministic");

    // A Hamming read names token positions, so the policy is refused there.
    let mut read_snap = synthetic_fbs_trunk_with(20_261_501, shape);
    read_snap.hamming = Some(synthetic_hamming(
        read_snap.width,
        read_snap.layers.len(),
        4,
        20_261_502,
    ));
    let read_bytes = export_fbs_trunk_uorlut01(&read_snap).expect("export a read-bearing trunk");
    let read_model = IntegerStackModel::parse(&read_bytes).expect("parse");
    assert!(!read_model.can_slide_tokens());
    let mut read_session = read_model.session();
    assert!(read_session.set_context_slide(true).is_err());
}

/// A read-bearing trunk: the same synthetic trunk with the native Q31 read added at depth 1.
fn read_bearing(seed: u64, shape: FbsSynth) -> Vec<u8> {
    use uor_r4_integer::stack::{add_q31_read_uorlut01, Q31SessionRead};
    use uor_r4_native_vector::Q31FirstPickRead;

    let snap = synthetic_fbs_trunk_with(seed, shape);
    let plain = export_fbs_trunk_uorlut01(&snap).expect("export");
    let width = snap.width;
    let gates = vec![[1i64 << 31, 0, 0, 0]; width / 4];
    let read = Q31FirstPickRead::new(gates.clone(), gates, vec![0; width], 4).expect("read");
    add_q31_read_uorlut01(&plain, &Q31SessionRead::new(1, read)).expect("the read is added")
}

#[test]
fn a_read_bearing_model_slides_and_the_reads_window_is_the_served_window() {
    // The native Q31 read addresses its own rows, so it slides only if its history is dropped
    // with the token window; otherwise it would pick a row the window no longer holds.
    let shape = FbsSynth {
        context: 8,
        ..FBS_SYNTH_SMALL
    };
    let bytes = read_bearing(20_261_503, shape);
    let model = IntegerStackModel::parse(&bytes).expect("parse");
    assert!(model.can_slide_tokens(), "the native read slides with its window");

    let mut session = model.session();
    session.set_q31_read_trace(true).expect("trace");
    session.set_context_slide(true).expect("slide");
    let mut served = Vec::new();
    for (step, &id) in ids().iter().enumerate() {
        served.push(session.step_native(id).expect("slid step").token);
        let live = (step + 1).min(shape.context);
        assert_eq!(session.q31_read_history_len(), live, "window at step {step}");
        assert_eq!(
            session.q31_read_window(),
            session.tokens(),
            "the read's window is the served window at step {step}"
        );
    }
    assert_eq!(served.len(), 16);
    let picks: Vec<u32> = session
        .q31_read_trace()
        .expect("on")
        .iter()
        .map(|step| step.pick)
        .collect();
    assert!(
        picks.iter().all(|&pick| pick as usize <= shape.context),
        "a pick is a live row: {picks:?}"
    );

    // The policy is deterministic with the read.
    let mut again = IntegerStackModel::parse(&bytes).expect("parse");
    let mut repeated = again.session();
    repeated.set_q31_read_trace(true).expect("trace");
    repeated.set_context_slide(true).expect("slide");
    let same: Vec<u32> = ids()
        .iter()
        .map(|&id| repeated.step_native(id).expect("slid step").token)
        .collect();
    assert_eq!(served, same);
    assert_eq!(repeated.q31_read_window(), session.q31_read_window());
}

#[test]
fn a_copy_boost_refuses_the_slide() {
    // The copy boost indexes absolute rows; a slide would leave it reading rows the window no
    // longer holds.
    let shape = FbsSynth {
        context: 8,
        ..FBS_SYNTH_SMALL
    };
    let bytes = export_fbs_trunk_uorlut01(&synthetic_fbs_trunk_with(20_261_504, shape))
        .expect("export");
    let model = IntegerStackModel::parse(&bytes).expect("parse");
    let mut session = model.session();
    session.set_copy_scale(1 << 16).expect("a non-pointer model takes a copy scale");
    let message = session.set_context_slide(true).expect_err("the slide is refused");
    assert!(format!("{message}").contains("copy boost"), "{message}");
}

#[test]
fn under_the_bound_the_slide_changes_nothing() {
    // The policy may only take effect at the bound: below it, the two runs must agree token for
    // token (and the read with it).
    let shape = FbsSynth {
        context: 8,
        ..FBS_SYNTH_SMALL
    };
    let bytes = read_bearing(20_261_505, shape);
    let model = IntegerStackModel::parse(&bytes).expect("parse");
    let mut plain = model.session();
    plain.set_q31_read_trace(true).expect("trace");
    let mut sliding = model.session();
    sliding.set_q31_read_trace(true).expect("trace");
    sliding.set_context_slide(true).expect("slide");
    let below = ids();
    for (step, &id) in below.iter().take(8).enumerate() {
        assert_eq!(
            plain.step_native(id).expect("plain").token,
            sliding.step_native(id).expect("slid").token,
            "step {step}"
        );
    }
    let plain_picks: Vec<u32> = plain
        .q31_read_trace()
        .expect("on")
        .iter()
        .map(|step| step.pick)
        .collect();
    let sliding_picks: Vec<u32> = sliding
        .q31_read_trace()
        .expect("on")
        .iter()
        .map(|step| step.pick)
        .collect();
    assert_eq!(plain_picks, sliding_picks);
    assert_eq!(plain.tokens(), sliding.tokens());
}
