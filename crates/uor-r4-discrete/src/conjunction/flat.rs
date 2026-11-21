//! The reader's question forms and held rows as flat `u32` tables in key
//! order, for a reader that finds a row by binary search and holds no map
//! (the CUDA read kernel, `reader_read.cu`).
//!
//! Every wide key is cut into `u32` words, most significant first, so the
//! order of the words is the order of the key and the words are the ones a
//! replay compares ([`super::Asked::forms_by_id`], `step_words`).
//!
//! Only the read choices of the saved M2 readers are laid out: the pair
//! stand, slots ranked `precise`, a place answered `across` its forms, and
//! none of `explain`, `tail`, `agree`. Any other reader is refused.
use super::saved::{entry_fields, Reader};
use super::{invalid, Reads, StandBy, MAX_SLOTS, ROW_RIGHT};
use crate::Result;

/// A form: its three key words as twelve `u32`, the gap's place, its length.
pub const FORM_WORDS: usize = 14;
/// A held row's key: form id, empty mark, tail, the history as eight `u32`.
pub const KEY_WORDS: usize = 11;
/// A slot: kind, back, ahead, wide, value, times right, times wrong.
pub const SLOT_WORDS: usize = 7;
/// A held row's slots; an unused slot is all zero.
pub const ROW_WORDS: usize = MAX_SLOTS * SLOT_WORDS;

/// One reader's tables. `forms` is in id order, which is key order;
/// `row_keys` rises strictly and `row_slots` follows it row for row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flat {
    pub vocabulary: u32,
    pub context: u32,
    pub bos: u32,
    pub eos: u32,
    pub forms: Vec<u32>,
    pub row_keys: Vec<u32>,
    pub row_slots: Vec<u32>,
    /// One class code a token, or empty; under `trunk`, one trunk code a
    /// token.
    pub codes: Vec<u32>,
    /// The forms are keyed by the trunk code of each kept token.
    pub trunk: bool,
}

fn quarters(word: u128) -> [u32; 4] {
    std::array::from_fn(|part| (word >> (96 - 32 * part)) as u32)
}

/// The index of `key` among the `width`-word rows of a rising `table`.
pub fn find(table: &[u32], width: usize, key: &[u32]) -> Option<usize> {
    let mut low = 0;
    let mut high = table.len() / width;
    while low < high {
        let middle = low + (high - low) / 2;
        match table[middle * width..(middle + 1) * width].cmp(key) {
            std::cmp::Ordering::Less => low = middle + 1,
            std::cmp::Ordering::Greater => high = middle,
            std::cmp::Ordering::Equal => return Some(middle),
        }
    }
    None
}

fn rises(table: &[u32], width: usize) -> bool {
    table
        .chunks_exact(width)
        .zip(table.chunks_exact(width).skip(1))
        .all(|(before, after)| before < after)
}

impl Flat {
    /// The tables of `reader`, with the slots a read can reach (right at
    /// least [`ROW_RIGHT`] times), as its file holds them.
    pub fn of(reader: &Reader) -> Result<Self> {
        let asked = &reader.asked;
        let wanted = Reads {
            precise: true,
            across: true,
            ..Reads::default()
        };
        if asked.reads() != wanted || asked.stand_by() != StandBy::Pair {
            return Err(invalid(
                "flat tables are laid out for the pair stand read precise,across alone",
            ));
        }
        // A form keyed a kept token at a time is found by the kernel with the
        // trunk codes in the place of token ids; one keyed by a single frame
        // is not laid out here.
        let trunk_codes: Vec<u32> = match asked.trunk() {
            None => Vec::new(),
            Some(trunk) if trunk.codes.len() == reader.vocabulary as usize => {
                trunk.codes.iter().map(|&code| u32::from(code)).collect()
            }
            Some(_) => {
                return Err(invalid(
                    "flat tables hold forms keyed a token at a time: not a single trunk frame",
                ))
            }
        };
        let trunk = !trunk_codes.is_empty();
        let mut by_id: Vec<(u32, [u32; FORM_WORDS])> = asked.forms_by_id().into_iter().collect();
        by_id.sort_by_key(|&(id, _)| id);
        let mut forms = Vec::with_capacity(by_id.len() * FORM_WORDS);
        for (place, (id, words)) in by_id.into_iter().enumerate() {
            if id as usize != place {
                return Err(invalid("form ids must count from zero"));
            }
            forms.extend(words);
        }
        let mut rows: Vec<([u32; KEY_WORDS], [u32; ROW_WORDS])> = Vec::new();
        for (key, row) in &asked.rows {
            let mut slots = [0u32; ROW_WORDS];
            let mut live = 0;
            for slot in 0..MAX_SLOTS {
                if row.count[slot] < ROW_RIGHT {
                    continue;
                }
                let at = live * SLOT_WORDS;
                slots[at..at + 5].copy_from_slice(&entry_fields(row.entry[slot]));
                slots[at + 5] = row.count[slot];
                slots[at + 6] = row.wrong[slot];
                live += 1;
            }
            if live == 0 {
                continue;
            }
            let mut words = [0u32; KEY_WORDS];
            words[0] = key.form;
            words[1] = u32::from(key.empty);
            words[2] = key.tail;
            words[3..7].copy_from_slice(&quarters(key.history[0]));
            words[7..11].copy_from_slice(&quarters(key.history[1]));
            rows.push((words, slots));
        }
        rows.sort_by_key(|&(words, _)| words);
        let mut row_keys = Vec::with_capacity(rows.len() * KEY_WORDS);
        let mut row_slots = Vec::with_capacity(rows.len() * ROW_WORDS);
        for (words, slots) in rows {
            row_keys.extend(words);
            row_slots.extend(slots);
        }
        if !rises(&forms, FORM_WORDS) || !rises(&row_keys, KEY_WORDS) {
            return Err(invalid("flat tables must rise"));
        }
        Ok(Self {
            vocabulary: reader.vocabulary,
            context: asked.context as u32,
            bos: asked.bos,
            eos: asked.eos,
            forms,
            row_keys,
            row_slots,
            codes: if trunk {
                trunk_codes
            } else {
                asked.codes().iter().map(|&code| u32::from(code)).collect()
            },
            trunk,
        })
    }
    pub fn form_count(&self) -> usize {
        self.forms.len() / FORM_WORDS
    }
    pub fn row_count(&self) -> usize {
        self.row_keys.len() / KEY_WORDS
    }
}

