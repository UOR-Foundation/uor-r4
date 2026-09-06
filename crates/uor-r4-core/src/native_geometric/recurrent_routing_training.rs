//! Offline Rust learning of the assembled Base/Emit/EOS decision. The parent
//! runs once on each teacher-forced history; proposals run the exact hard
//! two-read route and shared integer scorer against those causal parent choices.
//! Free generation is evaluated separately; cached histories are not rollouts.
use super::learned_routing::{Emission, Head, RoutingBlock, HEADS, ROOTS, WINDOW};
use super::learned_routing_training::{field, next_random, ranks};
use super::recurrent_routing::{features, JointOutput, JointRow, SCHEMA};
use super::{
    Control, Document, DocumentReceipt, Error, Model, Result, RoutingFitConfig, RoutingWork,
    TokenScore, ValueExample, WordCopyProgress, BOS, EOS, SCORE_SCALE,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrentRoutingFitReport {
    pub schema: String,
    pub positions: usize,
    pub response_positions: usize,
    pub forced_copy_positions: usize,
    pub documents: usize,
    pub response_examples: usize,
    pub proposals: u64,
    pub accepted_moves: [u64; 4],
    pub initial_correct: usize,
    pub final_correct: usize,
    pub initial_assembled_nll: f64,
    pub final_assembled_nll: f64,
    /// Real candidate-model teacher forcing, after the cached-parent search.
    pub actual_teacher_forced_correct: usize,
    pub cached_actual_output_agreement: usize,
    pub elapsed_ms: u128,
    pub stopped_at_time_limit: bool,
    pub block_bytes: usize,
    pub config: RoutingFitConfig,
}

struct Example {
    sequence: usize,
    position: usize,
    context: [u32; WINDOW],
    length: usize,
    baseline: u32,
    target: u32,
    flags: u32,
}

#[derive(Clone, Copy)]
struct Objective {
    correct: usize,
    nll: f64,
}
impl Objective {
    fn improves(self, other: Self) -> bool {
        self.correct > other.correct
            || (self.correct == other.correct && self.nll + 1e-9 < other.nll)
    }
}

fn fit_output(
    model: &Model,
    block: &RoutingBlock,
    examples: &[Example],
    response_examples: usize,
) -> (JointOutput, Objective) {
    let vocab = model.vocabulary_size();
    let mut keys_used = BTreeSet::new();
    let mut actions = BTreeSet::from([BOS, EOS]);
    let mut frames = Vec::with_capacity(examples.len());
    for example in examples {
        let route = block.route(
            model,
            &example.context[..example.length],
            Control::Full,
            &mut RoutingWork::default(),
        );
        let keys = features(
            model,
            route,
            example.baseline,
            example.flags,
            &mut RoutingWork::default(),
        );
        let action = if example.target == example.baseline {
            BOS
        } else {
            example.target
        };
        actions.insert(action);
        keys_used.extend(keys);
        frames.push((keys, action));
    }
    // Fit discriminative additive scores to the joint action. Dense scratch
    // here is offline weight storage, never a matrix-product serving path.
    let key_indexes: BTreeMap<_, _> = keys_used
        .into_iter()
        .enumerate()
        .map(|(i, k)| (k, i))
        .collect();
    let indexed: Vec<_> = frames
        .iter()
        .map(|(keys, target)| (keys.map(|k| key_indexes[&k]), *target))
        .collect();
    let mut weights = vec![vec![0_i32; vocab]; key_indexes.len()];
    let mut priors = vec![0_i32; vocab];
    let active: Vec<_> = actions.into_iter().collect();
    for _ in 0..4 {
        for (example, (rows, target)) in examples.iter().zip(&indexed) {
            let mut best = (BOS, i32::MIN);
            for &action in &active {
                if action == example.baseline && action != BOS {
                    continue;
                }
                let mut score = priors[action as usize];
                for &row in rows {
                    score += weights[row][action as usize];
                }
                if score > best.1 {
                    best = (action, score);
                }
            }
            if best.0 != *target {
                for &row in rows {
                    weights[row][*target as usize] += 32;
                    weights[row][best.0 as usize] -= 32;
                }
                priors[*target as usize] += 32;
                priors[best.0 as usize] -= 32;
            }
        }
    }
    let priors = priors.into_iter().map(|s| s.clamp(-32768, 32767)).collect();
    let rows = key_indexes
        .into_iter()
        .map(|(key, index)| {
            let scores: Vec<_> = weights[index]
                .iter()
                .enumerate()
                .filter(|(_, s)| **s != 0)
                .map(|(token, &score)| TokenScore {
                    token: token as u32,
                    score: score.clamp(-32768, 32767),
                })
                .collect();
            let mut ranked = scores.clone();
            ranked.sort_by(|a, b| b.score.cmp(&a.score).then(a.token.cmp(&b.token)));
            JointRow {
                key,
                emission: Emission {
                    default_score: 0,
                    scores,
                    postings: ranked
                        .iter()
                        .filter(|p| p.score > 0)
                        .take(8)
                        .map(|p| p.token)
                        .collect(),
                },
            }
        })
        .collect();
    let output = JointOutput {
        additive_scores: true,
        rows,
        priors,
        positions: examples.len(),
        response_examples,
    };
    let mut correct = 0;
    let mut nll = 0.0;
    for (example, (keys, target)) in examples.iter().zip(frames) {
        let mut work = RoutingWork::default();
        let rows = output.rows_for(keys, &mut work);
        let (candidates, count) = output.candidates(rows, example.baseline, &mut work);
        let mut logits = [0.0_f64; super::recurrent_routing::CANDIDATES];
        let mut best = (BOS, i64::MIN);
        let mut target_score = None;
        for (slot, &action) in logits.iter_mut().zip(&candidates[..count]) {
            let score = output.score(rows, action, &mut work);
            *slot = score as f64 / SCORE_SCALE;
            if action == target {
                target_score = Some(*slot);
            }
            if score > best.1 || (score == best.1 && action < best.0) {
                best = (action, score);
            }
        }
        correct += usize::from(best.0 == target);
        let maximum = logits[..count]
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        nll += logits[..count]
            .iter()
            .map(|s| (s - maximum).exp())
            .sum::<f64>()
            .ln()
            + maximum
            - target_score.unwrap_or(maximum - 20.0);
    }
    (
        output,
        Objective {
            correct,
            nll: nll / examples.len() as f64,
        },
    )
}

impl Model {
    pub fn fit_recurrent_routing(
        &self,
        documents: &[Document],
        responses: &[ValueExample],
        config: RoutingFitConfig,
    ) -> Result<(Self, RecurrentRoutingFitReport)> {
        config.validate()?;
        if documents.is_empty()
            || documents.len() > 256
            || responses.is_empty()
            || responses.len() > 256
            || self.vocabulary_size() > 8192
            || self.learned_routing.is_some()
        {
            return Err(Error(
                "recurrent routing source or parent bound invalid".into(),
            ));
        }
        let started = Instant::now();
        let vocab = self.vocabulary_size();
        let mut examples = Vec::new();
        let mut prepared = Vec::new();
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut frequencies = vec![0_u32; vocab];
        let mut forced = 0;
        let mut response_positions = 0;
        // Reserve half the positions for actual response boundaries/termination.
        for (is_response, count, budget) in [
            (false, documents.len(), config.max_positions / 2),
            (
                true,
                responses.len(),
                config.max_positions - config.max_positions / 2,
            ),
        ] {
            for index in 0..count {
                let (id, prompt, text) = if is_response {
                    let e = &responses[index];
                    (&e.id, e.prompt.as_str(), e.response.as_str())
                } else {
                    let d = &documents[index];
                    (&d.id, "", d.text.as_str())
                };
                if id.is_empty()
                    || text.is_empty()
                    || prompt.len() + text.len() > 65536
                    || !ids.insert(id.clone())
                {
                    return Err(Error("recurrent routing input invalid".into()));
                }
                let binding = if is_response {
                    serde_json::to_vec(&(prompt, text)).map_err(|e| Error(e.to_string()))?
                } else {
                    text.as_bytes().to_vec()
                };
                receipts.push(DocumentReceipt {
                    id: id.clone(),
                    text_cid: format!("blake3:{}", blake3::hash(&binding).to_hex()),
                    bytes: prompt.len() + text.len(),
                });
                let mut session = self.session(Control::Full)?;
                session.observe(self, BOS)?;
                let prompt_tokens = if is_response {
                    self.encode(prompt)?
                } else {
                    Vec::new()
                };
                if is_response {
                    for &token in &prompt_tokens {
                        session.observe(self, token)?;
                    }
                    session.begin_response(self)?;
                }
                let mut tokens = self.encode(text)?;
                tokens.push(EOS);
                let quota =
                    (budget / count + usize::from(index < budget % count)).min(tokens.len());
                for (position, target) in tokens.iter().copied().enumerate() {
                    // Run the real parent on every teacher-forced position so
                    // pending causal commits are the same as normal serving.
                    let copying = super::word_copy_runtime::composed(self)
                        && session.word_copy.as_ref().is_some_and(|s| {
                            matches!(s.progress, WordCopyProgress::Emitting { .. })
                        });
                    let baseline = session.predict(self)?.token;
                    let (context, length) = session.recurrent_context();
                    if quota > 0
                        && (position + 1) * quota / tokens.len() > position * quota / tokens.len()
                    {
                        if copying {
                            forced += 1;
                        } else {
                            for &token in &context[..length] {
                                frequencies[token as usize] += 1;
                            }
                            examples.push(Example {
                                sequence: prepared.len(),
                                position,
                                context,
                                length,
                                baseline,
                                target,
                                flags: session.recurrent_flags(),
                            });
                            response_positions += usize::from(is_response);
                        }
                    }
                    session.observe(self, target)?;
                }
                prepared.push((prompt_tokens, tokens, is_response));
            }
        }
        if examples.is_empty() {
            return Err(Error("no recurrent routing positions".into()));
        }
        let mut active: Vec<_> = (1..vocab).filter(|&t| frequencies[t] > 0).collect();
        active.sort_by_key(|&t| (std::cmp::Reverse(frequencies[t]), t));
        active.truncate(config.learned_tokens);
        let mut seed = config.seed.max(1);
        let mut heads = Vec::new();
        for _ in 0..HEADS {
            let mut codes = || {
                (0..vocab)
                    .map(|_| (next_random(&mut seed) % ROOTS as u64) as u16)
                    .collect()
            };
            heads.push(Head {
                queries: codes(),
                keys: codes(),
                values: codes(),
                operators: vec![self.geometry.identity; ROOTS],
                emissions: vec![
                    Emission {
                        default_score: 0,
                        scores: vec![],
                        postings: vec![]
                    };
                    ROOTS
                ],
            });
        }
        let mut block = RoutingBlock {
            schema: SCHEMA.into(),
            parent_artifact: self.artifact_cid.clone(),
            mode: config.mode,
            heads,
            angular_rank: ranks(self),
            training: receipts,
            fit_config: config.clone(),
            joint: Some(JointOutput {
                additive_scores: true,
                rows: vec![],
                priors: vec![],
                positions: examples.len(),
                response_examples: responses.len(),
            }),
        };
        let (initial_output, initial) = fit_output(self, &block, &examples, responses.len());
        block.joint = Some(initial_output);
        let mut best = initial;
        let mut report = RecurrentRoutingFitReport {
            schema: SCHEMA.into(),
            positions: examples.len(),
            response_positions,
            forced_copy_positions: forced,
            documents: documents.len(),
            response_examples: responses.len(),
            proposals: 0,
            accepted_moves: [0; 4],
            initial_correct: initial.correct,
            final_correct: 0,
            initial_assembled_nll: initial.nll,
            final_assembled_nll: 0.0,
            actual_teacher_forced_correct: 0,
            cached_actual_output_agreement: 0,
            elapsed_ms: 0,
            stopped_at_time_limit: false,
            block_bytes: 0,
            config: config.clone(),
        };
        'passes: for _ in 0..config.passes {
            for head in 0..HEADS {
                for kind in 0..4 {
                    if kind < 3 && !config.learn_placement {
                        continue;
                    }
                    let indices = if kind == 3 {
                        (0..ROOTS).collect()
                    } else {
                        active.clone()
                    };
                    for index in indices {
                        let original = field(&mut block.heads[head], kind)[index];
                        let mut winner = original;
                        for _ in 0..3 {
                            if started.elapsed().as_secs_f64() >= config.max_seconds as f64 {
                                field(&mut block.heads[head], kind)[index] = winner;
                                report.stopped_at_time_limit = true;
                                break 'passes;
                            }
                            let proposal = (next_random(&mut seed) % ROOTS as u64) as u16;
                            field(&mut block.heads[head], kind)[index] = proposal;
                            let (output, objective) =
                                fit_output(self, &block, &examples, responses.len());
                            report.proposals += 1;
                            if objective.improves(best) {
                                best = objective;
                                winner = proposal;
                                block.joint = Some(output);
                            }
                        }
                        field(&mut block.heads[head], kind)[index] = winner;
                        report.accepted_moves[kind] += u64::from(winner != original);
                    }
                }
            }
        }
        // The fitted rows always belong to the restored winning coordinates.
        let (output, final_objective) = fit_output(self, &block, &examples, responses.len());
        block.joint = Some(output);
        block.validate(self)?;
        report.final_correct = final_objective.correct;
        report.final_assembled_nll = final_objective.nll;
        report.block_bytes = serde_json::to_vec(&block)
            .map_err(|e| Error(e.to_string()))?
            .len();
        let mut model = self.clone();
        model.learned_routing = Some(block);
        model.refresh_identity()?;
        // Changing dispatch can change an inherited pending commit. Quantify
        // that cached-parent-history approximation with actual candidate-model
        // teacher forcing; it is not silently called end-to-end rollout loss.
        let block = model
            .learned_routing
            .as_ref()
            .ok_or_else(|| Error("missing fitted block".into()))?;
        let joint = block
            .joint
            .as_ref()
            .ok_or_else(|| Error("missing joint output".into()))?;
        let mut cursor = 0;
        for (sequence, (prompt, tokens, response)) in prepared.iter().enumerate() {
            let mut session = model.session(Control::Full)?;
            session.observe(&model, BOS)?;
            for &token in prompt {
                session.observe(&model, token)?;
            }
            if *response {
                session.begin_response(&model)?;
            }
            for (position, &target) in tokens.iter().enumerate() {
                let actual = session.predict(&model)?.token;
                if let Some(example) = examples
                    .get(cursor)
                    .filter(|e| e.sequence == sequence && e.position == position)
                {
                    let decision = block.route(
                        &model,
                        &example.context[..example.length],
                        Control::Full,
                        &mut RoutingWork::default(),
                    );
                    let keys = features(
                        &model,
                        decision,
                        example.baseline,
                        example.flags,
                        &mut RoutingWork::default(),
                    );
                    let (action, _, _) =
                        joint.choose(keys, example.baseline, &mut RoutingWork::default());
                    let cached = if action == BOS {
                        example.baseline
                    } else {
                        action
                    };
                    report.actual_teacher_forced_correct += usize::from(actual == target);
                    report.cached_actual_output_agreement += usize::from(actual == cached);
                    cursor += 1;
                }
                session.observe(&model, target)?;
            }
        }
        if cursor != examples.len() {
            return Err(Error("recurrent replay coverage mismatch".into()));
        }
        report.elapsed_ms = started.elapsed().as_millis();
        Ok((model, report))
    }
}
