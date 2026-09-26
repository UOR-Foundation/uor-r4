//! A loaded model and an incremental decoding session.

use std::path::Path;

use uor_r4_simd::{dot_rows_i16_i8, mix_rows, Backend, Tables};

use crate::format::{Artifact, Header, MatrixSpec, Numerics, Shape};
use crate::kernels::{
    dequant_row, exp_neg, gemv, quantize16, quantize8, rms_norm, rope, shift, silu, to_exp_i32,
    Act16, MatrixView, Packed,
};
use crate::{format_error, invalid, Result, RESIDUAL_EXP};

/// Exponent of attention scores inside the softmax (`2^-12` nats).
const SCORE_EXP: i32 = -12;
/// Largest head dimension whose 16-by-8-bit dot products fit `i32`.
const MAX_HEAD_DIM: usize = 256;
/// Heads run on separate threads once `(position + 1) * head_dim` reaches this.
const PARALLEL_ATTENTION: usize = 1 << 14;

/// The row-major embedding, read one row per token.
struct Embedding {
    spec: MatrixSpec,
    nibbles: Vec<u8>,
    scales: Vec<u8>,
}

impl Embedding {
    fn view(&self) -> MatrixView<'_> {
        MatrixView {
            rows: self.spec.rows,
            cols: self.spec.cols,
            exp_base: self.spec.exp_base,
            nibbles: &self.nibbles,
            scales: &self.scales,
        }
    }
}

struct Layer {
    q: Packed,
    k: Packed,
    v: Packed,
    o: Packed,
    gate: Packed,
    up: Packed,
    down: Packed,
}

/// One layer's key/value cache, read by attention.
struct LayerCache<'a> {
    keys: &'a [i8],
    values: &'a [i8],
    key_exp: &'a [i32],
    value_exp: &'a [i32],
}

/// Buffers of one attention head (one set per worker thread).
#[derive(Default)]
struct HeadScratch {
    wide: Vec<i64>,
    q16: Act16,
    dots: Vec<i32>,
    scores: Vec<i64>,
    weights: Vec<i32>,
    downs: Vec<i32>,
    mix: Vec<i64>,
}

