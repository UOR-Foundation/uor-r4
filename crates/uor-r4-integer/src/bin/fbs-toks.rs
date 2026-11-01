//! Time a SpiralCore/FBS stack artifact's integer session on this machine
//! (d11-exporter-float-trunk Stage 4b, claude-126).
//!
//! ```text
//! fbs-toks ARTIFACT [--prompt ID,ID,...] [--stages] [--parts]
//! ```
//!
//! The protocol follows the step-6 one (`docs/labs/m1-d11-artifact-scorer`,
//! claude-116), fitted to a session that cannot slide its window: 1 warmup and
//! 20 timed runs, each in a fresh session, of a 128-token prefill (context)
//! and then 64 greedy decode steps, each feeding the token the writer chose.
//! The two together fill a 192-token context. It records the raw durations in
//! integer nanoseconds, the median (the integer mean of the middle two for an
//! even count), the nearest-rank p95 (rank ceil(0.95 n)), integer tokens per
//! second for each, the reply, and the hardware profile. It refuses an
//! artifact whose context is shorter than 192, a prompt of the wrong length,
//! and runs that write different replies.
//!
//! The served path is `IntegerStackSession::step_native`; argument parsing,
//! timing, statistics and JSON are outside it. It prints one JSON line.
//!
//! `--stages` (fbs-cost-profile, claude-158) runs the protocol a second time
//! through `IntegerStackSession::measure_fbs_step_stages` and adds `stages`:
//! each stage's median over the timed decode steps, their sum against the
//! step, a whole-table nearest search's median beside the writer, and the
//! step against the plain protocol's per-token time. The plain protocol's
//! numbers are taken first and unchanged.
//!
//! `--parts` (fbs-serving-kernels, claude-163) implies `--stages` and runs the
//! protocol a third time through `IntegerStackSession::measure_fbs_step_parts`,
//! which splits each layer's two blocks into nine parts. It adds `parts`: the
//! same profile by part, and under `against_blocks` each block's parts summed
//! against that block's median from the `--stages` run, per mille.
//!
//! The Hamming read's trace (integer-hamming-read, claude-235):
//! - `--hamming-trace D11_READOUT_JSON TRACE_JSON` steps a fresh session over each sequence of
//!   the float D11 readout (`fbs-snapshot`'s) and writes what each read chose
//!   (`uor-r4.hamming-trace/1`), as `fbs-snapshot --hamming-trace` writes the float's;
//! - `--hamming-trace-compare FIRST SECOND` holds two traces side by side
//!   (`compare_hamming_traces`) and prints the comparison;
//! - `--copy-codes FLOAT_TRACE INTEGER_TRACE OUT_JSON` (claude-237) compares every vocabulary
//!   entry's copy-head code three ways (`copy_code_check`): the CPU float control's and the
//!   session's on the artifact's own key rotations and writer table, and the two traces' at
//!   each position, matched by token, each trace also against the control with the rotations
//!   snapped to the icosians, as the GPU's D11 forward snaps them. It writes
//!   `uor-r4.copy-codes/1` new to `OUT_JSON`;
//! - `--context-sweep` runs the protocol with prefills of 32, 64, 96 and 128 tokens, each
//!   followed by the 64 decode steps, with the artifact's Hamming read and without it: decode
//!   tokens per second against context, the read's long-context cost;
//! - `--context-sweep --tokens TOKENS_U16` (claude-242) runs the same protocol at prefills of
//!   128, 512, 2,048, 8,192 and 32,704 tokens, over the first 32,704 ids of a `UORT` token
//!   file, so the contexts end at 192 to 32,768. The artifact's context must hold the last.

use std::process::{Command, ExitCode};
use std::time::Instant;

use serde::Deserialize;
use serde_json::json;
use uor_r4_integer::fbs_snapshot::{
    compare_hamming_traces, copy_code_check, copy_code_reads, write_new_file, FbsFloatReadout,
    FbsReadoutMode, HammingTrace, COPY_CODES_SCHEMA, HAMMING_TRACE_SCHEMA,
};
use uor_r4_integer::stack::{
    hamming_trace_window, panel_reply, read_hamming_uorlut01, served_copy_codes_uorlut01,
    IntegerStackModel, ReplyEnd,
};

const USAGE: &str = "usage: fbs-toks ARTIFACT [--prompt ID,ID,...] [--stages] [--parts]\n       fbs-toks ARTIFACT --replies PROMPTS_JSON --out FILE [--reply-tokens N]\n       fbs-toks ARTIFACT --hamming-trace D11_READOUT_JSON TRACE_JSON\n       fbs-toks --hamming-trace-compare FIRST_TRACE SECOND_TRACE\n       fbs-toks ARTIFACT --copy-codes FLOAT_TRACE INTEGER_TRACE OUT_JSON\n       fbs-toks ARTIFACT --context-sweep [--tokens TOKENS_U16]";

/// The prefills `--context-sweep` times, each followed by `DECODE` steps (claude-235).
const SWEEP_PREFILLS: [usize; 4] = [32, 64, 96, 128];

/// The prefills `--context-sweep --tokens` times (claude-242), each followed by `DECODE` steps,
/// so the contexts end at 192, 576, 2,112, 8,256 and 32,768.
const LONG_SWEEP_PREFILLS: [usize; 5] = [128, 512, 2_048, 8_192, 32_704];

