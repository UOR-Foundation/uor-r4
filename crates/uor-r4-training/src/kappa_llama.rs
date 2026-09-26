//! Curvature-homotopy conversion of a Llama-architecture checkpoint's attention
//! to hyperbolic (Lorentz) attention.
//!
//! This is an offline floating-point training and measurement tool, never a
//! serving path. It loads a Hugging Face Llama checkpoint (for example
//! SmolLM2-135M/360M-Instruct), keeps every checkpoint weight, and replaces
//! only the attention score of each head.
//!
//! Scores for one head, with head width `r`, curvature `kappa = eps^2` and
//! scaled coordinates `a = eps q`, `b = eps k` lifted to the unit hyperboloid
//! `x -> (sqrt(1 + |x|^2), x)` whose geodesic distance is `d`:
//!
//! - `Dot`: `<q, k> / sqrt(r)`, the checkpoint's own score.
//! - `KeyNorm`: `(|b|^2 - d(a, b)^2) / (2 eps^2 sqrt(r))`.
//! - `Intrinsic`: `(d(o, b)^2 - d(a, b)^2) / (2 eps^2 sqrt(r))`, where `o` is
//!   the hyperboloid origin, so `d(o, b)` is the key's hyperbolic radius.
//!
//! Near the origin `d(a, b)^2 = |a - b|^2 + O(eps^4)`, and the polarization
//! identity `2 <q, k> = |k|^2 + |q|^2 - |q - k|^2` makes both curved scores equal
//! `<q, k> / sqrt(r)` minus a per-query constant, which softmax ignores. The
//! conversion therefore starts at the checkpoint's own attention (to
//! `O(kappa)`) and curvature grows only where training rewards it. RoPE
//! rotates spatial coordinates only, which is an isometry of the hyperboloid,
//! so both curved scores stay relative-position scores.
//!
//! Every head also carries a learnable log inverse temperature (`log_beta`), in
//! every score kind, so a `Dot` control has the same non-curvature freedom.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use candle_core::{Device, Tensor, Var, D};
use safetensors::{Dtype as SafeDtype, SafeTensors};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{invalid, Result};

/// Name of the per-head log curvature scale (`kappa = exp(2 log_eps)`).
pub const LOG_EPS: &str = "curvature.log_eps";
/// Name of the per-head log inverse temperature.
pub const LOG_BETA: &str = "curvature.log_beta";
/// Below this argument the squared arcosh uses its four-term series (relative
/// truncation error below 5e-8 here), which has finite gradients at zero; above
/// it the closed form's f32 rounding error is below 1e-6 relative.
pub const SERIES_LIMIT: f32 = 0.05;

/// Attention score used by every converted head.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreKind {
    Dot,
    KeyNorm,
    Intrinsic,
}

impl ScoreKind {
    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "dot" => Ok(Self::Dot),
            "key_norm" => Ok(Self::KeyNorm),
            "intrinsic" => Ok(Self::Intrinsic),
            _ => Err(invalid(format!(
                "score must be dot, key_norm or intrinsic, not {text}"
            ))),
        }
    }

    pub fn is_curved(self) -> bool {
        self != Self::Dot
    }
}

/// Architecture of a Hugging Face Llama checkpoint, as far as this tool uses it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LlamaShape {
    pub vocab: usize,
    pub width: usize,
    pub layers: usize,
    pub heads: usize,
    pub kv_heads: usize,
    pub head_dim: usize,
    pub ffn: usize,
    pub rope_theta: f64,
    pub rms_eps: f64,
    pub tied_embeddings: bool,
}

fn config_usize(config: &Value, key: &str) -> Result<usize> {
    config
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| invalid(format!("config.json needs a non-negative integer {key}")))
}

fn config_f64(config: &Value, key: &str, default: f64) -> Result<f64> {
    match config.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(value) => value
            .as_f64()
            .filter(|value| value.is_finite() && *value > 0.0)
            .ok_or_else(|| invalid(format!("config.json {key} must be a positive number"))),
    }
}

fn config_flag(config: &Value, key: &str) -> Result<bool> {
    match config.get(key) {
        None | Some(Value::Null) => Ok(false),
        Some(Value::Bool(flag)) => Ok(*flag),
        Some(_) => Err(invalid(format!("config.json {key} must be a boolean"))),
    }
}

