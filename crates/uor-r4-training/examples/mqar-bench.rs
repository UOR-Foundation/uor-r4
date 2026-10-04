//! MQAR read diagnostic (References #820): train a small geometric stack
//! (`uor_r4_training::geometric_stack`) on synthetic multi-query associative
//! recall and measure in-context recall accuracy by query distance.
//!
//! ```text
//! mqar-bench out=NEW_REPORT_ROOT [pattern=aaaaaa] [read=l2|dot|lorentz] [rotation=true|false] \
//!   [device=cpu|metal] [width=128] [heads=4] [mlp=384] [context=512] [pairs_per_bucket=8] \
//!   [batch=8] [steps=1500] [lr=0.001] [warmup=100] [min_lr=0.1] [weight_decay=0.1] [clip=1.0] \
//!   [eval_every=100] [curve_sequences=8] [final_sequences=64] [seed=1] [max_seconds=1200]
//! ```
//!
//! Each sequence is a fixed-length window of filler tokens holding
//! `pairs_per_bucket` key-value pairs per distance bucket. A pair writes its
//! key at position `p` and its value at `p + 1`; its single query writes the key
//! again at `q = p + d`, and the training target at `q` (the only weighted
//! position) is the value. The distance `d = q - p` is drawn uniformly inside
//! the pair's bucket, so no fixed offset reaches the key. Keys, values and
//! filler come from disjoint token ranges; keys and values are distinct inside
//! a sequence. Every sequence draws fresh pairings from its own seed, so a key's
//! value cannot be memorized. The pairing space is split by
//! `(key_index + value_index) % 4`: training draws only classes 1..3, and the
//! primary evaluation draws only class 0, pairings no training sequence ever
//! contains. A second evaluation reports fresh pairings of the training class.
//!
//! The report root is claimed exclusively before any model work and sealed
//! with its manifest at the end. Offline floating-point training only; nothing
//! here is a serving path, and there is no transformer control.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{Device, Tensor};
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_training::geometric_stack::{
    ReadScore, RotationGroup, StackAdamW, StackArch, StackConfig, StackModel,
};
use uor_r4_training::{Result, TrainingError};

const VOCAB: usize = 512;
/// Filler tokens: noise between the pairs and queries.
const FILLER: (u32, u32) = (8, 64);
/// Key tokens.
const KEYS: (u32, u32) = (64, 288);
/// Value tokens.
const VALUES: (u32, u32) = (288, 512);
/// Pairing classes `(key_index + value_index) % CLASSES`; class 0 is held out.
const CLASSES: u32 = 4;
const HELD_OUT_CLASS: u32 = 0;
/// Rejection-sampling attempts per pair placement.
const PLACEMENT_ATTEMPTS: usize = 10_000;

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

/// An inclusive range of query-to-key distances.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Bucket {
    name: &'static str,
    low: usize,
    high: usize,
}

const ALL_BUCKETS: [Bucket; 5] = [
    Bucket {
        name: "d16",
        low: 12,
        high: 24,
    },
    Bucket {
        name: "d64",
        low: 48,
        high: 96,
    },
    Bucket {
        name: "d200",
        low: 150,
        high: 300,
    },
    Bucket {
        name: "d400",
        low: 350,
        high: 480,
    },
    Bucket {
        name: "d1000",
        low: 1000,
        high: 1100,
    },
];

/// The buckets a window of `context` tokens can hold (a query at `p + d`
/// needs `d <= context - 1`).
fn buckets_for(context: usize) -> Vec<Bucket> {
    ALL_BUCKETS
        .iter()
        .copied()
        .filter(|bucket| bucket.high < context)
        .collect()
}

/// SplitMix64.
#[derive(Clone)]
struct Rng(u64);

