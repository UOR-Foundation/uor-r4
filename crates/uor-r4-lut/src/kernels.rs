//! Portable integer kernels.
//!
//! Learned weight maps (`gemv`, `dequant_row`) use additions, subtractions,
//! shifts and table reads only. The other kernels multiply runtime values or
//! fixed non-learned constants, which owner decision D10 allows; the few
//! per-vector reciprocals (normalization, softmax) use one integer division
//! per vector.

/// Round-half-up arithmetic shift right by `shift` (left if negative),
/// saturating at the `i64` range.
pub fn shift(value: i64, shift: i32) -> i64 {
    if shift > 0 {
        if shift >= 63 {
            return if value < 0 { -1 } else { 0 };
        }
        let half = 1i64 << (shift - 1);
        value.saturating_add(half) >> shift
    } else if shift < 0 {
        let k = (-shift).min(63) as u32;
        let limit = i64::MAX >> k;
        if value > limit {
            i64::MAX
        } else if value < -limit {
            i64::MIN
        } else {
            value << k
        }
    } else {
        value
    }
}

/// Convert `value * 2^from` to the nearest `v * 2^to`, saturated to `i32`.
pub fn to_exp_i32(value: i64, from: i32, to: i32) -> i32 {
    shift(value, to - from).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn bit_length(value: u64) -> i32 {
    64 - value.leading_zeros() as i32
}

/// A vector of `i16` with one power-of-two exponent.
#[derive(Clone, Debug, Default)]
pub struct Act16 {
    pub values: Vec<i16>,
    pub exp: i32,
}

/// Requantize `values * 2^exp` to 16 bits with the smallest exponent that fits.
pub fn quantize16(values: &[i64], exp: i32, out: &mut Act16) {
    let max = values.iter().map(|v| v.unsigned_abs()).max().unwrap_or(0);
    let s = (bit_length(max) - 15).max(0);
    out.values.clear();
    out.values.extend(
        values
            .iter()
            .map(|&v| shift(v, s).clamp(-32767, 32767) as i16),
    );
    out.exp = exp + s;
}

/// Requantize `values * 2^exp` to 8 bits; returns the new exponent.
pub fn quantize8(values: &[i64], exp: i32, out: &mut [i8]) -> i32 {
    let max = values.iter().map(|v| v.unsigned_abs()).max().unwrap_or(0);
    let s = (bit_length(max) - 7).max(0);
    for (o, &v) in out.iter_mut().zip(values) {
        *o = shift(v, s).clamp(-127, 127) as i8;
    }
    exp + s
}

/// `(16 + m) * a` by shifts and adds (`m < 16`).
#[inline]
pub fn scale_16_plus(a: i64, m: u8) -> i64 {
    let mut t = a << 4;
    for bit in 0..4 {
        if m & (1 << bit) != 0 {
            t += a << bit;
        }
    }
    t
}

/// Per-activation tables of multiples: `table[i * 16 + (q + 8)] = q * x[i]` for
/// `q` in `-8..=7`, built with shifts, additions and negations only.
pub fn build_multiples(x: &[i16], table: &mut Vec<i32>) {
    table.clear();
    table.resize(x.len() * 16, 0);
    for (i, &value) in x.iter().enumerate() {
        let v = i32::from(value);
        let t = &mut table[i * 16..i * 16 + 16];
        let (v2, v4) = (v << 1, v << 2);
        let positive = [0, v, v2, v2 + v, v4, v4 + v, v4 + v2, v4 + v2 + v];
        for (q, p) in positive.iter().enumerate() {
            t[8 + q] = *p;
        }
        for q in 1..8 {
            t[8 - q] = -positive[q];
        }
        t[0] = -(v << 3);
    }
}

/// One packed 4-bit matrix.
pub struct MatrixView<'a> {
    pub rows: usize,
    pub cols: usize,
    pub exp_base: i32,
    pub nibbles: &'a [u8],
    pub scales: &'a [u8],
    /// Smallest scale exponent offset in each row.
    pub row_min_de: &'a [u8],
}

/// Smallest scale exponent offset (`de`) of every row.
pub fn row_min_de(rows: usize, cols: usize, scales: &[u8]) -> Vec<u8> {
    let groups = cols / crate::GROUP;
    (0..rows)
        .map(|r| {
            scales[r * groups..(r + 1) * groups]
                .iter()
                .map(|s| s >> 4)
                .min()
                .unwrap_or(0)
        })
        .collect()
}

/// Matrices with at least this many weights split their rows across threads.
pub const PARALLEL_WEIGHTS: usize = 1 << 18;

