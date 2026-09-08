//! Offline continuation of literal operand selection; serving reuses typed_routing.
use super::source_routing::{SourceCode, SourceRoutingRefinement};
use super::source_routing_training::{learn, Alternative, Frame};
use super::typed_routing;
use super::value_types::ValueWork;
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

impl Model {
    /// Preserve the dictionary and feature law; admit construction-observed codes
    /// and continue the existing literal router beneath a frozen parent witness.
    pub fn refine_literal_routing(
        &self,
        docs: &[ValueExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        config.validate()?;
        let old = self
            .typed_literals
            .as_ref()
            .ok_or_else(|| Error("literal router absent".into()))?;
        if self.literal_routing_refinement.is_some()
            || self.joint_admission.is_none()
            || config.role_context_only
            || config.learned_features < old.router.codes.len()
            || docs.is_empty()
            || docs.len() > 1024
        {
            return Err(Error(
                "invalid literal refinement parent/configuration".into(),
            ));
        }
        let start = Instant::now();
        let mut frames = Vec::new();
        let mut frequency = BTreeMap::<_, [usize; 2]>::new();
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut skipped = Vec::new();
        let mut unreachable = Vec::new();
        for doc in docs {
            if doc.id.trim().is_empty()
                || !ids.insert(doc.id.clone())
                || doc.prompt.len() + doc.response.len() > 65536
            {
                return Err(Error("invalid literal refinement document".into()));
            }
            receipts.push(super::training::receipt(&Document {
                id: doc.id.clone(),
                text: serde_json::to_string(&(&doc.prompt, &doc.response))
                    .map_err(|e| Error(e.to_string()))?,
            }));
            let response = doc.response.trim_start();
            let end = response
                .bytes()
                .take_while(|b| b.is_ascii_digit() || *b == b'-')
                .count();
            let target = if end == 0 {
                None
            } else {
                Some(
                    response[..end]
                        .parse::<i64>()
                        .map_err(|_| Error("numeric response prefix invalid".into()))?,
                )
            };
            let mut session = self.session(Control::Full)?;
            session.observe(self, BOS)?;
            for token in self.encode(&doc.prompt)? {
                session.observe(self, token)?;
            }
            session.begin_response(self)?;
            let values = session
                .values
                .as_ref()
                .ok_or_else(|| Error("literal values absent".into()))?;
            let Some(context) =
                typed_routing::context(self, values, Control::Full, &mut ValueWork::default())
                    .filter(|c| c.literal_component)
            else {
                skipped.push(doc.id.clone());
                continue;
            };
            let mut alternatives = Vec::new();
            let mut add = |operands, action, correct| {
                let (f, n) = typed_routing::features_with_provenance(
                    values,
                    operands,
                    &context.addresses,
                    context.depths.as_ref(),
                    context.provenance.as_ref(),
                    &mut ValueWork::default(),
                );
                alternatives.push(Alternative {
                    features: f[..n].to_vec(),
                    codes: Vec::new(),
                    action,
                    correct,
                });
            };
            add(None, 2, target.is_none());
            for i in 0..528 {
                let Some((action, a, b)) = values.proposal(i) else {
                    continue;
                };
                let result = match action {
                    ValueAction::Copy => Some(a.value),
                    ValueAction::Add => a.value.checked_add(b.value),
                    ValueAction::Sub => a.value.checked_sub(b.value),
                };
                if result.is_none() {
                    continue;
                }
                add(
                    Some((a, b)),
                    match action {
                        ValueAction::Copy => 0,
                        ValueAction::Add | ValueAction::Sub => 1,
                    },
                    target.is_some() && result == target,
                );
            }
            if !alternatives.iter().any(|a| a.correct) {
                unreachable.push(doc.id.clone());
                continue;
            }
            for a in &alternatives {
                for f in &a.features {
                    frequency.entry(*f).or_default()[usize::from(a.correct)] += 1;
                }
            }
            frames.push(Frame { alternatives });
        }
        if frames.is_empty() {
            return Err(Error("no literal refinement frames".into()));
        }
        // Within-frame contrast discounts features equally common in correct
        // and incorrect alternatives, as in the retained source refinement.
        let mut contrast = BTreeMap::<_, usize>::new();
        for frame in &frames {
            let positives = frame.alternatives.iter().filter(|a| a.correct).count();
            let negatives = frame.alternatives.len() - positives;
            let mut counts = BTreeMap::<_, [usize; 2]>::new();
            for a in &frame.alternatives {
                for f in a.features.iter().copied().collect::<BTreeSet<_>>() {
                    counts.entry(f).or_default()[usize::from(a.correct)] += 1;
                }
            }
            for (f, c) in counts {
                if c[1] * negatives != c[0] * positives {
                    *contrast.entry(f).or_default() += 1;
                }
            }
        }
        let mut additions: Vec<_> = frequency
            .into_iter()
            .filter(|(f, _)| {
                old.router
                    .codes
                    .binary_search_by_key(f, |c| c.feature)
                    .is_err()
            })
            .collect();
        additions.sort_by(|a, b| {
            contrast
                .get(&b.0)
                .unwrap_or(&0)
                .cmp(contrast.get(&a.0).unwrap_or(&0))
                .then_with(|| (b.1[0] + b.1[1]).cmp(&(a.1[0] + a.1[1])))
                .then_with(|| a.0.cmp(&b.0))
        });
        let unseen = additions.len();
        additions.truncate(config.learned_features - old.router.codes.len());
        let mut block = old.router.clone();
        block
            .codes
            .extend(additions.into_iter().map(|(feature, _)| SourceCode {
                feature,
                roots: [self.geometry.identity; 2],
            }));
        block.codes.sort_by_key(|c| c.feature);
        block.training = receipts;
        block.config = config;
        let binding_feature = value_types::ValueFeature {
            kind: 5,
            a: 2,
            b: 66,
        };
        let binding_before = block
            .codes
            .iter()
            .find(|c| c.feature == binding_feature)
            .cloned();
        let mut report = learn(self, &mut block, &mut frames, start);
        report["schema"] = serde_json::json!("uor-r4.literal-refinement-fit/1");
        report["documents"] = serde_json::json!(docs.len());
        report["frames"] = serde_json::json!(frames.len());
        report["skipped"] = serde_json::json!(skipped);
        report["unreachable"] = serde_json::json!(unreachable);
        report["old_features"] = serde_json::json!(old.router.codes.len());
        report["unseen_features"] = serde_json::json!(unseen);
        report["binding_before"] = serde_json::json!(binding_before);
        report["binding_after"] =
            serde_json::json!(block.codes.iter().find(|c| c.feature == binding_feature));
        let mut model = self.clone();
        model
            .typed_literals
            .as_mut()
            .ok_or_else(|| Error("literal router absent".into()))?
            .router = block;
        model.literal_routing_refinement = Some(SourceRoutingRefinement {
            parent_artifact: self.artifact_cid.clone(),
            previous: old.router.clone(),
        });
        model.refresh_identity()?;
        model.validate()?;
        report["parent"] = serde_json::json!(self.artifact_cid());
        report["artifact"] = serde_json::json!(model.artifact_cid());
        report["elapsed_ms"] = serde_json::json!(start.elapsed().as_millis());
        report["scope"] = serde_json::json!("Fixed dictionary/feature law and exact parent witness; supervised numeric result prefix or NoOperation. Equal numeric values do not establish occurrence identity. No derived-role fitting.");
        Ok((model, report))
    }
}
