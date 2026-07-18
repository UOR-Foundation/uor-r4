//! The TEACHER: faithful Rust port of karpathy run.c forward pass (v0 checkpoint).
//! Arithmetic order mirrors the C exactly: sequential adds in matmul rows,
//! rmsnorm/softmax/RoPE/SwiGLU op-for-op, libm via glibc on gnu targets.
//! matmul is parallelized over OUTPUT ROWS only; each row's chain is the
//! same serial reduction, so threaded output is bit-identical to serial
//! (witnessed in greedy_check).

pub struct Config {
    pub dim: usize,
    pub hidden: usize,
    pub n_layers: usize,
    pub n_heads: usize,
    pub n_kv_heads: usize,
    pub vocab: usize,
    pub seq_len: usize,
}

pub struct Llama {
    pub cfg: Config,
    w: Vec<f32>,
    // float offsets into w
    emb: usize,
    rms_att: usize,
    wq: usize,
    wk: usize,
    wv: usize,
    wo: usize,
    rms_ffn: usize,
    w1: usize,
    w2: usize,
    w3: usize,
    rms_final: usize,
    wcls: usize,
}

pub struct State {
    pub x: Vec<f32>,
    xb: Vec<f32>,
    xb2: Vec<f32>,
    hb: Vec<f32>,
    hb2: Vec<f32>,
    q: Vec<f32>,
    att: Vec<f32>,
    key_cache: Vec<f32>,
    value_cache: Vec<f32>,
    pub logits: Vec<f32>,
}

impl State {
    pub fn new(c: &Config) -> Self {
        let kv_dim = c.dim * c.n_kv_heads / c.n_heads;
        State {
            x: vec![0.0; c.dim],
            xb: vec![0.0; c.dim],
            xb2: vec![0.0; c.dim],
            hb: vec![0.0; c.hidden],
            hb2: vec![0.0; c.hidden],
            q: vec![0.0; c.dim],
            att: vec![0.0; c.n_heads * c.seq_len],
            key_cache: vec![0.0; c.n_layers * c.seq_len * kv_dim],
            value_cache: vec![0.0; c.n_layers * c.seq_len * kv_dim],
            logits: vec![0.0; c.vocab],
        }
    }
}

fn rmsnorm(o: &mut [f32], x: &[f32], weight: &[f32]) {
    let size = x.len();
    let mut ss = 0.0f32;
    for j in 0..size {
        ss += x[j] * x[j];
    }
    ss /= size as f32;
    ss += 1e-5f32;
    ss = 1.0f32 / ss.sqrt();
    for j in 0..size {
        o[j] = weight[j] * (ss * x[j]);
    }
}

/// In-place variant matching C's rmsnorm(x, x, w): C computes ss from x
/// first, then writes; identical here.
fn rmsnorm_inplace(x: &mut [f32], weight: &[f32]) {
    let size = x.len();
    let mut ss = 0.0f32;
    for j in 0..size {
        ss += x[j] * x[j];
    }
    ss /= size as f32;
    ss += 1e-5f32;
    ss = 1.0f32 / ss.sqrt();
    for j in 0..size {
        x[j] = weight[j] * (ss * x[j]);
    }
}

fn softmax(x: &mut [f32]) {
    let mut max_val = x[0];
    for i in 1..x.len() {
        if x[i] > max_val {
            max_val = x[i];
        }
    }
    let mut sum = 0.0f32;
    for i in 0..x.len() {
        x[i] = (x[i] - max_val).exp();
        sum += x[i];
    }
    for i in 0..x.len() {
        x[i] /= sum;
    }
}

/// W (d,n) @ x (n,) -> xout (d,). Row-parallel; each row is the C serial
/// chain, so the result is bit-identical to a serial loop.
fn matmul(xout: &mut [f32], x: &[f32], w: &[f32], n: usize, threads: usize) {
    let d = xout.len();
    if threads <= 1 || d < 64 {
        // 4 rows in flight to hide FP add latency. Each row's accumulation
        // chain is unchanged (strictly sequential in j), so every output
        // bit matches the naive loop.
        let mut i = 0usize;
        while i + 4 <= d {
            let r0 = &w[i * n..i * n + n];
            let r1 = &w[(i + 1) * n..(i + 1) * n + n];
            let r2 = &w[(i + 2) * n..(i + 2) * n + n];
            let r3 = &w[(i + 3) * n..(i + 3) * n + n];
            let (mut v0, mut v1, mut v2, mut v3) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
            for j in 0..n {
                let xj = x[j];
                v0 += r0[j] * xj;
                v1 += r1[j] * xj;
                v2 += r2[j] * xj;
                v3 += r3[j] * xj;
            }
            xout[i] = v0;
            xout[i + 1] = v1;
            xout[i + 2] = v2;
            xout[i + 3] = v3;
            i += 4;
        }
        while i < d {
            let mut val = 0.0f32;
            let row = &w[i * n..i * n + n];
            for j in 0..n {
                val += row[j] * x[j];
            }
            xout[i] = val;
            i += 1;
        }
        return;
    }
    let chunk = (d + threads - 1) / threads;
    std::thread::scope(|s| {
        for (ci, out) in xout.chunks_mut(chunk).enumerate() {
            let base = ci * chunk;
            s.spawn(move || {
                for (o, i) in out.iter_mut().zip(base..) {
                    let mut val = 0.0f32;
                    let row = &w[i * n..i * n + n];
                    for j in 0..n {
                        val += row[j] * x[j];
                    }
                    *o = val;
                }
            });
        }
    });
}