/// One output row of [`gemv`]: the accumulated value and its exponent offset
/// `de_min`. Additions, shifts and table reads only.
#[inline]
fn gemv_row(m: &MatrixView<'_>, table: &[i32], r: usize) -> i64 {
    let groups = m.cols / crate::GROUP;
    let half_group = crate::GROUP / 2;
    let nibbles = &m.nibbles[r * m.cols / 2..(r + 1) * m.cols / 2];
    let scales = &m.scales[r * groups..(r + 1) * groups];
    let de_min = m.row_min_de[r];
    let mut acc = 0i64;
    for (g, &scale) in scales.iter().enumerate() {
        let mut a = 0i32;
        let base = g * crate::GROUP;
        for (b, &byte) in nibbles[g * half_group..(g + 1) * half_group]
            .iter()
            .enumerate()
        {
            let i = base + 2 * b;
            a += table[i * 16 + usize::from(byte & 15)];
            a += table[(i + 1) * 16 + usize::from(byte >> 4)];
        }
        acc += scale_16_plus(i64::from(a), scale & 15) << ((scale >> 4) - de_min);
    }
    acc
}

/// `out[r] = sum_c W[r][c] x[c]`, with `x` given by its multiples table and
/// exponent, written at exponent `out_exp`. Additions, shifts and table reads
/// only: no multiplier touches a weight. Large matrices split their rows
/// across threads; the arithmetic, and so the result, is the same.
pub fn gemv(m: &MatrixView<'_>, table: &[i32], x_exp: i32, out: &mut [i32], out_exp: i32) {
    use rayon::prelude::*;
    let finish = |r: usize, slot: &mut i32| {
        let acc = gemv_row(m, table, r);
        let from = m.exp_base + i32::from(m.row_min_de[r]) - 4 + x_exp;
        *slot = to_exp_i32(acc, from, out_exp);
    };
    let out = &mut out[..m.rows];
    if m.rows * m.cols >= PARALLEL_WEIGHTS {
        const CHUNK: usize = 32;
        out.par_chunks_mut(CHUNK)
            .enumerate()
            .for_each(|(c, chunk)| {
                for (i, slot) in chunk.iter_mut().enumerate() {
                    finish(c * CHUNK + i, slot);
                }
            });
    } else {
        for (r, slot) in out.iter_mut().enumerate() {
            finish(r, slot);
        }
    }
}

/// Row `row` of a packed matrix, written at exponent `out_exp` (embedding lookup).
pub fn dequant_row(m: &MatrixView<'_>, row: usize, out: &mut [i32], out_exp: i32) {
    let groups = m.cols / crate::GROUP;
    let nibbles = &m.nibbles[row * m.cols / 2..(row + 1) * m.cols / 2];
    let scales = &m.scales[row * groups..(row + 1) * groups];
    for (c, slot) in out.iter_mut().enumerate().take(m.cols) {
        let byte = nibbles[c / 2];
        let q = i64::from(if c % 2 == 0 { byte & 15 } else { byte >> 4 }) - 8;
        let scale = scales[c / crate::GROUP];
        let value = scale_16_plus(q, scale & 15);
        *slot = to_exp_i32(value, m.exp_base + i32::from(scale >> 4) - 4, out_exp);
    }
}

/// Floor integer square root (digit by digit: shifts, compares, subtractions).
pub fn isqrt(value: u128) -> u128 {
    let mut remainder = value;
    let mut root = 0u128;
    let mut bit = 1u128 << 126;
    while bit > value {
        bit >>= 2;
    }
    while bit != 0 {
        if remainder >= root + bit {
            remainder -= root + bit;
            root = (root >> 1) + bit;
        } else {
            root >>= 1;
        }
        bit >>= 2;
    }
    root
}

/// RMSNorm without its gain (folded into the following weights): `x / rms(x)`,
/// for `x` at exponent `x_exp`, into `out` (16-bit, fitted exponent).
pub fn rms_norm(
    x: &[i32],
    x_exp: i32,
    eps: crate::format::Fixed,
    scratch: &mut Vec<i64>,
    out: &mut Act16,
) {
    let max = x.iter().map(|v| v.unsigned_abs()).max().unwrap_or(0);
    let sh = (bit_length(u64::from(max)) - 24).max(0);
    scratch.clear();
    scratch.extend(x.iter().map(|&v| shift(i64::from(v), sh)));
    let sum: i64 = scratch.iter().map(|&v| v * v).sum();
    let mean = sum / x.len().max(1) as i64;
    let eps_int = shift(eps.mantissa, 2 * (x_exp + sh) - eps.exp).max(0);
    let ms = (mean + eps_int).max(1) as u128;
    // s = sqrt(ms) 2^34, r = 2^96 / s = 2^62 / sqrt(ms)
    let s = isqrt(ms << 68).max(1);
    let r = (1u128 << 96) / s;
    for v in scratch.iter_mut() {
        let y = (i128::from(*v) * r as i128) >> 48; // y * 2^14
        *v = y.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64;
    }
    quantize16(scratch, -14, out);
}