/// `--hamming-trace` (claude-235): each read's choices in a fresh session over each sequence of
/// the float D11 readout at `readout`, written new to `out`; it prints the window count and the
/// file's sha256.
fn hamming_trace(
    model: &IntegerStackModel,
    bytes: &[u8],
    readout: &str,
    out: &str,
) -> Result<(), String> {
    let image = read_hamming_uorlut01(bytes)
        .map_err(|e| format!("the Hamming reads: {e}"))?
        .ok_or("the artifact has no Hamming read to trace")?;
    let text = std::fs::read(readout).map_err(|e| format!("{readout}: {e}"))?;
    let readout: FbsFloatReadout =
        serde_json::from_slice(&text).map_err(|e| format!("{readout}: {e}"))?;
    if readout.mode != FbsReadoutMode::D11 {
        return Err(format!(
            "the readout was read in the {:?} mode, not D11",
            readout.mode
        ));
    }
    let mut windows = Vec::with_capacity(readout.sequences.len());
    for (i, sequence) in readout.sequences.iter().enumerate() {
        let mut session = model.session();
        session.set_hamming_trace(true);
        for &token in &sequence.tokens {
            session
                .step_native(token)
                .map_err(|e| format!("sequence {i}: {e}"))?;
        }
        let steps = session.take_hamming_trace().ok_or("the trace was off")?;
        windows.push(
            hamming_trace_window(&sequence.tokens, &steps)
                .map_err(|e| format!("sequence {i}: {e}"))?,
        );
    }
    let trace = HammingTrace {
        schema: HAMMING_TRACE_SCHEMA.to_owned(),
        side: "integer".to_owned(),
        arm: "integer".to_owned(),
        heads: image.record.heads,
        kept: image.record.kept,
        windows,
    };
    let text = serde_json::to_vec(&trace).map_err(|e| e.to_string())?;
    write_new_file(std::path::Path::new(out), &text).map_err(|e| format!("{out}: {e}"))?;
    println!(
        "{}",
        json!({ "windows": trace.windows.len(), "out": out, "sha256": hex::encode(<sha2::Sha256 as sha2::Digest>::digest(&text)) })
    );
    Ok(())
}

/// `--hamming-trace-compare` (claude-235): the comparison of the trace at `first` with the one
/// at `second`, as one JSON line.
fn hamming_trace_compare(first: &str, second: &str) -> Result<(), String> {
    let read = |path: &str| -> Result<HammingTrace, String> {
        let text = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        serde_json::from_slice(&text).map_err(|e| format!("{path}: {e}"))
    };
    let (a, b) = (read(first)?, read(second)?);
    let comparison = compare_hamming_traces(&a, &b)?;
    println!(
        "{}",
        json!({
            "schema": "uor-r4.hamming-trace-compare/1",
            "first": { "path": first, "side": a.side, "arm": a.arm },
            "second": { "path": second, "side": b.side, "arm": b.arm },
            "comparison": comparison,
        })
    );
    Ok(())
}

/// `--copy-codes` (claude-237): every vocabulary entry's copy-head code three ways
/// (`copy_code_check`), on the artifact at `path` and the traces at `float_trace` and
/// `integer_trace`, written new to `out`. Everything is read and checked before anything is
/// compared or written. It prints one JSON line: the file's sha256 and, per read, the codes on
/// which the control and the session agree, the float trace's query codes equal to each and to
/// the snapped control, the integer trace's equal to its session, and each trace's
/// inconsistent pairs.
fn copy_codes(path: &str, float_trace: &str, integer_trace: &str, out: &str) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    IntegerStackModel::parse(&bytes).map_err(|e| format!("parse: {e}"))?;
    let served = served_copy_codes_uorlut01(&bytes)
        .map_err(|e| format!("the Hamming reads: {e}"))?
        .ok_or("the artifact has no Hamming read to check")?;
    let (vocab, width, heads) = (served.vocab, served.width, served.image.record.heads);
    let read_trace = |path: &str| -> Result<(HammingTrace, Vec<u8>), String> {
        let text = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        let trace = serde_json::from_slice(&text).map_err(|e| format!("{path}: {e}"))?;
        Ok((trace, text))
    };
    let (float, float_text) = read_trace(float_trace)?;
    let (integer, integer_text) = read_trace(integer_trace)?;
    let inputs: Vec<(usize, &[i32], Vec<u64>)> = served
        .image
        .reads
        .iter()
        .zip(&served.codes)
        .map(|(read, codes)| (read.depth, read.key_rotation.as_slice(), codes.clone()))
        .collect();
    let rotation_exp = served.image.record.rotation_exp;
    let reads = copy_code_reads(&inputs, rotation_exp, &served.table, vocab, width, heads)?;
    let bytes = bytes.as_slice();
    let check = copy_code_check(vocab, width, heads, &reads, &float, &integer)?;
    let sha = |b: &[u8]| hex::encode(<sha2::Sha256 as sha2::Digest>::digest(b));
    let record = json!({
        "schema": COPY_CODES_SCHEMA,
        "artifact": { "path": path, "sha256": sha(bytes) },
        "float_trace": { "path": float_trace, "sha256": sha(&float_text), "arm": float.arm },
        "integer_trace": { "path": integer_trace, "sha256": sha(&integer_text), "arm": integer.arm },
        "width": width,
        "check": check,
    });
    let text = serde_json::to_vec(&record).map_err(|e| e.to_string())?;
    write_new_file(std::path::Path::new(out), &text).map_err(|e| format!("{out}: {e}"))?;
    let summary: Vec<_> = check
        .reads
        .iter()
        .map(|r| {
            json!({
                "depth": r.depth,
                "codes": r.control_vs_integer.codes,
                "control_equal_integer": r.control_vs_integer.equal,
                "float_query_equal_control": r.float.query.equal_control,
                "float_query_equal_integer": r.float.query.equal_integer,
                "float_query_equal_snapped": r.float.query.equal_snapped,
                "float_query_samples": r.float.query.samples,
                "integer_query_equal_integer": r.integer.query.equal_integer,
                "inconsistent": [r.float.inconsistent, r.integer.inconsistent],
            })
        })
        .collect();
    println!(
        "{}",
        json!({ "out": out, "sha256": sha(&text), "reads": summary })
    );
    Ok(())
}

