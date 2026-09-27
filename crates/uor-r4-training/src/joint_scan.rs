//! R2 scan-native dialogue cell (part 1: cell, monoid, parity harness).
//!
//! Declared vehicle change from the R1c joint cell. The historical cell has no
//! exact finite-state scan because its gate/transport inputs depend on the
//! recurrent state and its read couples all prior events. This module defines
//! the scan-native cell:
//!
//! ```text
//! f_t = e[tok_t]·W_inᵀ + b            (token affine only; no RMSNorm(s_{t-1}))
//! u_t = tanh(f_t[0:d]); z_t = σ(f_t[d:2d]); v_t = f_t[2d:3d]
//! h_t = (1−z_t)⊙Tport(h_{t-1}, v_t) + z_t⊙u_t     associative R4 transport scan
//! k_i = W_k·RMSNorm(h_i); val_i = tanh(W_v·RMSNorm(h_i))
//! read_t = local causal softmax over the last W events (+ optional linear long-range)
//! m_t = tanh(W_up·[h_t; read_t]);  ρ_t = σ(W_ρ·[h_t; read_t])
//! s_t = (1−ρ_t)⊙h_t + ρ_t⊙m_t                     output state (not the recurrence)
//! c_t = σ(W_c·[s_t; read_t])
//! ```
//!
//! The recurrence `h_t = A_t h_{t-1} + d_t` with block-diagonal
//! `A_t = diag(1−z_t)·blkdiag₄(R(q_t))` and `d_t = z_t⊙u_t` is exactly a
//! block-diagonal monoid, so a chunk is summarised by `(Φ, ψ)` and two
//! summaries compose associatively. The read/update/overt `s_t` is a
//! per-position output transform derived from `h`; it is **not** fed back into
//! the recurrence. That separation is what makes chunked and sequential
//! evaluation of the same cell two orders of one pure function. The scan
//! primary variant allocates and uses no `recurrent.state.weight`.
//!
//! This is continuous F32 offline training code. It is not integer serving.

use candle_core::{DType, Device, Tensor};
use serde::{Deserialize, Serialize};

use crate::joint_model::{JointConfig, JointModel, JointOutput, ReadMode, Transport};
use crate::joint_parallel::MaskedBatchGradients;
use crate::{invalid, Result};