impl Rng {
    fn new(seed: u64, domain: u64, index: u64) -> Self {
        let mut rng = Rng(seed ^ domain.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        rng.0 ^= rng
            .next()
            .wrapping_add(index.wrapping_mul(0xD1B5_4A32_D192_ED03));
        rng.next();
        rng
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `low..=high`.
    fn range(&mut self, low: usize, high: usize) -> usize {
        low + (self.next() % (high - low + 1) as u64) as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pairing {
    /// Classes other than the held-out one.
    Train,
    /// The held-out class only.
    HeldOut,
}

impl Pairing {
    fn admits(self, key: u32, value: u32) -> bool {
        let class = ((key - KEYS.0) + (value - VALUES.0)) % CLASSES;
        match self {
            Pairing::Train => class != HELD_OUT_CLASS,
            Pairing::HeldOut => class == HELD_OUT_CLASS,
        }
    }
}

#[derive(Clone, Debug)]
struct Query {
    position: usize,
    distance: usize,
    bucket: usize,
    key: u32,
    value: u32,
}

#[derive(Clone, Debug)]
struct Sequence {
    tokens: Vec<u32>,
    queries: Vec<Query>,
}

fn generate(
    rng: &mut Rng,
    context: usize,
    buckets: &[Bucket],
    pairs_per_bucket: usize,
    pairing: Pairing,
) -> Result<Sequence> {
    let pairs = buckets.len() * pairs_per_bucket;
    if pairs == 0
        || pairs > (KEYS.1 - KEYS.0) as usize
        || pairs > (VALUES.1 - VALUES.0) as usize
        || 3 * pairs > context / 2
    {
        return Err(invalid("too many or too few pairs for the window"));
    }
    let filler = (FILLER.1 - FILLER.0) as u64;
    let mut tokens: Vec<u32> = (0..context)
        .map(|_| FILLER.0 + (rng.next() % filler) as u32)
        .collect();
    let mut used = vec![false; context];
    let mut keys_used = vec![false; (KEYS.1 - KEYS.0) as usize];
    let mut values_used = vec![false; (VALUES.1 - VALUES.0) as usize];
    let mut queries = Vec::with_capacity(pairs);
    // The longest buckets first: their keys have the fewest admissible slots.
    let mut order: Vec<usize> = (0..buckets.len()).collect();
    order.sort_by_key(|&b| std::cmp::Reverse(buckets[b].high));
    for &b in &order {
        let bucket = buckets[b];
        for _ in 0..pairs_per_bucket {
            let key = loop {
                let index = (rng.next() % keys_used.len() as u64) as usize;
                if !keys_used[index] {
                    keys_used[index] = true;
                    break KEYS.0 + index as u32;
                }
            };
            let mut value = None;
            for _ in 0..PLACEMENT_ATTEMPTS {
                let index = (rng.next() % values_used.len() as u64) as usize;
                let candidate = VALUES.0 + index as u32;
                if !values_used[index] && pairing.admits(key, candidate) {
                    values_used[index] = true;
                    value = Some(candidate);
                    break;
                }
            }
            let value = value.ok_or_else(|| invalid("no admissible value for a key"))?;
            let mut placed = None;
            for _ in 0..PLACEMENT_ATTEMPTS {
                let distance = rng.range(bucket.low, bucket.high);
                let key_position = rng.range(0, context - 1 - distance);
                let query = key_position + distance;
                if !used[key_position] && !used[key_position + 1] && !used[query] {
                    placed = Some((key_position, distance));
                    break;
                }
            }
            let (key_position, distance) =
                placed.ok_or_else(|| invalid("could not place a pair in the window"))?;
            let position = key_position + distance;
            for p in [key_position, key_position + 1, position] {
                used[p] = true;
            }
            tokens[key_position] = key;
            tokens[key_position + 1] = value;
            tokens[position] = key;
            queries.push(Query {
                position,
                distance,
                bucket: b,
                key,
                value,
            });
        }
    }
    queries.sort_by_key(|query| query.position);
    Ok(Sequence { tokens, queries })
}

/// Inputs, targets and weights for `weighted_loss`: weight 1 on queries only.
fn batch_arrays(sequences: &[Sequence], context: usize) -> (Vec<u32>, Vec<u32>, Vec<f32>) {
    let total = sequences.len() * context;
    let (mut ids, mut targets, mut weights) = (
        Vec::with_capacity(total),
        vec![0u32; total],
        vec![0f32; total],
    );
    for (s, sequence) in sequences.iter().enumerate() {
        ids.extend_from_slice(&sequence.tokens);
        for query in &sequence.queries {
            targets[s * context + query.position] = query.value;
            weights[s * context + query.position] = 1.0;
        }
    }
    (ids, targets, weights)
}

#[derive(Clone, Default)]
struct Tally {
    correct: usize,
    total: usize,
    nll: f64,
    distance: usize,
    /// Wrong answers that are another pair's value in the same window.
    other_pair_value: usize,
    /// Wrong answers that are a value token absent from the window.
    absent_value: usize,
    /// Wrong answers that echo the query key.
    echoed_key: usize,
}

/// Recall by bucket: argmax over the whole vocabulary at each query.
fn evaluate(
    model: &StackModel,
    sequences: &[Sequence],
    context: usize,
    buckets: &[Bucket],
    chunk: usize,
) -> Result<Value> {
    let mut tallies = vec![Tally::default(); buckets.len()];
    for group in sequences.chunks(chunk.max(1)) {
        let (ids, _, _) = batch_arrays(group, context);
        let logits = model.forward(&ids, group.len(), context)?;
        let mut rows = Vec::new();
        for (s, sequence) in group.iter().enumerate() {
            for query in &sequence.queries {
                rows.push((s * context + query.position) as u32);
            }
        }
        let index = Tensor::from_vec(rows.clone(), rows.len(), model.device())?;
        let selected = logits
            .index_select(&index, 0)?
            .to_device(&Device::Cpu)?
            .to_vec2::<f32>()?;
        let mut row = 0;
        for sequence in group {
            let in_window: Vec<u32> = sequence.queries.iter().map(|q| q.value).collect();
            for query in &sequence.queries {
                let scores = &selected[row];
                row += 1;
                let (argmax, maximum) =
                    scores
                        .iter()
                        .enumerate()
                        .fold((0usize, f32::NEG_INFINITY), |best, (i, &v)| {
                            if v > best.1 {
                                (i, v)
                            } else {
                                best
                            }
                        });
                let sum: f64 = scores.iter().map(|&v| f64::from(v - maximum).exp()).sum();
                let nll = sum.ln() - f64::from(scores[query.value as usize] - maximum);
                let tally = &mut tallies[query.bucket];
                tally.total += 1;
                tally.correct += usize::from(argmax == query.value as usize);
                tally.nll += nll;
                tally.distance += query.distance;
                let predicted = argmax as u32;
                if predicted != query.value {
                    if in_window.contains(&predicted) {
                        tally.other_pair_value += 1;
                    } else if (VALUES.0..VALUES.1).contains(&predicted) {
                        tally.absent_value += 1;
                    } else if predicted == query.key {
                        tally.echoed_key += 1;
                    }
                }
            }
        }
    }
    let mut by_bucket = serde_json::Map::new();
    let (mut correct, mut total) = (0, 0);
    for (bucket, tally) in buckets.iter().zip(&tallies) {
        correct += tally.correct;
        total += tally.total;
        by_bucket.insert(
            bucket.name.into(),
            json!({
                "distance_low": bucket.low, "distance_high": bucket.high,
                "correct": tally.correct, "total": tally.total,
                "accuracy": tally.correct as f64 / tally.total.max(1) as f64,
                "mean_nll": tally.nll / tally.total.max(1) as f64,
                "mean_distance": tally.distance as f64 / tally.total.max(1) as f64,
                "wrong_other_pair_value": tally.other_pair_value,
                "wrong_absent_value": tally.absent_value,
                "wrong_echoed_key": tally.echoed_key,
            }),
        );
    }
    Ok(json!({
        "by_bucket": by_bucket,
        "correct": correct,
        "total": total,
        "accuracy": correct as f64 / total.max(1) as f64,
        "chance_accuracy": 1.0 / f64::from(VALUES.1 - VALUES.0),
    }))
}

fn cosine_rate(lr: f64, warmup: usize, min_lr: f64, steps: usize, step: usize) -> f64 {
    if step < warmup {
        return lr * (step + 1) as f64 / warmup as f64;
    }
    let span = (steps.saturating_sub(warmup)).max(1) as f64;
    let progress = ((step - warmup) as f64 / span).min(1.0);
    let floor = lr * min_lr;
    floor + (lr - floor) * 0.5 * (1.0 + (std::f64::consts::PI * progress).cos())
}

struct Args(BTreeMap<String, String>);

impl Args {
    fn parse() -> Result<Self> {
        let mut map = BTreeMap::new();
        for arg in std::env::args().skip(1) {
            let (key, value) = arg
                .split_once('=')
                .ok_or_else(|| invalid(format!("expected key=value, got {arg}")))?;
            if map.insert(key.to_owned(), value.to_owned()).is_some() {
                return Err(invalid(format!("{key} given twice")));
            }
        }
        Ok(Self(map))
    }

    fn take(&mut self, key: &str) -> Option<String> {
        self.0.remove(key)
    }

    fn parsed<T: std::str::FromStr>(&mut self, key: &str, default: T) -> Result<T> {
        match self.take(key) {
            None => Ok(default),
            Some(text) => text
                .parse()
                .map_err(|_| invalid(format!("invalid {key}={text}"))),
        }
    }

    fn finish(self) -> Result<()> {
        if let Some(key) = self.0.keys().next() {
            return Err(invalid(format!("unknown argument {key}")));
        }
        Ok(())
    }
}

struct Settings {
    out: PathBuf,
    config: StackConfig,
    device_name: String,
    pairs_per_bucket: usize,
    batch: usize,
    steps: usize,
    lr: f64,
    warmup: usize,
    min_lr: f64,
    weight_decay: f64,
    clip: f64,
    eval_every: usize,
    curve_sequences: usize,
    final_sequences: usize,
    seed: u64,
    max_seconds: f64,
}

fn settings() -> Result<Settings> {
    let mut args = Args::parse()?;
    let out = PathBuf::from(
        args.take("out")
            .ok_or_else(|| invalid("out=NEW_REPORT_ROOT is required"))?,
    );
    let pattern = args.take("pattern").unwrap_or_else(|| "aaaaaa".into());
    let read = match args.take("read").as_deref() {
        None | Some("l2") => ReadScore::L2,
        Some("dot") => ReadScore::Dot,
        Some("lorentz") => ReadScore::Lorentz,
        Some(other) => return Err(invalid(format!("invalid read={other}"))),
    };
    let rotation = args.parsed("rotation", true)?;
    let width = args.parsed("width", 128usize)?;
    let heads = args.parsed("heads", 4usize)?;
    let mlp_hidden = args.parsed("mlp", 384usize)?;
    let context = args.parsed("context", 512usize)?;
    let seed = args.parsed("seed", 1u64)?;
    let config = StackConfig {
        arch: StackArch::Geometric,
        vocab_size: VOCAB,
        width,
        heads,
        mlp_hidden,
        context,
        pattern,
        read,
        rotation,
        rotation_group: RotationGroup::Quaternion,
        seed,
        memory: None,
        select: None,
        pointer: None,
    };
    config.validate()?;
    let settings = Settings {
        out,
        config,
        device_name: args.take("device").unwrap_or_else(|| "cpu".into()),
        pairs_per_bucket: args.parsed("pairs_per_bucket", 8usize)?,
        batch: args.parsed("batch", 8usize)?,
        steps: args.parsed("steps", 1500usize)?,
        lr: args.parsed("lr", 1e-3f64)?,
        warmup: args.parsed("warmup", 100usize)?,
        min_lr: args.parsed("min_lr", 0.1f64)?,
        weight_decay: args.parsed("weight_decay", 0.1f64)?,
        clip: args.parsed("clip", 1.0f64)?,
        eval_every: args.parsed("eval_every", 100usize)?,
        curve_sequences: args.parsed("curve_sequences", 8usize)?,
        final_sequences: args.parsed("final_sequences", 64usize)?,
        seed,
        max_seconds: args.parsed("max_seconds", 1200f64)?,
    };
    args.finish()?;
    if settings.batch == 0
        || settings.steps == 0
        || settings.eval_every == 0
        || settings.curve_sequences == 0
        || settings.final_sequences == 0
        || !(settings.lr > 0.0)
        || !(settings.max_seconds > 0.0)
    {
        return Err(invalid(
            "batch, steps, eval_every, sequences, lr and max_seconds must be positive",
        ));
    }
    if buckets_for(settings.config.context).is_empty() {
        return Err(invalid("the context holds no distance bucket"));
    }
    Ok(settings)
}

const TRAIN_DOMAIN: u64 = 0x7472_6169_6E;
const CURVE_DOMAIN: u64 = 0x6375_7276_65;
const HELD_OUT_DOMAIN: u64 = 0x6865_6C64;
const IN_CLASS_DOMAIN: u64 = 0x636C_6173_73;

fn sequences(
    s: &Settings,
    buckets: &[Bucket],
    domain: u64,
    count: usize,
    pairing: Pairing,
) -> Result<Vec<Sequence>> {
    (0..count)
        .map(|i| {
            let mut rng = Rng::new(s.seed, domain, i as u64);
            generate(
                &mut rng,
                s.config.context,
                buckets,
                s.pairs_per_bucket,
                pairing,
            )
        })
        .collect()
}

fn run(s: &Settings, log: &mut fs::File) -> Result<Value> {
    use std::io::Write;
    let started = Instant::now();
    let device = uor_r4_training::baseline_protocol::device(&s.device_name)?;
    let buckets = buckets_for(s.config.context);
    let model = StackModel::new(s.config.clone(), &device)?;
    let mut optimizer = StackAdamW::new(&model, s.weight_decay, s.clip)?;
    let parameters = model.parameter_count();
    let line = format!(
        "pattern {} read {:?} rotation {}: {} parameters, context {}, buckets {:?}",
        s.config.pattern,
        s.config.read,
        s.config.rotation,
        parameters,
        s.config.context,
        buckets.iter().map(|b| b.name).collect::<Vec<_>>()
    );
    eprintln!("{line}");
    writeln!(log, "{line}")?;
    let curve_set = sequences(
        s,
        &buckets,
        CURVE_DOMAIN,
        s.curve_sequences,
        Pairing::HeldOut,
    )?;
    let chunk = s.batch;
    let mut curve = Vec::new();
    let initial = evaluate(&model, &curve_set, s.config.context, &buckets, chunk)?;
    curve.push(json!({"step": 0, "held_out": initial}));
    let (mut window_loss, mut window_steps) = (0f64, 0usize);
    let mut step_seconds = Vec::new();
    let mut stopped_early = false;
    let mut step = 0;
    while step < s.steps {
        if started.elapsed().as_secs_f64() > s.max_seconds {
            stopped_early = true;
            break;
        }
        let clock = Instant::now();
        let lr = cosine_rate(s.lr, s.warmup, s.min_lr, s.steps, step);
        let batch: Vec<Sequence> = (0..s.batch)
            .map(|b| {
                let mut rng = Rng::new(s.seed, TRAIN_DOMAIN, (step * s.batch + b) as u64);
                generate(
                    &mut rng,
                    s.config.context,
                    &buckets,
                    s.pairs_per_bucket,
                    Pairing::Train,
                )
            })
            .collect::<Result<_>>()?;
        let (ids, targets, weights) = batch_arrays(&batch, s.config.context);
        let loss = model.weighted_loss(&ids, &targets, &weights, s.batch, s.config.context)?;
        let value = f64::from(loss.to_scalar::<f32>()?);
        if !value.is_finite() {
            return Err(invalid(format!("nonfinite loss at step {step}")));
        }
        let grads = loss.backward()?;
        let grad_norm = optimizer.update(&model, &grads, lr)?;
        step += 1;
        step_seconds.push(clock.elapsed().as_secs_f64());
        window_loss += value;
        window_steps += 1;
        if step % s.eval_every == 0 || step == s.steps {
            let held_out = evaluate(&model, &curve_set, s.config.context, &buckets, chunk)?;
            let train_loss = window_loss / window_steps.max(1) as f64;
            let line = format!(
                "step {step} lr {lr:.2e} train query NLL {train_loss:.4} grad {grad_norm:.3} held-out acc {:.4} {} ({:.0}s)",
                held_out["accuracy"].as_f64().unwrap_or(f64::NAN),
                buckets
                    .iter()
                    .map(|b| format!(
                        "{}={:.3}",
                        b.name,
                        held_out["by_bucket"][b.name]["accuracy"]
                            .as_f64()
                            .unwrap_or(f64::NAN)
                    ))
                    .collect::<Vec<_>>()
                    .join(" "),
                started.elapsed().as_secs_f64()
            );
            eprintln!("{line}");
            writeln!(log, "{line}")?;
            curve.push(json!({
                "step": step, "lr": lr, "train_query_nll": train_loss,
                "grad_norm": grad_norm, "held_out": held_out,
                "elapsed_seconds": started.elapsed().as_secs_f64(),
            }));
            window_loss = 0.0;
            window_steps = 0;
        }
    }
    let train_seconds: f64 = step_seconds.iter().sum();
    let eval_clock = Instant::now();
    let held_out_set = sequences(
        s,
        &buckets,
        HELD_OUT_DOMAIN,
        s.final_sequences,
        Pairing::HeldOut,
    )?;
    let held_out = evaluate(&model, &held_out_set, s.config.context, &buckets, chunk)?;
    let in_class_set = sequences(
        s,
        &buckets,
        IN_CLASS_DOMAIN,
        s.final_sequences / 2,
        Pairing::Train,
    )?;
    let in_class = evaluate(&model, &in_class_set, s.config.context, &buckets, chunk)?;
    let line = format!(
        "final held-out acc {:.4}; in-class fresh acc {:.4}",
        held_out["accuracy"].as_f64().unwrap_or(f64::NAN),
        in_class["accuracy"].as_f64().unwrap_or(f64::NAN)
    );
    eprintln!("{line}");
    writeln!(log, "{line}")?;
    let mut sorted = step_seconds.clone();
    sorted.sort_by(f64::total_cmp);
    Ok(json!({
        "parameters": parameters,
        "steps_completed": step,
        "stopped_early_at_max_seconds": stopped_early,
        "final_held_out_pairings": held_out,
        "final_in_class_fresh_pairings": in_class,
        "curve": curve,
        "train_seconds": train_seconds,
        "median_step_seconds": sorted.get(sorted.len() / 2).copied(),
        "final_eval_seconds": eval_clock.elapsed().as_secs_f64(),
        "wall_seconds": started.elapsed().as_secs_f64(),
        "supervised_queries_seen": step * s.batch * buckets.len() * s.pairs_per_bucket,
    }))
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn main() -> Result<()> {
    let s = settings()?;
    // Claimed exclusively after argument validation, before any model work.
    report_output::claim(&s.out)?;
    let buckets = buckets_for(s.config.context);
    let config = json!({
        "stack_config": s.config,
        "device": s.device_name,
        "threads": std::env::var("RAYON_NUM_THREADS").ok(),
        "vocab": VOCAB,
        "token_ranges": {"filler": FILLER, "keys": KEYS, "values": VALUES},
        "pairing_split": {
            "classes": CLASSES, "held_out_class": HELD_OUT_CLASS,
            "rule": "(key - 64 + value - 288) % 4; training draws classes 1..3, the primary evaluation class 0",
        },
        "buckets": buckets.iter().map(|b| json!({"name": b.name, "low": b.low, "high": b.high})).collect::<Vec<_>>(),
        "pairs_per_bucket": s.pairs_per_bucket,
        "batch": s.batch, "steps": s.steps, "lr": s.lr, "warmup": s.warmup, "min_lr": s.min_lr,
        "weight_decay": s.weight_decay, "clip": s.clip, "eval_every": s.eval_every,
        "curve_sequences": s.curve_sequences, "final_sequences": s.final_sequences,
        "seed": s.seed, "max_seconds": s.max_seconds,
        "argv": std::env::args().collect::<Vec<_>>(),
    });
    write_json(&s.out.join("config.json"), &config)?;
    let mut log = fs::File::create(s.out.join("log.txt"))?;
    let result = run(&s, &mut log);
    drop(log);
    let report = match &result {
        Ok(run) => json!({"status": "complete", "config": config, "run": run}),
        Err(error) => json!({"status": "failed", "config": config, "error": error.to_string()}),
    };
    write_json(&s.out.join("report.json"), &report)?;
    report_output::seal(&s.out)?;
    report_output::verify(&s.out)?;
    result.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draw(domain: u64, count: usize, pairing: Pairing, context: usize) -> Vec<Sequence> {
        let buckets = buckets_for(context);
        (0..count)
            .map(|i| {
                let mut rng = Rng::new(7, domain, i as u64);
                generate(&mut rng, context, &buckets, 8, pairing).expect("generation")
            })
            .collect()
    }

    #[test]
    fn the_answer_sits_at_the_stated_distance() {
        for context in [1152, 512] {
            let buckets = buckets_for(context);
            for sequence in draw(TRAIN_DOMAIN, 64, Pairing::Train, context) {
                assert_eq!(sequence.tokens.len(), context);
                assert_eq!(sequence.queries.len(), 8 * buckets.len());
                for query in &sequence.queries {
                    let bucket = buckets[query.bucket];
                    assert!((bucket.low..=bucket.high).contains(&query.distance));
                    let key_position = query.position - query.distance;
                    assert_eq!(sequence.tokens[query.position], query.key);
                    assert_eq!(sequence.tokens[key_position], query.key);
                    assert_eq!(sequence.tokens[key_position + 1], query.value);
                    // The key occurs exactly twice (pair and query); the value
                    // exactly once, before the query.
                    let key_count = sequence.tokens.iter().filter(|&&t| t == query.key).count();
                    let value_positions: Vec<usize> = (0..context)
                        .filter(|&p| sequence.tokens[p] == query.value)
                        .collect();
                    assert_eq!(key_count, 2);
                    assert_eq!(value_positions, vec![key_position + 1]);
                    assert!(query.value >= VALUES.0 && query.value < VALUES.1);
                    assert!(query.key >= KEYS.0 && query.key < KEYS.1);
                }
            }
        }
    }

    #[test]
    fn pairings_are_fresh_and_the_held_out_class_never_trains() {
        let train = draw(TRAIN_DOMAIN, 256, Pairing::Train, 1152);
        let held = draw(HELD_OUT_DOMAIN, 64, Pairing::HeldOut, 1152);
        let mut seen: BTreeMap<u32, BTreeMap<u32, usize>> = BTreeMap::new();
        let mut total = 0usize;
        for sequence in &train {
            for query in &sequence.queries {
                assert!(Pairing::Train.admits(query.key, query.value));
                assert!(!Pairing::HeldOut.admits(query.key, query.value));
                *seen
                    .entry(query.key)
                    .or_default()
                    .entry(query.value)
                    .or_default() += 1;
                total += 1;
            }
        }
        // Fresh pairings: repeats of a (key, value) pairing across sequences
        // occur at the rate of independent uniform draws over the admissible
        // pairings (about total^2 / (2 * pairings)), never as a fixed mapping.
        let repeated: usize = seen
            .values()
            .flat_map(|values| values.values())
            .map(|&n| n - 1)
            .sum();
        let most: usize = seen
            .values()
            .flat_map(|values| values.values())
            .copied()
            .max()
            .unwrap_or(0);
        let admissible = (KEYS.1 - KEYS.0) as f64 * (VALUES.1 - VALUES.0) as f64 * 0.75;
        let chance = (total * total) as f64 / (2.0 * admissible);
        assert!(
            (repeated as f64) < 1.5 * chance,
            "{repeated} repeats of {total}, chance {chance}"
        );
        assert!(most <= 8, "a pairing recurs {most} times");
        let keys_with_one_value = seen.values().filter(|values| values.len() == 1).count();
        assert_eq!(keys_with_one_value, 0);
        for sequence in &held {
            for query in &sequence.queries {
                assert!(Pairing::HeldOut.admits(query.key, query.value));
                assert!(
                    seen.get(&query.key)
                        .is_none_or(|values| !values.contains_key(&query.value)),
                    "a held-out pairing appeared in training"
                );
            }
        }
        // Distinct sequences differ (no reused draws across sequences).
        assert_ne!(train[0].tokens, train[1].tokens);
        let again = draw(TRAIN_DOMAIN, 1, Pairing::Train, 1152);
        assert_eq!(again[0].tokens, train[0].tokens);
    }

    #[test]
    fn weights_fall_on_queries_only() {
        let batch = draw(CURVE_DOMAIN, 2, Pairing::HeldOut, 1152);
        let (ids, targets, weights) = batch_arrays(&batch, 1152);
        assert_eq!(ids.len(), 2 * 1152);
        let weighted: Vec<usize> = (0..weights.len()).filter(|&i| weights[i] > 0.0).collect();
        assert_eq!(weighted.len(), 2 * 8 * buckets_for(1152).len());
        for &i in &weighted {
            assert!(targets[i] >= VALUES.0 && targets[i] < VALUES.1);
            assert!(ids[i] >= KEYS.0 && ids[i] < KEYS.1);
        }
    }
}