impl LlamaShape {
    /// Parse and validate a Hugging Face `config.json`. Features this tool does
    /// not implement (biases, RoPE scaling, interleaved RoPE, other
    /// activations) are refused rather than silently ignored.
    pub fn from_config(config: &Value) -> Result<Self> {
        let heads = config_usize(config, "num_attention_heads")?;
        let width = config_usize(config, "hidden_size")?;
        let kv_heads = match config.get("num_key_value_heads") {
            None | Some(Value::Null) => heads,
            Some(_) => config_usize(config, "num_key_value_heads")?,
        };
        if config.get("hidden_act").and_then(Value::as_str) != Some("silu") {
            return Err(invalid("only silu Llama checkpoints are supported"));
        }
        for key in ["attention_bias", "mlp_bias", "rope_interleaved"] {
            if config_flag(config, key)? {
                return Err(invalid(format!("{key}=true is not supported")));
            }
        }
        if !matches!(config.get("rope_scaling"), None | Some(Value::Null)) {
            return Err(invalid("rope_scaling is not supported"));
        }
        let shape = Self {
            vocab: config_usize(config, "vocab_size")?,
            width,
            layers: config_usize(config, "num_hidden_layers")?,
            heads,
            kv_heads,
            head_dim: width.checked_div(heads).unwrap_or(0),
            ffn: config_usize(config, "intermediate_size")?,
            rope_theta: config_f64(config, "rope_theta", 10_000.0)?,
            rms_eps: config_f64(config, "rms_norm_eps", 1e-6)?,
            tied_embeddings: config_flag(config, "tie_word_embeddings")?,
        };
        shape.validate()?;
        Ok(shape)
    }

    pub fn validate(&self) -> Result<()> {
        let positive = [
            self.vocab,
            self.width,
            self.layers,
            self.heads,
            self.kv_heads,
            self.head_dim,
            self.ffn,
        ];
        if positive.contains(&0)
            || self.width != self.heads * self.head_dim
            || !self.heads.is_multiple_of(self.kv_heads)
            || !self.head_dim.is_multiple_of(2)
            || self.vocab > 1 << 20
            || self.width > 1 << 14
            || self.layers > 256
        {
            return Err(invalid("inconsistent or unsupported Llama shape"));
        }
        Ok(())
    }

    /// Every checkpoint tensor this tool requires, with its row-major shape.
    pub fn expected_tensors(&self) -> BTreeMap<String, Vec<usize>> {
        let q_rows = self.heads * self.head_dim;
        let kv_rows = self.kv_heads * self.head_dim;
        let mut shapes = BTreeMap::from([
            (
                "model.embed_tokens.weight".to_owned(),
                vec![self.vocab, self.width],
            ),
            ("model.norm.weight".to_owned(), vec![self.width]),
        ]);
        if !self.tied_embeddings {
            shapes.insert("lm_head.weight".to_owned(), vec![self.vocab, self.width]);
        }
        for layer in 0..self.layers {
            let prefix = format!("model.layers.{layer}");
            for name in ["input_layernorm", "post_attention_layernorm"] {
                shapes.insert(format!("{prefix}.{name}.weight"), vec![self.width]);
            }
            let attention = [
                ("q_proj", vec![q_rows, self.width]),
                ("k_proj", vec![kv_rows, self.width]),
                ("v_proj", vec![kv_rows, self.width]),
                ("o_proj", vec![self.width, q_rows]),
            ];
            for (name, shape) in attention {
                shapes.insert(format!("{prefix}.self_attn.{name}.weight"), shape);
            }
            for name in ["gate_proj", "up_proj"] {
                shapes.insert(
                    format!("{prefix}.mlp.{name}.weight"),
                    vec![self.ffn, self.width],
                );
            }
            shapes.insert(
                format!("{prefix}.mlp.down_proj.weight"),
                vec![self.width, self.ffn],
            );
        }
        shapes
    }
}

fn decode_values(name: &str, dtype: SafeDtype, bytes: &[u8]) -> Result<Vec<f32>> {
    let values: Vec<f32> = match dtype {
        SafeDtype::F32 => bytes
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect(),
        // bfloat16 is the upper half of an IEEE f32.
        SafeDtype::BF16 => bytes
            .chunks_exact(2)
            .map(|b| f32::from_bits(u32::from(u16::from_le_bytes([b[0], b[1]])) << 16))
            .collect(),
        other => {
            return Err(invalid(format!(
                "{name}: dtype {other:?} is not supported (use F32 or BF16)"
            )))
        }
    };
    if values.iter().any(|value| !value.is_finite()) {
        return Err(invalid(format!("{name}: nonfinite coefficient")));
    }
    Ok(values)
}

/// A checkpoint read into f32 tensors, with its shape and weights-file digest.
pub struct Checkpoint {
    pub shape: LlamaShape,
    pub tensors: BTreeMap<String, Tensor>,
    pub weights_sha256: String,
}

