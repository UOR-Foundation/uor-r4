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

use super::teacher::TeacherOracle;
use std::collections::HashMap;
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
/// recs: per token: story u32 | next u32 | top_tokens [u32; 3] | top_weights [u32; 3]
pub struct Corpus {
    pub n: usize,
    pub stories: u64,
    pub story: Vec<u32>,
    pub input: Vec<u32>,
    pub next: Vec<u32>,
    pub t_argmax: Vec<u32>,
    pub top_tokens: Vec<[u32; 3]>,
    pub top_weights: Vec<[u32; 3]>,
}

pub fn corpus_paths() -> (&'static str, &'static str) {
    ("/tmp/c_meta.bin", "/tmp/c_recs.bin")
}

pub fn load_corpus() -> Option<Corpus> {
    let (mp, rp) = corpus_paths();
    load_corpus_from(mp, rp)
}

/// Load a corpus record stream from explicit paths (fixtures, mirrors).
pub fn load_corpus_from(mp: &str, rp: &str) -> Option<Corpus> {
    let meta = std::fs::read(mp).ok()?;
    if meta.len() != 25 || meta[24] != 1 {
        return None;
    }
    let n = u64::from_le_bytes(meta[0..8].try_into().unwrap()) as usize;
    let stories = u64::from_le_bytes(meta[8..16].try_into().unwrap());
    let rb = std::fs::read(rp).ok()?;
    let is_legacy = rb.len() == n * 12;
    if rb.len() != n * 32 && !is_legacy {
        return None;
    }
    let mut story = Vec::with_capacity(n);
    let mut next = Vec::with_capacity(n);
    let mut t_argmax = Vec::with_capacity(n);
    let mut top_tokens = Vec::with_capacity(n);
    let mut top_weights = Vec::with_capacity(n);
    for i in 0..n {
        if is_legacy {
            let o = i * 12;
            story.push(u32::from_le_bytes(rb[o..o + 4].try_into().unwrap()));
            let nxt = u16::from_le_bytes(rb[o + 4..o + 6].try_into().unwrap()) as u32;
            next.push(nxt);
            let argmax = u16::from_le_bytes(rb[o + 6..o + 8].try_into().unwrap()) as u32;
            t_argmax.push(argmax);
            let lp = f32::from_le_bytes(rb[o + 8..o + 12].try_into().unwrap());
            let next_prob = (lp.exp() * 100.0).clamp(0.0, 100.0) as u32;

            let mut tokens_val = [0u32; 3];
            let mut weights_val = [0u32; 3];
            tokens_val[0] = argmax;
            if nxt == argmax {
                weights_val[0] = next_prob.max(50);
            } else {
                weights_val[0] = (100 - next_prob).min(90);
                tokens_val[1] = nxt;
                weights_val[1] = next_prob;
            }
            top_tokens.push(tokens_val);
            top_weights.push(weights_val);
        } else {
            let o = i * 32;
            story.push(u32::from_le_bytes(rb[o..o + 4].try_into().unwrap()));
            let nxt = u32::from_le_bytes(rb[o + 4..o + 8].try_into().unwrap());
            next.push(nxt);
            let mut tokens_val = [0u32; 3];
            let mut weights_val = [0u32; 3];
            for j in 0..3 {
                let offset_tok = o + 8 + j * 4;
                let offset_wt = o + 20 + j * 4;
                tokens_val[j] =
                    u32::from_le_bytes(rb[offset_tok..offset_tok + 4].try_into().unwrap());
                weights_val[j] =
                    u32::from_le_bytes(rb[offset_wt..offset_wt + 4].try_into().unwrap());
            }
            t_argmax.push(tokens_val[0]);
            top_tokens.push(tokens_val);
            top_weights.push(weights_val);
        }
    }
    let mut input = Vec::with_capacity(n);
    for i in 0..n {
        if i == 0 || story[i] != story[i - 1] {
            input.push(1u32);
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
        top_tokens,
        top_weights,
    })
}

