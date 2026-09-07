//! Offline warm continuation of the existing exact-owner/value writer.
//! Serving preserves candidate/address/role support with optional learned gap features.
use super::relation::RELATION_ROWS;
use super::relation_training::{writer_frame, Frame, WriterRevision};
use super::value_lexemes::LexemeState;
use super::value_types::{ValueEntry, ValueFeature, ValueRow, ValueWork};
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

const MAX_SECONDS: u64 = 10;
const MAX_FRAMES: usize = 8192;

/// Nonexecuting witness for a single refinement beneath the complete parent.
/// The previous writer includes its original exact NoWrite cache and receipts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WriterRefinement {
    pub schema: String,
    pub parent_artifact: String,
    pub previous: WriterRevision,
    pub max_seconds: u64,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub boundary_context: bool,
}

fn fixed_fields(current: &WriterRevision, previous: &WriterRevision) -> bool {
    current.schema == previous.schema
        && current.parent == previous.parent
        && current.dictionary == previous.dictionary
        && current.role_context == previous.role_context
        && !current.reuse_admission
        && previous.rows.iter().all(|old| {
            current
                .rows
                .binary_search_by_key(&old.feature, |row| row.feature)
                .is_ok()
        })
}

impl WriterRefinement {
    fn parent(&self, model: &Model) -> Result<Model> {
        let mut parent = model.clone();
        parent.relation_writer_refinement = None;
        parent.relation_writer = Some(self.previous.clone());
        parent.refresh_identity()?;
        if parent.artifact_cid != self.parent_artifact {
            return Err(Error("writer refinement frozen parent differs".into()));
        }
        Ok(parent)
    }

    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        let current = model
            .relation_writer
            .as_ref()
            .ok_or_else(|| Error("writer refinement active writer absent".into()))?;
        if self.schema != "uor-r4.relation-writer-refinement/1"
            || self.max_seconds != MAX_SECONDS
            || !fixed_fields(current, &self.previous)
            || (self.boundary_context
                && current.rows.iter().any(|row| {
                    matches!(row.feature.kind & 31, 13 | 14)
                        && (row.feature.a > 512 || row.feature.b > 1)
                }))
        {
            return Err(Error(
                "writer refinement changed fixed fields or support".into(),
            ));
        }
        self.parent(model)?.validate()?;
        // This checks a new cache against the refined uncached model. The
        // original cache exists only in the nonexecuting parent witness.
        current.validate(model)?;
        let mut duplicate = model.clone();
        duplicate.refresh_identity()?;
        if duplicate.artifact_cid != model.artifact_cid
            || duplicate.uor_model_address != model.uor_model_address
        {
            return Err(Error("writer refinement identity differs".into()));
        }
        Ok(())
    }
}

