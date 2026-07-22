//! Semantic anti-degeneracy corpus transformations, perturbation suites,
//! MDL objective J(C) calculation, and evaluation harness (Phase 3 / PDF §7 / Gate G).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PerturbationKind {
    Masking { mask_rate: f64, mask_token: u32 },
    SpanSubstitution { substitute_rate: f64 },
    Truncation { keep_fraction: f64 },
    Reorder { shuffle_window: usize },
    Counterfactual { flip_polarity: bool },
}

pub struct PerturbationSuite;

impl PerturbationSuite {
    /// Apply deterministic perturbation transformation on token stream under seed `s`.
    pub fn apply_perturbation(tokens: &[u32], kind: &PerturbationKind, seed: u64) -> Vec<u32> {
        if tokens.is_empty() {
            return Vec::new();
        }
        let mut out = tokens.to_vec();
        match kind {
            PerturbationKind::Masking {
                mask_rate,
                mask_token,
            } => {
                let rate = mask_rate.clamp(0.0, 1.0);
                for (i, tok) in out.iter_mut().enumerate() {
                    let pseudo_rand = (seed.wrapping_add(i as u64).wrapping_mul(6364136223846793005) >> 33) as f64 / 2147483648.0;
                    if pseudo_rand < rate {
                        *tok = *mask_token;
                    }
                }
            }
            PerturbationKind::SpanSubstitution { substitute_rate } => {
                let rate = substitute_rate.clamp(0.0, 1.0);
                for (i, tok) in out.iter_mut().enumerate() {
                    let pseudo_rand = (seed.wrapping_add((i as u64).wrapping_mul(17)).wrapping_mul(6364136223846793005) >> 33) as f64 / 2147483648.0;
                    if pseudo_rand < rate {
                        *tok = tok.wrapping_add(100);
                    }
                }
            }
            PerturbationKind::Truncation { keep_fraction } => {
                let keep_len = ((tokens.len() as f64) * keep_fraction.clamp(0.0, 1.0)).round() as usize;
                out.truncate(keep_len.max(1));
            }
            PerturbationKind::Reorder { shuffle_window } => {
                let win = (*shuffle_window).max(1);
                for chunk in out.chunks_mut(win) {
                    if chunk.len() > 1 {
                        chunk.rotate_left(1);
                    }
                }
            }
            PerturbationKind::Counterfactual { flip_polarity } => {
                if *flip_polarity {
                    for tok in out.iter_mut() {
                        *tok ^= 0x01;
                    }
                }
            }
        }
        out
    }
}

/// Compute Minimum Description Length (MDL) objective J(C) = L(Graph) + lambda * L(Residuals | Graph)
pub struct MdlObjective;

impl MdlObjective {
    pub fn compute_j_c(graph_model_bytes: usize, residual_entropy_bits: f64, lambda: f64) -> f64 {
        let graph_bits = (graph_model_bytes as f64) * 8.0;
        graph_bits + lambda * residual_entropy_bits
    }
}

/// Evaluate paraphrase trajectory agreement between original and paraphrased token sequences.
pub struct ParaphraseEvaluator;

impl ParaphraseEvaluator {
    pub fn evaluate_paraphrase_agreement(trajectory1: &[u32], trajectory2: &[u32]) -> f64 {
        if trajectory1.is_empty() || trajectory2.is_empty() {
            return 0.0;
        }
        let min_len = trajectory1.len().min(trajectory2.len());
        let matches = trajectory1.iter().zip(trajectory2.iter()).filter(|(a, b)| a == b).count();
        matches as f64 / min_len as f64
    }
}

/// Evaluate polysemy separation across distinct context windows.
pub struct PolysemyEvaluator;

impl PolysemyEvaluator {
    pub fn evaluate_polysemy_separation(contexts_a: &[Vec<u32>], contexts_b: &[Vec<u32>]) -> f64 {
        if contexts_a.is_empty() || contexts_b.is_empty() {
            return 1.0;
        }
        let total_diffs: usize = contexts_a
            .iter()
            .zip(contexts_b.iter())
            .map(|(ca, cb)| {
                ca.iter().zip(cb.iter()).filter(|(a, b)| a != b).count()
            })
            .sum();
        let total_compared: usize = contexts_a
            .iter()
            .zip(contexts_b.iter())
            .map(|(ca, cb)| ca.len().min(cb.len()))
            .sum();
        if total_compared == 0 {
            return 1.0;
        }
        total_diffs as f64 / total_compared as f64
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticCoherenceCertificate {
    pub region_reuse_rate: f64,
    pub invariance_score: f64,
    pub boundary_stability: f64,
    pub mdl_cost_j_c: f64,
    pub anti_memorization_passed: bool,
}

impl SemanticCoherenceCertificate {
    pub fn verify(&self) -> bool {
        self.region_reuse_rate >= 0.5
            && self.invariance_score >= 0.7
            && self.boundary_stability >= 0.8
            && self.anti_memorization_passed
    }
}