/// A validated artifact repacked for integer serving. The artifact bytes are
/// released after loading; the header and its SHA-256 are kept.
pub struct Model {
    header: Header,
    sha256: String,
    backend: Backend,
    threads: usize,
    /// Worker pool for more than one thread; a decoding step runs inside it,
    /// so its parallel matrix products cost no cross-thread hand-off.
    pool: Option<rayon::ThreadPool>,
    shape: Shape,
    numerics: Numerics,
    embed: Embedding,
    head: Packed,
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
        let spec = |name: &str, rows: usize, cols: usize| -> Result<MatrixSpec> {
            let spec = artifact.matrix(name)?.clone();
            if spec.rows != rows || spec.cols != cols {
                return Err(format_error(format!(
                    "matrix {name} is {}x{}, expected {rows}x{cols}",
                    spec.rows, spec.cols
                )));
            }
            Ok(spec)
        };
        let packed = |name: &str, rows: usize, cols: usize| -> Result<Packed> {
            let spec = spec(name, rows, cols)?;
            Packed::new(&MatrixView {
                rows,
                cols,
                exp_base: spec.exp_base,
                nibbles: artifact.section(spec.nibbles),
                scales: artifact.section(spec.scales),
            })
        };
        let mut layers = Vec::with_capacity(shape.layers);
        for l in 0..shape.layers {
            layers.push(Layer {
                q: packed(&format!("l{l}.q"), shape.width, shape.width)?,
                k: packed(&format!("l{l}.k"), kv_rows, shape.width)?,
                v: packed(&format!("l{l}.v"), kv_rows, shape.width)?,
                o: packed(&format!("l{l}.o"), shape.width, shape.width)?,
                gate: packed(&format!("l{l}.gate"), shape.ffn, shape.width)?,
                up: packed(&format!("l{l}.up"), shape.ffn, shape.width)?,
                down: packed(&format!("l{l}.down"), shape.width, shape.ffn)?,
            });
        }
        let embed_spec = spec("embed", shape.vocab, shape.width)?;
        let embed = Embedding {
            nibbles: artifact.section(embed_spec.nibbles).to_vec(),
            scales: artifact.section(embed_spec.scales).to_vec(),
            spec: embed_spec,
        };
        let head = packed("head", shape.vocab, shape.width)?;
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
            || shape.head_dim > MAX_HEAD_DIM
        {
            return Err(format_error(
                "sealed tables do not match the declared numerics",
            ));
        }
        Ok(Self {
            header: artifact.header.clone(),
            sha256: artifact.sha256.clone(),
            backend: Backend::detect(),
            threads: 1,
            pool: None,
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
        &self.sha256
    }

    pub fn source(&self) -> &serde_json::Value {
        &self.header.source
    }

    /// The sealed exp table (`round(2^31 exp(-i 2^exp_step_log2))`) and its step.
    pub fn exp_table(&self) -> (&[u32], i32) {
        (&self.exp_table, self.numerics.exp_step_log2)
    }

    /// The vector backend of the weight kernels (detected at load).
    pub fn backend(&self) -> Backend {
        self.backend
    }

    pub fn threads(&self) -> usize {
        self.threads
    }

    /// Use `threads` worker threads (1: none) for large matrix products;
    /// every thread count computes the same integers.
    pub fn set_threads(&mut self, threads: usize) -> Result<()> {
        if threads == 0 {
            return Err(invalid("threads must be positive"));
        }
        self.pool = if threads == 1 {
            None
        } else {
            Some(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(threads)
                    .build()
                    .map_err(|e| invalid(format!("thread pool: {e}")))?,
            )
        };
        self.threads = threads;
        Ok(())
    }

    /// Select a backend; every backend computes the same integers.
    pub fn set_backend(&mut self, backend: Backend) -> Result<()> {
        if !backend.available() {
            return Err(invalid(format!(
                "the {} backend is not available on this CPU",
                backend.name()
            )));
        }
        self.backend = backend;
        Ok(())
    }

    /// One attention head (key/value head `g`) over positions `0..=position`,
    /// written into `out` at the residual exponent.
    fn attend(
        &self,
        g: usize,
        query: &[i32],
        cache: &LayerCache<'_>,
        position: usize,
        scratch: &mut HeadScratch,
        out: &mut [i32],
    ) -> Result<()> {
        let (s, n) = (&self.shape, &self.numerics);
        let (hd, kv) = (s.head_dim, s.kv_heads);
        let stride = kv * hd;
        scratch.wide.clear();
        scratch.wide.extend(query.iter().map(|v| i64::from(*v)));
        quantize16(&scratch.wide, RESIDUAL_EXP, &mut scratch.q16);
        scratch.dots.clear();
        scratch.dots.resize(position + 1, 0);
        dot_rows_i16_i8(
            &scratch.q16.values,
            cache.keys,
            stride,
            g * hd,
            &mut scratch.dots,
        )?;
        scratch.scores.clear();
        for (u, dot) in scratch.dots.iter().enumerate() {
            let exp = scratch.q16.exp + cache.key_exp[u * kv + g] - 30;
            scratch
                .scores
                .push(shift(i64::from(*dot) * n.score_scale_q30, SCORE_EXP - exp));
        }
        let max = scratch.scores.iter().copied().max().unwrap_or(0);
        scratch.weights.clear();
        let mut total = 0u64;
        for score in &scratch.scores {
            // At most 2^31 >> 7 = 2^24.
            let w = exp_neg(max - score, SCORE_EXP, &self.exp_table, n.exp_step_log2) >> 7;
            total += w;
            scratch.weights.push(w as i32);
        }
        let total = total.max(1);
        let e_max = (0..=position)
            .map(|u| cache.value_exp[u * kv + g])
            .max()
            .unwrap_or(RESIDUAL_EXP);
        scratch.downs.clear();
        scratch
            .downs
            .extend((0..=position).map(|u| e_max - cache.value_exp[u * kv + g]));
        scratch.mix.clear();
        scratch.mix.resize(hd, 0);
        mix_rows(
            &scratch.weights,
            &scratch.downs,
            cache.values,
            stride,
            g * hd,
            &mut scratch.mix,
        )?;
        let reciprocal = (1u128 << 62) / u128::from(total);
        for (slot, m) in out.iter_mut().zip(&scratch.mix) {
            let normalized = (i128::from(*m) * reciprocal as i128) >> 62;
            *slot = to_exp_i32(normalized as i64, e_max, RESIDUAL_EXP);
        }
        Ok(())
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
            tables: Tables::default(),
            scratch: Vec::new(),
            q: vec![0; s.width],
            k: vec![0; s.kv_heads * s.head_dim],
            v: vec![0; s.kv_heads * s.head_dim],
            head_out: vec![0; s.width],
            head: HeadScratch::default(),
            proj: vec![0; s.width],
            gate: vec![0; s.ffn],
            up: vec![0; s.ffn],
            hidden: Act16::default(),
            logits: vec![0; s.vocab],
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
    tables: Tables,
    scratch: Vec<i64>,
    q: Vec<i32>,
    k: Vec<i32>,
    v: Vec<i32>,
    head_out: Vec<i32>,
    head: HeadScratch,
    proj: Vec<i32>,
    gate: Vec<i32>,
    up: Vec<i32>,
    hidden: Act16,
    logits: Vec<i32>,
    cache8: Vec<i8>,
}

impl<'m> Session<'m> {
    /// The model this session decodes with.
    pub fn model(&self) -> &'m Model {
        self.model
    }
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
        match &model.pool {
            Some(pool) => pool.install(|| self.advance(token))?,
            None => self.advance(token)?,
        }
        Ok(&self.logits)
    }