/// Fixed decay of the optional linear long-range value memory. No new
/// parameter shape is introduced by the linear-read mode.
const LINEAR_READ_DECAY: f64 = 0.5;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanMode {
    GatedTransport,
    GatedTransportLinearRead,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScanConfig {
    pub chunk: usize,
    pub window: usize,
    pub mode: ScanMode,
}

impl ScanConfig {
    pub fn validate(&self, context: usize) -> Result<()> {
        if !matches!(self.chunk, 16 | 32 | 64 | 128) {
            return Err(invalid("scan chunk must be 16, 32, 64 or 128"));
        }
        if !matches!(self.window, 32 | 64) {
            return Err(invalid("scan window must be 32 or 64"));
        }
        if context == 0 || context % self.chunk != 0 {
            return Err(invalid("scan context must be a positive multiple of chunk"));
        }
        if self.window > context {
            return Err(invalid("scan window must not exceed context"));
        }
        Ok(())
    }
}

/// Block-diagonal chunk summary `(Φ, ψ)`: `Φ` is one 4×4 matrix per R4 lane,
/// `ψ` is the accumulated driving term.
#[derive(Clone)]
pub struct ScanSummary {
    pub phi: Tensor,
    pub psi: Tensor,
}

struct ScanInputs {
    z: Tensor,
    u: Tensor,
    q: Tensor,
}

fn identity_summary(
    batch: usize,
    lanes: usize,
    dtype: DType,
    device: &Device,
) -> Result<ScanSummary> {
    Ok(ScanSummary {
        phi: Tensor::eye(4, dtype, device)?
            .unsqueeze(0)?
            .unsqueeze(0)?
            .broadcast_as((batch, lanes, 4, 4))?
            .contiguous()?,
        psi: Tensor::zeros((batch, lanes * 4), dtype, device)?,
    })
}

fn apply_phi(phi: &Tensor, x: &Tensor) -> Result<Tensor> {
    let (batch, width) = x.dims2()?;
    let lanes = width / 4;
    let column = x.reshape((batch, lanes, 4, 1))?;
    Ok(phi.matmul(&column)?.reshape((batch, width))?)
}

/// Compose two adjacent summaries: `a` covers the earlier range, `b` the later
/// range, and the result covers the concatenation. Associative by construction.
pub fn compose(a: &ScanSummary, b: &ScanSummary) -> Result<ScanSummary> {
    let phi = b.phi.matmul(&a.phi)?;
    let psi = apply_phi(&b.phi, &a.psi)?.add(&b.psi)?;
    Ok(ScanSummary { phi, psi })
}

/// Inclusive prefix of a summary sequence, in sequence order.
pub fn scan_prefix(summaries: &[ScanSummary]) -> Result<Vec<ScanSummary>> {
    if summaries.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::with_capacity(summaries.len());
    let mut accumulator = summaries[0].clone();
    out.push(accumulator.clone());
    for summary in &summaries[1..] {
        accumulator = compose(&accumulator, summary)?;
        out.push(accumulator.clone());
    }
    Ok(out)
}

fn rms(input: &Tensor) -> Result<Tensor> {
    Ok(input.broadcast_div(
        &input
            .sqr()?
            .mean_keepdim(1)?
            .affine(1.0, crate::joint_model::RMS_EPSILON)?
            .sqrt()?,
    )?)
}

impl JointModel {
    /// Offline scan-native constructor. Widths outside the retained serving
    /// loader contract are permitted here (dialogue-only, never served).
    /// Drops `recurrent.state.weight`, which the input-only transition replaces.
    pub fn new_dialogue_scan(
        config: JointConfig,
        scan: ScanConfig,
        device: &Device,
    ) -> Result<Self> {
        scan.validate(config.context)?;
        if config.width % 4 != 0 {
            return Err(invalid("scan width must be a multiple of four"));
        }
        let mut model = Self::new_dialogue(config, device)?;
        model.drop_parameter("recurrent.state.weight")?;
        model.scan = Some(scan);
        Ok(model)
    }

    fn scan_config(&self) -> Result<&ScanConfig> {
        self.scan
            .as_ref()
            .ok_or_else(|| invalid("scan operation requires a scan-native model"))
    }

    fn affine_linear(&self, input: &Tensor, prefix: &str, training: bool) -> Result<Tensor> {
        let weight = self.weight(&format!("{prefix}.weight"), training)?;
        let bias = self.weight(&format!("{prefix}.bias"), training)?;
        Ok(input.matmul(&weight.t()?)?.broadcast_add(&bias)?)
    }

    /// Per-position input-only gate/candidate/transport-derived units.
    fn scan_inputs(&self, affine: &Tensor, training: bool) -> Result<ScanInputs> {
        let (batch, time, width3) = affine.dims3()?;
        let width = self.config.width;
        if width3 != 3 * width || width % 4 != 0 {
            return Err(invalid("scan token affine shape mismatch"));
        }
        let fused = affine.broadcast_add(&self.weight("recurrent.bias", training)?)?;
        let u = fused.narrow(2, 0, width)?.tanh()?;
        let z = candle_nn::ops::sigmoid(&fused.narrow(2, width, width)?)?;
        let raw = fused.narrow(2, 2 * width, width)?;
        let lanes = width / 4;
        let raw = raw.reshape((batch, time, lanes, 4))?;
        let identity = Tensor::from_vec(vec![1f32, 0.0, 0.0, 0.0], (1, 1, 1, 4), affine.device())?
            .broadcast_as((batch, time, lanes, 4))?;
        let scale = match self.config.transport {
            Transport::Quaternion => crate::joint_model::QUATERNION_DELTA_SCALE,
            Transport::HouseholderPair => crate::joint_model::QUATERNION_DELTA_SCALE / 2f64.sqrt(),
        };
        let centered = raw.affine(scale, 0.0)?.add(&identity)?;
        let squared = centered.sqr()?.sum_keepdim(3)?;
        let floor = crate::joint_model::TRANSPORT_MIN_NORM * crate::joint_model::TRANSPORT_MIN_NORM;
        let denominator = squared.clamp(floor as f32, f32::MAX)?.sqrt()?;
        let normalized = centered.broadcast_div(&denominator)?;
        let active = squared
            .gt(floor as f32)?
            .broadcast_as((batch, time, lanes, 4))?;
        let q = active.where_cond(&normalized, &identity)?;
        Ok(ScanInputs { z, u, q })
    }

    fn quaternion_matrix(q: &Tensor) -> Result<Tensor> {
        let (batch, lanes, _) = q.dims3()?;
        let w = q.narrow(2, 0, 1)?;
        let a = q.narrow(2, 1, 1)?;
        let b = q.narrow(2, 2, 1)?;
        let c = q.narrow(2, 3, 1)?;
        let row0 = Tensor::cat(&[&w, &a.neg()?, &b.neg()?, &c.neg()?], 2)?;
        let row1 = Tensor::cat(&[&a, &w, &c.neg()?, &b], 2)?;
        let row2 = Tensor::cat(&[&b, &c, &w, &a.neg()?], 2)?;
        let row3 = Tensor::cat(&[&c, &b.neg()?, &a, &w], 2)?;
        Ok(Tensor::cat(
            &[
                &row0.unsqueeze(2)?,
                &row1.unsqueeze(2)?,
                &row2.unsqueeze(2)?,
                &row3.unsqueeze(2)?,
            ],
            2,
        )?
        .reshape((batch, lanes, 4, 4))?)
    }

    fn householder_matrix(q: &Tensor) -> Result<Tensor> {
        let (batch, lanes, _) = q.dims3()?;
        let outer = q.unsqueeze(3)?.matmul(&q.unsqueeze(2)?)?;
        let identity = Tensor::eye(4, DType::F32, q.device())?
            .broadcast_as((batch, lanes, 4, 4))?
            .contiguous()?;
        let reflection = identity.sub(&outer.affine(2.0, 0.0)?)?;
        let signs = Tensor::from_vec(vec![-1f32, 1.0, 1.0, 1.0], (1, 1, 1, 4), q.device())?;
        Ok(reflection.mul(&signs.broadcast_as((batch, lanes, 4, 4))?)?)
    }

    /// One-position transition block `A_t = diag(1−z_t)·blkdiag₄(T(q_t))`.
    fn one_step_block(&self, q: &Tensor, z: &Tensor) -> Result<Tensor> {
        let (batch, width) = z.dims2()?;
        let z = z.reshape((batch, width / 4, 4))?;
        let rotation = match self.config.transport {
            Transport::Quaternion => Self::quaternion_matrix(q)?,
            Transport::HouseholderPair => Self::householder_matrix(q)?,
        };
        let gate = z.affine(-1.0, 1.0)?.unsqueeze(3)?;
        Ok(rotation.broadcast_mul(&gate)?)
    }

    fn step_drive(&self, inputs: &ScanInputs, position: usize) -> Result<Tensor> {
        let z = inputs.z.narrow(1, position, 1)?.squeeze(1)?;
        let u = inputs.u.narrow(1, position, 1)?.squeeze(1)?;
        Ok(z.mul(&u)?)
    }

    fn chunk_summary(&self, inputs: &ScanInputs, start: usize, len: usize) -> Result<ScanSummary> {
        let (batch, _, width) = inputs.z.dims3()?;
        let lanes = width / 4;
        let mut summary = identity_summary(batch, lanes, inputs.z.dtype(), inputs.z.device())?;
        for position in start..start + len {
            let q = inputs.q.narrow(1, position, 1)?.squeeze(1)?;
            let z = inputs.z.narrow(1, position, 1)?.squeeze(1)?;
            let block = self.one_step_block(&q, &z)?;
            let drive = self.step_drive(inputs, position)?;
            summary.psi = apply_phi(&block, &summary.psi)?.add(&drive)?;
            summary.phi = block.matmul(&summary.phi)?;
        }
        Ok(summary)
    }

    /// Summaries for every chunk of `time`; `time` must be a multiple of chunk.
    pub fn chunk_summaries(
        &self,
        affine: &Tensor,
        batch: usize,
        time: usize,
    ) -> Result<Vec<ScanSummary>> {
        if affine.dim(0)? != batch || affine.dim(1)? != time {
            return Err(invalid("scan affine batch/time mismatch"));
        }
        let inputs = self.scan_inputs(affine, false)?;
        self.chunk_summaries_with(&inputs, time)
    }

    fn chunk_summaries_with(&self, inputs: &ScanInputs, time: usize) -> Result<Vec<ScanSummary>> {
        let chunk = self.scan_config()?.chunk;
        if time % chunk != 0 {
            return Err(invalid("scan time must be a multiple of chunk"));
        }
        let mut out = Vec::with_capacity(time / chunk);
        for start in (0..time).step_by(chunk) {
            out.push(self.chunk_summary(inputs, start, chunk)?);
        }
        Ok(out)
    }

    /// Resolve the entry state of a chunk from the summary of all earlier work.
    pub fn resolve_chunk(&self, entry: &ScanSummary, s0: &Tensor) -> Result<Tensor> {
        Ok(apply_phi(&entry.phi, s0)?.add(&entry.psi)?)
    }

    fn resolve_chunk_states(
        &self,
        inputs: &ScanInputs,
        start: usize,
        len: usize,
        entry: &Tensor,
    ) -> Result<Tensor> {
        let (batch, _, width) = inputs.z.dims3()?;
        let lanes = width / 4;
        let device = inputs.z.device();
        let dtype = inputs.z.dtype();
        let mut phi = identity_summary(batch, lanes, dtype, device)?.phi;
        let mut psi = Tensor::zeros((batch, width), dtype, device)?;
        let mut states = Vec::with_capacity(len);
        for position in start..start + len {
            let q = inputs.q.narrow(1, position, 1)?.squeeze(1)?;
            let z = inputs.z.narrow(1, position, 1)?.squeeze(1)?;
            let block = self.one_step_block(&q, &z)?;
            let drive = self.step_drive(inputs, position)?;
            psi = apply_phi(&block, &psi)?.add(&drive)?;
            phi = block.matmul(&phi)?;
            let state = apply_phi(&phi, entry)?.add(&psi)?;
            states.push(state);
        }
        Ok(Tensor::stack(&states, 1)?)
    }

    fn resolve_states_chunked(&self, inputs: &ScanInputs, time: usize) -> Result<Tensor> {
        let chunk = self.scan_config()?.chunk;
        let (batch, _, width) = inputs.z.dims3()?;
        let summaries = self.chunk_summaries_with(inputs, time)?;
        let prefix = scan_prefix(&summaries)?;
        let mut chunks = Vec::with_capacity(prefix.len());
        for index in 0..prefix.len() {
            let entry = if index == 0 {
                Tensor::zeros((batch, width), inputs.z.dtype(), inputs.z.device())?
            } else {
                prefix[index - 1].psi.clone()
            };
            chunks.push(self.resolve_chunk_states(inputs, index * chunk, chunk, &entry)?);
        }
        Ok(Tensor::cat(&chunks.iter().collect::<Vec<_>>(), 1)?)
    }

    fn resolve_states_sequential(&self, inputs: &ScanInputs, time: usize) -> Result<Tensor> {
        let (batch, _, width) = inputs.z.dims3()?;
        let mut state = Tensor::zeros((batch, width), inputs.z.dtype(), inputs.z.device())?;
        let mut states = Vec::with_capacity(time);
        for position in 0..time {
            let q = inputs.q.narrow(1, position, 1)?.squeeze(1)?;
            let z = inputs.z.narrow(1, position, 1)?.squeeze(1)?;
            let block = self.one_step_block(&q, &z)?;
            let drive = self.step_drive(inputs, position)?;
            state = apply_phi(&block, &state)?.add(&drive)?;
            states.push(state.clone());
        }
        Ok(Tensor::stack(&states, 1)?)
    }

    /// Read/update/output stages shared by both evaluation orders.
    fn finish_states(
        &self,
        ids: &[u32],
        states: &Tensor,
        batch: usize,
        time: usize,
        mode: ReadMode,
        training: bool,
    ) -> Result<JointOutput> {
        let scan = self.scan_config()?;
        let width = self.config.width;
        let read_width = self.config.read_width;
        let vocab = self.config.vocab_size;
        let device = self.device();
        let flat = states.reshape((batch * time, width))?;
        let normalized = rms(&flat)?;
        let keys = self
            .affine_linear(&normalized, "read.key", training)?
            .reshape((batch, time, read_width))?;
        let values = self
            .affine_linear(&normalized, "read.value", training)?
            .tanh()?
            .reshape((batch, time, width))?;
        let queries = self
            .affine_linear(&normalized, "read.query", training)?
            .reshape((batch, time, read_width))?;
        let null = self
            .affine_linear(&normalized, "read.no_read", training)?
            .reshape((batch, time, 1))?;
        let window = scan.window.min(time);
        let scale = 1.0 / (read_width as f64).sqrt();
        let mut reads = Vec::with_capacity(time);
        let mut no_reads = Vec::with_capacity(time);
        let mut masses = Vec::with_capacity(time);
        let mut copies = Vec::with_capacity(time);
        let mut memory = Tensor::zeros((batch, read_width, width), DType::F32, device)?;
        for position in 0..time {
            let start = position.saturating_sub(window);
            let length = position - start;
            let mut read = Tensor::zeros((batch, width), DType::F32, device)?;
            let mut no_read = Tensor::ones((batch, 1), DType::F32, device)?;
            let mut padded = Tensor::zeros((batch, window), DType::F32, device)?;
            if mode == ReadMode::Enabled && length > 0 {
                let key_slice = keys.narrow(1, start, length)?;
                let value_slice = values.narrow(1, start, length)?;
                let query = queries.narrow(1, position, 1)?.squeeze(1)?;
                let scores = query
                    .unsqueeze(1)?
                    .matmul(&key_slice.transpose(1, 2)?.contiguous()?)?
                    .squeeze(1)?
                    .affine(scale, 0.0)?;
                let ages: Vec<u32> = (0..length as u32).rev().collect();
                let age = self
                    .weight("read.age", training)?
                    .index_select(&Tensor::from_vec(ages, (length,), device)?, 0)?;
                let scores = scores.broadcast_add(&age)?;
                let null_row = null.narrow(1, position, 1)?.squeeze(1)?;
                let mass = candle_nn::ops::softmax(&Tensor::cat(&[&null_row, &scores], 1)?, 1)?;
                let mass = mass.narrow(1, 1, length)?.contiguous()?;
                read = mass.unsqueeze(1)?.matmul(&value_slice)?.squeeze(1)?;
                no_read = Tensor::ones((batch, 1), DType::F32, device)?
                    .sub(&mass.sum(1)?.unsqueeze(1)?)?;
                padded = Tensor::cat(
                    &[
                        &Tensor::zeros((batch, window - length), DType::F32, device)?,
                        &mass,
                    ],
                    1,
                )?;
                copies.push(self.window_copy(ids, batch, time, start, length, &mass)?);
                if scan.mode == ScanMode::GatedTransportLinearRead {
                    let long = query.unsqueeze(1)?.matmul(&memory)?.squeeze(1)?;
                    read = read.add(&long)?;
                }
            } else {
                copies.push(Tensor::zeros((batch, vocab), DType::F32, device)?);
            }
            if scan.mode == ScanMode::GatedTransportLinearRead {
                let key = keys.narrow(1, position, 1)?.squeeze(1)?;
                let value = values.narrow(1, position, 1)?.squeeze(1)?;
                let outer = key.unsqueeze(2)?.matmul(&value.unsqueeze(1)?)?;
                memory = memory.affine(LINEAR_READ_DECAY, 0.0)?.add(&outer)?;
            }
            reads.push(read);
            no_reads.push(no_read);
            masses.push(padded);
        }
        let read = Tensor::stack(&reads, 1)?;
        let no_read = Tensor::stack(&no_reads, 1)?;
        let read_masses = Tensor::stack(&masses, 1)?;
        let copy = Tensor::stack(&copies, 1)?.reshape((batch * time, vocab))?;
        let read_flat = read.reshape((batch * time, width))?;
        let update_input = Tensor::cat(&[&flat, &read_flat], 1)?;
        let update = self
            .affine_linear(&update_input, "update", training)?
            .tanh()?;
        let rho = candle_nn::ops::sigmoid(&self.affine_linear(
            &update_input,
            "update.gate",
            training,
        )?)?;
        let output_state = flat
            .broadcast_mul(&rho.affine(-1.0, 1.0)?)?
            .add(&update.broadcast_mul(&rho)?)?;
        let copy_input = Tensor::cat(&[&output_state, &read_flat], 1)?;
        let gate =
            candle_nn::ops::sigmoid(&self.affine_linear(&copy_input, "copy.gate", training)?)?;
        let probabilities = self
            .output_distribution(
                &output_state,
                &no_read.reshape((batch * time, 1))?,
                &gate,
                &copy,
                training,
            )?
            .reshape((batch, time, vocab))?;
        Ok(JointOutput {
            probabilities,
            no_read_mass: no_read.reshape((batch, time))?,
            read_masses,
            copy_gate: gate.reshape((batch, time))?,
            states: output_state.reshape((batch, time, width))?,
        })
    }

    fn window_copy(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        start: usize,
        length: usize,
        masses: &Tensor,
    ) -> Result<Tensor> {
        let vocab = self.config.vocab_size;
        let mut destinations = Vec::with_capacity(batch * length);
        for lane in 0..batch {
            for offset in 0..length {
                let token = ids[lane * time + start + offset] as usize;
                destinations.push((lane * vocab + token) as u32);
            }
        }
        let zeros = Tensor::zeros((batch * vocab,), DType::F32, self.device())?;
        Ok(zeros
            .index_add(
                &Tensor::from_vec(destinations, (batch * length,), self.device())?,
                &masses.flatten_all()?.contiguous()?,
                0,
            )?
            .reshape((batch, vocab))?)
    }

    fn scan_token_affine(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        training: bool,
    ) -> Result<Tensor> {
        let width = self.config.width;
        let embedding = self.weight("embedding.weight", training)?;
        let index = Tensor::from_vec(ids.to_vec(), (ids.len(),), self.device())?;
        Ok(embedding
            .index_select(&index, 0)?
            .matmul(&self.weight("recurrent.input.weight", training)?.t()?)?
            .reshape((batch, time, 3 * width))?)
    }

    fn validate_scan_input(&self, ids: &[u32], batch: usize, time: usize) -> Result<()> {
        if time == 0
            || time > self.config.context
            || ids.len() != batch * time
            || batch == 0
            || batch > 64
            || ids.iter().any(|&id| id as usize >= self.config.vocab_size)
        {
            return Err(invalid("scan input shape/context/vocabulary"));
        }
        Ok(())
    }

    /// Chunked scan-native forward. Same cell as [`Self::scan_sequential_forward`].
    pub fn scan_forward(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        mode: ReadMode,
        training: bool,
    ) -> Result<JointOutput> {
        self.validate_scan_input(ids, batch, time)?;
        let affine = self.scan_token_affine(ids, batch, time, training)?;
        let inputs = self.scan_inputs(&affine, training)?;
        let states = self.resolve_states_chunked(&inputs, time)?;
        self.finish_states(ids, &states, batch, time, mode, training)
    }

    /// Sequential evaluation order of the same scan-native cell. Reference for
    /// the chunked-parity harness, not a separate mechanism.
    pub fn scan_sequential_forward(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        mode: ReadMode,
        training: bool,
    ) -> Result<JointOutput> {
        self.validate_scan_input(ids, batch, time)?;
        let affine = self.scan_token_affine(ids, batch, time, training)?;
        let inputs = self.scan_inputs(&affine, training)?;
        let states = self.resolve_states_sequential(&inputs, time)?;
        self.finish_states(ids, &states, batch, time, mode, training)
    }

    /// Response-masked scan gradients, matching the historical
    /// response-masked dialogue objective: supervised-count-normalized mean.
    pub fn scan_block_gradients(
        &self,
        inputs: &[u32],
        targets: &[u32],
        masks: &[u8],
        batch: usize,
        time: usize,
    ) -> Result<MaskedBatchGradients> {
        if targets.len() != inputs.len() || masks.len() != inputs.len() {
            return Err(invalid("scan masked target/mask shape"));
        }
        let output = self.scan_forward(inputs, batch, time, ReadMode::Enabled, true)?;
        let (nats, supervised) =
            crate::dialogue::masked_nats(&output.probabilities, targets, masks)?;
        if supervised <= 0.0 {
            return Err(invalid(
                "scan masked batch has no supervised response targets",
            ));
        }
        let loss = nats.affine(1.0 / supervised, 0.0)?;
        let mean_nll = loss.to_scalar::<f32>()?;
        if !mean_nll.is_finite() {
            return Err(invalid("nonfinite scan masked loss"));
        }
        let gradients = loss.backward()?;
        Ok(MaskedBatchGradients {
            mean_nll,
            supervised,
            gradients,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan_model(width: usize, context: usize, chunk: usize, window: usize) -> Result<JointModel> {
        JointModel::new_dialogue_scan(
            JointConfig {
                width,
                context,
                transport: Transport::Quaternion,
                seed: 7,
                ..JointConfig::default()
            },
            ScanConfig {
                chunk,
                window,
                mode: ScanMode::GatedTransport,
            },
            &Device::Cpu,
        )
    }

    fn ids_of(count: usize) -> Vec<u32> {
        (0..count)
            .map(|index| (((index * 17 + index / 7) % 4095) + 1) as u32)
            .collect()
    }

    fn targets_of(ids: &[u32]) -> Vec<u32> {
        ids.iter().map(|&id| (id * 3 + 1) % 4096).collect()
    }

    fn max_delta(a: &Tensor, b: &Tensor) -> Result<f32> {
        Ok(a.sub(b)?.abs()?.flatten_all()?.max(0)?.to_scalar::<f32>()?)
    }

    #[test]
    fn scan_chunked_matches_sequential_forward_and_loss() -> Result<()> {
        let batch = 2;
        let time = 64;
        let ids = ids_of(batch * time);
        let targets = targets_of(&ids);
        let mut worst_loss = 0f32;
        let mut worst_probability = 0f32;
        for chunk in [16usize, 32, 64] {
            let model = scan_model(128, time, chunk, 32)?;
            let chunked = model.scan_forward(&ids, batch, time, ReadMode::Enabled, false)?;
            let sequential =
                model.scan_sequential_forward(&ids, batch, time, ReadMode::Enabled, false)?;
            let loss_chunked = chunked.loss(&targets)?.to_scalar::<f32>()?;
            let loss_sequential = sequential.loss(&targets)?.to_scalar::<f32>()?;
            let loss_delta = (loss_chunked - loss_sequential).abs();
            let probability_delta = max_delta(&chunked.probabilities, &sequential.probabilities)?;
            println!(
                "chunk {chunk}: loss delta {loss_delta:.3e} ({loss_chunked} vs {loss_sequential}), \
                 probability delta {probability_delta:.3e}"
            );
            worst_loss = worst_loss.max(loss_delta);
            worst_probability = worst_probability.max(probability_delta);
            assert!(loss_delta <= 1e-6, "chunk {chunk} loss delta {loss_delta}");
        }
        println!(
            "worst loss delta {worst_loss:.3e}, worst probability delta {worst_probability:.3e}"
        );
        assert!(worst_loss <= 1e-6);
        Ok(())
    }

    #[test]
    fn scan_chunked_matches_sequential_gradients() -> Result<()> {
        let batch = 2;
        let time = 64;
        let ids = ids_of(batch * time);
        let targets = targets_of(&ids);
        let model = scan_model(128, time, 32, 32)?;
        let chunked = model.scan_forward(&ids, batch, time, ReadMode::Enabled, true)?;
        let chunked = chunked.loss(&targets)?.backward()?;
        let sequential =
            model.scan_sequential_forward(&ids, batch, time, ReadMode::Enabled, true)?;
        let sequential = sequential.loss(&targets)?.backward()?;
        let mut worst_relative = 0f32;
        let mut worst_scaled = 0f32;
        let mut worst_absolute = 0f32;
        let mut worst_name = String::new();
        for (name, variable) in model.variables() {
            let a = chunked
                .get(variable.as_tensor())
                .ok_or_else(|| invalid(format!("chunked missing gradient {name}")))?;
            let b = sequential
                .get(variable.as_tensor())
                .ok_or_else(|| invalid(format!("sequential missing gradient {name}")))?;
            let difference = a.sub(b)?.abs()?;
            let relative = difference
                .broadcast_div(&b.abs()?.affine(1.0, 1e-12)?)?
                .flatten_all()?
                .max(0)?
                .to_scalar::<f32>()?;
            let scale = b.abs()?.flatten_all()?.max(0)?.to_scalar::<f32>()?;
            let absolute = difference.flatten_all()?.max(0)?.to_scalar::<f32>()?;
            let scaled = absolute / (scale + 1e-12);
            if relative > worst_relative {
                worst_relative = relative;
                worst_name = name.clone();
            }
            worst_scaled = worst_scaled.max(scaled);
            worst_absolute = worst_absolute.max(absolute);
        }
        let residual_f64 = f64_state_order_residual(&model, &ids, batch, time)?;
        println!(
            "worst gradient per-element relative {worst_relative:.3e} ({worst_name}), \
             tensor-scale relative {worst_scaled:.3e}, absolute {worst_absolute:.3e}, \
             f64 state-order residual {residual_f64:.3e}"
        );
        if worst_relative <= 1e-5 {
            return Ok(());
        }
        assert!(
            worst_absolute <= 1e-6,
            "gradient absolute residual {worst_absolute} is structural, not reassociation"
        );
        assert!(
            residual_f64 <= 1e-9,
            "f64 state-order residual {residual_f64} persists: structural bug"
        );
        eprintln!(
            "gradient per-element relative {worst_relative} exceeds 1e-5 only on near-zero \
             elements; f64 recomputation residual {residual_f64:.3e} vanishes, so the F32 \
             residual is reassociation and is accepted and recorded"
        );
        Ok(())
    }

    /// Recompute both evaluation orders of the associative state in f64 and
    /// return their maximum absolute difference. A residual that persists here
    /// would be a structural bug; one that vanishes is F32 reassociation.
    fn f64_state_order_residual(
        model: &JointModel,
        ids: &[u32],
        batch: usize,
        time: usize,
    ) -> Result<f64> {
        let affine = model.scan_token_affine(ids, batch, time, false)?;
        let inputs = model.scan_inputs(&affine, false)?;
        let inputs = ScanInputs {
            z: inputs.z.to_dtype(DType::F64)?,
            u: inputs.u.to_dtype(DType::F64)?,
            q: inputs.q.to_dtype(DType::F64)?,
        };
        let sequential = model.resolve_states_sequential(&inputs, time)?;
        let chunked = model.resolve_states_chunked(&inputs, time)?;
        Ok(chunked
            .sub(&sequential)?
            .abs()?
            .flatten_all()?
            .max(0)?
            .to_scalar::<f64>()?)
    }

    #[test]
    fn scan_chunk_summary_monoid_associativity() -> Result<()> {
        let make = |base: f32| -> Result<ScanSummary> {
            let phi: Vec<f32> = (0..32)
                .map(|index| ((index as f32) * 0.017 + base).sin())
                .collect();
            let psi: Vec<f32> = (0..8)
                .map(|index| ((index as f32) * 0.031 + base).cos())
                .collect();
            Ok(ScanSummary {
                phi: Tensor::from_vec(phi, (1, 2, 4, 4), &Device::Cpu)?,
                psi: Tensor::from_vec(psi, (1, 8), &Device::Cpu)?,
            })
        };
        let a = make(0.1)?;
        let b = make(1.3)?;
        let c = make(2.7)?;
        let left = compose(&compose(&a, &b)?, &c)?;
        let right = compose(&a, &compose(&b, &c)?)?;
        let phi_delta = left
            .phi
            .sub(&right.phi)?
            .abs()?
            .flatten_all()?
            .max(0)?
            .to_scalar::<f32>()?;
        let phi_scale = right.phi.abs()?.flatten_all()?.max(0)?.to_scalar::<f32>()?;
        let psi_delta = left
            .psi
            .sub(&right.psi)?
            .abs()?
            .flatten_all()?
            .max(0)?
            .to_scalar::<f32>()?;
        let psi_scale = right.psi.abs()?.flatten_all()?.max(0)?.to_scalar::<f32>()?;
        println!(
            "monoid phi delta {phi_delta:.3e} scale {phi_scale:.3e}, psi delta {psi_delta:.3e} scale {psi_scale:.3e}"
        );
        assert!(phi_delta <= 1e-6 * (phi_scale + 1e-12), "phi {phi_delta}");
        assert!(psi_delta <= 1e-6 * (psi_scale + 1e-12), "psi {psi_delta}");
        Ok(())
    }

    #[test]
    fn scan_chunk_size_invariance() -> Result<()> {
        let batch = 2;
        let time = 128;
        let ids = ids_of(batch * time);
        let mut reference: Option<(Tensor, Tensor)> = None;
        for chunk in [16usize, 32, 64, 128] {
            let model = scan_model(128, time, chunk, 64)?;
            let output = model.scan_forward(&ids, batch, time, ReadMode::Enabled, false)?;
            match &reference {
                None => reference = Some((output.states.clone(), output.probabilities.clone())),
                Some((states, probabilities)) => {
                    let state_delta = max_delta(&output.states, states)?;
                    let probability_delta = max_delta(&output.probabilities, probabilities)?;
                    println!("chunk {chunk}: state delta {state_delta:.3e}, probability delta {probability_delta:.3e}");
                    assert!(
                        state_delta <= 1e-6,
                        "chunk {chunk} state delta {state_delta}"
                    );
                    assert!(
                        probability_delta <= 1e-6,
                        "chunk {chunk} probability delta {probability_delta}"
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn scan_session_matches_full_forward() -> Result<()> {
        let batch = 2;
        let time = 64;
        let chunk = 16;
        let ids = ids_of(batch * time);
        let model = scan_model(128, time, chunk, 32)?;
        let affine = model.scan_token_affine(&ids, batch, time, false)?;
        let inputs = model.scan_inputs(&affine, false)?;
        let summaries = model.chunk_summaries_with(&inputs, time)?;
        let mut accumulator = identity_summary(batch, 128 / 4, DType::F32, &Device::Cpu)?;
        let mut session_states = Vec::new();
        for summary in &summaries {
            let entry = model.resolve_chunk(
                &accumulator,
                &Tensor::zeros((batch, 128), DType::F32, &Device::Cpu)?,
            )?;
            let start = session_states.len() * chunk;
            session_states.push(model.resolve_chunk_states(&inputs, start, chunk, &entry)?);
            accumulator = compose(&accumulator, summary)?;
        }
        let session = Tensor::cat(&session_states.iter().collect::<Vec<_>>(), 1)?;
        let session_output =
            model.finish_states(&ids, &session, batch, time, ReadMode::Enabled, false)?;
        let full = model.scan_sequential_forward(&ids, batch, time, ReadMode::Enabled, false)?;
        let state_delta = max_delta(&session, &model.resolve_states_sequential(&inputs, time)?)?;
        let probability_delta = max_delta(&session_output.probabilities, &full.probabilities)?;
        println!("session vs sequential state delta {state_delta:.3e}, probability delta {probability_delta:.3e}");
        assert!(state_delta <= 1e-6, "session state delta {state_delta}");
        assert!(
            probability_delta <= 1e-6,
            "session probability delta {probability_delta}"
        );
        Ok(())
    }

    #[test]
    fn scan_causality_no_future_reads() -> Result<()> {
        let batch = 1;
        let time = 32;
        let model = scan_model(128, time, 16, 32)?;
        let mut ids = ids_of(batch * time);
        let base = model.scan_forward(&ids, batch, time, ReadMode::Enabled, false)?;
        let changed = 12usize;
        ids[changed] = (ids[changed] + 777) % 4096;
        let perturbed = model.scan_forward(&ids, batch, time, ReadMode::Enabled, false)?;
        let prefix_base = base.probabilities.narrow(1, 0, changed)?;
        let prefix_perturbed = perturbed.probabilities.narrow(1, 0, changed)?;
        let prefix_delta = max_delta(&prefix_base, &prefix_perturbed)?;
        let future_delta = max_delta(
            &base.probabilities.narrow(1, changed, time - changed)?,
            &perturbed.probabilities.narrow(1, changed, time - changed)?,
        )?;
        println!("causal prefix delta {prefix_delta:.3e}, live future delta {future_delta:.3e}");
        assert!(
            prefix_delta <= 1e-7,
            "future leak into the causal prefix: {prefix_delta}"
        );
        assert!(
            future_delta > 1e-5,
            "perturbation did not reach any later position"
        );
        Ok(())
    }

    #[test]
    fn scan_finite_difference_check() -> Result<()> {
        let batch = 1;
        let time = 32;
        let model = scan_model(64, time, 16, 32)?;
        let ids = ids_of(batch * time);
        let targets = targets_of(&ids);
        let output = model.scan_forward(&ids, batch, time, ReadMode::Enabled, true)?;
        let gradients = output.loss(&targets)?.backward()?;
        let mut candidates: Vec<(String, usize, f32)> = Vec::new();
        for (name, variable) in model.variables() {
            let gradient = gradients
                .get(variable.as_tensor())
                .ok_or_else(|| invalid(format!("missing finite-difference gradient {name}")))?;
            let values = gradient.flatten_all()?.to_vec1::<f32>()?;
            for (index, value) in values.iter().enumerate() {
                candidates.push((name.clone(), index, *value));
            }
        }
        candidates.sort_by(|a, b| {
            b.2.abs()
                .partial_cmp(&a.2.abs())
                .expect("finite gradient order")
        });
        let epsilon = 1e-3f32;
        let mut worst = 0f64;
        for (name, index, analytic) in candidates.into_iter().take(3) {
            let variable = &model.variables()[&name];
            let dims = variable.dims().to_vec();
            let original = variable.detach().flatten_all()?.to_vec1::<f32>()?;
            let mut plus = original.clone();
            plus[index] += epsilon;
            variable.set(&Tensor::from_vec(plus, dims.as_slice(), &Device::Cpu)?)?;
            let high = model
                .scan_forward(&ids, batch, time, ReadMode::Enabled, false)?
                .loss(&targets)?
                .to_scalar::<f32>()?;
            let mut minus = original.clone();
            minus[index] -= epsilon;
            variable.set(&Tensor::from_vec(minus, dims.as_slice(), &Device::Cpu)?)?;
            let low = model
                .scan_forward(&ids, batch, time, ReadMode::Enabled, false)?
                .loss(&targets)?
                .to_scalar::<f32>()?;
            variable.set(&Tensor::from_vec(original, dims.as_slice(), &Device::Cpu)?)?;
            let numeric = (f64::from(high) - f64::from(low)) / (2.0 * f64::from(epsilon));
            let relative = (numeric - f64::from(analytic)).abs()
                / numeric.abs().max(f64::from(analytic).abs()).max(1e-12);
            println!(
                "{name}[{index}]: analytic {analytic:.6e}, numeric {numeric:.6e}, relative {relative:.3e}"
            );
            worst = worst.max(relative);
        }
        assert!(worst <= 3e-3, "finite-difference relative {worst}");
        Ok(())
    }

    #[test]
    fn scan_width_1024_smoke() -> Result<()> {
        let model = scan_model(1024, 256, 16, 32)?;
        assert_eq!(model.parameter_count(), 10_632_578);
        let ids = ids_of(16);
        for _ in 0..2 {
            let output = model.scan_forward(&ids, 1, 16, ReadMode::Enabled, false)?;
            assert_eq!(output.probabilities.dims(), &[1, 16, 4096]);
        }
        println!(
            "width-1024 scan parameter count {}",
            model.parameter_count()
        );
        Ok(())
    }

    #[test]
    #[ignore = "release-only throughput probe: cargo test --release -- --ignored scan_throughput"]
    fn scan_throughput_probe() -> Result<()> {
        if cfg!(debug_assertions) {
            eprintln!("scan_throughput_probe requires --release; skipping debug build");
            return Ok(());
        }
        let context = 256usize;
        let chunk = 64usize;
        let steps = 3usize;
        for batch in [16usize, 24] {
            let scan = scan_model(1024, context, chunk, 64)?;
            let sequential = JointModel::new_dialogue(
                JointConfig {
                    width: 1024,
                    context,
                    transport: Transport::Quaternion,
                    seed: 7,
                    ..JointConfig::default()
                },
                &Device::Cpu,
            )?;
            let ids = ids_of(batch * context);
            let targets = targets_of(&ids);
            let measure = |model: &JointModel, use_scan: bool| -> Result<f64> {
                let mut elapsed = 0f64;
                for step in 0..steps {
                    let started = std::time::Instant::now();
                    let output = if use_scan {
                        model.scan_forward(&ids, batch, context, ReadMode::Enabled, true)?
                    } else {
                        model.forward(&ids, batch, context, ReadMode::Enabled, true)?
                    };
                    output.loss(&targets)?.backward()?;
                    if step > 0 {
                        elapsed += started.elapsed().as_secs_f64();
                    }
                }
                Ok((steps - 1) as f64 * (batch * context) as f64 / elapsed)
            };
            let scan_rate = measure(&scan, true)?;
            let sequential_rate = measure(&sequential, false)?;
            println!(
                "batch {batch}: scan {scan_rate:.1} targets/s, sequential {sequential_rate:.1} targets/s, \
                 ratio {:.2}x",
                scan_rate / sequential_rate
            );
        }
        Ok(())
    }
}
