//! Differentiable, full-prefix Track B model and shared attention seam.
//!
//! This is an offline floating-point teacher/student construction, not a native
//! runtime or KV-cache implementation. It reuses the existing safe checkpoint
//! loader and full tensor dimensions, but has not yet passed the upstream
//! Candle or exact-reference bridge. No teacher parity is asserted by this file.
//!
//! The Llama computation is adapted from Hugging Face Candle 0.9.2, specifically
//! [`candle-transformers/src/models/llama.rs`](https://github.com/huggingface/candle/blob/3b39794c14b1de8ba4c2d10d49354d557a415547/candle-transformers/src/models/llama.rs).
//! Upstream is dual licensed under [MIT](https://github.com/huggingface/candle/blob/3b39794c14b1de8ba4c2d10d49354d557a415547/LICENSE-MIT)
//! or [Apache-2.0](https://github.com/huggingface/candle/blob/3b39794c14b1de8ba4c2d10d49354d557a415547/LICENSE-APACHE).
//! This adaptation uses the MIT option; its notice is retained below. Changes
//! add differentiable primitives, all-position logits, native GQA inputs,
//! pluggable attention and early-exit teacher captures. Existing UOR-R4 Kappa
//! checkpoint and split-half rotation code is reused rather than duplicated.

// Upstream MIT permission notice (Candle source revision named above):
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::collections::BTreeMap;
use std::fs;
use std::ops::Range;
use std::path::Path;

use candle_core::{DType, Device, Tensor};

use crate::kappa_llama::{load_checkpoint, rope, Checkpoint, LlamaShape};
use crate::{invalid, Result};

use super::conversion::parse_config;

/// Detached ordinary tensors, never optimizer variables. Shapes are Q/O [W,W],
/// K/V [KV*D,W], norm [W]. Clones share underlying frozen checkpoint storage.
#[derive(Clone)]
pub struct FrozenAttentionWeights {
    pub query: Tensor,
    pub key: Tensor,
    pub value: Tensor,
    pub output: Tensor,
    pub input_norm: Tensor,
}

struct FrozenLayer {
    attention: FrozenAttentionWeights,
    post_attention_norm: Tensor,
    gate: Tensor,
    up: Tensor,
    down: Tensor,
}

/// Native grouped-query tensors after RoPE. Query is [B,H,T,D]; key and value
/// are [B,KV,T,D]. GQA repetition has not happened at this interface.
///
/// `excluded` is a shared causal U8 mask [1,1,T,T], with one above the diagonal.
/// Hooks return [B,H,T,D] attended values before the unchanged output map.
pub struct AttentionQkv {
    pub query: Tensor,
    pub key: Tensor,
    pub value: Tensor,
    pub excluded: Tensor,
    /// [B,T,W] after RMS gain; retained so a future hook can form LoRA deltas
    /// from the actual composed student input rather than a teacher trace.
    pub normalized_input: Tensor,
    /// Shared [1,1,T,D/2] tables allow the same RoPE on future projection deltas.
    pub cosine: Tensor,
    pub sine: Tensor,
}

/// One teacher-forced layer example. Both arrays are detached [B,T,W].
pub struct LayerCapture {
    pub layer: usize,
    /// RMS normalization INCLUDING the frozen input gain, ready for Q/K/V maps.
    pub normalized_input: Tensor,
    /// Frozen WO applied to attended values; residual and MLP are not included.
    pub attention_output: Tensor,
}

/// Absolute positions shared by every batch row. Full-prefix evaluation starts
/// both ranges at zero and gives queries and keys the same inclusive causal
/// horizon. Explicit ranges prevent plug-ins from treating a row index as an
/// unspecified relative position. Incremental/cached prefixes are not supported.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttentionPositions {
    pub query: Range<usize>,
    pub key: Range<usize>,
}

