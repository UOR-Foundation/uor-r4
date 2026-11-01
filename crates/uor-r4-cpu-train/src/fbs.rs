//! Exact CPU execution of the no-Hamming-read SpiralCore/FBS forward.
//!
//! The tables are copied from the parsed serving artifact through `FbsImage`.
//! Arithmetic follows the integer session's order, rounding and saturation;
//! this module does not call the serving forward and does not write its files.

use uor_r4_integer::h4_classifier::H4_ROOT_COEFFICIENTS;
use uor_r4_integer::stack::{
    read_hamming_uorlut01, read_memory_uorlut01, stack_snap_select, FbsImage, FbsLayerImage,
    FbsMapImage, Fixed, IntegerStackModel, GROUP, PHI_Q32,
};

use crate::hamming::CpuHamming;
use crate::memory::{CpuTurnMemory, MemoryAnchors, MemoryMove};

const RESIDUAL_EXP: i32 = FbsImage::RESIDUAL_EXP;
const PRODUCT_EXP: i32 = FbsImage::PRODUCT_EXP;
const STATE_EXP: i32 = FbsImage::STATE_EXP;
const NORM_FINE_EXP: i32 = FbsImage::NORM_FINE_EXP;
const Q12_EXP: i32 = FbsImage::Q12_EXP;
const GRID_EXP_BIAS: i32 = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FbsForwardError {
    Invalid(String),
}

impl std::fmt::Display for FbsForwardError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for FbsForwardError {}

type Result<T> = std::result::Result<T, FbsForwardError>;

