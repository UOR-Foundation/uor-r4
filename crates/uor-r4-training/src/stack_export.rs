//! Offline export of a trained geometric stack ([`crate::geometric_stack`]) to
//! the integer serving artifacts of `uor-r4-lut` (owner decision D10).
//!
//! A geometric stack becomes a stack artifact ([`export_stack`], served by
//! `uor_r4_lut::stack`). Norm gains are folded into the maps that read the
//! normalized state. Every weight matrix is quantized to 4 bits in groups of
//! `GROUP` (round to nearest, as [`crate::lut_export::quantize_matrix`]), and
//! the MLP is padded with zero units to a multiple of the group. Each learned
//! scalar that multiplies a runtime value (convolution taps, decay rates,
//! Lorentz scales) becomes a grid code `±(16 + m) 2^(e - 4)`. Biases, age
//! tables and Lorentz offsets become integers, and the exp, SiLU, GELU and
//! arcosh tables are sealed. The transformer control has the shape of a Llama
//! checkpoint, so [`control_checkpoint`] renames its tensors for
//! [`crate::lut_export::export_llama`].
//!
//! Floating point is used here only, once, offline.

use std::collections::BTreeMap;

use candle_core::Tensor;
use serde_json::{json, Value};
use uor_r4_lut::format::{Fixed, StackArtifactBuilder, StackNumerics, StackShape, TableValues};
use uor_r4_lut::kernels::grid_encode;
use uor_r4_lut::GROUP;

use crate::geometric_stack::{ReadScore, StackArch, StackModel};
use crate::kappa_llama::{Checkpoint, LlamaShape};
use crate::lut_export::{
    arcosh_table, grid_nearest, quantize_matrix, EXP_RANGE, EXP_STEP_LOG2, SILU_RANGE_LOG2,
    SILU_STEP_LOG2,
};
use crate::{invalid, Result};

/// GELU table step (`2^-8`) and half range (`2^4 = 16`).
pub const GELU_STEP_LOG2: i32 = -8;
pub const GELU_RANGE_LOG2: i32 = 4;
/// The stack's RMSNorm epsilon.
const RMS_EPSILON: f64 = 1e-5;
/// The stack's decay exponent: `log lambda = 8 r log a`.
const DECAY_EXPONENT: f64 = 8.0;
/// Scalars below this magnitude become the zero grid code.
const GRID_ZERO: f64 = 1e-18;

/// The grid code nearest to `value` (zero for negligible values).
pub fn grid_code(value: f64) -> Result<i16> {
    if !value.is_finite() {
        return Err(invalid("a nonfinite scalar has no grid code"));
    }
    if value.abs() < GRID_ZERO {
        return Ok(0);
    }
    let (m, e) = grid_nearest(value.abs());
    grid_encode(m, e, value < 0.0).ok_or_else(|| invalid("scalar outside the grid-code range"))
}

/// The value of a grid code (diagnostics and parity references).
pub fn grid_value(code: i16) -> f64 {
    if code == 0 {
        return 0.0;
    }
    let c = code.unsigned_abs();
    let e = i32::from(c >> 4) - 64;
    let value = (16.0 + f64::from(c & 15)) * 2f64.powi(e - 4);
    if code < 0 {
        -value
    } else {
        value
    }
}

/// `8 softplus(-decay)`: the rate with `lambda = exp(-rate r)`.
pub fn decay_rate(decay: f64) -> f64 {
    DECAY_EXPONENT * (-decay).exp().ln_1p()
}

fn values(model: &StackModel, name: &str) -> Result<Vec<f32>> {
    let var = model
        .variables()
        .get(name)
        .ok_or_else(|| invalid(format!("the stack has no tensor {name}")))?;
    Ok(var.as_tensor().flatten_all()?.to_vec1::<f32>()?)
}

/// Multiply column `c` of a row-major matrix by `gain[c]` (RMSNorm folding).
fn fold_columns(values: &mut [f32], cols: usize, gain: &[f32]) {
    for row in values.chunks_exact_mut(cols) {
        for (v, g) in row.iter_mut().zip(gain) {
            *v *= g;
        }
    }
}