/// Shared replacement seam for flock, harmonic and dense controls.
///
/// Inputs contain post-RoPE Q [B,H,T,D] and native K/V [B,KV,T,D], normalized
/// input [B,T,W], RoPE tables and the causal mask. Implementations return F32
/// [B,H,T,D] values BEFORE WO, without residual or MLP work. They must preserve
/// absolute-position causality and document any additional state or key storage.
/// Returning a differentiable tensor preserves gradients through later frozen
/// layers. The unchanged source Q/K/V/O remain available through the model.
pub trait AttentionKernel {
    fn attend(
        &mut self,
        layer: usize,
        inputs: &AttentionQkv,
        positions: &AttentionPositions,
    ) -> Result<Tensor>;
}

/// Ordinary causal softmax, with no approximation or sparse selection. Numerical
/// agreement with the exact model-source executor remains an empirical gate.
#[derive(Default)]
pub struct DenseAttention;

impl AttentionKernel for DenseAttention {
    fn attend(
        &mut self,
        _layer: usize,
        inputs: &AttentionQkv,
        positions: &AttentionPositions,
    ) -> Result<Tensor> {
        dense_attention(inputs, positions)
    }
}

/// Adapter for experiments whose operator is naturally written as a closure.
pub struct ClosureAttention<F>(pub F);

impl<F> AttentionKernel for ClosureAttention<F>
where
    F: FnMut(usize, &AttentionQkv, &AttentionPositions) -> Result<Tensor>,
{
    fn attend(
        &mut self,
        layer: usize,
        inputs: &AttentionQkv,
        positions: &AttentionPositions,
    ) -> Result<Tensor> {
        (self.0)(layer, inputs, positions)
    }
}

enum ForwardResult {
    Logits(Tensor),
    Capture(LayerCapture),
}

pub struct TrackBModel {
    shape: LlamaShape,
    max_position_embeddings: usize,
    weights_sha256: String,
    embedding: Tensor,
    layers: Vec<FrozenLayer>,
    final_norm: Tensor,
    output_head: Tensor,
    device: Device,
}

impl TrackBModel {
    pub fn load(source: &Path, device: &Device) -> Result<Self> {
        // Apply exactly the conversion loader's supported configuration before
        // loading any weights; Kappa's historical parser alone accepts fields
        // (e.g. sliding_window) that this forward does not implement.
        let config = parse_config(&fs::read(source.join("config.json"))?)
            .map_err(|error| invalid(error.to_string()))?;
        Self::from_checkpoint(
            load_checkpoint(source, device)?,
            config.max_position_embeddings,
            device,
        )
    }

