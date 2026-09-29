//! B3: E8 Lattice Weight Codecs for Track B Conversion (SmolLM2 MLP layers).
//!
//! Provides geometric weight coding at 2, 3, and 4 bits per weight using the canonical
//! E8 lattice codebook (241 codewords from `uor_r4_integer::codec::E8Codebook` as evaluated
//! in the float reference and matched bit-for-bit in Kimi's gating re-run),
//! evaluated against standard 4-bit round-to-nearest (RTN).
//!
//! Bits per weight are derived strictly from serialized bytes:
//! - RTN 4-bit: 16 code bytes + 1 scale byte per 32 weights = 17 bytes / 32 = 4.2500 bpw.
//! - E8 2-bit:  1 index byte + 1 scale byte per 8 weights = 2 bytes / 8 = 2.0000 bpw.
//! - E8 3-bit:  2 index bytes + 1 scale byte per 8 weights = 3 bytes / 8 = 3.0000 bpw.
//! - E8 4-bit:  2 index bytes + 2 scale bytes per 8 weights = 4 bytes / 8 = 4.0000 bpw.

use crate::{invalid, Result};
pub use uor_r4_integer::codec::{E8Codebook, E8_DIM};

pub const E8_CODEWORDS: usize = 241;
pub const RTN_GROUP_SIZE: usize = 32;

/// Find the nearest E8 codeword and optimal scale for an 8D block `b`.
///
/// Codewords 1..241 all have exact squared norm 8 in `E8Codebook`.
/// Returns `(best_index, continuous_scale)`.
pub fn quantize_e8_block(codebook: &E8Codebook, block: &[f32; E8_DIM]) -> (u8, f32) {
    let mut best_idx = 0u8;
    let mut best_dot = 0.0f32;

    for (i, root) in codebook.roots.iter().enumerate().skip(1) {
        let mut dot = 0.0f32;
        for j in 0..E8_DIM {
            dot += block[j] * (root[j] as f32);
        }
        if dot > best_dot {
            best_dot = dot;
            best_idx = i as u8;
        }
    }

    if best_idx == 0 || best_dot <= 0.0 {
        (0, 0.0)
    } else {
        let scale = best_dot / 8.0;
        (best_idx, scale)
    }
}

/// Dequantize an 8D block from E8 root index and scale.
pub fn dequantize_e8_block(codebook: &E8Codebook, idx: u8, scale: f32) -> Result<[f32; E8_DIM]> {
    let root = codebook.roots.get(idx as usize).ok_or_else(|| {
        invalid(format!(
            "E8 root index {} out of bounds (max {})",
            idx,
            codebook.roots.len().saturating_sub(1)
        ))
    })?;
    let mut out = [0.0f32; E8_DIM];
    for i in 0..E8_DIM {
        out[i] = (root[i] as f32) * scale;
    }
    Ok(out)
}

/// 8-bit scale quantization.
/// Encodes a positive scale in `[0, max_scale]` into 8 bits.
pub fn encode_scale_u8(scale: f32, max_scale: f32) -> u8 {
    if scale <= 0.0 || max_scale <= 0.0 {
        return 0;
    }
    let ratio = (scale / max_scale).clamp(0.0, 1.0);
    (ratio * 255.0).round() as u8
}

pub fn decode_scale_u8(code: u8, max_scale: f32) -> f32 {
    (code as f32 / 255.0) * max_scale
}

// ---------------------------------------------------------------------------
// RTN 4-bit Codec (Group size 32, 4.2500 bpw)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Rtn4BitMatrix {
    pub rows: usize,
    pub cols: usize,
    pub scales: Vec<u8>,
    pub max_scales: Vec<f32>,
    pub packed_codes: Vec<u8>,
}

