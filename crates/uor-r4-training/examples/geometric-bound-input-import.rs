//! Preserve authenticated portable input bytes under a fresh import seal.
//! This creates import provenance, never a replacement historical experiment seal.
use serde_json::{json, Value};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use uor_r4_core::report_output;
use uor_r4_training::sha256_file;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn bad(s: &str) -> Box<dyn std::error::Error> {
    io::Error::new(io::ErrorKind::InvalidData, s).into()
}
fn copy_bound(src: &Path, dst: &Path, original: &Path, bound: &Value) -> Result<()> {
    let meta = fs::symlink_metadata(src)?;
    if meta.file_type().is_symlink() {
        return Err(bad("symlink input refused"));
    }
    if meta.is_dir() {
        fs::create_dir(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            copy_bound(&entry.path(), &dst.join(entry.file_name()), original, bound)?;
        }
    } else if meta.is_file() {
        let relative = src.strip_prefix(original)?;
        let key = format!(
            "/{}",
            relative.to_str().ok_or_else(|| bad("nonUTF8 input"))?
        );
        let identity = &bound["files"][&key];
        let expected = identity["sha256"]
            .as_str()
            .ok_or_else(|| bad("input absent portable binding"))?;
        if identity["bytes"].as_u64() != Some(meta.len()) || sha256_file(src)? != expected {
            return Err(bad("portable input SHA or length mismatch"));
        }
        fs::copy(src, dst)?;
        if sha256_file(dst)? != expected {
            return Err(bad("import copy SHA mismatch"));
        }
    } else {
        return Err(bad("nonregular input refused"));
    }
    Ok(())
}
fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 4 {
        return Err(bad(
            "requires OLD_CONFIG PORTABLE_ROOT NEW_IMPORT_ROOT NEW_MODEL_REPORT",
        ));
    }
    let old = PathBuf::from(&args[0]);
    let original = fs::canonicalize(&args[1])?;
    let out = PathBuf::from(&args[2]);
    if out.starts_with(&original) || original.starts_with(&out) {
        return Err(bad("import/input overlap"));
    }
    report_output::claim(&out)?;
    let bytes = fs::read(&old)?;
    let mut config: Value = serde_json::from_slice(&bytes)?;
    let bound_path = original.join("portable-bound-files.json");
    let bound: Value = serde_json::from_slice(&fs::read(&bound_path)?)?;
    for key in [
        "source_weights",
        "native_artifact",
        "trusted_binding",
        "cue_bundle",
        "prefix_bundle",
        "canonical_exp",
        "training_inputs",
        "training_labels",
        "development_inputs",
        "development_labels",
    ] {
        let src = PathBuf::from(
            config[key]
                .as_str()
                .ok_or_else(|| bad("config input absent"))?,
        );
        let dst = out.join(key);
        copy_bound(&src, &dst, &original, &bound)?;
        config[key] = json!(dst);
    }
    config["out"] = json!(&args[3]);
    fs::write(out.join("config.json"), serde_json::to_vec_pretty(&config)?)?;
    fs::write(
        out.join("import-provenance.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"uor-r4.portable-bound-input-import/1","original_config_sha256":sha256_file(&old)?,"portable_binding_sha256":sha256_file(&bound_path)?,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"scope":"new import seal over exact authenticated source/data bytes; original enclosing experiment seals were not archived and are NOT claimed restored; no model execution or optimizer updates"}),
        )?,
    )?;
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    println!("{}", out.join("config.json").display());
    Ok(())
}
