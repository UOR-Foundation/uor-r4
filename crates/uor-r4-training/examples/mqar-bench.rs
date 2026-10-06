//! MQAR read diagnostic (References #820): train a small geometric stack
//! (`uor_r4_training::geometric_stack`) on synthetic multi-query associative
//! recall and measure in-context recall accuracy by query distance.
//!
//! ```text
//! mqar-bench out=NEW_REPORT_ROOT [device=cpu|metal|cuda] [context=512] [pairs_per_bucket=8] \
//!   [batch=8] [steps=1800] [lr=0.001] [warmup=100] [min_lr=0.1] [weight_decay=0.1] [clip=1.0] \
//!   [eval_every=100] [curve_sequences=8] [final_sequences=64] [seed=1] [max_seconds=1200] \
//!   [probe_steps=0,600,final|none] [probe_sequences=16] [save_model=false|true] \
//!   [dump_scores=false|true] \
//!   [mode=train|task-baselines] [layout=synthetic|fact] FACT OPTIONS [arm=stack] ARM OPTIONS
//! arm=stack: [pattern=aaaaaa] [read=l2|dot|lorentz] [rotation=true|false] [width=128] [heads=4] \
//!   [mlp=384] [age=default|flat|spread] [key_shift=false|true] \
//!   [lineage=none|f2|qk|qk_jj|identity|so4|conv|wprev] [learned_init=lag1|zero]
//! arm=transformer: [layers=<pattern length>] [width=128] [heads=4] [mlp=matched|<n>] \
//!   [match_pattern=rrarra] [match_read=l2|dot|lorentz] [match_rotation=true|false]
//! layout=fact: tokenizer=TOKENIZER_JSON [gaps=0,1,2,3] [forms=rehearse,bare]
//!   (context <= 384; defaults pairs_per_bucket=4, final_sequences=256)
//! mqar-bench mode=decide runs=DIR_OF_SEALED_FACT_ROOTS out=NEW_REPORT_ROOT
//! ```
//!
//! `layout=fact` (Step 2, the deployment-parity bench) and `mode=decide` (its
//! frozen decision table) are documented in `mqar_bench_step2/fact.rs` and
//! `mqar_bench_step2/decide.rs`. `lineage` selects the key/query lineage arm
//! (`StackModel::set_read_key_shift` for `f2`, `StackModel::set_read_lineage`
//! for the research-only controls); `key_shift=true` is kept as `lineage=f2`.
//!
//! `dump_scores=1` is a TEMPORARY T2 diagnostic: after the final evaluation it
//! writes the read's own softmax weight rows on the held-out set (every read
//! layer and head, at each item's first-content predicting position) into
//! `dump_scores/` under the report root, plus the gold source index, the
//! item's landmarks and the model's argmax there. It trains nothing, changes
//! no parameter and moves no tally; the default (`dump_scores=0`) path writes
//! no such file.
//!
//! Each sequence is a fixed-length window of filler tokens holding
//! `pairs_per_bucket` key-value pairs per distance bucket. A pair writes its
//! key at position `p` and its value at `p + 1`; its single query writes the key
//! again at `q = p + d`, and the training target at `q` (the only weighted
//! position) is the value. The distance `d = q - p` is drawn uniformly inside
//! the pair's bucket, so no fixed offset reaches the key. Keys, values and
//! filler come from disjoint token ranges; keys and values are distinct inside
//! a sequence. Every sequence draws fresh pairings from its own seed, so a key's
//! value cannot be memorized: a pairing recurs across sequences only at the rate
//! of independent uniform draws.
//!
//! Two final evaluations use fresh seeds. The primary one (`in_class`) draws
//! pairings from the training pairing classes. The pairing space is split by
//! `(key_index + value_index) % 4`; training never draws class 0, and the
//! second evaluation (`held_out_class`) draws only class 0. It is adversarial:
//! a model that learns the class rule as a prior is penalized there even when
//! it reads the context, so it measures prior-over-context, not recall alone.
//!
//! The arm (the context-access mechanism under test) is defined separately
//! from the task, training loop and scoring: see `ContextArm` and `ArmSpec`.
//! `arm=stack` is the geometric stack; `arm=transformer` is the ordinary
//! causal-softmax attention control whose total parameter count is matched to
//! a named geometric arm (D20 section 2 condition 1). Each report records the
//! arm in a fixed schema (`REPORT_SCHEMA`), including its context-access cost
//! in positions scored per query token and the parameter-match residual.
//!
//! Two reachability counters run once on the final in-class panel (D20
//! section 2 condition 3): `read_firing` evaluates the geometric read's own
//! binding mass at every scored query position (the `Work::selected_operations`
//! analogue), and `context_reachability` replaces each query's source key with
//! a filler token and counts the scored rows whose logits move, for every arm.
//! Both are observation only.
//!
//! The report root is claimed exclusively before any model work and sealed
//! with its manifest at the end. Offline floating-point training only; nothing
//! here is a serving path.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{Device, Tensor};
use serde_json::{json, Value};
use uor_r4_core::report_output;
#[cfg(test)]
use uor_r4_training::geometric_stack::quaternion_j_left;
use uor_r4_training::geometric_stack::{
    parse_pointer_route, PointerConfig, PrimeRoute, ReadBinding, ReadBindingTarget, ReadLineage,
    ReadScore, RotationGroup, StackAdamW, StackArch, StackConfig, StackModel,
};
use uor_r4_training::{Result, TrainingError};

#[path = "mqar_bench_step2/decide.rs"]
mod decide;
#[path = "mqar_bench_step2/fact.rs"]
mod fact;

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
    /// Wrong answers that are the value written most recently before the
    /// query (by another pair): a recency read instead of a key match.
    most_recent_value: usize,
}

// ---------------------------------------------------------------------------
// Context-access arms. The task generator above, and the training loop and
// per-distance scoring below, see an arm only through `ContextArm`. To add an
// arm (for example a routed or admitted read), add an `ArmSpec` variant, its
// parse branch and its constructor in `ArmSpec::build`; nothing else changes.
// ---------------------------------------------------------------------------

/// What every arm gives the bench.
trait ContextArm {
    /// Stable arm kind, e.g. `geometric_stack`.
    fn kind(&self) -> &'static str;
    /// The arm's own configuration, recorded verbatim in the report.
    fn record(&self) -> Value;
    fn parameters(&self) -> usize;
    fn device(&self) -> &Device;
    /// Weighted next-token loss over `batch` windows of `time` tokens.
    fn loss(
        &self,
        ids: &[u32],
        targets: &[u32],
        weights: &[f32],
        batch: usize,
        time: usize,
    ) -> Result<Tensor>;
    /// Logits [batch * time, vocabulary].
    fn logits(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor>;
    /// One optimizer update from the loss's gradients; returns the gradient norm.
    fn update(&mut self, loss: &Tensor, lr: f64) -> Result<f64>;
    /// Context positions the arm scores for the token at `position`, summed
    /// over its layers and heads (a recurrence scores none; a full causal read
    /// head scores `position + 1`).
    fn positions_scored(&self, position: usize) -> usize;
    /// What `positions_scored` counts, for the report.
    fn access_note(&self) -> String;
    /// Saves the arm's weights under `directory` (`save_model=true`); an arm
    /// without a saved form refuses.
    fn save(&self, directory: &Path) -> Result<()>;
    /// The (layer, head) read heads whose source weights the probe observes;
    /// empty for an arm without position-scoring reads.
    fn read_heads(&self) -> Vec<(usize, usize)> {
        Vec::new()
    }
    /// The read weight of head (`layer`, `head`) on each row's sources, for
    /// rows of (batch item, query position, source positions), at most one
    /// row per batch item. Observation only: it changes no logits.
    fn source_mass(
        &self,
        _ids: &[u32],
        _batch: usize,
        _time: usize,
        _layer: usize,
        _head: usize,
        _rows: &[(usize, usize, Vec<usize>)],
    ) -> Result<Vec<f32>> {
        Err(invalid("this arm has no read heads to probe"))
    }
    /// TEMPORARY (T2 `dump_scores=1`): the read's full softmax weight rows at
    /// declared `(batch, query)` positions, per read layer, and the logits of
    /// the same forward (see `StackModel::read_weight_rows`). Observation only.
    fn read_weight_rows(
        &self,
        _ids: &[u32],
        _batch: usize,
        _time: usize,
        _rows: &[(usize, usize)],
    ) -> Result<(Vec<(usize, Tensor)>, Tensor)> {
        Err(invalid("this arm has no read weight rows"))
    }
}

/// Initial values of the reads' learned per-distance age bias.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AgeInit {
    /// The stack's own: head `h` of `H` starts at slope `2^(-8 (h + 1) / H)`.
    Default,
    /// Zero at every distance.
    Flat,
    /// F1: slopes spread geometrically, `2^(-2 - 4 h)` for `h < H - 1`, and
    /// exactly zero for the last head, so it reaches every distance at init.
    Spread,
}

impl AgeInit {
    fn name(self) -> &'static str {
        match self {
            AgeInit::Default => "default",
            AgeInit::Flat => "flat",
            AgeInit::Spread => "spread",
        }
    }

    /// The slope of `head` of `heads`, or `None` to keep the stack's own.
    fn slope(self, head: usize, heads: usize) -> Option<f64> {
        match self {
            AgeInit::Default => None,
            AgeInit::Flat => Some(0.0),
            AgeInit::Spread if head + 1 == heads => Some(0.0),
            AgeInit::Spread => Some(2f64.powi(-2 - 4 * head as i32)),
        }
    }
}

