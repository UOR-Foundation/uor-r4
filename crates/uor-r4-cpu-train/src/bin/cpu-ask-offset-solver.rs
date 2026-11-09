//! Offline CPU solver for Claude-330's checkpoint-compatible ask tables.

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use uor_r4_cpu_train::ask_offset_audit::TrainedAskOffsets;
use uor_r4_cpu_train::ask_offset_solver::{AskOffsetSolveBasis, AskPullCorpus};

const SCHEMA: &str = "uor-r4.cpu-ask-offset-solver/1";
const DEFAULT_MAX_EVALUATIONS: u64 = 200_000;
const RELATIONS: std::ops::Range<u32> = 16..32;
const QUERY_SHIFT: u32 = 16;

type Result<T> = std::result::Result<T, String>;

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-ask-offset-solver: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = parse_args(std::env::args().skip(1))?;
    run_with(&args)
}

fn run_with(args: &BTreeMap<String, String>) -> Result<()> {
    let checkpoint = Path::new(&args["checkpoint"]);
    let model_path = checkpoint.join("model.safetensors");
    let hamming_path = checkpoint.join("hamming_read.safetensors");
    let model =
        std::fs::read(&model_path).map_err(|error| format!("{}: {error}", model_path.display()))?;
    let hamming = std::fs::read(&hamming_path)
        .map_err(|error| format!("{}: {error}", hamming_path.display()))?;
    let model_sha = uor_r4_discrete::corpus::sha256(&model);
    let hamming_sha = uor_r4_discrete::corpus::sha256(&hamming);
    validate_sha256(&args["expected-model-sha256"], "model")?;
    validate_sha256(&args["expected-hamming-sha256"], "hamming")?;
    if model_sha != args["expected-model-sha256"] {
        return Err(format!(
            "model.safetensors SHA-256 differs: got {model_sha}"
        ));
    }
    if hamming_sha != args["expected-hamming-sha256"] {
        return Err(format!(
            "hamming_read.safetensors SHA-256 differs: got {hamming_sha}"
        ));
    }
    let out = PathBuf::from(&args["out"]);
    let hamming_out = PathBuf::from(&args["hamming-out"]);
    if out == hamming_out {
        return Err("--out and --hamming-out must be distinct".to_owned());
    }
    if out.exists() {
        return Err(format!(
            "{} exists; reports are never replaced",
            out.display()
        ));
    }
    if hamming_out.exists() {
        return Err(format!(
            "{} exists; candidate sidecars are never replaced",
            hamming_out.display()
        ));
    }
    let corpus = match args.get("pulls") {
        Some(path) => {
            let bytes = std::fs::read(path).map_err(|error| format!("{path}: {error}"))?;
            AskPullCorpus::from_json(&bytes)?
        }
        None => AskPullCorpus::b2_relations(RELATIONS, QUERY_SHIFT),
    };
    let max_evaluations = args
        .get("max-evaluations")
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|_| "--max-evaluations wants a positive integer".to_owned())
        })
        .transpose()?
        .unwrap_or(DEFAULT_MAX_EVALUATIONS);
    let basis = AskOffsetSolveBasis::from_bytes(&model, &hamming)?;
    let solved = basis.solve(&corpus, max_evaluations)?;
    let audited = TrainedAskOffsets::from_bytes(&model, &solved.hamming_bytes)?;
    let relations = audited.audit_relations(RELATIONS, QUERY_SHIFT)?;
    let audit_cells = relations
        .iter()
        .flat_map(|relation| relation.cells.iter())
        .collect::<Vec<_>>();
    let audit_summary = serde_json::json!({
        "cells": audit_cells.len(),
        "trained_exact_codes": audit_cells.iter().filter(|cell| cell.trained_bits == 0).count(),
        "trained_target_nearest": audit_cells.iter().filter(|cell| cell.trained_target_nearest).count(),
        "zero_target_nearest": audit_cells.iter().filter(|cell| cell.zero_target_nearest).count(),
        "trained_total_bits": audit_cells.iter().map(|cell| u64::from(cell.trained_bits)).sum::<u64>(),
        "zero_total_bits": audit_cells.iter().map(|cell| u64::from(cell.zero_bits)).sum::<u64>(),
    });
    let candidate_sha = uor_r4_discrete::corpus::sha256(&solved.hamming_bytes);
    let summary = serde_json::json!({
        "questions": solved.readings.iter().map(|reading| reading.question).collect::<std::collections::BTreeSet<_>>().len(),
        "depths": solved.readings.iter().map(|reading| reading.depth).collect::<std::collections::BTreeSet<_>>().len(),
        "readings": solved.readings.len(),
        "zero_weighted_bits": solved.readings.iter().map(|reading| reading.zero_weighted_bits).sum::<u64>(),
        "solved_weighted_bits": solved.readings.iter().map(|reading| reading.solved_weighted_bits).sum::<u64>(),
        "improvement_bits": solved.readings.iter().map(|reading| reading.improvement_bits).sum::<u64>(),
        "improved_readings": solved.readings.iter().filter(|reading| reading.improvement_bits > 0).count(),
        "evaluations": solved.readings.iter().map(|reading| reading.evaluations).sum::<u64>(),
    });
    let report = serde_json::json!({
        "schema": SCHEMA,
        "checkpoint": {
            "model_sha256": model_sha,
            "hamming_sha256": hamming_sha,
            "vocab": audited.vocab(),
            "width": audited.width(),
            "heads": audited.heads(),
            "ask_heads": audited.ask_head_count(),
            "depths": audited.depths().collect::<Vec<_>>(),
        },
        "settings": {
            "pull_corpus": args.get("pulls"),
            "max_evaluations_per_question_depth": max_evaluations,
            "audit_relations": RELATIONS,
            "query_shift": QUERY_SHIFT,
        },
        "hardware": {
            "cpu": cpu_model(),
            "executable_sha256": executable_sha256()?,
        },
        "summary": summary,
        "audit_after_solve": audit_summary,
        "candidate_hamming_sha256": candidate_sha,
        "readings": solved.readings,
        "claim": "offline checkpoint ask-offset initializer; not a trained ask-head model, B2 result, artifact schema or serving result"
    });
    let summary = report["summary"].clone();
    let mut report_bytes = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    report_bytes.push(b'\n');
    write_new(&hamming_out, &solved.hamming_bytes)?;
    write_new(&out, &report_bytes)?;
    println!(
        "{}",
        serde_json::json!({
            "schema": SCHEMA,
            "out": out,
            "hamming_out": hamming_out,
            "report_sha256": uor_r4_discrete::corpus::sha256(&report_bytes),
            "candidate_hamming_sha256": candidate_sha,
            "summary": summary,
        })
    );
    Ok(())
}