/// Safe in-memory safetensors deserialization of `DIR/config.json` and
/// `DIR/model.safetensors`. No pickle, mmap, network or unsafe code. The tensor
/// inventory must equal the expected Llama inventory exactly.
pub fn load_checkpoint(directory: &Path, device: &Device) -> Result<Checkpoint> {
    let config: Value = serde_json::from_slice(&fs::read(directory.join("config.json"))?)?;
    let shape = LlamaShape::from_config(&config)?;
    let weights_path = directory.join("model.safetensors");
    let weights_sha256 = crate::sha256_file(&weights_path)?;
    let bytes = fs::read(&weights_path)?;
    let file = SafeTensors::deserialize(&bytes)?;
    let expected = shape.expected_tensors();
    let names: BTreeSet<String> = file.names().into_iter().map(str::to_owned).collect();
    let wanted: BTreeSet<String> = expected.keys().cloned().collect();
    if names != wanted {
        let missing: Vec<_> = wanted.difference(&names).take(4).collect();
        let extra: Vec<_> = names.difference(&wanted).take(4).collect();
        return Err(invalid(format!(
            "tensor inventory differs from the Llama shape: missing {missing:?}, unexpected {extra:?}"
        )));
    }
    let mut tensors = BTreeMap::new();
    for (name, dims) in expected {
        let view = file.tensor(&name)?;
        if view.shape() != dims.as_slice() {
            return Err(invalid(format!(
                "{name}: shape {:?}, expected {dims:?}",
                view.shape()
            )));
        }
        let values = decode_values(&name, view.dtype(), view.data())?;
        tensors.insert(name, Tensor::from_vec(values, dims.as_slice(), device)?);
    }
    Ok(Checkpoint {
        shape,
        tensors,
        weights_sha256,
    })
}

/// Squared hyperbolic distance `arcosh(1 + z)^2` for `z >= 0` (negative inputs,
/// which only arise from rounding, are clamped). The series
/// `2z - z^2/3 + 4z^3/45 - z^4/35` (from inverting `z = cosh(d) - 1`) is used
/// below [`SERIES_LIMIT`]; the closed form's argument is clamped at the limit
/// so its unused branch never produces a nonfinite gradient.
pub fn arcosh1p_squared(z: &Tensor) -> Result<Tensor> {
    let z = z.clamp(0f32, f32::MAX)?;
    let small = z.le(SERIES_LIMIT)?;
    let wide = z.clamp(SERIES_LIMIT, f32::MAX)?;
    let root = wide.mul(&wide.affine(1.0, 2.0)?)?.sqrt()?;
    let exact = wide.affine(1.0, 1.0)?.add(&root)?.log()?.sqr()?;
    let cubic = z.affine(-1.0 / 35.0, 4.0 / 45.0)?;
    let quadratic = z.mul(&cubic)?.affine(1.0, -1.0 / 3.0)?;
    let series = z.mul(&z.mul(&quadratic)?.affine(1.0, 2.0)?)?;
    Ok(small.where_cond(&series, &exact)?)
}

/// Scores for one layer. `query` and `key` are post-RoPE `(batch, heads, time,
/// head_dim)`; `log_eps` and `log_beta` are `(1, heads, 1, 1)`. Returns
/// `(batch, heads, time, time)` before the causal mask.
pub fn head_scores(
    kind: ScoreKind,
    query: &Tensor,
    key: &Tensor,
    log_eps: Option<&Tensor>,
    log_beta: &Tensor,
) -> Result<Tensor> {
    let head_dim = query.dim(D::Minus1)? as f64;
    let beta = log_beta.exp()?;
    if kind == ScoreKind::Dot {
        let key_t = key.transpose(2, 3)?.contiguous()?;
        let scores = query.matmul(&key_t)?.affine(1.0 / head_dim.sqrt(), 0.0)?;
        return Ok(scores.broadcast_mul(&beta)?);
    }
    let log_eps = log_eps.ok_or_else(|| invalid("a curved score needs log_eps"))?;
    let eps = log_eps.exp()?;
    let a = query.broadcast_mul(&eps)?;
    let b = key.broadcast_mul(&eps)?;
    let a2 = a.sqr()?.sum_keepdim(3)?;
    let b2 = b.sqr()?.sum_keepdim(3)?.transpose(2, 3)?.contiguous()?;
    // x0 - 1 = |x|^2 / (1 + sqrt(1 + |x|^2)), free of cancellation near the origin.
    let lift = |s: &Tensor| -> Result<Tensor> {
        Ok(s.div(&s.affine(1.0, 1.0)?.sqrt()?.affine(1.0, 1.0)?)?)
    };
    let xm = lift(&a2)?;
    let ym = lift(&b2)?;
    let ab = a.matmul(&b.transpose(2, 3)?.contiguous()?)?;
    // -<x, y>_L - 1 = (x0 - 1) + (y0 - 1) + (x0 - 1)(y0 - 1) - <a, b>.
    let z = xm
        .broadcast_add(&ym)?
        .broadcast_add(&xm.broadcast_mul(&ym)?)?
        .sub(&ab)?;
    let distance_sq = arcosh1p_squared(&z)?;
    let bias = match kind {
        ScoreKind::KeyNorm => b2,
        ScoreKind::Intrinsic => arcosh1p_squared(&ym)?,
        ScoreKind::Dot => return Err(invalid("unreachable dot branch")),
    };
    let denominator = log_eps
        .affine(2.0, 0.0)?
        .exp()?
        .affine(2.0 * head_dim.sqrt(), 0.0)?;
    Ok(bias
        .broadcast_sub(&distance_sq)?
        .broadcast_div(&denominator)?
        .broadcast_mul(&beta)?)
}

