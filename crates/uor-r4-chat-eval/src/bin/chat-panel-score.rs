//! `chat-panel-score`: score arms on the frozen chat panel and calibrate the
//! instrument. Evaluation-only; no model is trained or loaded here.
//!
//! Usage:
//!   chat-panel-score panel-hash --panel PANEL.json
//!   chat-panel-score score --panel PANEL.json --generations GEN.jsonl
//!        [--tokenizer tokenizer.json] [--out report.json] [--label NAME]
//!   chat-panel-score calibrate --panel PANEL.json --degenerate GEN.jsonl
//!        [--degenerate-format panel|integer] [--tokenizer tokenizer.json]
//!        [--out calibration.json]
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use uor_r4_chat_eval::{
    calibrate, canonical_panel_sha256, load_integer_generations, load_panel,
    load_panel_generations, score_arm, tokenizer_from_bytes, ArmReport, Calibration, EvalError,
    TokenizerRef,
};

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  chat-panel-score panel-hash --panel PANEL.json\n  chat-panel-score score --panel PANEL.json --generations GEN.jsonl [--tokenizer tokenizer.json] [--out report.json] [--label NAME]\n  chat-panel-score calibrate --panel PANEL.json --degenerate GEN.jsonl [--degenerate-format panel|integer] [--tokenizer tokenizer.json] [--out calibration.json]"
    );
    ExitCode::from(2)
}

struct Args {
    command: String,
    values: std::collections::HashMap<String, String>,
}

fn parse_args() -> Result<Args, String> {
    let mut argv = std::env::args().skip(1);
    let command = argv.next().ok_or_else(|| "missing command".to_string())?;
    let mut values = std::collections::HashMap::new();
    while let Some(flag) = argv.next() {
        if !flag.starts_with("--") {
            return Err(format!("unexpected argument {flag}"));
        }
        let key = flag.trim_start_matches("--").to_string();
        let value = argv
            .next()
            .ok_or_else(|| format!("flag --{key} needs a value"))?;
        values.insert(key, value);
    }
    Ok(Args { command, values })
}

