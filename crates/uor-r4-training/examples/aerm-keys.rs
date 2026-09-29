//! G v2 frozen-trunk key probe (Lab 2, #973 comment 5890046442).
//!
//! A frozen S4 trunk is wrapped by four newly trained heads — a five-way tag
//! head, a four-way trigger head, an eight-way entity-atom head read at the
//! entity slot and a fourteen-way relation-atom head read at the trigger
//! position — on the D2-natural v2 world. The one compared factor is the atom
//! head parametrization: a learned linear map to a quaternion scored against
//! fixed unit icosians (`2I`), versus a plain K-way softmax over the same
//! atoms. Keys are the semiprime of the entity-atom prime and the relation-atom
//! prime, whose primes are reserved above the vocabulary band.
//!
//! ```text
//! aerm-keys run out=NEW_REPORT_ROOT trunk=TRUNK_DIR tokenizer=TOKENIZER.json \
//!   [splits=2,4] [steps=1500] [batch=16] [context=256] [lr=0.003] [warmup=100] \
//!   [weight_decay=0.1] [clip=1] [trigger_positive=5] [eval_episodes=512] \
//!   [eval_batch=16] [dev_episodes=256] [seed=9001] [data_seed=9002] \
//!   [eval_seed=9003] [dev_seed=9004]
//! aerm-keys roots
//! ```
//!
//! `run` claims an exclusive report root before loading anything, refuses a
//! trunk whose `model.safetensors` SHA-256 is not the pinned S4 arm-A value,
//! loads it through `AermModel::from_stack` with its saved icosian snap
//! restored, trains only the four heads (the trunk feature is detached, so no
//! gradient reaches it), evaluates held-out phrasings per phenomenon and per
//! class, and seals the report. `roots` prints the fixed 2I assignment. Set
//! `RAYON_NUM_THREADS=2` and `nice` for the fit.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{backprop::GradStore, DType, Device, Tensor, Var, D};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde::Serialize;
use serde_json::{json, Value};

use uor_r4_core::prime_route_attention::{PrimeAtom, SemiprimeExpert};
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::geometric_stack::{logits_cross_entropy, StackModel, TransportSnap};
use uor_r4_training::stack_aerm::{
    entity_atom, AermModel, AtomStore, Clause, DialogueBatch, Episode, Phenomenon, RelationWorld,
    STATUS_ABSENT, STATUS_HIT, TAG_ENTITY, TAG_RELATION, TAG_VALUE, TRIGGER_NONE, TRIGGER_WRITE,
};
use uor_r4_training::stack_tracking::Rng;
use uor_r4_training::{sha256_file, Result, TrainingError};

/// The pinned S4 arm-A trunk (`kimi-s4-fit-20260929/arm-a-snap-1/model`).
const TRUNK_SHA256: &str = "2b3b568141dfe213dd85e4d67e9ce0018e802f0654ee73ea10c7e552fc327b96";

const ENTITY_ATOMS: usize = 8;
const RELATION_ATOMS: usize = 14;
/// Every pair of assigned roots has `|q . r|` at most this.
const ROOT_BOUND: f32 = 0.5;
#[cfg(test)]
const ROOT_BOUND_TOLERANCE: f32 = 1e-6;

const TAG_CLASSES: usize = 5;
const TAG5_OTHER: u32 = 0;
const TAG5_SPEAKER: u32 = 1;
const TAG5_ENTITY: u32 = 2;
const TAG5_RELATION: u32 = 3;
const TAG5_VALUE: u32 = 4;

const TRIGGER_CLASSES: usize = 4;

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

// ---------------------------------------------------------------------------
// Arguments.

struct Args(BTreeMap<String, String>);

