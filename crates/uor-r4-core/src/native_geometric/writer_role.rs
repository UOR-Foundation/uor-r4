//! Nonpositive shared writer-role correction without candidate value identity.
//! The entire retained writer remains frozen; typed exterior context is exact.
use super::relation;
use super::value_lexemes::WordAtom;
use super::value_types::{ValueFeature, ValueRow, ValueWork};
use super::writer_choice::{bind_target, learn, winner, Alternative, Frame};
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};
const MAX_DOCUMENTS: usize = 1200;
const MAX_FRAMES: usize = 32768;
const MAX_ROWS: usize = 16384;
const WEIGHT_FLOOR: i32 = -1000000;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WriterRole {
    pub parent_artifact: String,
    pub rows: Vec<ValueRow>,
    pub training: Vec<DocumentReceipt>,
    pub epochs: usize,
    pub max_seconds: u64,
}
impl WriterRole {
    fn parent(&self, model: &Model) -> Result<Model> {
        let mut parent = model.clone();
        parent.writer_role = None;
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("writer role frozen parent differs".into()));
        }
        parent.validate()?;
        Ok(parent)
    }
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        self.parent(model)?;
        let lexical = model
            .writer_lexical
            .as_ref()
            .ok_or_else(|| Error("writer role requires inherited lexical dictionary".into()))?;
        if model.current_source.is_some()
            || model
                .writer_choice
                .as_ref()
                .is_none_or(|w| w.feature_layout != 2)
            || lexical.dictionary.len() > 128
            || lexical.dictionary.iter().any(|d| d.prime > 719)
            || !(1..=64).contains(&self.epochs)
            || !(1..=300).contains(&self.max_seconds)
        {
            return Err(Error("invalid writer role configuration".into()));
        }
        let primes: BTreeSet<_> = std::iter::once(0)
            .chain(lexical.dictionary.iter().map(|d| d.prime))
            .collect();
        if self.rows.len() > MAX_ROWS
            || self.rows.windows(2).any(|w| w[0].feature >= w[1].feature)
            || self.rows.iter().any(|r| !valid_row(r, &primes))
        {
            return Err(Error("invalid writer role residual rows".into()));
        }
        if self.training.is_empty()
            || self.training.len() > MAX_DOCUMENTS
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
            return Err(Error("invalid writer role training receipts".into()));
        }
        let mut identity = model.clone();
        identity.refresh_identity()?;
        if identity.artifact_cid != model.artifact_cid
            || identity.uor_model_address != model.uor_model_address
        {
            return Err(Error("writer role identity differs".into()));
        }
        Ok(())
    }
}
fn valid_row(row: &ValueRow, primes: &BTreeSet<u32>) -> bool {
    let f = row.feature;
    let old_b = f.b >> 14;
    (WEIGHT_FLOOR..=0).contains(&row.weight)
        && f.kind & 31 == 17
        && (1..=3).contains(&(f.kind >> 5))
        && f.a & 0xffffffff == 0
        && f.b & 15 != 0
        && f.b & 15 != 8
        && (f.b >> 4) & 1023 <= 512
        && [f.a >> 32, old_b >> 32, old_b & 0xffffffff]
            .iter()
            .all(|p| *p <= 719 && primes.contains(&(*p as u32)))
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
/// Keep owner, both exterior primes, exact gap and directed endpoint separation.
/// The namespace distinguishes omitted value identity from dictionary code zero.
pub(super) fn feature(layout_two: ValueFeature) -> ValueFeature {
    ValueFeature {
        kind: 17,
        a: layout_two.a & 0xffffffff00000000,
        b: layout_two.b,
    }
}
pub(super) fn enabled(control: Control) -> bool {
    !matches!(
        control,
        Control::WriterRoleDisabled | Control::WriterRoleContextDisabled
    )
}
pub(super) fn score(
    block: &WriterRole,
    feature: ValueFeature,
    action: u8,
    work: &mut ValueWork,
) -> i64 {
    work.relations.feature_writes = work.relations.feature_writes.saturating_add(1);
    relation::score(&block.rows, &[feature], action, work)
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END
fn frame(
    model: &Model,
    words: &[WordAtom],
    label: Option<&WriterChoiceOverride>,
    id: &str,
) -> Result<Frame> {
    let writer = model
        .relation_writer
        .as_ref()
        .ok_or_else(|| Error("writer role parent writer absent".into()))?;
    let lexical = model
        .writer_lexical
        .as_ref()
        .ok_or_else(|| Error("writer role lexical parent absent".into()))?;
    let mut work = ValueWork::default();
    let addr = relation::writer_addresses(model, words, &mut work);
    let lex = super::writer_lexical::addresses(&lexical.dictionary, words, &mut work);
    let gaps = relation::boundary_metadata(words, &mut work);
    let oldgaps = relation::boundary_context(model).then_some(&gaps);
    let parent = relation::write_choice(model, words, &mut work);
    let wanted = if let Some(label) = label {
        bind_target(words, label)?
    } else {
        parent
    };
    let mut target = wanted.is_none().then_some(0);
    let mut alternatives = vec![Alternative {
        baseline: 0,
        key: None,
    }];
    for o in 0..words.len() {
        for v in 0..words.len() {
            if o == v || (o != 0 && v != 0) {
                continue;
            }
            let (f, n) = relation::write_features_with_metadata(
                model,
                words,
                &addr,
                o,
                v,
                Some(&writer.role_context),
                oldgaps,
                &mut work,
            );
            let old_feature =
                super::writer_choice::feature(&lex, words.len(), o, v, &gaps, false, 2);
            let key = feature(old_feature);
            for action in 1..=3 {
                if wanted == Some((o, v, action)) {
                    target = Some(alternatives.len());
                }
                let baseline = relation::score(&writer.rows, &f[..n], action, &mut work)
                    + super::writer_lexical::score(
                        lexical,
                        &lex,
                        words.len(),
                        o,
                        v,
                        action,
                        &mut work,
                    )
                    + model.writer_choice.as_ref().map_or(0, |block| {
                        super::writer_choice::score(block, old_feature, action, &mut work)
                    });
                alternatives.push(Alternative {
                    baseline,
                    key: Some(relation::key(key, action)),
                });
            }
        }
    }
    let target =
        target.ok_or_else(|| Error("writer role labelled alternative unavailable".into()))?;
    if target != 0 && alternatives[target].baseline <= 0 {
        return Err(Error(format!(
            "writer role nonpositive correction cannot expose target: {} byte{} score{}",
            id, words[0].byte_end, alternatives[target].baseline
        )));
    }
    let result = Frame {
        alternatives,
        target,
        id: id.into(),
        end: words[0].byte_end,
        overridden: label.is_some(),
    };
    if label.is_none() && winner(&result, &BTreeMap::new()) != target {
        return Err(Error(
            "writer role parent cached/uncached selection differs".into(),
        ));
    }
    Ok(result)
}
impl Model {
    pub fn without_writer_role(&self) -> Result<Model> {
        self.writer_role
            .as_ref()
            .ok_or_else(|| Error("writer role absent".into()))?
            .parent(self)
    }
    pub fn fit_writer_role(
        &self,
        docs: &[WriterChoiceExample],
        epochs: usize,
        max_seconds: u64,
    ) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        if self.writer_role.is_some()
            || self.current_source.is_some()
            || self
                .writer_choice
                .as_ref()
                .is_none_or(|w| w.feature_layout != 2)
            || self.writer_lexical.is_none()
            || self.relation_writer.is_none()
            || docs.is_empty()
            || docs.len() > MAX_DOCUMENTS
            || !(1..=64).contains(&epochs)
            || !(1..=300).contains(&max_seconds)
            || docs.iter().map(|d| d.prompt.len()).sum::<usize>() > 8 * 1024 * 1024
        {
            return Err(Error("invalid writer role fit configuration".into()));
        }
        let start = Instant::now();
        let limit = Duration::from_secs(max_seconds);
        let mut ids = BTreeSet::new();
        let mut training = Vec::new();
        let mut frames = Vec::<Frame>::new();
        let mut indexed = BTreeMap::<Vec<Alternative>, usize>::new();
        let mut conflicts = Vec::new();
        let mut count = 0_usize;
        let mut labelled = 0_usize;
        for d in docs {
            if d.id.trim().is_empty()
                || !ids.insert(d.id.clone())
                || d.prompt.is_empty()
                || d.prompt.len() > 8192
                || d.overrides.len() > 32
                || d.overrides
                    .iter()
                    .map(|o| o.at_end_byte)
                    .collect::<BTreeSet<_>>()
                    .len()
                    != d.overrides.len()
            {
                return Err(Error("invalid writer role document/override bounds".into()));
            }
            let bytes = serde_json::to_vec(d).map_err(|e| Error(e.to_string()))?;
            training.push(DocumentReceipt {
                id: d.id.clone(),
                bytes: bytes.len(),
                text_cid: format!("blake3:{}", blake3::hash(&bytes).to_hex()),
            });
            let mut consumed = BTreeSet::new();
            super::writer_lexical::visit_words(self, &d.prompt, |s| {
                if start.elapsed() >= limit {
                    return Err(Error("writer role preparation time limit".into()));
                }
                let words = &s.recent[..s.recent_len.min(8)];
                let end = words[0].byte_end;
                let label = d.overrides.iter().find(|o| o.at_end_byte == end);
                if label.is_some() {
                    consumed.insert(end);
                    labelled += 1;
                }
                let f = frame(self, words, label, &d.id)?;
                count += 1;
                if let Some(&index) = indexed.get(&f.alternatives) {
                    let old = &frames[index];
                    if old.target != f.target && conflicts.len() < 64 {
                        conflicts.push(serde_json::json!({"first_id":old.id,"first_end_byte":old.end,"first_target":old.target,"id":f.id,"end_byte":f.end,"target":f.target,"overridden":f.overridden,"keys":f.alternatives.iter().map(|a|a.key).collect::<Vec<_>>()}));
                    }
                } else {
                    if frames.len() >= MAX_FRAMES {
                        return Err(Error("writer role frame cap32768".into()));
                    }
                    indexed.insert(f.alternatives.clone(), frames.len());
                    frames.push(f);
                }
                Ok(())
            })?;
            if consumed.len() != d.overrides.len() {
                return Err(Error(format!(
                    "writer role override boundary not observed: {}",
                    d.id
                )));
            }
        }
        if !conflicts.is_empty() {
            return Err(Error(format!(
                "writer role contradictory construction frames: {}",
                serde_json::json!({"conflicts":conflicts,"presented_boundaries":count,"frames":frames.len(),"labelled_boundaries":labelled})
            )));
        }
        let preparation_ms = start.elapsed().as_millis();
        let (rows, fit) = learn(&frames, epochs, start, limit)?;
        let mut model = self.clone();
        model.writer_role = Some(WriterRole {
            parent_artifact: self.artifact_cid().into(),
            rows,
            training,
            epochs,
            max_seconds,
        });
        model.refresh_identity()?;
        model.validate()?;
        let block = model
            .writer_role
            .as_ref()
            .ok_or_else(|| Error("writer role fitted block absent".into()))?;
        let report = serde_json::json!({"schema":"uor-r4.writer-role-fit/1","parent_artifact":self.artifact_cid(),"artifact":model.artifact_cid(),
            "documents":docs.len(),"presented_boundaries":count,"labelled_boundaries":labelled,"frames":frames.len(),"rows":block.rows.len(),"training":block.training,
            "preparation_ms":preparation_ms,"fit":fit,"epochs":epochs,"max_seconds":max_seconds,"elapsed_ms":start.elapsed().as_millis(),"dictionary_reused":true,
            "cache_preservation":"all new coefficients nonpositive; every parent certified NoWrite remains NoWrite", "parent_components_frozen":true,
            "feature_kind":17,
            "feature_law":"existing full lexical key plus10-bit earlier-endpoint exterior descriptor and4-bit directed endpoint separation; kind17 with only candidate value-prime omitted; no dictionary change",
            "scope":"Offline exact endpoint/action overrides; all other completed-word windows preserve parent winner; actual span commits and free generation require separate evaluation"});
        Ok((model, report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn old(value: u32, owner: u32, older: u32, gap: u16, distance: usize) -> ValueFeature {
        let mut addresses = [0; 8];
        addresses[0] = value;
        addresses[distance] = owner;
        addresses[distance + 1] = older;
        let mut gaps = [0; 8];
        gaps[distance] = gap;
        super::super::writer_choice::feature(&addresses, 8, distance, 0, &gaps, false, 2)
    }
    fn block(rows: Vec<ValueRow>) -> WriterRole {
        WriterRole {
            parent_artifact: "blake3:parent".into(),
            rows,
            training: vec![DocumentReceipt {
                id: "d".into(),
                text_cid: "blake3:receipt".into(),
                bytes: 1,
            }],
            epochs: 4,
            max_seconds: 120,
        }
    }
    fn alternative(baseline: i64, key: ValueFeature) -> Alternative {
        Alternative {
            baseline,
            key: Some(key),
        }
    }
    fn frame_for(alternatives: Vec<Alternative>, target: usize) -> Frame {
        let mut a = vec![Alternative {
            baseline: 0,
            key: None,
        }];
        a.extend(alternatives);
        Frame {
            alternatives: a,
            target,
            id: "role".into(),
            end: 42,
            overridden: true,
        }
    }
    #[test]
    fn writer_role_masks_only_value_identity_and_retains_context_fields() {
        let reference = feature(old(0, 479, 587, 33, 2));
        for value in [0, 2, 31, 719] {
            let full = old(value, 479, 587, 33, 2);
            let f = feature(full);
            assert_eq!(f, reference);
            assert_eq!(f.kind, 17);
            assert_eq!(f.a >> 32, 479);
            assert_eq!(f.a & 0xffffffff, 0);
            assert_eq!(f.b, full.b);
        }
        for other in [
            old(2, 0, 587, 33, 2),
            old(2, 479, 0, 33, 2),
            old(2, 479, 587, 289, 2),
            old(2, 479, 587, 33, 3),
        ] {
            assert_ne!(feature(other), reference);
        }
        // Opposite endpoint direction and both exterior prime fields survive.
        let mut addresses = [0; 8];
        addresses[1] = 719;
        addresses[2] = 479;
        addresses[4] = 587;
        addresses[5] = 353;
        let full = super::super::writer_choice::feature(&addresses, 8, 2, 4, &[512; 8], false, 2);
        let f = feature(full);
        assert_eq!(f.b, full.b);
        assert_eq!(f.b & 15, 6);
        assert_eq!((f.b >> 4) & 1023, 512);
        assert_eq!((f.b >> 14) >> 32, 353);
        assert_eq!((f.b >> 14) & 0xffffffff, 719);
    }
    #[test]
    fn writer_role_validation_rejects_erased_slot_payloads_and_invalid_context() {
        let primes = BTreeSet::from([0, 2, 31, 479, 587, 719]);
        let row = ValueRow {
            feature: relation::key(feature(old(2, 479, 587, 33, 2)), 1),
            weight: -3,
        };
        assert!(valid_row(&row, &primes));
        let mut bad = row.clone();
        bad.weight = 1;
        assert!(!valid_row(&bad, &primes));
        for feature in [
            ValueFeature {
                a: row.feature.a | 2,
                ..row.feature
            },
            ValueFeature {
                kind: 48,
                ..row.feature
            },
            ValueFeature {
                a: 720_u64 << 32,
                ..row.feature
            },
            ValueFeature {
                b: (row.feature.b & !15) | 8,
                ..row.feature
            },
            ValueFeature {
                b: row.feature.b & !15,
                ..row.feature
            },
            ValueFeature {
                b: (row.feature.b & !(1023 << 4)) | (513 << 4),
                ..row.feature
            },
        ] {
            assert!(!valid_row(
                &ValueRow {
                    feature,
                    ..row.clone()
                },
                &primes
            ));
        }
    }
    #[test]
    fn writer_role_full_parent_baseline_includes_existing_choice_correction() {
        let full = old(31, 479, 587, 33, 2);
        let role_key = relation::key(feature(full), 1);
        let prior = super::super::writer_choice::WriterChoice {
            parent_artifact: "p".into(),
            feature_layout: 2,
            rows: vec![ValueRow {
                feature: relation::key(full, 1),
                weight: -3,
            }],
            training: Vec::new(),
            epochs: 1,
            max_seconds: 1,
        };
        let baseline =
            3 + super::super::writer_choice::score(&prior, full, 1, &mut ValueWork::default());
        assert_eq!(baseline, 0);
        let correct_key = relation::key(feature(old(31, 587, 0, 33, 3)), 2);
        let f = frame_for(
            vec![alternative(baseline, role_key), alternative(1, correct_key)],
            2,
        );
        let (rows, report) = learn(&[f], 2, Instant::now(), Duration::from_secs(1)).unwrap();
        assert!(rows.is_empty());
        assert_eq!(report["final_correct"], 1);
        let f = frame_for(
            vec![alternative(3, role_key), alternative(1, correct_key)],
            2,
        );
        let (rows, report) = learn(&[f], 2, Instant::now(), Duration::from_secs(1)).unwrap();
        assert_eq!(rows[0].feature, role_key);
        assert_eq!(rows[0].weight, -3);
        assert_eq!(report["final_correct"], 1);
    }
    #[test]
    fn writer_role_preserves_directed_conflict_separation_and_reports_aliases() {
        let literal = relation::key(feature(old(2, 0, 0, 33, 3)), 2);
        let valid = relation::key(feature(old(31, 0, 0, 33, 2)), 2);
        assert_ne!(literal, valid);
        let correct = relation::key(feature(old(2, 479, 0, 289, 2)), 1);
        let rewrite = frame_for(vec![alternative(3, correct), alternative(6, literal)], 1);
        let preserve = frame_for(vec![alternative(2, valid)], 1);
        let (_, report) = learn(
            &[rewrite, preserve],
            4,
            Instant::now(),
            Duration::from_secs(1),
        )
        .unwrap();
        assert_eq!(report["final_correct"], 2);
        let alias = frame_for(vec![alternative(3, valid), alternative(1, valid)], 2);
        let (_, report) = learn(&[alias], 4, Instant::now(), Duration::from_secs(1)).unwrap();
        assert!(!report["impossible_constraints"]
            .as_array()
            .unwrap()
            .is_empty());
    }
    #[test]
    fn writer_role_nonpositive_scores_keep_no_write_and_controls_omit_only_new_operator() {
        let f = feature(old(2, 479, 587, 33, 2));
        let key = relation::key(f, 1);
        for weight in [-100, -3, 0] {
            let b = block(vec![ValueRow {
                feature: key,
                weight,
            }]);
            let delta = score(&b, f, 1, &mut ValueWork::default());
            assert!(delta <= 0);
            for baseline in [-5, -1, 0] {
                let frame = frame_for(vec![alternative(baseline, key)], 0);
                assert_eq!(winner(&frame, &BTreeMap::from([(key, weight)])), 0);
            }
        }
        assert!(enabled(Control::Full));
        assert!(!enabled(Control::WriterRoleDisabled));
        assert!(!enabled(Control::WriterRoleContextDisabled));
        assert!(enabled(Control::WriterChoiceDisabled));
    }
    #[test]
    fn writer_role_serialization_preserves_exact_context_keys_and_rejects_unknown_fields() {
        let original = block(vec![ValueRow {
            feature: relation::key(feature(old(2, 479, 587, 512, 3)), 2),
            weight: -3,
        }]);
        let bytes = serde_json::to_vec(&original).unwrap();
        let restored: WriterRole = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(original, restored);
        assert_eq!(bytes, serde_json::to_vec(&restored).unwrap());
        let mut wire = serde_json::to_value(&original).unwrap();
        wire["response"] = serde_json::json!("payload");
        assert!(serde_json::from_value::<WriterRole>(wire).is_err());
    }
}
