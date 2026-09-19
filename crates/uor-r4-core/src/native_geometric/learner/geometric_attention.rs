//! Geometric attention: an exactly-addressed associative memory over the 2I/H4 address space.
//!
//! # The mechanism
//!
//! Linear attention stores *pairs* and reads them with a soft matched filter:
//! `y = Σ_s (q·k_s) v_s`. Its limit is cross-talk — an unmatched pair contributes
//! `Θ(√dk)` of noise, so the signal-to-noise ratio falls as `√(dk/N)` and the horizon is bought
//! only with width (`lowbit_attention.rs` measures exactly this).
//!
//! This core stores the same pairs but reads them by **exact address**:
//!
//! ```text
//!   write:  S[addr(w)]  += v(t)            w = previous token, t = current token
//!   read:   y          = S[addr(q)]        q = current (query) token
//! ```
//!
//! Different addresses do not interfere. A pair whose address is unique is retrieved with **zero
//! cross-talk**, at `O(dv)` per token instead of `O(dk·dv)`, and with no multiplier anywhere: the
//! address is a table read, the update is an add, the read is a table read.
//!
//! The semantics are a learned, exactly-addressed **bigram/associative table** — store "what
//! followed address `a`", retrieve it when `a` appears again. That is the project's *engram*
//! idea, and it is the mechanism `LowBitAttention` can only approximate.
//!
//! # The address space is the project's group
//!
//! Addresses are elements of the project's verified binary icosahedral group `2I` — the 120
//! canonical H4 roots, with composition by the machine-checked group table
//! (`learner/group_table.rs`, associativity over all 1,728,000 triples). With two factors the
//! address is the exact composition `compose(a₁, a₂)` paired with `a₁`, giving 120² = 14,400
//! addresses at 13.8 bits. This is the project's own resolution lever: `H4^k` factors with exact
//! integer composition.
//!
//! # Honest limits
//!
//! * The address **assignment** is fixed at construction (a deterministic function of the token id),
//!   exactly as the project itself initialises `token_to_root`. Learning the factors is the Stage 3
//!   product-code work, not this module. What is learned here is *what is written* and *how it is
//!   read*.
//! * Capacity is `n_addr`; with more distinct keys than addresses the buckets collide and the
//!   behaviour degrades gracefully toward a hashed n-gram table, not toward a failure.
//! * This is a first-order (bigram) memory. Composition over longer contexts is what the
//!   content-addressed core's matrix state provides, so the two are complements, not rivals.

#![forbid(unsafe_code)]

use super::group_table::{group_table, GROUP_ORDER, ROW_STRIDE};
use super::lowbit::TernaryLinear;
use super::lowbit_core::{adam_update, quantize_codes, softmax_f32, xorshift_unit, TrainConfig};

/// One factor: the 120 elements of `2I`.
pub const ADDR_SPACE_ONE: usize = GROUP_ORDER;
/// Two factors: 14,400 addresses, composed exactly through the group table.
pub const ADDR_SPACE_TWO: usize = GROUP_ORDER * GROUP_ORDER;

/// The fixed address of every token, built once.
///
/// Two hashed factors, composed by the exact group table. The composition is what makes the pair an
/// element of the project's geometry rather than an arbitrary bucket index; pairing it with the
/// first factor is what restores the capacity a closed group would otherwise collapse.
pub fn build_addresses(vocab: usize, factors: usize) -> Vec<u32> {
    let table = group_table();
    (0..vocab)
        .map(|t| {
            let u = t as u64;
            let a1 = ((u.wrapping_mul(2_654_435_761) >> 7) % GROUP_ORDER as u64) as usize;
            if factors <= 1 {
                return a1 as u32;
            }
            let a2 =
                ((u.wrapping_mul(40_503).wrapping_add(12_345) >> 5) % GROUP_ORDER as u64) as usize;
            let composed = table.product[a1 * ROW_STRIDE + a2] as usize;
            (composed * GROUP_ORDER + a1) as u32
        })
        .collect()
}

