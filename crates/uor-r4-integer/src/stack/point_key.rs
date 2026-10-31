//! The served point key in the integer artifact (claude-222, `point-key-integer`): TRAIN's
//! opening points under the integer trunk, at the writer query's Q12, with the first token of
//! each opening's reply, as a `uor-r4.lut-stack/8` container stores them in
//! `memory.point.points` and `memory.point.targets`.
//!
//! Where the turn memory's exact key misses a reply's opening, the session names the opening's
//! first token by [`ServedPointKey::name`]: the commonest target among the rows at the least L1
//! distance from the opening row's own query, ties to the smaller id. That is the float side's
//! `l1_named` rule (claude-219, claude-220), over the integer trunk's own points (the item's
//! condition (b)).

use super::format::{Container, StackPointKey, POINT_KEY_EXP};
use super::memory::MEMORY_BOS_ID;
use super::StackError;

/// A stack's point table: `rows × width` Q12 points, row-major, and each row's target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PointKeyImage {
    pub width: usize,
    pub points: Vec<i16>,
    pub targets: Vec<u32>,
}

impl PointKeyImage {
    /// The rows the table holds.
    pub fn rows(&self) -> usize {
        self.targets.len()
    }

    /// The header record naming this table.
    pub fn record(&self) -> StackPointKey {
        StackPointKey {
            rows: self.rows(),
            width: self.width,
            exp: POINT_KEY_EXP,
        }
    }

    /// Refuse a table the session cannot serve: no rows, points that are not `rows × width`, or a
    /// target that is BOS or past `vocab`.
    pub fn check(&self, vocab: usize) -> Result<(), StackError> {
        if self.targets.is_empty() || self.width == 0 {
            return Err(StackError::Numerics("the point table has no rows".to_owned()));
        }
        if self.rows().checked_mul(self.width) != Some(self.points.len()) {
            return Err(StackError::Numerics("the point table's points are not rows × width".to_owned()));
        }
        if let Some(&bad) = self.targets.iter().find(|&&t| t == MEMORY_BOS_ID || t as usize >= vocab) {
            return Err(StackError::Numerics(format!("the point table names {bad}, BOS or past the vocabulary")));
        }
        Ok(())
    }
}

/// The point table of a parsed container, every section checked against its record, or `None`
/// for a stack without one.
pub(crate) fn point_image(artifact: &Container<'_>) -> Result<Option<PointKeyImage>, StackError> {
    let Some(record) = &artifact.point else {
        return Ok(None);
    };
    let image = PointKeyImage {
        width: record.width,
        points: artifact.table_i16("memory.point.points")?,
        targets: artifact.table_u32("memory.point.targets")?,
    };
    if image.rows() != record.rows {
        return Err(StackError::Numerics(format!(
            "memory.point.targets holds {} rows, not the record's {}",
            image.rows(),
            record.rows
        )));
    }
    image.check(artifact.shape.vocab)?;
    Ok(Some(image))
}

/// The point table as a session serves it.
pub(crate) struct ServedPointKey {
    width: usize,
    points: Vec<i16>,
    targets: Vec<u32>,
}

impl ServedPointKey {
    /// The served table of a checked image.
    pub(crate) fn new(image: PointKeyImage) -> Self {
        Self {
            width: image.width,
            points: image.points,
            targets: image.targets,
        }
    }

    /// The rows the table holds: the capacity [`Self::name`]'s scratch needs.
    pub(crate) fn rows(&self) -> usize {
        self.targets.len()
    }

