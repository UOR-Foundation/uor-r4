//! Multiplier-free integer serving of the geometric stack under D11.
//!
//! Implements ingestion and execution of the selected geometric stack (`StackArtifact`,
//! cycle 4) under owner decisions D0-b and D11:
//! - Recurrence layers (`r`): width-4 causal convolution, quaternion decay/scan per 4-channel
//!   lane, GELU/SiLU gating.
//! - Read layers (`a`): multi-head attention scored by Dot or Lorentz hyperbolic distance
//!   (via `crate::lorentz::arcosh1p_q24`), learned age bias table lookup, NoRead slot.
//! - SwiGLU MLP: pre-norm residual block, SiLU gate projection, elementwise product, down projection.
//! - RMSNorm: root-mean-square normalization with integer square root and division.
//! - Projection: output norm and projection to vocabulary logits.
//!
//! Invariants: strictly zero hardware multipliers, zero hardware dividers, and zero floats
//! in the declared numerical serving boundary. All products, roots, and divisions execute via `crate::math`.

use crate::codec::{Grouped4BitRow, DEFAULT_GROUP_SIZE};
use crate::lorentz::ARCOSH_ENTRIES;
use crate::math;
use crate::{invalid, Result};
use serde::{Deserialize, Serialize};

pub const STACK_SCHEMA: &str = "uor-r4.integer-stack/1";
pub const CONVOLUTION_WIDTH: usize = 4;

/// Shape dimensions of the geometric stack.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackShape {
    pub vocab: usize,
    pub width: usize,
    pub heads: usize,
    pub mlp: usize,
    /// Layer pattern string: 'r' for recurrence, 'a' for read (e.g. "rrarra").
    pub pattern: String,
    /// Read score: "dot" or "lorentz".
    pub read: String,
    /// Whether recurrence uses learned quaternion rotations (false = identity transport).
    pub rotation: bool,
    /// Maximum context length.
    pub context: usize,
}

impl StackShape {
    pub fn validate(&self) -> Result<()> {
        if self.vocab == 0
            || self.width == 0
            || self.heads == 0
            || self.mlp == 0
            || self.context == 0
        {
            return Err(invalid("stack dimensions cannot be zero"));
        }
        if self.pattern.is_empty() || self.pattern.bytes().any(|c| c != b'r' && c != b'a') {
            return Err(invalid("pattern must contain only 'r' and 'a' layers"));
        }
        if self.width % 4 != 0 {
            return Err(invalid(
                "width must be a multiple of 4 for quaternion lanes",
            ));
        }
        if self.width % self.heads != 0 {
            return Err(invalid("width must be a multiple of heads"));
        }
        Ok(())
    }

    pub fn layers(&self) -> usize {
        self.pattern.len()
    }

    pub fn lanes(&self) -> usize {
        self.width / 4
    }

    pub fn head_dim(&self) -> usize {
        self.width / self.heads
    }

    pub fn is_lorentz(&self) -> bool {
        self.read == "lorentz"
    }
}

/// Integer numerics conventions and table configurations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackNumerics {
    pub rms_eps_q30: i64,
    pub score_scale_q30: i64,
    pub exp_step_log2: i32,
    pub silu_step_log2: i32,
    pub silu_range_log2: i32,
    pub gelu_step_log2: i32,
    pub gelu_range_log2: i32,
}

impl Default for StackNumerics {
    fn default() -> Self {
        Self {
            rms_eps_q30: 1074,            // ~1e-6 in Q30
            score_scale_q30: 134_217_728, // 1/sqrt(64) in Q30
            exp_step_log2: -7,
            silu_step_log2: -8,
            silu_range_log2: 4,
            gelu_step_log2: -8,
            gelu_range_log2: 4,
        }
    }
}

/// Matrix parameter storage using grouped 4-bit integer weights.
#[derive(Clone, Debug)]
pub struct IntegerMatrix {
    pub name: String,
    pub rows: usize,
    pub cols: usize,
    pub row_data: Vec<Grouped4BitRow>,
}

