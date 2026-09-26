//! A loaded model and an incremental decoding session.

use std::path::Path;

use crate::format::{Artifact, MatrixSpec, Numerics, Shape};
use crate::kernels::{
    build_multiples, dequant_row, exp_neg, gemv, quantize16, quantize8, rms_norm, rope, row_min_de,
    shift, silu, to_exp_i32, Act16, MatrixView,
};
use crate::{format_error, invalid, Result, RESIDUAL_EXP};

/// Exponent of attention scores inside the softmax (`2^-12` nats).
const SCORE_EXP: i32 = -12;

struct Matrix {
    spec: MatrixSpec,
    row_min_de: Vec<u8>,
}

struct Layer {
    q: Matrix,
    k: Matrix,
    v: Matrix,
    o: Matrix,
    gate: Matrix,
    up: Matrix,
    down: Matrix,
}

/// A validated artifact ready for integer serving.
pub struct Model {
    artifact: Artifact,
    shape: Shape,
    numerics: Numerics,
    embed: Matrix,
    head: Matrix,
    layers: Vec<Layer>,
    exp_table: Vec<u32>,
    silu_table: Vec<i32>,
    cos: Vec<i16>,
    sin: Vec<i16>,
}

impl Model {
    pub fn load(path: &Path) -> Result<Self> {
        Self::from_artifact(Artifact::load(path)?)
    }

    pub fn from_artifact(artifact: Artifact) -> Result<Self> {
        let shape = artifact.header.shape.clone();
        let numerics = artifact.header.numerics.clone();
        let kv_rows = shape.kv_heads * shape.head_dim;
        let matrix = |name: &str, rows: usize, cols: usize| -> Result<Matrix> {
            let spec = artifact.matrix(name)?.clone();
            if spec.rows != rows || spec.cols != cols {
                return Err(format_error(format!(
                    "matrix {name} is {}x{}, expected {rows}x{cols}",
                    spec.rows, spec.cols
                )));
            }
            let row_min_de = row_min_de(rows, cols, artifact.section(spec.scales));
            Ok(Matrix { spec, row_min_de })
        };
        let mut layers = Vec::with_capacity(shape.layers);
        for l in 0..shape.layers {
            layers.push(Layer {
                q: matrix(&format!("l{l}.q"), shape.width, shape.width)?,
                k: matrix(&format!("l{l}.k"), kv_rows, shape.width)?,
                v: matrix(&format!("l{l}.v"), kv_rows, shape.width)?,
                o: matrix(&format!("l{l}.o"), shape.width, shape.width)?,
                gate: matrix(&format!("l{l}.gate"), shape.ffn, shape.width)?,
                up: matrix(&format!("l{l}.up"), shape.ffn, shape.width)?,
                down: matrix(&format!("l{l}.down"), shape.width, shape.ffn)?,
            });
        }
        let embed = matrix("embed", shape.vocab, shape.width)?;
        let head = matrix("head", shape.vocab, shape.width)?;
        let exp_table = artifact.table_u32("exp")?;
        let silu_table = artifact.table_i32("silu")?;
        let cos = artifact.table_i16("rope_cos")?;
        let sin = artifact.table_i16("rope_sin")?;
        let rope_len = shape.max_positions * shape.head_dim / 2;
        if exp_table.len() < 2
            || silu_table.len()
                != (2usize << (numerics.silu_range_log2 - numerics.silu_step_log2)) + 1
            || cos.len() != rope_len
            || sin.len() != rope_len
            || !(-16..0).contains(&numerics.silu_step_log2)
            || !(SCORE_EXP..0).contains(&numerics.exp_step_log2)
            || !(1..=15).contains(&numerics.rope_q)
        {
            return Err(format_error(
                "sealed tables do not match the declared numerics",
            ));
        }
        Ok(Self {
            artifact,
            shape,
            numerics,
            embed,
            head,
            layers,
            exp_table,
            silu_table,
            cos,
            sin,
        })
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// SHA-256 of the artifact bytes.
    pub fn artifact_sha256(&self) -> &str {
        &self.artifact.sha256
    }

    pub fn source(&self) -> &serde_json::Value {
        &self.artifact.header.source
    }

    fn view<'a>(&'a self, m: &'a Matrix) -> MatrixView<'a> {
        MatrixView {
            rows: m.spec.rows,
            cols: m.spec.cols,
            exp_base: m.spec.exp_base,
            nibbles: self.artifact.section(m.spec.nibbles),
            scales: self.artifact.section(m.spec.scales),
            row_min_de: &m.row_min_de,
        }
    }

    /// A fresh decoding session with an empty cache.
    pub fn session(&self) -> Session<'_> {
        let s = &self.shape;
        Session {
            model: self,
            position: 0,
            keys: vec![Vec::new(); s.layers],
            key_exp: vec![Vec::new(); s.layers],
            values: vec![Vec::new(); s.layers],
            value_exp: vec![Vec::new(); s.layers],
            x: vec![0; s.width],
            norm: Act16::default(),
            table: Vec::new(),
            scratch: Vec::new(),
            q: vec![0; s.width],
            k: vec![0; s.kv_heads * s.head_dim],
            v: vec![0; s.kv_heads * s.head_dim],
            q16: Act16::default(),
            head_out: vec![0; s.width],
            proj: vec![0; s.width],
            gate: vec![0; s.ffn],
            up: vec![0; s.ffn],
            hidden: Act16::default(),
            logits: vec![0; s.vocab],
            scores: Vec::new(),
            weights: Vec::new(),
            mix: vec![0; s.head_dim],
            cache8: vec![0; s.head_dim],
        }
    }
}

