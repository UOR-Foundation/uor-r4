//! Per-mechanism ablation BPB sweep for the discrete geometric prose artifact (Card P7).
//!
//! Measures the held-out contribution of each mechanism that contributes to the
//! discrete scorer by disabling exactly one of them at a time in an in-memory copy
//! of the artifact and re-evaluating. No training, no fitting, no new draw: this is
//! a diagnostic, and the held-out split it uses is already-open development evidence.
//!
//! # What is measured
//!
//! Held-out **bits-per-byte under the discrete model's own additive scorer**
//! (`ExportedGeometricModel::score_context_candidate_with_vsa`), normalised over the
//! **full vocabulary** with the serving sampler's scaling
//! `exp((s - max) / (temperature * 8192))` at `temperature = 1.0`.
//!
//! This is deliberately *not* `JepaTrainer::evaluate_bpb_with_engram`, which:
//!   * runs on the continuous `f64` trainer, not on the quantised artifact that serves;
//!   * interpolates a second, different probability model (`root_prob * leaf_prob`)
//!     with engram probabilities at lambda 0.85..0.30.
//! The two are different scorers. Every number this tool prints is reproducible from
//! the artifact bytes alone; the training-time number is not.
//!
//! # Exactness
//!
//! Full-vocabulary normalisation (vocab is 4096) rather than the 64-candidate serving
//! shortlist, so the value is an upper bound on what the served path can achieve and is
//! free of shortlist-recall confounds. Ablation deltas are **paired** on identical
//! positions, so their bootstrap CI is far tighter than the absolute BPB uncertainty.
//!
//! # Contract note
//!
//! This is an offline evaluation harness and is deliberately outside the frozen
//! integer-kernel marker region. It uses floating point for the softmax and the
//! bootstrap; the served path it evaluates does not.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use uor_r4_core::native_geometric::learner::ExportedGeometricModel;
use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;
use uor_r4_core::native_geometric::vsa::{encode_attended_multiscale_context, Codebook};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const DEFAULT_MODEL: &str = "native_geometric_prose_model.rgm";
const DEFAULT_CORPUS: &str = "tinystories_train.u16";
const DEFAULT_TOKENIZER: &str = ".uor-models/research/issue-1014/export/tokenizer.json";
const DEFAULT_SEQ_LEN: usize = 64;
const DEFAULT_POSITIONS: usize = 8_192;
/// Serving sampler divisor: `(s - max) / (temperature * 8192)`.
const SAMPLER_SCALE: f64 = 8192.0;
const BOOTSTRAP_RESAMPLES: usize = 1_000;

/// Every ablation this tool understands, in report order.
const ABLATIONS: [&str; 10] = [
    "none",
    "vsa",
    "engram",
    "lattice",
    "lattice_coarse",
    "lattice_fine",
    "jepa",
    "lanes",
    "s2_readout",
    "bias",
];

struct Args {
    model: PathBuf,
    corpus: PathBuf,
    tokenizer: PathBuf,
    seq_len: usize,
    positions: usize,
    ablations: Vec<String>,
}

