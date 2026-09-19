//! Geometric attention: an exactly-addressed associative memory over the 2I/H4 address space.
//!
//! # The mechanism
//!
//! Linear attention stores *pairs* and reads them with a soft matched filter:
//! `y = Σ_s (q·k_s) v_s`. Its limit is cross-talk — an unmatched pair contributes
//! `Θ(√dk)` of noise, so the signal-to-noise ratio falls as `√(dk/N)`.
//!
//! This core stores the same pairs but reads them by **exact address**:
//!
//! ```text
//!   write:  S[address(word)] += value(t)     word = the ordered context ending at t-1
//!   read:   y                = norm(S[address(word)])   word = the ordered context ending at the query
//! ```
//!
//! Different addresses do not interfere, so a word whose address is unique is retrieved with **zero
//! cross-talk**, at `O(dv)` per token instead of `O(dk·dv)`, and with no multiplier anywhere: the
//! address is a table read and index arithmetic, the update is an add, the read is a table read.
//!
//! # The word is ordered, and that is the point
//!
//! `order = 1` addresses by one token. `order = 2` addresses by the **ordered pair** of the last two,
//! giving `120² = 14,400` ordered addresses, and `(a,b) ≠ (b,a)` by construction.
//!
//! Order is not cosmetic. Measured (`order = 1`): a duplicated queried key — the same token appearing
//! twice with two different successors — costs **3× accuracy** (0.44 clean vs 0.14 duplicated), because
//! a first-order address accumulates several successors into one bucket. This is the same distinction
//! W33 supplies in exact finite form (`PLPL = LPLP`, `Ω = LP − PL`, `ker(Ω)` order-insensitive) and the
//! project's own `r(i,j) = inverse(g_i)·g_j`: *the same operation multiset can act differently, so
//! order carries information a set or a commutative hash cannot.*
//!
//! # What this is not
//!
//! Still a synthetic-task measurement, not a language model. The address assignment is fixed, not
//! learned. Capacity is `n_addr = 120^order`; dense buffers make `order ≥ 3` (1.7M+ addresses) a
//! sparse-store problem. The `dv` scaling, the read activation result and the negative controls are in
//! `docs/evidence/native_geometric_geometric_attention_2026-09-19.txt`.

#![forbid(unsafe_code)]

use super::group_table::GROUP_ORDER;
use super::lowbit::TernaryLinear;
use super::lowbit_core::{adam_update, quantize_codes, softmax_f32, xorshift_unit, TrainConfig};

/// Radix of the word: one element per position is an element of 2I.
pub const RADIX: usize = GROUP_ORDER;

/// Number of address buckets for a word length.
pub fn address_space(order: usize) -> usize {
    match order {
        0 | 1 => RADIX,
        n => RADIX.pow(n as u32),
    }
}

/// The fixed element of `2I` assigned to each token.
///
/// Injective for a vocabulary of at most 120, which is what makes an ordered word a unique address at
/// this size. A larger vocabulary needs an injective learned assignment over the 120 roots (the
/// project's `nearest_h4_root` over learned embeddings) rather than a wider hash.
pub fn element_table(vocab: usize) -> Vec<u16> {
    (0..vocab).map(|t| (t % RADIX) as u16).collect()
}

/// The address of an ordered word: positional radix composition, order-preserving by construction.
#[inline]
pub fn word_address(elements: &[u16], order: usize) -> usize {
    let mut a = 0usize;
    for k in 0..order {
        a = a * RADIX + elements[k] as usize;
    }
    a
}

/// Serving form of the geometric attention core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometricAttention {
    pub vocab: usize,
    /// Value width.
    pub dv: usize,
    /// Word length in tokens. `1` addresses by one token, `2` by the ordered pair.
    pub order: usize,
    pub n_addr: usize,
    /// Fixed `2I` element per token.
    pub elements: Vec<u16>,
    /// Values, `vocab x dv`, ternary with a per-token power-of-two magnitude.
    pub w_v: TernaryLinear,
    /// Output map, `vocab x dv`.
    pub w_o: TernaryLinear,
    /// Target readout magnitude in bits for the power-of-two normalisation.
    pub norm_bits: u32,
    /// Apply `relu` to the read. With exact addressing the *selection* is the nonlinearity, and a
    /// `relu` after the read discards the retrieved value's sign half.
    pub use_relu: bool,
}

