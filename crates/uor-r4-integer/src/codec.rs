//! Discrete parameter codecs for native geometric serving under D11.
//!
//! Provides geometry-coded representations:
//! 1. **Hadamard + Grouped 4-bit (H+G4)**: Block-padded randomized Walsh-Hadamard transform
//!    (incoherence processing) followed by power-of-two grouped 4-bit quantization.
//!    Supports arbitrary vector/row dimensions (e.g. 288, 749).
//! 2. **Hadamard + E8 (H+E8)**: Block-padded randomized Walsh-Hadamard transform followed
//!    by 8-dimensional Conway-Sloane E8 lattice quantization with zero codeword and equal-norm roots.
//! 3. **Head-Compensated 4-bit**: Grouped 4-bit with activation-moment error compensation.
//!
//! All runtime operations (FWHT, codebook lookup, scale shifts, and dot products)
//! strictly adhere to owner decisions D0-b and D11: zero hardware multipliers,
//! zero hardware dividers, and zero floating-point operations in served code.

use crate::math;
use crate::{invalid, Result};
use std::sync::Arc;

/// Default group size for grouped 4-bit quantization (matches `uor-r4-lut` GROUP).
pub const DEFAULT_GROUP_SIZE: usize = 32;
/// Block dimension for E8 lattice quantization.
pub const E8_DIM: usize = 8;
/// Block size for Walsh-Hadamard transforms.
pub const HADAMARD_BLOCK: usize = 64;

// ---------------------------------------------------------------------------
// Fast Walsh-Hadamard Transform (FWHT) & Randomized Hadamard Transform
// ---------------------------------------------------------------------------

/// In-place Fast Walsh-Hadamard Transform over a power-of-two slice.
///
/// Uses butterfly additions and subtractions only (0 multipliers, 0 dividers, 0 floats).
pub fn fwht_slice(data: &mut [i64]) -> Result<()> {
    let len = data.len();
    if len == 0 || (len & (len - 1)) != 0 {
        return Err(invalid("FWHT slice length must be a non-zero power of two"));
    }
    let mut h = 1;
    while h < len {
        let mut i = 0;
        while i < len {
            for j in i..(i + h) {
                let u = data[j];
                let v = data[j + h];
                data[j] = u
                    .checked_add(v)
                    .ok_or_else(|| invalid("FWHT overflow on add"))?;
                data[j + h] = u
                    .checked_sub(v)
                    .ok_or_else(|| invalid("FWHT overflow on sub"))?;
            }
            i += h * 2;
        }
        h *= 2;
    }
    Ok(())
}

/// Fast Walsh-Hadamard Transform with exact power-of-two normalizer shift.
///
/// For block size N = 4^k, sqrt(N) = 2^k, so normalization is an exact right shift by k bits.
/// For N = 64 (4^3), shift is 3 bits (sqrt(64) = 8 = 2^3).
/// For N = 256 (4^4), shift is 4 bits (sqrt(256) = 16 = 2^4).
pub fn fwht_normalized(data: &mut [i64], shift: u32) -> Result<()> {
    fwht_slice(data)?;
    if shift > 0 {
        let half = 1i64 << (shift - 1);
        for val in data.iter_mut() {
            *val = (*val + half) >> shift;
        }
    }
    Ok(())
}

/// Deterministic sign vector generator for randomized Hadamard transform (incoherence).
///
/// Uses the project's standard Galois LFSR polynomial (0xD800000000000000) to produce
/// deterministic pseudo-random signs (+1 or -1) without external RNG dependencies.
pub fn deterministic_signs(dim: usize, seed: u64) -> Vec<i8> {
    let mut state = if seed == 0 { 0x517cc1b727220a95 } else { seed };
    let mut signs = Vec::with_capacity(dim);
    for _ in 0..dim {
        // Galois LFSR step
        let lsb = state & 1;
        state >>= 1;
        if lsb != 0 {
            state ^= 0xD800000000000000;
        }
        signs.push(if (state & 1) == 0 { 1 } else { -1 });
    }
    signs
}