impl Args {
    fn parse(arguments: &[String], allowed: &[&str]) -> Result<Self> {
        let mut pairs = BTreeMap::new();
        for argument in arguments {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| invalid(format!("arguments are key=value, got {argument}")))?;
            if !allowed.contains(&key) {
                return Err(invalid(format!("unknown argument {key}=")));
            }
            pairs.insert(key.to_owned(), value.to_owned());
        }
        Ok(Self(pairs))
    }

    fn required(&self, key: &str) -> Result<String> {
        self.0
            .get(key)
            .cloned()
            .ok_or_else(|| invalid(format!("missing required argument {key}=")))
    }

    fn number<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T> {
        match self.0.get(key) {
            None => Ok(default),
            Some(value) => value
                .parse()
                .map_err(|_| invalid(format!("invalid {key}={value}"))),
        }
    }

    fn list<T: std::str::FromStr>(&self, key: &str, default: &str) -> Result<Vec<T>> {
        self.0
            .get(key)
            .map(String::as_str)
            .unwrap_or(default)
            .split(',')
            .map(|item| {
                item.trim()
                    .parse()
                    .map_err(|_| invalid(format!("invalid {key} item {item}")))
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// The fixed 2I atom roots.

struct AtomRoots {
    entity: Vec<[f32; 4]>,
    relation: Vec<[f32; 4]>,
    entity_indices: Vec<usize>,
    relation_indices: Vec<usize>,
    entity_bound: f32,
    relation_bound: f32,
}

fn dot(left: &[f32; 4], right: &[f32; 4]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2] + left[3] * right[3]
}

fn pair_bound(set: &[usize], roots: &[[f32; 4]]) -> f32 {
    pair_score(set, roots).0
}

/// The largest pairwise `|q . r|` of `set` and how many pairs attain it.
fn pair_score(set: &[usize], roots: &[[f32; 4]]) -> (f32, usize) {
    let mut maximum = 0f32;
    let mut count = 0usize;
    for left in 0..set.len() {
        for right in left + 1..set.len() {
            let inner = dot(&roots[set[left]], &roots[set[right]]).abs();
            if inner > maximum + 1e-9 {
                maximum = inner;
                count = 1;
            } else if (inner - maximum).abs() <= 1e-9 {
                count += 1;
            }
        }
    }
    (maximum, count)
}

/// Deterministically selects `size` roots from `roots`, avoiding `forbidden`,
/// that minimise the largest pairwise `|q . r|`: every seed is tried and each
/// greedy step adds the candidate whose worst inner product to the chosen set
/// is smallest (ties to the lowest index).
fn select_dispersed(roots: &[[f32; 4]], size: usize, forbidden: &[usize]) -> Vec<usize> {
    let mut best = Vec::new();
    let mut best_score = (f32::INFINITY, usize::MAX);
    for seed in 0..roots.len() {
        if forbidden.contains(&seed) {
            continue;
        }
        let mut chosen = vec![seed];
        while chosen.len() < size {
            let mut pick: Option<(usize, f32)> = None;
            for candidate in 0..roots.len() {
                if forbidden.contains(&candidate) || chosen.contains(&candidate) {
                    continue;
                }
                let worst = chosen
                    .iter()
                    .map(|&other| dot(&roots[other], &roots[candidate]).abs())
                    .fold(0.0f32, f32::max);
                if pick.is_none_or(|(index, value)| {
                    worst < value - 1e-9 || ((worst - value).abs() <= 1e-9 && candidate < index)
                }) {
                    pick = Some((candidate, worst));
                }
            }
            match pick {
                Some((candidate, _)) => chosen.push(candidate),
                None => break,
            }
        }
        if chosen.len() < size {
            continue;
        }
        let score = pair_score(&chosen, roots);
        if score.0 < best_score.0 - 1e-9
            || ((score.0 - best_score.0).abs() <= 1e-9 && score.1 < best_score.1)
        {
            best_score = score;
            best = chosen;
        }
    }
    best
}

/// The fixed atom-root assignment. The pre-registered "pairwise `|q . r| <= 1/2`
/// for the 14 relation roots" is infeasible: the largest set of unit icosians
/// with pairwise `|q . r| <= 1/2` has size 12 (asserted in the tests), so no 14
/// roots meet it. Each head instead takes the maximum-separation set the
/// icosians allow (8 entity roots at `<= 1/2`, and 14 relation roots disjoint
/// from them at the smallest achievable bound). The selection is fixed before
/// any fit.
fn select_roots() -> Result<AtomRoots> {
    let roots = TransportSnap::Icosian.roots();
    let entity_indices = select_dispersed(roots, ENTITY_ATOMS, &[]);
    if entity_indices.len() < ENTITY_ATOMS {
        return Err(invalid("the icosians cannot supply the entity roots"));
    }
    let relation_indices = select_dispersed(roots, RELATION_ATOMS, &entity_indices);
    if relation_indices.len() < RELATION_ATOMS {
        return Err(invalid("the icosians cannot supply the relation roots"));
    }
    Ok(AtomRoots {
        entity: entity_indices.iter().map(|&index| roots[index]).collect(),
        relation: relation_indices.iter().map(|&index| roots[index]).collect(),
        entity_bound: pair_bound(&entity_indices, roots),
        relation_bound: pair_bound(&relation_indices, roots),
        entity_indices,
        relation_indices,
    })
}

/// The largest set of unit icosians with pairwise `|q . r| <= bound`, by
/// branch-and-bound with a greedy-colouring bound. Documents the
/// pre-registration's infeasibility: it is 12 at the required `1/2`.
#[cfg(test)]
fn max_compatible_set(roots: &[[f32; 4]], bound: f32) -> usize {
    let count = roots.len();
    let mut adjacency = vec![0u128; count];
    for left in 0..count {
        for right in 0..count {
            if left != right
                && dot(&roots[left], &roots[right]).abs() <= bound + ROOT_BOUND_TOLERANCE
            {
                adjacency[left] |= 1u128 << right;
            }
        }
    }
    let mut best = 0usize;
    expand_clique((1u128 << count) - 1, 0, &mut best, &adjacency);
    best
}

#[cfg(test)]
fn color_sort(candidates: u128, adjacency: &[u128]) -> (Vec<usize>, Vec<usize>) {
    let mut order = Vec::new();
    let mut colors = Vec::new();
    let mut remaining = candidates;
    let mut color = 0usize;
    while remaining != 0 {
        color += 1;
        let mut available = remaining;
        while available != 0 {
            let vertex = available.trailing_zeros() as usize;
            available &= !(1u128 << vertex);
            remaining &= !(1u128 << vertex);
            available &= !adjacency[vertex];
            order.push(vertex);
            colors.push(color);
        }
    }
    (order, colors)
}

#[cfg(test)]
fn expand_clique(candidates: u128, size: usize, best: &mut usize, adjacency: &[u128]) {
    if candidates == 0 {
        *best = (*best).max(size);
        return;
    }
    let (order, colors) = color_sort(candidates, adjacency);
    let mut remaining = candidates;
    for index in (0..order.len()).rev() {
        if size + colors[index] <= *best {
            return;
        }
        let vertex = order[index];
        expand_clique(remaining & adjacency[vertex], size + 1, best, adjacency);
        remaining &= !(1u128 << vertex);
    }
}

// ---------------------------------------------------------------------------
// Atom primes above the vocabulary band, and their semiprime keys.

/// The first `count` primes.
fn first_primes(count: usize) -> Result<Vec<u64>> {
    if count == 0 {
        return Err(invalid("the prime sieve needs at least one prime"));
    }
    let n = count as f64;
    let bound = if count < 6 {
        15
    } else {
        (n * (n.ln() + n.ln().ln())).ceil() as usize + 16
    };
    let mut composite = vec![false; bound + 1];
    let mut primes = Vec::with_capacity(count);
    for candidate in 2..=bound {
        if composite[candidate] {
            continue;
        }
        primes.push(candidate as u64);
        if primes.len() == count {
            break;
        }
        let mut multiple = candidate * candidate;
        while multiple <= bound {
            composite[multiple] = true;
            multiple += candidate;
        }
    }
    if primes.len() < count {
        return Err(invalid("the prime sieve produced too few primes"));
    }
    Ok(primes)
}

struct AtomKeys {
    entity: Vec<u64>,
    relation: Vec<u64>,
    vocab_prime_ceiling: u64,
}

impl AtomKeys {
    /// Reserves 22 primes above the vocabulary's primes. `uor-r4-core`'s
    /// `PrimeRegistry::compile` assigns its primes sequentially from 5 in
    /// semantic-ID order and cannot reserve arbitrary atom primes above the
    /// vocabulary band, so the reservation is this smallest correct mapping:
    /// the token `t` prime is the `(t + 1)`-th prime, the ceiling is the
    /// `vocab_size`-th prime and the atom primes are the next 22. Core's
    /// [`PrimeAtom`]/[`SemiprimeExpert`] validate the primes and give the
    /// order-independent semiprime identity.
    fn new(vocab_size: usize) -> Result<Self> {
        if vocab_size == 0 {
            return Err(invalid("the vocabulary must be non-empty"));
        }
        let primes = first_primes(vocab_size + ENTITY_ATOMS + RELATION_ATOMS)?;
        let ceiling = primes[vocab_size - 1];
        let entity = primes[vocab_size..vocab_size + ENTITY_ATOMS].to_vec();
        let relation = primes[vocab_size + ENTITY_ATOMS..].to_vec();
        for &prime in entity.iter().chain(relation.iter()) {
            PrimeAtom::new(prime as u32)
                .map_err(|error| invalid(format!("atom prime {prime} rejected: {error}")))?;
        }
        Ok(Self {
            entity,
            relation,
            vocab_prime_ceiling: ceiling,
        })
    }

    fn prime_of(&self, table: &[u64], atom: u32, kind: &str) -> Result<u64> {
        let index = atom
            .checked_sub(1)
            .ok_or_else(|| invalid(format!("{kind} atom 0 has no prime")))?
            as usize;
        table
            .get(index)
            .copied()
            .ok_or_else(|| invalid(format!("no {kind} prime for atom {atom}")))
    }

    fn semiprime(&self, entity_atom: u32, relation_atom: u32) -> Result<u64> {
        let left = PrimeAtom::new(self.prime_of(&self.entity, entity_atom, "entity")? as u32)
            .map_err(|error| invalid(format!("entity prime: {error}")))?;
        let right =
            PrimeAtom::new(self.prime_of(&self.relation, relation_atom, "relation")? as u32)
                .map_err(|error| invalid(format!("relation prime: {error}")))?;
        Ok(SemiprimeExpert::new(left, right)
            .map_err(|error| invalid(format!("semiprime key: {error}")))?
            .product())
    }
}

/// An exact atom store keyed by the semiprime, with the world `AtomStore`'s
/// version semantics.
#[derive(Default)]
struct SemiprimeStore {
    records: BTreeMap<u64, (u32, Option<u32>, u32)>,
}

impl SemiprimeStore {
    fn write(&mut self, key: u64, value: u32) {
        match self.records.get_mut(&key) {
            Some(record) => {
                if record.0 != value {
                    record.1 = Some(record.0);
                    record.0 = value;
                }
                record.2 += 1;
            }
            None => {
                self.records.insert(key, (value, None, 1));
            }
        }
    }

    fn read(&self, key: u64, previous: bool) -> (u32, u32) {
        match self.records.get(&key) {
            Some(record) => match (previous, record.1) {
                (false, _) => (STATUS_HIT, record.0),
                (true, Some(value)) => (STATUS_HIT, value),
                (true, None) => (STATUS_ABSENT, 0),
            },
            None => (STATUS_ABSENT, 0),
        }
    }
}

// ---------------------------------------------------------------------------
// The trained heads.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AtomKind {
    TwoI,
    Softmax,
}

impl AtomKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::TwoI => "2i",
            Self::Softmax => "softmax",
        }
    }
}

