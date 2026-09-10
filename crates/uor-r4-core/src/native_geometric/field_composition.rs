//! Typed owner/value reads and lexical bytes share the retained geometric selector.
//! Offline pieces supply supervision only; no phrase or output plan is served.
use super::lexical_emission::LexicalEmission;
use super::response_entry_types::{ResponseEntryAction, ResponseEntryDecision, ResponseEntryState};
use super::source_routing::SourceCode;
use super::source_routing_training::{learn_code_subset, Alternative, Frame};
use super::value_types::{ValueFeature, ValueState};
use super::word_copy_types::{WordCopyAction, WordCopyAddress, WordCopyState, WordCopyWork};
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FieldComposition {
    pub parent_artifact: String,
    pub previous_lexical: LexicalEmission,
    pub dictionary: Vec<WordCopyAddress>,
    pub tokens: Vec<u32>,
}
impl FieldComposition {
    fn parent(&self, model: &Model) -> Result<Model> {
        let mut parent = model.clone();
        parent.field_composition = None;
        parent.lexical_emission = Some(self.previous_lexical.clone());
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("field composition frozen parent differs".into()));
        }
        parent.validate()?;
        Ok(parent)
    }
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        let current = model
            .lexical_emission
            .as_ref()
            .ok_or_else(|| Error("field composition lexical block missing".into()))?;
        current.validate_shape(model)?;
        if self.dictionary.is_empty()
            || self.dictionary.len() > 128
            || self.tokens.is_empty()
            || self.tokens.len() > 64
            || self.tokens.windows(2).any(|w| w[0] >= w[1])
            || self
                .tokens
                .iter()
                .any(|&t| t != EOS && !(2..258).contains(&t))
        {
            return Err(Error("invalid field composition admission".into()));
        }
        let primes = crate::corpus_induced_spin_placement::first_primes(self.dictionary.len())
            .map_err(|e| Error(e.to_string()))?;
        if self.dictionary.iter().zip(primes).any(|(w, p)| {
            w.len == 0
                || w.len > 32
                || u64::from(w.prime) != p
                || w.bytes[..usize::from(w.len)]
                    .iter()
                    .any(|b| !b.is_ascii_lowercase() && *b != b'_')
                || w.bytes[usize::from(w.len)..].iter().any(|&b| b != 0)
        }) || self.dictionary.windows(2).any(|w| w[0].bytes >= w[1].bytes)
        {
            return Err(Error("invalid field composition dictionary".into()));
        }
        let old = &self.previous_lexical;
        if current.tokens != old.tokens
            || current.dictionary != old.dictionary
            || current.router.landmarks != old.router.landmarks
            || current.router.biases != old.router.biases
            || current.router.ranks != old.router.ranks
            || current.router.config.mode != old.router.config.mode
            || current.router.config.role_context_only != old.router.config.role_context_only
            || current.router.parent_artifact != old.router.parent_artifact
            || old.router.codes.iter().any(|c| {
                current
                    .router
                    .codes
                    .binary_search_by_key(&c.feature, |r| r.feature)
                    .ok()
                    .is_none_or(|i| current.router.codes[i] != *c)
            })
            || current.router.codes.iter().any(|c| {
                old.router
                    .codes
                    .binary_search_by_key(&c.feature, |r| r.feature)
                    .is_err()
                    && !field_feature(c.feature)
            })
        {
            return Err(Error(
                "field composition inherited lexical parameters differ".into(),
            ));
        }
        self.parent(model)?;
        let mut identity = model.clone();
        identity.refresh_identity()?;
        if identity.artifact_cid != model.artifact_cid
            || identity.uor_model_address != model.uor_model_address
        {
            return Err(Error("field composition identity differs".into()));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldAnchor {
    pub source: u8,
    pub span_words: u8,
    pub relation_id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_revision: Option<u64>,
    /// Exact validated link count from `current_revision` to `relation_id`; None is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ancestor_depth: Option<u8>,
    /// The ancestor path contains a same-value reassertion link proven under the
    /// versioned chain contract. Absent means revision links only, the legacy form
    /// every revision-only path keeps; restore re-proves the path under exactly the
    /// stated contract.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reassertion_links: bool,
    pub source_end: u64,
    pub source_byte_end: u64,
    pub boundary_seen: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldRead {
    pub field: u8,
    pub cursor: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldDecision {
    pub token: u32,
    pub score: i64,
    pub at_seen: u64,
    pub step: u8,
    pub anchor: FieldAnchor,
    /// Zero is a learned lexical byte; one/two read the exact owner/value.
    pub field: u8,
    pub cursor: u8,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FieldState {
    pub anchor: Option<FieldAnchor>,
    pub read: Option<FieldRead>,
    pub last: u32,
    pub previous: u32,
    pub previous2: u32,
    pub steps: u8,
    pub seen: u64,
    #[serde(skip)]
    pub pending: Option<FieldDecision>,
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
const FIELD_TAG: u64 = 17_u64 << 56;
const PREFIX: u32 = 131072;
pub(super) const OWNER_CHOICE: u32 = 131073;
pub(super) const VALUE_CHOICE: u32 = 131074;
fn field_feature(f: ValueFeature) -> bool {
    matches!(f.kind, 0 | 3) && f.a >> 56 == 17
}
fn prefix_identity(previous2: u32, previous: u32, last: u32) -> u64 {
    (u64::from(previous2) << 36) | (u64::from(previous) << 18) | u64::from(last)
}

fn enabled(control: Control) -> bool {
    !matches!(
        control,
        Control::FieldCompositionDisabled
            | Control::WordCopyDisabled
            | Control::LexicalEmissionDisabled
            | Control::GeometryDisabled
            | Control::H4Disabled
            | Control::LearnedRoutingDisabled
            | Control::ResponseEntryDisabled
            | Control::ValuesDisabled
            | Control::MemoryDisabled
    )
}
pub(super) fn record<'a>(
    values: &'a ValueState,
    anchor: FieldAnchor,
    work: &mut WordCopyWork,
) -> Option<&'a super::relation::RelationRecord> {
    let relations = values.relations.as_ref()?;
    let r = relations.record(anchor.relation_id)?;
    if let Some(id) = anchor.current_revision {
        work.persistent_read.relations.record_reads += 2;
        let current = relations.record(id)?;
        // None is the canonical one-link proof; explicit depths start at two.
        let depth = match anchor.ancestor_depth {
            None => 1,
            Some(depth) if depth >= 2 => depth,
            Some(_) => return None,
        };
        if super::historical_read::ancestor(
            relations,
            current,
            depth,
            anchor.reassertion_links,
            &mut work.persistent_read,
        )?
        .id != anchor.relation_id
            || anchor.source != super::relation::RELATION_SOURCE + ((r.id - 1) & 15) as u8
        {
            return None;
        }
    } else if anchor.ancestor_depth.is_some()
        || anchor.reassertion_links
        || !relations.directory.contains(&anchor.relation_id)
    {
        return None;
    }
    let source = super::relation::source(values, anchor.source)?;
    if r.conflict
        || source.end != anchor.source_end
        || source.byte_end != anchor.source_byte_end
        || *source != r.value
    {
        return None;
    }
    Some(r)
}
pub(super) fn initial_anchor(
    copy: &WordCopyState,
    entry: &ResponseEntryState,
    values: &ValueState,
    baseline: Candidate,
    work: &mut WordCopyWork,
) -> Option<FieldAnchor> {
    if entry.steps != 0 || entry.active || values.pending.is_some() {
        return None;
    }
    let decision = copy.pending?;
    if !matches!(
        decision.action,
        WordCopyAction::Read | WordCopyAction::Prepare
    ) || decision.dependency.is_some()
        || decision.token != baseline.token
        || decision.score != baseline.score
    {
        return None;
    }
    let source = super::relation::source(values, decision.word_index)?;
    let relations = values.relations.as_ref()?;
    let mut selected = None;
    for &id in &relations.directory {
        let Some(r) = relations.record(id) else {
            continue;
        };
        work.word_record_reads = work.word_record_reads.saturating_add(1);
        if !r.conflict && r.value == *source {
            if selected.is_some() {
                return None;
            }
            selected = Some(r);
        }
    }
    let r = selected?;
    let anchor = FieldAnchor {
        source: decision.word_index,
        span_words: decision.span_words,
        relation_id: r.id,
        current_revision: None,
        ancestor_depth: None,
        reassertion_links: false,
        source_end: source.end,
        source_byte_end: source.byte_end,
        boundary_seen: entry.boundary?.at_seen,
    };
    let length = super::source_span::len(values, anchor.source, anchor.span_words, work)?;
    let expected = r.span.as_ref().map_or(r.value.len, |s| s.len);
    if length != expected {
        return None;
    }
    for i in 0..length {
        let expected = r
            .span
            .as_ref()
            .map_or(r.value.bytes[usize::from(i)], |s| s.bytes[usize::from(i)]);
        if super::source_span::byte(values, anchor.source, anchor.span_words, i, work)? != expected
        {
            return None;
        }
    }
    Some(anchor)
}
pub(super) fn field_len(
    values: &ValueState,
    anchor: FieldAnchor,
    field: u8,
    work: &mut WordCopyWork,
) -> Option<u8> {
    let r = record(values, anchor, work)?;
    match field {
        1 => Some(r.owner.len),
        2 => super::source_span::len(values, anchor.source, anchor.span_words, work),
        _ => None,
    }
}
pub(super) fn field_byte(
    values: &ValueState,
    anchor: FieldAnchor,
    field: u8,
    cursor: u8,
    work: &mut WordCopyWork,
) -> Option<u8> {
    let r = record(values, anchor, work)?;
    match field {
        1 => (cursor < r.owner.len).then(|| r.owner.bytes[usize::from(cursor)]),
        2 => super::source_span::byte(values, anchor.source, anchor.span_words, cursor, work),
        _ => None,
    }
}
fn features(
    block: &FieldComposition,
    state: &FieldState,
    values: &ValueState,
    choice: u32,
    erase: bool,
) -> ([ValueFeature; 40], usize) {
    features_tag(block, state, values, choice, erase, FIELD_TAG)
}
pub(super) fn historical_features(
    block: &FieldComposition,
    state: &FieldState,
    values: &ValueState,
    choice: u32,
    erase: bool,
) -> ([ValueFeature; 40], usize) {
    features_tag(block, state, values, choice, erase, 18_u64 << 56)
}
fn features_tag(
    block: &FieldComposition,
    state: &FieldState,
    values: &ValueState,
    choice: u32,
    erase: bool,
    tag: u64,
) -> ([ValueFeature; 40], usize) {
    let mut f = [ValueFeature::default(); 40];
    if erase {
        return (f, 0);
    }
    let (last, previous, previous2) = if state.anchor.is_none() {
        (PREFIX, PREFIX, PREFIX)
    } else {
        (state.last, state.previous, state.previous2)
    };
    let prefix = prefix_identity(previous2, previous, last);
    f[0] = ValueFeature {
        kind: 0,
        a: tag | prefix,
        b: u64::from(choice),
    };
    let mut n = 1;
    if let Some(words) = &values.lexemes {
        for word in &words.queries[..words.query_len] {
            if values.query_boundary.is_some_and(|start| word.end < start) {
                continue;
            }
            if let Some(d) = block.dictionary.iter().find(|d| {
                d.len == word.len
                    && d.bytes[..usize::from(d.len)]
                        .iter()
                        .zip(word.bytes)
                        .all(|(a, b)| *a == b.to_ascii_lowercase())
            }) {
                f[n] = ValueFeature {
                    kind: 3,
                    // Prime32 and choice18 occupy50 bits below the type tag.
                    // The other word retains all three18-bit prefix symbols.
                    a: tag | (u64::from(d.prime) << 18) | u64::from(choice),
                    b: prefix,
                };
                n += 1;
            }
        }
    }
    (f, n)
}
pub(super) fn prepare(
    state: &mut FieldState,
    copy: &mut WordCopyState,
    entry: &mut ResponseEntryState,
    values: &ValueState,
    anchor: FieldAnchor,
    token: u32,
    field: u8,
    cursor: u8,
    baseline: Candidate,
) -> Option<Candidate> {
    let score = baseline.score.saturating_add(1);
    state.pending = Some(FieldDecision {
        token,
        score,
        at_seen: values.seen,
        step: entry.steps,
        anchor,
        field,
        cursor,
    });
    copy.pending = None;
    entry.pending = Some(ResponseEntryDecision {
        token,
        score,
        boundary_seen: anchor.boundary_seen,
        step: entry.steps,
        at_seen: values.seen,
        action: if token == EOS {
            ResponseEntryAction::Stop
        } else if entry.active {
            ResponseEntryAction::Emit
        } else {
            ResponseEntryAction::Enter
        },
    });
    Some(Candidate { token, score })
}
pub(super) fn offer(
    model: &Model,
    state: &mut FieldState,
    copy: &mut WordCopyState,
    entry: &mut ResponseEntryState,
    values: &ValueState,
    baseline: Candidate,
    control: Control,
    work: &mut WordCopyWork,
) -> Option<Candidate> {
    state.pending = None;
    if !enabled(control)
        || !super::response_entry_runtime::eligible(model, values, control)
        || values.pending.is_some()
        || entry.steps >= 32
        || entry.seen != values.seen
    {
        return None;
    }
    let block = model.field_composition.as_ref()?;
    let anchor = if let Some(anchor) = state.anchor {
        if !entry.active
            || state.steps != entry.steps
            || state.seen != values.seen
            || entry.boundary?.at_seen != anchor.boundary_seen
        {
            return None;
        }
        if anchor.current_revision.is_some()
            && (model.historical_field_composition.is_none()
                || control == Control::HistoricalFieldCompositionDisabled)
        {
            return None;
        }
        if anchor.ancestor_depth.is_some()
            && (model.historical_version_intent.is_none()
                || control == Control::HistoricalVersionIntentDisabled)
        {
            return None;
        }
        // A path proven under the versioned chain contract stays valid only while
        // that contract is active for this model and control.
        if anchor.reassertion_links
            && !super::historical_version_intent::reassertion_links(model, control)
        {
            return None;
        }
        record(values, anchor, work)?;
        anchor
    } else {
        initial_anchor(copy, entry, values, baseline, work).or_else(|| {
            if model.historical_field_composition.is_none()
                || control == Control::HistoricalFieldCompositionDisabled
            {
                return None;
            }
            super::historical_field_composition::initial_anchor(
                model, copy, entry, values, baseline, control, work,
            )
        })?
    };
    let historical = anchor.current_revision.is_some();
    let tokens = if historical {
        &model.historical_field_composition.as_ref()?.tokens
    } else {
        &block.tokens
    };
    if let Some(read) = state.read {
        if control == Control::FieldCompositionReadDisabled {
            return None;
        }
        let token = u32::from(field_byte(values, anchor, read.field, read.cursor, work)?) + 2;
        work.byte_reads = work.byte_reads.saturating_add(1);
        return prepare(
            state,
            copy,
            entry,
            values,
            anchor,
            token,
            read.field,
            read.cursor,
            baseline,
        );
    }
    let router = &model.lexical_emission.as_ref()?.router;
    let geometry = if control == Control::FieldCompositionGeometryDisabled {
        Control::LearnedRoutingTransformDisabled
    } else {
        control
    };
    let erased = control == Control::FieldCompositionContextDisabled;
    let (mut best, token) = super::lexical_emission::token_choice(
        model,
        router,
        tokens,
        |t| {
            if historical {
                historical_features(block, state, values, t, erased)
            } else {
                features(block, state, values, t, erased)
            }
        },
        geometry,
        &mut work.routing,
        &mut work.selector,
    );
    let mut selected = token.map(|t| (t, 0));
    if control != Control::FieldCompositionReadDisabled {
        for (choice, field) in [(OWNER_CHOICE, 1), (VALUE_CHOICE, 2)] {
            let Some(length) = field_len(values, anchor, field, work) else {
                continue;
            };
            if entry.steps.saturating_add(length) >= 32 {
                continue;
            }
            let (f, n) = if historical {
                historical_features(block, state, values, choice, erased)
            } else {
                features(block, state, values, choice, erased)
            };
            let encoded = router.encode(model, &f[..n], geometry, &mut work.routing);
            let score = router.score(model, encoded, 1, &mut work.routing);
            work.selector.candidate_evaluations =
                work.selector.candidate_evaluations.saturating_add(1);
            if score > best {
                best = score;
                selected = Some((
                    u32::from(field_byte(values, anchor, field, 0, work)?) + 2,
                    field,
                ));
            }
        }
    }
    let (token, field) = selected?;
    prepare(
        state, copy, entry, values, anchor, token, field, 0, baseline,
    )
}
impl FieldState {
    pub(super) fn selected(&mut self, best: Candidate) {
        if self
            .pending
            .is_some_and(|d| d.token != best.token || d.score != best.score)
        {
            self.pending = None;
        }
    }
    pub(super) fn observe(
        &mut self,
        _model: &Model,
        entry: &mut ResponseEntryState,
        values: &ValueState,
        token: u32,
        work: &mut WordCopyWork,
    ) {
        let prior_active = self.anchor.is_some();
        let pending = self.pending.take();
        if token == EOS || !entry.active {
            *self = Self::default();
            return;
        }
        let matched = pending.filter(|d| {
            d.token == token
                && d.at_seen.checked_add(1) == Some(values.seen)
                && d.step.checked_add(1) == Some(entry.steps)
                && record(values, d.anchor, work).is_some()
        });
        let Some(d) = matched else {
            if prior_active {
                entry.reset();
            }
            *self = Self::default();
            return;
        };
        if self.anchor.is_none() {
            self.last = PREFIX;
            self.previous = PREFIX;
            self.previous2 = PREFIX;
        }
        self.anchor = Some(d.anchor);
        self.steps = entry.steps;
        self.seen = values.seen;
        if d.field == 0 {
            self.previous2 = self.previous;
            self.previous = self.last;
            self.last = token;
            self.read = None;
        } else {
            let cursor = d.cursor.saturating_add(1);
            let Some(length) = field_len(values, d.anchor, d.field, work) else {
                entry.reset();
                *self = Self::default();
                return;
            };
            if cursor == length {
                self.previous2 = PREFIX;
                self.previous = PREFIX;
                self.last = PREFIX + u32::from(d.field);
                self.read = None;
            } else if cursor < length {
                self.read = Some(FieldRead {
                    field: d.field,
                    cursor,
                });
            } else {
                entry.reset();
                *self = Self::default();
            }
        }
    }
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "content",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum FieldPiece {
    Owner,
    Value,
    Bytes(String),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldCompositionExample {
    pub id: String,
    pub prompt: String,
    pub owner: String,
    pub value: String,
    pub pieces: Vec<FieldPiece>,
    pub inherit: bool,
}
fn append_frame(
    block: &FieldComposition,
    state: &FieldState,
    values: &ValueState,
    target: Option<u32>,
    frames: &mut Vec<Frame>,
) -> Result<()> {
    if frames.len() >= 4096 {
        return Err(Error("field composition frame cap4096".into()));
    }
    let alternatives = std::iter::once((BOS, 0))
        .chain(block.tokens.iter().copied().map(|t| (t, 1)))
        .chain([(OWNER_CHOICE, 1), (VALUE_CHOICE, 1)])
        .map(|(choice, action)| {
            let (f, n) = features(block, state, values, choice, false);
            Alternative {
                features: f[..n].to_vec(),
                codes: vec![],
                action,
                correct: target.map_or(action == 0, |t| action == 1 && choice == t),
            }
        })
        .collect();
    frames.push(Frame { alternatives });
    Ok(())
}
// Offline document-presence association over explicitly paired design rows.
// A lexical identity occurring at the same class-normalized frequency carries
// no request distinction. Payload words receive no special exclusion rule.
fn associated_query_words(
    classes: [usize; 2],
    presence: &BTreeMap<String, [usize; 2]>,
) -> BTreeSet<String> {
    let total = classes[0] + classes[1];
    presence
        .iter()
        .filter_map(|(word, counts)| {
            let count = counts[0] + counts[1];
            (counts[0] * total != count * classes[0]).then(|| word.clone())
        })
        .collect()
}
impl Model {
    pub fn without_field_composition(&self) -> Result<Model> {
        self.field_composition
            .as_ref()
            .ok_or_else(|| Error("field composition absent".into()))?
            .parent(self)
    }
    pub fn fit_field_composition(
        &self,
        docs: &[FieldCompositionExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        config.validate_word_emission()?;
        self.validate()?;
        if self.field_composition.is_some() || docs.is_empty() || docs.len() > 1024 {
            return Err(Error("invalid field composition parent/population".into()));
        }
        let old = self
            .lexical_emission
            .as_ref()
            .ok_or_else(|| Error("field composition lexical parent absent".into()))?;
        if config.mode != old.router.config.mode
            || config.role_context_only != old.router.config.role_context_only
        {
            return Err(Error("field composition scoring mode differs".into()));
        }
        let mut ids = BTreeSet::new();
        let mut presence = BTreeMap::<String, usize>::new();
        let mut design_presence = BTreeMap::<String, [usize; 2]>::new();
        let mut design_classes = [0_usize; 2];
        let mut design_ids = Vec::new();
        let mut tokens = BTreeSet::from([EOS]);
        let mut receipts = Vec::new();
        for d in docs {
            if d.id.is_empty()
                || !ids.insert(d.id.clone())
                || d.prompt.is_empty()
                || d.prompt.len() > 4096
                || d.owner.is_empty() != d.value.is_empty()
                || (!d.inherit && (d.owner.is_empty() || d.value.is_empty() || d.pieces.is_empty()))
            {
                return Err(Error(format!(
                    "invalid field composition document: {}",
                    d.id
                )));
            }
            let present: BTreeSet<_> = d
                .prompt
                .split(|c: char| !c.is_ascii_alphabetic() && c != '_')
                .filter(|w| !w.is_empty() && w.len() <= 32)
                .map(str::to_ascii_lowercase)
                .collect();
            let design = !d.owner.is_empty();
            if design {
                design_classes[usize::from(d.inherit)] += 1;
                design_ids.push(d.id.clone());
            }
            for w in present {
                if design {
                    design_presence.entry(w.clone()).or_default()[usize::from(d.inherit)] += 1;
                }
                *presence.entry(w).or_default() += 1;
            }
            for p in &d.pieces {
                if let FieldPiece::Bytes(s) = p {
                    if !s.is_ascii() {
                        return Err(Error("field composition bytes must be ASCII".into()));
                    }
                    tokens.extend(s.bytes().map(|b| u32::from(b) + 2));
                }
            }
            let bytes = serde_json::to_vec(d).map_err(|e| Error(e.to_string()))?;
            receipts.push(DocumentReceipt {
                id: d.id.clone(),
                bytes: bytes.len(),
                text_cid: format!("blake3:{}", blake3::hash(&bytes).to_hex()),
            });
        }
        if design_classes.contains(&0) {
            return Err(Error(
                "field composition requires paired composition/Base vocabulary design".into(),
            ));
        }
        let words = associated_query_words(design_classes, &design_presence);
        if words.is_empty() || words.len() > 128 {
            return Err(Error(format!(
                "field composition associated dictionary cap128: {}",
                words.len()
            )));
        }
        let primes = crate::corpus_induced_spin_placement::first_primes(words.len())
            .map_err(|e| Error(e.to_string()))?;
        let dictionary = words
            .into_iter()
            .zip(primes)
            .map(|(w, p)| {
                let mut bytes = [0; 32];
                bytes[..w.len()].copy_from_slice(w.as_bytes());
                WordCopyAddress {
                    bytes,
                    len: w.len() as u8,
                    prime: p as u32,
                }
            })
            .collect();
        if tokens.len() > 64 {
            return Err(Error("field composition token cap64".into()));
        }
        let witness = FieldComposition {
            parent_artifact: self.artifact_cid().into(),
            previous_lexical: old.clone(),
            dictionary,
            tokens: tokens.into_iter().collect(),
        };
        let mut frames = Vec::new();
        let mut selected = Vec::new();
        let mut inherited = 0;
        let mut unavailable = 0;
        for d in docs {
            let mut s = self.session(Control::Full)?;
            s.observe(self, BOS)?;
            for t in self.encode(&d.prompt)? {
                s.observe(self, t)?;
            }
            s.begin_response(self)?;
            let prediction = s.predict(self)?;
            let baseline = Candidate {
                token: prediction.token,
                score: prediction.score,
            };
            let mut state = FieldState::default();
            let anchor = initial_anchor(
                s.word_copy
                    .as_ref()
                    .ok_or_else(|| Error("field copy absent".into()))?,
                s.response_entry
                    .as_ref()
                    .ok_or_else(|| Error("field entry absent".into()))?,
                s.values
                    .as_ref()
                    .ok_or_else(|| Error("field values absent".into()))?,
                baseline,
                &mut s.work.word_copy,
            );
            if d.inherit {
                if anchor.is_some() {
                    append_frame(
                        &witness,
                        &state,
                        s.values
                            .as_ref()
                            .ok_or_else(|| Error("field values absent".into()))?,
                        None,
                        &mut frames,
                    )?;
                    inherited += 1;
                } else {
                    unavailable += 1;
                }
                continue;
            }
            let anchor = anchor.ok_or_else(|| {
                Error(format!(
                    "field selected current relation unavailable: {}",
                    d.id
                ))
            })?;
            let values = s
                .values
                .as_ref()
                .ok_or_else(|| Error("field values absent".into()))?;
            let r = record(values, anchor, &mut s.work.word_copy)
                .ok_or_else(|| Error("field record absent".into()))?;
            if &r.owner.bytes[..usize::from(r.owner.len)] != d.owner.as_bytes() {
                return Err(Error(format!("field selected owner differs: {}", d.id)));
            }
            let mut actual_value = Vec::new();
            let length = field_len(values, anchor, 2, &mut s.work.word_copy)
                .ok_or_else(|| Error("field span absent".into()))?;
            for i in 0..length {
                actual_value.push(
                    field_byte(values, anchor, 2, i, &mut s.work.word_copy)
                        .ok_or_else(|| Error("field span byte absent".into()))?,
                );
            }
            if actual_value != d.value.as_bytes() {
                return Err(Error(format!("field selected value differs: {}", d.id)));
            }
            selected.push(serde_json::json!({"id":d.id,"anchor":anchor,"owner":d.owner,"value":d.value,"source_selected_by_frozen_parent":true}));
            let mut labels = Vec::new();
            for piece in &d.pieces {
                match piece {
                    FieldPiece::Bytes(text) => {
                        labels.extend(text.bytes().map(|b| (u32::from(b) + 2, 0, 0)))
                    }
                    FieldPiece::Owner | FieldPiece::Value => {
                        let field = if matches!(piece, FieldPiece::Owner) {
                            1
                        } else {
                            2
                        };
                        let values = s
                            .values
                            .as_ref()
                            .ok_or_else(|| Error("field values absent".into()))?;
                        let len = field_len(values, anchor, field, &mut s.work.word_copy)
                            .ok_or_else(|| Error("field label read absent".into()))?;
                        for cursor in 0..len {
                            labels.push((
                                u32::from(
                                    field_byte(
                                        values,
                                        anchor,
                                        field,
                                        cursor,
                                        &mut s.work.word_copy,
                                    )
                                    .ok_or_else(|| Error("field label byte absent".into()))?,
                                ) + 2,
                                field,
                                cursor,
                            ));
                        }
                    }
                }
            }
            labels.push((EOS, 0, 0));
            if labels.len() > 32 {
                return Err(Error(format!(
                    "field output cap32: {} needs{}",
                    d.id,
                    labels.len()
                )));
            }
            for (token, field, cursor) in labels {
                let values = s
                    .values
                    .as_ref()
                    .ok_or_else(|| Error("field values absent".into()))?;
                if cursor == 0 {
                    let choice = match field {
                        1 => OWNER_CHOICE,
                        2 => VALUE_CHOICE,
                        _ => token,
                    };
                    append_frame(&witness, &state, values, Some(choice), &mut frames)?;
                }
                prepare(
                    &mut state,
                    s.word_copy
                        .as_mut()
                        .ok_or_else(|| Error("field copy absent".into()))?,
                    s.response_entry
                        .as_mut()
                        .ok_or_else(|| Error("field entry absent".into()))?,
                    values,
                    anchor,
                    token,
                    field,
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
                        .ok_or_else(|| Error("field entry absent".into()))?,
                    s.values
                        .as_ref()
                        .ok_or_else(|| Error("field values absent".into()))?,
                    token,
                    &mut s.work.word_copy,
                );
            }
        }
        let presented = frames.len();
        let mut unique = BTreeSet::new();
        frames.retain(|f| {
            unique.insert(
                f.alternatives
                    .iter()
                    .map(|a| (a.features.clone(), a.action, a.correct))
                    .collect::<Vec<_>>(),
            )
        });
        let conflicts = super::action_emission::operation_frame_conflicts(&frames)?;
        if conflicts["count"].as_u64().unwrap_or(0) > 0 {
            return Err(Error(format!(
                "field composition contradictory construction frames: {conflicts}"
            )));
        }
        let vocab: BTreeSet<_> = frames
            .iter()
            .flat_map(|f| {
                f.alternatives
                    .iter()
                    .flat_map(|a| a.features.iter().copied())
            })
            .collect();
        let mut block = old.clone();
        for feature in vocab {
            if old
                .router
                .codes
                .binary_search_by_key(&feature, |c| c.feature)
                .is_err()
            {
                block.router.codes.push(SourceCode {
                    feature,
                    roots: [self.geometry.identity; 2],
                });
            }
        }
        block.router.codes.sort_by_key(|c| c.feature);
        if block.router.codes.len() > config.learned_features {
            return Err(Error(format!(
                "field composition complete feature cap exceeded: {} required",
                block.router.codes.len()
            )));
        }
        let mutable: Vec<_> = block
            .router
            .codes
            .iter()
            .map(|c| {
                old.router
                    .codes
                    .binary_search_by_key(&c.feature, |r| r.feature)
                    .is_err()
            })
            .collect();
        block.router.config = config;
        block.router.training = receipts;
        let start = Instant::now();
        let fit = learn_code_subset(self, &mut block.router, &mut frames, start, &mutable)?;
        let elapsed = start.elapsed().as_millis();
        let failures =
            super::action_emission::operation_misclassified_frames(self, &block.router, &frames);
        let mut model = self.clone();
        model.lexical_emission = Some(block);
        model.field_composition = Some(witness);
        model.refresh_identity()?;
        model.validate()?;
        let report = serde_json::json!({"parent_artifact":self.artifact_cid(),"artifact":model.artifact_cid(),"fit":fit,"fit_elapsed_ms":elapsed,
            "frames":frames.len(),"presented_frames":presented,"inherited_initial_frames":inherited,"ineligible_inherited_documents":unavailable,
            "misclassified_frames":failures,"construction_frame_conflicts":conflicts,"selected_relations":selected,"features":mutable.len(),
            "mutable_features":mutable.iter().filter(|x|**x).count(),"vocabulary_presence":presence,
            "vocabulary_selection":"exact document-presence association over declared paired owner/value metadata rows; class0 composition, class1 Base",
            "vocabulary_design_ids":design_ids,"vocabulary_design_classes":design_classes,
            "vocabulary_design_population":design_classes[0]+design_classes[1],"vocabulary_design_presence":design_presence,
            "all_document_receipts":model.lexical_emission.as_ref().map(|b|&b.router.training),
            "dictionary":model.field_composition.as_ref().map(|b|b.dictionary.iter().map(|d|String::from_utf8_lossy(&d.bytes[..usize::from(d.len)]).into_owned()).collect::<Vec<_>>()),
            "all_parent_roots_landmarks_biases_ranks_frozen":true,"shared_lexical_router":true,"supervision":"typed pieces offline only; source selection from frozen parent; emission teacher forced","free_generation":"NOT_RUN by fit","response_step_cap":32});
        Ok((model, report))
    }
}

#[cfg(test)]
mod tests {
    use super::super::relation::{RelationRecord, RelationState, RELATION_SOURCE};
    use super::super::response_entry_types::ResponseEntryAnchor;
    use super::super::value_lexemes::{LexemeState, WordAtom};
    use super::super::value_types::{ValueEntry, QUERY};
    use super::super::word_copy_types::WordCopyDecision;
    use super::*;
    fn fixture() -> (ValueState, WordCopyState, ResponseEntryState, Candidate) {
        let mut value = WordAtom {
            len: 4,
            end: 20,
            byte_end: 20,
            ..Default::default()
        };
        value.bytes[..4].copy_from_slice(b"Dusk");
        let mut owner = WordAtom {
            len: 5,
            end: 10,
            byte_end: 10,
            ..Default::default()
        };
        owner.bytes[..5].copy_from_slice(b"selvi");
        let mut relations = RelationState::default();
        relations.records[0] = RelationRecord {
            id: 1,
            owner,
            value,
            ..Default::default()
        };
        relations.directory[0] = 1;
        relations.next_id = 2;
        let mut lexemes = LexemeState::default();
        lexemes.queries[0] = value;
        lexemes.query_len = 1;
        let values = ValueState {
            relations: Some(relations),
            scanner: Default::default(),
            lexemes: Some(lexemes),
            recent: [ValueEntry::default(); 32],
            recent_len: 0,
            recent_cursor: 0,
            records: vec![],
            sources: vec![],
            next_id: 0,
            seen: 40,
            pose: 0,
            phases: [0; PHASE_CHANNELS],
            active: true,
            consumed: false,
            operations_committed: 0,
            max_operations: 2,
            started_at: 40,
            query_boundary: None,
            queries: [ValueEntry::default(); QUERY],
            query_len: 0,
            emission: None,
            pending: None,
        };
        let copy = WordCopyState {
            pending: Some(WordCopyDecision {
                span_words: 0,
                dependency: None,
                token: 34,
                score: 1,
                word_index: 0,
                cursor: 0,
                source_end: 20,
                source_byte_end: 20,
                at_seen: 40,
                step: 0,
                action: WordCopyAction::Prepare,
            }),
            ..Default::default()
        };
        let entry = ResponseEntryState {
            boundary: Some(ResponseEntryAnchor {
                at_seen: 40,
                pose: 0,
                phases: [0; PHASE_CHANNELS],
                query_prime: 2,
            }),
            seen: 40,
            ..Default::default()
        };
        (
            values,
            copy,
            entry,
            Candidate {
                token: 34,
                score: 1,
            },
        )
    }
    #[test]
    fn native_historical_field_proof_is_optional_and_revalidated_with_work() {
        let (mut values, copy, entry, base) = fixture();
        let mut work = WordCopyWork::default();
        let mut anchor = initial_anchor(&copy, &entry, &values, base, &mut work).unwrap();
        let legacy = serde_json::to_value(anchor).unwrap();
        assert!(legacy.get("current_revision").is_none());
        assert_eq!(
            serde_json::from_value::<FieldAnchor>(legacy).unwrap(),
            anchor
        );
        let relations = values.relations.as_mut().unwrap();
        let mut current = relations.records[0];
        current.id = 2;
        current.previous = 1;
        current.action = 2;
        current.value.end = 30;
        current.value.byte_end = 30;
        relations.records[1] = current;
        relations.directory[0] = 2;
        relations.next_id = 3;
        assert!(record(&values, anchor, &mut work).is_none());
        anchor.source = RELATION_SOURCE;
        anchor.current_revision = Some(2);
        let before = work.persistent_read.relations.directory_reads;
        assert_eq!(record(&values, anchor, &mut work).unwrap().id, 1);
        assert!(work.persistent_read.relations.directory_reads > before);
        assert_eq!(field_byte(&values, anchor, 1, 0, &mut work), Some(b's'));
        assert_eq!(field_byte(&values, anchor, 2, 0, &mut work), Some(b'D'));
        let mut wrong = anchor;
        wrong.current_revision = Some(1);
        assert!(record(&values, wrong, &mut work).is_none());
        let mut wrong = anchor;
        wrong.current_revision = None;
        assert!(record(&values, wrong, &mut work).is_none());
        let mut wrong = anchor;
        wrong.source = 0;
        assert!(record(&values, wrong, &mut work).is_none());
        values.relations.as_mut().unwrap().records[1].previous = 0;
        assert!(record(&values, anchor, &mut work).is_none());
        values.relations.as_mut().unwrap().records[1].previous = 1;
        values.relations.as_mut().unwrap().records[1].conflict = true;
        assert!(record(&values, anchor, &mut work).is_none());
        values.relations.as_mut().unwrap().records[1].conflict = false;
        values.relations.as_mut().unwrap().records[1].owner.bytes[0] = b'x';
        assert!(record(&values, anchor, &mut work).is_none());
    }
    #[test]
    fn native_field_composition_binds_occurrence_and_current_version_not_spelling() {
        let (mut values, copy, entry, base) = fixture();
        let mut work = WordCopyWork::default();
        let a = initial_anchor(&copy, &entry, &values, base, &mut work).unwrap();
        assert_eq!(a.relation_id, 1);
        assert_eq!(field_byte(&values, a, 1, 0, &mut work), Some(b's'));
        assert_eq!(field_byte(&values, a, 2, 0, &mut work), Some(b'D'));
        values.lexemes.as_mut().unwrap().queries[0].byte_end += 1;
        assert!(initial_anchor(&copy, &entry, &values, base, &mut work).is_none());
        assert!(record(&values, a, &mut work).is_none());
        values.lexemes.as_mut().unwrap().queries[0].byte_end -= 1;
        values.relations.as_mut().unwrap().directory[0] = 0;
        assert!(record(&values, a, &mut work).is_none());
    }
    #[test]
    fn native_field_composition_rejects_ambiguity_conflict_dependency_and_changed_winner() {
        let (mut values, mut copy, entry, base) = fixture();
        let mut work = WordCopyWork::default();
        let relations = values.relations.as_mut().unwrap();
        relations.records[1] = RelationRecord {
            id: 2,
            ..relations.records[0]
        };
        relations.directory[1] = 2;
        assert!(initial_anchor(&copy, &entry, &values, base, &mut work).is_none());
        values.relations.as_mut().unwrap().directory[1] = 0;
        values.relations.as_mut().unwrap().records[0].conflict = true;
        assert!(initial_anchor(&copy, &entry, &values, base, &mut work).is_none());
        values.relations.as_mut().unwrap().records[0].conflict = false;
        copy.pending.as_mut().unwrap().dependency = Some([1, 2]);
        assert!(initial_anchor(&copy, &entry, &values, base, &mut work).is_none());
        copy.pending.as_mut().unwrap().dependency = None;
        assert!(initial_anchor(
            &copy,
            &entry,
            &values,
            Candidate { score: 2, ..base },
            &mut work
        )
        .is_none());
    }
    #[test]
    fn native_field_composition_persistent_and_recent_reads_share_exact_field_anchor() {
        let (values, mut copy, entry, base) = fixture();
        let mut work = WordCopyWork::default();
        let recent = initial_anchor(&copy, &entry, &values, base, &mut work).unwrap();
        copy.pending.as_mut().unwrap().word_index = RELATION_SOURCE;
        let retained = initial_anchor(&copy, &entry, &values, base, &mut work).unwrap();
        assert_eq!(recent.relation_id, retained.relation_id);
        for field in [1, 2] {
            let len = field_len(&values, recent, field, &mut work).unwrap();
            for i in 0..len {
                assert_eq!(
                    field_byte(&values, recent, field, i, &mut work),
                    field_byte(&values, retained, field, i, &mut work)
                );
            }
            assert!(field_byte(&values, retained, field, len, &mut work).is_none());
        }
        assert!(field_len(&values, retained, 0, &mut work).is_none());
    }
    #[test]
    fn native_field_composition_namespace_and_offline_pieces_are_explicit() {
        let f = ValueFeature {
            kind: 0,
            a: FIELD_TAG | (u64::from(PREFIX) << 18) | u64::from(PREFIX),
            b: u64::from(OWNER_CHOICE),
        };
        assert!(field_feature(f));
        assert!(!field_feature(ValueFeature {
            a: (16_u64 << 56) | (f.a & ((1_u64 << 56) - 1)),
            ..f
        }));
        let pieces = vec![
            FieldPiece::Owner,
            FieldPiece::Bytes(" is in ".into()),
            FieldPiece::Value,
        ];
        let wire = serde_json::to_value(&pieces).unwrap();
        assert_eq!(wire[0]["type"], "owner");
        assert_eq!(wire[1]["content"], " is in ");
        assert_eq!(
            serde_json::from_value::<Vec<FieldPiece>>(wire).unwrap(),
            pieces
        );
    }
    #[test]
    fn native_field_composition_three_symbols_separate_is_in_and_preserve_exact_packing() {
        let space = u32::from(b' ') + 2;
        let i = u32::from(b'i') + 2;
        let s = u32::from(b's') + 2;
        // The exact observed s/n conflict shares its last two symbols.
        let after_owner_i = [OWNER_CHOICE, space, i];
        let after_is_i = [s, space, i];
        assert_eq!(&after_owner_i[1..], &after_is_i[1..]);
        assert_ne!(
            prefix_identity(after_owner_i[0], space, i),
            prefix_identity(after_is_i[0], space, i)
        );
        let mask = (1_u64 << 18) - 1;
        for previous2 in [BOS, 65793, PREFIX, OWNER_CHOICE, VALUE_CHOICE] {
            for previous in [BOS, 65793, PREFIX, OWNER_CHOICE, VALUE_CHOICE] {
                for last in [BOS, 65793, PREFIX, OWNER_CHOICE, VALUE_CHOICE] {
                    let prefix = prefix_identity(previous2, previous, last);
                    assert_eq!(prefix >> 54, 0);
                    assert_eq!((prefix >> 36) & mask, u64::from(previous2));
                    assert_eq!((prefix >> 18) & mask, u64::from(previous));
                    assert_eq!(prefix & mask, u64::from(last));
                    for prime in [0_u32, 2, 719, u32::MAX] {
                        for choice in [BOS, EOS, 65793, OWNER_CHOICE, VALUE_CHOICE] {
                            let a = FIELD_TAG | (u64::from(prime) << 18) | u64::from(choice);
                            assert_eq!(a >> 56, 17);
                            assert_eq!((a & ((1_u64 << 56) - 1)) >> 18, u64::from(prime));
                            assert_eq!(a & mask, u64::from(choice));
                            assert_eq!((FIELD_TAG | prefix) >> 56, 17);
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn native_field_composition_balanced_query_association_removes_neutral_identities() {
        let classes = [4, 6];
        let mut presence = BTreeMap::from([
            ("selvi".into(), [2, 3]),
            ("dusk".into(), [2, 3]),
            ("ridge".into(), [4, 6]),
            ("where".into(), [4, 6]),
            ("first".into(), [4, 0]),
            ("name".into(), [2, 0]),
            ("sentence".into(), [0, 4]),
        ]);
        let expected = BTreeSet::from(["first".into(), "name".into(), "sentence".into()]);
        assert_eq!(associated_query_words(classes, &presence), expected);
        let renamed = presence.remove("selvi").unwrap();
        presence.insert("replacement".into(), renamed);
        assert_eq!(associated_query_words(classes, &presence), expected);
        // Presence association is class-normalized, not equal raw counts.
        presence.insert("unbalanced".into(), [2, 2]);
        assert!(associated_query_words(classes, &presence).contains("unbalanced"));
    }
}