/// Construction labels apply only to completed words in the native tokenizer
/// stream. The fixed role path ignores payload/global poses, as serving does.
fn frames(model: &Model, documents: &[RelationExample]) -> Result<Vec<Frame>> {
    let writer = model
        .relation_writer
        .as_ref()
        .ok_or_else(|| Error("writer refinement parent absent".into()))?;
    let mut frames = BTreeSet::new();
    for doc in documents {
        let mut words = LexemeState::default();
        let mut last = None;
        let mut consumed = BTreeSet::new();
        let mut capture = |words: &LexemeState| -> Result<()> {
            if words.recent_len == 0 || last == Some(words.recent[0].byte_end) {
                return Ok(());
            }
            last = Some(words.recent[0].byte_end);
            let mut labels = doc.writes.iter().enumerate().filter(|(_, label)| {
                label.owner_end_byte.max(label.value_end_byte) == words.recent[0].byte_end
            });
            let label = labels.next();
            if labels.next().is_some() {
                return Err(Error("multiple labels at one relation boundary".into()));
            }
            frames.insert(writer_frame(
                model,
                words,
                label.map(|(_, label)| label),
                Some(&writer.role_context),
            )?);
            if frames.len() > MAX_FRAMES {
                return Err(Error("writer refinement frame bound exceeded".into()));
            }
            if let Some((index, _)) = label {
                consumed.insert(index);
            }
            Ok(())
        };
        for (sequence, token) in model.encode(&doc.prompt)?.into_iter().enumerate() {
            let byte;
            let bytes = if token < LEXICAL_BASE {
                byte = [(token - 2) as u8];
                &byte[..]
            } else {
                &model.lexical_pieces[(token - LEXICAL_BASE) as usize][..]
            };
            for &byte in bytes {
                words.feed(
                    byte,
                    ValueEntry {
                        sequence: sequence as u64,
                        token,
                        pose: model.geometry.identity,
                        ..Default::default()
                    },
                    &mut ValueWork::default(),
                );
                capture(&words)?;
            }
        }
        // Session::begin_response finishes the final word in ValueState::begin,
        // then calls observe_relation before finishing pending spans. Capture
        // that actual caller boundary, including an unterminated final word.
        words.finish(&mut ValueWork::default());
        capture(&words)?;
        if consumed.len() != doc.writes.len() {
            return Err(Error(format!(
                "writer refinement labels not observed in {}",
                doc.id
            )));
        }
    }
    Ok(frames.into_iter().collect())
}

fn scores(frame: &Frame, weights: &BTreeMap<ValueFeature, i32>) -> Vec<i64> {
    frame
        .alternatives
        .iter()
        .map(|alternative| {
            alternative
                .keys
                .iter()
                .map(|key| i64::from(weights[key]))
                .sum()
        })
        .collect()
}

/// First maximum reproduces NoWrite-at-zero and the serving candidate order.
fn selection(frame: &Frame, scores: &[i64]) -> Result<(usize, usize)> {
    let mut winner = 0;
    let mut wanted = None;
    for (index, alternative) in frame.alternatives.iter().enumerate() {
        if scores[index] > scores[winner] {
            winner = index;
        }
        if alternative.correct && wanted.is_none_or(|old: usize| scores[index] > scores[old]) {
            wanted = Some(index);
        }
    }
    Ok((
        winner,
        wanted.ok_or_else(|| Error("writer refinement target unreachable".into()))?,
    ))
}

fn correct(
    frames: &[Frame],
    weights: &BTreeMap<ValueFeature, i32>,
    started: Instant,
) -> Result<Option<usize>> {
    let mut correct = 0;
    for frame in frames {
        if started.elapsed() >= Duration::from_secs(MAX_SECONDS) {
            return Ok(None);
        }
        let scores = scores(frame, weights);
        let (winner, _) = selection(frame, &scores)?;
        correct += usize::from(frame.alternatives[winner].correct);
    }
    Ok(Some(correct))
}