impl GeometricAttention {
    pub fn from_f32(
        vocab: usize,
        dv: usize,
        order: usize,
        norm_bits: u32,
        wv: &[f32],
        wo: &[f32],
    ) -> Result<Self, String> {
        if vocab == 0 || dv == 0 {
            return Err("vocab and dv must be non-zero".into());
        }
        if order == 0 || order > 2 {
            return Err("order must be 1 or 2 (dense buffers)".into());
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
            order,
            n_addr: address_space(order),
            elements: element_table(vocab),
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

    /// The address of a word given as tokens.
    #[inline]
    fn address(&self, ctx: &[u32]) -> usize {
        let mut a = 0usize;
        for k in 0..self.order {
            let t = (ctx[k] as usize).min(self.vocab - 1);
            a = a * RADIX + self.elements[t] as usize;
        }
        a
    }

    /// Write the word `ctx` (the ordered context ending at the predecessor) with `value(token)`.
    /// Adds only; one selected bucket.
    pub fn observe(&self, s: &mut [i32], ctx: &[u32], token: u32) {
        let t = (token as usize).min(self.vocab - 1);
        let a = self.address(ctx);
        let base = a * self.dv;
        let vshift = self.w_v.shift(t);
        for j in 0..self.dv {
            s[base + j] += self.w_v.weight(t, j) << vshift;
        }
        let cbase = self.n_addr * self.dv;
        s[cbase + a] += 1;
    }

    /// Read the bucket addressed by `ctx`, normalised to a power-of-two magnitude.
    pub fn logits(&self, s: &[i32], ctx: &[u32]) -> Vec<i32> {
        let a = self.address(ctx);
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

    /// Ingest a prompt and read with the final word.
    pub fn forward_i32(&self, tokens: &[u32]) -> Vec<i32> {
        let mut s = self.initial_state();
        self.ingest_all(&mut s, tokens);
        self.logits(&s, &self.tail_word(tokens))
    }

    /// The write side of the recurrence: for each position `i >= order`, write the word ending at
    /// `i-1` with the value of token `i`.
    fn ingest_all(&self, s: &mut [i32], tokens: &[u32]) {
        let order = self.order;
        for i in order..tokens.len() {
            self.observe(s, &tokens[i - order..i], tokens[i]);
        }
    }

    /// The word ending at the final token (padded with zero if the prompt is shorter than `order`).
    fn tail_word(&self, tokens: &[u32]) -> Vec<u32> {
        let len = tokens.len();
        let mut w = Vec::with_capacity(self.order);
        for k in 0..self.order {
            let idx = len + k - self.order;
            w.push(tokens.get(idx).copied().unwrap_or(0));
        }
        w
    }

    /// The same computation in `f64`, for verifying the integer path.
    pub fn forward_reference_f64(&self, tokens: &[u32]) -> Vec<f64> {
        let mut s = vec![0f64; self.state_len()];
        let cbase = self.n_addr * self.dv;
        for i in self.order..tokens.len() {
            let a = self.address(&tokens[i - self.order..i]);
            let t = (tokens[i] as usize).min(self.vocab - 1);
            let base = a * self.dv;
            let vscale = (1u64 << self.w_v.shift(t)) as f64;
            for j in 0..self.dv {
                s[base + j] += self.w_v.weight(t, j) as f64 * vscale;
            }
            s[cbase + a] += 1.0;
        }
        let ctx = self.tail_word(tokens);
        let a = self.address(&ctx);
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
            if self.use_relu && num[j] < 0.0 {
                num[j] = 0.0;
            }
        }
        self.w_o.forward_reference(&num)
    }

    pub fn weight_bytes(&self) -> usize {
        self.w_v.weight_bytes() + self.w_o.weight_bytes()
    }

    /// Mean next-token cross-entropy under the quantised weights (offline only).
    pub fn sequence_loss(&self, tokens: &[u32]) -> f32 {
        let n = tokens.len();
        if n < 2 {
            return 0.0;
        }
        let order = self.order;
        let cbase = self.n_addr * self.dv;
        let mut s = vec![0f32; self.state_len()];
        let preds = n - 1;
        let mut total = 0f64;
        for i in 0..preds {
            if i >= order {
                let a = self.address(&tokens[i - order..i]);
                let t = (tokens[i] as usize).min(self.vocab - 1);
                let base = a * self.dv;
                let vscale = (1u64 << self.w_v.shift(t)) as f32;
                for j in 0..self.dv {
                    s[base + j] += self.w_v.weight(t, j) as f32 * vscale;
                }
                s[cbase + a] += 1.0;
            }
            let ctx: Vec<u32> = (0..order)
                .map(|k| {
                    let idx = i + k + 1 - order;
                    tokens.get(idx).copied().unwrap_or(0)
                })
                .collect();
            let a = self.address(&ctx);
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
                let v = (s[base + j] / div).floor();
                num[j] = if self.use_relu { v.max(0.0) } else { v };
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
// Offline training. Only the value and output tables are learned; addresses are fixed.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct GeometricAttentionTrainer {
    pub vocab: usize,
    pub dv: usize,
    pub order: usize,
    pub n_addr: usize,
    pub elements: Vec<u16>,
    pub norm_bits: u32,
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
    scratch: Vec<f32>,
    dscratch: Vec<f32>,
    touched: Vec<u32>,
}

impl GeometricAttentionTrainer {
    pub fn new(
        vocab: usize,
        dv: usize,
        order: usize,
        norm_bits: u32,
        seed: u64,
    ) -> Result<Self, String> {
        if vocab == 0 || dv == 0 {
            return Err("vocab and dv must be non-zero".into());
        }
        if order == 0 || order > 2 {
            return Err("order must be 1 or 2 (dense buffers)".into());
        }
        let mut st = seed ^ 0x9E37_79B9_7F4A_7C15;
        if st == 0 {
            st = 0x1234_5678_9ABC_DEF0;
        }
        let mut fill = |n: usize| -> Vec<f32> { (0..n).map(|_| xorshift_unit(&mut st)).collect() };
        Ok(Self {
            vocab,
            dv,
            order,
            n_addr: address_space(order),
            elements: element_table(vocab),
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

    #[inline]
    fn address(&self, ctx: &[u32]) -> usize {
        let mut a = 0usize;
        for k in 0..self.order {
            let t = (ctx[k] as usize).min(self.vocab - 1);
            a = a * RADIX + self.elements[t] as usize;
        }
        a
    }

    /// The word ending at position `i` (padded with zero on the left).
    #[inline]
    fn word_at(&self, tokens: &[u32], i: usize) -> Vec<u32> {
        (0..self.order)
            .map(|k| {
                let idx = i + k + 1 - self.order;
                tokens.get(idx).copied().unwrap_or(0)
            })
            .collect()
    }

    pub fn to_core(&self) -> Result<GeometricAttention, String> {
        let mut core = GeometricAttention::from_f32(
            self.vocab,
            self.dv,
            self.order,
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
        let order = self.order;
        let (qv, sv) = quantize_codes(&self.wv, self.vocab, self.dv);
        let (qo, so) = quantize_codes(&self.wo, self.vocab, self.dv);
        let cbase = self.n_addr * self.dv;
        let need = cbase + self.n_addr;
        let inv = 1.0f32 / (n - 1) as f32;

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

        // Forward. Cache what each read sees: the addressed bucket, its magnitude and its count.
        let mut reads: Vec<(Vec<f32>, u32)> = Vec::with_capacity(n - 1);
        for i in 0..(n - 1) {
            if i >= order {
                let a = self.address(&tokens[i - order..i]);
                let t = (tokens[i] as usize).min(self.vocab - 1);
                let base = a * self.dv;
                if self.scratch[cbase + a] == 0.0 {
                    self.touched.push(a as u32);
                }
                for j in 0..self.dv {
                    self.scratch[base + j] += qv[t * self.dv + j] as f32 * sv[t];
                }
                self.scratch[cbase + a] += 1.0;
            }
            let ctx = self.word_at(tokens, i);
            let a = self.address(&ctx);
            let base = a * self.dv;
            let mut bucket = vec![0f32; self.dv];
            bucket.copy_from_slice(&self.scratch[base..base + self.dv]);
            reads.push((bucket, self.scratch[cbase + a].max(0.0) as u32));
        }

        // Backward. The state is a pure accumulator, so a write's gradient is the sum of the read
        // gradients at its bucket over *later* steps: a suffix sum, accumulated in reverse.
        let mut loss = 0f64;
        for i in (0..(n - 1)).rev() {
            let ctx = self.word_at(tokens, i);
            let a = self.address(&ctx);
            let base = a * self.dv;
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
                let v = (reads[i].0[j] * dec).floor();
                num[j] = if self.use_relu { v.max(0.0) } else { v };
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
            // Read gradient. The shift is treated as a constant (STE through the bit scan).
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

            // Distribute the accumulated bucket gradient to the write made at this step.
            if i >= order {
                let wa = self.address(&tokens[i - order..i]);
                let wbase = wa * self.dv;
                let t = (tokens[i] as usize).min(self.vocab - 1);
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

    fn random_attention(vocab: usize, dv: usize, order: usize, seed: u64) -> GeometricAttention {
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
        GeometricAttention::from_f32(vocab, dv, order, 0, &nf(vocab * dv), &nf(vocab * dv))
            .expect("build")
    }

    #[test]
    fn integer_path_is_exactly_the_float_reference() {
        for order in [1usize, 2] {
            let a = random_attention(17, 7, order, 0xABCD_EF01);
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
                        "order {order}: integer path diverged at {j} for {tokens:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn ordered_words_are_distinct_and_order_sensitive() {
        // Injective elements up to 120 mean an ordered pair is a unique address, and (a,b) != (b,a).
        let elems = element_table(64);
        let mut seen = std::collections::HashSet::new();
        for &a in &elems {
            for &b in &elems {
                assert!(seen.insert(word_address(&[a, b], 2)));
            }
        }
        assert_eq!(seen.len(), 64 * 64);
        assert_ne!(word_address(&[1, 2], 2), word_address(&[2, 1], 2));
        assert!(word_address(&[1, 2], 2) < address_space(2));
    }

    #[test]
    fn gradients_reach_both_tables() {
        let mut t = GeometricAttentionTrainer::new(32, 16, 2, 0, 3).expect("build");
        t.zero_grads();
        for seq in repeat_batch(11, 4, 6, 32) {
            t.accumulate(&seq);
        }
        let nz = |g: &[f32]| g.iter().any(|v| v.abs() > 0.0);
        assert!(nz(&t.gwv), "no value gradient");
        assert!(nz(&t.gwo), "no output gradient");
    }

    /// Context repetition: a random token run `R`, then `R` again. The second copy can only be
    /// predicted from memory of the words in the first.
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

    fn geometric_repeat(k: usize, steps: usize, seed: u64, dv: usize, order: usize) -> f32 {
        let train = repeat_batch(0xA5A5_1234, 64, k, ALPHABET);
        let held = repeat_batch(0x0BAD_F00D, 64, k, ALPHABET);
        let mut t = GeometricAttentionTrainer::new(VOCAB, dv, order, 6, seed).expect("build");
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
            let g = geometric_repeat(k, 900, 2026_0919, 128, 2);
            let l = linear_repeat(k, 900, 2026_0919);
            eprintln!("context={k:>3} geometric={g:.2} linear={l:.2}");
            assert!(
                g > l,
                "exact addressing must beat the matched filter at context {k}: {g:.2} vs {l:.2}"
            );
        }
    }

    /// The falsifiable prediction from the previous round: a duplicated queried key costs 3x accuracy
    /// under a first-order address, and an **ordered pair** address should recover it.
    #[test]
    fn ordered_words_recover_the_duplicated_key_collapse() {
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
                        r[k - 2] = r[0];
                    }
                    let mut seq = r.clone();
                    seq.extend_from_slice(&r);
                    seq
                })
                .collect()
        };
        let train = build(0xA5A5_1234, 64, false);
        let run = |order: usize| {
            let mut t =
                GeometricAttentionTrainer::new(VOCAB, 128, order, 6, 2026_0919).expect("build");
            t.cfg.lr = 0.05;
            for _ in 0..900 {
                t.train_batch(&train);
            }
            let score = |seqs: &[Vec<u32>]| {
                let hits = seqs.iter().filter(|s| t.final_token_correct(s)).count();
                hits as f32 / seqs.len() as f32
            };
            let clean = score(&build(0x0BAD_F00D, 64, false));
            let dup = score(&build(0x0BAD_F00D, 64, true));
            (clean, dup)
        };
        let (c1, d1) = run(1);
        let (c2, d2) = run(2);
        eprintln!(
            "collision order=1 clean={c1:.2} dup={d1:.2} | order=2 clean={c2:.2} dup={d2:.2}"
        );
        assert!(
            d2 > d1,
            "an ordered-pair address must recover the duplicated-key collapse: {d1:.2} -> {d2:.2}"
        );
    }

    /// Likewise measured at `order = 1`, where the task has headroom; at `order = 2` both widths
    /// reach 1.00.
    #[test]
    fn resolution_scales_the_mechanism() {
        let narrow = geometric_repeat(8, 900, 2026_0919, 32, 1);
        let wide = geometric_repeat(8, 900, 2026_0919, 256, 1);
        eprintln!("resolution dv=32 {narrow:.2} vs dv=256 {wide:.2}");
        assert!(
            wide > narrow + 0.1,
            "value/output resolution must scale the mechanism: {narrow:.2} vs {wide:.2}"
        );
    }

    /// Ablations of the read must be run at `order = 1`: at `order = 2` this task saturates (1.00) and
    /// a `relu` would have no headroom to show its cost.
    #[test]
    fn selection_is_the_nonlinearity_not_the_read() {
        let k = 8usize;
        let train = repeat_batch(0xA5A5_1234, 64, k, ALPHABET);
        let held = repeat_batch(0x0BAD_F00D, 64, k, ALPHABET);
        let run = |use_relu: bool| {
            let mut t = GeometricAttentionTrainer::new(VOCAB, 32, 1, 6, 2026_0919).expect("build");
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

    /// The headline result: an ordered-word address copies a repeated context of 16 tokens, where a
    /// matched-filter memory scores zero.
    #[test]
    fn copies_a_repeated_context() {
        for k in [4usize, 8, 16] {
            let acc = geometric_repeat(k, 900, 2026_0919, 32, 2);
            assert!(
                acc >= 0.9,
                "an ordered-word address must copy a {k}-token repeated context, got {acc:.2}"
            );
        }
    }
}
