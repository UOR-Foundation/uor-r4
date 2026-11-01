//! Convert a saved SpiralCore/FBS float trunk (`StackModel::save`) to the
//! integer crate's snapshot JSON (d11-exporter-float-trunk, claude-123).
//!
//! It runs on one GPU, because the routes are read back through the model's
//! CUDA projection (`fbs_export`). The serving build's `fbs-export` turns the
//! JSON into the `UORLUT01` artifact. The optional readouts, for the serving
//! build's `fbs-compare`, are the float trunk's own outputs, in order:
//! - `READOUT_JSON`: plain;
//! - `EXPORT_READOUT_JSON`: in the float sheets' D11 export mode;
//! - `D11_READOUT_JSON`: in the D11 mode, after rounding the loaded trunk's
//!   tables as exported (claude-125).
//!
//! ```text
//! fbs-snapshot MODEL_DIR OUT_JSON [READOUT_JSON [EXPORT_READOUT_JSON [D11_READOUT_JSON]]]
//! fbs-snapshot --hamming-trace ARM MODEL_DIR D11_READOUT_JSON TRACE_JSON
//! ```
//!
//! The second form (claude-235) writes the D11 readout of a checkpoint with
//! Hamming reads and what each read chose on its fixed sequences
//! (`uor-r4.hamming-trace/1`), for one arm:
//! - `d11`: as the first form's D11 readout;
//! - `d11-tie`: with the integer session's tie rule;
//! - `d11-rounded`: with the reads' parameters rounded as exported;
//! - `d11-tie-rounded`: both.
//!
//! It never replaces or leaves a partial output file, and it prints each
//! file's sha256.
//!
//! The snapshot carries every read the checkpoint holds, or it is refused
//! (claude-198): the turn memory (`turn_memory*.safetensors`) and the Hamming
//! reads (`hamming_read.safetensors`, claude-200) are carried, and any other
//! read file (the Flock, content and opening reads, the prompt register) is
//! refused by name until its export exists.

use std::path::Path;

use candle_core::Device;
use sha2::{Digest, Sha256};
use uor_r4_float_train::fbs_export::{
    check_checkpoint_reads, check_readout_reads, fbs_float_readout, fbs_hamming_snapshot,
    fbs_memory_snapshot, fbs_trunk_snapshot, round_hamming_as_exported, round_tables_as_exported,
    run_rate_reading, READOUT_SEQUENCES,
};
use uor_r4_float_train::flock_read_hamming::{
    set_hamming_reads, set_hamming_tie_rule, set_hamming_trace, take_hamming_trace, HammingReads,
    HAMMING_DEPTHS, HAMMING_KEPT,
};
use uor_r4_float_train::flock_read_memory::TurnMemory;
use uor_r4_float_train::geometric_stack::{set_retention_rates, StackModel};
use uor_r4_float_train::hamilton_sheet::SHEET_CONTEXT;
use uor_r4_float_train::{FloatTrainError, Result};
use uor_r4_integer::fbs_snapshot::{
    run_readout_nearest, write_new_file, FbsReadoutMode, HammingTrace, HAMMING_TRACE_SCHEMA,
};

fn write(path: &str, json: &[u8]) -> Result<()> {
    write_new_file(Path::new(path), json)?;
    println!("{}  {path}", hex::encode(Sha256::digest(json)));
    Ok(())
}

/// The run's `config.json`, two levels above `MODEL_DIR` (`models/step-N`).
fn run_config(dir: &Path) -> Result<String> {
    let path = dir
        .parent()
        .and_then(Path::parent)
        .map(|run| run.join("config.json"))
        .ok_or_else(|| {
            FloatTrainError::Invalid("MODEL_DIR is not a run's models/step-N".to_owned())
        })?;
    Ok(std::fs::read_to_string(path)?)
}

/// A trace arm's switches: the tie rule, and the reads rounded as exported.
fn trace_arm(name: &str) -> Option<(bool, bool)> {
    match name {
        "d11" => Some((false, false)),
        "d11-tie" => Some((true, false)),
        "d11-rounded" => Some((false, true)),
        "d11-tie-rounded" => Some((true, true)),
        _ => None,
    }
}