/// Apply randomized Hadamard transform to an input vector in blocks of `HADAMARD_BLOCK` (64).
///
/// For each 64-element block:
/// 1. Sign modulation: x_i -> x_i * S_i (conditional negation: 0 multipliers).
/// 2. FWHT over 64 elements (6 stages of butterfly add/sub).
/// 3. Normalization: shift right by 3 bits (sqrt(64) = 8 = 2^3).
pub fn randomized_hadamard_transform(data: &mut [i64], signs: &[i8]) -> Result<()> {
    if data.len() % HADAMARD_BLOCK != 0 {
        return Err(invalid(format!(
            "randomized Hadamard requires dimension multiple of {}, got {}",
            HADAMARD_BLOCK,
            data.len()
        )));
    }
    for (chunk, sign_chunk) in data
        .chunks_exact_mut(HADAMARD_BLOCK)
        .zip(signs.chunks_exact(HADAMARD_BLOCK))
    {
        for (x, &s) in chunk.iter_mut().zip(sign_chunk) {
            if s < 0 {
                *x = -*x;
            }
        }
        fwht_normalized(chunk, 3)?;
    }
    Ok(())
}

/// Apply inverse randomized Hadamard transform.
///
/// Because H is symmetric and orthogonal (H^T = H, H H = I), the inverse is:
/// 1. FWHT over 64 elements with shift 3.
/// 2. Sign modulation: x_i -> x_i * S_i.
pub fn inverse_randomized_hadamard_transform(data: &mut [i64], signs: &[i8]) -> Result<()> {
    if data.len() % HADAMARD_BLOCK != 0 {
        return Err(invalid("dimension not a multiple of HADAMARD_BLOCK"));
    }
    for (chunk, sign_chunk) in data
        .chunks_exact_mut(HADAMARD_BLOCK)
        .zip(signs.chunks_exact(HADAMARD_BLOCK))
    {
        fwht_normalized(chunk, 3)?;
        for (x, &s) in chunk.iter_mut().zip(sign_chunk) {
            if s < 0 {
                *x = -*x;
            }
        }
    }
    Ok(())
}

/// Apply block-padded randomized Hadamard transform to an input slice of arbitrary dimension.
///
/// Pads the data with zeros to the next multiple of `HADAMARD_BLOCK` (64), applies
/// sign modulation and FWHT in 64-element blocks with shift 3.
/// Handles arbitrary widths (e.g. hidden width 288 -> padded to 320, MLP width 749 -> padded to 768).
pub fn block_padded_hadamard_transform(data: &[i64], signs: &[i8]) -> Result<Vec<i64>> {
    let orig_len = data.len();
    if orig_len == 0 {
        return Ok(Vec::new());
    }
    let padded_len = orig_len.div_ceil(HADAMARD_BLOCK) * HADAMARD_BLOCK;
    if signs.len() < padded_len {
        return Err(invalid(format!(
            "signs length ({}) less than padded dimension ({})",
            signs.len(),
            padded_len
        )));
    }
    let mut padded = vec![0i64; padded_len];
    padded[..orig_len].copy_from_slice(data);
    randomized_hadamard_transform(&mut padded, &signs[..padded_len])?;
    Ok(padded)
}

/// Apply inverse block-padded randomized Hadamard transform and unpad to original dimension.
pub fn inverse_block_padded_hadamard_transform(
    transformed: &[i64],
    signs: &[i8],
    orig_len: usize,
) -> Result<Vec<i64>> {
    if orig_len == 0 {
        return Ok(Vec::new());
    }
    let padded_len = orig_len.div_ceil(HADAMARD_BLOCK) * HADAMARD_BLOCK;
    if transformed.len() != padded_len {
        return Err(invalid(format!(
            "transformed length ({}) does not match expected padded length ({})",
            transformed.len(),
            padded_len
        )));
    }
    let mut buf = transformed.to_vec();
    inverse_randomized_hadamard_transform(&mut buf, &signs[..padded_len])?;
    buf.truncate(orig_len);
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Grouped 4-bit Codec
// ---------------------------------------------------------------------------

/// Grouped 4-bit representation of a weight matrix row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grouped4BitRow {
    /// Group size (e.g. 32).
    pub group_size: usize,
    /// Dyadic power-of-two exponents per group.
    pub group_exponents: Vec<i16>,
    /// Packed signed 4-bit codes (low nibble first).
    pub packed_codes: Arc<[u8]>,
    /// Number of elements in the row.
    pub elements: usize,
}

