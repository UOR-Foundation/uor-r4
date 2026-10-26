//! Export a SpiralCore/FBS float-trunk snapshot to a `uor-r4.lut-stack/4`
//! `UORLUT01` artifact (d11-exporter-float-trunk, claude-123).
//!
//! The snapshot is the JSON that `uor-r4-float-train`'s `fbs-snapshot` writes
//! from a saved model. The float converter builds it without the serving
//! build, and this serving-build binary turns it into the artifact.
//!
//! ```text
//! fbs-export SNAPSHOT_JSON OUT_PATH
//! ```
//!
//! It never overwrites `OUT_PATH`. It parses the artifact back before writing
//! it, and prints its sha256.

use std::io::Write;
use std::process::ExitCode;

use sha2::{Digest, Sha256};
use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, FbsTrunkSnapshot, IntegerStackModel};

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [snapshot, out] = args.as_slice() else {
        return Err("usage: fbs-export SNAPSHOT_JSON OUT_PATH".to_owned());
    };
    let bytes = std::fs::read(snapshot).map_err(|e| format!("{snapshot}: {e}"))?;
    let snap: FbsTrunkSnapshot =
        serde_json::from_slice(&bytes).map_err(|e| format!("{snapshot}: {e}"))?;
    let artifact = export_fbs_trunk_uorlut01(&snap).map_err(|e| format!("export: {e}"))?;
    IntegerStackModel::parse(&artifact)
        .map_err(|e| format!("the exported artifact does not parse: {e}"))?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out)
        .map_err(|e| format!("{out}: {e}"))?;
    file.write_all(&artifact)
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("{out}: {e}"))?;
    println!("{}  {out}", hex::encode(Sha256::digest(&artifact)));
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("fbs-export: {e}");
            ExitCode::FAILURE
        }
    }
}