/// Incremental decoding with an 8-bit key/value cache.
pub struct Session<'m> {
    model: &'m Model,
    position: usize,
    keys: Vec<Vec<i8>>,
    key_exp: Vec<Vec<i32>>,
    values: Vec<Vec<i8>>,
    value_exp: Vec<Vec<i32>>,
    x: Vec<i32>,
    norm: Act16,
    table: Vec<i32>,
    scratch: Vec<i64>,
    q: Vec<i32>,
    k: Vec<i32>,
    v: Vec<i32>,
    q16: Act16,
    head_out: Vec<i32>,
    proj: Vec<i32>,
    gate: Vec<i32>,
    up: Vec<i32>,
    hidden: Act16,
    logits: Vec<i32>,
    scores: Vec<i64>,
    weights: Vec<u64>,
    mix: Vec<i64>,
    cache8: Vec<i8>,
}

impl Session<'_> {
    pub fn position(&self) -> usize {
        self.position
    }

    pub fn reset(&mut self) {
        self.position = 0;
        for cache in [&mut self.keys, &mut self.values] {
            for layer in cache.iter_mut() {
                layer.clear();
            }
        }
        for cache in [&mut self.key_exp, &mut self.value_exp] {
            for layer in cache.iter_mut() {
                layer.clear();
            }
        }
    }

    /// Feed one token at the next position; returns the next-token logits
    /// (value `v` means `v * 2^-16`).
    pub fn step(&mut self, token: u32) -> Result<&[i32]> {
        let model = self.model;
        let s = &model.shape;
        let n = &model.numerics;
        if token as usize >= s.vocab {
            return Err(invalid(format!("token {token} outside the vocabulary")));
        }
        if self.position >= s.max_positions {
            return Err(invalid("the session is full (max_positions)"));
        }
        let (hd, group) = (s.head_dim, s.heads / s.kv_heads);
        let position = self.position;
        dequant_row(
            &model.view(&model.embed),
            token as usize,
            &mut self.x,
            RESIDUAL_EXP,
        );
        for (l, layer) in model.layers.iter().enumerate() {
            // Attention block.
            rms_norm(
                &self.x,
                RESIDUAL_EXP,
                n.rms_eps,
                &mut self.scratch,
                &mut self.norm,
            );
            build_multiples(&self.norm.values, &mut self.table);
            gemv(
                &model.view(&layer.q),
                &self.table,
                self.norm.exp,
                &mut self.q,
                RESIDUAL_EXP,
            );
            gemv(
                &model.view(&layer.k),
                &self.table,
                self.norm.exp,
                &mut self.k,
                RESIDUAL_EXP,
            );
            gemv(
                &model.view(&layer.v),
                &self.table,
                self.norm.exp,
                &mut self.v,
                RESIDUAL_EXP,
            );
            for head in self.q.chunks_exact_mut(hd) {
                rope(head, position, &model.cos, &model.sin, n.rope_q);
            }
            for head in self.k.chunks_exact_mut(hd) {
                rope(head, position, &model.cos, &model.sin, n.rope_q);
            }
            for g in 0..s.kv_heads {
                for (source, cache, exps) in [
                    (&self.k, &mut self.keys[l], &mut self.key_exp[l]),
                    (&self.v, &mut self.values[l], &mut self.value_exp[l]),
                ] {
                    self.scratch.clear();
                    self.scratch
                        .extend(source[g * hd..(g + 1) * hd].iter().map(|v| i64::from(*v)));
                    let exp = quantize8(&self.scratch, RESIDUAL_EXP, &mut self.cache8);
                    cache.extend_from_slice(&self.cache8);
                    exps.push(exp);
                }
            }
            let keys = &self.keys[l];
            let values = &self.values[l];
            let (key_exp, value_exp) = (&self.key_exp[l], &self.value_exp[l]);
            let stride = s.kv_heads * hd;
            for h in 0..s.heads {
                let g = h / group;
                self.scratch.clear();
                self.scratch
                    .extend(self.q[h * hd..(h + 1) * hd].iter().map(|v| i64::from(*v)));
                quantize16(&self.scratch, RESIDUAL_EXP, &mut self.q16);
                self.scores.clear();
                for u in 0..=position {
                    let key = &keys[u * stride + g * hd..u * stride + (g + 1) * hd];
                    let dot: i64 = self
                        .q16
                        .values
                        .iter()
                        .zip(key)
                        .map(|(a, b)| i64::from(*a) * i64::from(*b))
                        .sum();
                    let exp = self.q16.exp + key_exp[u * s.kv_heads + g] - 30;
                    self.scores
                        .push(shift(dot * n.score_scale_q30, SCORE_EXP - exp));
                }
                let max = self.scores.iter().copied().max().unwrap_or(0);
                self.weights.clear();
                let mut total = 0u64;
                for score in &self.scores {
                    let w = exp_neg(max - score, SCORE_EXP, &model.exp_table, n.exp_step_log2) >> 7;
                    total += w;
                    self.weights.push(w);
                }
                let total = total.max(1);
                let e_max = (0..=position)
                    .map(|u| value_exp[u * s.kv_heads + g])
                    .max()
                    .unwrap_or(RESIDUAL_EXP);
                self.mix.iter_mut().for_each(|m| *m = 0);
                for (u, w) in self.weights.iter().enumerate() {
                    if *w == 0 {
                        continue;
                    }
                    let value = &values[u * stride + g * hd..u * stride + (g + 1) * hd];
                    let down = e_max - value_exp[u * s.kv_heads + g];
                    for (m, v) in self.mix.iter_mut().zip(value) {
                        *m += shift(*w as i64 * i64::from(*v), down);
                    }
                }
                let reciprocal = (1u128 << 62) / u128::from(total);
                for (i, m) in self.mix.iter().enumerate() {
                    let normalized = (i128::from(*m) * reciprocal as i128) >> 62;
                    self.head_out[h * hd + i] = to_exp_i32(normalized as i64, e_max, RESIDUAL_EXP);
                }
            }
            self.scratch.clear();
            self.scratch
                .extend(self.head_out.iter().map(|v| i64::from(*v)));
            quantize16(&self.scratch, RESIDUAL_EXP, &mut self.norm);
            build_multiples(&self.norm.values, &mut self.table);
            gemv(
                &model.view(&layer.o),
                &self.table,
                self.norm.exp,
                &mut self.proj,
                RESIDUAL_EXP,
            );
            for (x, p) in self.x.iter_mut().zip(&self.proj) {
                *x = x.saturating_add(*p);
            }
            // MLP block.
            rms_norm(
                &self.x,
                RESIDUAL_EXP,
                n.rms_eps,
                &mut self.scratch,
                &mut self.norm,
            );
            build_multiples(&self.norm.values, &mut self.table);
            gemv(
                &model.view(&layer.gate),
                &self.table,
                self.norm.exp,
                &mut self.gate,
                RESIDUAL_EXP,
            );
            gemv(
                &model.view(&layer.up),
                &self.table,
                self.norm.exp,
                &mut self.up,
                RESIDUAL_EXP,
            );
            self.scratch.clear();
            for (g, u) in self.gate.iter().zip(&self.up) {
                let a = silu(*g, &model.silu_table, n.silu_step_log2, n.silu_range_log2);
                self.scratch.push(i64::from(a) * i64::from(*u));
            }
            quantize16(&self.scratch, 2 * RESIDUAL_EXP, &mut self.hidden);
            build_multiples(&self.hidden.values, &mut self.table);
            gemv(
                &model.view(&layer.down),
                &self.table,
                self.hidden.exp,
                &mut self.proj,
                RESIDUAL_EXP,
            );
            for (x, p) in self.x.iter_mut().zip(&self.proj) {
                *x = x.saturating_add(*p);
            }
        }
        rms_norm(
            &self.x,
            RESIDUAL_EXP,
            n.rms_eps,
            &mut self.scratch,
            &mut self.norm,
        );
        build_multiples(&self.norm.values, &mut self.table);
        gemv(
            &model.view(&model.head),
            &self.table,
            self.norm.exp,
            &mut self.logits,
            RESIDUAL_EXP,
        );
        self.position += 1;
        Ok(&self.logits)
    }
}
