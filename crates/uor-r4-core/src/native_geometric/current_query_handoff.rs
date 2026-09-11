//! Learned current-record selection before the frozen dependent/recent dispatch.
use super::historical_read;
use super::relation::{RelationRecord, RelationState};
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{learn_code_subset, Alternative, Frame};
use super::value_lexemes::LexemeState;
use super::value_types::{ValueFeature, ValueState};
use super::word_copy_types::WordCopyAddress;
use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurrentQueryExample {
    pub id: String,
    pub prompt: String,
    /// Offline exact current-record target; None preserves the entire parent dispatch.
    pub current_record: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CurrentQueryHandoff {
    pub parent_artifact: String,
    pub router: SourceRouting,
    pub config: SourceRoutingConfig,
    pub training: Vec<DocumentReceipt>,
    /// None preserves the inherited-dictionary law and legacy artifact bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dictionary: Option<Vec<WordCopyAddress>>,
    /// Limit selector context to source words after the latest committed fact.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub committed_query_scope: bool,
}

fn seed(model: &Model) -> Result<SourceRouting> {
    let h = historical_read::head(model, Control::Full)
        .ok_or_else(|| Error("current query historical context absent".into()))?;
    if h.query_scope != 2 {
        return Err(Error("current query requires local scope".into()));
    }
    let mut router = h.router.clone();
    router.codes.clear();
    Ok(router)
}

fn config_valid(config: &SourceRoutingConfig, previous: &SourceRouting) -> Result<()> {
    config.validate()?;
    if config.learned_features > previous.config.learned_features
        || config.mode != previous.config.mode
        || config.role_context_only != previous.config.role_context_only
    {
        return Err(Error(
            "current query changed frozen capacity or mode".into(),
        ));
    }
    Ok(())
}

/// Parent restoration and non-code equality validate the router's remaining shape.
fn validate_lexical_codes(dictionary: &[WordCopyAddress], router: &SourceRouting) -> Result<()> {
    if dictionary.is_empty() || dictionary.len() > 256 {
        return Err(Error(
            "invalid current query lexical dictionary bound".into(),
        ));
    }
    let expected = crate::corpus_induced_spin_placement::first_primes(dictionary.len())
        .map_err(|e| Error(e.to_string()))?;
    if dictionary.iter().zip(expected).any(|(word, prime)| {
        word.len == 0
            || word.len > 32
            || u64::from(word.prime) != prime
            || !(word.bytes[0].is_ascii_alphabetic() || word.bytes[0] == b'_')
            || word.bytes[..usize::from(word.len)]
                .iter()
                .any(|b| !b.is_ascii_alphanumeric() && *b != b'_')
            || word.bytes[usize::from(word.len)..].iter().any(|b| *b != 0)
    }) || dictionary.windows(2).any(|w| w[0].bytes >= w[1].bytes)
    {
        return Err(Error("invalid current query lexical dictionary".into()));
    }
    let primes: BTreeSet<u64> = std::iter::once(0)
        .chain(dictionary.iter().map(|w| u64::from(w.prime)))
        .collect();
    let valid = |f: ValueFeature| match f.kind {
        0 | 2 => f.a == 1 && f.b == 0,
        1 => f.a <= 1 && f.b == 0,
        3 | 7 => primes.contains(&f.a) && primes.contains(&f.b),
        4 | 5 => f.a <= 1 && primes.contains(&f.b),
        6 => primes.contains(&f.a) && f.b == 0,
        _ => false,
    };
    if router.codes.is_empty()
        || router.codes.len() > router.config.learned_features
        || router
            .codes
            .windows(2)
            .any(|c| c[0].feature >= c[1].feature)
        || router
            .codes
            .iter()
            .any(|c| !valid(c.feature) || c.roots.iter().any(|r| *r >= 120))
    {
        return Err(Error("invalid current query lexical codes".into()));
    }
    Ok(())
}

impl CurrentQueryHandoff {
    fn parent(&self, model: &Model) -> Result<Model> {
        let mut parent = model.clone();
        parent.current_query_handoff = None;
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("current query parent differs".into()));
        }
        parent.validate()?;
        Ok(parent)
    }

    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        let parent = self.parent(model)?;
        let previous = seed(&parent)?;
        config_valid(&self.config, &previous)?;
        if let Some(dictionary) = &self.dictionary {
            validate_lexical_codes(dictionary, &self.router)?;
        } else {
            let mut shape = historical_read::head(&parent, Control::Full)
                .ok_or_else(|| Error("current query historical shape absent".into()))?
                .clone();
            shape.router = self.router.clone();
            shape.validate(model)?;
        }
        let mut restored = self.router.clone();
        restored.codes.clear();
        if (self.committed_query_scope && self.dictionary.is_none())
            || restored != previous
            || self.router.codes.len() > self.config.learned_features
            || self.router.codes.iter().any(|c| c.feature.kind > 7)
            || self.training.is_empty()
            || self.training.len() > 4096
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
                "invalid current query frozen parameters or receipts".into(),
            ));
        }
        let mut duplicate = model.clone();
        duplicate.refresh_identity()?;
        if duplicate.artifact_cid != model.artifact_cid
            || duplicate.uor_model_address != model.uor_model_address
        {
            return Err(Error("current query identity differs".into()));
        }
        Ok(())
    }
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
/// Source occurrence scope only; this does not parse a linguistic request.
pub(super) fn scoped_words(
    words: &LexemeState,
    boundary: Option<u64>,
    relations: Option<&RelationState>,
    committed: bool,
    work: &mut WordCopyWork,
) -> (LexemeState, Option<u64>) {
    let mut view = historical_read::query_view(words, boundary, true, work);
    let mut cutoff = None;
    if committed {
        if let Some(relations) = relations {
            for record in &relations.records {
                work.persistent_read.relations.record_reads += 1;
                work.routing.comparisons += 1;
                if record.id == 0 {
                    continue;
                }
                // Include noncurrent and conflicted committed facts as source occurrences.
                for end in [
                    record.owner.byte_end,
                    record.value.byte_end,
                    record
                        .span
                        .as_ref()
                        .map_or(record.value.byte_end, |s| s.terminal.byte_end),
                ] {
                    work.routing.comparisons += 1;
                    if cutoff.is_none_or(|prior| end > prior) {
                        cutoff = Some(end);
                    }
                }
            }
        }
        if let Some(cutoff) = cutoff {
            let mut count = 0;
            for word in &view.queries[..view.query_len] {
                work.word_record_reads += 1;
                work.routing.comparisons += 1;
                if word.byte_end <= cutoff {
                    break;
                }
                count += 1;
            }
            view.query_len = count;
        }
    }
    (view, cutoff)
}

