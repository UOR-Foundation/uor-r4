//! G0a — cheap-predictability feasibility for the canonical-address cache (CAR-LM).
//!
//! Evaluation-only. This loads a retained continuous joint checkpoint, runs the
//! unchanged learner forward over the exposed evaluator-v2 development blocks,
//! records the read-enabled recurrent state at every target position, and then
//! measures hand-built causal address caches against matched-access controls.
//! No optimizer step, no training data change and no model modification occurs.
//!
//! # Predeclared G0a criteria (fixed before any run in this module)
//!
//! * **quality** — a cache arm's mean next-token NLL is at most the better
//!   order-2/3 count n-gram's and its top-1 accuracy is at least that n-gram's.
//! * **cost** — the event-driven ("sticky") median cache-path operations per
//!   token are at most 10% of the retained dense full-read step
//!   (`1_672_448` learned-code inspections, the CAR-LM step-1 measurement).
//! * **hit** — the causal cache-hit rate is at least 50%.
//!
//! A G0a pass requires all three. The report contains the full quality-vs-ops
//! Pareto and the alternative always-recompute cost so a failure is diagnostic,
//! not binary.
//!
//! # Scope
//!
//! The dense denominator counts the whole learned read+output step; the cache
//! path counts only address computation plus the table read, so it does not pay
//! the recurrence that produces the address. That asymmetry is deliberate (it is
//! the plan's framing) and is reported as a limitation.

use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use candle_core::Device;
use serde::Serialize;
use serde_json::{json, Value};
use uor_r4_core::native_geometric::learner::embedding::canonical_h4_roots_q30;
use uor_r4_core::report_output;

use crate::baseline_protocol::{load_evaluator, verify_identity};
use crate::joint_evaluation::{score_probabilities, EVALUATION_CONTEXT};
use crate::joint_model::{JointModel, ReadMode};
use crate::{invalid, sha256_file, Result};

const VOCAB: usize = 4096;
const CALIBRATION_BLOCKS: usize = 64;
const DENSE_MEASURED_INSPECTIONS: u64 = 1_672_448;
/// Share of the dense step measured in the 4096-row vocabulary projection.
const VOCAB_PROJECTION_SHARE: f64 = 0.627;
const CACHE_ALPHA: f64 = 0.1;
const KMEANS_ITERATIONS: usize = 12;
const GROUP_ORDER: usize = 120;
const MAX_BATCH: usize = 16;
const NGRAM_LAMBDA_GRID: [f64; 5] = [0.3, 0.5, 0.7, 0.9, 0.95];
const GAP_BUCKETS: usize = 7;