#[cfg(test)]
mod tests {
    use super::super::bind::BindTable;
    use super::*;

    fn train(reads: Reads, by: StandBy) -> Result<Reader> {
        train_beside(reads, by, None)
    }

    fn train_beside(
        reads: Reads,
        by: StandBy,
        trunk: Option<crate::conjunction::trunk_key::TrunkKey>,
    ) -> Result<Reader> {
        let [bos, eos, name, is, dot, what, mark] = [0u32, 1, 2, 3, 4, 5, 6];
        let mut tokens = Vec::new();
        let mut eligible = Vec::new();
        for index in 0..16u32 {
            let value = if index % 2 == 0 { 9 } else { 20 + index };
            tokens.extend([bos, name, is, value, dot, eos, what, name, mark]);
            let at = tokens.len();
            tokens.extend([value, dot, eos]);
            eligible.extend(at..at + 3);
        }
        let (reader, _) = Reader::train_by(
            &tokens,
            &eligible,
            64,
            192,
            bos,
            eos,
            2,
            &[],
            8,
            &[],
            by,
            BindTable::default(),
            false,
            reads,
            crate::conjunction::free::Update::InOrder,
            trunk,
        )?;
        Ok(reader)
    }

    const READS: Reads = Reads {
        precise: true,
        explain: false,
        tail_row: false,
        across: true,
        agree: false,
    };

    /// Every form and every saved row is found at its own index by the
    /// search the kernel uses, a key that is not held is not found, and the
    /// tables of the trained reader are those of its file.
    #[test]
    fn every_saved_row_is_found_by_its_key_words() -> Result<()> {
        let reader = train(READS, StandBy::Pair)?;
        let flat = Flat::of(&reader)?;
        let (bytes, saved) = reader.to_bytes(1);
        assert!(flat.form_count() > 0 && flat.row_count() > 0);
        assert_eq!(flat.form_count() as u64, saved.forms);
        assert_eq!(flat.row_count() as u64, saved.held_rows);
        let live = flat
            .row_slots
            .chunks_exact(SLOT_WORDS)
            .filter(|slot| slot[5] > 0)
            .count();
        assert_eq!(live as u64, saved.held_slots);
        assert_eq!(Flat::of(&Reader::from_bytes(&bytes)?)?, flat);
        for (index, key) in flat.forms.chunks_exact(FORM_WORDS).enumerate() {
            assert_eq!(find(&flat.forms, FORM_WORDS, key), Some(index));
        }
        for (index, key) in flat.row_keys.chunks_exact(KEY_WORDS).enumerate() {
            assert_eq!(find(&flat.row_keys, KEY_WORDS, key), Some(index));
            let mut other = key.to_vec();
            other[2] ^= 0x4000_0000;
            assert_eq!(find(&flat.row_keys, KEY_WORDS, &other), None);
        }
        Ok(())
    }

    /// A reader keyed a kept token at a time by the trunk is laid out with
    /// one trunk code a token and every form marked; one keyed by a single
    /// frame is refused.
    #[test]
    fn a_reader_keyed_by_trunk_codes_is_laid_out_with_them() -> Result<()> {
        use crate::conjunction::trunk_key::{tests::toy, Keyed};
        let plain = Flat::of(&train(READS, StandBy::Pair)?)?;
        assert!(!plain.trunk && plain.codes.is_empty());
        for by in [Keyed::Path, Keyed::Address] {
            let key = toy(64).keyed(by)?;
            let flat = Flat::of(&train_beside(READS, StandBy::Pair, Some(key.clone()))?)?;
            assert!(flat.trunk);
            assert_eq!(flat.codes.len(), 64);
            assert_eq!(flat.codes[5], key.code(5).expect("a code"));
            assert_eq!(flat.form_count(), plain.form_count());
            assert!(flat
                .forms
                .chunks_exact(FORM_WORDS)
                .all(|form| form[13] & 0x20 != 0));
            for (index, form) in flat.forms.chunks_exact(FORM_WORDS).enumerate() {
                assert_eq!(find(&flat.forms, FORM_WORDS, form), Some(index));
            }
        }
        assert!(Flat::of(&train_beside(READS, StandBy::Pair, Some(toy(64)))?).is_err());
        Ok(())
    }

    #[test]
    fn a_reader_of_other_read_choices_is_refused() -> Result<()> {
        assert!(Flat::of(&train(READS, StandBy::Latest)?).is_err());
        assert!(Flat::of(&train(Reads::default(), StandBy::Pair)?).is_err());
        let explain = Reads {
            explain: true,
            ..READS
        };
        assert!(Flat::of(&train(explain, StandBy::Pair)?).is_err());
        Ok(())
    }
}
