//! Shared geometric Stop/Copy/Add selection at a learned completion boundary.
//! Operands are exact committed records. Prediction never refreshes captured input.
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{learn, Alternative, Frame};
use super::value_types::{ValueDecision, ValueFeature, ValueRecord, ValueState, ValueWork};
use super::*;
use std::collections::BTreeSet;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperationTransition {
    pub router: SourceRouting,
    pub dictionary: Vec<word_copy_types::WordCopyAddress>,
    pub max_operations: u8,
}
impl OperationTransition {
    pub(super) fn validate_shape(&self, model: &Model) -> Result<()> {
        self.router.validate_shape(model, 3, 7)?;
        if self.max_operations != 3 || model.completion.is_none() || model.values.is_none() {
            return Err(Error("invalid operation transition contract".into()));
        }
        let primes = crate::corpus_induced_spin_placement::first_primes(self.dictionary.len())
            .map_err(|e| Error(e.to_string()))?;
        if self.dictionary.is_empty()
            || self.dictionary.len() > 256
            || self.dictionary.iter().zip(primes).any(|(w, p)| {
                w.len == 0
                    || w.len > 32
                    || u64::from(w.prime) != p
                    || w.bytes[..usize::from(w.len)]
                        .iter()
                        .any(|b| !b.is_ascii_lowercase() && *b != b'_')
                    || w.bytes[usize::from(w.len)..].iter().any(|b| *b != 0)
            })
            || self.dictionary.windows(2).any(|w| w[0].bytes >= w[1].bytes)
        {
            return Err(Error("invalid operation transition dictionary".into()));
        }
        Ok(())
    }
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        self.validate_shape(model)?;
        let mut parent = model.clone();
        parent.operation_transition = None;
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.router.parent_artifact {
            return Err(Error("operation transition frozen parent differs".into()));
        }
        parent.validate()?;
        let mut identity = model.clone();
        identity.refresh_identity()?;
        if identity.artifact_cid != model.artifact_cid
            || identity.uor_model_address != model.uor_model_address
        {
            return Err(Error("operation transition identity differs".into()));
        }
        Ok(())
    }
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
pub(super) fn proposal(
    values: &ValueState,
    index: usize,
) -> Option<(ValueAction, [ValueRecord; 2])> {
    let n = values.records.len();
    if index < 16 {
        let a = *values.records.get(index)?;
        return Some((ValueAction::Copy, [a, a]));
    }
    let i = (index - 16) >> 4;
    let j = (index - 16) & 15;
    if i >= n || j >= n || i >= j {
        return None;
    }
    let origin = |mut id| {
        for _ in 0..16 {
            let Some(d) = values
                .records
                .iter()
                .find(|r| r.id == id)
                .and_then(|r| r.derivation)
                .filter(|d| d.action == ValueAction::Copy)
            else {
                break;
            };
            if d.operand_ids[0] >= id {
                break;
            }
            id = d.operand_ids[0];
        }
        id
    };
    if origin(values.records[i].id) == origin(values.records[j].id) {
        return None;
    }
    Some((ValueAction::Add, [values.records[i], values.records[j]]))
}
fn latest(values: &ValueState) -> Option<u64> {
    values
        .records
        .iter()
        .rev()
        .find(|r| r.derived && r.start >= values.started_at)
        .map(|r| r.id)
}
pub(super) fn features(
    block: &OperationTransition,
    values: &ValueState,
    operands: Option<[ValueRecord; 2]>,
    work: &mut RoutingWork,
) -> ([ValueFeature; 39], usize) {
    let mut f = [ValueFeature::default(); 39];
    let last = latest(values);
    let mut flags = 0;
    let mut ranks = [16_u64; 2];
    let mut current = 0;
    if let Some(pair) = operands {
        for (k, r) in pair.iter().enumerate() {
            flags |= u64::from(r.derived) << k;
            current |= u64::from(Some(r.id) == last) << k;
            for (rank, known) in values.records.iter().rev().enumerate() {
                work.sources_examined += 1;
                if known.id == r.id {
                    ranks[k] = rank as u64;
                }
            }
        }
    }
    f[0] = ValueFeature {
        kind: 0,
        a: u64::from(values.operations_committed),
        b: u64::from(last.is_some()),
    };
    f[1] = ValueFeature {
        kind: 1,
        a: flags,
        b: current,
    };
    f[2] = ValueFeature {
        kind: 2,
        a: ranks[0],
        b: ranks[1],
    };
    f[3] = ValueFeature {
        kind: 3,
        a: u64::from(values.operations_committed),
        b: current,
    };
    let mut n = 4;
    if let Some(words) = &values.lexemes {
        for (position, word) in words.queries[..words.query_len].iter().enumerate() {
            work.context_tokens_read += 1;
            if values.query_boundary.is_some_and(|start| word.end < start) {
                continue;
            }
            let found = block.dictionary.iter().find(|d| {
                work.comparisons += 1;
                d.len == word.len
                    && d.bytes[..usize::from(d.len)]
                        .iter()
                        .zip(&word.bytes)
                        .all(|(a, b)| *a == b.to_ascii_lowercase())
            });
            if let Some(d) = found {
                f[n] = ValueFeature {
                    kind: 4,
                    a: u64::from(d.prime),
                    b: 0,
                };
                n += 1;
                f[n] = ValueFeature {
                    kind: 5,
                    a: position as u64,
                    b: u64::from(d.prime),
                };
                n += 1;
            }
        }
    }
    f[n] = ValueFeature {
        kind: 6,
        a: u64::from(values.operations_committed),
        b: flags,
    };
    n += 1;
    (f, n)
}
pub(super) fn prepare(
    values: &mut ValueState,
    action: ValueAction,
    operands: [ValueRecord; 2],
    score: i64,
    work: &mut ValueWork,
) -> Option<Candidate> {
    if values.next_id == u64::MAX {
        return None;
    }
    let value = super::value_runtime::execute(action, operands[0].value, operands[1].value, work)?;
    let numeral =
        super::numeral::Numeral::from_zphi(crate::prime_route_attention::ZPhi::new(value, 0))?;
    work.numeral_steps += 190;
    let token = numeral.tokens[0];
    values.pending = Some(ValueDecision {
        action,
        operands,
        value,
        write_id: values.next_id,
        token,
        cursor: 0,
        score,
        at_seen: values.seen,
    });
    Some(Candidate { token, score })
}
pub(super) fn offer(
    model: &Model,
    values: &mut ValueState,
    completion: &completion_types::CompletionState,
    baseline: Candidate,
    control: Control,
    work: &mut ValueWork,
) -> Option<Candidate> {
    let block = if matches!(
        control,
        Control::MixedOperatorsDisabled
            | Control::MixedTransitionDisabled
            | Control::ComposedOutputDisabled
    ) {
        model
            .mixed_operators
            .as_ref()
            .map(|w| &w.previous_operation)
            .or(model.operation_transition.as_ref())?
    } else {
        model.operation_transition.as_ref()?
    };
    if values.next_id == u64::MAX
        || baseline.token != EOS
        || !values.can_transition()
        || values.operations_committed >= block.max_operations
        || matches!(
            control,
            Control::OperationTransitionDisabled
                | Control::ValuesDisabled
                | Control::MemoryDisabled
                | Control::LearnedRoutingDisabled
                | Control::GeometryDisabled
                | Control::H4Disabled
        )
        || !completion.pending.is_some_and(|d| {
            d.token == EOS && d.at_seen == values.seen && Some(d.write_id) == latest(values)
        })
    {
        return None;
    }
    let router = if control == Control::ComposedOutputDisabled {
        model
            .composed_output
            .as_ref()
            .map_or(&block.router, |w| &w.previous_operation)
    } else {
        &block.router
    };
    let (f, n) = features(block, values, None, &mut work.routing);
    let state = router.encode(model, &f[..n], control, &mut work.routing);
    let mut best = router.score(model, state, 0, &mut work.routing);
    let mut selected = None;
    for index in 0..272 {
        if let Some((action, pair)) = proposal(values, index) {
            if control == Control::OperationTransitionIntermediateDisabled
                && pair.iter().any(|r| Some(r.id) == latest(values))
            {
                continue;
            }
            if super::value_runtime::execute(action, pair[0].value, pair[1].value, work).is_none() {
                continue;
            }
            let (f, n) = features(block, values, Some(pair), &mut work.routing);
            let state = router.encode(model, &f[..n], control, &mut work.routing);
            let score = router.score(
                model,
                state,
                if action == ValueAction::Copy { 1 } else { 2 },
                &mut work.routing,
            );
            if score > best {
                best = score;
                selected = Some((action, pair));
            }
        }
    }
    let (action, pair) = selected?;
    prepare(values, action, pair, baseline.score.saturating_add(1), work)
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationTransitionExample {
    pub id: String,
    pub history: Vec<TypedRoutingTurn>,
    pub query: String,
    /// Complete first response generated by the parent, used only as a check.
    pub first_response: String,
    /// Supervised next operator and literal operand. None labels Stop.
    pub next: Option<(ValueAction, i64)>,
}
pub(super) fn prefix(model: &Model, d: &OperationTransitionExample) -> Result<Session> {
    let mut s = model.session(Control::Full)?;
    s.observe(model, BOS)?;
    for turn in &d.history {
        for t in model.encode(&turn.prompt)? {
            s.observe(model, t)?;
        }
        s.begin_response(model)?;
        let bytes = until_stop(model, &mut s)?;
        if bytes != turn.response {
            return Err(Error(format!("history failed {}: {bytes:?}", d.id)));
        }
        s.observe(model, EOS)?;
        s.end_response(model)?;
    }
    for t in model.encode(&d.query)? {
        s.observe(model, t)?;
    }
    s.begin_response(model)?;
    Ok(s)
}
pub(super) fn until_stop(model: &Model, s: &mut Session) -> Result<String> {
    let mut tokens = Vec::new();
    for _ in 0..64 {
        let p = s.predict(model)?;
        if p.token == EOS {
            return String::from_utf8(model.decode(&tokens)?).map_err(|e| Error(e.to_string()));
        }
        tokens.push(p.token);
        s.observe(model, p.token)?;
    }
    Err(Error("operation response exceeded64 tokens".into()))
}
pub(super) fn frame(
    block: &OperationTransition,
    s: &Session,
    target: Option<(ValueAction, i64)>,
) -> Result<(Frame, Option<(ValueAction, [ValueRecord; 2])>)> {
    let v = s
        .values
        .as_ref()
        .ok_or_else(|| Error("typed state absent".into()))?;
    if let Some((ValueAction::Add, extra)) = target {
        if v.records
            .iter()
            .filter(|r| {
                !r.derived
                    && r.value == extra
                    && v.query_boundary.is_none_or(|start| r.start >= start)
            })
            .count()
            != 1
        {
            return Err(Error("ambiguous transition literal occurrence".into()));
        }
    }
    let last = latest(v);
    let (f, n) = features(block, v, None, &mut Default::default());
    let mut alternatives = vec![Alternative {
        features: f[..n].to_vec(),
        codes: vec![],
        action: 0,
        correct: target.is_none(),
    }];
    let mut chosen = None;
    for index in 0..272 {
        if let Some((action, pair)) = proposal(v, index) {
            if !super::joint_admission::legal(action, pair[0].value, pair[1].value) {
                continue;
            }
            let correct = target.is_some_and(|(a, extra)| {
                a == action
                    && pair.iter().any(|r| Some(r.id) == last)
                    && if a == ValueAction::Copy {
                        Some(pair[0].id) == last
                    } else {
                        pair.iter().any(|r| {
                            !r.derived
                                && r.value == extra
                                && v.query_boundary.is_none_or(|start| r.start >= start)
                        })
                    }
            });
            if correct {
                chosen = Some((action, pair));
            }
            let (f, n) = features(block, v, Some(pair), &mut Default::default());
            alternatives.push(Alternative {
                features: f[..n].to_vec(),
                codes: vec![],
                action: if action == ValueAction::Copy { 1 } else { 2 },
                correct,
            });
        }
    }
    if !alternatives.iter().any(|a| a.correct) {
        return Err(Error("transition target unreachable".into()));
    }
    Ok((Frame { alternatives }, chosen))
}
impl Model {
    pub fn fit_operation_transition(
        &self,
        docs: &[OperationTransitionExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        config.validate()?;
        if self.operation_transition.is_some()
            || docs.is_empty()
            || docs.len() > 256
            || docs.iter().any(|d| {
                d.id.is_empty()
                    || d.query.len() > 4096
                    || d.history.len() > 4
                    || d.history
                        .iter()
                        .any(|t| t.prompt.len() > 4096 || t.response.len() > 128)
                    || d.first_response.len() > 128
                    || d.next
                        .is_some_and(|(a, _)| !matches!(a, ValueAction::Copy | ValueAction::Add))
            })
        {
            return Err(Error("invalid transition construction".into()));
        }
        let mut words = BTreeSet::new();
        for d in docs {
            for w in d
                .query
                .as_bytes()
                .split(|b| !b.is_ascii_alphabetic() && *b != b'_')
            {
                if !w.is_empty() && w.len() <= 32 {
                    words.insert(w.to_ascii_lowercase());
                }
            }
        }
        if words.len() > 256 {
            return Err(Error("transition dictionary exceeds256".into()));
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
        let mut block = OperationTransition {
            max_operations: 3,
            dictionary,
            router: SourceRouting {
                schema: "uor-r4.geometric-source-routing/1".into(),
                parent_artifact: self.artifact_cid.clone(),
                codes: vec![],
                landmarks: (0..3)
                    .map(|_| {
                        std::array::from_fn(|_| {
                            (learned_routing_training::next_random(&mut rng) % 120) as u16
                        })
                    })
                    .collect(),
                biases: vec![0; 3],
                ranks: learned_routing_training::ranks(self),
                training: vec![],
                config,
            },
        };
        let mut frames = Vec::new();
        let mut rollout = Vec::new();
        for d in docs {
            let mut s = prefix(self, d)?;
            let first = until_stop(self, &mut s)?;
            if first != d.first_response {
                return Err(Error(format!("first response failed {}: {first:?}", d.id)));
            }
            let (f, chosen) = frame(&block, &s, d.next)?;
            frames.push(f);
            let mut second = None;
            if let Some((action, pair)) = chosen {
                // Offline action supervision executes the exact operator against
                // actual records. No expected response text is observed.
                let v = s
                    .values
                    .as_mut()
                    .ok_or_else(|| Error("typed state absent".into()))?;
                let candidate = prepare(v, action, pair, 1, &mut s.work.values)
                    .ok_or_else(|| Error("supervised operator unavailable".into()))?;
                s.completion
                    .as_mut()
                    .ok_or_else(|| Error("completion absent".into()))?
                    .reset();
                s.observe(self, candidate.token)?;
                let mut bytes = self.decode(&[candidate.token])?;
                bytes.extend(until_stop(self, &mut s)?.as_bytes());
                second = Some(String::from_utf8(bytes).map_err(|e| Error(e.to_string()))?);
                frames.push(frame(&block, &s, None)?.0);
            }
            rollout.push(
                serde_json::json!({"id":d.id,"first":first,"supervised_operator_response":second}),
            );
            let b = serde_json::to_vec(d).map_err(|e| Error(e.to_string()))?;
            block.router.training.push(DocumentReceipt {
                id: d.id.clone(),
                bytes: b.len(),
                text_cid: format!("blake3:{}", blake3::hash(&b)),
            });
        }
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
                "transition features {} exceed cap",
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
        let mut model = self.clone();
        model.operation_transition = Some(block);
        model.refresh_identity()?;
        model.validate()?;
        Ok((
            model,
            serde_json::json!({"fit":report,"rollout":rollout,"offline_action_supervision":true}),
        ))
    }
    pub fn evaluate_operation_transition(
        &self,
        docs: &[OperationTransitionExample],
        control: Control,
    ) -> Result<serde_json::Value> {
        let mut rows = Vec::new();
        for d in docs {
            let mut s = prefix(self, d)?;
            s.control = control;
            let mut decisions = Vec::new();
            let mut out = Vec::new();
            let mut eos = false;
            let mut checkpoints = 0;
            let mut times = Vec::new();
            for _ in 0..64 {
                let checkpoint = if control == Control::Full {
                    Some(s.checkpoint()?)
                } else {
                    None
                };
                let started = Instant::now();
                let p = s.predict(self)?;
                let predict_ns = started.elapsed().as_nanos();
                let decision = s.value_decision();
                if let Some(bytes) = checkpoint {
                    let mut restored = self.restore_session(&bytes)?;
                    if restored.predict(self)? != p
                        || restored.value_decision() != decision
                        || s.predict(self)? != p
                        || s.value_decision() != decision
                    {
                        return Err(Error(format!(
                            "transition checkpoint or repeat prediction differs: {}",
                            d.id
                        )));
                    }
                    checkpoints += 1;
                }
                if let Some(v) = decision.filter(|v| v.cursor == 0) {
                    decisions.push(v);
                }
                let started = Instant::now();
                s.observe(self, p.token)?;
                times.push(predict_ns + started.elapsed().as_nanos());
                if p.token == EOS {
                    eos = true;
                    break;
                }
                out.push(p.token);
            }
            let bytes = String::from_utf8(self.decode(&out)?).map_err(|e| Error(e.to_string()))?;
            let mut expected = d.first_response.clone();
            if let Some((action, extra)) = d.next {
                let first: i64 = d
                    .first_response
                    .trim()
                    .trim_end_matches('.')
                    .parse()
                    .map_err(|_| Error("numeric expectation absent".into()))?;
                let next = if action == ValueAction::Copy {
                    first
                } else {
                    first
                        .checked_add(extra)
                        .ok_or_else(|| Error("expectation overflow".into()))?
                };
                expected.push_str(&format!("{next}.\n"));
            }
            let dependency = if let Some((action, extra)) = d.next {
                decisions.len() == 2
                    && decisions[1].action == action
                    && decisions[1]
                        .operands
                        .iter()
                        .any(|r| r.id == decisions[0].write_id)
                    && (action == ValueAction::Copy
                        || decisions[1].operands.iter().any(|r| {
                            !r.derived
                                && r.value == extra
                                && s.values.as_ref().is_some_and(|v| {
                                    v.query_boundary.is_none_or(|start| r.start >= start)
                                })
                        }))
            } else {
                decisions.len() == 1
            };
            rows.push(serde_json::json!({"id":d.id,"response":bytes,"expected":expected,"exact":bytes==expected&&eos&&dependency,"dependency":dependency,"eos":eos,"decisions":decisions,"checkpoint_positions":checkpoints,"predict_observe_ns":times,"verification_work_includes_repeat_predictions":s.work}));
        }
        Ok(
            serde_json::json!({"artifact":self.artifact_cid(),"exact":rows.iter().filter(|r|r["exact"]==true).count(),"total":rows.len(),"rows":rows}),
        )
    }
}