fn validate_sha256(value: &str, name: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("--expected-{name}-sha256 is not a SHA-256"));
    }
    Ok(())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<BTreeMap<String, String>> {
    const FLAGS: [&str; 7] = [
        "checkpoint",
        "expected-model-sha256",
        "expected-hamming-sha256",
        "out",
        "hamming-out",
        "pulls",
        "max-evaluations",
    ];
    const REQUIRED: [&str; 5] = [
        "checkpoint",
        "expected-model-sha256",
        "expected-hamming-sha256",
        "out",
        "hamming-out",
    ];
    let mut parsed = BTreeMap::new();
    let mut args = args;
    while let Some(argument) = args.next() {
        let name = argument
            .strip_prefix("--")
            .filter(|name| FLAGS.contains(name))
            .ok_or_else(|| format!("unknown argument {argument}"))?;
        let value = args
            .next()
            .ok_or_else(|| format!("{argument} needs a value"))?;
        if parsed.insert(name.to_owned(), value).is_some() {
            return Err(format!("{argument} given twice"));
        }
    }
    for name in REQUIRED {
        if !parsed.contains_key(name) {
            return Err(format!("--{name} is required"));
        }
    }
    Ok(parsed)
}

fn cpu_model() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("model name\t: ")
                    .map(std::string::ToString::to_string)
            })
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

