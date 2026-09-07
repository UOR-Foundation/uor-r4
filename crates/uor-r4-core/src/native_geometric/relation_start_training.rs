//! Offline response labels rank existing reverse starts. Neither target bytes
//! nor construction vocabulary are consulted by the serving selector.
use super::relation_span;
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{Alternative, Frame};
use super::value_types::ValueWork;
use super::*;
use std::collections::BTreeSet;
use std::time::Instant;

pub(super) fn validate(block: &SourceRouting, model: &Model) -> Result<()> {
    block.validate_shape(model, 1, 7)?;
    if block.config.learned_features > 64
        || block.config.passes > 4
        || block.config.proposals > 12
        || block.config.max_seconds > 3
        || block.config.mode != RoutingMode::Angular
        || !block.config.role_context_only
        || block.codes.iter().any(|code| {
            let f = code.feature;
            match f.kind {
                0..=2 => f.a > 6 || f.b != 0,
                3 => f.a > 3 || f.b != 0,
                4 | 5 => f.a > 6 || f.b > 6,
                6 => f.a > 1 || f.b != 0,
                _ => true,
            }
        })
    {
        return Err(Error(
            "invalid learned relation start configuration or feature".into(),
        ));
    }
    Ok(())
}

fn construction_frame(model: &Model, doc: &ValueExample) -> Result<(Frame, serde_json::Value)> {
    let expected = doc
        .response
        .strip_prefix(' ')
        .and_then(|text| text.strip_suffix(".\n"))
        .filter(|text| !text.is_empty() && text.len() <= 28 && text.is_ascii())
        .ok_or_else(|| Error(format!("invalid complete value response in {}", doc.id)))?;
    let mut session = model.session(Control::Full)?;
    session.observe(model, BOS)?;
    let mut next_id = 1;
    let mut result = None;
    for token in model.encode(&doc.prompt)? {
        session.observe(model, token)?;
        let values = session
            .values
            .as_ref()
            .ok_or_else(|| Error("relation start values absent".into()))?;
        let state = values
            .relations
            .as_ref()
            .ok_or_else(|| Error("relation start memory absent".into()))?;
        if state.next_id == next_id {
            continue;
        }
        if state.next_id != next_id + 1 || result.is_some() {
            return Err(Error(format!(
                "relation start requires one write per document: {}",
                doc.id
            )));
        }
        let record = state
            .record(next_id)
            .ok_or_else(|| Error("relation start write missing".into()))?;
        next_id = state.next_id;
        if record.owner.byte_end <= record.value.byte_end {
            return Err(Error(format!(
                "relation start requires reverse writer endpoint: {}",
                doc.id
            )));
        }
        let words = values
            .lexemes
            .as_ref()
            .ok_or_else(|| Error("relation start words absent".into()))?;
        // The completed owner is the actual reverse-write event boundary.
        // Refuse a tokenizer piece that hides later completed words in that event.
        if words.recent_len == 0 || words.recent[0] != record.owner {
            return Err(Error(format!(
                "reverse write boundary unavailable: {}",
                doc.id
            )));
        }
        let endpoint = words.recent[..words.recent_len]
            .iter()
            .position(|word| *word == record.value)
            .ok_or_else(|| Error("reverse endpoint not retained".into()))?;
        let candidates = relation_span::reverse_candidates(
            model,
            words,
            0,
            endpoint,
            record.action,
            &mut ValueWork::default(),
        );
        let mut alternatives = Vec::new();
        let mut trace = Vec::new();
        // Match runtime tie order exactly: longest admitted first, singleton last.
        for &candidate in candidates.rows[..candidates.len].iter().rev() {
            let features =
                relation_start::features(words, endpoint, candidate, &mut ValueWork::default());
            let payload = relation_span::payload(&record.value, candidate.span.as_ref())
                .ok_or_else(|| Error("invalid reverse candidate payload".into()))?;
            let correct = payload == expected.as_bytes();
            alternatives.push(Alternative {
                features: features.to_vec(),
                codes: Vec::new(),
                action: 0,
                correct,
            });
            trace.push(serde_json::json!({"payload":String::from_utf8_lossy(payload),"start":candidate.start,"features":features,"correct":correct}));
        }
        if !alternatives.iter().any(|a| a.correct) {
            return Err(Error(format!(
                "complete value is outside admitted reverse candidates: {}",
                doc.id
            )));
        }
        result = Some((
            Frame { alternatives },
            serde_json::json!({"id":doc.id,"endpoint_byte_end":record.value.byte_end,"candidates":trace}),
        ));
    }
    result.ok_or_else(|| Error(format!("no completed reverse write in {}", doc.id)))
}

