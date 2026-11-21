//! Order-free TRAIN of the held rows.
//!
//! [`Asked::write`] writes each reply in turn and reads the rows the replies
//! before it left, so its result depends on the order of TRAIN and no two
//! replies under one form can be written at once. Here every reply of a pass
//! is counted against the rows of the pass before it (none for the first), so
//! a pass is three steps whose sums do not depend on any order:
//!
//! 1. **Offer.** At every reply place each row the place is written under is
//!    offered the target token and each read that names it (the next-token
//!    read, the copy read, the first pointers), one count an offer.
//! 2. **Keep.** A row keeps the [`MAX_SLOTS`] slots offered most often (a read
//!    before a token on a tie, then the lower code).
//! 3. **Count.** At every place of a row each kept slot is counted right when
//!    it names the target and, under [`super::Reads::precise`], wrong when it
//!    names another token.
//!
//! The reply so far and the place a copy goes on from are those the rows of
//! the pass before give where they are right, as in [`Asked::write`]; where
//! they are wrong or absent, the first read that names the target.
//!
//! Every step is an integer compare, add or table lookup. The work is split
//! over the machine's threads; the rows are the same for any number of them.
use super::{
    window_at, Asked, Entry, HeldKey, HeldRow, History, Ladder, Place, ANY_HISTORY, MAX_SLOTS,
    POINTERS_ADDED, POINTER_AHEAD, POINTER_BACK,
};
use std::cmp::Reverse;
use std::collections::HashMap;

/// How TRAIN writes the held rows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Update {
    /// Each reply in turn, reading the rows written so far.
    #[default]
    InOrder,
    /// Every reply counted against the rows of the pass before, in any
    /// order, this many passes.
    Free { passes: usize },
}

/// A reply to count: the forms it stands under and its place in TRAIN.
#[derive(Debug, Clone)]
struct Job {
    forms: Vec<u32>,
    run: usize,
    start: usize,
    end: usize,
}

/// A slot offered to a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Offer {
    key: HeldKey,
    entry: Entry,
}

/// Times each kept slot of a row was right and wrong.
#[derive(Debug, Clone, Copy, Default)]
struct Tally {
    right: [u32; MAX_SLOTS],
    wrong: [u32; MAX_SLOTS],
}

/// The order of slots offered equally often: pointers, the next-token read,
/// the copy read, then tokens by id.
fn rank(entry: Entry) -> u64 {
    match entry {
        Entry::Pointer {
            back,
            ahead,
            wide,
            before,
        } => {
            (u64::from(back) << 32)
                | (u64::from(ahead) << 24)
                | (u64::from(wide) << 16)
                | u64::from(before)
        }
        Entry::Next => 1 << 40,
        Entry::Copy => 2 << 40,
        Entry::Token(token) => (3 << 40) | u64::from(token),
        Entry::Empty => u64::MAX,
    }
}

/// The reads that name `target` at this place, in the order
/// [`HeldRow::take`] adds them: the next-token read, the copy read, then at
/// most [`POINTERS_ADDED`] pointers.
fn right_reads(at: &Place, target: u32) -> Vec<Entry> {
    let right = |entry: Entry| entry.name(at).is_some_and(|named| named.token == target);
    let mut reads: Vec<Entry> = [Entry::Next, Entry::Copy]
        .into_iter()
        .filter(|&entry| right(entry))
        .collect();
    reads.extend(
        (0..POINTER_BACK)
            .flat_map(|back| {
                (0..=POINTER_AHEAD).flat_map(move |ahead| {
                    [true, false].map(move |wide| (back as u8, ahead as u8, wide))
                })
            })
            .filter(|&(_, ahead, wide)| !(ahead == 0 && wide))
            .filter_map(|(back, ahead, wide)| Entry::pointer(at, back, ahead, wide, target))
            .filter(|&entry| right(entry))
            .take(POINTERS_ADDED),
    );
    reads
}

/// `work` over `jobs` split across the machine's threads, one result a part.
fn in_parts<J: Sync, T: Send>(jobs: &[J], work: impl Fn(&[J]) -> T + Sync) -> Vec<T> {
    let threads = std::thread::available_parallelism().map_or(1, usize::from);
    let size = jobs.len().div_ceil(threads).max(1);
    let work = &work;
    std::thread::scope(|scope| {
        let parts: Vec<_> = jobs
            .chunks(size)
            .map(|part| scope.spawn(move || work(part)))
            .collect();
        parts
            .into_iter()
            .map(|part| {
                part.join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            })
            .collect()
    })
}