/// Number of address buckets for a factor count.
pub fn address_space(factors: usize) -> usize {
    if factors <= 1 {
        ADDR_SPACE_ONE
    } else {
        ADDR_SPACE_TWO
    }
}

/// Serving form of the geometric attention core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometricAttention {
    pub vocab: usize,
    /// Value width.
    pub dv: usize,
    pub n_addr: usize,
    /// Fixed address per token.
    pub addr: Vec<u32>,
    /// Values, `vocab x dv`, ternary with a per-token power-of-two magnitude.
    pub w_v: TernaryLinear,
    /// Output map, `vocab x dv`.
    pub w_o: TernaryLinear,
    /// Target readout magnitude in bits for the power-of-two count normalisation. `0` normalises a
    /// bucket with `n` writes by `2^floor(log2 n)`, i.e. to the stored mean.
    pub norm_bits: u32,
    /// Apply `relu` to the read. With exact addressing the *selection* is the nonlinearity, and a
    /// `relu` after the read discards the sign half of the retrieved value.
    pub use_relu: bool,
}

impl GeometricAttention {
    #[allow(clippy::too_many_arguments)]
    pub fn from_f32(
        vocab: usize,
        dv: usize,
        factors: usize,
        norm_bits: u32,
        wv: &[f32],
        wo: &[f32],
    ) -> Result<Self, String> {
        if vocab == 0 || dv == 0 {
            return Err("vocab and dv must be non-zero".into());
        }
        if wv.len() != vocab * dv {
            return Err(format!("w_v must be {vocab}x{dv}, got {}", wv.len()));
        }
        if wo.len() != vocab * dv {
            return Err(format!("w_o must be {vocab}x{dv}, got {}", wo.len()));
        }
        Ok(Self {
            vocab,
            dv,
            n_addr: address_space(factors),
            addr: build_addresses(vocab, factors),
            w_v: TernaryLinear::quantize(wv, vocab, dv),
            w_o: TernaryLinear::quantize(wo, vocab, dv),
            norm_bits,
            use_relu: false,
        })
    }

    /// Choose whether the read is passed through `relu`.
    #[must_use]
    pub fn with_relu(mut self, use_relu: bool) -> Self {
        self.use_relu = use_relu;
        self
    }

    #[inline]
    pub fn state_len(&self) -> usize {
        self.n_addr * self.dv + self.n_addr
    }

    #[inline]
    pub fn initial_state(&self) -> Vec<i32> {
        vec![0i32; self.state_len()]
    }

    /// Write the pair `addr(prev) ← value(token)`. Adds only; one selected bucket.
    pub fn observe(&self, s: &mut [i32], prev: u32, token: u32) {
        let p = (prev as usize).min(self.vocab - 1);
        let t = (token as usize).min(self.vocab - 1);
        let a = self.addr[p] as usize;
        let base = a * self.dv;
        let vshift = self.w_v.shift(t);
        for j in 0..self.dv {
            s[base + j] += self.w_v.weight(t, j) << vshift;
        }
        let cbase = self.n_addr * self.dv;
        s[cbase + a] += 1;
    }

    /// Read the bucket addressed by `token`, normalised to a power-of-two magnitude.
    ///
    /// The normalisation is a bit scan plus a shift — no divide, no multiply. It matters because
    /// each token's value carries its own power-of-two gain, so without it the read magnitude varies
    /// per token and the softmax temperature varies with it.
    pub fn logits(&self, s: &[i32], token: u32) -> Vec<i32> {
        let t = (token as usize).min(self.vocab - 1);
        let a = self.addr[t] as usize;
        let base = a * self.dv;
        let mut m = 0i32;
        for j in 0..self.dv {
            m = m.max(s[base + j].abs());
        }
        let shift = if self.norm_bits > 0 && m > 0 {
            (32 - (m as u32).leading_zeros()).saturating_sub(self.norm_bits)
        } else {
            0
        };
        let mut num = vec![0i32; self.dv];
        for (j, slot) in num.iter_mut().enumerate() {
            *slot = s[base + j] >> shift;
        }
        if self.use_relu {
            for v in num.iter_mut() {
                if *v < 0 {
                    *v = 0;
                }
            }
        }
        self.w_o.forward_i32(&num)
    }

