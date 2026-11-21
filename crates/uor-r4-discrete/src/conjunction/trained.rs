//! The host's part of TRAIN on the GPU (`reader_train.cu`): the order the
//! kernels work in, and the reader their tables make.
//!
//! The kernels compute every row. The host only lays the work out: replies
//! as runs of adjacent TRAIN targets, each pair's targets in stream order,
//! and the levels held rows are written in. Two replies can write the same
//! held row only when they hold a form in common, so a reply's level is one
//! past the highest level of an earlier reply that holds one of its forms;
//! the replies of one level are written at once and the levels in order.
//!
//! Laid out for the choices of the saved M2 readers alone: no class codes,
//! no suffix orders above the pair, the pair stand, slots ranked `precise`,
//! a place answered `across` its forms.
use super::bind::BindTable;
use super::flat::{FORM_WORDS, KEY_WORDS, ROW_WORDS, SLOT_WORDS};
use super::saved::{entry_of, Reader};
use super::{invalid, window_at, Asked, Form, HeldKey, HeldRow, Ladder, Reads, Row, StandBy};
use super::{MAX_SLOTS, OPEN_HEAD, OPEN_HEAD_LEAST, OPEN_TAIL};
use crate::Result;

/// A pair row: eight pointers, eight counts, the two trust counts.
pub const PAIR_WORDS: usize = 2 * MAX_SLOTS + 2;
/// A job: the reply's start and end, its form count and six form ids.
pub const JOB_WORDS: usize = 9;
/// A reply's kept forms: token forms (count, six ids), open forms the same.
pub const RUN_WORDS: usize = 14;
/// Most forms one turn is counted under: the whole turn, every gap of one
/// to twelve tokens in thirty, and every open form.
pub const FORMS_A_TURN: usize = 1 + 12 * 30 + (OPEN_HEAD - OPEN_HEAD_LEAST + 1) * (OPEN_TAIL + 1);

/// The read choices these tables are laid out for.
pub const READS: Reads = Reads {
    precise: true,
    explain: false,
    tail_row: false,
    across: true,
    agree: false,
};

/// Replies: runs of adjacent TRAIN targets, as `[start, end)`.
pub fn runs(eligible: &[usize]) -> Vec<[usize; 2]> {
    let mut runs: Vec<[usize; 2]> = Vec::new();
    for &position in eligible {
        match runs.last_mut() {
            Some(run) if run[1] == position => run[1] = position + 1,
            _ => runs.push([position, position + 1]),
        }
    }
    runs
}

/// Every pair's TRAIN targets in stream order. `keys` rises; pair `p` owns
/// `positions[offsets[p]..offsets[p + 1]]`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pairs {
    /// `4096 * a + b` for the pair `a`, `b` (`b` the nearer token).
    pub keys: Vec<u32>,
    pub offsets: Vec<u32>,
    pub positions: Vec<u32>,
}

/// Groups the targets a pair row is written from: those with at least two
/// tokens in their window (`Ladder::write`).
pub fn pairs(tokens: &[u32], eligible: &[usize], window: usize, bos: u32) -> Pairs {
    let mut keyed: Vec<(u32, u32)> = eligible
        .iter()
        .filter_map(
            |&position| match *window_at(tokens, position, window, bos) {
                [.., a, b] => Some((a * 4096 + b, position as u32)),
                _ => None,
            },
        )
        .collect();
    keyed.sort_unstable();
    let mut found = Pairs::default();
    for (key, position) in keyed {
        if found.keys.last() != Some(&key) {
            found.keys.push(key);
            found.offsets.push(found.positions.len() as u32);
        }
        found.positions.push(position);
    }
    found.offsets.push(found.positions.len() as u32);
    found
}

/// The jobs of `runs`: each reply once under its token forms and once under
/// its open forms, where it has any (`Asked::write_under`).
pub fn jobs(runs: &[[usize; 2]], run_forms: &[u32]) -> Result<Vec<u32>> {
    if run_forms.len() != runs.len() * RUN_WORDS {
        return Err(invalid("one row of kept forms a reply"));
    }
    let mut jobs = Vec::new();
    for (run, forms) in runs.iter().zip(run_forms.chunks_exact(RUN_WORDS)) {
        for list in forms.chunks_exact(RUN_WORDS / 2) {
            if list[0] == 0 {
                continue;
            }
            jobs.extend([run[0] as u32, run[1] as u32]);
            jobs.extend(list);
        }
    }
    Ok(jobs)
}