    pub fn from_checkpoint(
        checkpoint: Checkpoint,
        max_position_embeddings: usize,
        device: &Device,
    ) -> Result<Self> {
        let Checkpoint {
            shape,
            mut tensors,
            weights_sha256,
        } = checkpoint;
        if shape.width == 0
            || shape.width > 1 << 14
            || shape.heads == 0
            || shape.heads > shape.width
            || shape.kv_heads == 0
            || shape.kv_heads > shape.heads
            || shape.head_dim == 0
            || shape.head_dim > shape.width
            || shape.ffn == 0
            || shape.ffn > 1 << 20
            || max_position_embeddings == 0
            || max_position_embeddings > u32::MAX as usize
        {
            return Err(invalid("invalid Track B shape/context dimensions"));
        }
        shape.validate()?;
        if !shape.rms_eps.is_finite()
            || shape.rms_eps <= 0.0
            || !shape.rope_theta.is_finite()
            || shape.rope_theta <= 0.0
            || !(shape.rope_theta as f32).is_finite()
            || (shape.rope_theta as f32) <= 0.0
        {
            return Err(invalid(
                "Track B RMS epsilon / F32 RoPE theta must be positive",
            ));
        }
        let embedding = take_weight(
            &mut tensors,
            "model.embed_tokens.weight",
            &[shape.vocab, shape.width],
            device,
        )?;
        let output_head = if shape.tied_embeddings {
            embedding.clone()
        } else {
            take_weight(
                &mut tensors,
                "lm_head.weight",
                &[shape.vocab, shape.width],
                device,
            )?
        };
        let final_norm = take_weight(&mut tensors, "model.norm.weight", &[shape.width], device)?;
        let mut layers = Vec::with_capacity(shape.layers);
        let kv_width = shape.kv_heads * shape.head_dim;
        for layer in 0..shape.layers {
            let prefix = format!("model.layers.{layer}");
            let mut weight = |suffix: &str, dimensions: &[usize]| {
                take_weight(
                    &mut tensors,
                    &format!("{prefix}.{suffix}.weight"),
                    dimensions,
                    device,
                )
            };
            layers.push(FrozenLayer {
                attention: FrozenAttentionWeights {
                    query: weight("self_attn.q_proj", &[shape.width, shape.width])?,
                    key: weight("self_attn.k_proj", &[kv_width, shape.width])?,
                    value: weight("self_attn.v_proj", &[kv_width, shape.width])?,
                    output: weight("self_attn.o_proj", &[shape.width, shape.width])?,
                    input_norm: weight("input_layernorm", &[shape.width])?,
                },
                post_attention_norm: weight("post_attention_layernorm", &[shape.width])?,
                gate: weight("mlp.gate_proj", &[shape.ffn, shape.width])?,
                up: weight("mlp.up_proj", &[shape.ffn, shape.width])?,
                down: weight("mlp.down_proj", &[shape.width, shape.ffn])?,
            });
        }
        if !tensors.is_empty() {
            return Err(invalid(format!(
                "unexpected Track B checkpoint tensors: {:?}",
                tensors.keys().take(4).collect::<Vec<_>>()
            )));
        }
        Ok(Self {
            shape,
            max_position_embeddings,
            weights_sha256,
            embedding,
            layers,
            final_norm,
            output_head,
            device: device.clone(),
        })
    }

    pub fn shape(&self) -> &LlamaShape {
        &self.shape
    }
    pub fn max_position_embeddings(&self) -> usize {
        self.max_position_embeddings
    }
    pub fn weights_sha256(&self) -> &str {
        &self.weights_sha256
    }
    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn attention_weights(&self, layer: usize) -> Result<FrozenAttentionWeights> {
        self.layers
            .get(layer)
            .map(|layer| layer.attention.clone())
            .ok_or_else(|| invalid(format!("Track B layer {layer} is out of range")))
    }

    /// Dense teacher full-prefix logits [B,T,V]. Weights are ordinary detached
    /// tensors; no graph is retained unless an upstream caller introduced Vars.
    pub fn forward(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        match self.run(ids, batch, time, None, None)? {
            ForwardResult::Logits(logits) => Ok(logits),
            ForwardResult::Capture(_) => Err(invalid("unexpected layer capture from full forward")),
        }
    }

    /// All layers invoke the kernel; callers may use DenseAttention for
    /// unchanged layers. No hook output is detached, so gradients propagate
    /// through later frozen layers into the replacement's own variables.
    pub fn forward_with_attention(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        kernel: &mut dyn AttentionKernel,
    ) -> Result<Tensor> {
        match self.run(ids, batch, time, None, Some(kernel))? {
            ForwardResult::Logits(logits) => Ok(logits),
            ForwardResult::Capture(_) => {
                Err(invalid("unexpected layer capture from hooked forward"))
            }
        }
    }

    /// Convenience adapter; the shared plug-in contract remains AttentionKernel.
    pub fn forward_with_attention_fn<F>(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        function: F,
    ) -> Result<Tensor>
    where
        F: FnMut(usize, &AttentionQkv, &AttentionPositions) -> Result<Tensor>,
    {
        self.forward_with_attention(ids, batch, time, &mut ClosureAttention(function))
    }