/// The key/query lineage of a stack arm (Step 2 arms).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LineageArm {
    /// No lineage: the plain read keys.
    None,
    /// F2: `k_t + j k_{t-1}` (`StackModel::set_read_key_shift`, saveable).
    F2,
    /// A research-only lineage (`StackModel::set_read_lineage`).
    Research(ReadLineage),
}

impl LineageArm {
    fn name(self) -> &'static str {
        match self {
            LineageArm::None => "none",
            LineageArm::F2 => "f2",
            LineageArm::Research(lineage) => lineage.name(),
        }
    }

    /// `lineage=NAME`; `seed` seeds the random SO(4) map, `lag1` the learned
    /// controls' initial lag-1 channel (identity) or its absence (zero).
    fn parse(name: &str, seed: u64, lag1: bool) -> Result<Self> {
        Ok(match name {
            "none" => LineageArm::None,
            "f2" => LineageArm::F2,
            "qk" => LineageArm::Research(ReadLineage::QueryKeyJ),
            "qk_jj" => LineageArm::Research(ReadLineage::QueryKeyJSame),
            "identity" => LineageArm::Research(ReadLineage::IdentityShift),
            "so4" => LineageArm::Research(ReadLineage::RandomSo4 { seed }),
            "conv" => LineageArm::Research(ReadLineage::LearnedConv { init_lag1: lag1 }),
            "wprev" => LineageArm::Research(ReadLineage::LearnedPrev {
                init_identity: lag1,
            }),
            other => return Err(invalid(format!("invalid lineage={other}"))),
        })
    }

    fn record(self) -> Value {
        match self {
            LineageArm::None | LineageArm::F2 => json!({"name": self.name()}),
            LineageArm::Research(lineage) => json!({"name": self.name(), "spec": lineage}),
        }
    }
}

/// The arm registry: one variant per context-access mechanism.
#[derive(Clone, Debug)]
enum ArmSpec {
    /// `arm=stack`: the geometric stack with a layer pattern of `r`
    /// (quaternion recurrence) and `a` (read) letters.
    Stack {
        pattern: String,
        read: ReadScore,
        rotation: bool,
        width: usize,
        heads: usize,
        mlp_hidden: usize,
        age: AgeInit,
        /// The key/query lineage. F2 (`key_shift=true` or `lineage=f2`):
        /// every read key also carries the previous position's key, turned by
        /// the unit quaternion `j` (`StackModel::set_read_key_shift`).
        lineage: LineageArm,
    },
    /// `arm=transformer`: the ordinary matched control -- `layers` plain
    /// causal-softmax attention blocks (`StackArch::Transformer`, RoPE plus a
    /// strict causal mask, SwiGLU MLP, pre-norm), exchanging the geometric
    /// read for the library's own ordinary attention path
    /// (`StackModel::attention` -> `fused_read_selected(ReadScore::Dot)`,
    /// `geometric_stack.rs:2021-2052`).
    ///
    /// This is D20 section 2 condition 1's ordinary control. It is a NEW arm:
    /// no geometric code path is touched (see `MIGRATION` in the report).
    ///
    /// Matching. `match_pattern`, `match_read` and `match_rotation` name the
    /// geometric arm whose total parameter count is matched; `width`, `heads`
    /// and `context` are shared. `layers` defaults to that pattern's length.
    /// `mlp=matched` (the default) solves the MLP width whose
    /// `StackConfig::parameter_count` is closest to the reference arm's, the
    /// mirror of the library's `geometric_matched_to`; `mlp=<n>` sets it
    /// directly. The report records both counts and the residual
    /// (`parameter_match`), so the match is a number, not an assertion.
    Transformer {
        layers: usize,
        width: usize,
        heads: usize,
        /// `None` = solve for the reference arm's parameter count.
        mlp_hidden: Option<usize>,
        reference_pattern: String,
        reference_read: ReadScore,
        reference_rotation: bool,
    },
}

/// The frozen Step 2 recipe's MLP width (barrier assessment
/// `proposals-and-reviews.md:637`: `rrarra`, width 128, 4 heads, MLP 384,
/// context 512, 1800 steps, batch 8, lr 1e-3, `l2` read). The reference arm
/// that `arm=transformer` matches in parameters uses it.
const REFERENCE_MLP: usize = 384;

/// The geometric arm `arm=transformer` matches: the same width, heads,
/// context, vocab and seed as the control, at the recipe's MLP width. Only its
/// total parameter count is used; it is never trained here.
fn reference_config(
    common: &Common,
    width: usize,
    heads: usize,
    pattern: &str,
    read: ReadScore,
    rotation: bool,
) -> Result<StackConfig> {
    let config = StackConfig {
        arch: StackArch::Geometric,
        vocab_size: common.vocab,
        width,
        heads,
        mlp_hidden: REFERENCE_MLP,
        context: common.context,
        pattern: pattern.to_owned(),
        read,
        rotation,
        rotation_group: RotationGroup::Quaternion,
        seed: common.seed,
        memory: None,
        select: None,
        pointer: None,
    };
    config.validate()?;
    Ok(config)
}

/// The MLP width of an ordinary attention control whose total parameter count
/// is closest to `target`. The exact mirror of the library's private
/// `matched_mlp_hidden` (`geometric_stack.rs:1339-1352`), applied to the
/// control's config: every layer carries `3 * width * mlp_hidden` MLP
/// parameters, so the solve is the same arithmetic.
fn matched_control_mlp(config: &StackConfig, target: usize) -> Result<usize> {
    let mut probe = config.clone();
    probe.mlp_hidden = 1;
    let base = probe.parameter_count()?;
    let per_unit = 3 * config.width * config.layers();
    if target <= base {
        return Err(invalid("the matched control has no room for an MLP"));
    }
    Ok((((target - base) as f64 / per_unit as f64).round() as usize + 1).max(1))
}

impl ArmSpec {
    fn parse(args: &mut Args, seed: u64) -> Result<Self> {
        let key_shift: bool = args.parsed("key_shift", false)?;
        let lag1 = match args.take("learned_init").as_deref() {
            None | Some("lag1") => true,
            Some("zero") => false,
            Some(other) => return Err(invalid(format!("invalid learned_init={other}"))),
        };
        let lineage = match args.take("lineage") {
            None if key_shift => LineageArm::F2,
            None => LineageArm::None,
            Some(name) => {
                let lineage = LineageArm::parse(&name, seed, lag1)?;
                if key_shift && lineage != LineageArm::F2 {
                    return Err(invalid("key_shift=true is lineage=f2; give one of them"));
                }
                lineage
            }
        };
        match args.take("arm").as_deref() {
            None | Some("stack") => Ok(ArmSpec::Stack {
                pattern: args.take("pattern").unwrap_or_else(|| "aaaaaa".into()),
                read: match args.take("read").as_deref() {
                    None | Some("l2") => ReadScore::L2,
                    Some("dot") => ReadScore::Dot,
                    Some("lorentz") => ReadScore::Lorentz,
                    Some(other) => return Err(invalid(format!("invalid read={other}"))),
                },
                rotation: args.parsed("rotation", true)?,
                width: args.parsed("width", 128usize)?,
                heads: args.parsed("heads", 4usize)?,
                mlp_hidden: args.parsed("mlp", 384usize)?,
                age: match args.take("age").as_deref() {
                    None | Some("default") => AgeInit::Default,
                    Some("flat") => AgeInit::Flat,
                    Some("spread") => AgeInit::Spread,
                    Some(other) => return Err(invalid(format!("invalid age={other}"))),
                },
                lineage,
            }),
            Some("transformer") => {
                // The control has no read lineage: reject rather than silently ignore.
                if lineage != LineageArm::None {
                    return Err(invalid(
                        "arm=transformer is the ordinary control and takes no key/query lineage",
                    ));
                }
                let reference_pattern = args
                    .take("match_pattern")
                    .unwrap_or_else(|| "rrarra".into());
                let reference_read = match args.take("match_read").as_deref() {
                    None | Some("l2") => ReadScore::L2,
                    Some("dot") => ReadScore::Dot,
                    Some("lorentz") => ReadScore::Lorentz,
                    Some(other) => return Err(invalid(format!("invalid match_read={other}"))),
                };
                let reference_rotation = args.parsed("match_rotation", true)?;
                let layers = args.parsed("layers", reference_pattern.chars().count())?;
                let mlp_hidden = match args.take("mlp").as_deref() {
                    None | Some("matched") => None,
                    Some(text) => Some(
                        text.parse()
                            .map_err(|_| invalid(format!("invalid mlp={text}")))?,
                    ),
                };
                Ok(ArmSpec::Transformer {
                    layers,
                    width: args.parsed("width", 128usize)?,
                    heads: args.parsed("heads", 4usize)?,
                    mlp_hidden,
                    reference_pattern,
                    reference_read,
                    reference_rotation,
                })
            }
            Some(other) => Err(invalid(format!("unknown arm={other}"))),
        }
    }

    /// A short label for logs and report paths.
    fn label(&self) -> String {
        match self {
            ArmSpec::Stack {
                pattern,
                read,
                rotation,
                age,
                lineage,
                ..
            } => format!(
                "stack-{pattern}-{read:?}{}{}{}",
                match age {
                    AgeInit::Default => String::new(),
                    other => format!("-{}-age", other.name()),
                },
                match lineage {
                    LineageArm::None => String::new(),
                    LineageArm::F2 => "-key-shift".into(),
                    other => format!("-lineage-{}", other.name()),
                },
                if *rotation { "" } else { "-no-rotation" },
            )
            .to_lowercase(),
            ArmSpec::Transformer {
                layers,
                mlp_hidden,
                reference_pattern,
                ..
            } => format!(
                "control-transformer-{layers}x-{reference_pattern}{}",
                match mlp_hidden {
                    None => "-mlp-matched".to_string(),
                    Some(mlp) => format!("-mlp{mlp}"),
                }
            )
            .to_lowercase(),
        }
    }

