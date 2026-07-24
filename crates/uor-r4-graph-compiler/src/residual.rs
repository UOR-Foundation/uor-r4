//! Residual quantization (Phase 4).
//! Quantizes teacher log-probs into fixed-point ScoreQ residuals.

use uor_r4_graph_format::ScoreQ;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantizedResidual {
    pub token: u32,
    pub score: ScoreQ,
}

/// Quantize a list of f32 log-probabilities into `ScoreQ` fixed-point entries.
pub fn quantize_logprobs(tokens: &[u32], logprobs: &[f32]) -> Vec<QuantizedResidual> {
    tokens
        .iter()
        .zip(logprobs.iter())
        .map(|(&token, &lp)| QuantizedResidual {
            token,
            score: ScoreQ::from_logprob(lp),
        })
        .collect()
}

/// Compute residual delta corrections between child region scores and parent region scores:
/// Delta = ScoreQ(child) - ScoreQ(parent).
pub fn compute_residual_deltas(
    child_residuals: &[QuantizedResidual],
    parent_residuals: &[QuantizedResidual],
) -> Vec<QuantizedResidual> {
    let mut deltas = Vec::new();
    for child in child_residuals {
        let parent_score = parent_residuals
            .iter()
            .find(|p| p.token == child.token)
            .map(|p| p.score)
            .unwrap_or(ScoreQ::ZERO);
        let delta_score = child.score.saturating_sub(parent_score);
        deltas.push(QuantizedResidual {
            token: child.token,
            score: delta_score,
        });
    }
    deltas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantize_and_delta() {
        let tokens = vec![1, 2, 3];
        let child_lps = vec![-0.5, -1.2, -2.0];
        let parent_lps = vec![-1.0, -1.2, -3.0];

        let child_q = quantize_logprobs(&tokens, &child_lps);
        let parent_q = quantize_logprobs(&tokens, &parent_lps);
        let deltas = compute_residual_deltas(&child_q, &parent_q);

        assert_eq!(deltas.len(), 3);
        assert!(deltas[0].score.raw() > 0); // child was better than parent
        assert_eq!(deltas[1].score.raw(), 0); // child and parent equal
        assert!(deltas[2].score.raw() > 0); // child better than parent
    }
}
