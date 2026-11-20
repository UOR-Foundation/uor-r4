//! Conjunction memory (PLAN row 39, step 2): one row per token pair, addressed
//! by a prime product, holding a few pointer tokens with counts. A row changes
//! which tokens it points at only when the output was wrong.
//!
//! 1. **Address.** Unordered ([`Order::Unordered`]) is the rule of
//!    `uor-r4-integer`'s `fbs_pair_rows`: token `t` stands for the `t`-th prime
//!    and the pair of different tokens `a`, `b` has the address
//!    `primes[a] * primes[b]`, under 2^31. Ordered ([`Order::Ordered`]) gives
//!    the nearer token its own primes: `primes[a] * primes[vocabulary + b]`,
//!    under 2^32, so `a b` and `b a` have different rows and equal tokens have
//!    one. Unique factorisation keeps both free of collisions.
//! 2. **Pair of a target.** The two tokens before it, when the nearer one is
//!    not BOS (a window never crosses a record).
//! 3. **Read.** The output is the row's pointer with the largest count (the
//!    earliest slot on a tie) when the pair has one, else the model's output.
//! 4. **Write.** With no row: nothing when the model's output was right, else a
//!    new row pointing at the target. With a row: a slot that points at the
//!    target gains one count; otherwise the target takes an empty slot, and
//!    with none empty every slot loses one count and the target takes a slot
//!    that reached zero.
//!
//! 5. **Readout from the pick** (PLAN row 39, step 3). The copy read
//!    ([`copy_at`]) is the Pick read at distance zero: the token after the
//!    latest earlier place of the same pair in the window. Where it and the
//!    row output name different tokens, the row's two trust counts decide;
//!    each count is one side's wins on targets the row had not yet seen
//!    ([`Conjunction::judge`], [`Row::answer`]).
//!
//! Every step is an integer compare, add or table lookup.
#![forbid(unsafe_code)]
use crate::{Error, Result};
use std::collections::HashMap;

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(format!("conjunction: {}", message.into()))
}

/// Most pointer slots a row holds.
pub const MAX_SLOTS: usize = 8;
pub const SCHEMA: &str = "uor-r4.discrete-conjunction-probe/3";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    Unordered,
    Ordered,
}

/// A row: `count[s] == 0` marks slot `s` empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Row {
    pub pointer: [u32; MAX_SLOTS],
    pub count: [u32; MAX_SLOTS],
    /// Times the copy read was right against the row output, and the row
    /// output against the copy read, counted only where the two differed.
    pub trust: [u32; 2],
}
impl Row {
    /// The readout that answers from the pick: where the copy read names a
    /// token other than the row output, the side with more wins on this row
    /// answers (the copy read on a tie); else the row output; else the copy.
    pub fn answer(&self, copy: Option<u32>) -> Option<u32> {
        match (self.output(), copy) {
            (Some(output), Some(copy)) if copy != output => {
                Some(if self.trust[0] >= self.trust[1] {
                    copy
                } else {
                    output
                })
            }
            (Some(output), _) => Some(output),
            (None, copy) => copy,
        }
    }
    /// The pointer with the largest count, the earliest slot on a tie.
    pub fn output(&self) -> Option<u32> {
        let mut best: Option<usize> = None;
        for slot in 0..MAX_SLOTS {
            if self.count[slot] > best.map_or(0, |held| self.count[held]) {
                best = Some(slot);
            }
        }
        best.map(|slot| self.pointer[slot])
    }
}

/// What one write did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Written {
    /// The pair has no address.
    NoAddress,
    /// No row, and the model's output was right: nothing stored.
    NotNeeded,
    /// A new row, or an empty slot of a row, took the target.
    Inserted,
    Strengthened,
    /// Every slot lost one count and none reached zero.
    Weakened,
    /// Every slot lost one count and the target took one that reached zero.
    Replaced,
}

/// Counts of [`Written`] outcomes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct WriteTally {
    pub no_address: u64,
    pub not_needed: u64,
    pub inserted: u64,
    pub strengthened: u64,
    pub weakened: u64,
    pub replaced: u64,
}
impl WriteTally {
    pub fn add(&mut self, written: Written) {
        match written {
            Written::NoAddress => self.no_address += 1,
            Written::NotNeeded => self.not_needed += 1,
            Written::Inserted => self.inserted += 1,
            Written::Strengthened => self.strengthened += 1,
            Written::Weakened => self.weakened += 1,
            Written::Replaced => self.replaced += 1,
        }
    }
}

fn first_primes(count: usize) -> Vec<u32> {
    let mut primes: Vec<u32> = Vec::with_capacity(count);
    let mut candidate = 2u32;
    while primes.len() < count {
        let prime = primes
            .iter()
            .take_while(|&&p| u64::from(p) * u64::from(p) <= u64::from(candidate))
            .all(|&p| candidate % p != 0);
        if prime {
            primes.push(candidate);
        }
        candidate += 1;
    }
    primes
}

#[derive(Debug)]
pub struct Conjunction {
    vocabulary: usize,
    slots: usize,
    order: Order,
    primes: Vec<u32>,
    rows: HashMap<u32, Row>,
}