    /// Validates without building, so a bad arm fails before the report claim.
    fn validate(&self, common: &Common) -> Result<()> {
        match self {
            ArmSpec::Stack {
                pattern,
                lineage,
                width,
                ..
            } => {
                if *lineage != LineageArm::None && !pattern.contains('a') {
                    return Err(invalid("a key/query lineage needs a read layer"));
                }
                if *lineage != LineageArm::None && !width.is_multiple_of(4) {
                    return Err(invalid("a key/query lineage needs four-channel lanes"));
                }
                self.stack_config(common)?.validate()
            }
            ArmSpec::Transformer { .. } => {
                self.stack_config(common)?.validate()?;
                self.parameter_match(common)?;
                Ok(())
            }
        }
    }

    /// The parameter-match evidence: the reference geometric arm's count, the
    /// control's count and the residual. `null` for a geometric arm.
    fn parameter_match(&self, common: &Common) -> Result<Value> {
        match self {
            ArmSpec::Stack { .. } => Ok(Value::Null),
            ArmSpec::Transformer {
                width,
                heads,
                reference_pattern,
                reference_read,
                reference_rotation,
                ..
            } => {
                let reference = reference_config(
                    common,
                    *width,
                    *heads,
                    reference_pattern,
                    *reference_read,
                    *reference_rotation,
                )?;
                let target = reference.parameter_count()?;
                let control = self.stack_config(common)?;
                let got = control.parameter_count()?;
                Ok(json!({
                    "reference_geometric": {
                        "pattern": reference.pattern,
                        "read": reference.read,
                        "rotation": reference.rotation,
                        "mlp_hidden": reference.mlp_hidden,
                        "parameters": target,
                    },
                    "control": {
                        "arch": control.arch,
                        "pattern": control.pattern,
                        "layers": control.layers(),
                        "mlp_hidden": control.mlp_hidden,
                        "parameters": got,
                    },
                    "residual_parameters": got as i64 - target as i64,
                    "residual_fraction": (got as f64 - target as f64) / target as f64,
                }))
            }
        }
    }

    fn stack_config(&self, common: &Common) -> Result<StackConfig> {
        match self {
            ArmSpec::Stack {
                pattern,
                read,
                rotation,
                width,
                heads,
                mlp_hidden,
                ..
            } => {
                let config = StackConfig {
                    arch: StackArch::Geometric,
                    vocab_size: common.vocab,
                    width: *width,
                    heads: *heads,
                    mlp_hidden: *mlp_hidden,
                    context: common.context,
                    pattern: pattern.clone(),
                    read: *read,
                    rotation: *rotation,
                    rotation_group: RotationGroup::Quaternion,
                    seed: common.seed,
                    memory: None,
                    select: None,
                    pointer: None,
                };
                config.validate()?;
                Ok(config)
            }
            ArmSpec::Transformer {
                layers,
                width,
                heads,
                mlp_hidden,
                reference_pattern,
                reference_read,
                reference_rotation,
            } => {
                let reference = reference_config(
                    common,
                    *width,
                    *heads,
                    reference_pattern,
                    *reference_read,
                    *reference_rotation,
                )?;
                let target = reference.parameter_count()?;
                let mut control = StackConfig {
                    arch: StackArch::Transformer,
                    vocab_size: common.vocab,
                    width: *width,
                    heads: *heads,
                    mlp_hidden: 1,
                    context: common.context,
                    pattern: "a".repeat(*layers),
                    read: ReadScore::Dot,
                    rotation: false,
                    rotation_group: RotationGroup::Quaternion,
                    seed: common.seed,
                    memory: None,
                    select: None,
                    pointer: None,
                };
                control.mlp_hidden = match mlp_hidden {
                    Some(explicit) => *explicit,
                    None => matched_control_mlp(&control, target)?,
                };
                control.validate()?;
                Ok(control)
            }
        }
    }

    fn build(&self, common: &Common, device: &Device) -> Result<Box<dyn ContextArm>> {
        match self {
            ArmSpec::Stack { age, lineage, .. } => {
                let config = self.stack_config(common)?;
                let mut model = StackModel::new(config.clone(), device)?;
                // The pointer needs BOTH a head and a route: StackConfig carries a
                // PointerConfig (the head), the route is installed on the model. Both are off
                // unless `pointer=` is given, which is what every prior run did -- so the
                // geometric `ngram` successor route (longest ordered n-let match) has never
                // been exercised here. This is D20 section 2 condition 3 in its exact form.
                if let Some(route) = common.pointer.clone() {
                    model.add_pointer(PointerConfig::new(config.width), common.seed)?;
                    model.set_pointer_route(Some(route))?;
                }
                if *age != AgeInit::Default {
                    for (name, var) in model.variables() {
                        if !name.ends_with(".read.age") {
                            continue;
                        }
                        let mut values = Vec::with_capacity(config.heads * config.context);
                        for head in 0..config.heads {
                            let slope = age
                                .slope(head, config.heads)
                                .ok_or_else(|| invalid("age init without a slope"))?;
                            values.extend((0..config.context).map(|d| (-slope * d as f64) as f32));
                        }
                        var.set(&Tensor::from_vec(
                            values,
                            (config.heads, config.context),
                            device,
                        )?)?;
                    }
                }
                // Before the optimizer: learned lineages add variables.
                match lineage {
                    LineageArm::None => {}
                    LineageArm::F2 => model.set_read_key_shift(true)?,
                    LineageArm::Research(research) => model.set_read_lineage(Some(*research))?,
                }
                let optimizer = StackAdamW::new(&model, common.weight_decay, common.clip)?;
                Ok(Box::new(StackArm {
                    model,
                    optimizer,
                    age: *age,
                    lineage: *lineage,
                }))
            }
            ArmSpec::Transformer { .. } => {
                // The control has no recurrence, no read layer, no age bias and
                // no lineage: the model is the library's ordinary attention stack.
                let config = self.stack_config(common)?;
                let model = StackModel::new(config, device)?;
                let optimizer = StackAdamW::new(&model, common.weight_decay, common.clip)?;
                Ok(Box::new(StackArm {
                    model,
                    optimizer,
                    age: AgeInit::Default,
                    lineage: LineageArm::None,
                }))
            }
        }
    }
}

struct StackArm {
    model: StackModel,
    optimizer: StackAdamW,
    age: AgeInit,
    lineage: LineageArm,
}

impl StackArm {
    fn read_layers(&self) -> usize {
        self.model
            .config
            .pattern
            .chars()
            .filter(|&c| c == 'a')
            .count()
    }
}

impl ContextArm for StackArm {
    fn kind(&self) -> &'static str {
        match self.model.config.arch {
            StackArch::Geometric => "geometric_stack",
            StackArch::Transformer => "ordinary_attention",
        }
    }

    fn record(&self) -> Value {
        let heads = self.model.config.heads;
        json!({
            "stack_config": self.model.config,
            "age_init": self.age.name(),
            "age_init_slopes": (0..heads)
                .map(|h| self.age.slope(h, heads)
                    .unwrap_or_else(|| 2f64.powf(-8.0 * (h + 1) as f64 / heads as f64)))
                .collect::<Vec<_>>(),
            "read_key_shift": self.model.read_key_shift(),
            "lineage": self.lineage.name(),
            "lineage_record": self.lineage.record(),
            "read_layers": self.read_layers(),
            "recurrence_layers": self.model.config.layers() - self.read_layers(),
            "recurrent_state_floats_per_layer": self.model.config.width,
        })
    }

    fn parameters(&self) -> usize {
        self.model.parameter_count()
    }

    fn device(&self) -> &Device {
        self.model.device()
    }

    fn loss(
        &self,
        ids: &[u32],
        targets: &[u32],
        weights: &[f32],
        batch: usize,
        time: usize,
    ) -> Result<Tensor> {
        self.model.weighted_loss(ids, targets, weights, batch, time)
    }

    fn logits(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        self.model.forward(ids, batch, time)
    }

    fn update(&mut self, loss: &Tensor, lr: f64) -> Result<f64> {
        let grads = loss.backward()?;
        self.optimizer.update(&self.model, &grads, lr)
    }

    fn positions_scored(&self, position: usize) -> usize {
        self.read_layers() * self.model.config.heads * (position + 1)
    }

    fn access_note(&self) -> String {
        match self.model.config.arch {
            StackArch::Geometric => "read layers x heads x (position + 1): every causal position is scored by each read head (plus one NoRead slot, not counted); recurrence layers score no positions".into(),
            StackArch::Transformer => "attention layers x heads x (position + 1): every causal position is scored by each head of the ordinary causal-softmax control; no NoRead slot".into(),
        }
    }

    fn save(&self, directory: &Path) -> Result<()> {
        self.model.save(directory)
    }

    fn read_heads(&self) -> Vec<(usize, usize)> {
        let config = &self.model.config;
        // The binding-mass probe reads the geometric read
        // (`StackModel::read_binding_masses`); the control's attention path does
        // not populate that capture (`geometric_stack.rs:6332-6334`), so it
        // reports no read heads rather than an error. The geometric arm's
        // answer is unchanged.
        if config.arch != StackArch::Geometric {
            return Vec::new();
        }
        (0..config.layers())
            .filter(|&layer| config.pattern.as_bytes()[layer] == b'a')
            .flat_map(|layer| (0..config.heads).map(move |head| (layer, head)))
            .collect()
    }

    fn source_mass(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        layer: usize,
        head: usize,
        rows: &[(usize, usize, Vec<usize>)],
    ) -> Result<Vec<f32>> {
        let target = ReadBindingTarget {
            layer,
            head,
            rows: rows
                .iter()
                .map(|(batch, query, sources)| ReadBinding {
                    batch: *batch,
                    query: *query,
                    sources: sources.clone(),
                })
                .collect(),
        };
        Ok(self
            .model
            .read_binding_masses(ids, batch, time, &target)?
            .flatten_all()?
            .to_device(&Device::Cpu)?
            .to_vec1::<f32>()?)
    }

    fn read_weight_rows(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        rows: &[(usize, usize)],
    ) -> Result<(Vec<(usize, Tensor)>, Tensor)> {
        self.model.read_weight_rows(ids, batch, time, rows)
    }
}