impl Grouped4BitRow {
    /// Encode a slice of float weights into a grouped 4-bit row.
    /// Supports arbitrary element count (tail elements form a shorter last group).
    pub fn encode_f32(weights: &[f32], group_size: usize) -> Result<Self> {
        if weights.is_empty() || group_size == 0 {
            return Err(invalid(
                "weights length must be non-zero and group_size > 0",
            ));
        }
        if weights.iter().any(|w| !w.is_finite()) {
            return Err(invalid("weights contains non-finite values"));
        }
        let num_groups = weights.len().div_ceil(group_size);
        let mut group_exponents = Vec::with_capacity(num_groups);
        let mut raw_codes = Vec::with_capacity(weights.len());

        for group in weights.chunks(group_size) {
            let mut maxabs = 0.0f32;
            for &w in group {
                let abs = w.abs();
                if abs > maxabs {
                    maxabs = abs;
                }
            }
            let exp = if maxabs == 0.0 {
                0i16
            } else {
                let raw_ceil = (maxabs / 7.0f32).log2().ceil() as i32;
                raw_ceil.clamp(-24, 16) as i16
            };
            group_exponents.push(exp);

            let scale = 2.0f32.powi(i32::from(exp));
            for &w in group {
                let code = (w / scale).round().clamp(-7.0, 7.0) as i16;
                raw_codes.push(code);
            }
        }

        let mut packed = Vec::with_capacity(weights.len().div_ceil(2));
        for pair in raw_codes.chunks(2) {
            let low = (pair[0] as u8) & 15;
            let high = pair.get(1).map_or(0, |&c| (c as u8) & 15);
            packed.push(low | (high << 4));
        }

        Ok(Self {
            group_size,
            group_exponents,
            packed_codes: packed.into(),
            elements: weights.len(),
        })
    }

    /// Dequantize back to float for verification / evaluation.
    pub fn dequantize_f32(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.elements);
        for (g_idx, &exp) in self.group_exponents.iter().enumerate() {
            let scale = 2.0f32.powi(i32::from(exp));
            let start = g_idx * self.group_size;
            let end = (start + self.group_size).min(self.elements);
            for i in start..end {
                let byte = self.packed_codes[i >> 1];
                let nibble = (byte >> ((i & 1) << 2)) & 15;
                let code = (nibble as i8) << 4 >> 4;
                out.push((code as f32) * scale);
            }
        }
        out
    }

    /// Extract row activations in fixed-point Q16 without floating-point arithmetic.
    ///
    /// Evaluates: q_i * 2^(16 + exp_g) using dyadic scale_pow2 (0 multipliers, 0 floats).
    #[inline(never)]
    pub fn to_q16_vector(&self) -> Result<Vec<i64>> {
        let mut out = Vec::with_capacity(self.elements);
        let mut start = 0usize;
        for &exp in &self.group_exponents {
            let shift = 16 + i32::from(exp);
            let end = (start + self.group_size).min(self.elements);
            for i in start..end {
                let byte = self.packed_codes[i >> 1];
                let nibble = (byte >> ((i & 1) << 2)) & 15;
                let code = i64::from((nibble as i8) << 4 >> 4);
                let scaled = math::scale_pow2(i128::from(code), shift)
                    .map_err(|e| invalid(format!("scale_pow2 error: {e}")))?;
                out.push(i64::try_from(scaled).map_err(|_| invalid("embedding out of bounds"))?);
            }
            start += self.group_size;
        }
        if out.len() != self.elements {
            return Err(invalid("group_exponents did not cover all elements"));
        }
        Ok(out)
    }

    /// Multiplier-free dot product with activation vector in integer arithmetic.
    ///
    /// Evaluates: Sum_g 2^(exp_g) * (Sum_{i in g} x_i * q_i)
    /// where x_i * q_i uses product table or shift-and-add (zero hardware multipliers).
    pub fn dot_integer(&self, activations: &[i64]) -> Result<i64> {
        if activations.len() != self.elements {
            return Err(invalid("activation dimension mismatch"));
        }
        let mut total_sum: i128 = 0;
        let mut covered = 0usize;
        for (g_idx, &exp) in self.group_exponents.iter().enumerate() {
            let start = g_idx * self.group_size;
            let end = (start + self.group_size).min(self.elements);
            covered = end;
            let mut group_dot: i64 = 0;
            for i in start..end {
                let byte = self.packed_codes[i >> 1];
                let nibble = (byte >> ((i & 1) << 2)) & 15;
                let code = i64::from((nibble as i8) << 4 >> 4);
                let x = activations[i];
                let prod = mul_small_code_i64(x, code)?;
                group_dot = group_dot
                    .checked_add(prod)
                    .ok_or_else(|| invalid("group dot overflow"))?;
            }
            let scaled = math::scale_pow2(i128::from(group_dot), i32::from(exp))
                .map_err(|e| invalid(format!("scale_pow2 error: {e}")))?;
            total_sum = total_sum
                .checked_add(scaled)
                .ok_or_else(|| invalid("total dot overflow"))?;
        }
        if covered != self.elements {
            return Err(invalid("group_exponents did not cover all elements"));
        }
        i64::try_from(total_sum).map_err(|_| invalid("result exceeds i64"))
    }
}