impl Conjunction {
    pub fn new(vocabulary: u32, slots: usize, order: Order) -> Result<Self> {
        if vocabulary < 2 || !(1..=MAX_SLOTS).contains(&slots) {
            return Err(invalid("needs a vocabulary of at least 2 and 1..8 slots"));
        }
        let vocabulary = vocabulary as usize;
        let (count, limit) = match order {
            Order::Unordered => (vocabulary, 1u64 << 31),
            Order::Ordered => (2 * vocabulary, 1u64 << 32),
        };
        let primes = first_primes(count);
        let top = match order {
            Order::Unordered => {
                u64::from(primes[vocabulary - 1]) * u64::from(primes[vocabulary - 2])
            }
            Order::Ordered => u64::from(primes[vocabulary - 1]) * u64::from(primes[count - 1]),
        };
        if top >= limit {
            return Err(invalid(format!(
                "a vocabulary of {vocabulary} has the pair address {top}, past the address width"
            )));
        }
        Ok(Self {
            vocabulary,
            slots,
            order,
            primes,
            rows: HashMap::new(),
        })
    }
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    /// The address of the pair `a`, `b` (`b` the nearer token).
    pub fn address(&self, a: u32, b: u32) -> Option<u32> {
        let (a, b) = (a as usize, b as usize);
        if a >= self.vocabulary || b >= self.vocabulary {
            return None;
        }
        match self.order {
            Order::Unordered if a == b => None,
            Order::Unordered => Some(self.primes[a] * self.primes[b]),
            Order::Ordered => Some(self.primes[a] * self.primes[self.vocabulary + b]),
        }
    }
    pub fn read(&self, a: u32, b: u32) -> Option<Row> {
        self.rows.get(&self.address(a, b)?).copied()
    }
    /// The row's output when the pair has one, else the model's output.
    pub fn output(&self, a: u32, b: u32, model: Option<u32>) -> Option<u32> {
        self.read(a, b).and_then(|row| row.output()).or(model)
    }
    /// Counts which side was right where the copy read and the row output
    /// differ and one of them is the target. Call before `write`, so the row
    /// is judged on a target it has not yet seen. True when a count moved.
    pub fn judge(&mut self, a: u32, b: u32, copy: Option<u32>, target: u32) -> bool {
        let (Some(address), Some(copy)) = (self.address(a, b), copy) else {
            return false;
        };
        let Some(row) = self.rows.get_mut(&address) else {
            return false;
        };
        let Some(output) = row.output() else {
            return false;
        };
        if copy == output {
            return false;
        }
        let side = if copy == target {
            0
        } else if output == target {
            1
        } else {
            return false;
        };
        row.trust[side] = row.trust[side].saturating_add(1);
        true
    }
    pub fn write(&mut self, a: u32, b: u32, model: Option<u32>, target: u32) -> Written {
        let Some(address) = self.address(a, b) else {
            return Written::NoAddress;
        };
        let slots = self.slots;
        let Some(row) = self.rows.get_mut(&address) else {
            if model == Some(target) {
                return Written::NotNeeded;
            }
            let mut row = Row::default();
            row.pointer[0] = target;
            row.count[0] = 1;
            self.rows.insert(address, row);
            return Written::Inserted;
        };
        for slot in 0..slots {
            if row.count[slot] > 0 && row.pointer[slot] == target {
                row.count[slot] = row.count[slot].saturating_add(1);
                return Written::Strengthened;
            }
        }
        let mut written = Written::Inserted;
        if (0..slots).all(|slot| row.count[slot] > 0) {
            for slot in 0..slots {
                row.count[slot] -= 1;
            }
            written = Written::Replaced;
        }
        match (0..slots).find(|&slot| row.count[slot] == 0) {
            Some(slot) => {
                row.pointer[slot] = target;
                row.count[slot] = 1;
                written
            }
            None => Written::Weakened,
        }
    }
}

/// The pair of the target at `position`: the two tokens before it, unless the
/// nearer one is BOS or the stream is too short.
pub fn pair_at(tokens: &[u32], position: usize, bos: u32) -> Option<[u32; 2]> {
    if position < 2 || position > tokens.len() {
        return None;
    }
    let near = tokens[position - 1];
    if near == bos {
        return None;
    }
    Some([tokens[position - 2], near])
}

/// One scored DEV target: its pair, its target token and the model's output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scored {
    pub pair: Option<[u32; 2]>,
    pub target: u32,
    pub model: Option<u32>,
    /// The model's ranked outputs, best first; a value past the vocabulary
    /// marks an unused place.
    pub ranked: [u32; 8],
    /// The exact copy read ([`copy_at`]) on the target's window.
    pub copy: Option<u32>,
    /// The token after the position the model's Pick read chose.
    pub picked: Option<u32>,
}

/// The exact copy read (the Pick read at distance zero, `kn_pick`'s
/// `exact_pair_pick`): `window` ends just before the target; the latest earlier
/// place where its last two tokens stood together gives the token that
/// followed them there.
pub fn copy_at(window: &[u32]) -> Option<u32> {
    if window.len() < 3 {
        return None;
    }
    let t = window.len() - 1;
    (1..t)
        .rev()
        .find(|&p| window[p - 1] == window[t - 1] && window[p] == window[t])
        .map(|p| window[p + 1])
}

