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
//! fbs-snapshot [--dev DATA_ROOT] MODEL_DIR OUT_JSON [READOUT_JSON [EXPORT_READOUT_JSON [D11_READOUT_JSON]]]
//! fbs-snapshot [--dev DATA_ROOT] --hamming-trace ARM MODEL_DIR D11_READOUT_JSON TRACE_JSON
//! ```
//!
//! `--dev DATA_ROOT` (claude-240) takes every readout, and the trace, on a sample of DEV in
//! place of the fixed sequences: `DATA_ROOT/ft/dev`, loaded with its pins, cut into the
//! trainer's windows of `SHEET_CONTEXT` and sampled every `PARITY_WINDOW_STRIDE`-th window, as
//! claude-126 sampled it. The snapshot itself does not depend on it.
//!
//! The second form (claude-235) writes the D11 readout of a checkpoint with
//! Hamming reads and what each read chose on its fixed sequences
//! (`uor-r4.hamming-trace/1`), for one arm:
//! - `d11`: with the integer session's tie rule turned off, so the GPU sort's own order breaks
//!   the reads' ties (the first form's D11 readout before claude-239);
//! - `d11-tie`: with the rule, as the first form's D11 readout since claude-239, the rule's
//!   default;
//! - `d11-rounded`: `d11` with the reads' parameters rounded as exported;
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

use std::path::{Path, PathBuf};

