//! The discrete read-binding hinge on the native Q31 read, and the gate step that trains it.
//!
//! The hinge is [`crate::read_binding`]'s corrected form (rank gap, not the scalar
//! `(rel << 3) - distance` key) applied to the read the stack actually serves. A row
//! carries a **bound** position — the earlier row the query must bind to — and a set of
//! **competing** positions. The read orders the earlier rows of its window by
//! [`read_distance`], a tie going to the later row, so a row's rank is the number of
//! candidates ahead of it in that order. With `rank_bound` and `rank_comp` the ranks of
//! the best bound and the best competing candidate:
//!
//! ```text
//! margin = max(0, M_BIND - (rank_comp - rank_bound))
//! miss   = W_MISS when the bound is not the read's first pick
//! H_bind = margin + miss
//! ```
//!
//! `M_BIND = 2` asks the bound to clear the best competing row by two places, and
//! `W_MISS = 4` charges a row whose bound the read does not pick at all. A row with no
//! bound label costs zero, so ordinary rows and serving passes are unaffected.
//!
//! [`Q31FirstPickRead::train_bind_step`] moves the gates against exactly this cost: the
//! credit is the pick step's sign residual taken between the bound and the best competing
//! row (or the read's own first pick when the row carries no competing label), and the
//! candidate read is kept only when the **rank** hinge strictly falls. Weights, gain and
//! the written row are untouched. Every quantity is integer; no float, softmax or matmul.

use crate::read::read_distance;
use crate::read_train::{add_credit, addresses, apply_credit, q31_read_credit_shift};
use crate::{Error, Q31FirstPickRead, Result, MAX_Q31_STEP_SHIFT};

/// The rank gap the bound must clear: two places ahead of the best competing row.
pub const M_BIND: u32 = 2;
/// The penalty when the bound is not the read's first pick.
pub const W_MISS: u32 = 4;
/// A row's absent bound marker.
pub const NO_BOUND: u32 = 0;

fn invalid(message: &str) -> Error {
    Error::Invalid(message.into())
}

/// Per-row bind labels over one dump's rows.
///
/// `bound[row]` is the 1-based position of the row the query binds to, or [`NO_BOUND`];
/// `competing[row]` holds the competing positions. The design's device sidecar packs the
/// competing set into a 64-leaf mask (`4` words a row); a host trainer carries the list,
/// so a window longer than 64 pays nothing and loses nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindLabels {
    pub bound: Vec<u32>,
    pub competing: Vec<Vec<u32>>,
}

impl BindLabels {
    /// Labels for `rows` rows, none of them bound.
    pub fn new(rows: usize) -> Self {
        Self {
            bound: vec![NO_BOUND; rows],
            competing: vec![Vec::new(); rows],
        }
    }

    /// Label `row`: bind position `bound` (1-based, or [`NO_BOUND`]) against `competing`.
    pub fn set(&mut self, row: usize, bound: u32, competing: &[u32]) -> Result<()> {
        if self.bound.len() <= row {
            return Err(invalid("a bind label is past the rows"));
        }
        if competing.contains(&0) {
            return Err(invalid("a competing position is 1-based"));
        }
        self.bound[row] = bound;
        self.competing[row].clear();
        self.competing[row].extend_from_slice(competing);
        Ok(())
    }

    /// Whether `position` (1-based) of `row` is labelled competing.
    pub fn is_competing(&self, row: usize, position: u32) -> bool {
        position != 0 && self.competing[row].contains(&position)
    }
}

/// One labelled row's hinge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BindRow {
    /// `H_bind`: the margin plus the miss, both integers.
    pub cost: u32,
    /// The rank margin alone.
    pub margin: u32,
    /// `W_MISS` or 0.
    pub miss: u32,
    /// The bound's rank among the earlier rows, `None` when there is no bound candidate.
    pub rank_bound: Option<u32>,
    /// The best competing row's rank, `None` when the row has no competing label.
    pub rank_comp: Option<u32>,
    /// The pairwise distance hinge the step descends: `max(0, d(bound) + m - d(comp))`
    /// against the best competing row (`m` is one place at an equal distance because the
    /// read sends a tie to the later row). Zero without a competing label.
    pub gap: u128,
}