/// The window the copy read sees for the target at `position`: up to
/// `context` tokens before it, starting no earlier than its record's BOS.
pub fn window_at(tokens: &[u32], position: usize, context: usize, bos: u32) -> &[u32] {
    let position = position.min(tokens.len());
    let floor = position.saturating_sub(context);
    let mut start = position;
    while start > floor {
        start -= 1;
        if tokens[start] == bos {
            break;
        }
    }
    &tokens[start..position]
}

/// A pointer readout over scored targets: a context position (the exact copy
/// read, or the model's pick) names a token.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Pointer {
    /// Scored targets where the position names a token.
    pub present: u64,
    /// Of those, the token is the target.
    pub right: u64,
    /// Covered targets whose row holds the named token in a slot.
    pub covered_in_row: u64,
    /// Of those, the named token is the target.
    pub covered_in_row_right: u64,
    /// Covered: the named token when the row holds it, else the row output.
    /// Not covered: the model's output.
    pub slot_top1: u64,
    /// The named token wherever there is one; else the row output where
    /// covered; else the model's output.
    pub first_top1: u64,
}

impl Pointer {
    fn add(&mut self, named: Option<u32>, item: &Scored, live: Option<(Row, u32)>) {
        let model_right = item.model == Some(item.target);
        let named_right = named == Some(item.target);
        self.present += u64::from(named.is_some());
        self.right += u64::from(named_right);
        match live {
            Some((row, output)) => {
                let in_row = named.is_some_and(|token| {
                    (0..MAX_SLOTS).any(|slot| row.count[slot] > 0 && row.pointer[slot] == token)
                });
                self.covered_in_row += u64::from(in_row);
                self.covered_in_row_right += u64::from(in_row && named_right);
                let slot = if in_row { named } else { Some(output) };
                self.slot_top1 += u64::from(slot == Some(item.target));
                self.first_top1 += u64::from(named.unwrap_or(output) == item.target);
            }
            None => {
                self.slot_top1 += u64::from(model_right);
                self.first_top1 += u64::from(if named.is_some() {
                    named_right
                } else {
                    model_right
                });
            }
        }
    }
}

/// A reading of the memory beside the model on scored targets.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Reading {
    pub rows: u64,
    pub scored_targets: u64,
    pub model_top1: u64,
    /// Targets whose pair has a row with an output.
    pub covered: u64,
    /// Covered targets whose row output is the target.
    pub covered_row_right: u64,
    /// Covered targets with the target in any slot of the row.
    pub covered_row_holds: u64,
    /// Covered targets the model alone had right.
    pub covered_model_right: u64,
    /// Row output where covered, model output elsewhere.
    pub combined_top1: u64,
    /// Covered targets where the model ranks at least one of the row's slots.
    pub covered_model_ranks_a_slot: u64,
    /// As `combined_top1`, but a covered target takes the row slot the model
    /// ranks first when it ranks any, else the row output.
    pub picked_top1: u64,
    /// Pointer readout from the exact copy read.
    pub copy: Pointer,
    /// Pointer readout from the model's Pick position.
    pub pick: Pointer,
    /// Targets where both name a token and the tokens agree.
    pub pick_agrees_with_copy: u64,
    /// Covered targets where the copy read names a token other than the row
    /// output.
    pub disagreements: u64,
    /// Of those, the row's trust counts gave the answer to the copy read.
    pub disagreements_to_copy: u64,
    /// [`Row::answer`] where covered; else the copy read; else the model.
    pub answered_top1: u64,
}

pub fn read_scored(memory: &Conjunction, scored: &[Scored]) -> Reading {
    let mut reading = Reading {
        rows: memory.len() as u64,
        scored_targets: scored.len() as u64,
        ..Reading::default()
    };
    for item in scored {
        let model_right = item.model == Some(item.target);
        reading.model_top1 += u64::from(model_right);
        let row = item.pair.and_then(|[a, b]| memory.read(a, b));
        let live = row.and_then(|row| row.output().map(|output| (row, output)));
        reading.copy.add(item.copy, item, live);
        reading.pick.add(item.picked, item, live);
        reading.pick_agrees_with_copy += u64::from(item.copy.is_some() && item.copy == item.picked);
        let answer = match live {
            Some((row, output)) => {
                let answer = row.answer(item.copy);
                if item.copy.is_some_and(|copy| copy != output) {
                    reading.disagreements += 1;
                    reading.disagreements_to_copy += u64::from(answer == item.copy);
                }
                answer
            }
            None => item.copy.or(item.model),
        };
        reading.answered_top1 += u64::from(answer == Some(item.target));
        match live {
            Some((row, output)) => {
                let right = output == item.target;
                let holds = (0..MAX_SLOTS)
                    .any(|slot| row.count[slot] > 0 && row.pointer[slot] == item.target);
                reading.covered += 1;
                reading.covered_row_right += u64::from(right);
                reading.covered_row_holds += u64::from(holds);
                reading.covered_model_right += u64::from(model_right);
                reading.combined_top1 += u64::from(right);
                let picked = item.ranked.iter().copied().find(|&token| {
                    (0..MAX_SLOTS).any(|slot| row.count[slot] > 0 && row.pointer[slot] == token)
                });
                reading.covered_model_ranks_a_slot += u64::from(picked.is_some());
                reading.picked_top1 += u64::from(picked.unwrap_or(output) == item.target);
            }
            None => {
                reading.combined_top1 += u64::from(model_right);
                reading.picked_top1 += u64::from(model_right);
            }
        }
    }
    reading
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// TRAIN targets written with the model's output read on the GPU.
    pub model_targets: u64,
    /// Further TRAIN targets written with no model output (the memory's own
    /// output error decides the write).
    pub bulk_targets: u64,
    pub slots: usize,
    pub order: Order,
    /// Last TRAIN targets kept out of every write and read as a second scored
    /// set from the training corpus itself.
    pub holdout: u64,
}