fn invalid(message: impl Into<String>) -> FbsForwardError {
    FbsForwardError::Invalid(message.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuFbsHead {
    Nearest,
    Writer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuFbsStep {
    pub token: u32,
    pub kept_cells: Vec<u32>,
    pub query_q12: Vec<i32>,
}

#[derive(Clone)]
struct Map {
    atoms: Vec<u16>,
    codes: Vec<[u8; 8]>,
}

impl Map {
    fn from_image(image: &FbsMapImage<'_>) -> Self {
        Self {
            atoms: image.atoms.to_vec(),
            codes: image.codes.to_vec(),
        }
    }
}

#[derive(Clone)]
struct Layer {
    rec_gain: Vec<i16>,
    mlp_gain: Vec<i16>,
    rec_in: Map,
    rec_gate: Map,
    rec_out: Map,
    mlp_gate: Map,
    mlp_up: Map,
    mlp_down: Map,
    taps: Vec<i16>,
    conv_bias: Vec<i32>,
    gate_bias: Vec<i32>,
    decay_offset: Vec<i32>,
}

impl Layer {
    fn from_image(image: &FbsLayerImage<'_>) -> Self {
        Self {
            rec_gain: image.rec_gain.to_vec(),
            mlp_gain: image.mlp_gain.to_vec(),
            rec_in: Map::from_image(&image.rec_in),
            rec_gate: Map::from_image(&image.rec_gate),
            rec_out: Map::from_image(&image.rec_out),
            mlp_gate: Map::from_image(&image.mlp_gate),
            mlp_up: Map::from_image(&image.mlp_up),
            mlp_down: Map::from_image(&image.mlp_down),
            taps: image.taps.to_vec(),
            conv_bias: image.conv_bias.to_vec(),
            gate_bias: image.gate_bias.to_vec(),
            decay_offset: image.decay_offset.to_vec(),
        }
    }
}

#[derive(Clone)]
pub struct CpuFbsForward {
    vocab: usize,
    width: usize,
    mlp: usize,
    rms_eps: Fixed,
    layers: Vec<Layer>,
    final_gain: Vec<i16>,
    writer_table: Vec<i16>,
    roots_q12: [[i32; 8]; 16],
    kept_cells: usize,
    context: usize,
    position: usize,
    state: Vec<Vec<i64>>,
    history: Vec<Vec<i32>>,
    tokens: Vec<u32>,
    hamming: Option<CpuHamming>,
    hamming_payloads: Vec<Vec<i64>>,
    memory: Option<CpuTurnMemory>,
    memory_anchors: MemoryAnchors,
    reply_start: Option<usize>,
}

impl CpuFbsForward {
    pub fn from_model(model: &IntegerStackModel) -> Result<Self> {
        validate_shape(model)?;
        if model.turn_memory().is_some() {
            return Err(invalid(
                "CPU FBS forward does not yet support exact turn memory",
            ));
        }
        let image = model.fbs_image().ok_or_else(|| {
            invalid("CPU FBS forward does not yet support Hamming reads or turn memory")
        })?;
        Self::from_image(image, model.rms_epsilon(), model.shape().context)
    }

    /// Parse artifact bytes and attach the Hamming reads as well as the FBS
    /// trunk. A model without reads behaves exactly as [`Self::from_model`].
    pub fn from_artifact(bytes: &[u8]) -> Result<Self> {
        let parsed = IntegerStackModel::parse(bytes)
            .map_err(|error| invalid(format!("CPU FBS artifact does not parse: {error}")))?;
        validate_shape(&parsed)?;
        let hamming = read_hamming_uorlut01(bytes)
            .map_err(|error| invalid(format!("CPU Hamming tables do not parse: {error}")))?;
        let memory = read_memory_uorlut01(bytes)
            .map_err(|error| invalid(format!("CPU turn-memory tables do not parse: {error}")))?;
        let context = parsed.shape().context;
        let model = parsed.without_hamming_read();
        let image = model
            .fbs_trunk_image()
            .ok_or_else(|| invalid("CPU FBS forward could not recover the FBS trunk"))?;
        let mut forward = Self::from_image(image, model.rms_epsilon(), context)?;
        if let Some(hamming) = hamming {
            let reads = CpuHamming::new(
                &hamming,
                &forward.writer_table,
                forward.vocab,
                forward.width,
                forward.layers.len(),
                context,
            )
            .map_err(invalid)?;
            forward.hamming_payloads = reads.payload_caches(context);
            forward.hamming = Some(reads);
        }
        if let Some(memory) = memory {
            forward.memory = Some(CpuTurnMemory::new(memory, forward.width).map_err(invalid)?);
        }
        Ok(forward)
    }

    fn from_image(image: FbsImage<'_>, rms_eps: Fixed, context: usize) -> Result<Self> {
        let width = image.final_gain.len();
        if width == 0 || !width.is_multiple_of(GROUP) || !width.is_multiple_of(4) {
            return Err(invalid("FBS width must be a positive multiple of 32"));
        }
        if !image.writer_table.len().is_multiple_of(width) {
            return Err(invalid("writer table length is not a multiple of width"));
        }
        let vocab = image.writer_table.len() / width;
        if vocab == 0 {
            return Err(invalid("writer table is empty"));
        }
        let mlp = image
            .layers
            .first()
            .map(|layer| layer.mlp_gate.atoms.len() * 8)
            .ok_or_else(|| invalid("FBS model has no layers"))?;
        let layers = image
            .layers
            .iter()
            .map(Layer::from_image)
            .collect::<Vec<_>>();
        for layer in &layers {
            validate_layer(layer, width, mlp)?;
        }
        if image.kept_cells == 0 || image.kept_cells > 16 {
            return Err(invalid("kept cells must be in 1..=16"));
        }
        Ok(Self {
            vocab,
            width,
            mlp,
            rms_eps,
            state: layers.iter().map(|_| vec![0; width]).collect(),
            history: layers.iter().map(|_| vec![0; 3 * width]).collect(),
            layers,
            final_gain: image.final_gain.to_vec(),
            writer_table: image.writer_table.to_vec(),
            roots_q12: *image.roots_q12,
            kept_cells: image.kept_cells,
            context,
            position: 0,
            tokens: Vec::with_capacity(context),
            hamming: None,
            hamming_payloads: Vec::new(),
            memory: None,
            memory_anchors: MemoryAnchors::default(),
            reply_start: None,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn vocabulary(&self) -> usize {
        self.vocab
    }

    /// Mark that the next step's row is the prompt's last row and the opening move runs there.
    pub fn mark_reply_start(&mut self) {
        self.reply_start = Some(self.position + 1);
    }

    /// Stop keying and moving rows, and discard the anchors from any prior reply.
    pub fn clear_reply_start(&mut self) {
        self.reply_start = None;
        self.memory_anchors = MemoryAnchors::default();
    }

    pub fn step(&mut self, token: u32, head: CpuFbsHead) -> Result<CpuFbsStep> {
        if token as usize >= self.vocab {
            return Err(invalid(format!(
                "token {token} is outside vocabulary {}",
                self.vocab
            )));
        }
        if self.position >= self.context {
            return Err(invalid(format!(
                "context is full at {} positions",
                self.context
            )));
        }
        self.tokens.push(token);
        let d = self.width;
        let row = token as usize * d;
        let mut x: Vec<i32> = self.writer_table[row..row + d]
            .iter()
            .map(|&value| to_exp_i32(i64::from(value), Q12_EXP, RESIDUAL_EXP))
            .collect();
        let mut scratch = vec![0i64; d.max(self.mlp)];
        let mut gate = vec![0i32; self.mlp];
        let mut up = vec![0i32; self.mlp];
        let mut act = vec![0i16; d.max(self.mlp)];

        for index in 0..self.layers.len() {
            let layer = &self.layers[index];
            rms_norm_fine(&x, self.rms_eps, &mut scratch[..d]);
            apply_gain(&mut scratch[..d], &layer.rec_gain);
            let mut branches = vec![0i32; 2 * d];
            let gate_rows = d / 4 + d;
            let mut gate_out = vec![0i32; gate_rows];
            gather(&layer.rec_in, &scratch[..d], NORM_FINE_EXP, &mut branches);
            gather(&layer.rec_gate, &scratch[..d], NORM_FINE_EXP, &mut gate_out);
            let act_exp = recurrence(
                d,
                layer,
                &mut self.state[index],
                &mut self.history[index],
                &branches,
                &mut gate_out,
                &mut scratch,
                &mut act,
            );
            let mut projected = vec![0i32; d];
            gather(&layer.rec_out, &act[..d], act_exp, &mut projected);
            for (left, right) in x.iter_mut().zip(&projected) {
                *left = left.saturating_add(*right);
            }

            rms_norm_fine(&x, self.rms_eps, &mut scratch[..d]);
            apply_gain(&mut scratch[..d], &layer.mlp_gain);
            gather(&layer.mlp_gate, &scratch[..d], NORM_FINE_EXP, &mut gate);
            gather(&layer.mlp_up, &scratch[..d], NORM_FINE_EXP, &mut up);
            hamilton_feed_forward(&gate, &up, &mut scratch[..self.mlp]);
            let act_exp = quantize16(&scratch[..self.mlp], PRODUCT_EXP, &mut act[..self.mlp]);
            gather(&layer.mlp_down, &act[..self.mlp], act_exp, &mut projected);
            for (left, right) in x.iter_mut().zip(&projected) {
                *left = left.saturating_add(*right);
            }
            if let Some(hamming) = &self.hamming {
                hamming.stream(
                    index + 1,
                    &self.tokens,
                    &self.writer_table,
                    &mut self.hamming_payloads,
                    &mut x,
                );
            }
        }

        rms_norm_fine(&x, self.rms_eps, &mut scratch[..d]);
        apply_gain(&mut scratch[..d], &self.final_gain);
        if let Some(hamming) = &self.hamming {
            hamming.readout(
                &self.tokens,
                &self.writer_table,
                &mut self.hamming_payloads,
                &mut scratch[..d],
            );
        }
        let memory_move: Option<MemoryMove> = match (&self.memory, self.reply_start) {
            (Some(memory), Some(reply_start)) => memory.apply(
                &self.tokens,
                self.position,
                reply_start,
                &mut self.memory_anchors,
                &self.writer_table,
                &mut scratch[..d],
            ),
            _ => None,
        };
        let query_q12: Vec<i32> = scratch[..d]
            .iter()
            .map(|&value| to_exp_i32(value, NORM_FINE_EXP, Q12_EXP).clamp(-32768, 32767))
            .collect();
        let (mut token, kept_cells) = match head {
            CpuFbsHead::Nearest => (
                nearest(&query_q12, &self.writer_table, self.vocab),
                vec![0; self.kept_cells],
            ),
            CpuFbsHead::Writer => writer(
                &query_q12,
                &self.writer_table,
                self.vocab,
                &self.roots_q12,
                self.kept_cells,
            ),
        };
        if memory_move
            .as_ref()
            .is_some_and(|memory| memory.opening && memory.stored.is_none())
        {
            if let Some((name, _)) = self
                .memory
                .as_ref()
                .and_then(|memory| memory.point_name(&query_q12))
                .transpose()
                .map_err(invalid)?
            {
                token = name;
            }
        }
        self.position += 1;
        Ok(CpuFbsStep {
            token,
            kept_cells,
            query_q12,
        })
    }
}

fn validate_shape(model: &IntegerStackModel) -> Result<()> {
    let shape = model.shape();
    if !shape.projection_fbs()
        || !shape.rec_geometric()
        || shape.rec_retention()
        || !shape.gate_hamilton()
        || !shape.rotation
        || shape.raw_gates
        || shape.raw_transition
        || shape.pointer.is_some()
        || shape.pattern.bytes().any(|kind| kind != b'r')
    {
        return Err(invalid(
            "CPU FBS forward requires a snapped, rotated, decay-rate, recurrence-only FBS model",
        ));
    }
    Ok(())
}

fn validate_layer(layer: &Layer, width: usize, mlp: usize) -> Result<()> {
    let lanes = width / 4;
    let gate_rows = lanes + width;
    if layer.rec_gain.len() != width
        || layer.mlp_gain.len() != width
        || layer.taps.len() != 4 * width
        || layer.conv_bias.len() != width
        || layer.gate_bias.len() != gate_rows
        || layer.decay_offset.len() != lanes
        || layer.rec_in.atoms.len() != 2 * width / 8
        || layer.rec_gate.atoms.len() != gate_rows / 8
        || layer.rec_out.atoms.len() != width / 8
        || layer.mlp_gate.atoms.len() != mlp / 8
        || layer.mlp_up.atoms.len() != mlp / 8
        || layer.mlp_down.atoms.len() != width / 8
    {
        return Err(invalid("FBS layer table dimensions do not match the trunk"));
    }
    for map in [
        &layer.rec_in,
        &layer.rec_gate,
        &layer.rec_out,
        &layer.mlp_gate,
        &layer.mlp_up,
        &layer.mlp_down,
    ] {
        if map.atoms.len() != map.codes.len() {
            return Err(invalid("FBS route atom/code lengths differ"));
        }
    }
    validate_atoms(&layer.rec_in, width / 8)?;
    validate_atoms(&layer.rec_gate, width / 8)?;
    validate_atoms(&layer.rec_out, width / 8)?;
    validate_atoms(&layer.mlp_gate, width / 8)?;
    validate_atoms(&layer.mlp_up, width / 8)?;
    validate_atoms(&layer.mlp_down, mlp / 8)?;
    Ok(())
}

fn validate_atoms(map: &Map, input_blocks: usize) -> Result<()> {
    if map
        .atoms
        .iter()
        .any(|&atom| usize::from(atom) >= input_blocks)
    {
        return Err(invalid("FBS route atom lies outside its input"));
    }
    Ok(())
}

fn gather<T: Copy + Into<i64>>(map: &Map, input: &[T], from_exp: i32, out: &mut [i32]) {
    for ((block, &atom), codes) in out.chunks_exact_mut(8).zip(&map.atoms).zip(&map.codes) {
        let base = usize::from(atom) * 8;
        for (slot, &code) in block.iter_mut().zip(codes) {
            let value: i64 = input[base + usize::from(code & 7)].into();
            let value = if code & 8 == 0 {
                value
            } else {
                value.saturating_neg()
            };
            *slot = to_exp_i32(value, from_exp, RESIDUAL_EXP);
        }
    }
}

fn apply_gain(values: &mut [i64], codes: &[i16]) {
    for (value, &code) in values.iter_mut().zip(codes) {
        *value = grid_apply(*value, code);
    }
}

#[allow(clippy::too_many_arguments)]
fn recurrence(
    width: usize,
    layer: &Layer,
    state: &mut [i64],
    history: &mut [i32],
    branches: &[i32],
    gate_out: &mut [i32],
    scratch: &mut [i64],
    act: &mut [i16],
) -> i32 {
    for (value, &bias) in gate_out.iter_mut().zip(&layer.gate_bias) {
        *value = value.saturating_add(bias);
    }
    let (drive, gate) = branches.split_at(width);
    let (tap0, rest) = layer.taps.split_at(width);
    let (tap1, rest) = rest.split_at(width);
    let (tap2, tap3) = rest.split_at(width);
    let (back1, rest) = history.split_at(width);
    let (back2, back3) = rest.split_at(width);
    for i in 0..width {
        scratch[i] = i64::from(layer.conv_bias[i])
            .wrapping_add(grid_apply(i64::from(drive[i]), tap0[i]))
            .wrapping_add(grid_apply(i64::from(back1[i]), tap1[i]))
            .wrapping_add(grid_apply(i64::from(back2[i]), tap2[i]))
            .wrapping_add(grid_apply(i64::from(back3[i]), tap3[i]));
    }
    history.copy_within(..2 * width, width);
    history[..width].copy_from_slice(drive);

    let lanes = width / 4;
    let (decays, rotations) = gate_out.split_at(lanes);
    for lane in 0..lanes {
        let rate = decays[lane].saturating_add(layer.decay_offset[lane]);
        let idx = ((i64::from(rate.clamp(0, 1 << 16)) >> 12) as usize).min(15);
        let lambda = 1u64 << (30 - idx);
        let raw = [
            rotations[lane * 4],
            rotations[lane * 4 + 1],
            rotations[lane * 4 + 2],
            rotations[lane * 4 + 3],
        ];
        let transition = snap_rotation(raw, lambda);
        let held = [
            state[lane * 4],
            state[lane * 4 + 1],
            state[lane * 4 + 2],
            state[lane * 4 + 3],
        ];
        let moved = hamilton(transition, held);
        let keep = FbsImage::GEOMETRIC_KEEP_Q31[idx];
        for channel in 0..4 {
            let drive = i128::from(keep) * i128::from(scratch[lane * 4 + channel]);
            state[lane * 4 + channel] = shift_wide(moved[channel].wrapping_add(drive << 16), 31);
        }
    }

    for lane in 0..lanes {
        let q = snap_rotation(
            [
                gate[lane * 4],
                gate[lane * 4 + 1],
                gate[lane * 4 + 2],
                gate[lane * 4 + 3],
            ],
            1u64 << 31,
        );
        let rhs = [
            shift(state[lane * 4], RESIDUAL_EXP - STATE_EXP),
            shift(state[lane * 4 + 1], RESIDUAL_EXP - STATE_EXP),
            shift(state[lane * 4 + 2], RESIDUAL_EXP - STATE_EXP),
            shift(state[lane * 4 + 3], RESIDUAL_EXP - STATE_EXP),
        ];
        let product = hamilton(q, rhs);
        for channel in 0..4 {
            scratch[lane * 4 + channel] = shift_wide(product[channel], 15);
        }
    }
    quantize16(&scratch[..width], PRODUCT_EXP, &mut act[..width])
}

fn hamilton_feed_forward(gate: &[i32], up: &[i32], out: &mut [i64]) {
    for ((out_lane, gate_lane), up_lane) in out
        .chunks_exact_mut(4)
        .zip(gate.chunks_exact(4))
        .zip(up.chunks_exact(4))
    {
        let q = snap_rotation(
            [gate_lane[0], gate_lane[1], gate_lane[2], gate_lane[3]],
            1u64 << 31,
        );
        let product = hamilton(
            q,
            [
                i64::from(up_lane[0]),
                i64::from(up_lane[1]),
                i64::from(up_lane[2]),
                i64::from(up_lane[3]),
            ],
        );
        for (slot, value) in out_lane.iter_mut().zip(product) {
            *slot = shift_wide(value, 15);
        }
    }
}

fn nearest(query: &[i32], table: &[i16], vocab: usize) -> u32 {
    let width = query.len();
    let mut best = (u32::MAX, i64::MIN);
    for token in 0..vocab {
        let row = &table[token * width..(token + 1) * width];
        let score = query
            .iter()
            .zip(row)
            .fold(0i64, |sum, (&left, &right)| {
                sum.wrapping_add(i64::from(left.abs_diff(i32::from(right))))
            })
            .wrapping_neg();
        if score > best.1 {
            best = (token as u32, score);
        }
    }
    best.0
}

fn writer(
    query: &[i32],
    table: &[i16],
    vocab: usize,
    roots: &[[i32; 8]; 16],
    kept_cells: usize,
) -> (u32, Vec<u32>) {
    let width = query.len();
    let score = |left: &[i32], right: &[i32]| -> i64 {
        left.iter()
            .zip(right)
            .fold(0i64, |sum, (&left, &right)| {
                sum.wrapping_add(i64::from(left.abs_diff(right)))
            })
            .wrapping_neg()
    };
    let mut cells = [0i64; 256];
    for cell in 0..256 {
        cells[cell] = score(&query[..8], &roots[cell / 16])
            .wrapping_add(score(&query[width - 8..], &roots[cell % 16]));
    }
    let mut kept = vec![0u32; kept_cells];
    for slot in &mut kept {
        let mut best = 0;
        for cell in 1..256 {
            if cells[cell] > cells[best] {
                best = cell;
            }
        }
        *slot = best as u32;
        cells[best] = i64::MIN;
    }
    let mut best = (u32::MAX, i64::MIN);
    for &cell in &kept {
        let mut token = cell as usize;
        while token < vocab {
            let row = &table[token * width..(token + 1) * width];
            let candidate = query
                .iter()
                .zip(row)
                .fold(0i64, |sum, (&left, &right)| {
                    sum.wrapping_add(i64::from(left.abs_diff(i32::from(right))))
                })
                .wrapping_neg();
            if candidate > best.1 || (candidate == best.1 && (token as u32) < best.0) {
                best = (token as u32, candidate);
            }
            token += 256;
        }
    }
    (best.0, kept)
}

fn quantize16(values: &[i64], exp: i32, out: &mut [i16]) -> i32 {
    let max = values
        .iter()
        .fold(0u64, |max, value| max.max(value.unsigned_abs()));
    let shift_count = (bit_length(max) - 15).max(0);
    for (out, &value) in out.iter_mut().zip(values) {
        *out = shift(value, shift_count).clamp(-32767, 32767) as i16;
    }
    exp + shift_count
}

fn rms_norm_fine(x: &[i32], eps: Fixed, out: &mut [i64]) {
    let max = x
        .iter()
        .fold(0u32, |max, value| max.max(value.unsigned_abs()));
    let shrink = (bit_length(u64::from(max)) - 24).max(0);
    let mut sum = 0u64;
    for (slot, &value) in out.iter_mut().zip(x) {
        let y = shift(i64::from(value), shrink);
        *slot = y;
        sum = sum.wrapping_add(square(y));
    }
    let mean = sum / x.len().max(1) as u64;
    let square_exp = (RESIDUAL_EXP + shrink) << 1;
    let eps_int = shift(eps.mantissa, square_exp - eps.exp).max(0);
    let mean_square = mean.wrapping_add(eps_int as u64).max(1) as u128;
    let root = isqrt(mean_square << 68).max(1);
    let reciprocal = (1u128 << 96) / root;
    for value in out {
        let magnitude = u128::from(value.unsigned_abs()).wrapping_mul(reciprocal);
        let product = if *value < 0 {
            magnitude.wrapping_neg() as i128
        } else {
            magnitude as i128
        };
        *value = (product.wrapping_add(1i128 << 37) >> 38).clamp(i64::MIN as i128, i64::MAX as i128)
            as i64;
    }
}

fn snap_rotation(raw: [i32; 4], lambda: u64) -> [i64; 4] {
    let root = &H4_ROOT_COEFFICIENTS[stack_snap_select(raw)];
    let lambda_phi = lambda.wrapping_add(
        lambda
            .wrapping_mul((PHI_Q32 - (1 << 32)) as u64)
            .wrapping_add(1 << 31)
            >> 32,
    );
    std::array::from_fn(|channel| {
        let sum = small_mul(lambda as i64, root[channel][0])
            .wrapping_add(small_mul(lambda_phi as i64, root[channel][1]));
        shift(sum, 1)
    })
}

fn small_mul(value: i64, coefficient: i8) -> i64 {
    match coefficient {
        -2 => (value << 1).wrapping_neg(),
        -1 => value.wrapping_neg(),
        0 => 0,
        1 => value,
        2 => value << 1,
        _ => {
            let mut result = 0i64;
            let mut addend = value;
            let mut remaining = coefficient.unsigned_abs();
            while remaining != 0 {
                if remaining & 1 != 0 {
                    result = result.wrapping_add(addend);
                }
                remaining >>= 1;
                addend <<= 1;
            }
            if coefficient < 0 {
                result.wrapping_neg()
            } else {
                result
            }
        }
    }
}

fn hamilton(a: [i64; 4], b: [i64; 4]) -> [i128; 4] {
    let mult = |index: usize, other: usize| i128::from(a[index]).wrapping_mul(i128::from(b[other]));
    [
        mult(0, 0)
            .wrapping_sub(mult(1, 1))
            .wrapping_sub(mult(2, 2))
            .wrapping_sub(mult(3, 3)),
        mult(0, 1)
            .wrapping_add(mult(1, 0))
            .wrapping_add(mult(2, 3))
            .wrapping_sub(mult(3, 2)),
        mult(0, 2)
            .wrapping_sub(mult(1, 3))
            .wrapping_add(mult(2, 0))
            .wrapping_add(mult(3, 1)),
        mult(0, 3)
            .wrapping_add(mult(1, 2))
            .wrapping_sub(mult(2, 1))
            .wrapping_add(mult(3, 0)),
    ]
}

fn grid_apply(value: i64, code: i16) -> i64 {
    if code == 0 {
        return 0;
    }
    let negative = code < 0;
    let magnitude = code.unsigned_abs();
    let exponent = i32::from(magnitude >> 4) - GRID_EXP_BIAS;
    let scaled = shift(scale_16_plus(value, (magnitude & 15) as u8), 4 - exponent);
    if negative {
        scaled.saturating_neg()
    } else {
        scaled
    }
}

fn scale_16_plus(value: i64, remainder: u8) -> i64 {
    let select = |bit: u32| value & i64::from((remainder >> bit) & 1).wrapping_neg();
    let value = (value << 1).wrapping_add(select(3));
    let value = (value << 1).wrapping_add(select(2));
    let value = (value << 1).wrapping_add(select(1));
    (value << 1).wrapping_add(select(0))
}

fn shift(value: i64, amount: i32) -> i64 {
    if amount > 0 {
        if amount >= 63 {
            return if value < 0 { -1 } else { 0 };
        }
        value.saturating_add(1i64 << (amount - 1)) >> amount
    } else if amount < 0 {
        let amount = amount.unsigned_abs().min(63);
        let limit = i64::MAX >> amount;
        if value > limit {
            i64::MAX
        } else if value < -limit {
            i64::MIN
        } else {
            value << amount
        }
    } else {
        value
    }
}

fn shift_wide(value: i128, amount: u32) -> i64 {
    let rounded = if amount == 0 {
        value
    } else {
        value.wrapping_add(1i128 << (amount - 1)) >> amount
    };
    rounded.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

fn to_exp_i32(value: i64, from: i32, to: i32) -> i32 {
    shift(value, to - from).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn square(value: i64) -> u64 {
    let magnitude = value.unsigned_abs();
    magnitude.wrapping_mul(magnitude)
}

fn isqrt(value: u128) -> u128 {
    if value == 0 {
        return 0;
    }
    let mut remainder = value;
    let mut root = 0u128;
    let mut bit = 1u128 << ((127 - value.leading_zeros()) & !1);
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

fn bit_length(value: u64) -> i32 {
    64 - value.leading_zeros() as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, synthetic_hamming, FbsSynth};
    use uor_r4_integer::stack::{
        add_point_key_uorlut01, export_fbs_trunk_uorlut01, export_fbs_trunk_uorlut01_with,
        FbsExportSnaps, FbsMemoryEntry, FbsMemorySnapshot, FbsReplyTable, IntegerStackModel,
        PointKeyImage, MEMORY_PAD,
    };

    fn fixture(seed: u64, layers: usize, nearest: bool) -> (IntegerStackModel, CpuFbsForward) {
        let mut snapshot = synthetic_fbs_trunk_with(
            seed,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers,
                context: 16,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.readout_nearest = nearest;
        let bytes = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let model = IntegerStackModel::parse(&bytes).unwrap();
        let cpu = CpuFbsForward::from_model(&model).unwrap();
        (model, cpu)
    }

    fn assert_parity(seed: u64, layers: usize, head: CpuFbsHead) {
        let (model, mut cpu) = fixture(seed, layers, head == CpuFbsHead::Nearest);
        let mut session = model.session();
        for token in [0, 1, 17, 63, 255, 511, 33, 7] {
            let expected = session.step_native(token).unwrap();
            let actual = cpu.step(token, head).unwrap();
            assert_eq!(
                actual.query_q12, expected.query_q12,
                "seed {seed} token {token}"
            );
            assert_eq!(actual.token, expected.token, "seed {seed} token {token}");
            assert_eq!(
                actual.kept_cells, expected.kept_cells,
                "seed {seed} token {token}"
            );
        }
    }

    fn hamming_fixture(
        seed: u64,
        layers: usize,
        head: CpuFbsHead,
        copy_scale: f32,
        type_scale: f32,
        depths: &[usize],
    ) -> (IntegerStackModel, CpuFbsForward, IntegerStackModel) {
        let mut snapshot = synthetic_fbs_trunk_with(
            seed,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers,
                context: 24,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.readout_nearest = head == CpuFbsHead::Nearest;
        let free_bytes = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let free = IntegerStackModel::parse(&free_bytes).unwrap();

        let mut hamming = synthetic_hamming(snapshot.width, layers, 4, seed ^ 194);
        hamming.reads = hamming
            .depths
            .iter()
            .zip(hamming.reads)
            .filter(|(depth, _)| depths.contains(depth))
            .map(|(_, read)| read)
            .collect();
        hamming.depths = depths.to_vec();
        for read in &mut hamming.reads {
            let (copy, types) = read.weight.split_at_mut(snapshot.width);
            copy.iter_mut().for_each(|weight| *weight *= copy_scale);
            types.iter_mut().for_each(|weight| *weight *= type_scale);
        }
        snapshot.hamming = Some(hamming);
        let bytes = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let model = IntegerStackModel::parse(&bytes).unwrap();
        let cpu = CpuFbsForward::from_artifact(&bytes).unwrap();
        (model, cpu, free)
    }

    fn repeating_tokens() -> [u32; 16] {
        std::array::from_fn(|row| {
            if row < 12 {
                ((row * 7 + 3) % 11) as u32
            } else {
                ((row - 12) * 5 + 1) as u32
            }
        })
    }

    fn assert_hamming_parity(
        seed: u64,
        depths: &[usize],
        copy_scale: f32,
        type_scale: f32,
        head: CpuFbsHead,
    ) -> usize {
        let (model, mut cpu, free) = hamming_fixture(seed, 3, head, copy_scale, type_scale, depths);
        let mut session = model.session();
        let mut free_session = free.session();
        let mut changed = 0;
        for (row, token) in repeating_tokens().into_iter().enumerate() {
            let expected = session.step_native(token).unwrap();
            let free_step = free_session.step_native(token).unwrap();
            let actual = cpu.step(token, head).unwrap();
            assert_eq!(
                actual.query_q12, expected.query_q12,
                "row {row}: complete query"
            );
            assert_eq!(actual.token, expected.token, "row {row}: token");
            assert_eq!(
                actual.kept_cells, expected.kept_cells,
                "row {row}: kept cells"
            );
            changed += usize::from(actual.query_q12 != free_step.query_q12);
        }
        changed
    }

    fn memory_script() -> [u32; 6] {
        [0, 7, 11, 42, 77, 99]
    }

    fn memory_fixture(
        seed: u64,
        head: CpuFbsHead,
        hamming: bool,
        point: bool,
        opening_hit: bool,
    ) -> (IntegerStackModel, CpuFbsForward) {
        let mut snapshot = synthetic_fbs_trunk_with(
            seed,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 3,
                context: 24,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.readout_nearest = head == CpuFbsHead::Nearest;
        if hamming {
            let mut reads = synthetic_hamming(snapshot.width, 3, 4, seed ^ 351);
            reads
                .reads
                .iter_mut()
                .for_each(|read| read.weight.fill(1.0));
            snapshot.hamming = Some(reads);
        }
        let opening_key = if opening_hit {
            vec![7, 11]
        } else {
            vec![13, 17]
        };
        let weight = vec![1.0; snapshot.width];
        snapshot.memory = Some(FbsMemorySnapshot {
            k: 4,
            key_window: 192,
            opening: vec![FbsMemoryEntry {
                key: opening_key,
                value: 42,
            }],
            opening_weight: weight.clone(),
            reply: Some(FbsReplyTable {
                turn: 2,
                entries: vec![
                    FbsMemoryEntry {
                        key: vec![7, 11, MEMORY_PAD, 1, MEMORY_PAD, 42],
                        value: 77,
                    },
                    FbsMemoryEntry {
                        key: vec![7, 11, MEMORY_PAD, 2, MEMORY_PAD, 77],
                        value: 99,
                    },
                ],
                weight,
            }),
        });
        let mut bytes = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        if point {
            bytes = add_point_key_uorlut01(
                &bytes,
                &PointKeyImage {
                    width: snapshot.width,
                    points: vec![0; snapshot.width],
                    targets: vec![99],
                },
            )
            .unwrap();
        }
        let model = IntegerStackModel::parse(&bytes).unwrap();
        let cpu = CpuFbsForward::from_artifact(&bytes).unwrap();
        (model, cpu)
    }

    fn assert_memory_parity(
        seed: u64,
        head: CpuFbsHead,
        hamming: bool,
        point: bool,
        opening_hit: bool,
    ) -> Vec<u32> {
        let (model, mut cpu) = memory_fixture(seed, head, hamming, point, opening_hit);
        let mut session = model.session();
        let mut served = Vec::new();
        for (row, token) in memory_script().into_iter().enumerate() {
            if row + 1 == 3 {
                session.mark_reply_start();
                cpu.mark_reply_start();
            }
            let expected = session.step_native(token).unwrap();
            let actual = cpu.step(token, head).unwrap();
            assert_eq!(actual.query_q12, expected.query_q12, "row {row}: query");
            assert_eq!(actual.token, expected.token, "row {row}: token");
            assert_eq!(
                actual.kept_cells, expected.kept_cells,
                "row {row}: kept cells"
            );
            served.push(actual.token);
        }
        served
    }

    #[test]
    fn one_layer_nearest_matches_serving_forward() {
        assert_parity(7, 1, CpuFbsHead::Nearest);
    }

    #[test]
    fn three_layer_nearest_matches_serving_forward() {
        assert_parity(11, 3, CpuFbsHead::Nearest);
    }

    #[test]
    fn writer_head_matches_serving_forward() {
        assert_parity(13, 2, CpuFbsHead::Writer);
    }

    #[test]
    fn six_layers_match_serving_forward() {
        assert_parity(17, 6, CpuFbsHead::Nearest);
    }

    #[test]
    fn copy_heads_at_both_hooks_match_serving() {
        let changed = assert_hamming_parity(41, &[1, 3], 1.0, 0.0, CpuFbsHead::Writer);
        assert!(changed > 0, "the copy reads must move the complete query");
    }

    #[test]
    fn type_heads_at_both_hooks_match_serving() {
        let changed = assert_hamming_parity(43, &[1, 3], 0.0, 1.0, CpuFbsHead::Nearest);
        assert!(changed > 0, "the type reads must move the complete query");
    }

    #[test]
    fn combined_copy_and_type_reads_match_serving() {
        let changed = assert_hamming_parity(47, &[1, 3], 1.0, 1.0, CpuFbsHead::Writer);
        assert!(
            changed > 0,
            "the combined reads must move the complete query"
        );
    }

    #[test]
    fn erased_copy_codes_change_the_complete_query() {
        let (_, mut expected, _) = hamming_fixture(49, 3, CpuFbsHead::Nearest, 1.0, 1.0, &[1, 3]);
        let mut corrupted = expected.clone();
        corrupted.hamming.as_mut().unwrap().erase_copy_codes();
        for token in repeating_tokens().into_iter().take(3) {
            expected.step(token, CpuFbsHead::Nearest).unwrap();
            corrupted.step(token, CpuFbsHead::Nearest).unwrap();
        }
        let expected = expected.step(7, CpuFbsHead::Nearest).unwrap();
        let corrupted = corrupted.step(7, CpuFbsHead::Nearest).unwrap();
        assert_ne!(expected.query_q12, corrupted.query_q12);
    }

    #[test]
    fn stream_only_and_readout_only_reads_match_serving() {
        assert!(assert_hamming_parity(53, &[1], 1.0, 1.0, CpuFbsHead::Nearest) > 0);
        assert!(assert_hamming_parity(59, &[3], 1.0, 1.0, CpuFbsHead::Nearest) > 0);
    }

    #[test]
    fn zero_weight_hamming_matches_the_read_free_trunk() {
        let changed = assert_hamming_parity(61, &[1, 3], 0.0, 0.0, CpuFbsHead::Writer);
        assert_eq!(changed, 0, "zero read weights must add nothing");
    }

    #[test]
    fn opening_and_reply_memory_moves_match_serving() {
        assert_memory_parity(71, CpuFbsHead::Writer, false, false, true);
        assert_memory_parity(73, CpuFbsHead::Nearest, false, false, true);
    }

    #[test]
    fn point_fallback_names_the_missed_opening() {
        let served = assert_memory_parity(79, CpuFbsHead::Writer, false, true, false);
        assert_eq!(served[2], 99);
    }

    #[test]
    fn combined_hamming_and_memory_match_serving() {
        assert_memory_parity(83, CpuFbsHead::Writer, true, false, true);
        assert_memory_parity(89, CpuFbsHead::Nearest, true, true, false);
    }

    #[test]
    fn memory_without_a_reply_signal_or_after_clear_adds_nothing() {
        let (model, mut cpu) = memory_fixture(97, CpuFbsHead::Writer, false, false, true);
        let mut session = model.session();
        for token in memory_script() {
            let expected = session.step_native(token).unwrap();
            let actual = cpu.step(token, CpuFbsHead::Writer).unwrap();
            assert_eq!(actual.query_q12, expected.query_q12);
            assert_eq!(actual.token, expected.token);
            assert_eq!(actual.kept_cells, expected.kept_cells);
        }

        let (model, mut cpu) = memory_fixture(101, CpuFbsHead::Writer, false, false, true);
        let mut session = model.session();
        for (row, token) in memory_script().into_iter().enumerate() {
            if row + 1 == 3 {
                session.mark_reply_start();
                session.clear_reply_start();
                cpu.mark_reply_start();
                cpu.clear_reply_start();
            }
            let actual = cpu.step(token, CpuFbsHead::Writer).unwrap();
            let expected = session.step_native(token).unwrap();
            assert_eq!(actual.query_q12, expected.query_q12);
            assert_eq!(actual.token, expected.token);
            assert_eq!(actual.kept_cells, expected.kept_cells);
        }
    }

    #[test]
    fn the_trunk_view_does_not_weaken_fbs_image_refusal() {
        let (model, _) = memory_fixture(103, CpuFbsHead::Writer, true, false, true);
        assert!(model.fbs_image().is_none());
        assert!(model.fbs_trunk_image().is_some());
    }

    #[test]
    fn a_route_sign_fault_changes_the_query() {
        let (_, mut forward) = fixture(19, 1, true);
        let mut corrupted = forward.clone();
        corrupted.layers[0].rec_in.codes[0][0] ^= 8;
        let expected = forward.step(5, CpuFbsHead::Nearest).unwrap();
        let altered = corrupted.step(5, CpuFbsHead::Nearest).unwrap();
        assert_eq!(expected.query_q12.len(), altered.query_q12.len());
        assert_ne!(expected.query_q12, altered.query_q12);
    }

    #[test]
    fn raw_gate_and_transition_artifacts_are_refused() {
        let snapshot = synthetic_fbs_trunk_with(
            23,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 1,
                context: 16,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        let bytes = export_fbs_trunk_uorlut01_with(
            &snapshot,
            FbsExportSnaps {
                gates: false,
                transition: false,
            },
        )
        .unwrap();
        let model = IntegerStackModel::parse(&bytes).unwrap();
        assert!(CpuFbsForward::from_model(&model).is_err());
    }

    #[test]
    fn retention_rate_artifacts_require_their_other_rate_tables() {
        let mut snapshot = synthetic_fbs_trunk_with(
            31,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 1,
                context: 16,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.retain_rates = true;
        let bytes = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let model = IntegerStackModel::parse(&bytes).unwrap();
        assert!(CpuFbsForward::from_model(&model).is_err());
    }

    #[test]
    fn context_exhaustion_matches_serving() {
        let (model, mut cpu) = fixture(29, 1, true);
        let mut session = model.session();
        for step in 0..16 {
            let token = (step % 512) as u32;
            session.step_native(token).unwrap();
            cpu.step(token, CpuFbsHead::Nearest).unwrap();
        }
        assert!(session.step_native(0).is_err());
        assert!(cpu.step(0, CpuFbsHead::Nearest).is_err());
    }
}