/// Pad a row-major `rows x cols` matrix with zero rows and columns.
fn pad(values: &[f32], rows: usize, cols: usize, to_rows: usize, to_cols: usize) -> Vec<f32> {
    let mut out = vec![0f32; to_rows * to_cols];
    for r in 0..rows {
        out[r * to_cols..r * to_cols + cols].copy_from_slice(&values[r * cols..(r + 1) * cols]);
    }
    out
}

fn fixed(value: f64, exp: i32) -> Result<i32> {
    let scaled = (value * 2f64.powi(-exp)).round();
    if !scaled.is_finite() || scaled.abs() > f64::from(i32::MAX) {
        return Err(invalid("a bias or offset overflows its integer"));
    }
    Ok(scaled as i32)
}

/// Export a geometric stack; returns the artifact bytes and a report of the
/// quantization errors (relative RMS per matrix, worst relative error per
/// table of grid codes).
pub fn export_stack(model: &StackModel, source: Value) -> Result<(Vec<u8>, Value)> {
    let c = &model.config;
    if c.arch != StackArch::Geometric {
        return Err(invalid(
            "export_stack takes a geometric stack; the control exports as a Llama checkpoint",
        ));
    }
    let (d, heads) = (c.width, c.heads);
    let mlp = c.mlp_hidden.div_ceil(GROUP) * GROUP;
    let shape = StackShape {
        vocab: c.vocab_size,
        width: d,
        heads,
        mlp,
        pattern: c.pattern.clone(),
        read: match c.read {
            ReadScore::Dot => "dot",
            ReadScore::Lorentz => "lorentz",
        }
        .to_owned(),
        rotation: c.rotation,
        context: c.context,
    };
    let numerics = StackNumerics {
        rms_eps: Fixed {
            mantissa: (RMS_EPSILON * 2f64.powi(48)).round() as i64,
            exp: -48,
        },
        score_scale_q30: (2f64.powi(30) / (c.head_width() as f64).sqrt()).round() as i64,
        exp_step_log2: EXP_STEP_LOG2,
        silu_step_log2: SILU_STEP_LOG2,
        silu_range_log2: SILU_RANGE_LOG2,
        gelu_step_log2: GELU_STEP_LOG2,
        gelu_range_log2: GELU_RANGE_LOG2,
    };
    let lanes = shape.lanes();
    let gate_rows = shape.gate_rows();
    let mut builder =
        StackArtifactBuilder::new(shape, numerics, source).map_err(|e| invalid(e.to_string()))?;
    let mut errors = serde_json::Map::new();
    let mut add = |builder: &mut StackArtifactBuilder,
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
    let mut code_errors = serde_json::Map::new();
    let mut codes =
        |builder: &mut StackArtifactBuilder, name: &str, scalars: &[f64]| -> Result<()> {
            let codes = scalars
                .iter()
                .map(|&v| grid_code(v))
                .collect::<Result<Vec<i16>>>()?;
            let worst = scalars
                .iter()
                .zip(&codes)
                .filter(|(v, _)| v.abs() >= GRID_ZERO)
                .map(|(v, &code)| ((grid_value(code) - v) / v).abs())
                .fold(0.0, f64::max);
            code_errors.insert(name.to_owned(), json!(worst));
            builder
                .add_table(name, TableValues::I16(&codes))
                .map_err(|e| invalid(e.to_string()))
        };
    let integers =
        |builder: &mut StackArtifactBuilder, name: &str, values: &[f32], exp: i32| -> Result<()> {
            let values = values
                .iter()
                .map(|&v| fixed(f64::from(v), exp))
                .collect::<Result<Vec<i32>>>()?;
            builder
                .add_table(name, TableValues::I32(&values))
                .map_err(|e| invalid(e.to_string()))
        };

    let embed = values(model, "embedding.weight")?;
    add(&mut builder, "embed", &embed, c.vocab_size, d)?;
    for (l, kind) in c.pattern.bytes().enumerate() {
        let tensor = |suffix: &str| values(model, &format!("layers.{l:02}.{suffix}"));
        let name = |part: &str| format!("l{l}.{part}");
        if kind == b'r' {
            let gain = tensor("rec_norm.weight")?;
            let mut input = tensor("rec.in.weight")?;
            fold_columns(&mut input, d, &gain);
            add(&mut builder, &name("rec_in"), &input, 2 * d, d)?;
            let mut gates = tensor("rec.gate.weight")?;
            fold_columns(&mut gates, d, &gain);
            add(&mut builder, &name("rec_gate"), &gates, gate_rows, d)?;
            add(
                &mut builder,
                &name("rec_out"),
                &tensor("rec.out.weight")?,
                d,
                d,
            )?;
            let taps: Vec<f64> = tensor("rec.conv.weight")?
                .iter()
                .map(|&v| f64::from(v))
                .collect();
            codes(&mut builder, &name("conv_taps"), &taps)?;
            integers(
                &mut builder,
                &name("conv_bias"),
                &tensor("rec.conv.bias")?,
                -16,
            )?;
            integers(
                &mut builder,
                &name("gate_bias"),
                &tensor("rec.gate.bias")?,
                -16,
            )?;
            let rates: Vec<f64> = tensor("rec.decay")?
                .iter()
                .map(|&v| decay_rate(f64::from(v)))
                .collect();
            if rates.len() != lanes {
                return Err(invalid("decay count differs from the lane count"));
            }
            codes(&mut builder, &name("decay_rate"), &rates)?;
        } else {
            let gain = tensor("read_norm.weight")?;
            for (part, rows) in [("query", d), ("key", d), ("value", d), ("null", heads)] {
                let mut w = tensor(&format!("read.{part}.weight"))?;
                fold_columns(&mut w, d, &gain);
                add(&mut builder, &name(part), &w, rows, d)?;
            }
            add(
                &mut builder,
                &name("out"),
                &tensor("read.out.weight")?,
                d,
                d,
            )?;
            integers(
                &mut builder,
                &name("null_bias"),
                &tensor("read.null.bias")?,
                -16,
            )?;
            integers(&mut builder, &name("age"), &tensor("read.age")?, -16)?;
            if c.read == ReadScore::Lorentz {
                let beta: Vec<f64> = tensor("read.log_beta")?
                    .iter()
                    .map(|&v| f64::from(v).exp())
                    .collect();
                codes(&mut builder, &name("beta"), &beta)?;
                integers(&mut builder, &name("offset"), &tensor("read.offset")?, -24)?;
            }
        }
        let gain = tensor("mlp_norm.weight")?;
        for part in ["gate", "up"] {
            let mut w = tensor(&format!("mlp.{part}.weight"))?;
            fold_columns(&mut w, d, &gain);
            add(
                &mut builder,
                &name(part),
                &pad(&w, c.mlp_hidden, d, mlp, d),
                mlp,
                d,
            )?;
        }
        let down = tensor("mlp.down.weight")?;
        add(
            &mut builder,
            &name("down"),
            &pad(&down, d, c.mlp_hidden, d, mlp),
            d,
            mlp,
        )?;
    }
    let mut head = embed;
    fold_columns(&mut head, d, &values(model, "final_norm.weight")?);
    add(&mut builder, "head", &head, c.vocab_size, d)?;

    let exp: Vec<u32> = (0..EXP_RANGE * (1 << -EXP_STEP_LOG2) + 2)
        .map(|i| (2f64.powi(31) * (-(i as f64) * 2f64.powi(EXP_STEP_LOG2)).exp()).round() as u32)
        .collect();
    let activation = |step_log2: i32, range_log2: i32, f: &dyn Fn(f64) -> f64| -> Vec<i32> {
        let half = 1i64 << (range_log2 - step_log2);
        (0..=2 * half)
            .map(|i| (2f64.powi(16) * f((i - half) as f64 * 2f64.powi(step_log2))).round() as i32)
            .collect()
    };
    let silu = activation(SILU_STEP_LOG2, SILU_RANGE_LOG2, &|x| x / (1.0 + (-x).exp()));
    let gelu = activation(GELU_STEP_LOG2, GELU_RANGE_LOG2, &|x| {
        let k = (2.0 / std::f64::consts::PI).sqrt();
        0.5 * x * (1.0 + (k * (x + 0.044_715 * x * x * x)).tanh())
    });
    let table = |b: &mut StackArtifactBuilder, name: &str, v: TableValues<'_>| -> Result<()> {
        b.add_table(name, v).map_err(|e| invalid(e.to_string()))
    };
    table(&mut builder, "exp", TableValues::U32(&exp))?;
    table(&mut builder, "silu", TableValues::I32(&silu))?;
    table(&mut builder, "gelu", TableValues::I32(&gelu))?;
    if c.read == ReadScore::Lorentz {
        table(&mut builder, "arcosh", TableValues::U32(&arcosh_table()))?;
    }
    let bytes = builder.finish().map_err(|e| invalid(e.to_string()))?;
    Ok((
        bytes,
        json!({
            "method": {"quantizer": "round_to_nearest", "mlp_padded_from": c.mlp_hidden, "mlp_padded_to": mlp},
            "relative_rms_error": errors,
            "grid_code_worst_relative_error": code_errors,
        }),
    ))
}