/// Half-split RoPE on one head (`head.len()` even), values at any exponent.
pub fn rope(head: &mut [i32], position: usize, cos: &[i16], sin: &[i16], q: u32) {
    let half = head.len() / 2;
    let row = position * half;
    for i in 0..half {
        let (a, b) = (i64::from(head[i]), i64::from(head[i + half]));
        let (c, s) = (i64::from(cos[row + i]), i64::from(sin[row + i]));
        head[i] =
            shift(a * c - b * s, q as i32).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
        head[i + half] =
            shift(b * c + a * s, q as i32).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
    }
}

/// `round(2^31 exp(-d))` for `d = value * 2^exp_d`, from the sealed table with
/// step `2^step_log2` and linear interpolation.
pub fn exp_neg(d: i64, exp_d: i32, table: &[u32], step_log2: i32) -> u64 {
    let frac_bits = step_log2 - exp_d;
    if d <= 0 {
        return u64::from(table[0]);
    }
    if frac_bits <= 0 {
        let index = (d << (-frac_bits)) as usize;
        return table.get(index).map_or(0, |v| u64::from(*v));
    }
    let index = (d >> frac_bits) as usize;
    if index + 1 >= table.len() {
        return 0;
    }
    let frac = (d & ((1 << frac_bits) - 1)) as u64;
    let (a, b) = (u64::from(table[index]), u64::from(table[index + 1]));
    a - (((a - b) * frac) >> frac_bits)
}

/// SiLU of `x * 2^-16`, returned at exponent -16, from the sealed table.
pub fn silu(x: i32, table: &[i32], step_log2: i32, range_log2: i32) -> i32 {
    // x is in units of 2^-16 and the table step is 2^step_log2 (-16 <= step_log2 < 0).
    let frac_bits = step_log2 + 16;
    debug_assert!((0..=16).contains(&frac_bits));
    let half = 1i64 << (range_log2 - step_log2);
    let index = (i64::from(x) >> frac_bits) + half;
    if index < 0 {
        return 0;
    }
    if index as usize + 1 >= table.len() {
        return x;
    }
    let frac = i64::from(x) & ((1i64 << frac_bits) - 1);
    let (a, b) = (
        i64::from(table[index as usize]),
        i64::from(table[index as usize + 1]),
    );
    (a + (((b - a) * frac) >> frac_bits)) as i32
}

