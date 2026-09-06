//! Two dependent reads with a shared Base/Emit/EOS decision. BOS names the
//! Base action here; it is never emitted. This module is integer/table serving.
use super::learned_routing::{product, Emission};
use super::{Error, Model, Result, RoutingDecision, RoutingWork, BOS, EOS};
use serde::{Deserialize, Serialize};

pub(super) const SCHEMA: &str = "uor-r4.learned-h4-routing/2";
pub(super) const FEATURES: usize = 5;
pub(super) const CANDIDATES: usize = 42;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JointRow {
    pub key: u32,
    pub emission: Emission,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JointOutput {
    #[serde(default, skip_serializing_if = "is_false")]
    pub additive_scores: bool,
    pub rows: Vec<JointRow>,
    pub priors: Vec<i32>,
    pub positions: usize,
    pub response_examples: usize,
}

fn is_false(value: &bool) -> bool {
    !*value
}

pub(super) fn features(
    model: &Model,
    decision: RoutingDecision,
    baseline_token: u32,
    flags: u32,
    work: &mut RoutingWork,
) -> [u32; FEATURES] {
    let [first, second] = decision.heads;
    let inverse = model.geometry.inverses[usize::from(first.output)];
    work.table_reads = work.table_reads.saturating_add(1);
    work.logical_bytes_read = work.logical_bytes_read.saturating_add(2);
    let relative = product(model, second.output, inverse, work);
    [
        u32::from(first.output),
        0x10000 | u32::from(second.output),
        0x20000 | u32::from(relative),
        0x30000 | baseline_token,
        0x40000 | flags,
    ]
}

impl JointOutput {
    pub(super) fn validate(&self, vocab: usize) -> Result<()> {
        let maximum = if self.additive_scores { 32767 } else { 0 };
        if self.rows.len() > 8192
            || self.priors.len() != vocab
            || self
                .priors
                .iter()
                .any(|&s| !(-32768..=maximum).contains(&s))
            || self.positions == 0
            || self.positions > 8192
            || self.response_examples > 256
            || self.rows.windows(2).any(|p| p[0].key >= p[1].key)
        {
            return Err(Error("recurrent routing output shape invalid".into()));
        }
        for row in &self.rows {
            let domain = row.key >> 16;
            let index = row.key & 0xffff;
            if !match domain {
                0..=2 => index < 120,
                3 => index < vocab as u32,
                4 => index < 8,
                _ => false,
            } || row.emission.scores.len() > vocab
                || row
                    .emission
                    .scores
                    .windows(2)
                    .any(|p| p[0].token >= p[1].token)
                || row
                    .emission
                    .scores
                    .iter()
                    .any(|p| p.token as usize >= vocab || !(-32768..=maximum).contains(&p.score))
                || (if self.additive_scores {
                    row.emission.default_score != 0
                } else {
                    !(-32768..=0).contains(&row.emission.default_score)
                })
                || row.emission.postings.len() > 8
                || row.emission.postings.iter().any(|token| {
                    row.emission
                        .scores
                        .binary_search_by_key(token, |p| p.token)
                        .is_err()
                })
            {
                return Err(Error("recurrent routing output row invalid".into()));
            }
        }
        Ok(())
    }

    pub(super) fn rows_for(
        &self,
        keys: [u32; FEATURES],
        work: &mut RoutingWork,
    ) -> [Option<usize>; FEATURES] {
        keys.map(|key| {
            work.emission_queries = work.emission_queries.saturating_add(1);
            self.rows
                .binary_search_by(|row| {
                    work.comparisons = work.comparisons.saturating_add(1);
                    work.logical_bytes_read = work.logical_bytes_read.saturating_add(4);
                    row.key.cmp(&key)
                })
                .ok()
        })
    }

    pub(super) fn candidates(
        &self,
        rows: [Option<usize>; FEATURES],
        baseline: u32,
        work: &mut RoutingWork,
    ) -> ([u32; CANDIDATES], usize) {
        let mut result = [BOS; CANDIDATES];
        let mut count = 1;
        if baseline != EOS {
            result[count] = EOS;
            count += 1;
        }
        for index in rows.into_iter().flatten() {
            for &token in &self.rows[index].emission.postings {
                work.table_reads = work.table_reads.saturating_add(1);
                work.logical_bytes_read = work.logical_bytes_read.saturating_add(4);
                work.comparisons = work.comparisons.saturating_add(1);
                if token == baseline
                    || result[..count].iter().any(|&t| {
                        work.comparisons = work.comparisons.saturating_add(1);
                        t == token
                    })
                {
                    continue;
                }
                result[count] = token;
                count += 1;
            }
        }
        (result, count)
    }

    pub(super) fn score(
        &self,
        rows: [Option<usize>; FEATURES],
        action: u32,
        work: &mut RoutingWork,
    ) -> i64 {
        let prior = i64::from(self.priors[action as usize]);
        work.table_reads = work.table_reads.saturating_add(1);
        work.logical_bytes_read = work.logical_bytes_read.saturating_add(4);
        let mut score = prior;
        for index in rows.into_iter().flatten() {
            let row = &self.rows[index].emission;
            work.emission_queries = work.emission_queries.saturating_add(1);
            let position = row.scores.binary_search_by(|entry| {
                work.comparisons = work.comparisons.saturating_add(1);
                work.logical_bytes_read = work.logical_bytes_read.saturating_add(4);
                entry.token.cmp(&action)
            });
            let conditional = position
                .map(|i| row.scores[i].score)
                .unwrap_or(row.default_score);
            work.table_reads = work.table_reads.saturating_add(1);
            work.logical_bytes_read = work.logical_bytes_read.saturating_add(4);
            score += if self.additive_scores {
                i64::from(conditional)
            } else {
                (i64::from(conditional) - prior) >> 1
            };
        }
        score
    }

    pub(super) fn choose(
        &self,
        keys: [u32; FEATURES],
        baseline: u32,
        work: &mut RoutingWork,
    ) -> (u32, i64, i64) {
        let rows = self.rows_for(keys, work);
        let (candidates, count) = self.candidates(rows, baseline, work);
        let base_score = self.score(rows, BOS, work);
        let mut best = (BOS, base_score);
        for &action in &candidates[1..count] {
            let score = self.score(rows, action, work);
            work.comparisons = work.comparisons.saturating_add(2);
            if score > best.1 || (score == best.1 && action < best.0) {
                best = (action, score);
            }
        }
        (best.0, best.1, base_score)
    }
}
