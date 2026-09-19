//! A low-bit recurrent core: the learned composition D0-b unlocks.
//!
//! # What this is
//!
//! A minimal autoregressive core that can actually be *trained* and then served with no multiplier:
//!
//! ```text
//!   h_t = relu( W_x[token_t] + W_h · h_{t-1} )      integer, ternaries, relu = max(0, .)
//!   logits = W_o · h_t                               integer, ternaries
//! ```
//!
//! Every matrix is a [`TernaryLinear`]: ternary weights at 2 bits with a power-of-two per-row
//! scale, so each layer is adds, subtracts and shifts. `relu` is a `max` against zero. There is no
//! floating point, no multiplier and no transcendental on the serving path, and the exactness of
//! the integer path against its floating reference is verified by test rather than asserted.
//!
//! # Why a recurrence, and why these three matrices
//!
//! A fixed-window feed-forward map cannot express "what came earlier", and the project has already
//! paid for that mistake: an additive scorer over count features is why the current model emits
//! repetitive fragments. A recurrent state is the smallest structure that can carry context
//! forward, and `W_h` is where the learned transition lives. `W_x` is the learned embedding —
//! replacing the `prime % 120` hash, which was a fixed function of the token id and therefore
//! carried no learned information at all. `W_o` is the learned readout.
//!
//! That is a genuine change in kind from the current model, not more of it: there is now a learned
//! function from context to a distribution, rather than a sum of count lookups.
//!
//! # What is deliberately absent
//!
//! * **The training loop.** Master weights in `f32`, a straight-through estimator for the ternary
//!   quantisation, BPTT and Adam are not written here. `float_forward_ste` provides the forward
//!   shape that a trainer would need; the backward pass is the next piece.
//! * **Chat data.** This core will learn whatever it is given; TinyStories will not produce a
//!   conversational model.
//! * **Scale.** `dim` here is tens, not thousands. The point is a working, verifiable composition,
//!   not a claim of capability.
//!
//! No capability is claimed for this module. It is a core, and the evidence for it is that its
//! arithmetic is exact and its serving path is multiplier-free.

#![forbid(unsafe_code)]

use super::lowbit::TernaryLinear;

/// Dimension of the recurrent state.
pub const DEFAULT_STATE_DIM: usize = 64;

/// A ternary recurrent core in its serving form: three ternary matrices, integer arithmetic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LowBitCore {
    pub vocab: usize,
    pub dim: usize,
    /// Learned embedding, `dim x vocab`.
    pub w_x: TernaryLinear,
    /// Learned recurrent transition, `dim x dim`.
    pub w_h: TernaryLinear,
    /// Learned readout, `vocab x dim`.
    pub w_o: TernaryLinear,
}

impl LowBitCore {
    /// Build from float master matrices, quantising to ternary with power-of-two scales.
    ///
    /// Offline only. This is the bridge from training (float) to serving (integer): the trainer
    /// holds `f32` masters and hands them here, and what comes out is what executes.
    pub fn from_f32(
        vocab: usize,
        dim: usize,
        wx: &[f32],
        wh: &[f32],
        wo: &[f32],
    ) -> Result<Self, String> {
        if vocab == 0 || dim == 0 {
            return Err("vocab and dim must be non-zero".into());
        }
        if wx.len() != dim * vocab {
            return Err(format!(
                "w_x must be {dim}x{vocab} = {}, got {}",
                dim * vocab,
                wx.len()
            ));
        }
        if wh.len() != dim * dim {
            return Err(format!(
                "w_h must be {dim}x{dim} = {}, got {}",
                dim * dim,
                wh.len()
            ));
        }
        if wo.len() != vocab * dim {
            return Err(format!(
                "w_o must be {vocab}x{dim} = {}, got {}",
                vocab * dim,
                wo.len()
            ));
        }
        Ok(Self {
            vocab,
            dim,
            w_x: TernaryLinear::quantize(wx, dim, vocab),
            w_h: TernaryLinear::quantize(wh, dim, dim),
            w_o: TernaryLinear::quantize(wo, vocab, dim),
        })
    }

    /// The zero state: all zeros, which `relu` leaves at zero.
    #[inline]
    pub fn initial_state(&self) -> Vec<i32> {
        vec![0i32; self.dim]
    }

    /// One recurrent step. `h` must have length `dim`; `token` must be in `0..vocab`.
    ///
    /// Multiplier-free: `W_x[token]` is a selected column read, `W_h · h` is adds/shifts, and
    /// `relu` is a max. Returns the next state.
    pub fn step(&self, h: &[i32], token: u32) -> Vec<i32> {
        assert_eq!(h.len(), self.dim, "state length must equal dim");
        let t = (token as usize).min(self.vocab - 1);

        // W_x[token]: column `t` of a dim x vocab ternary matrix, scaled per row. A selected
        // column read, not a contraction over the vocabulary.
        let mut next = vec![0i32; self.dim];
        for (r, slot) in next.iter_mut().enumerate() {
            let w = self.w_x.weight(r, t);
            *slot = w << self.w_x.shift(r);
        }

        // W_h · h, added in. Both terms are pre-scaled, so they combine as integers directly.
        let recurrent = self.w_h.forward_i32(h);
        for (r, v) in recurrent.iter().enumerate() {
            next[r] += *v;
        }

        // relu: a max against zero, no multiply and no float.
        for v in next.iter_mut() {
            if *v < 0 {
                *v = 0;
            }
        }
        next
    }

