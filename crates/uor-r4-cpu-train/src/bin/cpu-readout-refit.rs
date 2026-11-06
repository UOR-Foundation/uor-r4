use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use rayon::prelude::*;
use uor_r4_cpu_train::{
    evaluate_output_table, observe_refit_thresholds, paired_two_sigma, patch_writer_table,
    refit_output_table, shuffle_refit_targets, validate_corpus_manifest, CpuCorpusIdentity,
    CpuFbsForward, CpuFbsHead, CpuMaskedCorpus, RankMarginSample, ReadoutEvaluation,
    ReadoutObservation, ReadoutRefitMethod, RefittedOutputTable,
};

type Result<T> = std::result::Result<T, String>;

const SUBSTRATE: &str = "cpu-development-research";
const QUALIFICATION: &str = "gpu-required";
const THRESHOLDS: [usize; 10] = [1, 2, 3, 4, 6, 8, 12, 16, 32, 64];
const EXPERIMENT_SEED: u64 = 2789;
const MINIMUM_OBSERVATIONS: usize = 8;
const MINIMUM_REFITTED_ROWS: usize = 100;
const MINIMUM_REFITTED_OBSERVATIONS: u64 = 1_000;
const MINIMUM_PANEL_TARGETS: usize = 1_000;
const EXPECTED_TOKENIZER_SHA256: &str =
    "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
const EXPECTED_BOS: u32 = 0;

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-readout-refit: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("preflight") => preflight(&args[1..]),
        Some("run") => run_experiment(&args[1..]),
        Some("aggregate") => aggregate_reports(&args[1..]),
        _ => Err(usage()),
    }
}

fn usage() -> String {
    concat!(
        "usage:\n",
        "  cpu-readout-refit preflight TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST ARTIFACT ",
        "--expected-tokenizer SHA --expected-train-tokens SHA --expected-train-mask SHA ",
        "--expected-train-manifest SHA --expected-artifact SHA\n",
        "  cpu-readout-refit run TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST PANEL_TOKENS ",
        "PANEL_MASK PANEL_MANIFEST ARTIFACT --expected-tokenizer SHA ",
        "--expected-train-tokens SHA --expected-train-mask SHA --expected-train-manifest SHA ",
        "--expected-panel-tokens SHA --expected-panel-mask SHA ",
        "--expected-panel-manifest SHA --expected-artifact SHA\n",
        "  cpu-readout-refit aggregate REPORT REPORT REPORT REPORT"
    )
    .to_owned()
}