    /// Ingest a prompt (observing each consecutive pair) and read with the final token.
    pub fn forward_i32(&self, tokens: &[u32]) -> Vec<i32> {
        let mut s = self.initial_state();
        for w in tokens.windows(2) {
            self.observe(&mut s, w[0], w[1]);
        }
        let last = tokens.last().copied().unwrap_or(0);
        self.logits(&s, last)
    }

    /// The same computation in `f64`, for verifying the integer path.
    pub fn forward_reference_f64(&self, tokens: &[u32]) -> Vec<f64> {
        let mut s = vec![0f64; self.state_len()];
        let cbase = self.n_addr * self.dv;
        for w in tokens.windows(2) {
            let p = (w[0] as usize).min(self.vocab - 1);
            let t = (w[1] as usize).min(self.vocab - 1);
            let a = self.addr[p] as usize;
            let base = a * self.dv;
            let vscale = (1u64 << self.w_v.shift(t)) as f64;
            for j in 0..self.dv {
                s[base + j] += self.w_v.weight(t, j) as f64 * vscale;
            }
            s[cbase + a] += 1.0;
        }
        let last = tokens.last().copied().unwrap_or(0);
        let t = (last as usize).min(self.vocab - 1);
        let a = self.addr[t] as usize;
        let base = a * self.dv;
        let mut m = 0f64;
        for j in 0..self.dv {
            m = m.max(s[base + j].abs());
        }
        let shift = if self.norm_bits > 0 && m >= 1.0 {
            (64 - (m as u64).leading_zeros()).saturating_sub(self.norm_bits)
        } else {
            0
        };
        let div = (1u64 << shift) as f64;
        let mut num = vec![0f64; self.dv];
        for j in 0..self.dv {
            num[j] = (s[base + j] / div).floor();
            if self.use_relu {
                for v in num.iter_mut() {
                    if *v < 0.0 {
                        *v = 0.0;
                    }
                }
            }
        }
        self.w_o.forward_reference(&num)
    }

    pub fn weight_bytes(&self) -> usize {
        self.w_v.weight_bytes() + self.w_o.weight_bytes()
    }

    /// Mean next-token cross-entropy under the quantised weights (offline only).
    pub fn sequence_loss(&self, tokens: &[u32]) -> f32 {
        if tokens.len() < 2 {
            return 0.0;
        }
        let mut s = vec![0f32; self.state_len()];
        let cbase = self.n_addr * self.dv;
        let preds = tokens.len() - 1;
        let mut total = 0f64;
        for i in 0..preds {
            if i > 0 {
                let p = (tokens[i - 1] as usize).min(self.vocab - 1);
                let t = (tokens[i] as usize).min(self.vocab - 1);
                let a = self.addr[p] as usize;
                let base = a * self.dv;
                let vscale = (1u64 << self.w_v.shift(t)) as f32;
                for j in 0..self.dv {
                    s[base + j] += self.w_v.weight(t, j) as f32 * vscale;
                }
                s[cbase + a] += 1.0;
            }
            let q = (tokens[i] as usize).min(self.vocab - 1);
            let a = self.addr[q] as usize;
            let base = a * self.dv;
            let mut m = 0f32;
            for j in 0..self.dv {
                m = m.max(s[base + j].abs());
            }
            let shift = if self.norm_bits > 0 && m >= 1.0 {
                (32 - (m as u32).leading_zeros()).saturating_sub(self.norm_bits)
            } else {
                0
            };
            let div = (1u64 << shift) as f32;
            let mut num = vec![0f32; self.dv];
            for j in 0..self.dv {
                num[j] = if self.use_relu {
                    (s[base + j] / div).floor().max(0.0)
                } else {
                    (s[base + j] / div).floor()
                };
            }
            let mut logits = vec![0f32; self.vocab];
            for (r, slot) in logits.iter_mut().enumerate() {
                let mut acc = 0f32;
                for j in 0..self.dv {
                    acc += self.w_o.weight(r, j) as f32 * num[j];
                }
                *slot = acc * (1u64 << self.w_o.shift(r)) as f32;
            }
            let p = softmax_f32(&logits);
            let target = (tokens[i + 1] as usize).min(self.vocab - 1);
            total += -(p[target].max(1e-9)).ln() as f64;
        }
        (total / preds as f64) as f32
    }
}

