//! A rule-conditioned relation read: the asked fact's other fact, as a `ReadOut`.
//!
//! The two-class recall task writes each fact as `relation value` before the
//! tails and asks one relation as `Q(k) = k + query_shift`. This read scans the
//! context for those pairs, matches the query's relation to the asked fact,
//! takes the query's distance bucket, and selects the consistent candidate that
//! is not the asked fact and not an earlier slot's asked fact — the 397 rule.
//! On a `K = 1` row that candidate is the other fact by construction.
//!
//! It returns the same [`ReadOut`] as every other read: the picked value
//! position as the 1-based pick (support `pick - 1`) and the value as the
//! payload. The scan is integer-only and adds no serving arithmetic beyond the
//! comparison; the ranges are a configuration, not a learned table.

use super::ReadOut;

/// The probe's fact relations, `K`.
pub const PROBE_RELATIONS: std::ops::Range<u32> = 16..32;
/// The probe's fact values, `v`.
pub const PROBE_VALUES: std::ops::Range<u32> = 528..1_040;
/// The probe's query token is `k + PROBE_QUERY_SHIFT`.
pub const PROBE_QUERY_SHIFT: u32 = 16;
/// The probe's distance buckets: near, far.
pub const PROBE_BUCKETS: [(usize, usize); 2] = [(33, 96), (97, 180)];

/// A relation read over one vocabulary's fact encoding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationRead {
    /// The fact relation tokens.
    pub relations: std::ops::Range<u32>,
    /// The fact value tokens.
    pub values: std::ops::Range<u32>,
    /// The query token is `relation + query_shift`.
    pub query_shift: u32,
    /// The distance buckets, in the probe's order.
    pub buckets: [(usize, usize); 2],
}

impl RelationRead {
    /// The two-class recall task's encoding.
    pub const PROBE: Self = Self {
        relations: PROBE_RELATIONS,
        values: PROBE_VALUES,
        query_shift: PROBE_QUERY_SHIFT,
        buckets: PROBE_BUCKETS,
    };

    /// The fact value positions `p` in `tokens[..limit]`: `tokens[p - 1]` is a
    /// relation and `tokens[p]` a value.
    fn facts(&self, tokens: &[u32], limit: usize) -> Vec<usize> {
        (1..limit.min(tokens.len()))
            .filter(|&p| {
                self.relations.contains(&tokens[p - 1]) && self.values.contains(&tokens[p])
            })
            .collect()
    }

    /// The asked fact's value position, matched from the query token.
    fn asked(&self, tokens: &[u32], query_pos: usize) -> Option<(usize, usize)> {
        let relation = tokens.get(query_pos)?.checked_sub(self.query_shift)?;
        if !self.relations.contains(&relation) {
            return None;
        }
        let facts = self.facts(tokens, query_pos);
        let asked = *facts
            .iter()
            .find(|&&p| tokens[p - 1] == relation && p < query_pos)?;
        let distance = query_pos.checked_sub(asked)?;
        let bucket = self
            .buckets
            .iter()
            .position(|&(low, high)| (low..=high).contains(&distance))?;
        Some((asked, bucket))
    }

    /// The read at `query_pos`: the asked fact's other fact, or `None` when the
    /// context does not name one. `earlier_asked` are the earlier slots' asked
    /// value positions in the same window.
    pub fn select(
        &self,
        tokens: &[u32],
        query_pos: usize,
        earlier_asked: &[usize],
    ) -> Option<ReadOut> {
        let (asked, bucket) = self.asked(tokens, query_pos)?;
        let (low, high) = self.buckets[bucket];
        let facts = self.facts(tokens, query_pos);
        let pick = *facts.iter().find(|&&p| {
            p != asked
                && p < query_pos
                && (low..=high).contains(&(query_pos - p))
                && !earlier_asked.contains(&p)
        })?;
        let value = i128::from(*tokens.get(pick)?);
        Some(ReadOut::first_pick(pick as u32, vec![value]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BOS, facts at 10 and 50 (and optionally 70), a query at 100 asking
    /// relation 16 (value 528) whose asked fact is at 10.
    fn window(extra: bool) -> Vec<u32> {
        let mut tokens = vec![0u32; 120];
        tokens[9] = 16;
        tokens[10] = 528;
        tokens[49] = 17;
        tokens[50] = 529;
        if extra {
            tokens[59] = 18;
            tokens[60] = 530;
        }
        tokens[100] = 32; // Q(16)
        tokens[101] = 528; // the asked value
        tokens
    }

    #[test]
    fn the_read_picks_the_only_other_fact_on_a_k1_row() {
        let out = RelationRead::PROBE
            .select(&window(false), 100, &[])
            .expect("the other fact");
        assert_eq!(out.pick(), 50);
        assert_eq!(out.support, vec![49]);
        assert_eq!(out.payload.as_deref(), Some(&[529][..]));
        assert_eq!(out.check(), Ok(()));
    }

    #[test]
    fn an_ambiguous_row_takes_the_first_consistent_candidate() {
        let out = RelationRead::PROBE
            .select(&window(true), 100, &[])
            .expect("a candidate");
        assert_eq!(out.pick(), 50);
        // An earlier slot's asked fact is excluded.
        let out = RelationRead::PROBE
            .select(&window(true), 100, &[50])
            .expect("the next candidate");
        assert_eq!(out.pick(), 60);
        assert_eq!(out.payload.as_deref(), Some(&[530][..]));
    }

    #[test]
    fn no_asked_relation_or_no_bucket_selects_nothing() {
        let mut tokens = window(false);
        tokens[100] = 48; // Q(32): not a fact relation
        assert!(RelationRead::PROBE.select(&tokens, 100, &[]).is_none());

        // A fact at distance 10 is outside both probe buckets.
        let mut tokens = window(false);
        tokens[10] = 0;
        tokens[9] = 0;
        tokens[89] = 16;
        tokens[90] = 528;
        assert!(RelationRead::PROBE.select(&tokens, 100, &[]).is_none());
    }
}
