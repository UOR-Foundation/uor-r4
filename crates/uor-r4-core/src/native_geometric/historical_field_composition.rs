//! Exact selected previous-version fields share the frozen lexical router.
//! Typed pieces are offline labels; serving has no phrase or response plan.
use super::field_composition::{self as field, FieldAnchor, FieldState};
use super::lexical_emission::LexicalEmission;
use super::response_entry_types::ResponseEntryState;
use super::source_routing::SourceCode;
use super::source_routing_training::{learn_code_subset, Alternative, Frame};
use super::value_types::{ValueFeature, ValueState};
use super::word_copy_types::{WordCopyAction, WordCopyState, WordCopyWork};
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HistoricalFieldComposition {
    pub parent_artifact: String,
    pub previous_lexical: LexicalEmission,
    pub tokens: Vec<u32>,
    pub config: SourceRoutingConfig,
    pub training: Vec<DocumentReceipt>,
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
/// Requires the actual historical choice, not merely an old word in recent memory.
pub(super) fn initial_anchor(
    model: &Model,
    copy: &WordCopyState,
    entry: &ResponseEntryState,
    values: &ValueState,
    baseline: Candidate,
    control: Control,
    work: &mut WordCopyWork,
) -> Option<FieldAnchor> {
    if entry.steps != 0 || entry.active || values.pending.is_some() {
        return None;
    }
    let decision = copy.pending?;
    if decision.dependency.is_some()
        || decision.token != baseline.token
        || decision.score != baseline.score
        || !matches!(
            decision.action,
            WordCopyAction::Read | WordCopyAction::Prepare
        )
    {
        return None;
    }
    if let Some(v) = super::historical_version_intent::choose_detail(model, values, control, work) {
        // The learned version selector owns this response; the frozen reader is bypassed.
        // An abstention has no record to compose.
        if v.source == super::role_read::NO_SOURCE
            || v.source != decision.word_index
            || decision.span_words != 0
        {
            return None;
        }
        let action = super::role_read::head(model)?.actions.get(v.action)?;
        if !action.copy
            || action.prefix != Some(decision.token)
            || decision.action != WordCopyAction::Prepare
        {
            return None;
        }
        let relations = values.relations.as_ref()?;
        let source = super::relation::source(values, v.source)?;
        if source.end != decision.source_end || source.byte_end != decision.source_byte_end {
            return None;
        }
        work.persistent_read.relations.record_reads += 1;
        let old = relations.record(v.record)?;
        if old.value != *source || relations.directory.contains(&old.id) {
            return None;
        }
        let anchor = FieldAnchor {
            source: v.source,
            span_words: decision.span_words,
            relation_id: old.id,
            current_revision: Some(v.current),
            ancestor_depth: if v.depth > 1 { Some(v.depth) } else { None },
            // Recorded only when the proven path actually contains a reassertion
            // link, so revision-only anchors keep their exact legacy wire form.
            reassertion_links: v.reassertion,
            source_end: source.end,
            source_byte_end: source.byte_end,
            boundary_seen: entry.boundary?.at_seen,
        };
        field::record(values, anchor, work)?;
        return span_matches(values, anchor, old, work).then_some(anchor);
    }
    let (source_index, action) = super::historical_read::choose(model, values, control, work)?;
    if source_index != decision.word_index || decision.span_words != 0 {
        return None;
    }
    let action = super::role_read::head(model)?.actions.get(action)?;
    // The historical reader's validated copy action uses its exact Prepare prefix.
    if !action.copy
        || action.prefix != Some(decision.token)
        || decision.action != WordCopyAction::Prepare
    {
        return None;
    }
    let relations = values.relations.as_ref()?;
    let source = super::relation::source(values, source_index)?;
    if source.end != decision.source_end || source.byte_end != decision.source_byte_end {
        return None;
    }
    let mut found = None;
    for id in &relations.directory {
        work.persistent_read.relations.record_reads += 1;
        let Some(current) = relations.record(*id) else {
            continue;
        };
        let Some(old) =
            super::historical_read::previous(relations, current, &mut work.persistent_read)
        else {
            continue;
        };
        if old.value != *source
            || source_index != super::relation::RELATION_SOURCE + ((old.id - 1) & 15) as u8
        {
            continue;
        }
        if found.is_some() || relations.directory.contains(&old.id) {
            return None;
        }
        found = Some((current.id, old));
    }
    let (current_revision, old) = found?;
    let anchor = FieldAnchor {
        source: source_index,
        span_words: decision.span_words,
        relation_id: old.id,
        current_revision: Some(current_revision),
        ancestor_depth: None,
        reassertion_links: false,
        source_end: source.end,
        source_byte_end: source.byte_end,
        boundary_seen: entry.boundary?.at_seen,
    };
    field::record(values, anchor, work)?;
    span_matches(values, anchor, old, work).then_some(anchor)
}
/// The retained source window must replay the selected record's complete exact payload.
fn span_matches(
    values: &ValueState,
    anchor: FieldAnchor,
    old: &super::relation::RelationRecord,
    work: &mut WordCopyWork,
) -> bool {
    let Some(length) = super::source_span::len(values, anchor.source, anchor.span_words, work)
    else {
        return false;
    };
    if length != old.span.as_ref().map_or(old.value.len, |s| s.len) {
        return false;
    }
    for i in 0..length {
        let expected = old
            .span
            .as_ref()
            .map_or(old.value.bytes[usize::from(i)], |s| s.bytes[usize::from(i)]);
        if super::source_span::byte(values, anchor.source, anchor.span_words, i, work)
            != Some(expected)
        {
            return false;
        }
    }
    true
}
fn historical_feature(f: ValueFeature) -> bool {
    matches!(f.kind, 0 | 3) && f.a >> 56 == 18
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

fn frozen(active: &LexicalEmission, previous: &LexicalEmission) -> bool {
    if previous
        .router
        .codes
        .iter()
        .any(|c| historical_feature(c.feature))
    {
        return false;
    }
    let mut restored = active.clone();
    restored
        .router
        .codes
        .retain(|c| !historical_feature(c.feature));
    restored == *previous
}
impl HistoricalFieldComposition {
    fn parent(&self, model: &Model) -> Result<Model> {
        let mut parent = model.clone();
        parent.historical_field_composition = None;
        parent.lexical_emission = Some(self.previous_lexical.clone());
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("historical field frozen parent differs".into()));
        }
        parent.validate()?;
        Ok(parent)
    }
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        self.config.validate_word_emission()?;
        self.parent(model)?;
        let field = model
            .field_composition
            .as_ref()
            .ok_or_else(|| Error("historical field current adapter absent".into()))?;
        let active = model
            .lexical_emission
            .as_ref()
            .ok_or_else(|| Error("historical field lexical router absent".into()))?;
        active.validate_shape(model)?;
        let choices: BTreeSet<_> = std::iter::once(BOS)
            .chain(self.tokens.iter().copied())
            .chain([field::OWNER_CHOICE, field::VALUE_CHOICE])
            .collect();
        let primes: BTreeSet<_> = field
            .dictionary
            .iter()
            .map(|d| u64::from(d.prime))
            .collect();
        let symbol = |x: u64| {
            x == 131072
                || x == u64::from(field::OWNER_CHOICE)
                || x == u64::from(field::VALUE_CHOICE)
                || self.tokens.iter().any(|t| u64::from(*t) == x)
        };
        let prefix = |p: u64| {
            p >> 54 == 0
                && symbol(p & 262143)
                && symbol((p >> 18) & 262143)
                && symbol((p >> 36) & 262143)
        };
        let feature = |f: ValueFeature| {
            if !historical_feature(f) {
                return false;
            }
            let a = f.a & 0x00ff_ffff_ffff_ffff;
            match f.kind {
                0 => prefix(a) && choices.contains(&(f.b as u32)) && f.b <= u64::from(u32::MAX),
                3 => {
                    a >> 50 == 0
                        && primes.contains(&(a >> 18))
                        && choices.contains(&((a & 262143) as u32))
                        && prefix(f.b)
                }
                _ => false,
            }
        };
        if model.historical_read.is_none()
            || !frozen(active, &self.previous_lexical)
            || self.tokens.is_empty()
            || self.tokens.len() > 64
            || !self.tokens.contains(&EOS)
            || self.tokens.windows(2).any(|w| w[0] >= w[1])
            || self
                .tokens
                .iter()
                .any(|t| *t != EOS && !(2..258).contains(t))
            || active.router.codes.len() > self.config.learned_features
            || self.config.mode != active.router.config.mode
            || self.config.role_context_only != active.router.config.role_context_only
            || active
                .router
                .codes
                .iter()
                .filter(|c| historical_feature(c.feature))
                .any(|c| !feature(c.feature))
            || self.training.is_empty()
            || self.training.len() > 2048
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
            return Err(Error(
                "invalid historical field witness or frozen parameters".into(),
            ));
        }
        let mut duplicate = model.clone();
        duplicate.refresh_identity()?;
        if duplicate.artifact_cid != model.artifact_cid
            || duplicate.uor_model_address != model.uor_model_address
        {
            return Err(Error("historical field artifact identity differs".into()));
        }
        Ok(())
    }
}

