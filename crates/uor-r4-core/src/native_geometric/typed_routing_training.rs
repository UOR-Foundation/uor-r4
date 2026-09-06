//! Offline joint operator/operand learning over causally generated states.
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{learn, Alternative, Frame};
use super::typed_routing::{self, TypedRouting};
use super::value_types::{ValueAction, ValueFeature};
use super::word_copy_types::WordCopyAddress;
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedRoutingExample {
    pub id: String,
    pub initial_prompt: String,
    pub initial_response: String,
    pub query: String,
    pub response: String,
    /// Offline action/operand supervision, never passed into serving.
    pub action: Option<ValueAction>,
    pub operands: Option<[i64; 2]>,
}

impl TypedRouting {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        self.router.validate_shape(model, 3, 4)?;
        let primes = crate::corpus_induced_spin_placement::first_primes(self.dictionary.len())
            .map_err(|e| Error(e.to_string()))?;
        if model.values.is_none()
            || self.dictionary.is_empty()
            || self.dictionary.len() > 256
            || self.dictionary.iter().zip(primes).any(|(w, p)| {
                w.len == 0
                    || w.len > 32
                    || u64::from(w.prime) != p
                    || !w.bytes[..usize::from(w.len)]
                        .iter()
                        .any(|b| b.is_ascii_alphabetic() || *b == b'_')
                    || w.bytes[..usize::from(w.len)]
                        .iter()
                        .any(|b| !b.is_ascii_alphanumeric() && *b != b'_')
                    || w.bytes[usize::from(w.len)..].iter().any(|b| *b != 0)
                    || (self.fold_ascii_case
                        && w.bytes[..usize::from(w.len)]
                            .iter()
                            .any(u8::is_ascii_uppercase))
            })
            || self
                .dictionary
                .windows(2)
                .any(|p| p[0].bytes[..usize::from(p[0].len)] >= p[1].bytes[..usize::from(p[1].len)])
        {
            return Err(Error("invalid geometric typed routing artifact".into()));
        }
        Ok(())
    }
}

fn generate(model: &Model, s: &mut Session) -> Result<(Vec<u8>, Option<ValueDecision>, bool)> {
    let mut out = Vec::new();
    let mut decision = None;
    for _ in 0..32 {
        let token = s.predict(model)?.token;
        if let Some(d) = s.value_decision().filter(|d| d.cursor == 0) {
            decision = Some(d);
        }
        s.observe(model, token)?;
        if token == EOS {
            return Ok((model.decode(&out)?, decision, true));
        }
        out.push(token);
    }
    Ok((model.decode(&out)?, decision, false))
}

fn initial(model: &Model, d: &TypedRoutingExample) -> Result<(Session, ValueDecision)> {
    let mut s = model.session(Control::Full)?;
    s.observe(model, BOS)?;
    for t in model.encode(&d.initial_prompt)? {
        s.observe(model, t)?;
    }
    s.begin_response(model)?;
    let (bytes, decision, eos) = generate(model, &mut s)?;
    if bytes != d.initial_response.as_bytes() || !eos {
        return Err(Error(format!(
            "initial generated response failed: {}: {}",
            d.id,
            String::from_utf8_lossy(&bytes)
        )));
    }
    let first =
        decision.ok_or_else(|| Error("initial response has no committed typed decision".into()))?;
    s.end_response(model)?;
    for t in model.encode(&d.query)? {
        s.observe(model, t)?;
    }
    s.begin_response(model)?;
    Ok((s, first))
}