/// Half-split RoPE (Hugging Face Llama layout) on `(batch, heads, time, head_dim)`.
pub fn rope(input: &Tensor, cosine: &Tensor, sine: &Tensor) -> Result<Tensor> {
    let half = input.dim(D::Minus1)? / 2;
    let first = input.narrow(3, 0, half)?;
    let second = input.narrow(3, half, half)?;
    let a = first
        .broadcast_mul(cosine)?
        .sub(&second.broadcast_mul(sine)?)?;
    let b = second
        .broadcast_mul(cosine)?
        .add(&first.broadcast_mul(sine)?)?;
    Ok(Tensor::cat(&[&a, &b], 3)?)
}

/// RoPE tables `(1, 1, time, head_dim / 2)` for positions `offset..offset + time`.
pub fn rope_tables(
    theta: f64,
    head_dim: usize,
    offset: usize,
    time: usize,
    device: &Device,
) -> Result<(Tensor, Tensor)> {
    let half = head_dim / 2;
    let mut cosine = Vec::with_capacity(time * half);
    let mut sine = Vec::with_capacity(time * half);
    for position in offset..offset + time {
        for pair in 0..half {
            let inverse = 1.0 / theta.powf((2 * pair) as f64 / head_dim as f64);
            let angle = position as f64 * inverse;
            cosine.push(angle.cos() as f32);
            sine.push(angle.sin() as f32);
        }
    }
    Ok((
        Tensor::from_vec(cosine, (1, 1, time, half), device)?,
        Tensor::from_vec(sine, (1, 1, time, half), device)?,
    ))
}

/// Which checkpoint weights become trainable in addition to the per-head
/// curvature and temperature scalars.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trainable {
    /// Only `log_eps` (curved scores) and `log_beta`.
    Scalars,
    /// Scalars plus every layer's query and key projections.
    QueryKey,
}

impl Trainable {
    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "scalars" => Ok(Self::Scalars),
            "query_key" => Ok(Self::QueryKey),
            _ => Err(invalid(format!(
                "trainable must be scalars or query_key, not {text}"
            ))),
        }
    }
}

/// A checkpoint whose attention scores are replaced by [`ScoreKind`].
pub struct KappaLlama {
    shape: LlamaShape,
    score: ScoreKind,
    frozen: BTreeMap<String, Tensor>,
    variables: BTreeMap<String, Var>,
    device: Device,
}

impl KappaLlama {
    /// Wrap loaded checkpoint tensors. Curved scores start at `init_log_eps` in
    /// every head (`kappa = exp(2 init_log_eps)`); every `log_beta` starts at 0.
    pub fn new(
        checkpoint: Checkpoint,
        score: ScoreKind,
        init_log_eps: f32,
        trainable: Trainable,
        device: &Device,
    ) -> Result<Self> {
        let Checkpoint { shape, tensors, .. } = checkpoint;
        shape.validate()?;
        if !init_log_eps.is_finite() || !(-20.0..=5.0).contains(&init_log_eps) {
            return Err(invalid("init_log_eps must be finite and in [-20, 5]"));
        }
        let expected = shape.expected_tensors();
        for (name, dims) in &expected {
            let tensor = tensors
                .get(name)
                .ok_or_else(|| invalid(format!("missing checkpoint tensor {name}")))?;
            if tensor.dims() != dims.as_slice() {
                return Err(invalid(format!("{name}: wrong shape")));
            }
        }
        let mut frozen = BTreeMap::new();
        let mut variables = BTreeMap::new();
        for (name, tensor) in tensors {
            if !expected.contains_key(&name) {
                return Err(invalid(format!("unexpected checkpoint tensor {name}")));
            }
            let projection = name.ends_with("self_attn.q_proj.weight")
                || name.ends_with("self_attn.k_proj.weight");
            if trainable == Trainable::QueryKey && projection {
                variables.insert(name, Var::from_tensor(&tensor.to_device(device)?)?);
            } else {
                frozen.insert(name, tensor.to_device(device)?.detach());
            }
        }
        let grid = (shape.layers, shape.heads);
        variables.insert(
            LOG_BETA.to_owned(),
            Var::from_tensor(&Tensor::zeros(grid, candle_core::DType::F32, device)?)?,
        );
        if score.is_curved() {
            variables.insert(
                LOG_EPS.to_owned(),
                Var::from_tensor(&Tensor::full(init_log_eps, grid, device)?)?,
            );
        }
        Ok(Self {
            shape,
            score,
            frozen,
            variables,
            device: device.clone(),
        })
    }