    fn advance(&mut self, token: u32) -> Result<()> {
        let model = self.model;
        let parallel = model.pool.is_some();
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
            &model.embed.view(),
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
            self.tables.build(&self.norm.values)?;
            gemv(
                &layer.q,
                model.backend,
                parallel,
                &self.tables,
                self.norm.exp,
                &mut self.q,
                RESIDUAL_EXP,
            )?;
            gemv(
                &layer.k,
                model.backend,
                parallel,
                &self.tables,
                self.norm.exp,
                &mut self.k,
                RESIDUAL_EXP,
            )?;
            gemv(
                &layer.v,
                model.backend,
                parallel,
                &self.tables,
                self.norm.exp,
                &mut self.v,
                RESIDUAL_EXP,
            )?;
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
            let cache = LayerCache {
                keys: &self.keys[l],
                values: &self.values[l],
                key_exp: &self.key_exp[l],
                value_exp: &self.value_exp[l],
            };
            let q = &self.q;
            if parallel && (position + 1) * hd >= PARALLEL_ATTENTION {
                use rayon::prelude::*;
                self.head_out
                    .par_chunks_mut(hd)
                    .enumerate()
                    .try_for_each_init(HeadScratch::default, |scratch, (h, out)| {
                        let query = &q[h * hd..(h + 1) * hd];
                        model.attend(h / group, query, &cache, position, scratch, out)
                    })?;
            } else {
                for (h, out) in self.head_out.chunks_mut(hd).enumerate() {
                    let query = &q[h * hd..(h + 1) * hd];
                    model.attend(h / group, query, &cache, position, &mut self.head, out)?;
                }
            }
            self.scratch.clear();
            self.scratch
                .extend(self.head_out.iter().map(|v| i64::from(*v)));
            quantize16(&self.scratch, RESIDUAL_EXP, &mut self.norm);
            self.tables.build(&self.norm.values)?;
            gemv(
                &layer.o,
                model.backend,
                parallel,
                &self.tables,
                self.norm.exp,
                &mut self.proj,
                RESIDUAL_EXP,
            )?;
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
            self.tables.build(&self.norm.values)?;
            gemv(
                &layer.gate,
                model.backend,
                parallel,
                &self.tables,
                self.norm.exp,
                &mut self.gate,
                RESIDUAL_EXP,
            )?;
            gemv(
                &layer.up,
                model.backend,
                parallel,
                &self.tables,
                self.norm.exp,
                &mut self.up,
                RESIDUAL_EXP,
            )?;
            self.scratch.clear();
            for (g, u) in self.gate.iter().zip(&self.up) {
                let a = silu(*g, &model.silu_table, n.silu_step_log2, n.silu_range_log2);
                self.scratch.push(i64::from(a) * i64::from(*u));
            }
            quantize16(&self.scratch, 2 * RESIDUAL_EXP, &mut self.hidden);
            self.tables.build(&self.hidden.values)?;
            gemv(
                &layer.down,
                model.backend,
                parallel,
                &self.tables,
                self.hidden.exp,
                &mut self.proj,
                RESIDUAL_EXP,
            )?;
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
        self.tables.build(&self.norm.values)?;
        gemv(
            &model.head,
            model.backend,
            parallel,
            &self.tables,
            self.norm.exp,
            &mut self.logits,
            RESIDUAL_EXP,
        )?;
        self.position += 1;
        Ok(())
    }
}
