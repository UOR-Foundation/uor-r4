//! Offline shared admission fit; no response labels enter serving.
use super::joint_admission::{self, JointAdmission};
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{Alternative, Frame};
use super::value_types::ValueWork;
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

impl Model {
    pub fn fit_joint_admission(
        &self,
        docs: &[ValueExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        config.validate()?;
        if config.role_context_only {
            return Err(Error(
                "role_context_only is not an admission feature mode".into(),
            ));
        }
        if self.joint_admission.is_some()
            || self.typed_literals.is_none()
            || self.source_routing.is_none()
            || docs.is_empty()
            || docs.len() > 1024
        {
            return Err(Error(
                "joint admission requires a literal/source parent and 1..1024 examples".into(),
            ));
        }
        let start = Instant::now();
        let mut frames = Vec::new();
        let mut frequency = BTreeMap::<_, [usize; 2]>::new();
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut skipped = 0;
        let mut numeric = 0;
        let mut mismatched_numeric = Vec::new();
        for doc in docs {
            if doc.id.trim().is_empty() || !ids.insert(doc.id.clone()) {
                return Err(Error("duplicate/empty admission example id".into()));
            }
            receipts.push(super::training::receipt(&Document {
                id: doc.id.clone(),
                text: serde_json::to_string(&(&doc.prompt, &doc.response))
                    .map_err(|e| Error(e.to_string()))?,
            }));
            let mut session = self.session(Control::Full)?;
            session.observe(self, BOS)?;
            for token in self.encode(&doc.prompt)? {
                session.observe(self, token)?;
            }
            session.begin_response(self)?;
            session.predict(self)?;
            let values = session
                .values
                .as_ref()
                .ok_or_else(|| Error("admission values absent".into()))?;
            let context = super::typed_routing::context(
                self,
                values,
                Control::Full,
                &mut ValueWork::default(),
            );
            let Some(context) = context.filter(|c| c.literal_component) else {
                skipped += 1;
                continue;
            };
            let Some(decision) = values.pending else {
                skipped += 1;
                continue;
            };
            // The supplied response identifies the desired action domain. Wrong
            // parent numerals remain Numeric domain positives and are reported
            // separately; this fit cannot repair inherited operand selection.
            let response = doc.response.trim_start();
            let wants_numeric = response
                .as_bytes()
                .first()
                .is_some_and(|b| b.is_ascii_digit() || *b == b'-');
            if wants_numeric {
                let numeral = decision.value.to_string();
                if !response.starts_with(&numeral)
                    || response
                        .as_bytes()
                        .get(numeral.len())
                        .is_some_and(u8::is_ascii_digit)
                {
                    mismatched_numeric.push(doc.id.clone());
                }
            }
            let action = usize::from(!wants_numeric);
            numeric += usize::from(wants_numeric);
            // offer's score is baseline-relative. Recover the exact frozen
            // Copy/Add-minus-NoOperation margin from identical metadata.
            let mut work = ValueWork::default();
            let op = usize::from(decision.action == ValueAction::Add);
            let margin = super::typed_routing::score(
                self,
                values,
                Some((decision.operands[0], decision.operands[1])),
                op,
                &context,
                Control::Full,
                &mut work,
            ) - super::typed_routing::score(
                self,
                values,
                None,
                2,
                &context,
                Control::Full,
                &mut work,
            );
            let (f, n) = joint_admission::features(
                values,
                decision.action,
                (decision.operands[0], decision.operands[1]),
                margin,
                &context,
                &mut work,
            );
            let features = f[..n].to_vec();
            for f in &features {
                frequency.entry(*f).or_default()[action] += 1;
            }
            frames.push(Frame {
                alternatives: (0..2)
                    .map(|a| Alternative {
                        features: features.clone(),
                        codes: Vec::new(),
                        action: a,
                        correct: a == action,
                    })
                    .collect(),
            });
        }
        if frames.is_empty() || numeric == 0 || numeric == frames.len() {
            return Err(Error(
                "admission needs both numeric and lexical construction frames".into(),
            ));
        }
        let mut vocabulary: Vec<_> = frequency.into_iter().collect();
        vocabulary.sort_by(|a, b| {
            let separation = |v: &[usize; 2]| v[0].abs_diff(v[1]);
            separation(&b.1)
                .cmp(&separation(&a.1))
                .then_with(|| (b.1[0] + b.1[1]).cmp(&(a.1[0] + a.1[1])))
                .then_with(|| a.0.cmp(&b.0))
        });
        let universe = vocabulary.len();
        vocabulary.truncate(config.learned_features);
        vocabulary.sort_by_key(|entry| entry.0);
        // Distinct seeded action landmarks break the identity-code symmetry;
        // identical landmarks would make every feature move score both actions
        // equally and trap the discrete learner in a constant decision.
        let mut rng = config.seed;
        let landmarks = (0..2)
            .map(|_| {
                std::array::from_fn(|_| {
                    (super::learned_routing_training::next_random(&mut rng) % 120) as u16
                })
            })
            .collect();
        let mut block = SourceRouting {
            schema: "uor-r4.geometric-source-routing/1".into(),
            parent_artifact: self.artifact_cid().into(),
            codes: vocabulary
                .into_iter()
                .map(|(feature, _)| SourceCode {
                    feature,
                    roots: [self.geometry.identity; 2],
                })
                .collect(),
            landmarks,
            biases: vec![0; 2],
            ranks: super::learned_routing_training::ranks(self),
            training: receipts,
            config,
        };
        let mut report =
            super::source_routing_training::learn(self, &mut block, &mut frames, start);
        let mut model = self.clone();
        model.joint_admission = Some(JointAdmission { router: block });
        model.refresh_identity()?;
        model.validate()?;
        report["schema"] = serde_json::json!("uor-r4.joint-admission-fit/1");
        report["parent"] = serde_json::json!(self.artifact_cid());
        report["artifact"] = serde_json::json!(model.artifact_cid());
        report["documents"] = serde_json::json!(docs.len());
        report["frames"] = serde_json::json!(frames.len());
        report["numeric_frames"] = serde_json::json!(numeric);
        report["lexical_frames"] = serde_json::json!(frames.len() - numeric);
        report["skipped_no_literal_proposal"] = serde_json::json!(skipped);
        report["numeric_parent_mismatches"] = serde_json::json!(mismatched_numeric);
        report["feature_universe"] = serde_json::json!(universe);
        report["elapsed_ms"] = serde_json::json!(start.elapsed().as_millis());
        report["scope"] = serde_json::json!("Learned binary admission before literal numeric execution; inherited operator/operand ranking, derived roles, source/NoRead and emission are unchanged. Labels are raw supplied responses. Does not unify all operand/source alternatives or establish general language.");
        Ok((model, report))
    }
}