pub fn run_cli(args: &[String]) -> Result<()> {
    if args.len() != 6 && args.len() != 7 {
        return Err(invalid(
            "usage: joint-cache-eval SEALED_CONTINUOUS_CHECKPOINT EVALUATOR_JSON NEW_REPORT_ROOT {cpu|metal} BATCH [NGRAM_TRAIN_TOKENS]",
        ));
    }
    if !["cpu", "metal"].contains(&args[4].as_str()) {
        return Err(invalid("joint-cache-eval device cpu|metal"));
    }
    let batch: usize = args[5].parse().map_err(|_| invalid("cache-eval batch"))?;
    if !(1..=MAX_BATCH).contains(&batch) {
        return Err(invalid("joint-cache-eval batch 1..=16"));
    }
    let ngram_limit: usize = match args.get(6) {
        Some(value) => value
            .parse()
            .map_err(|_| invalid("cache-eval ngram train-token prefix"))?,
        None => 6_000_000,
    };
    if ngram_limit < 1_000_000 {
        return Err(invalid(
            "cache-eval ngram prefix must be at least 1,000,000",
        ));
    }
    let out = Path::new(&args[3]);
    report_output::claim(out)?;
    let result = run(
        Path::new(&args[1]),
        Path::new(&args[2]),
        out,
        &args[4],
        batch,
        ngram_limit,
    );
    if let Err(error) = &result {
        std::fs::write(
            out.join("failed-attempt.json"),
            serde_json::to_vec_pretty(&json!({
                "status": "FAILED_ATTEMPT",
                "error": error.to_string(),
                "source_commit": option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND")
            }))?,
        )?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    result
}

#[derive(Clone, Debug, Default, Serialize)]
struct CacheMetric {
    hits: usize,
    positions: usize,
    hit_rate: f64,
    top1_majority: f64,
    top1_first: f64,
    nll_majority: f64,
    hit_by_position: Vec<f64>,
    hit_by_gap: Vec<f64>,
}

#[derive(Clone, Debug, Serialize)]
struct ArmReport {
    name: String,
    kind: String,
    address_cardinality: usize,
    address_stability: f64,
    hit_rate_on_stable: f64,
    stable_positions: usize,
    ops_address: u64,
    ops_sticky_median: u64,
    ops_sticky_p95: u64,
    ops_sticky_mean: f64,
    ops_always: u64,
    ops_ratio_full: f64,
    ops_ratio_vocab_projection: f64,
    static_cache: CacheMetric,
    causal_cache: CacheMetric,
    quality_pass: bool,
    cost_pass: bool,
    hit_pass: bool,
    g0a_pass: bool,
    notes: String,
}

struct Unigram {
    p: Vec<f64>,
    best: u32,
}

fn select_device(name: &str) -> Result<Device> {
    match name {
        "cpu" => Ok(Device::Cpu),
        "metal" => {
            #[cfg(feature = "metal")]
            {
                Ok(Device::new_metal(0)?)
            }
            #[cfg(not(feature = "metal"))]
            {
                Err(invalid(
                    "metal requested but the metal feature is not compiled",
                ))
            }
        }
        _ => Err(invalid("unsupported device")),
    }
}

fn read_u16(path: &Path) -> Result<Vec<u16>> {
    let bytes = std::fs::read(path)?;
    if bytes.len() % 2 != 0 {
        return Err(invalid("u16 token store length"));
    }
    Ok(bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect())
}

fn read_u16_prefix(path: &Path, limit: usize) -> Result<Vec<u16>> {
    let bytes = std::fs::read(path)?;
    let usable = bytes.len().min(limit.saturating_mul(2));
    let usable = usable - (usable % 2);
    Ok(bytes[..usable]
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect())
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Fit a deterministic k-means codebook on the tune-split states.
fn kmeans_fit(states: &[f32], n_fit: usize, d: usize, v: usize, seed: u64) -> Vec<f32> {
    let mut rng = seed;
    let mut chosen: Vec<usize> = Vec::with_capacity(v);
    let mut seen = std::collections::HashSet::new();
    while chosen.len() < v {
        let candidate = (splitmix64(&mut rng) % n_fit as u64) as usize;
        if seen.insert(candidate) {
            chosen.push(candidate);
        }
    }
    let mut centroids = vec![0f32; v * d];
    for (j, &index) in chosen.iter().enumerate() {
        centroids[j * d..(j + 1) * d].copy_from_slice(&states[index * d..(index + 1) * d]);
    }
    for _ in 0..KMEANS_ITERATIONS {
        let mut sums = vec![0f64; v * d];
        let mut counts = vec![0u64; v];
        for i in 0..n_fit {
            let row = &states[i * d..(i + 1) * d];
            let (best, _) = nearest(row, &centroids, v, d);
            counts[best] += 1;
            let target = &mut sums[best * d..(best + 1) * d];
            for c in 0..d {
                target[c] += f64::from(row[c]);
            }
        }
        for j in 0..v {
            if counts[j] > 0 {
                let divisor = counts[j] as f64;
                for c in 0..d {
                    centroids[j * d + c] = (sums[j * d + c] / divisor) as f32;
                }
            }
        }
    }
    centroids
}

#[inline]
fn nearest(row: &[f32], centroids: &[f32], v: usize, d: usize) -> (usize, f32) {
    let mut best = 0usize;
    let mut best_distance = f32::INFINITY;
    for j in 0..v {
        let centroid = &centroids[j * d..(j + 1) * d];
        let mut sum = 0f32;
        for c in 0..d {
            let delta = row[c] - centroid[c];
            sum += delta * delta;
        }
        if sum < best_distance {
            best_distance = sum;
            best = j;
        }
    }
    (best, best_distance)
}

fn assign_all(states: &[f32], n: usize, d: usize, centroids: &[f32], v: usize) -> Vec<u32> {
    let mut out = vec![0u32; n];
    for i in 0..n {
        let (best, _) = nearest(&states[i * d..(i + 1) * d], centroids, v, d);
        out[i] = best as u32;
    }
    out
}

/// Signed aggregate quaternion nearest an exact 2I root. `q` and `-q` stay
/// distinct (chirality preserved) because the dot product is signed.
fn group_roots() -> Vec<[f64; 4]> {
    canonical_h4_roots_q30()
        .iter()
        .map(|root| {
            let mut q = [0f64; 4];
            for c in 0..4 {
                q[c] = root.0[c] as f64;
            }
            normalize4(&mut q);
            q
        })
        .collect()
}

fn normalize4(q: &mut [f64; 4]) {
    let norm = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if norm > 1e-12 {
        for value in q.iter_mut() {
            *value /= norm;
        }
    }
}

fn group_address(state_row: &[f32], roots: &[[f64; 4]]) -> u32 {
    let mut aggregate = [0f64; 4];
    for lane in state_row.chunks_exact(4) {
        let mut q = [0f64; 4];
        for c in 0..4 {
            q[c] = f64::from(lane[c]);
        }
        normalize4(&mut q);
        for c in 0..4 {
            aggregate[c] += q[c];
        }
    }
    normalize4(&mut aggregate);
    let mut best = 0usize;
    let mut best_dot = f64::NEG_INFINITY;
    for (index, root) in roots.iter().enumerate() {
        let dot = aggregate[0] * root[0]
            + aggregate[1] * root[1]
            + aggregate[2] * root[2]
            + aggregate[3] * root[3];
        if dot > best_dot {
            best_dot = dot;
            best = index;
        }
    }
    best as u32
}

fn group_ops(d: usize) -> u64 {
    ((d / 4) as u64) * 8 + (GROUP_ORDER as u64) * 4
}

#[derive(Clone)]
struct CacheEntry {
    first: u32,
    total: u64,
    best_token: u32,
    best_count: u64,
    counts: HashMap<u32, u64>,
}

struct CausalCache {
    entries: HashMap<u128, CacheEntry>,
}

impl CausalCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    fn observe(&mut self, key: u128, target: u32) {
        let entry = self.entries.entry(key).or_insert_with(|| CacheEntry {
            first: target,
            total: 0,
            best_token: target,
            best_count: 0,
            counts: HashMap::new(),
        });
        entry.total += 1;
        let count = entry.counts.entry(target).or_insert(0);
        *count += 1;
        if *count > entry.best_count || (*count == entry.best_count && target < entry.best_token) {
            entry.best_count = *count;
            entry.best_token = target;
        }
    }

    fn hit_probability(&self, key: u128, target: u32, unigram: &Unigram) -> (f64, u32, bool) {
        match self.entries.get(&key) {
            Some(entry) => {
                let count = entry.counts.get(&target).copied().unwrap_or(0) as f64;
                let probability =
                    (count + CACHE_ALPHA) / (entry.total as f64 + CACHE_ALPHA * VOCAB as f64);
                (probability, entry.best_token, true)
            }
            None => (unigram.p[target as usize], unigram.best, false),
        }
    }

    fn first_token(&self, key: u128) -> Option<u32> {
        self.entries.get(&key).map(|entry| entry.first)
    }
}

fn unigram_from(counts: &[u64], total: u64) -> Unigram {
    let mut p = vec![0f64; VOCAB];
    let mut best = 0usize;
    for (token, &count) in counts.iter().enumerate() {
        p[token] = (count as f64 + 1.0) / (total as f64 + VOCAB as f64);
        if count > counts[best] {
            best = token;
        }
    }
    Unigram {
        p,
        best: best as u32,
    }
}

fn gap_bucket(gap: usize) -> usize {
    match gap {
        0 | 1 => 0,
        2 | 3 => 1,
        4..=7 => 2,
        8..=15 => 3,
        16..=31 => 4,
        32..=63 => 5,
        _ => 6,
    }
}

fn eval_cache(
    keys: &[u128],
    targets: &[u32],
    tune_end: usize,
    update: bool,
    unigram: &Unigram,
) -> CacheMetric {
    let mut cache = CausalCache::new();
    let mut last_seen: HashMap<u128, usize> = HashMap::new();
    for index in 0..tune_end {
        if keys[index] != u128::MAX {
            cache.observe(keys[index], targets[index]);
            last_seen.insert(keys[index], index);
        }
    }
    let positions = targets.len().saturating_sub(tune_end);
    let mut metric = CacheMetric {
        positions,
        hit_by_position: vec![0f64; EVALUATION_CONTEXT],
        hit_by_gap: vec![0f64; GAP_BUCKETS],
        ..CacheMetric::default()
    };
    let mut position_hits = vec![0u64; EVALUATION_CONTEXT];
    let mut position_total = vec![0u64; EVALUATION_CONTEXT];
    let mut gap_hits = vec![0u64; GAP_BUCKETS];
    let mut gap_total = vec![0u64; GAP_BUCKETS];
    let mut nll = 0f64;
    let mut correct_majority = 0usize;
    let mut correct_first = 0usize;
    let mut hits = 0usize;
    for index in tune_end..targets.len() {
        let target = targets[index];
        let position = index % EVALUATION_CONTEXT;
        position_total[position] += 1;
        if keys[index] == u128::MAX {
            // No context: unigram only, counted as a miss.
            nll -= unigram.p[target as usize].ln();
            continue;
        }
        let key = keys[index];
        if let Some(&previous) = last_seen.get(&key) {
            gap_total[gap_bucket(index - previous)] += 1;
        }
        let (probability, prediction, hit) = cache.hit_probability(key, target, unigram);
        nll -= probability.ln();
        if prediction == target {
            correct_majority += 1;
        }
        if let Some(first) = cache.first_token(key) {
            if first == target {
                correct_first += 1;
            }
        } else if unigram.best == target {
            correct_first += 1;
        }
        if hit {
            hits += 1;
            position_hits[position] += 1;
            if let Some(&previous) = last_seen.get(&key) {
                gap_hits[gap_bucket(index - previous)] += 1;
            }
        }
        last_seen.insert(key, index);
        if update {
            cache.observe(key, target);
        }
    }
    for position in 0..EVALUATION_CONTEXT {
        if position_total[position] > 0 {
            metric.hit_by_position[position] =
                position_hits[position] as f64 / position_total[position] as f64;
        }
    }
    for bucket in 0..GAP_BUCKETS {
        if gap_total[bucket] > 0 {
            metric.hit_by_gap[bucket] = gap_hits[bucket] as f64 / gap_total[bucket] as f64;
        }
    }
    metric.hits = hits;
    metric.hit_rate = hits as f64 / positions.max(1) as f64;
    metric.top1_majority = correct_majority as f64 / positions.max(1) as f64;
    metric.top1_first = correct_first as f64 / positions.max(1) as f64;
    metric.nll_majority = nll / positions.max(1) as f64;
    metric
}

fn context_keys(tokens: &[u16], k: usize, n_total: usize) -> Vec<u128> {
    let mut keys = vec![u128::MAX; n_total];
    for (index, key) in keys.iter_mut().enumerate() {
        let position = index % EVALUATION_CONTEXT;
        if position + 1 < k {
            continue;
        }
        let start = index + 1 - k;
        let mut value = 0u128;
        for token in &tokens[start..=index] {
            value = value * VOCAB as u128 + u128::from(*token);
        }
        *key = value;
    }
    keys
}

fn median_p95(values: &mut [u64]) -> (u64, u64, f64) {
    if values.is_empty() {
        return (0, 0, 0.0);
    }
    values.sort_unstable();
    let n = values.len();
    let median = if n % 2 == 1 {
        values[n / 2]
    } else {
        values[n / 2 - 1] + (values[n / 2] - values[n / 2 - 1]) / 2
    };
    let rank = ((95u128 * n as u128 + 99) / 100) as usize;
    let p95 = values[rank.saturating_sub(1).min(n - 1)];
    let mean = values.iter().sum::<u64>() as f64 / n as f64;
    (median, p95, mean)
}

fn sticky_ops(keys: &[u128], tune_end: usize, address_ops: u64) -> Vec<u64> {
    let mut ops = Vec::with_capacity(keys.len().saturating_sub(tune_end));
    for index in tune_end..keys.len() {
        let position = index % EVALUATION_CONTEXT;
        let stable = position > 0 && keys[index] == keys[index - 1] && keys[index] != u128::MAX;
        ops.push(if stable { 1 } else { address_ops + 1 });
    }
    ops
}

fn stability(keys: &[u128], tune_end: usize) -> (f64, usize) {
    let mut stable = 0usize;
    let mut total = 0usize;
    for index in tune_end..keys.len() {
        if index % EVALUATION_CONTEXT == 0 || keys[index] == u128::MAX {
            continue;
        }
        total += 1;
        if keys[index] == keys[index - 1] {
            stable += 1;
        }
    }
    (stable as f64 / total.max(1) as f64, stable)
}

fn hit_rate_on_stable(keys: &[u128], targets: &[u32], tune_end: usize) -> (f64, usize) {
    let mut cache = CausalCache::new();
    for index in 0..tune_end {
        if keys[index] != u128::MAX {
            cache.observe(keys[index], targets[index]);
        }
    }
    let mut stable = 0usize;
    let mut hits = 0usize;
    for index in tune_end..targets.len() {
        if keys[index] != u128::MAX {
            if index % EVALUATION_CONTEXT != 0 && keys[index] == keys[index - 1] {
                stable += 1;
                if cache.entries.contains_key(&keys[index]) {
                    hits += 1;
                }
            }
            cache.observe(keys[index], targets[index]);
        }
    }
    (hits as f64 / stable.max(1) as f64, stable)
}

#[derive(Default)]
struct NgramTables {
    order2_pairs: Vec<(u64, u32)>,
    order2_contexts: Vec<(u64, u32)>,
    order3_pairs: Vec<(u64, u32)>,
    order3_contexts: Vec<(u64, u32)>,
}

fn merge_counts(values: &mut Vec<(u64, u32)>) {
    values.sort_unstable_by_key(|entry| entry.0);
    let mut write = 0usize;
    for read in 0..values.len() {
        if write > 0 && values[write - 1].0 == values[read].0 {
            values[write - 1].1 += values[read].1;
        } else {
            values[write] = values[read];
            write += 1;
        }
    }
    values.truncate(write);
}

fn build_ngram(tokens: &[u16], unigram_counts: &mut [u64; VOCAB]) -> (NgramTables, u64) {
    let mut order2_pairs = Vec::with_capacity(tokens.len());
    let mut order2_contexts = Vec::with_capacity(tokens.len());
    let mut order3_pairs = Vec::with_capacity(tokens.len());
    let mut order3_contexts = Vec::with_capacity(tokens.len());
    let mut total = 0u64;
    for index in 0..tokens.len() {
        let token = u64::from(tokens[index]);
        unigram_counts[token as usize] += 1;
        total += 1;
        if index >= 1 {
            let previous = u64::from(tokens[index - 1]);
            order2_pairs.push(((previous << 12) | token, 1));
            order2_contexts.push((previous, 1));
        }
        if index >= 2 {
            let first = u64::from(tokens[index - 2]);
            let second = u64::from(tokens[index - 1]);
            let context = (first << 12) | second;
            order3_pairs.push(((context << 12) | token, 1));
            order3_contexts.push((context, 1));
        }
    }
    merge_counts(&mut order2_pairs);
    merge_counts(&mut order2_contexts);
    merge_counts(&mut order3_pairs);
    merge_counts(&mut order3_contexts);
    (
        NgramTables {
            order2_pairs,
            order2_contexts,
            order3_pairs,
            order3_contexts,
        },
        total,
    )
}

fn lookup_count(values: &[(u64, u32)], key: u64) -> u32 {
    match values.binary_search_by_key(&key, |entry| entry.0) {
        Ok(index) => values[index].1,
        Err(_) => 0,
    }
}

fn eval_ngram(
    order: usize,
    tables: &NgramTables,
    tokens: &[u16],
    unigram: &Unigram,
    lambda: f64,
    start: usize,
    end: usize,
) -> (f64, f64, usize) {
    let mut nll = 0f64;
    let mut correct = 0usize;
    let mut count = 0usize;
    for index in start..end {
        let target = u32::from(tokens[index + 1]);
        let context = if order == 2 {
            if index < 1 {
                None
            } else {
                Some(u64::from(tokens[index - 1]))
            }
        } else if index < 2 {
            None
        } else {
            Some((u64::from(tokens[index - 2]) << 12) | u64::from(tokens[index - 1]))
        };
        let (pairs, contexts) = if order == 2 {
            (&tables.order2_pairs, &tables.order2_contexts)
        } else {
            (&tables.order3_pairs, &tables.order3_contexts)
        };
        let (probability, prediction) = match context {
            Some(context) => {
                let context_total = lookup_count(contexts, context);
                let pair_key = (context << 12) | u64::from(target);
                let pair_count = lookup_count(pairs, pair_key) as f64;
                let probability = lambda * (pair_count / context_total.max(1) as f64)
                    + (1.0 - lambda) * unigram.p[target as usize];
                let low_key = context << 12;
                let high_key = low_key + VOCAB as u64;
                let low = pairs.partition_point(|entry| entry.0 < low_key);
                let high = pairs.partition_point(|entry| entry.0 < high_key);
                let total = context_total.max(1) as f64;
                let mut best = unigram.best;
                let mut best_score = (1.0 - lambda) * unigram.p[best as usize];
                for entry in &pairs[low..high] {
                    let candidate = (entry.0 & 0xFFF) as u32;
                    let score = lambda * (entry.1 as f64 / total)
                        + (1.0 - lambda) * unigram.p[candidate as usize];
                    if score > best_score {
                        best_score = score;
                        best = candidate;
                    }
                }
                (probability, best)
            }
            None => (unigram.p[target as usize], unigram.best),
        };
        nll -= probability.max(1e-12).ln();
        if prediction == target {
            correct += 1;
        }
        count += 1;
    }
    (
        nll / count.max(1) as f64,
        correct as f64 / count.max(1) as f64,
        count,
    )
}

fn calibrate_lambda(
    order: usize,
    tables: &NgramTables,
    tokens: &[u16],
    unigram: &Unigram,
    start: usize,
    end: usize,
) -> f64 {
    let mut best_lambda = NGRAM_LAMBDA_GRID[0];
    let mut best_nll = f64::INFINITY;
    for lambda in NGRAM_LAMBDA_GRID {
        let (nll, _, _) = eval_ngram(order, tables, tokens, unigram, lambda, start, end);
        if nll < best_nll {
            best_nll = nll;
            best_lambda = lambda;
        }
    }
    best_lambda
}

#[allow(clippy::too_many_arguments)]
fn arm_report(
    name: &str,
    kind: &str,
    cardinality: usize,
    keys: &[u128],
    targets: &[u32],
    tune_end: usize,
    unigram: &Unigram,
    address_ops: u64,
    best_ngram_nll: f64,
    best_ngram_top1: f64,
    notes: &str,
) -> ArmReport {
    let (address_stability, stable_positions) = stability(keys, tune_end);
    let static_cache = eval_cache(keys, targets, tune_end, false, unigram);
    let causal_cache = eval_cache(keys, targets, tune_end, true, unigram);
    let (hit_rate_on_stable, _) = hit_rate_on_stable(keys, targets, tune_end);
    let mut ops = sticky_ops(keys, tune_end, address_ops);
    let (median, p95, mean) = median_p95(&mut ops);
    let ops_always = address_ops + 1;
    let ratio_full = median as f64 / DENSE_MEASURED_INSPECTIONS as f64;
    let replaced = DENSE_MEASURED_INSPECTIONS as f64 * VOCAB_PROJECTION_SHARE;
    let ratio_replaced = median as f64 / replaced;
    let quality_pass = causal_cache.nll_majority <= best_ngram_nll
        && causal_cache.top1_majority >= best_ngram_top1;
    let cost_pass = ratio_full <= 0.10;
    let hit_pass = causal_cache.hit_rate >= 0.50;
    ArmReport {
        name: name.to_owned(),
        kind: kind.to_owned(),
        address_cardinality: cardinality,
        address_stability,
        hit_rate_on_stable,
        stable_positions,
        ops_address: address_ops,
        ops_sticky_median: median,
        ops_sticky_p95: p95,
        ops_sticky_mean: mean,
        ops_always,
        ops_ratio_full: ratio_full,
        ops_ratio_vocab_projection: ratio_replaced,
        static_cache,
        causal_cache,
        quality_pass,
        cost_pass,
        hit_pass,
        g0a_pass: quality_pass && cost_pass && hit_pass,
        notes: notes.to_owned(),
    }
}

#[allow(clippy::too_many_arguments)]
fn run(
    checkpoint: &Path,
    evaluator_path: &Path,
    out: &Path,
    device_name: &str,
    batch: usize,
    ngram_limit: usize,
) -> Result<()> {
    let started = Instant::now();
    let evaluator = load_evaluator(evaluator_path)?;
    let dev_path = verify_identity(&evaluator.document["dev_source"])?;
    let device = select_device(device_name)?;
    let model = JointModel::load(checkpoint, &device)?;
    if model.config.context != EVALUATION_CONTEXT || model.config.vocab_size != VOCAB {
        return Err(invalid(
            "joint-cache-eval requires context256 and vocabulary4096",
        ));
    }
    let checkpoint_weights_sha256 = sha256_file(&checkpoint.join("model.safetensors"))?;
    let parameter_count = model.parameter_count();
    let d = model.config.width;

    let tokens = read_u16(&dev_path)?;
    let blocks = tokens.len().saturating_sub(1) / EVALUATION_CONTEXT;
    if blocks <= CALIBRATION_BLOCKS {
        return Err(invalid(
            "joint-cache-eval needs more than 64 development blocks",
        ));
    }
    let n_total = blocks * EVALUATION_CONTEXT;
    let tune_end = CALIBRATION_BLOCKS * EVALUATION_CONTEXT;

    let mut targets = vec![0u32; n_total];
    let mut model_pred = vec![0u32; n_total];
    let mut model_nll = vec![f32::NAN; n_total];
    let mut model_noread_pred = vec![0u32; n_total];
    let mut model_noread_nll = vec![f32::NAN; n_total];
    let mut states = vec![0f32; n_total * d];

    for pass in 0..2 {
        let read_mode = if pass == 0 {
            ReadMode::Enabled
        } else {
            ReadMode::NoRead
        };
        let mut cursor = 0usize;
        while cursor < blocks {
            let lane_count = batch.min(blocks - cursor);
            let mut ids = Vec::with_capacity(lane_count * EVALUATION_CONTEXT);
            for lane in 0..lane_count {
                let start = (cursor + lane) * EVALUATION_CONTEXT;
                ids.extend(
                    tokens[start..start + EVALUATION_CONTEXT]
                        .iter()
                        .map(|&token| u32::from(token)),
                );
            }
            let output = model.forward(&ids, lane_count, EVALUATION_CONTEXT, read_mode, false)?;
            let state_values = if pass == 0 {
                Some(output.states.flatten_all()?.to_vec1::<f32>()?)
            } else {
                None
            };
            let probability_values = output.probabilities.flatten_all()?.to_vec1::<f32>()?;
            for lane in 0..lane_count {
                let block = cursor + lane;
                let start = block * EVALUATION_CONTEXT;
                if let Some(state_values) = &state_values {
                    for position in 0..EVALUATION_CONTEXT {
                        let source = (lane * EVALUATION_CONTEXT + position) * d;
                        let destination = (block * EVALUATION_CONTEXT + position) * d;
                        states[destination..destination + d]
                            .copy_from_slice(&state_values[source..source + d]);
                    }
                }
                for position in 0..EVALUATION_CONTEXT {
                    let index = block * EVALUATION_CONTEXT + position;
                    let target = u32::from(tokens[start + position + 1]);
                    let row_start = (lane * EVALUATION_CONTEXT + position) * VOCAB;
                    let score = score_probabilities(
                        &probability_values[row_start..row_start + VOCAB],
                        target,
                    )?;
                    if pass == 0 {
                        targets[index] = target;
                        model_pred[index] = score.predicted_token;
                        model_nll[index] = score.nll_nats as f32;
                    } else {
                        model_noread_pred[index] = score.predicted_token;
                        model_noread_nll[index] = score.nll_nats as f32;
                    }
                }
            }
            cursor += lane_count;
            eprintln!(
                "joint-cache-eval pass {} forward {}/{} blocks",
                pass,
                cursor.min(blocks),
                blocks
            );
        }
    }

    // Count n-gram tables on a bounded prefix of the first retained train store.
    let train_identity = evaluator.document["train_sources"]
        .as_array()
        .and_then(|sources| sources.first())
        .ok_or_else(|| invalid("evaluator train source"))?;
    let train_path = verify_identity(train_identity)?;
    let train_tokens = read_u16_prefix(&train_path, ngram_limit)?;
    let mut unigram_counts = [0u64; VOCAB];
    let (ngram_tables, train_total) = build_ngram(&train_tokens, &mut unigram_counts);
    let unigram = unigram_from(&unigram_counts, train_total);
    let lambda2 = calibrate_lambda(2, &ngram_tables, &tokens, &unigram, 0, tune_end);
    let lambda3 = calibrate_lambda(3, &ngram_tables, &tokens, &unigram, 0, tune_end);
    let (ngram2_nll, ngram2_top1, _) = eval_ngram(
        2,
        &ngram_tables,
        &tokens,
        &unigram,
        lambda2,
        tune_end,
        n_total,
    );
    let (ngram3_nll, ngram3_top1, _) = eval_ngram(
        3,
        &ngram_tables,
        &tokens,
        &unigram,
        lambda3,
        tune_end,
        n_total,
    );
    let best_ngram_nll = ngram2_nll.min(ngram3_nll);
    let best_ngram_top1 = ngram2_top1.max(ngram3_top1);

    let comparison = n_total - tune_end;
    let mut model_read_nll = 0f64;
    let mut model_read_correct = 0usize;
    let mut model_noread_nll_sum = 0f64;
    let mut model_noread_correct = 0usize;
    let mut unigram_correct = 0usize;
    for index in tune_end..n_total {
        let target = targets[index];
        model_read_nll += f64::from(model_nll[index]);
        model_read_correct += usize::from(model_pred[index] == target);
        model_noread_nll_sum += f64::from(model_noread_nll[index]);
        model_noread_correct += usize::from(model_noread_pred[index] == target);
        unigram_correct += usize::from(unigram.best == target);
    }
    let model_read = (
        model_read_nll / comparison as f64,
        model_read_correct as f64 / comparison as f64,
    );
    let model_noread = (
        model_noread_nll_sum / comparison as f64,
        model_noread_correct as f64 / comparison as f64,
    );
    let unigram_top1 = unigram_correct as f64 / comparison as f64;

    // Canonical-address arms.
    let roots = group_roots();
    let mut arms: Vec<ArmReport> = Vec::new();
    let group_keys: Vec<u128> = (0..n_total)
        .map(|index| group_address(&states[index * d..(index + 1) * d], &roots) as u128)
        .collect();
    arms.push(arm_report(
        "group2i120",
        "fixed_group_cell",
        GROUP_ORDER,
        &group_keys,
        &targets,
        tune_end,
        &unigram,
        group_ops(d),
        best_ngram_nll,
        best_ngram_top1,
        "signed aggregate quaternion -> nearest exact 2I root; chirality preserved",
    ));

    for v in [64usize, 120, 256, 1024] {
        let centroids = kmeans_fit(&states, tune_end, d, v, 0x5EED_0000 ^ v as u64);
        let assigned = assign_all(&states, n_total, d, &centroids, v);
        let keys: Vec<u128> = assigned.iter().map(|&key| key as u128).collect();
        arms.push(arm_report(
            &format!("voronoi{v}"),
            "kmeans_state",
            v,
            &keys,
            &targets,
            tune_end,
            &unigram,
            (v * d) as u64,
            best_ngram_nll,
            best_ngram_top1,
            "k-means over the full 256-d recurrent state, fit on the 64 tune blocks",
        ));
    }

    // Matched-access controls: exact-context caches.
    let mut controls: Vec<ArmReport> = Vec::new();
    for k in [2usize, 4, 8] {
        let keys = context_keys(&tokens, k, n_total);
        let ops_always = (k as u64) + 2;
        let mut control = arm_report(
            &format!("exact_context_k{k}"),
            "exact_token_hash",
            0,
            &keys,
            &targets,
            tune_end,
            &unigram,
            (k as u64) + 1,
            best_ngram_nll,
            best_ngram_top1,
            "causal cache keyed by the last k observed tokens, reset per 256 block",
        );
        control.ops_sticky_median = ops_always;
        control.ops_sticky_p95 = ops_always;
        control.ops_sticky_mean = ops_always as f64;
        control.ops_always = ops_always;
        control.ops_ratio_full = ops_always as f64 / DENSE_MEASURED_INSPECTIONS as f64;
        control.ops_ratio_vocab_projection =
            ops_always as f64 / (DENSE_MEASURED_INSPECTIONS as f64 * VOCAB_PROJECTION_SHARE);
        control.cost_pass = control.ops_ratio_full <= 0.10;
        controls.push(control);
    }

    let verdict = json!({
        "criteria": {
            "quality": "causal-cache NLL <= better order-2/3 n-gram NLL and top-1 >= that n-gram",
            "cost": "sticky median cache-path ops/token <= 10% of the 1,672,448-inspection dense full-read step",
            "hit": "causal cache-hit rate >= 50%",
        },
        "best_ngram": {"nll": best_ngram_nll, "top1": best_ngram_top1,
            "source": "order-2 and order-3 interpolated counts on a bounded train prefix"},
        "pass": arms.iter().any(|arm| arm.g0a_pass),
        "passing_arms": arms.iter().filter(|arm| arm.g0a_pass).map(|arm| arm.name.clone()).collect::<Vec<_>>(),
    });

    let report = json!({
        "schema": "uor-r4.canonical-address-cache-eval/1",
        "status": "COMPLETE",
        "scope": "evaluation-only G0a feasibility on exposed evaluator-v2 development text with a retained continuous joint checkpoint; no training, no artifact modification",
        "source_commit": option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND"),
        "executable_sha256": sha256_file(&std::env::current_exe()?)?,
        "checkpoint": checkpoint,
        "checkpoint_weights_sha256": checkpoint_weights_sha256,
        "evaluator_sha256": evaluator.sha256,
        "model_config": &model.config,
        "parameter_count": parameter_count,
        "development": {
            "blocks": blocks, "tune_blocks": CALIBRATION_BLOCKS, "comparison_blocks": blocks - CALIBRATION_BLOCKS,
            "context": EVALUATION_CONTEXT, "vocabulary": VOCAB, "scored_targets": n_total,
            "comparison_targets": comparison, "batch": batch, "device": device_name,
        },
        "dense_reference": {
            "parameter_count": parameter_count,
            "measured_full_read_inspections": DENSE_MEASURED_INSPECTIONS,
            "vocab_projection_share": VOCAB_PROJECTION_SHARE,
            "vocab_projection_inspections": (DENSE_MEASURED_INSPECTIONS as f64 * VOCAB_PROJECTION_SHARE).round(),
        },
        "model_readout": {"nll": model_read.0, "top1": model_read.1, "kind": "read-enabled recurrent learner forward"},
        "model_noread": {"nll": model_noread.0, "top1": model_noread.1, "kind": "whole-prefix NoRead forward"},
        "baselines": {
            "unigram": {"nll": null, "top1": unigram_top1, "note": "top1 only; a unigram NLL is not the comparison quantity"},
            "ngram_order2": {"lambda": lambda2, "nll": ngram2_nll, "top1": ngram2_top1},
            "ngram_order3": {"lambda": lambda3, "nll": ngram3_nll, "top1": ngram3_top1},
            "ngram_train_tokens": train_total,
            "ngram_train_source": train_identity,
        },
        "arms": &arms,
        "controls": &controls,
        "verdict": verdict,
        "elapsed_seconds": started.elapsed().as_secs_f64(),
        "limitations": [
            "cache-path ops count address computation + table read only; the recurrent state update that produces the address is not counted, while the dense denominator counts the full learned step",
            "the state is a floating-point offline learner state, not a canonical integer/group state; 2I is used only as a fixed address codebook",
            "exposed development text, one seed; not a fresh final evaluation",
        ],
    });

    std::fs::write(
        out.join("cache-eval.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    let markdown = render_markdown(
        &report,
        &arms,
        &controls,
        ngram2_nll,
        ngram2_top1,
        ngram3_nll,
        ngram3_top1,
        model_read.0,
        model_read.1,
        model_noread.0,
        unigram_top1,
    );
    std::fs::write(out.join("summary.md"), markdown)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_markdown(
    report: &Value,
    arms: &[ArmReport],
    controls: &[ArmReport],
    ngram2_nll: f64,
    ngram2_top1: f64,
    ngram3_nll: f64,
    ngram3_top1: f64,
    model_nll: f64,
    model_top1: f64,
    noread_nll: f64,
    unigram_top1: f64,
) -> String {
    let mut out = String::new();
    out.push_str("# G0a canonical-address cache — machine summary\n\n");
    out.push_str(&format!(
        "Checkpoint weights SHA-256: `{}`\n\n",
        report["checkpoint_weights_sha256"].as_str().unwrap_or("?")
    ));
    out.push_str(&format!(
        "Development: {} scored targets ({} comparison); evaluator `{}`.\n\n",
        report["development"]["scored_targets"]
            .as_u64()
            .unwrap_or(0),
        report["development"]["comparison_targets"]
            .as_u64()
            .unwrap_or(0),
        report["evaluator_sha256"].as_str().unwrap_or("?"),
    ));
    out.push_str("## Pareto (comparison partition)\n\n");
    out.push_str(
        "| arm | kind | hit% | top1% | NLL | sticky ops | % dense | always % dense | verdict |\n",
    );
    out.push_str("|---|---|---:|---:|---:|---:|---:|---:|---|\n");
    let row = |out: &mut String, arm: &ArmReport| {
        out.push_str(&format!(
            "| {} | {} | {:.2} | {:.2} | {:.4} | {} | {:.3} | {:.3} | {} |\n",
            arm.name,
            arm.kind,
            arm.causal_cache.hit_rate * 100.0,
            arm.causal_cache.top1_majority * 100.0,
            arm.causal_cache.nll_majority,
            arm.ops_sticky_median,
            arm.ops_ratio_full * 100.0,
            arm.ops_always as f64 / DENSE_MEASURED_INSPECTIONS as f64 * 100.0,
            if arm.g0a_pass { "PASS" } else { "miss" },
        ));
    };
    for arm in arms {
        row(&mut out, arm);
    }
    for arm in controls {
        row(&mut out, arm);
    }
    out.push_str(&format!(
        "| model readout | recurrent_read | — | {:.2} | {:.4} | {} | 100 | 100 | reference |\n",
        model_top1 * 100.0,
        model_nll,
        DENSE_MEASURED_INSPECTIONS
    ));
    out.push_str(&format!(
        "| unigram | count | — | {:.2} | — | 1 | ~0 | ~0 | floor |\n",
        unigram_top1 * 100.0
    ));
    out.push_str(&format!(
        "| ngram order2 | count | — | {:.2} | {:.4} | 3 | ~0 | ~0 | control |\n",
        ngram2_top1 * 100.0,
        ngram2_nll
    ));
    out.push_str(&format!(
        "| ngram order3 | count | — | {:.2} | {:.4} | 4 | ~0 | ~0 | control |\n",
        ngram3_top1 * 100.0,
        ngram3_nll
    ));
    out.push_str(&format!(
        "| model NoRead | recurrent_no_read | — | — | {:.4} | {} | 100 | 100 | control |\n",
        noread_nll, DENSE_MEASURED_INSPECTIONS
    ));
    out.push_str("\n## Address stability and static (frozen) cache\n\n");
    out.push_str("| arm | address stability% | hit% on stable addresses | static hit% | static top1% | static NLL |\n");
    out.push_str("|---|---:|---:|---:|---:|---:|\n");
    for arm in arms.iter().chain(controls.iter()) {
        out.push_str(&format!(
            "| {} | {:.2} | {:.2} | {:.2} | {:.2} | {:.4} |\n",
            arm.name,
            arm.address_stability * 100.0,
            arm.hit_rate_on_stable * 100.0,
            arm.static_cache.hit_rate * 100.0,
            arm.static_cache.top1_majority * 100.0,
            arm.static_cache.nll_majority,
        ));
    }
    out.push_str("\n## Causal hit rate by position-in-block\n\n");
    out.push_str("| arm | 0-15 | 16-31 | 32-63 | 64-127 | 128-191 | 192-255 |\n");
    out.push_str("|---|---:|---:|---:|---:|---:|---:|\n");
    for arm in arms.iter().chain(controls.iter()) {
        let by = &arm.causal_cache.hit_by_position;
        let bucket = |range: std::ops::Range<usize>| -> f64 {
            let values = &by[range];
            if values.is_empty() {
                0.0
            } else {
                values.iter().sum::<f64>() / values.len() as f64
            }
        };
        out.push_str(&format!(
            "| {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
            arm.name,
            bucket(0..16) * 100.0,
            bucket(16..32) * 100.0,
            bucket(32..64) * 100.0,
            bucket(64..128) * 100.0,
            bucket(128..192) * 100.0,
            bucket(192..256) * 100.0,
        ));
    }
    out.push_str("\n## Causal hit rate by distance since last same-address occurrence\n\n");
    out.push_str("| arm | 1 | 2-3 | 4-7 | 8-15 | 16-31 | 32-63 | 64+ |\n");
    out.push_str("|---|---:|---:|---:|---:|---:|---:|---:|\n");
    for arm in arms.iter().chain(controls.iter()) {
        let by = &arm.causal_cache.hit_by_gap;
        out.push_str(&format!(
            "| {} | {} |\n",
            arm.name,
            by.iter()
                .map(|value| format!("{:.1}", value * 100.0))
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    out.push_str(&format!(
        "\n## G0a verdict\n\n```json\n{}\n```\n",
        serde_json::to_string_pretty(&report["verdict"]).unwrap_or_default()
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_counts_sums_equal_keys() {
        let mut values = vec![(3u64, 1u32), (1, 2), (3, 4), (2, 1)];
        merge_counts(&mut values);
        assert_eq!(values, vec![(1, 2), (2, 1), (3, 5)]);
    }

    #[test]
    fn context_keys_reset_at_block_boundaries() {
        let tokens: Vec<u16> = (0..(EVALUATION_CONTEXT * 2 + 1) as u16).collect();
        let keys = context_keys(&tokens, 2, EVALUATION_CONTEXT * 2);
        assert_eq!(keys[0], u128::MAX);
        assert_eq!(keys[1], 1);
        assert_eq!(keys[EVALUATION_CONTEXT], u128::MAX);
        assert_eq!(
            keys[EVALUATION_CONTEXT + 1],
            u128::from(EVALUATION_CONTEXT as u16) * VOCAB as u128
                + u128::from(EVALUATION_CONTEXT as u16 + 1)
        );
    }

    #[test]
    fn group_address_is_range_bounded_and_deterministic() {
        let roots = group_roots();
        assert_eq!(roots.len(), GROUP_ORDER);
        let row: Vec<f32> = (0..256).map(|i| ((i % 7) as f32) - 3.0).collect();
        let a = group_address(&row, &roots);
        let b = group_address(&row, &roots);
        assert_eq!(a, b);
        assert!((a as usize) < GROUP_ORDER);
    }

    #[test]
    fn ngram_lookup_and_successors_are_consistent() {
        let tokens: Vec<u16> = vec![5, 7, 9, 7, 9, 11, 7, 9, 13];
        let mut counts = [0u64; VOCAB];
        let (tables, total) = build_ngram(&tokens, &mut counts);
        assert_eq!(total, tokens.len() as u64);
        assert_eq!(lookup_count(&tables.order2_pairs, (7u64 << 12) | 9), 3);
        let pairs = &tables.order2_pairs;
        let low_key = 9u64 << 12;
        let low = pairs.partition_point(|entry| entry.0 < low_key);
        let high = pairs.partition_point(|entry| entry.0 < low_key + VOCAB as u64);
        let successors: Vec<u32> = pairs[low..high]
            .iter()
            .map(|entry| (entry.0 & 0xFFF) as u32)
            .collect();
        assert_eq!(successors, vec![7, 11, 13]);
    }

    #[test]
    fn causal_cache_hits_after_first_occurrence() {
        let keys = vec![5u128, 5, 5, 7, 7];
        let targets = vec![1u32, 2, 1, 3, 3];
        let unigram = Unigram {
            p: vec![1.0 / VOCAB as f64; VOCAB],
            best: 0,
        };
        let metric = eval_cache(&keys, &targets, 0, true, &unigram);
        assert_eq!(metric.positions, 5);
        assert_eq!(metric.hits, 3);
    }
}
