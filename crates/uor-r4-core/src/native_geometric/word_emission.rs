//! Completed exact word/span contexts reuse the numeric lexical selector.
//! The witness admits word-only bytes and query addresses, never phrases.
use super::lexical_emission::LexicalEmission;
use super::response_entry_types::{ResponseEntryAction, ResponseEntryDecision, ResponseEntryState};
use super::source_routing::SourceCode;
use super::source_routing_training::{learn_code_pairs, learn_code_subset, Alternative, Frame};
use super::value_types::{ValueFeature, ValueState};
use super::word_copy_types::{
    WordCopyAction, WordCopyAddress, WordCopyDecision, WordCopyProgress, WordCopyState,
    WordCopyWork,
};
use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WordEmission {
    pub parent_artifact: String,
    pub previous_lexical: LexicalEmission,
    pub dictionary: Vec<WordCopyAddress>,
    pub tokens: Vec<u32>,
    /// Offline presence/label association selected this query vocabulary.
    /// Omission preserves the original full-prompt vocabulary artifacts.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub balanced_query_vocabulary: bool,
    /// Bind each query/byte choice to the same actual normalized suffix prefix.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub prefix_query_context: bool,
}
impl WordEmission {
    fn parent(&self, model: &Model) -> Result<Model> {
        let mut parent = model.clone();
        parent.word_emission = None;
        parent.lexical_emission = Some(self.previous_lexical.clone());
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("word emission frozen parent differs".into()));
        }
        parent.validate()?;
        Ok(parent)
    }
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        let current = model
            .lexical_emission
            .as_ref()
            .ok_or_else(|| Error("word emission lexical block absent".into()))?;
        current.validate_shape(model)?;
        if self.tokens.is_empty()
            || self.tokens.len() > 64
            || self.tokens.windows(2).any(|w| w[0] >= w[1])
            || self
                .tokens
                .iter()
                .any(|t| *t != EOS && !(2..258).contains(t))
        {
            return Err(Error("invalid word emission tokens".into()));
        }
        if self.dictionary.is_empty() || self.dictionary.len() > 128 {
            return Err(Error("invalid word emission dictionary".into()));
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
                || w.bytes[usize::from(w.len)..].iter().any(|b| *b != 0)
        }) || self.dictionary.windows(2).any(|w| w[0].bytes >= w[1].bytes)
        {
            return Err(Error("invalid word emission dictionary".into()));
        }
        // Candidate admission and all inherited geometry remain exact. New
        // roots are disjoint from every numeric action/query feature.
        if current.tokens != self.previous_lexical.tokens
            || current.dictionary != self.previous_lexical.dictionary
            || current.router.landmarks != self.previous_lexical.router.landmarks
            || current.router.biases != self.previous_lexical.router.biases
            || current.router.ranks != self.previous_lexical.router.ranks
            || current.router.config.mode != self.previous_lexical.router.config.mode
            || current.router.config.role_context_only
                != self.previous_lexical.router.config.role_context_only
            || current.router.parent_artifact != self.previous_lexical.router.parent_artifact
            || self.previous_lexical.router.codes.iter().any(|old| {
                current
                    .router
                    .codes
                    .binary_search_by_key(&old.feature, |c| c.feature)
                    .ok()
                    .is_none_or(|i| current.router.codes[i] != *old)
            })
            || current.router.codes.iter().any(|c| {
                self.previous_lexical
                    .router
                    .codes
                    .binary_search_by_key(&c.feature, |old| old.feature)
                    .is_err()
                    && !word_feature(c.feature)
            })
        {
            return Err(Error(
                "word emission frozen lexical parameters differ".into(),
            ));
        }
        self.parent(model)?;
        let mut identity = model.clone();
        identity.refresh_identity()?;
        if identity.artifact_cid != model.artifact_cid
            || identity.uor_model_address != model.uor_model_address
        {
            return Err(Error("word emission identity differs".into()));
        }
        Ok(())
    }
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
const WORD_TAG: u64 = 16_u64 << 56;
fn word_feature(f: ValueFeature) -> bool {
    matches!(f.kind, 0 | 3) && f.a >> 56 == 16
}
#[derive(Clone, Copy)]
struct Context {
    prefix: u64,
    prefix_query_context: bool,
    primes: [u32; 32],
    len: usize,
}
fn context(
    w: &WordEmission,
    copy: &WordCopyState,
    entry: &ResponseEntryState,
    values: &ValueState,
    work: &mut WordCopyWork,
) -> Option<Context> {
    if copy.progress != WordCopyProgress::Complete
        || !entry.active
        || entry.steps >= 32
        || entry.seen != values.seen
    {
        return None;
    }
    let commit = copy.read_commit?;
    let origin = commit.source?;
    if copy.origin != Some(origin)
        || !commit
            .dependency
            .is_none_or(|ids| super::dependent_read::valid(values, ids, &mut work.persistent_read))
    {
        return None;
    }
    let source = super::relation::source(values, origin)?;
    if source.end != commit.source_end
        || source.byte_end != commit.source_byte_end
        || super::relation::source_version(values, origin) != commit.relation_id
    {
        return None;
    }
    let length = super::source_span::len(values, origin, copy.span_words, work)?;
    let steps = entry
        .steps
        .checked_sub(copy.start_step)?
        .checked_sub(length)?;
    const PREFIX: u64 = 131072;
    let last = if steps == 0 {
        PREFIX
    } else {
        u64::from(entry.last)
    };
    let previous = if steps < 2 {
        PREFIX
    } else {
        u64::from(entry.previous)
    };
    let mut result = Context {
        prefix: WORD_TAG | (previous << 18) | last,
        prefix_query_context: w.prefix_query_context,
        primes: [0; 32],
        len: 0,
    };
    if let Some(words) = &values.lexemes {
        for word in &words.queries[..words.query_len] {
            if values.query_boundary.is_some_and(|start| word.end < start) {
                continue;
            }
            if let Some(d) = w.dictionary.iter().find(|d| {
                d.len == word.len
                    && d.bytes[..usize::from(d.len)]
                        .iter()
                        .zip(&word.bytes)
                        .all(|(a, b)| *a == b.to_ascii_lowercase())
            }) {
                if result.len >= result.primes.len() {
                    return None;
                }
                result.primes[result.len] = d.prime;
                result.len += 1;
            }
        }
    }
    Some(result)
}
fn features(context: Context, token: u32, erase: bool) -> ([ValueFeature; 40], usize) {
    let mut result = [ValueFeature::default(); 40];
    if erase {
        return (result, 0);
    }
    result[0] = ValueFeature {
        kind: 0,
        a: context.prefix,
        b: u64::from(token),
    };
    let mut len = 1;
    for &prime in &context.primes[..context.len] {
        result[len] = ValueFeature {
            kind: 3,
            a: WORD_TAG | u64::from(prime),
            // Two18-bit actual prefix tokens plus the18-bit candidate occupy
            // disjoint54-bit fields. The word type tag remains in `a` only.
            b: u64::from(token)
                | if context.prefix_query_context {
                    (context.prefix & ((1_u64 << 36) - 1)) << 18
                } else {
                    0
                },
        };
        len += 1;
    }
    (result, len)
}
pub(super) fn offer(
    model: &Model,
    copy: &WordCopyState,
    entry: &ResponseEntryState,
    values: &ValueState,
    control: Control,
    work: &mut WordCopyWork,
) -> Option<u32> {
    if matches!(
        control,
        Control::WordEmissionDisabled
            | Control::LexicalEmissionDisabled
            | Control::GeometryDisabled
            | Control::H4Disabled
            | Control::LearnedRoutingDisabled
    ) {
        return None;
    }
    let witness = model.word_emission.as_ref()?;
    let mut context = context(witness, copy, entry, values, work)?;
    if control == Control::WordEmissionPrefixContextDisabled {
        context.prefix_query_context = false;
    }
    let block = model.lexical_emission.as_ref()?;
    let geometry = if control == Control::WordEmissionGeometryDisabled {
        Control::LearnedRoutingTransformDisabled
    } else {
        control
    };
    let (_, token) = super::lexical_emission::token_choice(
        model,
        &block.router,
        &witness.tokens,
        |token| {
            features(
                context,
                token,
                control == Control::WordEmissionContextDisabled,
            )
        },
        geometry,
        &mut work.routing,
        &mut work.selector,
    );
    token
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WordEmissionExample {
    pub id: String,
    pub prompt: String,
    pub prefix: String,
    pub suffix: String,
    /// Teach Base only along the freely generated frozen-parent continuation.
    /// False omission preserves all previously bound document receipts.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub inherit_suffix: bool,
}

fn append_frame(
    witness: &WordEmission,
    frames: &mut Vec<Frame>,
    s: &mut Session,
    target: Option<u32>,
    id: &str,
) -> Result<()> {
    if frames.len() >= 4096 {
        return Err(Error("word emission presented frame cap4096".into()));
    }
    let c = context(
        witness,
        s.word_copy
            .as_ref()
            .ok_or_else(|| Error("word emission copy absent".into()))?,
        s.response_entry
            .as_ref()
            .ok_or_else(|| Error("word emission entry absent".into()))?,
        s.values
            .as_ref()
            .ok_or_else(|| Error("word emission values absent".into()))?,
        &mut s.work.word_copy,
    )
    .ok_or_else(|| Error(format!("word emission completed context unavailable: {id}")))?;
    let alternatives = std::iter::once((BOS, 0))
        .chain(witness.tokens.iter().map(|t| (*t, 1)))
        .map(|(token, action)| {
            let (f, n) = features(c, token, false);
            Alternative {
                features: f[..n].to_vec(),
                codes: vec![],
                action,
                correct: match target {
                    Some(t) => action == 1 && token == t,
                    None => action == 0,
                },
            }
        })
        .collect();
    frames.push(Frame { alternatives });
    Ok(())
}

fn prepare_label(s: &mut Session, token: u32) -> Result<()> {
    let values = s
        .values
        .as_ref()
        .ok_or_else(|| Error("word emission values absent".into()))?;
    let copy = s
        .word_copy
        .as_mut()
        .ok_or_else(|| Error("word emission copy state absent".into()))?;
    let entry = s
        .response_entry
        .as_mut()
        .ok_or_else(|| Error("word emission entry state absent".into()))?;
    let index = copy
        .read_commit
        .and_then(|c| c.source)
        .ok_or_else(|| Error("word emission committed source absent".into()))?;
    let source = super::relation::source(values, index)
        .ok_or_else(|| Error("word emission committed source unavailable".into()))?;
    let length = super::source_span::len(values, index, copy.span_words, &mut s.work.word_copy)
        .ok_or_else(|| Error("word emission source span unavailable".into()))?;
    let anchor = entry
        .boundary
        .ok_or_else(|| Error("word emission response boundary absent".into()))?;
    copy.pending = Some(WordCopyDecision {
        span_words: copy.span_words,
        dependency: copy.read_commit.and_then(|c| c.dependency),
        token,
        score: 1,
        word_index: index,
        cursor: length,
        source_end: source.end,
        source_byte_end: source.byte_end,
        at_seen: values.seen,
        step: entry.steps,
        action: if token == EOS {
            WordCopyAction::Stop
        } else {
            WordCopyAction::Emit
        },
    });
    entry.pending = Some(ResponseEntryDecision {
        token,
        score: 1,
        boundary_seen: anchor.at_seen,
        step: entry.steps,
        at_seen: values.seen,
        action: if token == EOS {
            ResponseEntryAction::Stop
        } else {
            ResponseEntryAction::Emit
        },
    });
    Ok(())
}
// Offline exact document-presence association. Class identity is the
// declared suffix label, never a serving parser or output-family gate.
fn associated_words(
    classes: &BTreeMap<String, usize>,
    counts: &BTreeMap<String, BTreeMap<String, usize>>,
    documents: usize,
) -> BTreeSet<String> {
    counts
        .iter()
        .filter_map(|(word, by_class)| {
            let total: usize = by_class.values().sum();
            classes
                .iter()
                .any(|(class, size)| {
                    by_class.get(class).copied().unwrap_or(0) * documents != total * size
                })
                .then(|| word.clone())
        })
        .collect()
}
impl Model {
    /// Initial full-vocabulary fit, or an exact-data resume under the artifact's
    /// already bound vocabulary law.
    pub fn fit_word_emission(
        &self,
        docs: &[WordEmissionExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_word_emission_impl(
            docs,
            config,
            self.word_emission
                .as_ref()
                .is_some_and(|w| w.balanced_query_vocabulary),
            self.word_emission
                .as_ref()
                .is_some_and(|w| w.prefix_query_context),
            false,
            false,
        )
    }
    /// Select query words by exact document-presence association with declared
    /// suffix labels. Balanced owner/value substitutions have zero association.
    pub fn fit_word_emission_balanced(
        &self,
        docs: &[WordEmissionExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_word_emission_impl(docs, config, true, false, false, false)
    }
    /// Balanced query identities act locally at each actual suffix prefix.
    pub fn fit_word_emission_contextual(
        &self,
        docs: &[WordEmissionExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_word_emission_impl(docs, config, true, true, false, false)
    }
    /// Resume the exact word artifact/data with bounded joint root updates.
    /// Serving candidates, feature law and every inherited parameter stay fixed.
    pub fn fit_word_emission_pairs(
        &self,
        docs: &[WordEmissionExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        let witness = self
            .word_emission
            .as_ref()
            .ok_or_else(|| Error("word emission pair fit requires existing witness".into()))?;
        self.fit_word_emission_impl(
            docs,
            config,
            witness.balanced_query_vocabulary,
            witness.prefix_query_context,
            true,
            false,
        )
    }
    /// Refine Base/defer choices against new, explicitly labelled contexts.
    /// Existing word admission and the complete original parent remain fixed.
    pub fn fit_word_emission_preserving(
        &self,
        docs: &[WordEmissionExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        let witness = self.word_emission.as_ref().ok_or_else(|| {
            Error("word emission preserving fit requires existing witness".into())
        })?;
        self.fit_word_emission_impl(
            docs,
            config,
            witness.balanced_query_vocabulary,
            witness.prefix_query_context,
            false,
            true,
        )
    }
    fn fit_word_emission_impl(
        &self,
        docs: &[WordEmissionExample],
        config: SourceRoutingConfig,
        balanced_query_vocabulary: bool,
        prefix_query_context: bool,
        pair_updates: bool,
        changed_training_data: bool,
    ) -> Result<(Model, serde_json::Value)> {
        config.validate_word_emission()?;
        self.validate()?;
        if docs.is_empty() || docs.len() > 256 {
            return Err(Error("invalid word emission population/parent".into()));
        }
        let seed_block = self
            .lexical_emission
            .as_ref()
            .ok_or_else(|| Error("word emission lexical parent absent".into()))?;
        let resumed = self.word_emission.is_some();
        let old = self
            .word_emission
            .as_ref()
            .map_or(seed_block, |w| &w.previous_lexical);
        if config.mode != old.router.config.mode
            || config.role_context_only != old.router.config.role_context_only
        {
            return Err(Error("word emission scoring mode differs".into()));
        }
        let mut ids = BTreeSet::new();
        let mut vocabulary = BTreeSet::new();
        let mut tokens = BTreeSet::from([EOS]);
        let mut receipts = Vec::new();
        let mut class_sizes = BTreeMap::<String, usize>::new();
        let mut word_presence = BTreeMap::<String, BTreeMap<String, usize>>::new();
        for d in docs {
            if d.id.trim().is_empty()
                || !ids.insert(d.id.clone())
                || d.prompt.is_empty()
                || d.prompt.len() > 4096
                || d.prefix.is_empty()
                || !d.prefix.is_ascii()
                || !d.suffix.is_ascii()
                || d.prefix.len() + d.suffix.len() + 1 > 32
            {
                return Err(Error(format!(
                    "invalid word emission data bounds/id: {}",
                    d.id
                )));
            }
            let mut present = BTreeSet::new();
            for word in d
                .prompt
                .split(|c: char| !c.is_ascii_alphabetic() && c != '_')
                .filter(|w| !w.is_empty())
            {
                if word.len() > 32 {
                    return Err(Error("word emission dictionary word exceeds32".into()));
                }
                present.insert(word.to_ascii_lowercase());
            }
            *class_sizes.entry(d.suffix.clone()).or_default() += 1;
            for word in present {
                vocabulary.insert(word.clone());
                *word_presence
                    .entry(word)
                    .or_default()
                    .entry(d.suffix.clone())
                    .or_default() += 1;
            }
            if !d.inherit_suffix {
                tokens.extend(d.suffix.bytes().map(|b| u32::from(b) + 2));
            }
            let bytes = serde_json::to_vec(d).map_err(|e| Error(e.to_string()))?;
            receipts.push(DocumentReceipt {
                id: d.id.clone(),
                bytes: bytes.len(),
                text_cid: format!("blake3:{}", blake3::hash(&bytes).to_hex()),
            });
        }
        let observed_words = vocabulary.len();
        if balanced_query_vocabulary && !resumed {
            vocabulary = associated_words(&class_sizes, &word_presence, docs.len());
            if vocabulary.is_empty() {
                return Err(Error("word emission balanced vocabulary is empty".into()));
            }
        }
        let witness = if let Some(previous) = &self.word_emission {
            if previous.balanced_query_vocabulary != balanced_query_vocabulary
                || previous.prefix_query_context != prefix_query_context
                || tokens.iter().any(|t| !previous.tokens.contains(t))
                || (!changed_training_data && seed_block.router.training != receipts)
            {
                return Err(Error("word emission resume data/admission differs".into()));
            }
            previous.clone()
        } else {
            if changed_training_data {
                return Err(Error(
                    "word emission preserving fit requires existing witness".into(),
                ));
            }
            if vocabulary.len() > 128 || tokens.len() > 64 {
                return Err(Error("word emission vocabulary cap".into()));
            }
            let primes = crate::corpus_induced_spin_placement::first_primes(vocabulary.len())
                .map_err(|e| Error(e.to_string()))?;
            let dictionary = vocabulary
                .into_iter()
                .zip(primes)
                .map(|(word, prime)| {
                    let mut bytes = [0; 32];
                    bytes[..word.len()].copy_from_slice(word.as_bytes());
                    WordCopyAddress {
                        bytes,
                        len: word.len() as u8,
                        prime: prime as u32,
                    }
                })
                .collect();
            WordEmission {
                parent_artifact: self.artifact_cid().to_string(),
                previous_lexical: old.clone(),
                dictionary,
                tokens: tokens.into_iter().collect(),
                balanced_query_vocabulary,
                prefix_query_context,
            }
        };
        let vocabulary_report = serde_json::json!({"balanced_query_vocabulary":balanced_query_vocabulary,
            "prefix_query_context":prefix_query_context,"reused_admission":resumed,"observed_words":observed_words,
            "retained_words":witness.dictionary.iter().map(|w|String::from_utf8_lossy(&w.bytes[..usize::from(w.len)]).into_owned()).collect::<Vec<_>>(),
            "document_classes_by_suffix_label":class_sizes,"document_presence_counts":word_presence,
            "scope":"Initial fitting selects vocabulary offline; every resume reuses exact artifact admission. Counts on changed data are descriptive, not vocabulary reselection."});
        let reference = if changed_training_data || docs.iter().any(|d| d.inherit_suffix) {
            Some(if self.word_emission.is_some() {
                witness.parent(self)?
            } else {
                self.clone()
            })
        } else {
            None
        };
        let source_model = reference.as_ref().unwrap_or(self);
        let mut frames = Vec::new();
        let mut prefixes = Vec::new();
        let mut inherited_continuations = Vec::new();
        let mut inherited_frames = 0_usize;
        for d in docs {
            let mut s = source_model.session(Control::Full)?;
            s.observe(source_model, BOS)?;
            for token in source_model.encode(&d.prompt)? {
                s.observe(source_model, token)?;
            }
            s.begin_response(source_model)?;
            let mut output = Vec::new();
            for _ in 0..32 {
                let p = s.predict(source_model)?;
                s.observe(source_model, p.token)?;
                if p.token == EOS {
                    break;
                }
                output.push(p.token);
                if s.word_copy
                    .as_ref()
                    .is_some_and(|c| c.progress == WordCopyProgress::Complete)
                {
                    break;
                }
            }
            let actual = source_model.decode(&output)?;
            if actual != d.prefix.as_bytes()
                || s.work.values.derived_writes != 0
                || s.completion.as_ref().is_some_and(|c| c.active)
            {
                return Err(Error(format!(
                    "word emission actual copied prefix differs: {}: {:?}",
                    d.id,
                    String::from_utf8_lossy(&actual)
                )));
            }
            let entry = s
                .response_entry
                .as_ref()
                .ok_or_else(|| Error("word emission entry absent".into()))?;
            if usize::from(entry.steps) + d.suffix.len() + 1 > 32 {
                return Err(Error(format!(
                    "word emission actual response cap32: {}",
                    d.id
                )));
            }
            let copy = s
                .word_copy
                .as_ref()
                .ok_or_else(|| Error("word emission copy absent".into()))?;
            prefixes.push(serde_json::json!({"id":d.id,"actual_prefix":String::from_utf8_lossy(&actual),"read_commit":copy.read_commit,"origin":copy.origin,"span_words":copy.span_words,"entry_steps":entry.steps}));
            if d.inherit_suffix {
                let mut suffix_tokens = Vec::new();
                let mut terminated = false;
                let before = frames.len();
                for _ in 0..(32 - usize::from(entry.steps)) {
                    append_frame(&witness, &mut frames, &mut s, None, &d.id)?;
                    let p = s.predict(source_model)?;
                    s.observe(source_model, p.token)?;
                    if p.token == EOS {
                        terminated = true;
                        break;
                    }
                    suffix_tokens.push(p.token);
                }
                let suffix = source_model.decode(&suffix_tokens)?;
                if !terminated || suffix != d.suffix.as_bytes() || s.work.values.derived_writes != 0
                {
                    return Err(Error(format!(
                        "word emission inherited continuation differs: {}: {:?}; eos={terminated}",
                        d.id,
                        String::from_utf8_lossy(&suffix)
                    )));
                }
                inherited_frames += frames.len() - before;
                inherited_continuations.push(serde_json::json!({"id":d.id,"source_artifact":source_model.artifact_cid(),"actual_suffix":String::from_utf8_lossy(&suffix),"eos":terminated,"frames":frames.len()-before}));
            } else {
                for target in d
                    .suffix
                    .bytes()
                    .map(|b| u32::from(b) + 2)
                    .chain(std::iter::once(EOS))
                {
                    append_frame(&witness, &mut frames, &mut s, Some(target), &d.id)?;
                    prepare_label(&mut s, target)?;
                    s.observe(source_model, target)?;
                }
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
        let vocab: BTreeSet<_> = frames
            .iter()
            .flat_map(|f| {
                f.alternatives
                    .iter()
                    .flat_map(|a| a.features.iter().copied())
            })
            .collect();
        if resumed
            && !changed_training_data
            && vocab.iter().any(|feature| {
                seed_block
                    .router
                    .codes
                    .binary_search_by_key(feature, |c| c.feature)
                    .is_err()
            })
        {
            return Err(Error(
                "word emission resume feature vocabulary differs".into(),
            ));
        }
        let conflicts = super::action_emission::operation_frame_conflicts(&frames)?;
        if conflicts["count"].as_u64().unwrap_or(0) > 0 {
            return Err(Error(format!(
                "word emission contradictory construction frames: {conflicts}"
            )));
        }
        let mut block = seed_block.clone();
        for feature in vocab {
            if seed_block
                .router
                .codes
                .binary_search_by_key(&feature, |c| c.feature)
                .is_ok()
            {
                continue;
            }
            let mut inherited = feature;
            inherited.a &= (1_u64 << 56) - 1;
            let roots = if feature.kind == 0 {
                old.router
                    .codes
                    .binary_search_by_key(&inherited, |c| c.feature)
                    .ok()
                    .map_or([self.geometry.identity; 2], |i| old.router.codes[i].roots)
            } else {
                [self.geometry.identity; 2]
            };
            block.router.codes.push(SourceCode { feature, roots });
        }
        block.router.codes.sort_by_key(|c| c.feature);
        if block.router.codes.len() > config.learned_features {
            return Err(Error(format!(
                "word emission complete feature cap exceeded: {} required",
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
                    .binary_search_by_key(&c.feature, |old| old.feature)
                    .is_err()
            })
            .collect();
        block.router.config = config;
        block.router.training = receipts;
        let fit_start = Instant::now();
        let fit = if pair_updates {
            learn_code_pairs(self, &mut block.router, &mut frames, fit_start, &mutable)?
        } else {
            learn_code_subset(self, &mut block.router, &mut frames, fit_start, &mutable)?
        };
        let fit_elapsed_ms = fit_start.elapsed().as_millis();
        let failures =
            super::action_emission::operation_misclassified_frames(self, &block.router, &frames);
        let parent_artifact = witness.parent_artifact.clone();
        let mut model = self.clone();
        model.lexical_emission = Some(block);
        model.word_emission = Some(witness);
        model.refresh_identity()?;
        model.validate()?;
        let report = serde_json::json!({"parent_artifact":parent_artifact,"start_artifact":self.artifact_cid(),"vocabulary_selection":vocabulary_report,"resumed":resumed,"changed_training_data":changed_training_data,"inherited_continuations":inherited_continuations,"inherited_frames":inherited_frames,"construction_frame_conflicts":conflicts,"pair_updates":pair_updates,"fit_elapsed_ms":fit_elapsed_ms,"same_resume_data_and_admission":resumed && !changed_training_data,"word_admission_frozen":resumed,"artifact":model.artifact_cid(),"fit":fit,"frames":frames.len(),"presented_frames":presented,"features":mutable.len(),"added_features":mutable.len()-seed_block.router.codes.len(),"mutable_word_features":mutable.iter().filter(|m|**m).count(),"frozen_features":old.router.codes.len(),"copied_prefixes":prefixes,"misclassified_frames":failures,"suffix_teacher_forced":docs.iter().any(|d|!d.inherit_suffix),"inheritance_freely_generated":docs.iter().any(|d|d.inherit_suffix),"prefix_freely_generated":true,"prefix_source_artifact":source_model.artifact_cid(),"free_generation":"NOT_RUN by fit; evaluate returned candidate separately","numeric_candidates_and_features_frozen":true,"inherited_roots_landmarks_biases_ranks_frozen":true,"shared_lexical_router":true,"total_response_steps_including_eos":32});
        Ok((model, report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_word_emission_inheritance_label_preserves_previous_receipt_serialization() {
        let old =
            serde_json::json!({"id":"receipt","prompt":"prompt","prefix":" source","suffix":".\n"});
        let mut example: WordEmissionExample = serde_json::from_value(old.clone()).unwrap();
        assert!(!example.inherit_suffix);
        assert_eq!(serde_json::to_value(&example).unwrap(), old);
        example.inherit_suffix = true;
        let current = serde_json::to_value(&example).unwrap();
        assert_eq!(current["inherit_suffix"], true);
        assert!(
            serde_json::from_value::<WordEmissionExample>(current)
                .unwrap()
                .inherit_suffix
        );
    }
    #[test]
    fn native_word_emission_balanced_vocabulary_uses_presence_and_class_sizes() {
        let classes = BTreeMap::from([("short".into(), 2), ("long".into(), 4)]);
        let counts = BTreeMap::from([
            (
                "owner".into(),
                BTreeMap::from([("short".into(), 1), ("long".into(), 2)]),
            ),
            (
                "common".into(),
                BTreeMap::from([("short".into(), 2), ("long".into(), 4)]),
            ),
            ("format".into(), BTreeMap::from([("long".into(), 4)])),
            ("otherformat".into(), BTreeMap::from([("short".into(), 2)])),
        ]);
        assert_eq!(
            associated_words(&classes, &counts, 6),
            BTreeSet::from(["format".into(), "otherformat".into()])
        );
        // Renaming a balanced identity changes no selected context word.
        let mut renamed = counts;
        let owner = renamed.remove("owner").unwrap();
        renamed.insert("renamed".into(), owner);
        assert_eq!(
            associated_words(&classes, &renamed, 6),
            BTreeSet::from(["format".into(), "otherformat".into()])
        );
    }
    #[test]
    fn native_word_emission_prefix_query_fields_preserve_candidate_and_legacy_context() {
        let mask = (1_u64 << 18) - 1;
        for previous in [0, 65793, 131072] {
            for last in [0, 65793, 131072] {
                let c = Context {
                    prefix: WORD_TAG | (previous << 18) | last,
                    prefix_query_context: true,
                    primes: [7; 32],
                    len: 1,
                };
                for token in [BOS, EOS, 65793] {
                    let (qualified, n) = features(c, token, false);
                    assert_eq!(n, 2);
                    assert_eq!(qualified[1].b & mask, u64::from(token));
                    assert_eq!((qualified[1].b >> 18) & mask, last);
                    assert_eq!((qualified[1].b >> 36) & mask, previous);
                    assert_eq!(qualified[1].b >> 54, 0);
                    let mut erased = c;
                    erased.prefix_query_context = false;
                    let (legacy, m) = features(erased, token, false);
                    assert_eq!(n, m);
                    assert_eq!(qualified[0], legacy[0]);
                    assert_eq!(qualified[1].a, legacy[1].a);
                    assert_eq!(legacy[1].b, u64::from(token));
                }
            }
        }
    }
    #[test]
    fn native_word_emission_namespace_preserves_candidate_and_query_order() {
        let c = Context {
            prefix: WORD_TAG | (131072 << 18) | 131072,
            prefix_query_context: false,
            primes: [2; 32],
            len: 2,
        };
        let (f, n) = features(c, EOS, false);
        assert_eq!(n, 3);
        assert!(f[..n].iter().all(|f| word_feature(*f)));
        assert_eq!(f[0].b, u64::from(EOS));
        let (base, _) = features(c, BOS, false);
        assert_ne!(base[0], f[0]);
        let mut ordered = c;
        ordered.primes[1] = 3;
        let (forward, n) = features(ordered, EOS, false);
        ordered.primes.swap(0, 1);
        let (reverse, m) = features(ordered, EOS, false);
        assert_eq!(n, m);
        assert_eq!(forward[0], reverse[0]);
        assert_eq!(forward[1], reverse[2]);
        assert_eq!(forward[2], reverse[1]);
        assert_ne!(forward[..n], reverse[..m]);
        for action in 0_u64..=3 {
            let numeric = ValueFeature {
                kind: 0,
                a: (action << 56) | (c.prefix & ((1_u64 << 56) - 1)),
                b: u64::from(EOS),
            };
            assert_ne!(numeric, f[0]);
            assert!(!word_feature(numeric));
        }
        assert!(!word_feature(ValueFeature {
            kind: 3,
            a: 2,
            b: u64::from(EOS)
        }));
    }
    #[test]
    fn native_word_emission_context_erasure_removes_prefix_and_query_features() {
        for prefix in [
            WORD_TAG,
            WORD_TAG | (131072 << 18) | 131072,
            WORD_TAG | (34 << 18) | 117,
        ] {
            let c = Context {
                prefix,
                prefix_query_context: false,
                primes: [17; 32],
                len: 32,
            };
            for token in [BOS, EOS, 34, 117] {
                let (f, n) = features(c, token, true);
                assert_eq!(n, 0);
                assert_eq!(f, [ValueFeature::default(); 40]);
                let (_, normal) = features(c, token, false);
                assert_eq!(normal, 33);
            }
        }
    }
    #[test]
    fn native_word_emission_larger_fit_window_keeps_legacy_config_bound() {
        let mut config = SourceRoutingConfig::default();
        config.learned_features = 768;
        assert!(config.validate().is_ok());
        config.learned_features = 769;
        assert!(config.validate().is_err());
        assert!(config.validate_word_emission().is_ok());
        config.learned_features = 4096;
        assert!(config.validate_word_emission().is_ok());
        config.learned_features = 4097;
        assert!(config.validate_word_emission().is_err());
        config.learned_features = 4096;
        config.max_seconds = 121;
        assert!(config.validate_word_emission().is_err());
        config.max_seconds = 120;
        config.passes = 9;
        assert!(config.validate_word_emission().is_err());
    }
}