/// The transformer control as a Llama checkpoint (RoPE rotate-half with theta
/// 10,000, RMSNorm with epsilon 1e-5, SwiGLU, tied embedding), for
/// [`crate::lut_export::export_llama`].
pub fn control_checkpoint(model: &StackModel, weights_sha256: String) -> Result<Checkpoint> {
    let c = &model.config;
    if c.arch != StackArch::Transformer {
        return Err(invalid("control_checkpoint takes the transformer control"));
    }
    let shape = LlamaShape {
        vocab: c.vocab_size,
        width: c.width,
        layers: c.layers(),
        heads: c.heads,
        kv_heads: c.heads,
        head_dim: c.head_width(),
        ffn: c.mlp_hidden,
        rope_theta: 10_000.0,
        rms_eps: RMS_EPSILON,
        tied_embeddings: true,
    };
    let get = |name: &str| -> Result<Tensor> {
        Ok(model
            .variables()
            .get(name)
            .ok_or_else(|| invalid(format!("the control has no tensor {name}")))?
            .as_tensor()
            .clone())
    };
    let mut tensors = BTreeMap::new();
    tensors.insert(
        "model.embed_tokens.weight".to_owned(),
        get("embedding.weight")?,
    );
    tensors.insert("model.norm.weight".to_owned(), get("final_norm.weight")?);
    for l in 0..c.layers() {
        let from = |suffix: &str| get(&format!("layers.{l:02}.{suffix}"));
        let to = |suffix: &str| format!("model.layers.{l}.{suffix}");
        tensors.insert(to("input_layernorm.weight"), from("attn_norm.weight")?);
        tensors.insert(
            to("post_attention_layernorm.weight"),
            from("mlp_norm.weight")?,
        );
        for part in ["q", "k", "v", "o"] {
            tensors.insert(
                to(&format!("self_attn.{part}_proj.weight")),
                from(&format!("attn.{part}.weight"))?,
            );
        }
        for part in ["gate", "up", "down"] {
            tensors.insert(
                to(&format!("mlp.{part}_proj.weight")),
                from(&format!("mlp.{part}.weight"))?,
            );
        }
    }
    Ok(Checkpoint {
        shape,
        tensors,
        weights_sha256,
    })
}