impl Rtn4BitMatrix {
    pub fn encode(weights: &[f32], rows: usize, cols: usize) -> Result<Self> {
        let total = rows * cols;
        if weights.len() != total {
            return Err(invalid(format!(
                "weights length {} != {rows}x{cols}",
                weights.len()
            )));
        }
        if total % RTN_GROUP_SIZE != 0 {
            return Err(invalid(format!(
                "total weights {total} not divisible by 32"
            )));
        }

        let num_groups = total / RTN_GROUP_SIZE;
        let mut scales = Vec::with_capacity(num_groups);
        let mut max_scales = Vec::with_capacity(rows);
        let mut packed_codes = Vec::with_capacity(num_groups * 16);

        for row_idx in 0..rows {
            let row_weights = &weights[row_idx * cols..(row_idx + 1) * cols];
            let mut row_max = 0.0f32;
            for group in row_weights.chunks_exact(RTN_GROUP_SIZE) {
                for &w in group {
                    row_max = row_max.max(w.abs());
                }
            }
            if row_max == 0.0 {
                row_max = 1.0;
            }
            max_scales.push(row_max);

            for group in row_weights.chunks_exact(RTN_GROUP_SIZE) {
                let mut g_max = 0.0f32;
                for &w in group {
                    g_max = g_max.max(w.abs());
                }
                let scale_f32 = g_max / 7.0;
                let scale_u8 = encode_scale_u8(scale_f32, row_max / 7.0);
                scales.push(scale_u8);
                let eff_scale = decode_scale_u8(scale_u8, row_max / 7.0);

                for pair in group.chunks_exact(2) {
                    let q0 = if eff_scale > 0.0 {
                        (pair[0] / eff_scale).round().clamp(-7.0, 7.0) as i8
                    } else {
                        0
                    };
                    let q1 = if eff_scale > 0.0 {
                        (pair[1] / eff_scale).round().clamp(-7.0, 7.0) as i8
                    } else {
                        0
                    };
                    let nib0 = (q0 & 0x0F) as u8;
                    let nib1 = (q1 & 0x0F) as u8;
                    packed_codes.push(nib0 | (nib1 << 4));
                }
            }
        }

        Ok(Self {
            rows,
            cols,
            scales,
            max_scales,
            packed_codes,
        })
    }

    pub fn serialized_bytes(&self) -> usize {
        8 + (self.rows * 4) + self.scales.len() + self.packed_codes.len()
    }

    pub fn payload_bits_per_weight(&self) -> f64 {
        let total = self.rows * self.cols;
        let payload_bytes = self.scales.len() + self.packed_codes.len();
        (payload_bytes as f64 * 8.0) / (total as f64)
    }

    pub fn dequantize(&self) -> Result<Vec<f32>> {
        let total = self.rows * self.cols;
        let mut out = Vec::with_capacity(total);
        let groups_per_row = self.cols / RTN_GROUP_SIZE;

        for r in 0..self.rows {
            let row_max = self.max_scales[r];
            for g in 0..groups_per_row {
                let g_idx = r * groups_per_row + g;
                let scale_u8 = self.scales[g_idx];
                let eff_scale = decode_scale_u8(scale_u8, row_max / 7.0);

                let code_offset = g_idx * 16;
                for i in 0..16 {
                    let byte = self.packed_codes[code_offset + i];
                    let mut nib0 = (byte & 0x0F) as i8;
                    let mut nib1 = ((byte >> 4) & 0x0F) as i8;
                    if nib0 >= 8 {
                        nib0 -= 16;
                    }
                    if nib1 >= 8 {
                        nib1 -= 16;
                    }
                    out.push((nib0 as f32) * eff_scale);
                    out.push((nib1 as f32) * eff_scale);
                }
            }
        }
        Ok(out)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.serialized_bytes());
        bytes.extend_from_slice(&(self.rows as u32).to_le_bytes());
        bytes.extend_from_slice(&(self.cols as u32).to_le_bytes());
        for &m in &self.max_scales {
            bytes.extend_from_slice(&m.to_le_bytes());
        }
        bytes.extend_from_slice(&self.scales);
        bytes.extend_from_slice(&self.packed_codes);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 8 {
            return Err(invalid("buffer too short for Rtn4BitMatrix header"));
        }
        let rows = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
        let cols = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let num_groups = (rows * cols) / RTN_GROUP_SIZE;