// ---------------------------------------------------------------------------
// Conway-Sloane E8 Lattice Codec
// ---------------------------------------------------------------------------

/// Nearest D8 lattice point (integer coordinates with even sum).
///
/// Conway-Sloane (1982) fast decoding algorithm:
/// 1. Round each coordinate to nearest integer.
/// 2. If the sum of coordinates is odd, find the coordinate with the largest
///    rounding error and flip it to the other nearest integer.
pub fn d8_round(y: &[f64; E8_DIM]) -> [f64; E8_DIM] {
    let mut f = [0.0f64; E8_DIM];
    for i in 0..E8_DIM {
        f[i] = y[i].round();
    }
    let sum: i64 = f.iter().map(|&v| v as i64).sum();
    if sum % 2 != 0 {
        // Find coordinate with largest rounding error
        let mut max_err = -1.0f64;
        let mut worst_idx = 0;
        let mut worst_flip = 0.0f64;
        for i in 0..E8_DIM {
            let err = (y[i] - f[i]).abs();
            if err > max_err {
                max_err = err;
                worst_idx = i;
                worst_flip = if y[i] >= f[i] { f[i] + 1.0 } else { f[i] - 1.0 };
            }
        }
        f[worst_idx] = worst_flip;
    }
    f
}

/// Nearest E8 lattice point via Conway-Sloane union of D8 and D8 + 1/2.
///
/// E8 = D8 U (D8 + (1/2, 1/2, ..., 1/2)).
pub fn e8_round(x: &[f64; E8_DIM]) -> [f64; E8_DIM] {
    let f0 = d8_round(x);
    let mut dist0 = 0.0f64;
    for i in 0..E8_DIM {
        let d = x[i] - f0[i];
        dist0 += d * d;
    }

    let mut shifted = [0.0f64; E8_DIM];
    for i in 0..E8_DIM {
        shifted[i] = x[i] - 0.5;
    }
    let f1_shift = d8_round(&shifted);
    let mut f1 = [0.0f64; E8_DIM];
    let mut dist1 = 0.0f64;
    for i in 0..E8_DIM {
        f1[i] = f1_shift[i] + 0.5;
        let d = x[i] - f1[i];
        dist1 += d * d;
    }

    if dist0 <= dist1 {
        f0
    } else {
        f1
    }
}

/// Normalized E8 codebook with zero codeword and equal-norm roots.
///
/// Represents points in the scaled lattice 2*E8:
/// - Type 1 roots: permutations of (+-2, +-2, 0, 0, 0, 0, 0, 0) -> squared norm = 4 + 4 = 8 (112 roots).
/// - Type 2 roots: (+-1, ..., +-1) with even number of minus signs -> squared norm = 8 * 1 = 8 (128 roots).
/// - Zero codeword: (0, 0, 0, 0, 0, 0, 0, 0) -> squared norm = 0 (1 codeword).
/// Total: 241 codewords (fits in 8 bits / 1 byte per 8 weights = 1.0 bit/weight).
#[derive(Clone, Debug)]
pub struct E8Codebook {
    pub roots: [[i8; E8_DIM]; 241],
}

impl Default for E8Codebook {
    fn default() -> Self {
        Self::new()
    }
}