// ---------------------------------------------------------------------------
// Offline training. Only the value and output tables are learned; the addresses are fixed.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct GeometricAttentionTrainer {
    pub vocab: usize,
    pub dv: usize,
    pub factors: usize,
    pub n_addr: usize,
    pub addr: Vec<u32>,
    pub norm_bits: u32,
    /// Apply `relu` to the read.
    pub use_relu: bool,
    pub wv: Vec<f32>,
    pub wo: Vec<f32>,
    pub cfg: TrainConfig,
    gwv: Vec<f32>,
    gwo: Vec<f32>,
    mwv: Vec<f32>,
    vwv: Vec<f32>,
    mwo: Vec<f32>,
    vwo: Vec<f32>,
    step: u64,
    /// Reusable state buffer; only `touched` buckets are ever non-zero.
    scratch: Vec<f32>,
    /// Reusable bucket-gradient buffer for the backward pass.
    dscratch: Vec<f32>,
    touched: Vec<u32>,
}

impl GeometricAttentionTrainer {
    pub fn new(
        vocab: usize,
        dv: usize,
        factors: usize,
        norm_bits: u32,
        seed: u64,
    ) -> Result<Self, String> {
        if vocab == 0 || dv == 0 {
            return Err("vocab and dv must be non-zero".into());
        }
        let mut st = seed ^ 0x9E37_79B9_7F4A_7C15;
        if st == 0 {
            st = 0x1234_5678_9ABC_DEF0;
        }
        let mut fill = |n: usize| -> Vec<f32> { (0..n).map(|_| xorshift_unit(&mut st)).collect() };
        Ok(Self {
            vocab,
            dv,
            factors,
            n_addr: address_space(factors),
            addr: build_addresses(vocab, factors),
            norm_bits,
            use_relu: false,
            wv: fill(vocab * dv),
            wo: fill(vocab * dv),
            cfg: TrainConfig::default(),
            gwv: vec![0f32; vocab * dv],
            gwo: vec![0f32; vocab * dv],
            mwv: vec![0f32; vocab * dv],
            vwv: vec![0f32; vocab * dv],
            mwo: vec![0f32; vocab * dv],
            vwo: vec![0f32; vocab * dv],
            step: 0,
            scratch: Vec::new(),
            dscratch: Vec::new(),
            touched: Vec::new(),
        })
    }

    pub fn to_core(&self) -> Result<GeometricAttention, String> {
        let mut core = GeometricAttention::from_f32(
            self.vocab,
            self.dv,
            self.factors,
            self.norm_bits,
            &self.wv,
            &self.wo,
        )?;
        core.use_relu = self.use_relu;
        Ok(core)
    }

    pub fn final_token_correct(&self, tokens: &[u32]) -> bool {
        if tokens.len() < 2 {
            return false;
        }
        let core = match self.to_core() {
            Ok(c) => c,
            Err(_) => return false,
        };
        let logits = core.forward_i32(&tokens[..tokens.len() - 1]);
        let mut best = 0usize;
        for (i, &v) in logits.iter().enumerate() {
            if v > logits[best] {
                best = i;
            }
        }
        best == (tokens[tokens.len() - 1] as usize).min(self.vocab - 1)
    }