/// The deciding probe: each read head's softmax weight on the true key
/// position `q - d` and on the value position `q - d + 1` after it, by bucket.
/// One query per probe sequence and bucket (its first of that bucket).
fn read_probe(
    arm: &dyn ContextArm,
    sequences: &[Sequence],
    context: usize,
    buckets: &[Bucket],
) -> Result<Value> {
    let heads = arm.read_heads();
    if heads.is_empty() {
        return Ok(json!({"status": "UNAVAILABLE: the arm has no read heads"}));
    }
    let (ids, _, _) = batch_arrays(sequences, context);
    let mut by_bucket = serde_json::Map::new();
    for (b, bucket) in buckets.iter().enumerate() {
        let picks: Vec<(usize, &Query)> = sequences
            .iter()
            .enumerate()
            .filter_map(|(s, sequence)| {
                sequence
                    .queries
                    .iter()
                    .find(|query| query.bucket == b)
                    .map(|query| (s, query))
            })
            .collect();
        let uniform = picks
            .iter()
            .map(|(_, query)| 1.0 / (query.position + 1) as f64)
            .sum::<f64>()
            / picks.len().max(1) as f64;
        let mut per_head = Vec::new();
        let (mut best_key, mut best_value) = (0f64, 0f64);
        for &(layer, head) in &heads {
            let mut means = [0f64; 2];
            let mut over_half = [0usize; 2];
            for (which, offset) in [(0usize, 0usize), (1, 1)] {
                let rows: Vec<(usize, usize, Vec<usize>)> = picks
                    .iter()
                    .map(|(s, query)| {
                        (
                            *s,
                            query.position,
                            vec![query.position - query.distance + offset],
                        )
                    })
                    .collect();
                let masses = arm.source_mass(&ids, sequences.len(), context, layer, head, &rows)?;
                means[which] =
                    masses.iter().map(|&m| f64::from(m)).sum::<f64>() / masses.len().max(1) as f64;
                over_half[which] = masses.iter().filter(|&&m| m > 0.5).count();
            }
            best_key = best_key.max(means[0]);
            best_value = best_value.max(means[1]);
            per_head.push(json!({
                "layer": layer, "head": head,
                "mean_weight_on_key": means[0], "mean_weight_on_value": means[1],
                "queries_over_half_on_key": over_half[0],
                "queries_over_half_on_value": over_half[1],
            }));
        }
        by_bucket.insert(
            bucket.name.into(),
            json!({
                "queries": picks.len(),
                "uniform_weight_reference": uniform,
                "best_head_mean_weight_on_key": best_key,
                "best_head_mean_weight_on_value": best_value,
                "heads": per_head,
            }),
        );
    }
    Ok(json!({"by_bucket": by_bucket}))
}

