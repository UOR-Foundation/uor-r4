//! DRAFT / NOT_RUN: isolated B2 LoRA and harmonic transfer operators.
//!
//! No module registration, optimizer loop, fit, flock arm, or acceptance result.
//! The default harmonic path materializes a causal T*T polynomial Gram matrix
//! for differentiable training. Its cost is QUADRATIC, regardless of the
//! mathematically equivalent feature recurrence implemented separately below.
//!
//! Q/K/V LoRA: rank 4, alpha 4; frozen full-width base matrices; random nonzero
//! A and zero B. Harmonic Q/K projections are independent per query head. Native
//! GQA K/V are repeated before these projections, so harmonic state has H
//! instances, not KV instances. Q/K projection parameters begin equal but are
//! distinct trainable variables. Delta=1e-6 and norm epsilon=1e-6 are fixed.

use std::collections::BTreeMap;

use candle_core::{DType, Device, Tensor, Var};
use serde::{Deserialize, Serialize};

use crate::kappa_llama::{rope, LlamaShape};
use crate::stack_tracking::Rng;
use crate::track_b::harmonics::{normalize_rows, HarmonicFeatures};
use crate::track_b::model::{
    dense_attention, AttentionKernel, AttentionPositions, AttentionQkv, FrozenAttentionWeights,
};
use crate::{invalid, Result};