fn usage() -> String {
    format!(
        "ablate-prose -- per-mechanism ablation BPB sweep (Card P7)\n\
         \n\
         USAGE:\n  ablate-prose [OPTIONS]\n\
         \n\
         OPTIONS:\n\
         \x20 --model <path>      artifact (.rgm or .json)   [default: {DEFAULT_MODEL}]\n\
         \x20 --corpus <path>     .u16 token stream          [default: {DEFAULT_CORPUS}]\n\
         \x20 --tokenizer <path>  tokenizer.json             [default: {DEFAULT_TOKENIZER}]\n\
         \x20 --seq-len <n>       teacher-forced chunk size  [default: {DEFAULT_SEQ_LEN}]\n\
         \x20 --positions <n>     held-out positions to score [default: {DEFAULT_POSITIONS}]\n\
         \x20 --ablations <list>  comma-separated subset; default all\n\
         \x20 --help\n\
         \n\
         ABLATIONS: {}\n",
        ABLATIONS.join(", ")
    )
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        model: PathBuf::from(DEFAULT_MODEL),
        corpus: PathBuf::from(DEFAULT_CORPUS),
        tokenizer: PathBuf::from(DEFAULT_TOKENIZER),
        seq_len: DEFAULT_SEQ_LEN,
        positions: DEFAULT_POSITIONS,
        ablations: ABLATIONS.iter().map(|s| s.to_string()).collect(),
    };
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        let next = |i: usize| -> Result<&String, String> {
            argv.get(i + 1)
                .ok_or_else(|| format!("missing value for {}", argv[i]))
        };
        match argv[i].as_str() {
            "--help" | "-h" => return Err(usage()),
            "--model" => {
                args.model = PathBuf::from(next(i)?);
                i += 2;
            }
            "--corpus" => {
                args.corpus = PathBuf::from(next(i)?);
                i += 2;
            }
            "--tokenizer" => {
                args.tokenizer = PathBuf::from(next(i)?);
                i += 2;
            }
            "--seq-len" => {
                args.seq_len = next(i)?.parse().map_err(|_| "bad --seq-len".to_string())?;
                i += 2;
            }
            "--positions" => {
                args.positions = next(i)?
                    .parse()
                    .map_err(|_| "bad --positions".to_string())?;
                i += 2;
            }
            "--ablations" => {
                args.ablations = next(i)?
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                i += 2;
            }
            other => return Err(format!("unknown argument: {other}\n\n{}", usage())),
        }
    }
    if args.seq_len < 2 {
        return Err("--seq-len must be >= 2 (teacher forcing needs a predecessor)".into());
    }
    if args.positions == 0 {
        return Err("--positions must be >= 1".into());
    }
    for name in &args.ablations {
        if !ABLATIONS.contains(&name.as_str()) {
            return Err(format!(
                "unknown ablation '{name}'; known: {}",
                ABLATIONS.join(", ")
            ));
        }
    }
    Ok(args)
}

fn load_model(path: &PathBuf) -> Result<ExportedGeometricModel, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    if bytes.len() >= 4 && &bytes[..4] == b"RGM1" {
        ExportedGeometricModel::from_binary(&bytes)
            .map_err(|e| format!("binary artifact {}: {e:?}", path.display()))
    } else {
        serde_json::from_slice::<ExportedGeometricModel>(&bytes)
            .map_err(|e| format!("json artifact {}: {e}", path.display()))
    }
}

/// Disable exactly one mechanism in place. Unknown names are rejected by `parse_args`.
fn apply_ablation(model: &mut ExportedGeometricModel, name: &str) -> Result<(), String> {
    match name {
        "none" => {}
        "vsa" => model.vsa_scale_q15 = 0,
        "engram" => model.engram_table = None,
        "lattice" => model.hierarchical_lattice = None,
        "lattice_coarse" => {
            let lattice = model
                .hierarchical_lattice
                .as_mut()
                .ok_or("artifact has no hierarchical lattice to ablate")?;
            lattice.coarse_trigram.fill(0);
        }
        "lattice_fine" => {
            let lattice = model
                .hierarchical_lattice
                .as_mut()
                .ok_or("artifact has no hierarchical lattice to ablate")?;
            lattice.fine_residual.fill(0);
        }
        "jepa" => {
            model.discrete_jepa_w_state = [0; 9];
            model.discrete_jepa_w_token = [0; 9];
            model.discrete_jepa_bias = [0; 3];
            model.discrete_jepa_fiber_w_state = [0; 4];
            model.discrete_jepa_fiber_w_token = [0; 4];
            model.discrete_jepa_fiber_bias = [0; 2];
        }
        "lanes" => {
            for table in &mut model.discrete_tables {
                table.scores.fill(0);
            }
        }
        "s2_readout" => model.discrete_s2_readout.fill([0; 5]),
        "bias" => model.discrete_bias.fill(0),
        other => return Err(format!("unhandled ablation '{other}'")),
    }
    Ok(())
}