/// `--context-sweep` (claude-235): the protocol at each of `prefills` over the prompt's first
/// tokens, on `model` and on the same model without its Hamming read.
fn context_sweep(
    model: &IntegerStackModel,
    without: &IntegerStackModel,
    prompt: &[u32],
    prefills: &[usize],
) -> Result<serde_json::Value, String> {
    let mut points = Vec::with_capacity(prefills.len());
    for &prefill in prefills {
        let prompt = &prompt[..prefill];
        points.push(json!({
            "prefill": prefill,
            "context_end": prefill + DECODE,
            "with_read": protocol(|| model.session(), prompt)?,
            "without_read": protocol(|| without.session(), prompt)?,
        }));
    }
    Ok(json!(points))
}

/// The prompt `--context-sweep --tokens` sweeps (claude-242): a `UORT` token file's first ids, as
/// many as the largest of `LONG_SWEEP_PREFILLS`. The file is read as `fbs-export --point-key`
/// reads one: a 64-byte header starting `UORT`, whose count at bytes 8 to 16 must match the
/// payload, then little-endian `u16` ids. It refuses an artifact whose `context` cannot hold that
/// prefill and its decode, a file that is not `UORT` or whose count disagrees, one with too few
/// ids, and an id outside `vocab`.
fn long_sweep_prompt(bytes: &[u8], vocab: usize, context: usize) -> Result<Vec<u32>, String> {
    let largest = LONG_SWEEP_PREFILLS[LONG_SWEEP_PREFILLS.len() - 1];
    if context < largest + DECODE {
        return Err(format!(
            "the artifact's context {context} is shorter than {largest} + {DECODE}"
        ));
    }
    if bytes.len() < 64 || &bytes[..4] != b"UORT" {
        return Err("the token file is not a UORT token file".to_owned());
    }
    let mut count = [0u8; 8];
    count.copy_from_slice(&bytes[8..16]);
    let count = u64::from_le_bytes(count);
    let payload = &bytes[64..];
    if usize::try_from(count).ok().and_then(|c| c.checked_mul(2)) != Some(payload.len()) {
        return Err(format!(
            "the token file's header counts {count} ids, its payload holds {} bytes",
            payload.len()
        ));
    }
    if payload.len() / 2 < largest {
        return Err(format!(
            "the token file holds {} ids, fewer than {largest}",
            payload.len() / 2
        ));
    }
    let prompt: Vec<u32> = payload[..2 * largest]
        .chunks_exact(2)
        .map(|b| u32::from(u16::from_le_bytes([b[0], b[1]])))
        .collect();
    if let Some((i, &t)) = prompt
        .iter()
        .enumerate()
        .find(|(_, &t)| t as usize >= vocab)
    {
        return Err(format!(
            "token {t} at {i} is outside the vocabulary {vocab}"
        ));
    }
    Ok(prompt)
}

/// One panel prompt for `--replies` (claude-224), as `train-in-space --panel-prompts` writes it:
/// the reply's corpus position, the prompt's ids, the reference reply, and optionally the float
/// side's served name at the opening.
#[derive(Deserialize)]
struct PanelPrompt {
    position: usize,
    prompt: Vec<u32>,
    reference: Vec<u32>,
    #[serde(default)]
    float_keyed: Option<u32>,
}

/// `--replies` (point-key-integer unit 2, claude-224): each prompt's greedy reply on this
/// artifact (`panel_reply`), written new to `out` as JSON. Each sample holds the opening's
/// point-key name and least distance at Q12, where the key served, and the float name's least
/// distance. It prints one JSON line: rows, keyed openings, capped replies and the file's sha256.
/// Each keyed opening matched the reference scan (claude-228): `panel_reply` stops the decode on
/// any that does not, so `reference_checked` is the keyed count of a decode that finished.
fn replies(
    model: &IntegerStackModel,
    prompts: &str,
    out: &str,
    reply_tokens: usize,
) -> Result<(), String> {
    let bytes = std::fs::read(prompts).map_err(|e| format!("{prompts}: {e}"))?;
    let rows: Vec<PanelPrompt> =
        serde_json::from_slice(&bytes).map_err(|e| format!("{prompts}: {e}"))?;
    let (mut samples, mut keyed, mut capped) = (Vec::with_capacity(rows.len()), 0usize, 0usize);
    for row in &rows {
        let reply = panel_reply(model, &row.prompt, reply_tokens, row.float_keyed)
            .map_err(|e| format!("position {}: {e}", row.position))?;
        keyed += usize::from(reply.keyed.is_some());
        capped += usize::from(reply.end == ReplyEnd::Context);
        samples.push(json!({
            "position": row.position,
            "prompt": row.prompt,
            "reply": reply.reply,
            "reference": row.reference,
            "keyed": reply.keyed.map(|k| k.0),
            "served_distance_q12": reply.keyed.map(|k| k.1),
            "float_keyed": row.float_keyed,
            "float_distance_q12": reply.float_distance,
            "end": format!("{:?}", reply.end).to_lowercase(),
        }));
    }
    let text = serde_json::to_vec(&samples).map_err(|e| e.to_string())?;
    uor_r4_integer::fbs_snapshot::write_new_file(std::path::Path::new(out), &text)
        .map_err(|e| format!("{out}: {e}"))?;
    println!(
        "{}",
        json!({ "rows": rows.len(), "keyed": keyed, "reference_checked": keyed, "capped": capped, "out": out, "sha256": hex::encode(<sha2::Sha256 as sha2::Digest>::digest(&text)) })
    );
    Ok(())
}
const PREFILL: usize = 128;
const DECODE: usize = 64;
const WARMUP: usize = 1;
const RUNS: usize = 20;