/// The levels the jobs listed in `subset` are written in: the job indexes
/// level by level (in `subset`'s order inside a level), and each level's
/// first place and count in that list.
pub fn levels(jobs: &[u32], subset: &[u32], forms: usize) -> Result<(Vec<u32>, Vec<[u32; 2]>)> {
    // One past the level of the latest job that held the form.
    let mut after = vec![0u32; forms];
    let mut level_of = Vec::with_capacity(subset.len());
    let mut sizes: Vec<u32> = Vec::new();
    for &job in subset {
        let at = job as usize * JOB_WORDS;
        let words = jobs
            .get(at..at + JOB_WORDS)
            .ok_or_else(|| invalid("a job index past the jobs"))?;
        let held = words
            .get(3..3 + words[2] as usize)
            .ok_or_else(|| invalid("a job holds at most six forms"))?;
        let mut level = 0u32;
        for &form in held {
            let seen = after
                .get(form as usize)
                .ok_or_else(|| invalid("a job's form id is past the kept forms"))?;
            level = level.max(*seen);
        }
        for &form in held {
            after[form as usize] = level + 1;
        }
        if sizes.len() <= level as usize {
            sizes.resize(level as usize + 1, 0);
        }
        sizes[level as usize] += 1;
        level_of.push(level);
    }
    let mut bounds = Vec::with_capacity(sizes.len());
    let mut first = 0u32;
    for &size in &sizes {
        bounds.push([first, size]);
        first += size;
    }
    let mut next: Vec<u32> = bounds.iter().map(|bound| bound[0]).collect();
    let mut order = vec![0u32; subset.len()];
    for (&job, &level) in subset.iter().zip(&level_of) {
        order[next[level as usize] as usize] = job;
        next[level as usize] += 1;
    }
    Ok((order, bounds))
}

/// What the kernels leave: forms in id order, every held row with every
/// slot (in no order), and the pair rows in key order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tables {
    pub forms: Vec<u32>,
    pub row_keys: Vec<u32>,
    pub row_slots: Vec<u32>,
    pub pair_keys: Vec<u32>,
    pub pair_rows: Vec<u32>,
}

fn wide(words: &[u32]) -> u128 {
    words
        .iter()
        .fold(0u128, |wide, &word| (wide << 32) | u128::from(word))
}

impl Tables {
    /// The reader these tables are, as `Reader::train` would hold it.
    pub fn reader(
        &self,
        vocabulary: u32,
        window: usize,
        bos: u32,
        eos: u32,
        slots: usize,
        open: bool,
    ) -> Result<Reader> {
        if self.forms.len() % FORM_WORDS != 0
            || self.row_keys.len() % KEY_WORDS != 0
            || self.row_slots.len() != self.row_keys.len() / KEY_WORDS * ROW_WORDS
            || self.pair_rows.len() != self.pair_keys.len() * PAIR_WORDS
        {
            return Err(invalid("the trained tables are not of their stated shape"));
        }
        let mut ladder = Ladder::new(vocabulary, slots, &[])?;
        for (&key, words) in self
            .pair_keys
            .iter()
            .zip(self.pair_rows.chunks_exact(PAIR_WORDS))
        {
            let address = ladder
                .pair
                .address(key >> 12, key & 4095)
                .ok_or_else(|| invalid("a pair key past the vocabulary"))?;
            let mut row = Row::default();
            row.pointer.copy_from_slice(&words[..MAX_SLOTS]);
            row.count.copy_from_slice(&words[MAX_SLOTS..2 * MAX_SLOTS]);
            row.trust = [words[2 * MAX_SLOTS], words[2 * MAX_SLOTS + 1]];
            if ladder.pair.rows.insert(address, row).is_some() {
                return Err(invalid("a pair key stands twice"));
            }
        }
        let mut asked = Asked::new(window, bos, eos)
            .with_stand(StandBy::Pair, BindTable::default())?
            .with_open_forms(open)
            .with_reads(READS);
        for (id, words) in self.forms.chunks_exact(FORM_WORDS).enumerate() {
            let form = Form {
                words: [wide(&words[..4]), wide(&words[4..8]), wide(&words[8..12])],
                gap_at: u8::try_from(words[12]).map_err(|_| invalid("a form's gap place"))?,
                gap_len: u8::try_from(words[13]).map_err(|_| invalid("a form's gap length"))?,
            };
            if asked.seen.insert(form, id as u32).is_some() {
                return Err(invalid("a form stands twice"));
            }
        }
        let forms = asked.seen.len() as u32;
        for (key, slots) in self
            .row_keys
            .chunks_exact(KEY_WORDS)
            .zip(self.row_slots.chunks_exact(ROW_WORDS))
        {
            if key[0] >= forms || key[1] > 1 {
                return Err(invalid("a held row under no kept form"));
            }
            let key = HeldKey {
                form: key[0],
                empty: key[1] == 1,
                tail: key[2],
                history: [wide(&key[3..7]), wide(&key[7..11])],
            };
            let mut row = HeldRow::default();
            for (slot, words) in slots.chunks_exact(SLOT_WORDS).enumerate() {
                if words[5] == 0 {
                    continue;
                }
                row.entry[slot] = entry_of(
                    [words[0], words[1], words[2], words[3], words[4]],
                    vocabulary,
                )?;
                row.count[slot] = words[5];
                row.wrong[slot] = words[6];
            }
            if asked.rows.insert(key, row).is_some() {
                return Err(invalid("a held row stands twice"));
            }
        }
        Ok(Reader {
            vocabulary,
            ladder,
            asked,
        })
    }
}

