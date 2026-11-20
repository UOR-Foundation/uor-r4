//! Write the conjunction memory from TRAIN and read it beside the model on
//! DEV (`uor_r4_discrete::conjunction`). Its own binary, so the driver source
//! pin (which covers `main.rs`) and every existing checkpoint stay valid.
//!
//! Usage: conjunction-probe CONFIG_JSON CHECKPOINT_JSON|- OUT_JSON
//! MODEL_TARGETS BULK_TARGETS
//! OUT_JSON must not exist. `-` reads a new model from the config.

use std::process::ExitCode;

#[cfg(feature = "cuda")]
fn run(args: &[String]) -> Result<(), String> {
    use std::path::Path;
    use uor_r4_discrete::conjunction::{self, Options};
    use uor_r4_discrete::training::DriverConfig;

    if args.len() != 5 {
        return Err(
            "usage: conjunction-probe CONFIG_JSON CHECKPOINT_JSON|- OUT_JSON MODEL_TARGETS BULK_TARGETS"
                .into(),
        );
    }
    let bytes = std::fs::read(&args[0]).map_err(|e| format!("{}: {e}", args[0]))?;
    let config: DriverConfig = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let checkpoint = (args[1] != "-").then(|| Path::new(&args[1]));
    let number = |text: &String| text.parse::<u64>().map_err(|e| format!("{text}: {e}"));
    let options = Options {
        model_targets: number(&args[3])?,
        bulk_targets: number(&args[4])?,
    };
    let report = conjunction::run(config, checkpoint, Path::new(&args[2]), options)
        .map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::json!({
            "empty": report["dev_empty_memory"],
            "model_written": report["model_written"]["dev"],
            "bulk_written": report["bulk_written"]["dev"],
        })
    );
    Ok(())
}

#[cfg(not(feature = "cuda"))]
fn run(_: &[String]) -> Result<(), String> {
    Err("conjunction-probe needs the cuda feature".into())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("conjunction-probe: {message}");
            ExitCode::FAILURE
        }
    }
}