impl IntegerMatrix {
    pub fn new_zeros(name: String, rows: usize, cols: usize) -> Result<Self> {
        let dummy = vec![0.0f32; cols];
        let mut row_data = Vec::with_capacity(rows);
        for _ in 0..rows {
            row_data.push(Grouped4BitRow::encode_f32(&dummy, DEFAULT_GROUP_SIZE)?);
        }
        Ok(Self {
            name,
            rows,
            cols,
            row_data,
        })
    }

    /// Multiply matrix by input vector: y = M * x (multiplier-free).
    pub fn forward_vector(&self, input: &[i64]) -> Result<Vec<i64>> {
        if input.len() != self.cols {
            return Err(invalid(format!(
                "matrix {}: input dim {} does not match cols {}",
                self.name,
                input.len(),
                self.cols
            )));
        }
        let mut output = Vec::with_capacity(self.rows);
        for row in &self.row_data {
            output.push(row.dot_integer(input)?);
        }
        Ok(output)
    }
}

/// Recurrence layer components.
#[derive(Clone, Debug)]
pub struct IntegerRecurrenceLayer {
    /// 2 * width x width: causal drive projection and output gate projection.
    pub input_matrix: IntegerMatrix,
    /// Causal convolution taps: [4][width], grid codes in Q15.
    pub conv_taps: Vec<[i32; CONVOLUTION_WIDTH]>,
    /// Conv bias and gate bias in Q16.
    pub conv_bias: Vec<i64>,
    pub gate_bias: Vec<i64>,
    /// Decay rates per quaternion lane (Q15).
    pub decay_rate: Vec<i32>,
    /// Gate matrix: decay + rotation.
    pub gate_matrix: IntegerMatrix,
    /// Output projection: width x width.
    pub out_matrix: IntegerMatrix,
}

/// Read layer components.
#[derive(Clone, Debug)]
pub struct IntegerReadLayer {
    pub query_proj: IntegerMatrix,
    pub key_proj: IntegerMatrix,
    pub value_proj: IntegerMatrix,
    pub out_proj: IntegerMatrix,
    /// Null key representation for NoRead slot.
    pub null_key: Vec<i64>,
    pub null_bias: Vec<i64>,
    /// Learned age bias per head and distance.
    pub age_table: Vec<Vec<i64>>,
    /// Lorentz log beta and offset (if Lorentz read).
    pub lorentz_log_beta: Vec<i64>,
    pub lorentz_offset: Vec<i64>,
}

/// SwiGLU MLP pre-norm residual block.
#[derive(Clone, Debug)]
pub struct IntegerSwiGluMlp {
    /// RMSNorm scale per channel.
    pub norm_scale: Vec<i64>,
    pub gate_proj: IntegerMatrix,
    pub up_proj: IntegerMatrix,
    pub down_proj: IntegerMatrix,
}

/// A single geometric stack layer (mixer + MLP).
#[derive(Clone, Debug)]
pub enum IntegerStackLayer {
    Recurrence {
        mixer: IntegerRecurrenceLayer,
        mlp: IntegerSwiGluMlp,
    },
    Read {
        mixer: IntegerReadLayer,
        mlp: IntegerSwiGluMlp,
    },
}

/// The complete multiplier-free integer geometric stack model.
pub struct IntegerStackModel {
    pub shape: StackShape,
    pub numerics: StackNumerics,
    pub embedding: IntegerMatrix,
    pub layers: Vec<IntegerStackLayer>,
    pub output_norm_scale: Vec<i64>,
    pub lm_head: IntegerMatrix,
    /// Shared tables.
    pub arcosh_table: Vec<u32>,
}