    /// Dense teacher capture exits immediately after the selected layer's WO.
    /// Its MLP, every later layer, final normalization, and vocabulary head are
    /// not executed. Previous layers remain the unmodified dense teacher.
    pub fn capture_layer(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        layer: usize,
    ) -> Result<LayerCapture> {
        if layer >= self.layers.len() {
            return Err(invalid(format!("capture layer {layer} is out of range")));
        }
        match self.run(ids, batch, time, Some(layer), None)? {
            ForwardResult::Capture(capture) => Ok(capture),
            ForwardResult::Logits(_) => Err(invalid("requested capture was not produced")),
        }
    }

    fn run(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        stop_at: Option<usize>,
        mut kernel: Option<&mut dyn AttentionKernel>,
    ) -> Result<ForwardResult> {
        let rows = batch
            .checked_mul(time)
            .ok_or_else(|| invalid("batch*time overflow"))?;
        if batch == 0
            || time == 0
            || time > self.max_position_embeddings
            || ids.len() != rows
            || ids.iter().any(|id| *id as usize >= self.shape.vocab)
        {
            return Err(invalid("Track B token, batch or context bounds"));
        }
        let shape = &self.shape;
        let (cosine, sine) = candle_f32_rope_tables(shape, time, &self.device)?;
        let positions = AttentionPositions {
            query: 0..time,
            key: 0..time,
        };
        let excluded = causal_mask(&positions, &self.device)?;
        let ids = Tensor::new(ids, &self.device)?;
        let mut state = self.embedding.index_select(&ids, 0)?;
        let mut dense = DenseAttention;
        for (layer_index, layer) in self.layers.iter().enumerate() {
            let normalized = rms_norm(&state, &layer.attention.input_norm, shape.rms_eps)?;
            let project = |weight: &Tensor, heads: usize| -> Result<Tensor> {
                Ok(linear(&normalized, weight)?
                    .reshape((batch, time, heads, shape.head_dim))?
                    .transpose(1, 2)?
                    .contiguous()?)
            };
            let qkv = AttentionQkv {
                query: rope(
                    &project(&layer.attention.query, shape.heads)?,
                    &cosine,
                    &sine,
                )?,
                key: rope(
                    &project(&layer.attention.key, shape.kv_heads)?,
                    &cosine,
                    &sine,
                )?,
                value: project(&layer.attention.value, shape.kv_heads)?,
                excluded: excluded.clone(),
                normalized_input: normalized.reshape((batch, time, shape.width))?,
                cosine: cosine.clone(),
                sine: sine.clone(),
            };
            let attended = match kernel.as_deref_mut() {
                Some(replace) => replace.attend(layer_index, &qkv, &positions)?,
                None => dense.attend(layer_index, &qkv, &positions)?,
            };
            if attended.dims() != [batch, shape.heads, time, shape.head_dim]
                || attended.dtype() != DType::F32
            {
                return Err(invalid(format!(
                    "attention hook at layer {layer_index} must return F32 [B,H,T,D]"
                )));
            }
            let attended = attended
                .transpose(1, 2)?
                .contiguous()?
                .reshape((rows, shape.width))?;
            let attention_output = linear(&attended, &layer.attention.output)?;
            if stop_at == Some(layer_index) {
                return Ok(ForwardResult::Capture(LayerCapture {
                    layer: layer_index,
                    normalized_input: normalized.reshape((batch, time, shape.width))?.detach(),
                    attention_output: attention_output
                        .reshape((batch, time, shape.width))?
                        .detach(),
                }));
            }
            state = state.add(&attention_output)?;
            let normalized = rms_norm(&state, &layer.post_attention_norm, shape.rms_eps)?;
            let gate = linear(&normalized, &layer.gate)?.silu()?;
            let gated = gate.mul(&linear(&normalized, &layer.up)?)?;
            state = state.add(&linear(&gated, &layer.down)?)?;
        }
        let normalized = rms_norm(&state, &self.final_norm, shape.rms_eps)?;
        Ok(ForwardResult::Logits(
            linear(&normalized, &self.output_head)?.reshape((batch, time, shape.vocab))?,
        ))
    }
}

