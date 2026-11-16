//! The read cost ledger (ADR-0007 §2 rule 3): exactness, first divergence, and rate.
//!
//! Selection compares arms on one frozen corpus. Exactness is bit-identity with the reference
//! read, and the two numbers that say whether an arm is exact are the rows whose pick matches and
//! the first row where it does not. Rate is rows a second at a supplied elapsed time, so a report
//! can carry `tokens/s` beside the agreement without inventing a clock here.
//!
//! The ledger compares **picks**: one `u32` a row, `0` for a row the read chose nothing on. That is
//! the form both `native_read_train --save-picks` and the served session produce, and it is the
//! `support` of a first-pick read. A read whose result is a set of positions (the Hamming heads)
//! is compared on its best pick until the set form is needed; the ledger does not pretend to
//! measure a set it was not given.

use super::StackError;

/// One comparison of a served read against its reference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReadLedger {
    /// The rows compared.
    pub rows: usize,
    /// The rows whose pick equals the reference's.
    pub agree: usize,
    /// The first row whose pick differs, `None` when every row agrees.
    pub first_divergence: Option<usize>,
}

impl ReadLedger {
    /// Compare a served pick list with its reference, row for row.
    pub fn compare(reference: &[u32], served: &[u32]) -> Result<Self, StackError> {
        let scored = vec![true; reference.len().min(served.len())];
        if !scored.is_empty() {
            Self::compare_on(reference, served, &scored)
        } else {
            Self::compare_on(reference, served, &[])
        }
    }

    /// Compare only the `scored` rows; the divergence is the row's index in the whole list, so it
    /// names a position in the corpus and not only in the selection.
    pub fn compare_on(
        reference: &[u32],
        served: &[u32],
        scored: &[bool],
    ) -> Result<Self, StackError> {
        if reference.len() != served.len() || scored.len() != reference.len() {
            return Err(StackError::Shape(
                "the read ledger needs one served row a reference row",
            ));
        }
        let (mut rows, mut agree, mut first_divergence) = (0usize, 0usize, None);
        for (row, ((&reference, &served), &scored)) in
            reference.iter().zip(served).zip(scored).enumerate()
        {
            if !scored {
                continue;
            }
            rows += 1;
            if reference == served {
                agree += 1;
            } else if first_divergence.is_none() {
                first_divergence = Some(row);
            }
        }
        Ok(Self {
            rows,
            agree,
            first_divergence,
        })
    }

    /// Agreement in parts per million, floored; `0` for an empty comparison.
    pub fn agreement_ppm(&self) -> u64 {
        if self.rows == 0 {
            return 0;
        }
        let agree = self.agree as u128;
        (agree * 1_000_000 / self.rows as u128) as u64
    }

    /// Rows a second at `nanos` elapsed, or `None` for a zero time or a rate that does not fit.
    pub fn rows_per_second(&self, nanos: u64) -> Option<u64> {
        if nanos == 0 || self.rows == 0 {
            return None;
        }
        let rate = self.rows as u128 * 1_000_000_000 / u128::from(nanos);
        u64::try_from(rate).ok()
    }

    /// The ledger as one JSON object: every number integer, the divergence a row or null.
    pub fn to_json(&self, nanos: Option<u64>) -> serde_json::Value {
        serde_json::json!({
            "rows": self.rows,
            "agree": self.agree,
            "agreement_ppm": self.agreement_ppm(),
            "first_divergence": self.first_divergence,
            "rows_per_second": nanos.and_then(|nanos| self.rows_per_second(nanos)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_divergence_is_the_first_differing_row() {
        let ledger = ReadLedger::compare(&[1, 2, 3, 0], &[1, 2, 4, 0]).expect("compare");
        assert_eq!(ledger.rows, 4);
        assert_eq!(ledger.agree, 3);
        assert_eq!(ledger.first_divergence, Some(2));
        assert_eq!(ledger.agreement_ppm(), 750_000);

        let exact = ReadLedger::compare(&[0, 5], &[0, 5]).expect("compare");
        assert_eq!(exact.first_divergence, None);
        assert_eq!(exact.agreement_ppm(), 1_000_000);
    }

    #[test]
    fn a_zero_pick_is_a_row_the_read_chose_nothing_on() {
        // Both reads choosing nothing is agreement, not a missing row.
        let ledger = ReadLedger::compare(&[0, 0], &[0, 1]).expect("compare");
        assert_eq!(ledger.agree, 1);
        assert_eq!(ledger.first_divergence, Some(1));
    }

    #[test]
    fn a_mask_restricts_the_rows_and_keeps_the_divergence_absolute() {
        let reference = [1u32, 2, 3, 4];
        let served = [1u32, 9, 3, 8];
        let scored = [true, false, true, true];
        let ledger = ReadLedger::compare_on(&reference, &served, &scored).expect("compare");
        assert_eq!(ledger.rows, 3);
        assert_eq!(ledger.agree, 2);
        assert_eq!(
            ledger.first_divergence,
            Some(3),
            "row 3, not the third scored row"
        );
        // The masked row's disagreement does not count and does not open the divergence.
        let scored = [true, false, true, false];
        let ledger = ReadLedger::compare_on(&reference, &served, &scored).expect("compare");
        assert_eq!(ledger.first_divergence, None);
        assert_eq!(ledger.rows, 2);
        assert!(ReadLedger::compare_on(&reference, &served, &[true]).is_err());
    }

    #[test]
    fn a_length_mismatch_is_refused() {
        let message = ReadLedger::compare(&[1], &[1, 2]).expect_err("mismatch");
        assert!(format!("{message}").contains("one served row a reference row"));
    }

    #[test]
    fn the_rate_is_integer_and_refuses_a_zero_clock() {
        let ledger = ReadLedger::compare(&[1, 2, 3, 4], &[1, 2, 3, 4]).expect("compare");
        assert_eq!(ledger.rows_per_second(2_000_000), Some(2_000));
        assert_eq!(ledger.rows_per_second(0), None);
        let empty = ReadLedger::compare(&[], &[]).expect("compare");
        assert_eq!(empty.agreement_ppm(), 0);
        assert_eq!(empty.rows_per_second(1), None);
    }
}
