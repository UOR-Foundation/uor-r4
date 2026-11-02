use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use one_model_end_to_end_panel::evidence::evaluate;

fn main() {
    if let Err(error) = run() {
        eprintln!("m2-evidence: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: BTreeMap<String, String> = std::env::args()
        .skip(1)
        .map(|argument| {
            argument
                .split_once('=')
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                .ok_or_else(|| format!("argument {argument:?} is not KEY=VALUE"))
        })
        .collect::<Result<_, _>>()?;
    let artifact = PathBuf::from(args.get("artifact").ok_or_else(usage)?);
    let panel = PathBuf::from(args.get("panel").ok_or_else(usage)?);
    let train = args.get("train").map(PathBuf::from);
    let out = PathBuf::from(args.get("out").ok_or_else(usage)?);
    let max_targets = args
        .get("max-targets")
        .map(|value| value.parse::<usize>())
        .transpose()
        .map_err(|_| "max-targets= wants a non-negative whole number".to_owned())?;
    if out.exists() {
        return Err(format!(
            "{} exists; reports are never replaced",
            out.display()
        ));
    }
    let report = evaluate(&artifact, &panel, train.as_deref(), max_targets)?;
    let text = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    fs::write(&out, &text).map_err(|error| format!("{}: {error}", out.display()))?;
    println!(
        "{}",
        serde_json::json!({
            "schema": report.schema,
            "measured_targets": report.measured_targets,
            "overall": report.overall,
            "out": out,
            "sha256": report_sha256(&text),
        })
    );
    Ok(())
}

fn report_sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(bytes))
}

fn usage() -> String {
    "usage: m2-evidence artifact=ARTIFACT panel=DIR [train=DIR] [max-targets=N] out=NEW_JSON"
        .to_owned()
}
