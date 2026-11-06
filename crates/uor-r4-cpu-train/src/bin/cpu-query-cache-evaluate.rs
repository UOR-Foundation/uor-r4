//! Evaluate centroid and matched-control output tables on a sealed FBS query cache.
//!
//! `cpu-query-cache-evaluate CACHE --train SPLIT --panel SPLIT ... [options]`

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use uor_r4_cpu_train::{
    evaluate_query_cache, CentroidEvaluationRequest, CpuTrainError,
    REGISTERED_MINIMUM_OBSERVATIONS, REGISTERED_MIN_FITTED_ROWS, REGISTERED_MIN_FIT_OBSERVATIONS,
};

fn main() -> Result<(), CpuTrainError> {
    let mut args = std::env::args().skip(1);
    let cache = PathBuf::from(args.next().ok_or_else(usage)?);
    let mut request = CentroidEvaluationRequest {
        train_split: "train".to_owned(),
        panel_splits: Vec::new(),
        memory: false,
        minimum_observations: REGISTERED_MINIMUM_OBSERVATIONS,
        minimum_fitted_rows: REGISTERED_MIN_FITTED_ROWS,
        minimum_fit_observations: REGISTERED_MIN_FIT_OBSERVATIONS,
        expected_rows: None,
        expected_row_identity: None,
    };
    let mut out = None;
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| CpuTrainError::Invalid(format!("missing value for {flag}")))?;
        match flag.as_str() {
            "--train" => request.train_split = value,
            "--panel" => request.panel_splits.push(value),
            "--memory" => {
                request.memory = match value.as_str() {
                    "0" => false,
                    "1" => true,
                    _ => return Err(CpuTrainError::Invalid("--memory wants 0 or 1".to_owned())),
                }
            }
            "--minimum" => request.minimum_observations = number(&flag, &value)?,
            "--min-fitted-rows" => request.minimum_fitted_rows = number(&flag, &value)?,
            "--min-fit-observations" => request.minimum_fit_observations = number(&flag, &value)?,
            "--expect-rows" => request.expected_rows = Some(number(&flag, &value)?),
            "--expect-identity" => request.expected_row_identity = Some(value),
            "--out" => out = Some(PathBuf::from(value)),
            _ => return Err(CpuTrainError::Invalid(format!("unknown flag {flag}"))),
        }
    }
    if request.panel_splits.is_empty() {
        return Err(CpuTrainError::Invalid(
            "at least one --panel is required".to_owned(),
        ));
    }
    let report = evaluate_query_cache(&cache, &request)?;
    let bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| CpuTrainError::Invalid(error.to_string()))?;
    match out {
        Some(path) => write_new(&path, &bytes),
        None => {
            std::io::stdout()
                .write_all(&bytes)
                .map_err(|error| CpuTrainError::Invalid(error.to_string()))?;
            std::io::stdout()
                .write_all(b"\n")
                .map_err(|error| CpuTrainError::Invalid(error.to_string()))?;
            Ok(())
        }
    }
}

fn number(flag: &str, value: &str) -> Result<usize, CpuTrainError> {
    value
        .parse()
        .map_err(|_| CpuTrainError::Invalid(format!("{flag} wants a positive integer")))
        .and_then(|value| {
            if value == 0 {
                Err(CpuTrainError::Invalid(format!(
                    "{flag} wants a positive integer"
                )))
            } else {
                Ok(value)
            }
        })
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), CpuTrainError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| CpuTrainError::Invalid(format!("{}: {error}", path.display())))?;
    file.write_all(bytes)
        .map_err(|error| CpuTrainError::Invalid(error.to_string()))?;
    Ok(())
}

fn usage() -> CpuTrainError {
    CpuTrainError::Invalid(
        "usage: cpu-query-cache-evaluate CACHE --train SPLIT --panel SPLIT ... [--memory 0|1] [--minimum N] [--min-fitted-rows N] [--min-fit-observations N] [--expect-rows N] [--expect-identity SHA256] [--out FILE]"
            .to_owned(),
    )
}
