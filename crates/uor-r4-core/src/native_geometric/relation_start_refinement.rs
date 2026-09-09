//! Offline exact occurrence labels refine the existing reverse-start geometry.
//! This witness restores the entire parent; it adds no serving feature law.
use super::relation::RelationRecord;
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{learn_code_subset, Alternative, Frame};
use super::value_lexemes::LexemeState;
use super::value_types::ValueWork;
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationStartExample {
    pub id: String,
    pub prompt: String,
    pub overrides: Vec<RelationStartOverride>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationStartOverride {
    pub record_id: u64,
    pub owner_end: u64,
    pub owner_byte_end: u64,
    pub endpoint_end: u64,
    pub endpoint_byte_end: u64,
    pub start_end: u64,
    pub start_byte_end: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RelationStartRefinement {
    pub parent_artifact: String,
    pub previous: SourceRouting,
    pub config: SourceRoutingConfig,
    pub training: Vec<DocumentReceipt>,
}

fn config_valid(config: &SourceRoutingConfig) -> Result<()> {
    config.validate()?;
    if config.learned_features > 256
        || config.mode != RoutingMode::Angular
        || !config.role_context_only
    {
        return Err(Error(
            "invalid relation start refinement configuration".into(),
        ));
    }
    Ok(())
}

fn frozen(current: &SourceRouting, previous: &SourceRouting) -> bool {
    // All old keys survive; only their roots and additional legal keys may differ.
    if previous.codes.iter().any(|old| {
        current
            .codes
            .binary_search_by_key(&old.feature, |c| c.feature)
            .is_err()
    }) {
        return false;
    }
    let mut restored = current.clone();
    restored.codes = previous.codes.clone();
    restored == *previous
}

impl RelationStartRefinement {
    fn parent(&self, model: &Model) -> Result<Model> {
        let mut parent = model.clone();
        parent.relation_start_refinement = None;
        parent.relation_start = Some(self.previous.clone());
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error(
                "relation start refinement frozen parent differs".into(),
            ));
        }
        parent.validate()?;
        Ok(parent)
    }

    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        config_valid(&self.config)?;
        if self.training.is_empty()
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
            return Err(Error("invalid relation start refinement receipts".into()));
        }
        self.parent(model)?;
        let active = model
            .relation_start
            .as_ref()
            .ok_or_else(|| Error("refined relation start absent".into()))?;
        super::relation_start_training::validate(active, model)?;
        if !frozen(active, &self.previous) || active.codes.len() > self.config.learned_features {
            return Err(Error(
                "relation start refinement changed frozen parameters".into(),
            ));
        }
        let mut duplicate = model.clone();
        duplicate.refresh_identity()?;
        if duplicate.artifact_cid != model.artifact_cid
            || duplicate.uor_model_address != model.uor_model_address
        {
            return Err(Error("relation start refinement identity differs".into()));
        }
        Ok(())
    }
}

struct Event {
    record: RelationRecord,
    words: LexemeState,
    endpoint: usize,
    observed_tokens: u64,
}

/// Capture every reverse commit from actual parent observation. A tokenizer
/// piece that hides the exact event window is rejected, never approximated.
fn collect(model: &Model, prompt: &str, start: Instant, seconds: u64) -> Result<Vec<Event>> {
    if prompt.is_empty() || prompt.len() > 65536 {
        return Err(Error("invalid reverse-start prompt bound".into()));
    }
    let mut session = model.session(Control::Full)?;
    session.observe(model, BOS)?;
    let mut next_id = 1;
    let mut events = Vec::new();
    for token in model.encode(prompt)? {
        if start.elapsed().as_secs() >= seconds {
            return Err(Error("reverse-start preparation time limit".into()));
        }
        session.observe(model, token)?;
        let values = session
            .values
            .as_ref()
            .ok_or_else(|| Error("reverse-start values absent".into()))?;
        let state = values
            .relations
            .as_ref()
            .ok_or_else(|| Error("reverse-start relation state absent".into()))?;
        if state.next_id < next_id || state.next_id.saturating_sub(next_id) > 16 {
            return Err(Error("reverse-start commit window unavailable".into()));
        }
        for id in next_id..state.next_id {
            let record = *state
                .record(id)
                .ok_or_else(|| Error("reverse-start committed record unavailable".into()))?;
            if record.owner.byte_end <= record.value.byte_end {
                continue;
            }
            let words = values
                .lexemes
                .as_ref()
                .ok_or_else(|| Error("reverse-start lexemes absent".into()))?;
            if words.recent_len == 0 || words.recent[0] != record.owner {
                return Err(Error(format!(
                    "reverse-start exact commit window unavailable at record {id}"
                )));
            }
            let endpoint = words.recent[..words.recent_len]
                .iter()
                .position(|word| *word == record.value)
                .ok_or_else(|| Error("reverse-start endpoint occurrence absent".into()))?;
            events.push(Event {
                record,
                words: *words,
                endpoint,
                observed_tokens: values.seen,
            });
            if events.len() > 4096 {
                return Err(Error("reverse-start event cap4096".into()));
            }
        }
        next_id = state.next_id;
    }
    Ok(events)
}

