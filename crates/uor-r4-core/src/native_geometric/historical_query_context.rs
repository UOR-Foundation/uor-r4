//! Artifact-bound wider historical query context; the complete inner model is frozen.
use super::historical_read::{self, HistoricalRead};
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{learn_code_subset, Alternative, Frame};
use super::value_types::ValueFeature;
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoricalQueryExample {
    pub id: String,
    pub prompt: String,
    /// Exact current record whose immediate predecessor is the offline target.
    /// With inherit=false, None explicitly labels historical Base/defer.
    pub current_record: Option<u64>,
    /// Preserve the actual parent historical choice; current_record must be None.
    pub inherit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HistoricalQueryContext {
    pub parent_artifact: String,
    pub active: HistoricalRead,
    pub config: SourceRoutingConfig,
    /// Empty only for an exposure-only artifact with byte-exact inherited parameters.
    pub training: Vec<DocumentReceipt>,
}

fn frozen(active: &HistoricalRead, previous: &HistoricalRead) -> bool {
    if previous.router.codes.iter().any(|old| {
        active
            .router
            .codes
            .binary_search_by_key(&old.feature, |c| c.feature)
            .is_err()
    }) {
        return false;
    }
    let mut restored = active.clone();
    restored.router.codes = previous.router.codes.clone();
    restored == *previous
}

fn config_valid(config: &SourceRoutingConfig, previous: &SourceRouting) -> Result<()> {
    config.validate()?;
    if config.learned_features > previous.config.learned_features
        || config.mode != previous.config.mode
        || config.role_context_only != previous.config.role_context_only
    {
        return Err(Error(
            "historical query configuration changed frozen capacity or mode".into(),
        ));
    }
    Ok(())
}

impl HistoricalQueryContext {
    fn parent(&self, model: &Model) -> Result<Model> {
        let mut parent = model.clone();
        parent.historical_query_context = None;
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("historical query frozen parent differs".into()));
        }
        parent.validate()?;
        Ok(parent)
    }

    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        self.parent(model)?;
        let previous = model
            .historical_read
            .as_ref()
            .ok_or_else(|| Error("historical query inner reader absent".into()))?;
        config_valid(&self.config, &previous.router)?;
        self.active.validate(model)?;
        if previous.query_scope != 2
            || !frozen(&self.active, previous)
            || self.active.router.codes.len() > self.config.learned_features
            || (self.training.is_empty()
                && (self.active != *previous || self.config != previous.router.config))
        {
            return Err(Error("historical query changed frozen parameters".into()));
        }
        if self.training.len() > 4096
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
            return Err(Error("invalid historical query training receipts".into()));
        }
        let mut duplicate = model.clone();
        duplicate.refresh_identity()?;
        if duplicate.artifact_cid != model.artifact_cid
            || duplicate.uor_model_address != model.uor_model_address
        {
            return Err(Error("historical query identity differs".into()));
        }
        Ok(())
    }
}

impl Model {
    pub fn without_historical_query_context(&self) -> Result<Model> {
        match &self.historical_query_context {
            Some(witness) => witness.parent(self),
            None => Ok(self.clone()),
        }
    }

    /// Expose all sixteen captured query words with unchanged inherited roots.
    /// This constructor provides an intervention artifact, not a quality claim.
    pub fn with_historical_query_context(&self) -> Result<Model> {
        self.validate()?;
        if self.historical_query_context.is_some() {
            return Err(Error(
                "historical query intervention already present".into(),
            ));
        }
        let previous = self
            .historical_read
            .as_ref()
            .ok_or_else(|| Error("historical query inner reader absent".into()))?;
        if previous.query_scope != 2 {
            return Err(Error("historical query requires local query scope".into()));
        }
        let mut model = self.clone();
        model.historical_query_context = Some(HistoricalQueryContext {
            parent_artifact: self.artifact_cid().to_owned(),
            active: previous.clone(),
            config: previous.router.config.clone(),
            training: Vec::new(),
        });
        model.refresh_identity()?;
        model.validate()?;
        Ok(model)
    }