impl Llama {
    pub fn load(path: &str) -> Llama {
        let raw = std::fs::read(path).expect("checkpoint");
        let i32at = |o: usize| i32::from_le_bytes(raw[o..o + 4].try_into().unwrap());
        let vocab_raw = i32at(20);
        let cfg = Config {
            dim: i32at(0) as usize,
            hidden: i32at(4) as usize,
            n_layers: i32at(8) as usize,
            n_heads: i32at(12) as usize,
            n_kv_heads: i32at(16) as usize,
            vocab: vocab_raw.unsigned_abs() as usize,
            seq_len: i32at(24) as usize,
        };
        let shared = vocab_raw > 0;
        let nf = (raw.len() - 28) / 4;
        let mut w = vec![0.0f32; nf];
        for i in 0..nf {
            let o = 28 + i * 4;
            w[i] = f32::from_le_bytes(raw[o..o + 4].try_into().unwrap());
        }
        let (dim, hid, nl, hs) = (cfg.dim, cfg.hidden, cfg.n_layers, cfg.dim / cfg.n_heads);
        let kv_dim = cfg.dim * cfg.n_kv_heads / cfg.n_heads;
        let mut p = 0usize;
        let emb = p;
        p += cfg.vocab * dim;
        let rms_att = p;
        p += nl * dim;
        let wq = p;
        p += nl * dim * dim;
        let wk = p;
        p += nl * dim * kv_dim;
        let wv = p;
        p += nl * dim * kv_dim;
        let wo = p;
        p += nl * dim * dim;
        let rms_ffn = p;
        p += nl * dim;
        let w1 = p;
        p += nl * dim * hid;
        let w2 = p;
        p += nl * hid * dim;
        let w3 = p;
        p += nl * dim * hid;
        let rms_final = p;
        p += dim;
        p += cfg.seq_len * hs / 2; // skip legacy freq_cis_real
        p += cfg.seq_len * hs / 2; // skip legacy freq_cis_imag
        let wcls = if shared { emb } else { p };
        Llama {
            cfg,
            w,
            emb,
            rms_att,
            wq,
            wk,
            wv,
            wo,
            rms_ffn,
            w1,
            w2,
            w3,
            rms_final,
            wcls,
        }
    }