fn preflight(args: &[String]) -> Result<()> {
    if args.len() < 4 {
        return Err(usage());
    }
    let paths = TrainPaths::new(&args[0], &args[1], &args[2]);
    let artifact_path = Path::new(&args[3]);
    let flags = parse_flags(&args[4..])?;
    let expected = Expected::from_flags(&flags, false)?;
    let started = Instant::now();
    let artifact = read_checked(artifact_path, &expected.artifact)?;
    let forward = CpuFbsForward::from_artifact(&artifact).map_err(|error| error.to_string())?;
    let (train, identity, train_read) =
        read_corpus(&paths, &expected.train, forward.context_length())?;
    if identity.vocab != forward.vocabulary() {
        return Err("TRAIN vocabulary differs from artifact".to_owned());
    }
    let samples = train.causal_samples().map_err(|error| error.to_string())?;
    validate_target_inventory(&identity, samples.len(), "TRAIN")?;
    let capture_started = Instant::now();
    let observations = capture_observations(&artifact, &samples)?;
    let capture_elapsed_ms = elapsed_ms(capture_started);
    let original_tokens = observations
        .iter()
        .map(|observation| observation.original_token)
        .collect::<Vec<_>>();
    let identity_evaluation = evaluate_output_table(
        forward.shared_table(),
        forward.vocabulary(),
        forward.width(),
        &observations,
        &original_tokens,
    )
    .map_err(|error| error.to_string())?;
    let identity_exact = decision_matches_capture(&identity_evaluation, &original_tokens);
    let census =
        observe_refit_thresholds(&observations, &THRESHOLDS).map_err(|error| error.to_string())?;
    let median = refit_output_table(
        forward.shared_table(),
        forward.vocabulary(),
        forward.width(),
        &observations,
        ReadoutRefitMethod::Median,
        MINIMUM_OBSERVATIONS,
    )
    .map_err(|error| error.to_string())?;
    let mean = refit_output_table(
        forward.shared_table(),
        forward.vocabulary(),
        forward.width(),
        &observations,
        ReadoutRefitMethod::MeanCentroid,
        MINIMUM_OBSERVATIONS,
    )
    .map_err(|error| error.to_string())?;
    let shuffled = shuffle_refit_targets(
        &observations,
        forward.vocabulary(),
        forward.width(),
        MINIMUM_OBSERVATIONS,
    )
    .map_err(|error| error.to_string())?;
    let mut report = serde_json::json!({
        "schema": "uor-r4.cpu-readout-refit-preflight/1",
        "status": if identity_exact { "COMPLETE" } else { "CONTROL_FAULT" },
        "substrate": SUBSTRATE,
        "qualification": QUALIFICATION,
        "inputs": inputs_json(
            artifact_path,
            &expected.artifact,
            &paths,
            &train_read.hashes,
            &identity,
            None,
        ),
        "artifact": {
            "vocab": forward.vocabulary(),
            "width": forward.width(),
            "context": forward.context_length(),
            "kept_cells": observations
                .first()
                .map(|observation| observation.kept_cells.len())
                .unwrap_or(0),
            "table_sha256": table_sha256(forward.shared_table()),
        },
        "capture": {
            "samples": samples.len(),
            "elapsed_ms": capture_elapsed_ms,
            "query_and_decision_sha256": capture_sha256(&observations),
        },
        "identity": {
            "exact": identity_exact,
            "changed_decisions": identity_evaluation.changed_decisions,
            "decision_sha256": identity_evaluation.decision_sha256,
            "captured_decision_sha256": token_sha256(&original_tokens),
        },
        "selected_minimum_observations": MINIMUM_OBSERVATIONS,
        "thresholds": census,
        "fit": {
            "median_rows": median.fitted_tokens.len(),
            "median_observations": median.observations,
            "median_changed_coordinates": median.changed_coordinates,
            "median_displacement_l1": median.displacement_l1,
            "median_maximum_row_displacement_l1": median.maximum_row_displacement_l1,
            "median_table_sha256": table_sha256(&median.table),
            "mean_rows": mean.fitted_tokens.len(),
            "mean_changed_coordinates": mean.changed_coordinates,
            "mean_displacement_l1": mean.displacement_l1,
            "mean_table_sha256": table_sha256(&mean.table),
            "shuffle_changed_targets": shuffled.changed_targets,
            "shuffle_table_sha256": table_sha256(
                &shuffled_table(
                    forward.shared_table(),
                    forward.vocabulary(),
                    forward.width(),
                    &shuffled.observations,
                    MINIMUM_OBSERVATIONS,
                )?.table
            ),
        },
        "power": {
            "minimum_refitted_rows": MINIMUM_REFITTED_ROWS,
            "minimum_refitted_observations": MINIMUM_REFITTED_OBSERVATIONS,
            "eligible": median.fitted_tokens.len() >= MINIMUM_REFITTED_ROWS
                && median.observations >= MINIMUM_REFITTED_OBSERVATIONS,
        },
        "execution": {
            "threads": rayon::current_num_threads(),
            "wall_elapsed_ms": elapsed_ms(started),
            "peak_rss_kib": peak_rss_kib(),
        },
    });
    finish_report(&mut report);
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn run_experiment(args: &[String]) -> Result<()> {
    if args.len() < 7 {
        return Err(usage());
    }
    let train_paths = TrainPaths::new(&args[0], &args[1], &args[2]);
    let panel_paths = PanelPaths::new(&args[3], &args[4], &args[5]);
    let artifact_path = Path::new(&args[6]);
    let flags = parse_flags(&args[7..])?;
    let expected = Expected::from_flags(&flags, true)?;
    let started = Instant::now();

    let artifact = read_checked(artifact_path, &expected.artifact)?;
    let forward = CpuFbsForward::from_artifact(&artifact).map_err(|error| error.to_string())?;
    let (train, train_identity, train_read) =
        read_corpus(&train_paths, &expected.train, forward.context_length())?;
    let expected_panel = expected
        .panel
        .as_ref()
        .ok_or_else(|| "panel pins are absent".to_owned())?;
    let (panel, panel_identity, panel_read) =
        read_corpus(&panel_paths, expected_panel, forward.context_length())?;
    if train_identity.vocab != forward.vocabulary() || panel_identity.vocab != forward.vocabulary()
    {
        return Err("corpus vocabulary differs from artifact".to_owned());
    }
    validate_panel(&panel_read.manifest_bytes, &panel_identity)?;

    let train_samples = train.causal_samples().map_err(|error| error.to_string())?;
    let panel_samples = panel.causal_samples().map_err(|error| error.to_string())?;
    validate_target_inventory(&train_identity, train_samples.len(), "TRAIN")?;
    validate_target_inventory(&panel_identity, panel_samples.len(), "fresh panel")?;
    let train_capture_started = Instant::now();
    let train_observations = capture_observations(&artifact, &train_samples)?;
    let train_capture_elapsed_ms = elapsed_ms(train_capture_started);
    let train_original_tokens = train_observations
        .iter()
        .map(|observation| observation.original_token)
        .collect::<Vec<_>>();
    let train_identity_evaluation = evaluate_output_table(
        forward.shared_table(),
        forward.vocabulary(),
        forward.width(),
        &train_observations,
        &train_original_tokens,
    )
    .map_err(|error| error.to_string())?;
    if !decision_matches_capture(&train_identity_evaluation, &train_original_tokens) {
        let mut report = early_artifact_report(
            artifact_path,
            &expected.artifact,
            &train_paths,
            &train_read.hashes,
            &train_identity,
            Some((&panel_paths, &panel_read.hashes, &panel_identity)),
            "CONTROL_FAULT",
            "CONTROL_FAULT",
            "TRAIN scorer does not reproduce the artifact's captured decisions",
            serde_json::json!({
                "changed_decisions": train_identity_evaluation.changed_decisions,
                "decision_sha256": train_identity_evaluation.decision_sha256,
                "captured_decision_sha256": token_sha256(&train_original_tokens),
            }),
            started,
        );
        finish_report(&mut report);
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
        );
        return Ok(());
    }
    let census = observe_refit_thresholds(&train_observations, &THRESHOLDS)
        .map_err(|error| error.to_string())?;
    let train_power = census
        .get(&MINIMUM_OBSERVATIONS)
        .ok_or_else(|| "minimum-observation census is absent".to_owned())?;
    if train_power.eligible_tokens < MINIMUM_REFITTED_ROWS as u64
        || train_power.eligible_observations < MINIMUM_REFITTED_OBSERVATIONS
    {
        let mut report = early_artifact_report(
            artifact_path,
            &expected.artifact,
            &train_paths,
            &train_read.hashes,
            &train_identity,
            Some((&panel_paths, &panel_read.hashes, &panel_identity)),
            "INSUFFICIENT_POWER",
            "COMPLETE",
            "TRAIN refit inventory is below the registered power floor",
            serde_json::json!({
                "minimum_observations": MINIMUM_OBSERVATIONS,
                "eligible_tokens": train_power.eligible_tokens,
                "eligible_observations": train_power.eligible_observations,
            }),
            started,
        );
        finish_report(&mut report);
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
        );
        return Ok(());
    }
    let original_table = forward.shared_table().to_vec();
    let median = refit_output_table(
        &original_table,
        forward.vocabulary(),
        forward.width(),
        &train_observations,
        ReadoutRefitMethod::Median,
        MINIMUM_OBSERVATIONS,
    )
    .map_err(|error| error.to_string())?;
    let mean = refit_output_table(
        &original_table,
        forward.vocabulary(),
        forward.width(),
        &train_observations,
        ReadoutRefitMethod::MeanCentroid,
        MINIMUM_OBSERVATIONS,
    )
    .map_err(|error| error.to_string())?;
    let shuffled = shuffle_refit_targets(
        &train_observations,
        forward.vocabulary(),
        forward.width(),
        MINIMUM_OBSERVATIONS,
    )
    .map_err(|error| error.to_string())?;
    let shuffled_fit = shuffled_table(
        &original_table,
        forward.vocabulary(),
        forward.width(),
        &shuffled.observations,
        MINIMUM_OBSERVATIONS,
    )?;

    let panel_capture_started = Instant::now();
    let panel_observations = capture_observations(&artifact, &panel_samples)?;
    let panel_capture_elapsed_ms = elapsed_ms(panel_capture_started);
    let original_tokens = panel_observations
        .iter()
        .map(|observation| observation.original_token)
        .collect::<Vec<_>>();
    let original = evaluate_output_table(
        &original_table,
        forward.vocabulary(),
        forward.width(),
        &panel_observations,
        &original_tokens,
    )
    .map_err(|error| error.to_string())?;
    let identical = evaluate_output_table(
        &original_table,
        forward.vocabulary(),
        forward.width(),
        &panel_observations,
        &original_tokens,
    )
    .map_err(|error| error.to_string())?;
    let median_eval = evaluate_output_table(
        &median.table,
        forward.vocabulary(),
        forward.width(),
        &panel_observations,
        &original_tokens,
    )
    .map_err(|error| error.to_string())?;
    let mean_eval = evaluate_output_table(
        &mean.table,
        forward.vocabulary(),
        forward.width(),
        &panel_observations,
        &original_tokens,
    )
    .map_err(|error| error.to_string())?;
    let shuffled_eval = evaluate_output_table(
        &shuffled_fit.table,
        forward.vocabulary(),
        forward.width(),
        &panel_observations,
        &original_tokens,
    )
    .map_err(|error| error.to_string())?;

    let shared_artifact =
        patch_writer_table(&artifact, &median.table).map_err(|error| error.to_string())?;
    let shared_capture_started = Instant::now();
    let shared_observations = capture_observations(&shared_artifact, &panel_samples)?;
    let shared_capture_elapsed_ms = elapsed_ms(shared_capture_started);
    let shared_original_tokens = shared_observations
        .iter()
        .map(|observation| observation.original_token)
        .collect::<Vec<_>>();
    let shared_eval = evaluate_output_table(
        &median.table,
        forward.vocabulary(),
        forward.width(),
        &shared_observations,
        &shared_original_tokens,
    )
    .map_err(|error| error.to_string())?;

    let baseline = count_table_baselines(&train_samples, &panel_samples);
    let original_correct = original.top1_rows.clone();
    let median_correct = median_eval.top1_rows.clone();
    let shuffled_correct = shuffled_eval.top1_rows.clone();
    let median_vs_original =
        paired_two_sigma(&median_correct, &original_correct).map_err(|error| error.to_string())?;
    let median_vs_shuffled =
        paired_two_sigma(&median_correct, &shuffled_correct).map_err(|error| error.to_string())?;

    let original_decision_sha256 = token_sha256(&original_tokens);
    let identity_exact = decision_matches_capture(&original, &original_tokens)
        && decision_matches_capture(&identical, &original_tokens);
    let power = median.fitted_tokens.len() >= MINIMUM_REFITTED_ROWS
        && median.observations >= MINIMUM_REFITTED_OBSERVATIONS
        && panel_samples.len() >= MINIMUM_PANEL_TARGETS;
    let primary_pass = identity_exact
        && power
        && median_vs_original["passes"] == true
        && median_vs_shuffled["passes"] == true
        && median_eval.top8_correct >= original.top8_correct;
    let artifact_outcome = if !identity_exact {
        "CONTROL_FAULT"
    } else if !power {
        "INSUFFICIENT_POWER"
    } else if primary_pass {
        "ARTIFACT_KEEP"
    } else {
        "ARTIFACT_REJECT"
    };

    let mut report = serde_json::json!({
        "schema": "uor-r4.cpu-readout-refit/1",
        "report_kind": "artifact",
            "status": if identity_exact { "COMPLETE" } else { "CONTROL_FAULT" },
        "substrate": SUBSTRATE,
        "qualification": QUALIFICATION,
        "claim": "fixed-input, output-only nearest-token readout refit on exact CPU Q12 queries",
        "seed": EXPERIMENT_SEED,
        "inputs": inputs_json(
            artifact_path,
            &expected.artifact,
            &train_paths,
            &train_read.hashes,
            &train_identity,
            Some((&panel_paths, &panel_read.hashes, &panel_identity)),
        ),
        "settings": {
            "minimum_observations": MINIMUM_OBSERVATIONS,
            "median_rounding": "lower-midpoint integer median in the L1 minimizing interval",
            "mean_rounding": "nearest integer, ties away from zero",
            "shuffle": "maximum-change cyclic label permutation among eligible observations",
            "paired_rule": "candidate_sum > control_sum and candidate_only >= max(2, ceil(2*sqrt(b+c)))",
        },
        "artifact": {
            "vocab": forward.vocabulary(),
            "width": forward.width(),
            "context": forward.context_length(),
            "original_table_sha256": table_sha256(&original_table),
        },
        "capture": {
            "train_samples": train_samples.len(),
            "train_response_targets": train_identity.response_targets,
            "train_elapsed_ms": train_capture_elapsed_ms,
            "train_sha256": capture_sha256(&train_observations),
            "panel_samples": panel_samples.len(),
            "panel_response_targets": panel_identity.response_targets,
            "panel_elapsed_ms": panel_capture_elapsed_ms,
            "panel_sha256": capture_sha256(&panel_observations),
            "shared_panel_elapsed_ms": shared_capture_elapsed_ms,
            "shared_panel_sha256": capture_sha256(&shared_observations),
        },
        "thresholds": census,
        "fit": {
            "median": fit_json(&median, &median.table),
            "mean_centroid": fit_json(&mean, &mean.table),
            "shuffled_target": {
                "fitted_tokens": shuffled.fitted_tokens.len(),
                "observations": shuffled_fit.observations,
                "changed_targets": shuffled.changed_targets,
                "table_sha256": table_sha256(&shuffled_fit.table),
                "changed_coordinates": shuffled_fit.changed_coordinates,
                "displacement_l1": shuffled_fit.displacement_l1,
            },
            "shared_refit_artifact_sha256": uor_r4_discrete::corpus::sha256(&shared_artifact),
        },
        "arms": {
            "original": original,
            "decoupled_identical": identical,
            "output_median": median_eval,
            "output_mean_centroid": mean_eval,
            "shuffled_target": shuffled_eval,
            "shared_median_diagnostic": shared_eval,
        },
        "comparisons": {
            "median_top1_vs_original": median_vs_original,
            "median_top1_vs_shuffled": median_vs_shuffled,
            "identity_decision_sha_match": identity_exact,
            "captured_original_decision_sha256": original_decision_sha256,
            "shared_vs_decoupled_changed_decisions": decisions_changed(&shared_eval, &median_eval),
        },
        "baselines": baseline,
        "power": {
            "minimum_refitted_rows": MINIMUM_REFITTED_ROWS,
            "minimum_refitted_observations": MINIMUM_REFITTED_OBSERVATIONS,
            "minimum_panel_targets": MINIMUM_PANEL_TARGETS,
            "eligible": power,
        },
        "bar": {
            "identity_exact": identity_exact,
            "median_beats_original": median_vs_original["passes"],
            "median_beats_shuffled": median_vs_shuffled["passes"],
            "top8_not_below_original": median_eval.top8_correct >= original.top8_correct,
            "eligible": power,
            "artifact_outcome": artifact_outcome,
            "overall_decision": "PENDING_TWO_ARTIFACT_REPEAT_AGGREGATE",
        },
        "execution": {
            "threads": rayon::current_num_threads(),
            "wall_elapsed_ms": elapsed_ms(started),
            "peak_rss_kib": peak_rss_kib(),
        },
    });
    finish_report(&mut report);
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn decisions_changed(left: &ReadoutEvaluation, right: &ReadoutEvaluation) -> u64 {
    left.decision_tokens
        .iter()
        .zip(&right.decision_tokens)
        .filter(|(left, right)| left != right)
        .count() as u64
}