fn frame(
    model: &Model,
    event: &Event,
    label: Option<&RelationStartOverride>,
) -> Result<(Frame, serde_json::Value)> {
    let record = &event.record;
    let selected = record
        .span
        .as_ref()
        .and_then(|s| s.start.as_ref())
        .unwrap_or(&record.value);
    let target = if let Some(label) = label {
        if label.record_id != record.id
            || label.owner_end != record.owner.end
            || label.owner_byte_end != record.owner.byte_end
            || label.endpoint_end != record.value.end
            || label.endpoint_byte_end != record.value.byte_end
        {
            return Err(Error(format!(
                "reverse-start override identity differs at record {}",
                record.id
            )));
        }
        (label.start_end, label.start_byte_end)
    } else {
        (selected.end, selected.byte_end)
    };
    let router = model
        .relation_start
        .as_ref()
        .ok_or_else(|| Error("reverse-start selector absent".into()))?;
    let admitted = super::relation_span::reverse_candidates(
        model,
        &event.words,
        0,
        event.endpoint,
        record.action,
        &mut ValueWork::default(),
    );
    let mut alternatives = Vec::new();
    let mut candidates = Vec::new();
    let mut winning = None;
    let mut best = i64::MIN;
    for candidate in admitted.rows[..admitted.len].iter().rev().copied() {
        let first = event.words.recent[candidate.start];
        let (features, count) = super::relation_start::contextual_features(
            model.relation_start_context.as_deref(),
            &event.words,
            0,
            event.endpoint,
            candidate,
            record.action,
            &mut ValueWork::default(),
        );
        let features = &features[..count];
        let state = router.encode(model, features, Control::Full, &mut RoutingWork::default());
        let score = router.score(model, state, 0, &mut RoutingWork::default());
        if score > best {
            best = score;
            winning = Some((first.end, first.byte_end));
        }
        let payload = super::relation_span::payload(&record.value, candidate.span.as_ref())
            .ok_or_else(|| Error("reverse-start candidate payload invalid".into()))?;
        let correct = (first.end, first.byte_end) == target;
        alternatives.push(Alternative {
            features: features.to_vec(),
            codes: Vec::new(),
            action: 0,
            correct,
        });
        let mapped: Vec<_> = features.iter().map(|f| {
            let code = router.codes.binary_search_by_key(f, |c| c.feature).ok();
            serde_json::json!({"feature":f,"code_index":code,"roots":code.map(|i|router.codes[i].roots)})
        }).collect();
        candidates.push(
            serde_json::json!({"start_end":first.end,"start_byte_end":first.byte_end,
            "start":first,"payload":String::from_utf8_lossy(payload),"features":features,
            "mapped_codes":mapped,"state":state,"score":score,
            "selected":first==*selected,"correct":correct}),
        );
    }
    if winning != Some((selected.end, selected.byte_end)) {
        return Err(Error(format!(
            "reverse-start trace differs from actual commit at record {}",
            record.id
        )));
    }
    if alternatives.iter().filter(|a| a.correct).count() != 1 {
        return Err(Error(format!(
            "reverse-start exact target not uniquely admitted at record {}",
            record.id
        )));
    }
    let excluded: Vec<_> = (event.endpoint + 1..event.words.recent_len)
        .filter(|i| !admitted.rows[..admitted.len].iter().any(|c|c.start==*i))
        .map(|i| serde_json::json!({"start":event.words.recent[i],
            "reason":"not admitted by unchanged extent/adjacency/learned-edge law; no ranking score"})).collect();
    Ok((
        Frame { alternatives },
        serde_json::json!({"record_id":record.id,"action":record.action,
        "observed_tokens":event.observed_tokens,"owner":record.owner,"endpoint":record.value,
        "committed_span":record.span,"selected_start_end":selected.end,"selected_start_byte_end":selected.byte_end,
        "override":label,"candidates":candidates,"not_admitted":excluded}),
    ))
}