fn learn(
    frames: &[Frame],
    previous: &[ValueRow],
    epochs: usize,
) -> Result<(Vec<ValueRow>, serde_json::Value)> {
    let initialization = Instant::now();
    let mut weights: BTreeMap<_, _> = previous
        .iter()
        .map(|row| (row.feature, row.weight))
        .collect();
    for frame in frames {
        for alternative in &frame.alternatives {
            for key in &alternative.keys {
                weights.entry(*key).or_default();
            }
        }
    }
    if weights.len() > RELATION_ROWS {
        return Err(Error("writer refinement row bound exceeded".into()));
    }
    let new_rows = weights.len() - previous.len();
    let initialization_ms = initialization.elapsed().as_millis();
    let started = Instant::now();
    let initial_correct = correct(frames, &weights, started)?;
    let mut best = weights.clone();
    let mut best_correct = initial_correct;
    let mut selected_epoch = 0;
    let mut completed_epochs = 0;
    let mut updates = 0;
    let mut stopped = initial_correct.is_none();
    if !stopped && initial_correct != Some(frames.len()) {
        'epochs: for epoch in 0..epochs {
            for frame in frames {
                if started.elapsed() >= Duration::from_secs(MAX_SECONDS) {
                    stopped = true;
                    break 'epochs;
                }
                let scores = scores(frame, &weights);
                let (winner, wanted) = selection(frame, &scores)?;
                // Preserve the existing writer's margin-eight update law.
                if !frame.alternatives[winner].correct || scores[wanted] < scores[winner] + 8 {
                    for key in &frame.alternatives[winner].keys {
                        if let Some(weight) = weights.get_mut(key) {
                            *weight = (*weight - 1).max(-1_000_000);
                        }
                    }
                    for key in &frame.alternatives[wanted].keys {
                        if let Some(weight) = weights.get_mut(key) {
                            *weight = (*weight + 1).min(1_000_000);
                        }
                    }
                    updates += 1;
                }
            }
            let Some(count) = correct(frames, &weights, started)? else {
                stopped = true;
                break;
            };
            completed_epochs = epoch + 1;
            // A partial or worse epoch cannot replace the original warm state.
            if best_correct.is_none_or(|best| count > best) {
                best = weights.clone();
                best_correct = Some(count);
                selected_epoch = completed_epochs;
            }
            if count == frames.len() {
                break;
            }
        }
    }
    Ok((
        best.into_iter()
            .map(|(feature, weight)| ValueRow { feature, weight })
            .collect(),
        serde_json::json!({"initial_correct":initial_correct,"final_correct":best_correct,
            "writer_frames":frames.len(),"old_rows":previous.len(),"new_rows":new_rows,
            "epochs":epochs,"completed_epochs":completed_epochs,"selected_epoch":selected_epoch,
            "updates":updates,"max_seconds":MAX_SECONDS,"stopped_at_time_limit":stopped,
            "initialization_ms":initialization_ms,"learning_ms":started.elapsed().as_millis()}),
    ))
}

impl Model {
    /// Continue one existing writer, preserving its dictionary, role geometry,
    /// feature law and every old row. Bounds: 256 documents, 1 MiB input, 8192
    /// unique frames, 16384 rows, 1..=64 epochs and ten seconds of learning.
    /// The old cache is retained only in the parent witness; compile a new cache
    /// with `compile_relation_admission` after fitting this uncached model.
    pub fn refine_relation_writer(
        &self,
        documents: &[RelationExample],
        epochs: usize,
    ) -> Result<(Model, serde_json::Value)> {
        self.refine_writer_inner(documents, epochs, false)
    }

    /// Add learned first/last source-gap context to the same writer. Exact
    /// separator identity and adjacency affect scores, not candidate admission.
    pub fn refine_relation_writer_with_boundaries(
        &self,
        documents: &[RelationExample],
        epochs: usize,
    ) -> Result<(Model, serde_json::Value)> {
        self.refine_writer_inner(documents, epochs, true)
    }

