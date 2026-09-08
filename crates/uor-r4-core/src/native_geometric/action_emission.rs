//! Offline action-conditioned refinement of shared lexical emission. Labels are
//! construction supervision; serving selects bytes and exact records geometrically.
use super::completion_types::LexicalRead;
use super::lexical_emission::{frame, operand, prepare, LexicalEmission};
use super::operation_transition::OperationTransition;
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{learn, learn_code_subset, Frame};
use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ActionEmission {
    pub parent_artifact: String,
    pub previous_lexical: LexicalEmission,
    pub previous_roles: SourceRouting,
    pub previous_operation: OperationTransition,
    pub context_enabled: bool,
}
impl ActionEmission {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        model
            .lexical_emission
            .as_ref()
            .ok_or_else(|| Error("action emitter absent".into()))?
            .validate_shape(model)?;
        model
            .typed_roles
            .as_ref()
            .ok_or_else(|| Error("action roles absent".into()))?
            .router
            .validate_shape(model, 3, 6)?;
        model
            .operation_transition
            .as_ref()
            .ok_or_else(|| Error("action operation absent".into()))?
            .validate_shape(model)?;
        let mut parent = model.clone();
        parent.action_emission = None;
        parent.lexical_emission = Some(self.previous_lexical.clone());
        parent
            .typed_roles
            .as_mut()
            .ok_or_else(|| Error("action role parent absent".into()))?
            .router = self.previous_roles.clone();
        parent.operation_transition = Some(self.previous_operation.clone());
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("action emission frozen parent differs".into()));
        }
        parent.validate()?;
        let mut identity = model.clone();
        identity.refresh_identity()?;
        if identity.artifact_cid != model.artifact_cid
            || identity.uor_model_address != model.uor_model_address
        {
            return Err(Error("action emission identity differs".into()));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionEmissionSegment {
    pub action: ValueAction,
    pub operand_ids: [u64; 2],
    pub prefix: String,
    pub suffix: Option<Vec<EmissionPiece>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionEmissionExample {
    pub id: String,
    pub history: Vec<TypedRoutingTurn>,
    pub query: String,
    pub segments: Vec<ActionEmissionSegment>,
}
struct Frames {
    operation_block: OperationTransition,
    values: Vec<Frame>,
    skipped_legacy: usize,
    operation: Vec<Frame>,
    rejection_frames: usize,
    unreachable_final_boundaries: usize,
}
fn push(frames: &mut Frames, value: Frame) -> Result<()> {
    if value.alternatives.iter().all(|a| {
        a.features
            .iter()
            .filter(|f| f.kind == 0)
            .all(|f| f.a >> 56 == 0)
    }) {
        frames.skipped_legacy += 1;
        return Ok(());
    }
    if frames.values.len() + frames.operation.len() >= 4096 {
        return Err(Error("action emission frame cap4096".into()));
    }
    frames.values.push(value);
    Ok(())
}
fn push_operation(frames: &mut Frames, frame: Frame) -> Result<()> {
    let rejection = super::mixed_operators::rejection_frame(&frame);
    let increment = 1 + usize::from(rejection.is_some());
    if frames.values.len() + frames.operation.len() + increment > 4096 {
        return Err(Error("combined action construction frame cap4096".into()));
    }
    frames.operation.push(frame);
    if let Some(rejection) = rejection {
        frames.operation.push(rejection);
        frames.rejection_frames += 1;
    }
    Ok(())
}
fn begin(model: &Model, s: &mut Session, prompt: &str) -> Result<()> {
    for t in model.encode(prompt)? {
        s.observe(model, t)?;
    }
    s.begin_response(model)
}
fn new_session(model: &Model) -> Result<Session> {
    let mut s = model.session(Control::Full)?;
    s.observe(model, BOS)?;
    Ok(s)
}
// Collect the actual parent's lexical preference, including Base. Query fields
// are identical; new frames differ only in the declared action feature law.
fn preservation_response(
    model: &Model,
    block: &LexicalEmission,
    s: &mut Session,
    frames: &mut Frames,
) -> Result<String> {
    let mut out = Vec::new();
    for _ in 0..96 {
        if let (Some(c), Some(v)) = (&s.completion, &s.values) {
            if c.active && !c.lexical_read.is_some_and(|r| r.cursor < r.numeral.len) {
                let mut copied = *c;
                let chosen = super::lexical_emission::offer(
                    model,
                    &mut copied,
                    v,
                    Candidate {
                        token: EOS,
                        score: 0,
                    },
                    Control::Full,
                    &mut Default::default(),
                );
                let target =
                    chosen.map(|p| (p.token, copied.pending_lexical_read.map(|r| r.record_id)));
                push(frames, frame(block, s, target, true)?)?;
            }
        }
        let was_boundary = s.completion.as_ref().is_some_and(|c| c.active)
            && s.values
                .as_ref()
                .is_some_and(|v| v.operations_committed >= 1);
        let p = s.predict(model)?;
        if was_boundary {
            let operation = &frames.operation_block;
            if let Some(decision) = s.value_decision().filter(|d| d.cursor == 0) {
                let target = MixedOperatorTarget {
                    action: decision.action,
                    operand_ids: decision.operands.map(|r| r.id),
                };
                let frame = super::mixed_operators::exact_frame(operation, s, &target)?.0;
                push_operation(frames, frame)?;
            } else if p.token == EOS
                && super::composed_output_training::reachable(s, operation.max_operations)
            {
                let frame = super::operation_transition::frame(operation, s, None)?.0;
                push_operation(frames, frame)?;
            }
        }
        s.observe(model, p.token)?;
        if p.token == EOS {
            s.end_response(model)?;
            return String::from_utf8(model.decode(&out)?).map_err(|e| Error(e.to_string()));
        }
        out.push(p.token);
    }
    Err(Error("action preservation response exceeded96".into()))
}
fn history(
    model: &Model,
    block: &LexicalEmission,
    s: &mut Session,
    turns: &[TypedRoutingTurn],
    frames: &mut Frames,
) -> Result<()> {
    for turn in turns {
        begin(model, s, &turn.prompt)?;
        let text = preservation_response(model, block, s, frames)?;
        if text != turn.response {
            return Err(Error(format!("action emission history differs: {text:?}")));
        }
    }
    Ok(())
}
fn check_prefix(
    model: &Model,
    s: &mut Session,
    target: &ActionEmissionSegment,
    first: Option<u32>,
    id: &str,
) -> Result<()> {
    let mut tokens = first.into_iter().collect::<Vec<_>>();
    for _ in 0..32 {
        if s.completion.as_ref().is_some_and(|c| c.active) {
            break;
        }
        let p = s.predict(model)?;
        if p.token == EOS {
            return Err(Error(format!("action numeric prefix stopped: {id}")));
        }
        s.observe(model, p.token)?;
        tokens.push(p.token);
    }
    let text = model.decode(&tokens)?;
    let c = s
        .completion
        .as_ref()
        .ok_or_else(|| Error("action completion absent".into()))?;
    let anchor = c
        .anchor
        .ok_or_else(|| Error("action anchor absent".into()))?;
    let derivation = s
        .values
        .as_ref()
        .and_then(|v| v.records.iter().find(|r| r.id == anchor.write_id))
        .and_then(|r| r.derivation)
        .ok_or_else(|| Error("action committed derivation absent".into()))?;
    if !c.active
        || text != target.prefix.as_bytes()
        || anchor.action != target.action
        || derivation.action != target.action
        || derivation.operand_ids != target.operand_ids
    {
        return Err(Error(format!(
            "action prefix/identity differs {id}: text={:?}, action={:?}, operands={:?}",
            String::from_utf8_lossy(&text),
            anchor.action,
            derivation.operand_ids
        )));
    }
    Ok(())
}
fn labels(
    model: &Model,
    block: &LexicalEmission,
    s: &mut Session,
    parts: &[EmissionPiece],
    frames: &mut Frames,
) -> Result<()> {
    let mut total = 1usize;
    for p in parts {
        total += match p {
            EmissionPiece::Text(t) => t.len(),
            EmissionPiece::Operand(i) => operand(s, *i)?.value.to_string().len(),
        };
    }
    if total > 32 {
        return Err(Error("action suffix exceeds completion cap".into()));
    }
    for p in parts {
        match p {
            EmissionPiece::Text(t) => {
                for token in t.bytes().map(|b| u32::from(b) + 2) {
                    push(frames, frame(block, s, Some((token, None)), true)?)?;
                    prepare(
                        s.completion
                            .as_mut()
                            .ok_or_else(|| Error("action completion absent".into()))?,
                        token,
                        None,
                        1,
                    );
                    s.observe(model, token)?;
                }
            }
            EmissionPiece::Operand(i) => {
                let r = operand(s, *i)?;
                let numeral = super::numeral::Numeral::from_zphi(
                    crate::prime_route_attention::ZPhi::new(r.value, 0),
                )
                .ok_or_else(|| Error("action numeral absent".into()))?;
                push(
                    frames,
                    frame(block, s, Some((numeral.tokens[0], Some(r.id))), true)?,
                )?;
                let start_at = s
                    .completion
                    .as_ref()
                    .ok_or_else(|| Error("action completion absent".into()))?
                    .seen;
                for cursor in 0..numeral.len {
                    let token = numeral.tokens[usize::from(cursor)];
                    prepare(
                        s.completion
                            .as_mut()
                            .ok_or_else(|| Error("action completion absent".into()))?,
                        token,
                        Some(LexicalRead {
                            record_id: r.id,
                            start_at,
                            numeral,
                            cursor,
                        }),
                        1,
                    );
                    s.observe(model, token)?;
                }
            }
        }
    }
    push(frames, frame(block, s, Some((EOS, None)), true)?)?;
    prepare(
        s.completion
            .as_mut()
            .ok_or_else(|| Error("action completion absent".into()))?,
        EOS,
        None,
        1,
    );
    Ok(())
}
fn plain(
    model: &Model,
    block: &LexicalEmission,
    s: &mut Session,
    frames: &mut Frames,
) -> Result<()> {
    // Plain suffix construction labels Base, allowing the existing completion.
    // Disable transitions only while finding this individual segment boundary.
    s.control = Control::OperationTransitionDisabled;
    for _ in 0..32 {
        if s.completion.as_ref().is_some_and(|c| c.active) {
            push(frames, frame(block, s, None, true)?)?;
        }
        let p = s.predict(model)?;
        if p.token == EOS {
            s.control = Control::Full;
            return Ok(());
        }
        s.observe(model, p.token)?;
    }
    Err(Error("action plain suffix exceeded32".into()))
}
fn next_segment(
    model: &Model,
    s: &mut Session,
    target: Option<&ActionEmissionSegment>,
    frames: &mut Frames,
) -> Result<Option<u32>> {
    let operation = &frames.operation_block;
    if !super::composed_output_training::reachable(s, operation.max_operations) {
        if target.is_some() {
            return Err(Error(
                "supervised action transition boundary unavailable".into(),
            ));
        }
        frames.unreachable_final_boundaries += 1;
        s.observe(model, EOS)?;
        s.end_response(model)?;
        return Ok(None);
    }
    if let Some(target) = target {
        let target = MixedOperatorTarget {
            action: target.action,
            operand_ids: target.operand_ids,
        };
        let (frame, (action, pair)) = super::mixed_operators::exact_frame(operation, s, &target)?;
        push_operation(frames, frame)?;
        // Offline operator/record labels select an admitted pair; the unchanged
        // typed executor computes its numeral from actual retained values.
        let values = s
            .values
            .as_mut()
            .ok_or_else(|| Error("action transition values absent".into()))?;
        let candidate =
            super::operation_transition::prepare(values, action, pair, 1, &mut s.work.values)
                .ok_or_else(|| Error("supervised action execution unavailable".into()))?;
        s.completion
            .as_mut()
            .ok_or_else(|| Error("action completion absent".into()))?
            .reset();
        s.observe(model, candidate.token)?;
        Ok(Some(candidate.token))
    } else {
        let frame = super::operation_transition::frame(operation, s, None)?.0;
        push_operation(frames, frame)?;
        s.observe(model, EOS)?;
        s.end_response(model)?;
        Ok(None)
    }
}

/// Detect contradictory labels on exactly the same ordered alternative law.
/// Hashes identify the complete feature/action sequence; differing positive
/// slots are retained with their actual feature sequences for inspection.
pub(super) fn operation_frame_conflicts(frames: &[Frame]) -> Result<serde_json::Value> {
    type Signature = Vec<(Vec<value_types::ValueFeature>, usize)>;
    let mut groups = BTreeMap::<Signature, BTreeMap<Vec<usize>, Vec<usize>>>::new();
    for (index, frame) in frames.iter().enumerate() {
        let signature = frame
            .alternatives
            .iter()
            .map(|a| (a.features.clone(), a.action))
            .collect();
        let positives = frame
            .alternatives
            .iter()
            .enumerate()
            .filter_map(|(slot, a)| a.correct.then_some(slot))
            .collect();
        groups
            .entry(signature)
            .or_default()
            .entry(positives)
            .or_default()
            .push(index);
    }
    let mut conflicts = Vec::new();
    for (signature, variants) in groups {
        if variants.len() <= 1 {
            continue;
        }
        let mut common: Option<BTreeSet<usize>> = None;
        for slots in variants.keys() {
            let slots: BTreeSet<_> = slots.iter().copied().collect();
            common = Some(match common {
                None => slots,
                Some(old) => old.intersection(&slots).copied().collect(),
            });
        }
        if common.is_some_and(|slots| !slots.is_empty()) {
            continue;
        }
        let bytes = serde_json::to_vec(&signature).map_err(|e| Error(e.to_string()))?;
        let variants: Vec<_> = variants.into_iter().map(|(slots, frame_indices)| {
            let positive_alternatives: Vec<_> = slots.iter().map(|&slot|serde_json::json!({"slot":slot,"action":signature[slot].1,"features":signature[slot].0})).collect();
            serde_json::json!({"frame_indices":frame_indices,"correct_slots":slots,"positive_alternatives":positive_alternatives})
        }).collect();
        conflicts.push(serde_json::json!({"ordered_alternatives_cid":format!("blake3:{}",blake3::hash(&bytes)),"alternative_count":signature.len(),"label_variants":variants}));
    }
    Ok(
        serde_json::json!({"count":conflicts.len(),"groups":conflicts,"scope":"Exact ordered feature/action alternatives with disjoint jointly acceptable slot labels. Absence of these contradictions does not establish geometric separability or fit convergence."}),
    )
}
pub(super) fn operation_misclassified_frames(
    model: &Model,
    router: &SourceRouting,
    frames: &[Frame],
) -> serde_json::Value {
    let mut failures = Vec::new();
    for (index, frame) in frames.iter().enumerate() {
        let mut scores = Vec::new();
        let mut best = None;
        for (slot, alternative) in frame.alternatives.iter().enumerate() {
            let mut work = RoutingWork::default();
            let state = router.encode(model, &alternative.features, Control::Full, &mut work);
            let score = router.score(model, state, alternative.action, &mut work);
            if best.is_none_or(|(_, best_score)| score > best_score) {
                best = Some((slot, score));
            }
            scores.push(score);
        }
        let Some((slot, score)) = best else {
            continue;
        };
        if frame.alternatives[slot].correct {
            continue;
        }
        let positive_alternatives: Vec<_> = frame.alternatives.iter().enumerate().filter(|(_,a)|a.correct).map(|(slot,a)|serde_json::json!({"slot":slot,"action":a.action,"score":scores[slot],"features":a.features})).collect();
        let chosen = &frame.alternatives[slot];
        failures.push(serde_json::json!({"frame_index":index,"alternative_count":frame.alternatives.len(),"best":{"slot":slot,"action":chosen.action,"score":score,"features":chosen.features},"positive_alternatives":positive_alternatives}));
    }
    serde_json::json!({"count":failures.len(),"frames":failures,"scope":"Post-fit exact serving encode/score over retained construction alternatives; strict greater-than selection preserves first-slot tie behavior. No free-generation claim."})
}
impl Model {
    pub fn refine_action_emission(
        &self,
        docs: &[ActionEmissionExample],
        preservation: &[OperationTransitionExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        config.validate()?;
        self.validate()?;
        let witness = self
            .action_emission
            .as_ref()
            .ok_or_else(|| Error("action emission requires staged witness".into()))?;
        if witness.context_enabled
            || docs.is_empty()
            || docs.len() > 128
            || preservation.len() > 512
            || docs.iter().any(|d| d.query.len() > 4096)
        {
            return Err(Error("invalid action emission population/stage".into()));
        }
        let old = self
            .lexical_emission
            .as_ref()
            .ok_or_else(|| Error("action lexical parent absent".into()))?;
        if config.mode != old.router.config.mode
            || config.role_context_only != old.router.config.role_context_only
        {
            return Err(Error(
                "action fit must preserve lexical scoring mode".into(),
            ));
        }
        let mut block = old.clone();
        let mut operation_block = self
            .operation_transition
            .as_ref()
            .ok_or_else(|| Error("action operation parent absent".into()))?
            .clone();
        let dictionary_inputs: Vec<_> = docs
            .iter()
            .map(|d| MixedOperatorExample {
                context: OperationTransitionExample {
                    id: d.id.clone(),
                    history: vec![],
                    query: d.query.clone(),
                    first_response: String::new(),
                    next: None,
                },
                first: MixedOperatorTarget {
                    action: ValueAction::Copy,
                    operand_ids: [0, 0],
                },
                targets: vec![],
            })
            .collect();
        // Only query strings are consumed here. Word identities receive sorted
        // primes; existing feature roots follow exact words through remapping.
        super::mixed_operators::extend_dictionary(&mut operation_block, &dictionary_inputs)?;
        let mut frames = Frames {
            operation_block,
            values: vec![],
            skipped_legacy: 0,
            operation: vec![],
            rejection_frames: 0,
            unreachable_final_boundaries: 0,
        };
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut preserved = Vec::new();
        for d in docs {
            if d.id.trim().is_empty() || !ids.insert(d.id.clone()) || d.query.len() > 4096 || d.history.len() > 4 || d.history.iter().any(|h| h.prompt.len() > 4096 || h.response.len() > 128) || d.segments.is_empty() || d.segments.len() > 3 || d.segments.iter().any(|p| !matches!(p.action, ValueAction::Add | ValueAction::Copy) || p.prefix.len() > 24 || p.suffix.as_ref().is_some_and(|ps| ps.len()>16 || ps.iter().any(|p| matches!(p, EmissionPiece::Text(t) if !t.is_ascii() || t.len()>32)))) { return Err(Error("action emission data bounds/duplicate id".into())); }
            let mut s = new_session(self)?;
            history(self, &block, &mut s, &d.history, &mut frames)?;
            begin(self, &mut s, &d.query)?;
            let mut first = None;
            for (i, segment) in d.segments.iter().enumerate() {
                check_prefix(self, &mut s, segment, first, &d.id)?;
                if let Some(parts) = &segment.suffix {
                    labels(self, &block, &mut s, parts, &mut frames)?;
                } else {
                    plain(self, &block, &mut s, &mut frames)?;
                }
                first = next_segment(self, &mut s, d.segments.get(i + 1), &mut frames)?;
            }
            let bytes = serde_json::to_vec(d).map_err(|e| Error(e.to_string()))?;
            receipts.push(DocumentReceipt {
                id: d.id.clone(),
                bytes: bytes.len(),
                text_cid: format!("blake3:{}", blake3::hash(&bytes)),
            });
        }
        for d in preservation {
            if d.id.trim().is_empty()
                || !ids.insert(d.id.clone())
                || d.query.len() > 4096
                || d.history.len() > 4
                || d.history
                    .iter()
                    .any(|h| h.prompt.len() > 4096 || h.response.len() > 128)
            {
                return Err(Error("action preservation bounds/duplicate id".into()));
            }
            let mut s = new_session(self)?;
            history(self, &block, &mut s, &d.history, &mut frames)?;
            begin(self, &mut s, &d.query)?;
            let text = preservation_response(self, &block, &mut s, &mut frames)?;
            preserved.push(serde_json::json!({"id":d.id,"text":text}));
            let bytes = serde_json::to_vec(d).map_err(|e| Error(e.to_string()))?;
            receipts.push(DocumentReceipt {
                id: d.id.clone(),
                bytes: bytes.len(),
                text_cid: format!("blake3:{}", blake3::hash(&bytes)),
            });
        }
        let rejection_frames = frames.rejection_frames;
        let unreachable_final_boundaries = frames.unreachable_final_boundaries;
        let mut operation_frames = frames.operation;
        let operation_presented = operation_frames.len();
        let mut unique_operations = BTreeSet::new();
        operation_frames.retain(|f| {
            unique_operations.insert(
                f.alternatives
                    .iter()
                    .map(|a| (a.features.clone(), a.action, a.correct))
                    .collect::<Vec<_>>(),
            )
        });
        let operation_conflicts = operation_frame_conflicts(&operation_frames)?;
        if operation_conflicts["count"].as_u64().unwrap_or(0) != 0 {
            return Err(Error(format!(
                "contradictory action operation construction: {operation_conflicts}"
            )));
        }
        let mut operation = frames.operation_block;
        let operation_old_features = operation.router.codes.len();
        let operation_vocab: BTreeSet<_> = operation_frames
            .iter()
            .flat_map(|f| {
                f.alternatives
                    .iter()
                    .flat_map(|a| a.features.iter().copied())
            })
            .collect();
        for feature in operation_vocab {
            if !operation.router.codes.iter().any(|c| c.feature == feature) {
                operation.router.codes.push(SourceCode {
                    feature,
                    roots: [self.geometry.identity; 2],
                });
            }
        }
        operation.router.codes.sort_by_key(|c| c.feature);
        if operation.router.codes.len() > config.learned_features {
            return Err(Error(format!(
                "action operation feature cap exceeded: {}",
                operation.router.codes.len()
            )));
        }
        operation.router.config = config.clone();
        operation.router.training = receipts.clone();
        let skipped_legacy_frames = frames.skipped_legacy;
        let mut frames = frames.values;
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
            .filter(|f| f.kind == 0 && f.a >> 56 == 1)
            .collect();
        for feature in vocab {
            if block.router.codes.iter().any(|c| c.feature == feature) {
                continue;
            }
            let mut legacy = feature;
            legacy.a &= (1_u64 << 56) - 1;
            let roots = old
                .router
                .codes
                .binary_search_by_key(&legacy, |c| c.feature)
                .ok()
                .map_or([self.geometry.identity; 2], |i| old.router.codes[i].roots);
            block.router.codes.push(SourceCode { feature, roots });
        }
        block.router.codes.sort_by_key(|c| c.feature);
        if block.router.codes.len() > config.learned_features {
            return Err(Error(format!(
                "action emission feature cap exceeded: {}",
                block.router.codes.len()
            )));
        }
        let mutable: Vec<_> = block
            .router
            .codes
            .iter()
            .map(|c| {
                old.router
                    .codes
                    .binary_search_by_key(&c.feature, |old| old.feature)
                    .is_err()
            })
            .collect();
        block.router.config = config;
        block.router.training = receipts;
        let operation_fit = learn(
            self,
            &mut operation.router,
            &mut operation_frames,
            Instant::now(),
        );
        let operation_failures =
            operation_misclassified_frames(self, &operation.router, &operation_frames);
        let operation_report = serde_json::json!({"fit":operation_fit,"operation_frame_conflicts":operation_conflicts,"misclassified_frames":operation_failures,"frames":operation_frames.len(),"presented_frames":operation_presented,"rejection_frames":rejection_frames,"unreachable_final_boundaries":unreachable_final_boundaries,"features":operation.router.codes.len(),"added_features":operation.router.codes.len()-operation_old_features,"dictionary_expanded_from_construction_queries":true,"existing_prime_feature_roots_remapped_by_exact_word_identity":true,"max_operations_unchanged":true,"dictionary":operation.dictionary.iter().map(|w|String::from_utf8_lossy(&w.bytes[..usize::from(w.len)]).into_owned()).collect::<Vec<_>>()});
        let fit = learn_code_subset(
            self,
            &mut block.router,
            &mut frames,
            Instant::now(),
            &mutable,
        )?;
        let frozen = block.dictionary == old.dictionary
            && block.tokens == old.tokens
            && block.router.landmarks == old.router.landmarks
            && block.router.biases == old.router.biases
            && block.router.ranks == old.router.ranks
            && old.router.codes.iter().all(|c| {
                block
                    .router
                    .codes
                    .binary_search_by_key(&c.feature, |v| v.feature)
                    .is_ok_and(|i| block.router.codes[i] == *c)
            });
        if !frozen {
            return Err(Error("action emission frozen parameters changed".into()));
        }
        let features = block.router.codes.len();
        let mut model = self.clone();
        model.lexical_emission = Some(block);
        model.operation_transition = Some(operation);
        model
            .action_emission
            .as_mut()
            .ok_or_else(|| Error("action witness absent".into()))?
            .context_enabled = true;
        model.refresh_identity()?;
        model.validate()?;
        Ok((
            model,
            serde_json::json!({"fit":fit,"frames":frames.len(),"presented_frames":presented,"skipped_unchanged_legacy_frames":skipped_legacy_frames,"features":features,"added_features":mutable.iter().filter(|m|**m).count(),"inherited_lexical_parameters_exact":frozen,"teacher_forced_lexical_labels":true,"explicit_offline_record_alignment":true,"transitions_model_selected_at_supervised_lexical_boundary":false,"teacher_forced_operator_and_record_labels":true,"operation":operation_report,"preservation":preserved,"scope":"Only added Copy action-context H4 roots fitted; legacy lexical codes, query codes, dictionary, tokens, landmarks and biases frozen. Construction suffixes and subsequent operator/record selections are teacher forced; the typed executor computes values from actual records and numeric prefixes plus exact committed derivations are checked. The shared operation router is refined on these labels plus freely generated parent decisions and generic rejection companions. Free generation is separate acceptance."}),
        ))
    }
}
