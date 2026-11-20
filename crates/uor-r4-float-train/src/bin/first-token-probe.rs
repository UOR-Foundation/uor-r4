//! `first-token-probe` (20 Nov 2026): does the trunk condition on the prompt?
//!
//! The collapsed generation ("3 distinct replies for 16 prompts") has two possible causes, and they
//! need different fixes. This probe separates them on an existing checkpoint, with no training:
//!
//! * **the trunk ignores the prompt**: the whole-table top-k at the first reply position is the same
//!   for every prompt. Then it is a capacity/objective problem and no readout change helps.
//! * **the writer's pick is constant**: the trunk's top-k differs per prompt but the writer's kept
//!   cells (what serving uses) select one token regardless. Then it is a readout defect and a cheap
//!   fix.
//!
//! It prints, for several distinct prompts, the target token, the trunk's top-k (whole table), and
//! the writer's pick. Integer path, deterministic.
//!
//! Usage: `first-token-probe --out DIR [--prompts 8]` with the same data flags as `train-in-space`.

use std::process::ExitCode;

use uor_r4_float_train::in_space_long::{first_token_probe, set_dev_override, set_dev_tokens_cap};
use uor_r4_float_train::native_vector_writer::set_writer_centroids;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> std::result::Result<(), String> {
    let mut prompts = 8usize;
    let mut centroid = true;
    let mut checkpoint = None;
    let mut dev = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--prompts" => prompts = args.next().and_then(|v| v.parse().ok()).ok_or("--prompts wants a number")?,
            "--writer-address" => {
                centroid = match args.next().as_deref() {
                    Some("centroids") => true,
                    Some("ports") => false,
                    _ => return Err("--writer-address wants ports or centroids".into()),
                }
            }
            "--init-from" => checkpoint = args.next(),
            "--dev" => dev = args.next(),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    let checkpoint = checkpoint.ok_or("--init-from CHECKPOINT is required")?;
    let dev = dev.ok_or("--dev DIR is required")?;

    set_dev_override(Some(std::path::PathBuf::from(&dev)));
    set_dev_tokens_cap(Some(200_000));
    set_writer_centroids(centroid);
    let reading = first_token_probe(std::path::Path::new(&checkpoint), prompts).map_err(|e| e.to_string())?;
    println!("{reading}");
    Ok(())
}
