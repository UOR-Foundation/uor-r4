//! The snap parity report of a snapped model directory: the D11 engine's
//! root selections and logits against the float snapped forward, on evenly
//! spaced windows of given tokens.
//!
//! `stack-snap-parity model=DIR tokens=FILE out=DIR [windows=N]`

use std::fs;
use std::path::PathBuf;

use uor_r4_core::report_output;
use uor_r4_training::stack_snap_parity::snap_parity_report;
use uor_r4_training::{Result, TrainingError};

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

fn main() -> Result<()> {
    let mut model = None;
    let mut tokens = None;
    let mut out = None;
    let mut windows = 8usize;
    for argument in std::env::args().skip(1) {
        let (key, value) = argument
            .split_once('=')
            .ok_or_else(|| invalid(format!("arguments are key=value, got {argument}")))?;
        match key {
            "model" => model = Some(PathBuf::from(value)),
            "tokens" => tokens = Some(PathBuf::from(value)),
            "out" => out = Some(PathBuf::from(value)),
            "windows" => {
                windows = value
                    .parse()
                    .map_err(|_| invalid(format!("windows must be a count, got {value}")))?
            }
            _ => return Err(invalid(format!("unknown argument {key}="))),
        }
    }
    let model = model.ok_or_else(|| invalid("model= is required"))?;
    let tokens = tokens.ok_or_else(|| invalid("tokens= is required"))?;
    let out = out.ok_or_else(|| invalid("out= is required"))?;
    report_output::claim(&out)?;
    let bytes = fs::read(&tokens)?;
    if bytes.len() % 4 != 0 {
        return Err(invalid("the tokens file is little-endian u32"));
    }
    let tokens: Vec<u32> = bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes(c.try_into().expect("chunk")))
        .collect();
    let report = snap_parity_report(&model, &tokens, windows)?;
    fs::write(
        out.join("snap-parity.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}
