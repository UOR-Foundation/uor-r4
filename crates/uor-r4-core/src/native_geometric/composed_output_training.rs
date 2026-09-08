//! Offline continuation of the shared operator router at formatted boundaries.
use super::operation_transition::{frame, prefix, prepare, until_stop};
use super::source_routing::SourceCode;
use super::source_routing_training::learn;
use super::*;
use std::{collections::BTreeSet, time::Instant};

pub(super) fn reachable(s: &Session, max_operations: u8) -> bool {
    s.values
        .as_ref()
        .zip(s.completion.as_ref())
        .is_some_and(|(v, c)| {
            let latest = v
                .records
                .iter()
                .rev()
                .find(|r| r.derived && r.start >= v.started_at)
                .map(|r| r.id);
            v.next_id != u64::MAX
                && v.can_transition()
                && v.operations_committed < max_operations
                && c.pending.is_some_and(|d| {
                    d.token == EOS && d.at_seen == v.seen && Some(d.write_id) == latest
                })
        })
}

impl Model {
    pub fn fit_composed_output(
        &self,
        docs: &[OperationTransitionExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        config.validate()?;
        if self.composed_output.is_some()
            || self.lexical_emission.is_none()
            || docs.is_empty()
            || docs.len() > 384
            || docs.iter().any(|d| {
                d.id.is_empty()
                    || d.query.len() > 4096
                    || d.history.len() > 4
                    || d.history
                        .iter()
                        .any(|t| t.prompt.len() > 4096 || t.response.len() > 128)
                    || d.first_response.len() > 128
                    || d.next
                        .is_some_and(|(a, _)| !matches!(a, ValueAction::Copy | ValueAction::Add))
            })
        {
            return Err(Error("invalid composed output construction".into()));
        }
        let old = self
            .operation_transition
            .as_ref()
            .ok_or_else(|| Error("composed output requires existing operation router".into()))?;
        let mut block = old.clone();
        let mut frames = Vec::new();
        let mut rollout = Vec::new();
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut unavailable_boundaries = 0;
        for d in docs {
            if !ids.insert(&d.id) {
                return Err(Error("duplicate composed output construction id".into()));
            }
            let mut s = prefix(self, d)?;
            // The parent generates the complete first segment. Only its next
            // operation is suppressed while collecting an offline decision frame.
            s.control = Control::OperationTransitionDisabled;
            let first = until_stop(self, &mut s)?;
            if first != d.first_response {
                return Err(Error(format!(
                    "composed first response failed {}: {first:?}",
                    d.id
                )));
            }
            let available = reachable(&s, block.max_operations);
            let chosen = if available {
                let (f, chosen) = frame(&block, &s, d.next)?;
                frames.push(f);
                chosen
            } else {
                if d.next.is_some() {
                    return Err(Error(format!(
                        "requested composed transition has no eligible boundary: {}",
                        d.id
                    )));
                }
                unavailable_boundaries += 1;
                None
            };
            let mut second = None;
            if let Some((action, pair)) = chosen {
                let v = s
                    .values
                    .as_mut()
                    .ok_or_else(|| Error("typed state absent".into()))?;
                let candidate = prepare(v, action, pair, 1, &mut s.work.values)
                    .ok_or_else(|| Error("supervised composed operator unavailable".into()))?;
                s.completion
                    .as_mut()
                    .ok_or_else(|| Error("completion absent".into()))?
                    .reset();
                s.observe(self, candidate.token)?;
                let mut bytes = self.decode(&[candidate.token])?;
                bytes.extend(until_stop(self, &mut s)?.as_bytes());
                second = Some(String::from_utf8(bytes).map_err(|e| Error(e.to_string()))?);
                if !reachable(&s, block.max_operations) {
                    return Err(Error(format!(
                        "supervised composed response has no eligible stop boundary: {}",
                        d.id
                    )));
                }
                frames.push(frame(&block, &s, None)?.0);
            }
            rollout.push(serde_json::json!({"id":d.id,"first":first,"eligible_transition_boundary":available,"supervised_operator_response":second,"actual_records":s.values.as_ref().map(|v|&v.records)}));
            let bytes = serde_json::to_vec(d).map_err(|e| Error(e.to_string()))?;
            receipts.push(DocumentReceipt {
                id: d.id.clone(),
                bytes: bytes.len(),
                text_cid: format!("blake3:{}", blake3::hash(&bytes)),
            });
        }
        if frames.len() > 4096 {
            return Err(Error("composed construction frames exceed4096".into()));
        }
        let vocab: BTreeSet<_> = frames
            .iter()
            .flat_map(|f| {
                f.alternatives
                    .iter()
                    .flat_map(|a| a.features.iter().copied())
            })
            .collect();
        for feature in vocab {
            if block.router.codes.iter().all(|c| c.feature != feature) {
                block.router.codes.push(SourceCode {
                    feature,
                    roots: [self.geometry.identity; 2],
                });
            }
        }
        block.router.codes.sort_by_key(|c| c.feature);
        if block.router.codes.len() > config.learned_features {
            return Err(Error(format!(
                "composed feature cap exceeded: {}",
                block.router.codes.len()
            )));
        }
        block.router.config = config;
        block.router.training = receipts;
        let presented = frames.len();
        let mut unique = BTreeSet::new();
        frames.retain(|f| {
            unique.insert(
                f.alternatives
                    .iter()
                    .map(|a| (a.features.clone(), a.action, a.correct))
                    .collect::<Vec<_>>(),
            )
        });
        let fit = learn(self, &mut block.router, &mut frames, Instant::now());
        let added = block.router.codes.len() - old.router.codes.len();
        let mut model = self.clone();
        model.operation_transition = Some(block);
        model.composed_output = Some(super::composed_output::ComposedOutput {
            parent_artifact: self.artifact_cid.clone(),
            previous_operation: old.router.clone(),
        });
        model.refresh_identity()?;
        model.validate()?;
        Ok((
            model,
            serde_json::json!({"fit":fit,"parent":self.artifact_cid(),"presented_frames":presented,"unique_frames":frames.len(),"features_added":added,"unavailable_boundaries":unavailable_boundaries,"rollout":rollout,"offline_action_supervision":true,"serving_feature_law":"unchanged shared Stop/Copy/Add router; frozen lexical emitter and dictionaries"}),
        ))
    }
}
