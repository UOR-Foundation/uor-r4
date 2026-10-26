//! Convert a saved SpiralCore/FBS float trunk (`StackModel::save`) to the
//! integer crate's snapshot JSON (d11-exporter-float-trunk, claude-123).
//!
//! It runs on one GPU, because the routes are read back through the model's
//! CUDA projection (`fbs_export`). The serving build's `fbs-export` turns the
//! JSON into the `UORLUT01` artifact.
//!
//! ```text
//! fbs-snapshot MODEL_DIR OUT_JSON
//! ```
//!
//! It never overwrites `OUT_JSON`, and it prints the JSON's sha256.

use std::io::Write;
use std::path::Path;

use candle_core::Device;
use sha2::{Digest, Sha256};
use uor_r4_float_train::fbs_export::fbs_trunk_snapshot;
use uor_r4_float_train::geometric_stack::StackModel;
use uor_r4_float_train::Result;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [model_dir, out] = args.as_slice() else {
        eprintln!("usage: fbs-snapshot MODEL_DIR OUT_JSON");
        std::process::exit(2);
    };
    let device = Device::new_cuda(0)?;
    let model = StackModel::load(Path::new(model_dir), &device)?;
    let json = serde_json::to_vec(&fbs_trunk_snapshot(&model)?)?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out)?;
    file.write_all(&json)?;
    file.sync_all()?;
    println!("{}  {out}", hex::encode(Sha256::digest(&json)));
    Ok(())
}