    fn accumulate(&mut self, tokens: &[u32]) -> f32 {
        let n = tokens.len();
        if n < 2 {
            return 0.0;
        }
        let (qv, sv) = quantize_codes(&self.wv, self.vocab, self.dv);
        let (qo, so) = quantize_codes(&self.wo, self.vocab, self.dv);
        let cbase = self.n_addr * self.dv;
        let need = cbase + self.n_addr;
        let inv = 1.0f32 / (n - 1) as f32;

        // Reusable buffers. The state is `O(n_addr · dv)`, so a fresh allocation per sequence would
        // dominate at 14,400 addresses. Only touched buckets are ever non-zero, and they are the only
        // thing cleared.
        if self.scratch.len() != need {
            self.scratch = vec![0f32; need];
            self.dscratch = vec![0f32; need];
            self.touched.clear();
        }
        for k in 0..self.touched.len() {
            let a = self.touched[k] as usize;
            let base = a * self.dv;
            for j in 0..self.dv {
                self.scratch[base + j] = 0.0;
                self.dscratch[base + j] = 0.0;
            }
            self.scratch[cbase + a] = 0.0;
        }
        self.touched.clear();

        // Forward. Only what each read sees is cached: the addressed bucket and its write count.
        let mut reads: Vec<(Vec<f32>, u32)> = Vec::with_capacity(n - 1);
        for i in 0..(n - 1) {
            if i > 0 {
                let p = (tokens[i - 1] as usize).min(self.vocab - 1);
                let t = (tokens[i] as usize).min(self.vocab - 1);
                let a = self.addr[p] as usize;
                let base = a * self.dv;
                if self.scratch[cbase + a] == 0.0 {
                    self.touched.push(a as u32);
                }
                for j in 0..self.dv {
                    self.scratch[base + j] += qv[t * self.dv + j] as f32 * sv[t];
                }
                self.scratch[cbase + a] += 1.0;
            }
            let q = (tokens[i] as usize).min(self.vocab - 1);
            let a = self.addr[q] as usize;
            let base = a * self.dv;
            let mut bucket = vec![0f32; self.dv];
            bucket.copy_from_slice(&self.scratch[base..base + self.dv]);
            reads.push((bucket, self.scratch[cbase + a].max(0.0) as u32));
        }

        // Backward. The state is a pure accumulator, so a write's gradient is the sum of the read
        // gradients at its bucket over *later* steps: a suffix sum, accumulated in reverse.
        let mut loss = 0f64;
        for i in (0..(n - 1)).rev() {
            let q = (tokens[i] as usize).min(self.vocab - 1);
            let a = self.addr[q] as usize;
            let base = a * self.dv;
            let count = reads[i].1;
            let _ = count;
            let mut m = 0f32;
            for j in 0..self.dv {
                m = m.max(reads[i].0[j].abs());
            }
            let shift = if self.norm_bits > 0 && m >= 1.0 {
                (32 - (m as u32).leading_zeros()).saturating_sub(self.norm_bits)
            } else {
                0
            };
            let dec = 1.0f32 / (1u64 << shift) as f32;

            let mut num = vec![0f32; self.dv];
            for j in 0..self.dv {
                num[j] = if self.use_relu {
                    (reads[i].0[j] * dec).floor().max(0.0)
                } else {
                    (reads[i].0[j] * dec).floor()
                };
            }
            let mut logits = vec![0f32; self.vocab];
            for (r, slot) in logits.iter_mut().enumerate() {
                let mut acc = 0f32;
                for j in 0..self.dv {
                    acc += qo[r * self.dv + j] as f32 * num[j];
                }
                *slot = acc * so[r];
            }
            let mut p = softmax_f32(&logits);
            let target = (tokens[i + 1] as usize).min(self.vocab - 1);
            loss += -(p[target].max(1e-9)).ln() as f64;
            p[target] -= 1.0;

            let mut dnum = vec![0f32; self.dv];
            for r in 0..self.vocab {
                let g = p[r] * inv;
                if g == 0.0 {
                    continue;
                }
                let rbase = r * self.dv;
                let sr = so[r];
                for j in 0..self.dv {
                    self.gwo[rbase + j] += g * num[j];
                    dnum[j] += g * (qo[rbase + j] as f32 * sr);
                }
            }
            // Read gradient. The shift is treated as a constant (STE through the bit scan), and the
            // bucket is registered so a later sequence clears it.
            self.touched.push(a as u32);
            for j in 0..self.dv {
                let mask = if self.use_relu {
                    if num[j] > 0.0 {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    1.0
                };
                self.dscratch[base + j] += dnum[j] * mask * dec;
            }

            // Distribute the accumulated bucket gradient to this step's write.
            if i > 0 {
                let prev = (tokens[i - 1] as usize).min(self.vocab - 1);
                let t = (tokens[i] as usize).min(self.vocab - 1);
                let wa = self.addr[prev] as usize;
                let wbase = wa * self.dv;
                for j in 0..self.dv {
                    self.gwv[t * self.dv + j] += self.dscratch[wbase + j] * inv;
                }
            }
        }
        (loss / (n - 1) as f64) as f32
    }

    fn zero_grads(&mut self) {
        for g in self.gwv.iter_mut().chain(&mut self.gwo) {
            *g = 0.0;
        }
    }

    fn normalize_and_clip(&mut self, batch: usize) {
        let scale = 1.0f32 / batch.max(1) as f32;
        let mut sumsq = 0f64;
        for g in self.gwv.iter_mut().chain(&mut self.gwo) {
            *g *= scale;
            sumsq += (*g as f64) * (*g as f64);
        }
        if self.cfg.grad_clip > 0.0 {
            let norm = libm::sqrt(sumsq);
            let clip = self.cfg.grad_clip as f64;
            if norm > clip && norm > 0.0 {
                let s = (clip / norm) as f32;
                for g in self.gwv.iter_mut().chain(&mut self.gwo) {
                    *g *= s;
                }
            }
        }
    }

    fn apply_adam(&mut self) {
        self.step += 1;
        let step = self.step;
        let cfg = self.cfg.clone();
        adam_update(
            &mut self.wv,
            &self.gwv,
            &mut self.mwv,
            &mut self.vwv,
            &cfg,
            step,
        );
        adam_update(
            &mut self.wo,
            &self.gwo,
            &mut self.mwo,
            &mut self.vwo,
            &cfg,
            step,
        );
    }

    pub fn train_batch(&mut self, batch: &[Vec<u32>]) -> f32 {
        if batch.is_empty() {
            return 0.0;
        }
        self.zero_grads();
        let mut total = 0f64;
        for seq in batch {
            total += self.accumulate(seq) as f64;
        }
        self.normalize_and_clip(batch.len());
        self.apply_adam();
        (total / batch.len() as f64) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_geometric::learner::LowBitAttentionTrainer;

    fn random_attention(vocab: usize, dv: usize, factors: usize, seed: u64) -> GeometricAttention {
        let mut st = seed | 1;
        let mut nf = |n: usize| -> Vec<f32> {
            (0..n)
                .map(|_| {
                    st ^= st << 13;
                    st ^= st >> 7;
                    st ^= st << 17;
                    let u = ((st >> 11) as f64 / (1u64 << 53) as f64) as f32;
                    u * 2.0 - 1.0
                })
                .collect()
        };
        GeometricAttention::from_f32(vocab, dv, factors, 0, &nf(vocab * dv), &nf(vocab * dv))
            .expect("build")
    }

    #[test]
    fn integer_path_is_exactly_the_float_reference() {
        let a = random_attention(17, 7, 1, 0xABCD_EF01);
        for tokens in [
            vec![0u32, 3, 7, 1, 16, 2],
            vec![5, 5, 5, 5],
            vec![1, 2],
            vec![9],
            vec![],
        ] {
            let got = a.forward_i32(&tokens);
            let want = a.forward_reference_f64(&tokens);
            assert_eq!(got.len(), a.vocab);
            for j in 0..a.vocab {
                assert_eq!(
                    got[j] as f64, want[j],
                    "integer path diverged at {j} for {tokens:?}"
                );
            }
        }
    }

    #[test]
    fn addresses_are_distinct_for_a_small_alphabet() {
        // Two factors give 14,400 buckets, so a 64-token alphabet must be collision-free, which is
        // the property that makes the read interference-free.
        let addrs = build_addresses(64, 2);
        let mut seen = std::collections::HashSet::new();
        for a in &addrs {
            assert!(*a < ADDR_SPACE_TWO as u32);
            seen.insert(*a);
        }
        assert_eq!(seen.len(), 64, "expected no address collisions");
    }

    /// Context repetition: a random token run `R`, then `R` again. The second copy can only be
    /// predicted by remembering `(prev → next)` from the first, which is exactly what this memory
    /// stores. The first copy is unpredictable by construction and is noise in the loss.
    fn repeat_batch(seed: u64, n: usize, k: usize, alphabet: usize) -> Vec<Vec<u32>> {
        let mut st = seed | 1;
        (0..n)
            .map(|_| {
                let mut r = Vec::with_capacity(k);
                for _ in 0..k {
                    st ^= st << 13;
                    st ^= st >> 7;
                    st ^= st << 17;
                    r.push((st % alphabet as u64) as u32);
                }
                let mut seq = r.clone();
                seq.extend_from_slice(&r);
                seq
            })
            .collect()
    }

    const ALPHABET: usize = 32;
    const VOCAB: usize = 32;

    fn geometric_repeat(k: usize, steps: usize, seed: u64, norm_bits: u32, dv: usize) -> f32 {
        let train = repeat_batch(0xA5A5_1234, 64, k, ALPHABET);
        let held = repeat_batch(0x0BAD_F00D, 64, k, ALPHABET);
        let mut t = GeometricAttentionTrainer::new(VOCAB, dv, 2, norm_bits, seed).expect("build");
        t.cfg.lr = 0.05;
        for _ in 0..steps {
            t.train_batch(&train);
        }
        let hits = held.iter().filter(|s| t.final_token_correct(s)).count();
        hits as f32 / held.len() as f32
    }

    fn linear_repeat(k: usize, steps: usize, seed: u64) -> f32 {
        let train = repeat_batch(0xA5A5_1234, 64, k, ALPHABET);
        let held = repeat_batch(0x0BAD_F00D, 64, k, ALPHABET);
        let mut t = LowBitAttentionTrainer::new(VOCAB, 120, 32, 0, seed).expect("build");
        t.cfg.lr = 0.02;
        for _ in 0..steps {
            t.train_batch(&train);
        }
        let hits = held.iter().filter(|s| t.final_token_correct(s)).count();
        hits as f32 / held.len() as f32
    }

    #[test]
    fn exact_addressing_beats_matched_filter_as_context_grows() {
        for k in [4usize, 8, 16] {
            let g = geometric_repeat(k, 900, 2026_0919, 6, 32);
            let l = linear_repeat(k, 900, 2026_0919);
            eprintln!("context={k:>3} geometric={g:.2} linear={l:.2}");
            assert!(
                g > l,
                "exact addressing must beat the matched filter at context {k}: {g:.2} vs {l:.2}"
            );
        }
    }

    /// The context-16 ceiling is a *first-order* address collision, not capacity or resolution.
    /// Controlled: the same K and alphabet, differing only in whether the queried token's bucket
    /// receives two different successors. Measured 0.44 clean vs 0.14 duplicated-key.
    #[test]
    fn a_duplicated_queried_key_breaks_first_order_addressing() {
        let k = 16usize;
        let build = |seed: u64, n: usize, force_dup: bool| -> Vec<Vec<u32>> {
            let mut st = seed | 1;
            (0..n)
                .map(|_| {
                    let mut r: Vec<u32> = (0..k)
                        .map(|_| {
                            st ^= st << 13;
                            st ^= st >> 7;
                            st ^= st << 17;
                            (st % ALPHABET as u64) as u32
                        })
                        .collect();
                    if force_dup {
                        // The queried token (index K-2) also appears at index 0 with a different
                        // successor, so its bucket receives two values.
                        r[k - 2] = r[0];
                    }
                    let mut seq = r.clone();
                    seq.extend_from_slice(&r);
                    seq
                })
                .collect()
        };
        let train = build(0xA5A5_1234, 64, false);
        let mut t = GeometricAttentionTrainer::new(VOCAB, 128, 2, 6, 2026_0919).expect("build");
        t.cfg.lr = 0.05;
        for _ in 0..900 {
            t.train_batch(&train);
        }
        let score = |t: &GeometricAttentionTrainer, seqs: &[Vec<u32>]| {
            let hits = seqs.iter().filter(|s| t.final_token_correct(s)).count();
            hits as f32 / seqs.len() as f32
        };
        let clean = score(&t, &build(0x0BAD_F00D, 64, false));
        let ambiguous = score(&t, &build(0x0BAD_F00D, 64, true));
        eprintln!("ambiguity clean={clean:.2} duplicated-key={ambiguous:.2}");
        assert!(
            clean > ambiguous,
            "a duplicated queried key must hurt: {clean:.2} vs {ambiguous:.2}"
        );
    }

    /// With a sign-preserving read, value/output resolution scales the mechanism. (The earlier
    /// `dv` sweep that showed no effect was confounded by the sign-destroying `relu`.)
    #[test]
    fn resolution_scales_the_mechanism() {
        let narrow = geometric_repeat(8, 900, 2026_0919, 6, 32);
        let wide = geometric_repeat(8, 900, 2026_0919, 6, 256);
        eprintln!("resolution dv=32 {narrow:.2} vs dv=256 {wide:.2}");
        assert!(
            wide > narrow + 0.1,
            "value/output resolution must scale the mechanism: {narrow:.2} vs {wide:.2}"
        );
    }

    /// Is the ceiling caused by the `relu` after the read discarding the retrieved value's sign
    /// half? With exact addressing the selection is already the nonlinearity.
    #[test]
    fn selection_is_the_nonlinearity_not_the_read() {
        let k = 8usize;
        let train = repeat_batch(0xA5A5_1234, 64, k, ALPHABET);
        let held = repeat_batch(0x0BAD_F00D, 64, k, ALPHABET);
        let run = |use_relu: bool| {
            let mut t = GeometricAttentionTrainer::new(VOCAB, 32, 2, 6, 2026_0919).expect("build");
            t.cfg.lr = 0.05;
            t.use_relu = use_relu;
            for _ in 0..900 {
                t.train_batch(&train);
            }
            let hits = held.iter().filter(|s| t.final_token_correct(s)).count();
            hits as f32 / held.len() as f32
        };
        let with = run(true);
        let without = run(false);
        eprintln!("read relu={with:.2} no-relu={without:.2}");
        assert!(
            without > with + 0.1,
            "the sign-destroying relu after an exact-address read must hurt: {with:.2} vs {without:.2}"
        );
    }

    #[test]
    fn copies_a_repeated_context() {
        let acc4 = geometric_repeat(4, 900, 2026_0919, 6, 32);
        let acc8 = geometric_repeat(8, 900, 2026_0919, 6, 32);
        assert!(
            acc4 >= 0.8 && acc8 >= 0.5,
            "exact addressed memory must copy a repeated context, got {acc4:.2} / {acc8:.2}"
        );
    }

    #[test]
    fn gradients_reach_both_tables() {
        let mut t = GeometricAttentionTrainer::new(VOCAB, 16, 2, 0, 3).expect("build");
        t.zero_grads();
        for seq in repeat_batch(11, 4, 6, ALPHABET) {
            t.accumulate(&seq);
        }
        let nz = |g: &[f32]| g.iter().any(|v| v.abs() > 0.0);
        assert!(nz(&t.gwv), "no value gradient");
        assert!(nz(&t.gwo), "no output gradient");
    }
}
