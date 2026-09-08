//! Offline exact-record supervision of the shared geometric Stop/Copy/Add router.
//! Instruction words are learned prime identities, never a serving command parser.
use super::composed_output_training::reachable;
use super::operation_transition::{
    features, frame, prefix, prepare, proposal, until_stop, OperationTransition,
};
use super::source_routing::SourceCode;
use super::source_routing_training::{learn, Alternative, Frame};
use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MixedOperators {
    pub parent_artifact: String,
    pub previous_operation: OperationTransition,
    pub previous_roles: super::source_routing::SourceRouting,
}
impl MixedOperators {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        model
            .operation_transition
            .as_ref()
            .ok_or_else(|| Error("mixed operation router absent".into()))?
            .validate_shape(model)?;
        model
            .typed_roles
            .as_ref()
            .ok_or_else(|| Error("mixed roles absent".into()))?
            .router
            .validate_shape(model, 3, 6)?;
        let mut parent = model.clone();
        parent.mixed_operators = None;
        parent.operation_transition = Some(self.previous_operation.clone());
        parent
            .typed_roles
            .as_mut()
            .ok_or_else(|| Error("mixed role parent absent".into()))?
            .router = self.previous_roles.clone();
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("mixed operators frozen parent differs".into()));
        }
        parent.validate()?;
        let mut identity = model.clone();
        identity.refresh_identity()?;
        if identity.artifact_cid != model.artifact_cid
            || identity.uor_model_address != model.uor_model_address
        {
            return Err(Error("mixed operators identity differs".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MixedOperatorTarget {
    pub action: ValueAction,
    /// Exact committed occurrence IDs for offline supervision only.
    pub operand_ids: [u64; 2],
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MixedOperatorExample {
    pub context: OperationTransitionExample,
    pub first: MixedOperatorTarget,
    pub targets: Vec<MixedOperatorTarget>,
}
fn exact_frame(
    block: &OperationTransition,
    s: &Session,
    target: &MixedOperatorTarget,
) -> Result<(Frame, (ValueAction, [value_types::ValueRecord; 2]))> {
    let values = s
        .values
        .as_ref()
        .ok_or_else(|| Error("mixed typed state absent".into()))?;
    let (f, n) = features(block, values, None, &mut Default::default());
    let mut alternatives = vec![Alternative {
        features: f[..n].to_vec(),
        codes: vec![],
        action: 0,
        correct: false,
    }];
    let mut chosen = None;
    for index in 0..272 {
        if let Some((action, pair)) = proposal(values, index) {
            if !super::joint_admission::legal(action, pair[0].value, pair[1].value) {
                continue;
            }
            let correct = action == target.action && pair.map(|r| r.id) == target.operand_ids;
            if correct {
                chosen = Some((action, pair));
            }
            let (f, n) = features(block, values, Some(pair), &mut Default::default());
            alternatives.push(Alternative {
                features: f[..n].to_vec(),
                codes: vec![],
                action: if action == ValueAction::Copy { 1 } else { 2 },
                correct,
            });
        }
    }
    Ok((
        Frame { alternatives },
        chosen.ok_or_else(|| {
            Error("exact mixed target unavailable or invalid operand order".into())
        })?,
    ))
}
/// Learn a rejection margin without changing state, features or serving controls.
/// When all labelled operations are unavailable, Stop must beat the remaining
/// substitutes. Positive frames still teach the labelled operation to beat Stop.
fn rejection_frame(frame: &Frame) -> Option<Frame> {
    if !frame
        .alternatives
        .iter()
        .any(|a| a.correct && a.action != 0)
    {
        return None;
    }
    Some(Frame {
        alternatives: frame
            .alternatives
            .iter()
            .filter(|a| !(a.correct && a.action != 0))
            .map(|a| Alternative {
                features: a.features.clone(),
                codes: vec![],
                action: a.action,
                correct: a.action == 0,
            })
            .collect(),
    })
}
fn extend_dictionary(block: &mut OperationTransition, docs: &[MixedOperatorExample]) -> Result<()> {
    let mut words: BTreeSet<Vec<u8>> = block
        .dictionary
        .iter()
        .map(|w| w.bytes[..usize::from(w.len)].to_vec())
        .collect();
    for d in docs {
        for w in d
            .context
            .query
            .as_bytes()
            .split(|b| !b.is_ascii_alphabetic() && *b != b'_')
        {
            if !w.is_empty() && w.len() <= 32 {
                words.insert(w.to_ascii_lowercase());
            }
        }
    }
    if words.len() > 256 {
        return Err(Error("mixed dictionary exceeds256".into()));
    }
    let primes = crate::corpus_induced_spin_placement::first_primes(words.len())
        .map_err(|e| Error(e.to_string()))?;
    let dictionary: Vec<_> = words
        .into_iter()
        .zip(primes)
        .map(|(w, p)| {
            let mut bytes = [0; 32];
            bytes[..w.len()].copy_from_slice(&w);
            word_copy_types::WordCopyAddress {
                bytes,
                len: w.len() as u8,
                prime: p as u32,
            }
        })
        .collect();
    let mut remap = BTreeMap::new();
    for old in &block.dictionary {
        let new = dictionary
            .iter()
            .find(|w| w.bytes == old.bytes && w.len == old.len)
            .ok_or_else(|| Error("mixed dictionary lost prior identity".into()))?;
        remap.insert(u64::from(old.prime), u64::from(new.prime));
    }
    for code in &mut block.router.codes {
        let key = match code.feature.kind {
            4 => Some(&mut code.feature.a),
            5 => Some(&mut code.feature.b),
            _ => None,
        };
        if let Some(key) = key {
            *key = *remap.get(key).ok_or_else(|| {
                Error("mixed code refers to absent prior lexical identity".into())
            })?;
        }
    }
    block.router.codes.sort_by_key(|c| c.feature);
    block.dictionary = dictionary;
    Ok(())
}
impl Model {
    pub fn fit_mixed_operators(
        &self,
        docs: &[MixedOperatorExample],
        preservation: &[OperationTransitionExample],
        prompts: &[Document],
        config: SourceRoutingConfig,
        mut checkpoint: impl FnMut(&Model, &serde_json::Value) -> Result<()>,
    ) -> Result<(Model, serde_json::Value)> {
        config.validate()?;
        if self.mixed_operators.is_some()
            || self.composed_output.is_none()
            || docs.is_empty()
            || docs.len() + preservation.len() > 512
        {
            return Err(Error("invalid mixed operator construction".into()));
        }
        let mut ids = BTreeSet::new();
        for (d, targets) in docs
            .iter()
            .map(|d| (&d.context, Some(&d.targets)))
            .chain(preservation.iter().map(|d| (d, None)))
        {
            if d.id.is_empty()
                || !ids.insert(&d.id)
                || d.query.len() > 4096
                || d.history.len() > 4
                || d.first_response.len() > 128
                || d.history
                    .iter()
                    .any(|t| t.prompt.len() > 4096 || t.response.len() > 128)
                || targets.is_some_and(|t| {
                    t.len() > 1
                        || d.next.is_some()
                        || t.iter()
                            .any(|t| !matches!(t.action, ValueAction::Copy | ValueAction::Add))
                })
            {
                return Err(Error("invalid mixed operator document or target".into()));
            }
        }
        let old = self
            .operation_transition
            .as_ref()
            .ok_or_else(|| Error("mixed operation parent absent".into()))?;
        let (roles, initial_fit) = self.fit_mixed_initial(docs, preservation, prompts, &config)?;
        let mut initial = self.clone();
        initial
            .typed_roles
            .as_mut()
            .ok_or_else(|| Error("mixed initial roles absent".into()))?
            .router = roles;
        initial.mixed_operators = Some(MixedOperators {
            parent_artifact: self.artifact_cid.clone(),
            previous_operation: old.clone(),
            previous_roles: self
                .typed_roles
                .as_ref()
                .ok_or_else(|| Error("mixed prior roles absent".into()))?
                .router
                .clone(),
        });
        initial.refresh_identity()?;
        initial.validate()?;
        checkpoint(&initial, &initial_fit)?;
        let mut block = old.clone();
        extend_dictionary(&mut block, docs)?;
        let mut frames = Vec::new();
        let mut receipts = Vec::new();
        let mut rollout = Vec::new();
        let mut unavailable = 0;
        for (d, targets) in docs
            .iter()
            .map(|d| (&d.context, Some(&d.targets)))
            .chain(preservation.iter().map(|d| (d, None)))
        {
            let mut s = prefix(&initial, d)?;
            s.control = Control::OperationTransitionDisabled;
            let before_writes = s.work.values.derived_writes;
            let first = until_stop(&initial, &mut s)?;
            if first != d.first_response {
                return Err(Error(format!(
                    "mixed first response failed {}: {first:?}",
                    d.id
                )));
            }
            if let Some(example) = docs.iter().find(|example| example.context.id == d.id) {
                let derivation = s
                    .values
                    .as_ref()
                    .and_then(|v| v.records.last())
                    .and_then(|record| record.derivation);
                if s.work.values.derived_writes.checked_sub(before_writes) != Some(1)
                    || !derivation.is_some_and(|actual| {
                        actual.action == example.first.action
                            && actual.operand_ids == example.first.operand_ids
                    })
                {
                    return Err(Error(format!("mixed first derivation differs {}", d.id)));
                }
            }
            let mut selected = None;
            if reachable(&s, block.max_operations) {
                if let Some(target) = targets.and_then(|t| t.first()) {
                    let (f, c) = exact_frame(&block, &s, target)?;
                    frames.push(f);
                    selected = Some(c);
                } else {
                    let (f, c) = frame(&block, &s, if targets.is_some() { None } else { d.next })?;
                    frames.push(f);
                    selected = c;
                }
            } else if targets.is_some_and(|t| !t.is_empty()) || d.next.is_some() {
                return Err(Error(format!(
                    "mixed positive boundary unavailable {}",
                    d.id
                )));
            } else {
                unavailable += 1;
            }
            let mut next = None;
            if let Some((action, pair)) = selected {
                let values = s
                    .values
                    .as_mut()
                    .ok_or_else(|| Error("mixed values absent".into()))?;
                let p = prepare(values, action, pair, 1, &mut s.work.values)
                    .ok_or_else(|| Error("mixed supervised operation cannot execute".into()))?;
                s.completion
                    .as_mut()
                    .ok_or_else(|| Error("mixed completion absent".into()))?
                    .reset();
                s.observe(&initial, p.token)?;
                let mut bytes = self.decode(&[p.token])?;
                bytes.extend(until_stop(&initial, &mut s)?.as_bytes());
                next = Some(String::from_utf8(bytes).map_err(|e| Error(e.to_string()))?);
                if !reachable(&s, block.max_operations) {
                    return Err(Error(format!(
                        "mixed learned stop boundary unavailable {}",
                        d.id
                    )));
                }
                frames.push(frame(&block, &s, None)?.0);
            }
            rollout.push(serde_json::json!({"id":d.id,"first":first,"supervised_next":next,"actual_records":s.values.as_ref().map(|v|&v.records)}));
            let bytes = serde_json::to_vec(&(d, targets)).map_err(|e| Error(e.to_string()))?;
            receipts.push(DocumentReceipt {
                id: d.id.clone(),
                bytes: bytes.len(),
                text_cid: format!("blake3:{}", blake3::hash(&bytes)),
            });
        }
        let rejection_frames: Vec<_> = frames.iter().filter_map(rejection_frame).collect();
        let rejection_count = rejection_frames.len();
        frames.extend(rejection_frames);
        if frames.len() > 4096 {
            return Err(Error("mixed frame cap exceeded".into()));
        }
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
                "mixed feature cap exceeded {}",
                block.router.codes.len()
            )));
        }
        block.router.config = config;
        block.router.training = receipts;
        let fit = learn(self, &mut block.router, &mut frames, Instant::now());
        let mut model = initial;
        model.operation_transition = Some(block);
        model.mixed_operators = Some(MixedOperators {
            parent_artifact: self.artifact_cid.clone(),
            previous_operation: old.clone(),
            previous_roles: self
                .typed_roles
                .as_ref()
                .ok_or_else(|| Error("mixed old roles absent".into()))?
                .router
                .clone(),
        });
        model.refresh_identity()?;
        model.validate()?;
        Ok((
            model,
            serde_json::json!({"parent":self.artifact_cid(),"initial_fit":initial_fit,"fit":fit,"presented_frames":presented,"rejection_margin_frames":rejection_count,"rejection_objective":"correct operation > Stop > incorrect substitutes; remove all labelled operation candidates while retaining state and feature values","unique_frames":frames.len(),"skipped_ineligible_boundaries":unavailable,"rollout":rollout,"supervision":"offline exact action and committed occurrence IDs; no target IDs or text in serving state","serving":"unchanged bounded signed-H4 features and proposals; expanded learned lexical dictionary"}),
        ))
    }
}

#[cfg(test)]
#[path = "mixed_operator_tests.rs"]
mod tests;