fn addresses(
    model: &Model,
    dictionary: Option<&[WordCopyAddress]>,
    words: &LexemeState,
    work: &mut WordCopyWork,
) -> Option<[u32; 16]> {
    let dictionary = match dictionary {
        Some(dictionary) => dictionary,
        None => &historical_read::head(model, Control::Full)?.dictionary,
    };
    let mut out = [0; 16];
    for (i, word) in words.queries[..words.query_len].iter().enumerate() {
        work.word_record_reads += 1;
        out[i] = super::word_copy_runtime::address_in(dictionary, word, work);
        work.selector.state_copies += 1;
    }
    Some(out)
}

fn features(
    model: &Model,
    record: &RelationRecord,
    words: &LexemeState,
    addr: &[u32; 16],
    work: &mut WordCopyWork,
) -> ([ValueFeature; 96], usize) {
    let mut out = [ValueFeature::default(); 96];
    let (base, n) =
        super::relation::read_features(model, record, words, addr, &mut work.persistent_read);
    out[..n].copy_from_slice(&base[..n]);
    let total = historical_read::append_ordered_context(&mut out, n, addr, words.query_len, 16);
    work.persistent_read.relations.feature_writes += (total - n) as u64;
    (out, total)
}

fn candidate(
    model: &Model,
    values: &ValueState,
    control: Control,
    work: &mut WordCopyWork,
) -> Option<(u8, usize, u64)> {
    let (source, action) = super::relation::read_choice_with_recent(
        model,
        values,
        true,
        control,
        &mut work.persistent_read,
    )?;
    let index = source.checked_sub(super::relation::RELATION_SOURCE)?;
    let state = values.relations.as_ref()?;
    let record = state.records.get(usize::from(index))?;
    work.persistent_read.relations.record_reads += 1;
    let present = state.directory.iter().any(|id| {
        work.persistent_read.relations.directory_reads += 1;
        *id == record.id
    });
    if record.conflict
        || !present
        || !historical_read::representable(model, values, record, action, work)
    {
        return None;
    }
    Some((source, action, record.id))
}

