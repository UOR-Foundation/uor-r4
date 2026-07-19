//! The COMPILER: teacher-time cross-compilation of a transformer source into
//! the table-native target. Multiplication is permitted here — it runs once,
//! offline, and every output is a frozen, blake3-κ-pinned artifact:
//!
//!   1. token codebook: RVQ (4 stages × 256) over the source's embedding
//!      table (mean-centered, normalized) — the "representation" surface;
//!   2. integer token vectors: i8-quantized stage-entry sums per token;
//!   3. context thresholds: per-dimension train means of the dyadic-recency
//!      bundle — the "vector at each bit": bit b of a context signature
//!      records the side of threshold b, a prefix of bits an intersection
//!      of regions;
//!   4. class signatures: context-RVQ centroids binarized to sign bits, so
//!      runtime assignment is Hamming (xor + popcount table + add);
//!   5. the datastore: graded context classes → next-token counts, built by
//!      running the corpus that the TEACHER generated and labeled (the
//!      "behavior" surface). The teacher never appears at runtime.
//!
//! The source-architecture interface is exactly two surfaces — an embedding
//! table and a next-token oracle — which is what makes the compilation
//! architecture-generic (llama / qwen / phi differ only in the teacher
//! adapter). This crate instantiates the llama-family adapter.

#[cfg(not(target_arch = "wasm32"))]
use super::teacher::TeacherOracle;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;

pub const STAGES: usize = 4;
pub const K: usize = 256;
pub const D: usize = 288;
pub const V: usize = 32000;
pub const WINDOW: usize = 8;
pub const ROT: usize = 17;
const EMB_ITERS: usize = 10;
const CTX_SAMPLE: usize = 6_000;
const CTX_ITERS: usize = 6;

pub fn xorshift(s: &mut u64) -> u64 {
    let mut x = *s;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *s = x;
    x
}

// ---------------------------------------------------------------- corpus --

