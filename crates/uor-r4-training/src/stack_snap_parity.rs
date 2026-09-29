//! The D11 snap parity report: a snapped model directory's float forward
//! against its exported multiplier-free artifact, root selection by root
//! selection and logit by logit, on evenly spaced windows of given tokens.
//!
//! This is the S1.4 acceptance vehicle: root-index parity with near-ties
//! counted separately, top-1 agreement and the largest absolute logit gap.
//! Floating point is used here only, offline, as the comparison reference.

use std::fs;
use std::path::Path;

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_integer::stack::IntegerStackModel;

use crate::geometric_stack::{StackModel, TransportSnap};
use crate::stack_export::export_stack;
use crate::{invalid, Result};

/// Run the snap parity report for the snapped model saved at `model_dir`
/// over `windows` evenly spaced windows of `tokens` (each of the model's
/// context length): export the artifact in memory, serve it with the D11
/// engine with the snap trace on, and compare against the float snapped
/// forward's [`crate::geometric_stack::StackModel::snap_selections`].
pub fn snap_parity_report(model_dir: &Path, tokens: &[u32], windows: usize) -> Result<Value> {
    let mut model = StackModel::load(model_dir, &Device::Cpu)?;
    let snap = StackModel::saved_transport_snap(model_dir)?
        .ok_or_else(|| invalid("snap parity needs a snapped save (transport_snap.json)"))?;
    model.set_transport_snap(Some(snap))?;
    let time = model.config.context;
    if windows == 0 || tokens.len() <= time + windows {
        return Err(invalid("too few tokens for the requested windows"));
    }
    let (bytes, _) = export_stack(
        &model,
        json!({"snap_parity": model_dir.display().to_string()}),
        None,
        Some(snap),
    )?;
    let integer = IntegerStackModel::parse(&bytes).map_err(|e| invalid(e.to_string()))?;
    let lanes = model.config.width / 4;
    let stride = (tokens.len() - time - 1) / windows;
    let mut session = integer.session();
    session.enable_snap_trace();
    let (mut compared, mut near_ties, mut mismatches, mut top1_agree) = (0usize, 0usize, 0, 0);
    let mut max_gap = 0f64;
    let mut per_window = Vec::with_capacity(windows);
    for window in 0..windows {
        let start = window * stride;
        let ids = &tokens[start..start + time];
        let float_logits = model.forward(ids, 1, time)?.to_vec2::<f32>()?;
        let selections = model.snap_selections(ids, 1, time)?;
        session.reset();
        session.enable_snap_trace();
        let mut window_mismatch = 0usize;
        for (t, &id) in ids.iter().enumerate() {
            let logits = session.step(id).map_err(|e| invalid(e.to_string()))?;
            let row = &float_logits[t];
            let top_i = logits
                .iter()
                .enumerate()
                .max_by_key(|&(_, &v)| v)
                .map(|(i, _)| i)
                .ok_or_else(|| invalid("an empty logit row"))?;
            let top_f = row
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .map(|(i, _)| i)
                .ok_or_else(|| invalid("an empty float logit row"))?;
            top1_agree += usize::from(top_i == top_f);
            for (&a, &b) in logits.iter().zip(row) {
                max_gap = max_gap.max((f64::from(a) / 65536.0 - f64::from(b)).abs());
            }
        }
        let trace = session
            .snap_trace()
            .ok_or_else(|| invalid("the snap trace is not enabled"))?;
        for (layer, selected) in &selections {
            for (k, selection) in selected.iter().enumerate() {
                let entry = trace
                    .iter()
                    .find(|e| e.layer == *layer && e.position == k / lanes && e.lane == k % lanes)
                    .ok_or_else(|| invalid("the trace missed a selection"))?;
                // The float margin to the runner-up: near zero, the integer
                // kernel's exact comparison can legitimately break an f32
                // tie the other way; count those separately.
                if selection.margin < 1e-4 {
                    near_ties += 1;
                } else {
                    compared += 1;
                    if entry.root != selection.index {
                        mismatches += 1;
                        window_mismatch += 1;
                    }
                }
            }
        }
        per_window.push(json!({
            "window": window,
            "start": start,
            "root_mismatches": window_mismatch,
        }));
    }
    let targets = windows * time;
    Ok(json!({
        "schema": "uor-r4.stack-snap-parity/1",
        "model": model_dir.display().to_string(),
        "transport_snap": snap.record(),
        "windows": windows,
        "positions": targets,
        "selections_compared": compared,
        "near_ties": near_ties,
        "root_mismatches": mismatches,
        "top1_agreement": top1_agree as f64 / targets as f64,
        "max_abs_logit_gap_nats": max_gap,
        "per_window": per_window,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_stack::{ReadScore, StackArch, StackConfig};

    #[test]
    fn a_tiny_snapped_model_reports_parity() {
        let dir = std::env::temp_dir().join(format!("uor-snap-parity-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let mut config = StackConfig::transformer_control(0x5A9);
        config.arch = StackArch::Geometric;
        config.vocab_size = 64;
        config.width = 32;
        config.heads = 2;
        config.mlp_hidden = 32;
        config.context = 8;
        config.pattern = "ra".to_owned();
        config.read = ReadScore::Lorentz;
        config.rotation = true;
        let mut model = StackModel::new(config, &Device::Cpu).expect("model");
        model
            .set_transport_snap(Some(TransportSnap::Icosian))
            .expect("snap");
        model.save(&dir).expect("save");
        let tokens: Vec<u32> = (0..40).map(|i| (i * 5 + 1) % 64).collect();
        let report = snap_parity_report(&dir, &tokens, 2).expect("report");
        assert_eq!(report["root_mismatches"], 0);
        assert_eq!(report["top1_agreement"], 1.0);
        assert!(report["selections_compared"].as_u64().expect("compared") > 0);
        fs::remove_dir_all(&dir).expect("cleanup");
    }
}