impl Model {
    pub fn fit_relation_start(&self, docs: &[ValueExample]) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        if self.relation_reverse_spans.is_none()
            || self.relation_start.is_some()
            || docs.is_empty()
            || docs.len() > 64
        {
            return Err(Error("invalid relation start parent/data bounds".into()));
        }
        let start = Instant::now();
        let mut frames = Vec::new();
        let mut vocabulary = BTreeSet::new();
        let mut ids = BTreeSet::new();
        let mut receipts = Vec::new();
        let mut traces = Vec::new();
        for doc in docs {
            if doc.id.trim().is_empty()
                || !ids.insert(&doc.id)
                || doc.prompt.is_empty()
                || doc.prompt.len() > 2048
                || doc.response.len() > 32
            {
                return Err(Error("invalid relation start document".into()));
            }
            let (frame, trace) = construction_frame(self, doc)?;
            vocabulary.extend(
                frame
                    .alternatives
                    .iter()
                    .flat_map(|a| a.features.iter().copied()),
            );
            frames.push(frame);
            traces.push(trace);
            receipts.push(super::training::receipt(&Document {
                id: doc.id.clone(),
                text: serde_json::to_string(&(&doc.prompt, &doc.response))
                    .map_err(|e| Error(e.to_string()))?,
            }));
        }
        if vocabulary.is_empty() || vocabulary.len() > 64 {
            return Err(Error(
                "relation start feature vocabulary exceeds bound".into(),
            ));
        }
        let config = SourceRoutingConfig {
            learned_features: 64,
            passes: 4,
            proposals: 12,
            max_seconds: 3,
            role_context_only: true,
            ..Default::default()
        };
        let mut rng = config.seed;
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
            landmarks: vec![std::array::from_fn(|_| {
                (learned_routing_training::next_random(&mut rng) % 120) as u16
            })],
            biases: vec![0],
            ranks: learned_routing_training::ranks(self),
            training: receipts,
            config,
        };
        let preparation_ms = start.elapsed().as_millis();
        let learning_start = Instant::now();
        let mut report =
            source_routing_training::learn(self, &mut block, &mut frames, learning_start);
        let learning_ms = learning_start.elapsed().as_millis();
        let feature_count = block.codes.len();
        let mut model = self.clone();
        model.relation_start = Some(block);
        model.refresh_identity()?;
        model.validate()?;
        report["artifact"] = serde_json::json!(model.artifact_cid());
        report["parent"] = serde_json::json!(self.artifact_cid());
        report["frames"] = serde_json::json!(frames.len());
        report["documents"] = serde_json::json!(docs.len());
        report["feature_count"] = serde_json::json!(feature_count);
        report["preparation_ms"] = serde_json::json!(preparation_ms);
        report["learning_ms"] = serde_json::json!(learning_ms);
        report["elapsed_ms"] = serde_json::json!(start.elapsed().as_millis());
        report["construction"] = serde_json::json!(traces);
        report["scope"] = serde_json::json!("Single-action signed-H4 scoring of unchanged admitted reverse starts plus singleton endpoint, longest-first ties. Seven ordered features: ASCII first/inside-next/predecessor shapes, predecessor gap class, first-next and predecessor-first shape pairs, singleton flag. No exact vocabulary or response labels at inference, no additional admission, no general phrase parsing claim.");
        Ok((model, report))
    }
}
