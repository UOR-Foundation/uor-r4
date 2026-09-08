//! Offline discrete learning of retained-source routes. The accepted reader is
//! frozen; its construction-fitted feature salience selects the code vocabulary.
//! Targets are supplied response bytes, never runtime comparator decisions.
use super::role_read::{self, NO_SOURCE};
use super::source_routing::{SourceCode, SourceRouting, LANES};
use super::value_types::ValueFeature;
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRoutingConfig {
    pub learned_features: usize,
    pub passes: usize,
    pub proposals: usize,
    pub max_seconds: u64,
    pub mode: RoutingMode,
    pub seed: u64,
    /// Restrict source binding to role context; inherited H4/zeta state remains
    /// in the model, but does not alter a name's role-based source decision.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub role_context_only: bool,
}
impl Default for SourceRoutingConfig {
    fn default() -> Self {
        Self {
            learned_features: 64,
            passes: 2,
            proposals: 8,
            max_seconds: 30,
            mode: RoutingMode::Angular,
            seed: 1139,
            role_context_only: false,
        }
    }
}
impl SourceRoutingConfig {
    pub(super) fn validate(&self) -> Result<()> {
        // Literal and derived-value routing share a bounded feature vocabulary.
        if !(1..=768).contains(&self.learned_features)
            || !(1..=8).contains(&self.passes)
            || !(1..=120).contains(&self.proposals)
            || !(1..=120).contains(&self.max_seconds)
        {
            return Err(Error("invalid source routing configuration".into()));
        }
        Ok(())
    }
}
impl SourceRouting {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        self.config.validate()?;
        let read = role_read::head(model)
            .ok_or_else(|| Error("source routing requires retained-word reader".into()))?;
        self.validate_shape(
            model,
            read.actions.len(),
            if self.config.role_context_only {
                20
            } else {
                30
            },
        )
    }
    pub(super) fn validate_shape(
        &self,
        model: &Model,
        actions: usize,
        feature_kinds: u8,
    ) -> Result<()> {
        self.config.validate()?;
        if self.schema != "uor-r4.geometric-source-routing/1"
            || self.codes.is_empty()
            || self.codes.len() > self.config.learned_features
            || self.codes.windows(2).any(|p| p[0].feature >= p[1].feature)
            || self
                .codes
                .iter()
                .any(|c| c.feature.kind >= feature_kinds || c.roots.iter().any(|&r| r >= 120))
            || self.landmarks.len() != actions
            || self.landmarks.iter().flatten().any(|&r| r >= 120)
            || self.biases.len() != actions
            || self.biases.iter().any(|b| !(-32..=32).contains(b))
            || self.ranks != super::learned_routing_training::ranks(model)
            || self.training.is_empty()
            || self.training.len() > 1024
            || self.training.iter().any(|r| {
                r.id.trim().is_empty() || r.bytes == 0 || !r.text_cid.starts_with("blake3:")
            })
            || self
                .training
                .iter()
                .map(|r| &r.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.training.len()
        {
            return Err(Error("invalid source routing artifact".into()));
        }
        Ok(())
    }
}
pub(super) struct Alternative {
    pub features: Vec<ValueFeature>,
    pub codes: Vec<usize>,
    pub action: usize,
    pub correct: bool,
}
pub(super) struct Frame {
    pub alternatives: Vec<Alternative>,
}
#[derive(Clone, Copy, Debug)]
struct Objective {
    correct: usize,
    hinge: i64,
}
impl Objective {
    fn improves(self, other: Self) -> bool {
        self.correct > other.correct || (self.correct == other.correct && self.hinge < other.hinge)
    }
}

// A feature-sequence alias is descriptive: it need not make the entire frame
// impossible if another positive alternative remains distinguishable.
fn feature_alias_frames(frames: &[Frame], codes: &[SourceCode]) -> usize {
    frames
        .iter()
        .filter(|frame| {
            let mut seen = BTreeMap::new();
            frame.alternatives.iter().any(|alternative| {
                let sequence: Vec<_> = alternative
                    .features
                    .iter()
                    .filter_map(|feature| {
                        codes
                            .binary_search_by_key(feature, |code| code.feature)
                            .ok()
                    })
                    .collect();
                seen.insert((alternative.action, sequence), alternative.correct)
                    .is_some_and(|prior| prior != alternative.correct)
            })
        })
        .count()
}