fn probe_line(probe: &Value, buckets: &[Bucket]) -> String {
    buckets
        .iter()
        .map(|b| {
            let entry = &probe["by_bucket"][b.name];
            format!(
                "{} key {:.3} value {:.3} (uniform {:.4})",
                b.name,
                entry["best_head_mean_weight_on_key"]
                    .as_f64()
                    .unwrap_or(f64::NAN),
                entry["best_head_mean_weight_on_value"]
                    .as_f64()
                    .unwrap_or(f64::NAN),
                entry["uniform_weight_reference"]
                    .as_f64()
                    .unwrap_or(f64::NAN),
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// D20 section 2 condition 3, for the geometric read: a firing counter in the
/// spirit of `Work::selected_operations` (the `addressed_attention` precedent).
///
/// It evaluates the arm's real read at **every** scored query position of
/// `sequences` through `ContextArm::source_mass` (`StackModel::read_binding_masses`,
/// "the actual read, including age and NoRead") and counts the read rows that
/// executed and the rows that put nonzero weight on the gold key `q - d`. A
/// read that was never reached cannot produce a nonzero row; an arm with no
/// read heads reports UNAVAILABLE, which is a finding, not a zero.
/// Observation only: a separate forward pass, no parameter and no logit change.
fn read_firing(
    arm: &dyn ContextArm,
    sequences: &[Sequence],
    context: usize,
    buckets: &[Bucket],
) -> Result<Value> {
    let heads = arm.read_heads();
    if heads.is_empty() {
        return Ok(json!({
            "status": "UNAVAILABLE: this arm's mechanism exposes no read-binding mass \
                       (the ordinary control's attention path does not populate the \
                       binding capture, geometric_stack.rs:6332-6334)"
        }));
    }
    // The binding probe admits at most one labelled query per batch item
    // (`validate_binding`, geometric_stack.rs:6502), so each selected query is
    // presented as its own batch item: its sequence's tokens, repeated. One
    // query per (sequence, bucket), for every sequence of the panel.
    let mut picks: Vec<(usize, usize)> = Vec::new(); // (sequence, query index)
    for (s, sequence) in sequences.iter().enumerate() {
        for (b, _) in buckets.iter().enumerate() {
            if let Some(index) = sequence.queries.iter().position(|query| query.bucket == b) {
                picks.push((s, index));
            }
        }
    }
    let batch = picks.len();
    let mut ids = Vec::with_capacity(batch * context);
    for (s, _) in &picks {
        ids.extend_from_slice(&sequences[*s].tokens);
    }
    let rows: Vec<(usize, usize, Vec<usize>)> = picks
        .iter()
        .enumerate()
        .map(|(item, (s, index))| {
            let query = &sequences[*s].queries[*index];
            (item, query.position, vec![query.position - query.distance])
        })
        .collect();
    let mut rows_evaluated = 0usize;
    let mut rows_nonzero = 0usize;
    let mut rows_over_half = 0usize;
    let mut mass_sum = 0f64;
    let mut uniform_sum = 0f64;
    let mut per_head = Vec::new();
    let mut per_bucket_rows = vec![0usize; buckets.len()];
    let mut per_bucket_nonzero = vec![0usize; buckets.len()];
    let mut per_bucket_mass = vec![0f64; buckets.len()];
    let mut per_bucket_uniform = vec![0f64; buckets.len()];
    for &(layer, head) in &heads {
        let masses = arm.source_mass(&ids, batch, context, layer, head, &rows)?;
        if masses.len() != rows.len() {
            return Err(invalid("the read firing counter got the wrong row count"));
        }
        let (mut head_nonzero, mut head_over_half, mut head_mass) = (0usize, 0usize, 0f64);
        for (i, &mass) in masses.iter().enumerate() {
            let (s, index) = picks[i];
            let query = &sequences[s].queries[index];
            let bucket = query.bucket;
            let uniform = 1.0 / (query.position + 1) as f64;
            rows_evaluated += 1;
            mass_sum += f64::from(mass);
            uniform_sum += uniform;
            per_bucket_rows[bucket] += 1;
            per_bucket_mass[bucket] += f64::from(mass);
            per_bucket_uniform[bucket] += uniform;
            if mass > 0.0 {
                rows_nonzero += 1;
                head_nonzero += 1;
                per_bucket_nonzero[bucket] += 1;
            }
            if mass > 0.5 {
                rows_over_half += 1;
                head_over_half += 1;
            }
            head_mass += f64::from(mass);
        }
        per_head.push(json!({
            "layer": layer, "head": head,
            "rows_evaluated": masses.len(),
            "rows_nonzero_mass_on_key": head_nonzero,
            "rows_over_half_on_key": head_over_half,
            "summed_mass_on_key": head_mass,
        }));
    }
    let by_bucket = buckets
        .iter()
        .enumerate()
        .map(|(b, bucket)| {
            (
                bucket.name.to_owned(),
                json!({
                    "rows": per_bucket_rows[b],
                    "rows_nonzero_mass_on_key": per_bucket_nonzero[b],
                    "mean_mass_on_key": per_bucket_mass[b] / per_bucket_rows[b].max(1) as f64,
                    "mean_uniform_reference": per_bucket_uniform[b] / per_bucket_rows[b].max(1) as f64,
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    Ok(json!({
        "status": "MEASURED",
        "counter": "read rows evaluated through StackModel::read_binding_masses at the scored query positions",
        "selection": "one held-out query per (sequence, bucket), every sequence of the panel, one batch item each",
        "read_heads": heads.len(),
        "sequences": sequences.len(),
        "queries": rows.len(),
        "batch_items": batch,
        "rows_evaluated": rows_evaluated,
        "rows_nonzero_mass_on_key": rows_nonzero,
        "rows_over_half_on_key": rows_over_half,
        "mean_mass_on_key": mass_sum / rows_evaluated.max(1) as f64,
        "mean_uniform_reference": uniform_sum / rows_evaluated.max(1) as f64,
        "by_bucket": by_bucket,
        "by_head": per_head,
    }))
}

/// D20 section 2 condition 3, arch-symmetric: the mechanism is reached on a
/// scored position iff that position's logits depend on the source key token.
/// Each query's key occurrence (`q - d`) is replaced by a filler token and the
/// logits at `q` are compared with the unmodified forward. A mechanism that
/// never read the context leaves them bit-identical, whatever its parameter
/// count. Observation only: a separate forward pass, no parameter or training
/// change.
fn context_reachability(
    arm: &dyn ContextArm,
    sequences: &[Sequence],
    context: usize,
) -> Result<Value> {
    let (ids, _, _) = batch_arrays(sequences, context);
    let before = arm.logits(&ids, sequences.len(), context)?;
    let mut ablated = sequences.to_vec();
    let mut replaced = 0usize;
    for sequence in ablated.iter_mut() {
        for query in sequence.queries.iter() {
            let at = query.position - query.distance;
            let original = sequence.tokens[at];
            // A filler token, never equal to the key that was there.
            let mut replacement = FILLER.0 + (original % (FILLER.1 - FILLER.0));
            if replacement == original {
                replacement = FILLER.0 + (original + 1) % (FILLER.1 - FILLER.0);
            }
            sequence.tokens[at] = replacement;
            replaced += 1;
        }
    }
    let (ablated_ids, _, _) = batch_arrays(&ablated, context);
    let after = arm.logits(&ablated_ids, sequences.len(), context)?;
    // The scored rows, in panel order.
    let mut rows = Vec::new();
    for (s, sequence) in sequences.iter().enumerate() {
        for query in sequence.queries.iter() {
            rows.push((
                (s * context + query.position) as u32,
                query.value as usize,
            ));
        }
    }
    let index = Tensor::from_vec(
        rows.iter().map(|row| row.0).collect::<Vec<u32>>(),
        rows.len(),
        arm.device(),
    )?;
    let b = before
        .index_select(&index, 0)?
        .to_device(&Device::Cpu)?
        .to_vec2::<f32>()?;
    let a = after
        .index_select(&index, 0)?
        .to_device(&Device::Cpu)?
        .to_vec2::<f32>()?;
    let (mut changed, mut argmax_changed, mut gold_logit_changed) = (0usize, 0usize, 0usize);
    let (mut max_delta, mut sum_delta) = (0f64, 0f64);
    for (row, (_, gold)) in rows.iter().enumerate() {
        let (mut best_b, mut best_a) = (0usize, 0usize);
        let (mut mb, mut ma) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut delta = 0f64;
        for (i, (&x, &y)) in b[row].iter().zip(a[row].iter()).enumerate() {
            delta = delta.max(f64::from((x - y).abs()));
            if x > mb {
                mb = x;
                best_b = i;
            }
            if y > ma {
                ma = y;
                best_a = i;
            }
        }
        let any = b[row]
            .iter()
            .zip(a[row].iter())
            .any(|(x, y)| x.to_bits() != y.to_bits());
        if any {
            changed += 1;
        }
        if b[row][*gold].to_bits() != a[row][*gold].to_bits() {
            gold_logit_changed += 1;
        }
        if best_b != best_a {
            argmax_changed += 1;
        }
        max_delta = max_delta.max(delta);
        sum_delta += delta;
    }
    let total = rows.len().max(1);
    Ok(json!({
        "method": "replace the query's source key at q - d with a filler token; compare logits at q",
        "rows": rows.len(),
        "key_tokens_replaced": replaced,
        "rows_with_any_logit_change": changed,
        "rows_with_gold_logit_change": gold_logit_changed,
        "rows_whose_argmax_changed": argmax_changed,
        "max_abs_logit_delta": max_delta,
        "mean_max_abs_logit_delta": sum_delta / total as f64,
    }))
}

/// The value written most recently before `query` (by any pair): what a pure
/// recency read with no key match would answer.
fn most_recent_value(sequence: &Sequence, query: &Query) -> Option<u32> {
    sequence
        .queries
        .iter()
        .map(|pair| (pair.position - pair.distance + 1, pair.value))
        .filter(|&(value_position, _)| value_position < query.position)
        .max_by_key(|&(value_position, _)| value_position)
        .map(|(_, value)| value)
}

/// Data-only baselines per bucket, independent of any arm: the accuracy of
/// answering with the most recently written value (recency, no key match).
fn task_baselines(sequences: &[Sequence], buckets: &[Bucket]) -> Value {
    let mut hits = vec![(0usize, 0usize); buckets.len()];
    for sequence in sequences {
        for query in &sequence.queries {
            let entry = &mut hits[query.bucket];
            entry.1 += 1;
            entry.0 += usize::from(most_recent_value(sequence, query) == Some(query.value));
        }
    }
    let mut by_bucket = serde_json::Map::new();
    for (bucket, (hit, total)) in buckets.iter().zip(&hits) {
        by_bucket.insert(
            bucket.name.into(),
            json!({"recency_accuracy": *hit as f64 / (*total).max(1) as f64, "total": total}),
        );
    }
    json!({"by_bucket": by_bucket, "rule": "answer the value written most recently before the query"})
}

/// Recall by bucket: argmax over the whole vocabulary at each query.
fn evaluate(
    arm: &dyn ContextArm,
    sequences: &[Sequence],
    context: usize,
    buckets: &[Bucket],
    chunk: usize,
) -> Result<Value> {
    let mut tallies = vec![Tally::default(); buckets.len()];
    let mut scored = 0usize;
    for group in sequences.chunks(chunk.max(1)) {
        let (ids, _, _) = batch_arrays(group, context);
        let logits = arm.logits(&ids, group.len(), context)?;
        let mut rows = Vec::new();
        for (s, sequence) in group.iter().enumerate() {
            for query in &sequence.queries {
                rows.push((s * context + query.position) as u32);
            }
        }
        let index = Tensor::from_vec(rows.clone(), rows.len(), arm.device())?;
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
                scored += arm.positions_scored(query.position);
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
                // The most recent value written by another pair.
                let most_recent = sequence
                    .queries
                    .iter()
                    .filter(|other| {
                        other.value != query.value
                            && other.position - other.distance + 1 < query.position
                    })
                    .max_by_key(|other| other.position - other.distance)
                    .map(|other| other.value);
                let predicted = argmax as u32;
                if predicted != query.value {
                    tally.most_recent_value += usize::from(most_recent == Some(predicted));
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
                "wrong_most_recent_pair_value": tally.most_recent_value,
            }),
        );
    }
    Ok(json!({
        "by_bucket": by_bucket,
        "correct": correct,
        "total": total,
        "accuracy": correct as f64 / total.max(1) as f64,
        "chance_accuracy": 1.0 / f64::from(VALUES.1 - VALUES.0),
        "positions_scored_per_query_token_mean": scored as f64 / total.max(1) as f64,
        "task_baselines": task_baselines(sequences, buckets),
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

/// Task, training and evaluation settings shared by every arm.
struct Common {
    out: PathBuf,
    device_name: String,
    /// The model vocabulary: `VOCAB` for the synthetic layout, the
    /// tokenizer's for `layout=fact`.
    vocab: usize,
    /// Save the trained weights under `model/` of the report root.
    save_model: bool,
    context: usize,
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
    /// The pointer route, from `pointer=<spec>` (`none`, `prime:<w>`, `prime-ranked:<w>`,
    /// `ngram:<w>`, `ngram-ranked:<w>`). DEFAULTS TO `None`, which is what every prior run
    /// used -- the arm was hardcoded and the geometric `ngram` successor route (admission by
    /// the longest ordered n-let match) has therefore NEVER been exercised on this bench.
    /// That is D20 section 2 condition 3 ("wiring verified reached") in its exact form.
    pointer: Option<PrimeRoute>,
    max_seconds: f64,
    /// Steps at which the read probe runs (0 = before training).
    probe_steps: Vec<usize>,
    probe_sequences: usize,
    /// TEMPORARY (T2): dump the read's weight rows on the held-out set.
    dump_scores: bool,
}

impl Common {
    fn record(&self) -> Value {
        let buckets = buckets_for(self.context);
        json!({
            "device": self.device_name,
            "threads": std::env::var("RAYON_NUM_THREADS").ok(),
            "context": self.context,
            "vocab": self.vocab,
            "save_model": self.save_model,
            "token_ranges": {"filler": FILLER, "keys": KEYS, "values": VALUES},
            "pairing_split": {
                "classes": CLASSES, "held_out_class": HELD_OUT_CLASS,
                "rule": "(key - 64 + value - 288) % 4; training draws classes 1..3, the held-out evaluation class 0",
            },
            "buckets": buckets.iter().map(|b| json!({"name": b.name, "low": b.low, "high": b.high})).collect::<Vec<_>>(),
            "pairs_per_bucket": self.pairs_per_bucket,
            "batch": self.batch, "steps": self.steps, "lr": self.lr, "warmup": self.warmup,
            "min_lr": self.min_lr, "weight_decay": self.weight_decay, "clip": self.clip,
            "eval_every": self.eval_every, "curve_sequences": self.curve_sequences,
            "final_sequences": self.final_sequences, "seed": self.seed,
            "max_seconds": self.max_seconds,
            "probe_steps": self.probe_steps, "probe_sequences": self.probe_sequences,
            "dump_scores": self.dump_scores,
        })
    }
}

/// What `main` runs.
enum Mode {
    Train,
    TaskBaselines,
}

fn settings(mut args: Args) -> Result<(Common, ArmSpec, Mode, Option<fact::FactTask>)> {
    let mode = match args.take("mode").as_deref() {
        None | Some("train") => Mode::Train,
        Some("task-baselines") => Mode::TaskBaselines,
        Some(other) => return Err(invalid(format!("invalid mode={other}"))),
    };
    let fact_layout = match args.take("layout").as_deref() {
        None | Some("synthetic") => false,
        Some("fact") => true,
        Some(other) => return Err(invalid(format!("invalid layout={other}"))),
    };
    let out = PathBuf::from(
        args.take("out")
            .ok_or_else(|| invalid("out=NEW_REPORT_ROOT is required"))?,
    );
    let common = Common {
        out,
        device_name: args.take("device").unwrap_or_else(|| "cpu".into()),
        vocab: VOCAB,
        save_model: args.parsed("save_model", false)?,
        context: args.parsed(
            "context",
            if fact_layout {
                fact::SERVED_CONTEXT
            } else {
                512usize
            },
        )?,
        pairs_per_bucket: args.parsed("pairs_per_bucket", if fact_layout { 4 } else { 8usize })?,
        batch: args.parsed("batch", 8usize)?,
        steps: args.parsed("steps", 1800usize)?,
        lr: args.parsed("lr", 1e-3f64)?,
        warmup: args.parsed("warmup", 100usize)?,
        min_lr: args.parsed("min_lr", 0.1f64)?,
        weight_decay: args.parsed("weight_decay", 0.1f64)?,
        clip: args.parsed("clip", 1.0f64)?,
        eval_every: args.parsed("eval_every", 100usize)?,
        curve_sequences: args.parsed("curve_sequences", 8usize)?,
        final_sequences: args.parsed("final_sequences", if fact_layout { 256 } else { 64usize })?,
        seed: args.parsed("seed", 1u64)?,
        pointer: match args.take("pointer") {
            None => None,
            Some(text) => parse_pointer_route(&text)?,
        },
        max_seconds: args.parsed("max_seconds", 1200f64)?,
        probe_steps: Vec::new(),
        probe_sequences: args.parsed("probe_sequences", 16usize)?,
        // TEMPORARY (T2): `dump_scores=1` writes the read's own weight rows on
        // the held-out set into the report root; the default path is unchanged.
        dump_scores: match args.take("dump_scores").as_deref() {
            None | Some("0") | Some("false") => false,
            Some("1") | Some("true") => true,
            Some(other) => return Err(invalid(format!("invalid dump_scores={other}"))),
        },
    };
    let mut common = common;
    let probe_text = args
        .take("probe_steps")
        // The fact layout probes once, after training (or never: `none`).
        .unwrap_or_else(|| if fact_layout { "final" } else { "0,600,final" }.into());
    if fact_layout && !matches!(probe_text.as_str(), "final" | "none") {
        return Err(invalid("layout=fact takes probe_steps=final|none"));
    }
    if probe_text != "none" {
        for part in probe_text.split(',') {
            let step = match part {
                "final" => common.steps,
                number => number
                    .parse()
                    .map_err(|_| invalid(format!("invalid probe step {number}")))?,
            };
            if step > common.steps {
                return Err(invalid(format!("probe step {step} beyond steps")));
            }
            common.probe_steps.push(step);
        }
        common.probe_steps.sort_unstable();
        common.probe_steps.dedup();
    }
    let fact_task = if fact_layout {
        let task = fact::FactTask::parse(&mut args, &common)?;
        common.vocab = task.vocab.vocab;
        Some(task)
    } else {
        None
    };
    let arm = ArmSpec::parse(&mut args, common.seed)?;
    args.finish()?;
    if common.batch == 0
        || common.steps == 0
        || common.eval_every == 0
        || common.curve_sequences == 0
        || common.final_sequences < 2
        || !(common.lr > 0.0)
        || !(common.max_seconds > 0.0)
    {
        return Err(invalid(
            "batch, steps, eval_every, sequences, lr and max_seconds must be positive",
        ));
    }
    if buckets_for(common.context).is_empty() {
        return Err(invalid("the context holds no distance bucket"));
    }
    arm.validate(&common)?;
    Ok((common, arm, mode, fact_task))
}

const TRAIN_DOMAIN: u64 = 0x7472_6169_6E;
const CURVE_DOMAIN: u64 = 0x6375_7276_65;
const CURVE_IN_CLASS_DOMAIN: u64 = 0x6375_7276_63;
const HELD_OUT_DOMAIN: u64 = 0x6865_6C64;
const IN_CLASS_DOMAIN: u64 = 0x636C_6173_73;
const PROBE_DOMAIN: u64 = 0x7072_6F62_65;

fn sequences(
    s: &Common,
    buckets: &[Bucket],
    domain: u64,
    count: usize,
    pairing: Pairing,
) -> Result<Vec<Sequence>> {
    (0..count)
        .map(|i| {
            let mut rng = Rng::new(s.seed, domain, i as u64);
            generate(&mut rng, s.context, buckets, s.pairs_per_bucket, pairing)
        })
        .collect()
}

fn bucket_line(result: &Value, buckets: &[Bucket]) -> String {
    buckets
        .iter()
        .map(|b| {
            format!(
                "{}={:.3}",
                b.name,
                result["by_bucket"][b.name]["accuracy"]
                    .as_f64()
                    .unwrap_or(f64::NAN)
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Runs the read probe at its declared steps and keeps the results.
struct Prober<'a> {
    set: &'a [Sequence],
    steps: &'a [usize],
    context: usize,
    buckets: &'a [Bucket],
    probes: Vec<Value>,
    seconds: f64,
}

impl Prober<'_> {
    fn run(&mut self, arm: &dyn ContextArm, step: usize, log: &mut fs::File) -> Result<()> {
        use std::io::Write;
        if !self.steps.contains(&step) || self.probes.iter().any(|p| p["step"] == step) {
            return Ok(());
        }
        let clock = Instant::now();
        let result = read_probe(arm, self.set, self.context, self.buckets)?;
        let line = format!("probe step {step}: {}", probe_line(&result, self.buckets));
        eprintln!("{line}");
        writeln!(log, "{line}")?;
        self.probes.push(json!({"step": step, "probe": result}));
        self.seconds += clock.elapsed().as_secs_f64();
        Ok(())
    }
}

/// Trains `arm` on the task and scores it; returns the arm-independent
/// results block of the report.
fn run(s: &Common, arm: &mut dyn ContextArm, log: &mut fs::File) -> Result<Value> {
    use std::io::Write;
    let started = Instant::now();
    let buckets = buckets_for(s.context);
    let line = format!(
        "{} {}: {} parameters, context {}, buckets {:?}",
        arm.kind(),
        arm.record(),
        arm.parameters(),
        s.context,
        buckets.iter().map(|b| b.name).collect::<Vec<_>>()
    );
    eprintln!("{line}");
    writeln!(log, "{line}")?;
    let curve_held_out = sequences(
        s,
        &buckets,
        CURVE_DOMAIN,
        s.curve_sequences,
        Pairing::HeldOut,
    )?;
    let curve_in_class = sequences(
        s,
        &buckets,
        CURVE_IN_CLASS_DOMAIN,
        s.curve_sequences,
        Pairing::Train,
    )?;
    let chunk = s.batch;
    let mut curve = Vec::new();
    curve.push(json!({
        "step": 0,
        "in_class": evaluate(&*arm, &curve_in_class, s.context, &buckets, chunk)?,
        "held_out": evaluate(&*arm, &curve_held_out, s.context, &buckets, chunk)?,
    }));
    let probe_set = sequences(s, &buckets, PROBE_DOMAIN, s.probe_sequences, Pairing::Train)?;
    let mut prober = Prober {
        set: &probe_set,
        steps: &s.probe_steps,
        context: s.context,
        buckets: &buckets,
        probes: Vec::new(),
        seconds: 0.0,
    };
    prober.run(&*arm, 0, log)?;
    let (mut window_loss, mut window_steps) = (0f64, 0usize);
    let mut step_seconds = Vec::new();
    let mut stopped_early = false;
    let mut step = 0;
    while step < s.steps {
        // The probe is observation, not training: its time is not charged
        // against the training cap (it is reported separately).
        if started.elapsed().as_secs_f64() - prober.seconds > s.max_seconds {
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
                    s.context,
                    &buckets,
                    s.pairs_per_bucket,
                    Pairing::Train,
                )
            })
            .collect::<Result<_>>()?;
        let (ids, targets, weights) = batch_arrays(&batch, s.context);
        let loss = arm.loss(&ids, &targets, &weights, s.batch, s.context)?;
        let value = f64::from(loss.to_scalar::<f32>()?);
        if !value.is_finite() {
            return Err(invalid(format!("nonfinite loss at step {step}")));
        }
        let grad_norm = arm.update(&loss, lr)?;
        step += 1;
        step_seconds.push(clock.elapsed().as_secs_f64());
        window_loss += value;
        window_steps += 1;
        if step % s.eval_every == 0 || step == s.steps {
            let in_class = evaluate(&*arm, &curve_in_class, s.context, &buckets, chunk)?;
            let held_out = evaluate(&*arm, &curve_held_out, s.context, &buckets, chunk)?;
            let train_loss = window_loss / window_steps.max(1) as f64;
            let line = format!(
                "step {step} lr {lr:.2e} train query NLL {train_loss:.4} grad {grad_norm:.3} in-class acc {:.3} {} | held-out acc {:.3} ({:.0}s)",
                in_class["accuracy"].as_f64().unwrap_or(f64::NAN),
                bucket_line(&in_class, &buckets),
                held_out["accuracy"].as_f64().unwrap_or(f64::NAN),
                started.elapsed().as_secs_f64()
            );
            eprintln!("{line}");
            writeln!(log, "{line}")?;
            curve.push(json!({
                "step": step, "lr": lr, "train_query_nll": train_loss,
                "grad_norm": grad_norm, "in_class": in_class, "held_out": held_out,
                "elapsed_seconds": started.elapsed().as_secs_f64(),
            }));
            window_loss = 0.0;
            window_steps = 0;
        }
        prober.run(&*arm, step, log)?;
    }
    if stopped_early {
        prober.run(&*arm, s.steps, log)?;
    }
    let train_seconds: f64 = step_seconds.iter().sum();
    let eval_clock = Instant::now();
    let in_class_set = sequences(
        s,
        &buckets,
        IN_CLASS_DOMAIN,
        s.final_sequences,
        Pairing::Train,
    )?;
    let in_class = evaluate(&*arm, &in_class_set, s.context, &buckets, chunk)?;
    let held_out_set = sequences(
        s,
        &buckets,
        HELD_OUT_DOMAIN,
        s.final_sequences / 2,
        Pairing::HeldOut,
    )?;
    let held_out = evaluate(&*arm, &held_out_set, s.context, &buckets, chunk)?;
    let line = format!(
        "final in-class fresh acc {:.4} {}; held-out-class acc {:.4} {}",
        in_class["accuracy"].as_f64().unwrap_or(f64::NAN),
        bucket_line(&in_class, &buckets),
        held_out["accuracy"].as_f64().unwrap_or(f64::NAN),
        bucket_line(&held_out, &buckets),
    );
    eprintln!("{line}");
    writeln!(log, "{line}")?;
    // D20 section 2 condition 3: two reachability counters on the scored
    // positions of the final in-class panel. Both are observation only.
    let reach_clock = Instant::now();
    let firing = read_firing(&*arm, &in_class_set, s.context, &buckets)?;
    let reachability = context_reachability(&*arm, &in_class_set, s.context)?;
    let reachability_seconds = reach_clock.elapsed().as_secs_f64();
    let reach_line = format!(
        "reachability: read rows {} nonzero {}; key-ablation changed rows {}/{} (argmax {}), max |dlogit| {:.6}",
        firing["rows_evaluated"].as_u64().unwrap_or(0),
        firing["rows_nonzero_mass_on_key"].as_u64().unwrap_or(0),
        reachability["rows_with_any_logit_change"]
            .as_u64()
            .unwrap_or(0),
        reachability["rows"].as_u64().unwrap_or(0),
        reachability["rows_whose_argmax_changed"]
            .as_u64()
            .unwrap_or(0),
        reachability["max_abs_logit_delta"].as_f64().unwrap_or(f64::NAN),
    );
    eprintln!("{reach_line}");
    writeln!(log, "{reach_line}")?;
    let mut sorted = step_seconds.clone();
    sorted.sort_by(f64::total_cmp);
    let mean_positions = (0..s.context)
        .map(|t| arm.positions_scored(t) as f64)
        .sum::<f64>()
        / s.context as f64;
    Ok(json!({
        "context_access": {
            "positions_scored_per_query_token_mean": in_class["positions_scored_per_query_token_mean"],
            "positions_scored_per_token_mean_over_window": mean_positions,
            "positions_scored_at_last_position": arm.positions_scored(s.context - 1),
            "counts": arm.access_note(),
        },
        "steps_completed": step,
        "stopped_early_at_max_seconds": stopped_early,
        "read_probe": prober.probes,
        "probe_seconds": prober.seconds,
        "final_in_class_fresh_pairings": in_class,
        "final_held_out_class_pairings": held_out,
        "read_firing": firing,
        "context_reachability": reachability,
        "reachability_seconds": reachability_seconds,
        "curve": curve,
        "train_seconds": train_seconds,
        "median_step_seconds": sorted.get(sorted.len() / 2).copied(),
        "final_eval_seconds": eval_clock.elapsed().as_secs_f64(),
        "wall_seconds": started.elapsed().as_secs_f64(),
        "supervised_queries_seen": step * s.batch * buckets.len() * s.pairs_per_bucket,
    }))
}

/// `save_model=true`: the arm's weights under `model/` of the report root,
/// or the refusal of an arm without a saved form (recorded, not fatal).
fn save_model(common: &Common, arm: &dyn ContextArm) -> Value {
    if !common.save_model {
        return json!({"requested": false});
    }
    match arm.save(&common.out.join("model")) {
        Ok(()) => json!({"requested": true, "saved": true, "path": "model"}),
        Err(error) => json!({"requested": true, "saved": false, "refusal": error.to_string()}),
    }
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

/// The report's fixed per-arm record schema.
const REPORT_SCHEMA: &str = "uor-r4/mqar-bench/arm-record/v1";

/// `mode=task-baselines`: the data-only baselines of the final evaluation
/// sets that the same task settings generate, with no arm and no training.
fn task_baselines_report(common: &Common) -> Result<Value> {
    let buckets = buckets_for(common.context);
    let in_class = sequences(
        common,
        &buckets,
        IN_CLASS_DOMAIN,
        common.final_sequences,
        Pairing::Train,
    )?;
    let held_out = sequences(
        common,
        &buckets,
        HELD_OUT_DOMAIN,
        common.final_sequences / 2,
        Pairing::HeldOut,
    )?;
    Ok(json!({
        "schema": REPORT_SCHEMA, "status": "complete", "mode": "task-baselines",
        "task": common.record(), "argv": std::env::args().collect::<Vec<_>>(),
        "final_in_class_fresh_pairings": task_baselines(&in_class, &buckets),
        "final_held_out_class_pairings": task_baselines(&held_out, &buckets),
    }))
}

fn main() -> Result<()> {
    let mut args = Args::parse()?;
    if args.0.get("mode").map(String::as_str) == Some("decide") {
        args.take("mode");
        return decide::main(args);
    }
    let (common, spec, mode, fact_task) = settings(args)?;
    if let Some(task) = fact_task {
        return fact::main(&common, &spec, matches!(mode, Mode::TaskBaselines), &task);
    }
    if matches!(mode, Mode::TaskBaselines) {
        report_output::claim(&common.out)?;
        let report = task_baselines_report(&common)?;
        write_json(&common.out.join("report.json"), &report)?;
        report_output::seal(&common.out)?;
        report_output::verify(&common.out)?;
        return Ok(());
    }
    // Claimed exclusively after argument validation, before any model work.
    report_output::claim(&common.out)?;
    let task = common.record();
    let argv = std::env::args().collect::<Vec<_>>();
    write_json(
        &common.out.join("config.json"),
        &json!({"schema": REPORT_SCHEMA, "task": task, "arm_label": spec.label(), "argv": argv}),
    )?;
    let mut log = fs::File::create(common.out.join("log.txt"))?;
    let result = (|| -> Result<(Value, Value)> {
        let device = uor_r4_training::baseline_protocol::device(&common.device_name)?;
        let mut arm = spec.build(&common, &device)?;
        let arm_record = json!({
            "label": spec.label(),
            "kind": arm.kind(),
            "parameters": arm.parameters(),
            "config": arm.record(),
            "parameter_match": spec.parameter_match(&common)?,
        });
        let mut results = run(&common, arm.as_mut(), &mut log)?;
        results["model_save"] = save_model(&common, arm.as_ref());
        Ok((arm_record, results))
    })();
    drop(log);
    let report = match &result {
        Ok((arm, results)) => json!({
            "schema": REPORT_SCHEMA, "status": "complete", "task": task, "argv": argv,
            "arm": arm, "results": results,
        }),
        Err(error) => json!({
            "schema": REPORT_SCHEMA, "status": "failed", "task": task, "argv": argv,
            "arm": {"label": spec.label()}, "error": error.to_string(),
        }),
    };
    write_json(&common.out.join("report.json"), &report)?;
    report_output::seal(&common.out)?;
    report_output::verify(&common.out)?;
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

    #[test]
    fn a_stack_arm_trains_scores_and_counts_its_context_access() {
        let common = Common {
            out: PathBuf::from("unused"),
            device_name: "cpu".into(),
            vocab: VOCAB,
            save_model: false,
            context: 64,
            pairs_per_bucket: 2,
            batch: 2,
            steps: 1,
            lr: 1e-3,
            warmup: 1,
            min_lr: 0.1,
            weight_decay: 0.1,
            clip: 1.0,
            eval_every: 1,
            curve_sequences: 1,
            final_sequences: 2,
            seed: 3,
            // `pointer` and `dump_scores` complete the literal; the test target
            // did not compile without them (test-only; `pointer` was already
            // missing on origin/main).
            pointer: None,
            max_seconds: 60.0,
            probe_steps: vec![0],
            probe_sequences: 2,
            dump_scores: false,
        };
        let spec = ArmSpec::Stack {
            pattern: "rar".into(),
            read: ReadScore::L2,
            rotation: true,
            width: 16,
            heads: 2,
            mlp_hidden: 16,
            age: AgeInit::Flat,
            lineage: LineageArm::F2,
        };
        spec.validate(&common).expect("valid arm");
        let mut arm = spec.build(&common, &Device::Cpu).expect("arm");
        assert_eq!(arm.positions_scored(0), 2);
        assert_eq!(arm.positions_scored(63), 2 * 64);
        let buckets = buckets_for(common.context);
        let batch = sequences(&common, &buckets, TRAIN_DOMAIN, 2, Pairing::Train).expect("batch");
        let (ids, targets, weights) = batch_arrays(&batch, common.context);
        let loss = arm
            .loss(&ids, &targets, &weights, 2, common.context)
            .expect("loss");
        assert!(arm.update(&loss, 1e-3).expect("update").is_finite());
        let report = evaluate(&*arm, &batch, common.context, &buckets, 2).expect("evaluate");
        assert_eq!(report["total"], 2 * 2 * buckets.len());
        assert_eq!(arm.record()["age_init"], "flat");
        assert_eq!(arm.record()["read_key_shift"], true);
        assert_eq!(arm.record()["lineage"], "f2");
        let probe = read_probe(&*arm, &batch, common.context, &buckets).expect("probe");
        for bucket in &buckets {
            let entry = &probe["by_bucket"][bucket.name];
            assert_eq!(entry["heads"].as_array().map(Vec::len), Some(2));
            let key = entry["best_head_mean_weight_on_key"]
                .as_f64()
                .expect("mass");
            assert!((0.0..=1.0).contains(&key));
        }
    }

    #[test]
    fn the_recency_baseline_reads_the_latest_value_before_the_query() {
        for sequence in draw(TRAIN_DOMAIN, 16, Pairing::Train, 512) {
            for query in &sequence.queries {
                let expected = (0..query.position)
                    .rev()
                    .find(|&p| (VALUES.0..VALUES.1).contains(&sequence.tokens[p]))
                    .map(|p| sequence.tokens[p]);
                assert_eq!(most_recent_value(&sequence, query), expected);
            }
        }
    }

    fn small(pattern: &str, rotation: bool) -> StackConfig {
        StackConfig {
            arch: StackArch::Geometric,
            vocab_size: VOCAB,
            width: 16,
            heads: 2,
            mlp_hidden: 16,
            context: 12,
            pattern: pattern.into(),
            read: ReadScore::L2,
            rotation,
            rotation_group: RotationGroup::Quaternion,
            seed: 5,
            memory: None,
            select: None,
            pointer: None,
        }
    }

    fn last_logits(model: &StackModel, ids: &[u32]) -> Vec<Vec<f32>> {
        model
            .forward(ids, 1, ids.len())
            .and_then(|t| Ok(t.to_vec2::<f32>()?))
            .expect("logits")
    }

    #[test]
    fn the_j_turn_is_the_exact_quaternion_product() {
        let x = Tensor::from_vec(
            vec![1f32, 2., 3., 4., -5., 6., 7., -8.],
            (1, 1, 8),
            &Device::Cpu,
        )
        .expect("tensor");
        let turned = quaternion_j_left(&x).expect("turn");
        let values = turned
            .flatten_all()
            .and_then(|t| t.to_vec1::<f32>())
            .expect("values");
        // j (a + bi + cj + dk) = -c + d i + a j - b k.
        assert_eq!(values, vec![-3., 4., 1., -2., -7., -8., -5., -6.]);
        let mut four = x.clone();
        for _ in 0..4 {
            four = quaternion_j_left(&four).expect("turn");
        }
        assert_eq!(
            four.flatten_all()
                .and_then(|t| t.to_vec1::<f32>())
                .expect("values"),
            x.flatten_all()
                .and_then(|t| t.to_vec1::<f32>())
                .expect("values")
        );
    }

    #[test]
    fn the_key_shift_is_causal_reads_the_previous_token_and_survives_save() {
        let mut model = StackModel::new(small("aa", true), &Device::Cpu).expect("model");
        let ids: Vec<u32> = (0..12).map(|i| 64 + 7 * i).collect();
        let plain = last_logits(&model, &ids);
        model.set_read_key_shift(true).expect("shift");
        let shifted = last_logits(&model, &ids);
        assert_ne!(plain[11], shifted[11], "the shift changes the read");
        // Causal: changing the last token leaves every earlier row unchanged.
        let mut changed = ids.clone();
        changed[11] = 300;
        let after = last_logits(&model, &changed);
        for t in 0..11 {
            assert_eq!(after[t], shifted[t]);
        }
        // Position 0 has no predecessor: its key is unchanged by the shift.
        assert_eq!(plain[0], shifted[0]);
        let dir = std::env::temp_dir().join(format!("mqar-key-shift-{}", std::process::id()));
        model.save(&dir).expect("save");
        let loaded = StackModel::load(&dir, &Device::Cpu).expect("load");
        assert!(loaded.read_key_shift());
        assert_eq!(last_logits(&loaded, &changed), after);
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn rotation_has_no_effect_on_an_all_read_stack() {
        // F3 finding: `rotation` parameterizes only the recurrence's transport.
        let on = small("aaa", true);
        let off = small("aaa", false);
        assert_eq!(on.shapes(), off.shapes());
        let a = StackModel::new(on, &Device::Cpu).expect("model");
        let b = StackModel::new(off, &Device::Cpu).expect("model");
        let ids: Vec<u32> = (0..12).map(|i| 64 + 5 * i).collect();
        assert_eq!(last_logits(&a, &ids), last_logits(&b, &ids));
        // ...while a recurrence layer does take rotation rows.
        assert_ne!(small("ra", true).shapes(), small("ra", false).shapes());
    }

    fn common_for(out: &str, context: usize) -> Common {
        Common {
            out: PathBuf::from(out),
            device_name: "cpu".into(),
            vocab: VOCAB,
            save_model: false,
            context,
            pairs_per_bucket: 8,
            batch: 8,
            steps: 1,
            lr: 1e-3,
            warmup: 1,
            min_lr: 0.1,
            weight_decay: 0.1,
            clip: 1.0,
            eval_every: 1,
            curve_sequences: 1,
            final_sequences: 1,
            seed: 5,
            pointer: None,
            max_seconds: 1.0,
            probe_steps: Vec::new(),
            probe_sequences: 1,
            dump_scores: false,
        }
    }

    /// The matched control's construction: the ordinary attention stack
    /// (`StackArch::Transformer`, attention layers only) whose parameter count
    /// is the geometric reference arm's, to within one MLP column.
    #[test]
    fn the_transformer_control_is_parameter_matched_to_the_geometric_arm() {
        let common = common_for("/tmp/unused-d20-control-match", 512);
        let spec = ArmSpec::Transformer {
            layers: 6,
            width: 128,
            heads: 4,
            mlp_hidden: None,
            reference_pattern: "rrarra".into(),
            reference_read: ReadScore::L2,
            reference_rotation: true,
        };
        spec.validate(&common).expect("valid control");
        let control = spec.stack_config(&common).expect("control config");
        assert_eq!(control.arch, StackArch::Transformer);
        assert_eq!(control.pattern, "aaaaaa");
        let reference =
            reference_config(&common, 128, 4, "rrarra", ReadScore::L2, true).expect("reference");
        let target = reference.parameter_count().expect("reference count");
        let got = control.parameter_count().expect("control count");
        assert_eq!(target, 1_370_008, "the frozen recipe's geometric arm");
        // The closest MLP width lands within one column of the target.
        assert!(
            (got as i64 - target as i64).unsigned_abs() <= (3 * control.width) as u64,
            "control {got} vs reference {target}"
        );
        let match_record = spec.parameter_match(&common).expect("match record");
        assert_eq!(match_record["reference_geometric"]["parameters"], target);
        assert_eq!(match_record["control"]["parameters"], got);
        // A geometric arm reports no match block: nothing about it changed.
        let stack = ArmSpec::Stack {
            pattern: "rrarra".into(),
            read: ReadScore::L2,
            rotation: true,
            width: 128,
            heads: 4,
            mlp_hidden: 384,
            age: AgeInit::Default,
            lineage: LineageArm::None,
        };
        assert!(stack.parameter_match(&common).expect("null").is_null());
        assert_eq!(
            stack
                .stack_config(&common)
                .expect("stack config")
                .parameter_count()
                .expect("count"),
            1_370_008
        );
    }

    /// The control runs, is causal, and reads the context: the reachability
    /// counter must fire for it as well as for the geometric arm.
    #[test]
    fn both_arms_read_the_context_at_the_scored_positions() {
        let common = common_for("/tmp/unused-d20-control-reach", 64);
        let buckets = buckets_for(64);
        let sequences =
            sequences(&common, &buckets, IN_CLASS_DOMAIN, 3, Pairing::Train).expect("panel");
        for (arch, pattern) in [
            (StackArch::Geometric, "rarra"),
            (StackArch::Transformer, "aaaaa"),
        ] {
            let mut config = small(pattern, arch == StackArch::Geometric);
            config.arch = arch;
            config.mlp_hidden = 32;
            config.context = 64;
            config.read = if arch == StackArch::Geometric {
                ReadScore::L2
            } else {
                ReadScore::Dot
            };
            let model = StackModel::new(config, &Device::Cpu).expect("model");
            let optimizer = StackAdamW::new(&model, 0.1, 1.0).expect("optimizer");
            let arm = StackArm {
                model,
                optimizer,
                age: AgeInit::Default,
                lineage: LineageArm::None,
            };
            let reach = context_reachability(&arm, &sequences, 64).expect("reachability");
            let queries = sequences
                .iter()
                .map(|sequence| sequence.queries.len())
                .sum::<usize>();
            assert_eq!(queries, 3 * buckets.len() * 8);
            assert_eq!(reach["rows"], queries);
            assert_eq!(
                reach["rows_with_any_logit_change"], queries,
                "{arch:?}: every scored position must depend on its source key"
            );
            assert!(
                reach["max_abs_logit_delta"].as_f64().unwrap_or(0.0) > 0.0,
                "{arch:?}: an unwired arm would leave the logits identical"
            );
            // Only the geometric arm exposes the read-binding counter.
            let firing = read_firing(&arm, &sequences, 64, &buckets).expect("firing");
            if arch == StackArch::Geometric {
                assert_eq!(firing["status"], "MEASURED");
                // One held-out query per (sequence, bucket), one batch item each.
                assert_eq!(firing["batch_items"], 3 * buckets.len());
                assert_eq!(
                    firing["rows_evaluated"],
                    arm.read_heads().len() * 3 * buckets.len()
                );
            } else {
                assert!(arm.read_heads().is_empty());
                assert!(firing["rows_evaluated"].is_null());
            }
        }
    }
}
