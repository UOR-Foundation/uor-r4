//! Unsupervised context perturbations for semantic anti-degeneracy (Phase 3).
//!
//! Evaluates region invariance under token masking, span substitution,
//! context truncation, and local word-order variation.

/// Apply random token masking to context tokens.
pub fn mask_tokens(tokens: &[u32], mask_rate: f32, mask_token: u32, seed: u64) -> Vec<u32> {
    let mut perturbed = tokens.to_vec();
    let mut state = seed;
    for token in perturbed.iter_mut() {
        // Simple xorshift PRNG for deterministic masking
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let rand_val = (state % 1000) as f32 / 1000.0;
        if rand_val < mask_rate {
            *token = mask_token;
        }
    }
    perturbed
}

/// Replace a contiguous span of tokens with a replacement sequence.
pub fn substitute_span(tokens: &[u32], start: usize, len: usize, replacement: &[u32]) -> Vec<u32> {
    let mut perturbed = Vec::with_capacity(tokens.len() + replacement.len());
    let end = (start + len).min(tokens.len());
    perturbed.extend_from_slice(&tokens[..start.min(tokens.len())]);
    perturbed.extend_from_slice(replacement);
    if end < tokens.len() {
        perturbed.extend_from_slice(&tokens[end..]);
    }
    perturbed
}

/// Truncate context to keep only the last `keep_len` tokens.
pub fn truncate_context(tokens: &[u32], keep_len: usize) -> Vec<u32> {
    let len = tokens.len();
    if len <= keep_len {
        tokens.to_vec()
    } else {
        tokens[len - keep_len..].to_vec()
    }
}

/// Swap adjacent tokens at position `idx` to test local word-order invariance.
pub fn swap_adjacent(tokens: &[u32], idx: usize) -> Vec<u32> {
    let mut perturbed = tokens.to_vec();
    if idx + 1 < perturbed.len() {
        perturbed.swap(idx, idx + 1);
    }
    perturbed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_tokens_deterministic() {
        let tokens = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let masked = mask_tokens(&tokens, 0.5, 0, 42);
        assert_eq!(masked.len(), tokens.len());
        // Verify determinism with same seed
        let masked2 = mask_tokens(&tokens, 0.5, 0, 42);
        assert_eq!(masked, masked2);
    }

    #[test]
    fn test_substitute_span() {
        let tokens = vec![10, 20, 30, 40, 50];
        let sub = substitute_span(&tokens, 1, 2, &[99, 100]);
        assert_eq!(sub, vec![10, 99, 100, 40, 50]);
    }

    #[test]
    fn test_truncate_context() {
        let tokens = vec![1, 2, 3, 4, 5];
        assert_eq!(truncate_context(&tokens, 3), vec![3, 4, 5]);
        assert_eq!(truncate_context(&tokens, 10), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_swap_adjacent() {
        let tokens = vec![1, 2, 3, 4];
        assert_eq!(swap_adjacent(&tokens, 1), vec![1, 3, 2, 4]);
    }
}
