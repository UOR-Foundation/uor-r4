//! Offline refinement of existing shared initial roles and continuation.
//! The lexical model and earlier action-emission witness remain exact. This
//! witness stores provenance; serving uses the same geometric shared operators.
use super::action_emission::{operation_frame_conflicts, operation_misclassified_frames};
use super::composed_output_training::reachable;
use super::mixed_operators::{exact_frame, extend_dictionary, rejection_frame};
use super::operation_transition::{frame, prefix, prepare, until_stop, OperationTransition};
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{learn, Frame};
use super::*;
use std::{collections::BTreeSet, time::Instant};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SharedOperatorRefinement {
    pub parent_artifact: String,
    pub previous_roles: SourceRouting,
    pub previous_operation: OperationTransition,
    /// Records completion of the fit stage, never capability acceptance.
    pub continuation_fitted: bool,
}
impl SharedOperatorRefinement {
    fn parent(&self, model: &Model) -> Result<Model> {
        let mut parent = model.clone();
        parent.shared_operator_refinement = None;
        parent
            .typed_roles
            .as_mut()
            .ok_or_else(|| Error("shared operator role parent absent".into()))?
            .router = self.previous_roles.clone();
        parent.operation_transition = Some(self.previous_operation.clone());
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("shared operator frozen parent differs".into()));
        }
        parent.validate()?;
        Ok(parent)
    }
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        model
            .typed_roles
            .as_ref()
            .ok_or_else(|| Error("shared operator roles absent".into()))?
            .router
            .validate_shape(model, 3, 6)?;
        model
            .operation_transition
            .as_ref()
            .ok_or_else(|| Error("shared operator transition absent".into()))?
            .validate_shape(model)?;
        self.parent(model)?;
        let mut identity = model.clone();
        identity.refresh_identity()?;
        if identity.artifact_cid != model.artifact_cid
            || identity.uor_model_address != model.uor_model_address
        {
            return Err(Error("shared operator identity differs".into()));
        }
        Ok(())
    }
}
struct Frames {
    values: Vec<Frame>,
    rejection_count: usize,
    unavailable_stops: usize,
}
impl Frames {
    fn push(&mut self, frame: Frame) -> Result<()> {
        let rejected = rejection_frame(&frame);
        if self.values.len() + 1 + usize::from(rejected.is_some()) > 4096 {
            return Err(Error("shared operator frame cap4096".into()));
        }
        self.values.push(frame);
        if let Some(rejected) = rejected {
            self.values.push(rejected);
            self.rejection_count += 1;
        }
        Ok(())
    }
}
fn validate_context(d: &OperationTransitionExample, ids: &mut BTreeSet<String>) -> Result<()> {
    if d.id.trim().is_empty()
        || !ids.insert(d.id.clone())
        || d.query.len() > 4096
        || d.first_response.len() > 128
        || d.history.len() > 4
        || d.history
            .iter()
            .any(|h| h.prompt.len() > 4096 || h.response.len() > 128)
    {
        return Err(Error("invalid shared operator context bounds/id".into()));
    }
    Ok(())
}
fn validate_inputs(
    docs: &[MixedOperatorExample],
    preservation: &[OperationTransitionExample],
) -> Result<()> {
    if docs.is_empty() || docs.len() > 128 || preservation.len() > 640 {
        return Err(Error("invalid shared operator population".into()));
    }
    let mut ids = BTreeSet::new();
    for d in docs {
        validate_context(&d.context, &mut ids)?;
        if d.context.next.is_some()
            || d.targets.len() > 2
            || std::iter::once(&d.first).chain(&d.targets).any(|t| {
                !matches!(t.action, ValueAction::Copy | ValueAction::Add)
                    || (t.action == ValueAction::Copy && t.operand_ids[0] != t.operand_ids[1])
            })
        {
            return Err(Error("invalid shared operator exact targets".into()));
        }
    }
    for d in preservation {
        validate_context(d, &mut ids)?;
    }
    Ok(())
}
fn verify_write(s: &Session, before: u64, target: &MixedOperatorTarget, id: &str) -> Result<()> {
    let derivation = s
        .values
        .as_ref()
        .and_then(|v| v.records.last())
        .and_then(|r| r.derivation);
    if s.work.values.derived_writes.checked_sub(before) != Some(1)
        || !derivation
            .is_some_and(|d| d.action == target.action && d.operand_ids == target.operand_ids)
    {
        return Err(Error(format!(
            "shared operator exact generated derivation differs: {id}"
        )));
    }
    Ok(())
}
fn collect_response(
    parent: &Model,
    block: &OperationTransition,
    s: &mut Session,
    frames: &mut Frames,
) -> Result<String> {
    let mut out = Vec::new();
    for _ in 0..96 {
        let was_completion = s.completion.as_ref().is_some_and(|c| c.active)
            && s.values
                .as_ref()
                .is_some_and(|v| v.operations_committed >= 1);
        let p = s.predict(parent)?;
        if was_completion {
            if let Some(d) = s.value_decision().filter(|d| d.cursor == 0) {
                let target = MixedOperatorTarget {
                    action: d.action,
                    operand_ids: d.operands.map(|r| r.id),
                };
                frames.push(exact_frame(block, s, &target)?.0)?;
            } else if p.token == EOS {
                if reachable(s, block.max_operations) {
                    frames.push(frame(block, s, None)?.0)?;
                } else {
                    frames.unavailable_stops += 1;
                }
            }
        }
        s.observe(parent, p.token)?;
        if p.token == EOS {
            s.end_response(parent)?;
            return String::from_utf8(parent.decode(&out)?).map_err(|e| Error(e.to_string()));
        }
        out.push(p.token);
    }
    Err(Error(
        "shared operator preservation exceeded96 tokens".into(),
    ))
}
fn begin(parent: &Model, s: &mut Session, prompt: &str) -> Result<()> {
    for token in parent.encode(prompt)? {
        s.observe(parent, token)?;
    }
    s.begin_response(parent)
}
impl Model {
    /// Initial-role checkpoint. Earlier components are preserved by exact parent
    /// reconstruction; the returned artifact is not continuation qualification.
    pub fn fit_shared_operator_binding(
        &self,
        docs: &[MixedOperatorExample],
        preservation: &[OperationTransitionExample],
        prompts: &[Document],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        validate_inputs(docs, preservation)?;
        if self.shared_operator_refinement.is_some()
            || !self
                .action_emission
                .as_ref()
                .is_some_and(|w| w.context_enabled)
        {
            return Err(Error(
                "shared operator binding requires retained action-emission parent".into(),
            ));
        }
        let previous_roles = self
            .typed_roles
            .as_ref()
            .ok_or_else(|| Error("shared operator roles absent".into()))?
            .router
            .clone();
        let previous_operation = self
            .operation_transition
            .as_ref()
            .ok_or_else(|| Error("shared operator transition absent".into()))?
            .clone();
        let (roles, fit) = self.fit_mixed_initial(docs, preservation, prompts, &config)?;
        let mut model = self.clone();
        model
            .typed_roles
            .as_mut()
            .ok_or_else(|| Error("shared operator roles absent".into()))?
            .router = roles;
        model.shared_operator_refinement = Some(SharedOperatorRefinement {
            parent_artifact: self.artifact_cid.clone(),
            previous_roles,
            previous_operation,
            continuation_fitted: false,
        });
        model.refresh_identity()?;
        model.validate()?;
        Ok((
            model,
            serde_json::json!({"initial_roles":fit,"parent":self.artifact_cid(),"stage":"INITIAL_ROLE_CHECKPOINT","lexical_component_exact":true,"continuation_component_exact":true,"scope":"Existing shared derived-role router only; exact first operation/record labels and frozen-parent preference preservation. Continuation fit and free-generation acceptance remain separate."}),
        ))
    }
    /// Refine the existing continuation router. Numeric operations execute on
    /// actual records, and all suffix bytes are generated by the frozen emitter.
    pub fn refine_shared_operators(
        &self,
        docs: &[MixedOperatorExample],
        preservation: &[OperationTransitionExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        config.validate()?;
        self.validate()?;
        validate_inputs(docs, preservation)?;
        let witness = self.shared_operator_refinement.as_ref().ok_or_else(|| {
            Error("shared operator continuation requires initial-role checkpoint".into())
        })?;
        let parent = witness.parent(self)?;
        let old = self
            .operation_transition
            .as_ref()
            .ok_or_else(|| Error("shared operator transition absent".into()))?;
        if witness.continuation_fitted {
            return Err(Error(
                "shared operator continuation stage already fitted".into(),
            ));
        }
        let mut block = old.clone();
        extend_dictionary(&mut block, docs)?;
        let mut frames = Frames {
            values: vec![],
            rejection_count: 0,
            unavailable_stops: 0,
        };
        let mut receipts = Vec::new();
        let mut rollout = Vec::new();
        for d in docs {
            let mut s = prefix(self, &d.context)?;
            s.control = Control::OperationTransitionDisabled;
            let before = s.work.values.derived_writes;
            let first = until_stop(self, &mut s)?;
            if first != d.context.first_response {
                return Err(Error(format!(
                    "shared first generated response differs {}: {first:?}",
                    d.context.id
                )));
            }
            verify_write(&s, before, &d.first, &d.context.id)?;
            let mut next = Vec::new();
            for target in &d.targets {
                if !reachable(&s, block.max_operations) {
                    return Err(Error(format!(
                        "shared positive transition boundary unavailable {}",
                        d.context.id
                    )));
                }
                let (f, (action, pair)) = exact_frame(&block, &s, target)?;
                frames.push(f)?;
                let before = s.work.values.derived_writes;
                let values = s
                    .values
                    .as_mut()
                    .ok_or_else(|| Error("shared operation values absent".into()))?;
                let p = prepare(values, action, pair, 1, &mut s.work.values)
                    .ok_or_else(|| Error("shared labelled operation unavailable".into()))?;
                s.completion
                    .as_mut()
                    .ok_or_else(|| Error("shared completion absent".into()))?
                    .reset();
                s.observe(self, p.token)?;
                let mut text = self.decode(&[p.token])?;
                text.extend(until_stop(self, &mut s)?.as_bytes());
                verify_write(&s, before, target, &d.context.id)?;
                next.push(String::from_utf8(text).map_err(|e| Error(e.to_string()))?);
            }
            if reachable(&s, block.max_operations) {
                frames.push(frame(&block, &s, None)?.0)?;
            } else {
                frames.unavailable_stops += 1;
            }
            rollout.push(serde_json::json!({"id":d.context.id,"first":first,"supervised_next_generated_responses":next,"actual_records":s.values.as_ref().map(|v|&v.records)}));
            let bytes = serde_json::to_vec(d).map_err(|e| Error(e.to_string()))?;
            receipts.push(DocumentReceipt {
                id: d.context.id.clone(),
                bytes: bytes.len(),
                text_cid: format!("blake3:{}", blake3::hash(&bytes)),
            });
        }
        let mut preserved = Vec::new();
        let mut history_boundaries = 0;
        // Include new construction histories as preservation contexts, so their
        // shared continuation preferences survive the refinement as well.
        for (d, query) in docs
            .iter()
            .map(|d| (&d.context, false))
            .chain(preservation.iter().map(|d| (d, true)))
        {
            let mut s = parent.session(Control::Full)?;
            s.observe(&parent, BOS)?;
            for h in &d.history {
                begin(&parent, &mut s, &h.prompt)?;
                let text = collect_response(&parent, &block, &mut s, &mut frames)?;
                if text != h.response {
                    return Err(Error(format!(
                        "shared preservation history differs {}: {text:?}",
                        d.id
                    )));
                }
                history_boundaries += 1;
            }
            if query {
                begin(&parent, &mut s, &d.query)?;
                let text = collect_response(&parent, &block, &mut s, &mut frames)?;
                preserved.push(serde_json::json!({"id":d.id,"text":text}));
                let bytes = serde_json::to_vec(d).map_err(|e| Error(e.to_string()))?;
                receipts.push(DocumentReceipt {
                    id: d.id.clone(),
                    bytes: bytes.len(),
                    text_cid: format!("blake3:{}", blake3::hash(&bytes)),
                });
            }
        }
        let presented = frames.values.len();
        let mut unique = BTreeSet::new();
        frames.values.retain(|f| {
            unique.insert(
                f.alternatives
                    .iter()
                    .map(|a| (a.features.clone(), a.action, a.correct))
                    .collect::<Vec<_>>(),
            )
        });
        let conflicts = operation_frame_conflicts(&frames.values)?;
        if conflicts["count"].as_u64().unwrap_or(0) > 0 {
            return Err(Error(format!(
                "contradictory shared operation construction: {conflicts}"
            )));
        }
        let vocab: BTreeSet<_> = frames
            .values
            .iter()
            .flat_map(|f| {
                f.alternatives
                    .iter()
                    .flat_map(|a| a.features.iter().copied())
            })
            .collect();
        for feature in vocab {
            if !block.router.codes.iter().any(|c| c.feature == feature) {
                block.router.codes.push(SourceCode {
                    feature,
                    roots: [self.geometry.identity; 2],
                });
            }
        }
        block.router.codes.sort_by_key(|c| c.feature);
        if block.router.codes.len() > config.learned_features {
            return Err(Error(format!(
                "shared operation feature cap exceeded {}",
                block.router.codes.len()
            )));
        }
        block.router.config = config;
        block.router.training = receipts;
        let fit = learn(self, &mut block.router, &mut frames.values, Instant::now());
        let failures = operation_misclassified_frames(self, &block.router, &frames.values);
        let mut model = self.clone();
        model.operation_transition = Some(block);
        model
            .shared_operator_refinement
            .as_mut()
            .ok_or_else(|| Error("shared operator witness absent".into()))?
            .continuation_fitted = true;
        if model.lexical_emission != parent.lexical_emission
            || model.action_emission != parent.action_emission
        {
            return Err(Error(
                "shared operator changed frozen lexical component".into(),
            ));
        }
        model.refresh_identity()?;
        model.validate()?;
        Ok((
            model,
            serde_json::json!({"parent":parent.artifact_cid(),"initial_checkpoint":self.artifact_cid(),"fit":fit,"frames":frames.values.len(),"presented_frames":presented,"rejection_frames":frames.rejection_count,"unavailable_stop_boundaries":frames.unavailable_stops,"operation_frame_conflicts":conflicts,"misclassified_frames":failures,"rollout":rollout,"preservation":preserved,"preservation_history_boundaries":history_boundaries,"preservation_source":"exact reconstructed original parent","lexical_component_exact":true,"teacher_forced_lexical_labels":false,"teacher_forced_operator_and_record_labels":true,"scope":"Refine existing shared continuation with exact offline action/operand labels and original-parent generated preferences; every numeric operation executes actual records and all suffixes are freely generated by the frozen lexical component. Serving features, proposal admission and geometry unchanged; dictionary prime roots remapped by exact word identity."}),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::super::source_routing_training::Alternative;
    use super::*;

    #[test]
    fn native_shared_operator_frame_budget_reserves_rejection_atomically() {
        let positive = || Frame {
            alternatives: vec![
                Alternative {
                    features: vec![],
                    codes: vec![],
                    action: 0,
                    correct: false,
                },
                Alternative {
                    features: vec![],
                    codes: vec![],
                    action: 1,
                    correct: true,
                },
            ],
        };
        let mut frames = Frames {
            values: (0..4095)
                .map(|_| Frame {
                    alternatives: vec![],
                })
                .collect(),
            rejection_count: 0,
            unavailable_stops: 0,
        };
        assert!(frames.push(positive()).is_err());
        assert_eq!(frames.values.len(), 4095);
        assert_eq!(frames.rejection_count, 0);
        frames.values.pop();
        frames.push(positive()).unwrap();
        assert_eq!(frames.values.len(), 4096);
        assert_eq!(frames.rejection_count, 1);
        let rejection = frames.values.last().unwrap();
        assert_eq!(rejection.alternatives.len(), 1);
        assert_eq!(rejection.alternatives[0].action, 0);
        assert!(rejection.alternatives[0].correct);
    }
}
