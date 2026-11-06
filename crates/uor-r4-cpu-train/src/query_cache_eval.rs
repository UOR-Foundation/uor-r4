//! D84 evaluator for immutable teacher-forced FBS query caches.
//!
//! The GPU scorer captures exact Q12 queries and decisions once. This module
//! verifies that evidence, streams TRAIN into candidate and matched-control
//! output tables, and evaluates both against frozen panel queries on CPU.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{invalid, CpuTrainError};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const QUERY_CACHE_SCHEMA: &str = "uor-r4.fbs-query-cache/1";
const QUERY_CACHE_SEAL_SCHEMA: &str = "uor-r4.fbs-query-cache-seal/1";
const EVALUATION_SCHEMA: &str = "uor-r4.cpu-query-cache-evaluation/1";
const ROW_MAGIC: &[u8; 8] = b"UORQROW1";
const Q12_SCALE: f32 = 4096.0;
const PINNED_TOKENIZER_SHA256: &str =
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";
const QUERY_BLOCK_ROWS: usize = 64;

pub const REGISTERED_MINIMUM_OBSERVATIONS: usize = 8;
pub const REGISTERED_MIN_FITTED_ROWS: usize = 100;
pub const REGISTERED_MIN_FIT_OBSERVATIONS: usize = 10_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryCacheManifest {
    pub schema: String,
    pub checkpoint: String,
    pub data_root: String,
    pub tokenizer_sha256: String,
    pub checkpoint_files: BTreeMap<String, String>,
    pub run_config: Option<String>,
    pub run_config_sha256: Option<String>,
    pub table_file: String,
    pub table_sha256: String,
    pub vocab: usize,
    pub width: usize,
    pub query_space: String,
    pub splits: Vec<QueryCacheSplit>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryCacheSplit {
    pub split: String,
    pub label: String,
    pub memory: bool,
    pub context: usize,
    pub rows: usize,
    pub tokens_sha256: String,
    pub mask_sha256: String,
    pub query_file: String,
    pub query_sha256: String,
    pub rows_file: String,
    pub rows_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct QueryCacheSeal {
    schema: String,
    manifest_sha256: String,
}

#[derive(Clone, Debug)]
struct CachedRows {
    targets: Vec<u32>,
    positions: Vec<u32>,
    nearest: Vec<u32>,
    writer: Vec<u32>,
    opens: Vec<u8>,
    copies: Vec<u8>,
}

struct QueryCache {
    root: PathBuf,
    manifest: QueryCacheManifest,
    manifest_sha256: String,
    table: Vec<i16>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReadoutCounts {
    pub rows: usize,
    pub top1_correct: usize,
    pub top8_correct: usize,
    pub changed_from_frozen: usize,
    pub decision_sha256: String,
}

#[derive(Clone, Debug)]
struct Decisions {
    counts: ReadoutCounts,
    top1_rows: Vec<bool>,
    top8_rows: Vec<bool>,
    tokens: Vec<u32>,
}

#[derive(Clone, Debug)]
struct FittedTables {
    centroid: Vec<i16>,
    shuffled: Vec<i16>,
    fitted_rows: usize,
    observations: usize,
    control_strata: usize,
    singleton_control_targets: usize,
    changed_coordinates: u64,
    displacement_l1: u64,
    shuffled_changed_targets: usize,
    shuffled_changed_coordinates: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct CentroidEvaluationRequest {
    pub train_split: String,
    pub panel_splits: Vec<String>,
    pub memory: bool,
    pub minimum_observations: usize,
    pub minimum_fitted_rows: usize,
    pub minimum_fit_observations: usize,
    pub expected_train_tokens_sha256: Option<String>,
    pub expected_train_mask_sha256: Option<String>,
    pub expected_rows: Option<usize>,
    pub expected_row_identity: Option<String>,
}

impl QueryCache {
    fn open(root: &Path) -> Result<Self> {
        let root = fs::canonicalize(root)
            .map_err(|error| invalid(format!("{}: {error}", root.display())))?;
        let manifest_bytes = fs::read(root.join("manifest.json"))
            .map_err(|error| invalid(format!("query cache manifest: {error}")))?;
        let seal_bytes = fs::read(root.join("seal.json"))
            .map_err(|error| invalid(format!("query cache seal: {error}")))?;
        let seal: QueryCacheSeal = serde_json::from_slice(&seal_bytes)
            .map_err(|error| invalid(format!("query cache seal: {error}")))?;
        let manifest_sha256 = sha256(&manifest_bytes);
        if seal.schema != QUERY_CACHE_SEAL_SCHEMA || seal.manifest_sha256 != manifest_sha256 {
            return Err(invalid("query cache manifest seal differs"));
        }
        let manifest: QueryCacheManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|error| invalid(format!("query cache manifest: {error}")))?;
        if manifest.schema != QUERY_CACHE_SCHEMA
            || manifest.query_space != "q12-f32"
            || manifest.tokenizer_sha256 != PINNED_TOKENIZER_SHA256
            || manifest.vocab == 0
            || manifest.width == 0
            || manifest.splits.is_empty()
        {
            return Err(invalid("query cache manifest identity differs"));
        }
        let mut split_keys = BTreeSet::new();
        for split in &manifest.splits {
            if split.context == 0 || !split_keys.insert((&split.split, split.memory)) {
                return Err(invalid("query cache split inventory differs"));
            }
        }
        let table_bytes = read_checked(&root, &manifest.table_file, &manifest.table_sha256)?;
        let table_f32 = decode_f32(&table_bytes)?;
        if table_f32.len() != manifest.vocab.saturating_mul(manifest.width) {
            return Err(invalid("query cache table shape differs"));
        }
        let table = table_f32
            .into_iter()
            .map(f32_to_q12)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            root,
            manifest,
            manifest_sha256,
            table,
        })
    }

    fn split(&self, name: &str, memory: bool) -> Result<(&QueryCacheSplit, CachedRows)> {
        let metadata = self
            .manifest
            .splits
            .iter()
            .find(|split| split.split == name && split.memory == memory)
            .ok_or_else(|| {
                invalid(format!(
                    "query cache has no {name} split with memory={memory}"
                ))
            })?;
        let row_bytes = read_checked(&self.root, &metadata.rows_file, &metadata.rows_sha256)?;
        let rows = decode_cached_rows(&row_bytes, metadata.rows, self.manifest.vocab)?;
        Ok((metadata, rows))
    }
}

pub fn evaluate_query_cache(
    root: &Path,
    request: &CentroidEvaluationRequest,
) -> Result<serde_json::Value> {
    validate_request(request)?;
    let cache = QueryCache::open(root)?;
    let (train_metadata, train_rows) = cache.split(&request.train_split, request.memory)?;
    let train_identity_matches = request
        .expected_train_tokens_sha256
        .as_ref()
        .is_none_or(|expected| expected == &train_metadata.tokens_sha256)
        && request
            .expected_train_mask_sha256
            .as_ref()
            .is_none_or(|expected| expected == &train_metadata.mask_sha256);

    let started = std::time::Instant::now();
    let fitted = fit_streamed_tables(&cache, train_metadata, &train_rows, request)?;
    let power_ok = fitted.fitted_rows >= request.minimum_fitted_rows
        && fitted.observations >= request.minimum_fit_observations;
    let shuffle_ok = fitted.shuffled_changed_targets > 0 && fitted.shuffled_changed_coordinates > 0;

    let mut panels = Vec::new();
    let mut identities = Vec::new();
    let mut original_top1 = Vec::new();
    let mut centroid_top1 = Vec::new();
    let mut shuffled_top1 = Vec::new();
    let mut original_top8 = Vec::new();
    let mut centroid_top8 = Vec::new();
    let mut shuffled_top8 = Vec::new();
    let mut pooled_original_tokens = Vec::new();
    let mut pooled_centroid_tokens = Vec::new();
    let mut pooled_shuffled_tokens = Vec::new();
    let mut identity_exact = true;

    for name in &request.panel_splits {
        let (metadata, rows) = cache.split(name, request.memory)?;
        let queries = load_q12_queries(&cache.root, metadata, cache.manifest.width)?;
        let identity = row_identity(&rows);
        let original = evaluate_table(
            &queries,
            &rows.targets,
            &rows,
            &cache.table,
            cache.manifest.vocab,
            cache.manifest.width,
        )?;
        let centroid = evaluate_table(
            &queries,
            &rows.targets,
            &rows,
            &fitted.centroid,
            cache.manifest.vocab,
            cache.manifest.width,
        )?;
        let shuffled = evaluate_table(
            &queries,
            &rows.targets,
            &rows,
            &fitted.shuffled,
            cache.manifest.vocab,
            cache.manifest.width,
        )?;
        let panel_identity_exact = original
            .tokens
            .iter()
            .zip(&rows.nearest)
            .all(|(left, right)| left == right);
        identity_exact &= panel_identity_exact;
        original_top1.extend_from_slice(&original.top1_rows);
        centroid_top1.extend_from_slice(&centroid.top1_rows);
        shuffled_top1.extend_from_slice(&shuffled.top1_rows);
        original_top8.extend_from_slice(&original.top8_rows);
        centroid_top8.extend_from_slice(&centroid.top8_rows);
        shuffled_top8.extend_from_slice(&shuffled.top8_rows);
        pooled_original_tokens.extend_from_slice(&original.tokens);
        pooled_centroid_tokens.extend_from_slice(&centroid.tokens);
        pooled_shuffled_tokens.extend_from_slice(&shuffled.tokens);
        identities.push((name.clone(), identity));
        panels.push(serde_json::json!({
            "split": name,
            "rows": rows.targets.len(),
            "tokens_sha256": &metadata.tokens_sha256,
            "mask_sha256": &metadata.mask_sha256,
            "row_identity_sha256": hex(&identities.last().expect("panel identity").1),
            "opening_rows": rows.opens.iter().filter(|&&flag| flag == 1).count(),
            "copy_rows": rows.copies.iter().filter(|&&flag| flag == 1).count(),
            "writer_differs_from_nearest": rows.writer.iter().zip(&rows.nearest).filter(|(left, right)| left != right).count(),
            "identity": {
                "exact": panel_identity_exact,
                "decision_sha256": original.counts.decision_sha256,
            },
            "original": original.counts,
            "centroid": centroid.counts,
            "exact_count_shuffled_target": shuffled.counts,
            "centroid_vs_original": paired(&centroid.top1_rows, &original.top1_rows),
            "centroid_vs_exact_count_shuffled": paired(&centroid.top1_rows, &shuffled.top1_rows),
            "centroid_top8_not_below_original": centroid.counts.top8_correct >= original.counts.top8_correct,
        }));
    }

    let pooled_identity = pooled_identity(&identities);
    let identity_registered = request
        .expected_row_identity
        .as_ref()
        .is_none_or(|expected| expected == &pooled_identity);
    let rows_registered = request
        .expected_rows
        .is_none_or(|expected| expected == pooled_original_tokens.len());
    let paired_original = paired(&centroid_top1, &original_top1);
    let paired_shuffled = paired(&centroid_top1, &shuffled_top1);
    let centroid_top1_correct = centroid_top1.iter().filter(|&&hit| hit).count();
    let original_top1_correct = original_top1.iter().filter(|&&hit| hit).count();
    let shuffled_top1_correct = shuffled_top1.iter().filter(|&&hit| hit).count();
    let original_top8_correct = original_top8.iter().filter(|&&hit| hit).count();
    let centroid_top8_correct = centroid_top8.iter().filter(|&&hit| hit).count();
    let shuffled_top8_correct = shuffled_top8.iter().filter(|&&hit| hit).count();
    let top8_holds = centroid_top8_correct >= original_top8_correct;
    let bar_passes = identity_exact
        && identity_registered
        && rows_registered
        && power_ok
        && shuffle_ok
        && top8_holds
        && centroid_top1_correct > original_top1_correct
        && centroid_top1_correct > shuffled_top1_correct
        && paired_original["passes"].as_bool() == Some(true)
        && paired_shuffled["passes"].as_bool() == Some(true);
    let status = if !identity_exact
        || !train_identity_matches
        || !identity_registered
        || !rows_registered
        || !shuffle_ok
    {
        "CONTROL_FAULT"
    } else if !power_ok {
        "INSUFFICIENT_POWER"
    } else if bar_passes {
        "KEEP"
    } else {
        "REJECT"
    };

    Ok(serde_json::json!({
        "schema": EVALUATION_SCHEMA,
        "status": status,
        "cache": {
            "root": cache.root,
            "manifest_sha256": cache.manifest_sha256,
            "checkpoint": cache.manifest.checkpoint,
            "checkpoint_files": &cache.manifest.checkpoint_files,
            "run_config": &cache.manifest.run_config,
            "run_config_sha256": &cache.manifest.run_config_sha256,
            "data_root": &cache.manifest.data_root,
            "tokenizer_sha256": &cache.manifest.tokenizer_sha256,
            "table_sha256": &cache.manifest.table_sha256,
            "vocab": cache.manifest.vocab,
            "width": cache.manifest.width,
            "query_space": &cache.manifest.query_space,
        },
        "request": request,
        "train": {
            "split": &request.train_split,
            "memory": request.memory,
            "rows": train_rows.targets.len(),
            "context": train_metadata.context,
            "tokens_sha256": &train_metadata.tokens_sha256,
            "mask_sha256": &train_metadata.mask_sha256,
            "expected_tokens_sha256": request.expected_train_tokens_sha256,
            "expected_mask_sha256": request.expected_train_mask_sha256,
            "identity_matches": train_identity_matches,
        },
        "workers": rayon::current_num_threads(),
        "wall_ms": started.elapsed().as_millis(),
        "fit": {
            "fitted_rows": fitted.fitted_rows,
            "observations": fitted.observations,
            "minimum_fitted_rows": request.minimum_fitted_rows,
            "minimum_fit_observations": request.minimum_fit_observations,
            "power_ok": power_ok,
            "control_strategy": "exact-count-stratified-cycle",
            "control_strata": fitted.control_strata,
            "singleton_control_targets": fitted.singleton_control_targets,
            "changed_coordinates": fitted.changed_coordinates,
            "displacement_l1": fitted.displacement_l1,
            "shuffled_changed_targets": fitted.shuffled_changed_targets,
            "shuffled_changed_coordinates": fitted.shuffled_changed_coordinates,
        },
        "pooled": {
            "rows": pooled_original_tokens.len(),
            "row_identity_sha256": pooled_identity,
            "expected_rows": request.expected_rows,
            "expected_row_identity": request.expected_row_identity,
            "rows_match": rows_registered,
            "identity_matches": identity_registered,
            "original_top1": original_top1_correct,
            "centroid_top1": centroid_top1_correct,
            "exact_count_shuffled_top1": shuffled_top1_correct,
            "original_top8": original_top8_correct,
            "centroid_top8": centroid_top8_correct,
            "exact_count_shuffled_top8": shuffled_top8_correct,
            "top8_holds": top8_holds,
            "centroid_vs_original": paired_original,
            "centroid_vs_exact_count_shuffled": paired_shuffled,
            "decision_sha256": sha256(
                &pooled_original_tokens
                    .iter()
                    .chain(&pooled_centroid_tokens)
                    .chain(&pooled_shuffled_tokens)
                    .flat_map(|token| token.to_le_bytes())
                    .collect::<Vec<_>>(),
            ),
            "bar_passes": bar_passes,
        },
        "panels": panels,
    }))
}

pub fn aggregate_query_cache_reports(reports: &[serde_json::Value]) -> Result<serde_json::Value> {
    if reports.len() != 2 {
        return Err(invalid(
            "query cache aggregate needs exactly two seed reports",
        ));
    }
    let mut manifests = BTreeSet::new();
    let mut seeds = Vec::new();
    let mut statuses = Vec::new();
    let mut binding = None;
    for report in reports {
        if report["schema"] != EVALUATION_SCHEMA {
            return Err(invalid("query cache aggregate report schema differs"));
        }
        let status = report["status"]
            .as_str()
            .ok_or_else(|| invalid("query cache aggregate report has no status"))?;
        if !matches!(
            status,
            "KEEP" | "REJECT" | "INSUFFICIENT_POWER" | "CONTROL_FAULT"
        ) {
            return Err(invalid("query cache aggregate report status differs"));
        }
        let manifest = report["cache"]["manifest_sha256"]
            .as_str()
            .ok_or_else(|| invalid("query cache aggregate manifest identity is absent"))?;
        if !manifests.insert(manifest.to_owned()) {
            return Err(invalid("query cache aggregate repeats a cache"));
        }
        if report["pooled"]["bar_passes"].as_bool() != Some(status == "KEEP") {
            return Err(invalid("query cache aggregate status and bar differ"));
        }
        if report["pooled"]["rows_match"].as_bool() != Some(true)
            || report["pooled"]["identity_matches"].as_bool() != Some(true)
        {
            return Err(invalid("query cache aggregate pooled identity differs"));
        }
        let panels = report["panels"]
            .as_array()
            .ok_or_else(|| invalid("query cache aggregate panels are absent"))?;
        if !panels
            .iter()
            .all(|panel| panel["identity"]["exact"].as_bool() == Some(true))
        {
            return Err(invalid("query cache aggregate panel identity differs"));
        }
        let panel_names = panels
            .iter()
            .map(|panel| {
                panel["split"]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| invalid("query cache aggregate panel name is absent"))
            })
            .collect::<Result<Vec<_>>>()?;
        let request = &report["request"];
        let rows = request["expected_rows"]
            .as_u64()
            .ok_or_else(|| invalid("query cache aggregate expected rows are absent"))?;
        let identity = request["expected_row_identity"]
            .as_str()
            .ok_or_else(|| invalid("query cache aggregate expected identity is absent"))?
            .to_owned();
        let train_tokens = request["expected_train_tokens_sha256"]
            .as_str()
            .ok_or_else(|| invalid("query cache aggregate TRAIN token pin is absent"))?
            .to_owned();
        let train_mask = request["expected_train_mask_sha256"]
            .as_str()
            .ok_or_else(|| invalid("query cache aggregate TRAIN mask pin is absent"))?
            .to_owned();
        if report["pooled"]["rows"].as_u64() != Some(rows)
            || report["pooled"]["row_identity_sha256"].as_str() != Some(identity.as_str())
        {
            return Err(invalid(
                "query cache aggregate pooled request binding differs",
            ));
        }
        let current = (
            rows,
            identity,
            train_tokens,
            train_mask,
            request["memory"].as_bool(),
            panel_names,
        );
        if binding.as_ref().is_some_and(|binding| binding != &current) {
            return Err(invalid("query cache aggregate seed bindings differ"));
        }
        binding = Some(current);
        seeds.push(serde_json::json!({
            "cache_manifest_sha256": manifest,
            "checkpoint": report["cache"]["checkpoint"],
            "status": status,
            "fit": report["fit"],
            "pooled": report["pooled"],
        }));
        statuses.push(status);
    }
    let status = if statuses.contains(&"CONTROL_FAULT") {
        "CONTROL_FAULT"
    } else if statuses.contains(&"INSUFFICIENT_POWER") {
        "INSUFFICIENT_POWER"
    } else if statuses.iter().all(|status| *status == "KEEP") {
        "KEEP_ON_DEVELOPMENT_PANEL"
    } else {
        "REJECT"
    };
    let payload = serde_json::json!({
        "schema": "uor-r4.cpu-query-cache-aggregate/1",
        "status": status,
        "seeds": seeds,
    });
    let semantic = serde_json::to_vec(&payload)
        .map_err(|error| invalid(format!("query cache aggregate: {error}")))?;
    Ok(serde_json::json!({
        "schema": payload["schema"],
        "status": status,
        "seeds": payload["seeds"],
        "decision_sha256": sha256(&semantic),
    }))
}