fn executable_sha256() -> Result<String> {
    let path = std::env::current_exe().map_err(|error| error.to_string())?;
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    Ok(uor_r4_discrete::corpus::sha256(&bytes))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    file.write_all(bytes).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use safetensors::{serialize, tensor::TensorView, Dtype};

    struct TestDir(PathBuf);

    impl TestDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "uor-r4-cpu-ask-offset-solver-{name}-{}",
                std::process::id()
            ));
            if path.exists() {
                std::fs::remove_dir_all(&path).unwrap();
            }
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn f32_bytes(values: &[f32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    fn u32_bytes(values: &[u32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    fn tensor_file(tensors: Vec<(&str, Dtype, Vec<usize>, Vec<u8>)>) -> Vec<u8> {
        let tensors = tensors
            .into_iter()
            .map(|(name, dtype, shape, data)| (name.to_owned(), dtype, shape, data))
            .collect::<Vec<_>>();
        let views = tensors
            .iter()
            .map(|(name, dtype, shape, data)| {
                (
                    name.clone(),
                    TensorView::new(*dtype, shape.clone(), data).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        serialize(views, None).unwrap()
    }

    fn write_checkpoint(dir: &Path) -> (String, String) {
        let (vocab, width, heads) = (64usize, 8usize, 2usize);
        let mut table = vec![1.0f32; vocab * width];
        table[17 * width..18 * width].fill(-1.0);
        let model = tensor_file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![vocab, width],
            f32_bytes(&table),
        )]);
        let rotation = (0..width / 4)
            .flat_map(|_| [1.0f32, 0.0, 0.0, 0.0])
            .collect::<Vec<_>>();
        let hamming = tensor_file(vec![
            ("heads", Dtype::U32, vec![1], u32_bytes(&[heads as u32])),
            (
                "d2.key_rotation",
                Dtype::F32,
                vec![width],
                f32_bytes(&rotation),
            ),
            (
                "d6.key_rotation",
                Dtype::F32,
                vec![width],
                f32_bytes(&rotation),
            ),
        ]);
        std::fs::write(dir.join("model.safetensors"), &model).unwrap();
        std::fs::write(dir.join("hamming_read.safetensors"), &hamming).unwrap();
        (
            uor_r4_discrete::corpus::sha256(&model),
            uor_r4_discrete::corpus::sha256(&hamming),
        )
    }

    #[test]
    fn parse_requires_outputs_and_hashes() {
        let base = [
            "--checkpoint",
            "c",
            "--expected-model-sha256",
            "m",
            "--expected-hamming-sha256",
            "h",
            "--out",
            "o",
            "--hamming-out",
            "s",
        ];
        let parsed = parse_args(base.into_iter().map(str::to_owned)).unwrap();
        assert_eq!(parsed.len(), 5);
        assert!(parse_args(["--checkpoint", "c"].into_iter().map(str::to_owned)).is_err());
    }

    #[test]
    fn sha_and_default_evaluations_are_validated() {
        assert!(validate_sha256(&"a".repeat(64), "model").is_ok());
        assert!(validate_sha256(&"a".repeat(63), "model").is_err());
        assert_eq!(DEFAULT_MAX_EVALUATIONS, 200_000);
    }

    #[test]
    fn synthetic_checkpoint_writes_solved_sidecar_report_and_repeat() {
        let dir = TestDir::new("report");
        let checkpoint = dir.0.join("checkpoint");
        std::fs::create_dir(&checkpoint).unwrap();
        let (model_sha, hamming_sha) = write_checkpoint(&checkpoint);
        let args_for = |suffix: &str| {
            BTreeMap::from([
                ("checkpoint".to_owned(), checkpoint.display().to_string()),
                ("expected-model-sha256".to_owned(), model_sha.clone()),
                ("expected-hamming-sha256".to_owned(), hamming_sha.clone()),
                (
                    "out".to_owned(),
                    dir.0
                        .join(format!("report-{suffix}.json"))
                        .display()
                        .to_string(),
                ),
                (
                    "hamming-out".to_owned(),
                    dir.0
                        .join(format!("hamming-{suffix}.safetensors"))
                        .display()
                        .to_string(),
                ),
            ])
        };
        let args = args_for("one");
        let mut collision = args.clone();
        collision.insert("hamming-out".to_owned(), args["out"].clone());
        assert!(run_with(&collision)
            .err()
            .expect("colliding outputs")
            .contains("must be distinct"));
        run_with(&args).unwrap();
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&args["out"]).unwrap()).unwrap();
        assert_eq!(report["schema"], SCHEMA);
        assert_eq!(report["summary"]["questions"], 16);
        assert_eq!(report["summary"]["solved_weighted_bits"], 0);
        assert_eq!(
            report["audit_after_solve"]["trained_target_nearest"],
            report["audit_after_solve"]["cells"]
        );

        let repeat = args_for("two");
        run_with(&repeat).unwrap();
        assert_eq!(
            std::fs::read(&args["hamming-out"]).unwrap(),
            std::fs::read(&repeat["hamming-out"]).unwrap()
        );
        assert_eq!(
            std::fs::read(&args["out"]).unwrap(),
            std::fs::read(&repeat["out"]).unwrap()
        );
        assert!(run_with(&args)
            .unwrap_err()
            .contains("reports are never replaced"));

        if let Ok(root) = std::env::var("UOR_KEEP_ASK_OFFSET_SOLVER_EVIDENCE") {
            std::fs::create_dir_all(&root).unwrap();
            std::fs::copy(&args["out"], Path::new(&root).join("synthetic-report.json")).unwrap();
            std::fs::copy(
                &args["hamming-out"],
                Path::new(&root).join("synthetic-hamming.safetensors"),
            )
            .unwrap();
        }
    }
}