    /// The target `query` names, and its least distance: the L1 distance to every row at Q12,
    /// summed exactly, then the commonest target among the rows at the least distance, ties to
    /// the smaller id. `tied` is the caller's scratch, of at least [`Self::rows`] capacity, so the
    /// search allocates nothing. Adds, absolute differences and compares only.
    #[inline(never)]
    pub(crate) fn name(&self, query: &[i32], tied: &mut Vec<u32>) -> Result<(u32, i64), StackError> {
        if query.len() != self.width {
            return Err(StackError::Numerics("the point key's query is not the table's width".to_owned()));
        }
        let mut least = i64::MAX;
        tied.clear();
        for (row, &target) in self.points.chunks_exact(self.width).zip(&self.targets) {
            // `black_box` keeps the sum scalar, as `stack_whole_table_nearest`'s: a vectorized
            // sum leaves through a float register move, which the serving audit refuses.
            let mut sum = 0i64;
            for (&x, &p) in query.iter().zip(row) {
                sum += i64::from(core::hint::black_box(x).abs_diff(i32::from(p)));
            }
            if sum < least {
                least = sum;
                tied.clear();
            }
            if sum == least {
                tied.push(target);
            }
        }
        // The commonest target among the tied rows: the longest run of the sorted ids, a longer
        // run replacing the held one only when strictly longer, so ties go to the smaller id.
        tied.sort_unstable();
        let (mut named, mut best) = (u32::MAX, 0usize);
        let mut at = 0;
        while at < tied.len() {
            let id = tied[at];
            let run = tied[at..].iter().take_while(|&&t| t == id).count();
            if run > best {
                (named, best) = (id, run);
            }
            at += run;
        }
        if best == 0 || named == MEMORY_BOS_ID {
            return Err(StackError::Numerics("the point key named no token, or BOS".to_owned()));
        }
        Ok((named, least))
    }
}

#[cfg(test)]
mod point_key_tests {
    use super::*;

    fn served(width: usize, points: Vec<i16>, targets: Vec<u32>) -> ServedPointKey {
        let image = PointKeyImage { width, points, targets };
        image.check(100).unwrap();
        ServedPointKey::new(image)
    }

    /// claude-222: the least L1 distance names the target, over every row.
    #[test]
    fn the_point_key_names_the_least_l1_rows_target() {
        let key = served(2, vec![10, 10, 0, 0, -5, 3], vec![7, 8, 9]);
        let mut tied = Vec::with_capacity(key.rows());
        assert_eq!(key.name(&[1, 1], &mut tied).unwrap(), (8, 2));
        assert_eq!(key.name(&[9, 12], &mut tied).unwrap(), (7, 3));
        assert_eq!(key.name(&[-6, 3], &mut tied).unwrap(), (9, 1));
    }

    /// claude-222: among the rows at the least distance the commonest target wins, and between
    /// equally common targets the smaller id.
    #[test]
    fn the_point_key_takes_the_commonest_tied_target_then_the_smaller_id() {
        // Rows 0–3 are all at distance 2 from [0, 0]; 9 twice, 5 once, 4 once.
        let key = served(2, vec![1, 1, 2, 0, -1, -1, 0, 2, 50, 50], vec![9, 5, 9, 4, 3]);
        let mut tied = Vec::with_capacity(key.rows());
        let capacity = tied.capacity();
        assert_eq!(key.name(&[0, 0], &mut tied).unwrap(), (9, 2), "the commonest of the tied");
        let even = served(2, vec![1, 1, 2, 0, -1, -1, 0, 2], vec![9, 5, 5, 9]);
        assert_eq!(even.name(&[0, 0], &mut tied).unwrap(), (5, 2), "a tie in counts goes to the smaller id");
        // A nearer single row beats a more common farther one.
        let near = served(2, vec![1, 1, 1, 1, 0, 1], vec![6, 6, 7]);
        assert_eq!(near.name(&[0, 0], &mut tied).unwrap(), (7, 1));
        assert_eq!(tied.capacity(), capacity, "the search grew no scratch past its capacity");
    }

    /// claude-222: a table holding BOS or an id past the vocabulary is refused, and so is a
    /// query of the wrong width or an empty table.
    #[test]
    fn the_point_key_refuses_bos_out_of_range_ids_and_a_wrong_width() {
        let image = |targets: Vec<u32>| PointKeyImage { width: 1, points: vec![0; targets.len()], targets };
        assert!(image(vec![MEMORY_BOS_ID, 3]).check(100).is_err(), "BOS");
        assert!(image(vec![3, 100]).check(100).is_err(), "past the vocabulary");
        assert!(image(vec![]).check(100).is_err(), "no rows");
        assert!(PointKeyImage { width: 2, points: vec![0; 3], targets: vec![3, 4] }.check(100).is_err(), "rows × width");
        let key = served(1, vec![0, 4], vec![3, 4]);
        assert!(key.name(&[0, 0], &mut Vec::new()).is_err(), "a wrong-width query");
    }
}
