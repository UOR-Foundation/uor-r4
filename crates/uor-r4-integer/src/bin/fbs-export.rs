//! Export a SpiralCore/FBS float-trunk snapshot to a `uor-r4.lut-stack/4`
//! `UORLUT01` artifact (d11-exporter-float-trunk, claude-123).
//!
//! The snapshot is the JSON that `uor-r4-float-train`'s `fbs-snapshot` writes
//! from a saved model. The float converter builds it without the serving
//! build, and this serving-build binary turns it into the artifact.
//!
//! ```text
//! fbs-export [--raw-gates] [--raw-transition] SNAPSHOT_JSON OUT_PATH
//! ```
//!
//! `--raw-gates` serves the two Hamilton gates, and `--raw-transition` the
//! recurrence transition, unsnapped: the unit quaternion of each 4-block by
//! integer square root and division, under `uor-r4.lut-stack/5` (claude-181).
//! With neither, the gates and the transition snap to the 120 icosians.
//!
//! It parses the artifact back before writing it, never replaces or leaves a
//! partial `OUT_PATH` (`fbs_snapshot::write_new_file`), and prints its sha256.

use std::path::Path;
use std::process::ExitCode;

use sha2::{Digest, Sha256};
use uor_r4_integer::fbs_snapshot::write_new_file;
use uor_r4_integer::stack::{
    export_fbs_trunk_uorlut01_with, FbsExportSnaps, FbsTrunkSnapshot, IntegerStackModel,
};

const USAGE: &str = "usage: fbs-export [--raw-gates] [--raw-transition] SNAPSHOT_JSON OUT_PATH";

fn run() -> Result<(), String> {
    let mut snaps = FbsExportSnaps::SNAPPED;
    let mut paths = Vec::new();
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--raw-gates" if !snaps.gates => return Err("--raw-gates given twice".to_owned()),
            "--raw-transition" if !snaps.transition => {
                return Err("--raw-transition given twice".to_owned())
            }
            "--raw-gates" => snaps.gates = false,
            "--raw-transition" => snaps.transition = false,
            flag if flag.starts_with("--") => return Err(format!("{flag}: {USAGE}")),
            _ => paths.push(arg),
        }
    }
    let [snapshot, out] = paths.as_slice() else {
        return Err(USAGE.to_owned());
    };
    let bytes = std::fs::read(snapshot).map_err(|e| format!("{snapshot}: {e}"))?;
    let snap: FbsTrunkSnapshot =
        serde_json::from_slice(&bytes).map_err(|e| format!("{snapshot}: {e}"))?;
    let artifact =
        export_fbs_trunk_uorlut01_with(&snap, snaps).map_err(|e| format!("export: {e}"))?;
    IntegerStackModel::parse(&artifact)
        .map_err(|e| format!("the exported artifact does not parse: {e}"))?;
    write_new_file(Path::new(out), &artifact).map_err(|e| format!("{out}: {e}"))?;
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
