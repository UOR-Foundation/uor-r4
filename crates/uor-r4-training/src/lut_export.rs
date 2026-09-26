//! Offline export of a Llama checkpoint to the integer serving artifact of
//! `uor-r4-lut` (owner decision D10).
//!
//! Floating point is used here only, once, offline: RMSNorm gains are folded
//! into the following weight maps, every weight matrix is quantized to 4 bits
//! in groups of `GROUP` with the scale of each group chosen from the grid
//! `(16 + m) 2^(e - 4)` by least squares, and the exp, SiLU and RoPE tables are
//! sealed into the artifact. The artifact is then served with integers only.

use serde_json::{json, Value};
use uor_r4_lut::format::{ArtifactBuilder, Fixed, Numerics, Shape, TableValues};
use uor_r4_lut::GROUP;

use crate::kappa_llama::Checkpoint;
use crate::{invalid, Result};

/// Exponent step of the exp table (`2^-8` nats).
pub const EXP_STEP_LOG2: i32 = -8;
/// The exp table covers `d` in `[0, EXP_RANGE]` nats.
pub const EXP_RANGE: usize = 32;
/// SiLU table step (`2^-8`) and half range (`2^4 = 16`).
pub const SILU_STEP_LOG2: i32 = -8;
pub const SILU_RANGE_LOG2: i32 = 4;
/// RoPE tables are stored in Q14.
pub const ROPE_Q: u32 = 14;

/// A 4-bit matrix in the artifact layout, with its relative RMS error.
pub struct Packed {
    pub nibbles: Vec<u8>,
    pub scales: Vec<u8>,
    pub exp_base: i32,
    pub relative_rms_error: f64,
}

fn grid_scale(m: u8, e: i32) -> f64 {
    (16.0 + f64::from(m)) * 2f64.powi(e - 4)
}

fn group_error(w: &[f32], s: f64) -> f64 {
    w.iter()
        .map(|v| {
            let q = (f64::from(*v) / s).round().clamp(-8.0, 7.0);
            (f64::from(*v) - q * s).powi(2)
        })
        .sum()
}

/// Quantize a row-major `rows x cols` matrix (`cols` a multiple of `GROUP`).
pub fn quantize_matrix(values: &[f32], rows: usize, cols: usize) -> Result<Packed> {
    if values.len() != rows * cols
        || !cols.is_multiple_of(GROUP)
        || values.iter().any(|v| !v.is_finite())
    {
        return Err(invalid(
            "matrix to quantize has the wrong size or a nonfinite value",
        ));
    }
    let groups = cols / GROUP;
    // First pass: best (m, e) per group, None for an all-zero group.
    let mut chosen: Vec<Option<(u8, i32)>> = Vec::with_capacity(rows * groups);
    for r in 0..rows {
        for g in 0..groups {
            let w = &values[r * cols + g * GROUP..r * cols + (g + 1) * GROUP];
            let a = w.iter().map(|v| f64::from(v.abs())).fold(0.0, f64::max);
            if a == 0.0 {
                chosen.push(None);
                continue;
            }
            let (lo, hi) = (a / 8.5, a / 5.5);
            let mut best: Option<(f64, u8, i32)> = None;
            let e_lo = (lo / 32.0).log2().floor() as i32 + 4;
            let e_hi = (hi / 16.0).log2().ceil() as i32 + 4;
            for e in e_lo..=e_hi {
                for m in 0..16u8 {
                    let s = grid_scale(m, e);
                    if s < lo || s > hi {
                        continue;
                    }
                    let err = group_error(w, s);
                    if best.is_none_or(|b| err < b.0) {
                        best = Some((err, m, e));
                    }
                }
            }
            let (m, e) = match best {
                Some((_, m, e)) => (m, e),
                None => {
                    // Smallest grid scale >= a / 7.
                    let e = (a / 7.0 / 16.0).log2().ceil() as i32 + 4;
                    (0, e)
                }
            };
            chosen.push(Some((m, e)));
        }
    }
    let e_max = chosen.iter().flatten().map(|(_, e)| *e).max().unwrap_or(0);
    let exp_base = e_max - 15;
    let mut nibbles = vec![0u8; rows * cols / 2];
    let mut scales = vec![0u8; rows * groups];
    let (mut error, mut energy) = (0f64, 0f64);
    for r in 0..rows {
        for g in 0..groups {
            let (m, e) = match chosen[r * groups + g] {
                Some((m, e)) if e >= exp_base => (m, e),
                _ => (0, exp_base),
            };
            let s = grid_scale(m, e);
            scales[r * groups + g] = m | (((e - exp_base) as u8) << 4);
            for c in g * GROUP..(g + 1) * GROUP {
                let v = f64::from(values[r * cols + c]);
                let q = (v / s).round().clamp(-8.0, 7.0);
                error += (v - q * s).powi(2);
                energy += v * v;
                let nibble = (q as i32 + 8) as u8;
                let index = (r * cols + c) / 2;
                if c % 2 == 0 {
                    nibbles[index] |= nibble;
                } else {
                    nibbles[index] |= nibble << 4;
                }
            }
        }
    }
    Ok(Packed {
        nibbles,
        scales,
        exp_base,
        relative_rms_error: if energy > 0.0 {
            (error / energy).sqrt()
        } else {
            0.0
        },
    })
}

