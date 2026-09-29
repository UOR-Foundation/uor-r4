//! Prime-route addressing for the relation store: an evaluation instrument
//! for the memory port's address (ROADMAP §2b), after the prime router of
//! ADR-0003 (`docs/adr/0003-fixed-zeta-prime-route-attention.md`).
//!
//! G v1 ([`crate::stack_aerm`]) keys a fact by the ordered, typed pair
//! (entity token, relation token). A tag head that swaps the two roles on an
//! unseen phrasing reads and writes a different key. The prime router keys
//! data by prime atoms: every token has a registered prime, and a transition
//! expert is the square-free semiprime of two atoms. Here the store is keyed
//! by the semiprime expert of the clause's two key atoms (the last two
//! distinct tokens tagged entity or relation), so the key is the same
//! whichever role the head gave each atom: by unique factorization, `p_a p_b`
//! identifies the unordered pair `{a, b}` exactly. The value is routed
//! through the expert, not multiplied into it, so a relation whose values are
//! names keeps its direction: "Alex's friend is Sam" does not answer "Sam's
//! friend".
//!
//! The read policy is G v1's: whenever a key tag completes an expert, both
//! registers (current and previous distinct value) read, and reads never
//! close the clause. Two write policies:
//! - [`WritePolicy::Trigger`]: the model's learned write trigger, as G v1;
//! - [`WritePolicy::TurnEnd`]: a write where the user's turn ends, if the
//!   clause then has an expert and a value atom. A question carries no value
//!   atom and never writes. A session knows where a user turn ends, so this
//!   uses no gold label; the learned trigger is ignored.
//!
//! The store's version semantics are the probe's ([`crate::stack_aerm::RelationStore`]):
//! a different value becomes current and keeps the previous distinct value; a
//! same-value write increments the version and keeps it.
//!
//! Evaluation only: nothing here trains or serves, and the store has no
//! capacity bound (D2's episodes never evict).

use std::collections::BTreeMap;

use candle_core::{Tensor, D};
use serde::{Deserialize, Serialize};

use crate::stack_aerm::{
    model_registers, AermModel, DialogueBatch, QueryClass, Registers, RelationWorld, STATUS_ABSENT,
    STATUS_HIT, STATUS_NONE, TAG_ENTITY, TAG_OTHER, TAG_RELATION, TAG_VALUE, TRIGGERS,
    TRIGGER_WRITE,
};
use crate::stack_tracking::Rng;
use crate::{invalid, Result};

/// Token id `t` is the atom of the `(t + 1)`-th prime.
#[derive(Clone, Debug)]
pub struct PrimeRegistry {
    primes: Vec<u64>,
}

impl PrimeRegistry {
    /// The first `atoms` primes, by a sieve bounded with Rosser's theorem:
    /// the `n`-th prime is below `n (ln n + ln ln n)` for `n >= 6`.
    pub fn new(atoms: usize) -> Result<Self> {
        if atoms == 0 {
            return Err(invalid("the registry needs at least one atom"));
        }
        let n = atoms as f64;
        let bound = if atoms < 6 {
            13
        } else {
            (n * (n.ln() + n.ln().ln())).ceil() as usize
        };
        let mut composite = vec![false; bound + 1];
        let mut primes = Vec::with_capacity(atoms);
        for i in 2..=bound {
            if composite[i] {
                continue;
            }
            primes.push(i as u64);
            if primes.len() == atoms {
                break;
            }
            let mut multiple = i * i;
            while multiple <= bound {
                composite[multiple] = true;
                multiple += i;
            }
        }
        if primes.len() < atoms {
            return Err(invalid("the sieve bound produced too few primes"));
        }
        Ok(Self { primes })
    }

    /// The prime registered for `token`.
    pub fn prime(&self, token: u32) -> Result<u64> {
        self.primes
            .get(token as usize)
            .copied()
            .ok_or_else(|| invalid(format!("token {token} is outside the prime registry")))
    }

    pub fn len(&self) -> usize {
        self.primes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.primes.is_empty()
    }