fn validate_request(request: &CentroidEvaluationRequest) -> Result<()> {
    if request.panel_splits.is_empty()
        || request.minimum_observations == 0
        || request.minimum_fitted_rows == 0
        || request.minimum_fit_observations == 0
    {
        return Err(invalid("query cache evaluation request is empty"));
    }
    if request.panel_splits.iter().collect::<BTreeSet<_>>().len() != request.panel_splits.len() {
        return Err(invalid("query cache evaluation repeats a panel"));
    }
    if request
        .panel_splits
        .iter()
        .any(|panel| panel == &request.train_split)
    {
        return Err(invalid("query cache evaluation uses train as a panel"));
    }
    Ok(())
}

fn fit_streamed_tables(
    cache: &QueryCache,
    metadata: &QueryCacheSplit,
    rows: &CachedRows,
    request: &CentroidEvaluationRequest,
) -> Result<FittedTables> {
    let vocab = cache.manifest.vocab;
    let width = cache.manifest.width;
    let mut counts = vec![0usize; vocab];
    for &target in &rows.targets {
        let target = target as usize;
        if target >= vocab {
            return Err(invalid("query cache TRAIN target is outside the table"));
        }
        counts[target] += 1;
    }
    let fitted_tokens = (0..vocab)
        .filter(|&token| counts[token] >= request.minimum_observations)
        .collect::<Vec<_>>();
    if fitted_tokens.is_empty() {
        return Err(invalid("query cache centroid has no eligible rows"));
    }
    let fitted_observations = fitted_tokens.iter().map(|&token| counts[token]).sum();
    let mut strata = BTreeMap::<usize, Vec<usize>>::new();
    for &token in &fitted_tokens {
        strata.entry(counts[token]).or_default().push(token);
    }
    let control_strata = strata.len();
    let singleton_control_targets = strata.values().filter(|targets| targets.len() == 1).count();
    let mut control_targets = vec![None; vocab];
    for targets in strata.values() {
        for (index, &target) in targets.iter().enumerate() {
            control_targets[target] = Some(targets[(index + 1) % targets.len()]);
        }
    }
    let shuffled_changed_targets = control_targets
        .iter()
        .enumerate()
        .filter(|(token, control)| control.is_some_and(|control| *token != control))
        .count();

    let mut candidate_sums = vec![0f64; vocab.saturating_mul(width)];
    let mut control_sums = vec![0f64; vocab.saturating_mul(width)];
    let query_path = cache.root.join(&metadata.query_file);
    let expected_bytes = rows
        .targets
        .len()
        .checked_mul(width)
        .and_then(|values| values.checked_mul(4))
        .ok_or_else(|| invalid("query cache TRAIN query length overflows"))?;
    let actual_bytes = fs::metadata(&query_path)
        .map_err(|error| invalid(format!("{}: {error}", query_path.display())))?
        .len();
    if actual_bytes != expected_bytes as u64 {
        return Err(invalid("query cache TRAIN query length differs"));
    }
    let mut reader = BufReader::new(
        File::open(&query_path)
            .map_err(|error| invalid(format!("{}: {error}", query_path.display())))?,
    );
    let mut hasher = Sha256::new();
    let row_bytes = width
        .checked_mul(4)
        .ok_or_else(|| invalid("query cache row width overflows"))?;
    let mut buffer = vec![0u8; row_bytes];
    for (row, &target) in rows.targets.iter().enumerate() {
        reader
            .read_exact(&mut buffer)
            .map_err(|error| invalid(format!("query cache TRAIN row {row}: {error}")))?;
        hasher.update(&buffer);
        let target = target as usize;
        let candidate = target.saturating_mul(width);
        let Some(control_target) = control_targets[target] else {
            continue;
        };
        let control_target = control_target.saturating_mul(width);
        for coordinate in 0..width {
            let at = coordinate * 4;
            let value =
                f32::from_le_bytes([buffer[at], buffer[at + 1], buffer[at + 2], buffer[at + 3]]);
            let value = f64::from(f32_to_q12(value)?);
            candidate_sums[candidate + coordinate] += value;
            control_sums[control_target + coordinate] += value;
        }
    }
    let mut tail = [0u8; 1];
    if reader
        .read(&mut tail)
        .map_err(|error| invalid(error.to_string()))?
        != 0
        || format!("{:x}", hasher.finalize()) != metadata.query_sha256
    {
        return Err(invalid("query cache TRAIN query hash or length differs"));
    }

    let mut centroid = cache.table.clone();
    let mut shuffled = cache.table.clone();
    let mut changed_coordinates = 0u64;
    let mut displacement_l1 = 0u64;
    let mut shuffled_changed_coordinates = 0u64;
    for &token in &fitted_tokens {
        let base = token.saturating_mul(width);
        for coordinate in 0..width {
            let candidate = q12_mean(candidate_sums[base + coordinate], counts[token])?;
            let control = q12_mean(control_sums[base + coordinate], counts[token])?;
            changed_coordinates += u64::from(candidate != cache.table[base + coordinate]);
            displacement_l1 += u64::from(cache.table[base + coordinate].abs_diff(candidate));
            centroid[base + coordinate] = candidate;
            shuffled[base + coordinate] = control;
            shuffled_changed_coordinates += u64::from(candidate != control);
        }
    }
    Ok(FittedTables {
        centroid,
        shuffled,
        fitted_rows: fitted_tokens.len(),
        observations: fitted_observations,
        control_strata,
        singleton_control_targets,
        changed_coordinates,
        displacement_l1,
        shuffled_changed_targets,
        shuffled_changed_coordinates,
    })
}