impl Model {
    pub fn fit_typed_routing(
        &self,
        docs: &[TypedRoutingExample],
        config: SourceRoutingConfig,
        fold_ascii_case: bool,
    ) -> Result<(Self, serde_json::Value)> {
        config.validate()?;
        self.validate()?;
        if self.typed_routing.is_some()
            || docs.is_empty()
            || docs.len() > 256
            || docs.iter().any(|d| {
                d.id.is_empty()
                    || d.initial_prompt.len() + d.query.len() > 4096
                    || d.response.len() > 128
                    || d.initial_response.len() > 128
                    || d.action.is_some() != d.operands.is_some()
            })
        {
            return Err(Error("invalid typed routing construction input".into()));
        }
        let mut words = BTreeSet::<Vec<u8>>::new();
        for d in docs {
            for w in d
                .query
                .as_bytes()
                .split(|b| !b.is_ascii_alphanumeric() && *b != b'_')
            {
                if !w.is_empty()
                    && w.len() <= 32
                    && w.iter().any(|b| b.is_ascii_alphabetic() || *b == b'_')
                {
                    words.insert(if fold_ascii_case {
                        w.to_ascii_lowercase()
                    } else {
                        w.to_vec()
                    });
                }
            }
        }
        if words.len() > 256 {
            return Err(Error("typed dictionary exceeds256".into()));
        }
        let primes = crate::corpus_induced_spin_placement::first_primes(words.len())
            .map_err(|e| Error(e.to_string()))?;
        let dictionary = words
            .into_iter()
            .zip(primes)
            .map(|(w, p)| {
                let mut bytes = [0; 32];
                bytes[..w.len()].copy_from_slice(&w);
                WordCopyAddress {
                    bytes,
                    len: w.len() as u8,
                    prime: p as u32,
                }
            })
            .collect();
        let mut rng = config.seed;
        let mut block = TypedRouting {
            fold_ascii_case,
            dictionary,
            router: SourceRouting {
                schema: "uor-r4.geometric-source-routing/1".into(),
                parent_artifact: self.artifact_cid.clone(),
                codes: Vec::new(),
                landmarks: (0..3)
                    .map(|_| {
                        std::array::from_fn(|_| {
                            (super::learned_routing_training::next_random(&mut rng) % 120) as u16
                        })
                    })
                    .collect(),
                biases: vec![0; 3],
                ranks: super::learned_routing_training::ranks(self),
                training: Vec::new(),
                config: config.clone(),
            },
        };
        let started = Instant::now();
        let mut frequency = BTreeMap::<ValueFeature, usize>::new();
        let mut frames = Vec::new();
        for d in docs {
            let (s, first) = initial(self, d)?;
            let values = s
                .values
                .as_ref()
                .ok_or_else(|| Error("typed state absent".into()))?;
            let addr = typed_routing::addresses(&block, values, &mut Default::default());
            let (f, n) = typed_routing::features(values, None, &addr, &mut Default::default());
            let mut alternatives = vec![Alternative {
                features: f[..n].to_vec(),
                codes: Vec::new(),
                action: 2,
                correct: d.action.is_none(),
            }];
            for i in 0..272 {
                let Some((action, a, b)) = values.proposal(i) else {
                    continue;
                };
                let correct = d.action == Some(action)
                    && d.operands.is_some_and(|pair| {
                        ([a.value, b.value] == pair
                            || (action == ValueAction::Add && [b.value, a.value] == pair))
                            && (a.id == first.write_id || b.id == first.write_id)
                    });
                let (f, n) =
                    typed_routing::features(values, Some((a, b)), &addr, &mut Default::default());
                alternatives.push(Alternative {
                    features: f[..n].to_vec(),
                    codes: Vec::new(),
                    action: if action == ValueAction::Copy { 0 } else { 1 },
                    correct,
                });
            }
            if !alternatives.iter().any(|a| a.correct) {
                return Err(Error(format!(
                    "unreachable typed construction target: {}",
                    d.id
                )));
            }
            for a in &alternatives {
                for f in &a.features {
                    *frequency.entry(*f).or_default() += 1;
                }
            }
            frames.push(Frame { alternatives });
            let bytes = serde_json::to_vec(d).map_err(|e| Error(e.to_string()))?;
            block.router.training.push(DocumentReceipt {
                id: d.id.clone(),
                bytes: bytes.len(),
                text_cid: format!("blake3:{}", blake3::hash(&bytes).to_hex()),
            });
        }
        let mut vocab: Vec<_> = frequency.iter().map(|(f, n)| (*f, *n)).collect();
        vocab.sort_by(|a, b| {
            (a.0.kind >= 2)
                .cmp(&(b.0.kind >= 2))
                .then(b.1.cmp(&a.1))
                .then(a.0.cmp(&b.0))
        });
        vocab.truncate(config.learned_features);
        vocab.sort_by_key(|p| p.0);
        block.router.codes = vocab
            .iter()
            .map(|(feature, _)| SourceCode {
                feature: *feature,
                roots: [self.geometry.identity; 2],
            })
            .collect();
        // Exact effective-feature collisions are inspected before the fit.
        let mut signatures = BTreeMap::<Vec<(usize, Vec<ValueFeature>)>, BTreeSet<usize>>::new();
        for frame in &frames {
            let signature = frame
                .alternatives
                .iter()
                .map(|a| {
                    (
                        a.action,
                        a.features
                            .iter()
                            .copied()
                            .filter(|f| {
                                block
                                    .router
                                    .codes
                                    .binary_search_by_key(f, |c| c.feature)
                                    .is_ok()
                            })
                            .collect(),
                    )
                })
                .collect();
            let correct = frame
                .alternatives
                .iter()
                .enumerate()
                .filter_map(|(i, a)| a.correct.then_some(i))
                .collect::<BTreeSet<_>>();
            let joint = signatures
                .entry(signature)
                .or_insert_with(|| correct.clone());
            *joint = joint.intersection(&correct).copied().collect();
            if joint.is_empty() {
                return Err(Error("typed effective feature collision before fit".into()));
            }
        }
        let construction_ms = started.elapsed().as_millis();
        let fit = learn(self, &mut block.router, &mut frames, Instant::now());
        let block_bytes = serde_json::to_vec(&block)
            .map_err(|e| Error(e.to_string()))?
            .len();
        let dictionary_words = block.dictionary.len();
        let mut model = self.clone();
        model.typed_routing = Some(block);
        model.refresh_identity()?;
        model.validate()?;
        let artifact = model.artifact_cid().to_owned();
        Ok((
            model,
            serde_json::json!({"parent":self.artifact_cid(),"artifact":artifact,"frames":frames.len(),"effective_query_classes":signatures.len(),"incompatible_feature_classes":0,"feature_universe":frequency.len(),"features":vocab.len(),"dictionary_words":dictionary_words,"block_bytes":block_bytes,"construction_ms":construction_ms,"fit":fit,"config":config,"scope":"Joint typed operator/operand labels over actual generated intermediates; source values and numeric query words are not encoded as geometric scoring features. Construction fit is not generated transfer."}),
        ))
    }

