//! Offline joint operator/operand learning over causally generated states.
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{learn, Alternative, Frame};
use super::typed_routing::{self, TypedRouting};
use super::value_types::{ValueAction, ValueFeature, ValueState};
use super::word_copy_types::WordCopyAddress;
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedRoutingTurn {
    pub prompt: String,
    pub response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedRoutingExample {
    pub id: String,
    /// Offline frame with no preceding response or supplied intermediate.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub literal_only: bool,
    pub initial_prompt: String,
    pub initial_response: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuation: Option<TypedRoutingTurn>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refresh: Vec<TypedRoutingTurn>,
    /// Which actually generated intermediate is the supervised role, 0 or 1.
    #[serde(default, skip_serializing_if = "typed_zero")]
    pub target_intermediate: usize,
    pub query: String,
    pub response: String,
    /// Offline action/operand supervision, never passed into serving.
    pub action: Option<ValueAction>,
    pub operands: Option<[i64; 2]>,
}

fn typed_zero(n: &usize) -> bool {
    *n == 0
}

impl TypedRouting {
    pub(super) fn validate(&self, model: &Model, roles: bool) -> Result<()> {
        self.router.validate_shape(
            model,
            3,
            if self.operand_provenance {
                6
            } else if roles {
                5
            } else {
                4
            },
        )?;
        let primes = crate::corpus_induced_spin_placement::first_primes(self.dictionary.len())
            .map_err(|e| Error(e.to_string()))?;
        if (self.literal_answers && !self.operand_provenance)
            || self.initialization_artifact.as_ref().is_some_and(|cid| {
                !self.operand_provenance
                    || cid.len() != 71
                    || !cid.starts_with("blake3:")
                    || !cid[7..].bytes().all(|b| b.is_ascii_hexdigit())
            })
            || (self.operand_provenance && (!roles || !self.local_query || !self.fold_ascii_case))
            || (roles && model.typed_routing.is_none())
            || model.values.is_none()
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

/// Offline continuation: preserve learned word identity when the sorted prime
/// dictionary grows. New features remain identity; no initialization lookup
/// occurs during serving.
pub(super) fn initialize_roles(block: &mut TypedRouting, old: &TypedRouting) {
    block.router.landmarks.clone_from(&old.router.landmarks);
    block.router.biases.clone_from(&old.router.biases);
    for code in &mut block.router.codes {
        let mut feature = code.feature;
        let prime = match feature.kind {
            2 => Some(&mut feature.a),
            3 => Some(&mut feature.b),
            _ => None,
        };
        if let Some(prime) = prime {
            let Some(word) = block
                .dictionary
                .iter()
                .find(|w| u64::from(w.prime) == *prime)
            else {
                continue;
            };
            let Some(previous) = old
                .dictionary
                .iter()
                .find(|w| w.bytes == word.bytes && w.len == word.len)
            else {
                continue;
            };
            *prime = u64::from(previous.prime);
        }
        if let Ok(i) = old
            .router
            .codes
            .binary_search_by_key(&feature, |c| c.feature)
        {
            code.roots = old.router.codes[i].roots;
        }
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

fn initial(
    model: &Model,
    d: &TypedRoutingExample,
    local_query: bool,
) -> Result<(Session, Option<ValueDecision>)> {
    let mut s = model.session(Control::Full)?;
    s.observe(model, BOS)?;
    if d.literal_only
        && (!d.initial_prompt.is_empty()
            || !d.initial_response.is_empty()
            || d.continuation.is_some()
            || !d.refresh.is_empty()
            || d.target_intermediate != 0)
    {
        return Err(Error(
            "literal frame cannot supply a preceding response".into(),
        ));
    }
    let mut first = None;
    if !d.literal_only {
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
        first = Some(
            decision
                .ok_or_else(|| Error("initial response has no committed typed decision".into()))?,
        );
        s.end_response(model)?;
        for (i, turn) in d.continuation.iter().chain(d.refresh.iter()).enumerate() {
            for t in model.encode(&turn.prompt)? {
                s.observe(model, t)?;
            }
            s.begin_response(model)?;
            let (bytes, decision, eos) = generate(model, &mut s)?;
            if bytes != turn.response.as_bytes() || !eos {
                return Err(Error(format!(
                    "generated intermediate failed: {} turn{}: {}",
                    d.id,
                    i,
                    String::from_utf8_lossy(&bytes)
                )));
            }
            if i == 0 && d.target_intermediate == 1 {
                first = Some(
                    decision.ok_or_else(|| Error("continuation has no typed decision".into()))?,
                );
            }
            s.end_response(model)?;
        }
    }
    if local_query {
        if let Some(v) = &mut s.values {
            v.query_boundary = Some(v.seen);
        }
    }
    for t in model.encode(&d.query)? {
        s.observe(model, t)?;
    }
    s.begin_response(model)?;
    Ok((s, first))
}

// Offline provenance checks follow exact Copy aliases only. Add remains a new
// computation even when its numeric output happens to equal an earlier value.
fn copy_origin(values: &ValueState, mut id: u64) -> u64 {
    for _ in 0..16 {
        let Some(r) = values.sources.iter().find(|r| r.id == id) else {
            break;
        };
        let Some(d) = r.derivation.filter(|d| d.action == ValueAction::Copy) else {
            break;
        };
        if d.operand_ids[0] >= id {
            break;
        }
        id = d.operand_ids[0];
    }
    id
}

impl Model {
    pub fn canonicalize_typed_role_aliases(&self) -> Result<Self> {
        self.validate()?;
        let mut model = self.clone();
        let block = model
            .typed_roles
            .as_mut()
            .ok_or_else(|| Error("typed role block absent".into()))?;
        if block.canonical_copy_aliases {
            return Err(Error("typed aliases already canonical".into()));
        }
        block.canonical_copy_aliases = true;
        model.refresh_identity()?;
        model.validate()?;
        Ok(model)
    }
    pub fn fit_typed_routing(
        &self,
        docs: &[TypedRoutingExample],
        config: SourceRoutingConfig,
        fold_ascii_case: bool,
    ) -> Result<(Self, serde_json::Value)> {
        self.fit_typed(
            docs,
            config,
            fold_ascii_case,
            false,
            false,
            false,
            false,
            None,
        )
    }
    pub fn fit_typed_roles(
        &self,
        docs: &[TypedRoutingExample],
        config: SourceRoutingConfig,
    ) -> Result<(Self, serde_json::Value)> {
        self.fit_typed(docs, config, true, true, false, false, false, None)
    }
    pub fn fit_typed_roles_local(
        &self,
        docs: &[TypedRoutingExample],
        config: SourceRoutingConfig,
    ) -> Result<(Self, serde_json::Value)> {
        self.fit_typed(docs, config, true, true, true, false, false, None)
    }
    pub fn fit_typed_roles_provenance(
        &self,
        docs: &[TypedRoutingExample],
        config: SourceRoutingConfig,
    ) -> Result<(Self, serde_json::Value)> {
        self.fit_typed_role_extension(docs, config, false)
    }
    pub fn fit_typed_literal_answers(
        &self,
        docs: &[TypedRoutingExample],
        config: SourceRoutingConfig,
    ) -> Result<(Self, serde_json::Value)> {
        self.fit_typed_role_extension(docs, config, true)
    }
    fn fit_typed_role_extension(
        &self,
        docs: &[TypedRoutingExample],
        config: SourceRoutingConfig,
        literal_answers: bool,
    ) -> Result<(Self, serde_json::Value)> {
        if let Some(block) = &self.typed_roles {
            self.validate()?;
            let mut parent = self.clone();
            parent.typed_roles = None;
            parent.refresh_identity()?;
            parent.fit_typed(
                docs,
                config,
                true,
                true,
                true,
                true,
                literal_answers,
                Some((block, self.artifact_cid())),
            )
        } else {
            self.fit_typed(docs, config, true, true, true, true, literal_answers, None)
        }
    }
    fn fit_typed(
        &self,
        docs: &[TypedRoutingExample],
        config: SourceRoutingConfig,
        fold_ascii_case: bool,
        roles: bool,
        local_query: bool,
        operand_provenance: bool,
        literal_answers: bool,
        initialization: Option<(&TypedRouting, &str)>,
    ) -> Result<(Self, serde_json::Value)> {
        config.validate()?;
        self.validate()?;
        if (if roles {
            self.typed_roles.is_some() || self.typed_routing.is_none()
        } else {
            self.typed_routing.is_some()
        }) || docs.is_empty()
            || docs.len() > 256
            || docs.iter().any(|d| {
                (d.literal_only && !literal_answers)
                    || d.id.is_empty()
                    || d.initial_prompt.len() + d.query.len() > 4096
                    || d.response.len() > 128
                    || d.initial_response.len() > 128
                    || d.action.is_some() != d.operands.is_some()
                    || d.target_intermediate > 1
                    || d.refresh.len() > 1
                    || (d.target_intermediate == 1 && d.continuation.is_none())
                    || d.continuation
                        .iter()
                        .chain(d.refresh.iter())
                        .any(|t| t.prompt.len() > 4096 || t.response.len() > 128)
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
            canonical_copy_aliases: local_query,
            local_query,
            operand_provenance,
            literal_answers,
            initialization_artifact: initialization.map(|(_, cid)| cid.to_owned()),
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
            let (s, first) = initial(self, d, local_query)?;
            let values = s
                .values
                .as_ref()
                .ok_or_else(|| Error("typed state absent".into()))?;
            if roles && !d.literal_only && values.sources.iter().filter(|r| r.derived).count() < 2 {
                return Err(Error(
                    "typed role frame has fewer than two derived sources".into(),
                ));
            }
            let depths =
                roles.then(|| typed_routing::lineage_depths(values, &mut Default::default()));
            let provenance = operand_provenance
                .then(|| typed_routing::operand_provenance(values, &mut Default::default()));
            let addr = typed_routing::addresses(&block, values, &mut Default::default());
            let (f, n) = typed_routing::features_with_provenance(
                values,
                None,
                &addr,
                depths.as_ref(),
                provenance.as_ref(),
                &mut Default::default(),
            );
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
                            && first.map_or(!a.derived && !b.derived, |first| {
                                a.id == first.write_id || b.id == first.write_id
                            })
                    });
                let (f, n) = typed_routing::features_with_provenance(
                    values,
                    Some((a, b)),
                    &addr,
                    depths.as_ref(),
                    provenance.as_ref(),
                    &mut Default::default(),
                );
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
            (a.0.kind >= 2 && a.0.kind != 4 && a.0.kind != 5)
                .cmp(&(b.0.kind >= 2 && b.0.kind != 4 && b.0.kind != 5))
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
        if let Some((old, _)) = initialization {
            initialize_roles(&mut block, old);
        }
        // Exact effective-feature collisions are inspected before the fit.
        let mut signatures =
            BTreeMap::<Vec<(usize, Vec<ValueFeature>)>, (usize, BTreeSet<usize>)>::new();
        for (frame_index, frame) in frames.iter().enumerate() {
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
                .or_insert_with(|| (frame_index, correct.clone()));
            joint.1 = joint.1.intersection(&correct).copied().collect();
            if joint.1.is_empty() {
                let full_equal = frames[joint.0]
                    .alternatives
                    .iter()
                    .zip(&frame.alternatives)
                    .all(|(a, b)| a.action == b.action && a.features == b.features);
                return Err(Error(format!("typed effective feature collision before fit: {} / {}; full_features_equal={full_equal}; retained={}/{}", docs[joint.0].id, docs[frame_index].id, vocab.len(), frequency.len())));
            }
        }
        let construction_ms = started.elapsed().as_millis();
        let fit = learn(self, &mut block.router, &mut frames, Instant::now());
        let block_bytes = serde_json::to_vec(&block)
            .map_err(|e| Error(e.to_string()))?
            .len();
        let dictionary_words = block.dictionary.len();
        let mut model = self.clone();
        if roles {
            model.typed_roles = Some(block);
        } else {
            model.typed_routing = Some(block);
        }
        model.refresh_identity()?;
        model.validate()?;
        let artifact = model.artifact_cid().to_owned();
        Ok((
            model,
            serde_json::json!({"parent":self.artifact_cid(),"artifact":artifact,"frames":frames.len(),"effective_query_classes":signatures.len(),"incompatible_feature_classes":0,"feature_universe":frequency.len(),"features":vocab.len(),"dictionary_words":dictionary_words,"block_bytes":block_bytes,"construction_ms":construction_ms,"fit":fit,"config":config,"scope":"Joint typed operator/operand labels over actual generated intermediates or explicit literal-only input frames; source values and numeric query words are not encoded as geometric scoring features. Construction fit is not generated transfer."}),
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
            let (mut s, first) = match initial(
                self,
                d,
                self.typed_roles.as_ref().is_some_and(|b| b.local_query),
            ) {
                Ok(state) => state,
                Err(error) => {
                    cases.push(serde_json::json!({"id":d.id,"exact":false,"initial_generation_failed":error.to_string(),"used_intermediate":false}));
                    continue;
                }
            };
            if remove_intermediate {
                if let (Some(v), Some(first)) = (&mut s.values, first) {
                    let target = copy_origin(v, first.write_id);
                    let removed: Vec<u64> = v
                        .sources
                        .iter()
                        .filter(|r| copy_origin(v, r.id) == target)
                        .map(|r| r.id)
                        .collect();
                    v.records.retain(|r| !removed.contains(&r.id));
                    v.sources.retain(|r| !removed.contains(&r.id));
                }
            }
            let captured = s.values.as_ref().map(|v| v.sources.clone());
            let (bytes, decision, eos) = generate(self, &mut s)?;
            let used = decision.is_some_and(|x| {
                s.values.as_ref().is_some_and(|v| {
                    x.operands.iter().any(|r| {
                        r.derived
                            && first.is_some_and(|first| {
                                copy_origin(v, r.id) == copy_origin(v, first.write_id)
                            })
                    })
                })
            });
            let action = decision.map(|d| d.action);
            let exact = bytes == d.response.as_bytes()
                && eos
                && action == d.action
                && (d.action.is_none()
                    || used
                    || (d.literal_only
                        && decision.is_some_and(|x| {
                            d.operands.is_some_and(|p| {
                                let pair = x.operands.map(|r| r.value);
                                pair == p || (x.action == ValueAction::Add && pair == [p[1], p[0]])
                            })
                        })));
            cases.push(serde_json::json!({"id":d.id,"query":d.query,"expected":d.response,"text":String::from_utf8_lossy(&bytes),"expected_action":d.action,"exact":exact,"used_intermediate":used,"terminated":eos,"decision":decision,"first_decision":first,"captured":captured,"work":s.work}));
        }
        Ok(
            serde_json::json!({"artifact":self.artifact_cid(),"total":cases.len(),"exact":cases.iter().filter(|r|r["exact"]==true).count(),"remove_intermediate_control":remove_intermediate,"elapsed_ms":started.elapsed().as_millis(),"cases":cases}),
        )
    }
}
