//! Offline response labels rank existing reverse starts. Target bytes are never
//! consulted by the serving selector. Optional exact prime identities are
//! established from construction-prompt words shared across actual writer owners.
use super::relation_span;
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{Alternative, Frame};
use super::value_types::ValueWork;
use super::*;
use std::collections::BTreeSet;
use std::time::Instant;

pub(super) fn validate(block: &SourceRouting, model: &Model) -> Result<()> {
    let contextual = model.relation_start_context.is_some();
    if let Some(registry) = &model.relation_start_context {
        source_span_training::validate_registry(registry)?;
    }
    let prime = |address: u64| {
        address == 0
            || model
                .relation_start_context
                .as_ref()
                .is_some_and(|registry| {
                    registry.iter().any(|word| u64::from(word.prime) == address)
                })
    };
    block.validate_shape(model, 1, if contextual { 15 } else { 7 })?;
    if block.config.learned_features > if contextual { 256 } else { 64 }
        || block.config.passes > 4
        || block.config.proposals > 12
        || block.config.max_seconds > if contextual { 5 } else { 3 }
        || block.config.mode != RoutingMode::Angular
        || !block.config.role_context_only
        || block.codes.iter().any(|code| {
            let f = code.feature;
            match f.kind {
                0..=2 => f.a > 6 || f.b != 0,
                3 => f.a > 3 || f.b != 0,
                4 | 5 => f.a > 6 || f.b > 6,
                6 => f.a > 1 || f.b != 0,
                7..=9 if contextual => !prime(f.a) || f.b != 0,
                10 | 11 | 13 | 14 if contextual => !prime(f.a) || !prime(f.b),
                12 if contextual => !(1..=3).contains(&f.a) || !prime(f.b),
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

struct ConstructionWrite {
    words: value_lexemes::LexemeState,
    owner: value_lexemes::WordAtom,
    value: value_lexemes::WordAtom,
    endpoint: usize,
    action: u8,
}

/// Observe construction prompts once, without reading expected response bytes.
/// Group identity comes from the frozen parent's actual selected write owner.
fn construction_write(model: &Model, doc: &ValueExample) -> Result<ConstructionWrite> {
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
        result = Some(ConstructionWrite {
            words: *words,
            owner: record.owner,
            value: record.value,
            endpoint,
            action: record.action,
        });
    }
    result.ok_or_else(|| Error(format!("no completed reverse write in {}", doc.id)))
}

fn contextual_registry(
    docs: &[ValueExample],
    owners: &[value_lexemes::WordAtom],
) -> Result<Vec<word_copy_types::WordCopyAddress>> {
    if docs.len() != owners.len() || owners.iter().any(|owner| owner.len == 0 || owner.len > 32) {
        return Err(Error(
            "invalid relation start observed owner population".into(),
        ));
    }
    let (mut registry, _, _) = word_copy_training::dictionary(docs)?;
    let per_doc = docs
        .iter()
        .map(|doc| word_copy_training::dictionary(std::slice::from_ref(doc)).map(|x| x.0))
        .collect::<Result<Vec<_>>>()?;
    registry.retain(|word| {
        let mut contexts = BTreeSet::new();
        for (words, owner) in per_doc.iter().zip(owners) {
            if words
                .iter()
                .any(|w| w.len == word.len && w.bytes == word.bytes)
            {
                contexts.insert(&owner.bytes[..usize::from(owner.len)]);
                if contexts.len() >= 2 {
                    return true;
                }
            }
        }
        false
    });
    let primes = crate::corpus_induced_spin_placement::first_primes(registry.len())
        .map_err(|error| Error(error.to_string()))?;
    for (word, prime) in registry.iter_mut().zip(primes) {
        word.prime = prime as u32;
    }
    source_span_training::validate_registry(&registry)?;
    Ok(registry)
}

fn construction_frame(
    model: &Model,
    registry: Option<&[word_copy_types::WordCopyAddress]>,
    doc: &ValueExample,
    observed: &ConstructionWrite,
) -> Result<(Frame, serde_json::Value)> {
    let expected = doc
        .response
        .strip_prefix(' ')
        .and_then(|text| text.strip_suffix(".\n"))
        .filter(|text| !text.is_empty() && text.len() <= 28 && text.is_ascii())
        .ok_or_else(|| Error(format!("invalid complete value response in {}", doc.id)))?;
    let candidates = relation_span::reverse_candidates(
        model,
        &observed.words,
        0,
        observed.endpoint,
        observed.action,
        &mut ValueWork::default(),
    );
    let mut alternatives = Vec::new();
    let mut trace = Vec::new();
    // Match runtime tie order exactly: longest admitted first, singleton last.
    for &candidate in candidates.rows[..candidates.len].iter().rev() {
        let (features, count) = relation_start::contextual_features(
            registry,
            &observed.words,
            0,
            observed.endpoint,
            candidate,
            observed.action,
            &mut ValueWork::default(),
        );
        let features = &features[..count];
        let payload = relation_span::payload(&observed.value, candidate.span.as_ref())
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
    Ok((
        Frame { alternatives },
        serde_json::json!({
            "id":doc.id, "endpoint_byte_end":observed.value.byte_end,
            "writer_owner": String::from_utf8_lossy(&observed.owner.bytes[..usize::from(observed.owner.len)]),
            "writer_owner_bytes": &observed.owner.bytes[..usize::from(observed.owner.len)],
            "writer_owner_byte_end": observed.owner.byte_end,
            "candidates":trace,
        }),
    ))
}

impl Model {
    pub fn fit_relation_start(&self, docs: &[ValueExample]) -> Result<(Model, serde_json::Value)> {
        self.fit_relation_start_inner(docs, false)
    }

    /// Fit shape and ordered lexical/role context on unchanged admitted starts.
    /// Prompt words must occur under two distinct parent-selected writer owners;
    /// response labels and document IDs do not establish registry membership.
    pub fn fit_contextual_relation_start(
        &self,
        docs: &[ValueExample],
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_relation_start_inner(docs, true)
    }

    /// Matched no-refit control: replace only lexical code roots with the H4
    /// identity. Retain every feature key, registry, shape root, landmark, bias,
    /// geometry and fit configuration, so feature and code lookup work matches.
    /// The separately addressed artifact has the same complete frozen parent.
    pub fn without_relation_start_context(&self) -> Result<Model> {
        self.validate()?;
        if self.relation_start_context.is_none() {
            return Err(Error("relation start context is absent".into()));
        }
        let mut model = self.clone();
        let block = model
            .relation_start
            .as_mut()
            .ok_or_else(|| Error("relation start selector is absent".into()))?;
        for code in &mut block.codes {
            if code.feature.kind >= 7 {
                code.roots = [model.geometry.identity; 2];
            }
        }
        model.refresh_identity()?;
        model.validate()?;
        Ok(model)
    }

    fn fit_relation_start_inner(
        &self,
        docs: &[ValueExample],
        contextual: bool,
    ) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        if self.relation_reverse_spans.is_none()
            || self.relation_start.is_some()
            || self.relation_start_context.is_some()
            || docs.is_empty()
            || docs.len() > 64
        {
            return Err(Error("invalid relation start parent/data bounds".into()));
        }
        let start = Instant::now();
        // Validate bounded input before scanning it to construct lexical identity.
        let mut ids = BTreeSet::new();
        if docs.iter().any(|doc| {
            doc.id.trim().is_empty()
                || !ids.insert(&doc.id)
                || doc.prompt.is_empty()
                || doc.prompt.len() > 2048
                || doc.response.len() > 32
        }) {
            return Err(Error("invalid relation start document".into()));
        }
        let observed = docs
            .iter()
            .map(|doc| construction_write(self, doc))
            .collect::<Result<Vec<_>>>()?;
        let registry = if contextual {
            let owners: Vec<_> = observed.iter().map(|write| write.owner).collect();
            Some(contextual_registry(docs, &owners)?)
        } else {
            None
        };
        let mut frames = Vec::new();
        let mut vocabulary = BTreeSet::new();
        let mut receipts = Vec::new();
        let mut traces = Vec::new();
        for (doc, observed) in docs.iter().zip(&observed) {
            let (frame, trace) = construction_frame(self, registry.as_deref(), doc, observed)?;
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
        let feature_limit = if contextual { 256 } else { 64 };
        if vocabulary.is_empty() || vocabulary.len() > feature_limit {
            return Err(Error(
                "relation start feature vocabulary exceeds bound".into(),
            ));
        }
        let config = SourceRoutingConfig {
            learned_features: feature_limit,
            passes: 4,
            proposals: 12,
            max_seconds: if contextual { 5 } else { 3 },
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
        model.relation_start_context = registry;
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
        if contextual {
            report["context_registry"] = serde_json::json!(model.relation_start_context);
            report["registry_policy"] = serde_json::json!("Prompt-only words occurring under at least two distinct exact writer-owner byte identities actually selected by the frozen parent. Multiple template variants for one owner count once. Existing scanner and 256-word cap, lexicographic first-prime assignment in an operator-local namespace. Document IDs and response labels do not determine membership; payload identities retained for only one owner collapse to unknown context.");
            report["scope"] = serde_json::json!("Single-action signed-H4 scoring of unchanged reverse starts and singleton endpoint. Seven shape features plus ordered predecessor/first/inside-next exact prime addresses, predecessor-first and first-next pairs, writer action with predecessor, observed writer predecessor with candidate predecessor and first. Zero denotes unknown or absent exact identity. Learned selection does not widen admission, change the endpoint or author payload bytes. No-refit control replaces only lexical code roots with H4 identity while retaining all feature keys, registry, shape roots, landmark, bias and fit configuration for matched lookup work. No general phrase parsing or reasoning claim.");
        }
        Ok((model, report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contextual_relation_start_registry_counts_distinct_actual_owners() {
        let mut docs = [
            ValueExample {
                id: "arbitrary-a".into(),
                prompt: "notes say quiet field holds ada.".into(),
                response: " quiet field.\n".into(),
            },
            ValueExample {
                id: "unrelated-b".into(),
                prompt: "quiet field holds ada.".into(),
                response: " quiet field.\n".into(),
            },
            ValueExample {
                id: "arbitrary-a/variant".into(),
                prompt: "notes say quiet river holds eva.".into(),
                response: " quiet river.\n".into(),
            },
        ];
        let owner = |text: &str, byte_end| {
            let mut word = value_lexemes::WordAtom {
                len: text.len() as u8,
                byte_end,
                ..Default::default()
            };
            word.bytes[..text.len()].copy_from_slice(text.as_bytes());
            word
        };
        let mut owners = [owner("ada", 31), owner("ada", 21), owner("eva", 31)];
        let registry = contextual_registry(&docs, &owners).unwrap();
        let spellings: Vec<_> = registry
            .iter()
            .map(|word| &word.bytes[..usize::from(word.len)])
            .collect();
        assert_eq!(spellings, [b"holds".as_slice(), b"notes", b"quiet", b"say"]);
        // "field" recurs in documents, but only for the same actual owner.
        assert!(!spellings.contains(&b"field".as_slice()));
        docs[0].id = "new-id".into();
        docs[1].response = " answeronly.\n".into();
        docs[2].response = " completely changed.\n".into();
        assert_eq!(contextual_registry(&docs, &owners).unwrap(), registry);
        // Distinct source positions do not manufacture distinct owner groups.
        owners[2] = owner("ada", 91);
        assert!(contextual_registry(&docs, &owners).is_err());
        assert!(contextual_registry(&docs, &owners[..2]).is_err());
    }

    #[test]
    fn contextual_relation_start_registry_depends_on_recurring_prompt_words_only() {
        let mut docs = [
            ValueExample {
                id: "first".into(),
                prompt: "notes say quiet field holds ada unique unique.".into(),
                response: " quiet field.\n".into(),
            },
            ValueExample {
                id: "second".into(),
                prompt: "notes say quiet river holds eva.".into(),
                response: " quiet river.\n".into(),
            },
        ];
        let registry = source_span_training::recurring_registry(&docs).unwrap();
        let spellings: Vec<_> = registry
            .iter()
            .map(|word| &word.bytes[..usize::from(word.len)])
            .collect();
        assert_eq!(spellings, [b"holds".as_slice(), b"notes", b"quiet", b"say"]);
        assert_eq!(
            registry.iter().map(|word| word.prime).collect::<Vec<_>>(),
            [2, 3, 5, 7]
        );
        docs[0].response = " entirely different labels.\n".into();
        docs[1].response = " responseonly.\n".into();
        assert_eq!(
            source_span_training::recurring_registry(&docs).unwrap(),
            registry
        );
        let mut invalid = registry.clone();
        invalid[0].prime = 4;
        assert!(source_span_training::validate_registry(&invalid).is_err());
        let mut invalid = registry;
        invalid.swap(0, 1);
        assert!(source_span_training::validate_registry(&invalid).is_err());
    }
}