/// `(bits, bytes)` contributed by each scored position.
struct Losses {
    bits: Vec<f64>,
    bytes: Vec<u64>,
    scored: usize,
}

impl Losses {
    fn bpb(&self) -> f64 {
        let total_bytes: u64 = self.bytes.iter().sum();
        if total_bytes == 0 {
            return f64::NAN;
        }
        self.bits.iter().sum::<f64>() / total_bytes as f64
    }
}

/// Teacher-forced evaluation of the discrete scorer over full-vocabulary
/// normalisation. Context never crosses a chunk boundary, matching the trainer's
/// `chunks_exact(seq_len)` evaluation.
fn evaluate(
    model: &ExportedGeometricModel,
    codebook: &Codebook<64>,
    tokens: &[u16],
    token_lens: &[u32],
    seq_len: usize,
    max_positions: usize,
    vocab: usize,
) -> Result<Losses, String> {
    let mut bits = Vec::with_capacity(max_positions);
    let mut bytes = Vec::with_capacity(max_positions);
    let mut ctx: Vec<usize> = Vec::with_capacity(seq_len);
    let mut scores: Vec<i32> = vec![0; vocab];

    for chunk in tokens.chunks_exact(seq_len) {
        if bits.len() >= max_positions {
            break;
        }
        ctx.clear();
        for j in 1..seq_len {
            if bits.len() >= max_positions {
                break;
            }
            ctx.push(chunk[j - 1] as usize);
            let target = chunk[j] as usize;
            if target >= vocab {
                continue;
            }

            let fiber = model.context_predicted_fiber_q30(&ctx);
            let mut buf = [0u32; 64];
            let n = ctx.len().min(64);
            for i in 0..n {
                buf[i] = ctx[ctx.len() - n + i] as u32;
            }
            let ctx_vsa = encode_attended_multiscale_context(&buf[..n], codebook, 64);

            // Full-vocabulary scores under the serving scorer, one pass.
            let mut max_score = i32::MIN;
            for cand in 0..vocab {
                let s = model.score_context_candidate_with_vsa(
                    &ctx,
                    cand,
                    fiber,
                    Some((codebook, &ctx_vsa)),
                );
                scores[cand] = s;
                if s > max_score {
                    max_score = s;
                }
            }

            // Serving sampler form: exp((s - max) / (temperature * 8192)), clamped.
            let mut sum = 0.0f64;
            let mut target_p = 0.0f64;
            for (cand, &s) in scores.iter().enumerate() {
                let w = libm::exp((((s - max_score) as f64) / SAMPLER_SCALE).clamp(-60.0, 0.0));
                sum += w;
                if cand == target {
                    target_p = w;
                }
            }
            if !(sum > 0.0) || !(target_p > 0.0) {
                return Err(format!(
                    "degenerate distribution at position {} (sum={sum}, target_p={target_p})",
                    bits.len()
                ));
            }
            bits.push(-(target_p / sum).log2());
            bytes.push(token_lens.get(target).copied().unwrap_or(1).max(1) as u64);
        }
    }
    let scored = bits.len();
    Ok(Losses {
        bits,
        bytes,
        scored,
    })
}

/// Paired bootstrap 95% CI for `bpb(ablated) - bpb(baseline)` over positions.
fn bootstrap_delta_ci(base: &Losses, abl: &Losses) -> (f64, f64, f64) {
    let n = base.bits.len().min(abl.bits.len());
    let delta = abl.bpb() - base.bpb();
    if n == 0 {
        return (delta, f64::NAN, f64::NAN);
    }
    // Deterministic xorshift bootstrap: reproducible without an RNG dependency.
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let mut samples = Vec::with_capacity(BOOTSTRAP_RESAMPLES);
    for _ in 0..BOOTSTRAP_RESAMPLES {
        let mut b_bits = 0.0f64;
        let mut b_bytes = 0u64;
        let mut a_bits = 0.0f64;
        let mut a_bytes = 0u64;
        for _ in 0..n {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let idx = (state as usize) % n;
            b_bits += base.bits[idx];
            b_bytes += base.bytes[idx];
            a_bits += abl.bits[idx];
            a_bytes += abl.bytes[idx];
        }
        if b_bytes == 0 || a_bytes == 0 {
            continue;
        }
        samples.push(a_bits / a_bytes as f64 - b_bits / b_bytes as f64);
    }
    if samples.is_empty() {
        return (delta, f64::NAN, f64::NAN);
    }
    samples.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    let lo = samples[samples.len() / 40];
    let hi = samples[samples.len() - 1 - samples.len() / 40];
    (delta, lo, hi)
}