    /// Offline exact-record/parent-choice labels refine only historical code roots.
    /// Response bytes and writer changes are not accepted as supervision.
    pub fn fit_historical_query_context(
        &self,
        docs: &[HistoricalQueryExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        let previous = self
            .historical_read
            .as_ref()
            .ok_or_else(|| Error("historical query inner reader absent".into()))?;
        config_valid(&config, &previous.router)?;
        if self.historical_query_context.is_some()
            || previous.query_scope != 2
            || docs.is_empty()
            || docs.len() > 4096
            || docs.iter().any(|d| {
                d.prompt.is_empty()
                    || d.prompt.len() > 65536
                    || d.current_record == Some(0)
                    || (d.inherit && d.current_record.is_some())
            })
            || docs.iter().map(|d| d.prompt.len()).sum::<usize>() > 8 * 1024 * 1024
        {
            return Err(Error(
                "invalid historical query fit bounds or labels".into(),
            ));
        }
        let start = Instant::now();
        let (defer, copy) = historical_read::action_indices(self)
            .ok_or_else(|| Error("historical query actions ambiguous".into()))?;
        let mut active = previous.clone();
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut frames = Vec::new();
        let mut seen = BTreeMap::new();
        let mut vocabulary: BTreeSet<ValueFeature> =
            previous.router.codes.iter().map(|c| c.feature).collect();
        let mut labels = Vec::new();
        let mut skipped = Vec::new();
        let mut duplicates = 0usize;
        for d in docs {
            if start.elapsed().as_secs() >= config.max_seconds {
                return Err(Error("historical query preparation time limit".into()));
            }
            if d.id.trim().is_empty() || !ids.insert(&d.id) {
                return Err(Error(
                    "historical query duplicate or empty document identity".into(),
                ));
            }
            receipts.push(super::training::receipt(&Document {
                id: d.id.clone(),
                text: serde_json::to_string(d).map_err(|e| Error(e.to_string()))?,
            }));
            let tokens = self.encode(&d.prompt)?;
            if tokens.len() > 8192 {
                return Err(Error(format!(
                    "historical query input token bound: {}",
                    d.id
                )));
            }
            let mut session = self.session(Control::Full)?;
            session.observe(self, BOS)?;
            for token in tokens {
                if start.elapsed().as_secs() >= config.max_seconds {
                    return Err(Error("historical query preparation time limit".into()));
                }
                session.observe(self, token)?;
            }
            session.begin_response(self)?;
            // Match actual response-entry eligibility, including higher-priority operators.
            session.predict(self)?;
            let values = session
                .values
                .as_ref()
                .ok_or_else(|| Error("historical query values absent".into()))?;
            let entry = session
                .response_entry
                .as_ref()
                .ok_or_else(|| Error("historical query entry absent".into()))?;
            if !super::word_copy_runtime::eligible(self, entry, values, Control::Full) {
                if d.current_record.is_some() {
                    return Err(Error(format!(
                        "historical query target ineligible: {}",
                        d.id
                    )));
                }
                skipped.push(d.id.clone());
                continue;
            }
            let parent_choice =
                historical_read::choose(self, values, Control::Full, &mut Default::default());
            let state = values
                .relations
                .as_ref()
                .ok_or_else(|| Error("historical query records absent".into()))?;
            let mut inherited_current = None;
            if d.inherit {
                if let Some((source, action)) = parent_choice {
                    for &id in &state.directory {
                        let Some(current) = state.record(id) else {
                            continue;
                        };
                        let Some(old) =
                            historical_read::previous(state, current, &mut Default::default())
                        else {
                            continue;
                        };
                        let old_source =
                            super::relation::RELATION_SOURCE + ((old.id - 1) & 15) as u8;
                        if source == old_source
                            && action == copy
                            && historical_read::representable(
                                self,
                                values,
                                old,
                                copy,
                                &mut Default::default(),
                            )
                        {
                            if inherited_current.replace(id).is_some() {
                                return Err(Error(format!(
                                    "historical query ambiguous inherited source: {}",
                                    d.id
                                )));
                            }
                        }
                    }
                    if inherited_current.is_none() {
                        return Err(Error(format!(
                            "historical query inherited source unavailable: {}",
                            d.id
                        )));
                    }
                }
            }
            let target = if d.inherit {
                inherited_current
            } else {
                d.current_record
            };
            let addr = historical_read::addresses(&active, values, &mut Default::default());
            let mut alternatives = vec![Alternative {
                features: Vec::new(),
                codes: Vec::new(),
                action: defer,
                correct: target.is_none(),
            }];
            let mut previous_id = None;
            for &id in &state.directory {
                let Some(current) = state.record(id) else {
                    continue;
                };
                let Some(old) = historical_read::previous(state, current, &mut Default::default())
                else {
                    continue;
                };
                if !historical_read::representable(self, values, old, copy, &mut Default::default())
                {
                    continue;
                }
                let (features, n) = historical_read::features_with_window(
                    self,
                    values,
                    current,
                    &addr,
                    true,
                    16,
                    &mut Default::default(),
                );
                if target == Some(id) {
                    previous_id = Some(old.id);
                }
                alternatives.push(Alternative {
                    features: features[..n].to_vec(),
                    codes: Vec::new(),
                    action: copy,
                    correct: target == Some(id),
                });
            }
            if alternatives.iter().filter(|a| a.correct).count() != 1 {
                return Err(Error(format!(
                    "historical query exact target unavailable: {}",
                    d.id
                )));
            }
            labels.push(serde_json::json!({"id":d.id,"inherit":d.inherit,
                "current_record":target,"previous_record":previous_id,"parent_choice":parent_choice}));
            if alternatives.len() == 1 {
                skipped.push(d.id.clone());
                continue;
            }
            let signature: Vec<_> = alternatives
                .iter()
                .map(|a| (a.features.clone(), a.action))
                .collect();
            let target_index = alternatives
                .iter()
                .position(|a| a.correct)
                .ok_or_else(|| Error("historical query target absent".into()))?;
            if let Some((prior_target, prior_id)) = seen.get(&signature) {
                if *prior_target != target_index {
                    return Err(Error(format!(
                        "historical query conflicting exact frames: {} and {}",
                        prior_id, d.id
                    )));
                }
                duplicates += 1;
                continue;
            }
            seen.insert(signature, (target_index, d.id.clone()));
            for a in &alternatives {
                vocabulary.extend(a.features.iter().copied());
            }
            if vocabulary.len() > config.learned_features {
                return Err(Error(format!(
                    "historical query feature bound: {} > {}",
                    vocabulary.len(),
                    config.learned_features
                )));
            }
            if frames.len() >= 4096 {
                return Err(Error("historical query frame cap4096".into()));
            }
            frames.push(Frame { alternatives });
        }
        if frames.is_empty() {
            return Err(Error(
                "historical query has no eligible competitive frames".into(),
            ));
        }
        active.router.codes = vocabulary
            .into_iter()
            .map(|feature| {
                previous
                    .router
                    .codes
                    .binary_search_by_key(&feature, |c| c.feature)
                    .map(|i| previous.router.codes[i].clone())
                    .unwrap_or(SourceCode {
                        feature,
                        roots: [self.geometry.identity; 2],
                    })
            })
            .collect();
        let previous_config = active.router.config.clone();
        active.router.config = config.clone();
        let mutable = vec![true; active.router.codes.len()];
        let fit = learn_code_subset(self, &mut active.router, &mut frames, start, &mutable)?;
        active.router.config = previous_config;
        let feature_count = active.router.codes.len();
        let mut model = self.clone();
        model.historical_query_context = Some(HistoricalQueryContext {
            parent_artifact: self.artifact_cid().to_owned(),
            active,
            config,
            training: receipts,
        });
        model.refresh_identity()?;
        model.validate()?;
        let report = serde_json::json!({"schema":"uor-r4.historical-query-context-fit/1",
            "parent":self.artifact_cid(),"artifact":model.artifact_cid(),"documents":docs.len(),
            "frames":frames.len(),"duplicate_frames":duplicates,"upstream_skipped":skipped,
            "labels":labels,"features":feature_count,"query_window":16,"parent_query_window":8,
            "fit":fit,"elapsed_ms":start.elapsed().as_millis(),
            "scope":"Exact parent historical selection or offline immediate-previous targets. Only active historical code roots learned; all inner model bytes and historical non-code parameters frozen. Generated behavior requires separate evaluation."});
        Ok((model, report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn block() -> HistoricalRead {
        HistoricalRead {
            query_scope: 2,
            dictionary: Vec::new(),
            router: SourceRouting {
                schema: "uor-r4.geometric-source-routing/1".into(),
                parent_artifact: "blake3:inner".into(),
                codes: vec![SourceCode {
                    feature: ValueFeature {
                        kind: 0,
                        a: 1,
                        b: 0,
                    },
                    roots: [1, 2],
                }],
                landmarks: vec![[3, 4]],
                biases: vec![0],
                ranks: Vec::new(),
                training: Vec::new(),
                config: SourceRoutingConfig::default(),
            },
        }
    }
    #[test]
    fn historical_query_context_freezes_all_noncode_fields_and_old_keys() {
        let previous = block();
        let mut active = previous.clone();
        active.router.codes[0].roots = [4, 5];
        assert!(frozen(&active, &previous));
        active.router.codes.push(SourceCode {
            feature: ValueFeature {
                kind: 6,
                a: 421,
                b: 0,
            },
            roots: [1, 1],
        });
        assert!(frozen(&active, &previous));
        active.router.biases[0] = 1;
        assert!(!frozen(&active, &previous));
        active.router.biases[0] = 0;
        active.query_scope = 1;
        assert!(!frozen(&active, &previous));
        active.query_scope = 2;
        active.router.codes.remove(0);
        assert!(!frozen(&active, &previous));
    }
    #[test]
    fn historical_query_context_wire_preserves_inner_and_rejects_answer_labels() {
        let previous = block();
        let before = serde_json::to_vec(&previous).unwrap();
        let witness = HistoricalQueryContext {
            parent_artifact: "blake3:complete-parent".into(),
            active: previous.clone(),
            config: previous.router.config.clone(),
            training: Vec::new(),
        };
        let wire = serde_json::to_value(&witness).unwrap();
        let restored: HistoricalQueryContext = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(restored, witness);
        assert_eq!(serde_json::to_vec(&restored.active).unwrap(), before);
        let mut unknown = wire;
        unknown["query_window"] = serde_json::json!(32);
        assert!(serde_json::from_value::<HistoricalQueryContext>(unknown).is_err());
        let d = HistoricalQueryExample {
            id: "target".into(),
            prompt: "prompt".into(),
            current_record: Some(3),
            inherit: false,
        };
        let mut label = serde_json::to_value(d).unwrap();
        label["response"] = serde_json::json!("answer");
        assert!(serde_json::from_value::<HistoricalQueryExample>(label).is_err());
    }
}