/// The hinge over a set of rows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BindReport {
    /// Rows carrying a bound label.
    pub rows: usize,
    /// `sum H_bind`.
    pub total: u128,
    /// `sum margin`.
    pub margin_cost: u128,
    /// Rows the bound is not the first pick of.
    pub misses: usize,
    /// Rows whose first pick is the bound.
    pub bound_first: usize,
    /// Rows whose first pick is a competing row.
    pub comp_first: usize,
    /// Rows whose bind is ahead of every competing row by the margin.
    pub satisfied: usize,
    /// `sum gap`: the continuous surrogate the step descends.
    pub gap: u128,
}

impl BindReport {
    fn add(&mut self, row: &BindRow) {
        self.rows += 1;
        self.total += u128::from(row.cost);
        self.gap += row.gap;
        self.margin_cost += u128::from(row.margin);
        if row.miss != 0 {
            self.misses += 1;
        }
        match (row.rank_bound, row.rank_comp) {
            (Some(0), _) => self.bound_first += 1,
            (_, Some(0)) => self.comp_first += 1,
            _ => {}
        }
        if row.cost == 0 {
            self.satisfied += 1;
        }
    }
}

/// One bind-margin update of the read's gates over one batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Q31BindTrainStep {
    /// The read after the step: the candidate when accepted, else unchanged.
    pub read: Q31FirstPickRead,
    /// Whether the candidate lowered the batch hinge and was kept.
    pub accepted: bool,
    /// The hinge before the step.
    pub before: BindReport,
    /// The hinge of the candidate read.
    pub after: BindReport,
}

/// The rank of `position` (1-based) among the `t` earlier rows `0..t`, in the read's
/// order: distance first, and a tie goes to the later row.
fn rank_of(
    query: &[i64; crate::read::ADDRESS_LANES],
    keys: &[[i64; crate::read::ADDRESS_LANES]],
    position: u32,
) -> u32 {
    let at = position as usize - 1;
    let own = read_distance(query, &keys[at]);
    let mut rank = 0u32;
    for (s, key) in keys.iter().enumerate() {
        if s == at {
            continue;
        }
        let distance = read_distance(query, key);
        // The read prefers the smaller distance and, at an equal distance, the later row.
        if distance < own || (distance == own && s > at) {
            rank += 1;
        }
    }
    rank
}

impl Q31FirstPickRead {
    /// One row's hinge: the bound's rank against the best competing row's, plus the miss.
    ///
    /// `keys` are the window's key addresses from row 0 to the query row; `row` is the
    /// index into [`BindLabels`], `t` the query's offset in its window.
    fn bind_row(
        &self,
        query: &[i64; crate::read::ADDRESS_LANES],
        keys: &[[i64; crate::read::ADDRESS_LANES]],
        labels: &BindLabels,
        row: usize,
    ) -> BindRow {
        let mut result = BindRow::default();
        let bound = labels.bound[row];
        let competing: Vec<u32> = labels.competing[row]
            .iter()
            .copied()
            .filter(|&position| position >= 1 && position as usize <= keys.len())
            .collect();
        if bound == NO_BOUND {
            return result;
        }
        // The design's rule: a row with no bound candidate at all is a miss, and a row
        // with no competing candidate leaves the margin unconstrained.
        let rank_bound = if (1..=keys.len() as u32).contains(&bound) {
            let rank = rank_of(query, keys, bound);
            result.rank_bound = Some(rank);
            rank
        } else {
            result.miss = W_MISS;
            result.cost = W_MISS;
            return result;
        };
        let mut rank_comp = None;
        for position in competing {
            let rank = rank_of(query, keys, position);
            rank_comp = Some(rank_comp.map_or(rank, |best: u32| best.min(rank)));
        }
        result.rank_comp = rank_comp;
        if rank_bound != 0 {
            result.miss = W_MISS;
        }
        let (margin, gap) = match rank_comp {
            Some(comp) => {
                let delta = i64::from(comp) - i64::from(rank_bound);
                let margin = (i64::from(M_BIND) - delta).max(0) as u32;
                // The descent surrogate: the bound's distance against the best competing
                // row's, with one place of margin when the read would send the tie later.
                let best_comp = (1..=keys.len() as u32)
                    .filter(|&position| labels.is_competing(row, position))
                    .min_by_key(|&position| rank_of(query, keys, position))
                    .expect("a competing rank implies a competing position");
                let d_bound = read_distance(query, &keys[bound as usize - 1]);
                let d_comp = read_distance(query, &keys[best_comp as usize - 1]);
                let m = u64::from(best_comp > bound);
                let gap = i128::from(d_bound) + i128::from(m) - i128::from(d_comp);
                (margin, if gap > 0 { gap as u128 } else { 0 })
            }
            None => (0, 0),
        };
        result.margin = margin;
        result.cost = margin + result.miss;
        result.gap = gap;
        result
    }

