//! Derive a trusted binding from an independently reloaded, sealed reader checkpoint.
//! Usage: geometric-selected-reader-binding CHECKPOINT NEW_OUTPUT_DIRECTORY.
//! This prepares evidence only; it neither selects a checkpoint nor runs predictions.
use serde_json::{json, Value};
use std::{
    fs, io,
    io::Write,
    path::{Component, Path, PathBuf},
};
use uor_r4_core::report_output;
use uor_r4_integer::geometric_source_realizer::{
    NativeArtifactBinding, NativeSourceRealizer as IntegerRealizer,
};
use uor_r4_training::{
    geometric_occurrence_consumer::source_realizer::{NativeSourceRealizer, SourceRealizerWeights},
    sha256_file,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn write_json(root: &Path, name: &str, value: &Value) -> Result<()> {
    fs::File::create_new(root.join(name))?.write_all(&serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

// Resolve existing ancestors before checking overlap, including symlinked parents.
fn resolved_future(path: &Path) -> Result<PathBuf> {
    if path.as_os_str().is_empty() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(invalid("empty paths and parent traversal are not accepted").into());
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut existing = absolute.clone();
    let mut suffix = Vec::new();
    while !existing.exists() {
        suffix.push(
            existing
                .file_name()
                .ok_or_else(|| invalid("output ancestor absent"))?
                .to_os_string(),
        );
        if !existing.pop() {
            return Err(invalid("output ancestor absent").into());
        }
    }
    let mut resolved = existing.canonicalize()?;
    for part in suffix.iter().rev() {
        resolved.push(part);
    }
    Ok(resolved)
}

fn prepare(checkpoint: &Path, out: &Path) -> Result<()> {
    let sealed = checkpoint
        .ancestors()
        .find(|p| p.join(report_output::MANIFEST_FILE).is_file())
        .ok_or_else(|| invalid("checkpoint has no sealed ancestor"))?;
    report_output::verify(sealed)?;
    let manifest_hash = sha256_file(&sealed.join(report_output::MANIFEST_FILE))?;
    let receipt_path = checkpoint.join("receipt.json");
    let receipt: Value = serde_json::from_slice(&fs::read(&receipt_path)?)?;
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["new_parent"].clone())?;
    let native_root = checkpoint.join("native");
    let source_root = checkpoint.join("source");
    let native = IntegerRealizer::load_native(&native_root, &binding)?;
    if native.artifact_binding() != &binding {
        return Err(invalid("integer reload binding differs from checkpoint receipt").into());
    }
    let tokenizer = fs::read(native_root.join("tokenizer.json"))?;
    let source = SourceRealizerWeights::load_source(&source_root, &tokenizer)?;
    let metadata: Value = serde_json::from_slice(&fs::read(native_root.join("metadata.json"))?)?;
    let identity = serde_json::from_value(metadata["identity"].clone())?;
    let training = NativeSourceRealizer::load(&native_root, &source, &identity)?;
    if training.artifact_binding()? != binding {
        return Err(invalid("source-backed reload binding differs from checkpoint receipt").into());
    }
    report_output::verify(sealed)?;
    if sha256_file(&sealed.join(report_output::MANIFEST_FILE))? != manifest_hash {
        return Err(invalid("input manifest changed during preparation").into());
    }
    write_json(
        out,
        "trusted-native-binding.json",
        &serde_json::to_value(&binding)?,
    )?;
    write_json(
        out,
        "provenance.json",
        &json!({
        "schema":"uor-r4.selected-reader-binding-preparation/1",
        "status":"PASS", "checkpoint":checkpoint, "verified_input_root":sealed,
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "executable_sha256":sha256_file(&std::env::current_exe()?)?,
            "input_manifest_sha256":manifest_hash,
            "checkpoint_receipt_sha256":sha256_file(&receipt_path)?,
            "native_metadata_sha256":sha256_file(&native_root.join("metadata.json"))?,
            "tokenizer_sha256":sha256_file(&native_root.join("tokenizer.json"))?,
            "trusted_native_binding_sha256":sha256_file(&out.join("trusted-native-binding.json"))?,
            "integer_and_source_backed_reload_bindings_equal":true,
            "scope":"caller-chosen checkpoint; no checkpoint-selection, prediction, fit-admission or language-capability claim"
        }),
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(invalid(
            "usage: geometric-selected-reader-binding CHECKPOINT NEW_OUTPUT_DIRECTORY",
        )
        .into());
    }
    let checkpoint = PathBuf::from(&args[0]).canonicalize()?;
    if !checkpoint.is_dir() {
        return Err(invalid("checkpoint must be a directory").into());
    }
    let out = resolved_future(Path::new(&args[1]))?;
    if out.starts_with(&checkpoint)
        || checkpoint.starts_with(&out)
        || out
            .ancestors()
            .any(|p| p.join(report_output::MANIFEST_FILE).exists())
    {
        return Err(invalid("output overlaps checkpoint or lies beneath a sealed attempt").into());
    }
    report_output::claim(&out)?;
    let result = prepare(&checkpoint, &out);
    if let Err(error) = &result {
        write_json(
            &out,
            "failure.json",
            &json!({"status":"FAIL","error":error.to_string()}),
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result
}
