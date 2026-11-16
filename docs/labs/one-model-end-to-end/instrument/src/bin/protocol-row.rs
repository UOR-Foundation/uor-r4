//! The protocol row: apply the pre-registered verdict rule to one input.
//!
//! `usage: protocol-row --input IN.json [--out OUT.json]`
//!
//! `IN.json` is a `VerdictInput` (the three strata for the arm, (r0) and the longest-suffix table,
//! plus the reply and novelty shares). The report carries the verdict and every reading it was
//! drawn from. The report is never written over an existing path.

use std::fs;
use std::path::Path;

use one_model_end_to_end_panel::verdict::{verdict, VerdictInput};

fn main() {
    if let Err(error) = run() {
        eprintln!("protocol-row: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut argv = std::env::args().skip(1);
    let (mut input, mut out) = (None, None);
    while let Some(flag) = argv.next() {
        let mut value = || argv.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--input" => input = Some(value()?),
            "--out" => out = Some(value()?),
            other => return Err(format!("unknown flag {other}")),
        }
    }
    let input = input.ok_or("--input is required")?;
    if let Some(out) = &out {
        if Path::new(out).exists() {
            return Err(format!("{out} exists; never written over"));
        }
    }
    let bytes = fs::read(&input).map_err(|e| format!("{input}: {e}"))?;
    let input: VerdictInput =
        serde_json::from_slice(&bytes).map_err(|e| format!("{input}: {e}"))?;
    let report = verdict(&input);
    let text = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    match out {
        Some(out) => fs::write(&out, text).map_err(|e| format!("{out}: {e}"))?,
        None => println!("{text}"),
    }
    Ok(())
}