fn required(args: &Args, key: &str) -> Result<String, String> {
    args.values
        .get(key)
        .cloned()
        .ok_or_else(|| format!("missing required --{key}"))
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let args = parse_args()?;
    match args.command.as_str() {
        "panel-hash" => {
            let panel = PathBuf::from(required(&args, "panel")?);
            let digest = canonical_panel_sha256(&panel).map_err(display)?;
            println!("{digest}");
            Ok(ExitCode::SUCCESS)
        }
        "score" => {
            let panel_path = PathBuf::from(required(&args, "panel")?);
            let generations_path = PathBuf::from(required(&args, "generations")?);
            let label = args
                .values
                .get("label")
                .cloned()
                .unwrap_or_else(|| "arm".to_string());
            let panel = load_panel(&panel_path).map_err(display)?;
            let generations = load_panel_generations(&generations_path).map_err(display)?;
            let tokenizer_owned = load_tokenizer(args.values.get("tokenizer"))?;
            let tokenizer = tokenizer_owned.as_ref().map(|t| TokenizerRef::ByteBpe(t));
            let report = score_arm(&panel, &label, &generations, tokenizer.as_ref());
            print_arm(&report);
            if let Some(out) = args.values.get("out") {
                write_json(Path::new(out), &report)?;
            }
            Ok(ExitCode::SUCCESS)
        }
        "calibrate" => {
            let panel_path = PathBuf::from(required(&args, "panel")?);
            let degenerate_path = PathBuf::from(required(&args, "degenerate")?);
            let format = args
                .values
                .get("degenerate-format")
                .cloned()
                .unwrap_or_else(|| "panel".to_string());
            let panel = load_panel(&panel_path).map_err(display)?;
            let aligned = match format.as_str() {
                "panel" => {
                    let provided = load_panel_generations(&degenerate_path).map_err(display)?;
                    uor_r4_chat_eval::align_to_development(&panel, &provided)
                }
                "integer" => load_integer_generations(&degenerate_path, &panel).map_err(display)?,
                other => return Err(format!("unknown --degenerate-format {other}")),
            };
            let tokenizer_owned = load_tokenizer(args.values.get("tokenizer"))?;
            let tokenizer = tokenizer_owned.as_ref().map(|t| TokenizerRef::ByteBpe(t));
            let calibration = calibrate(&panel, &aligned, tokenizer.as_ref());
            print_calibration(&calibration);
            if let Some(out) = args.values.get("out") {
                write_json(Path::new(out), &calibration)?;
            }
            Ok(if calibration.instrument_valid {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
        _ => Ok(usage()),
    }
}

fn load_tokenizer(
    path: Option<&String>,
) -> Result<Option<uor_r4_tokenizer::ByteBpeTokenizer>, String> {
    match path {
        None => Ok(None),
        Some(path) => {
            let bytes = std::fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
            let tokenizer = tokenizer_from_bytes(&bytes)
                .ok_or_else(|| format!("{path} is not a byte-level BPE tokenizer"))?;
            Ok(Some(tokenizer))
        }
    }
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(path, bytes).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(())
}

fn print_arm(report: &ArmReport) {
    println!("arm {}  rows={}", report.label, report.n_rows);
    println!(
        "  core on_topic        {:>3}/{:<3}  p={:.3}  wilson95=[{:.3},{:.3}]",
        report.aggregate.core_on_topic.k,
        report.aggregate.core_on_topic.n,
        report.aggregate.core_on_topic.p,
        report.aggregate.core_on_topic.wilson95[0],
        report.aggregate.core_on_topic.wilson95[1]
    );
    println!(
        "  instruction complies {:>3}/{:<3}  p={:.3}  wilson95=[{:.3},{:.3}]",
        report.aggregate.instruction_complies.k,
        report.aggregate.instruction_complies.n,
        report.aggregate.instruction_complies.p,
        report.aggregate.instruction_complies.wilson95[0],
        report.aggregate.instruction_complies.wilson95[1]
    );
    println!(
        "  memory fact          {:>3}/{:<3}  p={:.3}  wilson95=[{:.3},{:.3}]",
        report.aggregate.memory_fact.k,
        report.aggregate.memory_fact.n,
        report.aggregate.memory_fact.p,
        report.aggregate.memory_fact.wilson95[0],
        report.aggregate.memory_fact.wilson95[1]
    );
    println!(
        "  refusal ok           {:>3}/{:<3}  p={:.3}  wilson95=[{:.3},{:.3}]",
        report.aggregate.refusal_ok.k,
        report.aggregate.refusal_ok.n,
        report.aggregate.refusal_ok.p,
        report.aggregate.refusal_ok.wilson95[0],
        report.aggregate.refusal_ok.wilson95[1]
    );
    println!(
        "  cycle_rate           {}/{} = {:.3}  wilson95=[{:.3},{:.3}]",
        report.aggregate.cycle_rate.count,
        report.aggregate.cycle_rate.n,
        report.aggregate.cycle_rate.rate,
        report.aggregate.cycle_rate.wilson95[0],
        report.aggregate.cycle_rate.wilson95[1]
    );
    println!(
        "  trunc_rate           {}/{} = {:.3}  wilson95=[{:.3},{:.3}]",
        report.aggregate.trunc_rate.count,
        report.aggregate.trunc_rate.n,
        report.aggregate.trunc_rate.rate,
        report.aggregate.trunc_rate.wilson95[0],
        report.aggregate.trunc_rate.wilson95[1]
    );
    for check in &report.aggregate.thresholds {
        println!(
            "  [{}] {:22} {:>10}  {}",
            if check.pass { "PASS" } else { "FAIL" },
            check.name,
            check.observed,
            check.requirement
        );
    }
    println!("  passes_v0 = {}", report.aggregate.passes_v0);
}

fn print_calibration(calibration: &Calibration) {
    println!("chat instrument calibration");
    println!(
        "  panel_sha256 = {}",
        calibration.panel_sha256.as_deref().unwrap_or("(none)")
    );
    for report in [
        &calibration.scripted,
        &calibration.degenerate,
        &calibration.c1_retrieval,
    ] {
        println!(
            "  {:<14} core {}/{}  memory {}/{}  complies {}/{}  refusal {}/{}  cycle {:.3}  trunc {:.3}  passes_v0={}",
            report.label,
            report.aggregate.core_on_topic.k,
            report.aggregate.core_on_topic.n,
            report.aggregate.memory_fact.k,
            report.aggregate.memory_fact.n,
            report.aggregate.instruction_complies.k,
            report.aggregate.instruction_complies.n,
            report.aggregate.refusal_ok.k,
            report.aggregate.refusal_ok.n,
            report.aggregate.cycle_rate.rate,
            report.aggregate.trunc_rate.rate,
            report.aggregate.passes_v0
        );
    }
    for check in &calibration.checks {
        println!(
            "  [{}] {:26} {:>28}  {}",
            if check.pass { "PASS" } else { "FAIL" },
            check.name,
            check.observed,
            check.requirement
        );
    }
    println!("  instrument_valid = {}", calibration.instrument_valid);
}

fn display(error: EvalError) -> String {
    error.to_string()
}