/// Index of the largest value (first on ties).
pub fn argmax(values: &[i32]) -> usize {
    let mut best = 0;
    for (i, v) in values.iter().enumerate() {
        if *v > values[best] {
            best = i;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiples_tables_hold_every_product() {
        let x = [-32767i16, -5, 0, 1, 1234, 32767];
        let mut table = Vec::new();
        build_multiples(&x, &mut table);
        for (i, v) in x.iter().enumerate() {
            for q in -8i32..=7 {
                assert_eq!(table[i * 16 + (q + 8) as usize], q * i32::from(*v));
            }
        }
    }

    #[test]
    fn scale_by_shift_add_equals_product() {
        for a in [-1_000_000i64, -3, 0, 7, 123_456_789] {
            for m in 0..16u8 {
                assert_eq!(scale_16_plus(a, m), a * (16 + i64::from(m)));
            }
        }
    }

    #[test]
    fn gemv_matches_the_dequantized_product() {
        let (rows, cols) = (3, 64);
        let mut nibbles = vec![0u8; rows * cols / 2];
        let mut scales = vec![0u8; rows * cols / crate::GROUP];
        let mut seed = 12345u32;
        let mut next = || {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
            (seed >> 16) as u8
        };
        for b in nibbles.iter_mut() {
            *b = next();
        }
        for s in scales.iter_mut() {
            // de in 0..=3 keeps the exact result inside i32 at the output exponent
            *s = next() & 0x3F;
        }
        let x: Vec<i16> = (0..cols).map(|i| (i as i16 * 97 % 2000) - 1000).collect();
        let min_de = row_min_de(rows, cols, &scales);
        let m = MatrixView {
            rows,
            cols,
            exp_base: -6,
            nibbles: &nibbles,
            scales: &scales,
            row_min_de: &min_de,
        };
        let mut table = Vec::new();
        build_multiples(&x, &mut table);
        let mut out = vec![0i32; rows];
        // exact products are at exponent exp_base - 4 + x_exp = -13
        gemv(&m, &table, -3, &mut out, -13);
        for (r, got) in out.iter().enumerate() {
            let mut exact = 0i128;
            for c in 0..cols {
                let byte = nibbles[(r * cols + c) / 2];
                let q = i128::from(if c % 2 == 0 { byte & 15 } else { byte >> 4 }) - 8;
                let s = scales[(r * cols + c) / crate::GROUP];
                exact += (q * (16 + i128::from(s & 15)) * i128::from(x[c])) << (s >> 4);
            }
            assert_eq!(i128::from(*got), exact, "row {r}");
        }
    }

    #[test]
    fn integer_square_root_is_exact() {
        for v in [
            0u128,
            1,
            2,
            3,
            4,
            15,
            16,
            17,
            1 << 40,
            (1 << 100) + 12345,
            u128::MAX >> 2,
        ] {
            let r = isqrt(v);
            assert!(r * r <= v && (r + 1) * (r + 1) > v, "{v}");
        }
    }

    #[test]
    fn rms_norm_normalizes() {
        let x: Vec<i32> = (0..64).map(|i| (i - 32) * 40_000).collect();
        let eps = crate::format::Fixed {
            mantissa: 1,
            exp: -40,
        };
        let (mut scratch, mut out) = (Vec::new(), Act16::default());
        rms_norm(&x, -16, eps, &mut scratch, &mut out);
        let sum: f64 = out
            .values
            .iter()
            .map(|v| (f64::from(*v) * 2f64.powi(out.exp)).powi(2))
            .sum();
        assert!(
            (sum / 64.0 - 1.0).abs() < 1e-3,
            "mean square {}",
            sum / 64.0
        );
    }

    #[test]
    fn silu_table_lookup_matches_the_function() {
        // The exporter's table: step 2^-8, half range 16, Q16 entries.
        let (step, range) = (-8, 4);
        let half = 1i64 << (range - step);
        let table: Vec<i32> = (0..=2 * half)
            .map(|i| {
                let x = (i - half) as f64 / 256.0;
                (65536.0 * x / (1.0 + (-x).exp())).round() as i32
            })
            .collect();
        for x in [
            -20.0f64, -15.99, -3.3, -0.51, 0.0, 0.004, 0.7, 2.25, 9.9, 15.9, 40.0,
        ] {
            let fixed = (x * 65536.0).round() as i32;
            let got = f64::from(silu(fixed, &table, step, range)) / 65536.0;
            let want = x / (1.0 + (-x).exp());
            assert!((got - want).abs() < 2e-4, "silu({x}) = {got}, want {want}");
        }
    }

    #[test]
    fn exp_table_lookup_matches_the_function() {
        let step = -8;
        let table: Vec<u32> = (0..32 * 256 + 2)
            .map(|i| (2f64.powi(31) * (-(i as f64) / 256.0).exp()).round() as u32)
            .collect();
        for d in [0.0f64, 0.001, 0.5, 1.0, 3.7, 12.25, 31.9, 40.0] {
            let fixed = (d * 4096.0).round() as i64; // exponent -12
            let got = exp_neg(fixed, -12, &table, step) as f64 / 2f64.powi(31);
            // compare at the kernel's input resolution (2^-12 nats)
            let want = (-(fixed as f64) / 4096.0).exp();
            assert!((got - want).abs() < 2e-6, "exp(-{d}) = {got}, want {want}");
        }
    }

    #[test]
    fn parallel_gemv_matches_the_exact_product() {
        let (rows, cols) = (1024, 256);
        assert!(rows * cols >= PARALLEL_WEIGHTS);
        let mut seed = 99u32;
        let mut next = || {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
            (seed >> 16) as u8
        };
        let nibbles: Vec<u8> = (0..rows * cols / 2).map(|_| next()).collect();
        let scales: Vec<u8> = (0..rows * cols / crate::GROUP)
            .map(|_| next() & 0x3F)
            .collect();
        let x: Vec<i16> = (0..cols)
            .map(|i| ((i * 131) % 4001) as i16 - 2000)
            .collect();
        let min_de = row_min_de(rows, cols, &scales);
        let m = MatrixView {
            rows,
            cols,
            exp_base: -2,
            nibbles: &nibbles,
            scales: &scales,
            row_min_de: &min_de,
        };
        let mut table = Vec::new();
        build_multiples(&x, &mut table);
        let mut out = vec![0i32; rows];
        gemv(&m, &table, -8, &mut out, -14);
        for (r, got) in out.iter().enumerate() {
            let mut exact = 0i128;
            for c in 0..cols {
                let byte = nibbles[(r * cols + c) / 2];
                let q = i128::from(if c % 2 == 0 { byte & 15 } else { byte >> 4 }) - 8;
                let s = scales[(r * cols + c) / crate::GROUP];
                exact += (q * (16 + i128::from(s & 15)) * i128::from(x[c])) << (s >> 4);
            }
            // exact is at exponent -2 - 4 - 8 = -14, the output exponent
            assert_eq!(i128::from(*got), exact, "row {r}");
        }
    }
}