fn tensor(checkpoint: &Checkpoint, name: &str) -> Result<Vec<f32>> {
    Ok(checkpoint
        .tensors
        .get(name)
        .ok_or_else(|| invalid(format!("missing tensor {name}")))?
        .flatten_all()?
        .to_vec1::<f32>()?)
}

/// Multiply column `c` of a row-major matrix by `gain[c]` (RMSNorm folding).
fn fold_columns(values: &mut [f32], cols: usize, gain: &[f32]) {
    for row in values.chunks_exact_mut(cols) {
        for (v, g) in row.iter_mut().zip(gain) {
            *v *= g;
        }
    }
}

/// Export `checkpoint` for integer serving with sessions of up to
/// `max_positions` tokens. Returns the artifact bytes and a quantization report.
pub fn export_llama(
    checkpoint: &Checkpoint,
    max_positions: usize,
    source: Value,
) -> Result<(Vec<u8>, Value)> {
    let s = &checkpoint.shape;
    let shape = Shape {
        vocab: s.vocab,
        width: s.width,
        layers: s.layers,
        heads: s.heads,
        kv_heads: s.kv_heads,
        head_dim: s.head_dim,
        ffn: s.ffn,
        max_positions,
    };
    let numerics = Numerics {
        rms_eps: Fixed {
            mantissa: (s.rms_eps * 2f64.powi(48)).round() as i64,
            exp: -48,
        },
        score_scale_q30: (2f64.powi(30) / (s.head_dim as f64).sqrt()).round() as i64,
        exp_step_log2: EXP_STEP_LOG2,
        silu_step_log2: SILU_STEP_LOG2,
        silu_range_log2: SILU_RANGE_LOG2,
        rope_q: ROPE_Q,
    };
    let mut builder =
        ArtifactBuilder::new(shape, numerics, source).map_err(|e| invalid(e.to_string()))?;
    let mut errors = serde_json::Map::new();
    let mut add = |builder: &mut ArtifactBuilder,
                   name: &str,
                   values: &[f32],
                   rows: usize,
                   cols: usize|
     -> Result<()> {
        let packed = quantize_matrix(values, rows, cols)?;
        builder
            .add_matrix(
                name,
                rows,
                cols,
                packed.exp_base,
                &packed.nibbles,
                &packed.scales,
            )
            .map_err(|e| invalid(e.to_string()))?;
        errors.insert(name.to_owned(), json!(packed.relative_rms_error));
        Ok(())
    };
    let embed = tensor(checkpoint, "model.embed_tokens.weight")?;
    add(&mut builder, "embed", &embed, s.vocab, s.width)?;
    let kv_rows = s.kv_heads * s.head_dim;
    for l in 0..s.layers {
        let p = format!("model.layers.{l}");
        let input_gain = tensor(checkpoint, &format!("{p}.input_layernorm.weight"))?;
        let post_gain = tensor(checkpoint, &format!("{p}.post_attention_layernorm.weight"))?;
        for (name, rows, cols, gain) in [
            ("q_proj", s.width, s.width, Some(&input_gain)),
            ("k_proj", kv_rows, s.width, Some(&input_gain)),
            ("v_proj", kv_rows, s.width, Some(&input_gain)),
            ("o_proj", s.width, s.width, None),
        ] {
            let mut w = tensor(checkpoint, &format!("{p}.self_attn.{name}.weight"))?;
            if let Some(gain) = gain {
                fold_columns(&mut w, cols, gain);
            }
            let short = &name[..1];
            add(&mut builder, &format!("l{l}.{short}"), &w, rows, cols)?;
        }
        for (name, short, rows, cols, gain) in [
            ("gate_proj", "gate", s.ffn, s.width, Some(&post_gain)),
            ("up_proj", "up", s.ffn, s.width, Some(&post_gain)),
            ("down_proj", "down", s.width, s.ffn, None),
        ] {
            let mut w = tensor(checkpoint, &format!("{p}.mlp.{name}.weight"))?;
            if let Some(gain) = gain {
                fold_columns(&mut w, cols, gain);
            }
            add(&mut builder, &format!("l{l}.{short}"), &w, rows, cols)?;
        }
    }
    let mut head = if s.tied_embeddings {
        embed
    } else {
        tensor(checkpoint, "lm_head.weight")?
    };
    fold_columns(
        &mut head,
        s.width,
        &tensor(checkpoint, "model.norm.weight")?,
    );
    add(&mut builder, "head", &head, s.vocab, s.width)?;

    let exp: Vec<u32> = (0..EXP_RANGE * (1 << -EXP_STEP_LOG2) + 2)
        .map(|i| (2f64.powi(31) * (-(i as f64) * 2f64.powi(EXP_STEP_LOG2)).exp()).round() as u32)
        .collect();
    let half = 1i64 << (SILU_RANGE_LOG2 - SILU_STEP_LOG2);
    let silu: Vec<i32> = (0..=2 * half)
        .map(|i| {
            let x = (i - half) as f64 * 2f64.powi(SILU_STEP_LOG2);
            (2f64.powi(16) * x / (1.0 + (-x).exp())).round() as i32
        })
        .collect();
    let pairs = s.head_dim / 2;
    let mut cos = Vec::with_capacity(max_positions * pairs);
    let mut sin = Vec::with_capacity(max_positions * pairs);
    for position in 0..max_positions {
        for pair in 0..pairs {
            let inverse = 1.0 / s.rope_theta.powf((2 * pair) as f64 / s.head_dim as f64);
            let angle = position as f64 * inverse;
            let q = f64::from(1u32 << ROPE_Q);
            cos.push((q * angle.cos()).round() as i16);
            sin.push((q * angle.sin()).round() as i16);
        }
    }
    let table = |b: &mut ArtifactBuilder, name: &str, v: TableValues<'_>| -> Result<()> {
        b.add_table(name, v).map_err(|e| invalid(e.to_string()))
    };
    table(&mut builder, "exp", TableValues::U32(&exp))?;
    table(&mut builder, "silu", TableValues::I32(&silu))?;
    table(&mut builder, "rope_cos", TableValues::I16(&cos))?;
    table(&mut builder, "rope_sin", TableValues::I16(&sin))?;
    let bytes = builder.finish().map_err(|e| invalid(e.to_string()))?;
    Ok((bytes, json!({"relative_rms_error": errors})))
}

