//! The turn memory in the integer artifact (claude-198): claude-161's opening table and
//! claude-193's reply table, with their move weights, as a `uor-r4.lut-stack/7` container stores
//! them.
//!
//! Serving the memory is the next units of `turn-memory-integer`. Until then
//! [`super::IntegerStackModel::parse`] refuses such a container, so the memory is never dropped.

use super::format::{Container, StackTurnMemory, MEMORY_PAD, MEMORY_REPLY_KEY_LEN};
use super::StackError;

/// One memory table: its keys (pads cut) in strictly ascending order, the id each names, and the
/// move's weight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryTableImage {
    pub keys: Vec<Vec<u32>>,
    pub values: Vec<u32>,
    /// `width` values at `2^MEMORY_WEIGHT_EXP`.
    pub weight: Vec<i32>,
}

/// A stack's turn memory: the header record and its tables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryImage {
    pub record: StackTurnMemory,
    pub opening: MemoryTableImage,
    pub reply: Option<MemoryTableImage>,
}

/// The turn memory of a `UORLUT01` stack artifact, every section checked, or `None` for a stack
/// without one.
pub fn read_memory_uorlut01(bytes: &[u8]) -> Result<Option<MemoryImage>, StackError> {
    memory_image(&Container::parse(bytes)?)
}

/// The turn memory of a parsed container, as [`read_memory_uorlut01`].
pub(crate) fn memory_image(artifact: &Container<'_>) -> Result<Option<MemoryImage>, StackError> {
    let Some(record) = artifact.memory.clone() else {
        return Ok(None);
    };
    let (vocab, width) = (artifact.shape.vocab, artifact.shape.width);
    let (k, window) = (record.k, record.key_window);
    let opening = memory_table(artifact, "opening", record.opening, k, width, vocab, &|key| {
        opening_key_ok(key, k, vocab)
    })?;
    // The record's check holds the count and the turn together.
    let reply = match (record.reply, record.reply_turn) {
        (Some(count), Some(turn)) => Some(memory_table(
            artifact,
            "reply",
            count,
            MEMORY_REPLY_KEY_LEN,
            width,
            vocab,
            &|key| reply_key_ok(key, turn, window, vocab),
        )?),
        _ => None,
    };
    Ok(Some(MemoryImage {
        record,
        opening,
        reply,
    }))
}

/// An opening key: 1 to `k` turn ids, each a token id (the float memory never stores an empty one).
fn opening_key_ok(key: &[u32], k: usize, vocab: usize) -> bool {
    !key.is_empty() && turn_ok(key, k, vocab)
}

/// At most `k` turn ids, each a token id.
fn turn_ok(ids: &[u32], k: usize, vocab: usize) -> bool {
    ids.len() <= k && ids.iter().all(|&id| (id as usize) < vocab)
}

/// A reply key, `turn ids, MEMORY_PAD, length, MEMORY_PAD, last id` (the float memory's
/// `ReplyKey::TurnPosition(1)`): 0 to `turn` turn ids (the record's `reply_turn`, not `k`), a reply
/// length from 1 to the key window, and a last id that is a token id.
fn reply_key_ok(key: &[u32], turn_len: usize, window: usize, vocab: usize) -> bool {
    let n = key.len();
    if n < 4 {
        return false;
    }
    let (turn, tail) = key.split_at(n - 4);
    let (length, last) = (tail[1] as usize, tail[3] as usize);
    tail[0] == MEMORY_PAD
        && tail[2] == MEMORY_PAD
        && (1..=window).contains(&length)
        && last < vocab
        && turn_ok(turn, turn_len, vocab)
}

/// One table's sections `memory.{kind}.{keys,values,weight}`: `count` keys of `row` ids each
/// (padded with `MEMORY_PAD`), `count` values and `width` weights. Refuses a section of the wrong
/// length, a key `key_ok` rejects, keys out of strictly ascending order, or a value past the
/// vocabulary.
fn memory_table(
    artifact: &Container<'_>,
    kind: &str,
    count: usize,
    row: usize,
    width: usize,
    vocab: usize,
    key_ok: &dyn Fn(&[u32]) -> bool,
) -> Result<MemoryTableImage, StackError> {
    let name = |part: &str| format!("memory.{kind}.{part}");
    let section = |part: &str, values: Vec<u32>, len: usize| -> Result<Vec<u32>, StackError> {
        if values.len() == len {
            Ok(values)
        } else {
            Err(StackError::TableLength {
                name: name(part),
                len: values.len(),
                expected: len,
            })
        }
    };
    let raw = section("keys", artifact.table_u32(&name("keys"))?, count * row)?;
    let values = section("values", artifact.table_u32(&name("values"))?, count)?;
    let weight = artifact.table_i32(&name("weight"))?;
    if weight.len() != width {
        return Err(StackError::TableLength {
            name: name("weight"),
            len: weight.len(),
            expected: width,
        });
    }
    let mut keys: Vec<Vec<u32>> = Vec::with_capacity(count);
    for (i, padded) in raw.chunks_exact(row).enumerate() {
        // A key ends in a token id, so cutting the trailing pads reads back exactly what was stored.
        let len = padded.iter().rposition(|&id| id != MEMORY_PAD).map_or(0, |p| p + 1);
        let key = padded[..len].to_vec();
        if !key_ok(&key) {
            return Err(StackError::Numerics(format!(
                "{}: entry {i} is not a key this engine reads",
                name("keys")
            )));
        }
        if keys.last().is_some_and(|previous| key <= *previous) {
            return Err(StackError::Numerics(format!(
                "{}: the keys are not in strictly ascending order at entry {i}",
                name("keys")
            )));
        }
        keys.push(key);
    }
    if let Some(i) = values.iter().position(|&v| v as usize >= vocab) {
        return Err(StackError::Numerics(format!(
            "{}: entry {i} names an id past the vocabulary",
            name("values")
        )));
    }
    Ok(MemoryTableImage {
        keys,
        values,
        weight,
    })
}