fn frame(
    model: &Model,
    tokens: &[u32],
    state: &FieldState,
    values: &ValueState,
    target: Option<u32>,
) -> Result<Frame> {
    let block = model
        .field_composition
        .as_ref()
        .ok_or_else(|| Error("historical field adapter absent".into()))?;
    Ok(Frame {
        alternatives: std::iter::once((BOS, 0))
            .chain(tokens.iter().copied().map(|t| (t, 1)))
            .chain([(field::OWNER_CHOICE, 1), (field::VALUE_CHOICE, 1)])
            .map(|(choice, action)| {
                let (f, n) = field::historical_features(block, state, values, choice, false);
                Alternative {
                    features: f[..n].to_vec(),
                    codes: vec![],
                    action,
                    correct: target.map_or(action == 0, |t| action == 1 && choice == t),
                }
            })
            .collect(),
    })
}

impl Model {
    pub fn without_historical_field_composition(&self) -> Result<Model> {
        match &self.historical_field_composition {
            Some(w) => w.parent(self),
            None => Ok(self.clone()),
        }
    }
    pub fn fit_historical_field_composition(
        &self,
        docs: &[FieldCompositionExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        config.validate_word_emission()?;
        let start = Instant::now();
        if self.historical_field_composition.is_some()
            || self.historical_read.is_none()
            || docs.is_empty()
            || docs.len() > 2048
            || docs
                .iter()
                .any(|d| d.prompt.is_empty() || d.prompt.len() > 65536)
            || docs.iter().map(|d| d.prompt.len()).sum::<usize>() > 8 * 1024 * 1024
        {
            return Err(Error("invalid historical field fit population".into()));
        }
        let old = self
            .lexical_emission
            .as_ref()
            .ok_or_else(|| Error("historical field lexical parent absent".into()))?;
        let current = self
            .field_composition
            .as_ref()
            .ok_or_else(|| Error("historical field current adapter absent".into()))?;
        if config.mode != old.router.config.mode
            || config.role_context_only != old.router.config.role_context_only
        {
            return Err(Error("historical field scoring mode differs".into()));
        }
        let mut tokens: BTreeSet<_> = current.tokens.iter().copied().collect();
        let mut ids = BTreeSet::new();
        let mut training = Vec::new();
        for d in docs {
            if d.id.trim().is_empty()
                || !ids.insert(&d.id)
                || (!d.inherit && (d.owner.is_empty() || d.value.is_empty() || d.pieces.is_empty()))
            {
                return Err(Error(format!("invalid historical field example {}", d.id)));
            }
            for p in &d.pieces {
                if let FieldPiece::Bytes(s) = p {
                    if !s.is_ascii() || s.len() > 32 {
                        return Err(Error("historical field literal byte bound".into()));
                    }
                    tokens.extend(s.bytes().map(|b| u32::from(b) + 2));
                }
            }
            let bytes = serde_json::to_vec(d).map_err(|e| Error(e.to_string()))?;
            training.push(DocumentReceipt {
                id: d.id.clone(),
                bytes: bytes.len(),
                text_cid: format!("blake3:{}", blake3::hash(&bytes).to_hex()),
            });
        }
        if tokens.len() > 64 {
            return Err(Error("historical field token cap64".into()));
        }
        let tokens: Vec<_> = tokens.into_iter().collect();
        let mut frames = Vec::new();
        let mut selected = Vec::new();
        let mut skips = Vec::new();
        let mut inherited = 0;
        let mut presented = 0;
        let mut signatures = BTreeMap::new();
        let mut add = |f: Frame, id: &str| -> Result<()> {
            presented += 1;
            if presented > 32768 {
                return Err(Error("historical field frame cap32768".into()));
            }
            let signature: Vec<_> = f
                .alternatives
                .iter()
                .map(|a| (a.features.clone(), a.action))
                .collect();
            let target = f
                .alternatives
                .iter()
                .position(|a| a.correct)
                .ok_or_else(|| Error("historical field target absent".into()))?;
            if let Some((old_target, old_id)) = signatures.get(&signature) {
                if *old_target != target {
                    return Err(Error(format!(
                        "historical field contradictory frames: {old_id} and {id}"
                    )));
                }
            } else {
                signatures.insert(signature, (target, id.to_owned()));
                frames.push(f);
            }
            Ok(())
        };
        for d in docs {
            if start.elapsed().as_secs() >= config.max_seconds {
                return Err(Error("historical field preparation time limit".into()));
            }
            let mut s = self.session(Control::Full)?;
            s.observe(self, BOS)?;
            let input = self.encode(&d.prompt)?;
            if !d.inherit && (d.prompt.len() > 4096 || input.len() > 512) {
                return Err(Error(format!(
                    "historical field target input cap512/4096: {}",
                    d.id
                )));
            }
            if input.len() > 8192 {
                return Err(Error("historical field inherited token cap8192".into()));
            }
            for token in input {
                s.observe(self, token)?;
            }
            s.begin_response(self)?;
            let p = s.predict(self)?;
            let baseline = Candidate {
                token: p.token,
                score: p.score,
            };
            let mut state = FieldState::default();
            let anchor = initial_anchor(
                self,
                s.word_copy
                    .as_ref()
                    .ok_or_else(|| Error("historical field copy absent".into()))?,
                s.response_entry
                    .as_ref()
                    .ok_or_else(|| Error("historical field entry absent".into()))?,
                s.values
                    .as_ref()
                    .ok_or_else(|| Error("historical field values absent".into()))?,
                baseline,
                Control::Full,
                &mut s.work.word_copy,
            );
            if d.inherit {
                if anchor.is_some() {
                    add(
                        frame(
                            self,
                            &tokens,
                            &state,
                            s.values
                                .as_ref()
                                .ok_or_else(|| Error("historical field values absent".into()))?,
                            None,
                        )?,
                        &d.id,
                    )?;
                    inherited += 1;
                } else {
                    skips.push(d.id.clone());
                }
                continue;
            }
            let anchor = anchor.ok_or_else(|| {
                Error(format!(
                    "historical field exact selected old source unavailable: {}",
                    d.id
                ))
            })?;
            let values = s
                .values
                .as_ref()
                .ok_or_else(|| Error("historical field values absent".into()))?;
            let r = field::record(values, anchor, &mut s.work.word_copy)
                .ok_or_else(|| Error("historical field record invalid".into()))?;
            if &r.owner.bytes[..usize::from(r.owner.len)] != d.owner.as_bytes() {
                return Err(Error(format!("historical field owner differs: {}", d.id)));
            }
            let len = field::field_len(values, anchor, 2, &mut s.work.word_copy)
                .ok_or_else(|| Error("historical field value absent".into()))?;
            let value = (0..len)
                .map(|i| {
                    field::field_byte(values, anchor, 2, i, &mut s.work.word_copy)
                        .ok_or_else(|| Error("historical field value byte absent".into()))
                })
                .collect::<Result<Vec<_>>>()?;
            if value != d.value.as_bytes() {
                return Err(Error(format!(
                    "historical field complete value differs: {}",
                    d.id
                )));
            }
            selected.push(
                serde_json::json!({"id":d.id,"anchor":anchor,"owner":d.owner,"value":d.value}),
            );
            let mut labels = Vec::new();
            for piece in &d.pieces {
                match piece {
                    FieldPiece::Bytes(text) => {
                        labels.extend(text.bytes().map(|b| (u32::from(b) + 2, 0, 0)))
                    }
                    FieldPiece::Owner | FieldPiece::Value => {
                        let which = if matches!(piece, FieldPiece::Owner) {
                            1
                        } else {
                            2
                        };
                        let n = field::field_len(values, anchor, which, &mut s.work.word_copy)
                            .ok_or_else(|| Error("historical field label unavailable".into()))?;
                        for cursor in 0..n {
                            let byte = field::field_byte(
                                values,
                                anchor,
                                which,
                                cursor,
                                &mut s.work.word_copy,
                            )
                            .ok_or_else(|| Error("historical field label byte absent".into()))?;
                            labels.push((u32::from(byte) + 2, which, cursor));
                        }
                    }
                }
            }
            labels.push((EOS, 0, 0));
            if labels.len() > 32 {
                return Err(Error(format!(
                    "historical field response cap32: {} needs {}",
                    d.id,
                    labels.len()
                )));
            }
            for (token, which, cursor) in labels {
                let values = s
                    .values
                    .as_ref()
                    .ok_or_else(|| Error("historical field values absent".into()))?;
                if cursor == 0 {
                    let choice = match which {
                        1 => field::OWNER_CHOICE,
                        2 => field::VALUE_CHOICE,
                        _ => token,
                    };
                    add(frame(self, &tokens, &state, values, Some(choice))?, &d.id)?;
                }
                field::prepare(
                    &mut state,
                    s.word_copy
                        .as_mut()
                        .ok_or_else(|| Error("historical field copy absent".into()))?,
                    s.response_entry
                        .as_mut()
                        .ok_or_else(|| Error("historical field entry absent".into()))?,
                    values,
                    anchor,
                    token,
                    which,
                    cursor,
                    Candidate {
                        token: BOS,
                        score: 0,
                    },
                );
                s.observe(self, token)?;
                state.observe(
                    self,
                    s.response_entry
                        .as_mut()
                        .ok_or_else(|| Error("historical field entry absent".into()))?,
                    s.values
                        .as_ref()
                        .ok_or_else(|| Error("historical field values absent".into()))?,
                    token,
                    &mut s.work.word_copy,
                );
            }
        }
        drop(add);
        if frames.is_empty() || selected.is_empty() {
            return Err(Error("historical field has no target frames".into()));
        }
        let vocab: BTreeSet<_> = frames
            .iter()
            .flat_map(|f| {
                f.alternatives
                    .iter()
                    .flat_map(|a| a.features.iter().copied())
            })
            .collect();
        let mut active = old.clone();
        if old
            .router
            .codes
            .iter()
            .any(|c| historical_feature(c.feature))
        {
            return Err(Error("historical field namespace already occupied".into()));
        }
        for feature in vocab {
            if !historical_feature(feature) {
                return Err(Error("historical field namespace differs".into()));
            }
            active.router.codes.push(SourceCode {
                feature,
                roots: [self.geometry.identity; 2],
            });
        }
        active.router.codes.sort_by_key(|c| c.feature);
        if active.router.codes.len() > config.learned_features
            || active.router.codes.len() > old.router.config.learned_features
        {
            return Err(Error(format!(
                "historical field complete feature cap exceeded: {}",
                active.router.codes.len()
            )));
        }
        let mutable: Vec<_> = active
            .router
            .codes
            .iter()
            .map(|c| historical_feature(c.feature))
            .collect();
        active.router.config = config.clone();
        let fit = learn_code_subset(self, &mut active.router, &mut frames, start, &mutable)?;
        active.router.config = old.router.config.clone();
        let mut model = self.clone();
        model.lexical_emission = Some(active);
        model.historical_field_composition = Some(HistoricalFieldComposition {
            parent_artifact: self.artifact_cid.clone(),
            previous_lexical: old.clone(),
            tokens,
            config,
            training,
        });
        model.refresh_identity()?;
        model.validate()?;
        let report = serde_json::json!({"schema":"uor-r4.historical-field-fit/1","parent":self.artifact_cid(),"artifact":model.artifact_cid(),
            "documents":docs.len(),"presented_frames":presented,"frames":frames.len(),"inherited_initial_frames":inherited,
            "ineligible_inherited_documents":skips,"selected_relations":selected,"dictionary_inherited_exactly":true,
            "features":mutable.len(),"mutable_features":mutable.iter().filter(|x|**x).count(),"fit":fit,"elapsed_ms":start.elapsed().as_millis(),
            "scope":"Only tag18 shared lexical roots learned. Exact selected immediate previous source supplies fields; typed pieces offline only; no model free-generation qualification by fit."});
        Ok((model, report))
    }
}
