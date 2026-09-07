//! Offline labels are response bytes; serving never reads targets.
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{Alternative, Frame};
use super::value_types::ValueFeature;
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;
impl Model {
    pub fn fit_source_span(&self, docs: &[ValueExample]) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        if self.source_context.is_none()
            || self.source_span.is_some()
            || docs.is_empty()
            || docs.len() > 128
        {
            return Err(Error("invalid source span parent/data bounds".into()));
        }
        let start = Instant::now();
        let mut frames = Vec::new();
        let mut vocabulary = BTreeSet::new();
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut skipped = Vec::new();
        for doc in docs {
            if !ids.insert(&doc.id)
                || doc.id.is_empty()
                || doc.prompt.len() + doc.response.len() > 65536
            {
                return Err(Error("invalid source span document".into()));
            }
            receipts.push(super::training::receipt(&Document {
                id: doc.id.clone(),
                text: serde_json::to_string(&(&doc.prompt, &doc.response))
                    .map_err(|e| Error(e.to_string()))?,
            }));
            let mut session = self.session(Control::Full)?;
            session.observe(self, BOS)?;
            for t in self.encode(&doc.prompt)? {
                session.observe(self, t)?;
            }
            session.begin_response(self)?;
            session.predict(self)?;
            let Some(decision) = session.word_copy.as_ref().and_then(|c| c.pending) else {
                skipped.push(doc.id.clone());
                continue;
            };
            if decision.word_index >= 16 || decision.dependency.is_some() {
                skipped.push(doc.id.clone());
                continue;
            }
            let values = session
                .values
                .as_ref()
                .ok_or_else(|| Error("source span values absent".into()))?;
            let mut current = decision.word_index;
            let first = super::relation::source(values, current)
                .ok_or_else(|| Error("source span initial word absent".into()))?;
            let prefix = if decision.action == word_copy_types::WordCopyAction::Prepare {
                self.decode(&[decision.token])?
            } else {
                Vec::new()
            };
            let Some(mut tail) = doc
                .response
                .as_bytes()
                .strip_prefix(prefix.as_slice())
                .and_then(|t| t.strip_prefix(&first.bytes[..usize::from(first.len)]))
            else {
                skipped.push(doc.id.clone());
                continue;
            };
            while let Some((next, separator)) =
                source_span::edge(values, current, &mut WordCopyWork::default())
            {
                let word = super::relation::source(values, next)
                    .ok_or_else(|| Error("source span next word absent".into()))?;
                let rest = tail
                    .strip_prefix(&[separator])
                    .and_then(|t| t.strip_prefix(&word.bytes[..usize::from(word.len)]))
                    .filter(|rest| {
                        rest.first()
                            .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_')
                    });
                let advance = rest.is_some();
                let feature = ValueFeature {
                    kind: 0,
                    a: u64::from(separator),
                    b: 0,
                };
                vocabulary.insert(feature);
                frames.push(Frame {
                    alternatives: (0..2)
                        .map(|action| Alternative {
                            features: vec![feature],
                            codes: Vec::new(),
                            action,
                            correct: (action == 1) == advance,
                        })
                        .collect(),
                });
                if let Some(rest) = rest {
                    tail = rest;
                    current = next;
                } else {
                    break;
                }
            }
        }
        if frames.is_empty() || vocabulary.len() > 16 {
            return Err(Error("source span has no bounded transition frames".into()));
        }
        let mut rng = 1139;
        let config = SourceRoutingConfig {
            learned_features: 16,
            passes: 4,
            proposals: 12,
            max_seconds: 5,
            role_context_only: true,
            ..Default::default()
        };
        let mut block = SourceRouting {
            schema: "uor-r4.geometric-source-routing/1".into(),
            parent_artifact: self.artifact_cid.clone(),
            codes: vocabulary
                .into_iter()
                .map(|feature| SourceCode {
                    feature,
                    roots: [self.geometry.identity; 2],
                })
                .collect(),
            landmarks: (0..2)
                .map(|_| {
                    std::array::from_fn(|_| {
                        (learned_routing_training::next_random(&mut rng) % 120) as u16
                    })
                })
                .collect(),
            biases: vec![0; 2],
            ranks: learned_routing_training::ranks(self),
            training: receipts,
            config,
        };
        let indexes: BTreeMap<_, _> = block
            .codes
            .iter()
            .enumerate()
            .map(|(i, c)| (c.feature, i))
            .collect();
        for frame in &mut frames {
            for a in &mut frame.alternatives {
                a.codes = a.features.iter().map(|f| indexes[f]).collect();
            }
        }
        let mut report = source_routing_training::learn(self, &mut block, &mut frames, start);
        let mut model = self.clone();
        model.source_span = Some(block);
        model.refresh_identity()?;
        model.validate()?;
        report["artifact"] = serde_json::json!(model.artifact_cid());
        report["parent"] = serde_json::json!(self.artifact_cid());
        report["documents"] = serde_json::json!(docs.len());
        report["frames"] = serde_json::json!(frames.len());
        report["skipped"] = serde_json::json!(skipped);
        report["elapsed_ms"] = serde_json::json!(start.elapsed().as_millis());
        report["scope"]=serde_json::json!("Learned separator-conditioned Continue/Finish in the existing ordered H4 code/landmark learner. Exact source extent gathered by existing causal byte cursor; query occurrences only, <=28 span bytes, no semantic phrase-boundary or general prose claim.");
        Ok((model, report))
    }
}
