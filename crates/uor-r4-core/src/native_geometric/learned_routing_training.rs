//! Host-only hard-coordinate learning. Each proposal runs the serving selector
//! and transport, then refits sparse emission counts. No soft routing is used.
use super::learned_routing::{Emission, Head, RoutingBlock, HEADS, ROOTS, WINDOW};
use super::{
    Control, Document, DocumentReceipt, Error, Model, Result, RoutingMode, RoutingWork, TokenScore,
    BOS, EOS, SCORE_SCALE,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::time::Instant;

const SCHEMA: &str = "uor-r4.learned-h4-routing/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingFitConfig {
    pub max_positions: usize,
    pub learned_tokens: usize,
    pub passes: usize,
    pub max_seconds: u64,
    pub mode: RoutingMode,
    pub learn_placement: bool,
    pub seed: u64,
}
impl Default for RoutingFitConfig {
    fn default() -> Self {
        Self {
            max_positions: 2048,
            learned_tokens: 24,
            passes: 2,
            max_seconds: 15,
            mode: RoutingMode::Angular,
            learn_placement: true,
            seed: 1139,
        }
    }
}
impl RoutingFitConfig {
    fn validate(&self) -> Result<()> {
        if !(1..=8192).contains(&self.max_positions)
            || !(1..=64).contains(&self.learned_tokens)
            || !(1..=4).contains(&self.passes)
            || !(1..=120).contains(&self.max_seconds)
        {
            return Err(Error("learned routing fit exceeds supported bounds".into()));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingFitReport {
    pub schema: String,
    pub positions: usize,
    pub documents: usize,
    pub proposals: u64,
    pub accepted_query_moves: u64,
    pub accepted_key_moves: u64,
    pub accepted_value_moves: u64,
    pub accepted_operator_moves: u64,
    pub initial_conditional_nll: Vec<f64>,
    pub final_conditional_nll: Vec<f64>,
    pub elapsed_ms: u128,
    pub stopped_at_time_limit: bool,
    pub block_bytes: usize,
    pub config: RoutingFitConfig,
}
struct Example {
    context: [u32; WINDOW],
    length: usize,
    target: u32,
}

fn ranks(model: &Model) -> Vec<u16> {
    let mut cosines: Vec<_> = model
        .geometry
        .anchors
        .rows
        .iter()
        .map(|row| row.root_scaled_zphi[0])
        .collect();
    cosines.sort_by(|a, b| super::training::exact_sign([a[0] - b[0], a[1] - b[1]]).cmp(&0));
    cosines.dedup();
    model
        .geometry
        .anchors
        .rows
        .iter()
        .map(|row| {
            cosines
                .iter()
                .position(|v| *v == row.root_scaled_zphi[0])
                .unwrap_or(0) as u16
        })
        .collect()
}

impl RoutingBlock {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        self.fit_config.validate()?;
        if self.schema != SCHEMA
            || self.mode != self.fit_config.mode
            || self.heads.len() != HEADS
            || self.angular_rank != ranks(model)
            || self.training.is_empty()
        {
            return Err(Error(
                "learned routing schema, geometry or provenance invalid".into(),
            ));
        }
        let mut ids = BTreeSet::new();
        for receipt in &self.training {
            if receipt.id.is_empty()
                || receipt.bytes == 0
                || !ids.insert(&receipt.id)
                || !receipt.text_cid.starts_with("blake3:")
            {
                return Err(Error("learned routing receipt invalid".into()));
            }
        }
        let vocab = model.prior_scores.len();
        for head in &self.heads {
            for codes in [&head.queries, &head.keys, &head.values] {
                if codes.len() != vocab || codes.iter().any(|&v| usize::from(v) >= ROOTS) {
                    return Err(Error("learned routing token code out of range".into()));
                }
            }
            if head.operators.len() != ROOTS
                || head.operators.iter().any(|&v| usize::from(v) >= ROOTS)
                || head.emissions.len() != ROOTS
            {
                return Err(Error("learned routing operator shape invalid".into()));
            }
            for row in &head.emissions {
                if row.scores.len() > vocab
                    || row.scores.windows(2).any(|p| p[0].token >= p[1].token)
                    || row
                        .scores
                        .iter()
                        .any(|p| p.token == BOS || p.token as usize >= vocab || p.score > 0)
                    || row.default_score > 0
                    || row.postings.len() > 8
                    || row
                        .postings
                        .iter()
                        .any(|token| row.scores.binary_search_by_key(token, |p| p.token).is_err())
                {
                    return Err(Error("learned routing emission shape invalid".into()));
                }
            }
        }
        Ok(())
    }
}

struct Counts {
    counts: Vec<u32>,
    totals: Vec<u32>,
    touched: Vec<usize>,
    logs: Vec<f64>,
    vocab: usize,
}
impl Counts {
    fn new(vocab: usize, positions: usize) -> Self {
        Self {
            counts: vec![0; ROOTS * vocab],
            totals: vec![0; ROOTS],
            touched: Vec::with_capacity(positions),
            logs: (0..=positions).map(|n| (n as f64 + 0.5).ln()).collect(),
            vocab,
        }
    }
    fn fit(
        &mut self,
        head: &Head,
        model: &Model,
        ranks: &[u16],
        mode: RoutingMode,
        examples: &[Example],
    ) -> f64 {
        for &index in &self.touched {
            self.counts[index] = 0;
        }
        self.touched.clear();
        self.totals.fill(0);
        for example in examples {
            let route = head.route(
                model,
                ranks,
                mode,
                &example.context[..example.length],
                Control::Full,
                &mut RoutingWork::default(),
            );
            let root = usize::from(route.output);
            let index = root * self.vocab + example.target as usize;
            if self.counts[index] == 0 {
                self.touched.push(index);
            }
            self.counts[index] += 1;
            self.totals[root] += 1;
        }
        let normalizers: f64 = self
            .totals
            .iter()
            .filter(|&&n| n > 0)
            .map(|&n| n as f64 * (n as f64 + 0.5 * self.vocab as f64).ln())
            .sum();
        let observed: f64 = self
            .touched
            .iter()
            .map(|&i| {
                let n = self.counts[i] as usize;
                n as f64 * self.logs[n]
            })
            .sum();
        (normalizers - observed) / examples.len() as f64
    }
    fn emissions(&self) -> Vec<Emission> {
        (0..ROOTS)
            .map(|root| {
                let denominator = self.totals[root] as f64 + 0.5 * self.vocab as f64;
                let quantize =
                    |n| (SCORE_SCALE * ((n as f64 + 0.5) / denominator).ln()).round() as i32;
                let scores: Vec<_> = (1..self.vocab)
                    .filter_map(|token| {
                        let count = self.counts[root * self.vocab + token];
                        (count > 0).then(|| TokenScore {
                            token: token as u32,
                            score: quantize(count),
                        })
                    })
                    .collect();
                let mut ranked = scores.clone();
                ranked.sort_by(|a, b| b.score.cmp(&a.score).then(a.token.cmp(&b.token)));
                Emission {
                    default_score: quantize(0),
                    scores,
                    postings: ranked.iter().take(8).map(|p| p.token).collect(),
                }
            })
            .collect()
    }
}

fn next_random(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}
fn field(head: &mut Head, kind: usize) -> &mut Vec<u16> {
    match kind {
        0 => &mut head.queries,
        1 => &mut head.keys,
        2 => &mut head.values,
        _ => &mut head.operators,
    }
}

impl Model {
    pub fn fit_routing_block(
        &self,
        documents: &[Document],
        config: RoutingFitConfig,
    ) -> Result<(Self, RoutingFitReport)> {
        config.validate()?;
        if documents.is_empty()
            || documents.len() > 256
            || self.prior_scores.len() > 8192
            || self.learned_routing.is_some()
        {
            return Err(Error(
                "learned routing corpus/vocabulary bound exceeded".into(),
            ));
        }
        let started = Instant::now();
        let vocab = self.prior_scores.len();
        let mut examples = Vec::new();
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut frequencies = vec![0_u32; vocab];
        for (document_index, document) in documents.iter().enumerate() {
            if document.id.is_empty()
                || document.text.is_empty()
                || document.text.len() > 65536
                || !ids.insert(document.id.clone())
            {
                return Err(Error("learned routing document invalid".into()));
            }
            receipts.push(DocumentReceipt {
                id: document.id.clone(),
                text_cid: format!("blake3:{}", blake3::hash(document.text.as_bytes()).to_hex()),
                bytes: document.text.len(),
            });
            let mut tokens = self.encode(&document.text)?;
            tokens.push(EOS);
            let quota = (config.max_positions / documents.len()
                + usize::from(document_index < config.max_positions % documents.len()))
            .min(tokens.len());
            let mut context = [BOS; WINDOW];
            let mut length = 1;
            for (position, target) in tokens.iter().copied().enumerate() {
                // Uniformly cover the whole document instead of silently truncating its end.
                if quota > 0
                    && ((position + 1) * quota / tokens.len() > position * quota / tokens.len())
                {
                    examples.push(Example {
                        context,
                        length,
                        target,
                    });
                    for &token in &context[..length] {
                        frequencies[token as usize] += 1;
                    }
                }
                context.rotate_right(1);
                context[0] = target;
                length = (length + 1).min(WINDOW);
            }
        }
        if examples.is_empty() {
            return Err(Error("no learned routing positions".into()));
        }
        let mut active: Vec<_> = (1..vocab).filter(|&i| frequencies[i] > 0).collect();
        active.sort_by_key(|&i| (std::cmp::Reverse(frequencies[i]), i));
        active.truncate(config.learned_tokens);
        let angular_rank = ranks(self);
        let mut heads = Vec::new();
        let mut report = RoutingFitReport {
            schema: SCHEMA.into(),
            positions: examples.len(),
            documents: documents.len(),
            proposals: 0,
            accepted_query_moves: 0,
            accepted_key_moves: 0,
            accepted_value_moves: 0,
            accepted_operator_moves: 0,
            initial_conditional_nll: Vec::new(),
            final_conditional_nll: Vec::new(),
            elapsed_ms: 0,
            stopped_at_time_limit: false,
            block_bytes: 0,
            config: config.clone(),
        };
        for head_index in 0..HEADS {
            let mut seed = config.seed.wrapping_add(head_index as u64).max(1);
            let mut codes = || {
                (0..vocab)
                    .map(|_| (next_random(&mut seed) % ROOTS as u64) as u16)
                    .collect()
            };
            let mut head = Head {
                queries: codes(),
                keys: codes(),
                values: codes(),
                operators: vec![self.geometry.identity; ROOTS],
                emissions: Vec::new(),
            };
            let mut counts = Counts::new(vocab, examples.len());
            let mut best = counts.fit(&head, self, &angular_rank, config.mode, &examples);
            report.initial_conditional_nll.push(best);
            'passes: for _ in 0..config.passes {
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
                        let original = field(&mut head, kind)[index];
                        let mut winner = original;
                        for _ in 0..3 {
                            if started.elapsed().as_secs_f64() >= config.max_seconds as f64 {
                                field(&mut head, kind)[index] = winner;
                                report.stopped_at_time_limit = true;
                                break 'passes;
                            }
                            let proposal = (next_random(&mut seed) % ROOTS as u64) as u16;
                            field(&mut head, kind)[index] = proposal;
                            let loss =
                                counts.fit(&head, self, &angular_rank, config.mode, &examples);
                            report.proposals += 1;
                            if loss + 1e-9 < best {
                                best = loss;
                                winner = proposal;
                            }
                        }
                        field(&mut head, kind)[index] = winner;
                        if winner != original {
                            match kind {
                                0 => report.accepted_query_moves += 1,
                                1 => report.accepted_key_moves += 1,
                                2 => report.accepted_value_moves += 1,
                                _ => report.accepted_operator_moves += 1,
                            }
                        }
                    }
                }
            }
            report.final_conditional_nll.push(counts.fit(
                &head,
                self,
                &angular_rank,
                config.mode,
                &examples,
            ));
            head.emissions = counts.emissions();
            heads.push(head);
        }
        let block = RoutingBlock {
            schema: SCHEMA.into(),
            parent_artifact: self.artifact_cid.clone(),
            mode: config.mode,
            heads,
            angular_rank,
            training: receipts,
            fit_config: config,
        };
        block.validate(self)?;
        report.block_bytes = serde_json::to_vec(&block)
            .map_err(|e| Error(e.to_string()))?
            .len();
        let mut model = self.clone();
        model.learned_routing = Some(block);
        model.refresh_identity()?;
        report.elapsed_ms = started.elapsed().as_millis();
        Ok((model, report))
    }
}