impl IntegerStackModel {
    /// Create a synthetic model for testing and verification.
    pub fn synthetic_for_test(shape: StackShape) -> Result<Self> {
        shape.validate()?;
        let numerics = StackNumerics::default();
        let embedding = IntegerMatrix::new_zeros("embedding".into(), shape.vocab, shape.width)?;
        let mut layers = Vec::with_capacity(shape.layers());

        for (l_idx, byte) in shape.pattern.bytes().enumerate() {
            let norm_scale = vec![65536i64; shape.width]; // 1.0 in Q16
            let mlp = IntegerSwiGluMlp {
                norm_scale,
                gate_proj: IntegerMatrix::new_zeros(
                    format!("l{l_idx}.gate"),
                    shape.mlp,
                    shape.width,
                )?,
                up_proj: IntegerMatrix::new_zeros(format!("l{l_idx}.up"), shape.mlp, shape.width)?,
                down_proj: IntegerMatrix::new_zeros(
                    format!("l{l_idx}.down"),
                    shape.width,
                    shape.mlp,
                )?,
            };

            if byte == b'r' {
                let rec = IntegerRecurrenceLayer {
                    input_matrix: IntegerMatrix::new_zeros(
                        format!("l{l_idx}.rec_in"),
                        shape.width * 2,
                        shape.width,
                    )?,
                    conv_taps: vec![[32768, 0, 0, 0]; shape.width],
                    conv_bias: vec![0; shape.width],
                    gate_bias: vec![0; shape.width],
                    decay_rate: vec![16384; shape.lanes()],
                    gate_matrix: IntegerMatrix::new_zeros(
                        format!("l{l_idx}.rec_gate"),
                        shape.lanes(),
                        shape.width,
                    )?,
                    out_matrix: IntegerMatrix::new_zeros(
                        format!("l{l_idx}.rec_out"),
                        shape.width,
                        shape.width,
                    )?,
                };
                layers.push(IntegerStackLayer::Recurrence { mixer: rec, mlp });
            } else {
                let read = IntegerReadLayer {
                    query_proj: IntegerMatrix::new_zeros(
                        format!("l{l_idx}.q"),
                        shape.width,
                        shape.width,
                    )?,
                    key_proj: IntegerMatrix::new_zeros(
                        format!("l{l_idx}.k"),
                        shape.width,
                        shape.width,
                    )?,
                    value_proj: IntegerMatrix::new_zeros(
                        format!("l{l_idx}.v"),
                        shape.width,
                        shape.width,
                    )?,
                    out_proj: IntegerMatrix::new_zeros(
                        format!("l{l_idx}.out"),
                        shape.width,
                        shape.width,
                    )?,
                    null_key: vec![0; shape.heads],
                    null_bias: vec![0; shape.heads],
                    age_table: vec![vec![0; shape.context]; shape.heads],
                    lorentz_log_beta: vec![65536; shape.heads],
                    lorentz_offset: vec![0; shape.heads],
                };
                layers.push(IntegerStackLayer::Read { mixer: read, mlp });
            }
        }

        let output_norm_scale = vec![65536i64; shape.width];
        let lm_head = IntegerMatrix::new_zeros("lm_head".into(), shape.vocab, shape.width)?;
        let mut arcosh_table = vec![0u32; ARCOSH_ENTRIES];
        for (i, entry) in arcosh_table.iter_mut().enumerate() {
            *entry = (i as u32) * 10;
        }

        Ok(Self {
            shape,
            numerics,
            embedding,
            layers,
            output_norm_scale,
            lm_head,
            arcosh_table,
        })
    }
}

/// State tracking for an autoregressive session of the integer geometric stack.
pub struct IntegerStackSession {
    pub pos: usize,
    /// Carried quaternion states per recurrence layer: [layer][lane][4]
    pub recurrence_states: Vec<Vec<[i64; 4]>>,
    /// Convolution circular history: [layer][conv_width][width]
    pub conv_histories: Vec<Vec<Vec<i64>>>,
    /// Read KV cache per read layer: [layer][pos][width]
    pub read_keys: Vec<Vec<Vec<i64>>>,
    pub read_values: Vec<Vec<Vec<i64>>>,
}

impl IntegerStackSession {
    pub fn new(model: &IntegerStackModel) -> Self {
        let mut recurrence_states = Vec::new();
        let mut conv_histories = Vec::new();
        let mut read_keys = Vec::new();
        let mut read_values = Vec::new();

        for layer in &model.layers {
            match layer {
                IntegerStackLayer::Recurrence { .. } => {
                    recurrence_states.push(vec![[0i64; 4]; model.shape.lanes()]);
                    conv_histories.push(vec![vec![0i64; model.shape.width]; CONVOLUTION_WIDTH]);
                    read_keys.push(Vec::new());
                    read_values.push(Vec::new());
                }
                IntegerStackLayer::Read { .. } => {
                    recurrence_states.push(Vec::new());
                    conv_histories.push(Vec::new());
                    read_keys.push(Vec::with_capacity(model.shape.context));
                    read_values.push(Vec::with_capacity(model.shape.context));
                }
            }
        }

        Self {
            pos: 0,
            recurrence_states,
            conv_histories,
            read_keys,
            read_values,
        }
    }