fn validate_target_inventory(
    identity: &CpuCorpusIdentity,
    samples: usize,
    label: &str,
) -> Result<()> {
    if samples != identity.response_targets {
        return Err(format!(
            "{label} causal samples {samples}, response targets {}",
            identity.response_targets
        ));
    }
    Ok(())
}

fn decision_matches_capture(evaluation: &ReadoutEvaluation, captured_tokens: &[u32]) -> bool {
    evaluation.changed_decisions == 0 && evaluation.decision_sha256 == token_sha256(captured_tokens)
}

fn token_sha256(tokens: &[u32]) -> String {
    let bytes = tokens
        .iter()
        .flat_map(|token| token.to_le_bytes())
        .collect::<Vec<_>>();
    uor_r4_discrete::corpus::sha256(&bytes)
}

#[allow(clippy::too_many_arguments)]
fn early_artifact_report(
    artifact_path: &Path,
    artifact_sha256: &str,
    train_paths: &TrainPaths,
    train_hashes: &CorpusHashes,
    train_identity: &CpuCorpusIdentity,
    panel: Option<(&PanelPaths, &CorpusHashes, &CpuCorpusIdentity)>,
    artifact_outcome: &str,
    status: &str,
    reason: &str,
    details: serde_json::Value,
    started: Instant,
) -> serde_json::Value {
    serde_json::json!({
        "schema": "uor-r4.cpu-readout-refit/1",
        "report_kind": "artifact",
        "status": status,
        "substrate": SUBSTRATE,
        "qualification": QUALIFICATION,
        "seed": EXPERIMENT_SEED,
        "artifact_outcome": artifact_outcome,
        "reason": reason,
        "details": details,
        "inputs": inputs_json(
            artifact_path,
            artifact_sha256,
            train_paths,
            train_hashes,
            train_identity,
            panel,
        ),
        "bar": {
            "artifact_outcome": artifact_outcome,
            "overall_decision": "PENDING_TWO_ARTIFACT_REPEAT_AGGREGATE",
        },
        "execution": {
            "threads": rayon::current_num_threads(),
            "wall_elapsed_ms": elapsed_ms(started),
            "peak_rss_kib": peak_rss_kib(),
        },
    })
}

