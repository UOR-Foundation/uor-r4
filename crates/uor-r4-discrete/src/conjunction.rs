//! Conjunction memory (PLAN row 39, step 2): one row per unordered pair of
//! different tokens, addressed by the prime product, holding one pointer token
//! and a small confidence count. A row's pointer changes only when the output
//! was wrong.
//!
//! 1. **Address.** The rule of `uor-r4-integer`'s `fbs_pair_rows`: token `t`
//!    stands for the `t`-th prime, and the pair `a`, `b` has the address
//!    `primes[a] * primes[b]`, under 2^31. Equal tokens have no address.
//! 2. **Pair of a target.** The two tokens before it, when the nearer one is
//!    not BOS (a window never crosses a record).
//! 3. **Read.** The output is the row's pointer when the pair has a row, else
//!    the model's own output.
//! 4. **Write.** With no row: nothing when the model's output was right, else a
//!    new row pointing at the target. With a row: a right pointer gains one
//!    count (up to [`CONFIDENCE_CAP`]); a wrong one loses one, and at one count
//!    it is replaced by the target.
//!
//! Every step is an integer compare, add or table lookup.
#![forbid(unsafe_code)]
use crate::{Error, Result};
use std::collections::HashMap;

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(format!("conjunction: {}", message.into()))
}

/// Bound every pair address stays under.
const LIMIT: u64 = 1 << 31;
/// Largest confidence count a row holds.
pub const CONFIDENCE_CAP: u16 = 255;
pub const SCHEMA: &str = "uor-r4.discrete-conjunction-probe/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Row {
    pub pointer: u32,
    pub confidence: u16,
}

/// What one write did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Written {
    /// The pair has no address (equal tokens or a token outside the vocabulary).
    NoAddress,
    /// No row, and the model's output was right: nothing stored.
    NotNeeded,
    Inserted,
    Strengthened,
    Weakened,
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
    primes: Vec<u32>,
    rows: HashMap<u32, Row>,
}

