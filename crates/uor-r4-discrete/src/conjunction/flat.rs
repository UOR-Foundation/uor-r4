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
    /// The ladder's pair rows that have an output, `4096 * a + b` for the
    /// pair `a`, `b` (`b` the nearer token), rising.
    pub pair_keys: Vec<u32>,
    /// [`PAIR_READ_WORDS`] a pair row: its output and its two trust counts.
    pub pair_reads: Vec<u32>,
    /// The ladder is the pair row and the copy read alone (no suffix order
    /// above the pair), so a place no held row answers can be read from
    /// `pair_keys`.
    pub ladder: bool,
}

/// A pair row as a read needs it: output, copy-read trust, row trust.
pub const PAIR_READ_WORDS: usize = 3;

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
        // The pair rows by the pair itself: the host's address is a product
        // of two primes, which a read here never needs. Every pair is asked
        // for in key order, so the table rises as built.
        let pair = &reader.ladder.pair;
        let mut pair_keys = Vec::new();
        let mut pair_reads = Vec::new();
        if reader.vocabulary <= 4_096 {
            for a in 0..reader.vocabulary {
                for b in 0..reader.vocabulary {
                    let Some(row) = pair.read(a, b) else {
                        continue;
                    };
                    let Some(output) = row.output() else {
                        continue;
                    };
                    pair_keys.push(4_096 * a + b);
                    pair_reads.extend([output, row.trust[0], row.trust[1]]);
                }
            }
        }
        let ladder = reader.ladder.orders.is_empty() && reader.vocabulary <= 4_096;
        if !rises(&forms, FORM_WORDS) || !rises(&row_keys, KEY_WORDS) || !rises(&pair_keys, 1) {
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
            pair_keys,
            pair_reads,
            ladder,
        })
    }
    /// The ladder's next id after `window` from the pair table, as the read
    /// kernel takes it (`ladder_next` of `reader_read.cu`): the pair row's
    /// output, against the copy read where the two differ and the row trusts
    /// the copy more; the copy read alone where the pair has no row.
    pub fn ladder_next(&self, window: &[u32]) -> Option<u32> {
        let copy = super::copy_at(window);
        if let [.., a, b] = *window {
            if let Some(found) = find(&self.pair_keys, 1, &[4_096 * a + b]) {
                let row = &self.pair_reads[found * PAIR_READ_WORDS..][..PAIR_READ_WORDS];
                return Some(match copy {
                    Some(copy) if copy != row[0] && row[1] > row[2] => copy,
                    _ => row[0],
                });
            }
        }
        copy
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

    /// The pair table holds every ladder pair row that has an output, and
    /// the next id read from it is the host ladder's at every window of a
    /// stream that repeats pairs with different followers (so the copy read
    /// and the row output differ at some places and the trust counts
    /// decide). A table with its trust counts swapped reads another id
    /// somewhere, so the comparison can fail.
    #[test]
    fn the_pair_table_reads_what_the_host_ladder_reads() -> Result<()> {
        let reader = train(READS, StandBy::Pair)?;
        let flat = Flat::of(&reader)?;
        assert!(flat.ladder && !flat.pair_keys.is_empty());
        assert_eq!(
            flat.pair_reads.len(),
            flat.pair_keys.len() * PAIR_READ_WORDS
        );
        let with_output = reader
            .ladder
            .pair
            .rows
            .values()
            .filter(|row| row.output().is_some())
            .count();
        assert_eq!(flat.pair_keys.len(), with_output);
        let mut stream = Vec::new();
        for index in 0..40u32 {
            let value = if index % 3 == 0 { 9 } else { 20 + index % 5 };
            stream.extend([0, 2, 3, value, 4, 1, 5, 2, 6, value, 4, 1]);
        }
        for end in 0..=stream.len() {
            let window = crate::conjunction::window_at(&stream, end, 192, 0);
            let host = reader.ladder.next(window, true);
            assert_eq!(flat.ladder_next(window), host, "window ending at {end}");
        }
        // The pair "name mark" has a row whose output is 9; here it stood
        // earlier before 30, so the copy read names 30. TRAIN never saw the
        // two differ, so the row has no trust count and its output stands.
        let differs = [0u32, 2, 6, 30, 2, 6];
        assert_eq!(crate::conjunction::copy_at(&differs), Some(30));
        assert_eq!(reader.ladder.next(&differs, false), Some(9));
        assert_eq!(reader.ladder.next(&differs, true), Some(9));
        assert_eq!(flat.ladder_next(&differs), Some(9));
        // The rule itself: the copy read answers only where the row trusts
        // it more than its own output.
        let at = find(&flat.pair_keys, 1, &[4_096 * 2 + 6]).expect("the pair row");
        for (trust, next) in [([1, 0], 30), ([1, 1], 9), ([0, 1], 9)] {
            let mut trusted = flat.clone();
            trusted.pair_reads[at * PAIR_READ_WORDS + 1..][..2].copy_from_slice(&trust);
            assert_eq!(trusted.ladder_next(&differs), Some(next));
        }
        // No pair row: the copy read alone, or nothing.
        assert_eq!(flat.ladder_next(&[0, 40, 41, 42, 40, 41]), Some(42));
        assert_eq!(flat.ladder_next(&[0, 40, 41]), None);
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