/// Where two readers differ, map by map, every slot of every row compared
/// (a file holds only the slots a read can reach).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Differences {
    pub forms_not_in_the_other: u64,
    pub held_rows_missing: u64,
    pub held_rows_added: u64,
    pub held_rows_of_other_slots: u64,
    pub pair_rows_missing: u64,
    pub pair_rows_added: u64,
    pub pair_rows_of_other_slots: u64,
}

/// `ours` against `theirs`: rows `theirs` holds and `ours` does not are
/// missing, rows only `ours` holds are added.
pub fn differences(ours: &Reader, theirs: &Reader) -> Differences {
    let mut found = Differences::default();
    let (mine, other) = (&ours.asked.seen, &theirs.asked.seen);
    found.forms_not_in_the_other = (mine
        .iter()
        .filter(|&(form, id)| other.get(form) != Some(id))
        .count()
        + other
            .iter()
            .filter(|&(form, id)| mine.get(form) != Some(id))
            .count()) as u64;
    for (key, row) in &theirs.asked.rows {
        match ours.asked.rows.get(key) {
            None => found.held_rows_missing += 1,
            Some(held) if held != row => found.held_rows_of_other_slots += 1,
            Some(_) => {}
        }
    }
    found.held_rows_added = ours
        .asked
        .rows
        .keys()
        .filter(|key| !theirs.asked.rows.contains_key(key))
        .count() as u64;
    for (address, row) in &theirs.ladder.pair.rows {
        match ours.ladder.pair.rows.get(address) {
            None => found.pair_rows_missing += 1,
            Some(held) if held != row => found.pair_rows_of_other_slots += 1,
            Some(_) => {}
        }
    }
    found.pair_rows_added = ours
        .ladder
        .pair
        .rows
        .keys()
        .filter(|address| !theirs.ladder.pair.rows.contains_key(address))
        .count() as u64;
    found
}

#[cfg(test)]
mod tests {
    use super::super::saved::entry_fields;
    use super::*;

    const VOCABULARY: u32 = 64;

    fn stream() -> (Vec<u32>, Vec<usize>) {
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
        (tokens, eligible)
    }

    fn train() -> Result<Reader> {
        let (tokens, eligible) = stream();
        let (reader, _) = Reader::train(
            &tokens,
            &eligible,
            VOCABULARY,
            192,
            0,
            1,
            8,
            &[],
            8,
            &[],
            StandBy::Pair,
            BindTable::default(),
            true,
            READS,
        )?;
        Ok(reader)
    }

    fn quarters(word: u128) -> [u32; 4] {
        [96u32, 64, 32, 0].map(|shift| (word >> shift) as u32)
    }