/// Teacher-labeled corpus record stream (identical wire format to the
/// research pipeline, so existing state is adoptable):
/// meta: n u64 | stories u64 | rng u64 | done u8
/// recs: per token: story u32 | next u16 | teacher_argmax u16 | teacher_lp f32
pub struct Corpus {
    pub n: usize,
    pub stories: u64,
    pub story: Vec<u32>,
    pub input: Vec<u16>,
    pub next: Vec<u16>,
    pub t_argmax: Vec<u16>,
    pub t_lp: Vec<f32>,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn corpus_paths() -> (&'static str, &'static str) {
    ("/tmp/c_meta.bin", "/tmp/c_recs.bin")
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_corpus() -> Option<Corpus> {
    let (mp, rp) = corpus_paths();
    load_corpus_from(mp, rp)
}

/// Load a corpus record stream from explicit paths (fixtures, mirrors).
#[cfg(not(target_arch = "wasm32"))]
pub fn load_corpus_from(mp: &str, rp: &str) -> Option<Corpus> {
    let meta = std::fs::read(mp).ok()?;
    if meta.len() != 25 || meta[24] != 1 {
        return None;
    }
    let n = u64::from_le_bytes(meta[0..8].try_into().unwrap()) as usize;
    let stories = u64::from_le_bytes(meta[8..16].try_into().unwrap());
    let rb = std::fs::read(rp).ok()?;
    if rb.len() != n * 12 {
        return None;
    }
    let mut story = Vec::with_capacity(n);
    let mut next = Vec::with_capacity(n);
    let mut t_argmax = Vec::with_capacity(n);
    let mut t_lp = Vec::with_capacity(n);
    for i in 0..n {
        let o = i * 12;
        story.push(u32::from_le_bytes(rb[o..o + 4].try_into().unwrap()));
        next.push(u16::from_le_bytes(rb[o + 4..o + 6].try_into().unwrap()));
        t_argmax.push(u16::from_le_bytes(rb[o + 6..o + 8].try_into().unwrap()));
        t_lp.push(f32::from_le_bytes(rb[o + 8..o + 12].try_into().unwrap()));
    }
    let mut input = Vec::with_capacity(n);
    for i in 0..n {
        if i == 0 || story[i] != story[i - 1] {
            input.push(1u16);
        } else {
            input.push(next[i - 1]);
        }
    }
    Some(Corpus {
        n,
        stories,
        story,
        input,
        next,
        t_argmax,
        t_lp,
    })
}

/// Generate (or extend, resumably) the teacher-labeled corpus. Whole-story
/// chunking keeps the stream deterministic under any budget chunking.
#[cfg(not(target_arch = "wasm32"))]
pub fn generate(oracle: &mut dyn TeacherOracle, budget_s: u64, target: usize) {
    let (mp, rp) = corpus_paths();
    generate_to(oracle, budget_s, target, mp, rp);
}

/// Generate a resumable teacher corpus at explicit paths.
#[cfg(not(target_arch = "wasm32"))]
pub fn generate_to(
    oracle: &mut dyn TeacherOracle,
    budget_s: u64,
    target: usize,
    mp: &str,
    rp: &str,
) {
    let (mut n, mut stories, mut rng, mut done) = match std::fs::read(mp) {
        Ok(b) if b.len() == 25 => (
            u64::from_le_bytes(b[0..8].try_into().unwrap()),
            u64::from_le_bytes(b[8..16].try_into().unwrap()),
            u64::from_le_bytes(b[16..24].try_into().unwrap()),
            b[24],
        ),
        _ => (0, 0, 0x5EED, 0),
    };
    if (n as usize) < target {
        done = 0;
    }
    if done == 1 {
        println!("corpus already complete: {} tokens", n);
        return;
    }
    let vocab = oracle.vocab();
    let seq_len = oracle.seq_len();
    let starting_tokens = n;
    let mut logits = vec![0f32; vocab];
    let mut progress = super::progress::Progress::new("teacher corpus", target);
    progress.set(n as usize);
    let mut recs = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(rp)
        .unwrap();
    let t0 = std::time::Instant::now();
    while done == 0 && t0.elapsed().as_secs() < budget_s {
        oracle.reset();
        let mut token = oracle.bos_token();
        for pos in 0..seq_len {
            progress.set(n as usize);
            oracle.step(token, pos, &mut logits);
            let mut mx = logits[0];
            for &v in &logits[1..] {
                if v > mx {
                    mx = v;
                }
            }
            let mut sum = 0.0f32;
            for p in &mut logits {
                *p = (*p - mx).exp();
                sum += *p;
            }
            for p in &mut logits {
                *p /= sum;
            }
            let u = (xorshift(&mut rng) >> 40) as f32 / (1u64 << 24) as f32;
            let mut cdf = 0.0f32;
            let mut next = vocab - 1;
            for (i, &p) in logits.iter().enumerate() {
                cdf += p;
                if u < cdf {
                    next = i;
                    break;
                }
            }
            let mut am = 0usize;
            for i in 1..vocab {
                if logits[i] > logits[am] {
                    am = i;
                }
            }
            let mut record = [0u8; 12];
            record[0..4].copy_from_slice(&(stories as u32).to_le_bytes());
            record[4..6].copy_from_slice(&(next as u16).to_le_bytes());
            record[6..8].copy_from_slice(&(am as u16).to_le_bytes());
            record[8..12].copy_from_slice(&logits[next].max(1e-30).ln().to_le_bytes());
            recs.write_all(&record).unwrap();
            n += 1;
            progress.set(n as usize);
            if n as usize >= target {
                done = 1;
                break;
            }
            if next == oracle.eos_token() {
                break;
            }
            token = next;
        }
        stories += 1;
        let mut b = [0u8; 25];
        b[0..8].copy_from_slice(&n.to_le_bytes());
        b[8..16].copy_from_slice(&stories.to_le_bytes());
        b[16..24].copy_from_slice(&rng.to_le_bytes());
        b[24] = done;
        std::fs::write(mp, b).unwrap();
    }
    println!(
        "corpus: {} / {} tokens, {} stories, done={}",
        n, target, stories, done
    );
    let elapsed = t0.elapsed().as_secs_f64();
    let generated = n.saturating_sub(starting_tokens);
    if generated != 0 && elapsed > 0.0 {
        println!(
            "teacher throughput: {generated} tokens in {elapsed:.2}s ({:.1} tokens/s)",
            generated as f64 / elapsed
        );
    }
    if done == 1 {
        progress.finish();
    }
}

// ------------------------------------------------------------- artifacts --

/// Rotation offsets per window slot, derived once from the definition
/// (j·17 mod D). A compiled table: derivation may multiply, the runtime
/// reads it.
pub fn derive_rotations() -> [usize; WINDOW + 1] {
    let mut r = [0usize; WINDOW + 1];
    for (j, slot) in r.iter_mut().enumerate() {
        *slot = (j * ROT) % D;
    }
    r
}

/// Signature geometry, computed compiler-side so the runtime source
/// carries no const arithmetic either.
pub const SIG_BYTES: usize = D / 8;
pub const SIG_WORDS: usize = SIG_BYTES.div_ceil(8);

/// The train/held-out story cut (80/20), computed compiler-side.
pub fn train_cut(c: &Corpus) -> u32 {
    ((c.stories as f64 * 0.8) as u32).max(1)
}

pub const ART_PATH: &str = "/tmp/tless_artifacts.bin";

pub struct Compiled {
    /// COMPRESSED token representation: STAGES code bytes per token [V × STAGES]
    /// plus i8 stage books [STAGES × (K × D)]. The runtime decodes rows on
    /// demand by table reads and adds; the expanded form is never shipped.
    pub token_codes: Vec<u8>,
    pub stage_books: Vec<Vec<i8>>,
    /// per-stage fixed-point shifts: stage k's book holds round(x·2^e_k)
    /// and decode recombines as (b << shifts[k]) with shifts[k] = E − e_k,
    /// E = max e — shift-aligned fixed point, no multiplies at decode.
    pub stage_shifts: Vec<u8>,
    /// per-dimension sign thresholds for the context bundle: [D].
    pub thresholds: Vec<i64>,
    /// binarized context-class signatures per stage: STAGES × [K × D/8 bytes].
    pub class_sigs: Vec<Vec<u8>>,
    /// f32 context centroids — CERTIFIER-side only (the binarization
    /// ablation); not part of the runtime artifact accounting.
    pub ctx_cb: Vec<Vec<f32>>,
    /// κ-labels of the f32 token-codebook stages, recorded at compile time
    /// (the stages themselves quantize into `stage_books`; the labels are
    /// witness metadata and are not serialized into the container).
    pub token_stage_kappas: Vec<String>,
}

#[cfg(not(target_arch = "wasm32"))]
fn kmeans_rvq(
    vecs: &[f32],
    nvec: usize,
    stages: usize,
    k: usize,
    iters: usize,
) -> (Vec<Vec<f32>>, Vec<u8>) {
    let mut residual = vecs.to_vec();
    let mut codebooks: Vec<Vec<f32>> = Vec::new();
    let mut codes = vec![0u8; nvec * stages];
    for stage in 0..stages {
        eprintln!("embedding RVQ stage {}/{}", stage + 1, stages);
        let mut progress = super::progress::Progress::new("RVQ assignment", iters * nvec);
        let mut cent = vec![0f32; k * D];
        let mut seed = 0xC0DEB00C ^ stage as u64;
        let mut used = vec![false; nvec];
        for kk in 0..k {
            let mut idx = (xorshift(&mut seed) as usize) % nvec;
            while used[idx] {
                idx = (idx + 1) % nvec;
            }
            used[idx] = true;
            cent[kk * D..(kk + 1) * D].copy_from_slice(&residual[idx * D..(idx + 1) * D]);
        }
        let mut assign = vec![0usize; nvec];
        for iteration in 0..iters {
            for v in 0..nvec {
                progress.set(iteration * nvec + v);
                let rv = &residual[v * D..(v + 1) * D];
                let (mut bd, mut bk) = (f32::MAX, 0usize);
                for kk in 0..k {
                    let c = &cent[kk * D..(kk + 1) * D];
                    let mut d2 = 0f32;
                    for j in 0..D {
                        let t = rv[j] - c[j];
                        d2 += t * t;
                    }
                    if d2 < bd {
                        bd = d2;
                        bk = kk;
                    }
                }
                assign[v] = bk;
            }
            let mut sum = vec![0f32; k * D];
            let mut cnt = vec![0u32; k];
            for v in 0..nvec {
                cnt[assign[v]] += 1;
                for j in 0..D {
                    sum[assign[v] * D + j] += residual[v * D + j];
                }
            }
            for kk in 0..k {
                if cnt[kk] > 0 {
                    for j in 0..D {
                        cent[kk * D + j] = sum[kk * D + j] / cnt[kk] as f32;
                    }
                }
            }
        }
        progress.finish();
        for v in 0..nvec {
            codes[v * stages + stage] = assign[v] as u8;
            for j in 0..D {
                residual[v * D + j] -= cent[assign[v] * D + j];
            }
        }
        codebooks.push(cent);
    }
    (codebooks, codes)
}

/// Linear-time deterministic RVQ used for larger source vocabularies. Each
/// stage hashes the residual's signed, quantized geometry into 256 buckets,
/// averages each bucket, then removes that centroid before the next stage.
/// This keeps SmolLM2 compilation bounded instead of performing billions of
/// exhaustive centroid-distance evaluations.
#[cfg(not(target_arch = "wasm32"))]
fn hashed_rvq(vecs: &[f32], nvec: usize, stages: usize) -> (Vec<Vec<f32>>, Vec<u8>) {
    let mut residual = vecs.to_vec();
    let mut codebooks = Vec::with_capacity(stages);
    let mut codes = vec![0u8; nvec * stages];
    for stage in 0..stages {
        eprintln!("hashed RVQ stage {}/{}", stage + 1, stages);
        let mut progress = super::progress::Progress::new("hashed RVQ", nvec);
        let mut sums = vec![0.0f32; K * D];
        let mut counts = [0u32; K];
        for vector in 0..nvec {
            progress.set(vector);
            let row = &residual[vector * D..(vector + 1) * D];
            let mut hash = 0xcbf2_9ce4_8422_2325u64 ^ stage as u64;
            for &value in row {
                let quantized = (value * 127.0).round().clamp(-127.0, 127.0) as i8;
                hash ^= quantized as u8 as u64;
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
            let bucket = (hash & 0xff) as usize;
            codes[vector * stages + stage] = bucket as u8;
            counts[bucket] += 1;
            for dimension in 0..D {
                sums[bucket * D + dimension] += row[dimension];
            }
        }
        progress.finish();
        for bucket in 0..K {
            if counts[bucket] != 0 {
                let inverse = 1.0 / counts[bucket] as f32;
                for dimension in 0..D {
                    sums[bucket * D + dimension] *= inverse;
                }
            }
        }
        for vector in 0..nvec {
            let bucket = codes[vector * stages + stage] as usize;
            for dimension in 0..D {
                residual[vector * D + dimension] -= sums[bucket * D + dimension];
            }
        }
        codebooks.push(sums);
    }
    (codebooks, codes)
}

pub fn kappa_of_f32s(v: &[f32]) -> String {
    let mut hasher = blake3::Hasher::new();
    for value in v {
        hasher.update(&value.to_le_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn compile(oracle: &dyn TeacherOracle, corpus: &Corpus) -> Compiled {
    // 1–2: token codebook and integer token vectors, read exclusively
    // through the oracle's embedding surface.
    let vocab = oracle.vocab();
    assert!(
        vocab <= usize::from(u16::MAX) + 1,
        "source vocabulary exceeds u16 token ids"
    );
    assert_eq!(oracle.dim(), D, "compiler geometry vs source dim");
    println!(
        "source κ: {} ({} bytes)",
        oracle.kappa(),
        oracle.source_bytes()
    );
    let mut vecs = vec![0f32; vocab * D];
    let mut embedding_progress = super::progress::Progress::new("extracting embeddings", vocab);
    for t in 0..vocab {
        embedding_progress.set(t);
        oracle.embedding(t, &mut vecs[t * D..(t + 1) * D]);
    }
    embedding_progress.finish();
    let mut mean = vec![0f32; D];
    for v in 0..vocab {
        for j in 0..D {
            mean[j] += vecs[v * D + j];
        }
    }
    mean.iter_mut().for_each(|m| *m /= vocab as f32);
    for v in 0..vocab {
        let row = &mut vecs[v * D..(v + 1) * D];
        for j in 0..D {
            row[j] -= mean[j];
        }
        let n = row.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
        row.iter_mut().for_each(|x| *x /= n);
    }
    let (emb_cb, emb_codes) = if vocab == V {
        kmeans_rvq(&vecs, vocab, STAGES, K, EMB_ITERS)
    } else {
        hashed_rvq(&vecs, vocab, STAGES)
    };
    let emb_stage_kappas: Vec<String> = emb_cb.iter().map(|cb| kappa_of_f32s(cb)).collect();
    for (i, k) in emb_stage_kappas.iter().enumerate() {
        println!("token codebook stage {} κ: {}", i, k);
    }
    let mut stage_books: Vec<Vec<i8>> = Vec::new();
    let mut exps: Vec<i32> = Vec::new();
    for cb in &emb_cb {
        let m = cb.iter().fold(0f32, |a, &x| a.max(x.abs())).max(1e-9);
        let e = (127.0 / m).log2().floor() as i32;
        let scale = (2.0f32).powi(e);
        stage_books.push(
            cb.iter()
                .map(|&x| (x * scale).round().clamp(-127.0, 127.0) as i8)
                .collect(),
        );
        exps.push(e);
    }
    let emax = *exps.iter().max().unwrap();
    let stage_shifts: Vec<u8> = exps.iter().map(|&e| (emax - e) as u8).collect();
    println!(
        "stage fixed-point exponents e_k = {:?}, decode shifts = {:?}",
        exps, stage_shifts
    );
    let token_codes = emb_codes; // [V × STAGES], compressed representation
    for (i, book) in stage_books.iter().enumerate() {
        let bytes: Vec<u8> = book.iter().map(|&b| b as u8).collect();
        println!(
            "stage book {} (i8) κ: blake3:{}",
            i,
            blake3::hash(&bytes).to_hex()
        );
    }
    println!(
        "token codes κ: blake3:{}",
        blake3::hash(&token_codes).to_hex()
    );

    // Partial artifact so the RUNTIME's own bundle path produces the
    // training bundles (store/query identity by construction).
    let mut art = Compiled {
        token_codes,
        stage_books,
        stage_shifts,
        thresholds: Vec::new(),
        class_sigs: Vec::new(),
        ctx_cb: Vec::new(),
        token_stage_kappas: emb_stage_kappas,
    };
    let rot = derive_rotations();

    // 3: thresholds = per-dimension train means of the runtime bundle.
    let cut = train_cut(corpus);
    let mut sums = [0i128; D];
    let mut ntrain = 0i128;
    let mut threshold_progress = super::progress::Progress::new("context thresholds", corpus.n);
    for i in 0..corpus.n {
        threshold_progress.set(i);
        if corpus.story[i] >= cut {
            continue;
        }
        let b = super::runtime::bundle_plain(&art, &rot, corpus, i);
        for (s, &v) in sums.iter_mut().zip(b.iter()) {
            *s += v as i128;
        }
        ntrain += 1;
    }
    threshold_progress.finish();
    art.thresholds = sums.iter().map(|&s| (s / ntrain) as i64).collect();
    {
        let bytes: Vec<u8> = art
            .thresholds
            .iter()
            .flat_map(|t| t.to_le_bytes())
            .collect();
        println!(
            "threshold vector κ: blake3:{}",
            blake3::hash(&bytes).to_hex()
        );
    }

    // 4: context RVQ on (centered, normalized) runtime bundles; binarize
    // centroids to sign bits for Hamming assignment.
    let train_idx: Vec<usize> = (0..corpus.n).filter(|&i| corpus.story[i] < cut).collect();
    let mut s = 0x5A3B1Eu64;
    let mut samp = vec![0f32; CTX_SAMPLE * D];
    let mut context_progress = super::progress::Progress::new("context samples", CTX_SAMPLE);
    for v in 0..CTX_SAMPLE {
        context_progress.set(v);
        let i = train_idx[(xorshift(&mut s) as usize) % train_idx.len()];
        let b = super::runtime::bundle_plain(&art, &rot, corpus, i);
        let mut row = [0f32; D];
        let mut nn = 0f32;
        for d in 0..D {
            let x = (b[d] - art.thresholds[d]) as f32;
            row[d] = x;
            nn += x * x;
        }
        let nn = nn.sqrt().max(1e-9);
        for d in 0..D {
            samp[v * D + d] = row[d] / nn;
        }
    }
    context_progress.finish();
    let (ctx_cb, _) = if vocab == V {
        kmeans_rvq(&samp, CTX_SAMPLE, STAGES, K, CTX_ITERS)
    } else {
        hashed_rvq(&samp, CTX_SAMPLE, STAGES)
    };
    for (st, cb) in ctx_cb.iter().enumerate() {
        println!("context codebook stage {} κ: {}", st, kappa_of_f32s(cb));
        let mut sigs = vec![0u8; K * D / 8];
        for kk in 0..K {
            for d in 0..D {
                if cb[kk * D + d] > 0.0 {
                    sigs[kk * D / 8 + d / 8] |= 1 << (d % 8);
                }
            }
        }
        println!(
            "class signature stage {} κ: blake3:{}",
            st,
            blake3::hash(&sigs).to_hex()
        );
        art.class_sigs.push(sigs);
    }
    art.ctx_cb = ctx_cb;
    art
}

// ------------------------------------------------------- persistence --

/// Flat, versioned serialization of the compiled artifacts (κ-pins are
/// re-derivable from the bytes; the store is rebuilt from the corpus).
pub fn artifact_bytes(a: &Compiled) -> Vec<u8> {
    let vocab = a.token_codes.len() / STAGES;
    let mut b: Vec<u8> = if vocab == V {
        b"TLA3".to_vec()
    } else {
        let mut bytes = b"TLA4".to_vec();
        bytes.extend_from_slice(&(vocab as u32).to_le_bytes());
        bytes
    };
    b.extend_from_slice(&a.stage_shifts);
    b.extend_from_slice(&a.token_codes);
    for book in &a.stage_books {
        b.extend(book.iter().map(|&x| x as u8));
    }
    for t in &a.thresholds {
        b.extend_from_slice(&t.to_le_bytes());
    }
    for s in &a.class_sigs {
        b.extend_from_slice(s);
    }
    for cb in &a.ctx_cb {
        for f in cb {
            b.extend_from_slice(&f.to_le_bytes());
        }
    }
    b
}

/// κ-label of the artifact container bytes.
pub fn artifact_kappa(a: &Compiled) -> String {
    format!("blake3:{}", blake3::hash(&artifact_bytes(a)).to_hex())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_artifacts(a: &Compiled) {
    let b = artifact_bytes(a);
    std::fs::write(ART_PATH, &b).unwrap();
    println!(
        "artifacts saved: {} ({} bytes, κ blake3:{})",
        ART_PATH,
        b.len(),
        blake3::hash(&b).to_hex()
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_artifacts() -> Option<Compiled> {
    load_artifacts_from(ART_PATH)
}

/// Load artifacts from an explicit path (fixtures, mirrors).
#[cfg(not(target_arch = "wasm32"))]
pub fn load_artifacts_from(path: &str) -> Option<Compiled> {
    let b = std::fs::read(path).ok()?;
    let art = parse_artifacts(&b)?;
    println!("artifact file κ blake3:{}", blake3::hash(&b).to_hex());
    Some(art)
}

/// Parse a TLA3 or vocabulary-sized TLA4 container from bytes.
pub fn parse_artifacts(b: &[u8]) -> Option<Compiled> {
    let (vocab, mut o) = match b.get(..4)? {
        b"TLA3" => (V, 4usize),
        b"TLA4" => (
            u32::from_le_bytes(b.get(4..8)?.try_into().ok()?) as usize,
            8usize,
        ),
        _ => return None,
    };
    let tc = vocab.checked_mul(STAGES)?;
    let bk = STAGES * K * D;
    let th = D * 8;
    let cs = STAGES * K * D / 8;
    let cc = STAGES * K * D * 4;
    if b.len() != o + STAGES + tc + bk + th + cs + cc {
        return None;
    }
    let stage_shifts = b[o..o + STAGES].to_vec();
    o += STAGES;
    let token_codes = b[o..o + tc].to_vec();
    o += tc;
    let mut stage_books = Vec::new();
    for _ in 0..STAGES {
        stage_books.push(
            b[o..o + K * D]
                .iter()
                .map(|&x| x as i8)
                .collect::<Vec<i8>>(),
        );
        o += K * D;
    }
    let mut thresholds = vec![0i64; D];
    for t in thresholds.iter_mut() {
        *t = i64::from_le_bytes(b[o..o + 8].try_into().unwrap());
        o += 8;
    }
    let mut class_sigs = Vec::new();
    for _ in 0..STAGES {
        class_sigs.push(b[o..o + K * D / 8].to_vec());
        o += K * D / 8;
    }
    let mut ctx_cb = Vec::new();
    for _ in 0..STAGES {
        let mut cb = vec![0f32; K * D];
        for f in cb.iter_mut() {
            *f = f32::from_le_bytes(b[o..o + 4].try_into().unwrap());
            o += 4;
        }
        ctx_cb.push(cb);
    }
    Some(Compiled {
        token_codes,
        stage_books,
        stage_shifts,
        thresholds,
        class_sigs,
        ctx_cb,
        token_stage_kappas: Vec::new(),
    })
}