/// Generate (or extend, resumably) the teacher-labeled corpus. Whole-story
/// chunking keeps the stream deterministic under any budget chunking.
pub fn generate(oracle: &mut dyn TeacherOracle, budget_s: u64, target: usize) {
    let (mp, rp) = corpus_paths();
    generate_to(oracle, budget_s, target, mp, rp);
}

/// Generate a resumable teacher corpus at explicit paths.
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
    let starting_tokens = n;
    let vocab = oracle.vocab();
    let seq_len = oracle.seq_len();
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

            // Find top-3 tokens and their normalized weights
            let mut top_candidates: Vec<(usize, f32)> =
                logits.iter().enumerate().map(|(i, &p)| (i, p)).collect();
            top_candidates
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            let mut top_tokens_idx = [0u32; 3];
            let mut top_weights_val = [0u32; 3];

            let mut sum_top3 = 0.0f32;
            for i in 0..3 {
                if i < top_candidates.len() {
                    sum_top3 += top_candidates[i].1;
                }
            }
            if sum_top3 > 1e-9 {
                let mut accumulated = 0;
                for i in 0..3 {
                    if i < top_candidates.len() {
                        top_tokens_idx[i] = top_candidates[i].0 as u32;
                        let w = ((top_candidates[i].1 / sum_top3) * 100.0).round() as u32;
                        top_weights_val[i] = w;
                        accumulated += w;
                    }
                }
                if accumulated != 100 && top_weights_val[0] > 0 {
                    let diff = 100 - accumulated;
                    top_weights_val[0] = (top_weights_val[0] as i32 + diff as i32).max(0) as u32;
                }
            }

            // Sample the next token using the full logits cdf
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

            let mut record = [0u8; 32];
            record[0..4].copy_from_slice(&(stories as u32).to_le_bytes());
            record[4..8].copy_from_slice(&(next as u32).to_le_bytes());
            for i in 0..3 {
                let offset_tok = 8 + i * 4;
                let offset_wt = 20 + i * 4;
                record[offset_tok..offset_tok + 4]
                    .copy_from_slice(&top_tokens_idx[i].to_le_bytes());
                record[offset_wt..offset_wt + 4].copy_from_slice(&top_weights_val[i].to_le_bytes());
            }

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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RegionHammingCalibration {
    pub stage: u8,
    pub class: u16,
    pub mask_bits: u16,
    pub sample_count: u32,
    pub acceptance_radius: u16,
    pub hamming_histogram: Vec<u32>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HammingCalibrationReport {
    pub signature_bits: u16,
    pub quantile_numerator: u16,
    pub quantile_denominator: u16,
    pub regions: Vec<RegionHammingCalibration>,
}

fn quantile_radius(histogram: &[u32], numerator: u32, denominator: u32) -> u16 {
    let total: u64 = histogram.iter().map(|&count| u64::from(count)).sum();
    if total == 0 {
        return 0;
    }
    if denominator == 0 {
        return histogram.len().saturating_sub(1) as u16;
    }
    let numerator = u64::from(numerator);
    let denominator = u64::from(denominator);
    let target = total
        .saturating_mul(numerator)
        .saturating_add(denominator.saturating_sub(1))
        / denominator;
    let mut cumulative = 0u64;
    for (distance, &count) in histogram.iter().enumerate() {
        cumulative = cumulative.saturating_add(u64::from(count));
        if cumulative >= target {
            return distance as u16;
        }
    }
    histogram.len().saturating_sub(1) as u16
}

pub fn calibrate_hamming_regions_from_signatures(
    class_sigs: &[Vec<u8>],
    signatures: &[[u8; SIG_BYTES]],
) -> HammingCalibrationReport {
    const NUMERATOR: u32 = 95;
    const DENOMINATOR: u32 = 100;
    let mut histograms = vec![vec![vec![0u32; D + 1]; K]; STAGES];
    for sig in signatures {
        let mut words = [0u64; SIG_WORDS];
        for (word, chunk) in words.iter_mut().zip(sig.chunks(8)) {
            let mut bytes = [0u8; 8];
            bytes[..chunk.len()].copy_from_slice(chunk);
            *word = u64::from_le_bytes(bytes);
        }
        for (stage, stage_histograms) in histograms.iter_mut().enumerate().take(STAGES) {
            let Some(stage_sigs) = class_sigs.get(stage) else {
                continue;
            };
            if stage_sigs.len() < K * SIG_BYTES {
                continue;
            }
            let mut best_dist = u32::MAX;
            let mut best_class = 0usize;
            for (class, class_sig) in stage_sigs.chunks_exact(SIG_BYTES).enumerate().take(K) {
                let mut dist = 0u32;
                for (&word, chunk) in words.iter().zip(class_sig.chunks(8)) {
                    let mut bytes = [0u8; 8];
                    bytes[..chunk.len()].copy_from_slice(chunk);
                    dist += (word ^ u64::from_le_bytes(bytes)).count_ones();
                }
                if dist < best_dist {
                    best_dist = dist;
                    best_class = class;
                }
            }
            if best_dist <= D as u32 {
                stage_histograms[best_class][best_dist as usize] += 1;
            }
        }
    }
    let mut regions = Vec::with_capacity(STAGES * K);
    for (stage, stage_histograms) in histograms.iter().enumerate() {
        for (class, histogram) in stage_histograms.iter().enumerate() {
            let sample_count = histogram.iter().sum();
            regions.push(RegionHammingCalibration {
                stage: stage as u8,
                class: class as u16,
                mask_bits: D as u16,
                sample_count,
                acceptance_radius: quantile_radius(histogram, NUMERATOR, DENOMINATOR),
                hamming_histogram: histogram.clone(),
            });
        }
    }
    HammingCalibrationReport {
        signature_bits: D as u16,
        quantile_numerator: NUMERATOR as u16,
        quantile_denominator: DENOMINATOR as u16,
        regions,
    }
}

pub fn calibrate_hamming_regions(art: &Compiled, corpus: &Corpus) -> HammingCalibrationReport {
    let cut = train_cut(corpus);
    let held_out: Vec<usize> = (0..corpus.n).filter(|&i| corpus.story[i] >= cut).collect();
    let indices: Vec<usize> = if held_out.is_empty() {
        (0..corpus.n).collect()
    } else {
        held_out
    };
    let rot = derive_rotations();
    let mut signatures = Vec::with_capacity(indices.len());
    for i in indices {
        let bundle = super::runtime::bundle_plain(art, &rot, corpus, i);
        signatures.push(super::runtime::sig_plain(art, &bundle));
    }
    calibrate_hamming_regions_from_signatures(&art.class_sigs, &signatures)
}

fn hash_index(seed: &[u8; 32], dim: usize, key: &str) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(seed);
    hasher.update(&dim.to_le_bytes());
    hasher.update(key.as_bytes());
    let digest = hasher.finalize();
    let bytes = digest.as_bytes();
    u64::from_le_bytes(bytes[0..8].try_into().unwrap())
}

