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
//!
//! ```text
//! fbs-export --point-key ARTIFACT TOKENS_U16 RESPONSE_MASK_U8 OUT_PATH
//! ```
//!
//! adds the served point key (claude-222, `point-key-integer`) to a turn-memory
//! `uor-r4.lut-stack/7` artifact, as `/8`. The table holds TRAIN's openings, from
//! a `UORT` token file and its response mask, in the float sheet's 192-token
//! windows. At each opening is the integer trunk's own Q12 query, read with the
//! memory off, and its target. It also prints the table's rows.

use std::path::Path;
use std::process::ExitCode;

use sha2::{Digest, Sha256};
use uor_r4_integer::fbs_snapshot::write_new_file;
use uor_r4_integer::stack::{
    add_point_key_uorlut01, build_point_key, export_fbs_trunk_uorlut01_with, point_key_windows,
    FbsExportSnaps, FbsTrunkSnapshot, IntegerStackModel, POINT_KEY_BOS, POINT_KEY_WINDOW,
};

const USAGE: &str = "usage: fbs-export [--raw-gates] [--raw-transition] SNAPSHOT_JSON OUT_PATH\n       fbs-export --point-key ARTIFACT TOKENS_U16 RESPONSE_MASK_U8 OUT_PATH";

/// A `UORT` token file's ids: its 64-byte header, whose count must match, then little-endian u16s.
fn uort_tokens(path: &str) -> Result<Vec<u32>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    if bytes.len() < 64 || &bytes[..4] != b"UORT" {
        return Err(format!("{path}: not a UORT token file"));
    }
    let count = u64::from_le_bytes(bytes[8..16].try_into().map_err(|_| format!("{path}: truncated"))?) as usize;
    let payload = &bytes[64..];
    if payload.len() != count * 2 {
        return Err(format!("{path}: the header counts {count} tokens, the payload holds {} bytes", payload.len()));
    }
    Ok(payload.chunks_exact(2).map(|w| u32::from(u16::from_le_bytes([w[0], w[1]]))).collect())
}

/// `--point-key`: the `/8` stack of a `/7` one, with TRAIN's point table.
fn point_key(paths: &[String]) -> Result<(), String> {
    let [artifact, tokens, mask, out] = paths else {
        return Err(USAGE.to_owned());
    };
    let bytes = std::fs::read(artifact).map_err(|e| format!("{artifact}: {e}"))?;
    let model = IntegerStackModel::parse(&bytes).map_err(|e| format!("{artifact}: {e}"))?;
    let tokens = uort_tokens(tokens)?;
    let mask = std::fs::read(mask).map_err(|e| format!("{mask}: {e}"))?;
    let windows = point_key_windows(&tokens, &mask, POINT_KEY_WINDOW, POINT_KEY_BOS).map_err(|e| format!("windows: {e}"))?;
    let image = build_point_key(&model, &windows).map_err(|e| format!("the table: {e}"))?;
    let rows = image.rows();
    let with = add_point_key_uorlut01(&bytes, &image).map_err(|e| format!("the /8 stack: {e}"))?;
    IntegerStackModel::parse(&with).map_err(|e| format!("the /8 stack does not parse: {e}"))?;
    write_new_file(Path::new(out), &with).map_err(|e| format!("{out}: {e}"))?;
    println!("{}  {out}  {rows} openings in {} windows", hex::encode(Sha256::digest(&with)), windows.len());
    Ok(())
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--point-key") {
        return point_key(&args[1..]);
    }
    let mut snaps = FbsExportSnaps::SNAPPED;
    let mut paths = Vec::new();
    for arg in args {
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
