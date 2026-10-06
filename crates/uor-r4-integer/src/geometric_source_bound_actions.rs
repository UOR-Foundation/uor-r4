//! Opt-in flat source-bound Copy/Period/Stop normalization.
//!
//! Every real source retains two terminal actions. The last real Stop occupies
//! the existing reducer's null slot; no extra null or synthetic Stop is added.
//! Caller-supplied Q24 logits already contain geometry and any shared source
//! cue. This wrapper neither selects a source nor accepts an answer/label.
//! Equal emitted tokens aggregate after exact common Q31 normalization, while
//! occurrence/version/event identities remain distinct in the action trace.
//! Construction admits fixed numerical scratch; owned traces allocate per call.
//! This bounded arithmetic component is not a chat or learned-attention result.

use serde::Serialize;

use crate::{
    geometric_read::{NativeGeometricRead, MAX_ACTION_OCCURRENCE_ROWS},
    geometric_source_actions::{SourceActionBinding, TokenMass, MAX_HEADS},
    geometric_source_realizer::{invalid, SourceRuntimeResult as Result},
};
use uor_r4_tokenizer::dialogue::DialogueProtocol;

pub const MAX_COPY_ACTIONS: usize = 128;
pub const MAX_SOURCES: usize = 128;
pub const MAX_ACTIONS: usize = 384;
pub const POLICY: &str = "source-bound-copy-period-stop/1;checked-head-sum;flat-native-Q31;one-period-and-stop-per-real-source;last-real-stop-null;no-extra-null;token-alias-aggregation;smallest-token-ID-ties";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SourceBoundProvenance {
    pub source_ordinal: usize,
    pub source_segment_index: usize,
    pub record: u64,
    pub commit: u64,
    pub event: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SourceBoundAction {
    Copy {
        bank_index: usize,
        source_offset: usize,
    },
    Period,
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SourceBoundActionCandidate {
    pub action: SourceBoundAction,
    pub source: SourceBoundProvenance,
    pub token_id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceBoundActionMass {
    pub action_offset: usize,
    pub action: SourceBoundAction,
    pub source: SourceBoundProvenance,
    pub token_id: u32,
    pub score_q24: i64,
    pub weight_q31: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceBoundActionTrace {
    pub policy: &'static str,
    pub tokenizer_sha256: String,
    pub protocol: DialogueProtocol,
    pub period_token_id: u32,
    pub eos_token_id: u32,
    pub head_scores: Vec<Vec<i64>>,
    pub actions: Vec<SourceBoundActionMass>,
    pub token_masses: Vec<TokenMass>,
    pub max_score_q24: i64,
    pub total_weight_q31: u64,
    pub chosen_token_id: u32,
    pub chosen_weight_q31: u64,
    /// Numerical rows exclude the last Stop; action count includes it.
    pub occurrence_rows: usize,
    pub source_count: usize,
    pub copy_count: usize,
    /// Bounded linear-search comparisons actually executed during aggregation.
    pub alias_comparisons: usize,
}

pub struct NativeSourceBoundActions {
    binding: SourceActionBinding,
    heads: usize,
    reducer: NativeGeometricRead,
    scores: [i64; MAX_ACTIONS],
    zeros: [i64; MAX_ACTION_OCCURRENCE_ROWS],
    values: [i32; MAX_ACTION_OCCURRENCE_ROWS],
}

impl NativeSourceBoundActions {
    pub fn new(binding: SourceActionBinding, heads: usize, exp: &[u32]) -> Result<Self> {
        if !(1..=MAX_HEADS).contains(&heads) {
            return Err(invalid("source-bound action heads must be in 1..=2"));
        }
        Ok(Self {
            binding,
            heads,
            reducer: NativeGeometricRead::new_action_normalizer(MAX_ACTION_OCCURRENCE_ROWS, exp)
                .map_err(|e| invalid(format!("source-bound reducer: {e}")))?,
            scores: [0; MAX_ACTIONS],
            zeros: [0; MAX_ACTION_OCCURRENCE_ROWS],
            values: [0; MAX_ACTION_OCCURRENCE_ROWS],
        })
    }

    pub fn binding(&self) -> &SourceActionBinding {
        &self.binding
    }
    pub fn heads(&self) -> usize {
        self.heads
    }

    /// Each head lists every action in exactly caller order. Dense source
    /// ordinals distinguish identical record/commit/event values without merging
    /// them. Empty-source fallback belongs to the caller's explicit legacy arm.
    pub fn reduce(
        &mut self,
        candidates: &[SourceBoundActionCandidate],
        head_scores: &[&[i64]],
    ) -> Result<SourceBoundActionTrace> {
        let n = candidates.len();
        if !(2..=MAX_ACTIONS).contains(&n)
            || candidates
                .last()
                .is_none_or(|c| c.action != SourceBoundAction::Stop)
        {
            return Err(invalid(
                "source-bound actions require 2..=384 actions ending in real Stop",
            ));
        }
        if head_scores.len() != self.heads || head_scores.iter().any(|h| h.len() != n) {
            return Err(invalid("source-bound head/action shape differs"));
        }
        let mut identities = [None; MAX_SOURCES];
        let mut periods = [false; MAX_SOURCES];
        let mut stops = [false; MAX_SOURCES];
        let mut copy_count = 0;
        let mut source_count = 0;
        let mut bank_indices = [None; MAX_COPY_ACTIONS];
        for c in candidates {
            let ordinal = c.source.source_ordinal;
            if ordinal >= MAX_SOURCES {
                return Err(invalid("source-bound source ordinal exceeds127"));
            }
            match identities[ordinal] {
                Some(old) if old != c.source => {
                    return Err(invalid(
                        "source-bound provenance differs within source ordinal",
                    ))
                }
                None => {
                    identities[ordinal] = Some(c.source);
                    source_count += 1;
                }
                _ => {}
            }
            self.binding.validate_tokens(&[c.token_id])?;
            match c.action {
                SourceBoundAction::Copy { bank_index, .. } => {
                    if copy_count == MAX_COPY_ACTIONS {
                        return Err(invalid("source-bound copies exceed128"));
                    }
                    if bank_indices[..copy_count].contains(&Some(bank_index)) {
                        return Err(invalid("source-bound copy bank index repeats"));
                    }
                    bank_indices[copy_count] = Some(bank_index);
                    copy_count += 1;
                }
                SourceBoundAction::Period => {
                    if periods[ordinal] || c.token_id != self.binding.period_token_id() {
                        return Err(invalid(
                            "source-bound Period duplicated or token binding differs",
                        ));
                    }
                    periods[ordinal] = true;
                }
                SourceBoundAction::Stop => {
                    if stops[ordinal] || c.token_id != self.binding.eos_token_id() {
                        return Err(invalid(
                            "source-bound Stop duplicated or token binding differs",
                        ));
                    }
                    stops[ordinal] = true;
                }
            }
        }
        if (0..source_count).any(|i| identities[i].is_none() || !periods[i] || !stops[i])
            || identities[source_count..].iter().any(Option::is_some)
        {
            return Err(invalid(
                "source-bound sources must be dense with exactly one Period and Stop each",
            ));
        }
        for (offset, score) in self.scores[..n].iter_mut().enumerate() {
            *score = 0;
            for head in head_scores {
                *score = score.checked_add(head[offset]).ok_or_else(|| {
                    invalid(format!(
                        "source-bound head-score overflow at action {offset}"
                    ))
                })?;
            }
        }
        let rows = n - 1;
        let r = self
            .reducer
            .reduce(
                &self.scores[..rows],
                &self.zeros[..rows],
                self.scores[rows],
                &self.values[..rows],
            )
            .map_err(|e| invalid(format!("source-bound normalization: {e}")))?;
        let actions: Vec<_> = candidates
            .iter()
            .enumerate()
            .map(|(i, c)| SourceBoundActionMass {
                action_offset: i,
                action: c.action,
                source: c.source,
                token_id: c.token_id,
                score_q24: self.scores[i],
                weight_q31: if i == rows {
                    r.no_read_weight_q31
                } else {
                    r.occurrence_weights_q31[i]
                },
            })
            .collect();
        let mut token_masses: Vec<TokenMass> = Vec::with_capacity(n);
        let mut alias_comparisons = 0;
        for a in &actions {
            let mut found = None;
            for (i, t) in token_masses.iter().enumerate() {
                alias_comparisons += 1;
                if t.token_id == a.token_id {
                    found = Some(i);
                    break;
                }
            }
            if let Some(i) = found {
                let t = &mut token_masses[i];
                t.weight_q31 = t
                    .weight_q31
                    .checked_add(a.weight_q31)
                    .ok_or_else(|| invalid("source-bound alias mass overflow"))?;
                t.action_offsets.push(a.action_offset);
            } else {
                token_masses.push(TokenMass {
                    token_id: a.token_id,
                    weight_q31: a.weight_q31,
                    action_offsets: vec![a.action_offset],
                });
            }
        }
        token_masses.sort_unstable_by_key(|t| t.token_id);
        let mut total = 0u64;
        let mut chosen = None;
        for t in &token_masses {
            total = total
                .checked_add(t.weight_q31)
                .ok_or_else(|| invalid("source-bound total mass overflow"))?;
            if chosen.is_none_or(|old: &TokenMass| t.weight_q31 > old.weight_q31) {
                chosen = Some(t);
            }
        }
        if total != r.total_weight_q31 {
            return Err(invalid("source-bound token mass differs from denominator"));
        }
        let chosen = chosen.ok_or_else(|| invalid("source-bound actions have no token"))?;
        Ok(SourceBoundActionTrace {
            policy: POLICY,
            tokenizer_sha256: self.binding.tokenizer_sha256().to_owned(),
            protocol: self.binding.protocol().clone(),
            period_token_id: self.binding.period_token_id(),
            eos_token_id: self.binding.eos_token_id(),
            head_scores: head_scores.iter().map(|h| h.to_vec()).collect(),
            actions,
            chosen_token_id: chosen.token_id,
            chosen_weight_q31: chosen.weight_q31,
            token_masses,
            max_score_q24: r.max_score_q24,
            total_weight_q31: r.total_weight_q31,
            occurrence_rows: rows,
            source_count,
            copy_count,
            alias_comparisons,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_read::{EXP_TABLE_LEN, MAX_CONTEXT, WEIGHT_ONE};
    const TOKENIZER: &[u8] = br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"},{"id":9,"content":"<gap>"}]}"#;
    fn table() -> Vec<u32> {
        let mut t = vec![0; EXP_TABLE_LEN];
        t[0] = WEIGHT_ONE as u32;
        t[1] = (WEIGHT_ONE >> 1) as u32;
        t[2] = (WEIGHT_ONE >> 2) as u32;
        t
    }
    fn reducer(heads: usize) -> Result<NativeSourceBoundActions> {
        NativeSourceBoundActions::new(SourceActionBinding::new(TOKENIZER)?, heads, &table())
    }
    fn candidate(
        source_ordinal: usize,
        action: SourceBoundAction,
        token_id: u32,
    ) -> SourceBoundActionCandidate {
        SourceBoundActionCandidate {
            action,
            token_id,
            source: SourceBoundProvenance {
                source_ordinal,
                source_segment_index: source_ordinal,
                record: 1,
                commit: 7,
                event: 4,
            },
        }
    }
    fn one_source() -> Vec<SourceBoundActionCandidate> {
        vec![
            candidate(
                0,
                SourceBoundAction::Copy {
                    bank_index: 0,
                    source_offset: 0,
                },
                4,
            ),
            candidate(0, SourceBoundAction::Period, 3),
            candidate(0, SourceBoundAction::Stop, 1),
        ]
    }
    #[test]
    fn maximum_action_capacity_does_not_extend_public_context() -> Result<()> {
        assert!(NativeGeometricRead::new(MAX_CONTEXT + 1, 1, &table()).is_err());
        assert!(NativeGeometricRead::new_action_normalizer(384, &table()).is_err());
        let mut actions = Vec::new();
        for i in 0..MAX_SOURCES {
            actions.push(candidate(
                i,
                SourceBoundAction::Copy {
                    bank_index: i,
                    source_offset: 0,
                },
                4,
            ));
        }
        for i in 0..MAX_SOURCES {
            actions.push(candidate(i, SourceBoundAction::Period, 3));
            actions.push(candidate(i, SourceBoundAction::Stop, 1));
        }
        let scores = vec![0; MAX_ACTIONS];
        let trace = reducer(1)?.reduce(&actions, &[&scores])?;
        assert_eq!(trace.occurrence_rows, 383);
        assert_eq!(trace.actions.len(), 384);
        assert_eq!(trace.total_weight_q31, 384 * WEIGHT_ONE);
        assert_eq!(trace.source_count, 128);
        assert_eq!(
            trace
                .token_masses
                .iter()
                .map(|t| t.action_offsets.len())
                .sum::<usize>(),
            384
        );
        assert_eq!(trace.chosen_token_id, 1);
        Ok(())
    }
    #[test]
    fn aliases_retain_duplicate_record_occurrences_without_extra_stop() -> Result<()> {
        let actions = vec![
            candidate(
                0,
                SourceBoundAction::Copy {
                    bank_index: 0,
                    source_offset: 0,
                },
                1,
            ),
            candidate(
                1,
                SourceBoundAction::Copy {
                    bank_index: 1,
                    source_offset: 0,
                },
                3,
            ),
            candidate(0, SourceBoundAction::Period, 3),
            candidate(0, SourceBoundAction::Stop, 1),
            candidate(1, SourceBoundAction::Period, 3),
            candidate(1, SourceBoundAction::Stop, 1),
        ];
        let trace = reducer(1)?.reduce(&actions, &[&[0; 6]])?;
        assert_eq!(trace.actions.len(), 6);
        assert_eq!(trace.total_weight_q31, 6 * WEIGHT_ONE);
        assert_eq!(trace.token_masses[0].action_offsets, [0, 3, 5]);
        assert_eq!(trace.token_masses[1].action_offsets, [1, 2, 4]);
        assert_eq!(
            trace.actions[0].source.record,
            trace.actions[1].source.record
        );
        assert_ne!(
            trace.actions[0].source.source_ordinal,
            trace.actions[1].source.source_ordinal
        );
        assert_eq!(trace.chosen_token_id, 1);
        Ok(())
    }
    #[test]
    fn checked_head_sums_and_score_differences_reject_overflow() -> Result<()> {
        let actions = one_source();
        let mut read = reducer(2)?;
        let baseline = read.reduce(&actions, &[&[0; 3], &[0; 3]])?;
        for i in 0..3 {
            let mut a = [0; 3];
            let mut b = [0; 3];
            a[i] = i64::MAX;
            b[i] = 1;
            assert!(read.reduce(&actions, &[&a, &b]).is_err());
        }
        assert!(read
            .reduce(&actions, &[&[i64::MIN, i64::MAX, 0], &[0; 3]])
            .is_err());
        assert_eq!(read.reduce(&actions, &[&[0; 3], &[0; 3]])?, baseline);
        Ok(())
    }
    #[test]
    fn malformed_layouts_and_tokens_are_not_admitted() -> Result<()> {
        let valid = one_source();
        let mut read = reducer(1)?;
        assert!(read.reduce(&[], &[&[]]).is_err());
        assert!(read.reduce(&valid, &[]).is_err());
        assert!(read.reduce(&valid, &[&[0; 2]]).is_err());
        for mutation in 0..8 {
            let mut a = valid.clone();
            match mutation {
                0 => a[2].action = SourceBoundAction::Period,
                1 => a[1].action = SourceBoundAction::Stop,
                2 => a[1].token_id = 1,
                3 => a[0].token_id = 6,
                4 => a[0].source.record = 8,
                5 => {
                    for c in &mut a {
                        c.source.source_ordinal = 1;
                    }
                }
                6 => a[0].source.source_ordinal = 128,
                _ => a.insert(1, a[0]),
            }
            let zeros = vec![0; a.len()];
            assert!(read.reduce(&a, &[&zeros]).is_err(), "mutation {mutation}");
        }
        assert!(reducer(0).is_err());
        assert!(reducer(3).is_err());
        Ok(())
    }
    #[test]
    fn shared_source_offset_preserves_raw_within_source_contrasts() -> Result<()> {
        let a = one_source();
        let step = 1i64 << 16;
        let x = reducer(1)?.reduce(&a, &[&[step, 0, -step]])?;
        let y = reducer(1)?.reduce(&a, &[&[step + 17, 17, -step + 17]])?;
        assert_eq!(
            x.actions.iter().map(|a| a.weight_q31).collect::<Vec<_>>(),
            y.actions.iter().map(|a| a.weight_q31).collect::<Vec<_>>()
        );
        assert_eq!(x.chosen_token_id, y.chosen_token_id);
        assert_eq!(x.total_weight_q31, y.total_weight_q31);
        Ok(())
    }
    #[test]
    fn copy_only_shift_crosses_phase_and_terminal_aliases_can_win() -> Result<()> {
        let step = 1i64 << 16;
        let a = one_source();
        let parent = reducer(1)?.reduce(&a, &[&[step, 0, 0]])?;
        let copy_only = reducer(1)?.reduce(&a, &[&[0, 0, 0]])?;
        assert_eq!(parent.chosen_token_id, 4);
        assert_eq!(copy_only.chosen_token_id, 1);
        assert_ne!(
            parent.actions[0].score_q24 - parent.actions[1].score_q24,
            copy_only.actions[0].score_q24 - copy_only.actions[1].score_q24
        );
        let a = vec![
            candidate(
                0,
                SourceBoundAction::Copy {
                    bank_index: 0,
                    source_offset: 0,
                },
                4,
            ),
            candidate(
                1,
                SourceBoundAction::Copy {
                    bank_index: 1,
                    source_offset: 0,
                },
                5,
            ),
            candidate(0, SourceBoundAction::Period, 3),
            candidate(0, SourceBoundAction::Stop, 1),
            candidate(1, SourceBoundAction::Period, 3),
            candidate(1, SourceBoundAction::Stop, 1),
        ];
        let out = reducer(1)?.reduce(&a, &[&[step, -3 * step, 0, 0, 0, 0]])?;
        assert!(
            out.actions[0].score_q24
                > out.actions[1..]
                    .iter()
                    .map(|a| a.score_q24)
                    .max()
                    .ok_or_else(|| invalid("fixture max"))?
        );
        assert_eq!(out.chosen_token_id, 1);
        assert_eq!(out.token_masses[0].action_offsets, [3, 5]);
        assert_eq!(out.chosen_weight_q31, out.actions[0].weight_q31);
        Ok(())
    }
}