    fn refine_writer_inner(
        &self,
        documents: &[RelationExample],
        epochs: usize,
        boundary_context: bool,
    ) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        let previous = self
            .relation_writer
            .as_ref()
            .ok_or_else(|| Error("writer refinement parent absent".into()))?;
        if self.relation_writer_refinement.is_some()
            || documents.is_empty()
            || documents.len() > 256
            || !(1..=64).contains(&epochs)
            || documents
                .iter()
                .map(|doc| doc.prompt.len() + doc.response.len())
                .sum::<usize>()
                > 1024 * 1024
        {
            return Err(Error(
                "invalid writer refinement population/configuration".into(),
            ));
        }
        let started = Instant::now();
        let mut ids = BTreeSet::new();
        let mut training = Vec::new();
        for doc in documents {
            if doc.id.trim().is_empty()
                || !ids.insert(&doc.id)
                || doc.prompt.is_empty()
                || doc.writes.len() > 16
                || doc
                    .writes
                    .iter()
                    .any(|label| !(1..=3).contains(&label.action))
            {
                return Err(Error("invalid writer refinement supervision".into()));
            }
            training.push(super::training::receipt(&Document {
                id: doc.id.clone(),
                text: serde_json::to_string(doc).map_err(|e| Error(e.to_string()))?,
            }));
        }
        // This feature-only template binds the optional law before preparing
        // frames. It is finalized and validated after learning, before return.
        let mut model = self.clone();
        model.relation_writer_refinement = Some(WriterRefinement {
            schema: "uor-r4.relation-writer-refinement/1".into(),
            parent_artifact: self.artifact_cid.clone(),
            previous: previous.clone(),
            max_seconds: MAX_SECONDS,
            boundary_context,
        });
        let frames = frames(&model, documents)?;
        if frames.is_empty() {
            return Err(Error(
                "writer refinement has no completed-word frames".into(),
            ));
        }
        let preparation_ms = started.elapsed().as_millis();
        let (rows, mut report) = learn(&frames, &previous.rows, epochs)?;
        let current = model
            .relation_writer
            .as_mut()
            .ok_or_else(|| Error("writer refinement active writer absent".into()))?;
        current.rows = rows;
        current.training = training;
        current.epochs = epochs;
        current.admission = None;
        current.reuse_admission = false;
        model.refresh_identity()?;
        model.validate()?;
        report["schema"] = serde_json::json!("uor-r4.writer-refinement-fit/1");
        report["parent"] = serde_json::json!(self.artifact_cid());
        report["artifact"] = serde_json::json!(model.artifact_cid());
        report["documents"] = serde_json::json!(documents.len());
        report["writer_rows"] =
            serde_json::json!(model.relation_writer.as_ref().map(|w| w.rows.len()));
        report["dictionary"] = serde_json::json!(previous.dictionary.len());
        report["boundary_context"] = serde_json::json!(boundary_context);
        report["preparation_ms"] = serde_json::json!(preparation_ms);
        report["elapsed_ms"] = serde_json::json!(started.elapsed().as_millis());
        report["scope"] = serde_json::json!("Warm-start existing writer coefficients; zero-initialize only newly observed keys. Frozen dictionary, role geometry, owner/value candidate support and feature law. Same margin-eight integer update and first-maximum NoWrite tie. Retain the highest complete construction accuracy including epoch zero. Original writer/cache retained in the exact parent witness; active cache cleared and inherited cache disabled until independent recompilation. Source byte/action labels are offline only; no added grammar or phrase admission. All other model parameters remain unchanged.");
        if boundary_context {
            report["scope"] = serde_json::json!("Warm-start existing writer coefficients with frozen dictionary, role geometry and owner/value support. Add only learned features13/14: incoming gap at later endpoint and outgoing gap after earlier endpoint, with role direction. Descriptor retains last separator byte and exact single-byte versus wider adjacency; it is not a parser or hard admission rule. Original coefficient-only artifact law remains unchanged when the witness flag is absent/false. New NoWrite cache must certify the complete prime and boundary descriptor signature. Same margin-eight updates; exact parent/cache preserved in nonexecuting witness.");
        }
        Ok((model, report))
    }

    /// Restore the exact complete parent, including its original writer cache.
    pub fn without_relation_writer_refinement(&self) -> Result<Model> {
        self.validate()?;
        self.relation_writer_refinement
            .as_ref()
            .ok_or_else(|| Error("writer refinement absent".into()))?
            .parent(self)
    }
}

#[cfg(test)]
mod tests {
    use super::super::relation_training::Alternative;
    use super::*;

    fn feature(a: u64) -> ValueFeature {
        ValueFeature { kind: 33, a, b: 0 }
    }

    fn frame(key: ValueFeature, write: bool) -> Frame {
        Frame {
            alternatives: vec![
                Alternative {
                    keys: Vec::new(),
                    correct: !write,
                },
                Alternative {
                    keys: vec![key],
                    correct: write,
                },
            ],
        }
    }

