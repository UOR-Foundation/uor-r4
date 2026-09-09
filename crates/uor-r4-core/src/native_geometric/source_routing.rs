//! Learned ordered H4 metadata routes to existing exact word occurrences.
//! No payload is reconstructed from a geometric code. The selected reference
//! is handed to the existing causal copy operator. This is integer/table serving.
use super::learned_routing::product;
use super::role_read::{self, NO_SOURCE};
use super::value_types::{ValueFeature, ValueState};
use super::word_copy_types::WordCopyWork;
use super::*;

pub(super) const LANES: usize = 2;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceCode {
    pub feature: ValueFeature,
    pub roots: [u16; LANES],
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceRouting {
    pub schema: String,
    pub parent_artifact: String,
    pub codes: Vec<SourceCode>,
    pub landmarks: Vec<[u16; LANES]>,
    pub biases: Vec<i16>,
    pub ranks: Vec<u16>,
    pub training: Vec<DocumentReceipt>,
    pub config: SourceRoutingConfig,
}
/// A single, nonexecuting replacement witness. Descendant components remain
/// exactly as trained; restoring `previous` reconstructs their frozen parent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceRoutingRefinement {
    pub parent_artifact: String,
    pub previous: SourceRouting,
}
impl SourceRouting {
    pub(super) fn encode(
        &self,
        model: &Model,
        features: &[ValueFeature],
        control: Control,
        work: &mut RoutingWork,
    ) -> [u16; LANES] {
        let mut state = [model.geometry.identity; LANES];
        for feature in features {
            work.emission_queries = work.emission_queries.saturating_add(1);
            let found = self.codes.binary_search_by(|entry| {
                work.comparisons = work.comparisons.saturating_add(1);
                work.logical_bytes_read = work.logical_bytes_read.saturating_add(17);
                entry.feature.cmp(feature)
            });
            if let Ok(index) = found {
                for (root, &code) in state.iter_mut().zip(&self.codes[index].roots) {
                    work.code_reads = work.code_reads.saturating_add(1);
                    work.logical_bytes_read = work.logical_bytes_read.saturating_add(2);
                    let code = if control == Control::LearnedRoutingTransformDisabled {
                        model.geometry.identity
                    } else {
                        code
                    };
                    *root = product(model, *root, code, work);
                }
            }
        }
        state
    }
    pub(super) fn score(
        &self,
        model: &Model,
        state: [u16; LANES],
        action: usize,
        work: &mut RoutingWork,
    ) -> i64 {
        let mut score = i64::from(self.biases[action]);
        work.table_reads = work.table_reads.saturating_add(1);
        work.logical_bytes_read = work.logical_bytes_read.saturating_add(2);
        for (root, &landmark) in state.into_iter().zip(&self.landmarks[action]) {
            let inverse = model.geometry.inverses[usize::from(landmark)];
            work.table_reads = work.table_reads.saturating_add(2);
            work.logical_bytes_read = work.logical_bytes_read.saturating_add(4);
            let relative = product(model, root, inverse, work);
            score += match self.config.mode {
                RoutingMode::Angular => {
                    work.table_reads = work.table_reads.saturating_add(1);
                    work.logical_bytes_read = work.logical_bytes_read.saturating_add(2);
                    i64::from(self.ranks[usize::from(relative)])
                }
                RoutingMode::Equality => {
                    work.comparisons = work.comparisons.saturating_add(1);
                    if relative == model.geometry.identity {
                        work.table_reads = work.table_reads.saturating_add(1);
                        work.logical_bytes_read = work.logical_bytes_read.saturating_add(2);
                        i64::from(self.ranks[usize::from(model.geometry.identity)])
                    } else {
                        0
                    }
                }
            };
        }
        score
    }
}

pub(super) fn allowed(model: &Model, values: &ValueState, source: u8, action: usize) -> bool {
    let Some(head) = role_read::head(model) else {
        return false;
    };
    let a = &head.actions[action];
    if a.copy != (source != NO_SOURCE) {
        return false;
    }
    if !a.copy {
        return true;
    }
    super::relation::source(values, source).is_some_and(|w| {
        usize::from(w.len) + 1 + usize::from(a.prefix.is_some())
            <= usize::from(super::response_entry_types::RESPONSE_ENTRY_STEPS)
    })
}

pub(super) fn choose(
    model: &Model,
    values: &ValueState,
    control: Control,
    work: &mut WordCopyWork,
) -> Option<(u8, usize, i64)> {
    let block = if control == Control::SourceRoleRefinementDisabled {
        model
            .source_role_refinement
            .as_ref()
            .map(|w| &w.previous)
            .or(model.source_routing.as_ref())?
    } else if control == Control::SourceContextDisabled {
        model
            .source_context
            .as_ref()
            .map(|w| &w.previous)
            .or(model.source_routing.as_ref())?
    } else {
        model.source_routing.as_ref()?
    };
    let retained = model.source_context.is_some()
        && !matches!(
            control,
            Control::SourceContextDisabled | Control::SourceContextWindowOnly
        );
    let read = role_read::head(model)?;
    let words = values.lexemes.as_ref()?;
    let feature_control = if block.config.role_context_only {
        Control::WordCopyGeometryDisabled
    } else {
        control
    };
    let ctx = role_read::context(model, values, feature_control, work);
    work.routing.predictions = work.routing.predictions.saturating_add(1);
    let mut best = None;
    for index in 0..=words.query_len {
        let source = if index == words.query_len {
            NO_SOURCE
        } else {
            index as u8
        };
        let (features, n) = role_read::features_with_context(
            model,
            values,
            &ctx,
            index,
            feature_control,
            retained,
            work,
        );
        let state = block.encode(model, &features[..n], control, &mut work.routing);
        work.routing.sources_examined = work.routing.sources_examined.saturating_add(1);
        work.word_candidates = work
            .word_candidates
            .saturating_add(u64::from(source != NO_SOURCE));
        for action in 0..read.actions.len() {
            if !allowed(model, values, source, action) {
                continue;
            }
            let score = block.score(model, state, action, &mut work.routing);
            work.routing.comparisons = work.routing.comparisons.saturating_add(1);
            if best.is_none_or(|(_, _, prior)| score > prior) {
                best = Some((source, action, score));
            }
        }
    }
    // The subsequent existing offer/observe path owns the actual byte gather,
    // selected operator, source-version checks and commit. Do not double-count it.
    best
}