    pub fn evaluate_typed_routing(
        &self,
        docs: &[TypedRoutingExample],
        remove_intermediate: bool,
    ) -> Result<serde_json::Value> {
        if docs.is_empty() || docs.len() > 256 {
            return Err(Error("typed evaluation document bound".into()));
        }
        let started = Instant::now();
        let mut cases = Vec::new();
        for d in docs {
            let (mut s, first) = initial(self, d)?;
            if remove_intermediate {
                if let Some(v) = &mut s.values {
                    v.records.retain(|r| r.id != first.write_id);
                    v.sources.retain(|r| r.id != first.write_id);
                }
            }
            let captured = s.values.as_ref().map(|v| v.sources.clone());
            let (bytes, decision, eos) = generate(self, &mut s)?;
            let used = decision.is_some_and(|x| {
                x.operands
                    .iter()
                    .any(|r| r.derived && r.id == first.write_id)
            });
            let action = decision.map(|d| d.action);
            let exact = bytes == d.response.as_bytes()
                && eos
                && action == d.action
                && (d.action.is_none() || used);
            cases.push(serde_json::json!({"id":d.id,"query":d.query,"expected":d.response,"text":String::from_utf8_lossy(&bytes),"expected_action":d.action,"exact":exact,"used_intermediate":used,"terminated":eos,"decision":decision,"first_decision":first,"captured":captured,"work":s.work}));
        }
        Ok(
            serde_json::json!({"artifact":self.artifact_cid(),"total":cases.len(),"exact":cases.iter().filter(|r|r["exact"]==true).count(),"remove_intermediate_control":remove_intermediate,"elapsed_ms":started.elapsed().as_millis(),"cases":cases}),
        )
    }
}
