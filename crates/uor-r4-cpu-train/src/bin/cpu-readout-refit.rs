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
        _ => Err(usage()),
    }
}

fn usage() -> String {
    concat!(
        "usage:\n",
        "  cpu-readout-refit preflight TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST ARTIFACT ",
        "--min-observations N --expected-train-tokens SHA --expected-train-mask SHA ",
        "--expected-train-manifest SHA --expected-artifact SHA\n",
        "  cpu-readout-refit run TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST PANEL_TOKENS ",
        "PANEL_MASK PANEL_MANIFEST ARTIFACT --seed 2789 --min-observations N ",
        "--expected-train-tokens SHA --expected-train-mask SHA --expected-train-manifest SHA ",
        "--expected-panel-tokens SHA --expected-panel-mask SHA ",
        "--expected-panel-manifest SHA --expected-artifact SHA"
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
    let minimum = parse_minimum(&flags)?;
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
    let capture_started = Instant::now();
    let observations = capture_observations(&artifact, &samples)?;
    let capture_elapsed_ms = elapsed_ms(capture_started);
    let census =
        observe_refit_thresholds(&observations, &THRESHOLDS).map_err(|error| error.to_string())?;
    let median = refit_output_table(
        forward.shared_table(),
        forward.vocabulary(),
        forward.width(),
        &observations,
        ReadoutRefitMethod::Median,
        minimum,
    )
    .map_err(|error| error.to_string())?;
    let mean = refit_output_table(
        forward.shared_table(),
        forward.vocabulary(),
        forward.width(),
        &observations,
        ReadoutRefitMethod::MeanCentroid,
        minimum,
    )
    .map_err(|error| error.to_string())?;
    let shuffled = shuffle_refit_targets(
        &observations,
        forward.vocabulary(),
        forward.width(),
        minimum,
    )
    .map_err(|error| error.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "uor-r4.cpu-readout-refit-preflight/1",
            "status": "COMPLETE",
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
            "selected_minimum_observations": minimum,
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
                        minimum,
                    )?.table
                ),
            },
            "power": {
                "minimum_refitted_rows": 100,
                "minimum_refitted_observations": 1_000,
                "eligible": median.fitted_tokens.len() >= 100 && median.observations >= 1_000,
            },
            "execution": {
                "threads": rayon::current_num_threads(),
                "wall_elapsed_ms": elapsed_ms(started),
                "peak_rss_kib": peak_rss_kib(),
            },
        }))
        .map_err(|error| error.to_string())?
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
    let minimum = parse_minimum(&flags)?;
    let seed = parse_seed(&flags)?;
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
    validate_panel(&panel_read.manifest_bytes, seed, &panel_identity)?;

    let train_samples = train.causal_samples().map_err(|error| error.to_string())?;
    let panel_samples = panel.causal_samples().map_err(|error| error.to_string())?;
    if panel_samples.len() < 1_000 {
        return Err("fresh panel has fewer than 1,000 targets".to_owned());
    }
    let train_capture_started = Instant::now();
    let train_observations = capture_observations(&artifact, &train_samples)?;
    let train_capture_elapsed_ms = elapsed_ms(train_capture_started);
    let census = observe_refit_thresholds(&train_observations, &THRESHOLDS)
        .map_err(|error| error.to_string())?;
    let original_table = forward.shared_table().to_vec();
    let median = refit_output_table(
        &original_table,
        forward.vocabulary(),
        forward.width(),
        &train_observations,
        ReadoutRefitMethod::Median,
        minimum,
    )
    .map_err(|error| error.to_string())?;
    let mean = refit_output_table(
        &original_table,
        forward.vocabulary(),
        forward.width(),
        &train_observations,
        ReadoutRefitMethod::MeanCentroid,
        minimum,
    )
    .map_err(|error| error.to_string())?;
    let shuffled = shuffle_refit_targets(
        &train_observations,
        forward.vocabulary(),
        forward.width(),
        minimum,
    )
    .map_err(|error| error.to_string())?;
    let shuffled_fit = shuffled_table(
        &original_table,
        forward.vocabulary(),
        forward.width(),
        &shuffled.observations,
        minimum,
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

    let identity_exact =
        identical.decision_sha256 == original.decision_sha256 && identical.changed_decisions == 0;
    let power = median.fitted_tokens.len() >= 100
        && median.observations >= 1_000
        && panel_samples.len() >= 1_000;
    let primary_pass = identity_exact
        && power
        && median_vs_original["passes"] == true
        && median_vs_shuffled["passes"] == true
        && median_eval.top8_correct >= original.top8_correct;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "uor-r4.cpu-readout-refit/1",
            "status": "COMPLETE",
            "substrate": SUBSTRATE,
            "qualification": QUALIFICATION,
            "claim": "fixed-input, output-only nearest-token readout refit on exact CPU Q12 queries",
            "seed": seed,
            "inputs": inputs_json(
                artifact_path,
                &expected.artifact,
                &train_paths,
                &train_read.hashes,
                &train_identity,
                Some((&panel_paths, &panel_read.hashes, &panel_identity)),
            ),
            "settings": {
                "minimum_observations": minimum,
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
                "train_elapsed_ms": train_capture_elapsed_ms,
                "train_sha256": capture_sha256(&train_observations),
                "panel_samples": panel_samples.len(),
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
                    "observations": train_observations.len(),
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
                "shared_vs_decoupled_changed_decisions": decisions_changed(&shared_eval, &median_eval),
            },
            "baselines": baseline,
            "power": {
                "minimum_refitted_rows": 100,
                "minimum_refitted_observations": 1_000,
                "minimum_panel_targets": 1_000,
                "eligible": power,
            },
            "bar": {
                "identity_exact": identity_exact,
                "median_beats_original": median_vs_original["passes"],
                "median_beats_shuffled": median_vs_shuffled["passes"],
                "top8_not_below_original": median_eval.top8_correct >= original.top8_correct,
                "eligible": power,
                "outcome": if !power {
                    "INSUFFICIENT_POWER"
                } else if primary_pass {
                    "KEEP"
                } else {
                    "REJECT"
                },
            },
            "execution": {
                "threads": rayon::current_num_threads(),
                "wall_elapsed_ms": elapsed_ms(started),
                "peak_rss_kib": peak_rss_kib(),
            },
        }))
        .map_err(|error| error.to_string())?
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
        let train = CorpusHashes {
            tokens: required("expected-train-tokens")?,
            mask: required("expected-train-mask")?,
            manifest: required("expected-train-manifest")?,
        };
        let panel = include_panel
            .then(|| {
                Ok::<_, String>(CorpusHashes {
                    tokens: required("expected-panel-tokens")?,
                    mask: required("expected-panel-mask")?,
                    manifest: required("expected-panel-manifest")?,
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
        });
    }
    value
}

