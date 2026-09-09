//! Learned nonpositive corrections to the existing owner/value/action writer.
//! Exact exterior gaps and directed endpoint separation condition frozen scores.
use super::relation;
use super::value_lexemes::WordAtom;
use super::value_types::{ValueFeature, ValueRow, ValueWork};
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

const MAX_DOCUMENTS: usize = 1200;
const MAX_FRAMES: usize = 32768;
const MAX_ROWS: usize = 16384;
const WEIGHT_FLOOR: i32 = -1000000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WriterChoice {
    pub parent_artifact: String,
    #[serde(default = "legacy_layout", skip_serializing_if = "is_legacy_layout")]
    pub feature_layout: u8,
    pub rows: Vec<ValueRow>,
    pub training: Vec<DocumentReceipt>,
    pub epochs: usize,
    pub max_seconds: u64,
}
fn legacy_layout() -> u8 {
    1
}
fn is_legacy_layout(layout: &u8) -> bool {
    *layout == 1
}

impl WriterChoice {
    fn parent(&self, model: &Model) -> Result<Model> {
        let mut parent = model.clone();
        parent.writer_choice = None;
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("writer choice frozen parent differs".into()));
        }
        parent.validate()?;
        Ok(parent)
    }
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        self.parent(model)?;
        let lexical = model
            .writer_lexical
            .as_ref()
            .ok_or_else(|| Error("writer choice requires inherited lexical dictionary".into()))?;
        if !matches!(self.feature_layout, 1 | 2)
            || lexical.dictionary.len() > 128
            || lexical.dictionary.iter().any(|d| d.prime > 719)
            || !(1..=64).contains(&self.epochs)
            || !(1..=300).contains(&self.max_seconds)
        {
            return Err(Error("invalid writer choice configuration".into()));
        }
        let primes: BTreeSet<_> = std::iter::once(0)
            .chain(lexical.dictionary.iter().map(|d| d.prime))
            .collect();
        if self.rows.len() > MAX_ROWS
            || self.rows.windows(2).any(|w| w[0].feature >= w[1].feature)
            || self
                .rows
                .iter()
                .any(|r| !valid_row(r, &primes, self.feature_layout))
        {
            return Err(Error("invalid writer choice residual rows".into()));
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
            return Err(Error("invalid writer choice training receipts".into()));
        }
        let mut identity = model.clone();
        identity.refresh_identity()?;
        if identity.artifact_cid != model.artifact_cid
            || identity.uor_model_address != model.uor_model_address
        {
            return Err(Error("writer choice identity differs".into()));
        }
        Ok(())
    }
}
fn valid_row(row: &ValueRow, primes: &BTreeSet<u32>, layout: u8) -> bool {
    let f = row.feature;
    let (old_b, gap) = match layout {
        1 => (f.b >> 10, f.b & 1023),
        2 if f.b & 15 != 0 && f.b & 15 != 8 => (f.b >> 14, (f.b >> 4) & 1023),
        _ => return false,
    };
    (-1000000..=0).contains(&row.weight)
        && f.kind & 31 == 16
        && (1..=3).contains(&(f.kind >> 5))
        && gap <= 512
        && [f.a >> 32, f.a & 0xffffffff, old_b >> 32, old_b & 0xffffffff]
            .iter()
            .all(|p| *p <= 719 && primes.contains(&(*p as u32)))
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
/// Layout1 preserves the original lexical key plus ten-bit exterior gap.
/// Layout2 also retains the parent writer's four-bit directed endpoint separation.
/// Prime bounds keep both layouts lossless. Gap zero denotes unavailable context
/// and is also the matched gap-erasure control; distance remains under that control.
pub(super) fn feature(
    addr: &[u32; 8],
    len: usize,
    owner: usize,
    value: usize,
    boundaries: &[u16; 8],
    erase_gap: bool,
    layout: u8,
) -> ValueFeature {
    let old = super::writer_lexical::feature(addr, len, owner, value);
    let gap = u64::from(if erase_gap {
        0
    } else {
        boundaries[owner.max(value)]
    });
    ValueFeature {
        kind: 16,
        a: old.a,
        b: if layout == 2 {
            (old.b << 14) | (gap << 4) | (owner + 8 - value) as u64
        } else {
            (old.b << 10) | gap
        },
    }
}
pub(super) fn score(
    block: &WriterChoice,
    feature: ValueFeature,
    action: u8,
    work: &mut ValueWork,
) -> i64 {
    work.relations.feature_writes = work.relations.feature_writes.saturating_add(1);
    relation::score(&block.rows, &[feature], action, work)
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum WriterChoiceTarget {
    NoWrite,
    Write {
        /// Zero-based index of the final owner byte in the prompt.
        owner_end_byte: u64,
        /// Zero-based index of the final value-anchor byte in the prompt.
        value_end_byte: u64,
        action: u8,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WriterChoiceOverride {
    /// Zero-based final-byte index of the completed word at this decision.
    pub at_end_byte: u64,
    pub target: WriterChoiceTarget,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WriterChoiceExample {
    pub id: String,
    pub prompt: String,
    pub overrides: Vec<WriterChoiceOverride>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Alternative {
    pub(super) baseline: i64,
    pub(super) key: Option<ValueFeature>,
}
#[derive(Debug, Clone)]
pub(super) struct Frame {
    pub(super) alternatives: Vec<Alternative>,
    pub(super) target: usize,
    pub(super) id: String,
    pub(super) end: u64,
    pub(super) overridden: bool,
}
fn scores(frame: &Frame, weights: &BTreeMap<ValueFeature, i32>) -> Vec<i64> {
    frame
        .alternatives
        .iter()
        .map(|a| {
            a.baseline
                + a.key
                    .and_then(|k| weights.get(&k))
                    .map_or(0, |w| i64::from(*w))
        })
        .collect()
}
pub(super) fn winner(frame: &Frame, weights: &BTreeMap<ValueFeature, i32>) -> usize {
    let mut best = 0;
    let mut selected = 0;
    for (i, a) in frame.alternatives.iter().enumerate().skip(1) {
        let s = a.baseline
            + a.key
                .and_then(|k| weights.get(&k))
                .map_or(0, |w| i64::from(*w));
        if s > best {
            best = s;
            selected = i;
        }
    }
    selected
}
fn correctness(frames: &[Frame], weights: &BTreeMap<ValueFeature, i32>) -> usize {
    frames
        .iter()
        .filter(|f| winner(f, weights) == f.target)
        .count()
}
pub(super) fn bind_target(
    words: &[WordAtom],
    label: &WriterChoiceOverride,
) -> Result<Option<(usize, usize, u8)>> {
    if words.first().map(|w| w.byte_end) != Some(label.at_end_byte) {
        return Err(Error("writer choice override boundary differs".into()));
    }
    match label.target {
        WriterChoiceTarget::NoWrite => Ok(None),
        WriterChoiceTarget::Write {
            owner_end_byte,
            value_end_byte,
            action,
        } => {
            if !(1..=3).contains(&action)
                || owner_end_byte == value_end_byte
                || owner_end_byte.max(value_end_byte) != label.at_end_byte
            {
                return Err(Error(
                    "invalid writer choice exact endpoint/action override".into(),
                ));
            }
            let find = |end| -> Result<usize> {
                let mut found = words.iter().enumerate().filter(|(_, w)| w.byte_end == end);
                let i = found.next().map(|(i, _)| i).ok_or_else(|| {
                    Error("writer choice endpoint outside completed-word window".into())
                })?;
                if found.next().is_some() {
                    return Err(Error("writer choice ambiguous endpoint".into()));
                }
                Ok(i)
            };
            let o = find(owner_end_byte)?;
            let v = find(value_end_byte)?;
            if o == v || (o != 0 && v != 0) {
                return Err(Error(
                    "writer choice endpoint outside candidate support".into(),
                ));
            }
            Ok(Some((o, v, action)))
        }
    }
}
fn frame(
    model: &Model,
    words: &[WordAtom],
    label: Option<&WriterChoiceOverride>,
    id: &str,
) -> Result<Frame> {
    let writer = model
        .relation_writer
        .as_ref()
        .ok_or_else(|| Error("writer choice parent writer absent".into()))?;
    let lexical = model
        .writer_lexical
        .as_ref()
        .ok_or_else(|| Error("writer choice lexical parent absent".into()))?;
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
            let key = feature(&lex, words.len(), o, v, &gaps, false, 2);
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
                    );
                alternatives.push(Alternative {
                    baseline,
                    key: Some(relation::key(key, action)),
                });
            }
        }
    }
    let target =
        target.ok_or_else(|| Error("writer choice labelled alternative unavailable".into()))?;
    if target != 0 && alternatives[target].baseline <= 0 {
        return Err(Error(format!(
            "writer choice nonpositive correction cannot expose target: {} byte{} score{}",
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
            "writer choice parent cached/uncached selection differs".into(),
        ));
    }
    Ok(result)
}
/// Greatest (least suppressive) feasible weights under w<=0. Each correction
/// imposes only the minimum integer separation required by the first-max law.
/// A target falling below NoWrite cannot be repaired by further suppression.
pub(super) fn learn(
    frames: &[Frame],
    epochs: usize,
    start: Instant,
    limit: Duration,
) -> Result<(Vec<ValueRow>, serde_json::Value)> {
    let mut weights = BTreeMap::<ValueFeature, i32>::new();
    let initial = correctness(frames, &weights);
    let mut best = weights.clone();
    let mut best_correct = initial;
    let mut updates = 0;
    let mut completed = 0;
    let mut stopped = false;
    let mut impossible = Vec::new();
    'epochs: for _ in 0..epochs {
        let before_updates = updates;
        for f in frames {
            if start.elapsed() >= limit {
                stopped = true;
                break 'epochs;
            }
            for _ in 0..f.alternatives.len() {
                let selected = winner(f, &weights);
                if selected == f.target {
                    break;
                }
                let s = scores(f, &weights);
                let target_score = s[f.target];
                let Some(key) = f.alternatives[selected].key else {
                    impossible.push(serde_json::json!({"id":f.id,"end_byte":f.end,"target":f.target,"target_key":f.alternatives[f.target].key,"target_score":target_score,"reason":"required suppression made target nonpositive"}));
                    break 'epochs;
                };
                if f.alternatives[f.target].key == Some(key) {
                    impossible.push(serde_json::json!({"id":f.id,"end_byte":f.end,"target":f.target,"selected":selected,"key":key,"reason":"winner and target share exact residual key"}));
                    break 'epochs;
                }
                let strict = i64::from(selected < f.target);
                let reduction = s[selected] - target_score + strict;
                let current = i64::from(*weights.get(&key).unwrap_or(&0));
                let next = current - reduction;
                if next < i64::from(WEIGHT_FLOOR) {
                    impossible.push(serde_json::json!({"id":f.id,"end_byte":f.end,"key":key,"reason":"required weight below configured floor"}));
                    break 'epochs;
                }
                weights.insert(key, next as i32);
                updates += 1;
                if weights.len() > MAX_ROWS {
                    return Err(Error("writer choice row cap16384".into()));
                }
            }
        }
        completed += 1;
        let correct = correctness(frames, &weights);
        if correct > best_correct {
            best_correct = correct;
            best = weights.clone();
        }
        if correct == frames.len() || updates == before_updates {
            break;
        }
    }
    let final_correct = correctness(frames, &weights);
    if final_correct > best_correct {
        best_correct = final_correct;
        best = weights;
    }
    let failures:Vec<_>=frames.iter().filter(|f|winner(f,&best)!=f.target).take(64).map(|f|serde_json::json!({"id":f.id,"end_byte":f.end,"overridden":f.overridden,"target":f.target,"selected":winner(f,&best),"scores":scores(f,&best),"keys":f.alternatives.iter().map(|a|a.key).collect::<Vec<_>>()})).collect();
    let rows = best
        .into_iter()
        .filter(|(_, w)| *w != 0)
        .map(|(feature, weight)| ValueRow { feature, weight })
        .collect();
    Ok((
        rows,
        serde_json::json!({"initial_correct":initial,"final_correct":best_correct,"frames":frames.len(),"updates":updates,"completed_epochs":completed,
        "stopped_at_time_limit":stopped,"impossible_constraints":impossible,"failed_frames":failures,"elapsed_ms":start.elapsed().as_millis(),
        "learning_rule":"minimum negative winner-key correction under exact first-maximum tie order; parent scores frozen"}),
    ))
}
impl Model {
    pub fn without_writer_choice(&self) -> Result<Model> {
        self.writer_choice
            .as_ref()
            .ok_or_else(|| Error("writer choice absent".into()))?
            .parent(self)
    }
    pub fn fit_writer_choice(
        &self,
        docs: &[WriterChoiceExample],
        epochs: usize,
        max_seconds: u64,
    ) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        if self.writer_choice.is_some()
            || self.writer_lexical.is_none()
            || self.relation_writer.is_none()
            || docs.is_empty()
            || docs.len() > MAX_DOCUMENTS
            || !(1..=64).contains(&epochs)
            || !(1..=300).contains(&max_seconds)
            || docs.iter().map(|d| d.prompt.len()).sum::<usize>() > 8 * 1024 * 1024
        {
            return Err(Error("invalid writer choice fit configuration".into()));
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
                return Err(Error(
                    "invalid writer choice document/override bounds".into(),
                ));
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
                    return Err(Error("writer choice preparation time limit".into()));
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
                        return Err(Error("writer choice frame cap32768".into()));
                    }
                    indexed.insert(f.alternatives.clone(), frames.len());
                    frames.push(f);
                }
                Ok(())
            })?;
            if consumed.len() != d.overrides.len() {
                return Err(Error(format!(
                    "writer choice override boundary not observed: {}",
                    d.id
                )));
            }
        }
        if !conflicts.is_empty() {
            return Err(Error(format!(
                "writer choice contradictory construction frames: {}",
                serde_json::json!({"conflicts":conflicts,"presented_boundaries":count,"frames":frames.len(),"labelled_boundaries":labelled})
            )));
        }
        let preparation_ms = start.elapsed().as_millis();
        let (rows, fit) = learn(&frames, epochs, start, limit)?;
        let mut model = self.clone();
        model.writer_choice = Some(WriterChoice {
            parent_artifact: self.artifact_cid().into(),
            feature_layout: 2,
            rows,
            training,
            epochs,
            max_seconds,
        });
        model.refresh_identity()?;
        model.validate()?;
        let block = model
            .writer_choice
            .as_ref()
            .ok_or_else(|| Error("writer choice fitted block absent".into()))?;
        let report = serde_json::json!({"schema":"uor-r4.writer-choice-fit/1","parent_artifact":self.artifact_cid(),"artifact":model.artifact_cid(),
            "documents":docs.len(),"presented_boundaries":count,"labelled_boundaries":labelled,"frames":frames.len(),"rows":block.rows.len(),"training":block.training,
            "preparation_ms":preparation_ms,"fit":fit,"epochs":epochs,"max_seconds":max_seconds,"elapsed_ms":start.elapsed().as_millis(),"dictionary_reused":true,
            "cache_preservation":"all new coefficients nonpositive; every parent certified NoWrite remains NoWrite", "parent_components_frozen":true,
            "feature_layout":block.feature_layout,
            "feature_law":"existing full lexical key plus10-bit earlier-endpoint exterior descriptor and4-bit directed endpoint separation; kind16 layout2; no dictionary change",
            "scope":"Offline exact endpoint/action overrides; all other completed-word windows preserve parent winner; actual span commits and free generation require separate evaluation"});
        Ok((model, report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn key(x: u64) -> ValueFeature {
        ValueFeature {
            kind: 48,
            a: x,
            b: 33,
        }
    }
    fn alt(score: i64, key: ValueFeature) -> Alternative {
        Alternative {
            baseline: score,
            key: Some(key),
        }
    }
    fn population(target: usize) -> Frame {
        Frame {
            alternatives: vec![
                Alternative {
                    baseline: 0,
                    key: None,
                },
                alt(3, key(1)),
                alt(1, key(2)),
            ],
            target,
            id: "contrast".into(),
            end: 47,
            overridden: true,
        }
    }
    #[test]
    fn writer_choice_packing_preserves_full_lexical_key_and_exact_exterior_gap() {
        for p in [0, 2, 479, 719] {
            for q in [0, 31, 587, 719] {
                let a = [q, 2, p, 587, 719, 0, 0, 0];
                let old = super::super::writer_lexical::feature(&a, 8, 2, 0);
                for gap in [0, 33, 257, 289, 512] {
                    let mut gaps = [0; 8];
                    gaps[2] = gap;
                    let f = feature(&a, 8, 2, 0, &gaps, false, 1);
                    assert_eq!(f.a, old.a);
                    assert_eq!(f.b >> 10, old.b);
                    assert_eq!(f.b & 1023, u64::from(gap));
                    assert_eq!(feature(&a, 8, 2, 0, &gaps, true, 1).b, old.b << 10);
                }
            }
        }
        let a = [31, 2, 479, 587, 0, 0, 0, 0];
        let mut adjacent = [0; 8];
        adjacent[2] = 33;
        let mut separated = adjacent;
        separated[2] = 289;
        assert_ne!(
            feature(&a, 8, 2, 0, &adjacent, false, 1),
            feature(&a, 8, 2, 0, &separated, false, 1)
        );
    }

    #[test]
    fn writer_choice_layout_two_retains_primes_gap_and_directed_separation() {
        let primes = BTreeSet::from([0, 2, 719]);
        let addr = [719; 8];
        let mut identities = BTreeSet::new();
        for owner in 0..8 {
            for value in 0..8 {
                if owner == value || (owner != 0 && value != 0) {
                    continue;
                }
                let old = super::super::writer_lexical::feature(&addr, 8, owner, value);
                for gap in [0, 33, 289, 512] {
                    let gaps = [gap; 8];
                    let f = feature(&addr, 8, owner, value, &gaps, false, 2);
                    assert_eq!(f.a, old.a);
                    assert_eq!(f.b >> 14, old.b);
                    assert_eq!((f.b >> 4) & 1023, u64::from(gap));
                    assert_eq!(f.b & 15, (owner + 8 - value) as u64);
                    assert!(f.b < (1_u64 << 56));
                    assert!(identities.insert(f));
                    let erased = feature(&addr, 8, owner, value, &gaps, true, 2);
                    assert_eq!(erased.b >> 14, old.b);
                    assert_eq!(erased.b & 16383, (owner + 8 - value) as u64);
                    let row = ValueRow {
                        feature: relation::key(f, 2),
                        weight: -3,
                    };
                    assert!(valid_row(&row, &primes, 2));
                    for invalid in [0, 8] {
                        let mut bad = row.clone();
                        bad.feature.b = (bad.feature.b & !15) | invalid;
                        assert!(!valid_row(&bad, &primes, 2));
                    }
                    let mut bad = row.clone();
                    bad.feature.b = (bad.feature.b & !(1023 << 4)) | (513 << 4);
                    assert!(!valid_row(&bad, &primes, 2));
                    bad.feature.b = ((720_u64 << 32) << 14) | (f.b & 16383);
                    assert!(!valid_row(&bad, &primes, 2));
                    assert!(!valid_row(&row, &primes, 0));
                    assert!(!valid_row(&row, &primes, 3));
                }
            }
        }
    }
    #[test]
    fn writer_choice_legacy_serialization_omits_layout_and_roundtrips_exactly() {
        let old = r#"{"parent_artifact":"blake3:parent","rows":[{"feature":{"kind":80,"a":0,"b":33},"weight":-3}],"training":[],"epochs":1,"max_seconds":120}"#;
        let legacy: WriterChoice = serde_json::from_str(old).unwrap();
        assert_eq!(legacy.feature_layout, 1);
        assert_eq!(serde_json::to_string(&legacy).unwrap(), old);
        let explicit = old.replacen("{", "{\"feature_layout\":1,", 1);
        let explicit: WriterChoice = serde_json::from_str(&explicit).unwrap();
        assert_eq!(serde_json::to_string(&explicit).unwrap(), old);
        let mut current = legacy;
        current.feature_layout = 2;
        current.rows[0].feature.b = (33 << 4) | 10;
        let encoded = serde_json::to_string(&current).unwrap();
        assert!(encoded.contains("\"feature_layout\":2"));
        assert_eq!(
            serde_json::from_str::<WriterChoice>(&encoded).unwrap(),
            current
        );
    }
    #[test]
    fn writer_choice_separation_repairs_literal_and_valid_revision_key_conflict() {
        let addr = [0; 8];
        let gaps = [33; 8];
        for layout in [1, 2] {
            let literal = relation::key(feature(&addr, 8, 3, 0, &gaps, false, layout), 2);
            let revision = relation::key(feature(&addr, 8, 2, 0, &gaps, false, layout), 2);
            let mut rewrite = population(2);
            rewrite.alternatives[1] = alt(6, literal);
            rewrite.alternatives[2] = alt(3, key(99));
            let mut preserve = population(1);
            preserve.overridden = false;
            preserve.id = "preserve-revision".into();
            preserve.alternatives = vec![
                Alternative {
                    baseline: 0,
                    key: None,
                },
                alt(2, revision),
            ];
            let (_, report) = learn(
                &[rewrite, preserve],
                4,
                Instant::now(),
                Duration::from_secs(1),
            )
            .unwrap();
            if layout == 1 {
                assert_eq!(literal, revision);
                assert!(report["final_correct"].as_u64().unwrap() < 2);
                assert!(!report["impossible_constraints"]
                    .as_array()
                    .unwrap()
                    .is_empty());
            } else {
                assert_ne!(literal, revision);
                assert_eq!(report["final_correct"], 2);
                assert!(report["impossible_constraints"]
                    .as_array()
                    .unwrap()
                    .is_empty());
            }
        }
    }
    #[test]
    fn writer_choice_minimal_suppression_exposes_existing_positive_target() {
        let f = population(2);
        let (rows, report) = learn(&[f], 4, Instant::now(), Duration::from_secs(1)).unwrap();
        assert_eq!(
            rows,
            vec![ValueRow {
                feature: key(1),
                weight: -3
            }]
        );
        assert_eq!(report["final_correct"], 1);
        // Lower-index target needs no strict extra decrement on a later tie.
        let mut f = population(1);
        f.alternatives[1].baseline = 1;
        f.alternatives[2].baseline = 3;
        let (rows, _) = learn(&[f], 4, Instant::now(), Duration::from_secs(1)).unwrap();
        assert_eq!(rows[0].weight, -2);
    }
    #[test]
    fn writer_choice_negative_rows_preserve_no_write_dominance() {
        let keys = [key(1), key(2)];
        for a in [-12, -1, 0] {
            for b in [-6, -1, 0] {
                for wa in [-100, -3, 0] {
                    for wb in [-100, -3, 0] {
                        let f = Frame {
                            alternatives: vec![
                                Alternative {
                                    baseline: 0,
                                    key: None,
                                },
                                alt(a, keys[0]),
                                alt(b, keys[1]),
                            ],
                            target: 0,
                            id: "cache".into(),
                            end: 0,
                            overridden: false,
                        };
                        assert_eq!(
                            winner(&f, &BTreeMap::from([(keys[0], wa), (keys[1], wb)])),
                            0
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn writer_choice_exact_override_binding_rejects_wrong_boundary_and_support() {
        let words = [
            WordAtom {
                byte_end: 47,
                ..Default::default()
            },
            WordAtom {
                byte_end: 40,
                ..Default::default()
            },
            WordAtom {
                byte_end: 37,
                ..Default::default()
            },
            WordAtom {
                byte_end: 33,
                ..Default::default()
            },
        ];
        let mut label = WriterChoiceOverride {
            at_end_byte: 47,
            target: WriterChoiceTarget::Write {
                owner_end_byte: 33,
                value_end_byte: 47,
                action: 2,
            },
        };
        assert_eq!(bind_target(&words, &label).unwrap(), Some((3, 0, 2)));
        label.at_end_byte = 46;
        assert!(bind_target(&words, &label).is_err());
        label.at_end_byte = 47;
        label.target = WriterChoiceTarget::Write {
            owner_end_byte: 33,
            value_end_byte: 40,
            action: 2,
        };
        assert!(bind_target(&words, &label).is_err());
        label.target = WriterChoiceTarget::Write {
            owner_end_byte: 99,
            value_end_byte: 47,
            action: 2,
        };
        assert!(bind_target(&words, &label).is_err());
        label.target = WriterChoiceTarget::NoWrite;
        assert_eq!(bind_target(&words, &label).unwrap(), None);
    }
    #[test]
    fn writer_choice_reports_shared_key_impossibility_and_preservation_conflict() {
        let mut alias = population(2);
        alias.alternatives[2].key = alias.alternatives[1].key;
        let (_, report) = learn(&[alias], 4, Instant::now(), Duration::from_secs(1)).unwrap();
        assert_eq!(
            report["impossible_constraints"].as_array().unwrap().len(),
            1
        );
        let rewrite = population(2);
        let mut preserve = population(1);
        preserve.overridden = false;
        let (_, report) = learn(
            &[rewrite, preserve],
            4,
            Instant::now(),
            Duration::from_secs(1),
        )
        .unwrap();
        assert!(report["final_correct"].as_u64().unwrap() < 2);
        assert!(!report["impossible_constraints"]
            .as_array()
            .unwrap()
            .is_empty());
    }
    #[test]
    fn writer_choice_row_validation_excludes_positive_weights_invalid_keys_and_primes() {
        let primes = BTreeSet::from([0, 2, 719]);
        let addr = [2, 0, 719, 2, 0, 0, 0, 0];
        let gaps = [33; 8];
        let row = ValueRow {
            feature: relation::key(feature(&addr, 8, 2, 0, &gaps, false, 1), 1),
            weight: -3,
        };
        assert!(valid_row(&row, &primes, 1));
        assert!(!valid_row(
            &ValueRow {
                weight: 1,
                ..row.clone()
            },
            &primes,
            1
        ));
        assert!(!valid_row(
            &ValueRow {
                feature: ValueFeature {
                    b: (row.feature.b & !1023) | 513,
                    ..row.feature
                },
                ..row.clone()
            },
            &primes,
            1
        ));
        assert!(!valid_row(
            &ValueRow {
                feature: ValueFeature {
                    a: (720_u64 << 32) | 2,
                    ..row.feature
                },
                ..row.clone()
            },
            &primes,
            1
        ));
        assert!(!valid_row(
            &ValueRow {
                feature: ValueFeature {
                    kind: 47,
                    ..row.feature
                },
                ..row
            },
            &primes,
            1
        ));
    }
}