use candle_core::Device;
use sha2::{Digest, Sha256};
use uor_r4_float_train::fbs_export::{
    check_checkpoint_reads, check_readout_reads, fbs_float_readout, fbs_float_readout_on,
    fbs_hamming_snapshot, fbs_memory_snapshot, fbs_trunk_snapshot, round_hamming_as_exported,
    round_tables_as_exported, run_rate_reading, READOUT_SEQUENCES,
};
use uor_r4_float_train::flock_read_hamming::{
    set_hamming_reads, set_hamming_tie_rule, set_hamming_trace, take_hamming_trace, HammingReads,
    HAMMING_DEPTHS, HAMMING_KEPT,
};
use uor_r4_float_train::flock_read_memory::TurnMemory;
use uor_r4_float_train::geometric_stack::{set_retention_rates, StackModel};
use uor_r4_float_train::hamilton_sheet::{
    build_document_windows, load_pinned_uort_split, CausalWindow, EXPECTED_DEV_ELIGIBLE_TARGETS,
    EXPECTED_DEV_TOKENS, PINNED_DEV_MASK_SHA256, PINNED_DEV_TOKENS_SHA256, SHEET_BOS_ID,
    SHEET_CONTEXT, SHEET_VOCAB,
};
use uor_r4_float_train::spiralcore_fbs_sheet::PARITY_WINDOW_STRIDE;
use uor_r4_float_train::{FloatTrainError, Result};
use uor_r4_integer::fbs_snapshot::{
    run_readout_nearest, write_new_file, FbsFloatReadout, FbsReadoutMode, HammingTrace,
    HAMMING_TRACE_SCHEMA,
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

/// Where an output that does not exist yet would be: its folder, resolved, and its name.
fn output_place(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);
    let folder = path
        .parent()
        .filter(|folder| !folder.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = path
        .file_name()
        .ok_or_else(|| FloatTrainError::Invalid(format!("{} names no file", path.display())))?;
    Ok(std::fs::canonicalize(folder)?.join(name))
}

/// claude-240: every `PARITY_WINDOW_STRIDE`-th window, from the first (claude-126's rule).
fn sample_windows<T>(windows: Vec<T>) -> Vec<T> {
    windows.into_iter().step_by(PARITY_WINDOW_STRIDE).collect()
}

/// claude-240: `--dev DATA_ROOT`'s sample. DEV is loaded with its pins and cut into the
/// trainer's windows (`in_space_long.rs`), and every `PARITY_WINDOW_STRIDE`-th is kept. It is
/// read before the GPU opens, so a wrong data root costs no GPU time.
fn dev_sample(data_root: &str) -> Result<Vec<Vec<u32>>> {
    let dev = Path::new(data_root).join("ft/dev");
    let (tokens, mask) = load_pinned_uort_split(
        &dev.join("tokens.u16"),
        &dev.join("response_mask.u8"),
        PINNED_DEV_TOKENS_SHA256,
        PINNED_DEV_MASK_SHA256,
        EXPECTED_DEV_TOKENS,
        EXPECTED_DEV_ELIGIBLE_TARGETS,
        SHEET_VOCAB,
        SHEET_BOS_ID,
    )?;
    let (windows, _) = build_document_windows(&tokens, &mask, SHEET_CONTEXT, SHEET_BOS_ID)?;
    let scored: usize = windows.iter().map(|w| w.scored_targets.len()).sum();
    if scored != EXPECTED_DEV_ELIGIBLE_TARGETS {
        return Err(FloatTrainError::Invalid(format!(
            "DEV's windows score {scored} targets, not {EXPECTED_DEV_ELIGIBLE_TARGETS}"
        )));
    }
    let all = windows.len();
    let sample: Vec<Vec<u32>> = sample_windows(windows).iter().map(scored_prefix).collect();
    let positions: usize = sample.iter().map(Vec::len).sum();
    println!(
        "dev: {all} windows, {} sampled, {positions} positions",
        sample.len()
    );
    Ok(sample)
}

/// A window's inputs up to its last scored position. The trainer pads each window with BOS to
/// `SHEET_CONTEXT`, and the model is causal, so the cut changes no scored prediction and keeps the
/// padding out of the parity counts. `build_document_windows` keeps only windows that score.
fn scored_prefix(window: &CausalWindow) -> Vec<u32> {
    let last = window.scored_offsets.iter().max().copied().unwrap_or(0);
    window.input_ids[..=last].to_vec()
}

/// A readout on the DEV sample when there is one, else on the fixed sequences. The trainer cut
/// DEV at `SHEET_CONTEXT`, so a model with another context, or a window past it, is refused.
fn readout_on(
    model: &StackModel,
    mode: FbsReadoutMode,
    dev: Option<&[Vec<u32>]>,
) -> Result<FbsFloatReadout> {
    let Some(sample) = dev else {
        return fbs_float_readout(model, mode);
    };
    let context = model.config.context;
    if context != SHEET_CONTEXT || sample.iter().any(|w| w.is_empty() || w.len() > context) {
        return Err(FloatTrainError::Invalid(format!(
            "the model's context is {context}, and DEV was cut at {SHEET_CONTEXT} into windows of 1 to {SHEET_CONTEXT}"
        )));
    }
    fbs_float_readout_on(model, mode, sample)
}

/// `[--dev DATA_ROOT] --hamming-trace ARM MODEL_DIR D11_READOUT_JSON TRACE_JSON` (claude-235;
/// the DEV sample, claude-240).
fn hamming_trace(
    arm: &str,
    model_dir: &str,
    readout: &str,
    trace: &str,
    dev: Option<&[Vec<u32>]>,
) -> Result<()> {
    let (tie, rounded) = trace_arm(arm).ok_or_else(|| {
        FloatTrainError::Invalid(format!(
            "unknown trace arm {arm}: d11, d11-tie, d11-rounded or d11-tie-rounded"
        ))
    })?;
    // Both outputs are refused before the GPU runs, so a run never leaves a readout without its
    // trace. They are compared by where they would be, so `./t.json` and `t.json` are one file.
    if output_place(readout)? == output_place(trace)? {
        return Err(FloatTrainError::Invalid(format!(
            "D11_READOUT_JSON and TRACE_JSON are both {readout}"
        )));
    }
    if let Some(path) = [readout, trace].into_iter().find(|p| Path::new(p).exists()) {
        return Err(FloatTrainError::Invalid(format!(
            "{path} exists, and the trace never replaces a file"
        )));
    }
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
    let d11 = readout_on(&model, FbsReadoutMode::D11, dev)?;
    let windows = take_hamming_trace().unwrap_or_default();
    set_hamming_trace(false);
    // Only the sequences reach a read (a single token has no earlier position): each must be
    // traced once, at every read depth.
    let sequences = dev.map_or(READOUT_SEQUENCES, <[Vec<u32>]>::len);
    let depths: Vec<Vec<usize>> = windows
        .iter()
        .map(|w| w.reads.iter().map(|r| r.depth).collect())
        .collect();
    if windows.len() != sequences || depths.iter().any(|d| d[..] != HAMMING_DEPTHS[..]) {
        return Err(FloatTrainError::Invalid(format!(
            "the trace holds {} windows with depths {depths:?}, not {sequences} at {HAMMING_DEPTHS:?}",
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

fn usage() -> ! {
    eprintln!(
        "usage: fbs-snapshot [--dev DATA_ROOT] MODEL_DIR OUT_JSON [READOUT_JSON [EXPORT_READOUT_JSON [D11_READOUT_JSON]]]\n       fbs-snapshot [--dev DATA_ROOT] --hamming-trace ARM MODEL_DIR D11_READOUT_JSON TRACE_JSON"
    );
    std::process::exit(2);
}

fn main() -> Result<()> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let dev = if args.first().is_some_and(|flag| flag == "--dev") {
        if args.len() < 2 {
            usage();
        }
        let root = args.drain(..2).nth(1).unwrap_or_default();
        Some(dev_sample(&root)?)
    } else {
        None
    };
    let dev = dev.as_deref();
    if args.first().is_some_and(|flag| flag == "--hamming-trace") {
        let [_, arm, model_dir, readout, trace] = args.as_slice() else {
            usage();
        };
        return hamming_trace(arm, model_dir, readout, trace, dev);
    }
    let (model_dir, out, readouts) = match args.as_slice() {
        [model_dir, out, readouts @ ..] if readouts.len() <= 3 => (model_dir, out, readouts),
        _ => usage(),
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
        write(path, &serde_json::to_vec(&readout_on(&model, mode, dev)?)?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{output_place, sample_windows, scored_prefix, CausalWindow, PARITY_WINDOW_STRIDE};

    /// claude-240: a DEV window is cut after its last scored position, so its BOS padding never
    /// reaches the parity counts, and every scored position and its prefix stay.
    #[test]
    fn a_dev_window_is_cut_after_its_last_scored_position() {
        let window = CausalWindow {
            doc_id: 0,
            input_ids: vec![0, 5, 6, 7, 8, 0, 0, 0],
            scored_offsets: vec![1, 3],
            scored_targets: vec![6, 8],
            scored_corpus_positions: vec![2, 4],
        };
        assert_eq!(scored_prefix(&window), vec![0, 5, 6, 7]);
    }

    /// claude-240: the DEV sample is claude-126's, every 8th window from the first, in order.
    #[test]
    fn the_dev_sample_keeps_every_eighth_window_from_the_first() {
        assert_eq!(PARITY_WINDOW_STRIDE, 8);
        let windows: Vec<Vec<u32>> = (0..17u32).map(|w| vec![w; 3]).collect();
        assert_eq!(
            sample_windows(windows),
            vec![vec![0; 3], vec![8; 3], vec![16; 3]]
        );
        // DEV's 428 windows give 54.
        assert_eq!(sample_windows(vec![0u8; 428]).len(), 54);
    }

    /// claude-235: two spellings of one output are one place, so the trace refuses them.
    #[test]
    fn two_spellings_of_one_output_are_one_place() {
        assert_eq!(
            output_place("./trace.json").unwrap(),
            output_place("trace.json").unwrap()
        );
        assert_ne!(
            output_place("readout.json").unwrap(),
            output_place("trace.json").unwrap()
        );
        assert!(output_place("..").is_err(), "names no file");
    }
}
