//! The host twin of the FBS core equals the Rust integer session bit for bit
//! (token, kept cells and every Q12 query value) at every position of the
//! synthetic controls (claude-128, ORDER done-when 2).

use d11_fbs_evaluator::{first_difference, host_sequences, layouts_agree, session_sequences};
use uor_r4_integer::stack::synthetic::{
    fbs_synthetic_non_square, fbs_synthetic_tiny_gain, synthetic_fbs_trunk,
    synthetic_fbs_trunk_with, Lcg, FBS_SYNTH_NON_SQUARE,
};
use uor_r4_integer::stack::{
    export_fbs_trunk_uorlut01, export_fbs_trunk_uorlut01_with, FbsExportSnaps, FbsTrunkSnapshot,
    IntegerStackModel,
};

fn model_of(snap: &FbsTrunkSnapshot) -> IntegerStackModel {
    IntegerStackModel::parse(&export_fbs_trunk_uorlut01(snap).expect("export")).expect("parse")
}

/// Every token of the vocabulary, each from a fresh session.
fn single_tokens(vocab: usize) -> Vec<Vec<u32>> {
    (0..vocab as u32).map(|t| vec![t]).collect()
}

/// `count` sequences of `len` tokens drawn from `seed`.
fn sequences(vocab: usize, count: usize, len: usize, seed: u64) -> Vec<Vec<u32>> {
    let mut rng = Lcg(seed);
    (0..count)
        .map(|_| {
            (0..len)
                .map(|_| (rng.next() % vocab as u64) as u32)
                .collect()
        })
        .collect()
}

fn assert_parity(name: &str, model: &IntegerStackModel, seqs: &[Vec<u32>]) {
    let host = host_sequences(model, seqs).expect("host twin");
    let rust = session_sequences(model, seqs).expect("Rust session");
    let positions = host.sequences * host.len;
    eprintln!("{name}: {positions} positions compared");
    assert!(positions > 0, "{name}: nothing was compared");
    assert_eq!(
        first_difference(&host, &rust),
        None,
        "{name}: the host twin departs from the Rust session"
    );
    assert_eq!(host, rust, "{name}");
}

#[test]
fn the_c_and_rust_struct_sizes_agree() {
    assert!(layouts_agree());
}

#[test]
fn host_twin_equals_the_session_on_single_tokens() {
    for (name, snap) in [
        ("small", synthetic_fbs_trunk(20261025)),
        ("non-square", fbs_synthetic_non_square(20261026)),
        ("tiny-gain", fbs_synthetic_tiny_gain(20261026)),
    ] {
        let model = model_of(&snap);
        assert_parity(name, &model, &single_tokens(snap.vocab));
    }
}

#[test]
fn host_twin_equals_the_session_over_sequences() {
    for (name, snap) in [
        ("small", synthetic_fbs_trunk(20261025)),
        ("non-square", fbs_synthetic_non_square(20261026)),
        ("tiny-gain", fbs_synthetic_tiny_gain(20261026)),
    ] {
        let model = model_of(&snap);
        assert_parity(name, &model, &sequences(snap.vocab, 16, snap.context, 7));
    }
    // State carried across 64 positions of the non-square trunk.
    let long = synthetic_fbs_trunk_with(
        20261027,
        uor_r4_integer::stack::synthetic::FbsSynth {
            context: 64,
            ..FBS_SYNTH_NON_SQUARE
        },
    );
    let model = model_of(&long);
    assert_parity(
        "non-square, 64 positions",
        &model,
        &sequences(long.vocab, 8, 64, 11),
    );
}

/// claude-181's four snap choices (claude-182): the gates and the transition each snapped to the
/// icosians or served as the unit quaternion.
const SNAP_CHOICES: [FbsExportSnaps; 4] = [
    FbsExportSnaps::SNAPPED,
    FbsExportSnaps {
        gates: false,
        transition: true,
    },
    FbsExportSnaps {
        gates: true,
        transition: false,
    },
    FbsExportSnaps {
        gates: false,
        transition: false,
    },
];

/// The host twin equals the Rust session bit for bit in every snap choice, on single tokens and
/// over sequences (the transition moves a state only from a sequence's second token), and each
/// unsnapped choice changes the outputs.
#[test]
fn host_twin_equals_the_session_in_every_snap_choice() {
    for (name, snap) in [
        ("small", synthetic_fbs_trunk(20261025)),
        ("non-square", fbs_synthetic_non_square(20261026)),
        ("tiny-gain", fbs_synthetic_tiny_gain(20261026)),
    ] {
        let seqs = sequences(snap.vocab, 16, snap.context, 7);
        let snapped = host_sequences(&model_of(&snap), &seqs).expect("host twin");
        for snaps in SNAP_CHOICES {
            let model = IntegerStackModel::parse(
                &export_fbs_trunk_uorlut01_with(&snap, snaps).expect("export"),
            )
            .expect("parse");
            assert_eq!(
                (model.shape().raw_gates, model.shape().raw_transition),
                (!snaps.gates, !snaps.transition)
            );
            let label = format!("{name} {snaps:?}");
            assert_parity(&label, &model, &single_tokens(snap.vocab));
            assert_parity(&label, &model, &seqs);
            if snaps != FbsExportSnaps::SNAPPED {
                let host = host_sequences(&model, &seqs).expect("host twin");
                assert_ne!(host, snapped, "{label} computed the snapped forward");
            }
        }
    }
}

/// The snap trace (claude-134) changes no output, has `trace_stride` bytes per
/// position (per layer: 2 × width/4 + mlp/4), names only the 120 icosian
/// roots or 255 (a zero raw), repeats exactly, and is not one root everywhere.
#[test]
fn the_snap_trace_leaves_the_outputs_unchanged_and_names_roots() {
    use d11_fbs_evaluator::host_trace_sequences;
    for (name, snap) in [
        ("small", synthetic_fbs_trunk(20261025)),
        ("non-square", fbs_synthetic_non_square(20261026)),
    ] {
        let model = model_of(&snap);
        let shape = model.shape();
        let len = shape.context.min(8);
        let seqs = sequences(snap.vocab, 4, len, 7);
        let plain = host_sequences(&model, &seqs).expect("host twin");
        let (traced, trace) = host_trace_sequences(&model, &seqs).expect("traced host twin");
        assert_eq!(traced, plain, "{name}: tracing changed an output");
        let layers = model.fbs_image().expect("an FBS stack").layers.len();
        let stride = layers * (2 * (shape.width / 4) + shape.mlp / 4);
        assert_eq!(
            trace.len(),
            seqs.len() * len * stride,
            "{name}: trace length"
        );
        // The host buffer is prefilled with a byte the core never writes: none may survive.
        assert!(
            !trace.contains(&d11_fbs_evaluator::TRACE_UNWRITTEN),
            "{name}: a trace slot the core never wrote"
        );
        assert!(
            trace.iter().all(|&r| r < 120 || r == u8::MAX),
            "{name}: an index past the 120 roots"
        );
        let (_, again) = host_trace_sequences(&model, &seqs).expect("traced host twin");
        assert_eq!(trace, again, "{name}: the trace does not repeat");
        assert!(
            trace.iter().any(|&r| r != trace[0]),
            "{name}: one root everywhere"
        );
    }
}