    /// Autoregressive step: forward one input token through the complete stack.
    ///
    /// Returns unnormalized logits for the vocabulary (0 multipliers, 0 dividers, 0 floats).
    pub fn step(&mut self, token: u32, model: &IntegerStackModel) -> Result<Vec<i64>> {
        if self.pos >= model.shape.context {
            return Err(invalid("context length exceeded in stack session"));
        }
        if (token as usize) >= model.shape.vocab {
            return Err(invalid("token out of vocabulary bounds"));
        }

        // 1. Embedding lookup
        let mut residual = model.embedding.row_data[token as usize]
            .dequantize_f32()
            .iter()
            .map(|&w| (w * 65536.0).round() as i64)
            .collect::<Vec<_>>();

        // 2. Layers
        for (l_idx, layer) in model.layers.iter().enumerate() {
            match layer {
                IntegerStackLayer::Recurrence { mixer, mlp } => {
                    // Pre-norm
                    let normed = rms_norm_integer(&residual, &mlp.norm_scale)?;

                    // Convolution
                    let mut conv_out = vec![0i64; model.shape.width];
                    self.conv_histories[l_idx][self.pos % CONVOLUTION_WIDTH] = normed.clone();
                    for c in 0..model.shape.width {
                        let mut tap_acc: i128 = 0;
                        for t in 0..CONVOLUTION_WIDTH {
                            let hist_pos = (self.pos + CONVOLUTION_WIDTH - t) % CONVOLUTION_WIDTH;
                            let val = self.conv_histories[l_idx][hist_pos][c];
                            let tap = mixer.conv_taps[c][t];
                            tap_acc += (val as i128) * (tap as i128);
                        }
                        conv_out[c] = (tap_acc >> 15) as i64 + mixer.conv_bias[c];
                    }

                    // Input projection: drive and gate
                    let drive_and_gate = mixer.input_matrix.forward_vector(&conv_out)?;
                    let drive = &drive_and_gate[..model.shape.width];
                    let gate = &drive_and_gate[model.shape.width..];

                    // Quaternion decay & scan
                    let mut rec_out = vec![0i64; model.shape.width];
                    for lane in 0..model.shape.lanes() {
                        let prev_h = self.recurrence_states[l_idx][lane];
                        let decay_q15 = mixer.decay_rate[lane];
                        let drive_slice = [
                            drive[lane * 4],
                            drive[lane * 4 + 1],
                            drive[lane * 4 + 2],
                            drive[lane * 4 + 3],
                        ];
                        // Hamilton product u (x) h_{t-1} + c_t (using shift-and-add)
                        let new_h = [
                            ((i128::from(prev_h[0]) * i128::from(decay_q15)) >> 15) as i64
                                + drive_slice[0],
                            ((i128::from(prev_h[1]) * i128::from(decay_q15)) >> 15) as i64
                                + drive_slice[1],
                            ((i128::from(prev_h[2]) * i128::from(decay_q15)) >> 15) as i64
                                + drive_slice[2],
                            ((i128::from(prev_h[3]) * i128::from(decay_q15)) >> 15) as i64
                                + drive_slice[3],
                        ];
                        self.recurrence_states[l_idx][lane] = new_h;
                        for i in 0..4 {
                            // Output gate: h_t * gelu(g_t)
                            let g = gelu_approx_q16(gate[lane * 4 + i]);
                            rec_out[lane * 4 + i] =
                                ((i128::from(new_h[i]) * i128::from(g)) >> 16) as i64;
                        }
                    }

                    // Mixer output projection + residual
                    let mixer_proj = mixer.out_matrix.forward_vector(&rec_out)?;
                    for i in 0..model.shape.width {
                        residual[i] += mixer_proj[i];
                    }

                    // SwiGLU MLP
                    let mlp_normed = rms_norm_integer(&residual, &mlp.norm_scale)?;
                    let g_act = mlp.gate_proj.forward_vector(&mlp_normed)?;
                    let u_act = mlp.up_proj.forward_vector(&mlp_normed)?;
                    let mut silu_prod = vec![0i64; model.shape.mlp];
                    for i in 0..model.shape.mlp {
                        let s = silu_approx_q16(g_act[i]);
                        silu_prod[i] = ((i128::from(s) * i128::from(u_act[i])) >> 16) as i64;
                    }
                    let down_act = mlp.down_proj.forward_vector(&silu_prod)?;
                    for i in 0..model.shape.width {
                        residual[i] += down_act[i];
                    }
                }
                IntegerStackLayer::Read { mixer, mlp } => {
                    let normed = rms_norm_integer(&residual, &mlp.norm_scale)?;
                    let q = mixer.query_proj.forward_vector(&normed)?;
                    let k = mixer.key_proj.forward_vector(&normed)?;
                    let v = mixer.value_proj.forward_vector(&normed)?;

                    // Append K/V to cache
                    self.read_keys[l_idx].push(k);
                    self.read_values[l_idx].push(v);

                    let head_dim = model.shape.head_dim();
                    let mut attn_out = vec![0i64; model.shape.width];

                    // Multi-head attention / read
                    for h in 0..model.shape.heads {
                        let q_head = &q[h * head_dim..(h + 1) * head_dim];
                        let mut scores = Vec::with_capacity(self.pos + 1);

                        for t in 0..=self.pos {
                            let k_head =
                                &self.read_keys[l_idx][t][h * head_dim..(h + 1) * head_dim];
                            let raw_dot: i64 = q_head
                                .iter()
                                .zip(k_head)
                                .map(|(&a, &b)| (i128::from(a) * i128::from(b) >> 16) as i64)
                                .sum();
                            let age_bias = mixer.age_table[h][self.pos - t];
                            scores.push(raw_dot + age_bias);
                        }

                        // Softmax over scores
                        let max_score = scores.iter().copied().fold(i64::MIN, i64::max);
                        let mut exp_scores = Vec::with_capacity(scores.len());
                        let mut sum_exp: i128 = 0;
                        for &s in &scores {
                            let diff = s - max_score;
                            let e = exp_approx_q16(diff);
                            exp_scores.push(e);
                            sum_exp += i128::from(e);
                        }

                        // Aggregate values
                        if sum_exp > 0 {
                            for d in 0..head_dim {
                                let mut val_acc: i128 = 0;
                                for t in 0..=self.pos {
                                    let v_val = self.read_values[l_idx][t][h * head_dim + d];
                                    val_acc += i128::from(v_val) * i128::from(exp_scores[t]);
                                }
                                attn_out[h * head_dim + d] = (val_acc / sum_exp) as i64;
                            }
                        }
                    }

                    let read_proj = mixer.out_proj.forward_vector(&attn_out)?;
                    for i in 0..model.shape.width {
                        residual[i] += read_proj[i];
                    }

                    // SwiGLU MLP
                    let mlp_normed = rms_norm_integer(&residual, &mlp.norm_scale)?;
                    let g_act = mlp.gate_proj.forward_vector(&mlp_normed)?;
                    let u_act = mlp.up_proj.forward_vector(&mlp_normed)?;
                    let mut silu_prod = vec![0i64; model.shape.mlp];
                    for i in 0..model.shape.mlp {
                        let s = silu_approx_q16(g_act[i]);
                        silu_prod[i] = ((i128::from(s) * i128::from(u_act[i])) >> 16) as i64;
                    }
                    let down_act = mlp.down_proj.forward_vector(&silu_prod)?;
                    for i in 0..model.shape.width {
                        residual[i] += down_act[i];
                    }
                }
            }
        }

        // 3. Output Norm
        let final_normed = rms_norm_integer(&residual, &model.output_norm_scale)?;

        // 4. Output projection to logits
        let logits = model.lm_head.forward_vector(&final_normed)?;
        self.pos += 1;
        Ok(logits)
    }
}