/// `conjunction-probe`: write the memory from TRAIN and read it beside the
/// model on DEV.
#[cfg(feature = "cuda")]
pub fn run(
    config: crate::training::DriverConfig,
    checkpoint: Option<&std::path::Path>,
    out: &std::path::Path,
    options: Options,
) -> Result<serde_json::Value> {
    use crate::training::{DriverCheckpoint, PreparedDriver};
    use serde_json::json;
    use std::time::Instant;

    if out.exists() {
        return Err(invalid(format!("{} already exists", out.display())));
    }
    config.validate()?;
    let vocabulary = config.spec.vocabulary;
    let context = config.spec.context;
    let blocks = config.spec.blocks as usize;
    let batch_rows = config.batch_rows;
    let bos = config.bos.id;
    if context < 2 {
        return Err(invalid("needs a context of at least 2"));
    }
    let mut memory = Conjunction::new(vocabulary, options.slots, options.order)?;
    let driver = PreparedDriver::load(config)?;
    let model = match checkpoint {
        Some(path) => {
            let bytes =
                std::fs::read(path).map_err(|e| invalid(format!("{}: {e}", path.display())))?;
            let stored: DriverCheckpoint =
                serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?;
            let (model, _) = driver.restore(stored, 0)?;
            model
        }
        None => driver.new_model(0)?,
    };
    let model_output = |corpus: &crate::corpus::PreparedCorpus,
                        first: u64,
                        sink: &mut dyn FnMut(usize, u32, Option<u32>, [u32; 8])|
     -> Result<u64> {
        let batch = corpus.batch(first, context, batch_rows)?;
        let carried = model.carry_windows(&batch.windows)?;
        let evaluation = model.evaluate(&carried, &batch.targets)?;
        for ((row, position), target) in evaluation
            .rows
            .iter()
            .zip(&batch.target_positions)
            .zip(&batch.targets)
        {
            let top = row.top8[0];
            sink(
                *position as usize,
                *target,
                (top < vocabulary).then_some(top),
                row.top8,
            );
        }
        Ok(batch.next_target)
    };
    let window_mismatch = std::cell::Cell::new(0u64);
    // A scored set also takes the exact copy read on each window and, from the
    // forward trace, the token after the model's Pick position (first word of
    // a row's first block record; a Pick read only).
    let score = |corpus: &crate::corpus::PreparedCorpus,
                 first: u64,
                 scored: &mut Vec<Scored>|
     -> Result<u64> {
        let batch = corpus.batch(first, context, batch_rows)?;
        let carried = model.carry_windows(&batch.windows)?;
        let forward = model.forward_traced(&carried, &batch.targets)?;
        let picks = match (forward.read_trace()?, forward.trace_stride()) {
            (Some(words), Some(stride)) if model.read_spec() == crate::ReadSpec::Pick => {
                Some((words, stride * blocks))
            }
            _ => None,
        };
        drop(forward);
        let evaluation = model.evaluate(&carried, &batch.targets)?;
        for (index, row) in evaluation.rows.iter().enumerate() {
            let window: Vec<u32> = batch.windows[index]
                .iter()
                .map(|identity| identity.token)
                .collect();
            let picked = picks.as_ref().and_then(|(words, step)| {
                let pick = *words.get(index * step)? as usize;
                window.get(pick.checked_add(1)?).copied()
            });
            let host = window_at(
                corpus.tokens(),
                batch.target_positions[index] as usize,
                context as usize,
                bos,
            );
            if copy_at(host) != copy_at(&window) {
                window_mismatch.set(window_mismatch.get() + 1);
            }
            let top = row.top8[0];
            scored.push(Scored {
                pair: pair_at(corpus.tokens(), batch.target_positions[index] as usize, bos),
                target: batch.targets[index],
                model: (top < vocabulary).then_some(top),
                ranked: row.top8,
                copy: copy_at(&window),
                picked,
            });
        }
        Ok(batch.next_target)
    };

    let started = Instant::now();
    let dev = driver.dev();
    let mut scored = Vec::with_capacity(dev.eligible_len());
    let mut first = 0u64;
    while first < dev.eligible_len() as u64 {
        first = score(dev, first, &mut scored)?;
    }
    let dev_ms = started.elapsed().as_millis() as u64;

    let train = driver.train();
    let eligible = train.eligible_positions();
    let tokens = train.tokens();
    let write_limit = (eligible.len() as u64).saturating_sub(options.holdout);
    let model_end = options.model_targets.min(write_limit);
    let bulk_end = model_end
        .saturating_add(options.bulk_targets)
        .min(write_limit);
    let mut tail = Vec::new();
    let mut first = write_limit;
    while first < eligible.len() as u64 {
        first = score(train, first, &mut tail)?;
    }
    let empty = read_scored(&memory, &scored);

    let started = Instant::now();
    let mut judged = 0u64;
    let mut model_tally = WriteTally::default();
    let mut train_model_top1 = 0u64;
    let mut first = 0u64;
    while first < model_end {
        let mut pending = Vec::with_capacity(batch_rows as usize);
        let next = model_output(train, first, &mut |position, target, output, _| {
            pending.push((position, target, output));
        })?;
        for (index, (position, target, output)) in pending.into_iter().enumerate() {
            if first + index as u64 >= model_end {
                break;
            }
            train_model_top1 += u64::from(output == Some(target));
            if let Some([a, b]) = pair_at(tokens, position, bos) {
                let copy = copy_at(window_at(tokens, position, context as usize, bos));
                judged += u64::from(memory.judge(a, b, copy, target));
                model_tally.add(memory.write(a, b, output, target));
            }
        }
        first = next;
    }
    let model_ms = started.elapsed().as_millis() as u64;
    let after_model = read_scored(&memory, &scored);
    let tail_after_model = read_scored(&memory, &tail);

    let started = Instant::now();
    let mut bulk_tally = WriteTally::default();
    let mut curve = Vec::new();
    let mut mark = 1u64 << 20;
    for (done, &position) in eligible[model_end as usize..bulk_end as usize]
        .iter()
        .enumerate()
    {
        if let Some([a, b]) = pair_at(tokens, position, bos) {
            let copy = copy_at(window_at(tokens, position, context as usize, bos));
            judged += u64::from(memory.judge(a, b, copy, tokens[position]));
            bulk_tally.add(memory.write(a, b, None, tokens[position]));
        }
        if done as u64 + 1 == mark {
            curve.push(json!({
                "bulk_targets": mark,
                "dev": read_scored(&memory, &scored),
                "tail": read_scored(&memory, &tail),
            }));
            mark <<= 1;
        }
    }
    let bulk_ms = started.elapsed().as_millis() as u64;
    let after_bulk = read_scored(&memory, &scored);
    let tail_after_bulk = read_scored(&memory, &tail);

    let report = json!({
        "schema": SCHEMA,
        "order": options.order,
        "slots": options.slots,
        "pair_rule": "the two tokens before the target; none when the nearer is BOS",
        "lineage_sha256": driver.lineage_sha256()?,
        "checkpoint": checkpoint.map(|path| path.display().to_string()),
        "model_epoch": model.epoch()?,
        "train_eligible_targets": eligible.len(),
        "tail_rule": "the last TRAIN targets, never written",
        "tail_targets": tail.len(),
        "copy_rule": "the latest earlier place in the window where the pair stood; the token after it",
        "trust_writes": judged,
        "scored_copy_differs_between_host_and_batch_window": window_mismatch.get(),
        "dev_ms": dev_ms,
        "dev_empty_memory": empty,
        "model_written": {
            "targets": model_end,
            "train_model_top1": train_model_top1,
            "writes": model_tally,
            "ms": model_ms,
            "dev": after_model,
            "tail": tail_after_model,
        },
        "bulk_written": {
            "targets": bulk_end - model_end,
            "writes": bulk_tally,
            "ms": bulk_ms,
            "dev_curve": curve,
            "dev": after_bulk,
            "tail": tail_after_bulk,
        },
    });
    std::fs::write(
        out,
        serde_json::to_vec_pretty(&report).map_err(|e| invalid(e.to_string()))?,
    )
    .map_err(|e| invalid(format!("{}: {e}", out.display())))?;
    Ok(report)
}