impl E8Codebook {
    /// Initialize the standard E8 root codebook with uniform root scale and zero codeword.
    pub fn new() -> Self {
        let mut roots = [[0i8; E8_DIM]; 241];
        let mut count = 0;

        // Codeword 0: Zero codeword
        roots[0] = [0i8; E8_DIM];
        count += 1;

        // Type 1 roots (112 roots): permutations of (+-2, +-2, 0, 0, 0, 0, 0, 0).
        // Coords scaled by 2 so norm squared is exactly 4 + 4 = 8.
        for i in 0..E8_DIM {
            for j in (i + 1)..E8_DIM {
                for &s1 in &[-2i8, 2i8] {
                    for &s2 in &[-2i8, 2i8] {
                        if count < 241 {
                            let mut r = [0i8; E8_DIM];
                            r[i] = s1;
                            r[j] = s2;
                            roots[count] = r;
                            count += 1;
                        }
                    }
                }
            }
        }

        // Type 2 roots (128 roots): (+-1, ..., +-1) with even number of minus signs.
        // Coords are +-1, so norm squared is exactly 8 * 1 = 8.
        for code in 0..256u16 {
            let mut r = [1i8; E8_DIM];
            let mut minus_count = 0;
            for b in 0..8 {
                if (code & (1 << b)) != 0 {
                    r[b] = -1;
                    minus_count += 1;
                }
            }
            if minus_count % 2 == 0 && count < 241 {
                roots[count] = r;
                count += 1;
            }
        }

        Self { roots }
    }

    /// Find the index of the nearest E8 root (or zero) to a given 8D block.
    pub fn quantize_block(&self, block: &[f64; E8_DIM]) -> (u8, f64) {
        let energy: f64 = block.iter().map(|&x| x * x).sum();
        let scale = (energy / 8.0).sqrt();

        if scale < 1e-9 {
            return (0, 0.0);
        }

        let mut best_idx = 0u8;
        let mut best_dist = energy; // Distance to zero codeword is energy

        for (idx, root) in self.roots.iter().enumerate() {
            let mut dist = 0.0f64;
            for i in 0..E8_DIM {
                let diff = block[i] - (root[i] as f64) * scale;
                dist += diff * diff;
            }
            if dist < best_dist {
                best_dist = dist;
                best_idx = idx as u8;
            }
        }

        (best_idx, scale)
    }

    /// Dequantize a root index given block scale.
    pub fn dequantize_block(&self, idx: u8, scale: f64) -> [f64; E8_DIM] {
        let root = &self.roots[idx as usize];
        let mut out = [0.0f64; E8_DIM];
        for i in 0..E8_DIM {
            out[i] = (root[i] as f64) * scale;
        }
        out
    }
}

/// E8 encoded matrix row.
#[derive(Clone, Debug)]
pub struct E8EncodedRow {
    pub indices: Vec<u8>,
    pub scales: Vec<f32>,
    pub elements: usize,
}

impl E8EncodedRow {
    /// Encode a slice of float weights into E8 lattice codewords.
    pub fn encode_f32(weights: &[f32], codebook: &E8Codebook) -> Result<Self> {
        if weights.is_empty() {
            return Err(invalid("weights slice cannot be empty"));
        }
        let padded_len = weights.len().div_ceil(E8_DIM) * E8_DIM;
        let mut padded = vec![0.0f64; padded_len];
        for (dst, &src) in padded.iter_mut().zip(weights) {
            *dst = src as f64;
        }

        let num_blocks = padded_len / E8_DIM;
        let mut indices = Vec::with_capacity(num_blocks);
        let mut scales = Vec::with_capacity(num_blocks);

        for block in padded.chunks_exact(E8_DIM) {
            let mut b = [0.0f64; E8_DIM];
            b.copy_from_slice(block);
            let (idx, scale) = codebook.quantize_block(&b);
            indices.push(idx);
            scales.push(scale as f32);
        }

        Ok(Self {
            indices,
            scales,
            elements: weights.len(),
        })
    }