impl Asked {
    /// Every place of a reply with its target, the rows it is written under
    /// and the reads that name the target, reading `self.rows` and writing
    /// nothing.
    fn walk_free(
        &self,
        job: &Job,
        tokens: &[u32],
        empty: bool,
        each: &mut dyn FnMut(&Place, u32, &[HeldKey], &[Entry]),
    ) {
        let stand = &*self.stand;
        let mut cursor = None;
        let mut history = History::default();
        let mut keys = Vec::new();
        for position in job.start..job.end {
            let target = tokens[position];
            let at = self.place(stand, &tokens[..position], job.start, cursor);
            let read = self.answer(&job.forms, &history, &at, empty, &mut false);
            let (own, shared) = history.tails(at.ahead());
            let words = history.address();
            let by_tail = (stand.reads.tail_row && shared.is_some()).then_some((ANY_HISTORY, own));
            let tails = [
                Some((words, own)),
                shared.map(|tail| (words, tail)),
                by_tail,
            ];
            keys.clear();
            for &form in &job.forms {
                for (history, tail) in tails.into_iter().flatten() {
                    keys.push(HeldKey {
                        form,
                        history,
                        tail,
                        empty,
                    });
                }
            }
            let reads = right_reads(&at, target);
            each(&at, target, &keys, &reads);
            let written = reads
                .iter()
                .find_map(|entry| entry.name(&at).and_then(|named| named.source));
            cursor = match read {
                Some((named, _, _)) if named.token == target => named.source,
                _ => written,
            };
            history.push(target, cursor);
        }
    }

    /// One pass: the rows `jobs` leave, counted against `self.rows`.
    fn count_free(&self, jobs: &[Job], tokens: &[u32], empty: bool) -> HashMap<HeldKey, HeldRow> {
        let precise = self.stand.reads.precise;
        // Offer.
        let parts = in_parts(jobs, |part| {
            let mut offered: HashMap<Offer, u32> = HashMap::new();
            for job in part {
                self.walk_free(job, tokens, empty, &mut |_, target, keys, reads| {
                    for &key in keys {
                        let token = Entry::Token(target);
                        for &entry in std::iter::once(&token).chain(reads) {
                            *offered.entry(Offer { key, entry }).or_default() += 1;
                        }
                    }
                });
            }
            offered
        });
        let mut offered: HashMap<Offer, u32> = HashMap::new();
        for part in parts {
            for (offer, count) in part {
                *offered.entry(offer).or_default() += count;
            }
        }
        // Keep.
        let mut kept: HashMap<HeldKey, Vec<(Entry, u32)>> = HashMap::new();
        for (Offer { key, entry }, count) in offered {
            kept.entry(key).or_default().push((entry, count));
        }
        for slots in kept.values_mut() {
            slots.sort_by_key(|&(entry, count)| (Reverse(count), rank(entry)));
            slots.truncate(MAX_SLOTS);
        }
        // Count.
        let kept = &kept;
        let parts = in_parts(jobs, |part| {
            let mut tallies: HashMap<HeldKey, Tally> = HashMap::new();
            let mut names: HashMap<Entry, Option<u32>> = HashMap::new();
            for job in part {
                self.walk_free(job, tokens, empty, &mut |at, target, keys, _| {
                    names.clear();
                    for key in keys {
                        let Some(slots) = kept.get(key) else { continue };
                        let tally = tallies.entry(*key).or_default();
                        for (slot, &(entry, _)) in slots.iter().enumerate() {
                            let named = *names
                                .entry(entry)
                                .or_insert_with(|| entry.name(at).map(|named| named.token));
                            match named {
                                Some(token) if token == target => tally.right[slot] += 1,
                                Some(_) if precise => tally.wrong[slot] += 1,
                                _ => {}
                            }
                        }
                    }
                });
            }
            tallies
        });
        let mut tallies: HashMap<HeldKey, Tally> = HashMap::new();
        for part in parts {
            for (key, tally) in part {
                let sum = tallies.entry(key).or_default();
                for slot in 0..MAX_SLOTS {
                    sum.right[slot] += tally.right[slot];
                    sum.wrong[slot] += tally.wrong[slot];
                }
            }
        }
        let mut rows = HashMap::with_capacity(kept.len());
        for (key, slots) in kept {
            let tally = tallies.get(key).copied().unwrap_or_default();
            let mut live: Vec<(Entry, u32, u32)> = slots
                .iter()
                .enumerate()
                .map(|(slot, &(entry, _))| (entry, tally.right[slot], tally.wrong[slot]))
                .filter(|&(_, right, _)| right > 0)
                .collect();
            live.sort_by_key(|&(entry, right, _)| (Reverse(right), rank(entry)));
            let mut row = HeldRow::default();
            for (slot, (entry, right, wrong)) in live.into_iter().enumerate() {
                row.entry[slot] = entry;
                row.count[slot] = right;
                row.wrong[slot] = wrong;
            }
            rows.insert(*key, row);
        }
        rows
    }