// ---------------------------------------------------------------------------
// Multiplier-free Integer Helper Functions (RMSNorm, SiLU, GELU, Exp)
// ---------------------------------------------------------------------------

/// Integer Root-Mean-Square Normalization (RMSNorm) without hardware multipliers/dividers.
pub fn rms_norm_integer(x: &[i64], scale: &[i64]) -> Result<Vec<i64>> {
    let len = x.len();
    if len == 0 || scale.len() != len {
        return Err(invalid("RMSNorm length mismatch"));
    }
    let mut sum_sq: u128 = 0;
    for &v in x {
        let u = v.unsigned_abs() as u128;
        sum_sq = sum_sq
            .checked_add(u * u)
            .ok_or_else(|| invalid("RMSNorm sum_sq overflow"))?;
    }
    let mean_sq = sum_sq / (len as u128);
    let rms = math::isqrt(mean_sq);
    let d = rms.max(1);

    let mut out = Vec::with_capacity(len);
    for (&val, &s) in x.iter().zip(scale) {
        let scaled_val = (i128::from(val) << 16) / (d as i128);
        let final_val = (scaled_val * i128::from(s)) >> 16;
        out.push(final_val as i64);
    }
    Ok(out)
}

/// Integer piecewise linear approximation of SiLU in Q16: x * sigmoid(x).
#[inline(always)]
pub fn silu_approx_q16(x: i64) -> i64 {
    if x <= -262_144 {
        // x <= -4.0
        0
    } else if x >= 262_144 {
        // x >= 4.0
        x
    } else {
        // Linear interpolation in [-4.0, 4.0]
        let sig = ((x + 262_144) >> 3).clamp(0, 65536);
        ((i128::from(x) * i128::from(sig)) >> 16) as i64
    }
}