/// One session step: the token fed in, the token the writer chose.
trait Stepper {
    fn step(&mut self, token: u32) -> Result<u32, String>;
}

impl Stepper for uor_r4_integer::stack::IntegerStackSession<'_> {
    fn step(&mut self, token: u32) -> Result<u32, String> {
        self.step_native(token)
            .map(|s| s.token)
            .map_err(|e| e.to_string())
    }
}

/// One timed run in a fresh session: the prefill over `prompt` and `decode`
/// greedy steps after it, as `(prefill_ns, decode_ns, reply)`. The reply is
/// the prefill's last choice followed by each decode step's.
fn timed_run<S: Stepper>(
    mut session: S,
    prompt: &[u32],
    decode: usize,
) -> Result<(u64, u64, Vec<u32>), String> {
    let start = Instant::now();
    let mut next = 0;
    for &t in prompt {
        next = session.step(t)?;
    }
    let prefill = start.elapsed();
    let mut reply = vec![next];
    let start = Instant::now();
    for _ in 0..decode {
        next = session.step(next)?;
        reply.push(next);
    }
    let decode = start.elapsed();
    let ns = |d: std::time::Duration| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX);
    Ok((ns(prefill), ns(decode), reply))
}

/// The median of integer durations: the integer mean of the middle two for an
/// even count.
fn median(sorted: &[u64]) -> u64 {
    let n = sorted.len();
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2
    }
}

/// The nearest-rank p95: the value at rank ceil(0.95 n), counting from 1.
fn p95(sorted: &[u64]) -> u64 {
    let rank = (sorted.len() * 95).div_ceil(100).max(1);
    sorted[rank - 1]
}

/// Integer tokens per second for `tokens` in `ns` nanoseconds.
fn tokens_per_second(tokens: usize, ns: u64) -> u64 {
    (tokens as u128 * 1_000_000_000 / u128::from(ns.max(1))) as u64
}

/// Run the protocol: `WARMUP` untimed runs, then `RUNS` timed ones, every
/// run's reply the same.
fn protocol<S: Stepper>(
    mut fresh: impl FnMut() -> S,
    prompt: &[u32],
) -> Result<serde_json::Value, String> {
    for _ in 0..WARMUP {
        timed_run(fresh(), prompt, DECODE)?;
    }
    let mut prefill = Vec::with_capacity(RUNS);
    let mut decode = Vec::with_capacity(RUNS);
    let mut reply: Option<Vec<u32>> = None;
    for _ in 0..RUNS {
        let (p, d, r) = timed_run(fresh(), prompt, DECODE)?;
        match &reply {
            Some(first) if *first != r => {
                return Err("timed runs wrote different replies".to_owned())
            }
            Some(_) => {}
            None => reply = Some(r),
        }
        prefill.push(p);
        decode.push(d);
    }
    let summary = |raw: &[u64], tokens: usize| {
        let mut sorted = raw.to_vec();
        sorted.sort_unstable();
        let (med, slow) = (median(&sorted), p95(&sorted));
        json!({
            "tokens": tokens,
            "raw_ns": raw,
            "median_ns": med,
            "p95_ns": slow,
            "median_tokens_per_second": tokens_per_second(tokens, med),
            "p95_tokens_per_second": tokens_per_second(tokens, slow),
        })
    };
    Ok(json!({
        "warmup_runs": WARMUP,
        "timed_runs": RUNS,
        "context_prefill": summary(&prefill, prompt.len()),
        "decode": summary(&decode, DECODE),
        "reply": reply.unwrap_or_default(),
    }))
}

/// The two blocks `measure_fbs_step_stages` times in each layer, in its order.
const BLOCKS: [&str; 2] = ["recurrence", "mlp"];
/// The nine parts `measure_fbs_step_parts` times in each layer, in its order;
/// each is named after the block it belongs to.
const PARTS: [&str; 9] = [
    "recurrence.norm",
    "recurrence.gather_in",
    "recurrence.core",
    "recurrence.gather_out",
    "mlp.norm",
    "mlp.gather_in",
    "mlp.product",
    "mlp.requant",
    "mlp.gather_out",
];

/// The protocol's runs again through `measure_fbs_step_stages`
/// (fbs-cost-profile, claude-158), or with `parts` through
/// `measure_fbs_step_parts` (fbs-serving-kernels, claude-163): every timed
/// run's decode steps, each as its stage times, and the reply, which every
/// run must repeat.
fn stage_protocol(
    model: &IntegerStackModel,
    prompt: &[u32],
    parts: bool,
) -> Result<(Vec<Vec<u64>>, Vec<u32>), String> {
    let mut rows = Vec::with_capacity(RUNS * DECODE);
    let mut reply: Option<Vec<u32>> = None;
    let mut ns = Vec::new();
    for run in 0..WARMUP + RUNS {
        let mut session = model.session();
        let mut step = |token: u32, ns: &mut Vec<u64>| -> Result<u32, String> {
            if parts {
                session.measure_fbs_step_parts(token, ns)
            } else {
                session.measure_fbs_step_stages(token, ns)
            }
            .map(|(s, _)| s.token)
            .map_err(|e| e.to_string())
        };
        let mut next = 0;
        for &t in prompt {
            next = step(t, &mut ns)?;
        }
        let mut written = vec![next];
        for _ in 0..DECODE {
            next = step(next, &mut ns)?;
            written.push(next);
            if run >= WARMUP {
                rows.push(ns.clone());
            }
        }
        match &reply {
            Some(first) if *first != written => {
                return Err("stage-timed runs wrote different replies".to_owned())
            }
            Some(_) => {}
            None => reply = Some(written),
        }
    }
    Ok((rows, reply.unwrap_or_default()))
}