fn deterministic_bucket(seed: &[u8; 32], h: usize, target_dim: usize) -> usize {
    (hash_index(seed, h, "bucket") as usize) % target_dim
}

fn deterministic_sign(seed: &[u8; 32], h: usize) -> f32 {
    if hash_index(seed, h, "sign").is_multiple_of(2) {
        1.0
    } else {
        -1.0
    }
}

pub fn deterministic_project(
    seed_bytes: &[u8; 32],
    vocab: usize,
    source_dim: usize,
    target_dim: usize,
    oracle: &dyn TeacherOracle,
) -> Vec<f32> {
    let mut projected_vecs = vec![0f32; vocab * target_dim];
    let chunk_size = 1000;
    let mut raw_chunk = vec![0f32; chunk_size * source_dim];

    let mut sum_vec = vec![0f64; source_dim];
    for chunk_start in (0..vocab).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(vocab);
        let count = chunk_end - chunk_start;
        oracle
            .read_embedding_rows(chunk_start..chunk_end, &mut raw_chunk[..count * source_dim])
            .unwrap();
        for i in 0..count {
            for j in 0..source_dim {
                sum_vec[j] += raw_chunk[i * source_dim + j] as f64;
            }
        }
    }
    let mut mean = vec![0f32; source_dim];
    for j in 0..source_dim {
        mean[j] = (sum_vec[j] / vocab as f64) as f32;
    }

    let mut progress = super::progress::Progress::new("projecting embeddings", vocab);
    for chunk_start in (0..vocab).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(vocab);
        let count = chunk_end - chunk_start;
        oracle
            .read_embedding_rows(chunk_start..chunk_end, &mut raw_chunk[..count * source_dim])
            .unwrap();

        for i in 0..count {
            let t = chunk_start + i;
            progress.set(t);
            let mut projected_row = vec![0f32; target_dim];

            for h in 0..source_dim {
                let centered_val = raw_chunk[i * source_dim + h] - mean[h];
                let target = deterministic_bucket(seed_bytes, h, target_dim);
                let sign = deterministic_sign(seed_bytes, h);
                projected_row[target] += sign * centered_val;
            }

            let n = projected_row
                .iter()
                .map(|x| x * x)
                .sum::<f32>()
                .sqrt()
                .max(1e-9);
            for x in projected_row.iter_mut() {
                *x /= n;
            }

            projected_vecs[t * target_dim..(t + 1) * target_dim].copy_from_slice(&projected_row);
        }
    }
    progress.finish();
    projected_vecs
}