struct LinearHead {
    weight: Var,
    bias: Var,
}

impl LinearHead {
    fn new(classes: usize, width: usize, seed: u64, device: &Device) -> Result<Self> {
        let mut rng = Rng::new(seed);
        Ok(Self {
            weight: normal_var(&mut rng, &[classes, width], 0.02, device)?,
            bias: zeros_var(&[classes], device)?,
        })
    }

    fn logits(&self, feature: &Tensor) -> Result<Tensor> {
        Ok(feature
            .matmul(&self.weight.as_tensor().t()?)?
            .broadcast_add(self.bias.as_tensor())?)
    }
}

struct AtomArm {
    weight: Var,
    bias: Var,
    roots: Option<Tensor>,
}

impl AtomArm {
    fn new(
        kind: AtomKind,
        roots: &[[f32; 4]],
        width: usize,
        seed: u64,
        device: &Device,
    ) -> Result<Self> {
        let outputs = if kind == AtomKind::TwoI {
            4
        } else {
            roots.len()
        };
        let mut rng = Rng::new(seed);
        let weight = normal_var(&mut rng, &[outputs, width], 0.02, device)?;
        let bias = zeros_var(&[outputs], device)?;
        let root_tensor = if kind == AtomKind::TwoI {
            let flat: Vec<f32> = roots.iter().flat_map(|root| root.iter().copied()).collect();
            Some(Tensor::from_vec(flat, (roots.len(), 4), device)?)
        } else {
            None
        };
        Ok(Self {
            weight,
            bias,
            roots: root_tensor,
        })
    }

    fn logits(&self, feature: &Tensor) -> Result<Tensor> {
        let projected = feature
            .matmul(&self.weight.as_tensor().t()?)?
            .broadcast_add(self.bias.as_tensor())?;
        match &self.roots {
            Some(roots) => Ok(projected.matmul(&roots.t()?)?),
            None => Ok(projected),
        }
    }

    fn parameter_count(&self) -> usize {
        self.weight.elem_count() + self.bias.elem_count()
    }
}

struct ProbeHeads {
    tag: LinearHead,
    trigger: LinearHead,
    entity: AtomArm,
    relation: AtomArm,
}

impl ProbeHeads {
    fn new(
        kind: AtomKind,
        roots: &AtomRoots,
        width: usize,
        seed: u64,
        device: &Device,
    ) -> Result<Self> {
        Ok(Self {
            tag: LinearHead::new(TAG_CLASSES, width, seed ^ 0x01, device)?,
            trigger: LinearHead::new(TRIGGER_CLASSES, width, seed ^ 0x02, device)?,
            entity: AtomArm::new(kind, &roots.entity, width, seed ^ 0x03, device)?,
            relation: AtomArm::new(kind, &roots.relation, width, seed ^ 0x04, device)?,
        })
    }

    fn optimizer_groups(&self) -> (Vec<Var>, Vec<Var>) {
        (
            vec![
                self.tag.weight.clone(),
                self.trigger.weight.clone(),
                self.entity.weight.clone(),
                self.relation.weight.clone(),
            ],
            vec![
                self.tag.bias.clone(),
                self.trigger.bias.clone(),
                self.entity.bias.clone(),
                self.relation.bias.clone(),
            ],
        )
    }

    fn parameter_count(&self) -> usize {
        self.tag.weight.elem_count()
            + self.tag.bias.elem_count()
            + self.trigger.weight.elem_count()
            + self.trigger.bias.elem_count()
            + self.entity.parameter_count()
            + self.relation.parameter_count()
    }
}

