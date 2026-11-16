//! The read cost ledger over two pick files (ADR-0007 §2 rule 3).
//!
//! `usage: read-ledger --reference F --served F [--nanos N] --out F`
//!
//! Both files are little-endian `u32` pick arrays of the same length, one row a word, `0` for a
//! row the read chose nothing on — the form `native_read_train --save-picks` writes and a served
//! session produces. `--targets F` scores only the rows whose target is non-zero (the probe's
//! `.ask.u32`), and `--nanos` adds rows a second.

use std::fs;
use std::path::Path;

use uor_r4_integer::stack::ReadLedger;

fn main() {
    if let Err(e) = run() {
        eprintln!("read-ledger: {e}");
        std::process::exit(1);
    }
}

fn u32s(path: &Path) -> Result<Vec<u32>, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !bytes.len().is_multiple_of(4) {
        return Err(format!("{}: not a u32 array", path.display()));
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect())
}

fn run() -> Result<(), String> {
    let mut argv = std::env::args().skip(1);
    let (mut reference, mut served, mut targets, mut nanos, mut out) =
        (None, None, None, None, None);
    while let Some(flag) = argv.next() {
        let mut value = || argv.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--reference" => reference = Some(value()?),
            "--served" => served = Some(value()?),
            "--targets" => targets = Some(value()?),
            "--nanos" => {
                nanos = Some(
                    value()?
                        .parse::<u64>()
                        .map_err(|e| format!("--nanos: {e}"))?,
                )
            }
            "--out" => out = Some(value()?),
            other => return Err(format!("unknown flag {other}")),
        }
    }
    let reference = reference.ok_or("--reference is required")?;
    let served = served.ok_or("--served is required")?;
    let out = out.ok_or("--out is required")?;
    if Path::new(&out).exists() {
        return Err(format!("{out} exists; never written over"));
    }
    let reference = u32s(Path::new(&reference))?;
    let served = u32s(Path::new(&served))?;
    let ledger = match &targets {
        None => ReadLedger::compare(&reference, &served).map_err(|e| e.to_string())?,
        Some(path) => {
            let targets = u32s(Path::new(path))?;
            let scored: Vec<bool> = targets.iter().map(|&target| target != 0).collect();
            (ReadLedger::compare_on(&reference, &served, &scored)).map_err(|e| e.to_string())?
        }
    };
    let bytes = serde_json::to_vec_pretty(&ledger.to_json(nanos)).map_err(|e| e.to_string())?;
    fs::write(&out, bytes).map_err(|e| format!("{out}: {e}"))
}