        let max_scales_bytes = rows * 4;
        let expected_total = 8 + max_scales_bytes + num_groups + (num_groups * 16);
        if bytes.len() != expected_total {
            return Err(invalid(format!(
                "buffer length {} != expected {expected_total}",
                bytes.len()
            )));
        }

        let mut offset = 8;
        let mut max_scales = Vec::with_capacity(rows);
        for _ in 0..rows {
            max_scales.push(f32::from_le_bytes(
                bytes[offset..offset + 4].try_into().unwrap(),
            ));
            offset += 4;
        }

        let scales = bytes[offset..offset + num_groups].to_vec();
        offset += num_groups;
        let packed_codes = bytes[offset..offset + num_groups * 16].to_vec();

        Ok(Self {
            rows,
            cols,
            scales,
            max_scales,
            packed_codes,
        })
    }
}

// ---------------------------------------------------------------------------
// E8 2-bit Codec (1-stage E8, 2.0000 bpw)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct E8TwoBitMatrix {
    pub rows: usize,
    pub cols: usize,
    pub indices: Vec<u8>,
    pub scales: Vec<u8>,
    pub max_scale: f32,
}

impl E8TwoBitMatrix {
    pub fn encode(weights: &[f32], rows: usize, cols: usize) -> Result<Self> {
        let total = rows * cols;
        if weights.len() != total {
            return Err(invalid(format!(
                "weights length {} != {rows}x{cols}",
                weights.len()
            )));
        }
        if total % E8_DIM != 0 {
            return Err(invalid(format!("total weights {total} not divisible by 8")));
        }

        let codebook = E8Codebook::new();
        let num_blocks = total / E8_DIM;
        let mut indices = Vec::with_capacity(num_blocks);
        let mut raw_scales = Vec::with_capacity(num_blocks);
        let mut max_scale = 0.0f32;

        for block in weights.chunks_exact(E8_DIM) {
            let mut b = [0.0f32; E8_DIM];
            b.copy_from_slice(block);
            let (idx, scale) = quantize_e8_block(&codebook, &b);
            indices.push(idx);
            raw_scales.push(scale);
            max_scale = max_scale.max(scale);
        }

        if max_scale == 0.0 {
            max_scale = 1.0;
        }

        let mut scales = Vec::with_capacity(num_blocks);
        for &s in &raw_scales {
            scales.push(encode_scale_u8(s, max_scale));
        }

        Ok(Self {
            rows,
            cols,
            indices,
            scales,
            max_scale,
        })
    }

    pub fn serialized_bytes(&self) -> usize {
        12 + self.indices.len() + self.scales.len()
    }

    pub fn payload_bits_per_weight(&self) -> f64 {
        let total = self.rows * self.cols;
        let payload_bytes = self.indices.len() + self.scales.len();
        (payload_bytes as f64 * 8.0) / (total as f64)
    }

    pub fn dequantize(&self) -> Result<Vec<f32>> {
        let codebook = E8Codebook::new();
        let total = self.rows * self.cols;
        let mut out = Vec::with_capacity(total);

        for i in 0..self.indices.len() {
            let idx = self.indices[i];
            let s = decode_scale_u8(self.scales[i], self.max_scale);
            let deq = dequantize_e8_block(&codebook, idx, s)?;
            out.extend_from_slice(&deq);
        }
        Ok(out)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.serialized_bytes());
        bytes.extend_from_slice(&(self.rows as u32).to_le_bytes());
        bytes.extend_from_slice(&(self.cols as u32).to_le_bytes());
        bytes.extend_from_slice(&self.max_scale.to_le_bytes());
        bytes.extend_from_slice(&self.indices);
        bytes.extend_from_slice(&self.scales);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 12 {
            return Err(invalid("buffer too short for E8TwoBitMatrix header"));
        }
        let rows = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
        let cols = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let max_scale = f32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let num_blocks = (rows * cols) / E8_DIM;