    #[test]
    fn writer_refinement_keeps_warm_rows_and_zero_initializes_new_keys() {
        let previous = vec![ValueRow {
            feature: feature(1),
            weight: 9,
        }];
        let frames = vec![frame(feature(2), false)];
        let (rows, report) = learn(&frames, &previous, 8).unwrap();
        assert_eq!(
            rows,
            [
                previous[0].clone(),
                ValueRow {
                    feature: feature(2),
                    weight: 0
                }
            ]
        );
        assert_eq!(report["initial_correct"], 1);
        assert_eq!(report["selected_epoch"], 0);
        assert_eq!(report["updates"], 0);
    }

    #[test]
    fn writer_refinement_updates_from_old_weight_without_dropping_unobserved_rows() {
        let previous = vec![
            ValueRow {
                feature: feature(1),
                weight: 9,
            },
            ValueRow {
                feature: feature(2),
                weight: -3,
            },
        ];
        let (rows, report) = learn(&[frame(feature(2), true)], &previous, 8).unwrap();
        assert_eq!(rows[0], previous[0]);
        assert_eq!(rows[1].weight, 1);
        assert_eq!(report["initial_correct"], 0);
        assert_eq!(report["final_correct"], 1);
        assert_eq!(report["selected_epoch"], 4);
        assert_eq!(report["new_rows"], 0);
    }

    #[test]
    fn writer_refinement_cannot_replace_better_epoch_zero_with_a_worse_epoch() {
        let previous = vec![ValueRow {
            feature: feature(1),
            weight: 1,
        }];
        let frames = [
            frame(feature(1), true),
            frame(feature(1), true),
            frame(feature(1), false),
        ];
        let (rows, report) = learn(&frames, &previous, 1).unwrap();
        assert_eq!(rows, previous);
        assert_eq!(report["initial_correct"], 2);
        assert_eq!(report["final_correct"], 2);
        assert_eq!(report["selected_epoch"], 0);
        let tie = frame(feature(1), true);
        assert_eq!(selection(&tie, &[0, 0]).unwrap(), (0, 1));
    }

    fn writer() -> WriterRevision {
        WriterRevision {
            schema: "uor-r4.relation-writer/1".into(),
            parent: "parent".into(),
            dictionary: Vec::new(),
            role_context: Vec::new(),
            rows: vec![ValueRow {
                feature: feature(1),
                weight: 3,
            }],
            training: Vec::new(),
            epochs: 1,
            reuse_admission: false,
            admission: None,
        }
    }

    #[test]
    fn writer_refinement_freezes_metadata_and_support_and_disables_inherited_cache() {
        let previous = writer();
        let mut current = previous.clone();
        current.rows[0].weight += 1;
        current.epochs = 8;
        assert!(fixed_fields(&current, &previous));
        current.reuse_admission = true;
        assert!(!fixed_fields(&current, &previous));
        current.reuse_admission = false;
        current.rows.clear();
        assert!(!fixed_fields(&current, &previous));
        let mut changed = previous.clone();
        changed.parent = "different".into();
        assert!(!fixed_fields(&changed, &previous));
        let mut changed = previous.clone();
        changed.dictionary.push(word_copy_types::WordCopyAddress {
            bytes: [0; 32],
            len: 1,
            prime: 2,
        });
        assert!(!fixed_fields(&changed, &previous));
        let witness = WriterRefinement {
            schema: "uor-r4.relation-writer-refinement/1".into(),
            parent_artifact: "complete-parent".into(),
            previous,
            max_seconds: MAX_SECONDS,
            boundary_context: false,
        };
        let wire = serde_json::to_vec(&witness).unwrap();
        assert!(serde_json::from_slice::<serde_json::Value>(&wire)
            .unwrap()
            .get("boundary_context")
            .is_none());
        assert_eq!(
            serde_json::from_slice::<WriterRefinement>(&wire).unwrap(),
            witness
        );
    }
}
