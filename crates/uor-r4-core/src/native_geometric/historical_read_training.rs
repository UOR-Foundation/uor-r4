//! Offline exact previous-link labels; expected response bytes are never inputs.
use super::historical_read::{self, HistoricalRead};
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{learn, Alternative, Frame};
use super::value_types::ValueFeature;
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoricalReadExample {
    pub id: String,
    pub prompt: String,
    /// Offline current record identity whose immediate previous record is wanted.
    /// None teaches Base/defer to the complete unchanged parent.
    pub current_record: Option<u64>,
}

impl HistoricalRead {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        self.router.config.validate()?;
        if !matches!(self.query_scope, 1 | 2) {
            return Err(Error("invalid historical query scope".into()));
        }
        let read = super::role_read::head(model)
            .ok_or_else(|| Error("historical reader absent".into()))?;
        let (_, copy) = historical_read::action_indices(model)
            .ok_or_else(|| Error("historical read actions ambiguous".into()))?;
        let prefix = read.actions[copy]
            .prefix
            .ok_or_else(|| Error("historical read Prepare prefix absent".into()))?;
        if model.decode(&[prefix])? != b" " {
            return Err(Error(
                "historical read requires space Prepare action".into(),
            ));
        }
        if self.dictionary.is_empty()
            || self.dictionary.len() > 256
            || self.dictionary != read.dictionary
        {
            return Err(Error("invalid historical read dictionary bound".into()));
        }
        let expected_primes =
            crate::corpus_induced_spin_placement::first_primes(self.dictionary.len())
                .map_err(|e| Error(e.to_string()))?;
        if model.source_routing.is_none()
            || super::relation::head(model).is_none()
            || self.dictionary.is_empty()
            || self.dictionary.len() > 256
            || self.dictionary.iter().zip(expected_primes).any(|(w, p)| {
                w.len == 0
                    || w.len > 32
                    || u64::from(w.prime) != p
                    || w.bytes[..usize::from(w.len)]
                        .iter()
                        .any(|b| !b.is_ascii_alphanumeric() && *b != b'_')
                    || w.bytes[usize::from(w.len)..].iter().any(|b| *b != 0)
            })
            || self
                .dictionary
                .windows(2)
                .any(|w| w[0].bytes[..usize::from(w[0].len)] >= w[1].bytes[..usize::from(w[1].len)])
        {
            return Err(Error("invalid historical read dictionary".into()));
        }
        let r = &self.router;
        let primes: BTreeSet<_> = std::iter::once(0)
            .chain(self.dictionary.iter().map(|d| u64::from(d.prime)))
            .collect();
        let valid_feature = |f: ValueFeature| match f.kind {
            0 | 2 => f.a == 1 && f.b == 0,
            1 => f.a <= 1 && f.b == 0,
            3 | 7 => primes.contains(&f.a) && primes.contains(&f.b),
            4 | 5 => f.a <= 1 && primes.contains(&f.b),
            6 => primes.contains(&f.a) && f.b == 0,
            _ => false,
        };
        if r.schema != "uor-r4.geometric-source-routing/1"
            || !r.parent_artifact.starts_with("blake3:")
            || r.codes.is_empty()
            || r.codes.len() > r.config.learned_features
            || r.codes.windows(2).any(|c| c[0].feature >= c[1].feature)
            || r.codes
                .iter()
                .any(|c| !valid_feature(c.feature) || c.roots.iter().any(|root| *root >= 120))
            || r.landmarks.len() != read.actions.len()
            || r.landmarks.iter().flatten().any(|root| *root >= 120)
            || r.biases.len() != read.actions.len()
            || r.biases.iter().any(|b| !(-32..=32).contains(b))
            || r.ranks != super::learned_routing_training::ranks(model)
            || r.training.is_empty()
            || r.training.len() > 1500
            || r.training.iter().any(|d| {
                d.id.trim().is_empty() || d.bytes == 0 || !d.text_cid.starts_with("blake3:")
            })
            || r.training
                .iter()
                .map(|d| &d.id)
                .collect::<BTreeSet<_>>()
                .len()
                != r.training.len()
        {
            return Err(Error("invalid historical read router".into()));
        }
        Ok(())
    }
}

impl Model {
    pub fn without_historical_read(&self) -> Result<Model> {
        let Some(block) = &self.historical_read else {
            return Ok(self.clone());
        };
        let mut parent = self.clone();
        parent.historical_read = None;
        parent.refresh_identity()?;
        if parent.artifact_cid() != block.router.parent_artifact {
            return Err(Error("historical read frozen parent differs".into()));
        }
        parent.validate()?;
        Ok(parent)
    }