    /// The semiprime expert of the last two distinct atoms of `keys`, or
    /// `None` when there are fewer than two.
    pub fn expert(&self, keys: &[u32]) -> Result<Option<u64>> {
        let mut atoms = keys.iter().rev();
        let Some(&last) = atoms.next() else {
            return Ok(None);
        };
        let Some(&other) = atoms.find(|&&atom| atom != last) else {
            return Ok(None);
        };
        self.prime(last)?
            .checked_mul(self.prime(other)?)
            .map(Some)
            .ok_or_else(|| invalid("a semiprime expert overflowed u64"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExpertRecord {
    value: u32,
    previous: Option<u32>,
    version: u32,
}

/// An exact store keyed by semiprime experts, with the probe's version
/// semantics and no capacity bound.
#[derive(Clone, Debug, Default)]
pub struct ExpertStore {
    records: BTreeMap<u64, ExpertRecord>,
}

impl ExpertStore {
    /// A different value becomes current and keeps the previous distinct
    /// value; a same-value write increments the version and keeps it.
    pub fn write(&mut self, expert: u64, value: u32) {
        self.records
            .entry(expert)
            .and_modify(|record| {
                if record.value != value {
                    record.previous = Some(record.value);
                    record.value = value;
                }
                record.version += 1;
            })
            .or_insert(ExpertRecord {
                value,
                previous: None,
                version: 1,
            });
    }

    /// `(status, value)`: the current value, or the previous distinct value
    /// when `previous`; Absent when there is none.
    pub fn read(&self, expert: u64, previous: bool) -> (u32, u32) {
        match self.records.get(&expert) {
            Some(record) => match (previous, record.previous) {
                (false, _) => (STATUS_HIT, record.value),
                (true, Some(value)) => (STATUS_HIT, value),
                (true, None) => (STATUS_ABSENT, 0),
            },
            None => (STATUS_ABSENT, 0),
        }
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// When the prime-route store writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WritePolicy {
    /// The model's learned write trigger, which also closes the clause (G v1).
    Trigger,
    /// The end of each user turn, which also closes the clause.
    TurnEnd,
}

/// Registers over one sequence from its per-token tags and triggers, keyed
/// by semiprime experts. `turn_end[t]` marks the last position before an
/// assistant reply starts; `eos` also closes the open clause.
pub fn simulate_prime_route(
    tokens: &[u32],
    tags: &[u32],
    triggers: &[u32],
    turn_end: &[bool],
    eos: u32,
    registry: &PrimeRegistry,
    policy: WritePolicy,
) -> Result<Registers> {
    let n = tokens.len();
    if tags.len() != n || triggers.len() != n || turn_end.len() != n {
        return Err(invalid("one tag, trigger and turn mark per token"));
    }
    let mut store = ExpertStore::default();
    let mut keys: Vec<u32> = Vec::new();
    let mut value: Option<u32> = None;
    let mut register = (STATUS_NONE, 0u32);
    let mut previous = (STATUS_NONE, 0u32);
    let mut out = Registers {
        status: Vec::with_capacity(n),
        value: Vec::with_capacity(n),
        status_previous: Vec::with_capacity(n),
        value_previous: Vec::with_capacity(n),
        read_events: 0,
    };
    for position in 0..n {
        let (token, tag, trigger) = (tokens[position], tags[position], triggers[position]);
        if trigger as usize >= TRIGGERS {
            return Err(invalid("trigger outside the trigger set"));
        }
        match tag {
            TAG_OTHER => {}
            TAG_ENTITY | TAG_RELATION => keys.push(token),
            TAG_VALUE => value = Some(token),
            _ => return Err(invalid("tag outside the tag set")),
        }
        if matches!(tag, TAG_ENTITY | TAG_RELATION) {
            if let Some(expert) = registry.expert(&keys)? {
                register = store.read(expert, false);
                previous = store.read(expert, true);
                out.read_events += 2;
            }
        }
        let mut close = token == eos;
        let write_here = match policy {
            WritePolicy::Trigger => trigger == TRIGGER_WRITE,
            WritePolicy::TurnEnd => turn_end[position],
        };
        if write_here {
            if let (Some(expert), Some(value)) = (registry.expert(&keys)?, value) {
                store.write(expert, value);
            }
            close = true;
        }
        if close {
            keys.clear();
            value = None;
        }
        out.status.push(register.0);
        out.value.push(register.1);
        out.status_previous.push(previous.0);
        out.value_previous.push(previous.1);
    }
    Ok(out)
}

/// `turn_end` marks for one padded row: position `t` precedes the first token
/// of an assistant reply. Padding is never a turn end.
fn turn_ends(response_mask: &[u8], time: usize) -> Vec<bool> {
    (0..time)
        .map(|t| t + 1 < response_mask.len() && response_mask[t] == 0 && response_mask[t + 1] == 1)
        .collect()
}

/// Prime-route registers for every row of a padded batch, from `tags` and
/// `triggers` (row-major, one per input position).
pub fn prime_route_registers(
    data: &DialogueBatch,
    tags: &[u32],
    triggers: &[u32],
    eos: u32,
    registry: &PrimeRegistry,
    policy: WritePolicy,
) -> Result<Registers> {
    let (batch, time) = (data.batch, data.time);
    if tags.len() != batch * time || triggers.len() != batch * time {
        return Err(invalid("one tag and trigger per batch position"));
    }
    let mut out = Registers::default();
    for (b, episode) in data.episodes.iter().enumerate() {
        let range = b * time..(b + 1) * time;
        let row = simulate_prime_route(
            &data.ids[range.clone()],
            &tags[range.clone()],
            &triggers[range],
            &turn_ends(&episode.response_mask, time),
            eos,
            registry,
            policy,
        )?;
        out.read_events += row.read_events;
        out.status.extend(row.status);
        out.value.extend(row.value);
        out.status_previous.extend(row.status_previous);
        out.value_previous.extend(row.value_previous);
    }
    Ok(out)
}

/// The register sources compared on the same episodes and the same model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegisterArm {
    /// G v1: typed `(entity, relation)` keys, the learned write trigger.
    GV1,
    /// Semiprime experts, the learned write trigger.
    ExpertTrigger,
    /// Semiprime experts, a write at each user turn's end.
    ExpertTurnEnd,
    /// Gold registers: a perfect parser (the probe's oracle).
    Gold,
}

impl RegisterArm {
    pub const ALL: [RegisterArm; 4] = [
        RegisterArm::GV1,
        RegisterArm::ExpertTrigger,
        RegisterArm::ExpertTurnEnd,
        RegisterArm::Gold,
    ];
}

/// One arm's score on one query class.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmClassScore {
    pub queries: usize,
    /// Value classes: the value slot; abstaining classes: the whole answer
    /// (the probe's definition).
    pub correct: usize,
    /// The arm's registers at the answer's check row equal the gold registers.
    pub registers_equal_gold: usize,
}

/// One arm's scores on every query class.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmEvaluation {
    pub classes: BTreeMap<String, ArmClassScore>,
    pub read_events: usize,
}

/// All arms on the same episodes: the probe's dialogue batches for `seed`,
/// generated in `batch`-sized chunks exactly as
/// [`crate::stack_aerm::evaluate_dialogues`] generates them, so the G v1 arm
/// reproduces that function's per-class `correct` counts.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PrimeRouteEvaluation {
    pub episodes: usize,
    pub held: bool,
    pub seed: u64,
    /// The model's tag accuracy on real positions.
    pub tag_accuracy: f64,
    pub arms: BTreeMap<String, ArmEvaluation>,
}

fn argmax_rows(logits: &Tensor) -> Result<Vec<u32>> {
    Ok(logits.detach().argmax(D::Minus1)?.to_vec1::<u32>()?)
}

/// Scores every [`RegisterArm`] with `model` on `episodes` fresh dialogues.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_prime_route(
    model: &AermModel,
    world: &RelationWorld,
    registry: &PrimeRegistry,
    episodes: usize,
    seed: u64,
    held: bool,
    context: usize,
    batch: usize,
) -> Result<PrimeRouteEvaluation> {
    if !model.has_memory() {
        return Err(invalid("the prime-route arms need the memory arm"));
    }
    if batch == 0 {
        return Err(invalid("the batch must be positive"));
    }
    let mut rng = Rng::new(seed);
    let mut result = PrimeRouteEvaluation {
        episodes,
        held,
        seed,
        ..Default::default()
    };
    let (mut tags_right, mut tags_total) = (0usize, 0usize);
    let mut remaining = episodes;
    while remaining > 0 {
        let size = remaining.min(batch);
        remaining -= size;
        let data = world.batch(&mut rng, size, context, held)?;
        let bottom = model.bottom(&data.ids, size, context)?;
        let tags = argmax_rows(&bottom.tags)?;
        let triggers = argmax_rows(&bottom.triggers)?;
        for row in 0..size * context {
            if data.real[row] > 0.0 {
                tags_total += 1;
                tags_right += usize::from(tags[row] == data.tags[row]);
            }
        }
        for arm in RegisterArm::ALL {
            let registers = match arm {
                RegisterArm::GV1 => {
                    model_registers(&data.ids, &tags, &triggers, size, context, world.eos)?
                }
                RegisterArm::ExpertTrigger => prime_route_registers(
                    &data,
                    &tags,
                    &triggers,
                    world.eos,
                    registry,
                    WritePolicy::Trigger,
                )?,
                RegisterArm::ExpertTurnEnd => prime_route_registers(
                    &data,
                    &tags,
                    &triggers,
                    world.eos,
                    registry,
                    WritePolicy::TurnEnd,
                )?,
                RegisterArm::Gold => Registers {
                    status: data.status.clone(),
                    value: data.value.clone(),
                    status_previous: data.status_previous.clone(),
                    value_previous: data.value_previous.clone(),
                    read_events: 0,
                },
            };
            let predicted = argmax_rows(&model.top(
                &bottom.hidden,
                &registers.status,
                &registers.value,
                &registers.status_previous,
                &registers.value_previous,
            )?)?;
            let evaluation = result.arms.entry(format!("{arm:?}")).or_default();
            evaluation.read_events += registers.read_events;
            for (b, episode) in data.episodes.iter().enumerate() {
                let base = b * context;
                for query in &episode.queries {
                    let rows = (query.answer_start - 1)..query.answer_end;
                    let slot_row = query.value_position.map(|p| p - 1);
                    let correct = match (query.class.abstains(), slot_row) {
                        (false, Some(r)) => predicted[base + r] == data.targets[base + r],
                        _ => rows
                            .clone()
                            .all(|r| predicted[base + r] == data.targets[base + r]),
                    };
                    let check = base + slot_row.unwrap_or(query.answer_start - 1);
                    let equal = registers.status[check] == data.status[check]
                        && registers.value[check] == data.value[check]
                        && registers.status_previous[check] == data.status_previous[check]
                        && registers.value_previous[check] == data.value_previous[check];
                    let score = evaluation
                        .classes
                        .entry(format!("{:?}", query.class))
                        .or_default();
                    score.queries += 1;
                    score.correct += usize::from(correct);
                    score.registers_equal_gold += usize::from(equal);
                }
            }
        }
    }
    result.tag_accuracy = tags_right as f64 / tags_total.max(1) as f64;
    Ok(result)
}