fn run(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let model = load_model(&args.model)?;

    let tokenizer_bytes = std::fs::read(&args.tokenizer)?;
    let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| format!("failed to parse tokenizer {}", args.tokenizer.display()))?;
    let token_lens = tokenizer.token_byte_lengths();

    let reader = MmapCorpusReader::open(&args.corpus)?;
    let corpus = reader.as_slice();
    let vocab = model.vocab_size;
    let codebook = Codebook::<64>::on_demand(vocab, model.vsa_seed);

    // Held-out convention matches `train-native-prose`: the first tokens of the
    // mapped stream are evaluation, the remainder is training. Positions are drawn
    // from the head of that region so the split is identical for every ablation.
    let needed = args
        .positions
        .next_multiple_of(args.seq_len)
        .saturating_add(args.seq_len);
    let holdout = corpus.len().min(needed);
    let held_out = &corpus[..holdout];

    println!("ablate-prose (Card P7) -- per-mechanism ablation BPB sweep");
    println!("  model      : {} (vocab {})", args.model.display(), vocab);
    println!(
        "  corpus     : {} ({} tokens; held-out slice {} tokens)",
        args.corpus.display(),
        corpus.len(),
        held_out.len()
    );
    println!("  scorer     : discrete additive scorer, full-vocabulary softmax at temperature 1.0");
    println!(
        "  seq_len    : {}   positions requested: {}",
        args.seq_len, args.positions
    );
    println!();

    let baseline = evaluate(
        &model,
        &codebook,
        held_out,
        &token_lens,
        args.seq_len,
        args.positions,
        vocab,
    )?;
    println!(
        "baseline (no ablation): BPB = {:.4} over {} positions in {:.1}s",
        baseline.bpb(),
        baseline.scored,
        started.elapsed().as_secs_f64()
    );
    println!();
    println!(
        "{:<16} {:>10} {:>10} {:>24}  {}",
        "ablation", "BPB", "delta", "paired 95% CI", "verdict"
    );

    for name in &args.ablations {
        if name == "none" {
            continue;
        }
        let t = Instant::now();
        let mut ablated = model.clone();
        apply_ablation(&mut ablated, name)?;
        let losses = evaluate(
            &ablated,
            &codebook,
            held_out,
            &token_lens,
            args.seq_len,
            args.positions,
            vocab,
        )?;
        let (delta, lo, hi) = bootstrap_delta_ci(&baseline, &losses);
        let verdict = if lo > 0.0 {
            "CONTRIBUTES (ablation costs BPB)"
        } else if hi < 0.0 {
            "HARMFUL (ablation helps)"
        } else {
            "INERT (CI includes 0)"
        };
        println!(
            "{:<16} {:>10.4} {:>+10.4} {:>24}  {}",
            name,
            losses.bpb(),
            delta,
            format!("[{lo:+.4}, {hi:+.4}]"),
            verdict
        );
        eprintln!("  ({name}: {:.1}s)", t.elapsed().as_secs_f64());
    }

    println!();
    println!("elapsed_total_s: {:.1}", started.elapsed().as_secs_f64());
    println!(
        "A mechanism is RETAINED only if its ablation CI excludes 0 in the positive direction."
    );
    Ok(())
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return if msg.starts_with("ablate-prose --") {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }
    };
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
