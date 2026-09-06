//! Bounded learned placement, source selection and transported payload readout.
//! This residual block augments the existing predictor. It is not a dense
//! projection or a whole-context lookup. All products below read the existing
//! exact H4 group table; fitting and scalar angular-order construction are host work.

use super::{Control, Model};
use serde::{Deserialize, Serialize};

pub(super) const WINDOW: usize = 8;
pub(super) const ROOTS: usize = 120;
pub(super) const HEADS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingMode {
    Angular,
    /// Matched exact-code selector; H4 composition/readout remain the same.
    Equality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RoutingWork {
    pub predictions: u64,
    pub context_tokens_read: u64,
    pub code_reads: u64,
    pub table_reads: u64,
    pub sources_examined: u64,
    pub comparisons: u64,
    pub payload_gathers: u64,
    pub operator_executions: u64,
    pub emission_queries: u64,
    /// Logical operands read, not a physical cache/DRAM traffic measurement.
    pub logical_bytes_read: u64,
}
impl RoutingWork {
    pub(super) fn is_empty(&self) -> bool {
        *self == Self::default()
    }
    pub(super) fn add(&mut self, other: Self) {
        self.predictions = self.predictions.saturating_add(other.predictions);
        self.context_tokens_read = self
            .context_tokens_read
            .saturating_add(other.context_tokens_read);
        self.code_reads = self.code_reads.saturating_add(other.code_reads);
        self.table_reads = self.table_reads.saturating_add(other.table_reads);
        self.sources_examined = self.sources_examined.saturating_add(other.sources_examined);
        self.comparisons = self.comparisons.saturating_add(other.comparisons);
        self.payload_gathers = self.payload_gathers.saturating_add(other.payload_gathers);
        self.operator_executions = self
            .operator_executions
            .saturating_add(other.operator_executions);
        self.emission_queries = self.emission_queries.saturating_add(other.emission_queries);
        self.logical_bytes_read = self
            .logical_bytes_read
            .saturating_add(other.logical_bytes_read);
    }
    fn codes(&mut self, count: u64) {
        self.code_reads = self.code_reads.saturating_add(count);
        self.logical_bytes_read = self.logical_bytes_read.saturating_add(count << 1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RoutingHeadDecision {
    pub query: u16,
    pub source_offset: usize,
    pub source_token: u32,
    pub selected_key: u16,
    pub selected_value: u16,
    pub operator: u16,
    pub output: u16,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub heads: [RoutingHeadDecision; HEADS],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Emission {
    pub default_score: i32,
    pub scores: Vec<super::TokenScore>,
    pub postings: Vec<u32>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Head {
    pub queries: Vec<u16>,
    pub keys: Vec<u16>,
    pub values: Vec<u16>,
    pub operators: Vec<u16>,
    pub emissions: Vec<Emission>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RoutingBlock {
    pub schema: String,
    pub parent_artifact: String,
    pub mode: RoutingMode,
    pub heads: Vec<Head>,
    pub angular_rank: Vec<u16>,
    pub training: Vec<super::DocumentReceipt>,
    pub fit_config: super::RoutingFitConfig,
}

fn product(model: &Model, a: u16, b: u16, work: &mut RoutingWork) -> u16 {
    work.table_reads = work.table_reads.saturating_add(2);
    work.logical_bytes_read = work
        .logical_bytes_read
        .saturating_add(std::mem::size_of::<usize>() as u64 + 2);
    model.geometry.products[model.geometry.row_bases[usize::from(a)] + usize::from(b)]
}

impl Head {
    pub(super) fn route(
        &self,
        model: &Model,
        ranks: &[u16],
        mode: RoutingMode,
        context: &[u32],
        control: Control,
        work: &mut RoutingWork,
    ) -> RoutingHeadDecision {
        let last = context.first().copied().unwrap_or(super::BOS);
        let previous = context.get(1).copied().unwrap_or(super::BOS);
        work.codes(2);
        let query = product(
            model,
            self.queries[previous as usize],
            self.queries[last as usize],
            work,
        );
        let mut selected = last;
        let mut selected_key = self.keys[last as usize];
        work.codes(1);
        let mut selected_offset = 0;
        let mut best_rank = 0;
        for (offset, &token) in context.iter().enumerate() {
            let key = self.keys[token as usize];
            work.codes(1);
            work.sources_examined = work.sources_examined.saturating_add(1);
            let rank = match mode {
                RoutingMode::Angular => {
                    let inverse = model.geometry.inverses[usize::from(key)];
                    let relative = product(model, query, inverse, work);
                    work.table_reads = work.table_reads.saturating_add(2);
                    work.logical_bytes_read = work.logical_bytes_read.saturating_add(4);
                    ranks[usize::from(relative)]
                }
                RoutingMode::Equality => u16::from(query == key),
            };
            work.comparisons = work.comparisons.saturating_add(1);
            if offset == 0
                || (rank > best_rank && control != Control::LearnedRoutingSelectionDisabled)
            {
                best_rank = rank;
                selected = token;
                selected_key = key;
                selected_offset = offset;
            }
        }
        // Only the selected source's value code is gathered and transported.
        let value = self.values[selected as usize];
        let operator = if control == Control::LearnedRoutingTransformDisabled {
            model.geometry.identity
        } else {
            self.operators[usize::from(query)]
        };
        work.codes(2);
        work.payload_gathers = work.payload_gathers.saturating_add(1);
        let payload = product(model, query, value, work);
        let output = product(model, operator, payload, work);
        work.operator_executions = work.operator_executions.saturating_add(1);
        RoutingHeadDecision {
            query,
            source_offset: selected_offset,
            source_token: selected,
            selected_key,
            selected_value: value,
            operator,
            output,
        }
    }
}

impl RoutingBlock {
    pub(super) fn route(
        &self,
        model: &Model,
        context: &[u32],
        control: Control,
        work: &mut RoutingWork,
    ) -> RoutingDecision {
        work.predictions = work.predictions.saturating_add(1);
        work.context_tokens_read = work
            .context_tokens_read
            .saturating_add(context.len() as u64);
        work.logical_bytes_read = work
            .logical_bytes_read
            .saturating_add((context.len() as u64) << 2);
        let mut result = RoutingDecision::default();
        for (slot, head) in result.heads.iter_mut().zip(&self.heads) {
            *slot = head.route(model, &self.angular_rank, self.mode, context, control, work);
        }
        result
    }

    pub(super) fn score(
        &self,
        model: &Model,
        decision: RoutingDecision,
        token: u32,
        work: &mut RoutingWork,
    ) -> i64 {
        let mut score = 0;
        for (head, selected) in self.heads.iter().zip(decision.heads) {
            let row = &head.emissions[usize::from(selected.output)];
            work.emission_queries = work.emission_queries.saturating_add(1);
            if row.scores.is_empty() {
                continue;
            }
            let found = row.scores.binary_search_by(|entry| {
                work.comparisons = work.comparisons.saturating_add(1);
                work.logical_bytes_read = work.logical_bytes_read.saturating_add(4);
                entry.token.cmp(&token)
            });
            let conditional = found
                .map(|index| row.scores[index].score)
                .unwrap_or(row.default_score);
            work.table_reads = work.table_reads.saturating_add(2);
            work.logical_bytes_read = work.logical_bytes_read.saturating_add(8);
            // Bounded residual in the parent's quantized log-score units.
            score += (i64::from(conditional) - i64::from(model.prior_scores[token as usize])) >> 1;
        }
        score
    }
}