    /// One forward step. After return, st.x holds the post-final-rmsnorm
    /// hidden state (the kNN-LM context vector) and st.logits the logits.
    pub fn forward(&self, st: &mut State, token: usize, pos: usize, threads: usize) {
        let c = &self.cfg;
        let (dim, hid) = (c.dim, c.hidden);
        let kv_dim = c.dim * c.n_kv_heads / c.n_heads;
        let kv_mul = c.n_heads / c.n_kv_heads;
        let head_size = dim / c.n_heads;
        let w = &self.w;

        st.x.copy_from_slice(&w[self.emb + token * dim..self.emb + (token + 1) * dim]);

        for l in 0..c.n_layers {
            rmsnorm(&mut st.xb, &st.x, &w[self.rms_att + l * dim..self.rms_att + (l + 1) * dim]);

            let loff = l * c.seq_len * kv_dim;
            matmul(&mut st.q, &st.xb, &w[self.wq + l * dim * dim..], dim, threads);
            {
                let k = &mut st.key_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                matmul(k, &st.xb, &w[self.wk + l * dim * kv_dim..], dim, threads);
            }
            {
                let v = &mut st.value_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                matmul(v, &st.xb, &w[self.wv + l * dim * kv_dim..], dim, threads);
            }

            // RoPE, exactly as the pinned run.c (on-the-fly freqs).
            {
                let k = &mut st.key_cache[loff + pos * kv_dim..loff + (pos + 1) * kv_dim];
                let mut i = 0usize;
                while i < dim {
                    let head_dim = i % head_size;
                    let freq = 1.0f32 / 10000.0f32.powf(head_dim as f32 / head_size as f32);
                    let val = pos as f32 * freq;
                    let fcr = val.cos();
                    let fci = val.sin();
                    let rotn = if i < kv_dim { 2 } else { 1 };
                    for v in 0..rotn {
                        let vec: &mut [f32] = if v == 0 { &mut st.q } else { &mut *k };
                        let v0 = vec[i];
                        let v1 = vec[i + 1];
                        vec[i] = v0 * fcr - v1 * fci;
                        vec[i + 1] = v0 * fci + v1 * fcr;
                    }
                    i += 2;
                }
            }

            // multihead attention (serial over heads; per-head work is
            // independent of order).
            for h in 0..c.n_heads {
                let q = &st.q[h * head_size..(h + 1) * head_size];
                let att = &mut st.att[h * c.seq_len..h * c.seq_len + pos + 1];
                for t in 0..=pos {
                    let k = &st.key_cache
                        [loff + t * kv_dim + (h / kv_mul) * head_size..][..head_size];
                    let mut score = 0.0f32;
                    for i in 0..head_size {
                        score += q[i] * k[i];
                    }
                    score /= (head_size as f32).sqrt();
                    att[t] = score;
                }
                softmax(att);
                let xb = &mut st.xb[h * head_size..(h + 1) * head_size];
                xb.iter_mut().for_each(|v| *v = 0.0);
                for t in 0..=pos {
                    let v = &st.value_cache
                        [loff + t * kv_dim + (h / kv_mul) * head_size..][..head_size];
                    let a = att[t];
                    for i in 0..head_size {
                        xb[i] += a * v[i];
                    }
                }
            }

            matmul(&mut st.xb2, &st.xb, &w[self.wo + l * dim * dim..], dim, threads);
            for i in 0..dim {
                st.x[i] += st.xb2[i];
            }

            rmsnorm(&mut st.xb, &st.x, &w[self.rms_ffn + l * dim..self.rms_ffn + (l + 1) * dim]);
            matmul(&mut st.hb, &st.xb, &w[self.w1 + l * dim * hid..], dim, threads);
            matmul(&mut st.hb2, &st.xb, &w[self.w3 + l * dim * hid..], dim, threads);
            for i in 0..hid {
                let mut val = st.hb[i];
                val *= 1.0f32 / (1.0f32 + (-val).exp());
                val *= st.hb2[i];
                st.hb[i] = val;
            }
            matmul(&mut st.xb, &st.hb, &w[self.w2 + l * hid * dim..], hid, threads);
            for i in 0..dim {
                st.x[i] += st.xb[i];
            }
        }

        let rf = self.rms_final;
        // C: rmsnorm(x, x, w) — in-place with pre-read ss.
        {
            let (wslice, x) = (&w[rf..rf + dim], &mut st.x);
            rmsnorm_inplace(x, wslice);
        }
        matmul(&mut st.logits, &st.x, &w[self.wcls..], dim, threads);
    }
}

/// The TWO-SURFACE interface every source architecture must expose to the
/// compiler: the embedding table (representation) and a sequential
/// next-token forward (behavior). The compiler is written against this
/// trait and CANNOT touch anything else — the architecture-generality
/// claim (PROOF.md P4) is enforced by construction, not by inspection.
/// A qwen- or phi-class source implements this trait and nothing
/// downstream changes.
pub trait TeacherOracle {
    fn vocab(&self) -> usize;
    fn dim(&self) -> usize;
    fn seq_len(&self) -> usize;
    /// κ of the source artifact this oracle wraps.
    fn kappa(&self) -> String;
    /// Size in bytes of the source artifact (compression accounting).
    fn source_bytes(&self) -> usize;
    /// Copy the embedding row of `token` into `out` (len == dim).
    fn embedding(&self, token: usize, out: &mut [f32]);
    /// Run one sequential forward step; write logits (len == vocab).
    /// Positions must be fed in order from 0 within one session; call
    /// `reset` to start a new sequence.
    fn reset(&mut self);
    fn step(&mut self, token: usize, pos: usize, logits: &mut [f32]);
}

/// The llama-family adapter: `Llama` plus its recurrent state.
pub struct LlamaOracle {
    pub model: Llama,
    state: State,
    kappa: String,
    source_bytes: usize,
}

impl LlamaOracle {
    pub fn load(path: &str) -> Self {
        let bytes = std::fs::read(path).expect("source checkpoint");
        let kappa = format!("blake3:{}", blake3::hash(&bytes).to_hex());
        let source_bytes = bytes.len();
        let model = Llama::load(path);
        let state = State::new(&model.cfg);
        LlamaOracle { model, state, kappa, source_bytes }
    }
}

impl TeacherOracle for LlamaOracle {
    fn vocab(&self) -> usize {
        self.model.cfg.vocab
    }
    fn dim(&self) -> usize {
        self.model.cfg.dim
    }
    fn seq_len(&self) -> usize {
        self.model.cfg.seq_len
    }
    fn kappa(&self) -> String {
        self.kappa.clone()
    }
    fn source_bytes(&self) -> usize {
        self.source_bytes
    }
    fn embedding(&self, token: usize, out: &mut [f32]) {
        let d = self.model.cfg.dim;
        out.copy_from_slice(&self.model.w[self.model.emb + token * d..self.model.emb + (token + 1) * d]);
    }
    fn reset(&mut self) {
        self.state = State::new(&self.model.cfg);
    }
    fn step(&mut self, token: usize, pos: usize, logits: &mut [f32]) {
        self.model.forward(&mut self.state, token, pos, 1);
        logits.copy_from_slice(&self.state.logits);
    }
}