impl Model {
    pub fn without_relation_start_refinement(&self) -> Result<Model> {
        match &self.relation_start_refinement {
            Some(w) => w.parent(self),
            None => Ok(self.clone()),
        }
    }

    /// Allocating offline diagnostic. It observes actual input commits, then
    /// reports the exact existing admission/features/scores in runtime tie order.
    pub fn relation_start_trace(&self, prompt: &str) -> Result<serde_json::Value> {
        let events = collect(self, prompt, Instant::now(), 120)?;
        let traces = events
            .iter()
            .map(|e| frame(self, e, None).map(|(_, t)| t))
            .collect::<Result<Vec<_>>>()?;
        Ok(
            serde_json::json!({"artifact":self.artifact_cid(),"prompt":prompt,"events":traces,
            "scope":"Actual observed reverse commits and unchanged admission/score law. Target bytes are not used."}),
        )
    }

    pub fn fit_relation_start_refinement(
        &self,
        docs: &[RelationStartExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        config_valid(&config)?;
        if self.relation_start_refinement.is_some()
            || docs.is_empty()
            || docs.len() > 2048
            || docs
                .iter()
                .any(|d| d.prompt.is_empty() || d.prompt.len() > 65536 || d.overrides.len() > 4096)
            || docs.iter().map(|d| d.prompt.len()).sum::<usize>() > 8 * 1024 * 1024
        {
            return Err(Error("invalid reverse-start refinement bounds".into()));
        }
        let previous = self
            .relation_start
            .as_ref()
            .ok_or_else(|| Error("reverse-start parent selector absent".into()))?
            .clone();
        let start = Instant::now();
        let mut ids = BTreeSet::new();
        let mut seen = BTreeMap::new();
        let mut frames = Vec::new();
        let mut traces = Vec::new();
        let mut training = Vec::new();
        let mut vocabulary: BTreeSet<_> = previous.codes.iter().map(|c| c.feature).collect();
        let mut duplicates = 0;
        let mut event_count = 0;
        for doc in docs {
            if doc.id.trim().is_empty() || !ids.insert(&doc.id) {
                return Err(Error("reverse-start duplicate/empty document id".into()));
            }
            let labels: BTreeMap<_, _> = doc.overrides.iter().map(|l| (l.record_id, l)).collect();
            if labels.len() != doc.overrides.len() || labels.contains_key(&0) {
                return Err(Error(format!(
                    "reverse-start duplicate/zero override: {}",
                    doc.id
                )));
            }
            let events = collect(self, &doc.prompt, start, config.max_seconds)?;
            event_count += events.len();
            if event_count > 32768 {
                return Err(Error("reverse-start event population cap32768".into()));
            }
            let mut used = BTreeSet::new();
            for event in events {
                let label = labels.get(&event.record.id).copied();
                if label.is_some() {
                    used.insert(event.record.id);
                }
                let (f, trace) = frame(self, &event, label)?;
                let signature: Vec<_> = f
                    .alternatives
                    .iter()
                    .map(|a| (a.features.clone(), a.action))
                    .collect();
                let target = f
                    .alternatives
                    .iter()
                    .position(|a| a.correct)
                    .ok_or_else(|| Error("reverse-start target absent".into()))?;
                if let Some((prior_target, prior_id)) = seen.get(&signature) {
                    if *prior_target != target {
                        return Err(Error(format!(
                            "reverse-start contradictory exact frames: {prior_id} and {}",
                            doc.id
                        )));
                    }
                    duplicates += 1;
                } else {
                    if frames.len() >= 32768 {
                        return Err(Error("reverse-start distinct frame cap32768".into()));
                    }
                    vocabulary.extend(
                        f.alternatives
                            .iter()
                            .flat_map(|a| a.features.iter().copied()),
                    );
                    seen.insert(signature, (target, doc.id.clone()));
                    frames.push(f);
                }
                traces.push(serde_json::json!({"id":doc.id,"event":trace}));
            }
            if used.len() != labels.len() {
                return Err(Error(format!(
                    "reverse-start unobserved override: {}",
                    doc.id
                )));
            }
            training.push(super::training::receipt(&Document {
                id: doc.id.clone(),
                text: serde_json::to_string(doc).map_err(|e| Error(e.to_string()))?,
            }));
        }
        if frames.is_empty()
            || vocabulary.len() > config.learned_features
            || vocabulary.len() > previous.config.learned_features
        {
            return Err(Error(
                "reverse-start frame/feature capacity unavailable".into(),
            ));
        }
        let mut active = previous.clone();
        active.codes = vocabulary
            .into_iter()
            .map(|feature| {
                previous
                    .codes
                    .binary_search_by_key(&feature, |c| c.feature)
                    .ok()
                    .map(|i| previous.codes[i].clone())
                    .unwrap_or(SourceCode {
                        feature,
                        roots: [self.geometry.identity; 2],
                    })
            })
            .collect();
        super::relation_start_training::validate(&active, self)?;
        active.config = config.clone();
        let mask = vec![true; active.codes.len()];
        let fit = learn_code_subset(self, &mut active, &mut frames, start, &mask)?;
        active.config = previous.config.clone();
        let mut model = self.clone();
        model.relation_start = Some(active);
        model.relation_start_refinement = Some(RelationStartRefinement {
            parent_artifact: self.artifact_cid.clone(),
            previous,
            config,
            training,
        });
        model.refresh_identity()?;
        model.validate()?;
        let report = serde_json::json!({"schema":"uor-r4.relation-start-refinement/1",
            "parent_artifact":self.artifact_cid(),"artifact":model.artifact_cid(),"documents":docs.len(),
            "events":event_count,"frames":frames.len(),"duplicate_frames":duplicates,
            "features":model.relation_start.as_ref().map(|b|b.codes.len()),"fit":fit,"construction":traces,
            "elapsed_ms":start.elapsed().as_millis(),
            "scope":"Only existing reverse-start code roots learned from exact start labels or actual parent-winner preservation. Admission, dictionary, feature law and all other parent parameters frozen; free behavior requires separate evaluation."});
        Ok((model, report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn router() -> SourceRouting {
        SourceRouting {
            schema: "uor-r4.geometric-source-routing/1".into(),
            parent_artifact: "blake3:parent".into(),
            codes: vec![SourceCode {
                feature: super::super::value_types::ValueFeature {
                    kind: 0,
                    a: 1,
                    b: 0,
                },
                roots: [0, 0],
            }],
            landmarks: vec![[1, 2]],
            biases: vec![0],
            ranks: vec![1],
            training: vec![],
            config: SourceRoutingConfig::default(),
        }
    }
    #[test]
    fn relation_start_refinement_freezes_noncode_fields_and_keeps_inherited_keys() {
        let previous = router();
        let mut active = previous.clone();
        active.codes[0].roots = [3, 4];
        assert!(frozen(&active, &previous));
        active.codes.push(SourceCode {
            feature: super::super::value_types::ValueFeature {
                kind: 1,
                a: 1,
                b: 0,
            },
            roots: [0, 0],
        });
        assert!(frozen(&active, &previous));
        let mut bad = active.clone();
        bad.codes.remove(0);
        assert!(!frozen(&bad, &previous));
        let mut bad = active.clone();
        bad.landmarks[0][0] = 3;
        assert!(!frozen(&bad, &previous));
        let mut bad = active.clone();
        bad.biases[0] = 1;
        assert!(!frozen(&bad, &previous));
        let mut bad = active.clone();
        bad.config.seed += 1;
        assert!(!frozen(&bad, &previous));
        let mut bad = active;
        bad.parent_artifact.push('x');
        assert!(!frozen(&bad, &previous));
    }
    #[test]
    fn relation_start_refinement_witness_roundtrip_keeps_previous_router() {
        let witness = RelationStartRefinement {
            parent_artifact: "blake3:whole".into(),
            previous: router(),
            config: SourceRoutingConfig::default(),
            training: vec![],
        };
        let bytes = serde_json::to_vec(&witness).unwrap();
        let restored: RelationStartRefinement = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored, witness);
        assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
        let mut wire = serde_json::to_value(witness).unwrap();
        wire["response"] = serde_json::json!("bad");
        assert!(serde_json::from_value::<RelationStartRefinement>(wire).is_err());
    }
    #[test]
    fn relation_start_refinement_label_schema_rejects_response_and_imprecise_identity() {
        let doc = RelationStartExample {
            id: "x".into(),
            prompt: "p".into(),
            overrides: vec![],
        };
        let mut wire = serde_json::to_value(&doc).unwrap();
        assert_eq!(
            serde_json::from_value::<RelationStartExample>(wire.clone()).unwrap(),
            doc
        );
        wire["response"] = serde_json::json!("answer");
        assert!(serde_json::from_value::<RelationStartExample>(wire).is_err());
        assert!(
            serde_json::from_value::<RelationStartOverride>(serde_json::json!({
            "record_id":1.5,"owner_end":1,"owner_byte_end":1,"endpoint_end":0,
            "endpoint_byte_end":0,"start_end":0,"start_byte_end":0}))
            .is_err()
        );
    }
}