    /// The host reader's maps as the kernels would leave them.
    fn tables(reader: &Reader) -> Tables {
        let mut found = Tables::default();
        let mut forms: Vec<(u32, [u32; FORM_WORDS])> =
            reader.asked.forms_by_id().into_iter().collect();
        forms.sort_unstable();
        for (_, words) in forms {
            found.forms.extend(words);
        }
        for (key, row) in &reader.asked.rows {
            found
                .row_keys
                .extend([key.form, u32::from(key.empty), key.tail]);
            found.row_keys.extend(quarters(key.history[0]));
            found.row_keys.extend(quarters(key.history[1]));
            for slot in 0..MAX_SLOTS {
                if row.count[slot] == 0 {
                    found.row_slots.extend([0; SLOT_WORDS]);
                    continue;
                }
                found.row_slots.extend(entry_fields(row.entry[slot]));
                found.row_slots.extend([row.count[slot], row.wrong[slot]]);
            }
        }
        for a in 0..VOCABULARY {
            for b in 0..VOCABULARY {
                let held = reader
                    .ladder
                    .pair
                    .address(a, b)
                    .and_then(|address| reader.ladder.pair.rows.get(&address));
                if let Some(row) = held {
                    found.pair_keys.push(a * 4096 + b);
                    found.pair_rows.extend(row.pointer);
                    found.pair_rows.extend(row.count);
                    found.pair_rows.extend(row.trust);
                }
            }
        }
        found
    }

    /// Tables of every row and slot make the reader they were taken from,
    /// map for map and byte for byte; one changed count is told apart.
    #[test]
    fn tables_make_the_reader_they_were_taken_from() -> Result<()> {
        let host = train()?;
        let mut held = tables(&host);
        let ours = held.reader(VOCABULARY, 192, 0, 1, 8, true)?;
        assert_eq!(differences(&ours, &host), Differences::default());
        let (ours_bytes, _) = ours.to_bytes(1);
        let (host_bytes, _) = host.to_bytes(1);
        assert_eq!(ours_bytes, host_bytes);
        assert!(!host.asked.rows.is_empty() && !host.ladder.pair.rows.is_empty());

        let slot = held
            .row_slots
            .chunks_exact(SLOT_WORDS)
            .position(|slot| slot[5] >= 2)
            .ok_or_else(|| invalid("no read slot"))?;
        held.row_slots[slot * SLOT_WORDS + 5] += 1;
        let other = held.reader(VOCABULARY, 192, 0, 1, 8, true)?;
        let found = differences(&other, &host);
        assert_eq!(found.held_rows_of_other_slots, 1);
        assert_ne!(other.to_bytes(1), host.to_bytes(1));
        Ok(())
    }

    /// The pair groups are the pair rows the host ladder wrote, and each
    /// group's targets rise.
    #[test]
    fn pair_groups_are_the_rows_the_ladder_wrote() -> Result<()> {
        let (tokens, eligible) = stream();
        let host = train()?;
        let found = pairs(&tokens, &eligible, 192, 0);
        assert_eq!(found.keys.len(), host.ladder.pair.rows.len());
        assert!(found.keys.windows(2).all(|pair| pair[0] < pair[1]));
        for (index, &key) in found.keys.iter().enumerate() {
            let address = host.ladder.pair.address(key >> 12, key & 4095);
            assert!(address.is_some_and(|address| host.ladder.pair.rows.contains_key(&address)));
            let group =
                &found.positions[found.offsets[index] as usize..found.offsets[index + 1] as usize];
            assert!(!group.is_empty() && group.windows(2).all(|pair| pair[0] < pair[1]));
        }
        assert_eq!(found.positions.len(), eligible.len());
        assert_eq!(runs(&eligible).len(), 16);
        Ok(())
    }

    /// Jobs that hold a form in common stand in rising levels in their own
    /// order; jobs that share none stand in one level.
    #[test]
    fn jobs_that_share_a_form_are_levelled_in_order() -> Result<()> {
        let job = |forms: &[u32]| {
            let mut words = vec![0u32, 1, forms.len() as u32];
            words.extend(forms);
            words.resize(JOB_WORDS, 0);
            words
        };
        let jobs: Vec<u32> = [
            job(&[0, 1]),
            job(&[2]),
            job(&[1, 3]),
            job(&[3]),
            job(&[2, 4]),
        ]
        .concat();
        let (order, bounds) = levels(&jobs, &[0, 1, 2, 3, 4], 5)?;
        assert_eq!(order, [0, 1, 2, 4, 3]);
        assert_eq!(bounds, [[0, 2], [2, 2], [4, 1]]);
        // The same jobs the other way round are levelled the other way.
        let (order, bounds) = levels(&jobs, &[4, 3, 2, 1, 0], 5)?;
        assert_eq!(order, [4, 3, 2, 1, 0]);
        assert_eq!(bounds, [[0, 2], [2, 2], [4, 1]]);
        let (order, bounds) = levels(&jobs, &[1, 3], 5)?;
        assert_eq!((order, bounds), (vec![1, 3], vec![[0, 2]]));
        assert!(levels(&jobs, &[0], 1).is_err());
        Ok(())
    }
}
