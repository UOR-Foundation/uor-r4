//! Offline construction supervision for relation writes and exact reads.
//! Source byte labels are fit-only; the serving path receives raw tokens.
use super::relation::*;
use super::value_types::{ValueFeature, ValueRow, ValueState, ValueWork};
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationLabel {
    pub owner_end_byte: u64,
    pub value_end_byte: u64,
    pub action: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationExample {
    pub id: String,
    pub prompt: String,
    pub response: String,
    pub writes: Vec<RelationLabel>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Alternative {
    keys: Vec<ValueFeature>,
    correct: bool,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Frame {
    alternatives: Vec<Alternative>,
}

fn fit(frames: &[Frame], epochs: usize) -> Result<(Vec<ValueRow>, usize)> {
    let mut weights = BTreeMap::<ValueFeature, i32>::new();
    for f in frames {
        for a in &f.alternatives {
            for k in &a.keys {
                weights.entry(*k).or_default();
            }
        }
    }
    if weights.len() > RELATION_ROWS {
        return Err(Error("relation rows exceed bound".into()));
    }
    let mut best = weights.clone();
    let mut best_correct = 0;
    for _ in 0..epochs {
        for f in frames {
            let scores: Vec<i64> = f
                .alternatives
                .iter()
                .map(|a| a.keys.iter().map(|k| i64::from(weights[k])).sum())
                .collect();
            let mut winner = 0;
            let mut wanted = None;
            for (i, a) in f.alternatives.iter().enumerate() {
                if scores[i] > scores[winner] {
                    winner = i;
                }
                if a.correct && wanted.is_none_or(|j: usize| scores[i] > scores[j]) {
                    wanted = Some(i);
                }
            }
            let wanted = wanted.ok_or_else(|| Error("relation target unreachable".into()))?;
            if !f.alternatives[winner].correct || scores[wanted] < scores[winner] + 8 {
                for k in &f.alternatives[winner].keys {
                    if let Some(w) = weights.get_mut(k) {
                        *w = (*w - 1).max(-1000000);
                    }
                }
                for k in &f.alternatives[wanted].keys {
                    if let Some(w) = weights.get_mut(k) {
                        *w = (*w + 1).min(1000000);
                    }
                }
            }
        }
        let correct = frames
            .iter()
            .filter(|f| {
                let mut winner = 0;
                let mut best_score = i64::MIN;
                for (i, a) in f.alternatives.iter().enumerate() {
                    let s: i64 = a.keys.iter().map(|k| i64::from(weights[k])).sum();
                    if s > best_score {
                        best_score = s;
                        winner = i;
                    }
                }
                f.alternatives[winner].correct
            })
            .count();
        if correct > best_correct {
            best_correct = correct;
            best = weights.clone();
        }
        if correct == frames.len() {
            break;
        }
    }
    Ok((
        best.into_iter()
            .map(|(feature, weight)| ValueRow { feature, weight })
            .collect(),
        best_correct,
    ))
}

fn writer_frame(
    model: &Model,
    words: &super::value_lexemes::LexemeState,
    label: Option<&RelationLabel>,
    context: Option<&[TokenGeometry]>,
) -> Result<Frame> {
    let words = &words.recent[..words.recent_len.min(8)];
    let mut work = ValueWork::default();
    let addr = writer_addresses(model, words, &mut work);
    let mut alternatives = vec![Alternative {
        keys: Vec::new(),
        correct: label.is_none(),
    }];
    for o in 0..words.len() {
        for v in 0..words.len() {
            if o == v || (o != 0 && v != 0) {
                continue;
            }
            let (f, n) = write_features_with_context(model, words, &addr, o, v, context, &mut work);
            for a in 1..=3 {
                alternatives.push(Alternative {
                    keys: f[..n].iter().map(|f| key(*f, a)).collect(),
                    correct: label.is_some_and(|l| {
                        l.action == a
                            && l.owner_end_byte == words[o].byte_end
                            && l.value_end_byte == words[v].byte_end
                    }),
                });
            }
        }
    }
    if !alternatives.iter().any(|a| a.correct) {
        return Err(Error(
            "relation labeled source outside local write candidates".into(),
        ));
    }
    Ok(Frame { alternatives })
}

impl RelationModel {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        let valid = |rows: &[ValueRow]| {
            !rows.is_empty()
                && rows.len() <= RELATION_ROWS
                && rows.windows(2).all(|p| p[0].feature < p[1].feature)
                && rows.iter().all(|r| {
                    (-1000000..=1000000).contains(&r.weight)
                        && (r.feature.kind & 31) < 20
                        && (1..=3).contains(&(r.feature.kind >> 5))
                })
        };
        if !matches!(
            self.schema.as_str(),
            "uor-r4.exact-relation/1" | "uor-r4.exact-relation/2"
        ) || !valid(&self.writer)
            || !valid(&self.reader)
            || !(1..=64).contains(&self.epochs)
            || self.training.is_empty()
            || self.training.len() > 256
            || self.training.iter().any(|r| r.id.is_empty())
            || self
                .training
                .iter()
                .map(|r| &r.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.training.len()
        {
            return Err(Error("invalid learned relation model".into()));
        }
        if (self.schema == "uor-r4.exact-relation/1" && !self.role_context.is_empty())
            || (self.schema == "uor-r4.exact-relation/2"
                && self.role_context != compile_role_context(model)?)
        {
            return Err(Error(
                "relation role transport differs from parent tokenizer/geometry".into(),
            ));
        }
        let mut parent = model.clone();
        let role = parent
            .response_entry
            .as_mut()
            .and_then(|e| e.copy.as_mut())
            .and_then(|c| c.role_read.as_mut())
            .ok_or_else(|| Error("relation role parent missing".into()))?;
        if role.actions.len() != 3 {
            return Err(Error(
                "relation reader requires three parent entry actions".into(),
            ));
        }
        role.relations = None;
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent {
            return Err(Error("relation parent identity differs".into()));
        }
        if let Some(gate) = &self.admission {
            gate.validate(model)?;
        }
        Ok(())
    }
}

fn compile_role_context(model: &Model) -> Result<Vec<TokenGeometry>> {
    let role = super::role_read::head(model)
        .ok_or_else(|| Error("relation role parent missing".into()))?;
    compile_context(model, &role.dictionary)
}

fn compile_context(
    model: &Model,
    dictionary: &[super::word_copy_types::WordCopyAddress],
) -> Result<Vec<TokenGeometry>> {
    let mut rows = Vec::with_capacity(dictionary.len());
    for word in dictionary {
        let text = std::str::from_utf8(&word.bytes[..usize::from(word.len)])
            .map_err(|e| Error(e.to_string()))?;
        let mut row = TokenGeometry {
            prime: word.prime,
            leaf: model.geometry.identity,
            phases: [0; PHASE_CHANNELS],
        };
        for token in model.encode(text)? {
            let g = &model.geometry.tokens[token as usize];
            row.leaf = model.geometry.products
                [model.geometry.row_bases[usize::from(row.leaf)] + usize::from(g.leaf)];
            for (p, d) in row.phases.iter_mut().zip(g.phases) {
                *p = p.wrapping_add(d);
            }
        }
        rows.push(row);
    }
    rows.sort_by_key(|r| r.prime);
    Ok(rows)
}

impl Model {
    pub fn relation_training(&self) -> &[DocumentReceipt] {
        head(self).map_or(&[], |h| h.training.as_slice())
    }
    pub fn fit_relations(
        &self,
        documents: &[RelationExample],
        epochs: usize,
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_relations_mode(documents, epochs, false, false)
    }

    /// Learn participant-independent role context while preserving exact values.
    pub fn fit_relations_with_role_paths(
        &self,
        documents: &[RelationExample],
        epochs: usize,
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_relations_mode(documents, epochs, true, false)
    }

    /// Replace only the writer; all reader/operator parameters remain frozen.
    pub fn refit_relation_writer(
        &self,
        documents: &[RelationExample],
        epochs: usize,
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_relations_mode(documents, epochs, true, true)
    }

    fn fit_relations_mode(
        &self,
        documents: &[RelationExample],
        epochs: usize,
        role_paths: bool,
        writer_only: bool,
    ) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        if self.relation_writer.is_some()
            || (writer_only && head(self).is_none_or(|h| h.schema != "uor-r4.exact-relation/2"))
            || (!writer_only && head(self).is_some())
            || super::role_read::head(self).is_none()
            || documents.is_empty()
            || documents.len() > 256
            || !(1..=64).contains(&epochs)
            || documents
                .iter()
                .map(|d| d.prompt.len() + d.response.len())
                .sum::<usize>()
                > 1024 * 1024
        {
            return Err(Error("invalid relation fitting source/config".into()));
        }
        let mut template = self.clone();
        if writer_only {
            let dictionary = writer_dictionary(self, documents)?;
            let role_context = compile_context(self, &dictionary)?;
            template.relation_writer = Some(WriterRevision {
                schema: "uor-r4.relation-writer/1".into(),
                parent: self.artifact_cid.clone(),
                dictionary,
                role_context,
                rows: Vec::new(),
                training: Vec::new(),
                epochs,
                reuse_admission: false,
                admission: None,
            });
        }
        let role_context = if let Some(writer) = &template.relation_writer {
            writer.role_context.clone()
        } else if role_paths {
            compile_role_context(self)?
        } else {
            Vec::new()
        };
        let context = role_paths.then_some(role_context.as_slice());
        let mut frames = BTreeSet::new();
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut examples = Vec::new();
        for d in documents {
            if d.id.is_empty()
                || !ids.insert(&d.id)
                || d.writes.len() > 16
                || d.writes.iter().any(|l| !(1..=3).contains(&l.action))
            {
                return Err(Error("invalid relation supervision".into()));
            }
            receipts.push(super::training::receipt(&Document {
                id: d.id.clone(),
                text: serde_json::to_string(d).map_err(|e| Error(e.to_string()))?,
            }));
            let mut words = super::value_lexemes::LexemeState::default();
            let mut last = None;
            let mut consumed = BTreeSet::new();
            let mut expected = RelationState::default();
            let mut pose = self.geometry.identity;
            let mut phases = [0_u16; PHASE_CHANNELS];
            let mut seen = 0;
            // Match the actual tokenizer and per-token geometry while exposing
            // every completed word inside a lexical piece, as serving does.
            for (sequence, token) in std::iter::once(BOS)
                .chain(self.encode(&d.prompt)?)
                .enumerate()
            {
                let g = &self.geometry.tokens[token as usize];
                pose = self.geometry.products
                    [self.geometry.row_bases[usize::from(pose)] + usize::from(g.leaf)];
                for (p, d) in phases.iter_mut().zip(g.phases) {
                    *p = p.wrapping_add(d);
                }
                seen = sequence as u64 + 1;
                if token == BOS || token == EOS {
                    continue;
                }
                let single;
                let bytes = if token < LEXICAL_BASE {
                    single = [(token - 2) as u8];
                    &single[..]
                } else {
                    &self.lexical_pieces[(token - LEXICAL_BASE) as usize][..]
                };
                for &b in bytes {
                    words.feed(
                        b,
                        super::value_types::ValueEntry {
                            sequence: sequence as u64,
                            token,
                            cue: g.prime,
                            pose,
                            phases,
                        },
                        &mut ValueWork::default(),
                    );
                    if words.recent_len == 0 || last == Some(words.recent[0].byte_end) {
                        continue;
                    }
                    last = Some(words.recent[0].byte_end);
                    let labels: Vec<_> = d
                        .writes
                        .iter()
                        .enumerate()
                        .filter(|(_, l)| {
                            l.owner_end_byte.max(l.value_end_byte) == words.recent[0].byte_end
                        })
                        .collect();
                    if labels.len() > 1 {
                        return Err(Error("multiple labels at one relation boundary".into()));
                    }
                    let label = labels.first().map(|(_, l)| *l);
                    frames.insert(writer_frame(&template, &words, label, context)?);
                    if let Some((i, l)) = labels.first() {
                        consumed.insert(*i);
                        let o = words
                            .recent
                            .iter()
                            .find(|w| w.len != 0 && w.byte_end == l.owner_end_byte)
                            .ok_or_else(|| Error("owner label absent".into()))?;
                        let v = words
                            .recent
                            .iter()
                            .find(|w| w.len != 0 && w.byte_end == l.value_end_byte)
                            .ok_or_else(|| Error("value label absent".into()))?;
                        expected.commit(*o, *v, l.action, &mut ValueWork::default());
                    }
                }
            }
            if consumed.len() != d.writes.len() {
                return Err(Error("relation labels were not observed".into()));
            }
            words.finish(&mut ValueWork::default());
            words.begin();
            let mut values = ValueState::new(self);
            values.lexemes = Some(words);
            values.seen = seen;
            values.pose = pose;
            values.phases = phases;
            values.relations = Some(expected);
            examples.push((values, d));
        }
        let frames: Vec<_> = frames.into_iter().collect();
        if frames.len() > 8192 {
            return Err(Error("relation frame bound exceeded".into()));
        }
        let (writer, write_correct) = fit(&frames, epochs)?;
        if writer_only {
            let writer_head = template
                .relation_writer
                .as_mut()
                .ok_or_else(|| Error("writer absent".into()))?;
            writer_head.rows = writer;
            writer_head.training = receipts;
            let reuse = head(&template)
                .and_then(|h| h.admission.as_ref())
                .is_some_and(|g| g.compatible_writer(&template));
            template
                .relation_writer
                .as_mut()
                .ok_or_else(|| Error("writer absent".into()))?
                .reuse_admission = reuse;
            template.refresh_identity()?;
            template.validate()?;
            let h = template
                .relation_writer
                .as_ref()
                .ok_or_else(|| Error("writer absent".into()))?;
            let report = serde_json::json!({"schema":"uor-r4.writer-refit/1", "parent":self.artifact_cid(), "artifact":template.artifact_cid(), "documents":documents.len(), "writer_frames":frames.len(), "writer_correct":write_correct, "writer_rows":h.rows.len(), "dictionary":h.dictionary.len(), "epochs":epochs, "reuse_admission":reuse, "reader_parameters_unchanged":self.dependent_read==template.dependent_read && self.source_routing==template.source_routing && self.response_entry==template.response_entry});
            return Ok((template, report));
        }
        let mut reads = Vec::new();
        for (values, d) in &examples {
            let words = values
                .lexemes
                .as_ref()
                .ok_or_else(|| Error("read words absent".into()))?;
            let state = values
                .relations
                .as_ref()
                .ok_or_else(|| Error("read records absent".into()))?;
            let mut work = ValueWork::default();
            let addr = addresses(self, &words.queries[..words.query_len], &mut work);
            let wanted = d
                .response
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches('.');
            let offset = d
                .response
                .bytes()
                .position(|b| b.is_ascii_alphabetic())
                .unwrap_or(0);
            let prefix = self.encode(&d.response[..offset])?.first().copied();
            let mut alternatives = vec![Alternative {
                keys: Vec::new(),
                correct: false,
            }];
            let role =
                super::role_read::head(self).ok_or_else(|| Error("role parent absent".into()))?;
            for id in state.directory {
                let Some(record) = state.record(id) else {
                    continue;
                };
                let (f, n) = read_features(self, record, words, &addr, &mut work);
                let owner_mentioned = words.queries[..words.query_len.min(8)]
                    .iter()
                    .any(|w| record.owner.matches(w, &mut work));
                for (ai, a) in role.actions.iter().enumerate() {
                    let correct = owner_mentioned
                        && if record.conflict {
                            !a.copy && wanted == "Unknown"
                        } else {
                            a.copy
                                && a.prefix == prefix
                                && record.value.bytes[..usize::from(record.value.len)]
                                    == *wanted.as_bytes()
                        };
                    alternatives.push(Alternative {
                        keys: f[..n].iter().map(|f| key(*f, (ai + 1) as u8)).collect(),
                        correct,
                    });
                }
            }
            if !alternatives.iter().any(|a| a.correct) {
                alternatives[0].correct = true;
            }
            reads.push(Frame { alternatives });
        }
        let (reader, read_correct) = fit(&reads, epochs)?;
        let mut model = self.clone();
        model
            .response_entry
            .as_mut()
            .and_then(|e| e.copy.as_mut())
            .and_then(|c| c.role_read.as_mut())
            .ok_or_else(|| Error("role parent absent".into()))?
            .relations = Some(RelationModel {
            schema: if role_paths {
                "uor-r4.exact-relation/2"
            } else {
                "uor-r4.exact-relation/1"
            }
            .into(),
            role_context,
            admission: None,
            parent: self.artifact_cid.clone(),
            writer,
            reader,
            training: receipts,
            epochs,
        });
        model.refresh_identity()?;
        model.validate()?;
        let h = head(&model).ok_or_else(|| Error("relation fit absent".into()))?;
        let report = serde_json::json!({"schema":"uor-r4.relation-fit/1","parent":self.artifact_cid(),"artifact":model.artifact_cid(),"documents":documents.len(),"writer_frames":frames.len(),"writer_correct":write_correct,"reader_frames":reads.len(),"reader_correct":read_correct,"writer_rows":h.writer.len(),"reader_rows":h.reader.len(),"epochs":epochs,"scope":"Construction-supervised margin updates export sparse integer tables. Writer labels are exact source byte endpoints and typed actions offline only. Read fitting uses labeled construction writes; generated evaluation must test the assembled learned writer/reader. One association family; no general semantic memory claim."});
        let mut report = report;
        report["operator_schema"] = serde_json::json!(h.schema);
        report["role_context_rows"] = serde_json::json!(h.role_context.len());
        Ok((model, report))
    }
}

/// A replacement writer, with the complete previous model retained for lineage.
/// Serving executes only this writer's rows, never both writer scorers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WriterRevision {
    pub schema: String,
    pub parent: String,
    pub dictionary: Vec<super::word_copy_types::WordCopyAddress>,
    pub role_context: Vec<TokenGeometry>,
    pub rows: Vec<ValueRow>,
    pub training: Vec<DocumentReceipt>,
    pub epochs: usize,
    pub reuse_admission: bool,
    /// Exact NoWrite metadata compiled in this writer's cue namespace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admission: Option<super::relation_admission::Admission>,
}

fn writer_dictionary(
    _model: &Model,
    documents: &[RelationExample],
) -> Result<Vec<super::word_copy_types::WordCopyAddress>> {
    let examples: Vec<_> = documents
        .iter()
        .map(|d| ValueExample {
            id: d.id.clone(),
            prompt: d.prompt.clone(),
            response: d.response.clone(),
        })
        .collect();
    let (words, omitted, _) = super::word_copy_training::dictionary(&examples)?;
    if omitted != 0 {
        return Err(Error(
            "writer cue dictionary would omit construction words".into(),
        ));
    }
    // Supervised payload identities must not become new contextual cues.
    // They remain exact bytes in the relation records, including unseen names.
    let mut payloads = BTreeSet::new();
    for d in documents {
        for label in &d.writes {
            for end in [label.owner_end_byte, label.value_end_byte] {
                let end = usize::try_from(end).map_err(|e| Error(e.to_string()))?;
                let prefix = d
                    .prompt
                    .as_bytes()
                    .get(..=end)
                    .ok_or_else(|| Error("writer label outside prompt".into()))?;
                let start = prefix
                    .iter()
                    .rposition(|b| !b.is_ascii_alphanumeric() && *b != b'_')
                    .map_or(0, |i| i + 1);
                payloads.insert(prefix[start..].to_vec());
            }
        }
    }
    // This vocabulary belongs to the writer. Inherited reader names must not
    // acquire a different write role merely because they were seen elsewhere.
    let mut dictionary = Vec::new();
    let primes = crate::corpus_induced_spin_placement::first_primes(256)
        .map_err(|e| Error(e.to_string()))?;
    for mut word in words {
        if payloads.contains(&word.bytes[..usize::from(word.len)]) {
            continue;
        }
        if dictionary.len() >= 256 {
            return Err(Error("writer cue dictionary bound exceeded".into()));
        }
        word.prime = primes[dictionary.len()] as u32;
        dictionary.push(word);
    }
    dictionary.sort_by(|a, b| a.bytes[..usize::from(a.len)].cmp(&b.bytes[..usize::from(b.len)]));
    Ok(dictionary)
}

impl WriterRevision {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        let valid_word = |d: &super::word_copy_types::WordCopyAddress| {
            d.len > 0
                && d.len <= 32
                && d.bytes[..usize::from(d.len)]
                    .iter()
                    .all(|b| b.is_ascii_alphanumeric() || *b == b'_')
                && d.bytes[usize::from(d.len)..].iter().all(|b| *b == 0)
        };
        if self.schema != "uor-r4.relation-writer/1"
            || !(1..=64).contains(&self.epochs)
            || self.dictionary.is_empty()
            || self.dictionary.len() > 256
            || self.dictionary.iter().any(|d| !valid_word(d))
            || !self
                .dictionary
                .windows(2)
                .all(|p| p[0].bytes[..usize::from(p[0].len)] < p[1].bytes[..usize::from(p[1].len)])
            || self.rows.is_empty()
            || self.rows.len() > RELATION_ROWS
            || !self.rows.windows(2).all(|p| p[0].feature < p[1].feature)
            || self.rows.iter().any(|r| {
                !(-1000000..=1000000).contains(&r.weight)
                    || (r.feature.kind & 31) >= 20
                    || !(1..=3).contains(&(r.feature.kind >> 5))
            })
            || self.training.is_empty()
            || self.training.len() > 256
            || self.training.iter().any(|d| d.id.is_empty())
            || self
                .training
                .iter()
                .map(|d| &d.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.training.len()
        {
            return Err(Error("invalid writer revision bounds or rows".into()));
        }
        let mut assigned: Vec<_> = self.dictionary.iter().map(|d| u64::from(d.prime)).collect();
        assigned.sort_unstable();
        if assigned
            != crate::corpus_induced_spin_placement::first_primes(self.dictionary.len())
                .map_err(|e| Error(e.to_string()))?
            || self.role_context != compile_context(model, &self.dictionary)?
            || (self.reuse_admission
                && head(model)
                    .and_then(|h| h.admission.as_ref())
                    .is_none_or(|g| !g.compatible_writer(model)))
        {
            return Err(Error(
                "writer cue geometry or NoWrite compatibility differs".into(),
            ));
        }
        if let Some(gate) = &self.admission {
            gate.validate(model)?;
        }
        Ok(())
    }
}