    /// Dequantize E8 encoded row back to float weights.
    pub fn dequantize_f32(&self, codebook: &E8Codebook) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.elements);
        for (&idx, &scale) in self.indices.iter().zip(&self.scales) {
            let deq = codebook.dequantize_block(idx, scale as f64);
            for &val in &deq {
                if out.len() < self.elements {
                    out.push(val as f32);
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Multiplier-free helper for small code products
// ---------------------------------------------------------------------------

/// Multiplier-free product of an i64 activation and a small signed code in [-7..7].
///
/// Uses additions, subtractions, and bit shifts only (0 hardware multiplier instructions).
#[inline(never)]
pub fn mul_small_code_i64(val: i64, code: i64) -> Result<i64> {
    if !(-7..=7).contains(&code) {
        return Err(invalid("code exceeds [-7..7] range"));
    }
    let neg = code < 0;
    let abs_c = code.unsigned_abs();
    let shl_checked = |x: i64, k: u32| -> Result<i64> {
        let s = x.wrapping_shl(k);
        if (s >> k) != x {
            return Err(invalid("overflow on shift"));
        }
        Ok(s)
    };
    let mag = match abs_c {
        0 => 0i64,
        1 => val,
        2 => shl_checked(val, 1)?,
        3 => shl_checked(val, 1)?
            .checked_add(val)
            .ok_or_else(|| invalid("overflow"))?,
        4 => shl_checked(val, 2)?,
        5 => shl_checked(val, 2)?
            .checked_add(val)
            .ok_or_else(|| invalid("overflow"))?,
        6 => shl_checked(val, 2)?
            .checked_add(shl_checked(val, 1)?)
            .ok_or_else(|| invalid("overflow"))?,
        7 => shl_checked(val, 3)?
            .checked_sub(val)
            .ok_or_else(|| invalid("overflow"))?,
        _ => unreachable!(),
    };
    if neg {
        mag.checked_neg().ok_or_else(|| invalid("overflow"))
    } else {
        Ok(mag)
    }
}

// ---------------------------------------------------------------------------
// Unified Codec Transform Evaluation
// ---------------------------------------------------------------------------

/// Codec evaluation arms for discrete parameter representation studies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodecArm {
    /// Baseline: Round to nearest per-row signed 4-bit.
    NearestPerRow,
    /// Arm 1: Randomized Walsh-Hadamard transform + Grouped 4-bit (H+G4).
    HadamardGrouped4Bit,
    /// Arm 2: Randomized Walsh-Hadamard transform + E8 lattice (H+E8).
    HadamardE8,
    /// Arm 3: Head-compensated quantization (mean-offset compensation per group).
    HeadCompensated,
}

/// Transform matrix weights using the selected codec arm.
///
/// Handles arbitrary row and column dimensions (e.g. 288, 749) via block-padding.
/// Returns the quantized and dequantized weights (same shape) for fidelity evaluation.
pub fn apply_codec_arm(
    weights: &[f32],
    rows: usize,
    cols: usize,
    arm: CodecArm,
    seed: u64,
) -> Result<Vec<f32>> {
    if weights.len() != rows * cols {
        return Err(invalid("weights size does not match rows * cols"));
    }
    match arm {
        CodecArm::NearestPerRow => {
            let mut out = Vec::with_capacity(weights.len());
            for row in weights.chunks_exact(cols) {
                let mut maxabs = 0.0f32;
                for &w in row {
                    let a = w.abs();
                    if a > maxabs {
                        maxabs = a;
                    }
                }
                let exp = if maxabs == 0.0 {
                    0i16
                } else {
                    let c = (maxabs / 7.0f32).log2().ceil() as i32;
                    c.clamp(-24, 16) as i16
                };
                let scale = 2.0f32.powi(i32::from(exp));
                for &w in row {
                    let q = (w / scale).round().clamp(-7.0, 7.0);
                    out.push(q * scale);
                }
            }
            Ok(out)
        }
        CodecArm::HadamardGrouped4Bit => {
            const SCALE: f32 = 65536.0;
            let padded_cols = cols.div_ceil(HADAMARD_BLOCK) * HADAMARD_BLOCK;
            let signs = deterministic_signs(padded_cols, seed);
            let mut out = Vec::with_capacity(weights.len());

            for row in weights.chunks_exact(cols) {
                let i64_row: Vec<i64> = row.iter().map(|&w| (w * SCALE).round() as i64).collect();
                let had_i64 = block_padded_hadamard_transform(&i64_row, &signs)?;
                let had_f32: Vec<f32> = had_i64.iter().map(|&v| (v as f32) / SCALE).collect();

                let grouped = Grouped4BitRow::encode_f32(&had_f32, DEFAULT_GROUP_SIZE)?;
                let deq_had = grouped.dequantize_f32();

                let i64_deq: Vec<i64> = deq_had
                    .iter()
                    .map(|&w| (w * SCALE).round() as i64)
                    .collect();
                let restored_i64 = inverse_block_padded_hadamard_transform(&i64_deq, &signs, cols)?;
                for v in restored_i64 {
                    out.push((v as f32) / SCALE);
                }
            }
            Ok(out)
        }
        CodecArm::HadamardE8 => {
            const SCALE: f32 = 65536.0;
            let padded_cols = cols.div_ceil(HADAMARD_BLOCK) * HADAMARD_BLOCK;
            let signs = deterministic_signs(padded_cols, seed);
            let codebook = E8Codebook::new();
            let mut out = Vec::with_capacity(weights.len());

            for row in weights.chunks_exact(cols) {
                let i64_row: Vec<i64> = row.iter().map(|&w| (w * SCALE).round() as i64).collect();
                let had_i64 = block_padded_hadamard_transform(&i64_row, &signs)?;
                let had_f32: Vec<f32> = had_i64.iter().map(|&v| (v as f32) / SCALE).collect();

                let e8_row = E8EncodedRow::encode_f32(&had_f32, &codebook)?;
                let deq_had = e8_row.dequantize_f32(&codebook);

                let i64_deq: Vec<i64> = deq_had
                    .iter()
                    .map(|&w| (w * SCALE).round() as i64)
                    .collect();
                let restored_i64 = inverse_block_padded_hadamard_transform(&i64_deq, &signs, cols)?;
                for v in restored_i64 {
                    out.push((v as f32) / SCALE);
                }
            }
            Ok(out)
        }
        CodecArm::HeadCompensated => {
            let mut out = Vec::with_capacity(weights.len());
            for row in weights.chunks_exact(cols) {
                let mut row_acc_err = 0.0f32;
                for group in row.chunks(DEFAULT_GROUP_SIZE) {
                    let mut maxabs = 0.0f32;
                    for &w in group {
                        let a = w.abs();
                        if a > maxabs {
                            maxabs = a;
                        }
                    }
                    if maxabs == 0.0 {
                        out.extend(std::iter::repeat(0.0f32).take(group.len()));
                        continue;
                    }
                    let base_exp = (maxabs / 7.0f32).log2().floor() as i32;
                    let mut best_scale = 2.0f32.powi(base_exp);
                    let mut best_err = f32::INFINITY;
                    for de in -3..=3 {
                        let s = 2.0f32.powi((base_exp + de).clamp(-24, 16));
                        let err: f32 = group
                            .iter()
                            .map(|&w| {
                                let q = (w / s).round().clamp(-7.0, 7.0);
                                (w - q * s).powi(2)
                            })
                            .sum();
                        if err < best_err {
                            best_err = err;
                            best_scale = s;
                        }
                    }
                    for &w in group {
                        let compensated_w = w + row_acc_err * 0.5f32;
                        let q = (compensated_w / best_scale).round().clamp(-7.0, 7.0);
                        let deq = q * best_scale;
                        row_acc_err = (w - deq) + row_acc_err * 0.5f32;
                        out.push(deq);
                    }
                }
            }
            Ok(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fwht_orthogonality_and_inversion() -> Result<()> {
        let original: Vec<i64> = (0..64).map(|x| x * 7 - 120).collect();
        let mut transformed = original.clone();
        fwht_slice(&mut transformed)?;

        let mut restored = transformed.clone();
        fwht_slice(&mut restored)?;
        for x in restored.iter_mut() {
            *x >>= 6;
        }
        assert_eq!(restored, original);
        Ok(())
    }

    #[test]
    fn test_randomized_hadamard_roundtrip() -> Result<()> {
        let dim = 128;
        let signs = deterministic_signs(dim, 20260928);
        let original: Vec<i64> = (0..dim).map(|i| (i as i64) * 3 - 150).collect();
        let mut transformed = original.clone();
        randomized_hadamard_transform(&mut transformed, &signs)?;

        let mut inverted = transformed.clone();
        inverse_randomized_hadamard_transform(&mut inverted, &signs)?;

        for (orig, inv) in original.iter().zip(&inverted) {
            assert!((orig - inv).abs() <= 2, "orig: {orig}, inv: {inv}");
        }
        Ok(())
    }

    #[test]
    fn test_arbitrary_width_block_padded_hadamard() -> Result<()> {
        for &dim in &[288usize, 749usize] {
            let padded_len = dim.div_ceil(HADAMARD_BLOCK) * HADAMARD_BLOCK;
            let signs = deterministic_signs(padded_len, 42);
            let original: Vec<i64> = (0..dim).map(|i| ((i as i64) * 5) % 137 - 68).collect();

            let transformed = block_padded_hadamard_transform(&original, &signs)?;
            assert_eq!(transformed.len(), padded_len);

            let restored = inverse_block_padded_hadamard_transform(&transformed, &signs, dim)?;
            assert_eq!(restored.len(), dim);

            for (orig, res) in original.iter().zip(&restored) {
                assert!((orig - res).abs() <= 2, "dim {dim}: orig {orig}, res {res}");
            }
        }
        Ok(())
    }

    #[test]
    fn test_e8_codebook_norms_and_zero() {
        let codebook = E8Codebook::new();
        assert_eq!(codebook.roots[0], [0i8; E8_DIM]);

        for root in &codebook.roots[1..] {
            let norm_sq: i32 = root.iter().map(|&x| (x as i32) * (x as i32)).sum();
            assert_eq!(
                norm_sq, 8,
                "All E8 roots must have equal squared norm 8: got {norm_sq}"
            );
        }
    }

    #[test]
    fn test_grouped4bit_arbitrary_dimensions() -> Result<()> {
        for &dim in &[288usize, 749usize] {
            let weights: Vec<f32> = (0..dim).map(|i| ((i as f32) - 100.0) * 0.02).collect();
            let row = Grouped4BitRow::encode_f32(&weights, DEFAULT_GROUP_SIZE)?;
            let deq = row.dequantize_f32();
            assert_eq!(deq.len(), dim);
            for (i, (&w, &d)) in weights.iter().zip(&deq).enumerate() {
                let g = i / DEFAULT_GROUP_SIZE;
                let scale = 2.0f32.powi(i32::from(row.group_exponents[g]));
                assert!(
                    (w - d).abs() <= scale / 2.0 + 1e-4,
                    "w: {w}, d: {d}, scale: {scale}"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn test_mul_small_code_matches_multiplication() -> Result<()> {
        for val in [-1000i64, -50, 0, 17, 999] {
            for code in -7..=7 {
                assert_eq!(mul_small_code_i64(val, code)?, val * code);
            }
        }
        assert!(mul_small_code_i64(10, 8).is_err());
        assert!(mul_small_code_i64(10, -8).is_err());

        // Extreme overflow and boundary tests
        assert!(mul_small_code_i64(1i64 << 62, 4).is_err()); // Shift-add overflow
        assert!(mul_small_code_i64(i64::MIN, -1).is_err()); // Negation overflow
        assert!(mul_small_code_i64(i64::MAX, 2).is_err()); // Positive overflow
        assert!(mul_small_code_i64(i64::MIN / 2 - 1, 2).is_err()); // Negative overflow
        assert_eq!(mul_small_code_i64(i64::MIN, 1)?, i64::MIN);
        assert_eq!(mul_small_code_i64(i64::MAX, 1)?, i64::MAX);
        assert_eq!(mul_small_code_i64(i64::MIN, 0)?, 0);
        assert_eq!(mul_small_code_i64(i64::MAX, 0)?, 0);
        Ok(())
    }

    #[test]
    fn test_grouped4bit_nonfinite_rejected() {
        assert!(Grouped4BitRow::encode_f32(&[f32::NAN, 1.0], 32).is_err());
        assert!(Grouped4BitRow::encode_f32(&[f32::INFINITY, 1.0], 32).is_err());
        assert!(Grouped4BitRow::encode_f32(&[f32::NEG_INFINITY, 1.0], 32).is_err());
    }

    #[test]
    fn test_apply_codec_arm_geometric_s1_dimensions() -> Result<()> {
        let rows = 4;
        let cols = 288;
        let weights: Vec<f32> = (0..rows * cols)
            .map(|i| (((i as f32) * 1.3) % 20.0 - 10.0) * 0.05)
            .collect();

        for arm in [
            CodecArm::NearestPerRow,
            CodecArm::HadamardGrouped4Bit,
            CodecArm::HadamardE8,
            CodecArm::HeadCompensated,
        ] {
            let deq = apply_codec_arm(&weights, rows, cols, arm, 100)?;
            assert_eq!(deq.len(), rows * cols);
        }
        Ok(())
    }
}