/// `--hamming-trace ARM MODEL_DIR D11_READOUT_JSON TRACE_JSON` (claude-235).
fn hamming_trace(arm: &str, model_dir: &str, readout: &str, trace: &str) -> Result<()> {
    let (tie, rounded) = trace_arm(arm).ok_or_else(|| {
        FloatTrainError::Invalid(format!(
            "unknown trace arm {arm}: d11, d11-tie, d11-rounded or d11-tie-rounded"
        ))
    })?;
    let dir = Path::new(model_dir);
    check_checkpoint_reads(dir)?;
    check_readout_reads(dir, 1)?;
    set_retention_rates(run_rate_reading(&run_config(dir)?)?);
    let device = Device::new_cuda(0)?;
    let model = StackModel::load(dir, &device)?;
    let reads = HammingReads::load(dir, &device)?.ok_or_else(|| {
        FloatTrainError::Invalid("the checkpoint has no Hamming reads to trace".to_owned())
    })?;
    if rounded {
        round_hamming_as_exported(&reads)?;
    }
    set_hamming_reads(Some(reads));
    set_hamming_tie_rule(tie);
    round_tables_as_exported(&model)?;
    set_hamming_trace(true);
    let d11 = fbs_float_readout(&model, FbsReadoutMode::D11)?;
    let windows = take_hamming_trace().unwrap_or_default();
    set_hamming_trace(false);
    // Only the fixed sequences reach a read (a single token has no earlier position): each must
    // be traced once, at every read depth.
    let depths: Vec<Vec<usize>> = windows
        .iter()
        .map(|w| w.reads.iter().map(|r| r.depth).collect())
        .collect();
    if windows.len() != READOUT_SEQUENCES || depths.iter().any(|d| d[..] != HAMMING_DEPTHS[..]) {
        return Err(FloatTrainError::Invalid(format!(
            "the trace holds {} windows with depths {depths:?}, not {READOUT_SEQUENCES} at {HAMMING_DEPTHS:?}",
            windows.len()
        )));
    }
    let heads = windows[0].reads[0].heads.len();
    let trace_json = serde_json::to_vec(&HammingTrace {
        schema: HAMMING_TRACE_SCHEMA.to_owned(),
        side: "float-d11".to_owned(),
        arm: arm.to_owned(),
        heads,
        kept: HAMMING_KEPT,
        windows,
    })?;
    write(readout, &serde_json::to_vec(&d11)?)?;
    write(trace, &trace_json)
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let [flag, arm, model_dir, readout, trace] = args.as_slice() {
        if flag == "--hamming-trace" {
            return hamming_trace(arm, model_dir, readout, trace);
        }
    }
    let (model_dir, out, readouts) = match args.as_slice() {
        [model_dir, out, readouts @ ..] if readouts.len() <= 3 => (model_dir, out, readouts),
        _ => {
            eprintln!(
                "usage: fbs-snapshot MODEL_DIR OUT_JSON [READOUT_JSON [EXPORT_READOUT_JSON [D11_READOUT_JSON]]]\n       fbs-snapshot --hamming-trace ARM MODEL_DIR D11_READOUT_JSON TRACE_JSON"
            );
            std::process::exit(2);
        }
    };
    let dir = Path::new(model_dir);
    // Every read the checkpoint holds is carried, or the export is refused (claude-198), before
    // the GPU is opened.
    check_checkpoint_reads(dir)?;
    check_readout_reads(dir, readouts.len())?;
    // The run's rate reading (integer-retention-rates, claude-226), from its `config.json` two
    // levels above `models/step-N`. The snapshot carries it, and the float readouts below are
    // taken under it, as the run trained and scored.
    let config = run_config(dir)?;
    let retain = run_rate_reading(&config)?;
    // And its readout (integer-nearest-readout, claude-231): a cell or nearest objective decodes
    // as the whole table's nearest token.
    let nearest = run_readout_nearest(&config).map_err(FloatTrainError::Invalid)?;
    set_retention_rates(retain);
    let device = Device::new_cuda(0)?;
    let model = StackModel::load(dir, &device)?;
    let mut snapshot = fbs_trunk_snapshot(&model)?;
    snapshot.memory = TurnMemory::load(dir, &device)?
        .map(|memory| fbs_memory_snapshot(&memory, SHEET_CONTEXT))
        .transpose()?;
    let hamming = HammingReads::load(dir, &Device::Cpu)?
        .map(|reads| fbs_hamming_snapshot(&reads, snapshot.layers.len(), snapshot.width))
        .transpose()?;
    snapshot.hamming = hamming;
    snapshot.retain_rates = retain;
    snapshot.readout_nearest = nearest;
    write(out, &serde_json::to_vec(&snapshot)?)?;
    // The readouts are the float model with its Hamming reads (claude-200): `StackModel::hidden`
    // applies the reads set on this thread, here on the trunk's GPU.
    set_hamming_reads(HammingReads::load(dir, &device)?);
    for (path, mode) in readouts.iter().zip([
        FbsReadoutMode::Plain,
        FbsReadoutMode::Export,
        FbsReadoutMode::D11,
    ]) {
        if mode == FbsReadoutMode::D11 {
            round_tables_as_exported(&model)?;
        }
        write(
            path,
            &serde_json::to_vec(&fbs_float_readout(&model, mode)?)?,
        )?;
    }
    Ok(())
}
