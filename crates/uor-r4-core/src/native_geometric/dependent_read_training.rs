//! Construction-only source/operator labels; no query interpreter is deployed.
use super::dependent_read::{self, DependentRead};
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{learn, Alternative, Frame};
use super::value_types::ValueFeature;
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependentReadExample {
    pub example: ValueExample,
    /// Construction-only first owner label; None teaches defer-to-parent.
    pub via: Option<String>,
}

impl DependentRead {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        self.router.validate(model)?;
        let primes = crate::corpus_induced_spin_placement::first_primes(self.dictionary.len())
            .map_err(|e| Error(e.to_string()))?;
        if model.source_routing.is_none()
            || super::relation::head(model).is_none()
            || self.dictionary.len() > 256
            || self.dictionary.iter().zip(primes).any(|(w, p)| {
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
                .any(|p| p[0].bytes[..usize::from(p[0].len)] >= p[1].bytes[..usize::from(p[1].len)])
            || self.router.codes.iter().any(|c| c.feature.kind > 6)
        {
            return Err(Error("invalid dependent read artifact".into()));
        }
        Ok(())
    }
}

impl Model {
    pub fn fit_dependent_read(
        &self,
        docs: &[DependentReadExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        config.validate()?;
        self.validate()?;
        if self.dependent_read.is_some()
            || self.source_routing.is_none()
            || docs.is_empty()
            || docs.len() > 1024
            || docs.iter().any(|d| {
                d.example.prompt.len() + d.example.response.len() > 65536
                    || d.via.as_ref().is_some_and(|v| v.is_empty() || v.len() > 32)
            })
        {
            return Err(Error("dependent read parent/source bound invalid".into()));
        }
        let start = Instant::now();
        let examples: Vec<_> = docs.iter().map(|d| d.example.clone()).collect();
        let (dictionary, omitted, _) = super::word_copy_training::dictionary(&examples)?;
        let read = super::role_read::head(self).ok_or_else(|| Error("reader absent".into()))?;
        let defer = read
            .actions
            .iter()
            .position(|a| !a.copy)
            .ok_or_else(|| Error("NoRead absent".into()))?;
        let mut rng = config.seed;
        let mut block = DependentRead {
            dictionary,
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
        let mut unreachable = Vec::new();
        let mut skipped = 0;
        for d in docs {
            if d.example.id.trim().is_empty() || !ids.insert(&d.example.id) {
                return Err(Error("duplicate/empty dependent source identity".into()));
            }
            block
                .router
                .training
                .push(super::training::receipt(&Document {
                    id: d.example.id.clone(),
                    text: serde_json::to_string(d).map_err(|e| Error(e.to_string()))?,
                }));
            let mut session = self.session(Control::Full)?;
            session.observe(self, BOS)?;
            for token in self.encode(&d.example.prompt)? {
                session.observe(self, token)?;
            }
            session.begin_response(self)?;
            session.predict(self)?;
            let values = session
                .values
                .as_ref()
                .ok_or_else(|| Error("values absent".into()))?;
            let entry = session
                .response_entry
                .as_ref()
                .ok_or_else(|| Error("entry absent".into()))?;
            if !super::word_copy_runtime::eligible(self, entry, values, Control::Full) {
                if d.via.is_some() {
                    unreachable.push(d.example.id.clone());
                } else {
                    skipped += 1;
                }
                continue;
            }
            let state = values
                .relations
                .as_ref()
                .ok_or_else(|| Error("relations absent".into()))?;
            let addr = dependent_read::addresses(&block, values, &mut Default::default());
            let mut alternatives = vec![Alternative {
                features: Vec::new(),
                codes: Vec::new(),
                action: defer,
                correct: d.via.is_none(),
            }];
            for &id in &state.directory {
                let Some(record) = state.record(id) else {
                    continue;
                };
                let (f, n) =
                    dependent_read::features(self, values, record, &addr, &mut Default::default());
                for (action, a) in read.actions.iter().enumerate() {
                    if !a.copy {
                        continue;
                    }
                    let correct = d.via.as_ref().is_some_and(|via| {
                        via.as_bytes() == &record.owner.bytes[..usize::from(record.owner.len)]
                    }) && if let Some(last) =
                        dependent_read::follow(state, record, &mut Default::default())
                    {
                        let prefix = a
                            .prefix
                            .map(|t| self.decode(&[t]))
                            .transpose()?
                            .unwrap_or_default();
                        let mut target = prefix;
                        target.extend_from_slice(&last.value.bytes[..usize::from(last.value.len)]);
                        d.example.response.as_bytes().starts_with(&target)
                            && d.example
                                .response
                                .as_bytes()
                                .get(target.len())
                                .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_')
                    } else {
                        let token = read.actions[defer]
                            .prefix
                            .ok_or_else(|| Error("NoRead prefix absent".into()))?;
                        d.example
                            .response
                            .as_bytes()
                            .starts_with(&self.decode(&[token])?)
                    };
                    alternatives.push(Alternative {
                        features: f[..n].to_vec(),
                        codes: Vec::new(),
                        action,
                        correct,
                    });
                }
            }
            if !alternatives.iter().any(|a| a.correct) {
                unreachable.push(d.example.id.clone());
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
            return Err(Error("no dependent training frames".into()));
        }
        let mut vocabulary: Vec<_> = frequency.into_iter().collect();
        vocabulary.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let universe = vocabulary.len();
        vocabulary.truncate(block.router.config.learned_features);
        vocabulary.sort_by_key(|v| v.0);
        block.router.codes = vocabulary
            .into_iter()
            .map(|v| SourceCode {
                feature: v.0,
                roots: [self.geometry.identity; 2],
            })
            .collect();
        let fit = learn(self, &mut block.router, &mut frames, start);
        let bytes = serde_json::to_vec(&block)
            .map_err(|e| Error(e.to_string()))?
            .len();
        let mut model = self.clone();
        model.dependent_read = Some(block);
        model.refresh_identity()?;
        model.validate()?;
        let artifact = model.artifact_cid().to_owned();
        Ok((
            model,
            serde_json::json!({"schema":"uor-r4.dependent-read-fit/1","parent":self.artifact_cid(),"artifact":artifact,"documents":docs.len(),"frames":frames.len(),"upstream_skipped":skipped,"unreachable":unreachable,"dictionary_omitted":omitted,"feature_universe":universe,"block_bytes":bytes,"fit":fit,"elapsed_ms":start.elapsed().as_millis()}),
        ))
    }
}