    /// The order-free second and third passes over the replies `runs`
    /// (`[start, end]` each): `passes` counts of every reply, each against the
    /// rows of the one before, then one count of the replies whose own
    /// generated reply met an empty read, under the rows for that case.
    /// Returns the replies that stood under a form and those with an empty
    /// read.
    pub fn write_free(
        &mut self,
        ladder: &Ladder,
        tokens: &[u32],
        runs: &[[usize; 2]],
        limit: usize,
        passes: usize,
    ) -> (u64, u64) {
        let mut jobs = Vec::new();
        for (run, &[start, end]) in runs.iter().enumerate() {
            for forms in [
                self.held(tokens, start),
                self.held_classes(tokens, start),
                self.held_open(tokens, start),
            ] {
                if !forms.is_empty() {
                    jobs.push(Job {
                        forms,
                        run,
                        start,
                        end,
                    });
                }
            }
        }
        let distinct = |jobs: &[Job]| {
            let mut runs: Vec<usize> = jobs.iter().map(|job| job.run).collect();
            runs.dedup();
            runs.len() as u64
        };
        let written = distinct(&jobs);
        for _ in 0..passes.max(1) {
            self.rows = self.count_free(&jobs, tokens, false);
        }
        let this = &*self;
        let missed: Vec<Job> = in_parts(&jobs, |part| {
            part.iter()
                .filter(|job| {
                    // The reader sees the prompt's own window, not the corpus
                    // before it.
                    let seen = window_at(tokens, job.start, this.context, this.bos).len();
                    let prompt = &tokens[job.start - seen..job.start];
                    let (_, _, missed) =
                        this.generate(ladder, &job.forms, prompt, limit, false, None);
                    missed
                })
                .cloned()
                .collect::<Vec<Job>>()
        })
        .into_iter()
        .flatten()
        .collect();
        let empty_reads = distinct(&missed);
        let rows = self.count_free(&missed, tokens, true);
        self.rows.extend(rows);
        (written, empty_reads)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;

    /// Told:   BOS x IS v DOT EOS WHAT x QM MARK | YOU GREW v DOT EOS
    /// Untold: BOS EOS WHAT x QM MARK | YOU NO EOS
    /// A new x and v each record; the form "WHAT ▢ QM MARK" is in all.
    fn told(x: u32, v: u32) -> Vec<u32> {
        vec![0, x, 2, v, 3, 1, 4, x, 5, 6, 8, 9, v, 3, 1]
    }
    fn untold(x: u32) -> Vec<u32> {
        vec![0, 1, 4, x, 5, 6, 8, 7, 1]
    }
    fn train() -> (Vec<u32>, Vec<[usize; 2]>) {
        let mut tokens = Vec::new();
        let mut runs = Vec::new();
        for index in 0..12u32 {
            let at = tokens.len();
            tokens.extend(told(20 + index, 40 + index));
            runs.push([at + 10, at + 15]);
        }
        for index in 0..9u32 {
            let at = tokens.len();
            tokens.extend(untold(32 + index));
            runs.push([at + 6, at + 9]);
        }
        (tokens, runs)
    }
    fn counted(tokens: &[u32], runs: &[[usize; 2]], passes: usize) -> Result<(Asked, Ladder, u64)> {
        let ladder = Ladder::new(64, 2, &[3])?;
        let mut asked = Asked::new(192, 0, 1);
        for run in runs {
            asked.count(tokens, run[0]);
        }
        asked.seal();
        let (written, met) = asked.write_free(&ladder, tokens, runs, 8, passes);
        assert_eq!(written, runs.len() as u64);
        Ok((asked, ladder, met))
    }

    #[test]
    fn the_rows_do_not_depend_on_the_order_of_the_replies() -> Result<()> {
        let (tokens, runs) = train();
        let (first, _, _) = counted(&tokens, &runs, 2)?;
        let mut turned = runs.clone();
        turned.reverse();
        turned.rotate_left(5);
        let (second, _, _) = counted(&tokens, &turned, 2)?;
        assert!(first.rows() > 0);
        assert_eq!(first.rows, second.rows);
        // The in-order TRAIN of the same replies is another set of rows.
        let mut in_order = Asked::new(192, 0, 1);
        for run in &runs {
            in_order.count(&tokens, run[0]);
        }
        in_order.seal();
        for run in &runs {
            in_order.write(&tokens, run[0], run[1]);
        }
        assert_ne!(first.rows, in_order.rows);
        Ok(())
    }

    #[test]
    fn a_counted_form_answers_a_name_it_never_saw_and_an_untold_fact() -> Result<()> {
        let (tokens, runs) = train();
        let (asked, ladder, met) = counted(&tokens, &runs, 2)?;
        // The nine untold replies begin as the commoner told reply and meet
        // an empty read; they are counted again under the rows for it.
        assert_eq!(met, 9);
        let prompt = told(60, 61)[..10].to_vec();
        assert_eq!(
            asked.reply(&ladder, &prompt, 8),
            (vec![8, 9, 61, 3, 1], true)
        );
        let asked_untold = untold(60)[..6].to_vec();
        assert_eq!(
            asked.reply(&ladder, &asked_untold, 8),
            (vec![8, 7, 1], true)
        );
        Ok(())
    }

    #[test]
    fn kept_slots_stand_first_in_a_row_most_often_right_first() -> Result<()> {
        let (tokens, runs) = train();
        let (asked, _, _) = counted(&tokens, &runs, 1)?;
        for row in asked.rows.values() {
            let live = row.count.iter().take_while(|&&count| count > 0).count();
            assert!(live > 0);
            assert!(row.count[live..].iter().all(|&count| count == 0));
            assert!(row.count[..live].windows(2).all(|pair| pair[0] >= pair[1]));
        }
        Ok(())
    }
}