pub(super) fn choose(
    model: &Model,
    values: &ValueState,
    control: Control,
    work: &mut WordCopyWork,
) -> Option<(u8, usize)> {
    if matches!(
        control,
        Control::CurrentQueryHandoffDisabled
            | Control::LearnedRoutingDisabled
            | Control::LearnedRoutingSelectionDisabled
            | Control::H4Disabled
    ) {
        return None;
    }
    let block = model.current_query_handoff.as_ref()?;
    work.routing.predictions += 1;
    let (source, action, id) = candidate(model, values, control, work)?;
    work.routing.sources_examined += 1;
    let (defer, _) = historical_read::action_indices(model)?;
    let (words, _) = scoped_words(
        values.lexemes.as_ref()?,
        values.query_boundary,
        values.relations.as_ref(),
        block.committed_query_scope && control != Control::CurrentQueryHandoffScopeDisabled,
        work,
    );
    let addr = addresses(model, block.dictionary.as_deref(), &words, work)?;
    work.persistent_read.relations.record_reads += 1;
    let record = values.relations.as_ref()?.record(id)?;
    let (features, n) = features(model, record, &words, &addr, work);
    let transform_control = if control == Control::CurrentQueryHandoffTransformDisabled {
        Control::LearnedRoutingTransformDisabled
    } else {
        control
    };
    let roots = block
        .router
        .encode(model, &features[..n], transform_control, &mut work.routing);
    let base = block.router.score(
        model,
        [model.geometry.identity; 2],
        defer,
        &mut work.routing,
    );
    let score = block.router.score(model, roots, action, &mut work.routing);
    work.routing.comparisons += 1;
    (score > base).then_some((source, action))
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

impl Model {
    /// Allocating host diagnostic using the actual admission, lexical and H4 laws.
    /// Scores are selector scores; a historical result can bypass the handoff.
    pub fn current_query_trace(&self, prompt: &str) -> Result<serde_json::Value> {
        if prompt.is_empty() || prompt.len() > 65536 {
            return Err(Error("current query trace input bound".into()));
        }
        let block = self
            .current_query_handoff
            .as_ref()
            .ok_or_else(|| Error("current query trace handoff absent".into()))?;
        let tokens = self.encode(prompt)?;
        if tokens.len() > 8192 {
            return Err(Error("current query trace token bound".into()));
        }
        let mut session = self.session(Control::Full)?;
        session.observe(self, BOS)?;
        for token in tokens {
            session.observe(self, token)?;
        }
        session.begin_response(self)?;
        session.predict(self)?;
        let values = session
            .values
            .as_ref()
            .ok_or_else(|| Error("current query trace values absent".into()))?;
        let entry = session
            .response_entry
            .as_ref()
            .ok_or_else(|| Error("current query trace entry absent".into()))?;
        let historical =
            historical_read::choose(self, values, Control::Full, &mut Default::default());
        let eligible = super::word_copy_runtime::eligible(self, entry, values, Control::Full);
        let offered = candidate(self, values, Control::Full, &mut Default::default());
        let raw_words = values
            .lexemes
            .as_ref()
            .ok_or_else(|| Error("current query trace words absent".into()))?;
        let (words, cutoff) = scoped_words(
            raw_words,
            values.query_boundary,
            values.relations.as_ref(),
            block.committed_query_scope,
            &mut Default::default(),
        );
        let addr = addresses(
            self,
            block.dictionary.as_deref(),
            &words,
            &mut Default::default(),
        )
        .ok_or_else(|| Error("current query trace dictionary absent".into()))?;
        let captured: Vec<_> = words.queries[..words.query_len].iter().enumerate().map(|(q, word)| {
            serde_json::json!({"reverse_index":q,"word":String::from_utf8_lossy(&word.bytes[..usize::from(word.len)]),
                "prime":addr[q],"end":word.end,"byte_end":word.byte_end})
        }).collect();
        let (defer, _) = historical_read::action_indices(self)
            .ok_or_else(|| Error("current query trace actions absent".into()))?;
        let base = block.router.score(
            self,
            [self.geometry.identity; 2],
            defer,
            &mut Default::default(),
        );
        let mut detail = serde_json::Value::Null;
        if let Some((source, action, id)) = offered {
            let record = values
                .relations
                .as_ref()
                .and_then(|s| s.record(id))
                .ok_or_else(|| Error("current query trace record absent".into()))?;
            let (f, n) = features(self, record, &words, &addr, &mut Default::default());
            let roots = block
                .router
                .encode(self, &f[..n], Control::Full, &mut Default::default());
            let mapped: Vec<_> = f[..n]
                .iter()
                .map(|f| {
                    block
                        .router
                        .codes
                        .binary_search_by_key(f, |c| c.feature)
                        .ok()
                        .map(|i| &block.router.codes[i])
                })
                .collect();
            detail = serde_json::json!({"record":id,"source":source,"action":action,"features":f[..n],
                "mapped_codes":mapped,"roots":roots,"score":block.router.score(self,roots,action,&mut Default::default()),
                "source_end":record.value.end,"source_byte_end":record.value.byte_end});
        }
        Ok(
            serde_json::json!({"artifact":self.artifact_cid(),"dictionary_mode":if block.dictionary.is_some(){"construction_prompts"}else{"inherited"},
            "query_boundary":values.query_boundary,"committed_query_scope":block.committed_query_scope,
            "committed_source_byte_cutoff":cutoff,"captured":captured,"base_score":base,"candidate":detail,
            "word_copy_eligible":eligible,"historical_choice":historical,"handoff_reached":eligible&&historical.is_none(),
            "choice":choose(self,values,Control::Full,&mut Default::default()),
            "actual_word_copy":session.word_copy_decision(),"actual_field":session.field_composition_decision()}),
        )
    }

    pub fn without_current_query_handoff(&self) -> Result<Model> {
        match &self.current_query_handoff {
            Some(w) => w.parent(self),
            None => Ok(self.clone()),
        }
    }

    pub fn fit_current_query_handoff(
        &self,
        docs: &[CurrentQueryExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_current_query_mode(docs, config, false, false)
    }

    /// Learn an outer prompt dictionary (frequency cap128) and fresh code roots.
    /// Inner dictionaries, candidate admission and all parent parameters remain frozen.
    pub fn fit_current_query_lexical_handoff(
        &self,
        docs: &[CurrentQueryExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_current_query_mode(docs, config, true, false)
    }

    /// Use a construction dictionary and a selector-local post-commit source view.
    pub fn fit_current_query_scoped_handoff(
        &self,
        docs: &[CurrentQueryExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_current_query_mode(docs, config, true, true)
    }

    fn fit_current_query_mode(
        &self,
        docs: &[CurrentQueryExample],
        config: SourceRoutingConfig,
        lexical: bool,
        committed: bool,
    ) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        let previous = seed(self)?;
        config_valid(&config, &previous)?;
        if self.current_query_handoff.is_some()
            || docs.is_empty()
            || docs.len() > 4096
            || docs.iter().any(|d| {
                d.prompt.is_empty() || d.prompt.len() > 65536 || d.current_record == Some(0)
            })
            || docs.iter().map(|d| d.prompt.len()).sum::<usize>() > 8 * 1024 * 1024
        {
            return Err(Error("invalid current query fit bounds or labels".into()));
        }
        let start = Instant::now();
        let (dictionary, omitted_words, omitted_occurrences) = if lexical {
            let prompts: Vec<ValueExample> = docs
                .iter()
                .map(|d| ValueExample {
                    id: d.id.clone(),
                    prompt: d.prompt.clone(),
                    response: String::new(),
                })
                .collect();
            let (dictionary, words, occurrences) =
                super::word_copy_training::dictionary_with_limit(&prompts, 128)?;
            if dictionary.is_empty() {
                return Err(Error("current query construction dictionary empty".into()));
            }
            (Some(dictionary), words, occurrences)
        } else {
            (None, 0, 0)
        };
        let (defer, _) = historical_read::action_indices(self)
            .ok_or_else(|| Error("current query actions ambiguous".into()))?;
        let mut router = previous;
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut frames = Vec::new();
        let mut seen = BTreeMap::new();
        let mut vocabulary = BTreeSet::new();
        let mut labels = Vec::new();
        let mut skipped = Vec::new();
        let mut duplicates = 0;
        for d in docs {
            if start.elapsed().as_secs() >= config.max_seconds {
                return Err(Error("current query preparation time limit".into()));
            }
            if d.id.trim().is_empty() || !ids.insert(&d.id) {
                return Err(Error(
                    "current query duplicate or empty document identity".into(),
                ));
            }
            receipts.push(super::training::receipt(&Document {
                id: d.id.clone(),
                text: serde_json::to_string(d).map_err(|e| Error(e.to_string()))?,
            }));
            let tokens = self.encode(&d.prompt)?;
            if tokens.len() > 8192 {
                return Err(Error(format!("current query token bound: {}", d.id)));
            }
            let mut session = self.session(Control::Full)?;
            session.observe(self, BOS)?;
            for token in tokens {
                if start.elapsed().as_secs() >= config.max_seconds {
                    return Err(Error("current query preparation time limit".into()));
                }
                session.observe(self, token)?;
            }
            session.begin_response(self)?;
            session.predict(self)?;
            let values = session
                .values
                .as_ref()
                .ok_or_else(|| Error("current query values absent".into()))?;
            let entry = session
                .response_entry
                .as_ref()
                .ok_or_else(|| Error("current query entry absent".into()))?;
            let eligible = super::word_copy_runtime::eligible(self, entry, values, Control::Full)
                && historical_read::choose(self, values, Control::Full, &mut Default::default())
                    .is_none();
            let offered = if eligible {
                candidate(self, values, Control::Full, &mut Default::default())
            } else {
                None
            };
            if let Some(target) = d.current_record {
                if offered.is_none_or(|(_, _, id)| id != target) {
                    return Err(Error(format!(
                        "current query exact candidate unavailable: {} target{} offered{:?}",
                        d.id, target, offered
                    )));
                }
            }
            let Some((source, action, id)) = offered else {
                skipped.push(d.id.clone());
                continue;
            };
            let raw_words = values
                .lexemes
                .as_ref()
                .ok_or_else(|| Error("current query words absent".into()))?;
            let (words, _) = scoped_words(
                raw_words,
                values.query_boundary,
                values.relations.as_ref(),
                committed,
                &mut Default::default(),
            );
            let addr = addresses(self, dictionary.as_deref(), &words, &mut Default::default())
                .ok_or_else(|| Error("current query context absent".into()))?;
            let record = values
                .relations
                .as_ref()
                .and_then(|s| s.record(id))
                .ok_or_else(|| Error("current query record absent".into()))?;
            let (features, n) = features(self, record, &words, &addr, &mut Default::default());
            let features = features[..n].to_vec();
            let target = d.current_record.is_some();
            labels.push(serde_json::json!({"id":d.id,"offered_record":id,"offered_source":source,"action":action,"accept":target}));
            let signature = (features.clone(), action);
            if let Some((prior, prior_id)) = seen.get(&signature) {
                if *prior != target {
                    return Err(Error(format!(
                        "current query conflicting frames: {} and {}",
                        prior_id, d.id
                    )));
                }
                duplicates += 1;
                continue;
            }
            seen.insert(signature, (target, d.id.clone()));
            vocabulary.extend(features.iter().copied());
            if vocabulary.len() > config.learned_features {
                return Err(Error("current query feature bound".into()));
            }
            frames.push(Frame {
                alternatives: vec![
                    Alternative {
                        features: Vec::new(),
                        codes: Vec::new(),
                        action: defer,
                        correct: !target,
                    },
                    Alternative {
                        features,
                        codes: Vec::new(),
                        action,
                        correct: target,
                    },
                ],
            });
        }
        if frames.is_empty() {
            return Err(Error("current query has no competitive frames".into()));
        }
        router.codes = vocabulary
            .into_iter()
            .map(|feature| SourceCode {
                feature,
                roots: [self.geometry.identity; 2],
            })
            .collect();
        let original_config = router.config.clone();
        router.config = config.clone();
        let mutable = vec![true; router.codes.len()];
        let fit = learn_code_subset(self, &mut router, &mut frames, start, &mutable)?;
        router.config = original_config;
        let mut model = self.clone();
        model.current_query_handoff = Some(CurrentQueryHandoff {
            parent_artifact: self.artifact_cid().to_owned(),
            router,
            config,
            training: receipts,
            dictionary,
            committed_query_scope: committed,
        });
        model.refresh_identity()?;
        model.validate()?;
        let report = serde_json::json!({"schema":"uor-r4.current-query-handoff-fit/1","parent":self.artifact_cid(),"artifact":model.artifact_cid(),"documents":docs.len(),"frames":frames.len(),"duplicates":duplicates,"skipped":skipped,"labels":labels,"fit":fit,"elapsed_ms":start.elapsed().as_millis(),"scope":"Exact frozen current candidate or Base. Only new H4 code roots learned; parent unchanged. Requires generated behavior evaluation."});
        let mut report = report;
        if committed {
            report["committed_query_scope"] = serde_json::json!(true);
        }
        if lexical {
            let dictionary = model
                .current_query_handoff
                .as_ref()
                .and_then(|b| b.dictionary.as_ref())
                .ok_or_else(|| Error("current query fitted dictionary absent".into()))?;
            report["dictionary_mode"] = serde_json::json!("construction_prompts");
            report["dictionary_limit"] = serde_json::json!(128);
            report["dictionary_words"] = serde_json::json!(dictionary.len());
            report["dictionary_omitted_words"] = serde_json::json!(omitted_words);
            report["dictionary_omitted_occurrences"] = serde_json::json!(omitted_occurrences);
            report["dictionary"] = serde_json::json!(dictionary);
            report["features"] = serde_json::json!(model
                .current_query_handoff
                .as_ref()
                .map(|b| b.router.codes.len()));
        }
        Ok((model, report))
    }
}

#[cfg(test)]
mod lexical_tests {
    use super::*;
    fn fixture() -> (Vec<WordCopyAddress>, SourceRouting) {
        let docs = [ValueExample {
            id: "words".into(),
            prompt: "alpha beta".into(),
            response: String::new(),
        }];
        let dictionary = super::super::word_copy_training::dictionary_with_limit(&docs, 128)
            .unwrap()
            .0;
        let router = SourceRouting {
            schema: "uor-r4.geometric-source-routing/1".into(),
            parent_artifact: "blake3:parent".into(),
            codes: vec![SourceCode {
                feature: ValueFeature {
                    kind: 7,
                    a: 2,
                    b: 3,
                },
                roots: [119, 119],
            }],
            landmarks: Vec::new(),
            biases: Vec::new(),
            ranks: Vec::new(),
            training: Vec::new(),
            config: SourceRoutingConfig::default(),
        };
        (dictionary, router)
    }
    #[test]
    fn current_query_lexical_dictionary_requires_canonical_primes_order_and_padding() {
        let (dictionary, router) = fixture();
        assert!(validate_lexical_codes(&dictionary, &router).is_ok());
        let mut changed = dictionary.clone();
        changed[0].prime = 0;
        assert!(validate_lexical_codes(&changed, &router).is_err());
        changed = dictionary.clone();
        changed.swap(0, 1);
        assert!(validate_lexical_codes(&changed, &router).is_err());
        changed = dictionary.clone();
        changed[0].bytes[31] = b'x';
        assert!(validate_lexical_codes(&changed, &router).is_err());
        changed = dictionary;
        changed[0].len = 33;
        assert!(validate_lexical_codes(&changed, &router).is_err());
    }
    #[test]
    fn current_query_lexical_codes_reject_unknown_primes_and_bad_shapes() {
        let (dictionary, mut router) = fixture();
        router.codes[0].feature.a = 5;
        assert!(validate_lexical_codes(&dictionary, &router).is_err());
        router.codes[0].feature = ValueFeature {
            kind: 6,
            a: 2,
            b: 3,
        };
        assert!(validate_lexical_codes(&dictionary, &router).is_err());
        router.codes[0].feature = ValueFeature {
            kind: 6,
            a: 2,
            b: 0,
        };
        router.codes[0].roots[0] = 120;
        assert!(validate_lexical_codes(&dictionary, &router).is_err());
    }
    #[test]
    fn current_query_lexical_option_preserves_legacy_wire_shape() {
        let (dictionary, router) = fixture();
        let mut block = CurrentQueryHandoff {
            parent_artifact: "blake3:parent".into(),
            config: router.config.clone(),
            router,
            training: Vec::new(),
            dictionary: None,
            committed_query_scope: false,
        };
        let legacy = serde_json::to_vec(&block).unwrap();
        let restored: CurrentQueryHandoff = serde_json::from_slice(&legacy).unwrap();
        assert_eq!(serde_json::to_vec(&restored).unwrap(), legacy);
        assert!(serde_json::to_value(&block)
            .unwrap()
            .get("dictionary")
            .is_none());
        assert!(serde_json::to_value(&block)
            .unwrap()
            .get("committed_query_scope")
            .is_none());
        block.dictionary = Some(dictionary);
        let bytes = serde_json::to_vec(&block).unwrap();
        let restored: CurrentQueryHandoff = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored, block);
        assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    }
}

#[cfg(test)]
mod committed_scope_tests {
    use super::super::value_lexemes::WordAtom;
    use super::*;
    fn word(text: &str, token_end: u64, byte_end: u64) -> WordAtom {
        let mut word = WordAtom {
            len: text.len() as u8,
            end: token_end,
            byte_end,
            ..Default::default()
        };
        word.bytes[..text.len()].copy_from_slice(text.as_bytes());
        word
    }
    fn words(entries: &[WordAtom]) -> LexemeState {
        let mut words = LexemeState::default();
        words.queries[..entries.len()].copy_from_slice(entries);
        words.recent = words.queries;
        words.query_len = entries.len();
        words.recent_len = entries.len();
        words
    }
    #[test]
    fn current_query_committed_scope_keeps_later_words_in_same_token_and_span_terminal() {
        let source = words(&[
            word("Answer", 5, 50),
            word("What", 5, 40),
            word("tail", 5, 30),
            word("owner", 5, 10),
        ]);
        let mut state = RelationState::default();
        state.records[0] = RelationRecord {
            id: 1,
            owner: word("owner", 5, 10),
            value: word("fact", 5, 20),
            span: Some(super::super::relation_span::RelationSpan {
                bytes: [0; 28],
                len: 9,
                extra_words: 1,
                terminal: word("tail", 5, 30),
                start: None,
            }),
            ..Default::default()
        };
        let before = source;
        let mut work = WordCopyWork::default();
        let (view, cutoff) = scoped_words(&source, None, Some(&state), true, &mut work);
        assert_eq!(cutoff, Some(30));
        assert_eq!(view.query_len, 2);
        assert_eq!(&view.queries[..2], &source.queries[..2]);
        assert_eq!(view.recent, source.recent);
        assert_eq!(source, before);
        assert_eq!(
            work.persistent_read.relations.record_reads,
            state.records.len() as u64
        );
        assert!(work.routing.comparisons > state.records.len() as u64);
    }
    #[test]
    fn current_query_committed_scope_includes_noncurrent_records_and_ignores_spelling() {
        let mut state = RelationState::default();
        state.records[0] = RelationRecord {
            id: 1,
            owner: word("a", 2, 10),
            value: word("b", 2, 20),
            ..Default::default()
        };
        state.records[1] = RelationRecord {
            id: 2,
            owner: word("c", 3, 40),
            value: word("d", 3, 50),
            conflict: true,
            ..Default::default()
        };
        state.directory[0] = 1;
        let source = words(&[
            word("current", 4, 60),
            word("current", 3, 50),
            word("What", 2, 20),
        ]);
        let (view, cutoff) =
            scoped_words(&source, None, Some(&state), true, &mut Default::default());
        assert_eq!((view.query_len, cutoff), (1, Some(50)));
        let mut renamed = source;
        renamed.queries[0] = word("previous", 4, 60);
        renamed.queries[1] = word("Answer", 3, 50);
        let (other, other_cutoff) =
            scoped_words(&renamed, None, Some(&state), true, &mut Default::default());
        assert_eq!((other.query_len, other_cutoff), (view.query_len, cutoff));
        let (legacy, cutoff) =
            scoped_words(&source, None, Some(&state), false, &mut Default::default());
        assert_eq!(legacy, source);
        assert_eq!(cutoff, None);
    }
    #[test]
    fn current_query_committed_scope_keeps_existing_boundary_and_handles_no_records() {
        let source = words(&[
            word("new", 10, 100),
            word("turn", 9, 90),
            word("old", 8, 80),
        ]);
        let empty = RelationState::default();
        for records in [None, Some(&empty)] {
            let (view, cutoff) =
                scoped_words(&source, Some(9), records, true, &mut Default::default());
            assert_eq!((view.query_len, cutoff), (2, None));
        }
        let mut state = empty;
        state.records[0] = RelationRecord {
            id: 1,
            owner: word("fact", 1, 20),
            value: word("value", 2, 30),
            ..Default::default()
        };
        let (view, _) = scoped_words(
            &source,
            Some(9),
            Some(&state),
            true,
            &mut Default::default(),
        );
        assert_eq!(view.query_len, 2);
        state.records[0].value.byte_end = u64::MAX;
        let (view, cutoff) =
            scoped_words(&source, None, Some(&state), true, &mut Default::default());
        assert_eq!((view.query_len, cutoff), (0, Some(u64::MAX)));
    }
    #[test]
    fn current_query_source_byte_ordinal_survives_turn_end() {
        let mut source = words(&[word("fact", 5, 50)]);
        source.source_bytes_seen = 51;
        source.end();
        assert_eq!(source.source_bytes_seen, 51);
        assert_eq!(source.recent[0].byte_end, 50);
        assert_eq!(source.query_len, 0);
    }
}