/// A float model whose weights are an artifact's own values: dequantized
/// matrices, grid-code scalars, integer biases and unit norm gains. `head` is
/// the artifact's output map (the final gain folded in), which the float
/// model cannot hold because it ties its head to the embedding. Scoring it
/// beside the float original and the integer engine splits the integer gap
/// into weight rounding and integer arithmetic.
pub struct GridReference {
    pub model: StackModel,
    pub head: Tensor,
}

impl GridReference {
    /// Logits [batch * time, vocabulary] through the artifact's head.
    pub fn logits(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        Ok(self
            .model
            .hidden(ids, batch, time)?
            .matmul(&self.head.t()?)?)
    }
}

fn lut_error(error: uor_r4_lut::LutError) -> crate::TrainingError {
    invalid(error.to_string())
}

/// Dequantized values of one packed matrix, with its shape.
fn packed_values<H: uor_r4_lut::format::Sections>(
    artifact: &uor_r4_lut::format::Container<H>,
    name: &str,
) -> Result<(Vec<f32>, usize, usize)> {
    let spec = artifact.matrix(name).map_err(lut_error)?;
    let values = crate::lut_export::dequantize_matrix(
        spec.rows,
        spec.cols,
        spec.exp_base,
        artifact.section(spec.nibbles),
        artifact.section(spec.scales),
    )?;
    Ok((values, spec.rows, spec.cols))
}