fn take_weight(
    tensors: &mut BTreeMap<String, Tensor>,
    name: &str,
    dimensions: &[usize],
    device: &Device,
) -> Result<Tensor> {
    let tensor = tensors
        .remove(name)
        .ok_or_else(|| invalid(format!("missing tensor {name}")))?;
    if tensor.dims() != dimensions || tensor.dtype() != DType::F32 {
        return Err(invalid(format!(
            "{name} must be F32 with shape {dimensions:?}"
        )));
    }
    Ok(tensor.to_device(device)?.detach())
}

fn linear(input: &Tensor, weight: &Tensor) -> Result<Tensor> {
    Ok(input.matmul(&weight.t()?)?)
}

fn rms_norm(input: &Tensor, gain: &Tensor, epsilon: f64) -> Result<Tensor> {
    // Unlike fused rms_norm, this primitive composition carries backward.
    let denominator = input.sqr()?.mean_keepdim(1)?.affine(1.0, epsilon)?.sqrt()?;
    Ok(input.broadcast_div(&denominator)?.broadcast_mul(gain)?)
}

/// Match upstream Candle Cache's F32 inverse-frequency construction and device
/// phase matmul, rather than Kappa's F64 phase generation. Rotation itself uses
/// the existing differentiable split-half composition.
fn candle_f32_rope_tables(
    shape: &LlamaShape,
    time: usize,
    device: &Device,
) -> Result<(Tensor, Tensor)> {
    let frequencies = (0..shape.head_dim)
        .step_by(2)
        .map(|index| 1.0f32 / (shape.rope_theta as f32).powf(index as f32 / shape.head_dim as f32))
        .collect::<Vec<_>>();
    let frequencies = Tensor::from_vec(frequencies, (1, shape.head_dim / 2), device)?;
    let positions = Tensor::arange(0u32, time as u32, device)?
        .to_dtype(DType::F32)?
        .reshape((time, 1))?;
    let phase = positions.matmul(&frequencies)?;
    Ok((
        phase.cos()?.reshape((1, 1, time, shape.head_dim / 2))?,
        phase.sin()?.reshape((1, 1, time, shape.head_dim / 2))?,
    ))
}

/// Dense causal softmax control with native GQA input and differentiable ops.
/// This is a quadratic training/evaluation implementation, not recurrent cost.
pub fn dense_attention(input: &AttentionQkv, positions: &AttentionPositions) -> Result<Tensor> {
    let (batch, heads, time, width) = input.query.dims4()?;
    let (key_batch, kv_heads, key_time, key_width) = input.key.dims4()?;
    if batch == 0
        || heads == 0
        || kv_heads == 0
        || time == 0
        || width == 0
        || heads % kv_heads != 0
        || key_batch != batch
        || key_time != time
        || key_width != width
        || input.value.dims() != input.key.dims()
        || input.excluded.dims() != [1, 1, time, time]
        || input.excluded.dtype() != DType::U8
        || input.query.dtype() != DType::F32
        || input.key.dtype() != DType::F32
        || input.value.dtype() != DType::F32
        || positions.query != (0..time)
        || positions.key != (0..time)
    {
        return Err(invalid("invalid native GQA tensors / causal mask"));
    }
    let group = heads / kv_heads;
    let repeat = |tensor: &Tensor| -> Result<Tensor> {
        if group == 1 {
            return Ok(tensor.clone());
        }
        Ok(tensor
            .unsqueeze(2)?
            .broadcast_as((batch, kv_heads, group, time, width))?
            .contiguous()?
            .reshape((batch, heads, time, width))?)
    };
    let key = repeat(&input.key)?;
    let value = repeat(&input.value)?;
    let scores = input
        .query
        .matmul(&key.transpose(2, 3)?.contiguous()?)?
        .affine(1.0 / (width as f64).sqrt(), 0.0)?;
    let mask = input.excluded.broadcast_as(scores.shape())?;
    let excluded = Tensor::full(f32::NEG_INFINITY, scores.shape(), scores.device())?;
    let masked = mask.where_cond(&excluded, &scores)?;
    // softmax_last_dim is inference-only in Candle; ordinary softmax is not.
    let probabilities = candle_nn::ops::softmax(&masked, 3)?;
    Ok(probabilities.matmul(&value.contiguous()?)?)
}