    pub fn shape(&self) -> &LlamaShape {
        &self.shape
    }

    pub fn score(&self) -> ScoreKind {
        self.score
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Trainable variables by name (for the optimizer and checkpoints).
    pub fn variables(&self) -> &BTreeMap<String, Var> {
        &self.variables
    }

    fn tensor(&self, name: &str, detached: bool) -> Result<Tensor> {
        if let Some(variable) = self.variables.get(name) {
            let tensor = variable.as_tensor();
            return Ok(if detached {
                tensor.detach()
            } else {
                tensor.clone()
            });
        }
        self.frozen
            .get(name)
            .cloned()
            .ok_or_else(|| invalid(format!("missing tensor {name}")))
    }

    fn linear(&self, input: &Tensor, name: &str, detached: bool) -> Result<Tensor> {
        Ok(input.matmul(&self.tensor(name, detached)?.t()?)?)
    }

    fn rms_norm(&self, input: &Tensor, name: &str) -> Result<Tensor> {
        // Primitive composition: Candle's fused RMSNorm is inference-only.
        let denominator = input
            .sqr()?
            .mean_keepdim(D::Minus1)?
            .affine(1.0, self.shape.rms_eps)?
            .sqrt()?;
        Ok(input
            .broadcast_div(&denominator)?
            .broadcast_mul(&self.tensor(name, true)?)?)
    }

    /// Per-head scalars of one layer as `(1, heads, 1, 1)`.
    fn layer_scalars(&self, name: &str, layer: usize, detached: bool) -> Result<Tensor> {
        Ok(self.tensor(name, detached)?.narrow(0, layer, 1)?.reshape((
            1,
            self.shape.heads,
            1,
            1,
        ))?)
    }

    /// Causal logits `(batch, time, vocab)` for `batch` rows of `time` tokens.
    /// `detached` evaluates without recording a backward graph.
    pub fn forward(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        detached: bool,
    ) -> Result<Tensor> {
        let s = &self.shape;
        if batch == 0
            || time == 0
            || ids.len() != batch * time
            || ids.iter().any(|&token| token as usize >= s.vocab)
        {
            return Err(invalid("forward token bounds"));
        }
        let (cosine, sine) = rope_tables(s.rope_theta, s.head_dim, 0, time, &self.device)?;
        let causal: Vec<u8> = (0..time)
            .flat_map(|row| (0..time).map(move |column| u8::from(column > row)))
            .collect();
        let mask = Tensor::from_vec(causal, (1, 1, time, time), &self.device)?
            .broadcast_as((batch, s.heads, time, time))?;
        let excluded = Tensor::full(
            f32::NEG_INFINITY,
            (batch, s.heads, time, time),
            &self.device,
        )?;
        let ids = Tensor::new(ids, &self.device)?;
        let mut state = self
            .tensor("model.embed_tokens.weight", true)?
            .index_select(&ids, 0)?;
        let group = s.heads / s.kv_heads;
        for layer in 0..s.layers {
            let prefix = format!("model.layers.{layer}");
            let normalized = self.rms_norm(&state, &format!("{prefix}.input_layernorm.weight"))?;
            let project = |name: &str, heads: usize| -> Result<Tensor> {
                Ok(self
                    .linear(
                        &normalized,
                        &format!("{prefix}.self_attn.{name}.weight"),
                        detached,
                    )?
                    .reshape((batch, time, heads, s.head_dim))?
                    .transpose(1, 2)?
                    .contiguous()?)
            };
            let repeat = |tensor: Tensor| -> Result<Tensor> {
                if group == 1 {
                    return Ok(tensor);
                }
                Ok(tensor
                    .unsqueeze(2)?
                    .broadcast_as((batch, s.kv_heads, group, time, s.head_dim))?
                    .contiguous()?
                    .reshape((batch, s.heads, time, s.head_dim))?)
            };
            let query = rope(&project("q_proj", s.heads)?, &cosine, &sine)?;
            let key = repeat(rope(&project("k_proj", s.kv_heads)?, &cosine, &sine)?)?;
            let value = repeat(project("v_proj", s.kv_heads)?)?;
            let log_eps = if self.score.is_curved() {
                Some(self.layer_scalars(LOG_EPS, layer, detached)?)
            } else {
                None
            };
            let log_beta = self.layer_scalars(LOG_BETA, layer, detached)?;
            let scores = head_scores(self.score, &query, &key, log_eps.as_ref(), &log_beta)?;
            let masked = mask.where_cond(&excluded, &scores)?;
            let probability = candle_nn::ops::softmax(&masked, 3)?;
            let attended = probability
                .matmul(&value)?
                .transpose(1, 2)?
                .contiguous()?
                .reshape((batch * time, s.width))?;
            state = state.add(&self.linear(
                &attended,
                &format!("{prefix}.self_attn.o_proj.weight"),
                true,
            )?)?;
            let normalized =
                self.rms_norm(&state, &format!("{prefix}.post_attention_layernorm.weight"))?;
            let gate = self
                .linear(&normalized, &format!("{prefix}.mlp.gate_proj.weight"), true)?
                .silu()?;
            let up = self.linear(&normalized, &format!("{prefix}.mlp.up_proj.weight"), true)?;
            state = state.add(&self.linear(
                &gate.mul(&up)?,
                &format!("{prefix}.mlp.down_proj.weight"),
                true,
            )?)?;
        }
        let hidden = self.rms_norm(&state, "model.norm.weight")?;
        let head = if s.tied_embeddings {
            "model.embed_tokens.weight"
        } else {
            "lm_head.weight"
        };
        Ok(self
            .linear(&hidden, head, true)?
            .reshape((batch, time, s.vocab))?)
    }

    /// Curvature `kappa = exp(2 log_eps)` per layer and head (empty for `Dot`).
    pub fn curvature(&self) -> Result<Vec<Vec<f32>>> {
        match self.variables.get(LOG_EPS) {
            None => Ok(Vec::new()),
            Some(log_eps) => Ok(log_eps.as_tensor().affine(2.0, 0.0)?.exp()?.to_vec2()?),
        }
    }

    /// Inverse temperature `exp(log_beta)` per layer and head.
    pub fn temperature(&self) -> Result<Vec<Vec<f32>>> {
        Ok(self.tensor(LOG_BETA, true)?.exp()?.to_vec2()?)
    }

    /// Overwrite every head's `log_eps` (probes and tests).
    pub fn set_log_eps(&self, value: f32) -> Result<()> {
        let variable = self
            .variables
            .get(LOG_EPS)
            .ok_or_else(|| invalid("a dot model has no curvature"))?;
        variable.set(&Tensor::full(value, variable.shape(), &self.device)?)?;
        Ok(())
    }

    /// Project every head's `log_eps` onto `[floor, inf)`. An exact flat-limit
    /// start can sit in the flat basin, where learned curvature never grows; a
    /// rising floor anneals the heads into the curved regime while the rest of
    /// the model adapts.
    pub fn floor_log_eps(&self, floor: f32) -> Result<()> {
        if !floor.is_finite() {
            return Err(invalid("curvature floor must be finite"));
        }
        let variable = self
            .variables
            .get(LOG_EPS)
            .ok_or_else(|| invalid("a dot model has no curvature"))?;
        variable.set(&variable.as_tensor().maximum(floor)?)?;
        Ok(())
    }
}

/// Mean next-token negative log-likelihood (nats) of `(batch, time, vocab)`
/// logits against `targets` of length `batch * time`.
pub fn next_token_nll(logits: &Tensor, targets: &[u32]) -> Result<Tensor> {
    let (batch, time, vocab) = logits.dims3()?;
    if targets.len() != batch * time || targets.iter().any(|&t| t as usize >= vocab) {
        return Err(invalid("target bounds"));
    }
    let rows = logits.reshape((batch * time, vocab))?;
    let targets = Tensor::new(targets, logits.device())?.unsqueeze(1)?;
    Ok(candle_nn::ops::log_softmax(&rows, 1)?
        .gather(&targets, 1)?
        .mean_all()?
        .neg()?)
}

/// Mean per-position `KL(teacher || student)` in nats. The teacher is detached.
pub fn distillation_kl(student: &Tensor, teacher: &Tensor) -> Result<Tensor> {
    if student.dims() != teacher.dims() {
        return Err(invalid("student and teacher logits differ in shape"));
    }
    let vocab = student.dim(D::Minus1)?;
    let rows = student.elem_count() / vocab;
    let teacher_log = candle_nn::ops::log_softmax(&teacher.detach().reshape((rows, vocab))?, 1)?;
    let student_log = candle_nn::ops::log_softmax(&student.reshape((rows, vocab))?, 1)?;
    Ok(teacher_log
        .exp()?
        .mul(&teacher_log.sub(&student_log)?)?
        .sum(1)?
        .mean_all()?)
}

/// Largest absolute logit difference between two equally shaped logit tensors.
pub fn max_abs_difference(a: &Tensor, b: &Tensor) -> Result<f32> {
    Ok(a.sub(b)?.abs()?.flatten_all()?.max(0)?.to_scalar::<f32>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::backprop::GradStore;

    fn tiny_shape() -> LlamaShape {
        LlamaShape {
            vocab: 40,
            width: 32,
            layers: 2,
            heads: 4,
            kv_heads: 2,
            head_dim: 8,
            ffn: 48,
            rope_theta: 10_000.0,
            rms_eps: 1e-5,
            tied_embeddings: true,
        }
    }

    /// Deterministic pseudo-random checkpoint with trained-like magnitudes.
    fn tiny_checkpoint(device: &Device) -> Checkpoint {
        let shape = tiny_shape();
        let mut seed = 0x9e37_79b9_7f4a_7c15u64;
        let mut tensors = BTreeMap::new();
        for (name, dims) in shape.expected_tensors() {
            let count: usize = dims.iter().product();
            let norm = name.ends_with("norm.weight");
            let scale = if name.contains("q_proj") || name.contains("k_proj") {
                0.6
            } else {
                0.25
            };
            let values: Vec<f32> = (0..count)
                .map(|_| {
                    seed = seed
                        .wrapping_mul(6_364_136_223_846_793_005)
                        .wrapping_add(1_442_695_040_888_963_407);
                    let unit = ((seed >> 40) as f32) / ((1u64 << 24) as f32) - 0.5;
                    if norm {
                        1.0 + 0.2 * unit
                    } else {
                        scale * unit
                    }
                })
                .collect();
            tensors.insert(
                name,
                Tensor::from_vec(values, dims.as_slice(), device).expect("tensor"),
            );
        }
        Checkpoint {
            shape,
            tensors,
            weights_sha256: String::new(),
        }
    }

    fn ids(batch: usize, time: usize) -> Vec<u32> {
        (0..batch * time)
            .map(|i| ((i * 7 + 3) % 40) as u32)
            .collect()
    }

    fn model(score: ScoreKind, log_eps: f32, trainable: Trainable) -> KappaLlama {
        let device = Device::Cpu;
        KappaLlama::new(tiny_checkpoint(&device), score, log_eps, trainable, &device)
            .expect("model")
    }

    #[test]
    fn config_parsing_accepts_smollm2_and_refuses_unsupported_features() {
        let smollm2: Value = serde_json::json!({
            "hidden_act": "silu", "hidden_size": 576, "intermediate_size": 1536,
            "num_attention_heads": 9, "num_hidden_layers": 30, "num_key_value_heads": 3,
            "rms_norm_eps": 1e-5, "rope_theta": 100000, "tie_word_embeddings": true,
            "vocab_size": 49152, "attention_bias": false, "mlp_bias": false,
            "rope_interleaved": false, "rope_scaling": null
        });
        let shape = LlamaShape::from_config(&smollm2).expect("SmolLM2-135M shape");
        assert_eq!((shape.head_dim, shape.kv_heads), (64, 3));
        let params: usize = shape
            .expected_tensors()
            .values()
            .map(|dims| dims.iter().product::<usize>())
            .sum();
        assert_eq!(params, 134_515_008);
        let mut biased = smollm2.clone();
        biased["attention_bias"] = Value::Bool(true);
        assert!(LlamaShape::from_config(&biased).is_err());
        let mut scaled = smollm2;
        scaled["rope_scaling"] = serde_json::json!({"type": "linear", "factor": 2.0});
        assert!(LlamaShape::from_config(&scaled).is_err());
    }

    #[test]
    fn arcosh_series_and_closed_form_agree_at_the_switch() {
        let device = Device::Cpu;
        let points = [0.0f32, 1e-6, 1e-3, 0.0499, 0.0501, 0.5, 3.0];
        let z = Tensor::new(&points, &device).expect("z");
        let got: Vec<f32> = arcosh1p_squared(&z)
            .expect("arcosh")
            .to_vec1()
            .expect("vec");
        for (value, zz) in got.iter().zip(points) {
            let want = (1.0 + f64::from(zz)).acosh().powi(2);
            assert!(
                (f64::from(*value) - want).abs() <= 2e-6 * want.max(1e-6),
                "z={zz}: {value} vs {want}"
            );
        }
    }

    #[test]
    fn curved_scores_converge_to_dot_attention_as_curvature_vanishes() {
        let dot = model(ScoreKind::Dot, 0.0, Trainable::Scalars);
        let reference = dot.forward(&ids(2, 12), 2, 12, true).expect("dot");
        for score in [ScoreKind::KeyNorm, ScoreKind::Intrinsic] {
            let curved = model(score, -8.0, Trainable::Scalars);
            let mut errors = Vec::new();
            for log_eps in [-8.0f32, -6.0, -4.0, -1.0] {
                curved.set_log_eps(log_eps).expect("set");
                let logits = curved.forward(&ids(2, 12), 2, 12, true).expect("curved");
                errors.push(max_abs_difference(&logits, &reference).expect("diff"));
            }
            assert!(errors[0] < 2e-4, "{score:?} flat limit: {errors:?}");
            assert!(errors[1] < 2e-3, "{score:?}: {errors:?}");
            assert!(
                errors[3] > 10.0 * errors[1],
                "{score:?} curvature must matter: {errors:?}"
            );
        }
    }

    #[test]
    fn curved_scores_depend_on_relative_rope_position_only() {
        let device = Device::Cpu;
        let values: Vec<f32> = (0..2 * 3 * 8)
            .map(|i| ((i * 37 % 23) as f32 - 11.0) / 4.0)
            .collect();
        let base = Tensor::from_vec(values, (1, 2, 3, 8), &device).expect("base");
        let log_eps = Tensor::full(-0.5f32, (1, 2, 1, 1), &device).expect("eps");
        let log_beta = Tensor::zeros((1, 2, 1, 1), candle_core::DType::F32, &device).expect("beta");
        for kind in [ScoreKind::KeyNorm, ScoreKind::Intrinsic] {
            let mut rows = Vec::new();
            for offset in [0usize, 17] {
                let (cosine, sine) = rope_tables(1e4, 8, offset, 3, &device).expect("rope");
                let q = rope(&base, &cosine, &sine).expect("q");
                let k = rope(&base.affine(0.5, 0.25).expect("k"), &cosine, &sine).expect("k");
                let scores = head_scores(kind, &q, &k, Some(&log_eps), &log_beta).expect("scores");
                rows.push(scores);
            }
            let shift = max_abs_difference(&rows[0], &rows[1]).expect("diff");
            assert!(
                shift < 1e-4,
                "{kind:?}: a common RoPE offset changed scores by {shift}"
            );
        }
    }

    #[test]
    fn language_loss_reaches_curvature_and_query_key_weights() {
        let curved = model(ScoreKind::Intrinsic, -1.0, Trainable::QueryKey);
        let tokens = ids(2, 13);
        let inputs: Vec<u32> = tokens
            .chunks(13)
            .flat_map(|row| row[..12].to_vec())
            .collect();
        let targets: Vec<u32> = tokens
            .chunks(13)
            .flat_map(|row| row[1..].to_vec())
            .collect();
        let logits = curved.forward(&inputs, 2, 12, false).expect("forward");
        let loss = next_token_nll(&logits, &targets).expect("loss");
        let grads: GradStore = loss.backward().expect("backward");
        for name in [LOG_EPS, LOG_BETA, "model.layers.1.self_attn.k_proj.weight"] {
            let variable = &curved.variables()[name];
            let grad = grads.get(variable.as_tensor()).expect("gradient present");
            let norm = grad
                .sqr()
                .expect("sq")
                .sum_all()
                .expect("sum")
                .to_scalar::<f32>()
                .expect("s");
            assert!(
                norm.is_finite() && norm > 0.0,
                "{name}: gradient norm {norm}"
            );
        }
    }

    #[test]
    fn curvature_floor_projects_every_head() {
        let curved = model(ScoreKind::KeyNorm, -6.0, Trainable::Scalars);
        curved.floor_log_eps(-2.0).expect("floor");
        let kappa = curved.curvature().expect("kappa");
        let want = (-4.0f32).exp();
        assert!(kappa
            .iter()
            .flatten()
            .all(|k| (k - want).abs() <= 1e-6 * want));
        curved
            .floor_log_eps(-3.0)
            .expect("lower floor keeps higher values");
        assert!(curved
            .curvature()
            .expect("kappa")
            .iter()
            .flatten()
            .all(|k| (k - want).abs() <= 1e-6 * want));
        assert!(model(ScoreKind::Dot, 0.0, Trainable::Scalars)
            .floor_log_eps(0.0)
            .is_err());
    }

    #[test]
    fn distillation_moves_curvature_toward_the_teacher() {
        let teacher = model(ScoreKind::Dot, 0.0, Trainable::Scalars);
        let student = model(ScoreKind::KeyNorm, 0.0, Trainable::Scalars);
        let batch = ids(2, 12);
        let target = teacher.forward(&batch, 2, 12, true).expect("teacher");
        let kl = |m: &KappaLlama| -> f32 {
            let logits = m.forward(&batch, 2, 12, true).expect("student");
            distillation_kl(&logits, &target)
                .expect("kl")
                .to_scalar()
                .expect("scalar")
        };
        let before = kl(&student);
        let variable = &student.variables()[LOG_EPS];
        for _ in 0..40 {
            let logits = student.forward(&batch, 2, 12, false).expect("forward");
            let loss = distillation_kl(&logits, &target).expect("kl");
            let grads = loss.backward().expect("backward");
            let grad = grads.get(variable.as_tensor()).expect("grad");
            // Sign descent: the flat teacher is reached by lowering curvature.
            let step = grad.sign().expect("sign").affine(-0.1, 0.0).expect("step");
            variable
                .set(&variable.as_tensor().add(&step).expect("add"))
                .expect("set");
        }
        let after = kl(&student);
        assert!(
            before > 1e-4,
            "the curved start must differ from the teacher: {before}"
        );
        assert!(after < 0.1 * before, "KL {before} -> {after}");
    }
}