        let expected_total = 12 + num_blocks * 2;
        if bytes.len() != expected_total {
            return Err(invalid(format!(
                "buffer length {} != expected {expected_total}",
                bytes.len()
            )));
        }

        let indices = bytes[12..12 + num_blocks].to_vec();
        let scales = bytes[12 + num_blocks..12 + num_blocks * 2].to_vec();

        Ok(Self {
            rows,
            cols,
            indices,
            scales,
            max_scale,
        })
    }
}

// ---------------------------------------------------------------------------
// E8 3-bit Codec (2-stage residual E8 with single block scale, 3.0000 bpw)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct E8ThreeBitMatrix {
    pub rows: usize,
    pub cols: usize,
    pub stage1_indices: Vec<u8>,
    pub stage2_indices: Vec<u8>,
    pub scales: Vec<u8>,
    pub max_scale: f32,
}

impl E8ThreeBitMatrix {
    pub fn encode(weights: &[f32], rows: usize, cols: usize) -> Result<Self> {
        let total = rows * cols;
        if weights.len() != total {
            return Err(invalid(format!(
                "weights length {} != {rows}x{cols}",
                weights.len()
            )));
        }
        if total % E8_DIM != 0 {
            return Err(invalid(format!("total weights {total} not divisible by 8")));
        }

        let codebook = E8Codebook::new();
        let num_blocks = total / E8_DIM;
        let mut stage1_indices = Vec::with_capacity(num_blocks);
        let mut stage2_indices = Vec::with_capacity(num_blocks);
        let mut raw_scales = Vec::with_capacity(num_blocks);
        let mut max_scale = 0.0f32;

        for block in weights.chunks_exact(E8_DIM) {
            let mut b = [0.0f32; E8_DIM];
            b.copy_from_slice(block);

            // Stage 1
            let (idx1, scale1) = quantize_e8_block(&codebook, &b);
            let cw1 = dequantize_e8_block(&codebook, idx1, 1.0)?;

            // Residual: r = b - scale1 * cw1
            let mut res = [0.0f32; E8_DIM];
            for j in 0..E8_DIM {
                res[j] = b[j] - scale1 * cw1[j];
            }

            // Stage 2
            let (idx2, _scale2) = quantize_e8_block(&codebook, &res);
            let cw2 = dequantize_e8_block(&codebook, idx2, 1.0)?;

            // Coupled scale: joint least squares fit for b approx s * (cw1 + 0.5 * cw2)
            let mut combined = [0.0f32; E8_DIM];
            for j in 0..E8_DIM {
                combined[j] = cw1[j] + 0.5 * cw2[j];
            }
            let mut dot = 0.0f32;
            let mut norm_comb_sq = 0.0f32;
            for j in 0..E8_DIM {
                dot += b[j] * combined[j];
                norm_comb_sq += combined[j] * combined[j];
            }
            let joint_scale = if norm_comb_sq > 0.0 && dot > 0.0 {
                dot / norm_comb_sq
            } else {
                scale1
            };

            stage1_indices.push(idx1);
            stage2_indices.push(idx2);
            raw_scales.push(joint_scale);
            max_scale = max_scale.max(joint_scale);
        }

        if max_scale == 0.0 {
            max_scale = 1.0;
        }

        let mut scales = Vec::with_capacity(num_blocks);
        for &s in &raw_scales {
            scales.push(encode_scale_u8(s, max_scale));
        }

        Ok(Self {
            rows,
            cols,
            stage1_indices,
            stage2_indices,
            scales,
            max_scale,
        })
    }