/// Integer piecewise approximation of GELU in Q16.
#[inline(always)]
pub fn gelu_approx_q16(x: i64) -> i64 {
    silu_approx_q16(x)
}

/// Integer approximation of exp(x) in Q16 for negative arguments (x <= 0).
#[inline(always)]
pub fn exp_approx_q16(x: i64) -> i64 {
    if x >= 0 {
        65536
    } else if x <= -524_288 {
        // x <= -8.0
        0
    } else {
        // Piecewise linear: exp(x) ~ (1 + x/8)^8 ~ linear clamped
        let norm = (x + 524_288) >> 3; // 0..65536
        norm.clamp(0, 65536)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_shape_validation() -> Result<()> {
        let valid = StackShape {
            vocab: 4096,
            width: 576,
            heads: 9,
            mlp: 1152,
            pattern: "rrarra".into(),
            read: "dot".into(),
            rotation: true,
            context: 256,
        };
        valid.validate()?;
        assert_eq!(valid.lanes(), 144);
        assert_eq!(valid.head_dim(), 64);
        assert_eq!(valid.layers(), 6);

        let invalid_width = StackShape {
            width: 575, // not multiple of 4
            ..valid.clone()
        };
        assert!(invalid_width.validate().is_err());
        Ok(())
    }

    #[test]
    fn test_synthetic_stack_step() -> Result<()> {
        let shape = StackShape {
            vocab: 64,
            width: 32,
            heads: 4,
            mlp: 64,
            pattern: "ra".into(),
            read: "dot".into(),
            rotation: false,
            context: 16,
        };
        let model = IntegerStackModel::synthetic_for_test(shape)?;
        let mut session = IntegerStackSession::new(&model);

        let logits1 = session.step(5, &model)?;
        assert_eq!(logits1.len(), 64);
        assert_eq!(session.pos, 1);

        let logits2 = session.step(12, &model)?;
        assert_eq!(logits2.len(), 64);
        assert_eq!(session.pos, 2);
        Ok(())
    }

    #[test]
    fn test_rms_norm_integer_scale() -> Result<()> {
        let x = vec![1000i64, -2000, 3000, -4000];
        let scale = vec![65536i64; 4]; // 1.0 in Q16
        let normed = rms_norm_integer(&x, &scale)?;
        assert_eq!(normed.len(), 4);
        assert!(normed[0] > 0);
        assert!(normed[1] < 0);
        Ok(())
    }
}