fn load_q12_queries(root: &Path, metadata: &QueryCacheSplit, width: usize) -> Result<Vec<i16>> {
    let bytes = read_checked(root, &metadata.query_file, &metadata.query_sha256)?;
    let expected = metadata
        .rows
        .checked_mul(width)
        .and_then(|values| values.checked_mul(4))
        .ok_or_else(|| invalid("query cache query length overflows"))?;
    if bytes.len() != expected {
        return Err(invalid("query cache query row count differs"));
    }
    let mut queries = Vec::with_capacity(metadata.rows.saturating_mul(width));
    for chunk in bytes.chunks_exact(4) {
        let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        queries.push(f32_to_q12(value)?);
    }
    Ok(queries)
}

fn evaluate_table(
    queries: &[i16],
    targets: &[u32],
    rows: &CachedRows,
    table: &[i16],
    vocab: usize,
    width: usize,
) -> Result<Decisions> {
    let block_rows = QUERY_BLOCK_ROWS;
    let decoded = queries
        .par_chunks(width.saturating_mul(block_rows))
        .enumerate()
        .map(|(block, block_queries)| {
            let rows_in_block = block_queries.len() / width;
            let mut best = vec![Vec::<(i32, u32)>::with_capacity(8); rows_in_block];
            for token in 0..vocab {
                let candidate = &table[token * width..(token + 1) * width];
                for (row, query) in block_queries.chunks_exact(width).enumerate() {
                    let distance = query
                        .iter()
                        .zip(candidate)
                        .fold(0i32, |sum, (&left, &right)| {
                            sum.saturating_add(i32::from(left.abs_diff(right)))
                        });
                    insert_best(&mut best[row], distance, token as u32);
                }
            }
            (block, best)
        })
        .collect::<Vec<_>>();
    let decoded = decoded
        .into_iter()
        .flat_map(|(block, best)| {
            let start = block * block_rows;
            best.into_iter().enumerate().map(move |(row, candidates)| {
                let index = start + row;
                let top1 = candidates[0].1;
                (
                    top1,
                    top1 == targets[index],
                    candidates.iter().any(|&(_, token)| token == targets[index]),
                )
            })
        })
        .collect::<Vec<_>>();
    let mut tokens = Vec::with_capacity(decoded.len());
    let mut top1_rows = Vec::with_capacity(decoded.len());
    let mut top8_rows = Vec::with_capacity(decoded.len());
    let mut bytes = Vec::with_capacity(decoded.len() * 4);
    for (token, top1, top8) in decoded {
        tokens.push(token);
        top1_rows.push(top1);
        top8_rows.push(top8);
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    let top1_correct = top1_rows.iter().filter(|&&hit| hit).count();
    let top8_correct = top8_rows.iter().filter(|&&hit| hit).count();
    let changed_from_frozen = tokens
        .iter()
        .zip(&rows.nearest)
        .filter(|(left, right)| left != right)
        .count();
    Ok(Decisions {
        counts: ReadoutCounts {
            rows: tokens.len(),
            top1_correct,
            top8_correct,
            changed_from_frozen,
            decision_sha256: sha256(&bytes),
        },
        top1_rows,
        top8_rows,
        tokens,
    })
}

fn insert_best(best: &mut Vec<(i32, u32)>, distance: i32, token: u32) {
    let candidate = (distance, token);
    let position = best
        .iter()
        .position(|current| {
            candidate
                .0
                .cmp(&current.0)
                .then(candidate.1.cmp(&current.1))
                .is_lt()
        })
        .unwrap_or(best.len());
    if position < 8 {
        best.insert(position, candidate);
        best.truncate(8);
    }
}

fn paired(candidate: &[bool], control: &[bool]) -> serde_json::Value {
    if candidate.len() != control.len() {
        return serde_json::json!({"error": "paired decisions differ"});
    }
    let candidate_only = candidate
        .iter()
        .zip(control)
        .filter(|(left, right)| **left && !**right)
        .count() as u64;
    let control_only = candidate
        .iter()
        .zip(control)
        .filter(|(left, right)| !**left && **right)
        .count() as u64;
    let candidate_sum = candidate.iter().filter(|&&hit| hit).count() as u64;
    let control_sum = control.iter().filter(|&&hit| hit).count() as u64;
    let discordant = candidate_only + control_only;
    let threshold = ceil_sqrt(discordant.saturating_mul(4)).max(2);
    serde_json::json!({
        "candidate_only": candidate_only,
        "control_only": control_only,
        "candidate_sum": candidate_sum,
        "control_sum": control_sum,
        "discordant": discordant,
        "threshold": threshold,
        "passes": candidate_sum > control_sum && candidate_only >= threshold,
    })
}

fn ceil_sqrt(value: u64) -> u64 {
    if value <= 1 {
        return value;
    }
    let mut low = 1u64;
    let mut high = value.min(1u64 << 32);
    while low < high {
        let middle = low + (high - low) / 2;
        if middle.saturating_mul(middle) >= value {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

fn decode_cached_rows(bytes: &[u8], expected_rows: usize, vocab: usize) -> Result<CachedRows> {
    let minimum = 16usize
        .checked_add(expected_rows.saturating_mul(18))
        .ok_or_else(|| invalid("query cache row inventory overflows"))?;
    if bytes.len() != minimum || &bytes[..8] != ROW_MAGIC {
        return Err(invalid("query cache row file header or length differs"));
    }
    let stored = u64::from_le_bytes(bytes[8..16].try_into().unwrap_or([0; 8]));
    if stored != expected_rows as u64 {
        return Err(invalid("query cache row count differs"));
    }
    let mut at = 16usize;
    let mut read_u32 = |count: usize| -> Result<Vec<u32>> {
        let end = at
            .checked_add(count.saturating_mul(4))
            .ok_or_else(|| invalid("query cache row offset overflows"))?;
        let values = bytes
            .get(at..end)
            .ok_or_else(|| invalid("query cache row file is truncated"))?
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();
        at = end;
        Ok(values)
    };
    let targets = read_u32(expected_rows)?;
    let positions = read_u32(expected_rows)?;
    let nearest = read_u32(expected_rows)?;
    let writer = read_u32(expected_rows)?;
    if targets
        .iter()
        .chain(&nearest)
        .chain(&writer)
        .any(|&token| token as usize >= vocab)
    {
        return Err(invalid("query cache decision token is outside the table"));
    }
    if positions.iter().copied().collect::<BTreeSet<_>>().len() != expected_rows {
        return Err(invalid("query cache split has duplicate positions"));
    }
    let opens = bytes[at..at + expected_rows].to_vec();
    at += expected_rows;
    let copies = bytes[at..at + expected_rows].to_vec();
    if opens.iter().chain(&copies).any(|&flag| flag > 1) {
        return Err(invalid("query cache row flag differs"));
    }
    Ok(CachedRows {
        targets,
        positions,
        nearest,
        writer,
        opens,
        copies,
    })
}

fn read_checked(root: &Path, file: &str, expected: &str) -> Result<Vec<u8>> {
    safe_relative(file)?;
    let path = root.join(file);
    let bytes = fs::read(&path).map_err(|error| invalid(format!("{}: {error}", path.display())))?;
    if sha256(&bytes) != expected {
        return Err(invalid(format!("query cache file {file} hash differs")));
    }
    Ok(bytes)
}

fn safe_relative(file: &str) -> Result<()> {
    let path = Path::new(file);
    if file.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(invalid("query cache file path escapes its root"));
    }
    Ok(())
}

fn decode_f32(bytes: &[u8]) -> Result<Vec<f32>> {
    if !bytes.len().is_multiple_of(4) {
        return Err(invalid("query cache F32 file is not aligned"));
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect())
}

fn f32_to_q12(value: f32) -> Result<i16> {
    if !value.is_finite() {
        return Err(invalid("query cache F32 value is not finite"));
    }
    let scaled = value * Q12_SCALE;
    let rounded = scaled.round();
    let integer = i16::try_from(rounded as i32)
        .map_err(|_| invalid("query cache Q12 value is outside i16"))?;
    if f32::from(integer) / Q12_SCALE != value {
        return Err(invalid("query cache query or table is not on the Q12 grid"));
    }
    Ok(integer)
}

fn q12_mean(sum: f64, count: usize) -> Result<i16> {
    if count == 0 || !sum.is_finite() {
        return Err(invalid("query cache centroid count or sum differs"));
    }
    let value = sum / count as f64;
    let rounded = if value >= 0.0 {
        value.round()
    } else {
        -(-value).round()
    };
    i16::try_from(rounded as i32).map_err(|_| invalid("query cache centroid is outside i16"))
}

fn row_identity(rows: &CachedRows) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for (&position, &target) in rows.positions.iter().zip(&rows.targets) {
        hasher.update(u64::from(position).to_le_bytes());
        hasher.update(target.to_le_bytes());
    }
    hasher.finalize().into()
}

fn pooled_identity(panels: &[(String, [u8; 32])]) -> String {
    let mut hasher = Sha256::new();
    for (name, identity) in panels {
        hasher.update((name.len() as u64).to_le_bytes());
        hasher.update(name.as_bytes());
        hasher.update(identity);
    }
    format!("{:x}", hasher.finalize())
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q12(values: &[i16]) -> Vec<f32> {
        values
            .iter()
            .map(|&value| f32::from(value) / Q12_SCALE)
            .collect()
    }

    fn f32_bytes(values: &[f32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    fn row_bytes(rows: &CachedRows) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(ROW_MAGIC);
        bytes.extend_from_slice(&(rows.targets.len() as u64).to_le_bytes());
        for values in [&rows.targets, &rows.positions, &rows.nearest, &rows.writer] {
            for value in values {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&rows.opens);
        bytes.extend_from_slice(&rows.copies);
        bytes
    }

    fn fixture(name: &str) -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "uor-r4-cpu-query-cache-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let checkpoint = root.join("checkpoint");
        fs::create_dir_all(&checkpoint).unwrap();
        fs::write(checkpoint.join("config.json"), b"{}").unwrap();
        let cache_root = root.join("cache");
        fs::create_dir_all(cache_root.join("no-memory")).unwrap();

        let table = q12(&[0, 10, 20]);
        let table_bytes = f32_bytes(&table);
        fs::write(cache_root.join("table.f32"), &table_bytes).unwrap();
        let train_rows = CachedRows {
            targets: vec![0, 0, 0, 1, 1, 1, 2],
            positions: vec![1, 2, 3, 4, 5, 6, 7],
            nearest: vec![0, 0, 0, 1, 1, 1, 2],
            writer: vec![0, 0, 0, 1, 1, 1, 2],
            opens: vec![0; 7],
            copies: vec![0; 7],
        };
        let train_queries = f32_bytes(&q12(&[0, 2, 4, 14, 16, 18, 22]));
        fs::write(
            cache_root.join("no-memory/train.queries.f32"),
            &train_queries,
        )
        .unwrap();
        let train_rows_bytes = row_bytes(&train_rows);
        fs::write(
            cache_root.join("no-memory/train.rows.bin"),
            &train_rows_bytes,
        )
        .unwrap();
        let panel_rows = CachedRows {
            targets: vec![0, 1, 2],
            positions: vec![1, 2, 3],
            nearest: vec![0, 1, 2],
            writer: vec![0, 1, 2],
            opens: vec![0; 3],
            copies: vec![0; 3],
        };
        let panel_queries = f32_bytes(&q12(&[1, 14, 20]));
        fs::write(cache_root.join("no-memory/p.queries.f32"), &panel_queries).unwrap();
        let panel_rows_bytes = row_bytes(&panel_rows);
        fs::write(cache_root.join("no-memory/p.rows.bin"), &panel_rows_bytes).unwrap();

        let manifest = QueryCacheManifest {
            schema: QUERY_CACHE_SCHEMA.to_owned(),
            checkpoint: checkpoint.display().to_string(),
            data_root: root.display().to_string(),
            tokenizer_sha256: PINNED_TOKENIZER_SHA256.to_owned(),
            checkpoint_files: BTreeMap::new(),
            run_config: None,
            run_config_sha256: None,
            table_file: "table.f32".to_owned(),
            table_sha256: sha256(&table_bytes),
            vocab: 3,
            width: 1,
            query_space: "q12-f32".to_owned(),
            splits: vec![
                QueryCacheSplit {
                    split: "train".to_owned(),
                    label: "no-memory".to_owned(),
                    memory: false,
                    context: 4,
                    rows: 7,
                    tokens_sha256: "train-tokens".to_owned(),
                    mask_sha256: "train-mask".to_owned(),
                    query_file: "no-memory/train.queries.f32".to_owned(),
                    query_sha256: sha256(&train_queries),
                    rows_file: "no-memory/train.rows.bin".to_owned(),
                    rows_sha256: sha256(&train_rows_bytes),
                },
                QueryCacheSplit {
                    split: "p".to_owned(),
                    label: "no-memory".to_owned(),
                    memory: false,
                    context: 4,
                    rows: 3,
                    tokens_sha256: "panel-tokens".to_owned(),
                    mask_sha256: "panel-mask".to_owned(),
                    query_file: "no-memory/p.queries.f32".to_owned(),
                    query_sha256: sha256(&panel_queries),
                    rows_file: "no-memory/p.rows.bin".to_owned(),
                    rows_sha256: sha256(&panel_rows_bytes),
                },
            ],
        };
        let manifest_bytes = serde_json::to_vec_pretty(&manifest).unwrap();
        fs::write(cache_root.join("manifest.json"), &manifest_bytes).unwrap();
        let seal = QueryCacheSeal {
            schema: QUERY_CACHE_SEAL_SCHEMA.to_owned(),
            manifest_sha256: sha256(&manifest_bytes),
        };
        fs::write(
            cache_root.join("seal.json"),
            serde_json::to_vec_pretty(&seal).unwrap(),
        )
        .unwrap();
        (root, cache_root)
    }

    fn request() -> CentroidEvaluationRequest {
        CentroidEvaluationRequest {
            train_split: "train".to_owned(),
            panel_splits: vec!["p".to_owned()],
            memory: false,
            minimum_observations: 1,
            minimum_fitted_rows: 3,
            minimum_fit_observations: 7,
            expected_train_tokens_sha256: Some("train-tokens".to_owned()),
            expected_train_mask_sha256: Some("train-mask".to_owned()),
            expected_rows: Some(3),
            expected_row_identity: None,
        }
    }

    #[test]
    fn q12_round_trip_and_ties_are_exact() {
        assert_eq!(f32_to_q12(1.0 / Q12_SCALE).unwrap(), 1);
        assert_eq!(f32_to_q12(-1.0 / Q12_SCALE).unwrap(), -1);
        assert!(f32_to_q12(1.0 / 8192.0).is_err());
        let mut best = Vec::new();
        for token in [5, 2, 4, 1] {
            insert_best(&mut best, 3, token);
        }
        assert_eq!(
            best.iter().map(|candidate| candidate.1).collect::<Vec<_>>(),
            [1, 2, 4, 5]
        );
    }

    #[test]
    fn exact_count_shuffle_uses_every_eligible_row_and_is_not_vacuous() {
        let (root, cache_root) = fixture("matched");
        let report = evaluate_query_cache(&cache_root, &request()).unwrap();
        assert_eq!(report["status"], "REJECT");
        assert_eq!(report["fit"]["fitted_rows"], 3);
        assert_eq!(report["fit"]["observations"], 7);
        assert_eq!(report["fit"]["power_ok"], true);
        assert_eq!(
            report["fit"]["control_strategy"],
            "exact-count-stratified-cycle"
        );
        assert_eq!(report["fit"]["control_strata"], 2);
        assert_eq!(report["fit"]["singleton_control_targets"], 1);
        assert_eq!(report["fit"]["shuffled_changed_targets"], 2);
        assert!(
            report["fit"]["shuffled_changed_coordinates"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert_eq!(report["pooled"]["rows"], 3);
        assert_eq!(report["pooled"]["identity_matches"], true);
        assert_eq!(report["panels"][0]["identity"]["exact"], true);

        let identity = report["pooled"]["row_identity_sha256"]
            .as_str()
            .unwrap()
            .to_owned();
        let mut registered = request();
        registered.expected_row_identity = Some(identity);
        let one = rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .unwrap()
            .install(|| evaluate_query_cache(&cache_root, &registered).unwrap());
        let four = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap()
            .install(|| evaluate_query_cache(&cache_root, &registered).unwrap());
        let normalize = |mut value: serde_json::Value| {
            value["wall_ms"] = 0.into();
            value["workers"] = 0.into();
            value
        };
        assert_eq!(normalize(one), normalize(four));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn singleton_count_strata_use_their_own_query_in_both_arms() {
        let (root, cache_root) = fixture("singleton-control");
        let cache = QueryCache::open(&cache_root).unwrap();
        let (metadata, rows) = cache.split("train", false).unwrap();
        let fitted = fit_streamed_tables(&cache, metadata, &rows, &request()).unwrap();

        assert_eq!(fitted.centroid[0], 2);
        assert_eq!(fitted.shuffled[0], 16);
        assert_eq!(fitted.centroid[2], 22);
        assert_eq!(fitted.shuffled[2], 22);
        assert_eq!(fitted.singleton_control_targets, 1);
        assert_eq!(fitted.shuffled_changed_targets, 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn registered_power_identity_and_tamper_gates_fail_closed() {
        let (root, cache_root) = fixture("gates");
        let mut underpowered = request();
        underpowered.minimum_fitted_rows = 4;
        let report = evaluate_query_cache(&cache_root, &underpowered).unwrap();
        assert_eq!(report["status"], "INSUFFICIENT_POWER");

        let mut wrong_identity = request();
        wrong_identity.expected_row_identity = Some("00".repeat(32));
        let report = evaluate_query_cache(&cache_root, &wrong_identity).unwrap();
        assert_eq!(report["status"], "CONTROL_FAULT");

        let mut wrong_train = request();
        wrong_train.expected_train_tokens_sha256 = Some("not-the-train".to_owned());
        let report = evaluate_query_cache(&cache_root, &wrong_train).unwrap();
        assert_eq!(report["status"], "CONTROL_FAULT");
        assert_eq!(report["train"]["identity_matches"], false);

        let query = cache_root.join("no-memory/p.queries.f32");
        let mut bytes = fs::read(&query).unwrap();
        bytes[0] ^= 1;
        fs::write(&query, bytes).unwrap();
        assert!(evaluate_query_cache(&cache_root, &request()).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn two_seed_aggregate_has_fixed_precedence() {
        let report = |status: &str, manifest: &str, bar: bool| {
            serde_json::json!({
                "schema": EVALUATION_SCHEMA,
                "status": status,
                "cache": {"manifest_sha256": manifest, "checkpoint": manifest},
                "request": {
                    "expected_rows": 1_460_697,
                    "expected_row_identity": "identity",
                    "expected_train_tokens_sha256": "tokens",
                    "expected_train_mask_sha256": "mask",
                    "memory": false,
                },
                "fit": {"fitted_rows": 100, "observations": 10_000},
                "pooled": {
                    "bar_passes": bar,
                    "rows": 1_460_697,
                    "rows_match": true,
                    "identity_matches": true,
                    "row_identity_sha256": "identity",
                },
                "panels": [{"split": "p", "identity": {"exact": true}}],
            })
        };
        let keep = aggregate_query_cache_reports(&[
            report("KEEP", "s1", true),
            report("KEEP", "s2", true),
        ])
        .unwrap();
        assert_eq!(keep["status"], "KEEP_ON_DEVELOPMENT_PANEL");
        let reject = aggregate_query_cache_reports(&[
            report("KEEP", "s1", true),
            report("REJECT", "s2", false),
        ])
        .unwrap();
        assert_eq!(reject["status"], "REJECT");
        let fault = aggregate_query_cache_reports(&[
            report("INSUFFICIENT_POWER", "s1", false),
            report("CONTROL_FAULT", "s2", false),
        ])
        .unwrap();
        assert_eq!(fault["status"], "CONTROL_FAULT");
    }
}