    pub fn serialized_bytes(&self) -> usize {
        12 + self.stage1_indices.len() + self.stage2_indices.len() + self.scales.len()
    }

    pub fn payload_bits_per_weight(&self) -> f64 {
        let total = self.rows * self.cols;
        let payload_bytes =
            self.stage1_indices.len() + self.stage2_indices.len() + self.scales.len();
        (payload_bytes as f64 * 8.0) / (total as f64)
    }

    pub fn dequantize(&self) -> Result<Vec<f32>> {
        let codebook = E8Codebook::new();
        let total = self.rows * self.cols;
        let mut out = Vec::with_capacity(total);

        for i in 0..self.stage1_indices.len() {
            let idx1 = self.stage1_indices[i];
            let idx2 = self.stage2_indices[i];
            let s = decode_scale_u8(self.scales[i], self.max_scale);
            let cw1 = dequantize_e8_block(&codebook, idx1, 1.0)?;
            let cw2 = dequantize_e8_block(&codebook, idx2, 1.0)?;
            for j in 0..E8_DIM {
                out.push(s * (cw1[j] + 0.5 * cw2[j]));
            }
        }
        Ok(out)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.serialized_bytes());
        bytes.extend_from_slice(&(self.rows as u32).to_le_bytes());
        bytes.extend_from_slice(&(self.cols as u32).to_le_bytes());
        bytes.extend_from_slice(&self.max_scale.to_le_bytes());
        bytes.extend_from_slice(&self.stage1_indices);
        bytes.extend_from_slice(&self.stage2_indices);
        bytes.extend_from_slice(&self.scales);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 12 {
            return Err(invalid("buffer too short for E8ThreeBitMatrix header"));
        }
        let rows = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
        let cols = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let max_scale = f32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let num_blocks = (rows * cols) / E8_DIM;

        let expected_total = 12 + num_blocks * 3;
        if bytes.len() != expected_total {
            return Err(invalid(format!(
                "buffer length {} != expected {expected_total}",
                bytes.len()
            )));
        }

        let stage1_indices = bytes[12..12 + num_blocks].to_vec();
        let stage2_indices = bytes[12 + num_blocks..12 + num_blocks * 2].to_vec();
        let scales = bytes[12 + num_blocks * 2..12 + num_blocks * 3].to_vec();

        Ok(Self {
            rows,
            cols,
            stage1_indices,
            stage2_indices,
            scales,
            max_scale,
        })
    }
}

// ---------------------------------------------------------------------------
// E8 4-bit Codec (2-stage residual E8 with two scales, 4.0000 bpw)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct E8FourBitMatrix {
    pub rows: usize,
    pub cols: usize,
    pub stage1_indices: Vec<u8>,
    pub stage1_scales: Vec<u8>,
    pub stage2_indices: Vec<u8>,
    pub stage2_scales: Vec<u8>,
    pub max_scale1: f32,
    pub max_scale2: f32,
}