/// The stages timed in order for `layers` layers, each layer's named by
/// `per_layer` ([`BLOCKS`] or [`PARTS`]).
fn stage_names(layers: usize, per_layer: &[&str]) -> Vec<String> {
    let mut names = vec!["embed".to_owned()];
    for l in 0..layers {
        names.extend(per_layer.iter().map(|p| format!("layer{l}.{p}")));
    }
    names.extend(["final_norm", "writer", "bookkeeping"].map(str::to_owned));
    names
}

/// The stage profile of the decode steps in `rows`, each layer's stages named
/// by `per_layer`: each stage's median, their sum against the median step (per
/// mille), the whole-table nearest's median, and the step against the plain
/// protocol's per-token decode time. Their difference, `loop_overhead_ns`, is
/// the cost outside the step: the loop, and `step_native`'s copies of the kept
/// cells and the query into its `NativeStep`.
fn stage_report(
    rows: &[Vec<u64>],
    per_layer: &[&str],
    plain_decode_ns: u64,
    plain_reply: &[u32],
    staged_reply: &[u32],
) -> Result<serde_json::Value, String> {
    let width = rows.first().map_or(0, Vec::len);
    if width < 6
        || !(width - 6).is_multiple_of(per_layer.len())
        || rows.iter().any(|r| r.len() != width)
    {
        return Err(format!(
            "stage rows of {width} times do not fit the stage order"
        ));
    }
    let column = |i: usize| {
        let mut values: Vec<u64> = rows.iter().map(|r| r[i]).collect();
        values.sort_unstable();
        median(&values)
    };
    let names = stage_names((width - 6) / per_layer.len(), per_layer);
    let stages: Vec<serde_json::Value> = names
        .iter()
        .enumerate()
        .map(|(i, name)| json!({ "name": name, "median_ns": column(i) }))
        .collect();
    let stage_sum: u64 = (0..names.len()).map(column).sum();
    let (step, nearest) = (column(width - 2), column(width - 1));
    let per_token = plain_decode_ns / DECODE as u64;
    Ok(json!({
        "decode_steps": rows.len(),
        "stages": stages,
        "stage_sum_ns": stage_sum,
        "step_median_ns": step,
        "stage_sum_per_mille_of_step": stage_sum * 1000 / step.max(1),
        "whole_table_nearest_median_ns": nearest,
        "plain_decode_per_token_ns": per_token,
        "step_per_mille_of_plain_token": step * 1000 / per_token.max(1),
        "loop_overhead_ns": per_token as i64 - step as i64,
        "replies_match": plain_reply == staged_reply,
    }))
}

/// Each layer's block against its parts (fbs-serving-kernels, claude-163): for
/// every block in `blocks` (a [`BLOCKS`] report), the medians of its parts in
/// `parts` (a [`PARTS`] report) summed, against the block's own median, per
/// mille, and whether every block lies within 10% (900 to 1,100 per mille).
fn blocks_against_parts(
    blocks: &serde_json::Value,
    parts: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let medians = |report: &serde_json::Value| -> Result<Vec<(String, u64)>, String> {
        report["stages"]
            .as_array()
            .ok_or("a stage report without stages")?
            .iter()
            .map(|s| match (s["name"].as_str(), s["median_ns"].as_u64()) {
                (Some(name), Some(ns)) => Ok((name.to_owned(), ns)),
                _ => Err("a stage without a name or a median".to_owned()),
            })
            .collect()
    };
    let (blocks, parts) = (medians(blocks)?, medians(parts)?);
    let mut rows = Vec::new();
    let mut within = true;
    for (block, block_ns) in blocks.iter().filter(|(name, _)| {
        name.starts_with("layer") && BLOCKS.iter().any(|b| name.ends_with(&format!(".{b}")))
    }) {
        let prefix = format!("{block}.");
        // The block's kind (`recurrence` or `mlp`) names how many parts it has.
        let kind = block.rsplit('.').next().unwrap_or_default();
        let expected = PARTS
            .iter()
            .filter(|p| p.starts_with(&format!("{kind}.")))
            .count();
        let found: Vec<u64> = parts
            .iter()
            .filter(|(name, _)| name.starts_with(&prefix))
            .map(|(_, ns)| *ns)
            .collect();
        if found.len() != expected {
            return Err(format!(
                "{} of {block}'s {expected} parts were timed",
                found.len()
            ));
        }
        let summed: u64 = found.iter().sum();
        let per_mille = summed * 1000 / (*block_ns).max(1);
        within &= (900..=1100).contains(&per_mille);
        rows.push(json!({
            "block": block,
            "block_median_ns": block_ns,
            "parts_sum_ns": summed,
            "per_mille": per_mille,
        }));
    }
    if rows.is_empty() {
        return Err("the block report holds no layer blocks".to_owned());
    }
    Ok(json!({ "blocks": rows, "within_10_percent": within }))
}

