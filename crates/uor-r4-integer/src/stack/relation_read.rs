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

    /// The other fact's value position `p` at `query_pos`, or `None`. The
    /// earlier slots' asked value positions are excluded.
    fn pick(&self, tokens: &[u32], query_pos: usize, earlier_asked: &[usize]) -> Option<usize> {
        let (asked, bucket) = self.asked(tokens, query_pos)?;
        let (low, high) = self.buckets[bucket];
        let facts = self.facts(tokens, query_pos);
        facts.iter().copied().find(|&p| {
            p != asked
                && p < query_pos
                && (low..=high).contains(&(query_pos - p))
                && !earlier_asked.contains(&p)
        })
    }

    /// The number of consistent other-fact candidates at `query_pos`: the 397 `K`.
    pub fn candidate_count(
        &self,
        tokens: &[u32],
        query_pos: usize,
        earlier_asked: &[usize],
    ) -> Option<usize> {
        let (asked, bucket) = self.asked(tokens, query_pos)?;
        let (low, high) = self.buckets[bucket];
        let facts = self.facts(tokens, query_pos);
        Some(
            facts
                .iter()
                .filter(|&&p| {
                    p != asked
                        && p < query_pos
                        && (low..=high).contains(&(query_pos - p))
                        && !earlier_asked.contains(&p)
                })
                .count(),
        )
    }

    /// The asked fact's value position at `query_pos`, or `None` when the token
    /// there is not a query. The session tracks the earlier slots' asks with it.
    pub fn asked_at(&self, tokens: &[u32], query_pos: usize) -> Option<usize> {
        self.asked(tokens, query_pos).map(|(asked, _)| asked)
    }

    /// The read at `query_pos`: the asked fact's other fact, with the value
    /// token as a one-element payload. `earlier_asked` are the earlier slots'
    /// asked value positions in the same window.
    pub fn select(
        &self,
        tokens: &[u32],
        query_pos: usize,
        earlier_asked: &[usize],
    ) -> Option<ReadOut> {
        let pick = self.pick(tokens, query_pos, earlier_asked)?;
        let value = i128::from(*tokens.get(pick)?);
        Some(ReadOut::first_pick(pick as u32, vec![value]))
    }

    /// [`select`](Self::select) with the picked value's **table row** as the
    /// payload, the width-wide row the hard copy writes. The table is
    /// `vocab * width` i16 words; the value token indexes it.
    pub fn select_with_table(
        &self,
        tokens: &[u32],
        query_pos: usize,
        earlier_asked: &[usize],
        table: &[i16],
        width: usize,
    ) -> Option<ReadOut> {
        let pick = self.pick(tokens, query_pos, earlier_asked)?;
        let value = *tokens.get(pick)? as usize;
        let start = value.checked_mul(width)?;
        let row = table.get(start..start.checked_add(width)?)?;
        Some(ReadOut::first_pick(
            pick as u32,
            row.iter().map(|&word| i128::from(word)).collect(),
        ))
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
    fn the_served_payload_is_the_picked_values_table_row() {
        let mut table = vec![0i16; 1_200 * 2];
        table[529 * 2] = 7;
        table[529 * 2 + 1] = -8;
        let out = RelationRead::PROBE
            .select_with_table(&window(false), 100, &[], &table, 2)
            .expect("the other fact's row");
        assert_eq!(out.pick(), 50);
        assert_eq!(out.support, vec![49]);
        assert_eq!(out.payload.as_deref(), Some(&[7, -8][..]));
        assert_eq!(out.check(), Ok(()));
        // A table without the value's row is refused rather than truncated.
        assert!(RelationRead::PROBE
            .select_with_table(&window(false), 100, &[], &table[..2], 2)
            .is_none());
    }

    #[cfg(feature = "fbs-synthetic")]
    #[test]
    fn the_read_names_the_value_row_in_a_served_session() {
        use crate::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth, FBS_SYNTH_SMALL};
        use crate::stack::{export_fbs_trunk_uorlut01, IntegerStackModel};

        let shape = FbsSynth {
            vocab: 2_048,
            context: 192,
            ..FBS_SYNTH_SMALL
        };
        let snap = synthetic_fbs_trunk_with(20_261_700, shape);
        let bytes = export_fbs_trunk_uorlut01(&snap).expect("export a trunk");
        let model = IntegerStackModel::parse(&bytes).expect("parse");
        let mut session = model.session();
        for &id in &window(false) {
            session.step_native(id).expect("step the window");
        }
        let image = model.fbs_trunk_image().expect("the trunk image");
        let width = model.shape().width;
        let out = RelationRead::PROBE
            .select_with_table(session.tokens(), 100, &[], image.writer_table, width)
            .expect("the other fact's row in a served session");
        assert_eq!(out.pick(), 50);
        let value = session.tokens()[50] as usize;
        let row: Vec<i128> = image.writer_table[value * width..(value + 1) * width]
            .iter()
            .map(|&word| i128::from(word))
            .collect();
        assert_eq!(out.payload.as_deref(), Some(&row[..]));
        assert_eq!(out.check(), Ok(()));
    }

    #[cfg(feature = "fbs-synthetic")]
    #[test]
    fn a_served_session_applies_the_rule_pick() {
        use crate::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth, FBS_SYNTH_SMALL};
        use crate::stack::{export_fbs_trunk_uorlut01, IntegerStackModel};

        let shape = FbsSynth {
            vocab: 2_048,
            context: 192,
            ..FBS_SYNTH_SMALL
        };
        let snap = synthetic_fbs_trunk_with(20_261_710, shape);
        let bytes = export_fbs_trunk_uorlut01(&snap).expect("export a trunk");
        let model = IntegerStackModel::parse(&bytes).expect("parse");
        let mut tokens = window(false);
        tokens[101..=104].copy_from_slice(&[48, 49, 50, 51]);
        tokens[105] = 528;

        let mut session = model.session();
        session.set_relation_read(RelationRead::PROBE, 4, 51, 16);
        for &id in &tokens[..=104] {
            session.step_native(id).expect("step the header");
        }
        assert_eq!(session.last_relation_pick(), Some(50));

        let mut plain = model.session();
        for &id in &tokens[..=104] {
            plain.step_native(id).expect("step the header");
        }
        assert_eq!(plain.last_relation_pick(), None);
    }

    #[cfg(feature = "fbs-synthetic")]
    #[test]
    fn the_artifact_carries_the_relation_read() {
        use crate::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth, FBS_SYNTH_SMALL};
        use crate::stack::{
            add_relation_uorlut01, export_fbs_trunk_uorlut01, IntegerStackModel, StackRelation,
        };

        let shape = FbsSynth {
            vocab: 2_048,
            context: 192,
            ..FBS_SYNTH_SMALL
        };
        let snap = synthetic_fbs_trunk_with(20_261_720, shape);
        let plain = export_fbs_trunk_uorlut01(&snap).expect("export a trunk");
        let relation = StackRelation {
            relations: [16, 32],
            values: [528, 1_040],
            query_shift: 16,
            buckets: [(33, 96), (97, 180)],
            header: 4,
            header_last: 51,
            scale: 16,
        };
        let bytes = add_relation_uorlut01(&plain, &relation).expect("add the relation");
        assert!(
            add_relation_uorlut01(&bytes, &relation).is_err(),
            "a second relation is refused"
        );
        let bad = StackRelation {
            scale: 0,
            ..relation.clone()
        };
        assert!(
            add_relation_uorlut01(&plain, &bad).is_err(),
            "a zero scale is refused"
        );

        let mut tokens = window(false);
        tokens[101..=104].copy_from_slice(&[48, 49, 50, 51]);
        tokens[105] = 528;

        let model = IntegerStackModel::parse(&bytes).expect("parse the relation artifact");
        let mut session = model.session();
        for &id in &tokens[..=104] {
            session.step_native(id).expect("step the header");
        }
        assert_eq!(
            session.last_relation_pick(),
            Some(50),
            "the artifact configures the read"
        );

        let plain_model = IntegerStackModel::parse(&plain).expect("parse the plain artifact");
        let mut plain_session = plain_model.session();
        for &id in &tokens[..=104] {
            plain_session.step_native(id).expect("step the header");
        }
        assert_eq!(plain_session.last_relation_pick(), None);
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
