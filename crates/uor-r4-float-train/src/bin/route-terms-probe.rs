//! How much of a trained trunk's projection the served k-term gather form keeps.
//!
//! ```text
//! route-terms-probe MODEL_DIR OUT_JSON [K...]
//! ```
//!
//! For every layer and projection it reads the projection's effective dense map through the
//! model's own CUDA projection on the unit basis, fits `k` signed Cl(0,6) gathers to each output
//! block (`fbs_export::fit_routes_k`), and reports the residual Frobenius norm and the number of
//! blocks the fit reproduces exactly. `k = 1` is the historical one-atom form; the residual at
//! `k = 1` is the share of the trained map the integer artifact discards, and `k = 2`, `k = 4`
//! say whether the extra terms have that mass to spend.
//!
//! GPU only, because it reads the map through the model's own projection.

use std::path::PathBuf;

use candle_core::Device;
use serde_json::json;
use uor_r4_float_train::fbs_export::{
    projection_widths, route_term_report, FBS_PROJECTIONS,
};
use uor_r4_float_train::geometric_stack::StackModel;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        return Err(
            "usage: route-terms-probe MODEL_DIR OUT_JSON [K...] (default 1 2 4)".into(),
        );
    }
    let dir = PathBuf::from(&args[0]);
    let out = PathBuf::from(&args[1]);
    let ks: Vec<usize> = if args.len() > 2 {
        args[2..]
            .iter()
            .map(|k| {
                k.parse::<usize>()
                    .map_err(|e| format!("k {k}: {e}").into())
            })
            .collect::<Result<Vec<usize>>>()?
    } else {
        vec![1, 2, 4]
    };
    if ks.iter().any(|k| !(1..=4).contains(k)) {
        return Err(format!("every k must be 1..=4, got {ks:?}").into());
    }

    let device = Device::new_cuda(0)?;
    let model = StackModel::load(&dir, &device)?;
    let (d, m) = (model.config.width, model.config.mlp_hidden);

    let mut layers = Vec::new();
    let mut worst: Option<(String, usize, f64)> = None;
    for layer in 0..model.config.layers() {
        let mut projections = Vec::new();
        for part in FBS_PROJECTIONS {
            let (in_width, out_width) = projection_widths(part, d, m);
            let name = format!("layers.{layer:02}.{part}.spiralcore_fbs");
            let report = route_term_report(&model, &name, in_width, out_width, &ks)?;
            let mut arms = Vec::new();
            for (k, residual, dense, exact, blocks) in &report {
                let share = if *dense > 0.0 { residual / dense } else { 0.0 };
                if *k == 1 && worst.as_ref().is_none_or(|(_, _, w)| share > *w) {
                    worst = Some((name.clone(), layer, share));
                }
                arms.push(json!({
                    "k": k,
                    "residual": residual,
                    "dense": dense,
                    "residual_share": share,
                    "blocks_exact": exact,
                    "blocks": blocks,
                }));
            }
            projections.push(json!({ "part": part, "in_width": in_width, "out_width": out_width, "arms": arms }));
        }
        layers.push(json!({ "layer": layer, "projections": projections }));
    }

    let report = json!({
        "schema": "uor-r4.route-terms-probe/1",
        "model_dir": dir.display().to_string(),
        "width": d,
        "mlp_hidden": m,
        "ks": ks,
        "worst_k1_share": worst.map(|(name, layer, share)| json!({ "name": name, "layer": layer, "share": share })),
        "layers": layers,
    });
    std::fs::write(&out, serde_json::to_string_pretty(&report)?)?;

    // One line a projection at the k = 1 reading, so the probe reads at the terminal too.
    for layer in &layers {
        let mut line = format!("layer {:>2}", layer["layer"].as_u64().unwrap_or(0));
        for projection in layer["projections"].as_array().into_iter().flatten() {
            let arms = projection["arms"].as_array().into_iter().flatten().collect::<Vec<_>>();
            let first = arms.first().map(|a| a["residual_share"].as_f64().unwrap_or(0.0)).unwrap_or(0.0);
            let second = arms.get(1).map(|a| a["residual_share"].as_f64().unwrap_or(0.0)).unwrap_or(0.0);
            let fourth = arms.get(2).map(|a| a["residual_share"].as_f64().unwrap_or(0.0)).unwrap_or(0.0);
            line.push_str(&format!(
                " {} {:>6.2}%/{:>6.2}%/{:>6.2}%",
                projection["part"].as_str().unwrap_or("?"),
                first * 100.0,
                second * 100.0,
                fourth * 100.0
            ));
        }
        println!("{line}");
    }
    Ok(())
}