/// One line of a command's output, or `None` when it does not run.
fn probe(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

fn hardware() -> serde_json::Value {
    let sysctl = |key: &str| probe("sysctl", &["-n", key]);
    json!({
        "os": probe("sw_vers", &["-productVersion"]),
        "cpu": sysctl("machdep.cpu.brand_string"),
        "performance_cores": sysctl("hw.perflevel0.physicalcpu"),
        "cores": sysctl("hw.ncpu"),
        "memory_bytes": sysctl("hw.memsize"),
        "power": probe("pmset", &["-g", "batt"]).and_then(|s| s.lines().next().map(str::to_owned)),
    })
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--replies") {
        let (path, prompts, out, reply_tokens) = match args.as_slice() {
            [path, r, prompts, o, out] if r == "--replies" && o == "--out" => {
                (path, prompts, out, DECODE)
            }
            [path, r, prompts, o, out, t, n]
                if r == "--replies" && o == "--out" && t == "--reply-tokens" =>
            {
                (
                    path,
                    prompts,
                    out,
                    n.parse()
                        .map_err(|_| format!("--reply-tokens {n}: {USAGE}"))?,
                )
            }
            _ => return Err(USAGE.to_owned()),
        };
        let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        let model = IntegerStackModel::parse(&bytes).map_err(|e| format!("parse: {e}"))?;
        return replies(&model, prompts, out, reply_tokens);
    }
    match args.as_slice() {
        [flag, first, second] if flag == "--hamming-trace-compare" => {
            return hamming_trace_compare(first, second);
        }
        [path, flag, readout, out] if flag == "--hamming-trace" => {
            let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
            let model = IntegerStackModel::parse(&bytes).map_err(|e| format!("parse: {e}"))?;
            return hamming_trace(&model, &bytes, readout, out);
        }
        [path, flag, float, integer, out] if flag == "--copy-codes" => {
            return copy_codes(path, float, integer, out);
        }
        _ => {}
    }
    let sweep = args.iter().any(|a| a == "--context-sweep");
    let args: Vec<String> = args
        .into_iter()
        .filter(|a| a != "--context-sweep")
        .collect();
    let parts = args.iter().any(|a| a == "--parts");
    let stages = parts || args.iter().any(|a| a == "--stages");
    let args: Vec<String> = args
        .into_iter()
        .filter(|a| a != "--stages" && a != "--parts")
        .collect();
    let (path, prompt_arg, tokens_arg) = match args.as_slice() {
        [path] => (path, None, None),
        [path, flag, ids] if flag == "--prompt" => (path, Some(ids), None),
        [path, flag, file] if flag == "--tokens" && sweep => (path, None, Some(file)),
        _ => return Err(USAGE.to_owned()),
    };
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let model = IntegerStackModel::parse(&bytes).map_err(|e| format!("parse: {e}"))?;
    let (vocab, context) = (model.shape().vocab, model.shape().context);
    if context < PREFILL + DECODE {
        return Err(format!(
            "the artifact's context {context} is shorter than {PREFILL} + {DECODE}"
        ));
    }
    let prompt: Vec<u32> = match prompt_arg {
        Some(ids) => ids
            .split(',')
            .map(|s| {
                s.trim()
                    .parse::<u32>()
                    .map_err(|e| format!("--prompt {s}: {e}"))
            })
            .collect::<Result<_, _>>()?,
        // A fixed deterministic prompt, as the readouts' sequences are built.
        None => (0..PREFILL)
            .map(|t| ((t * 11 + 1) % vocab) as u32)
            .collect(),
    };
    if prompt.len() != PREFILL {
        return Err(format!(
            "the prompt has {} tokens, not {PREFILL}",
            prompt.len()
        ));
    }
    if let Some(&t) = prompt.iter().find(|&&t| t as usize >= vocab) {
        return Err(format!(
            "prompt token {t} is outside the vocabulary {vocab}"
        ));
    }
    if sweep {
        if stages || prompt_arg.is_some() {
            return Err(format!("--context-sweep takes no other flag: {USAGE}"));
        }
        if read_hamming_uorlut01(&bytes)
            .map_err(|e| format!("the Hamming reads: {e}"))?
            .is_none()
        {
            return Err(
                "--context-sweep times the Hamming read, and the artifact has none".to_owned(),
            );
        }
        let (prompt, prefills, tokens) = match tokens_arg {
            None => (prompt, &SWEEP_PREFILLS[..], None),
            Some(file) => {
                let raw = std::fs::read(file).map_err(|e| format!("{file}: {e}"))?;
                let prompt = long_sweep_prompt(&raw, vocab, context)?;
                let tokens = json!({
                    "file": file,
                    "sha256": hex::encode(<sha2::Sha256 as sha2::Digest>::digest(&raw)),
                    "ids": (raw.len() - 64) / 2,
                });
                (prompt, &LONG_SWEEP_PREFILLS[..], Some(tokens))
            }
        };
        let without = IntegerStackModel::parse(&bytes)
            .map_err(|e| format!("parse: {e}"))?
            .without_hamming_read();
        let mut report = json!({
            "schema": "uor-r4.fbs-toks-context/1",
            "artifact_sha256": model.artifact_sha256(),
            "decode_tokens": DECODE,
            "points": context_sweep(&model, &without, &prompt, prefills)?,
            "hardware": hardware(),
        });
        if let Some(tokens) = tokens {
            report["context"] = json!(context);
            report["tokens"] = tokens;
        }
        println!("{report}");
        return Ok(());
    }
    let timing = protocol(|| model.session(), &prompt)?;
    let mut report = json!({
        "schema": "uor-r4.fbs-toks/1",
        "artifact_sha256": model.artifact_sha256(),
        "prompt": prompt,
        "timing": timing,
        "hardware": hardware(),
    });
    if stages {
        let (rows, staged_reply) = stage_protocol(&model, &prompt, false)?;
        let plain_decode = report["timing"]["decode"]["median_ns"]
            .as_u64()
            .ok_or("the plain protocol wrote no decode median")?;
        let plain_reply: Vec<u32> = serde_json::from_value(report["timing"]["reply"].clone())
            .map_err(|e| format!("the plain protocol's reply: {e}"))?;
        report["stages"] = stage_report(&rows, &BLOCKS, plain_decode, &plain_reply, &staged_reply)?;
        if parts {
            let (rows, parted_reply) = stage_protocol(&model, &prompt, true)?;
            let mut parted =
                stage_report(&rows, &PARTS, plain_decode, &plain_reply, &parted_reply)?;
            parted["against_blocks"] = blocks_against_parts(&report["stages"], &parted)?;
            report["parts"] = parted;
        }
    }
    println!("{report}");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("fbs-toks: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A session that answers each token with the next id, or, once `flip`
    /// is set, a different reply on every run.
    struct Fake {
        runs: std::rc::Rc<std::cell::Cell<u32>>,
        flip: bool,
    }

    impl Stepper for Fake {
        fn step(&mut self, token: u32) -> Result<u32, String> {
            Ok(if self.flip {
                token + self.runs.get()
            } else {
                token + 1
            })
        }
    }

    #[test]
    fn the_stage_report_takes_medians_and_reads_the_step_against_the_plain_token() {
        // One layer: embed, recurrence, mlp, final norm, writer, bookkeeping, step, nearest.
        let row = |scale: u64| -> Vec<u64> {
            [10, 40, 20, 5, 15, 2, 100, 300]
                .iter()
                .map(|v| v * scale)
                .collect()
        };
        let rows = [row(1), row(2), row(3)];
        let report = stage_report(&rows, &BLOCKS, 64 * 250, &[1, 2], &[1, 2]).expect("report");
        assert_eq!(report["decode_steps"], 3);
        let names: Vec<&str> = report["stages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            names,
            [
                "embed",
                "layer0.recurrence",
                "layer0.mlp",
                "final_norm",
                "writer",
                "bookkeeping"
            ]
        );
        // The middle row's stages: 20 + 80 + 40 + 10 + 30 + 4 = 184, against its step of 200.
        assert_eq!(report["stage_sum_ns"], 184);
        assert_eq!(report["step_median_ns"], 200);
        assert_eq!(report["stage_sum_per_mille_of_step"], 920);
        assert_eq!(report["whole_table_nearest_median_ns"], 600);
        // The plain protocol's decode: 250 ns a token, so the loop costs 50 beside the step.
        assert_eq!(report["plain_decode_per_token_ns"], 250);
        assert_eq!(report["step_per_mille_of_plain_token"], 800);
        assert_eq!(report["loop_overhead_ns"], 50);
        assert_eq!(report["replies_match"], true);
        let other = stage_report(&rows, &BLOCKS, 64 * 250, &[1, 2], &[1, 3]).expect("report");
        assert_eq!(other["replies_match"], false);
        // Rows that do not fit the stage order are refused.
        assert!(stage_report(&[vec![1, 2, 3, 4, 5, 6, 7]], &BLOCKS, 1, &[], &[]).is_err());
        assert!(stage_report(&[], &BLOCKS, 1, &[], &[]).is_err());
        // Nine parts a layer: a row of 6 + 9 fits, one of 6 + 2 does not.
        assert!(stage_report(&[vec![1; 15]], &PARTS, 1, &[], &[]).is_ok());
        assert!(stage_report(&[vec![1; 8]], &PARTS, 1, &[], &[]).is_err());
    }

    #[test]
    fn each_block_is_read_against_the_sum_of_its_parts() {
        // One layer. Blocks: recurrence 100, mlp 200.
        let blocks = stage_report(
            &[vec![5, 100, 200, 1, 1, 1, 400, 9]],
            &BLOCKS,
            64 * 500,
            &[],
            &[],
        )
        .expect("blocks");
        // Parts: the recurrence's four sum to 95 (950 per mille); the mlp's five to 230 (1,150).
        let row = vec![5, 10, 40, 30, 15, 20, 60, 100, 20, 30, 1, 1, 1, 420, 9];
        let parts = stage_report(&[row], &PARTS, 64 * 500, &[], &[]).expect("parts");
        let names: Vec<&str> = parts["stages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["name"].as_str().unwrap())
            .collect();
        assert_eq!(names[1], "layer0.recurrence.norm");
        assert_eq!(names[9], "layer0.mlp.gather_out");
        let read = blocks_against_parts(&blocks, &parts).expect("against");
        assert_eq!(read["blocks"][0]["block"], "layer0.recurrence");
        assert_eq!(read["blocks"][0]["parts_sum_ns"], 95);
        assert_eq!(read["blocks"][0]["per_mille"], 950);
        assert_eq!(read["blocks"][1]["block"], "layer0.mlp");
        assert_eq!(read["blocks"][1]["parts_sum_ns"], 230);
        assert_eq!(read["blocks"][1]["per_mille"], 1150);
        // The mlp's parts lie 15% above its block, so not every block is within 10%.
        assert_eq!(read["within_10_percent"], false);
        // Within 10% at both: the mlp's parts brought to 210 (1,050 per mille).
        let row = vec![5, 10, 40, 30, 15, 20, 60, 80, 20, 30, 1, 1, 1, 400, 9];
        let parts = stage_report(&[row], &PARTS, 64 * 500, &[], &[]).expect("parts");
        let read = blocks_against_parts(&blocks, &parts).expect("against");
        assert_eq!(read["blocks"][1]["per_mille"], 1050);
        assert_eq!(read["within_10_percent"], true);
        // A part report that names no part of a block is refused.
        assert!(blocks_against_parts(&blocks, &blocks).is_err());
        // So is one that holds only some of a block's parts: a second layer's
        // blocks against a one-layer part report.
        let two = stage_report(
            &[vec![5, 100, 200, 100, 200, 1, 1, 1, 700, 9]],
            &BLOCKS,
            64 * 500,
            &[],
            &[],
        )
        .expect("two layers");
        assert!(blocks_against_parts(&two, &parts).is_err());
        let mut short = parts.clone();
        short["stages"].as_array_mut().unwrap().remove(9);
        let err = blocks_against_parts(&blocks, &short).expect_err("a missing part");
        assert_eq!(err, "4 of layer0.mlp's 5 parts were timed");
    }

    #[test]
    fn the_statistics_follow_the_step_6_protocol() {
        assert_eq!(median(&[1, 2, 9]), 2);
        // The integer mean of the middle two for an even count, not the upper one.
        assert_eq!(median(&[1, 2, 4, 9]), 3);
        // Rank ceil(0.95 n): 19 of 20, 1 of 1.
        let twenty: Vec<u64> = (1..=20).collect();
        assert_eq!(p95(&twenty), 19);
        assert_eq!(p95(&[7]), 7);
        assert_eq!(tokens_per_second(64, 1_000_000), 64_000);
        assert_eq!(tokens_per_second(64, 0), 64_000_000_000);
    }

    #[test]
    fn the_long_sweep_reads_a_uort_file_and_refuses_what_it_cannot_sweep() {
        // The tokenless sweep keeps claude-235's prefills; the long one ends at 192 and 32,768.
        assert_eq!(SWEEP_PREFILLS, [32, 64, 96, 128]);
        let largest = LONG_SWEEP_PREFILLS[LONG_SWEEP_PREFILLS.len() - 1];
        let end = largest + DECODE;
        assert_eq!(LONG_SWEEP_PREFILLS[0] + DECODE, 192);
        assert_eq!(end, 32_768);
        // A UORT file: the magic, a version, the id count, then the ids low byte first.
        let uort = |ids: &[u16], count: u64| -> Vec<u8> {
            let mut b = vec![0u8; 64];
            b[..4].copy_from_slice(b"UORT");
            b[4] = 1;
            b[8..16].copy_from_slice(&count.to_le_bytes());
            b.extend(ids.iter().flat_map(|i| i.to_le_bytes()));
            b
        };
        let bytes_of = |ids: &[u16]| uort(ids, ids.len() as u64);
        // One id more than the largest prefill. The first reads 258 low byte first (513 if not).
        let mut ids: Vec<u16> = (0..=largest).map(|i| (i % 1000) as u16).collect();
        ids[0] = 0x0102;
        let bytes = bytes_of(&ids);
        let prompt = long_sweep_prompt(&bytes, 1000, end).expect("prompt");
        assert_eq!(prompt.len(), largest);
        assert_eq!(prompt[0], 258);
        assert_eq!(prompt[largest - 1], ((largest - 1) % 1000) as u32);
        // An artifact one position short of the last context.
        assert_eq!(
            long_sweep_prompt(&bytes, 1000, end - 1).expect_err("short context"),
            format!(
                "the artifact's context {} is shorter than {largest} + {DECODE}",
                end - 1
            )
        );
        // Raw ids with no header, a wrong magic, and a header shorter than 64 bytes.
        let raw: Vec<u8> = ids.iter().flat_map(|i| i.to_le_bytes()).collect();
        let not_uort = "the token file is not a UORT token file";
        assert_eq!(
            long_sweep_prompt(&raw, 1000, end).expect_err("raw"),
            not_uort
        );
        let mut magic = bytes.clone();
        magic[3] = b'X';
        assert_eq!(
            long_sweep_prompt(&magic, 1000, end).expect_err("magic"),
            not_uort
        );
        assert_eq!(
            long_sweep_prompt(&bytes[..63], 1000, end).expect_err("short"),
            not_uort
        );
        // A count that disagrees with the payload, and a payload of odd length.
        assert_eq!(
            long_sweep_prompt(&uort(&ids, ids.len() as u64 + 1), 1000, end).expect_err("count"),
            format!(
                "the token file's header counts {} ids, its payload holds {} bytes",
                ids.len() + 1,
                2 * ids.len()
            )
        );
        assert!(long_sweep_prompt(&bytes[..bytes.len() - 1], 1000, end)
            .expect_err("odd length")
            .contains("its payload holds"));
        // A file an id short of the largest prefill.
        assert_eq!(
            long_sweep_prompt(&bytes_of(&ids[..largest - 1]), 1000, end).expect_err("an id short"),
            format!(
                "the token file holds {} ids, fewer than {largest}",
                largest - 1
            )
        );
        // An id at the vocabulary's size in the prompt is refused; past the prompt it is not read.
        let mut inside = ids.clone();
        inside[largest - 1] = 1000;
        assert_eq!(
            long_sweep_prompt(&bytes_of(&inside), 1000, end).expect_err("outside the vocabulary"),
            format!(
                "token 1000 at {} is outside the vocabulary 1000",
                largest - 1
            )
        );
        let mut past = ids;
        past[largest] = 1000;
        assert!(long_sweep_prompt(&bytes_of(&past), 1000, end).is_ok());
    }

    #[test]
    fn a_run_prefills_then_decodes_greedily_and_replies_must_agree() {
        let runs = std::rc::Rc::new(std::cell::Cell::new(0));
        let prompt: Vec<u32> = (0..PREFILL as u32).collect();
        let fresh = || {
            runs.set(runs.get() + 1);
            Fake {
                runs: runs.clone(),
                flip: false,
            }
        };
        let report = protocol(fresh, &prompt).expect("protocol");
        // One warmup and 20 timed runs, each in a fresh session.
        assert_eq!(runs.get(), (WARMUP + RUNS) as u32);
        // The prefill's last choice, then 64 decode steps each feeding the last.
        let reply: Vec<u32> = (PREFILL as u32..=(PREFILL + DECODE) as u32).collect();
        assert_eq!(report["reply"], json!(reply));
        assert_eq!(report["decode"]["tokens"], DECODE);
        assert_eq!(report["context_prefill"]["tokens"], PREFILL);
        assert_eq!(
            report["decode"]["raw_ns"].as_array().map(Vec::len),
            Some(RUNS)
        );
        let flipping = || {
            runs.set(runs.get() + 1);
            Fake {
                runs: runs.clone(),
                flip: true,
            }
        };
        assert_eq!(
            protocol(flipping, &prompt).expect_err("different replies"),
            "timed runs wrote different replies"
        );
    }
}
