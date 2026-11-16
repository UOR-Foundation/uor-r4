//! The FBS stream at the learned `/14` read, as a `Q31ReadDump` for `native_read_train`.
//!
//! `usage: fbs-read-dump --artifact FILE --dump DUMP --out FILE`
//!
//! The input dump supplies the windows' ids (and its `.ask/.other/.bucket` targets, copied beside
//! the output). The artifact's own step supplies each row's `x_in` at the read; the table is the
//! artifact's Q12 writer table. Host only.

use std::fs;
use std::path::PathBuf;

use uor_r4_integer::stack::{read_q31_read_uorlut01, IntegerStackModel};
use uor_r4_native_vector::Q31ReadDump;

fn main() {
    if let Err(e) = run() {
        eprintln!("fbs-read-dump: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut argv = std::env::args().skip(1);
    let (mut artifact, mut dump, mut out) = (None, None, None);
    while let Some(flag) = argv.next() {
        let mut value = || argv.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--artifact" => artifact = Some(PathBuf::from(value()?)),
            "--dump" => dump = Some(PathBuf::from(value()?)),
            "--out" => out = Some(PathBuf::from(value()?)),
            other => return Err(format!("unknown flag {other}")),
        }
    }
    let (artifact, dump, out) = (
        artifact.ok_or("--artifact is required")?,
        dump.ok_or("--dump is required")?,
        out.ok_or("--out is required")?,
    );
    if out.exists() {
        return Err(format!("{} exists; never written over", out.display()));
    }
    let bytes = fs::read(&artifact).map_err(|e| format!("{}: {e}", artifact.display()))?;
    let input = Q31ReadDump::from_bytes(&fs::read(&dump).map_err(|e| format!("{}: {e}", dump.display()))?)
        .map_err(|e| e.to_string())?;
    let model = IntegerStackModel::parse(&bytes).map_err(|e| e.to_string())?;
    let read = read_q31_read_uorlut01(&bytes)
        .map_err(|e| e.to_string())?
        .ok_or("the artifact carries no /14 read")?;
    let width = read.read().width();
    let table = model
        .fbs_writer_table()
        .ok_or("the artifact is not an FBS stack")?;
    // The artifact's own stream at the read, one row per token.
    let stream = model
        .q31_read_rows(&input.ids, input.time)
        .map_err(|e| e.to_string())?;
    if stream.len() != input.ids.len() * width {
        return Err("the read rows are not the read's width".to_owned());
    }
    let fbs = Q31ReadDump {
        read: read.read().clone(),
        batch: input.batch,
        time: input.time,
        stream,
        ids: input.ids.clone(),
        table: table.iter().map(|&v| i64::from(v)).collect(),
    };
    fs::write(&out, fbs.to_bytes().map_err(|e| e.to_string())?)
        .map_err(|e| format!("{}: {e}", out.display()))?;
    for ext in ["ask.u32", "other.u32", "bucket.u8"] {
        let from = PathBuf::from(format!("{}.{ext}", dump.display()));
        let to = PathBuf::from(format!("{}.{ext}", out.display()));
        fs::copy(&from, &to).map_err(|e| format!("{} -> {}: {e}", from.display(), to.display()))?;
    }
    Ok(())
}