impl Conjunction {
    pub fn new(vocabulary: u32) -> Result<Self> {
        if vocabulary < 2 {
            return Err(invalid("needs a vocabulary of at least 2"));
        }
        let primes = first_primes(vocabulary as usize);
        let last = primes.len();
        let top = u64::from(primes[last - 1]) * u64::from(primes[last - 2]);
        if top >= LIMIT {
            return Err(invalid(format!(
                "a vocabulary of {vocabulary} has the pair address {top}, past 31 bits"
            )));
        }
        Ok(Self {
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
    /// The address of the unordered pair `a`, `b`.
    pub fn address(&self, a: u32, b: u32) -> Option<u32> {
        if a == b {
            return None;
        }
        let pa = *self.primes.get(a as usize)?;
        let pb = *self.primes.get(b as usize)?;
        Some(pa * pb)
    }
    pub fn read(&self, a: u32, b: u32) -> Option<Row> {
        self.rows.get(&self.address(a, b)?).copied()
    }
    /// The row's pointer when the pair has a row, else the model's output.
    pub fn output(&self, a: u32, b: u32, model: Option<u32>) -> Option<u32> {
        self.read(a, b).map(|row| row.pointer).or(model)
    }
    pub fn write(&mut self, a: u32, b: u32, model: Option<u32>, target: u32) -> Written {
        let Some(address) = self.address(a, b) else {
            return Written::NoAddress;
        };
        match self.rows.get_mut(&address) {
            Some(row) if row.pointer == target => {
                row.confidence = row.confidence.saturating_add(1).min(CONFIDENCE_CAP);
                Written::Strengthened
            }
            Some(row) if row.confidence > 1 => {
                row.confidence -= 1;
                Written::Weakened
            }
            Some(row) => {
                row.pointer = target;
                row.confidence = 1;
                Written::Replaced
            }
            None if model == Some(target) => Written::NotNeeded,
            None => {
                self.rows.insert(
                    address,
                    Row {
                        pointer: target,
                        confidence: 1,
                    },
                );
                Written::Inserted
            }
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
}

/// A reading of the memory beside the model on scored targets.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Reading {
    pub rows: u64,
    pub scored_targets: u64,
    pub model_top1: u64,
    /// Targets whose pair has a row.
    pub covered: u64,
    /// Covered targets whose row points at the target.
    pub covered_row_right: u64,
    /// Covered targets the model alone had right.
    pub covered_model_right: u64,
    /// Row pointer where covered, model output elsewhere.
    pub combined_top1: u64,
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
        match row {
            Some(row) => {
                let right = row.pointer == item.target;
                reading.covered += 1;
                reading.covered_row_right += u64::from(right);
                reading.covered_model_right += u64::from(model_right);
                reading.combined_top1 += u64::from(right);
            }
            None => reading.combined_top1 += u64::from(model_right),
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
}

/// `conjunction-probe <config.json> <checkpoint.json|-> <out.json>
/// <model_targets> <bulk_targets>`: write the memory from TRAIN and read it
/// beside the model on DEV.
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
    let batch_rows = config.batch_rows;
    let bos = config.bos.id;
    if context < 2 {
        return Err(invalid("needs a context of at least 2"));
    }
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
                        sink: &mut dyn FnMut(usize, u32, Option<u32>)|
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
            );
        }
        Ok(batch.next_target)
    };

    let started = Instant::now();
    let dev = driver.dev();
    let mut scored = Vec::with_capacity(dev.eligible_len());
    let mut first = 0u64;
    while first < dev.eligible_len() as u64 {
        first = model_output(dev, first, &mut |position, target, output| {
            scored.push(Scored {
                pair: pair_at(dev.tokens(), position, bos),
                target,
                model: output,
            });
        })?;
    }
    let dev_ms = started.elapsed().as_millis() as u64;

    let train = driver.train();
    let eligible = train.eligible_positions();
    let tokens = train.tokens();
    let model_end = options.model_targets.min(eligible.len() as u64);
    let bulk_end = model_end
        .saturating_add(options.bulk_targets)
        .min(eligible.len() as u64);
    let mut memory = Conjunction::new(vocabulary)?;
    let empty = read_scored(&memory, &scored);

    let started = Instant::now();
    let mut model_tally = WriteTally::default();
    let mut train_model_top1 = 0u64;
    let mut first = 0u64;
    while first < model_end {
        let mut pending = Vec::with_capacity(batch_rows as usize);
        let next = model_output(train, first, &mut |position, target, output| {
            pending.push((position, target, output));
        })?;
        for (index, (position, target, output)) in pending.into_iter().enumerate() {
            if first + index as u64 >= model_end {
                break;
            }
            train_model_top1 += u64::from(output == Some(target));
            if let Some([a, b]) = pair_at(tokens, position, bos) {
                model_tally.add(memory.write(a, b, output, target));
            }
        }
        first = next;
    }
    let model_ms = started.elapsed().as_millis() as u64;
    let after_model = read_scored(&memory, &scored);

    let started = Instant::now();
    let mut bulk_tally = WriteTally::default();
    let mut curve = Vec::new();
    let mut mark = 1u64 << 20;
    for (done, &position) in eligible[model_end as usize..bulk_end as usize]
        .iter()
        .enumerate()
    {
        if let Some([a, b]) = pair_at(tokens, position, bos) {
            bulk_tally.add(memory.write(a, b, None, tokens[position]));
        }
        if done as u64 + 1 == mark {
            curve.push(json!({"bulk_targets": mark, "dev": read_scored(&memory, &scored)}));
            mark <<= 1;
        }
    }
    let bulk_ms = started.elapsed().as_millis() as u64;
    let after_bulk = read_scored(&memory, &scored);

    let report = json!({
        "schema": SCHEMA,
        "address_rule": "primes[a]*primes[b], unordered pair of different tokens",
        "pair_rule": "the two tokens before the target; none when the nearer is BOS",
        "confidence_cap": CONFIDENCE_CAP,
        "lineage_sha256": driver.lineage_sha256()?,
        "checkpoint": checkpoint.map(|path| path.display().to_string()),
        "model_epoch": model.epoch()?,
        "train_eligible_targets": eligible.len(),
        "dev_ms": dev_ms,
        "dev_empty_memory": empty,
        "model_written": {
            "targets": model_end,
            "train_model_top1": train_model_top1,
            "writes": model_tally,
            "ms": model_ms,
            "dev": after_model,
        },
        "bulk_written": {
            "targets": bulk_end - model_end,
            "writes": bulk_tally,
            "ms": bulk_ms,
            "dev_curve": curve,
            "dev": after_bulk,
        },
    });
    std::fs::write(
        out,
        serde_json::to_vec_pretty(&report).map_err(|e| invalid(e.to_string()))?,
    )
    .map_err(|e| invalid(format!("{}: {e}", out.display())))?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_is_the_prime_product_and_ignores_order() -> Result<()> {
        let memory = Conjunction::new(4_096)?;
        assert_eq!(memory.address(0, 1), Some(6));
        assert_eq!(memory.address(1, 0), Some(6));
        assert_eq!(memory.address(3, 3), None);
        assert_eq!(memory.address(0, 4_096), None);
        assert_eq!(memory.address(4_095, 4_094), Some(38_873 * 38_867));
        Ok(())
    }

    #[test]
    fn a_row_is_stored_only_on_an_output_error() -> Result<()> {
        let mut memory = Conjunction::new(64)?;
        assert_eq!(memory.write(1, 2, Some(9), 9), Written::NotNeeded);
        assert!(memory.is_empty());
        assert_eq!(memory.output(1, 2, Some(9)), Some(9));
        assert_eq!(memory.write(1, 2, Some(9), 7), Written::Inserted);
        assert_eq!(memory.output(2, 1, Some(9)), Some(7));
        assert_eq!(memory.write(5, 5, None, 7), Written::NoAddress);
        Ok(())
    }

    #[test]
    fn a_wrong_pointer_gives_way_only_after_its_count_is_spent() -> Result<()> {
        let mut memory = Conjunction::new(64)?;
        assert_eq!(memory.write(1, 2, None, 7), Written::Inserted);
        assert_eq!(memory.write(1, 2, None, 7), Written::Strengthened);
        assert_eq!(memory.write(1, 2, None, 8), Written::Weakened);
        assert_eq!(
            memory.read(1, 2),
            Some(Row {
                pointer: 7,
                confidence: 1
            })
        );
        assert_eq!(memory.write(1, 2, None, 8), Written::Replaced);
        assert_eq!(
            memory.read(1, 2),
            Some(Row {
                pointer: 8,
                confidence: 1
            })
        );
        for _ in 0..400 {
            memory.write(1, 2, None, 8);
        }
        assert_eq!(memory.read(1, 2).map(|row| row.confidence), Some(255));
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
        let mut memory = Conjunction::new(64)?;
        memory.write(5, 6, None, 7);
        let scored = [
            Scored {
                pair: Some([5, 6]),
                target: 7,
                model: Some(1),
            },
            Scored {
                pair: Some([5, 6]),
                target: 9,
                model: Some(9),
            },
            Scored {
                pair: None,
                target: 5,
                model: Some(5),
            },
        ];
        let reading = read_scored(&memory, &scored);
        assert_eq!(reading.model_top1, 2);
        assert_eq!(reading.covered, 2);
        assert_eq!(reading.covered_row_right, 1);
        assert_eq!(reading.covered_model_right, 1);
        assert_eq!(reading.combined_top1, 2);
        Ok(())
    }
}