/// Float values of a packed 4-bit matrix (diagnostics: isolates weight
/// quantization from integer arithmetic).
pub fn dequantize_matrix(
    rows: usize,
    cols: usize,
    exp_base: i32,
    nibbles: &[u8],
    scales: &[u8],
) -> Result<Vec<f32>> {
    if nibbles.len() != rows * cols / 2 || scales.len() != rows * cols / GROUP {
        return Err(invalid("packed matrix has the wrong size"));
    }
    let mut out = Vec::with_capacity(rows * cols);
    for r in 0..rows {
        for c in 0..cols {
            let byte = nibbles[(r * cols + c) / 2];
            let q = i32::from(if c % 2 == 0 { byte & 15 } else { byte >> 4 }) - 8;
            let scale = scales[(r * cols + c) / GROUP];
            out.push(
                (f64::from(q) * grid_scale(scale & 15, exp_base + i32::from(scale >> 4))) as f32,
            );
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantization_error_is_small_and_packing_round_trips() {
        let (rows, cols) = (4, 64);
        let values: Vec<f32> = (0..rows * cols)
            .map(|i| ((i * 7919 % 1000) as f32 / 500.0 - 1.0) * if i % 5 == 0 { 3.0 } else { 0.4 })
            .collect();
        let packed = quantize_matrix(&values, rows, cols).expect("quantize");
        // One weight in five is 7.5x the rest, so the outliers set the step: uniform
        // quantization predicts about 0.15 relative error.
        assert!(
            packed.relative_rms_error < 0.15,
            "{}",
            packed.relative_rms_error
        );
        for r in 0..rows {
            for c in 0..cols {
                let byte = packed.nibbles[(r * cols + c) / 2];
                let q = i32::from(if c % 2 == 0 { byte & 15 } else { byte >> 4 }) - 8;
                let scale = packed.scales[r * cols / GROUP + c / GROUP];
                let s = grid_scale(scale & 15, packed.exp_base + i32::from(scale >> 4));
                let error = (f64::from(values[r * cols + c]) - f64::from(q) * s).abs();
                assert!(error <= s * 0.5 + 1e-9 || q == 7 || q == -8, "r{r} c{c}");
            }
        }
    }

    #[test]
    fn near_gaussian_weights_quantize_to_under_ten_percent_error() {
        // Sum of four uniforms: close to Gaussian, like trained projection weights.
        let mut seed = 7u64;
        let mut uniform = || {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            ((seed >> 40) as f32) / ((1u64 << 24) as f32) - 0.5
        };
        let values: Vec<f32> = (0..64 * 256)
            .map(|_| (uniform() + uniform() + uniform() + uniform()) * 0.05)
            .collect();
        let packed = quantize_matrix(&values, 64, 256).expect("quantize");
        assert!(
            packed.relative_rms_error < 0.10,
            "{}",
            packed.relative_rms_error
        );
    }
}