    /// Logits over the vocabulary for a state.
    pub fn logits(&self, h: &[i32]) -> Vec<i32> {
        self.w_o.forward_i32(h)
    }

    /// Run a whole token sequence and return the logits of the final step.
    ///
    /// This is the autoregressive forward pass: feed `tokens`, get a distribution for the next one.
    pub fn forward_i32(&self, tokens: &[u32]) -> Vec<i32> {
        let mut h = self.initial_state();
        for &t in tokens {
            h = self.step(&h, t);
        }
        self.logits(&h)
    }

    /// Total packed weight bytes, for the bytes-per-token accounting.
    pub fn weight_bytes(&self) -> usize {
        self.w_x.weight_bytes() + self.w_h.weight_bytes() + self.w_o.weight_bytes()
    }

    /// Floating-point forward using the *quantised* weights in f64, for verifying the integer path.
    pub fn float_forward_ste(&self, tokens: &[u32]) -> Vec<f64> {
        let mut h = vec![0.0f64; self.dim];
        for &t in tokens {
            let token = (t as usize).min(self.vocab - 1);
            let mut next = vec![0.0f64; self.dim];
            for (r, slot) in next.iter_mut().enumerate() {
                *slot = self.w_x.weight(r, token) as f64 * (1u64 << self.w_x.shift(r)) as f64;
            }
            let recurrent = self.w_h.forward_reference(&h);
            for (r, v) in recurrent.iter().enumerate() {
                next[r] += *v;
            }
            for v in next.iter_mut() {
                if *v < 0.0 {
                    *v = 0.0;
                }
            }
            h = next;
        }
        self.w_o.forward_reference(&h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deterministic small core, built from pseudo-random float masters.
    fn tiny() -> LowBitCore {
        let vocab = 11usize;
        let dim = 8usize;
        let mut state = 0x1234_5678_9ABC_DEF0u64;
        let mut next_f32 = |scale: f32| -> f32 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let u = ((state >> 11) as f64 / (1u64 << 53) as f64) as f32;
            (u * 2.0 - 1.0) * scale
        };
        let wx: Vec<f32> = (0..dim * vocab).map(|_| next_f32(2.0)).collect();
        let wh: Vec<f32> = (0..dim * dim).map(|_| next_f32(1.0)).collect();
        let wo: Vec<f32> = (0..vocab * dim).map(|_| next_f32(3.0)).collect();
        LowBitCore::from_f32(vocab, dim, &wx, &wh, &wo).expect("build")
    }

    #[test]
    fn integer_forward_is_exactly_the_float_reference() {
        let core = tiny();
        let tokens: Vec<u32> = vec![0, 3, 7, 1, 10, 2, 2, 5];
        let got = core.forward_i32(&tokens);
        let want = core.float_forward_ste(&tokens);
        assert_eq!(got.len(), core.vocab);
        for j in 0..core.vocab {
            assert_eq!(
                got[j] as f64, want[j],
                "vocab {j}: serving path {} != reference {}",
                got[j], want[j]
            );
        }
    }

    #[test]
    fn relu_keeps_the_state_non_negative() {
        let core = tiny();
        let mut h = core.initial_state();
        for t in [1u32, 4, 9, 0, 3] {
            h = core.step(&h, t);
            assert!(
                h.iter().all(|&v| v >= 0),
                "relu must leave no negative state"
            );
        }
    }

    #[test]
    fn forward_is_deterministic_and_sensitive_to_order() {
        let core = tiny();
        let a = core.forward_i32(&[1, 2, 3]);
        let b = core.forward_i32(&[1, 2, 3]);
        assert_eq!(a, b, "same input must give the same output");
        // The recurrence means order matters; a bag-of-tokens core could not tell these apart.
        let c = core.forward_i32(&[3, 2, 1]);
        assert_ne!(a, c, "order must change the logits");
    }

    #[test]
    fn empty_sequence_gives_the_readout_of_the_zero_state() {
        let core = tiny();
        let logits = core.forward_i32(&[]);
        assert_eq!(logits.len(), core.vocab);
    }

    #[test]
    fn out_of_range_token_is_clamped_not_a_panic() {
        let core = tiny();
        let logits = core.forward_i32(&[u32::MAX]);
        assert_eq!(logits.len(), core.vocab);
    }

    #[test]
    fn weights_are_ternary_at_two_bits_and_the_core_is_small() {
        let core = tiny();
        let n = core.vocab * core.dim + core.dim * core.dim + core.vocab * core.dim;
        assert_eq!(core.weight_bytes(), n.div_ceil(4));
        for r in 0..core.dim {
            for c in 0..core.vocab {
                let w = core.w_x.weight(r, c);
                assert!(w == -1 || w == 0 || w == 1);
            }
        }
        // At 2 bits per weight, a 4096-vocabulary, 64-dim core would be about 130 KB of weights.
        assert!(core.weight_bytes() * 8 <= n * 4);
    }

    #[test]
    fn rejects_inconsistent_shapes() {
        let bad = LowBitCore::from_f32(4, 2, &[0.0; 7], &[0.0; 4], &[0.0; 8]);
        assert!(bad.is_err());
        assert!(LowBitCore::from_f32(0, 2, &[], &[0.0; 4], &[]).is_err());
    }
}