fn finish_report(report: &mut serde_json::Value) {
    report["deterministic_core_sha256"] =
        serde_json::Value::String(normalized_report_sha256(report));
}

fn normalized_report_sha256(report: &serde_json::Value) -> String {
    uor_r4_discrete::corpus::sha256(normalize_report(report).to_string().as_bytes())
}

fn normalize_report(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(object) => serde_json::Value::Object(
            object
                .iter()
                .filter(|(key, _)| {
                    !matches!(
                        key.as_str(),
                        "execution"
                            | "elapsed_ms"
                            | "train_elapsed_ms"
                            | "panel_elapsed_ms"
                            | "shared_panel_elapsed_ms"
                            | "wall_elapsed_ms"
                            | "peak_rss_kib"
                            | "deterministic_core_sha256"
                    )
                })
                .map(|(key, value)| (key.clone(), normalize_report(value)))
                .collect(),
        ),
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.iter().map(normalize_report).collect())
        }
        value => value.clone(),
    }
}

fn aggregate_reports(args: &[String]) -> Result<()> {
    if args.len() != 4 {
        return Err("aggregate requires four artifact reports".to_owned());
    }
    let reports = args
        .iter()
        .map(|path| {
            let bytes =
                std::fs::read(path).map_err(|error| format!("read report {path}: {error}"))?;
            serde_json::from_slice(&bytes).map_err(|error| format!("parse report {path}: {error}"))
        })
        .collect::<Result<Vec<serde_json::Value>>>()?;
    let mut aggregate = aggregate_values(&reports)?;
    aggregate["report_kind"] = serde_json::Value::String("aggregate".to_owned());
    aggregate["reports"] = serde_json::Value::Array(
        args.iter()
            .map(|path| serde_json::Value::String(path.clone()))
            .collect(),
    );
    finish_report(&mut aggregate);
    println!(
        "{}",
        serde_json::to_string_pretty(&aggregate).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn aggregate_values(reports: &[serde_json::Value]) -> Result<serde_json::Value> {
    if reports.len() != 4 {
        return Err("aggregate requires four artifact reports".to_owned());
    }
    let mut groups = BTreeMap::<String, Vec<(String, String)>>::new();
    for report in reports {
        if report["schema"].as_str() != Some("uor-r4.cpu-readout-refit/1")
            || report["report_kind"].as_str() != Some("artifact")
            || report["seed"].as_u64() != Some(EXPERIMENT_SEED)
        {
            return Err("aggregate report identity differs".to_owned());
        }
        let artifact = report["inputs"]["artifact_sha256"]
            .as_str()
            .ok_or_else(|| "aggregate report has no artifact hash".to_owned())?
            .to_owned();
        let outcome = report["bar"]["artifact_outcome"]
            .as_str()
            .ok_or_else(|| "aggregate report has no artifact outcome".to_owned())?
            .to_owned();
        groups
            .entry(artifact)
            .or_default()
            .push((normalized_report_sha256(report), outcome));
    }
    if groups.len() != 2 || groups.values().any(|reports| reports.len() != 2) {
        return Err("aggregate requires two artifacts with two reports each".to_owned());
    }
    let repeat_exact = groups
        .values()
        .all(|reports| reports[0].0 == reports[1].0 && reports[0].1 == reports[1].1);
    let outcomes = groups
        .iter()
        .map(|(artifact, reports)| {
            (
                artifact.clone(),
                reports[0].1.clone(),
                reports
                    .iter()
                    .map(|report| report.0.clone())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    let decision = if !repeat_exact
        || outcomes
            .iter()
            .any(|(_, outcome, _)| outcome == "CONTROL_FAULT")
    {
        "FINAL_CONTROL_FAULT"
    } else if outcomes
        .iter()
        .any(|(_, outcome, _)| outcome == "INSUFFICIENT_POWER")
    {
        "FINAL_INSUFFICIENT_POWER"
    } else if outcomes
        .iter()
        .all(|(_, outcome, _)| outcome == "ARTIFACT_KEEP")
    {
        "FINAL_KEEP"
    } else {
        "FINAL_REJECT"
    };
    Ok(serde_json::json!({
        "schema": "uor-r4.cpu-readout-refit-aggregate/1",
        "status": "COMPLETE",
        "substrate": SUBSTRATE,
        "qualification": QUALIFICATION,
        "seed": EXPERIMENT_SEED,
        "artifacts": outcomes
            .into_iter()
            .map(|(artifact, outcome, hashes)| {
                serde_json::json!({
                    "artifact_sha256": artifact,
                    "artifact_outcome": outcome,
                    "deterministic_core_sha256": hashes,
                })
            })
            .collect::<Vec<_>>(),
        "repeat_exact": repeat_exact,
        "overall_decision": decision,
    }))
}

fn capture_observations(
    artifact: &[u8],
    samples: &[RankMarginSample],
) -> Result<Vec<ReadoutObservation>> {
    let forward = CpuFbsForward::from_artifact(artifact).map_err(|error| error.to_string())?;
    samples
        .par_iter()
        .map_init(
            || forward.clone(),
            |forward, sample| -> Result<ReadoutObservation> {
                forward.reset();
                let mut step = None;
                for &token in &sample.input {
                    step = Some(
                        forward
                            .step(token, CpuFbsHead::Writer)
                            .map_err(|error| error.to_string())?,
                    );
                }
                let step = step.ok_or_else(|| "readout capture input is empty".to_owned())?;
                Ok(ReadoutObservation {
                    query_q12: step.query_q12,
                    target: sample.target,
                    kept_cells: step.kept_cells,
                    original_token: step.token,
                })
            },
        )
        .collect()
}

fn shuffled_table(
    original: &[i16],
    vocab: usize,
    width: usize,
    observations: &[ReadoutObservation],
    minimum: usize,
) -> Result<RefittedOutputTable> {
    refit_output_table(
        original,
        vocab,
        width,
        observations,
        ReadoutRefitMethod::Median,
        minimum,
    )
    .map_err(|error| error.to_string())
}

fn count_table_baselines(
    train: &[RankMarginSample],
    panel: &[RankMarginSample],
) -> serde_json::Value {
    let mut unigram = BTreeMap::<u32, u64>::new();
    let mut successor = BTreeMap::<(u32, u32), BTreeMap<u32, u64>>::new();
    for sample in train {
        *unigram.entry(sample.target).or_default() += 1;
        if sample.input.len() >= 2 {
            let key = (
                sample.input[sample.input.len() - 2],
                sample.input[sample.input.len() - 1],
            );
            *successor
                .entry(key)
                .or_default()
                .entry(sample.target)
                .or_default() += 1;
        }
    }
    let unigram_best = best_token(&unigram);
    let successor_best = successor
        .iter()
        .map(|(&key, counts)| (key, best_token(counts)))
        .collect::<BTreeMap<_, _>>();
    let mut unigram_correct = 0u64;
    let mut successor_correct = 0u64;
    let mut successor_backed_off = 0u64;
    for sample in panel {
        unigram_correct += u64::from(sample.target == unigram_best);
        let key = sample
            .input
            .len()
            .checked_sub(2)
            .map(|index| (sample.input[index], sample.input[index + 1]));
        let (prediction, backed_off) = key
            .and_then(|key| successor_best.get(&key).map(|&token| (token, false)))
            .unwrap_or((unigram_best, true));
        successor_correct += u64::from(sample.target == prediction);
        successor_backed_off += u64::from(backed_off);
    }
    serde_json::json!({
        "unigram_top1_correct": unigram_correct,
        "unigram_token": unigram_best,
        "t2_successor_top1_correct": successor_correct,
        "t2_backed_off_rows": successor_backed_off,
        "panel_targets": panel.len(),
    })
}

fn best_token(counts: &BTreeMap<u32, u64>) -> u32 {
    let mut best = (0u32, 0u64);
    for (&token, &count) in counts {
        if count > best.1 {
            best = (token, count);
        }
    }
    best.0
}

fn fit_json(fit: &RefittedOutputTable, table: &[i16]) -> serde_json::Value {
    serde_json::json!({
        "fitted_tokens": fit.fitted_tokens.len(),
        "observations": fit.observations,
        "changed_coordinates": fit.changed_coordinates,
        "displacement_l1": fit.displacement_l1,
        "maximum_row_displacement_l1": fit.maximum_row_displacement_l1,
        "table_sha256": table_sha256(table),
    })
}

fn capture_sha256(observations: &[ReadoutObservation]) -> String {
    let mut bytes = Vec::new();
    for observation in observations {
        bytes.extend_from_slice(&observation.target.to_le_bytes());
        bytes.extend_from_slice(&observation.original_token.to_le_bytes());
        bytes.extend_from_slice(&(observation.kept_cells.len() as u32).to_le_bytes());
        for &cell in &observation.kept_cells {
            bytes.extend_from_slice(&cell.to_le_bytes());
        }
        for &value in &observation.query_q12 {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    uor_r4_discrete::corpus::sha256(&bytes)
}

fn table_sha256(table: &[i16]) -> String {
    let bytes = table
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect::<Vec<_>>();
    uor_r4_discrete::corpus::sha256(&bytes)
}

struct CorpusHashes {
    tokens: String,
    mask: String,
    manifest: String,
    tokenizer: String,
}

struct Expected {
    train: CorpusHashes,
    panel: Option<CorpusHashes>,
    artifact: String,
}

impl Expected {
    fn from_flags(flags: &BTreeMap<String, String>, include_panel: bool) -> Result<Self> {
        let required = |name: &str| {
            flags
                .get(name)
                .cloned()
                .ok_or_else(|| format!("missing required --{name}"))
        };
        let tokenizer = required("expected-tokenizer")?;
        if tokenizer != EXPECTED_TOKENIZER_SHA256 {
            return Err(format!(
                "expected tokenizer {tokenizer}, registered {EXPECTED_TOKENIZER_SHA256}"
            ));
        }
        let train = CorpusHashes {
            tokens: required("expected-train-tokens")?,
            mask: required("expected-train-mask")?,
            manifest: required("expected-train-manifest")?,
            tokenizer: tokenizer.clone(),
        };
        let panel = include_panel
            .then(|| {
                Ok::<_, String>(CorpusHashes {
                    tokens: required("expected-panel-tokens")?,
                    mask: required("expected-panel-mask")?,
                    manifest: required("expected-panel-manifest")?,
                    tokenizer,
                })
            })
            .transpose()?;
        Ok(Self {
            train,
            panel,
            artifact: required("expected-artifact")?,
        })
    }
}

struct TrainPaths {
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
}

impl TrainPaths {
    fn new(tokens: &str, mask: &str, manifest: &str) -> Self {
        Self {
            tokens: tokens.into(),
            mask: mask.into(),
            manifest: manifest.into(),
        }
    }
}

struct PanelPaths {
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
}

impl PanelPaths {
    fn new(tokens: &str, mask: &str, manifest: &str) -> Self {
        Self {
            tokens: tokens.into(),
            mask: mask.into(),
            manifest: manifest.into(),
        }
    }
}

struct CorpusRead {
    hashes: CorpusHashes,
    manifest_bytes: Vec<u8>,
}

fn read_corpus(
    paths: &impl CorpusPaths,
    expected: &CorpusHashes,
    context: usize,
) -> Result<(CpuMaskedCorpus, CpuCorpusIdentity, CorpusRead)> {
    let tokens = read_checked(paths.tokens(), &expected.tokens)?;
    let mask = read_checked(paths.mask(), &expected.mask)?;
    let manifest = read_checked(paths.manifest(), &expected.manifest)?;
    let identity =
        validate_corpus_manifest(&manifest, &tokens, &mask).map_err(|error| error.to_string())?;
    if identity.tokenizer_sha256 != expected.tokenizer {
        return Err(format!(
            "{} tokenizer sha256 {}, expected {}",
            paths.manifest().display(),
            identity.tokenizer_sha256,
            expected.tokenizer
        ));
    }
    if identity.bos != EXPECTED_BOS {
        return Err(format!(
            "{} BOS {}, expected {}",
            paths.manifest().display(),
            identity.bos,
            EXPECTED_BOS
        ));
    }
    identity
        .validate_context(context)
        .map_err(|error| error.to_string())?;
    let corpus = CpuMaskedCorpus::from_bytes(&tokens, &mask, identity.vocab, context, identity.bos)
        .map_err(|error| error.to_string())?;
    Ok((
        corpus,
        identity,
        CorpusRead {
            hashes: CorpusHashes {
                tokens: expected.tokens.clone(),
                mask: expected.mask.clone(),
                manifest: expected.manifest.clone(),
                tokenizer: expected.tokenizer.clone(),
            },
            manifest_bytes: manifest,
        },
    ))
}

trait CorpusPaths {
    fn tokens(&self) -> &Path;
    fn mask(&self) -> &Path;
    fn manifest(&self) -> &Path;
}

impl CorpusPaths for TrainPaths {
    fn tokens(&self) -> &Path {
        &self.tokens
    }
    fn mask(&self) -> &Path {
        &self.mask
    }
    fn manifest(&self) -> &Path {
        &self.manifest
    }
}

impl CorpusPaths for PanelPaths {
    fn tokens(&self) -> &Path {
        &self.tokens
    }
    fn mask(&self) -> &Path {
        &self.mask
    }
    fn manifest(&self) -> &Path {
        &self.manifest
    }
}

fn inputs_json(
    artifact_path: &Path,
    artifact_sha: &str,
    train_paths: &TrainPaths,
    train_hashes: &CorpusHashes,
    train_identity: &CpuCorpusIdentity,
    panel: Option<(&PanelPaths, &CorpusHashes, &CpuCorpusIdentity)>,
) -> serde_json::Value {
    let mut value = serde_json::json!({
        "artifact": artifact_path,
        "artifact_sha256": artifact_sha,
        "train": {
            "tokens": train_paths.tokens,
            "tokens_sha256": train_hashes.tokens,
            "mask": train_paths.mask,
            "mask_sha256": train_hashes.mask,
            "manifest": train_paths.manifest,
            "manifest_sha256": train_hashes.manifest,
            "manifest_split": train_identity.split,
            "tokenizer_sha256": train_identity.tokenizer_sha256,
            "bos": train_identity.bos,
            "response_targets": train_identity.response_targets,
            "max_tokens": train_identity.max_tokens,
        },
    });
    if let Some((paths, hashes, identity)) = panel {
        value["panel"] = serde_json::json!({
            "tokens": paths.tokens,
            "tokens_sha256": hashes.tokens,
            "mask": paths.mask,
            "mask_sha256": hashes.mask,
            "manifest": paths.manifest,
            "manifest_sha256": hashes.manifest,
            "manifest_split": identity.split,
            "tokenizer_sha256": identity.tokenizer_sha256,
            "bos": identity.bos,
            "response_targets": identity.response_targets,
            "max_tokens": identity.max_tokens,
        });
    }
    value
}

fn validate_panel(manifest: &[u8], identity: &CpuCorpusIdentity) -> Result<()> {
    let value: serde_json::Value =
        serde_json::from_slice(manifest).map_err(|error| error.to_string())?;
    if value["seed"].as_u64() != Some(EXPERIMENT_SEED)
        || value["dialogue_protocol_version"].as_u64() != Some(2)
        || value["conversations"].as_u64() != Some(128)
        || value["cell"]["value"].as_str() != Some("dev")
        || value["cell"]["phrasing"].as_str() != Some("dev")
        || identity.split != "dev"
    {
        return Err(format!(
            "fresh panel identity differs from seed {EXPERIMENT_SEED}, dev,dev, protocol 2, 128"
        ));
    }
    Ok(())
}

fn read_checked(path: &Path, expected: &str) -> Result<Vec<u8>> {
    let bytes = std::fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let actual = uor_r4_discrete::corpus::sha256(&bytes);
    if actual != expected {
        return Err(format!(
            "{} sha256 {actual}, expected {expected}",
            path.display()
        ));
    }
    Ok(bytes)
}

fn parse_flags(args: &[String]) -> Result<BTreeMap<String, String>> {
    if !args.len().is_multiple_of(2) {
        return Err("flags require --name value pairs".to_owned());
    }
    let mut flags = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        let name = pair[0]
            .strip_prefix("--")
            .ok_or_else(|| format!("unknown argument {}", pair[0]))?;
        if !matches!(
            name,
            "expected-tokenizer"
                | "expected-train-tokens"
                | "expected-train-mask"
                | "expected-train-manifest"
                | "expected-panel-tokens"
                | "expected-panel-mask"
                | "expected-panel-manifest"
                | "expected-artifact"
        ) {
            return Err(format!("unknown flag --{name}"));
        }
        if flags.insert(name.to_owned(), pair[1].clone()).is_some() {
            return Err(format!("duplicate flag --{name}"));
        }
    }
    Ok(flags)
}

fn elapsed_ms(started: Instant) -> f64 {
    started.elapsed().as_secs_f64() * 1_000.0
}

fn peak_rss_kib() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status
        .lines()
        .find(|line| line.starts_with("VmHWM:"))?
        .split_whitespace()
        .nth(1)?;
    line.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact_report(artifact: &str, outcome: &str, elapsed: f64) -> serde_json::Value {
        serde_json::json!({
            "schema": "uor-r4.cpu-readout-refit/1",
            "report_kind": "artifact",
            "seed": EXPERIMENT_SEED,
            "inputs": {"artifact_sha256": artifact},
            "capture": {"train_elapsed_ms": elapsed},
            "bar": {"artifact_outcome": outcome},
            "execution": {"wall_elapsed_ms": elapsed, "peak_rss_kib": elapsed as u64},
        })
    }

    #[test]
    fn repeat_hash_ignores_timing_and_resource_fields() {
        let first = artifact_report("artifact-a", "ARTIFACT_KEEP", 1.0);
        let second = artifact_report("artifact-a", "ARTIFACT_KEEP", 99.0);
        assert_eq!(
            normalized_report_sha256(&first),
            normalized_report_sha256(&second)
        );
    }

    #[test]
    fn aggregate_requires_two_artifacts_and_two_matching_repeats() {
        let reports = vec![
            artifact_report("artifact-a", "ARTIFACT_KEEP", 1.0),
            artifact_report("artifact-a", "ARTIFACT_KEEP", 2.0),
            artifact_report("artifact-b", "ARTIFACT_KEEP", 3.0),
            artifact_report("artifact-b", "ARTIFACT_KEEP", 4.0),
        ];
        let aggregate = aggregate_values(&reports).unwrap();
        assert_eq!(aggregate["repeat_exact"], true);
        assert_eq!(aggregate["overall_decision"], "FINAL_KEEP");
    }

    #[test]
    fn aggregate_control_fault_takes_precedence_over_configuration_rejection() {
        let reports = vec![
            artifact_report("artifact-a", "CONTROL_FAULT", 1.0),
            artifact_report("artifact-a", "CONTROL_FAULT", 2.0),
            artifact_report("artifact-b", "ARTIFACT_REJECT", 3.0),
            artifact_report("artifact-b", "ARTIFACT_REJECT", 4.0),
        ];
        let aggregate = aggregate_values(&reports).unwrap();
        assert_eq!(aggregate["overall_decision"], "FINAL_CONTROL_FAULT");
    }

    #[test]
    fn aggregate_repeat_drift_is_a_control_fault() {
        let reports = vec![
            artifact_report("artifact-a", "ARTIFACT_KEEP", 1.0),
            artifact_report("artifact-a", "ARTIFACT_REJECT", 2.0),
            artifact_report("artifact-b", "ARTIFACT_KEEP", 3.0),
            artifact_report("artifact-b", "ARTIFACT_KEEP", 4.0),
        ];
        let aggregate = aggregate_values(&reports).unwrap();
        assert_eq!(aggregate["repeat_exact"], false);
        assert_eq!(aggregate["overall_decision"], "FINAL_CONTROL_FAULT");
    }
}