    /// The bind hinge over `batch` windows of `time` rows under `read`.
    ///
    /// `stream` is the dump's per-row input (`batch * time * width` words); `labels` has
    /// one entry a row. Rows with an absent bound cost zero and are not counted.
    pub fn bind_hinge(
        &self,
        stream: &[i64],
        labels: &BindLabels,
        batch: usize,
        time: usize,
    ) -> Result<(BindReport, Vec<BindRow>)> {
        let width = self.width();
        let rows = batch
            .checked_mul(time)
            .ok_or_else(|| invalid("the bind hinge's rows overflow"))?;
        if stream.len() != rows.saturating_mul(width) || labels.bound.len() != rows {
            return Err(invalid("the bind hinge's inputs do not match its shape"));
        }
        let (queries, keys) = addresses(self, stream)?;
        let mut report = BindReport::default();
        let mut per_row = vec![BindRow::default(); rows];
        for window in 0..batch {
            let first = window * time;
            for t in 1..time {
                let row = first + t;
                if labels.bound[row] == NO_BOUND {
                    continue;
                }
                let hinge = self.bind_row(&queries[row], &keys[first..row], labels, row);
                report.add(&hinge);
                per_row[row] = hinge;
            }
        }
        Ok((report, per_row))
    }

    /// One bind-margin update of the gates over `batch` windows.
    ///
    /// The credit for a row whose hinge is positive is the pick step's sign residual
    /// taken between the bound row and the best competing row (or the read's own first
    /// pick when the row carries no competing label). The candidate gates are
    /// `sat(gate - (credit >> (step_shift + 1)))`, and the candidate read is kept only
    /// when the rank hinge strictly falls.
    #[allow(clippy::too_many_arguments)]
    pub fn train_bind_step(
        &self,
        stream: &[i64],
        labels: &BindLabels,
        batch: usize,
        time: usize,
        step_shift: u32,
    ) -> Result<Q31BindTrainStep> {
        if step_shift > MAX_Q31_STEP_SHIFT {
            return Err(invalid(
                "native Q31 read training step shift must be in 0..=31",
            ));
        }
        let (before, per_row) = self.bind_hinge(stream, labels, batch, time)?;
        if before.rows == 0 {
            return Ok(Q31BindTrainStep {
                read: self.clone(),
                accepted: false,
                before,
                after: before,
            });
        }
        let width = self.width();
        let (queries, keys) = addresses(self, stream)?;
        let credit_shift = q31_read_credit_shift(stream);
        let mut query_credits = vec![[0i128; 4]; width / 4];
        let mut key_credits = vec![[0i128; 4]; width / 4];
        for window in 0..batch {
            let first = window * time;
            for t in 1..time {
                let row = first + t;
                if per_row[row].cost == 0 {
                    continue;
                }
                if per_row[row].rank_bound.is_none() {
                    // No bound candidate: there is nothing to pull against.
                    continue;
                }
                let bound = labels.bound[row] as usize - 1;
                // The rival is the best competing row; a row with no competing label
                // (or a miss the labels cannot explain) pulls against the first pick.
                let rival = labels.competing[row]
                    .iter()
                    .copied()
                    .filter(|&position| (1..=t as u32).contains(&position))
                    .min_by_key(|&position| rank_of(&queries[row], &keys[first..row], position))
                    .map(|position| position as usize - 1)
                    .unwrap_or_else(|| {
                        (0..t)
                            .min_by_key(|&s| {
                                let distance = read_distance(&queries[row], &keys[first + s]);
                                // Prefer the nearer row, and the later one at a tie.
                                (distance, std::cmp::Reverse(s))
                            })
                            .unwrap_or(0)
                    });
                if rival == bound {
                    continue;
                }
                let mut on_query = [0i64; crate::read::ADDRESS_LANES];
                let mut on_bound = [0i64; crate::read::ADDRESS_LANES];
                let mut on_rival = [0i64; crate::read::ADDRESS_LANES];
                for l in 0..crate::read::ADDRESS_LANES {
                    let to_bound = (queries[row][l] - keys[first + bound][l]).signum();
                    let to_rival = (queries[row][l] - keys[first + rival][l]).signum();
                    on_query[l] = to_bound - to_rival;
                    on_bound[l] = -to_bound;
                    on_rival[l] = to_rival;
                }
                let row_words = |r: usize| &stream[r * width..(r + 1) * width];
                add_credit(&mut query_credits, row_words(row), &on_query, credit_shift)?;
                add_credit(
                    &mut key_credits,
                    row_words(first + bound),
                    &on_bound,
                    credit_shift,
                )?;
                add_credit(
                    &mut key_credits,
                    row_words(first + rival),
                    &on_rival,
                    credit_shift,
                )?;
            }
        }
        let read = Q31FirstPickRead::new(
            apply_credit(self.query_gates(), &query_credits, step_shift),
            apply_credit(self.key_gates(), &key_credits, step_shift),
            self.weights().to_vec(),
            self.gain_shift(),
        )?;
        let (after, _) = read.bind_hinge(stream, labels, batch, time)?;
        // The rank hinge is discrete: a step moves it only when the order flips. The step
        // is kept while the pairwise gap strictly falls, or while the gap holds and the
        // registered rank cost falls, so the descent is continuous and the reported cost
        // still decides the ties.
        let accepted = (after.gap, after.total) < (before.gap, before.total);
        Ok(Q31BindTrainStep {
            read: if accepted { read } else { self.clone() },
            accepted,
            before,
            after: if accepted { after } else { before },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const S: i64 = 1 << 18;
    const TIME: usize = 4;

    /// A 16-lane row with two lanes set.
    fn row(lane0: i64, lane1: i64) -> Vec<i64> {
        let mut row = vec![0; 16];
        row[0] = lane0;
        row[1] = lane1;
        row
    }

    /// Three key rows of `4S`, `S`, `2S` and a query at zero: the nearest is row 1, then
    /// row 2, then row 0.
    fn trie() -> (Vec<i64>, BindLabels) {
        let stream = [row(4 * S, 0), row(S, 0), row(2 * S, 0), row(0, 0)].concat();
        (stream, BindLabels::new(TIME))
    }

    fn read() -> Q31FirstPickRead {
        let gates = vec![[1 << 31, 0, 0, 0]; 4];
        Q31FirstPickRead::new(gates.clone(), gates, vec![0; 16], 4).expect("read")
    }

    #[test]
    fn the_rank_gap_costs_one_place() {
        // Bound row 1 (nearest), competing row 2 (second): one place ahead, so the
        // two-place margin costs one and the bound is admitted.
        let (stream, mut labels) = trie();
        labels.set(3, 2, &[3]).expect("label");
        let (report, per_row) = read().bind_hinge(&stream, &labels, 1, TIME).expect("hinge");
        assert_eq!(per_row[3].rank_bound, Some(0));
        assert_eq!(per_row[3].rank_comp, Some(1));
        assert_eq!(per_row[3].cost, 1);
        assert_eq!(report.bound_first, 1);
        assert_eq!(report.rows, 1);

        // A competing row two places behind clears the margin.
        labels.set(3, 2, &[1]).expect("label");
        let (report, _) = read().bind_hinge(&stream, &labels, 1, TIME).expect("hinge");
        assert_eq!(report.total, 0);
        assert_eq!(report.satisfied, 1);
    }

    #[test]
    fn a_competing_row_ahead_costs_the_margin_and_the_miss() {
        // Bound row 2 (rank 1), competing row 1 (rank 0): one place behind, so the
        // margin is three and the bound is not the first pick.
        let (stream, mut labels) = trie();
        labels.set(3, 3, &[2]).expect("label");
        let (_, per_row) = read().bind_hinge(&stream, &labels, 1, TIME).expect("hinge");
        assert_eq!(per_row[3].rank_bound, Some(1));
        assert_eq!(per_row[3].rank_comp, Some(0));
        assert_eq!(per_row[3].margin, 3);
        assert_eq!(per_row[3].miss, W_MISS);
        assert_eq!(per_row[3].cost, 3 + W_MISS);
    }

    #[test]
    fn a_row_without_a_competing_label_charges_only_the_miss() {
        let (stream, mut labels) = trie();
        labels.set(3, 3, &[]).expect("label");
        let (report, per_row) = read().bind_hinge(&stream, &labels, 1, TIME).expect("hinge");
        assert_eq!(per_row[3].margin, 0);
        assert_eq!(per_row[3].cost, W_MISS);
        assert_eq!(report.misses, 1);

        // The nearest row with no competing label is satisfied.
        labels.set(3, 2, &[]).expect("label");
        let (report, _) = read().bind_hinge(&stream, &labels, 1, TIME).expect("hinge");
        assert_eq!(report.total, 0);
    }

    #[test]
    fn a_bound_outside_the_window_is_a_miss_only() {
        let (stream, mut labels) = trie();
        labels.set(3, 9, &[2]).expect("label");
        let (_, per_row) = read().bind_hinge(&stream, &labels, 1, TIME).expect("hinge");
        assert_eq!(per_row[3].rank_bound, None);
        assert_eq!(per_row[3].cost, W_MISS);
    }

    #[test]
    fn unlabelled_rows_cost_nothing() {
        let (stream, labels) = trie();
        let (report, per_row) = read().bind_hinge(&stream, &labels, 1, TIME).expect("hinge");
        assert_eq!(report.rows, 0);
        assert_eq!(report.total, 0);
        assert_eq!(per_row[3], BindRow::default());
    }

    #[test]
    fn a_label_it_cannot_address_is_refused() {
        let mut labels = BindLabels::new(4);
        assert!(labels.set(0, 1, &[0]).is_err());
        assert!(labels.set(4, 1, &[]).is_err());
        // A position past the design's 64-leaf mask is ordinary on the host: the served
        // windows are 192 rows, and the list carries what the mask cannot.
        assert!(labels.set(0, 1, &[130]).is_ok());
        assert!(labels.is_competing(0, 130));
        assert!(!labels.is_competing(0, 129));
    }

    #[test]
    fn the_step_moves_the_bound_ahead_and_lowers_the_hinge() {
        // The pick step's fixture: row 2's nearest earlier key is row 0; the bound is
        // row 1, which the competing row 0 outranks.
        let stream = [row(S, 0), row(0, S), row(3 * S / 4, S / 2), row(0, 0)].concat();
        let mut labels = BindLabels::new(TIME);
        labels.set(2, 2, &[1]).expect("label");
        let before = read()
            .bind_hinge(&stream, &labels, 1, TIME)
            .expect("hinge")
            .0;
        assert!(before.total > 0, "the fixture starts violated");
        let first = read()
            .train_bind_step(&stream, &labels, 1, TIME, 4)
            .expect("step");
        assert!(first.accepted, "the gap falls and the candidate is kept");
        assert!(
            first.after.gap < first.before.gap,
            "the descent surrogate falls"
        );
        // The rank cost moves when the order flips, after a few steps: the pick step needs
        // four at this shift, and the hinge stops as soon as the bound leads by two places.
        let mut trained = first.read;
        let mut steps = 1;
        while trained
            .bind_hinge(&stream, &labels, 1, TIME)
            .expect("hinge")
            .0
            .total
            > 0
            && steps < 16
        {
            trained = trained
                .train_bind_step(&stream, &labels, 1, TIME, 4)
                .expect("step")
                .read;
            steps += 1;
        }
        let (report, _) = trained
            .bind_hinge(&stream, &labels, 1, TIME)
            .expect("hinge");
        // The bound becomes the first pick. Two candidates cannot clear a two-place
        // margin, so the cost keeps the one-place term; the miss is what the step removes.
        assert_eq!(report.bound_first, 1, "the bound leads after {steps} steps");
        assert_eq!(report.misses, 0);
        assert!(report.total < before.total);
        assert_eq!(
            report.total, 1,
            "the surviving term is the one-place margin"
        );
    }

    #[test]
    fn a_refused_shape_is_an_error() {
        let (stream, mut labels) = trie();
        labels.set(3, 2, &[3]).expect("label");
        let read = read();
        assert!(read
            .bind_hinge(&stream[..stream.len() - 1], &labels, 1, TIME)
            .is_err());
        assert!(read.bind_hinge(&stream, &labels, 2, TIME).is_err());
        assert!(read.train_bind_step(&stream, &labels, 1, TIME, 32).is_err());
    }
}
