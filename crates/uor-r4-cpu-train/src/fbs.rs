//! Exact CPU execution of the no-Hamming-read SpiralCore/FBS forward.
//!
//! The tables are copied from the parsed serving artifact through `FbsImage`.
//! Arithmetic follows the integer session's order, rounding and saturation;
//! this module does not call the serving forward and does not write its files.

use uor_r4_integer::h4_classifier::H4_ROOT_COEFFICIENTS;
use uor_r4_integer::stack::{
    stack_snap_select, FbsImage, FbsLayerImage, FbsMapImage, Fixed, GROUP, PHI_Q32,
};

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
    state: Vec<Vec<i64>>,
    history: Vec<Vec<i32>>,
}

impl CpuFbsForward {
    pub fn from_image(image: FbsImage<'_>, rms_eps: Fixed) -> Result<Self> {
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
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn vocabulary(&self) -> usize {
        self.vocab
    }

    pub fn step(&mut self, token: u32, head: CpuFbsHead) -> Result<CpuFbsStep> {
        if token as usize >= self.vocab {
            return Err(invalid(format!(
                "token {token} is outside vocabulary {}",
                self.vocab
            )));
        }
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
        }

        rms_norm_fine(&x, self.rms_eps, &mut scratch[..d]);
        apply_gain(&mut scratch[..d], &self.final_gain);
        let query_q12: Vec<i32> = scratch[..d]
            .iter()
            .map(|&value| to_exp_i32(value, NORM_FINE_EXP, Q12_EXP).clamp(-32768, 32767))
            .collect();
        let (token, kept_cells) = match head {
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
        Ok(CpuFbsStep {
            token,
            kept_cells,
            query_q12,
        })
    }
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
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};
    use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, IntegerStackModel};

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
        let cpu =
            CpuFbsForward::from_image(model.fbs_image().unwrap(), model.rms_epsilon()).unwrap();
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
    fn a_route_sign_fault_changes_the_query() {
        let (_, mut forward) = fixture(19, 1, true);
        let mut corrupted = forward.clone();
        corrupted.layers[0].rec_in.codes[0][0] ^= 8;
        let expected = forward.step(5, CpuFbsHead::Nearest).unwrap();
        let altered = corrupted.step(5, CpuFbsHead::Nearest).unwrap();
        assert_eq!(expected.query_q12.len(), altered.query_q12.len());
        assert_ne!(expected.query_q12, altered.query_q12);
    }
}
