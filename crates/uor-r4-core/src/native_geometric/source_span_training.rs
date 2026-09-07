//! Offline labels are response bytes; serving never reads targets.
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{Alternative, Frame};
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;
impl Model {
    pub fn fit_source_span(&self, docs: &[ValueExample]) -> Result<(Model, serde_json::Value)> {
        self.fit_source_span_mode(docs, false)
    }
    /// Learn extent from separator, next-word prime and its original source-cue pair.
    pub fn fit_contextual_source_span(
        &self,
        docs: &[ValueExample],
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_source_span_mode(docs, true)
    }
    fn fit_source_span_mode(
        &self,
        docs: &[ValueExample],
        contextual: bool,
    ) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        if self.source_context.is_none()
            || self.source_span.is_some()
            || docs.is_empty()
            || docs.len() > 128
        {
            return Err(Error("invalid source span parent/data bounds".into()));
        }
        let start = Instant::now();
        let mut feature_model = self.clone();
        if contextual {
            feature_model.source_span_context = Some(recurring_registry(docs)?);
        }
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
                let (features, count) = source_span::features(
                    &feature_model,
                    values,
                    decision.word_index,
                    next,
                    separator,
                    contextual,
                    contextual,
                    &mut WordCopyWork::default(),
                );
                vocabulary.extend(features[..count].iter().copied());
                frames.push(Frame {
                    alternatives: (0..2)
                        .map(|action| Alternative {
                            features: features[..count].to_vec(),
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
        if frames.is_empty() || vocabulary.len() > if contextual { 64 } else { 16 } {
            return Err(Error("source span has no bounded transition frames".into()));
        }
        let mut rng = 1139;
        let config = SourceRoutingConfig {
            learned_features: if contextual { 64 } else { 16 },
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
        let mut model = feature_model;
        model.source_span = Some(block);
        model.refresh_identity()?;
        model.validate()?;
        report["artifact"] = serde_json::json!(model.artifact_cid());
        report["parent"] = serde_json::json!(self.artifact_cid());
        report["context_registry"] = serde_json::json!(model.source_span_context);
        report["registry_policy"] = serde_json::json!("Prompt-only words recurring in at least two construction documents, existing scanner and 256-word cap, lexicographic first-prime assignment; separate namespace from fixed parent copy dictionary.");
        report["contextual"] = serde_json::json!(contextual);
        report["feature_count"] =
            serde_json::json!(model.source_span.as_ref().map(|b| b.codes.len()));
        report["documents"] = serde_json::json!(docs.len());
        report["frames"] = serde_json::json!(frames.len());
        report["skipped"] = serde_json::json!(skipped);
        report["elapsed_ms"] = serde_json::json!(start.elapsed().as_millis());
        report["scope"]=serde_json::json!("Ordered H4 Continue/Finish over separator, optional exact following-source prime and its pair with original selected value preceding source cue. Fixed first-source selection and exact causal copying; query occurrences only, <=28 span bytes. Context mode is identity-bound; no unrestricted semantic phrase-boundary or prose claim.");
        Ok((model, report))
    }
}

// Construction prompts alone establish exact identities; response labels do not
// influence membership. Unique words retain the separator-only fallback.
fn recurring_registry(docs: &[ValueExample]) -> Result<Vec<word_copy_types::WordCopyAddress>> {
    let (mut registry, _, _) = word_copy_training::dictionary(docs)?;
    let per_doc = docs
        .iter()
        .map(|doc| word_copy_training::dictionary(std::slice::from_ref(doc)).map(|x| x.0))
        .collect::<Result<Vec<_>>>()?;
    registry.retain(|word| {
        per_doc
            .iter()
            .filter(|words| {
                words
                    .iter()
                    .any(|w| w.len == word.len && w.bytes == word.bytes)
            })
            .take(2)
            .count()
            == 2
    });
    let primes = crate::corpus_induced_spin_placement::first_primes(registry.len())
        .map_err(|error| Error(error.to_string()))?;
    for (word, prime) in registry.iter_mut().zip(primes) {
        word.prime = prime as u32;
    }
    validate_registry(&registry)?;
    Ok(registry)
}

pub(super) fn validate_registry(registry: &[word_copy_types::WordCopyAddress]) -> Result<()> {
    if registry.is_empty() || registry.len() > word_copy_types::WORD_COPY_DICTIONARY {
        return Err(Error("invalid source span registry capacity".into()));
    }
    let primes = crate::corpus_induced_spin_placement::first_primes(registry.len())
        .map_err(|error| Error(error.to_string()))?;
    if registry.iter().zip(primes).any(|(word, prime)| {
        let len = usize::from(word.len);
        len == 0
            || len > 32
            || u64::from(word.prime) != prime
            || !(word.bytes[0].is_ascii_alphabetic() || word.bytes[0] == b'_')
            || !word.bytes[..len]
                .iter()
                .all(|b| b.is_ascii_alphanumeric() || *b == b'_')
            || word.bytes[len..].iter().any(|b| *b != 0)
    }) || registry.windows(2).any(|pair| {
        pair[0].bytes[..usize::from(pair[0].len)] >= pair[1].bytes[..usize::from(pair[1].len)]
    }) {
        return Err(Error("invalid source span exact prime registry".into()));
    }
    Ok(())
}