/// The first `rows x cols` block of a row-major `stride`-column matrix.
fn block(values: &[f32], stride: usize, rows: usize, cols: usize) -> Vec<f32> {
    (0..rows)
        .flat_map(|r| values[r * stride..r * stride + cols].iter().copied())
        .collect()
}

fn set(model: &StackModel, name: &str, values: Vec<f32>) -> Result<()> {
    let var = model
        .variables()
        .get(name)
        .ok_or_else(|| invalid(format!("the reference has no tensor {name}")))?;
    var.set(&Tensor::from_vec(
        values,
        var.as_tensor().shape(),
        var.as_tensor().device(),
    )?)?;
    Ok(())
}

/// The grid reference of a geometric stack's artifact.
pub fn stack_grid_reference(
    model: &StackModel,
    artifact: &uor_r4_lut::format::StackArtifact,
) -> Result<GridReference> {
    let c = model.config.clone();
    if c.arch != StackArch::Geometric {
        return Err(invalid("a stack grid reference needs a geometric stack"));
    }
    let reference = StackModel::new(c.clone(), model.device())?;
    let d = c.width;
    let matrix = |name: &str| -> Result<Vec<f32>> { Ok(packed_values(artifact, name)?.0) };
    let ints = |name: &str, exp: i32| -> Result<Vec<f32>> {
        Ok(artifact
            .table_i32(name)
            .map_err(lut_error)?
            .iter()
            .map(|&v| (f64::from(v) * 2f64.powi(exp)) as f32)
            .collect())
    };
    let codes = |name: &str| -> Result<Vec<f64>> {
        Ok(artifact
            .table_i16(name)
            .map_err(lut_error)?
            .iter()
            .map(|&v| grid_value(v))
            .collect())
    };
    let mlp = artifact.header.shape.mlp;
    set(&reference, "embedding.weight", matrix("embed")?)?;
    set(&reference, "final_norm.weight", vec![1.0; d])?;
    for (l, kind) in c.pattern.bytes().enumerate() {
        let t = |suffix: &str| format!("layers.{l:02}.{suffix}");
        let a = |part: &str| format!("l{l}.{part}");
        if kind == b'r' {
            set(&reference, &t("rec_norm.weight"), vec![1.0; d])?;
            set(&reference, &t("rec.in.weight"), matrix(&a("rec_in"))?)?;
            set(&reference, &t("rec.gate.weight"), matrix(&a("rec_gate"))?)?;
            set(&reference, &t("rec.out.weight"), matrix(&a("rec_out"))?)?;
            let taps = codes(&a("conv_taps"))?.iter().map(|&v| v as f32).collect();
            set(&reference, &t("rec.conv.weight"), taps)?;
            set(&reference, &t("rec.conv.bias"), ints(&a("conv_bias"), -16)?)?;
            set(&reference, &t("rec.gate.bias"), ints(&a("gate_bias"), -16)?)?;
            // The decay whose rate 8 softplus(-decay) is the grid code's value.
            let decay = codes(&a("decay_rate"))?
                .iter()
                .map(|&rate| -((rate / DECAY_EXPONENT).exp_m1().ln()) as f32)
                .collect();
            set(&reference, &t("rec.decay"), decay)?;
        } else {
            set(&reference, &t("read_norm.weight"), vec![1.0; d])?;
            for part in ["query", "key", "value", "null", "out"] {
                set(
                    &reference,
                    &t(&format!("read.{part}.weight")),
                    matrix(&a(part))?,
                )?;
            }
            set(
                &reference,
                &t("read.null.bias"),
                ints(&a("null_bias"), -16)?,
            )?;
            set(&reference, &t("read.age"), ints(&a("age"), -16)?)?;
            if c.read == ReadScore::Lorentz {
                let beta = codes(&a("beta"))?.iter().map(|&v| v.ln() as f32).collect();
                set(&reference, &t("read.log_beta"), beta)?;
                set(&reference, &t("read.offset"), ints(&a("offset"), -24)?)?;
            }
        }
        set(&reference, &t("mlp_norm.weight"), vec![1.0; d])?;
        for part in ["gate", "up"] {
            let values = block(&matrix(&a(part))?, d, c.mlp_hidden, d);
            set(&reference, &t(&format!("mlp.{part}.weight")), values)?;
        }
        let down = block(&matrix(&a("down"))?, mlp, d, c.mlp_hidden);
        set(&reference, &t("mlp.down.weight"), down)?;
    }
    let (head, rows, cols) = packed_values(artifact, "head")?;
    Ok(GridReference {
        head: Tensor::from_vec(head, (rows, cols), model.device())?,
        model: reference,
    })
}