fn sampled_kmeans_rvq(
    vecs: &[f32],
    nvec: usize,
    stages: usize,
    k: usize,
    iters: usize,
    seed_bytes: &[u8; 32],
) -> (Vec<Vec<f32>>, Vec<u8>) {
    let mut rng_seed = 0xDECAFBADu64;
    for &b in seed_bytes.iter().take(8) {
        rng_seed = rng_seed.wrapping_add(b as u64).rotate_left(8);
    }

    let mut indices: Vec<usize> = (0..nvec).collect();
    for i in (1..nvec).rev() {
        let j = (xorshift(&mut rng_seed) as usize) % (i + 1);
        indices.swap(i, j);
    }

    let sample_size = nvec.min(10000);
    let mut sample_vecs = vec![0f32; sample_size * D];
    for i in 0..sample_size {
        let src_idx = indices[i];
        sample_vecs[i * D..(i + 1) * D].copy_from_slice(&vecs[src_idx * D..(src_idx + 1) * D]);
    }

    let mut residual = vecs.to_vec();
    let mut sample_residual = sample_vecs.to_vec();
    let mut codebooks: Vec<Vec<f32>> = Vec::new();
    let mut codes = vec![0u8; nvec * stages];

    for stage in 0..stages {
        eprintln!("sampled RVQ stage {}/{}", stage + 1, stages);
        let mut progress = super::progress::Progress::new("RVQ assignment", iters * sample_size);
        let mut cent = vec![0f32; k * D];
        let mut cent_seed = 0xC0DEB00C ^ stage as u64 ^ rng_seed;
        let mut used = vec![false; sample_size];
        for kk in 0..k {
            let mut idx = (xorshift(&mut cent_seed) as usize) % sample_size;
            while used[idx] {
                idx = (idx + 1) % sample_size;
            }
            used[idx] = true;
            cent[kk * D..(kk + 1) * D].copy_from_slice(&sample_residual[idx * D..(idx + 1) * D]);
        }
        let mut assign = vec![0usize; sample_size];
        for iteration in 0..iters {
            for v in 0..sample_size {
                progress.set(iteration * sample_size + v);
                let rv = &sample_residual[v * D..(v + 1) * D];
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
            for v in 0..sample_size {
                cnt[assign[v]] += 1;
                for j in 0..D {
                    sum[assign[v] * D + j] += sample_residual[v * D + j];
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
            codes[v * stages + stage] = bk as u8;
            for j in 0..D {
                residual[v * D + j] -= cent[bk * D + j];
            }
        }

        for v in 0..sample_size {
            let src_idx = indices[v];
            for j in 0..D {
                sample_residual[v * D + j] = residual[src_idx * D + j];
            }
        }

        codebooks.push(cent);
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

pub fn compile(oracle: &dyn TeacherOracle, corpus: &Corpus) -> Compiled {
    let vocab = oracle.vocab_size();
    assert!(
        vocab <= u32::MAX as usize,
        "source vocabulary exceeds u32 token ids"
    );
    println!(
        "source κ: {} ({} bytes)",
        oracle.kappa(),
        oracle.source_bytes()
    );

    let seed_string = format!(
        "{}{}{}",
        oracle.kappa(),
        oracle.tokenizer_address(),
        "r4-geometric-projection-v1"
    );
    let seed_hash = blake3::hash(seed_string.as_bytes());
    let seed_bytes = seed_hash.as_bytes();

    let source_dim = oracle.source_dimension();
    let vecs = deterministic_project(seed_bytes, vocab, source_dim, D, oracle);

    let (emb_cb, emb_codes) = sampled_kmeans_rvq(&vecs, vocab, STAGES, K, EMB_ITERS, seed_bytes);
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
    let (ctx_cb, _) = sampled_kmeans_rvq(&samp, CTX_SAMPLE, STAGES, K, CTX_ITERS, seed_bytes);
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

pub fn load_artifacts() -> Option<Compiled> {
    load_artifacts_from(ART_PATH)
}

/// Load artifacts from an explicit path (fixtures, mirrors).
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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct HierarchicalCodes {
    pub token_type_prefixes: HashMap<String, Vec<u8>>,
    pub relational_prefixes: Vec<Vec<u32>>,
}

pub fn induce_hierarchical_codes(
    token_codes: &[u8],
    vocab: usize,
    corpus: &Corpus,
) -> HierarchicalCodes {
    let mut token_type_prefixes = HashMap::new();
    for token_id in 0..vocab {
        let offset = token_id * STAGES;
        if offset + STAGES <= token_codes.len() {
            let prefix = token_codes[offset..offset + STAGES].to_vec();
            token_type_prefixes.insert(token_id.to_string(), prefix);
        }
    }

    let mut transition_counts = HashMap::new();
    for i in 0..corpus.n {
        let story_id = corpus.story[i];
        let token = corpus.next[i];

        if i + 1 < corpus.n && corpus.story[i + 1] == story_id {
            let next_tok = corpus.next[i + 1];
            let pair = vec![token, next_tok];
            *transition_counts.entry(pair).or_insert(0) += 1;

            if i + 2 < corpus.n && corpus.story[i + 2] == story_id {
                let next_next_tok = corpus.next[i + 2];
                let triplet = vec![token, next_tok, next_next_tok];
                *transition_counts.entry(triplet).or_insert(0) += 1;
            }
        }
    }

    let mut frequent_paths: Vec<(Vec<u32>, usize)> = transition_counts
        .into_iter()
        .filter(|(_, count)| *count >= 5)
        .collect();

    frequent_paths.sort_by_key(|entry| std::cmp::Reverse(entry.1));

    let relational_prefixes = frequent_paths
        .into_iter()
        .take(100)
        .map(|(path, _)| path)
        .collect();

    HierarchicalCodes {
        token_type_prefixes,
        relational_prefixes,
    }
}
