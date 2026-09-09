//! Learned nonpositive lexical context on the existing writer proposals.
//! Old NoWrite certificates remain sound: no proposal score can increase.
use super::relation::{self, RELATION_ROWS};
use super::value_lexemes::{LexemeState, WordAtom};
use super::value_types::{ValueEntry, ValueFeature, ValueRow};
use super::word_copy_types::WordCopyAddress;
use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::{Duration, Instant},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WriterLexicalExample {
    pub id: String,
    pub prompt: String,
    /// Inclusive source byte ends. At these completed-word boundaries every
    /// proposal is labelled NoWrite; all other boundaries preserve the parent.
    pub suppress_end_bytes: Vec<u64>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WriterLexical {
    pub parent_artifact: String,
    pub dictionary: Vec<WordCopyAddress>,
    pub rows: Vec<ValueRow>,
    pub training: Vec<DocumentReceipt>,
    pub epochs: usize,
    pub max_seconds: u64,
}
impl WriterLexical {
    pub(super) fn parent(&self, model: &Model) -> Result<Model> {
        let mut parent = model.clone();
        parent.writer_lexical = None;
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("writer lexical frozen parent differs".into()));
        }
        parent.validate()?;
        Ok(parent)
    }
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        self.parent(model)?;
        if !(1..=64).contains(&self.epochs)
            || !(1..=120).contains(&self.max_seconds)
            || self.dictionary.is_empty()
            || self.dictionary.len() > 128
        {
            return Err(Error("invalid writer lexical configuration".into()));
        }
        let primes = crate::corpus_induced_spin_placement::first_primes(self.dictionary.len())
            .map_err(|e| Error(e.to_string()))?;
        if self.dictionary.iter().zip(&primes).any(|(w, p)| {
            w.len == 0
                || w.len > 32
                || u64::from(w.prime) != *p
                || w.bytes[..usize::from(w.len)]
                    .iter()
                    .any(|b| !b.is_ascii_alphanumeric() && *b != b'_')
                || w.bytes[usize::from(w.len)..].iter().any(|b| *b != 0)
        }) || self.dictionary.windows(2).any(|w| w[0].bytes >= w[1].bytes)
        {
            return Err(Error("invalid writer lexical dictionary".into()));
        }
        let allowed: BTreeSet<_> = std::iter::once(0)
            .chain(self.dictionary.iter().map(|d| d.prime))
            .collect();
        if self.rows.len() > RELATION_ROWS
            || self.rows.windows(2).any(|w| w[0].feature >= w[1].feature)
            || self.rows.iter().any(|r| {
                !(-1000000..=0).contains(&r.weight)
                    || (r.feature.kind & 31) != 15
                    || !(1..=3).contains(&(r.feature.kind >> 5))
                    || [
                        r.feature.a >> 32,
                        r.feature.a & 0xffffffff,
                        r.feature.b >> 32,
                        r.feature.b & 0xffffffff,
                    ]
                    .iter()
                    .any(|p| !allowed.contains(&(*p as u32)))
            })
        {
            return Err(Error("invalid writer lexical residual rows".into()));
        }
        if self.training.is_empty()
            || self.training.len() > 1024
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
            return Err(Error("invalid writer lexical receipts".into()));
        }
        let mut identity = model.clone();
        identity.refresh_identity()?;
        if identity.artifact_cid != model.artifact_cid
            || identity.uor_model_address != model.uor_model_address
        {
            return Err(Error("writer lexical identity differs".into()));
        }
        Ok(())
    }
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
pub(super) fn addresses(
    dictionary: &[WordCopyAddress],
    words: &[WordAtom],
    work: &mut ValueWork,
) -> [u32; 8] {
    let mut out = [0; 8];
    for (i, w) in words.iter().take(8).enumerate() {
        work.relations.record_reads = work.relations.record_reads.saturating_add(1);
        let found = dictionary.binary_search_by(|d| {
            work.relations.dictionary_comparisons =
                work.relations.dictionary_comparisons.saturating_add(1);
            for j in 0..usize::from(w.len.min(d.len)) {
                work.relations.dictionary_byte_comparisons =
                    work.relations.dictionary_byte_comparisons.saturating_add(1);
                let cmp = d.bytes[j].cmp(&w.bytes[j]);
                if !cmp.is_eq() {
                    return cmp;
                }
            }
            d.len.cmp(&w.len)
        });
        out[i] = found.map_or(0, |j| dictionary[j].prime);
    }
    out
}
pub(super) fn feature(addr: &[u32; 8], len: usize, owner: usize, value: usize) -> ValueFeature {
    let older = owner.max(value) + 1;
    let newer = owner.min(value).checked_sub(1);
    ValueFeature {
        kind: 15,
        a: (u64::from(addr[owner]) << 32) | u64::from(addr[value]),
        b: (u64::from(if older < len { addr[older] } else { 0 }) << 32)
            | u64::from(newer.map_or(0, |i| addr[i])),
    }
}
pub(super) fn score(
    block: &WriterLexical,
    addr: &[u32; 8],
    len: usize,
    owner: usize,
    value: usize,
    action: u8,
    work: &mut ValueWork,
) -> i64 {
    let f = feature(addr, len, owner, value);
    work.relations.feature_writes = work.relations.feature_writes.saturating_add(1);
    relation::score(&block.rows, &[f], action, work)
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Alternative {
    baseline: i64,
    key: Option<ValueFeature>,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Frame {
    alternatives: Vec<Alternative>,
    target: usize,
}
fn winner(frame: &Frame, weights: &BTreeMap<ValueFeature, i32>) -> usize {
    let mut winner = 0;
    let mut best = 0;
    for (i, a) in frame.alternatives.iter().enumerate().skip(1) {
        let score = a.baseline
            + a.key
                .and_then(|k| weights.get(&k))
                .map_or(0, |v| i64::from(*v));
        if score > best {
            best = score;
            winner = i;
        }
    }
    winner
}
fn correctness(frames: &[Frame], weights: &BTreeMap<ValueFeature, i32>) -> usize {
    frames
        .iter()
        .filter(|f| winner(f, weights) == f.target)
        .count()
}
pub(super) fn visit_words(
    model: &Model,
    prompt: &str,
    mut capture: impl FnMut(&LexemeState) -> Result<()>,
) -> Result<()> {
    let mut words = LexemeState::default();
    let mut last = None;
    for (sequence, token) in model.encode(prompt)?.into_iter().enumerate() {
        let byte;
        let bytes = if token < LEXICAL_BASE {
            byte = [(token - 2) as u8];
            &byte[..]
        } else {
            &model.lexical_pieces[(token - LEXICAL_BASE) as usize][..]
        };
        for &b in bytes {
            words.feed(
                b,
                ValueEntry {
                    sequence: sequence as u64,
                    token,
                    pose: model.geometry.identity,
                    ..Default::default()
                },
                &mut ValueWork::default(),
            );
            if words.recent_len > 0 && last != Some(words.recent[0].byte_end) {
                last = Some(words.recent[0].byte_end);
                capture(&words)?;
            }
        }
    }
    words.finish(&mut ValueWork::default());
    if words.recent_len > 0 && last != Some(words.recent[0].byte_end) {
        capture(&words)?;
    }
    Ok(())
}
impl Model {
    pub fn without_writer_lexical(&self) -> Result<Model> {
        self.writer_lexical
            .as_ref()
            .ok_or_else(|| Error("writer lexical witness absent".into()))?
            .parent(self)
    }
    pub fn fit_writer_lexical(
        &self,
        docs: &[WriterLexicalExample],
        epochs: usize,
        max_seconds: u64,
    ) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        if self.writer_lexical.is_some()
            || docs.is_empty()
            || docs.len() > 1024
            || !(1..=64).contains(&epochs)
            || !(1..=120).contains(&max_seconds)
            || docs.iter().map(|d| d.prompt.len()).sum::<usize>() > 4 * 1024 * 1024
        {
            return Err(Error("invalid writer lexical training bounds".into()));
        }
        let writer = self
            .relation_writer
            .as_ref()
            .ok_or_else(|| Error("writer lexical parent writer absent".into()))?;
        if writer.role_context.is_empty() {
            return Err(Error(
                "writer lexical requires word-local role geometry".into(),
            ));
        }
        let started = Instant::now();
        let limit = Duration::from_secs(max_seconds);
        let mut ids = BTreeSet::new();
        let mut training = Vec::new();
        let mut frequency = BTreeMap::<Vec<u8>, usize>::new();
        for d in docs {
            if d.id.trim().is_empty()
                || !ids.insert(&d.id)
                || d.prompt.is_empty()
                || d.prompt.len() > 4096
                || d.suppress_end_bytes.iter().collect::<BTreeSet<_>>().len()
                    != d.suppress_end_bytes.len()
            {
                return Err(Error("invalid writer lexical document".into()));
            }
            training.push(super::training::receipt(&Document {
                id: d.id.clone(),
                text: serde_json::to_string(d).map_err(|e| Error(e.to_string()))?,
            }));
            let mut present = BTreeSet::new();
            visit_words(self, &d.prompt, |s| {
                if started.elapsed() >= limit {
                    return Err(Error("writer lexical preparation time bound".into()));
                }
                let w = &s.recent[0];
                present.insert(w.bytes[..usize::from(w.len)].to_vec());
                Ok(())
            })?;
            for w in present {
                *frequency.entry(w).or_default() += 1;
            }
        }
        let vocabulary_total = frequency.len();
        let mut selected: Vec<_> = frequency.into_iter().collect();
        selected.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        selected.truncate(128);
        selected.sort_by(|a, b| a.0.cmp(&b.0));
        let primes = crate::corpus_induced_spin_placement::first_primes(selected.len())
            .map_err(|e| Error(e.to_string()))?;
        let dictionary: Vec<_> = selected
            .iter()
            .zip(primes)
            .map(|((bytes, _), prime)| {
                let mut padded = [0; 32];
                padded[..bytes.len()].copy_from_slice(bytes);
                WordCopyAddress {
                    bytes: padded,
                    len: bytes.len() as u8,
                    prime: prime as u32,
                }
            })
            .collect();
        let mut unique = BTreeSet::new();
        let mut dose = 0_usize;
        let mut suppressed = 0_usize;
        let mut observations = Vec::new();
        for d in docs {
            let mut consumed = BTreeSet::new();
            visit_words(self, &d.prompt, |s| {
                if started.elapsed() >= limit {
                    return Err(Error("writer lexical preparation time bound".into()));
                }
                let words = &s.recent[..s.recent_len.min(8)];
                let end = words[0].byte_end;
                let suppress = d.suppress_end_bytes.contains(&end);
                if suppress {
                    consumed.insert(end);
                    suppressed += 1;
                }
                let mut work = ValueWork::default();
                let oldaddr = relation::writer_addresses(self, words, &mut work);
                let addr = addresses(&dictionary, words, &mut work);
                let actual = relation::write_choice(self, words, &mut work);
                let mut alternatives = vec![Alternative {
                    baseline: 0,
                    key: None,
                }];
                let mut target = None;
                if actual.is_none() || suppress {
                    target = Some(0);
                }
                for owner in 0..words.len() {
                    for value in 0..words.len() {
                        if owner == value || (owner != 0 && value != 0) {
                            continue;
                        }
                        let (features, n) = relation::write_features_with_context(
                            self,
                            words,
                            &oldaddr,
                            owner,
                            value,
                            Some(&writer.role_context),
                            &mut work,
                        );
                        for action in 1..=3 {
                            if !suppress && actual == Some((owner, value, action)) {
                                target = Some(alternatives.len());
                            }
                            alternatives.push(Alternative {
                                baseline: relation::score(
                                    &writer.rows,
                                    &features[..n],
                                    action,
                                    &mut work,
                                ),
                                key: Some(relation::key(
                                    feature(&addr, words.len(), owner, value),
                                    action,
                                )),
                            });
                        }
                    }
                }
                let target = target
                    .ok_or_else(|| Error("writer lexical exact parent target absent".into()))?;
                if suppress {
                    observations.push(serde_json::json!({"id":d.id,"end_byte":end,"parent_proposal":actual,"target":"no_write"}));
                }
                dose += 1;
                unique.insert(Frame {
                    alternatives,
                    target,
                });
                if unique.len() > 8192 {
                    return Err(Error("writer lexical frame cap8192".into()));
                }
                Ok(())
            })?;
            if consumed.len() != d.suppress_end_bytes.len() {
                return Err(Error(format!(
                    "writer lexical labelled byte end not observed: {}",
                    d.id
                )));
            }
        }
        let frames: Vec<_> = unique.into_iter().collect();
        let mut weights = BTreeMap::new();
        for f in &frames {
            if f.target == 0 {
                for a in &f.alternatives {
                    if a.baseline > 0 {
                        if let Some(k) = a.key {
                            weights.entry(k).or_insert(0_i32);
                        }
                    }
                }
            }
        }
        if weights.len() > RELATION_ROWS {
            return Err(Error("writer lexical row cap16384".into()));
        }
        let initial = correctness(&frames, &weights);
        let mut best_correct = initial;
        let mut best = weights.clone();
        let mut updates = 0;
        let mut completed_epochs = 0;
        let mut stopped = false;
        'epochs: for _ in 0..epochs {
            for f in &frames {
                if started.elapsed() >= limit {
                    stopped = true;
                    break 'epochs;
                }
                let selected = winner(f, &weights);
                if selected == f.target {
                    continue;
                }
                if let Some(key) = f.alternatives[selected].key {
                    if let Some(w) = weights.get_mut(&key) {
                        *w = (*w - 8).max(-1000000);
                    }
                }
                if let Some(key) = f.alternatives[f.target].key {
                    if let Some(w) = weights.get_mut(&key) {
                        *w = (*w + 8).min(0);
                    }
                }
                updates += 1;
            }
            completed_epochs += 1;
            let correct = correctness(&frames, &weights);
            if correct > best_correct {
                best_correct = correct;
                best = weights.clone();
            }
            if correct == frames.len() {
                break;
            }
        }
        // Retain the best complete population, including the last partial epoch.
        let final_correct = correctness(&frames, &weights);
        if final_correct > best_correct {
            best_correct = final_correct;
            best = weights;
        }
        let rows: Vec<_> = best
            .into_iter()
            .filter(|(_, w)| *w != 0)
            .map(|(feature, weight)| ValueRow { feature, weight })
            .collect();
        let final_weights: BTreeMap<_, _> = rows.iter().map(|r| (r.feature, r.weight)).collect();
        let failed:Vec<_>=frames.iter().enumerate().filter(|(_,f)|winner(f,&final_weights)!=f.target).take(32).map(|(i,f)|serde_json::json!({"frame":i,"target":f.target,"alternatives":f.alternatives.iter().map(|a|serde_json::json!({"baseline":a.baseline,"key":a.key})).collect::<Vec<_>>()})).collect();
        let mut model = self.clone();
        model.writer_lexical = Some(WriterLexical {
            parent_artifact: self.artifact_cid.clone(),
            dictionary,
            rows,
            training,
            epochs,
            max_seconds,
        });
        model.refresh_identity()?;
        model.validate()?;
        let block = model
            .writer_lexical
            .as_ref()
            .ok_or_else(|| Error("writer lexical fit result absent".into()))?;
        let report = serde_json::json!({"schema":"uor-r4.writer-lexical-fit/1","parent":self.artifact_cid(),"artifact":model.artifact_cid(),"documents":docs.len(),"presented_boundaries":dose,"frames":frames.len(),"suppressed_boundaries":suppressed,"suppression_labels":observations,"dictionary_total":vocabulary_total,"dictionary_selected":selected.len(),"dictionary_selection":selected.iter().map(|(b,n)|serde_json::json!({"word":String::from_utf8_lossy(b),"document_presence":n})).collect::<Vec<_>>(),"initial_correct":initial,"final_correct":best_correct,"epochs":completed_epochs,"updates":updates,"rows":block.rows.len(),"stopped_at_time_limit":stopped,"elapsed_ms":started.elapsed().as_millis(),"failed_frames":failed,"scope":"Offline completed-word proposal supervision; generic nonpositive lexical residual; frozen writer/cache sound by score dominance. Actual committed writes and free generation are evaluated separately."});
        Ok((model, report))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn writer_lexical_endpoint_context_is_ordered_and_lossless() {
        let a = [2, 3, 5, 7, 11, 0, 0, 0];
        let f = feature(&a, 5, 2, 0);
        assert_eq!(f.a, (5_u64 << 32) | 2);
        assert_eq!(f.b, 7_u64 << 32);
        assert_ne!(f, feature(&a, 5, 0, 2));
        let mut b = a;
        b[3] = 13;
        assert_ne!(f, feature(&b, 5, 2, 0));
    }
    #[test]
    fn writer_lexical_nonpositive_scores_preserve_no_write() {
        let key = ValueFeature {
            kind: 47,
            a: 0,
            b: 0,
        };
        let f = Frame {
            alternatives: vec![
                Alternative {
                    baseline: 0,
                    key: None,
                },
                Alternative {
                    baseline: 0,
                    key: Some(key),
                },
            ],
            target: 0,
        };
        assert_eq!(winner(&f, &BTreeMap::new()), 0);
        assert_eq!(winner(&f, &BTreeMap::from([(key, -8)])), 0);
    }
}