/// Accuracy of `class` under `arm`, if any query of that class was scored.
pub fn arm_accuracy(
    evaluation: &PrimeRouteEvaluation,
    arm: RegisterArm,
    class: QueryClass,
) -> Option<f64> {
    let score = evaluation
        .arms
        .get(&format!("{arm:?}"))?
        .classes
        .get(&format!("{class:?}"))?;
    (score.queries > 0).then(|| score.correct as f64 / score.queries as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack_aerm::{simulate, TRIGGER_NONE};
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// The probe's toy encoder: pretokens (a word with its leading space, an
    /// apostrophe-suffix or one punctuation mark) each get their own id.
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
                .map(|p| {
                    let next = vocab.len() as u32 + 3;
                    *vocab.entry(p).or_insert(next)
                })
                .collect()
        }
    }

    fn world() -> Result<RelationWorld> {
        RelationWorld::new(&toy_encoder(), 0, 1)
    }

    #[test]
    fn the_registry_holds_the_first_primes_and_experts_are_unordered() -> Result<()> {
        let registry = PrimeRegistry::new(4096)?;
        assert_eq!(registry.len(), 4096);
        assert_eq!(
            (0..10)
                .map(|t| registry.prime(t))
                .collect::<Result<Vec<_>>>()?,
            vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]
        );
        // The 4,096th prime.
        assert_eq!(registry.prime(4095)?, 38_873);
        assert!(registry.prime(4096).is_err());
        let ab = registry.expert(&[7, 11])?;
        let ba = registry.expert(&[11, 7])?;
        assert_eq!(ab, Some(19 * 37));
        assert_eq!(ab, ba);
        // The last two distinct atoms; a repeat is not a second atom.
        assert_eq!(registry.expert(&[3, 7, 7])?, Some(7 * 19));
        assert_eq!(registry.expert(&[7, 7])?, None);
        assert_eq!(registry.expert(&[])?, None);
        Ok(())
    }

    #[test]
    fn the_expert_store_keeps_the_probe_version_semantics() {
        let mut store = ExpertStore::default();
        assert_eq!(store.read(6, false), (STATUS_ABSENT, 0));
        store.write(6, 40);
        assert_eq!(store.read(6, false), (STATUS_HIT, 40));
        assert_eq!(store.read(6, true), (STATUS_ABSENT, 0));
        store.write(6, 41);
        assert_eq!(store.read(6, false), (STATUS_HIT, 41));
        assert_eq!(store.read(6, true), (STATUS_HIT, 40));
        // A same-value write keeps the previous distinct value.
        store.write(6, 41);
        assert_eq!(store.read(6, true), (STATUS_HIT, 40));
        assert_eq!(store.len(), 1);
    }

    /// With gold tags, the expert arms reproduce the gold registers at every
    /// input position, for the trigger policy and for the turn-end policy
    /// with every learned trigger removed.
    #[test]
    fn gold_tags_reproduce_the_gold_registers() -> Result<()> {
        let world = world()?;
        let registry = PrimeRegistry::new(4096)?;
        for held in [false, true] {
            let mut rng = Rng::new(if held { 91 } else { 90 });
            let data = world.batch(&mut rng, 24, 256, held)?;
            let silent = vec![TRIGGER_NONE; data.triggers.len()];
            for (policy, triggers) in [
                (WritePolicy::Trigger, &data.triggers),
                (WritePolicy::TurnEnd, &silent),
            ] {
                let registers = prime_route_registers(
                    &data, &data.tags, triggers, world.eos, &registry, policy,
                )?;
                for row in 0..data.batch * data.time {
                    if data.real[row] == 0.0 {
                        continue;
                    }
                    assert_eq!(
                        (
                            registers.status[row],
                            registers.value[row],
                            registers.status_previous[row],
                            registers.value_previous[row]
                        ),
                        (
                            data.status[row],
                            data.value[row],
                            data.status_previous[row],
                            data.value_previous[row]
                        ),
                        "{policy:?} held={held}: row {row}"
                    );
                }
            }
        }
        Ok(())
    }

    /// Swapping the entity and relation tags of every query turn (a turn with
    /// no write) makes a query's roles disagree with its fact's. That breaks
    /// G v1's typed keys but leaves the semiprime experts, and so the
    /// registers, unchanged. (A swap applied to every turn alike would keep
    /// the typed keys consistent.)
    #[test]
    fn swapped_query_roles_break_typed_keys_but_not_experts() -> Result<()> {
        let world = world()?;
        let registry = PrimeRegistry::new(4096)?;
        let mut rng = Rng::new(92);
        let data = world.batch(&mut rng, 24, 256, true)?;
        let mut swapped = data.tags.clone();
        for b in 0..data.batch {
            let row = b * data.time..(b + 1) * data.time;
            let mut turn_start = row.start;
            for position in row.clone() {
                if data.ids[position] != world.eos {
                    continue;
                }
                let turn = turn_start..=position;
                if !turn.clone().any(|p| data.triggers[p] == TRIGGER_WRITE) {
                    for p in turn {
                        swapped[p] = match data.tags[p] {
                            TAG_ENTITY => TAG_RELATION,
                            TAG_RELATION => TAG_ENTITY,
                            other => other,
                        };
                    }
                }
                turn_start = position + 1;
            }
        }
        let experts = prime_route_registers(
            &data,
            &swapped,
            &data.triggers,
            world.eos,
            &registry,
            WritePolicy::Trigger,
        )?;
        let typed = model_registers(
            &data.ids,
            &swapped,
            &data.triggers,
            data.batch,
            data.time,
            world.eos,
        )?;
        let (mut hits, mut typed_differs) = (0usize, 0usize);
        for row in 0..data.batch * data.time {
            if data.real[row] == 0.0 {
                continue;
            }
            assert_eq!(experts.status[row], data.status[row], "row {row}");
            assert_eq!(experts.value[row], data.value[row], "row {row}");
            hits += usize::from(data.status[row] == STATUS_HIT);
            typed_differs += usize::from(
                typed.status[row] != data.status[row] || typed.value[row] != data.value[row],
            );
        }
        assert!(hits > 0, "the batch must contain hits");
        assert!(typed_differs > 0, "swapped roles must break the typed keys");
        Ok(())
    }

    /// The value is routed through the expert, not multiplied into it: a
    /// relation whose value is a name keeps its direction.
    #[test]
    fn a_name_valued_relation_keeps_its_direction() -> Result<()> {
        let registry = PrimeRegistry::new(64)?;
        let (alex, friend, sam, eos) = (10u32, 11u32, 12u32, 2u32);
        // "Alex's friend is Sam." then "Who is Sam's friend?"
        let tokens = [alex, friend, sam, eos, sam, friend, eos];
        let tags = [
            TAG_ENTITY,
            TAG_RELATION,
            TAG_VALUE,
            TAG_OTHER,
            TAG_ENTITY,
            TAG_RELATION,
            TAG_OTHER,
        ];
        let triggers = [
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_WRITE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
        ];
        let turn_end = [false; 7];
        let registers = simulate_prime_route(
            &tokens,
            &tags,
            &triggers,
            &turn_end,
            eos,
            &registry,
            WritePolicy::Trigger,
        )?;
        assert_eq!(
            (registers.status[5], registers.value[5]),
            (STATUS_ABSENT, 0)
        );
        // The same query about Alex finds Sam.
        let tokens = [alex, friend, sam, eos, friend, alex, eos];
        let tags = [
            TAG_ENTITY,
            TAG_RELATION,
            TAG_VALUE,
            TAG_OTHER,
            TAG_RELATION,
            TAG_ENTITY,
            TAG_OTHER,
        ];
        let registers = simulate_prime_route(
            &tokens,
            &tags,
            &triggers,
            &turn_end,
            eos,
            &registry,
            WritePolicy::Trigger,
        )?;
        assert_eq!((registers.status[5], registers.value[5]), (STATUS_HIT, sam));
        // G v1 agrees on both when the tags are right.
        let typed = simulate(&tokens, &tags, &triggers, eos)?;
        assert_eq!((typed.status[5], typed.value[5]), (STATUS_HIT, sam));
        Ok(())
    }

    #[test]
    fn a_question_never_writes_at_its_turn_end() -> Result<()> {
        let registry = PrimeRegistry::new(64)?;
        let (alex, color, blue, eos) = (10u32, 11u32, 12u32, 2u32);
        // A question with no value atom, then the same question again.
        let tokens = [alex, color, eos, alex, color, eos];
        let tags = [
            TAG_ENTITY,
            TAG_RELATION,
            TAG_OTHER,
            TAG_ENTITY,
            TAG_RELATION,
            TAG_OTHER,
        ];
        let triggers = [TRIGGER_NONE; 6];
        let turn_end = [false, true, false, false, true, false];
        let registers = simulate_prime_route(
            &tokens,
            &tags,
            &triggers,
            &turn_end,
            eos,
            &registry,
            WritePolicy::TurnEnd,
        )?;
        assert_eq!(registers.status[4], STATUS_ABSENT);
        // An assertion writes at its turn end and the next question reads it.
        let tokens = [alex, color, blue, eos, alex, color, eos];
        let tags = [
            TAG_ENTITY,
            TAG_RELATION,
            TAG_VALUE,
            TAG_OTHER,
            TAG_ENTITY,
            TAG_RELATION,
            TAG_OTHER,
        ];
        let triggers = [TRIGGER_NONE; 7];
        let turn_end = [false, false, true, false, false, true, false];
        let registers = simulate_prime_route(
            &tokens,
            &tags,
            &triggers,
            &turn_end,
            eos,
            &registry,
            WritePolicy::TurnEnd,
        )?;
        assert_eq!(
            (registers.status[5], registers.value[5]),
            (STATUS_HIT, blue)
        );
        Ok(())
    }
}