impl E8FourBitMatrix {
    pub fn encode(weights: &[f32], rows: usize, cols: usize) -> Result<Self> {
        let total = rows * cols;
        if weights.len() != total {
            return Err(invalid(format!(
                "weights length {} != {rows}x{cols}",
                weights.len()
            )));
        }
        if total % E8_DIM != 0 {
            return Err(invalid(format!("total weights {total} not divisible by 8")));
        }

        let codebook = E8Codebook::new();
        let num_blocks = total / E8_DIM;
        let mut stage1_indices = Vec::with_capacity(num_blocks);
        let mut raw_scales1 = Vec::with_capacity(num_blocks);
        let mut stage2_indices = Vec::with_capacity(num_blocks);
        let mut raw_scales2 = Vec::with_capacity(num_blocks);
        let mut max_scale1 = 0.0f32;
        let mut max_scale2 = 0.0f32;

        for block in weights.chunks_exact(E8_DIM) {
            let mut b = [0.0f32; E8_DIM];
            b.copy_from_slice(block);

            // Stage 1
            let (idx1, scale1) = quantize_e8_block(&codebook, &b);
            stage1_indices.push(idx1);
            raw_scales1.push(scale1);
            max_scale1 = max_scale1.max(scale1);

            // Residual
            let cw1 = dequantize_e8_block(&codebook, idx1, scale1)?;
            let mut res = [0.0f32; E8_DIM];
            for j in 0..E8_DIM {
                res[j] = b[j] - cw1[j];
            }

            // Stage 2
            let (idx2, scale2) = quantize_e8_block(&codebook, &res);
            stage2_indices.push(idx2);
            raw_scales2.push(scale2);
            max_scale2 = max_scale2.max(scale2);
        }

        if max_scale1 == 0.0 {
            max_scale1 = 1.0;
        }
        if max_scale2 == 0.0 {
            max_scale2 = 1.0;
        }

        let mut stage1_scales = Vec::with_capacity(num_blocks);
        for &s in &raw_scales1 {
            stage1_scales.push(encode_scale_u8(s, max_scale1));
        }

        let mut stage2_scales = Vec::with_capacity(num_blocks);
        for &s in &raw_scales2 {
            stage2_scales.push(encode_scale_u8(s, max_scale2));
        }

        Ok(Self {
            rows,
            cols,
            stage1_indices,
            stage1_scales,
            stage2_indices,
            stage2_scales,
            max_scale1,
            max_scale2,
        })
    }

    pub fn serialized_bytes(&self) -> usize {
        16 + self.stage1_indices.len()
            + self.stage1_scales.len()
            + self.stage2_indices.len()
            + self.stage2_scales.len()
    }

    pub fn payload_bits_per_weight(&self) -> f64 {
        let total = self.rows * self.cols;
        let payload_bytes = self.stage1_indices.len()
            + self.stage1_scales.len()
            + self.stage2_indices.len()
            + self.stage2_scales.len();
        (payload_bytes as f64 * 8.0) / (total as f64)
    }

    pub fn dequantize(&self) -> Result<Vec<f32>> {
        let codebook = E8Codebook::new();
        let total = self.rows * self.cols;
        let mut out = Vec::with_capacity(total);

        for i in 0..self.stage1_indices.len() {
            let idx1 = self.stage1_indices[i];
            let s1 = decode_scale_u8(self.stage1_scales[i], self.max_scale1);
            let cw1 = dequantize_e8_block(&codebook, idx1, s1)?;

            let idx2 = self.stage2_indices[i];
            let s2 = decode_scale_u8(self.stage2_scales[i], self.max_scale2);
            let cw2 = dequantize_e8_block(&codebook, idx2, s2)?;

            for j in 0..E8_DIM {
                out.push(cw1[j] + cw2[j]);
            }
        }
        Ok(out)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.serialized_bytes());
        bytes.extend_from_slice(&(self.rows as u32).to_le_bytes());
        bytes.extend_from_slice(&(self.cols as u32).to_le_bytes());
        bytes.extend_from_slice(&self.max_scale1.to_le_bytes());
        bytes.extend_from_slice(&self.max_scale2.to_le_bytes());
        bytes.extend_from_slice(&self.stage1_indices);
        bytes.extend_from_slice(&self.stage1_scales);
        bytes.extend_from_slice(&self.stage2_indices);
        bytes.extend_from_slice(&self.stage2_scales);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 16 {
            return Err(invalid("buffer too short for E8FourBitMatrix header"));
        }
        let rows = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
        let cols = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let max_scale1 = f32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let max_scale2 = f32::from_le_bytes(bytes[12..16].try_into().unwrap());
        let num_blocks = (rows * cols) / E8_DIM;

        let expected_total = 16 + num_blocks * 4;
        if bytes.len() != expected_total {
            return Err(invalid(format!(
                "buffer length {} != expected {expected_total}",
                bytes.len()
            )));
        }

