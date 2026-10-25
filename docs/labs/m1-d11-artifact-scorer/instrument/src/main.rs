//! `d11-scorer export CHECKPOINT_JSON OUT_LUT REPORT_JSON`
//! `d11-scorer rows CONFIG_JSON OUT_JSON`
//!
//! Stage 1 of `m1-d11-artifact-scorer` (#3248). Every output is created new:
//! an existing file is never overwritten.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use m1_d11_artifact_scorer_instrument as inst;
use serde_json::Value;
use uor_r4_discrete::training::DriverConfig;

fn read_json(path: &str) -> Result<Value, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{path}: {e}"))
}

fn write_new(path: &str, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(Path::new(path))
        .map_err(|e| format!("{path}: {e} (outputs are never overwritten)"))?;
    file.write_all(bytes).map_err(|e| format!("{path}: {e}"))
}

fn json_bytes(value: &Value) -> Result<Vec<u8>, String> {
    let mut text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    text.push('\n');
    Ok(text.into_bytes())
}

fn run(args: &[String]) -> Result<(), String> {
    match args {
        [command, checkpoint, out, report] if command == "export" => {
            let exported = inst::export_checkpoint(&read_json(checkpoint)?)?;
            write_new(out, &exported.bytes)?;
            write_new(report, &json_bytes(&exported.report)?)?;
            println!("exported {} bytes, file sha256 {}", exported.bytes.len(), exported.report["file_sha256"]);
            Ok(())
        }
        [command, config, out] if command == "rows" => {
            let config: DriverConfig = serde_json::from_value(read_json(config)?)
                .map_err(|e| format!("{config}: {e}"))?;
            let rows = inst::dev_rows(&config)?;
            let report = inst::rows_report(&rows);
            write_new(out, &json_bytes(&report)?)?;
            println!("{} rows, sha256 {}", rows.len(), report["rows_sha256"]);
            Ok(())
        }
        _ => Err("usage: d11-scorer export CHECKPOINT_JSON OUT_LUT REPORT_JSON | rows CONFIG_JSON OUT_JSON".into()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("refused: {e}");
            ExitCode::from(1)
        }
    }
}
