//! Bounded current-version context for the existing shared geometric source reader.
//! Both historical and current occurrences remain admitted; parent routing is frozen.
use super::relation::{RelationRecord, RelationState};
use super::role_read::{self, NO_SOURCE};
use super::source_routing::SourceRouting;
use super::source_routing_training::{learn_code_subset, Alternative, Frame};
use super::value_lexemes::WordAtom;
use super::value_types::{ValueFeature, ValueState};
use super::word_copy_types::WordCopyContext;
use super::*;
use std::{collections::BTreeSet, time::Instant};
const KIND: u8 = 30;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurrentSourceTarget {
    Preserve,
    CurrentRevision,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurrentSourceExample {
    pub id: String,
    pub prompt: String,
    pub target: CurrentSourceTarget,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CurrentSource {
    pub parent_artifact: String,
    pub previous: SourceRouting,
    pub config: SourceRoutingConfig,
    pub training: Vec<DocumentReceipt>,
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
fn association<'a>(
    state: &'a RelationState,
    candidate: &WordAtom,
    work: &mut WordCopyWork,
) -> Option<(&'a RelationRecord, u64)> {
    let mut found = None;
    for id in state.directory {
        work.persistent_read.relations.directory_reads = work
            .persistent_read
            .relations
            .directory_reads
            .saturating_add(1);
        work.persistent_read.relations.record_reads = work
            .persistent_read
            .relations
            .record_reads
            .saturating_add(1);
        let Some(current) = state.record(id) else {
            continue;
        };
        if current.conflict || current.action != 2 || current.previous == 0 {
            continue;
        }
        let mut class = if current.value == *candidate { 1 } else { 0 };
        if class == 0 {
            let mut previous = current.previous;
            let mut newer_id = current.id;
            for _ in 0..16 {
                if previous == 0 || previous >= newer_id {
                    break;
                }
                work.persistent_read.relations.record_reads = work
                    .persistent_read
                    .relations
                    .record_reads
                    .saturating_add(1);
                let Some(ancestor) = state.record(previous) else {
                    break;
                };
                work.persistent_read.relations.source_presence_checks = work
                    .persistent_read
                    .relations
                    .source_presence_checks
                    .saturating_add(1);
                if !ancestor.conflict && ancestor.value == *candidate {
                    class = 2;
                    break;
                }
                newer_id = ancestor.id;
                previous = ancestor.previous;
            }
        }
        if class != 0 {
            if found.is_some() {
                return None;
            }
            found = Some((current, class));
        }
    }
    found
}
fn query_hints(
    current: &RelationRecord,
    class: u64,
    queries: &[WordAtom],
    addresses: &[u32; 16],
    work: &mut WordCopyWork,
) -> ([ValueFeature; 8], usize) {
    let mut out = [ValueFeature::default(); 8];
    let mut n = 0;
    for q in 0..queries.len().min(8) {
        if current
            .owner
            .matches(&queries[q], &mut work.persistent_read)
        {
            let before = if q + 1 < queries.len() {
                addresses[q + 1]
            } else {
                0
            };
            let after = q.checked_sub(1).map_or(0, |i| addresses[i]);
            out[n] = ValueFeature {
                kind: KIND,
                a: class,
                b: (u64::from(before) << 32) | u64::from(after),
            };
            n += 1;
        }
    }
    work.selector.feature_queries = work.selector.feature_queries.saturating_add(n as u64);
    (out, n)
}
fn hints_for_values(
    values: &ValueState,
    ctx: &WordCopyContext,
    index: usize,
    work: &mut WordCopyWork,
) -> ([ValueFeature; 8], usize) {
    let (Some(state), Some(words)) = (&values.relations, &values.lexemes) else {
        return ([ValueFeature::default(); 8], 0);
    };
    if index >= words.query_len {
        return ([ValueFeature::default(); 8], 0);
    }
    let Some((current, class)) = association(state, &words.queries[index], work) else {
        return ([ValueFeature::default(); 8], 0);
    };
    query_hints(
        current,
        class,
        &words.queries[..words.query_len],
        &ctx.addresses,
        work,
    )
}
fn enabled(control: Control) -> bool {
    !matches!(
        control,
        Control::CurrentSourceDisabled | Control::CurrentSourceVersionDisabled
    )
}
pub(super) fn hints(
    model: &Model,
    values: &ValueState,
    ctx: &WordCopyContext,
    index: usize,
    control: Control,
    work: &mut WordCopyWork,
) -> ([ValueFeature; 8], usize) {
    if model.current_source.is_none() || !enabled(control) {
        return ([ValueFeature::default(); 8], 0);
    }
    hints_for_values(values, ctx, index, work)
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

/// Offline target from the frozen learned persistent reader, allowing recent values.
fn current_target(model: &Model, values: &ValueState) -> Result<(u8, usize)> {
    let mut work = ValueWork::default();
    let choice = super::relation::read_choice_with_recent(model, values, true, &mut work)
        .ok_or_else(|| Error("current revision frozen read has no positive choice".into()))?;
    if choice.0 < super::relation::RELATION_SOURCE {
        return Err(Error("current revision frozen read abstains".into()));
    }
    let state = values
        .relations
        .as_ref()
        .ok_or_else(|| Error("current revision records absent".into()))?;
    let selected = state
        .records
        .get(usize::from(choice.0 - super::relation::RELATION_SOURCE))
        .ok_or_else(|| Error("current revision source unavailable".into()))?;
    if selected.conflict
        || selected.action != 2
        || selected.previous == 0
        || !state.directory.contains(&selected.id)
    {
        return Err(Error(
            "current revision target is not a current explicit revision".into(),
        ));
    }
    let words = values
        .lexemes
        .as_ref()
        .ok_or_else(|| Error("current revision words absent".into()))?;
    let role = role_read::head(model)
        .ok_or_else(|| Error("current revision role reader absent".into()))?;
    let reader = super::relation::head(model)
        .ok_or_else(|| Error("current revision persistent reader absent".into()))?;
    let addr = super::relation::addresses(model, &words.queries[..words.query_len], &mut work);
    let (features, n) = super::relation::read_features(model, selected, words, &addr, &mut work);
    let selected_score = super::relation::score(
        &reader.reader,
        &features[..n],
        (choice.1 + 1) as u8,
        &mut work,
    );
    // Reject tied positive records/actions rather than choosing an ambiguous label.
    for id in state.directory {
        let Some(record) = state.record(id) else {
            continue;
        };
        let (features, n) = super::relation::read_features(model, record, words, &addr, &mut work);
        for (action, a) in role.actions.iter().enumerate() {
            if a.copy
                && usize::from(record.span.as_ref().map_or(record.value.len, |s| s.len))
                    + 1
                    + usize::from(a.prefix.is_some())
                    > usize::from(super::response_entry_types::RESPONSE_ENTRY_STEPS)
            {
                continue;
            }
            let score = super::relation::score(
                &reader.reader,
                &features[..n],
                (action + 1) as u8,
                &mut work,
            );
            if (record.id != selected.id || action != choice.1) && score >= selected_score {
                return Err(Error(
                    "current revision frozen read target is ambiguous".into(),
                ));
            }
        }
    }
    let mut source = None;
    for (i, word) in words.queries[..words.query_len].iter().enumerate() {
        if *word == selected.value {
            if source.is_some() {
                return Err(Error("current revision recent occurrence ambiguous".into()));
            }
            source = Some(i as u8);
        }
    }
    let source = source.ok_or_else(|| Error("current revision value is not recent".into()))?;
    Ok((source, choice.1))
}

fn validate_config(config: &SourceRoutingConfig) -> Result<()> {
    if !(1..=4096).contains(&config.learned_features)
        || !(1..=8).contains(&config.passes)
        || !(1..=120).contains(&config.proposals)
        || !(1..=180).contains(&config.max_seconds)
    {
        return Err(Error("invalid current source configuration".into()));
    }
    Ok(())
}
fn frozen_equal(current: &SourceRouting, previous: &SourceRouting) -> bool {
    if previous.codes.iter().any(|c| c.feature.kind == KIND) {
        return false;
    }
    let mut restored = current.clone();
    restored.codes.retain(|c| c.feature.kind != KIND);
    if current.config.learned_features < previous.config.learned_features {
        return false;
    }
    restored.config.learned_features = previous.config.learned_features;
    restored == *previous
}
impl CurrentSource {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        validate_config(&self.config)?;
        if self.training.is_empty()
            || self.training.len() > 1200
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
            return Err(Error("invalid current source receipts".into()));
        }
        // Restore the entire retained model before checking the new parameters.
        let mut parent = model.clone();
        parent.current_source = None;
        parent.source_routing = Some(self.previous.clone());
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("current source frozen parent differs".into()));
        }
        parent.validate()?;
        let current = model
            .source_routing
            .as_ref()
            .ok_or_else(|| Error("current source router absent".into()))?;
        current.validate(model)?;
        if !current.codes.iter().any(|c| c.feature.kind == KIND)
            || !frozen_equal(current, &self.previous)
            || self.config.mode != current.config.mode
            || self.config.role_context_only != current.config.role_context_only
            || self.config.learned_features < current.codes.len()
        {
            return Err(Error("current source frozen parameters differ".into()));
        }
        let role = role_read::head(model)
            .ok_or_else(|| Error("current source role reader absent".into()))?;
        let primes: BTreeSet<_> = std::iter::once(0)
            .chain(role.dictionary.iter().map(|d| d.prime))
            .collect();
        if current
            .codes
            .iter()
            .filter(|c| c.feature.kind == KIND)
            .any(|c| {
                !matches!(c.feature.a, 1 | 2)
                    || !primes.contains(&((c.feature.b >> 32) as u32))
                    || !primes.contains(&(c.feature.b as u32))
            })
        {
            return Err(Error("invalid current source version features".into()));
        }
        let mut identity = model.clone();
        identity.refresh_identity()?;
        if identity.artifact_cid != model.artifact_cid
            || identity.uor_model_address != model.uor_model_address
        {
            return Err(Error("current source identity differs".into()));
        }
        Ok(())
    }
}
impl Model {
    pub fn without_current_source(&self) -> Result<Model> {
        let Some(w) = &self.current_source else {
            return Ok(self.clone());
        };
        let mut parent = self.clone();
        parent.current_source = None;
        parent.source_routing = Some(w.previous.clone());
        parent.refresh_identity()?;
        if parent.artifact_cid() != w.parent_artifact {
            return Err(Error("current source parent differs".into()));
        }
        parent.validate()?;
        Ok(parent)
    }
    /// Exact parent source/action preservation or an existing learned current revision read.
    pub fn fit_current_source(
        &self,
        docs: &[CurrentSourceExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        let start = Instant::now();
        self.validate()?;
        validate_config(&config)?;
        if self.current_source.is_some() || docs.is_empty() || docs.len() > 1200 {
            return Err(Error("invalid current source parent/population".into()));
        }
        let previous = self
            .source_routing
            .as_ref()
            .ok_or_else(|| Error("current source router absent".into()))?;
        if previous.codes.len() > config.learned_features
            || previous.config.mode != config.mode
            || previous.config.role_context_only != config.role_context_only
        {
            return Err(Error("current source vocabulary/context differs".into()));
        }
        let read =
            role_read::head(self).ok_or_else(|| Error("current source reader absent".into()))?;

        let feature_control = if previous.config.role_context_only {
            Control::WordCopyGeometryDisabled
        } else {
            Control::Full
        };
        let mut frames = Vec::new();
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut labels = Vec::new();
        let mut skips = Vec::new();
        let mut unique_frames = BTreeSet::new();
        let mut prompt_bytes = 0_usize;
        for doc in docs {
            if start.elapsed().as_secs() >= config.max_seconds {
                return Err(Error(
                    "current source preparation time bound reached".into(),
                ));
            }
            prompt_bytes = prompt_bytes.saturating_add(doc.prompt.len());
            if prompt_bytes > 8 * 1024 * 1024 {
                return Err(Error("current source prompt byte cap".into()));
            }
            if doc.id.trim().is_empty()
                || !ids.insert(&doc.id)
                || doc.prompt.is_empty()
                || doc.prompt.len() > 65536
            {
                return Err(Error("invalid current source document".into()));
            }
            receipts.push(super::training::receipt(&Document {
                id: doc.id.clone(),
                text: serde_json::to_string(doc).map_err(|e| Error(e.to_string()))?,
            }));
            let mut s = self.session(Control::Full)?;
            s.observe(self, BOS)?;
            for t in self.encode(&doc.prompt)? {
                s.observe(self, t)?;
            }
            s.begin_response(self)?;
            s.predict(self)?;
            let values = s
                .values
                .as_ref()
                .ok_or_else(|| Error("current source values absent".into()))?;
            let entry = s
                .response_entry
                .as_ref()
                .ok_or_else(|| Error("current source entry absent".into()))?;
            let skipped = if !super::word_copy_runtime::eligible(self, entry, values, Control::Full)
            {
                Some("ineligible")
            } else if super::dependent_read::choose(
                self,
                values,
                Control::Full,
                &mut WordCopyWork::default(),
            )
            .is_some()
            {
                Some("dependent")
            } else if super::relation::read_choice(self, values, &mut ValueWork::default())
                .is_some()
            {
                Some("persistent")
            } else {
                None
            };
            if let Some(reason) = skipped {
                if doc.target == CurrentSourceTarget::CurrentRevision {
                    return Err(Error(format!(
                        "current source revision target bypassed: {} ({reason})",
                        doc.id
                    )));
                }
                skips.push(serde_json::json!({"id":doc.id,"reason":reason}));
                continue;
            }
            let actual = super::source_routing::choose(
                self,
                values,
                Control::Full,
                &mut WordCopyWork::default(),
            )
            .ok_or_else(|| Error("current source parent choice absent".into()))?;
            let (source, action) = match doc.target {
                CurrentSourceTarget::Preserve => (actual.0, actual.1),
                CurrentSourceTarget::CurrentRevision => {
                    current_target(self, values).map_err(|e| Error(format!("{}: {}", doc.id, e)))?
                }
            };
            let words = values
                .lexemes
                .as_ref()
                .ok_or_else(|| Error("current source words absent".into()))?;
            let ctx =
                role_read::context(self, values, feature_control, &mut WordCopyWork::default());
            let mut alternatives = Vec::new();
            for index in 0..=words.query_len {
                let candidate = if index == words.query_len {
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
                    &mut WordCopyWork::default(),
                );
                let mut features = features[..n].to_vec();
                let (hint, hn) =
                    hints_for_values(values, &ctx, index, &mut WordCopyWork::default());
                features.extend_from_slice(&hint[..hn]);
                for ai in 0..read.actions.len() {
                    if super::source_routing::allowed(self, values, candidate, ai) {
                        alternatives.push(Alternative {
                            features: features.clone(),
                            codes: Vec::new(),
                            action: ai,
                            correct: candidate == source && ai == action,
                        });
                    }
                }
            }
            if !alternatives
                .iter()
                .any(|a| a.features.iter().any(|f| f.kind == KIND))
            {
                if doc.target == CurrentSourceTarget::CurrentRevision {
                    return Err(Error(format!(
                        "current revision has no version hints: {}",
                        doc.id
                    )));
                }
                skips.push(serde_json::json!({"id":doc.id,"reason":"no_new_hint","parent_source":actual.0,"parent_action":actual.1}));
                continue;
            }
            if alternatives.iter().filter(|a| a.correct).count() != 1 {
                return Err(Error("current source exact target unavailable".into()));
            }
            let atom = if source == NO_SOURCE {
                None
            } else {
                super::relation::source(values, source)
            };
            labels.push(serde_json::json!({"id":doc.id,"target":doc.target,"parent_source":actual.0,"parent_action":actual.1,"source":source,"action":action,"source_end":atom.map(|w|w.end),"source_byte_end":atom.map(|w|w.byte_end)}));
            let signature: Vec<_> = alternatives
                .iter()
                .map(|a| (a.features.clone(), a.action, a.correct))
                .collect();
            if unique_frames.insert(signature) {
                frames.push(Frame { alternatives });
            } else {
                skips.push(
                    serde_json::json!({"id":doc.id,"reason":"identical_full_constraint_frame"}),
                );
            }
        }
        if frames.is_empty() {
            return Err(Error("current source has no direct frames".into()));
        }
        let mut block = previous.clone();
        if block.codes.iter().any(|c| c.feature.kind == KIND) {
            return Err(Error("current source namespace already occupied".into()));
        }
        let new_features: BTreeSet<_> = frames
            .iter()
            .flat_map(|f| &f.alternatives)
            .flat_map(|a| &a.features)
            .filter(|f| f.kind == KIND)
            .copied()
            .collect();
        if new_features.is_empty() {
            return Err(Error("current source has no version features".into()));
        }
        for feature in new_features {
            block.codes.push(super::source_routing::SourceCode {
                feature,
                roots: [self.geometry.identity; 2],
            });
        }
        block.codes.sort_by_key(|c| c.feature);
        if block.codes.len() > config.learned_features {
            return Err(Error("current source feature capacity exceeded".into()));
        }
        block.config = config.clone();
        let mask: Vec<_> = block.codes.iter().map(|c| c.feature.kind == KIND).collect();
        let fit = learn_code_subset(self, &mut block, &mut frames, start, &mask)?;
        block.config = previous.config.clone();
        block.config.learned_features = block.config.learned_features.max(block.codes.len());
        let changed: Vec<_> = block
            .codes
            .iter()
            .filter(|c| c.feature.kind == KIND)
            .map(|c| serde_json::json!({"feature":c.feature,"roots":c.roots}))
            .collect();
        let mut model = self.clone();
        model.source_routing = Some(block);
        model.current_source = Some(CurrentSource {
            parent_artifact: self.artifact_cid.clone(),
            previous: previous.clone(),
            config,
            training: receipts,
        });
        model.refresh_identity()?;
        model.validate()?;
        let report = serde_json::json!({"schema":"uor-r4.current-source-fit/1","parent_artifact":self.artifact_cid(),"artifact":model.artifact_cid(),"documents":docs.len(),"frames":frames.len(),"labels":labels,"skips":skips,"mutable_codes":mask.iter().filter(|m|**m).count(),"changed_codes":changed,"fit":fit,"elapsed_ms":start.elapsed().as_millis(),"prompt_bytes":prompt_bytes,"training":model.current_source.as_ref().map(|w|&w.training),"namespace":KIND,"scope":"Offline exact parent source/action preservation or unique frozen learned current-revision read; only new version-context roots learned. Both sources remain admitted; free generation is separate."});
        Ok((model, report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn atom(text: &str, end: u64) -> WordAtom {
        let mut w = WordAtom {
            len: text.len() as u8,
            end,
            byte_end: end,
            ..Default::default()
        };
        w.bytes[..text.len()].copy_from_slice(text.as_bytes());
        w
    }
    fn records() -> RelationState {
        let mut s = RelationState::default();
        let owner = atom("owner", 9);
        s.records[0] = RelationRecord {
            id: 1,
            owner,
            value: atom("old", 15),
            action: 1,
            ..Default::default()
        };
        s.records[1] = RelationRecord {
            id: 2,
            owner: atom("owner", 20),
            value: atom("new", 25),
            action: 2,
            previous: 1,
            ..Default::default()
        };
        s.directory[0] = 2;
        s.next_id = 3;
        s
    }
    #[test]
    fn current_source_exact_occurrence_and_previous_chain_are_distinct_from_spelling() {
        let s = records();
        let mut work = WordCopyWork::default();
        assert_eq!(
            association(&s, &s.records[0].value, &mut work).map(|(r, c)| (r.id, c)),
            Some((2, 2))
        );
        assert_eq!(
            association(&s, &s.records[1].value, &mut work).map(|(r, c)| (r.id, c)),
            Some((2, 1))
        );
        assert!(association(&s, &atom("old", 99), &mut work).is_none());
        let mut changed = s.clone();
        changed.records[1].previous = 0;
        assert!(association(&changed, &changed.records[0].value, &mut work).is_none());
        changed.records[1].previous = 1;
        changed.records[0].id = 17;
        assert!(association(&changed, &s.records[0].value, &mut work).is_none());
    }
    #[test]
    fn current_source_assert_conflict_and_ambiguous_records_have_no_hint() {
        let mut s = records();
        let mut work = WordCopyWork::default();
        s.records[1].action = 1;
        assert!(association(&s, &s.records[1].value, &mut work).is_none());
        s.records[1].action = 2;
        s.records[1].conflict = true;
        assert!(association(&s, &s.records[0].value, &mut work).is_none());
        s.records[1].conflict = false;
        s.records[0].conflict = true;
        assert!(association(&s, &s.records[0].value, &mut work).is_none());
        s.records[0].conflict = false;
        s.records[2] = s.records[1];
        s.records[2].id = 3;
        s.directory[1] = 3;
        assert!(association(&s, &s.records[0].value, &mut work).is_none());
        assert!(association(&s, &s.records[1].value, &mut work).is_none());
    }
    #[test]
    fn current_source_hints_join_ordered_query_primes_without_owner_identity_hash() {
        let s = records();
        let queries = [atom("after", 40), atom("owner", 35), atom("before", 30)];
        let mut addr = [0; 16];
        addr[..3].copy_from_slice(&[719, 0, 353]);
        let mut work = WordCopyWork::default();
        let (h, n) = query_hints(&s.records[1], 2, &queries, &addr, &mut work);
        assert_eq!(n, 1);
        assert_eq!(
            h[0],
            ValueFeature {
                kind: 30,
                a: 2,
                b: (353_u64 << 32) | 719
            }
        );
        let absent = [atom("other", 35)];
        assert_eq!(
            query_hints(&s.records[1], 1, &absent, &addr, &mut work).1,
            0
        );
        let nine = [atom("owner", 45); 9];
        assert_eq!(query_hints(&s.records[1], 1, &nine, &addr, &mut work).1, 8);
        assert!(enabled(Control::Full));
        assert!(!enabled(Control::CurrentSourceDisabled));
        assert!(!enabled(Control::CurrentSourceVersionDisabled));
        assert!(enabled(Control::CurrentRelationReadAll));
    }
    fn router() -> SourceRouting {
        SourceRouting {
            schema: "test".into(),
            parent_artifact: "p".into(),
            codes: vec![super::super::source_routing::SourceCode {
                feature: ValueFeature {
                    kind: 6,
                    a: 1,
                    b: 0,
                },
                roots: [5, 119],
            }],
            landmarks: vec![[119, 119]],
            biases: vec![0],
            ranks: vec![0],
            training: Vec::new(),
            config: SourceRoutingConfig::default(),
        }
    }
    #[test]
    fn current_source_freezes_every_parent_root_landmark_bias_and_namespace() {
        let old = router();
        let mut new = old.clone();
        new.codes.push(super::super::source_routing::SourceCode {
            feature: ValueFeature {
                kind: 30,
                a: 1,
                b: 0,
            },
            roots: [1, 2],
        });
        assert!(frozen_equal(&new, &old));
        new.codes[0].roots[0] = 7;
        assert!(!frozen_equal(&new, &old));
        new.codes[0] = old.codes[0].clone();
        new.biases[0] = 1;
        assert!(!frozen_equal(&new, &old));
        new.biases = old.biases.clone();
        new.landmarks[0][0] = 0;
        assert!(!frozen_equal(&new, &old));
        new.landmarks = old.landmarks.clone();
        new.config.seed += 1;
        assert!(!frozen_equal(&new, &old));
        new.config = old.config.clone();
        new.codes[1].feature.kind = 29;
        assert!(!frozen_equal(&new, &old));
        let mut occupied = old.clone();
        occupied.codes[0].feature.kind = 30;
        assert!(!frozen_equal(&occupied, &occupied));
    }
    #[test]
    fn current_source_targets_and_configuration_are_explicit_and_bounded() {
        assert!(
            serde_json::from_str::<CurrentSourceExample>(r#"{"id":"x","prompt":"p"}"#).is_err()
        );
        let d: CurrentSourceExample =
            serde_json::from_str(r#"{"id":"x","prompt":"p","target":"current_revision"}"#).unwrap();
        assert_eq!(d.target, CurrentSourceTarget::CurrentRevision);
        let mut c = SourceRoutingConfig {
            learned_features: 4096,
            max_seconds: 180,
            ..Default::default()
        };
        assert!(validate_config(&c).is_ok());
        c.max_seconds = 181;
        assert!(validate_config(&c).is_err());
    }
}