/// The grid reference of the transformer control's Llama artifact.
pub fn control_grid_reference(
    model: &StackModel,
    artifact: &uor_r4_lut::format::Artifact,
) -> Result<GridReference> {
    let c = model.config.clone();
    if c.arch != StackArch::Transformer {
        return Err(invalid(
            "a control grid reference needs the transformer control",
        ));
    }
    let reference = StackModel::new(c.clone(), model.device())?;
    let d = c.width;
    let matrix = |name: &str| -> Result<Vec<f32>> { Ok(packed_values(artifact, name)?.0) };
    set(&reference, "embedding.weight", matrix("embed")?)?;
    set(&reference, "final_norm.weight", vec![1.0; d])?;
    for l in 0..c.layers() {
        let t = |suffix: &str| format!("layers.{l:02}.{suffix}");
        for norm in ["attn_norm.weight", "mlp_norm.weight"] {
            set(&reference, &t(norm), vec![1.0; d])?;
        }
        for part in ["q", "k", "v", "o"] {
            set(
                &reference,
                &t(&format!("attn.{part}.weight")),
                matrix(&format!("l{l}.{part}"))?,
            )?;
        }
        for part in ["gate", "up", "down"] {
            set(
                &reference,
                &t(&format!("mlp.{part}.weight")),
                matrix(&format!("l{l}.{part}"))?,
            )?;
        }
    }
    let (head, rows, cols) = packed_values(artifact, "head")?;
    Ok(GridReference {
        head: Tensor::from_vec(head, (rows, cols), model.device())?,
        model: reference,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_stack::StackConfig;
    use candle_core::Device;
    use uor_r4_lut::format::StackArtifact;
    use uor_r4_lut::stack::StackModel as IntegerStack;

    fn small(pattern: &str, read: ReadScore, rotation: bool) -> StackModel {
        let mut config = StackConfig::transformer_control(7);
        config.arch = StackArch::Geometric;
        config.vocab_size = 96;
        config.width = 64;
        config.heads = 2;
        config.mlp_hidden = 40;
        config.context = 24;
        config.pattern = pattern.to_owned();
        config.read = read;
        config.rotation = rotation;
        StackModel::new(config, &Device::Cpu).expect("small stack")
    }

    /// Perturb every tensor so the test does not sit at the initialization's
    /// identities (unit rotations, zero ages, unit gains).
    fn perturb(model: &StackModel, seed: u64) {
        let mut state = seed;
        let mut uniform = || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((state >> 11) as f64) / ((1u64 << 53) as f64) - 0.5
        };
        for (name, var) in model.variables() {
            let values = var
                .as_tensor()
                .flatten_all()
                .unwrap()
                .to_vec1::<f32>()
                .unwrap();
            // A wide embedding spreads the tied head's logits over several nats.
            let scale = if name == "embedding.weight" {
                0.3
            } else if name.ends_with("norm.weight") {
                0.4
            } else if name.contains(".age") || name.contains("bias") || name.contains("offset") {
                0.5
            } else if name.contains("conv.weight")
                || name.contains("decay")
                || name.contains("log_beta")
            {
                0.3
            } else {
                0.05
            };
            let changed: Vec<f32> = values
                .iter()
                .map(|v| v + (uniform() * 2.0 * scale) as f32)
                .collect();
            var.set(&Tensor::from_vec(changed, var.as_tensor().shape(), &Device::Cpu).unwrap())
                .unwrap();
        }
    }

    fn log_softmax(logits: &[f64]) -> Vec<f64> {
        let max = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let total: f64 = logits.iter().map(|v| (v - max).exp()).sum();
        logits.iter().map(|v| v - max - total.ln()).collect()
    }

    /// Over one sequence: the worst absolute log-probability gap between the
    /// integer engine and the grid reference, the worst target
    /// log-probability gap to the unrounded float model, and the widest
    /// log-probability spread of the reference.
    fn parity(pattern: &str, read: ReadScore, rotation: bool) -> (f64, f64, f64) {
        let model = small(pattern, read, rotation);
        perturb(&model, 3);
        let (bytes, _) = export_stack(&model, json!({"test": true})).unwrap();
        let artifact = StackArtifact::parse(bytes.clone()).unwrap();
        let reference = stack_grid_reference(&model, &artifact).unwrap();
        let integer = IntegerStack::from_artifact(StackArtifact::parse(bytes).unwrap()).unwrap();
        let ids: Vec<u32> = (0..model.config.context as u32)
            .map(|i| (i * 37 + 5) % 96)
            .collect();
        let time = ids.len();
        let logits = reference
            .logits(&ids, 1, time)
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        let original = model
            .forward(&ids, 1, time)
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        let mut session = integer.session();
        let (mut worst, mut nll_gap, mut spread) = (0f64, 0f64, 0f64);
        for (t, &id) in ids.iter().enumerate() {
            let got: Vec<f64> = session
                .step(id)
                .unwrap()
                .iter()
                .map(|&v| f64::from(v) / 65536.0)
                .collect();
            let want: Vec<f64> = logits[t].iter().map(|&v| f64::from(v)).collect();
            let float: Vec<f64> = original[t].iter().map(|&v| f64::from(v)).collect();
            let (got, want, float) = (log_softmax(&got), log_softmax(&want), log_softmax(&float));
            spread = spread.max(
                want.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                    - want.iter().copied().fold(f64::INFINITY, f64::min),
            );
            for (g, w) in got.iter().zip(&want) {
                worst = worst.max((g - w).abs());
            }
            let target = ids[(t + 1) % time] as usize;
            nll_gap = nll_gap.max((got[target] - float[target]).abs());
        }
        (worst, nll_gap, spread)
    }

    #[test]
    fn integer_stack_matches_its_grid_reference() {
        for (pattern, read, rotation) in [
            ("rarr", ReadScore::Lorentz, true),
            ("rarr", ReadScore::Dot, true),
            ("ra", ReadScore::Lorentz, false),
            ("aa", ReadScore::Dot, false),
        ] {
            let (worst, nll_gap, spread) = parity(pattern, read, rotation);
            assert!(spread > 5.0, "the test logits are too flat: {spread}");
            assert!(
                worst < 0.005,
                "{pattern} {read:?} rotation {rotation}: worst log-probability gap {worst}"
            );
            // Weight rounding moves the model, but not wildly.
            assert!(nll_gap < 2.0, "{pattern} {read:?}: float gap {nll_gap}");
        }
    }

    #[test]
    fn grid_codes_round_to_the_nearest_grid_value() {
        for value in [1e-9, 0.003, -0.4, 1.0, 17.3, -250.0] {
            let code = grid_code(value).unwrap();
            let back = grid_value(code);
            assert!(
                ((back - value) / value).abs() <= 1.0 / 32.0 + 1e-12,
                "{value} -> {back}"
            );
        }
        assert_eq!(grid_code(0.0).unwrap(), 0);
        assert!(grid_code(f64::NAN).is_err());
    }

    #[test]
    fn the_control_renames_to_a_llama_checkpoint() {
        let control = StackModel::new(StackConfig::transformer_control(3), &Device::Cpu).unwrap();
        let checkpoint = control_checkpoint(&control, "none".to_owned()).unwrap();
        assert_eq!(checkpoint.tensors.len(), control.variables().len());
        assert_eq!(checkpoint.shape.ffn, 768);
        assert!(checkpoint
            .tensors
            .contains_key("model.layers.5.mlp.down_proj.weight"));
        assert!(export_stack(&control, json!({})).is_err());
    }
}
