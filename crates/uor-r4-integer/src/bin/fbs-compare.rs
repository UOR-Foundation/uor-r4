//! Compare a converted SpiralCore/FBS trunk with the float GPU trunk it came
//! from (d11-exporter-float-trunk, claude-123 and claude-124).
//!
//! ```text
//! fbs-compare SNAPSHOT_JSON READOUT_JSON [EXPORT_READOUT_JSON]
//! ```
//!
//! `SNAPSHOT_JSON` is the converter's snapshot. `READOUT_JSON` is the float
//! GPU trunk's own plain single-token outputs, and `EXPORT_READOUT_JSON` the
//! same in the float sheets' D11 export mode (`fbs_snapshot::FbsFloatReadout`).
//! Each readout must hold every token once, in order.
//!
//! It exports and parses the snapshot, then for every token compares:
//! - the plain readout with the reference that snaps nothing
//!   (`FbsReferenceSnaps::FLOAT`), and the export readout with the reference
//!   in export form (`FLOAT_EXPORT`): agreement here says the converter mapped
//!   every table right (the parity proof);
//! - the integer session with the fully snapped, quantized reference;
//! - the reference variants with each other, which needs no GPU: how much of
//!   the float-to-integer gap each snap and the table rounding explain.
//!
//! It prints one JSON line: readings, not bars. It exits non-zero only when
//! the snapshot does not export, parse or serve, or a readout does not fit.

use std::process::ExitCode;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_integer::fbs_snapshot::FbsFloatReadout;
use uor_r4_integer::stack::{
    export_fbs_trunk_uorlut01, FbsReferenceSnaps, FbsTrunkSnapshot, FbsWriterOutput,
    IntegerStackModel,
};

fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{path}: {e}"))
}

fn check_readout(
    snap: &FbsTrunkSnapshot,
    readout: &FbsFloatReadout,
    export_mode: bool,
    path: &str,
) -> Result<(), String> {
    readout
        .check(snap.vocab, snap.width, export_mode)
        .map_err(|e| format!("{path}: {e}"))
}

/// Token agreement and the largest query gap between two sets of outputs.
fn pair(a: &[(u32, Vec<f64>)], b: &[(u32, Vec<f64>)]) -> Value {
    let tokens = a.iter().zip(b).filter(|(x, y)| x.0 == y.0).count();
    let gap = a
        .iter()
        .zip(b)
        .flat_map(|(x, y)| x.1.iter().zip(&y.1).map(|(p, q)| (p - q).abs()))
        .fold(0f64, f64::max);
    json!({"tokens_agree": tokens, "max_abs_query_gap": gap})
}

fn reference(
    snap: &FbsTrunkSnapshot,
    quantized: bool,
    snaps: FbsReferenceSnaps,
) -> Result<Vec<(u32, Vec<f64>)>, String> {
    (0..snap.vocab as u32)
        .map(|t| {
            let out: Vec<FbsWriterOutput> = snap
                .forward_f32_with(&[t], quantized, snaps)
                .map_err(|e| format!("token {t}: reference: {e}"))?;
            Ok((out[0].token, out[0].query.clone()))
        })
        .collect()
}

fn floats(readout: &FbsFloatReadout) -> Vec<(u32, Vec<f64>)> {
    readout
        .positions
        .iter()
        .map(|p| (p.next, p.hidden.iter().map(|&h| f64::from(h)).collect()))
        .collect()
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (snapshot, plain, export) = match args.as_slice() {
        [s, p] => (s, p, None),
        [s, p, e] => (s, p, Some(e)),
        _ => {
            return Err(
                "usage: fbs-compare SNAPSHOT_JSON READOUT_JSON [EXPORT_READOUT_JSON]".to_owned(),
            )
        }
    };
    let snap: FbsTrunkSnapshot = read_json(snapshot)?;
    let plain_readout: FbsFloatReadout = read_json(plain)?;
    check_readout(&snap, &plain_readout, false, plain)?;
    let export_readout: Option<FbsFloatReadout> = export.map(|e| read_json(e)).transpose()?;
    if let (Some(e), Some(readout)) = (export, &export_readout) {
        check_readout(&snap, readout, true, e)?;
    }

    let artifact = export_fbs_trunk_uorlut01(&snap).map_err(|e| format!("export: {e}"))?;
    let model = IntegerStackModel::parse(&artifact).map_err(|e| format!("parse: {e}"))?;
    let served: Vec<u32> = (0..snap.vocab as u32)
        .map(|t| {
            model
                .session()
                .step_native(t)
                .map(|s| s.token)
                .map_err(|e| format!("token {t}: integer step: {e}"))
        })
        .collect::<Result<_, _>>()?;

    let float_ref = reference(&snap, false, FbsReferenceSnaps::FLOAT)?;
    let export_ref = reference(&snap, false, FbsReferenceSnaps::FLOAT_EXPORT)?;
    let integer_ref = reference(&snap, false, FbsReferenceSnaps::INTEGER)?;
    let quantized_ref = reference(&snap, true, FbsReferenceSnaps::INTEGER)?;
    let plain_gpu = floats(&plain_readout);
    let integer_vs_quantized = served
        .iter()
        .zip(&quantized_ref)
        .filter(|(s, r)| **s == r.0)
        .count();

    let mut report = json!({
        "positions": snap.vocab,
        "artifact_sha256": hex::encode(Sha256::digest(&artifact)),
        "parity_plain_gpu_vs_float_reference": pair(&plain_gpu, &float_ref),
        "plain_gpu_vs_integer_reference": pair(&plain_gpu, &integer_ref),
        "integer_token_equals_quantized_reference": integer_vs_quantized,
        "attribution": {
            "float_vs_float_export_reference": pair(&float_ref, &export_ref),
            "float_export_vs_integer_reference": pair(&export_ref, &integer_ref),
            "integer_plain_vs_quantized_reference": pair(&integer_ref, &quantized_ref),
        },
    });
    if let Some(readout) = &export_readout {
        report["parity_export_gpu_vs_float_export_reference"] = pair(&floats(readout), &export_ref);
    }
    println!("{report}");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("fbs-compare: {e}");
            ExitCode::FAILURE
        }
    }
}