fn objective(model: &Model, block: &SourceRouting, frames: &[Frame]) -> Objective {
    let mut result = Objective {
        correct: 0,
        hinge: 0,
    };
    for frame in frames {
        let mut best = (i64::MIN, false);
        let mut positive = i64::MIN;
        let mut negative = i64::MIN;
        for a in &frame.alternatives {
            let mut state = [model.geometry.identity; LANES];
            for &index in &a.codes {
                for (root, &code) in state.iter_mut().zip(&block.codes[index].roots) {
                    *root = model.geometry.products
                        [model.geometry.row_bases[usize::from(*root)] + usize::from(code)];
                }
            }
            let mut score = i64::from(block.biases[a.action]);
            for (root, &landmark) in state.into_iter().zip(&block.landmarks[a.action]) {
                let inverse = model.geometry.inverses[usize::from(landmark)];
                let relative = model.geometry.products
                    [model.geometry.row_bases[usize::from(root)] + usize::from(inverse)];
                score += match block.config.mode {
                    RoutingMode::Angular => i64::from(block.ranks[usize::from(relative)]),
                    RoutingMode::Equality => {
                        if relative == model.geometry.identity {
                            i64::from(block.ranks[usize::from(model.geometry.identity)])
                        } else {
                            0
                        }
                    }
                };
            }
            if score > best.0 {
                best = (score, a.correct);
            }
            if a.correct {
                positive = positive.max(score);
            } else {
                negative = negative.max(score);
            }
        }
        result.correct += usize::from(best.1);
        if negative != i64::MIN {
            result.hinge += (1 + negative - positive).max(0);
        }
    }
    result
}
fn set(block: &mut SourceRouting, kind: u8, index: usize, lane: usize, value: i16) {
    match kind {
        0 => block.codes[index].roots[lane] = value as u16,
        1 => block.landmarks[index][lane] = value as u16,
        _ => block.biases[index] = value,
    }
}
impl Model {
    pub fn fit_source_routing(
        &self,
        docs: &[ValueExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_source_routing_mode(docs, config, false, false)
    }

    /// Refine the existing source/action router in place. Only one refinement
    /// is supported; the exact frozen parent is retained as a load-time witness.
    pub fn refine_source_routing(
        &self,
        docs: &[ValueExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_source_routing_mode(docs, config, true, false)
    }

    /// Warm continuation with four candidate-owned predecessor identities.
    /// The prior assembled artifact is reconstructed exactly by its witness.
    pub fn fit_retained_source_context(
        &self,
        docs: &[ValueExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_source_routing_mode(docs, config, true, true)
    }

    fn fit_source_routing_mode(
        &self,
        docs: &[ValueExample],
        config: SourceRoutingConfig,
        refine: bool,
        retained: bool,
    ) -> Result<(Model, serde_json::Value)> {
        config.validate()?;
        self.validate()?;
        if self.source_routing.is_some() != refine
            || (self.source_routing_refinement.is_some() && !retained)
            || self.source_context.is_some()
            || (retained && self.literal_routing_refinement.is_none())
            || self.learned_routing.is_some()
            || docs.is_empty()
            || docs.len() > 1024
            || docs
                .iter()
                .any(|d| d.prompt.len() + d.response.len() > 65536)
        {
            return Err(Error("source routing parent/source bound invalid".into()));
        }
        let previous = if refine {
            self.source_routing.as_ref()
        } else {
            None
        };
        if previous.is_some_and(|old| {
            old.codes.len() > config.learned_features
                || old.config.role_context_only != config.role_context_only
        }) {
            return Err(Error(
                "source refinement must retain feature vocabulary/context".into(),
            ));
        }
        let read = role_read::head(self)
            .ok_or_else(|| Error("source routing needs accepted reader".into()))?;
        let start = Instant::now();
        let mut frames = Vec::new();
        let mut frequency = BTreeMap::<ValueFeature, u64>::new();
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut skipped = 0;
        let mut persistent = 0;
        let mut dependent = 0;
        let mut unreachable = Vec::new();
        for doc in docs {
            if doc.id.trim().is_empty() || !ids.insert(&doc.id) {
                return Err(Error(
                    "duplicate/empty source routing document identity".into(),
                ));
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
            let values = session
                .values
                .as_ref()
                .ok_or_else(|| Error("missing retained values".into()))?;
            let entry = session
                .response_entry
                .as_ref()
                .ok_or_else(|| Error("missing entry state".into()))?;
            if !super::word_copy_runtime::eligible(self, entry, values, Control::Full) {
                skipped += 1;
                continue;
            }
            if refine
                && super::dependent_read::choose(
                    self,
                    values,
                    Control::Full,
                    &mut WordCopyWork::default(),
                )
                .is_some()
            {
                dependent += 1;
                continue;
            }
            if super::relation::read_choice(self, values, &mut ValueWork::default()).is_some() {
                persistent += 1;
                continue;
            }
            let words = values
                .lexemes
                .as_ref()
                .ok_or_else(|| Error("missing retained words".into()))?;
            let source_control = if config.role_context_only {
                Control::WordCopyGeometryDisabled
            } else {
                Control::Full
            };
            let ctx =
                role_read::context(self, values, source_control, &mut WordCopyWork::default());
            let response_tokens = self.encode(&doc.response)?;
            let mut alternatives = Vec::new();
            let mut copied_target = false;
            for index in 0..=words.query_len {
                let source = if index == words.query_len {
                    NO_SOURCE
                } else {
                    index as u8
                };
                let (features, n) = role_read::features_with_context(
                    self,
                    values,
                    &ctx,
                    index,
                    source_control,
                    retained,
                    &mut WordCopyWork::default(),
                );
                for action in 0..read.actions.len() {
                    if !super::source_routing::allowed(self, values, source, action) {
                        continue;
                    }
                    let a = &read.actions[action];
                    let correct = if a.copy {
                        let w = &words.queries[index];
                        let prefix = a
                            .prefix
                            .map(|t| self.decode(&[t]))
                            .transpose()?
                            .unwrap_or_default();
                        let tail = doc.response.as_bytes().strip_prefix(prefix.as_slice());
                        tail.is_some_and(|tail| {
                            tail.starts_with(&w.bytes[..usize::from(w.len)])
                                && tail
                                    .get(usize::from(w.len))
                                    .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_')
                        })
                    } else {
                        a.prefix == response_tokens.first().copied()
                    };
                    copied_target |= a.copy && correct;
                    alternatives.push(Alternative {
                        features: features[..n].to_vec(),
                        codes: Vec::new(),
                        action,
                        correct,
                    });
                }
            }
            if copied_target {
                for a in &mut alternatives {
                    if !read.actions[a.action].copy {
                        a.correct = false;
                    }
                }
            }
            if !alternatives.iter().any(|a| a.correct) {
                unreachable.push(doc.id.clone());
                continue;
            }
            for a in &alternatives {
                for &f in &a.features {
                    *frequency.entry(f).or_default() += 1;
                }
            }
            frames.push(Frame { alternatives });
        }
        if frames.is_empty() {
            return Err(Error("no reachable source routing training frames".into()));
        }
        let mut salience = BTreeMap::<ValueFeature, u64>::new();
        for row in &read.rows {
            let mut f = row.feature;
            f.kind &= 31;
            *salience.entry(f).or_default() += u64::from(row.weight.unsigned_abs());
        }
        let mut vocabulary: Vec<_> = frequency
            .iter()
            .map(|(f, n)| (*f, salience.get(f).copied().unwrap_or(0), *n))
            .collect();
        vocabulary.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)).then(a.0.cmp(&b.0)));
        if let Some(old) = previous {
            let retained: BTreeSet<_> = old.codes.iter().map(|c| c.feature).collect();
            let mut contrast = BTreeMap::<ValueFeature, usize>::new();
            for frame in &frames {
                let mut counts = BTreeMap::<ValueFeature, (usize, usize)>::new();
                let positives = frame.alternatives.iter().filter(|a| a.correct).count();
                let negatives = frame.alternatives.len() - positives;
                for a in &frame.alternatives {
                    for feature in a.features.iter().copied().collect::<BTreeSet<_>>() {
                        let count = counts.entry(feature).or_default();
                        if a.correct {
                            count.0 += 1;
                        } else {
                            count.1 += 1;
                        }
                    }
                }
                for (feature, (positive, negative)) in counts {
                    if positive * negatives != negative * positives {
                        *contrast.entry(feature).or_default() += 1;
                    }
                }
            }
            vocabulary.retain(|v| !retained.contains(&v.0));
            vocabulary.sort_by(|a, b| {
                contrast
                    .get(&b.0)
                    .copied()
                    .unwrap_or(0)
                    .cmp(&contrast.get(&a.0).copied().unwrap_or(0))
                    .then(b.2.cmp(&a.2))
                    .then(a.0.cmp(&b.0))
            });
            vocabulary.truncate(config.learned_features - old.codes.len());
            vocabulary.extend(old.codes.iter().map(|c| (c.feature, 0, 0)));
        } else {
            vocabulary.truncate(config.learned_features);
        }
        vocabulary.sort_by_key(|v| v.0);
        let mut rng = config.seed;
        let mut block = SourceRouting {
            schema: "uor-r4.geometric-source-routing/1".into(),
            parent_artifact: self.artifact_cid.clone(),
            codes: vocabulary
                .iter()
                .map(|v| SourceCode {
                    feature: v.0,
                    roots: previous
                        .and_then(|old| {
                            old.codes
                                .binary_search_by_key(&v.0, |c| c.feature)
                                .ok()
                                .map(|index| old.codes[index].roots)
                        })
                        .unwrap_or([self.geometry.identity; LANES]),
                })
                .collect(),
            landmarks: (0..read.actions.len())
                .map(|_| {
                    std::array::from_fn(|_| {
                        (super::learned_routing_training::next_random(&mut rng) % 120) as u16
                    })
                })
                .collect(),
            biases: vec![0; read.actions.len()],
            ranks: super::learned_routing_training::ranks(self),
            training: receipts,
            config: config.clone(),
        };
        if let Some(old) = previous {
            block.landmarks.clone_from(&old.landmarks);
            block.biases.clone_from(&old.biases);
        }
        let warm_initialization_preserved = previous.is_none_or(|old| {
            block.landmarks == old.landmarks
                && block.biases == old.biases
                && old.codes.iter().all(|code| {
                    block
                        .codes
                        .binary_search_by_key(&code.feature, |c| c.feature)
                        .is_ok_and(|index| block.codes[index] == *code)
                })
        });
        let warm_feature_alias_frames =
            previous.map(|old| feature_alias_frames(&frames, &old.codes));
        let expanded_feature_alias_frames = feature_alias_frames(&frames, &block.codes);
        let learned = learn(self, &mut block, &mut frames, start);
        let block_bytes = serde_json::to_vec(&block)
            .map_err(|e| Error(e.to_string()))?
            .len();
        let mut model = self.clone();
        model.source_routing = Some(block);
        if let Some(old) = previous {
            let witness = Some(super::source_routing::SourceRoutingRefinement {
                parent_artifact: self.artifact_cid.clone(),
                previous: old.clone(),
            });
            if retained {
                model.source_context = witness;
            } else {
                model.source_routing_refinement = witness;
            }
        }
        model.refresh_identity()?;
        model.validate()?;
        let mut report = serde_json::json!({"schema":"uor-r4.source-routing-fit/1","parent":self.artifact_cid(),"artifact":model.artifact_cid(),"config":config,"documents":docs.len(),"frames":frames.len(),"skipped_upstream":skipped,"preserved_persistent_dispatch":persistent,"unreachable_targets":unreachable,"feature_universe":frequency.len(),"features":vocabulary.len(),"elapsed_ms":start.elapsed().as_millis(),"block_bytes":block_bytes,"scope":"Source/action labels from supplied construction responses; accepted reader weights select feature vocabulary only. Ordered two-channel H4 composition and action landmarks learned with hard serving decisions. Persistent-reader dispatch and numeric eligibility are unchanged. Construction selection is not generation/transfer."});
        report["retained_source_predecessors"] = serde_json::json!(if retained { 4 } else { 0 });
        report["refinement"] = serde_json::json!(refine);
        report["warm_feature_alias_frames"] = serde_json::json!(warm_feature_alias_frames);
        report["expanded_feature_alias_frames"] = serde_json::json!(expanded_feature_alias_frames);
        if refine {
            report["scope"] = serde_json::json!("One warm in-place source/action refinement on exact assembled-parent sessions; all inherited code features retained, added vocabulary ranked by construction label contrasts and frequency. Descendant parameters and original training parent CIDs remain unchanged, verified by exact parent reconstruction. Feature alias counts are descriptive, not frame solvability or generation acceptance.");
        }
        report["warm_initialization_preserved"] = serde_json::json!(warm_initialization_preserved);
        report["preserved_dependent_dispatch"] = serde_json::json!(dependent);
        report["warm_features"] = serde_json::json!(previous.map_or(0, |old| old.codes.len()));
        report["added_features"] =
            serde_json::json!(vocabulary.len() - previous.map_or(0, |old| old.codes.len()));
        report["parent_reconstruction"] = serde_json::json!(if refine {
            "EXACT_FROZEN_PARENT"
        } else {
            "INITIAL_FIT"
        });
        if let (Some(r), Some(l)) = (report.as_object_mut(), learned.as_object()) {
            r.extend(l.clone());
        }
        Ok((model, report))
    }
}

pub(super) fn learn(
    model: &Model,
    block: &mut SourceRouting,
    frames: &mut [Frame],
    start: Instant,
) -> serde_json::Value {
    learn_impl(model, block, frames, start, None)
}

/// Offline restricted coordinate fitting. The mask indexes the final sorted
/// vocabulary; all other roots, landmarks and biases are frozen.
pub(super) fn learn_code_subset(
    model: &Model,
    block: &mut SourceRouting,
    frames: &mut [Frame],
    start: Instant,
    mutable: &[bool],
) -> Result<serde_json::Value> {
    if mutable.len() != block.codes.len() {
        return Err(Error("mutable source-code mask length differs".into()));
    }
    Ok(learn_impl(model, block, frames, start, Some(mutable)))
}
fn learn_impl(
    model: &Model,
    block: &mut SourceRouting,
    frames: &mut [Frame],
    start: Instant,
    mutable: Option<&[bool]>,
) -> serde_json::Value {
    let config = block.config.clone();
    let mut rng = config.seed;
    for _ in 0..block.landmarks.len() * LANES {
        super::learned_routing_training::next_random(&mut rng);
    }
    for frame in frames.iter_mut() {
        for a in &mut frame.alternatives {
            a.codes = a
                .features
                .iter()
                .filter_map(|f| block.codes.binary_search_by_key(f, |c| c.feature).ok())
                .collect();
        }
    }
    let initial = objective(model, block, frames);
    let mut best = initial;
    let mut proposals = 0;
    let mut accepted = [0_usize; 3];
    let mut stopped = false;
    'passes: for _ in 0..config.passes {
        for kind in [2, 0, 1] {
            if mutable.is_some() && kind != 0 {
                continue;
            }
            let count = if kind == 0 {
                block.codes.len()
            } else {
                block.landmarks.len()
            };
            for index in 0..count {
                if mutable.is_some_and(|mask| !mask[index]) {
                    continue;
                }
                for lane in 0..if kind == 2 { 1 } else { LANES } {
                    let mut value = match kind {
                        0 => block.codes[index].roots[lane] as i16,
                        1 => block.landmarks[index][lane] as i16,
                        _ => block.biases[index],
                    };
                    let candidates: Vec<i16> = match kind {
                        0 => std::iter::once(model.geometry.identity as i16)
                            .chain((0..config.proposals).map(|_| {
                                (super::learned_routing_training::next_random(&mut rng) % 120)
                                    as i16
                            }))
                            .collect(),
                        1 => (0..120).collect(),
                        _ => (-32..=32).collect(),
                    };
                    for candidate in candidates {
                        if start.elapsed().as_secs() >= config.max_seconds {
                            stopped = true;
                            set(block, kind, index, lane, value);
                            break 'passes;
                        }
                        if candidate == value {
                            continue;
                        }
                        set(block, kind, index, lane, candidate);
                        let score = objective(model, block, frames);
                        proposals += 1;
                        if score.improves(best) {
                            value = candidate;
                            best = score;
                            accepted[usize::from(kind)] += 1;
                        }
                    }
                    set(block, kind, index, lane, value);
                }
            }
        }
    }

