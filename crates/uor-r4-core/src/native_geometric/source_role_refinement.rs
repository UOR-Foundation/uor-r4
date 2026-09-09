//! Bounded offline correction of existing recent-source recency roots.
//! Candidate admission, feature extraction and exact occurrence copying are unchanged.
use super::role_read::{self, NO_SOURCE};
use super::source_routing::SourceRouting;
use super::source_routing_training::{learn_code_subset, Alternative, Frame};
use super::*;
use std::{collections::BTreeSet, time::Instant};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceRoleTarget {
    Preserve,
    NoRead,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRoleRefinementExample {
    pub id: String,
    pub prompt: String,
    pub target: SourceRoleTarget,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceRoleRefinement {
    pub parent_artifact: String,
    pub previous: SourceRouting,
    pub config: SourceRoutingConfig,
    pub training: Vec<DocumentReceipt>,
}
fn validate_config(config: &SourceRoutingConfig) -> Result<()> {
    if !(1..=4096).contains(&config.learned_features)
        || !(1..=8).contains(&config.passes)
        || !(1..=120).contains(&config.proposals)
        || !(1..=180).contains(&config.max_seconds)
    {
        return Err(Error("invalid source role refinement configuration".into()));
    }
    Ok(())
}
fn frozen_equal(current: &SourceRouting, previous: &SourceRouting, identity: u16) -> bool {
    let mut restored = current.clone();
    if restored.codes.len() != previous.codes.len() {
        return false;
    }
    for (code, old) in restored.codes.iter_mut().zip(&previous.codes) {
        if code.feature != old.feature {
            return false;
        }
        if old.feature.kind == 6 && old.roots.iter().any(|root| *root != identity) {
            code.roots = old.roots;
        }
    }
    restored == *previous
}
impl SourceRoleRefinement {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        validate_config(&self.config)?;
        if self.training.is_empty()
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
            return Err(Error("invalid source role refinement receipts".into()));
        }
        // Restore the entire retained model before checking the new parameters.
        let mut parent = model.clone();
        parent.source_role_refinement = None;
        parent.source_routing = Some(self.previous.clone());
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("source role refinement frozen parent differs".into()));
        }
        parent.validate()?;
        let current = model
            .source_routing
            .as_ref()
            .ok_or_else(|| Error("source role refinement router absent".into()))?;
        current.validate(model)?;
        if !frozen_equal(current, &self.previous, model.geometry.identity)
            || self.config.mode != current.config.mode
            || self.config.role_context_only != current.config.role_context_only
            || self.config.learned_features < current.codes.len()
        {
            return Err(Error(
                "source role refinement frozen parameters differ".into(),
            ));
        }
        let mut identity = model.clone();
        identity.refresh_identity()?;
        if identity.artifact_cid != model.artifact_cid
            || identity.uor_model_address != model.uor_model_address
        {
            return Err(Error("source role refinement identity differs".into()));
        }
        Ok(())
    }
}
impl Model {
    /// Labels the existing source selector; Preserve means the exact parent
    /// occurrence/action, and NoRead means its existing abstention action.
    pub fn fit_source_role_refinement(
        &self,
        docs: &[SourceRoleRefinementExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        let start = Instant::now();
        self.validate()?;
        validate_config(&config)?;
        if self.source_role_refinement.is_some() || docs.is_empty() || docs.len() > 4096 {
            return Err(Error(
                "invalid source role refinement parent/population".into(),
            ));
        }
        let previous = self
            .source_routing
            .as_ref()
            .ok_or_else(|| Error("source role refinement router absent".into()))?;
        if previous.codes.len() > config.learned_features
            || previous.config.mode != config.mode
            || previous.config.role_context_only != config.role_context_only
        {
            return Err(Error(
                "source role refinement vocabulary/context differs".into(),
            ));
        }
        let read = role_read::head(self)
            .ok_or_else(|| Error("source role refinement reader absent".into()))?;
        let no_read = read
            .actions
            .iter()
            .position(|a| !a.copy)
            .ok_or_else(|| Error("source role refinement NoRead absent".into()))?;
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
        for doc in docs {
            if doc.id.trim().is_empty()
                || !ids.insert(&doc.id)
                || doc.prompt.is_empty()
                || doc.prompt.len() > 65536
            {
                return Err(Error("invalid source role refinement document".into()));
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
                .ok_or_else(|| Error("source role refinement values absent".into()))?;
            let entry = s
                .response_entry
                .as_ref()
                .ok_or_else(|| Error("source role refinement entry absent".into()))?;
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
                if doc.target == SourceRoleTarget::NoRead {
                    return Err(Error(format!(
                        "source role NoRead target bypassed: {} ({reason})",
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
            .ok_or_else(|| Error("source role parent choice absent".into()))?;
            let (source, action) = match doc.target {
                SourceRoleTarget::Preserve => (actual.0, actual.1),
                SourceRoleTarget::NoRead => (NO_SOURCE, no_read),
            };
            let words = values
                .lexemes
                .as_ref()
                .ok_or_else(|| Error("source role words absent".into()))?;
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
                for ai in 0..read.actions.len() {
                    if super::source_routing::allowed(self, values, candidate, ai) {
                        alternatives.push(Alternative {
                            features: features[..n].to_vec(),
                            codes: Vec::new(),
                            action: ai,
                            correct: candidate == source && ai == action,
                        });
                    }
                }
            }
            if alternatives.iter().filter(|a| a.correct).count() != 1 {
                return Err(Error("source role exact target unavailable".into()));
            }
            let atom = if source == NO_SOURCE {
                None
            } else {
                super::relation::source(values, source)
            };
            labels.push(serde_json::json!({"id":doc.id,"target":doc.target,"parent_source":actual.0,"parent_action":actual.1,"source":source,"action":action,"source_end":atom.map(|w|w.end),"source_byte_end":atom.map(|w|w.byte_end)}));
            frames.push(Frame { alternatives });
        }
        if frames.is_empty() {
            return Err(Error("source role refinement has no direct frames".into()));
        }
        let mut block = previous.clone();
        block.config = config.clone();
        let mask: Vec<_> = block
            .codes
            .iter()
            .map(|c| {
                c.feature.kind == 6 && c.roots.iter().any(|root| *root != self.geometry.identity)
            })
            .collect();
        if !mask.iter().any(|m| *m) {
            return Err(Error(
                "source role refinement recency vocabulary absent".into(),
            ));
        }
        let fit = learn_code_subset(self, &mut block, &mut frames, start, &mask)?;
        block.config = previous.config.clone();
        let changed: Vec<_> = block.codes.iter().zip(&previous.codes).filter(|(a,b)| a != b).map(|(a,b)| serde_json::json!({"kind":a.feature.kind,"a_hex":format!("{:016x}",a.feature.a),"b_hex":format!("{:016x}",a.feature.b),"before":b.roots,"after":a.roots})).collect();
        let mut model = self.clone();
        model.source_routing = Some(block);
        model.source_role_refinement = Some(SourceRoleRefinement {
            parent_artifact: self.artifact_cid.clone(),
            previous: previous.clone(),
            config,
            training: receipts,
        });
        model.refresh_identity()?;
        model.validate()?;
        let report = serde_json::json!({"schema":"uor-r4.source-role-refinement-fit/1","parent_artifact":self.artifact_cid(),"artifact":model.artifact_cid(),"documents":docs.len(),"frames":frames.len(),"labels":labels,"skips":skips,"mutable_codes":mask.iter().filter(|m|**m).count(),"changed_codes":changed,"fit":fit,"elapsed_ms":start.elapsed().as_millis(),"scope":"Offline exact parent occurrence/NoRead supervision; only existing recency roots learned; free generation is separate."});
        Ok((model, report))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_role_configuration_is_scoped_and_bounded() {
        let mut c = SourceRoutingConfig {
            learned_features: 4096,
            max_seconds: 180,
            ..SourceRoutingConfig::default()
        };
        assert!(validate_config(&c).is_ok());
        assert!(c.validate().is_err());
        c.max_seconds = 181;
        assert!(validate_config(&c).is_err());
        c.max_seconds = 180;
        c.proposals = 121;
        assert!(validate_config(&c).is_err());
    }
    #[test]
    fn source_role_frozen_fields_allow_only_existing_recency_roots() {
        use super::super::{source_routing::SourceCode, value_types::ValueFeature};
        let old = SourceRouting {
            schema: "test".into(),
            parent_artifact: "parent".into(),
            codes: vec![
                SourceCode {
                    feature: ValueFeature {
                        kind: 0,
                        a: 1,
                        b: 0,
                    },
                    roots: [119, 119],
                },
                SourceCode {
                    feature: ValueFeature {
                        kind: 6,
                        a: 10,
                        b: 0,
                    },
                    roots: [86, 119],
                },
            ],
            landmarks: vec![[119, 119]],
            biases: vec![0],
            ranks: vec![0],
            training: Vec::new(),
            config: SourceRoutingConfig::default(),
        };
        let mut changed = old.clone();
        changed.codes[1].roots = [119, 119];
        assert!(frozen_equal(&changed, &old, 119));
        let mut identity_recency = old.clone();
        identity_recency.codes[1].roots = [119, 119];
        let mut new_shortcut = identity_recency.clone();
        new_shortcut.codes[1].roots = [86, 119];
        assert!(!frozen_equal(&new_shortcut, &identity_recency, 119));
        changed.codes[0].roots = [0, 119];
        assert!(!frozen_equal(&changed, &old, 119));
        let mut changed = old.clone();
        changed.biases[0] = 1;
        assert!(!frozen_equal(&changed, &old, 119));
        let mut changed = old.clone();
        changed.codes[1].feature.a = 11;
        assert!(!frozen_equal(&changed, &old, 119));
        let mut changed = old.clone();
        changed.config.seed += 1;
        assert!(!frozen_equal(&changed, &old, 119));
        let mut changed = old.clone();
        changed.landmarks[0][0] = 0;
        assert!(!frozen_equal(&changed, &old, 119));
    }
    #[test]
    fn source_role_targets_have_no_implicit_training_stage() {
        assert!(
            serde_json::from_str::<SourceRoleRefinementExample>(r#"{"id":"x","prompt":"p"}"#)
                .is_err()
        );
        assert!(serde_json::from_str::<SourceRoleRefinementExample>(
            r#"{"id":"x","prompt":"p","target":"no_read","fitted":true}"#
        )
        .is_err());
    }
}