/// Which parts answer when a reply is generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Readout {
    /// The row output alone.
    Rows,
    /// The copy read alone.
    Copy,
    /// [`Row::answer`]: the copy read against the row, decided by trust.
    Both,
}

/// The next token after `tokens`, or none when nothing answers.
pub fn next_token(
    memory: &Conjunction,
    readout: Readout,
    tokens: &[u32],
    window: usize,
    bos: u32,
) -> Option<u32> {
    let position = tokens.len();
    let copy = match readout {
        Readout::Rows => None,
        Readout::Copy | Readout::Both => copy_at(window_at(tokens, position, window, bos)),
    };
    let row = match readout {
        Readout::Copy => None,
        Readout::Rows | Readout::Both => {
            pair_at(tokens, position, bos).and_then(|[a, b]| memory.read(a, b))
        }
    };
    match row {
        Some(row) => row.answer(copy),
        None => copy,
    }
}

/// The greedy reply to `prompt`: tokens are taken one at a time until EOS,
/// `limit` tokens, or a place where nothing answers.
pub fn reply(
    memory: &Conjunction,
    readout: Readout,
    prompt: &[u32],
    window: usize,
    bos: u32,
    eos: u32,
    limit: usize,
) -> Vec<u32> {
    let mut tokens = prompt.to_vec();
    let start = tokens.len();
    while tokens.len() - start < limit {
        let Some(next) = next_token(memory, readout, &tokens, window, bos) else {
            break;
        };
        tokens.push(next);
        if next == eos {
            break;
        }
    }
    tokens.split_off(start)
}

/// One panel row: a prompt and its reference reply, as token ids.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PanelRow {
    pub prompt: Vec<u32>,
    pub reference: Vec<u32>,
}

/// Counts of one readout's greedy replies against the references.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct ReplyCount {
    pub rows: u64,
    /// The reply is the reference, id for id.
    pub exact: u64,
    /// The reply ends on EOS.
    pub ended: u64,
    /// The reply's first id is the reference's first id.
    pub first_right: u64,
    /// Reference ids, and how many the reply matches from the start.
    pub reference_ids: u64,
    pub prefix_ids: u64,
    /// Reply ids generated.
    pub reply_ids: u64,
}

