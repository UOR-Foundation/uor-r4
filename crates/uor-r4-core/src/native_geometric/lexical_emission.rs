//! Shared learned byte/read emission over actual completed typed records.
use super::completion_types::{
    CompletionAction, CompletionDecision, CompletionState, CompletionWork, LexicalRead,
};
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{learn, Alternative, Frame};
use super::value_types::{ValueFeature, ValueRecord, ValueState};
use super::*;
use std::collections::BTreeSet;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LexicalEmission {
    pub router: SourceRouting,
    pub dictionary: Vec<word_copy_types::WordCopyAddress>,
    pub tokens: Vec<u32>,
}
impl LexicalEmission {
    pub(super) fn validate_shape(&self, model: &Model) -> Result<()> {
        self.router.validate_shape(model, 2, 4)?;
        if model.completion.is_none()
            || self.tokens.is_empty()
            || self.tokens.len() > 64
            || self.tokens.windows(2).any(|w| w[0] >= w[1])
            || self
                .tokens
                .iter()
                .any(|t| *t != EOS && !(2..258).contains(t))
            || self.dictionary.is_empty()
            || self.dictionary.len() > 128
        {
            return Err(Error("invalid lexical emission shape".into()));
        }
        let primes = crate::corpus_induced_spin_placement::first_primes(self.dictionary.len())
            .map_err(|e| Error(e.to_string()))?;
        if self.dictionary.iter().zip(primes).any(|(w, p)| {
            w.len == 0
                || w.len > 32
                || u64::from(w.prime) != p
                || w.bytes[..usize::from(w.len)]
                    .iter()
                    .any(|b| !b.is_ascii_lowercase() && *b != b'_')
                || w.bytes[usize::from(w.len)..].iter().any(|b| *b != 0)
        }) || self.dictionary.windows(2).any(|w| w[0].bytes >= w[1].bytes)
        {
            return Err(Error("invalid lexical dictionary".into()));
        }
        Ok(())
    }
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        self.validate_shape(model)?;
        let mut parent = model.clone();
        parent.lexical_emission = None;
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.router.parent_artifact {
            return Err(Error("lexical emission frozen parent differs".into()));
        }
        parent.validate()?;
        let mut bound = model.clone();
        bound.refresh_identity()?;
        if bound.artifact_cid != model.artifact_cid
            || bound.uor_model_address != model.uor_model_address
        {
            return Err(Error("lexical emission identity differs".into()));
        }
        Ok(())
    }
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
fn role(values: &ValueState, s: &CompletionState, id: u64) -> u64 {
    let Some(anchor) = s.anchor else { return 0 };
    if id == anchor.write_id {
        return 1;
    }
    let Some(d) = values
        .records
        .iter()
        .find(|r| r.id == anchor.write_id)
        .and_then(|r| r.derivation)
    else {
        return 0;
    };
    if id == d.operand_ids[0] {
        2
    } else if id == d.operand_ids[1] {
        3
    } else {
        0
    }
}
pub(super) fn features(
    block: &LexicalEmission,
    s: &CompletionState,
    values: &ValueState,
    token: u32,
    record: Option<ValueRecord>,
    action_context: bool,
) -> ([ValueFeature; 40], usize) {
    // Typed prefix and exact-read digit widths never become lexical positions.
    // Config permits 65536 lexical pieces plus the 258 reserved/byte tokens.
    // These markers and the 18-bit fields are disjoint from every valid token.
    const PREFIX: u64 = 131072;
    let mut last = u64::from(s.last);
    let mut previous = u64::from(s.previous);
    if s.steps == 0 {
        last = PREFIX;
        previous = PREFIX;
    } else if s.steps == 1 {
        previous = PREFIX;
    }
    if let Some(r) = s.lexical_read.filter(|r| r.cursor == r.numeral.len) {
        let end = r.start_at.saturating_add(u64::from(r.cursor));
        let marker = PREFIX + 1 + role(values, s, r.record_id);
        if s.seen == end {
            last = marker;
            previous = PREFIX;
        } else if s.seen == end.saturating_add(1) {
            previous = marker;
        }
    }
    let choice = record.map_or(u64::from(token), |r| 1024 + role(values, s, r.id));
    let mut f = [ValueFeature::default(); 40];
    f[0] = ValueFeature {
        kind: 0,
        a: (s.lexical_read.map_or(0, |r| role(values, s, r.record_id)) << 36)
            | (previous << 18)
            | last
            | (if action_context {
                match s.anchor.map(|a| a.action) {
                    Some(ValueAction::Add) => 0,
                    Some(ValueAction::Copy) => 1,
                    Some(ValueAction::Sub) => 2,
                    Some(ValueAction::Mul) => 3,
                    None => 0,
                }
            } else {
                0
            } << 56),
        b: choice,
    };
    let mut n = 1;
    if let Some(words) = &values.lexemes {
        for word in &words.queries[..words.query_len] {
            if values.query_boundary.is_some_and(|start| word.end < start) {
                continue;
            }
            if let Some(d) = block.dictionary.iter().find(|d| {
                d.len == word.len
                    && d.bytes[..usize::from(d.len)]
                        .iter()
                        .zip(&word.bytes)
                        .all(|(a, b)| *a == b.to_ascii_lowercase())
            }) {
                f[n] = ValueFeature {
                    kind: 3,
                    a: u64::from(d.prime),
                    b: choice,
                };
                n += 1;
            }
        }
    }
    (f, n)
}
pub(super) fn prepare(
    s: &mut CompletionState,
    token: u32,
    read: Option<LexicalRead>,
    score: i64,
) -> Option<Candidate> {
    let anchor = s.anchor?;
    s.pending = Some(CompletionDecision {
        token,
        score,
        write_id: anchor.write_id,
        step: s.steps,
        at_seen: s.seen,
        action: if token == EOS {
            CompletionAction::Stop
        } else {
            CompletionAction::Emit
        },
    });
    s.pending_lexical_read = read;
    Some(Candidate { token, score })
}
pub(super) fn offer(
    model: &Model,
    s: &mut CompletionState,
    values: &ValueState,
    baseline: Candidate,
    control: Control,
    work: &mut CompletionWork,
) -> Option<Candidate> {
    let block = if control == Control::ActionEmissionDisabled {
        model
            .action_emission
            .as_ref()
            .map(|w| &w.previous_lexical)
            .or(model.lexical_emission.as_ref())?
    } else {
        model.lexical_emission.as_ref()?
    };
    let action_context = model
        .action_emission
        .as_ref()
        .is_some_and(|w| w.context_enabled)
        && !matches!(
            control,
            Control::ActionEmissionDisabled | Control::LexicalActionContextDisabled
        );
    s.pending_lexical_read = None;
    if matches!(
        control,
        Control::LexicalEmissionDisabled
            | Control::GeometryDisabled
            | Control::H4Disabled
            | Control::LearnedRoutingDisabled
    ) {
        return None;
    }
    if let Some(read) = s
        .lexical_read
        .filter(|r| r.cursor < r.numeral.len && control != Control::LexicalRecordReadDisabled)
    {
        return prepare(
            s,
            read.numeral.tokens[usize::from(read.cursor)],
            Some(read),
            baseline.score.saturating_add(1),
        );
    }
    let geometric_control = if control == Control::LexicalEmissionGeometryDisabled {
        Control::LearnedRoutingTransformDisabled
    } else {
        control
    };
    let mut routing = RoutingWork::default();
    let (f, n) = features(block, s, values, BOS, None, action_context);
    let state = block
        .router
        .encode(model, &f[..n], geometric_control, &mut routing);
    work.candidate_evaluations = work.candidate_evaluations.saturating_add(1);
    let mut best = block.router.score(model, state, 0, &mut routing);
    let mut selected = None;
    for &token in &block.tokens {
        let (f, n) = features(block, s, values, token, None, action_context);
        let state = block
            .router
            .encode(model, &f[..n], geometric_control, &mut routing);
        work.candidate_evaluations = work.candidate_evaluations.saturating_add(1);
        let score = block.router.score(model, state, 1, &mut routing);
        if score > best {
            best = score;
            selected = Some((token, None));
        }
    }
    if control != Control::LexicalRecordReadDisabled {
        for record in &values.records {
            let Some(numeral) = super::numeral::Numeral::from_zphi(
                crate::prime_route_attention::ZPhi::new(record.value, 0),
            ) else {
                continue;
            };
            if s.steps.saturating_add(numeral.len) >= 32 {
                continue;
            }
            let (f, n) = features(block, s, values, BOS, Some(*record), action_context);
            let state = block
                .router
                .encode(model, &f[..n], geometric_control, &mut routing);
            work.candidate_evaluations = work.candidate_evaluations.saturating_add(1);
            let score = block.router.score(model, state, 1, &mut routing);
            if score > best {
                best = score;
                selected = Some((
                    numeral.tokens[0],
                    Some(LexicalRead {
                        record_id: record.id,
                        start_at: s.seen,
                        numeral,
                        cursor: 0,
                    }),
                ));
            }
        }
    }

    let (token, read) = selected?;
    prepare(s, token, read, baseline.score.saturating_add(1))
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmissionPiece {
    Text(String),
    Operand(u8),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmissionExample {
    pub id: String,
    pub history: Vec<TypedRoutingTurn>,
    pub query: String,
    pub prefix: String,
    pub suffix: Option<Vec<EmissionPiece>>,
}
fn setup(model: &Model, d: &EmissionExample) -> Result<Session> {
    let mut s = model.session(Control::Full)?;
    s.observe(model, BOS)?;
    for h in &d.history {
        for t in model.encode(&h.prompt)? {
            s.observe(model, t)?;
        }
        s.begin_response(model)?;
        let mut out = Vec::new();
        let mut eos = false;
        for _ in 0..96 {
            let p = s.predict(model)?;
            s.observe(model, p.token)?;
            if p.token == EOS {
                eos = true;
                break;
            }
            out.push(p.token);
        }
        if !eos || model.decode(&out)? != h.response.as_bytes() {
            return Err(Error(format!("emission history failed {}", d.id)));
        }
        s.end_response(model)?;
    }
    for t in model.encode(&d.query)? {
        s.observe(model, t)?;
    }
    s.begin_response(model)?;
    Ok(s)
}
fn prefix(model: &Model, s: &mut Session, d: &EmissionExample) -> Result<()> {
    let mut out = Vec::new();
    for _ in 0..32 {
        if s.completion.as_ref().is_some_and(|c| c.active) {
            break;
        }
        let p = s.predict(model)?;
        if p.token == EOS {
            break;
        }
        s.observe(model, p.token)?;
        out.push(p.token);
    }
    if model.decode(&out)? != d.prefix.as_bytes()
        || !s.completion.as_ref().is_some_and(|c| c.active)
    {
        return Err(Error(format!(
            "emission numeric prefix failed {}: {:?}",
            d.id,
            String::from_utf8_lossy(&model.decode(&out)?)
        )));
    }
    Ok(())
}
pub(super) fn operand(s: &Session, index: u8) -> Result<ValueRecord> {
    let v = s
        .values
        .as_ref()
        .ok_or_else(|| Error("values absent".into()))?;
    let c = s
        .completion
        .as_ref()
        .ok_or_else(|| Error("completion absent".into()))?;
    let anchor = c.anchor.ok_or_else(|| Error("anchor absent".into()))?;
    let d = v
        .records
        .iter()
        .find(|r| r.id == anchor.write_id)
        .and_then(|r| r.derivation)
        .ok_or_else(|| Error("derivation absent".into()))?;
    let id = *d
        .operand_ids
        .get(usize::from(index))
        .ok_or_else(|| Error("operand index invalid".into()))?;
    v.records
        .iter()
        .find(|r| r.id == id)
        .copied()
        .ok_or_else(|| Error("operand evicted".into()))
}
pub(super) fn frame(
    block: &LexicalEmission,
    s: &Session,
    target: Option<(u32, Option<u64>)>,
    action_context: bool,
) -> Result<Frame> {
    let c = s
        .completion
        .as_ref()
        .ok_or_else(|| Error("completion absent".into()))?;
    let v = s
        .values
        .as_ref()
        .ok_or_else(|| Error("values absent".into()))?;
    let mut alternatives = Vec::new();
    let (f, n) = features(block, c, v, BOS, None, action_context);
    alternatives.push(Alternative {
        features: f[..n].to_vec(),
        codes: vec![],
        action: 0,
        correct: target.is_none(),
    });
    for &t in &block.tokens {
        let (f, n) = features(block, c, v, t, None, action_context);
        alternatives.push(Alternative {
            features: f[..n].to_vec(),
            codes: vec![],
            action: 1,
            correct: target == Some((t, None)),
        });
    }
    for &r in &v.records {
        let Some(numeral) =
            super::numeral::Numeral::from_zphi(crate::prime_route_attention::ZPhi::new(r.value, 0))
        else {
            continue;
        };
        if c.steps.saturating_add(numeral.len) >= 32 {
            continue;
        }
        let (f, n) = features(block, c, v, BOS, Some(r), action_context);
        alternatives.push(Alternative {
            features: f[..n].to_vec(),
            codes: vec![],
            action: 1,
            correct: target.is_some_and(|(_, id)| id == Some(r.id)),
        });
    }
    if !alternatives.iter().any(|a| a.correct) {
        return Err(Error("emission target absent".into()));
    }
    Ok(Frame { alternatives })
}
impl Model {
    pub fn fit_lexical_emission(
        &self,
        docs: &[EmissionExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        config.validate()?;
        if self.lexical_emission.is_some() || docs.is_empty() || docs.len() > 128 {
            return Err(Error("invalid emission fit population".into()));
        }
        let mut words = BTreeSet::new();
        let mut constant_words: Option<BTreeSet<Vec<u8>>> = None;
        let mut tokens = BTreeSet::from([EOS]);
        for d in docs {
            if d.query.len() > 4096
                || d.history.len() > 4
                || d.history
                    .iter()
                    .any(|t| t.prompt.len() > 4096 || t.response.len() > 128)
                || d.prefix.len() > 24
                || d.suffix.as_ref().is_some_and(|s| s.len() > 16)
            {
                return Err(Error("emission data bounds".into()));
            }
            let mut document_words = BTreeSet::new();
            for w in d
                .query
                .as_bytes()
                .split(|b| !b.is_ascii_alphabetic() && *b != b'_')
            {
                if !w.is_empty() && w.len() <= 32 {
                    document_words.insert(w.to_ascii_lowercase());
                }
            }
            words.extend(document_words.iter().cloned());
            constant_words = Some(match constant_words {
                None => document_words,
                Some(previous) => previous.intersection(&document_words).cloned().collect(),
            });
            if let Some(parts) = &d.suffix {
                for p in parts {
                    if let EmissionPiece::Text(t) = p {
                        if !t.is_ascii() || t.len() > 32 {
                            return Err(Error("emission text bounds".into()));
                        }
                        tokens.extend(t.bytes().map(|b| u32::from(b) + 2));
                    }
                }
            }
        }
        // Constant construction context cannot identify the requested output.
        // Excluding it keeps unrelated query vocabulary from perturbing Base.
        if let Some(constant) = constant_words {
            words.retain(|word| !constant.contains(word));
        }
        let primes = crate::corpus_induced_spin_placement::first_primes(words.len())
            .map_err(|e| Error(e.to_string()))?;
        let dictionary = words
            .into_iter()
            .zip(primes)
            .map(|(w, p)| {
                let mut bytes = [0; 32];
                bytes[..w.len()].copy_from_slice(&w);
                word_copy_types::WordCopyAddress {
                    bytes,
                    len: w.len() as u8,
                    prime: p as u32,
                }
            })
            .collect();
        let mut rng = config.seed;
        let mut block = LexicalEmission {
            dictionary,
            tokens: tokens.into_iter().collect(),
            router: SourceRouting {
                schema: "uor-r4.geometric-source-routing/1".into(),
                parent_artifact: self.artifact_cid.clone(),
                codes: vec![],
                landmarks: (0..2)
                    .map(|_| {
                        std::array::from_fn(|_| {
                            (learned_routing_training::next_random(&mut rng) % 120) as u16
                        })
                    })
                    .collect(),
                biases: vec![0; 2],
                ranks: learned_routing_training::ranks(self),
                training: vec![],
                config,
            },
        };
        let mut frames = Vec::new();
        for d in docs {
            let mut s = setup(self, d)?;
            prefix(self, &mut s, d)?;
            if let Some(parts) = &d.suffix {
                let mut total = 1_usize; // Reserve EOS inside the existing 32-step contract.
                for piece in parts {
                    total += match piece {
                        EmissionPiece::Text(t) => t.len(),
                        EmissionPiece::Operand(i) => operand(&s, *i)?.value.to_string().len(),
                    };
                }
                if total > 32 {
                    return Err(Error("lexical construction exceeds completion cap".into()));
                }
                for piece in parts {
                    match piece {
                        EmissionPiece::Text(text) => {
                            for token in text.bytes().map(|b| u32::from(b) + 2) {
                                frames.push(frame(&block, &s, Some((token, None)), false)?);
                                prepare(
                                    s.completion
                                        .as_mut()
                                        .ok_or_else(|| Error("completion absent".into()))?,
                                    token,
                                    None,
                                    1,
                                );
                                s.observe(self, token)?;
                            }
                        }
                        EmissionPiece::Operand(index) => {
                            let r = operand(&s, *index)?;
                            let numeral = super::numeral::Numeral::from_zphi(
                                crate::prime_route_attention::ZPhi::new(r.value, 0),
                            )
                            .ok_or_else(|| Error("numeral invalid".into()))?;
                            frames.push(frame(
                                &block,
                                &s,
                                Some((numeral.tokens[0], Some(r.id))),
                                false,
                            )?);
                            let start_at = s
                                .completion
                                .as_ref()
                                .ok_or_else(|| Error("completion absent".into()))?
                                .seen;
                            for cursor in 0..numeral.len {
                                let token = numeral.tokens[usize::from(cursor)];
                                prepare(
                                    s.completion
                                        .as_mut()
                                        .ok_or_else(|| Error("completion absent".into()))?,
                                    token,
                                    Some(LexicalRead {
                                        record_id: r.id,
                                        start_at,
                                        numeral,
                                        cursor,
                                    }),
                                    1,
                                );
                                s.observe(self, token)?;
                            }
                        }
                    }
                }
                frames.push(frame(&block, &s, Some((EOS, None)), false)?);
            } else {
                for _ in 0..32 {
                    if s.completion.as_ref().is_some_and(|c| c.active) {
                        frames.push(frame(&block, &s, None, false)?);
                    }
                    let p = s.predict(self)?;
                    s.observe(self, p.token)?;
                    if p.token == EOS {
                        break;
                    }
                }
            }
            let bytes = serde_json::to_vec(d).map_err(|e| Error(e.to_string()))?;
            block.router.training.push(DocumentReceipt {
                id: d.id.clone(),
                bytes: bytes.len(),
                text_cid: format!("blake3:{}", blake3::hash(&bytes)),
            });
        }
        if frames.len() > 4096 {
            return Err(Error("emission frames exceed4096".into()));
        }
        let presented_frames = frames.len();
        let mut unique = BTreeSet::new();
        frames.retain(|f| {
            unique.insert(
                f.alternatives
                    .iter()
                    .map(|a| (a.features.clone(), a.action, a.correct))
                    .collect::<Vec<_>>(),
            )
        });
        let vocab: BTreeSet<_> = frames
            .iter()
            .flat_map(|f| {
                f.alternatives
                    .iter()
                    .flat_map(|a| a.features.iter().copied())
            })
            .collect();
        if vocab.len() > block.router.config.learned_features {
            return Err(Error(format!(
                "emission features {} exceed cap",
                vocab.len()
            )));
        }
        block.router.codes = vocab
            .into_iter()
            .map(|feature| SourceCode {
                feature,
                roots: std::array::from_fn(|_| {
                    (learned_routing_training::next_random(&mut rng) % 120) as u16
                }),
            })
            .collect();
        let report = learn(self, &mut block.router, &mut frames, Instant::now());
        let n = frames.len();
        let features = block.router.codes.len();
        let mut model = self.clone();
        model.lexical_emission = Some(block);
        model.refresh_identity()?;
        model.validate()?;
        Ok((
            model,
            serde_json::json!({"fit":report,"frames":n,"presented_frames":presented_frames,"features":features,"teacher_forced_lexical_labels":true,"explicit_offline_record_alignment":true}),
        ))
    }
    pub fn evaluate_lexical_emission(
        &self,
        docs: &[EmissionExample],
        control: Control,
    ) -> Result<serde_json::Value> {
        let mut rows = Vec::new();
        for d in docs {
            let mut s = setup(self, d)?;
            s.control = control;
            let mut out = Vec::new();
            let mut reads = Vec::new();
            let mut writes = Vec::new();
            let mut eos = false;
            let mut checkpoints = 0;
            for _ in 0..96 {
                let bytes = s.checkpoint()?;
                let p = s.predict(self)?;
                if control == Control::Full {
                    let mut restored = self.restore_session(&bytes)?;
                    if restored.predict(self)? != p
                        || restored.completion.as_ref().and_then(|c| c.pending)
                            != s.completion.as_ref().and_then(|c| c.pending)
                        || restored
                            .completion
                            .as_ref()
                            .and_then(|c| c.pending_lexical_read)
                            != s.completion.as_ref().and_then(|c| c.pending_lexical_read)
                        || restored.value_decision() != s.value_decision()
                        || s.predict(self)? != p
                    {
                        return Err(Error(format!("emission checkpoint differs {}", d.id)));
                    }
                    checkpoints += 1;
                }
                if let Some(r) = s
                    .completion
                    .as_ref()
                    .and_then(|c| c.pending_lexical_read)
                    .filter(|r| r.cursor == 0)
                {
                    reads.push(r.record_id);
                }
                if let Some(v) = s.value_decision().filter(|v| v.cursor == 0) {
                    writes.push(v);
                }
                s.observe(self, p.token)?;
                if p.token == EOS {
                    eos = true;
                    break;
                }
                out.push(p.token);
            }
            let text = String::from_utf8(self.decode(&out)?).map_err(|e| Error(e.to_string()))?;
            // Truth is constructed offline from supplied arithmetic labels and actual ordered derivation operands.
            if writes.is_empty() {
                rows.push(serde_json::json!({"id":d.id,"query":d.query,"text":text,"expected_prefix":d.prefix,"exact":false,"eos":eos,"failure":"no actual typed write","checkpoint_positions":checkpoints}));
                continue;
            }
            let mut expected = d.prefix.clone();
            let mut expected_reads = Vec::new();
            if let Some(parts) = &d.suffix {
                let v = s
                    .values
                    .as_ref()
                    .ok_or_else(|| Error("values absent".into()))?;
                let anchor = writes
                    .first()
                    .ok_or_else(|| Error("no generated write".into()))?;
                for p in parts {
                    match p {
                        EmissionPiece::Text(t) => expected.push_str(t),
                        EmissionPiece::Operand(i) => {
                            let r = anchor
                                .operands
                                .get(usize::from(*i))
                                .ok_or_else(|| Error("operand index invalid".into()))?;
                            expected.push_str(&r.value.to_string());
                            expected_reads.push(r.id);
                            if !v.records.iter().any(|v| v.id == r.id) {
                                return Err(Error("expected record evicted".into()));
                            }
                        }
                    }
                }
            } else {
                expected.push_str(".\n");
            }
            let exact = text == expected && eos && reads == expected_reads && writes.len() == 1;
            rows.push(serde_json::json!({"id":d.id,"query":d.query,"expected":expected,"text":text,"exact":exact,"eos":eos,"reads":reads,"expected_reads":expected_reads,"writes":writes,"checkpoint_positions":checkpoints}));
        }
        Ok(
            serde_json::json!({"artifact":self.artifact_cid(),"exact":rows.iter().filter(|r|r["exact"]==true).count(),"total":rows.len(),"rows":rows}),
        )
    }
}