fn causal_mask(positions: &AttentionPositions, device: &Device) -> Result<Tensor> {
    if positions.query.start != 0
        || positions.key.start != 0
        || positions.query.end == 0
        || positions.query != positions.key
    {
        return Err(invalid(
            "Track B requires equal nonempty full-prefix positions starting at zero",
        ));
    }
    let time = positions.query.end;
    let length = time
        .checked_mul(time)
        .ok_or_else(|| invalid("causal mask size overflow"))?;
    let mut causal = Vec::with_capacity(length);
    for query in positions.query.clone() {
        for key in positions.key.clone() {
            causal.push(u8::from(key > query));
        }
    }
    Ok(Tensor::from_vec(causal, (1, 1, time, time), device)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Var;

    fn tiny_model() -> Result<TrackBModel> {
        let shape = LlamaShape {
            vocab: 11,
            width: 8,
            layers: 2,
            heads: 2,
            kv_heads: 1,
            head_dim: 4,
            ffn: 12,
            rope_theta: 100000.0,
            rms_eps: 1e-5,
            tied_embeddings: true,
        };
        let mut tensors = BTreeMap::new();
        for (ordinal, (name, dimensions)) in shape.expected_tensors().into_iter().enumerate() {
            let values = (0..dimensions.iter().product::<usize>())
                .map(|index| {
                    if dimensions.len() == 1 {
                        0.8 + index as f32 * 0.03
                    } else {
                        ((index * 13 + ordinal * 7) % 37) as f32 / 74.0 - 0.25
                    }
                })
                .collect::<Vec<_>>();
            tensors.insert(name, Tensor::from_vec(values, dimensions, &Device::Cpu)?);
        }
        TrackBModel::from_checkpoint(
            Checkpoint {
                shape,
                tensors,
                weights_sha256: "fixture".into(),
            },
            32,
            &Device::Cpu,
        )
    }

    fn max_abs(a: &Tensor, b: &Tensor) -> Result<f32> {
        Ok(a.sub(b)?.abs()?.flatten_all()?.max(0)?.to_scalar::<f32>()?)
    }

    #[test]
    fn dense_full_prefix_is_causal_and_batch_rows_are_independent() -> Result<()> {
        let model = tiny_model()?;
        let original = model.forward(&[1, 2, 3, 4, 5, 6, 7, 8], 2, 4)?;
        let changed = model.forward(&[1, 2, 9, 10, 0, 1, 2, 3], 2, 4)?;
        let prefix = model.forward(&[1, 2], 1, 2)?;
        let original_prefix = original.narrow(0, 0, 1)?.narrow(1, 0, 2)?;
        let changed_prefix = changed.narrow(0, 0, 1)?.narrow(1, 0, 2)?;
        assert!(max_abs(&original_prefix, &prefix)? < 1e-5);
        assert!(max_abs(&original_prefix, &changed_prefix)? < 1e-6);
        assert!(model.forward(&[], 1, 0).is_err());
        assert!(model.forward(&[11], 1, 1).is_err());
        Ok(())
    }

    #[test]
    fn teacher_capture_matches_full_forward_hook_and_stops_before_later_mlp() -> Result<()> {
        let mut model = tiny_model()?;
        let ids = [1, 2, 3];
        let capture = model.capture_layer(&ids, 1, 3, 0)?;
        let weights = model.attention_weights(0)?;
        let mut observed = None;
        let _ = model.forward_with_attention_fn(&ids, 1, 3, |layer, qkv, positions| {
            let attended = dense_attention(qkv, positions)?;
            if layer == 0 {
                observed = Some(
                    linear(
                        &attended.transpose(1, 2)?.contiguous()?.reshape((3, 8))?,
                        &weights.output,
                    )?
                    .reshape((1, 3, 8))?,
                );
            }
            Ok(attended)
        })?;
        let observed = observed.ok_or_else(|| invalid("test hook did not capture layer zero"))?;
        assert_eq!(capture.normalized_input.dims(), &[1, 3, 8]);
        assert!(max_abs(&capture.attention_output, &observed)? < 1e-6);
        // Capture must not execute even the selected layer's MLP. Deliberately
        // corrupt its private test-only shape to observe that early-exit seam.
        model.layers[0].gate = Tensor::zeros((1, 1), DType::F32, &Device::Cpu)?;
        assert!(model.capture_layer(&ids, 1, 3, 0).is_ok());
        assert!(model.forward(&ids, 1, 3).is_err());
        Ok(())
    }

    #[test]
    fn hook_gradients_cross_later_frozen_layers_and_weights_stay_ordinary() -> Result<()> {
        let model = tiny_model()?;
        let gain = Var::from_tensor(&Tensor::new(0.9f32, &Device::Cpu)?)?;
        let ids = [1, 2, 3];
        let logits = model.forward_with_attention_fn(&ids, 1, 3, |layer, qkv, positions| {
            let dense = dense_attention(qkv, positions)?;
            if layer == 0 {
                Ok(dense.broadcast_mul(gain.as_tensor())?)
            } else {
                Ok(dense)
            }
        })?;
        let target = Tensor::from_slice(&[0u32, 1, 2], (3, 1), &Device::Cpu)?;
        let loss = candle_nn::ops::log_softmax(&logits.reshape((3, 11))?, 1)?
            .gather(&target, 1)?
            .mean_all()?
            .neg()?;
        let gradients = loss.backward()?;
        let gradient = gradients
            .get(&gain)
            .ok_or_else(|| invalid("replacement gradient missing"))?
            .to_scalar::<f32>()?;
        assert!(gradient.is_finite() && gradient.abs() > 1e-7);
        let weights = model.attention_weights(0)?;
        assert!(!weights.query.is_variable());
        assert!(!weights.output.is_variable());
        assert!(model
            .forward_with_attention_fn(&ids, 1, 3, |_, qkv, _| {
                Ok(Tensor::zeros((1, 1), DType::F32, qkv.query.device())?)
            })
            .is_err());
        Ok(())
    }

    #[test]
    fn batched_2048_position_metadata_and_causal_mask_have_no_small_context_limit() -> Result<()> {
        let time = 2048;
        let positions = AttentionPositions {
            query: 0..time,
            key: 0..time,
        };
        let mask = causal_mask(&positions, &Device::Cpu)?;
        assert_eq!(mask.dims(), &[1, 1, time, time]);
        // Broadcasting is a shape view: exercise the two-batch GQA-compatible
        // contract without a full model or allocating floating T*T score arrays.
        let batched = mask.broadcast_as((2, 2, time, time))?;
        assert_eq!(batched.dims(), &[2, 2, time, time]);
        let entries = mask.flatten_all()?.to_vec1::<u8>()?;
        assert_eq!(entries[0], 0);
        assert_eq!(entries[time - 1], 1);
        assert_eq!(entries[(time - 1) * time], 0);
        assert_eq!(entries[time * time - 1], 0);
        assert_eq!(
            entries
                .iter()
                .map(|entry| usize::from(*entry))
                .sum::<usize>(),
            time * (time - 1) / 2
        );
        let shifted = AttentionPositions {
            query: 1..time + 1,
            key: 0..time,
        };
        assert!(causal_mask(&shifted, &Device::Cpu).is_err());
        Ok(())
    }
}