        let stage1_indices = bytes[16..16 + num_blocks].to_vec();
        let stage1_scales = bytes[16 + num_blocks..16 + num_blocks * 2].to_vec();
        let stage2_indices = bytes[16 + num_blocks * 2..16 + num_blocks * 3].to_vec();
        let stage2_scales = bytes[16 + num_blocks * 3..16 + num_blocks * 4].to_vec();

        Ok(Self {
            rows,
            cols,
            stage1_indices,
            stage1_scales,
            stage2_indices,
            stage2_scales,
            max_scale1,
            max_scale2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e8_codebook_properties() {
        let codebook = E8Codebook::new();
        assert_eq!(codebook.roots.len(), 241);
        assert_eq!(codebook.roots[0], [0; 8]);
        for (i, root) in codebook.roots.iter().enumerate().skip(1) {
            let mut norm_sq = 0i32;
            for &c in root {
                norm_sq += (c as i32) * (c as i32);
            }
            assert_eq!(norm_sq, 8, "root {i} norm squared must be 8");
        }
    }

    #[test]
    fn test_rtn_exact_round_trip_and_bitrate() {
        let rows = 4;
        let cols = 64; // 2 groups of 32
        let mut weights = Vec::with_capacity(rows * cols);
        for i in 0..(rows * cols) {
            weights.push((i as f32 - 128.0) * 0.01);
        }

        let matrix = Rtn4BitMatrix::encode(&weights, rows, cols).unwrap();
        assert_eq!(matrix.payload_bits_per_weight(), 4.2500);

        let bytes = matrix.to_bytes();
        let deserialized = Rtn4BitMatrix::from_bytes(&bytes).unwrap();
        assert_eq!(matrix, deserialized);

        let deq = deserialized.dequantize().unwrap();
        assert_eq!(deq.len(), weights.len());
    }

    #[test]
    fn test_e8_two_bit_round_trip_and_bitrate() {
        let rows = 4;
        let cols = 32; // 4 blocks of 8
        let weights = vec![0.1f32; rows * cols];

        let matrix = E8TwoBitMatrix::encode(&weights, rows, cols).unwrap();
        assert_eq!(matrix.payload_bits_per_weight(), 2.0000);

        let bytes = matrix.to_bytes();
        let deserialized = E8TwoBitMatrix::from_bytes(&bytes).unwrap();
        assert_eq!(matrix, deserialized);

        let deq = deserialized.dequantize().unwrap();
        assert_eq!(deq.len(), weights.len());
    }

    #[test]
    fn test_e8_three_bit_round_trip_and_bitrate() {
        let rows = 4;
        let cols = 32;
        let weights = vec![0.15f32; rows * cols];

        let matrix = E8ThreeBitMatrix::encode(&weights, rows, cols).unwrap();
        assert_eq!(matrix.payload_bits_per_weight(), 3.0000);

        let bytes = matrix.to_bytes();
        let deserialized = E8ThreeBitMatrix::from_bytes(&bytes).unwrap();
        assert_eq!(matrix, deserialized);

        let deq = deserialized.dequantize().unwrap();
        assert_eq!(deq.len(), weights.len());
    }

    #[test]
    fn test_e8_four_bit_round_trip_and_bitrate() {
        let rows = 4;
        let cols = 32;
        let weights = vec![0.25f32; rows * cols];

        let matrix = E8FourBitMatrix::encode(&weights, rows, cols).unwrap();
        assert_eq!(matrix.payload_bits_per_weight(), 4.0000);

        let bytes = matrix.to_bytes();
        let deserialized = E8FourBitMatrix::from_bytes(&bytes).unwrap();
        assert_eq!(matrix, deserialized);

        let deq = deserialized.dequantize().unwrap();
        assert_eq!(deq.len(), weights.len());
    }
}