pub const LORA_RANK: usize = 4;
pub const LORA_ALPHA: f64 = 4.0;
pub const HARMONIC_DELTA: f64 = 1e-6;
pub const NORM_EPSILON: f64 = 1e-6;
pub const INITIALIZATION_VERSION: &str = "stack-tracking-splitmix64-uniform-v1";

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TransferArm {
    /// Original softmax attention with the SAME Q/K/V LoRA rank and factors.
    /// It has no unused feature-projection parameters.
    DenseControl,
    Harmonic {
        dimensions: usize,
        degree: usize,
    },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TransferDiagnostics {
    pub calls: u64,
    pub pre_query_fallbacks: u64,
    pub pre_key_fallbacks: u64,
    pub projected_query_fallbacks: u64,
    pub projected_key_fallbacks: u64,
    pub denominator_rows_checked: u64,
    pub nonfinite_denominators: u64,
    pub nonpositive_denominators: u64,
    pub minimum_positive_denominator: Option<f32>,
    /// Only the most recent call is retained; no unbounded diagnostic history.
    /// K norms are repeated to query-head order, matching the independent maps.
    pub last_pre_query_squared_norms: Vec<f32>,
    pub last_pre_key_squared_norms: Vec<f32>,
}

/// Explicit limits for the optional detached recurrence. The bound is persistent
/// M+z state, excluding projected inputs, returned outputs, and temporary tensors.
/// An update can temporarily hold old state, outer product and new state.
#[derive(Clone, Copy, Debug)]
pub struct RecurrentLimits {
    pub max_positions: usize,
    pub max_state_bytes: usize,
}

/// Full-width W*x plus alpha/r * B*A*x. The original weight is never a Var.
pub struct LoraLinear {
    frozen: Tensor,
    a: Var,
    b: Var,
}

impl LoraLinear {
    fn new(frozen: &Tensor, rng: &mut Rng, device: &Device) -> Result<Self> {
        let (output, input) = frozen.dims2()?;
        if output == 0 || input == 0 || frozen.dtype() != DType::F32 {
            return Err(invalid("LoRA requires a nonempty F32 frozen matrix"));
        }
        let a = seeded_uniform_var(
            &[LORA_RANK, input],
            1.0 / (input as f64).sqrt(),
            rng,
            device,
        )?;
        let b = Var::from_tensor(&Tensor::zeros((output, LORA_RANK), DType::F32, device)?)?;
        Ok(Self {
            frozen: frozen.to_device(device)?.detach(),
            a,
            b,
        })
    }

    fn register(&self, prefix: &str, variables: &mut BTreeMap<String, Var>) {
        variables.insert(format!("{prefix}.lora_a"), self.a.clone());
        variables.insert(format!("{prefix}.lora_b"), self.b.clone());
    }

    pub fn frozen_weight(&self) -> &Tensor {
        &self.frozen
    }

    /// F32 [rows,input] -> [rows,output]. Evaluation detaches factors and input.
    pub fn delta(&self, input: &Tensor, detached: bool) -> Result<Tensor> {
        let (_, input_width) = input.dims2()?;
        if input.dtype() != DType::F32 || input_width != self.frozen.dim(1)? {
            return Err(invalid("LoRA input width/dtype mismatch"));
        }
        let input = if detached {
            input.detach()
        } else {
            input.clone()
        };
        let a = variable(&self.a, detached);
        let b = variable(&self.b, detached);
        Ok(input
            .matmul(&a.t()?)?
            .matmul(&b.t()?)?
            .affine(LORA_ALPHA / LORA_RANK as f64, 0.0)?)
    }

    pub fn forward(&self, input: &Tensor, detached: bool) -> Result<Tensor> {
        let x = if detached {
            input.detach()
        } else {
            input.clone()
        };
        Ok(x.matmul(&self.frozen.t()?)?
            .add(&self.delta(&x, detached)?)?)
    }
}

struct HarmonicProjection {
    dimensions: usize,
    degree: usize,
    query: Var,
    key: Var,
    features: HarmonicFeatures,
}

/// One selected layer's replacement. Other layers dispatch unchanged dense
/// attention; composing multiple trained layers requires an explicit router.
pub struct LayerTransfer {
    layer: usize,
    shape: LlamaShape,
    arm: TransferArm,
    seed: u64,
    query: LoraLinear,
    key: LoraLinear,
    value: LoraLinear,
    frozen_output: Tensor,
    harmonic: Option<HarmonicProjection>,
    parameters: BTreeMap<String, Var>,
    diagnostics: TransferDiagnostics,
}

impl LayerTransfer {
    pub fn new(
        layer: usize,
        shape: &LlamaShape,
        weights: FrozenAttentionWeights,
        arm: TransferArm,
        seed: u64,
        device: &Device,
    ) -> Result<Self> {
        // Bound dimensions before the historical shape validator's arithmetic.
        if shape.width == 0
            || shape.width > 1 << 14
            || shape.heads == 0
            || shape.heads > shape.width
            || shape.kv_heads == 0
            || shape.kv_heads > shape.heads
            || shape.head_dim == 0
            || shape.head_dim > shape.width
        {
            return Err(invalid(
                "transfer geometry is zero or exceeds supported bounds",
            ));
        }
        shape.validate()?;
        if layer >= shape.layers || shape.width != shape.heads * shape.head_dim {
            return Err(invalid("transfer layer/shape mismatch"));
        }
        let kv_width = shape.kv_heads * shape.head_dim;
        for (weight, expected) in [
            (&weights.query, [shape.width, shape.width]),
            (&weights.key, [kv_width, shape.width]),
            (&weights.value, [kv_width, shape.width]),
            (&weights.output, [shape.width, shape.width]),
        ] {
            if weight.dims() != expected || weight.dtype() != DType::F32 {
                return Err(invalid(
                    "transfer requires the original full-width F32 Q/K/V/O",
                ));
            }
        }
        // Local Rust RNG avoids global device seeds and makes CPU/Metal initial
        // coefficient arrays identical. This is not cross-backend fit parity.
        let mut rng = Rng::new(seed ^ (layer as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93));
        let query = LoraLinear::new(&weights.query, &mut rng, device)?;
        let key = LoraLinear::new(&weights.key, &mut rng, device)?;
        let value = LoraLinear::new(&weights.value, &mut rng, device)?;
        let mut parameters = BTreeMap::new();
        let prefix = format!("layers.{layer}");
        query.register(&format!("{prefix}.q_proj"), &mut parameters);
        key.register(&format!("{prefix}.k_proj"), &mut parameters);
        value.register(&format!("{prefix}.v_proj"), &mut parameters);
        let harmonic = match arm {
            TransferArm::DenseControl => None,
            TransferArm::Harmonic { dimensions, degree } => {
                let features = HarmonicFeatures::new(dimensions, degree, HARMONIC_DELTA, device)
                    .map_err(|error| invalid(error.to_string()))?;
                let query = seeded_uniform_var(
                    &[shape.heads, dimensions, shape.head_dim],
                    (3.0 / shape.head_dim as f64).sqrt(),
                    &mut rng,
                    device,
                )?;
                // Equal starts give a common random projection per Q/K head;
                // separate Vars permit independently learned maps afterwards.
                let key = Var::from_tensor(&query.as_tensor().detach())?;
                parameters.insert(format!("{prefix}.harmonic.query_projection"), query.clone());
                parameters.insert(format!("{prefix}.harmonic.key_projection"), key.clone());
                Some(HarmonicProjection {
                    dimensions,
                    degree,
                    query,
                    key,
                    features,
                })
            }
        };
        Ok(Self {
            layer,
            shape: shape.clone(),
            arm,
            seed,
            query,
            key,
            value,
            frozen_output: weights.output.to_device(device)?.detach(),
            harmonic,
            parameters,
            diagnostics: TransferDiagnostics::default(),
        })
    }

    pub fn layer(&self) -> usize {
        self.layer
    }
    pub fn arm(&self) -> TransferArm {
        self.arm
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    pub fn parameters(&self) -> &BTreeMap<String, Var> {
        &self.parameters
    }
    pub fn diagnostics(&self) -> &TransferDiagnostics {
        &self.diagnostics
    }
    pub fn reset_diagnostics(&mut self) {
        self.diagnostics = TransferDiagnostics::default();
    }
    pub fn frozen_output(&self) -> &Tensor {
        &self.frozen_output
    }

    /// Suitable for a caller's already-claimed checkpoint writer. No paths are
    /// opened here; save alongside arm, seed, source identity and feature version.
    pub fn parameter_tensors(&self) -> BTreeMap<String, Tensor> {
        self.parameters
            .iter()
            .map(|(name, value)| (name.clone(), value.as_tensor().detach()))
            .collect()
    }

    fn adapted_qkv(
        &self,
        input: &AttentionQkv,
        positions: &AttentionPositions,
        detached: bool,
    ) -> Result<AttentionQkv> {
        let (batch, heads, time, head_dim) = input.query.dims4()?;
        if batch == 0
            || time == 0
            || heads != self.shape.heads
            || head_dim != self.shape.head_dim
            || input.key.dims() != [batch, self.shape.kv_heads, time, head_dim]
            || input.value.dims() != input.key.dims()
            || input.normalized_input.dims() != [batch, time, self.shape.width]
            || input.cosine.dims() != [1, 1, time, head_dim / 2]
            || input.sine.dims() != input.cosine.dims()
            || positions.query != (0..time)
            || positions.key != (0..time)
            || input.excluded.dims() != [1, 1, time, time]
            || input.excluded.dtype() != DType::U8
            || [
                &input.query,
                &input.key,
                &input.value,
                &input.normalized_input,
                &input.cosine,
                &input.sine,
            ]
            .iter()
            .any(|tensor| tensor.dtype() != DType::F32)
        {
            return Err(invalid(
                "transfer needs matching native GQA, normalized input and full-prefix positions",
            ));
        }
        let rows = batch
            .checked_mul(time)
            .ok_or_else(|| invalid("transfer row overflow"))?;
        let normalized = if detached {
            input.normalized_input.detach()
        } else {
            input.normalized_input.clone()
        };
        let flat = normalized.reshape((rows, self.shape.width))?;
        let reshape = |tensor: Tensor, count: usize| -> Result<Tensor> {
            Ok(tensor
                .reshape((batch, time, count, head_dim))?
                .transpose(1, 2)?
                .contiguous()?)
        };
        let q_delta = rope(
            &reshape(self.query.delta(&flat, detached)?, heads)?,
            &input.cosine,
            &input.sine,
        )?;
        let k_delta = rope(
            &reshape(self.key.delta(&flat, detached)?, self.shape.kv_heads)?,
            &input.cosine,
            &input.sine,
        )?;
        let v_delta = reshape(self.value.delta(&flat, detached)?, self.shape.kv_heads)?;
        let base = |tensor: &Tensor| {
            if detached {
                tensor.detach()
            } else {
                tensor.clone()
            }
        };
        Ok(AttentionQkv {
            query: base(&input.query).add(&q_delta)?,
            key: base(&input.key).add(&k_delta)?,
            value: base(&input.value).add(&v_delta)?,
            excluded: input.excluded.clone(),
            normalized_input: normalized,
            cosine: input.cosine.clone(),
            sine: input.sine.clone(),
        })
    }

    /// Normalize full-width post-RoPE vectors, apply per-query-head learned maps,
    /// then normalize again onto the projected unit sphere. Native KV sharing
    /// ends here: different query heads have independently learned key maps.
    fn projected(
        &mut self,
        input: &AttentionQkv,
        detached: bool,
    ) -> Result<(Tensor, Tensor, Tensor)> {
        let harmonic = self
            .harmonic
            .as_ref()
            .ok_or_else(|| invalid("dense arm has no harmonic projection"))?;
        let (batch, heads, time, width) = input.query.dims4()?;
        let repeated_key = repeat_kv(&input.key, heads)?;
        let repeated_value = repeat_kv(&input.value, heads)?;
        let q_rows = input
            .query
            .contiguous()?
            .reshape((batch * heads * time, width))?;
        let k_rows = repeated_key
            .contiguous()?
            .reshape((batch * heads * time, width))?;
        let q_unit = normalize_rows(&q_rows, NORM_EPSILON).map_err(|e| invalid(e.to_string()))?;
        let k_unit = normalize_rows(&k_rows, NORM_EPSILON).map_err(|e| invalid(e.to_string()))?;
        self.diagnostics.pre_query_fallbacks = self
            .diagnostics
            .pre_query_fallbacks
            .saturating_add(q_unit.fallback_rows as u64);
        self.diagnostics.pre_key_fallbacks = self
            .diagnostics
            .pre_key_fallbacks
            .saturating_add(k_unit.fallback_rows as u64);
        self.diagnostics.last_pre_query_squared_norms = q_unit.squared_norms;
        self.diagnostics.last_pre_key_squared_norms = k_unit.squared_norms;
        let project = |unit: Tensor, matrix: &Var| -> Result<_> {
            let projected = unit
                .reshape((batch, heads, time, width))?
                .broadcast_matmul(&variable(matrix, detached).transpose(1, 2)?.unsqueeze(0)?)?
                .contiguous()?
                .reshape((batch * heads * time, harmonic.dimensions))?;
            normalize_rows(&projected, NORM_EPSILON).map_err(|e| invalid(e.to_string()))
        };
        let q_projected = project(q_unit.unit, &harmonic.query)?;
        let k_projected = project(k_unit.unit, &harmonic.key)?;
        self.diagnostics.projected_query_fallbacks = self
            .diagnostics
            .projected_query_fallbacks
            .saturating_add(q_projected.fallback_rows as u64);
        self.diagnostics.projected_key_fallbacks = self
            .diagnostics
            .projected_key_fallbacks
            .saturating_add(k_projected.fallback_rows as u64);
        Ok((
            q_projected
                .unit
                .reshape((batch, heads, time, harmonic.dimensions))?,
            k_projected
                .unit
                .reshape((batch, heads, time, harmonic.dimensions))?,
            repeated_value,
        ))
    }

    /// Explicit QUADRATIC positive-polynomial Gram path for training. No feature
    /// recurrence or linear-time claim applies to this implementation.
    fn quadratic_harmonic(&mut self, input: &AttentionQkv) -> Result<Tensor> {
        let (query, key, value) = self.projected(input, false)?;
        let degree = self
            .harmonic
            .as_ref()
            .ok_or_else(|| invalid("missing harmonic config"))?
            .degree;
        let dot = query.matmul(&key.transpose(2, 3)?.contiguous()?)?;
        let base = dot.affine(0.5, 0.5)?;
        let power = match degree {
            1 => base,
            2 => base.sqr()?,
            3 => base.sqr()?.mul(&base)?,
            _ => return Err(invalid("unsupported harmonic degree")),
        };
        let scores = power.affine(1.0, HARMONIC_DELTA)?;
        let mask = input.excluded.broadcast_as(scores.shape())?;
        let causal = mask.where_cond(&scores.zeros_like()?, &scores)?;
        let denominator = causal.sum_keepdim(3)?;
        check_denominators(&denominator, &mut self.diagnostics)?;
        Ok(causal
            .broadcast_div(&denominator)?
            .matmul(&value.contiguous()?)?)
    }

    /// Detached pure-harmonic full-prefix evaluation with a recurrent M,z scan.
    /// Input/output sequences still occupy O(T) memory; persistent state is
    /// B*H*features*(head_dim+1)*4 bytes. No history cache or future-token access.
    /// Current key/value is inserted BEFORE querying (inclusive causality).
    pub fn attend_recurrent_eval(
        &mut self,
        input: &AttentionQkv,
        positions: &AttentionPositions,
        limits: RecurrentLimits,
    ) -> Result<Tensor> {
        let (batch, heads, time, width) = input.query.dims4()?;
        if time == 0 || time > limits.max_positions {
            return Err(invalid("recurrent full-prefix position limit exceeded"));
        }
        let features = self
            .harmonic
            .as_ref()
            .ok_or_else(|| invalid("dense arm has no recurrence"))?
            .features
            .feature_dimensions();
        let width_with_mass = width
            .checked_add(1)
            .ok_or_else(|| invalid("recurrent state width overflow"))?;
        let state_bytes = [batch, heads, features, width_with_mass, 4]
            .into_iter()
            .try_fold(1usize, |size, factor| size.checked_mul(factor))
            .ok_or_else(|| invalid("recurrent state size overflow"))?;
        if state_bytes > limits.max_state_bytes {
            return Err(invalid(format!(
                "recurrent state {state_bytes} exceeds allowance {}",
                limits.max_state_bytes
            )));
        }
        self.diagnostics.calls = self.diagnostics.calls.saturating_add(1);
        let adapted = self.adapted_qkv(input, positions, true)?;
        let (query, key, value) = self.projected(&adapted, true)?;
        let harmonic = self
            .harmonic
            .as_ref()
            .ok_or_else(|| invalid("missing harmonic config"))?;
        let mut state = Tensor::zeros((batch, heads, features, width), DType::F32, query.device())?;
        let mut mass = Tensor::zeros((batch, heads, features), DType::F32, query.device())?;
        let mut outputs = Vec::with_capacity(time);
        for position in 0..time {
            let feature_at = |tensor: &Tensor| -> Result<Tensor> {
                let row = tensor
                    .narrow(2, position, 1)?
                    .contiguous()?
                    .reshape((batch * heads, harmonic.dimensions))?;
                Ok(harmonic
                    .features
                    .forward(&row)
                    .map_err(|e| invalid(e.to_string()))?
                    .reshape((batch, heads, features))?)
            };
            let q = feature_at(&query)?;
            let k = feature_at(&key)?;
            let v = value.narrow(2, position, 1)?.squeeze(2)?;
            state = state.add(&k.unsqueeze(3)?.broadcast_mul(&v.unsqueeze(2)?)?)?;
            mass = mass.add(&k)?;
            let denominator = q.mul(&mass)?.sum_keepdim(2)?;
            check_denominators(&denominator, &mut self.diagnostics)?;
            let numerator = q.unsqueeze(2)?.matmul(&state)?.squeeze(2)?;
            outputs.push(numerator.broadcast_div(&denominator)?.unsqueeze(2)?);
        }
        Ok(Tensor::cat(&outputs, 2)?)
    }

    /// Freeze WO; compare the actual post-WO, pre-residual operator output.
    pub fn post_wo_mse(&self, attended: &Tensor, target: &Tensor) -> Result<PostWoMse> {
        post_wo_mse(attended, &self.frozen_output, target)
    }
}

impl AttentionKernel for LayerTransfer {
    fn attend(
        &mut self,
        layer: usize,
        input: &AttentionQkv,
        positions: &AttentionPositions,
    ) -> Result<Tensor> {
        if layer != self.layer {
            return dense_attention(input, positions);
        }
        self.diagnostics.calls = self.diagnostics.calls.saturating_add(1);
        let adapted = self.adapted_qkv(input, positions, false)?;
        match self.arm {
            TransferArm::DenseControl => dense_attention(&adapted, positions),
            TransferArm::Harmonic { .. } => self.quadratic_harmonic(&adapted),
        }
    }
}

pub struct PostWoMse {
    /// Differentiable objective. The target is detached and WO is frozen.
    pub objective: Tensor,
    pub raw_mse: f64,
    pub target_energy: f64,
    pub normalized_squared_error: Option<f64>,
}

pub fn post_wo_mse(
    attended: &Tensor,
    output_weight: &Tensor,
    target: &Tensor,
) -> Result<PostWoMse> {
    let (batch, heads, time, head_dim) = attended.dims4()?;
    let width = heads
        .checked_mul(head_dim)
        .ok_or_else(|| invalid("post-WO width overflow"))?;
    if batch == 0
        || time == 0
        || width == 0
        || output_weight.dims() != [width, width]
        || target.dims() != [batch, time, width]
        || attended.dtype() != DType::F32
        || output_weight.dtype() != DType::F32
        || target.dtype() != DType::F32
    {
        return Err(invalid("post-WO loss shape/dtype mismatch"));
    }
    let prediction = attended
        .transpose(1, 2)?
        .contiguous()?
        .reshape((batch * time, width))?
        .matmul(&output_weight.detach().t()?)?
        .reshape((batch, time, width))?;
    let target = target.detach();
    let squared = prediction.sub(&target)?.sqr()?;
    let objective = squared.mean_all()?;
    let raw_mse = f64::from(objective.to_scalar::<f32>()?);
    let squared_error = f64::from(squared.sum_all()?.to_scalar::<f32>()?);
    let target_energy = f64::from(target.sqr()?.sum_all()?.to_scalar::<f32>()?);
    if !raw_mse.is_finite() || !squared_error.is_finite() || !target_energy.is_finite() {
        return Err(invalid("nonfinite post-WO loss or target energy"));
    }
    Ok(PostWoMse {
        objective,
        raw_mse,
        target_energy,
        normalized_squared_error: if target_energy > 0.0 {
            Some(squared_error / target_energy)
        } else {
            None
        },
    })
}

fn variable(value: &Var, detached: bool) -> Tensor {
    if detached {
        value.as_tensor().detach()
    } else {
        value.as_tensor().clone()
    }
}

fn seeded_uniform_var(shape: &[usize], bound: f64, rng: &mut Rng, device: &Device) -> Result<Var> {
    let count = shape
        .iter()
        .try_fold(1usize, |size, dimension| size.checked_mul(*dimension))
        .ok_or_else(|| invalid("parameter element count overflow"))?;
    if count == 0 || !bound.is_finite() || bound <= 0.0 {
        return Err(invalid("invalid seeded parameter shape/bound"));
    }
    let values = (0..count)
        .map(|_| {
            let uniform = ((rng.next_u64() >> 11) as f64 + 0.5) / (1u64 << 53) as f64;
            ((2.0 * uniform - 1.0) * bound) as f32
        })
        .collect::<Vec<_>>();
    Ok(Var::from_tensor(&Tensor::from_vec(values, shape, device)?)?)
}

fn repeat_kv(tensor: &Tensor, query_heads: usize) -> Result<Tensor> {
    let (batch, kv_heads, time, width) = tensor.dims4()?;
    if kv_heads == 0 || query_heads == 0 || query_heads % kv_heads != 0 {
        return Err(invalid("invalid GQA repetition"));
    }
    let group = query_heads / kv_heads;
    if group == 1 {
        return Ok(tensor.clone());
    }
    Ok(tensor
        .unsqueeze(2)?
        .broadcast_as((batch, kv_heads, group, time, width))?
        .contiguous()?
        .reshape((batch, query_heads, time, width))?)
}

fn check_denominators(denominator: &Tensor, diagnostic: &mut TransferDiagnostics) -> Result<()> {
    let rows = denominator.detach().flatten_all()?.to_vec1::<f32>()?;
    diagnostic.denominator_rows_checked = diagnostic
        .denominator_rows_checked
        .saturating_add(rows.len() as u64);
    let nonfinite = rows.iter().filter(|v| !v.is_finite()).count() as u64;
    let nonpositive = rows.iter().filter(|v| v.is_finite() && **v <= 0.0).count() as u64;
    diagnostic.nonfinite_denominators = diagnostic.nonfinite_denominators.saturating_add(nonfinite);
    diagnostic.nonpositive_denominators = diagnostic
        .nonpositive_denominators
        .saturating_add(nonpositive);
    for value in rows.into_iter().filter(|v| v.is_finite() && *v > 0.0) {
        diagnostic.minimum_positive_denominator = Some(
            diagnostic
                .minimum_positive_denominator
                .map_or(value, |old| old.min(value)),
        );
    }
    if nonfinite > 0 || nonpositive > 0 {
        return Err(invalid(format!("harmonic denominator failure: {nonfinite} nonfinite, {nonpositive} nonpositive; no clamp applied")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Result<(
        LlamaShape,
        FrozenAttentionWeights,
        AttentionQkv,
        AttentionPositions,
    )> {
        let shape = LlamaShape {
            vocab: 16,
            width: 8,
            layers: 1,
            heads: 2,
            kv_heads: 1,
            head_dim: 4,
            ffn: 12,
            rope_theta: 100000.0,
            rms_eps: 1e-5,
            tied_embeddings: true,
        };
        let matrix = |rows, cols| -> Result<Tensor> {
            let values = (0..rows * cols)
                .map(|i| ((i * 7 + 3) % 23) as f32 / 23.0 - 0.5)
                .collect::<Vec<_>>();
            Ok(Tensor::from_vec(values, (rows, cols), &Device::Cpu)?)
        };
        let weights = FrozenAttentionWeights {
            query: matrix(8, 8)?,
            key: matrix(4, 8)?,
            value: matrix(4, 8)?,
            output: matrix(8, 8)?,
            input_norm: Tensor::ones(8, DType::F32, &Device::Cpu)?,
        };
        let normalized = matrix(4, 8)?.reshape((1, 4, 8))?;
        let project = |weight: &Tensor, heads| -> Result<Tensor> {
            Ok(normalized
                .reshape((4, 8))?
                .matmul(&weight.t()?)?
                .reshape((1, 4, heads, 4))?
                .transpose(1, 2)?
                .contiguous()?)
        };
        let mask = (0..4)
            .flat_map(|q| (0..4).map(move |k| u8::from(k > q)))
            .collect::<Vec<_>>();
        let input = AttentionQkv {
            query: project(&weights.query, 2)?,
            key: project(&weights.key, 1)?,
            value: project(&weights.value, 1)?,
            excluded: Tensor::from_vec(mask, (1, 1, 4, 4), &Device::Cpu)?,
            normalized_input: normalized,
            cosine: Tensor::ones((1, 1, 4, 2), DType::F32, &Device::Cpu)?,
            sine: Tensor::zeros((1, 1, 4, 2), DType::F32, &Device::Cpu)?,
        };
        Ok((
            shape,
            weights,
            input,
            AttentionPositions {
                query: 0..4,
                key: 0..4,
            },
        ))
    }

    #[test]
    fn zero_b_preserves_dense_and_seeded_lora_is_matched_across_arms() -> Result<()> {
        let (shape, weights, input, positions) = fixture()?;
        let mut dense = LayerTransfer::new(
            0,
            &shape,
            weights.clone(),
            TransferArm::DenseControl,
            17,
            &Device::Cpu,
        )?;
        let harmonic = LayerTransfer::new(
            0,
            &shape,
            weights,
            TransferArm::Harmonic {
                dimensions: 16,
                degree: 2,
            },
            17,
            &Device::Cpu,
        )?;
        let reference = dense_attention(&input, &positions)?;
        let adapted = dense.attend(0, &input, &positions)?;
        assert_eq!(
            reference.flatten_all()?.to_vec1::<f32>()?,
            adapted.flatten_all()?.to_vec1::<f32>()?
        );
        assert_eq!(dense.parameters().len(), 6);
        assert_eq!(harmonic.parameters().len(), 8);
        for (name, parameter) in dense.parameters() {
            let counterpart = harmonic
                .parameters()
                .get(name)
                .ok_or_else(|| invalid("missing paired LoRA parameter"))?;
            let values = parameter.flatten_all()?.to_vec1::<f32>()?;
            assert_eq!(values, counterpart.flatten_all()?.to_vec1::<f32>()?);
            if name.ends_with("lora_b") {
                assert!(values.iter().all(|v| *v == 0.0));
            }
            if name.ends_with("lora_a") {
                assert!(values.iter().any(|v| *v != 0.0));
            }
        }
        Ok(())
    }

    #[test]
    fn quadratic_and_detached_recurrent_harmonics_agree() -> Result<()> {
        let (shape, weights, input, positions) = fixture()?;
        for dimensions in [16, 32] {
            for degree in [1, 2, 3] {
                let mut kernel = LayerTransfer::new(
                    0,
                    &shape,
                    weights.clone(),
                    TransferArm::Harmonic { dimensions, degree },
                    0,
                    &Device::Cpu,
                )?;
                let quadratic = kernel.attend(0, &input, &positions)?;
                let recurrent = kernel.attend_recurrent_eval(
                    &input,
                    &positions,
                    RecurrentLimits {
                        max_positions: 4,
                        max_state_bytes: 4 * 1024 * 1024,
                    },
                )?;
                let max_error = quadratic
                    .sub(&recurrent)?
                    .abs()?
                    .flatten_all()?
                    .max(0)?
                    .to_scalar::<f32>()?;
                assert!(
                    max_error < 2e-4,
                    "d={dimensions}, L={degree}, max={max_error}"
                );
                assert_eq!(kernel.diagnostics().nonpositive_denominators, 0);
                assert!(kernel
                    .attend_recurrent_eval(
                        &input,
                        &positions,
                        RecurrentLimits {
                            max_positions: 4,
                            max_state_bytes: 1
                        }
                    )
                    .is_err());
            }
        }
        Ok(())
    }

    #[test]
    fn harmonic_post_wo_mse_reaches_lora_b_and_feature_projections() -> Result<()> {
        let (shape, weights, input, positions) = fixture()?;
        let mut kernel = LayerTransfer::new(
            0,
            &shape,
            weights,
            TransferArm::Harmonic {
                dimensions: 16,
                degree: 2,
            },
            1,
            &Device::Cpu,
        )?;
        let output = kernel.attend(0, &input, &positions)?;
        let target = Tensor::zeros((1, 4, 8), DType::F32, &Device::Cpu)?;
        let loss = kernel.post_wo_mse(&output, &target)?;
        assert!(loss.raw_mse > 0.0 && loss.normalized_squared_error.is_none());
        let gradients = loss.objective.backward()?;
        for (name, parameter) in kernel.parameters() {
            let gradient = gradients
                .get(parameter)
                .ok_or_else(|| invalid(format!("missing gradient for {name}")))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(gradient.iter().all(|v| v.is_finite()), "{name}");
            if name.ends_with("lora_a") {
                assert!(
                    gradient.iter().all(|v| *v == 0.0),
                    "zero B must give zero initial A gradient"
                );
            } else {
                assert!(gradient.iter().any(|v| v.abs() > 1e-10), "{name}");
            }
        }
        Ok(())
    }

    #[test]
    fn invalid_denominators_are_counted_and_never_clamped() -> Result<()> {
        let mut diagnostics = TransferDiagnostics::default();
        let values = Tensor::from_slice(&[1.0f32, 0.0, -1.0, f32::NAN], 4, &Device::Cpu)?;
        assert!(check_denominators(&values, &mut diagnostics).is_err());
        assert_eq!(diagnostics.denominator_rows_checked, 4);
        assert_eq!(diagnostics.nonfinite_denominators, 1);
        assert_eq!(diagnostics.nonpositive_denominators, 2);
        Ok(())
    }
}