    serde_json::json!({"initial_correct":initial.correct,"final_correct":best.correct,"initial_hinge":initial.hinge,"final_hinge":best.hinge,"proposals":proposals,"accepted":accepted,"stopped_at_time_limit":stopped})
}

impl Model {
    /// Offline, allocating inspection of the existing first source decision.
    /// Candidate scores are joint action scores, not calibrated language scores.
    /// This does not alter the model, session cache, or serving feature law.
    pub fn source_routing_trace(&self, prompt: &str) -> Result<serde_json::Value> {
        let block = self
            .source_routing
            .as_ref()
            .ok_or_else(|| Error("source trace requires source router".into()))?;
        if prompt.len() > 65536 {
            return Err(Error("source trace prompt exceeds bound".into()));
        }
        let mut session = self.session(Control::Full)?;
        session.observe(self, BOS)?;
        for token in self.encode(prompt)? {
            session.observe(self, token)?;
        }
        session.begin_response(self)?;
        let prediction = session.predict(self)?;
        let values = session
            .values
            .as_ref()
            .ok_or_else(|| Error("source trace retained values absent".into()))?;
        let words = values
            .lexemes
            .as_ref()
            .ok_or_else(|| Error("source trace retained words absent".into()))?;
        let read =
            role_read::head(self).ok_or_else(|| Error("source trace role reader absent".into()))?;
        let entry = session
            .response_entry
            .as_ref()
            .ok_or_else(|| Error("source trace response entry absent".into()))?;
        let eligible = super::word_copy_runtime::eligible(self, entry, values, Control::Full);
        let dependent = super::dependent_read::choose(
            self,
            values,
            Control::Full,
            &mut WordCopyWork::default(),
        );
        let persistent = super::relation::read_choice(self, values, &mut ValueWork::default());
        let feature_control = if block.config.role_context_only {
            Control::WordCopyGeometryDisabled
        } else {
            Control::Full
        };
        let mut work = WordCopyWork::default();
        let ctx = role_read::context(self, values, feature_control, &mut work);
        let mut candidates = Vec::new();
        for index in 0..=words.query_len {
            let source = if index == words.query_len {
                NO_SOURCE
            } else {
                index as u8
            };
            let (features, n) = role_read::features_with_context(
                self,
                values,
                &ctx,
                index,
                feature_control,
                self.source_context.is_some(),
                &mut work,
            );
            let state = block.encode(self, &features[..n], Control::Full, &mut work.routing);
            let mapped: Vec<_> =
                features[..n]
                    .iter()
                    .filter_map(|feature| {
                        block.codes.binary_search_by_key(feature, |c| c.feature).ok().map(|i| {
                    serde_json::json!({"feature":feature,"roots":block.codes[i].roots})
                })
                    })
                    .collect();
            let mut scores = Vec::new();
            for action in 0..read.actions.len() {
                if super::source_routing::allowed(self, values, source, action) {
                    scores.push(serde_json::json!({
                        "action":action,"copy":read.actions[action].copy,
                        "prefix":read.actions[action].prefix,
                        "score":block.score(self, state, action, &mut work.routing)
                    }));
                }
            }
            let word = (index < words.query_len).then(|| &words.queries[index]);
            candidates.push(serde_json::json!({
                "source":source,
                "word":word.map(|w| String::from_utf8_lossy(&w.bytes[..usize::from(w.len)]).into_owned()),
                "address":if index < words.query_len {ctx.addresses[index]} else {0},
                "source_end":word.map(|w| w.end),"source_byte_end":word.map(|w| w.byte_end),
                "features":&features[..n],"mapped_codes":mapped,"state":state,"scores":scores
            }));
        }
        Ok(serde_json::json!({
            "schema":"uor-r4.source-routing-trace/1", "artifact":self.artifact_cid(),
            "prompt":prompt,"prediction_token":prediction.token,
            "actual_word_copy_decision":session.word_copy_decision(),
            "direct_source_dispatch":eligible && dependent.is_none() && persistent.is_none(),
            "word_copy_eligible":eligible,"dependent_choice":dependent,"persistent_choice":persistent,
            "direct_choice":super::source_routing::choose(self, values, Control::Full, &mut WordCopyWork::default()),
            "candidates":candidates,
            "scope":"Allocating offline inspection; counterfactual direct candidates are marked when upstream dispatch bypasses this router."
        }))
    }
}
