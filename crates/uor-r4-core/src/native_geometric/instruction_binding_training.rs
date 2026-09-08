//! Offline continuation of existing literal selection and numeric admission.
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{learn, Alternative, Frame};
use super::value_types::{ValueRecord, ValueState, ValueWork};
use super::*;
use std::{collections::BTreeSet, time::Instant};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstructionExample {
    pub id: String,
    pub prompt: String,
    pub action: ValueAction,
    pub operands: [usize; 2],
}
fn setup(model: &Model, prompt: &str) -> Result<Session> {
    if prompt.len() > 4096 {
        return Err(Error("instruction prompt exceeds4096 bytes".into()));
    }
    let mut s = model.session(Control::Full)?;
    s.observe(model, BOS)?;
    for t in model.encode(prompt)? {
        s.observe(model, t)?;
    }
    s.begin_response(model)?;
    Ok(s)
}
fn candidates(v: &ValueState) -> Vec<(ValueAction, ValueRecord, ValueRecord)> {
    (0..272)
        .filter_map(|i| v.proposal(i))
        .filter(|(a, x, y)| super::joint_admission::legal(*a, x.value, y.value))
        .collect()
}
fn action(a: ValueAction) -> usize {
    if a == ValueAction::Copy {
        0
    } else {
        1
    }
}
fn correct(
    d: &InstructionExample,
    v: &ValueState,
    p: (ValueAction, ValueRecord, ValueRecord),
) -> Result<bool> {
    let a = v
        .sources
        .get(d.operands[0])
        .ok_or_else(|| Error("instruction source index absent".into()))?;
    let b = v
        .sources
        .get(d.operands[1])
        .ok_or_else(|| Error("instruction source index absent".into()))?;
    Ok(p.0 == d.action && p.1.id == a.id && p.2.id == b.id)
}
fn frame(
    model: &Model,
    prompt: &str,
    target: Option<&InstructionExample>,
) -> Result<Option<Frame>> {
    let s = setup(model, prompt)?;
    let v = s
        .values
        .as_ref()
        .ok_or_else(|| Error("instruction values absent".into()))?;
    let Some(c) = super::typed_routing::context(model, v, Control::Full, &mut ValueWork::default())
        .filter(|c| c.literal_component)
    else {
        return Ok(None);
    };
    let options = candidates(v);
    let mut best = super::typed_routing::score(
        model,
        v,
        None,
        2,
        &c,
        Control::Full,
        &mut ValueWork::default(),
    );
    let mut chosen = None;
    for (i, (a, x, y)) in options.iter().enumerate() {
        let score = super::typed_routing::score(
            model,
            v,
            Some((*x, *y)),
            action(*a),
            &c,
            Control::Full,
            &mut ValueWork::default(),
        );
        if score > best {
            best = score;
            chosen = Some(i);
        }
    }
    let mut alternatives = Vec::new();
    let (f, n) = super::typed_routing::features_with_provenance(
        v,
        None,
        &c.addresses,
        c.depths.as_ref(),
        c.provenance.as_ref(),
        &mut ValueWork::default(),
    );
    alternatives.push(Alternative {
        features: f[..n].to_vec(),
        codes: vec![],
        action: 2,
        correct: target.is_none() && chosen.is_none(),
    });
    for p in &options {
        let (f, n) = super::typed_routing::features_with_provenance(
            v,
            Some((p.1, p.2)),
            &c.addresses,
            c.depths.as_ref(),
            c.provenance.as_ref(),
            &mut ValueWork::default(),
        );
        let good = if let Some(d) = target {
            correct(d, v, *p)?
        } else {
            chosen.is_some_and(|j| {
                options[j].0 == p.0 && options[j].1.id == p.1.id && options[j].2.id == p.2.id
            })
        };
        alternatives.push(Alternative {
            features: f[..n].to_vec(),
            codes: vec![],
            action: action(p.0),
            correct: good,
        });
    }
    if !alternatives.iter().any(|a| a.correct) {
        return Err(Error("instruction target unavailable".into()));
    }
    Ok(Some(Frame { alternatives }))
}
fn fit_router(
    model: &Model,
    old: &SourceRouting,
    frames: &mut Vec<Frame>,
    new_frames: usize,
    receipts: &[DocumentReceipt],
    config: &SourceRoutingConfig,
) -> Result<(SourceRouting, serde_json::Value)> {
    let new: BTreeSet<_> = frames[..new_frames]
        .iter()
        .flat_map(|f| {
            f.alternatives
                .iter()
                .flat_map(|a| a.features.iter().copied())
        })
        .collect();
    let mut block = old.clone();
    for feature in new {
        if block.codes.iter().all(|c| c.feature != feature) {
            block.codes.push(SourceCode {
                feature,
                roots: [model.geometry.identity; 2],
            });
        }
    }
    block.codes.sort_by_key(|c| c.feature);
    if block.codes.len() > config.learned_features {
        return Err(Error(format!(
            "instruction feature cap: {} > {}",
            block.codes.len(),
            config.learned_features
        )));
    }
    block.config = config.clone();
    block.training = receipts.to_vec();
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
    let report = learn(model, &mut block, frames, Instant::now());
    let features_added = block.codes.len() - old.codes.len();
    Ok((
        block,
        serde_json::json!({"fit":report,"presented_frames":presented,"unique_frames":frames.len(),"feature_expansion_frames":new_frames,"features_added":features_added,"scope":"Exact ordered action/operand-ID targets for new prompts; original ordered router preference targets for preservation"}),
    ))
}
impl Model {
    pub fn fit_instruction_binding(
        &self,
        docs: &[InstructionExample],
        preserve: &[Document],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        config.validate()?;
        if self.instruction_binding.is_some()
            || docs.is_empty()
            || docs.len() > 128
            || preserve.len() > 768
            || docs.iter().any(|d| {
                !matches!(d.action, ValueAction::Copy | ValueAction::Add)
                    || d.operands.iter().any(|i| *i >= 16)
            })
        {
            return Err(Error("invalid instruction fit population".into()));
        }
        let old = self
            .typed_literals
            .as_ref()
            .ok_or_else(|| Error("instruction literals absent".into()))?;
        let admission = self
            .joint_admission
            .as_ref()
            .ok_or_else(|| Error("instruction admission absent".into()))?;
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        for d in docs {
            if !ids.insert(d.id.clone()) {
                return Err(Error("duplicate instruction id".into()));
            }
            receipts.push(super::training::receipt(&Document {
                id: d.id.clone(),
                text: serde_json::to_string(d).map_err(|e| Error(e.to_string()))?,
            }));
        }
        for d in preserve {
            if !ids.insert(d.id.clone()) {
                return Err(Error("duplicate preservation id".into()));
            }
            receipts.push(super::training::receipt(d));
        }
        let mut frames = Vec::new();
        for d in docs {
            frames.push(
                frame(self, &d.prompt, Some(d))?
                    .ok_or_else(|| Error("new instruction lacks literal context".into()))?,
            );
        }
        let count = frames.len();
        for d in preserve {
            if let Some(f) = frame(self, &d.text, None)? {
                frames.push(f);
            }
        }
        let (router, selection) =
            fit_router(self, &old.router, &mut frames, count, &receipts, &config)?;
        let mut model = self.clone();
        model
            .typed_literals
            .as_mut()
            .ok_or_else(|| Error("instruction literals absent".into()))?
            .router = router;
        model.instruction_binding = Some(super::instruction_binding::InstructionBinding {
            parent_artifact: self.artifact_cid.clone(),
            previous_literals: old.router.clone(),
            previous_admission: admission.router.clone(),
        });
        // Admission learns against the newly selected margins. Preservation uses
        // the original parent's admission outcome for the same exact proposal.
        let mut gates = Vec::new();
        for (prompt, target) in docs
            .iter()
            .map(|d| (d.prompt.as_str(), Some(d)))
            .chain(preserve.iter().map(|d| (d.text.as_str(), None)))
        {
            let s = setup(&model, prompt)?;
            let v = s
                .values
                .as_ref()
                .ok_or_else(|| Error("instruction values absent".into()))?;
            let Some(c) =
                super::typed_routing::context(&model, v, Control::Full, &mut ValueWork::default())
                    .filter(|c| c.literal_component)
            else {
                continue;
            };
            let no = super::typed_routing::score(
                &model,
                v,
                None,
                2,
                &c,
                Control::Full,
                &mut ValueWork::default(),
            );
            let old_no = super::typed_routing::score(
                self,
                v,
                None,
                2,
                &c,
                Control::Full,
                &mut ValueWork::default(),
            );
            for p in candidates(v) {
                let margin = super::typed_routing::score(
                    &model,
                    v,
                    Some((p.1, p.2)),
                    action(p.0),
                    &c,
                    Control::Full,
                    &mut ValueWork::default(),
                ) - no;
                if margin <= 0 {
                    continue;
                }
                let permit = if let Some(d) = target {
                    correct(d, v, p)?
                } else {
                    let old_margin = super::typed_routing::score(
                        self,
                        v,
                        Some((p.1, p.2)),
                        action(p.0),
                        &c,
                        Control::Full,
                        &mut ValueWork::default(),
                    ) - old_no;
                    old_margin > 0
                        && super::joint_admission::permits(
                            self,
                            v,
                            p.0,
                            (p.1, p.2),
                            old_margin,
                            &c,
                            Control::Full,
                            &mut ValueWork::default(),
                        )
                };
                let (f, n) = super::joint_admission::features(
                    v,
                    p.0,
                    (p.1, p.2),
                    margin,
                    &c,
                    &mut ValueWork::default(),
                );
                gates.push(Frame {
                    alternatives: (0..2)
                        .map(|a| Alternative {
                            features: f[..n].to_vec(),
                            codes: vec![],
                            action: a,
                            correct: a == usize::from(!permit),
                        })
                        .collect(),
                });
            }
        }
        if gates.len() > 4096 {
            return Err(Error("instruction admission frames exceed4096".into()));
        }
        let all = gates.len();
        let (gate, admission_fit) = fit_router(
            &model,
            &admission.router,
            &mut gates,
            all,
            &receipts,
            &config,
        )?;
        model
            .joint_admission
            .as_mut()
            .ok_or_else(|| Error("instruction admission absent".into()))?
            .router = gate;
        model.refresh_identity()?;
        model.validate()?;
        Ok((
            model,
            serde_json::json!({"selection":selection,"admission":admission_fit,"parent":self.artifact_cid(),"new_documents":docs.len(),"preservation_documents":preserve.len(),"serving_feature_law":"unchanged existing literal/provenance and admission geometry","new_targets":"Copy/Add with exact operand identities; no Sub/Mul aliasing"}),
        ))
    }
}