pub fn count_replies(
    memory: &Conjunction,
    readout: Readout,
    panel: &[PanelRow],
    window: usize,
    bos: u32,
    eos: u32,
    limit: usize,
) -> (ReplyCount, Vec<Vec<u32>>) {
    let mut count = ReplyCount::default();
    let mut replies = Vec::with_capacity(panel.len());
    for row in panel {
        let got = reply(memory, readout, &row.prompt, window, bos, eos, limit);
        let prefix = got
            .iter()
            .zip(&row.reference)
            .take_while(|(a, b)| a == b)
            .count();
        count.rows += 1;
        count.exact += u64::from(got == row.reference);
        count.ended += u64::from(got.last() == Some(&eos));
        count.first_right += u64::from(prefix > 0);
        count.reference_ids += row.reference.len() as u64;
        count.prefix_ids += prefix as u64;
        count.reply_ids += got.len() as u64;
        replies.push(got);
    }
    (count, replies)
}

/// `conjunction-probe replies`: write rows and trust from every TRAIN target
/// with no model, then answer a panel's prompts greedily under each readout.
#[cfg(feature = "cuda")]
pub fn run_replies(
    config: crate::training::DriverConfig,
    panel_path: &std::path::Path,
    out: &std::path::Path,
    slots: usize,
    order: Order,
    window: usize,
    limit: usize,
    eos: u32,
) -> Result<serde_json::Value> {
    use crate::training::PreparedDriver;
    use serde_json::json;
    use std::time::Instant;

    if out.exists() {
        return Err(invalid(format!("{} already exists", out.display())));
    }
    config.validate()?;
    let vocabulary = config.spec.vocabulary;
    let context = config.spec.context as usize;
    let bos = config.bos.id;
    let bytes =
        std::fs::read(panel_path).map_err(|e| invalid(format!("{}: {e}", panel_path.display())))?;
    let panel: Vec<PanelRow> =
        serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?;
    if panel
        .iter()
        .flat_map(|row| row.prompt.iter().chain(&row.reference))
        .any(|&id| id >= vocabulary)
    {
        return Err(invalid("a panel id is past the vocabulary"));
    }
    let mut memory = Conjunction::new(vocabulary, slots, order)?;
    let driver = PreparedDriver::load(config)?;
    let train = driver.train();
    let tokens = train.tokens();
    let started = Instant::now();
    let mut tally = WriteTally::default();
    let mut judged = 0u64;
    for &position in train.eligible_positions() {
        if let Some([a, b]) = pair_at(tokens, position, bos) {
            let copy = copy_at(window_at(tokens, position, context, bos));
            judged += u64::from(memory.judge(a, b, copy, tokens[position]));
            tally.add(memory.write(a, b, None, tokens[position]));
        }
    }
    let write_ms = started.elapsed().as_millis() as u64;
    let started = Instant::now();
    let mut readouts = serde_json::Map::new();
    let mut replies = serde_json::Map::new();
    for (name, readout) in [
        ("rows", Readout::Rows),
        ("copy", Readout::Copy),
        ("both", Readout::Both),
    ] {
        let (count, got) = count_replies(&memory, readout, &panel, window, bos, eos, limit);
        readouts.insert(name.into(), json!(count));
        replies.insert(name.into(), json!(got));
    }
    let report = json!({
        "schema": "uor-r4.discrete-conjunction-replies/1",
        "panel": panel_path.display().to_string(),
        "panel_sha256": crate::corpus::sha256(&bytes),
        "order": order,
        "slots": slots,
        "write_window": context,
        "reply_window": window,
        "reply_limit": limit,
        "eos": eos,
        "lineage_sha256": driver.lineage_sha256()?,
        "train_targets": train.eligible_len(),
        "memory_rows": memory.len(),
        "writes": tally,
        "trust_writes": judged,
        "write_ms": write_ms,
        "reply_ms": started.elapsed().as_millis() as u64,
        "model": "none: rows and the copy read only",
        "readouts": readouts,
        "replies": replies,
    });
    std::fs::write(
        out,
        serde_json::to_vec(&report).map_err(|e| invalid(e.to_string()))?,
    )
    .map_err(|e| invalid(format!("{}: {e}", out.display())))?;
    Ok(json!({ "readouts": report["readouts"], "memory_rows": memory.len(), "write_ms": write_ms }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn held(memory: &Conjunction, a: u32, b: u32) -> Vec<[u32; 2]> {
        let row = memory.read(a, b).unwrap_or_default();
        (0..MAX_SLOTS)
            .filter(|&slot| row.count[slot] > 0)
            .map(|slot| [row.pointer[slot], row.count[slot]])
            .collect()
    }

    #[test]
    fn unordered_address_is_the_prime_product() -> Result<()> {
        let memory = Conjunction::new(4_096, 1, Order::Unordered)?;
        assert_eq!(memory.address(0, 1), Some(6));
        assert_eq!(memory.address(1, 0), Some(6));
        assert_eq!(memory.address(3, 3), None);
        assert_eq!(memory.address(0, 4_096), None);
        assert_eq!(memory.address(4_095, 4_094), Some(38_873 * 38_867));
        Ok(())
    }

    #[test]
    fn ordered_address_gives_the_nearer_token_its_own_primes() -> Result<()> {
        let memory = Conjunction::new(4, 1, Order::Ordered)?;
        // primes 2 3 5 7 for the farther token, 11 13 17 19 for the nearer.
        assert_eq!(memory.address(0, 1), Some(2 * 13));
        assert_eq!(memory.address(1, 0), Some(3 * 11));
        assert_eq!(memory.address(3, 3), Some(7 * 19));
        assert_eq!(memory.address(4, 0), None);
        let wide = Conjunction::new(4_096, 8, Order::Ordered)?;
        let mut seen = std::collections::HashSet::new();
        for a in [0u32, 1, 2, 4_094, 4_095] {
            for b in [0u32, 1, 2, 4_094, 4_095] {
                assert!(seen.insert(wide.address(a, b)));
            }
        }
        assert!(Conjunction::new(8_192, 1, Order::Ordered).is_err());
        Ok(())
    }

    #[test]
    fn a_row_is_stored_only_on_an_output_error() -> Result<()> {
        let mut memory = Conjunction::new(64, 1, Order::Unordered)?;
        assert_eq!(memory.write(1, 2, Some(9), 9), Written::NotNeeded);
        assert!(memory.is_empty());
        assert_eq!(memory.output(1, 2, Some(9)), Some(9));
        assert_eq!(memory.write(1, 2, Some(9), 7), Written::Inserted);
        assert_eq!(memory.output(2, 1, Some(9)), Some(7));
        assert_eq!(memory.write(5, 5, None, 7), Written::NoAddress);
        Ok(())
    }

    #[test]
    fn one_slot_gives_way_only_after_its_count_is_spent() -> Result<()> {
        let mut memory = Conjunction::new(64, 1, Order::Unordered)?;
        assert_eq!(memory.write(1, 2, None, 7), Written::Inserted);
        assert_eq!(memory.write(1, 2, None, 7), Written::Strengthened);
        assert_eq!(memory.write(1, 2, None, 8), Written::Weakened);
        assert_eq!(held(&memory, 1, 2), vec![[7, 1]]);
        assert_eq!(memory.write(1, 2, None, 8), Written::Replaced);
        assert_eq!(held(&memory, 1, 2), vec![[8, 1]]);
        Ok(())
    }

    #[test]
    fn slots_keep_the_commonest_pointer_through_other_targets() -> Result<()> {
        let mut memory = Conjunction::new(64, 2, Order::Ordered)?;
        // 7 is the commonest but never a majority of the stream.
        for target in [7u32, 8, 9, 7, 10, 11, 7, 12, 7, 13] {
            memory.write(1, 2, None, target);
        }
        assert_eq!(memory.output(1, 2, None), Some(7));
        assert_eq!(memory.output(2, 1, Some(3)), Some(3));
        let mut one = Conjunction::new(64, 1, Order::Ordered)?;
        for target in [7u32, 8, 9, 7, 10, 11, 7, 12, 7, 13] {
            one.write(1, 2, None, target);
        }
        assert_eq!(one.output(1, 2, None), Some(13));
        Ok(())
    }

    #[test]
    fn pair_and_reading_follow_the_stream() -> Result<()> {
        let bos = 0u32;
        let tokens = [0u32, 5, 6, 7, 0, 5, 6, 9];
        assert_eq!(pair_at(&tokens, 1, bos), None);
        assert_eq!(pair_at(&tokens, 2, bos), Some([0, 5]));
        assert_eq!(pair_at(&tokens, 3, bos), Some([5, 6]));
        assert_eq!(pair_at(&tokens, 5, bos), None);
        let mut memory = Conjunction::new(64, 2, Order::Unordered)?;
        memory.write(5, 6, None, 7);
        memory.write(5, 6, None, 7);
        memory.write(5, 6, None, 9);
        let scored = [
            Scored {
                pair: Some([5, 6]),
                target: 7,
                model: Some(1),
                ranked: [1, 7, 9, 64, 64, 64, 64, 64],
                copy: Some(9),
                picked: Some(7),
            },
            Scored {
                pair: Some([5, 6]),
                target: 9,
                model: Some(9),
                ranked: [9, 7, 64, 64, 64, 64, 64, 64],
                copy: Some(9),
                picked: None,
            },
            Scored {
                pair: None,
                target: 5,
                model: Some(5),
                ranked: [5, 64, 64, 64, 64, 64, 64, 64],
                copy: Some(5),
                picked: Some(3),
            },
        ];
        let reading = read_scored(&memory, &scored);
        assert_eq!(reading.model_top1, 2);
        assert_eq!(reading.covered, 2);
        assert_eq!(reading.covered_row_right, 1);
        assert_eq!(reading.covered_row_holds, 2);
        assert_eq!(reading.covered_model_right, 1);
        assert_eq!(reading.combined_top1, 2);
        // Row holds 7 (count 2) and 9 (count 1). The model ranks 7 before 9 on
        // the first target and 9 before 7 on the second, so the pick is right
        // on both; the third has no pair and keeps the model's output.
        assert_eq!(reading.covered_model_ranks_a_slot, 2);
        assert_eq!(reading.picked_top1, 3);
        // Copy names 9, 9, 5: wrong where the row also holds it (first
        // target), right on the second, right with no row on the third.
        assert_eq!(
            reading.copy,
            Pointer {
                present: 3,
                right: 2,
                covered_in_row: 2,
                covered_in_row_right: 1,
                slot_top1: 2,
                first_top1: 2,
            }
        );
        // The pick names 7, nothing, 3: the second target falls back to the
        // row output 7 (wrong), the third keeps the model under the slot rule.
        assert_eq!(
            reading.pick,
            Pointer {
                present: 2,
                right: 1,
                covered_in_row: 1,
                covered_in_row_right: 1,
                slot_top1: 2,
                first_top1: 1,
            }
        );
        assert_eq!(reading.pick_agrees_with_copy, 0);
        Ok(())
    }

    #[test]
    fn trust_counts_decide_between_the_copy_read_and_the_row() -> Result<()> {
        let mut memory = Conjunction::new(64, 2, Order::Ordered)?;
        assert!(!memory.judge(1, 2, Some(9), 9));
        memory.write(1, 2, None, 7);
        let row = |memory: &Conjunction| memory.read(1, 2).unwrap_or_default();
        assert_eq!(row(&memory).answer(None), Some(7));
        assert_eq!(row(&memory).answer(Some(7)), Some(7));
        // No evidence yet: the copy read answers.
        assert_eq!(row(&memory).answer(Some(9)), Some(9));
        // Agreement and a target neither side names move nothing.
        assert!(!memory.judge(1, 2, Some(7), 7));
        assert!(!memory.judge(1, 2, Some(9), 3));
        assert!(memory.judge(1, 2, Some(9), 7));
        assert_eq!(row(&memory).trust, [0, 1]);
        assert_eq!(row(&memory).answer(Some(9)), Some(7));
        assert!(memory.judge(1, 2, Some(9), 9));
        assert!(memory.judge(1, 2, Some(9), 9));
        assert_eq!(row(&memory).trust, [2, 1]);
        assert_eq!(row(&memory).answer(Some(9)), Some(9));
        assert_eq!(Row::default().answer(Some(4)), Some(4));
        assert_eq!(Row::default().answer(None), None);
        Ok(())
    }

    #[test]
    fn a_reply_is_generated_until_eos_the_limit_or_no_answer() -> Result<()> {
        let (bos, eos) = (0u32, 1u32);
        let mut memory = Conjunction::new(64, 2, Order::Ordered)?;
        memory.write(5, 6, None, 7);
        memory.write(6, 7, None, eos);
        let run =
            |readout, prompt: &[u32], limit| reply(&memory, readout, prompt, 192, bos, eos, limit);
        assert_eq!(run(Readout::Rows, &[0, 5, 6], 8), vec![7, 1]);
        assert_eq!(run(Readout::Rows, &[0, 5, 6], 1), vec![7]);
        assert_eq!(run(Readout::Rows, &[0, 9, 9], 8), Vec::<u32>::new());
        // The copy read repeats what followed the pair before; with no row for
        // (9, 4) and no earlier place of it, the reply stops.
        assert_eq!(
            run(Readout::Copy, &[0, 5, 6, 9, 4, 5, 6], 8),
            vec![9, 4, 5, 6, 9, 4, 5, 6]
        );
        assert_eq!(
            run(Readout::Copy, &[0, 5, 6, 9, 4, 8, 6], 8),
            Vec::<u32>::new()
        );
        // Both: the copy read names 9, the row 7, and no evidence favours the row.
        assert_eq!(run(Readout::Both, &[0, 5, 6, 9, 3, 5, 6], 1), vec![9]);
        let panel = [
            PanelRow {
                prompt: vec![0, 5, 6],
                reference: vec![7, 1],
            },
            PanelRow {
                prompt: vec![0, 5, 6],
                reference: vec![7, 8, 1],
            },
        ];
        let (count, replies) = count_replies(&memory, Readout::Rows, &panel, 192, bos, eos, 8);
        assert_eq!(replies, vec![vec![7, 1], vec![7, 1]]);
        assert_eq!(
            count,
            ReplyCount {
                rows: 2,
                exact: 1,
                ended: 2,
                first_right: 2,
                reference_ids: 5,
                prefix_ids: 3,
                reply_ids: 4,
            }
        );
        Ok(())
    }

    #[test]
    fn the_window_stops_at_the_record_start_and_the_context() {
        let tokens = [0u32, 5, 6, 7, 0, 5, 6, 9, 8];
        assert_eq!(window_at(&tokens, 3, 192, 0), &[0, 5, 6]);
        assert_eq!(window_at(&tokens, 8, 192, 0), &[0, 5, 6, 9]);
        assert_eq!(window_at(&tokens, 8, 2, 0), &[6, 9]);
        assert_eq!(window_at(&tokens, 0, 192, 0), &[] as &[u32]);
    }

    #[test]
    fn the_copy_read_takes_the_latest_earlier_place_of_the_pair() {
        assert_eq!(copy_at(&[5, 6]), None);
        assert_eq!(copy_at(&[1, 2, 3]), None);
        assert_eq!(copy_at(&[5, 6, 7, 1, 5, 6]), Some(7));
        assert_eq!(copy_at(&[5, 6, 7, 5, 6, 8, 5, 6]), Some(8));
        // The pair's own last place is not an earlier place.
        assert_eq!(copy_at(&[6, 5, 5, 6]), None);
    }
}