    pub fn fit_historical_read(
        &self,
        docs: &[HistoricalReadExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        config.validate()?;
        self.validate()?;
        if self.historical_read.is_some()
            || self.source_routing.is_none()
            || docs.is_empty()
            || docs.len() > 1500
            || docs.iter().any(|d| {
                d.prompt.is_empty() || d.prompt.len() > 65536 || d.current_record == Some(0)
            })
            || docs.iter().map(|d| d.prompt.len()).sum::<usize>() > 8 * 1024 * 1024
        {
            return Err(Error("invalid historical read fit bounds".into()));
        }
        let start = Instant::now();
        let read = super::role_read::head(self)
            .ok_or_else(|| Error("historical parent reader absent".into()))?;
        let dictionary = read.dictionary.clone();
        let (defer, copy) = historical_read::action_indices(self)
            .ok_or_else(|| Error("historical read actions ambiguous".into()))?;
        let prefix = read.actions[copy]
            .prefix
            .ok_or_else(|| Error("historical Prepare prefix absent".into()))?;
        if self.decode(&[prefix])? != b" " {
            return Err(Error(
                "historical read requires space Prepare action".into(),
            ));
        }
        let mut rng = config.seed;
        let mut block = HistoricalRead {
            dictionary,
            query_scope: 2,
            router: SourceRouting {
                schema: "uor-r4.geometric-source-routing/1".into(),
                parent_artifact: self.artifact_cid.clone(),
                codes: Vec::new(),
                landmarks: (0..read.actions.len())
                    .map(|_| {
                        std::array::from_fn(|_| {
                            (super::learned_routing_training::next_random(&mut rng) % 120) as u16
                        })
                    })
                    .collect(),
                biases: vec![0; read.actions.len()],
                ranks: super::learned_routing_training::ranks(self),
                training: Vec::new(),
                config,
            },
        };
        let mut frames = Vec::new();
        let mut frequency = BTreeMap::<ValueFeature, u64>::new();
        let mut ids = BTreeSet::new();
        let mut seen = BTreeMap::new();
        let mut skipped = Vec::new();
        let mut labels = Vec::new();
        let mut duplicates = 0;
        for d in docs {
            if start.elapsed().as_secs() >= block.router.config.max_seconds {
                return Err(Error("historical read preparation time limit".into()));
            }
            if d.id.trim().is_empty() || !ids.insert(&d.id) {
                return Err(Error(
                    "historical read duplicate/empty document identity".into(),
                ));
            }
            block
                .router
                .training
                .push(super::training::receipt(&Document {
                    id: d.id.clone(),
                    text: serde_json::to_string(d).map_err(|e| Error(e.to_string()))?,
                }));
            let mut session = self.session(Control::Full)?;
            session.observe(self, BOS)?;
            for token in self.encode(&d.prompt)? {
                session.observe(self, token)?;
            }
            session.begin_response(self)?;
            session.predict(self)?;
            let values = session
                .values
                .as_ref()
                .ok_or_else(|| Error("historical values absent".into()))?;
            let entry = session
                .response_entry
                .as_ref()
                .ok_or_else(|| Error("historical entry absent".into()))?;
            if !super::word_copy_runtime::eligible(self, entry, values, Control::Full) {
                if d.current_record.is_some() {
                    return Err(Error(format!("historical target ineligible: {}", d.id)));
                }
                skipped.push(d.id.clone());
                continue;
            }
            let state = values
                .relations
                .as_ref()
                .ok_or_else(|| Error("historical records absent".into()))?;
            let addr = historical_read::addresses(&block, values, &mut Default::default());
            let mut alternatives = vec![Alternative {
                features: Vec::new(),
                codes: Vec::new(),
                action: defer,
                correct: d.current_record.is_none(),
            }];
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
                let (f, n) = historical_read::features(
                    self,
                    values,
                    current,
                    &addr,
                    block.query_scope == 2,
                    &mut Default::default(),
                );
                let correct = d.current_record == Some(id);
                if correct {
                    labels.push(serde_json::json!({"id":d.id,"current_record":id,"previous_record":old.id,"source":super::relation::RELATION_SOURCE+((old.id-1)&15) as u8,"action":copy,"source_end":old.value.end,"source_byte_end":old.value.byte_end}));
                }
                alternatives.push(Alternative {
                    features: f[..n].to_vec(),
                    codes: Vec::new(),
                    action: copy,
                    correct,
                });
            }
            if alternatives.iter().filter(|a| a.correct).count() != 1 {
                return Err(Error(format!(
                    "historical exact current/previous target unavailable: {}",
                    d.id
                )));
            }
            let signature: Vec<_> = alternatives
                .iter()
                .map(|a| (a.features.clone(), a.action))
                .collect();
            let target = alternatives
                .iter()
                .position(|a| a.correct)
                .ok_or_else(|| Error("historical target absent".into()))?;
            if let Some((old_target, old_id)) = seen.get(&signature) {
                if *old_target != target {
                    return Err(Error(format!(
                        "historical conflicting exact frames: {} and {}",
                        old_id, d.id
                    )));
                }
                duplicates += 1;
                continue;
            }
            seen.insert(signature, (target, d.id.clone()));
            if frames.len() >= 4096 {
                return Err(Error("historical frame cap4096".into()));
            }
            for a in &alternatives {
                for &f in &a.features {
                    *frequency.entry(f).or_default() += 1;
                }
            }
            frames.push(Frame { alternatives });
        }
        if frames.is_empty() {
            return Err(Error("historical read has no eligible frames".into()));
        }
        let mut vocabulary: Vec<_> = frequency.into_iter().collect();
        vocabulary.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let universe = vocabulary.len();
        vocabulary.truncate(block.router.config.learned_features);
        vocabulary.sort_by_key(|v| v.0);
        block.router.codes = vocabulary
            .into_iter()
            .map(|(feature, _)| SourceCode {
                feature,
                roots: [self.geometry.identity; 2],
            })
            .collect();
        let fit = learn(self, &mut block.router, &mut frames, start);
        let mut model = self.clone();
        model.historical_read = Some(block);
        model.refresh_identity()?;
        model.validate()?;
        let artifact = model.artifact_cid().to_owned();
        Ok((
            model,
            serde_json::json!({"schema":"uor-r4.historical-read-fit/1","parent":self.artifact_cid(),"artifact":artifact,"documents":docs.len(),"frames":frames.len(),"upstream_skipped":skipped,"duplicate_frames":duplicates,"labels":labels,"dictionary_inherited_exactly":true,"query_scope":2,"feature_universe":universe,"fit":fit,"elapsed_ms":start.elapsed().as_millis(),"scope":"Offline exact immediate previous-link targets or Base preservation. Only new shared geometric router learned; inherited lexical dictionary, parent and serving payloads unchanged. Free generation and control preservation require separate checks."}),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn historical_read_labels_are_exact_optional_records_without_response_fields() {
        let x = HistoricalReadExample {
            id: "x".into(),
            prompt: "p".into(),
            current_record: Some(3),
        };
        let mut wire = serde_json::to_value(&x).unwrap();
        assert_eq!(
            serde_json::from_value::<HistoricalReadExample>(wire.clone()).unwrap(),
            x
        );
        wire["response"] = serde_json::json!("secret answer");
        assert!(serde_json::from_value::<HistoricalReadExample>(wire).is_err());
        assert!(serde_json::from_str::<HistoricalReadExample>(
            r#"{"id":"x","prompt":"p","current_record":1.5}"#
        )
        .is_err());
    }
    #[test]
    fn historical_read_witness_wire_binds_parent_router_without_payload_templates() {
        let b = HistoricalRead {
            dictionary: Vec::new(),
            query_scope: 1,
            router: SourceRouting {
                schema: "uor-r4.geometric-source-routing/1".into(),
                parent_artifact: "blake3:parent".into(),
                codes: Vec::new(),
                landmarks: Vec::new(),
                biases: Vec::new(),
                ranks: Vec::new(),
                training: Vec::new(),
                config: SourceRoutingConfig::default(),
            },
        };
        let bytes = serde_json::to_vec(&b).unwrap();
        let restored: HistoricalRead = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored, b);
        assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
        assert_eq!(restored.query_scope, 1);
        let mut modern = restored.clone();
        modern.query_scope = 2;
        let modern_wire = serde_json::to_value(&modern).unwrap();
        assert_eq!(modern_wire["query_scope"], 2);
        assert_eq!(
            serde_json::from_value::<HistoricalRead>(modern_wire).unwrap(),
            modern
        );
        let mut wire = serde_json::to_value(b).unwrap();
        assert_eq!(wire.as_object().unwrap().len(), 2);
        assert_eq!(wire["router"]["parent_artifact"], "blake3:parent");
        wire["response"] = serde_json::json!("template");
        assert!(serde_json::from_value::<HistoricalRead>(wire).is_err());
    }
}