fn validate_panel(manifest: &[u8], seed: u64, identity: &CpuCorpusIdentity) -> Result<()> {
    let value: serde_json::Value =
        serde_json::from_slice(manifest).map_err(|error| error.to_string())?;
    if value["seed"].as_u64() != Some(seed)
        || value["dialogue_protocol_version"].as_u64() != Some(2)
        || value["conversations"].as_u64() != Some(128)
        || value["cell"]["value"].as_str() != Some("dev")
        || value["cell"]["phrasing"].as_str() != Some("dev")
        || identity.split != "dev"
    {
        return Err(
            "fresh panel identity differs from seed 2789, dev,dev, protocol 2, 128".to_owned(),
        );
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
            "min-observations"
                | "seed"
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

fn parse_minimum(flags: &BTreeMap<String, String>) -> Result<usize> {
    flags
        .get("min-observations")
        .ok_or_else(|| "missing required --min-observations".to_owned())?
        .parse::<usize>()
        .map_err(|error| format!("invalid --min-observations: {error}"))
}

fn parse_seed(flags: &BTreeMap<String, String>) -> Result<u64> {
    flags
        .get("seed")
        .ok_or_else(|| "missing required --seed".to_owned())?
        .parse::<u64>()
        .map_err(|error| format!("invalid --seed: {error}"))
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