fn normal_var(rng: &mut Rng, shape: &[usize], std: f64, device: &Device) -> Result<Var> {
    let count: usize = shape.iter().product();
    let mut values = Vec::with_capacity(count);
    while values.len() < count {
        let u1 = ((rng.next_u64() >> 11) as f64 + 1.0) / (1u64 << 53) as f64;
        let u2 = (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        let radius = (-2.0 * u1.ln()).sqrt();
        values.push((radius * (std::f64::consts::TAU * u2).cos() * std) as f32);
    }
    Ok(Var::from_vec(values, shape, device)?)
}

fn zeros_var(shape: &[usize], device: &Device) -> Result<Var> {
    Ok(Var::from_tensor(&Tensor::zeros(
        shape,
        DType::F32,
        device,
    )?)?)
}

// ---------------------------------------------------------------------------
// Frozen-trunk features and the head losses.

fn split_features(
    stack: &StackModel,
    split: usize,
    ids: &[u32],
    batch: usize,
    time: usize,
) -> Result<Tensor> {
    let x = stack.embed(ids, batch, time)?;
    let hidden = stack.run_layers(x, 0..split)?;
    let width = stack.config.width;
    let flat = hidden.reshape((batch * time, width))?;
    let rms = (flat.sqr()?.mean_keepdim(D::Minus1)? + 1e-5)?.sqrt()?;
    Ok(flat.broadcast_div(&rms)?.detach())
}

fn gold_tags5(batch: &DialogueBatch) -> Vec<u32> {
    (0..batch.batch * batch.time)
        .map(|row| match batch.tags[row] {
            TAG_ENTITY if batch.entity_atom[row] == entity_atom::SPEAKER => TAG5_SPEAKER,
            TAG_ENTITY => TAG5_ENTITY,
            TAG_RELATION => TAG5_RELATION,
            TAG_VALUE => TAG5_VALUE,
            _ => TAG5_OTHER,
        })
        .collect()
}

fn gather_rows(logits: &Tensor, rows: &[u32], device: &Device) -> Result<Tensor> {
    let index = Tensor::from_vec(rows.to_vec(), rows.len(), device)?;
    Ok(logits.index_select(&index, 0)?)
}

fn head_losses(
    stack: &StackModel,
    split: usize,
    heads: &ProbeHeads,
    batch: &DialogueBatch,
    config: &ProbeConfig,
) -> Result<Tensor> {
    let device = stack.device();
    let rows = batch.batch * batch.time;
    let feature = split_features(stack, split, &batch.ids, batch.batch, batch.time)?;

    let tag_logits = heads.tag.logits(&feature)?;
    let tag_loss = logits_cross_entropy(&tag_logits, &gold_tags5(batch), Some(&batch.real))?;

    let trigger_logits = heads.trigger.logits(&feature)?;
    let trigger_weights: Vec<f32> = batch
        .triggers
        .iter()
        .zip(&batch.real)
        .map(|(&trigger, &real)| {
            if trigger == TRIGGER_NONE {
                real
            } else {
                real * config.trigger_positive
            }
        })
        .collect();
    let trigger_loss =
        logits_cross_entropy(&trigger_logits, &batch.triggers, Some(&trigger_weights))?;

    let entity_rows: Vec<u32> = (0..rows)
        .filter(|&row| batch.entity_atom[row] != 0 && batch.real[row] > 0.0)
        .map(|row| row as u32)
        .collect();
    let entity_targets: Vec<u32> = entity_rows
        .iter()
        .map(|&row| batch.entity_atom[row as usize] - 1)
        .collect();

    let mut relation_rows = Vec::new();
    let mut relation_targets = Vec::new();
    for (episode_index, episode) in batch.episodes.iter().enumerate() {
        for clause in &episode.clauses {
            relation_rows.push((episode_index * batch.time + clause.trigger_position) as u32);
            relation_targets.push(clause.relation_atom - 1);
        }
    }

    let mut loss = (&tag_loss + &trigger_loss)?;
    if !entity_rows.is_empty() {
        let logits = heads.entity.logits(&feature)?;
        let selected = gather_rows(&logits, &entity_rows, device)?;
        loss = (&loss + &logits_cross_entropy(&selected, &entity_targets, None)?)?;
    }
    if !relation_rows.is_empty() {
        let logits = heads.relation.logits(&feature)?;
        let selected = gather_rows(&logits, &relation_rows, device)?;
        loss = (&loss + &logits_cross_entropy(&selected, &relation_targets, None)?)?;
    }
    Ok(loss)
}

fn clip_gradients(grads: &mut GradStore, vars: &[Var], max_norm: f64) -> Result<f64> {
    let mut total = 0f64;
    for var in vars {
        if let Some(grad) = grads.get(var.as_tensor()) {
            total += f64::from(grad.sqr()?.sum_all()?.to_scalar::<f32>()?);
        }
    }
    let norm = total.sqrt();
    if norm.is_finite() && norm > max_norm {
        let scale = max_norm / (norm + 1e-6);
        for var in vars {
            if let Some(grad) = grads.get(var.as_tensor()) {
                let scaled = (grad * scale)?;
                grads.insert(var.as_tensor(), scaled);
            }
        }
    }
    Ok(norm)
}

fn schedule(step: usize, warmup: usize, steps: usize) -> f64 {
    if step < warmup {
        return (step + 1) as f64 / warmup.max(1) as f64;
    }
    let span = steps.saturating_sub(warmup).max(1);
    let progress = (step - warmup) as f64 / span as f64;
    0.1 + 0.9 * 0.5 * (1.0 + (std::f64::consts::PI * progress.min(1.0)).cos())
}

#[derive(Clone, Debug, Serialize)]
struct StepRecord {
    step: usize,
    loss: f64,
    seconds: f64,
}

// ---------------------------------------------------------------------------
// Fit and evaluation.

#[derive(Clone)]
struct ProbeConfig {
    steps: usize,
    batch: usize,
    context: usize,
    learning_rate: f64,
    warmup: usize,
    weight_decay: f64,
    clip: f64,
    trigger_positive: f32,
    eval_episodes: usize,
    eval_batch: usize,
    dev_episodes: usize,
    seed: u64,
    data_seed: u64,
    eval_seed: u64,
    dev_seed: u64,
}

fn train_heads(
    stack: &StackModel,
    split: usize,
    kind: AtomKind,
    roots: &AtomRoots,
    world: &RelationWorld,
    config: &ProbeConfig,
) -> Result<(ProbeHeads, Vec<StepRecord>)> {
    let device = stack.device().clone();
    let width = stack.config.width;
    let heads = ProbeHeads::new(kind, roots, width, config.seed, &device)?;
    let (decayed, plain) = heads.optimizer_groups();
    let all: Vec<Var> = decayed.iter().chain(&plain).cloned().collect();
    let params = |weight_decay| ParamsAdamW {
        lr: config.learning_rate,
        beta1: 0.9,
        beta2: 0.95,
        eps: 1e-8,
        weight_decay,
    };
    let mut decayed_optimizer = AdamW::new(decayed, params(config.weight_decay))?;
    let mut plain_optimizer = AdamW::new(plain, params(0.0))?;
    let mut rng = Rng::new(config.data_seed);
    let mut records = Vec::new();
    let started = Instant::now();
    for step in 0..config.steps {
        let lr = config.learning_rate * schedule(step, config.warmup, config.steps);
        decayed_optimizer.set_learning_rate(lr);
        plain_optimizer.set_learning_rate(lr);
        let batch = world.batch(&mut rng, config.batch, config.context, false)?;
        let loss = head_losses(stack, split, &heads, &batch, config)?;
        let mut grads = loss.backward()?;
        clip_gradients(&mut grads, &all, config.clip)?;
        decayed_optimizer.step(&grads)?;
        plain_optimizer.step(&grads)?;
        if step % 50 == 0 || step + 1 == config.steps {
            records.push(StepRecord {
                step,
                loss: f64::from(loss.to_scalar::<f32>()?),
                seconds: started.elapsed().as_secs_f64(),
            });
        }
    }
    Ok((heads, records))
}

#[derive(Clone, Debug, Default, Serialize)]
struct Counts {
    clauses: usize,
    key_correct: usize,
    writes: usize,
    write_key_correct: usize,
    reads: usize,
    read_key_correct: usize,
    register_correct: usize,
}

impl Counts {
    fn absorb(&mut self, other: &Counts) {
        self.clauses += other.clauses;
        self.key_correct += other.key_correct;
        self.writes += other.writes;
        self.write_key_correct += other.write_key_correct;
        self.reads += other.reads;
        self.read_key_correct += other.read_key_correct;
        self.register_correct += other.register_correct;
    }

    fn key_accuracy(&self) -> Option<f64> {
        (self.clauses > 0).then(|| self.key_correct as f64 / self.clauses as f64)
    }

    fn read_key_accuracy(&self) -> Option<f64> {
        (self.reads > 0).then(|| self.read_key_correct as f64 / self.reads as f64)
    }

    fn register_accuracy(&self) -> Option<f64> {
        (self.reads > 0).then(|| self.register_correct as f64 / self.reads as f64)
    }
}

#[derive(Clone, Debug, Default, Serialize)]
struct EvalReport {
    episodes: usize,
    totals: Counts,
    phenomena: BTreeMap<String, Counts>,
    classes: BTreeMap<String, Counts>,
    entity_confusion: BTreeMap<String, usize>,
    relation_confusion: BTreeMap<String, usize>,
}

fn argmax_rows(logits: &Tensor) -> Result<Vec<u32>> {
    Ok(logits.detach().argmax(D::Minus1)?.to_vec1::<u32>()?)
}

fn clause_slots(episode: &Episode, clause: &Clause, turn_start: usize) -> Result<(usize, usize)> {
    let entity = (turn_start..=clause.trigger_position)
        .find(|&position| episode.entity_atom[position] == clause.entity_atom)
        .ok_or_else(|| invalid("a natural clause lacks its entity slot"))?;
    let relation = (turn_start..=clause.trigger_position)
        .find(|&position| episode.relation_atom[position] == clause.relation_atom)
        .ok_or_else(|| invalid("a natural clause lacks its relation slot"))?;
    Ok((entity, relation))
}

fn evaluate_heads(
    stack: &StackModel,
    split: usize,
    heads: &ProbeHeads,
    keys: &AtomKeys,
    world: &RelationWorld,
    config: &ProbeConfig,
    held: bool,
    seed: u64,
    episodes: usize,
) -> Result<EvalReport> {
    let mut report = EvalReport::default();
    let mut rng = Rng::new(seed);
    while report.episodes < episodes {
        let batch = world.batch(&mut rng, config.eval_batch, config.context, held)?;
        let feature = split_features(stack, split, &batch.ids, batch.batch, batch.time)?;
        let entity_argmax = argmax_rows(&heads.entity.logits(&feature)?)?;
        let relation_argmax = argmax_rows(&heads.relation.logits(&feature)?)?;
        for (episode_index, episode) in batch.episodes.iter().enumerate() {
            let base = episode_index * batch.time;
            let mut turn_start = 0usize;
            let mut gold = AtomStore::new();
            let mut model_store = SemiprimeStore::default();
            for clause in &episode.clauses {
                let (entity_slot, _) = clause_slots(episode, clause, turn_start)?;
                turn_start = clause.trigger_position + 1;
                let predicted_entity = entity_argmax[base + entity_slot] + 1;
                let predicted_relation = relation_argmax[base + clause.trigger_position] + 1;
                let key_correct = predicted_entity == clause.entity_atom
                    && predicted_relation == clause.relation_atom;
                let semiprime = keys.semiprime(predicted_entity, predicted_relation)?;
                let phenomenon = clause.phenomenon.as_str();
                let class = format!("{:?}", clause.class);
                let mut counts = Counts {
                    clauses: 1,
                    key_correct: usize::from(key_correct),
                    ..Counts::default()
                };
                if episode.triggers[clause.trigger_position] == TRIGGER_WRITE {
                    counts.writes = 1;
                    counts.write_key_correct = usize::from(key_correct);
                    let value = clause
                        .value
                        .ok_or_else(|| invalid("a natural write clause lacks its value"))?;
                    gold.write(clause.entity_atom, clause.relation_atom, value);
                    model_store.write(semiprime, value);
                } else {
                    counts.reads = 1;
                    counts.read_key_correct = usize::from(key_correct);
                    let gold_read =
                        gold.read(clause.entity_atom, clause.relation_atom, clause.previous);
                    let model_read = model_store.read(semiprime, clause.previous);
                    counts.register_correct = usize::from(gold_read == model_read);
                }
                if !key_correct {
                    if predicted_entity != clause.entity_atom {
                        *report
                            .entity_confusion
                            .entry(format!(
                                "{phenomenon}:{}->{predicted_entity}",
                                clause.entity_atom
                            ))
                            .or_default() += 1;
                    }
                    if predicted_relation != clause.relation_atom {
                        *report
                            .relation_confusion
                            .entry(format!(
                                "{phenomenon}:{}->{predicted_relation}",
                                clause.relation_atom
                            ))
                            .or_default() += 1;
                    }
                }
                report.totals.absorb(&counts);
                report
                    .phenomena
                    .entry(phenomenon.to_owned())
                    .or_default()
                    .absorb(&counts);
                report.classes.entry(class).or_default().absorb(&counts);
            }
            report.episodes += 1;
        }
        if report.episodes >= episodes {
            break;
        }
    }
    Ok(report)
}

// ---------------------------------------------------------------------------
// Reporting helpers.

fn counts_json(counts: &Counts) -> Value {
    json!({
        "clauses": counts.clauses,
        "key_correct": counts.key_correct,
        "key_accuracy": counts.key_accuracy(),
        "writes": counts.writes,
        "write_key_correct": counts.write_key_correct,
        "write_key_accuracy": (counts.writes > 0).then(|| counts.write_key_correct as f64 / counts.writes as f64),
        "reads": counts.reads,
        "read_key_correct": counts.read_key_correct,
        "read_key_accuracy": counts.read_key_accuracy(),
        "register_correct": counts.register_correct,
        "register_accuracy": counts.register_accuracy(),
    })
}

fn report_json(report: &EvalReport) -> Value {
    json!({
        "episodes": report.episodes,
        "totals": counts_json(&report.totals),
        "phenomena": report
            .phenomena
            .iter()
            .map(|(name, counts)| (name.clone(), counts_json(counts)))
            .collect::<BTreeMap<_, _>>(),
        "classes": report
            .classes
            .iter()
            .map(|(name, counts)| (name.clone(), counts_json(counts)))
            .collect::<BTreeMap<_, _>>(),
        "entity_confusion": report.entity_confusion,
        "relation_confusion": report.relation_confusion,
    })
}

fn phenomenon_accuracy(report: &EvalReport, phenomenon: Phenomenon) -> Option<f64> {
    report
        .phenomena
        .get(phenomenon.as_str())
        .and_then(Counts::key_accuracy)
}

/// The lexical phenomena whose held-out accuracy routes the < 0.5 rule.
const LEXICAL_PHENOMENA: [Phenomenon; 2] = [Phenomenon::SynonymsMorphology, Phenomenon::WhCarried];

fn decision_rules(arms: &[ArmResult]) -> Value {
    let two_i: Vec<&ArmResult> = arms
        .iter()
        .filter(|arm| arm.kind == AtomKind::TwoI)
        .collect();
    let softmax: Vec<&ArmResult> = arms
        .iter()
        .filter(|arm| arm.kind == AtomKind::Softmax)
        .collect();
    let best_two_i = two_i
        .iter()
        .max_by(|left, right| {
            left.held
                .totals
                .key_accuracy()
                .unwrap_or(0.0)
                .total_cmp(&right.held.totals.key_accuracy().unwrap_or(0.0))
        })
        .copied();
    let best_softmax = softmax
        .iter()
        .max_by(|left, right| {
            left.held
                .totals
                .key_accuracy()
                .unwrap_or(0.0)
                .total_cmp(&right.held.totals.key_accuracy().unwrap_or(0.0))
        })
        .copied();
    let (Some(best_two_i), Some(best_softmax)) = (best_two_i, best_softmax) else {
        return json!({"available": false, "reason": "both arms required"});
    };
    let per_phenomenon: BTreeMap<String, f64> = Phenomenon::ALL
        .iter()
        .filter_map(|&phenomenon| {
            phenomenon_accuracy(&best_two_i.held, phenomenon)
                .map(|accuracy| (phenomenon.as_str().to_owned(), accuracy))
        })
        .collect();
    let all_phenomena = Phenomenon::ALL.iter().all(|&phenomenon| {
        per_phenomenon
            .get(phenomenon.as_str())
            .is_some_and(|&a| a >= 0.90)
    });
    let in_range: BTreeMap<String, f64> = per_phenomenon
        .iter()
        .filter(|(_, &accuracy)| (0.5..0.9).contains(&accuracy))
        .map(|(name, &accuracy)| (name.clone(), accuracy))
        .collect();
    let lexical_below_half_both_splits = LEXICAL_PHENOMENA.iter().all(|&phenomenon| {
        two_i.iter().all(|arm| {
            phenomenon_accuracy(&arm.held, phenomenon).is_some_and(|accuracy| accuracy < 0.5)
        })
    });
    let two_i_accuracy = best_two_i.held.totals.key_accuracy().unwrap_or(0.0);
    let softmax_accuracy = best_softmax.held.totals.key_accuracy().unwrap_or(0.0);
    let gap = softmax_accuracy - two_i_accuracy;
    json!({
        "best_two_i_split": best_two_i.split,
        "best_softmax_split": best_softmax.split,
        "two_i_held_key_accuracy": two_i_accuracy,
        "softmax_held_key_accuracy": softmax_accuracy,
        "two_i_minus_softmax": two_i_accuracy - softmax_accuracy,
        "softmax_minus_two_i_gap": gap,
        "per_phenomenon_two_i_accuracy": per_phenomenon,
        "all_phenomena_at_least_0_90": all_phenomena,
        "heads_to_integration": all_phenomena,
        "geometric_atoms_used": gap <= 0.02,
        "phenomena_between_0_5_and_0_9": in_range,
        "lexical_below_0_5_at_both_splits": lexical_below_half_both_splits,
        "report_to_lab_1": lexical_below_half_both_splits,
        "lexical_phenomena": LEXICAL_PHENOMENA
            .iter()
            .map(|phenomenon| phenomenon.as_str())
            .collect::<Vec<_>>(),
    })
}

struct ArmResult {
    split: usize,
    kind: AtomKind,
    learnable_parameters: usize,
    held: EvalReport,
    dev: EvalReport,
    records: Vec<StepRecord>,
}

// ---------------------------------------------------------------------------
// Entry points.

fn run(args: &Args, out: &Path) -> Result<()> {
    let trunk_directory = PathBuf::from(args.required("trunk")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let splits: Vec<usize> = args.list("splits", "2,4")?;
    if splits.is_empty() || splits.iter().any(|&split| split == 0) {
        return Err(invalid("splits must be non-empty positive layer counts"));
    }
    let config = ProbeConfig {
        steps: args.number("steps", 1500)?,
        batch: args.number("batch", 16)?,
        context: args.number("context", 256)?,
        learning_rate: args.number("lr", 0.003)?,
        warmup: args.number("warmup", 100)?,
        weight_decay: args.number("weight_decay", 0.1)?,
        clip: args.number("clip", 1.0)?,
        trigger_positive: args.number("trigger_positive", 5.0)?,
        eval_episodes: args.number("eval_episodes", 512)?,
        eval_batch: args.number("eval_batch", 16)?,
        dev_episodes: args.number("dev_episodes", 256)?,
        seed: args.number("seed", 9001)?,
        data_seed: args.number("data_seed", 9002)?,
        eval_seed: args.number("eval_seed", 9003)?,
        dev_seed: args.number("dev_seed", 9004)?,
    };
    if config.steps == 0 || config.batch == 0 || config.eval_batch == 0 || config.context < 2 {
        return Err(invalid(
            "steps, batch, eval_batch and context must be usable",
        ));
    }

    let trunk_weights = trunk_directory.join("model.safetensors");
    let trunk_sha256 = sha256_file(&trunk_weights)?;
    if trunk_sha256 != TRUNK_SHA256 {
        return Err(invalid(format!(
            "trunk {} is not the pinned S4 arm-A checkpoint: model.safetensors SHA-256 {trunk_sha256}",
            trunk_directory.display()
        )));
    }
    let device = Device::Cpu;
    let mut stack = StackModel::load(&trunk_directory, &device)?;
    let mut snap_restored = false;
    if let Some(snap) = StackModel::saved_transport_snap(&trunk_directory)? {
        stack.set_transport_snap(Some(snap))?;
        snap_restored = true;
    }
    if splits.iter().any(|&split| split >= stack.config.layers()) {
        return Err(invalid("a split must leave layers on both sides"));
    }
    let model = AermModel::from_stack(stack, splits[0], false, config.seed)?;
    let stack = &model.stack;

    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(&tokenizer_path)?)
        .ok_or_else(|| invalid("tokenizer JSON is not a supported byte-level BPE"))?;
    let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)
        .map_err(|error| invalid(format!("protocol: {error}")))?;
    let encode = |text: &str| tokenizer.encode(text);
    let world = RelationWorld::natural(&encode, protocol.bos_id, protocol.eos_id)?;
    let roots = select_roots()?;
    let keys = AtomKeys::new(stack.config.vocab_size)?;

    let mut arms = Vec::new();
    for &split in &splits {
        for kind in [AtomKind::TwoI, AtomKind::Softmax] {
            let (heads, records) = train_heads(stack, split, kind, &roots, &world, &config)?;
            let held = evaluate_heads(
                stack,
                split,
                &heads,
                &keys,
                &world,
                &config,
                true,
                config.eval_seed,
                config.eval_episodes,
            )?;
            let dev = evaluate_heads(
                stack,
                split,
                &heads,
                &keys,
                &world,
                &config,
                false,
                config.dev_seed,
                config.dev_episodes,
            )?;
            arms.push(ArmResult {
                split,
                kind,
                learnable_parameters: heads.parameter_count(),
                held,
                dev,
                records,
            });
        }
    }

    let decisions = decision_rules(&arms);
    let guard_passes = arms
        .iter()
        .filter(|arm| arm.kind == AtomKind::TwoI)
        .all(|arm| arm.dev.totals.key_accuracy().unwrap_or(0.0) >= 0.98);
    let report = json!({
        "schema": "uor-r4.g2-key-probe/1",
        "trunk": {
            "directory": trunk_directory.display().to_string(),
            "model_safetensors_sha256": trunk_sha256,
            "pinned_sha256": TRUNK_SHA256,
            "splits": splits,
            "loaded_through": "AermModel::from_stack",
            "transport_snap_restored": snap_restored,
            "layers": stack.config.layers(),
            "width": stack.config.width,
        },
        "roots": {
            "snap": TransportSnap::Icosian.name(),
            "root_count": TransportSnap::Icosian.roots().len(),
            "entity_indices": roots.entity_indices,
            "relation_indices": roots.relation_indices,
            "entity_bound": roots.entity_bound,
            "relation_bound": roots.relation_bound,
            "registered_bound": ROOT_BOUND,
            "deviation": "the pre-registered 14 relation roots at pairwise |q.r| <= 1/2 are infeasible: the largest such icosian set has 12 (see the test); the relation head uses the maximum-separation 14-root set disjoint from the entity roots instead",
        },
        "keys": {
            "vocabulary_size": stack.config.vocab_size,
            "vocab_prime_ceiling": keys.vocab_prime_ceiling,
            "entity_primes": keys.entity,
            "relation_primes": keys.relation,
            "registry": "uor-r4-token primes are the first vocab_size primes; atom primes are the next 22 above the ceiling, validated by uor_r4_core::prime_route_attention::PrimeAtom and combined by SemiprimeExpert",
        },
        "config": {
            "steps": config.steps,
            "batch": config.batch,
            "context": config.context,
            "learning_rate": config.learning_rate,
            "warmup": config.warmup,
            "weight_decay": config.weight_decay,
            "clip": config.clip,
            "trigger_positive": config.trigger_positive,
            "eval_episodes": config.eval_episodes,
            "eval_batch": config.eval_batch,
            "dev_episodes": config.dev_episodes,
            "seed": config.seed,
            "data_seed": config.data_seed,
            "eval_seed": config.eval_seed,
            "dev_seed": config.dev_seed,
        },
        "arms": arms.iter().map(|arm| json!({
            "split": arm.split,
            "atom_kind": arm.kind.as_str(),
            "learnable_parameters": arm.learnable_parameters,
            "held_out": report_json(&arm.held),
            "in_distribution": report_json(&arm.dev),
            "in_distribution_key_accuracy": arm.dev.totals.key_accuracy(),
            "history": arm.records,
        })).collect::<Vec<_>>(),
        "guard": {
            "in_distribution_key_accuracy_at_least": 0.98,
            "passes": guard_passes,
        },
        "decisions": decisions,
        "parameter_comparison": {
            "2i_learnable_parameters": arms.iter().find(|arm| arm.kind == AtomKind::TwoI).map(|arm| arm.learnable_parameters),
            "softmax_learnable_parameters": arms.iter().find(|arm| arm.kind == AtomKind::Softmax).map(|arm| arm.learnable_parameters),
        },
    });
    fs::write(out.join("result.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

fn roots_mode(arguments: &[String]) -> Result<()> {
    let roots = select_roots()?;
    let value = json!({
        "snap": TransportSnap::Icosian.name(),
        "entity_indices": roots.entity_indices,
        "relation_indices": roots.relation_indices,
        "entity_roots": roots.entity,
        "relation_roots": roots.relation,
        "entity_bound": roots.entity_bound,
        "relation_bound": roots.relation_bound,
        "registered_bound": ROOT_BOUND,
    });
    println!("{}", serde_json::to_string_pretty(&value)?);
    let _ = arguments;
    Ok(())
}

fn finish(out: &Path, result: Result<()>) -> Result<()> {
    if let Err(error) = &result {
        fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error": error.to_string()}))?,
        )?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    result
}

fn execute(arguments: &[String]) -> Result<()> {
    let (mode, rest) = arguments
        .split_first()
        .ok_or_else(|| invalid("usage: aerm-keys {run|roots} ..."))?;
    match mode.as_str() {
        "run" => {
            let args = Args::parse(
                rest,
                &[
                    "out",
                    "trunk",
                    "tokenizer",
                    "splits",
                    "steps",
                    "batch",
                    "context",
                    "lr",
                    "warmup",
                    "weight_decay",
                    "clip",
                    "trigger_positive",
                    "eval_episodes",
                    "eval_batch",
                    "dev_episodes",
                    "seed",
                    "data_seed",
                    "eval_seed",
                    "dev_seed",
                ],
            )?;
            let out = PathBuf::from(args.required("out")?);
            report_output::claim(&out)?;
            let result = run(&args, &out);
            finish(&out, result)
        }
        "roots" => roots_mode(rest),
        other => Err(invalid(format!("unknown mode {other}"))),
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = execute(&arguments) {
        eprintln!("aerm-keys: {error}");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Focused tests (no heavy fit).

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::{BTreeSet, HashMap};

    use uor_r4_training::geometric_stack::{ReadScore, StackArch, StackConfig};

    use super::*;

    fn toy_encoder() -> impl Fn(&str) -> Vec<u32> {
        let vocab: RefCell<HashMap<String, u32>> = RefCell::new(HashMap::new());
        move |text: &str| {
            let mut pieces: Vec<String> = Vec::new();
            let mut current = String::new();
            for ch in text.chars() {
                let boundary = ch == ' ' || ch == '\'' || ch == '\n' || ch.is_ascii_punctuation();
                if boundary && !current.is_empty() && !(current == "'" && ch.is_alphabetic()) {
                    pieces.push(std::mem::take(&mut current));
                }
                current.push(ch);
                if ch.is_ascii_punctuation() && ch != '\'' {
                    pieces.push(std::mem::take(&mut current));
                }
            }
            if !current.is_empty() {
                pieces.push(current);
            }
            let mut vocab = vocab.borrow_mut();
            pieces
                .into_iter()
                .map(|piece| {
                    let next = vocab.len() as u32 + 3;
                    *vocab.entry(piece).or_insert(next)
                })
                .collect()
        }
    }

    fn natural_world() -> Result<RelationWorld> {
        RelationWorld::natural(&toy_encoder(), 0, 1)
    }

    fn tiny_stack() -> Result<StackModel> {
        let config = StackConfig {
            arch: StackArch::Geometric,
            vocab_size: 4096,
            width: 32,
            heads: 4,
            mlp_hidden: 64,
            context: 256,
            pattern: "rar".into(),
            read: ReadScore::Dot,
            rotation: true,
            seed: 7,
            memory: None,
        };
        config.validate()?;
        StackModel::new(config, &Device::Cpu)
    }

    fn tiny_config() -> ProbeConfig {
        ProbeConfig {
            steps: 2,
            batch: 2,
            context: 256,
            learning_rate: 0.01,
            warmup: 1,
            weight_decay: 0.0,
            clip: 1.0,
            trigger_positive: 5.0,
            eval_episodes: 4,
            eval_batch: 2,
            dev_episodes: 2,
            seed: 11,
            data_seed: 13,
            eval_seed: 17,
            dev_seed: 19,
        }
    }

    #[test]
    fn two_i_roots_are_pairwise_bounded_and_complete() -> Result<()> {
        let icosians = TransportSnap::Icosian.roots();
        let roots = select_roots()?;
        assert_eq!(roots.entity.len(), ENTITY_ATOMS);
        assert_eq!(roots.relation.len(), RELATION_ATOMS);
        for (index, root) in roots.entity.iter().chain(&roots.relation).enumerate() {
            let norm = dot(root, root).sqrt();
            assert!(
                (norm - 1.0).abs() <= 1e-5,
                "root {index} is not unit: {norm}"
            );
        }
        assert!(
            roots.entity_bound <= ROOT_BOUND + ROOT_BOUND_TOLERANCE,
            "entity roots exceed the 1/2 bound: {}",
            roots.entity_bound
        );
        assert!(
            roots.relation_bound <= 0.809_017 + ROOT_BOUND_TOLERANCE,
            "relation roots exceed the smallest achievable bound: {}",
            roots.relation_bound
        );
        assert!(
            roots
                .entity_indices
                .iter()
                .all(|index| !roots.relation_indices.contains(index)),
            "the entity and relation root sets must be disjoint"
        );
        assert_eq!(
            pair_bound(&roots.entity_indices, icosians),
            roots.entity_bound
        );
        assert_eq!(
            pair_bound(&roots.relation_indices, icosians),
            roots.relation_bound
        );
        Ok(())
    }

    #[test]
    fn fourteen_relation_roots_cannot_meet_the_half_bound() {
        let icosians = TransportSnap::Icosian.roots();
        let maximum = max_compatible_set(icosians, ROOT_BOUND);
        assert_eq!(
            maximum, 12,
            "the pre-registration assumed 14 roots exist at |q.r| <= 1/2"
        );
        assert!(maximum < RELATION_ATOMS);
    }

    #[test]
    fn semiprime_keys_are_order_independent_and_collision_free() -> Result<()> {
        let keys = AtomKeys::new(4096)?;
        let mut products = BTreeSet::new();
        for entity in 1..=ENTITY_ATOMS as u32 {
            for relation in 1..=RELATION_ATOMS as u32 {
                let product = keys.semiprime(entity, relation)?;
                assert!(
                    products.insert(product),
                    "semiprime collision at ({entity}, {relation}) = {product}"
                );
            }
        }
        assert_eq!(products.len(), ENTITY_ATOMS * RELATION_ATOMS);

        let left =
            PrimeAtom::new(keys.entity[0] as u32).map_err(|error| invalid(format!("{error}")))?;
        let right =
            PrimeAtom::new(keys.relation[0] as u32).map_err(|error| invalid(format!("{error}")))?;
        assert_eq!(
            SemiprimeExpert::new(left, right)
                .map_err(|error| invalid(format!("{error}")))?
                .product(),
            SemiprimeExpert::new(right, left)
                .map_err(|error| invalid(format!("{error}")))?
                .product()
        );

        let vocabulary: BTreeSet<u64> = first_primes(4096)?.into_iter().collect();
        assert!(
            products.iter().all(|product| !vocabulary.contains(product)),
            "an atom semiprime collided with a vocabulary prime"
        );
        assert!(keys
            .entity
            .iter()
            .chain(&keys.relation)
            .all(|prime| !vocabulary.contains(prime)));
        Ok(())
    }

    #[test]
    fn head_step_leaves_the_frozen_trunk_without_gradient() -> Result<()> {
        let stack = tiny_stack()?;
        let roots = select_roots()?;
        let heads = ProbeHeads::new(
            AtomKind::TwoI,
            &roots,
            stack.config.width,
            23,
            stack.device(),
        )?;
        let config = tiny_config();
        let world = natural_world()?;
        let mut rng = Rng::new(config.data_seed);
        let batch = world.batch(&mut rng, config.batch, config.context, false)?;
        let loss = head_losses(&stack, 1, &heads, &batch, &config)?;
        let grads = loss.backward()?;
        for (name, var) in stack.variables() {
            assert!(
                grads.get(var.as_tensor()).is_none(),
                "trunk variable {name} received a gradient from a head step"
            );
        }
        let (decayed, plain) = heads.optimizer_groups();
        assert!(decayed
            .iter()
            .any(|var| grads.get(var.as_tensor()).is_some()));

        let before = stack
            .variables()
            .values()
            .next()
            .ok_or_else(|| invalid("the tiny stack has no variables"))?
            .as_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        let all: Vec<Var> = decayed.iter().chain(&plain).cloned().collect();
        let params = |weight_decay| ParamsAdamW {
            lr: 0.01,
            beta1: 0.9,
            beta2: 0.95,
            eps: 1e-8,
            weight_decay,
        };
        let mut step_grads = grads;
        clip_gradients(&mut step_grads, &all, 1.0)?;
        AdamW::new(decayed, params(0.0))?.step(&step_grads)?;
        AdamW::new(plain, params(0.0))?.step(&step_grads)?;
        let after = stack
            .variables()
            .values()
            .next()
            .ok_or_else(|| invalid("the tiny stack has no variables"))?
            .as_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert_eq!(before, after, "an optimizer step changed the frozen trunk");
        Ok(())
    }

    #[test]
    fn two_update_smoke_runs_end_to_end() -> Result<()> {
        let stack = tiny_stack()?;
        let roots = select_roots()?;
        let keys = AtomKeys::new(stack.config.vocab_size)?;
        let world = natural_world()?;
        let config = tiny_config();
        for kind in [AtomKind::TwoI, AtomKind::Softmax] {
            let (heads, records) = train_heads(&stack, 1, kind, &roots, &world, &config)?;
            assert!(!records.is_empty());
            let held = evaluate_heads(
                &stack,
                1,
                &heads,
                &keys,
                &world,
                &config,
                true,
                config.eval_seed,
                config.eval_episodes,
            )?;
            assert!(held.episodes >= config.eval_episodes.min(1));
            assert!(held.totals.clauses > 0, "no held clauses were scored");
            assert!(held.totals.reads > 0, "no held reads were scored");
            assert!(heads.parameter_count() > 0);
        }
        Ok(())
    }
}
