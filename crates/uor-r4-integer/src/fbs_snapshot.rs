//! The offline SpiralCore/FBS float-trunk snapshot: plain data, with no serving
//! code, so a converter in the native graph can build it without the
//! `serving-wasm` feature (d11-exporter-float-trunk, claude-123).
//!
//! The serving build exports it with `stack::export_fbs_trunk_uorlut01`, and
//! `stack::format` re-exports these types. The snapshot serializes to JSON for
//! the handoff between the two builds; `serde_json` writes each `f32` so that
//! it reads back exactly.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The fixed-point exponent the exporter stores biases and decay offsets at
/// (`stack::kernels::RESIDUAL_EXP`, held equal by a stack test).
pub const FBS_FIXED_EXP: i32 = -16;

/// Encode a finite scalar as a grid code `±(16 + m) 2^(e - 4)` with
/// `1 <= e + 64 <= 127`: the exporter's form for gains and convolution taps.
/// `stack::encode_grid_code` wraps it.
pub fn grid_code(value: f64, positive: bool) -> Result<i16, String> {
    if !value.is_finite() || (positive && value <= 0.0) {
        return Err("nonfinite or non-positive scalar".to_owned());
    }
    if value.abs() < 1e-18 {
        if positive {
            return Err("zero positive grid scalar".to_owned());
        }
        return Ok(0);
    }
    let abs = value.abs();
    let mut best_code = 0i16;
    let mut best_err = f64::INFINITY;
    for field in 1i16..=127 {
        let e = i32::from(field) - 64;
        let step = 2f64.powi(e - 4);
        for m in 0i16..16 {
            let cand = f64::from(16 + m) * step;
            let err = (abs - cand).abs();
            if err < best_err {
                best_err = err;
                best_code = (field << 4) | m;
            }
        }
    }
    Ok(if value < 0.0 { -best_code } else { best_code })
}

/// Decode a grid code `±(16 + m) 2^(e - 4)` to `f64`.
pub fn grid_value(code: i16) -> f64 {
    if code == 0 {
        return 0.0;
    }
    let c = code.unsigned_abs();
    let e = i32::from(c >> 4) - 64;
    let mag = f64::from(16 + (c & 15)) * 2f64.powi(e - 4);
    if code < 0 {
        -mag
    } else {
        mag
    }
}

/// `v` rounded to fixed point at `2^exp`, as the exporter stores it.
pub fn fixed_value(v: f32, exp: i32) -> f64 {
    (f64::from(v) * 2f64.powi(-exp)).round() * 2f64.powi(exp)
}

/// `v` at Q12 in `i16`, as the exporter writes the writer table, or `None`
/// when it is not finite or does not fit (the exporter refuses both).
pub fn q12_value(v: f32) -> Option<f64> {
    if !v.is_finite() {
        return None;
    }
    i16::try_from((f64::from(v) * 4096.0).round() as i64)
        .ok()
        .map(|k| f64::from(k) / 4096.0)
}

/// Write `bytes` to a new file at `path`, never leaving a partial file there
/// and never replacing one. The bytes go to `<path>.partial` first, which is
/// linked into place only when complete (the link refuses an existing
/// `path`); the partial file is removed either way. For the handoff bins.
pub fn write_new_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut partial = path.as_os_str().to_owned();
    partial.push(".partial");
    let partial = PathBuf::from(partial);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&partial)?;
    let written = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    let linked = written.and_then(|()| std::fs::hard_link(&partial, path));
    let removed = std::fs::remove_file(&partial);
    linked.and(removed)
}

/// SpiralCore/FBS routes of one projection: one `(atom, state)` per output 8-block.
///
/// Output block `o` reads input block `atoms[o]`: its channel `j` is
/// `y[o*8 + j] = ±x[atoms[o]*8 + (code & 7)]`, negated when `code & 8`, where
/// `code` is the `j`-th code of the 64-state Cl(0,6) signed-axis table at
/// `states[o]` (`discrete_d11::spiralcore_signed_axes`). The integer session
/// serves it as a gather and a sign (`stack_fbs_gather`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsRoutes {
    /// Input 8-blocks the atoms index.
    pub in_blocks: usize,
    /// Output 8-blocks, one route each.
    pub out_blocks: usize,
    /// `out_blocks` input-block indices, each below `in_blocks`.
    pub atoms: Vec<u16>,
    /// `out_blocks` signed-axis states, each below 64.
    pub states: Vec<u8>,
}

/// One recurrence layer of a SpiralCore/FBS trunk snapshot (`d = width`,
/// `m = mlp`, `lanes = d / 4`, `g = lanes + d`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsTrunkLayer {
    /// `d` RMSNorm gain before the recurrence.
    pub rec_norm: Vec<f32>,
    /// `d` RMSNorm gain before the Hamilton feed-forward block.
    pub mlp_norm: Vec<f32>,
    /// `d/8 -> 2d/8` drive and output-gate routes.
    pub rec_in: FbsRoutes,
    /// `d/8 -> g/8` decay-gate and rotation routes.
    pub rec_gate: FbsRoutes,
    /// `d/8 -> d/8` output routes.
    pub rec_out: FbsRoutes,
    /// `d/8 -> m/8` feed-forward gate routes.
    pub mlp_gate: FbsRoutes,
    /// `d/8 -> m/8` feed-forward up routes.
    pub mlp_up: FbsRoutes,
    /// `m/8 -> d/8` feed-forward down routes.
    pub mlp_down: FbsRoutes,
    /// `4 * d` causal convolution taps `[tap][channel]`, tap 0 on the current
    /// position, as the dense trunk's `FloatTrunkRecurrence::conv_taps`.
    pub conv_taps: Vec<f32>,
    /// `d` causal convolution biases.
    pub conv_bias: Vec<f32>,
    /// `g` gate biases (decay-gate bias then rotation bias).
    pub gate_bias: Vec<f32>,
    /// `lanes` learned per-lane decay offsets.
    pub decay: Vec<f32>,
}

/// The float GPU trunk's own output at one single-token position: the
/// comparison side of `fbs-compare`, written by the float converter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsFloatPosition {
    /// The input token.
    pub token: u32,
    /// The float native writer's next token.
    pub next: u32,
    /// `width` final hidden values, after the final RMSNorm gain.
    pub hidden: Vec<f32>,
}

/// The float GPU trunk's outputs at every position of one token sequence: the
/// recurrence's transition, its later convolution taps and the carried state
/// act only after the first position (claude-124).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsFloatSequence {
    /// The input tokens, in order.
    pub tokens: Vec<u32>,
    /// The float native writer's next token at each position.
    pub next: Vec<u32>,
    /// `width` final hidden values at each position.
    pub hidden: Vec<Vec<f32>>,
}

/// How the float GPU trunk was read (claude-124, claude-125).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FbsReadoutMode {
    /// Plain float: nothing snapped, tables at full precision.
    Plain,
    /// The float sheets' D11 export mode: Q12-rounded, icosian-snapped
    /// Hamilton gates; the transition and the tables as in `Plain`.
    Export,
    /// The D11 mode: the gates and the transition snapped to the 120 icosians
    /// directly (no Q12 rounding of the gates, as the integer path), the
    /// tables rounded as exported and the final query rounded to Q12, so the
    /// float trunk computes what the artifact computes.
    D11,
}

/// The float GPU trunk's single-token and sequence outputs, for comparison
/// with its converted snapshot's reference and integer session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsFloatReadout {
    pub vocab: usize,
    pub width: usize,
    /// The mode the trunk was read in.
    pub mode: FbsReadoutMode,
    /// One position per token, in token order.
    pub positions: Vec<FbsFloatPosition>,
    /// Token sequences read position by position.
    pub sequences: Vec<FbsFloatSequence>,
}

impl FbsFloatReadout {
    /// Refuse a readout that does not hold every token of a `vocab`-token,
    /// `width`-wide trunk once, in token order, with a full hidden vector,
    /// that holds a sequence that is empty, longer than `context`, out of the
    /// vocabulary or not fully read, or that was read in the other mode.
    pub fn check(
        &self,
        vocab: usize,
        width: usize,
        context: usize,
        mode: FbsReadoutMode,
    ) -> Result<(), String> {
        if self.sequences.is_empty() {
            return Err("the readout holds no sequences".to_owned());
        }
        for (s, q) in self.sequences.iter().enumerate() {
            let n = q.tokens.len();
            if n == 0
                || n > context
                || q.tokens.iter().any(|&t| t as usize >= vocab)
                || q.next.len() != n
                || q.hidden.len() != n
                || q.hidden.iter().any(|h| h.len() != width)
            {
                return Err(format!(
                    "sequence {s} is not 1 to {context} tokens with an output at each"
                ));
            }
        }
        if self.vocab != vocab || self.width != width {
            return Err("the readout's vocabulary or width differs from the trunk's".to_owned());
        }
        if self.mode != mode {
            return Err(format!("the readout was read in mode {:?}", self.mode));
        }
        if self.positions.len() != vocab {
            return Err(format!(
                "{} positions for a vocabulary of {vocab}",
                self.positions.len()
            ));
        }
        match self
            .positions
            .iter()
            .enumerate()
            .find(|(i, p)| p.token as usize != *i || p.hidden.len() != width)
        {
            Some((i, _)) => Err(format!("position {i} is not token {i} with {width} values")),
            None => Ok(()),
        }
    }
}

/// Offline SpiralCore/FBS trunk snapshot handed to `stack::export_fbs_trunk_uorlut01`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsTrunkSnapshot {
    pub vocab: usize,
    pub width: usize,
    pub mlp: usize,
    pub context: usize,
    pub rms_eps: f64,
    /// Cells the native vector head keeps (1 to 16).
    pub kept_cells: usize,
    /// `vocab * width` row-major: the input embedding and the writer table.
    pub embed: Vec<f32>,
    /// `width` final RMSNorm gain.
    pub final_norm: Vec<f32>,
    pub layers: Vec<FbsTrunkLayer>,
    /// The induction-keyed Hamming reads (claude-146) the trunk serves; absent on a trunk without
    /// them, so a snapshot written before them still parses (claude-188).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hamming: Option<FbsHammingSnapshot>,
    /// The turn memory (claude-161, claude-193) the trunk was trained with; absent on a trunk
    /// without one, so every snapshot written before it still parses (claude-198).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<FbsMemorySnapshot>,
    /// Whether the trunk was trained with the recurrence's retention reading of its rate table
    /// (the float trainer's `retain_rates`, integer-retention-rates, claude-226). The export then
    /// serves `rec_gates = "geometric-retention"`. False, and not written, for the decay reading,
    /// so every earlier snapshot parses and exports as before.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub retain_rates: bool,
    /// Whether the trunk decodes as the whole table's L1-nearest token, as a model trained with
    /// the `cell` or `nearest` objective does ([`run_readout_nearest`], integer-nearest-readout,
    /// claude-231). The export then serves `head = "native_vector_nearest"`. False, and not
    /// written, for the writer's pick, so every earlier snapshot parses and exports as before.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub readout_nearest: bool,
}

/// The readout a run's `config.json` trained for (integer-nearest-readout, claude-231): its
/// `objective`, where `cell` and `nearest` decode as the whole table's nearest token and `sheet`
/// and `voronoi` as the writer's pick (the float trainer's `PointObjective::decodes_nearest`). A
/// config without an objective is a run from before the objectives, read by its writer. Any other
/// value is refused.
pub fn run_readout_nearest(config: &str) -> Result<bool, String> {
    let value: serde_json::Value =
        serde_json::from_str(config).map_err(|e| format!("the run config: {e}"))?;
    match value.get("objective") {
        None | Some(serde_json::Value::Null) => Ok(false),
        Some(serde_json::Value::String(o)) => match o.as_str() {
            "cell" | "nearest" => Ok(true),
            "sheet" | "voronoi" => Ok(false),
            other => Err(format!(
                "the run config's objective {other:?} is not sheet, voronoi, nearest or cell"
            )),
        },
        Some(other) => Err(format!("the run config's objective {other} is not a name")),
    }
}

/// The longest memory key, and the width a reply key is padded to: the float memory's
/// `MAX_REPLY_KEY` (16 turn ids, two separators, a length and an id).
pub const MEMORY_REPLY_KEY_LEN: usize = 20;
/// The most turn ids an opening key may hold (the float memory's `MAX_KEY`).
pub const MEMORY_MAX_K: usize = 16;
/// Pads a stored key shorter than its row, and joins a reply key's parts; never a token id.
pub const MEMORY_PAD: u32 = u32::MAX;

/// The turn memory of `turn_memory.safetensors` and `turn_memory_reply.safetensors`: exact tables
/// of token ids, and the per-coordinate weights of their moves on the readout point.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsMemorySnapshot {
    /// The turn ids an opening key holds (claude-161's 8).
    pub k: usize,
    /// The training window the keys were cut in (`hamilton_sheet`'s `SHEET_CONTEXT`, 192). A
    /// session keys over blocks of it, whatever its own context (claude-196).
    pub key_window: usize,
    /// The opening table: each key, at most `k` turn ids, and the id it names, in ascending key
    /// order.
    pub opening: Vec<FbsMemoryEntry>,
    /// `width`: the opening move's weight.
    pub opening_weight: Vec<f32>,
    /// The reply table (claude-193), when the memory has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply: Option<FbsReplyTable>,
}

/// One memory table entry: a key of token ids (a reply key joins its parts with `u32::MAX`), and
/// the token id it names.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsMemoryEntry {
    pub key: Vec<u32>,
    pub value: u32,
}

/// The reply table: keys `turn ids, u32::MAX, reply length, u32::MAX, last reply id` in ascending
/// order, and the reply move's weight.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsReplyTable {
    /// The most turn ids a reply key holds: the float trainer's `TURN_KEY` (8), which does not
    /// follow the opening table's `k`.
    pub turn: usize,
    pub entries: Vec<FbsMemoryEntry>,
    /// `width`: the reply move's weight.
    pub weight: Vec<f32>,
}

/// The Hamming reads of claude-146's `hamming_read.safetensors`: one read per depth.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsHammingSnapshot {
    /// Heads per read: the first half copy heads, the rest type heads, each matching on its own
    /// `width / heads` coordinates.
    pub heads: usize,
    /// Each read's depth, in the float trunk's `HAMMING_DEPTHS` order: a read after that many
    /// layers, inside the stream, or on the final norm's output at the layer count.
    pub depths: Vec<usize>,
    /// One read per depth, in the same order.
    pub reads: Vec<FbsHammingRead>,
}

/// One Hamming read's learned parameters, as `hamming_read.safetensors` stores them under
/// `d{depth}.{name}`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FbsHammingRead {
    /// `width / 4` quaternions `[w, x, y, z]`, row-major: the type heads' query rotation.
    pub query_rotation: Vec<f32>,
    /// `width / 4` quaternions: the key rotation, which the copy heads' queries share.
    pub key_rotation: Vec<f32>,
    /// `width / 4` quaternions: the type heads' value rotation.
    pub value_rotation: Vec<f32>,
    /// `heads` margins, in bits.
    pub margin: Vec<f32>,
    /// `2 * width`: row 0 the copy heads' weights `w_c`, row 1 the type heads' `w_y`.
    pub weight: Vec<f32>,
}

/// The schema a [`HammingTrace`] carries.
pub const HAMMING_TRACE_SCHEMA: &str = "uor-r4.hamming-trace/1";

/// What each Hamming read chose, per window, read and head (claude-235, the item's step 5). The
/// float D11 readout writes one (`fbs-snapshot --hamming-trace`), the integer session another
/// (`fbs-toks --hamming-trace`), and `fbs-toks --hamming-trace-compare` holds the two side by side.
/// A head matches on one 64-bit word, so every code is one `u64`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HammingTrace {
    /// [`HAMMING_TRACE_SCHEMA`].
    pub schema: String,
    /// Who wrote it: `float-d11` or `integer`.
    pub side: String,
    /// The float arm (`d11`, `d11-tie`, `d11-rounded`, `d11-tie-rounded`), or `integer`.
    pub arm: String,
    /// Heads per read: the first `heads / 2` are copy heads, the rest type heads.
    pub heads: usize,
    /// The most positions a head keeps (`HAMMING_KEPT`).
    pub kept: usize,
    /// One entry per window read, in the order read.
    pub windows: Vec<HammingTraceWindow>,
}

/// One window's trace.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HammingTraceWindow {
    /// The window's token ids.
    pub ids: Vec<u32>,
    /// Each read in the order the forward runs them: the stream read, then the readout read.
    pub reads: Vec<HammingTraceRead>,
}

/// One read's choices over one window.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HammingTraceRead {
    /// The read's depth: the layers before it, or the layer count for the readout read.
    pub depth: usize,
    /// One entry per head, in head order.
    pub heads: Vec<HammingTraceHead>,
}

/// One head's codes and choices at every position of one window.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HammingTraceHead {
    /// Each position's query code: the head's 64 coordinates' sign bits, a coordinate at or above
    /// 0 a 1, bit `i` the head's coordinate `i` (as `pack_signs` packs them).
    pub query: Vec<u64>,
    /// Each position's key code, packed the same way. Position 0's names no candidate.
    pub key: Vec<u64>,
    /// Each position's kept positions: real candidates only (`1 ≤ s < t`, fewer than `kept`
    /// when `t ≤ kept`), in no set order.
    pub kept: Vec<Vec<u32>>,
}

/// How two [`HammingTrace`]s' kept sets compare (claude-235, the item's step 5): a row per read
/// and head kind, in read order, the copy row before the type row.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HammingTraceComparison {
    pub windows: usize,
    pub rows: Vec<HammingCompareRow>,
    /// The first [`MAX_FAULTS_LISTED`] instrument faults, each with its window, read, head and
    /// position.
    pub faults: Vec<String>,
}

/// The most instrument faults a [`HammingTraceComparison`] lists by place.
pub const MAX_FAULTS_LISTED: usize = 20;

/// One read's sets of one head kind, compared.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HammingCompareRow {
    /// The read's depth.
    pub depth: usize,
    /// `copy` or `type`.
    pub kind: String,
    /// The sets judged, and of them those equal.
    pub judged: usize,
    pub equal: usize,
    /// Of the judged sets that hold a candidate, those whose nearest member, by each side's own
    /// codes (bit distance, then position), is the same.
    pub top1_judged: usize,
    pub top1_equal: usize,
    /// A judged set that differs, by cause: a tie across the last kept place; a sign flip in
    /// the query alone, in a key alone, or in both; an instrument fault.
    pub ties: usize,
    pub query_flips: usize,
    pub key_flips: usize,
    pub query_and_key_flips: usize,
    pub instrument_faults: usize,
    /// Type-head sets of a later read where an earlier read's sets differ at or before the
    /// position, so its residual carries that difference: counted apart, not judged.
    pub downstream: usize,
    /// Type-head sets the second trace does not hold (its read serves no type heads).
    pub not_served: usize,
    /// Of the judged sets that are real kept sets, the empty ones at rows 0 and 1 included, how
    /// many have their two query codes `d` bits apart, at index `d` (0 to 64): a few bits is a
    /// coordinate near a sign, half the word codes that share nothing.
    pub query_bits: Vec<usize>,
}

/// One side's kept set at row `t`, sorted, or why it cannot be the set that side chose: real
/// candidates are `1 ≤ s < t`, each once, `min(kept, t − 1)` of them, and by the side's own codes
/// no candidate it leaves out is nearer the query than one it keeps (bit distance; the order
/// among equal distances is the side's own). A side whose recorded codes are not the ones it
/// chose with fails here, so its differences are never read as sign flips.
fn kept_set(head: &HammingTraceHead, t: usize, most: usize) -> Result<Vec<u32>, String> {
    let mut set = head.kept[t].to_vec();
    set.sort_unstable();
    let want = most.min(t.saturating_sub(1));
    if set.len() != want {
        return Err(format!("{} kept, not {want}", set.len()));
    }
    if set.windows(2).any(|w| w[0] == w[1]) || set.iter().any(|&s| s == 0 || s as usize >= t) {
        return Err(format!("{set:?} are not distinct positions 1 ≤ s < {t}"));
    }
    let distance = |s: usize| (head.query[t] ^ head.key[s]).count_ones();
    if let Some(place) = set.iter().map(|&s| distance(s as usize)).max() {
        let nearer =
            (1..t).find(|&s| distance(s) < place && set.binary_search(&(s as u32)).is_err());
        if let Some(s) = nearer {
            return Err(format!(
                "a member {place} bits from the query is kept and position {s}, {} bits, is not: these are not the codes the set was chosen with",
                distance(s)
            ));
        }
    }
    Ok(set)
}

/// Compare the `first` trace with the `second` on the same windows (claude-235): normally the
/// float D11 readout's against the integer session's. A record that does not line up (other
/// windows, reads, heads or lengths) is refused; a set that cannot be a kept set, or is not the
/// nearest by its own side's recorded codes, is counted an instrument fault.
///
/// Every copy-head set, and every type-head set of the first read, is judged at every position:
/// their codes come from the table or from the trunk below the read. A later read's type-head
/// set is judged only where every earlier read's sets agree at its position and before it, and
/// counted as downstream elsewhere. A judged set that differs is a tie when both sides have the
/// same query code and the same key code at every position either keeps; anything else is a
/// sign flip.
pub fn compare_hamming_traces(
    first: &HammingTrace,
    second: &HammingTrace,
) -> Result<HammingTraceComparison, String> {
    for trace in [first, second] {
        if trace.schema != HAMMING_TRACE_SCHEMA {
            return Err(format!(
                "the {} trace's schema is {}, not {HAMMING_TRACE_SCHEMA}",
                trace.side, trace.schema
            ));
        }
    }
    let (heads, most) = (first.heads, first.kept);
    if heads == 0 || heads % 2 != 0 || second.heads != heads || second.kept != most {
        return Err(format!(
            "the traces hold {} and {} heads keeping {} and {}: they must agree, an even count",
            first.heads, second.heads, first.kept, second.kept
        ));
    }
    if first.windows.len() != second.windows.len() || first.windows.is_empty() {
        return Err(format!(
            "the traces hold {} and {} windows",
            first.windows.len(),
            second.windows.len()
        ));
    }
    let copy = heads / 2;
    let mut rows: Vec<HammingCompareRow> = first.windows[0]
        .reads
        .iter()
        .flat_map(|r| {
            ["copy", "type"].map(|kind| HammingCompareRow {
                depth: r.depth,
                kind: kind.to_owned(),
                query_bits: vec![0; 65],
                ..Default::default()
            })
        })
        .collect();
    let mut faults = Vec::new();
    for (w, (a, b)) in first.windows.iter().zip(&second.windows).enumerate() {
        let time = a.ids.len();
        let depths = |x: &HammingTraceWindow| x.reads.iter().map(|r| r.depth).collect::<Vec<_>>();
        if a.ids != b.ids
            || depths(a) != depths(b)
            || 2 * a.reads.len() != rows.len()
            || depths(a) != rows.iter().step_by(2).map(|r| r.depth).collect::<Vec<_>>()
        {
            return Err(format!("window {w}: the traces' ids or reads differ"));
        }
        // Whether every earlier read's sets agreed at each position and before it.
        let mut earlier = vec![true; time];
        for (r, (ra, rb)) in a.reads.iter().zip(&b.reads).enumerate() {
            if ra.heads.len() != heads || (rb.heads.len() != heads && rb.heads.len() != copy) {
                return Err(format!(
                    "window {w}, read {r}: {} and {} heads, not {heads}",
                    ra.heads.len(),
                    rb.heads.len()
                ));
            }
            let mut agree = vec![true; time];
            for h in 0..heads {
                let row = &mut rows[2 * r + usize::from(h >= copy)];
                let Some(hb) = rb.heads.get(h) else {
                    row.not_served += time;
                    continue;
                };
                let ha = &ra.heads[h];
                for x in [ha, hb] {
                    if x.query.len() != time || x.key.len() != time || x.kept.len() != time {
                        return Err(format!("window {w}, read {r}, head {h}: the codes and sets do not cover the window's {time} positions"));
                    }
                }
                for t in 0..time {
                    let (sa, sb) = (kept_set(ha, t, most), kept_set(hb, t, most));
                    let same = matches!((&sa, &sb), (Ok(x), Ok(y)) if x == y);
                    agree[t] &= same;
                    if r > 0 && h >= copy && !earlier[t] {
                        row.downstream += 1;
                        continue;
                    }
                    row.judged += 1;
                    let (sa, sb) = match (sa, sb) {
                        (Ok(sa), Ok(sb)) => (sa, sb),
                        (sa, sb) => {
                            row.instrument_faults += 1;
                            let why = sa.err().or(sb.err()).unwrap_or_default();
                            if faults.len() < MAX_FAULTS_LISTED {
                                faults.push(format!(
                                    "window {w}, read {r}, head {h}, position {t}: {why}"
                                ));
                            }
                            continue;
                        }
                    };
                    row.query_bits[(ha.query[t] ^ hb.query[t]).count_ones() as usize] += 1;
                    let distance = |x: &HammingTraceHead, s: u32| {
                        (x.query[t] ^ x.key[s as usize]).count_ones()
                    };
                    if !sa.is_empty() {
                        row.top1_judged += 1;
                        let nearest = |x: &HammingTraceHead, set: &[u32]| {
                            set.iter().copied().min_by_key(|&s| (distance(x, s), s))
                        };
                        row.top1_equal += usize::from(nearest(ha, &sa) == nearest(hb, &sb));
                    }
                    if same {
                        row.equal += 1;
                        continue;
                    }
                    let mut union: Vec<u32> = sa.iter().chain(&sb).copied().collect();
                    union.sort_unstable();
                    union.dedup();
                    let query_same = ha.query[t] == hb.query[t];
                    let keys_same = union
                        .iter()
                        .all(|&s| ha.key[s as usize] == hb.key[s as usize]);
                    match (query_same, keys_same) {
                        // Both sets are the nearest by the same codes on their union, so each
                        // position only one keeps sits at the last kept place of both: a tie.
                        (true, true) => row.ties += 1,
                        (false, true) => row.query_flips += 1,
                        (true, false) => row.key_flips += 1,
                        (false, false) => row.query_and_key_flips += 1,
                    }
                }
            }
            // A read's sets agree at `t` when they agree there and at every earlier position.
            let mut so_far = true;
            for (e, a) in earlier.iter_mut().zip(agree) {
                so_far &= a;
                *e &= so_far;
            }
        }
    }
    Ok(HammingTraceComparison {
        windows: first.windows.len(),
        rows,
        faults,
    })
}

/// The CPU float control's lane product (claude-237): the float trainer's
/// `kernel_controls::hamilton_gate_fwd_cpu`, the CPU twin of the GPU forward's `hamilton_gate`,
/// term for term, so that it gives the same bits (a float-train test holds the two equal). Each
/// lane of `up` is turned on the left by the unit `g / |g|` of its lane of `gate`.
pub fn hamilton_gate_f32(gate: &[f32], up: &[f32]) -> Vec<f32> {
    assert_eq!(gate.len(), up.len());
    assert_eq!(gate.len() % 4, 0);
    let mut out = vec![0.0f32; gate.len()];
    for i in (0..gate.len()).step_by(4) {
        let (g0, g1, g2, g3) = (gate[i], gate[i + 1], gate[i + 2], gate[i + 3]);
        let (u0, u1, u2, u3) = (up[i], up[i + 1], up[i + 2], up[i + 3]);
        let r = (g0 * g0 + g1 * g1 + g2 * g2 + g3 * g3 + 1e-12f32).sqrt();
        let (q0, q1, q2, q3) = (g0 / r, g1 / r, g2 / r, g3 / r);
        out[i] = q0 * u0 - q1 * u1 - q2 * u2 - q3 * u3;
        out[i + 1] = q0 * u1 + q1 * u0 + q2 * u3 - q3 * u2;
        out[i + 2] = q0 * u2 - q1 * u3 + q2 * u0 + q3 * u1;
        out[i + 3] = q0 * u3 + q1 * u2 - q2 * u1 + q3 * u0;
    }
    out
}

/// Each coordinate's sign bit, packed 64 to a word in coordinate order (claude-237): the float
/// trainer's `flock_read_hamming::pack_signs`, held equal by the same test. A coordinate at or
/// above 0 is a 1.
pub fn pack_signs_f32(coordinates: &[f32]) -> Vec<u64> {
    coordinates
        .chunks(64)
        .map(|chunk| {
            chunk
                .iter()
                .enumerate()
                .fold(0u64, |word, (i, &c)| word | (u64::from(c >= 0.0) << i))
        })
        .collect()
}

/// The copy heads' codes by the CPU float control (claude-237). Each of the `vocab` rows of
/// `table` (`width` floats each) is turned lane by lane by the `width` floats of `key_rotation`,
/// as the float forward turns a token's point for a copy head's query and key. Each copy head's
/// `width / heads` coordinates are then packed by [`pack_signs_f32`]; the first `heads / 2`
/// heads are the copy heads.
///
/// It returns the codes, entry `v`'s code for copy head `h` at `v * copy + h` (as `CopyHeads`
/// lays them), and the turned copy coordinates, entry `v`'s coordinate `c` at `v * copy_width +
/// c`, so that a differing bit's value can be read.
pub fn float_copy_codes(
    key_rotation: &[f32],
    table: &[f32],
    vocab: usize,
    width: usize,
    heads: usize,
) -> Result<(Vec<u64>, Vec<f32>), String> {
    if heads == 0
        || !heads.is_multiple_of(2)
        || !width.is_multiple_of(4)
        || !width.is_multiple_of(heads)
        || width / heads > 64
    {
        return Err(format!(
            "{heads} heads over width {width}: an even count of heads of at most 64 coordinates, over whole lanes"
        ));
    }
    if key_rotation.len() != width || table.len() != vocab * width {
        return Err(format!(
            "a rotation of {} and a table of {}, not {width} and {vocab} × {width}",
            key_rotation.len(),
            table.len()
        ));
    }
    let (copy, head_width) = (heads / 2, width / heads);
    let mut codes = Vec::with_capacity(vocab * copy);
    let mut values = Vec::with_capacity(vocab * copy * head_width);
    for row in table.chunks_exact(width) {
        let turned = hamilton_gate_f32(key_rotation, row);
        for h in 0..copy {
            codes.push(pack_signs_f32(&turned[h * head_width..][..head_width])[0]);
        }
        values.extend_from_slice(&turned[..copy * head_width]);
    }
    Ok((codes, values))
}

/// The 120 unit icosians of 2I in the float trainer's order (claude-237): its
/// `hamilton_sheet::H4_ROOT_COEFFICIENTS`, each coordinate `(a + b·φ) / 2` evaluated in f64 and
/// rounded to f32, as its `h4_unit_roots_f32` does (a float-train test holds the two equal). The
/// order is ±1 on each axis, then `(±½)⁴`, then `½(0, ±1, ±φ, ±1/φ)` in the 12 even orders.
pub fn icosian_roots_f32() -> [[f32; 4]; 120] {
    const EVEN_ORDERS: [[usize; 4]; 12] = [
        [0, 1, 2, 3],
        [0, 2, 3, 1],
        [0, 3, 1, 2],
        [1, 0, 3, 2],
        [1, 2, 0, 3],
        [1, 3, 2, 0],
        [2, 0, 1, 3],
        [2, 1, 3, 0],
        [2, 3, 0, 1],
        [3, 0, 2, 1],
        [3, 1, 0, 2],
        [3, 2, 1, 0],
    ];
    let mut coefficients = [[[0i8; 2]; 4]; 120];
    for (axis, pair) in coefficients[..8].chunks_exact_mut(2).enumerate() {
        pair[0][axis] = [-2, 0];
        pair[1][axis] = [2, 0];
    }
    let mut index = 8;
    for signs in 0..16 {
        for (axis, c) in coefficients[index].iter_mut().enumerate() {
            *c = [if signs & (1 << axis) == 0 { -1 } else { 1 }, 0];
        }
        index += 1;
    }
    for order in EVEN_ORDERS {
        for signs in 0..8 {
            let sign = |bit: i32| -> i8 {
                if signs & bit == 0 {
                    -1
                } else {
                    1
                }
            };
            let signed = [[0, 0], [sign(4), 0], [0, sign(2)], [-sign(1), sign(1)]];
            for (axis, c) in coefficients[index].iter_mut().enumerate() {
                *c = signed[order[axis]];
            }
            index += 1;
        }
    }
    let phi = 0.5 * (1.0 + 5.0_f64.sqrt());
    coefficients.map(|root| root.map(|[a, b]| (0.5 * (f64::from(a) + f64::from(b) * phi)) as f32))
}

/// Each lane of `rotation` replaced by the icosian of [`icosian_roots_f32`] with the largest
/// inner product, the first on a tie (claude-237). An all-zero lane takes root 1, the identity.
/// This is the snap the GPU forward's D11 mode applies to every Hamilton gate
/// (`hamilton_sheet::icosian_snap_cuda_tensor`), the Hamming read's rotations among them.
pub fn snap_icosian_f32(rotation: &[f32]) -> Vec<f32> {
    assert_eq!(rotation.len() % 4, 0);
    let roots = icosian_roots_f32();
    rotation
        .chunks_exact(4)
        .flat_map(|lane| {
            if lane.iter().all(|&x| x == 0.0) {
                return roots[1];
            }
            let score =
                |root: &[f32; 4]| -> f32 { root.iter().zip(lane).map(|(r, x)| r * x).sum() };
            let mut best = 0;
            for (i, root) in roots.iter().enumerate().skip(1) {
                if score(root) > score(&roots[best]) {
                    best = i;
                }
            }
            roots[best]
        })
        .collect()
}

/// One read's copy-head codes by the CPU float control and by the integer session, for
/// [`copy_code_check`] (claude-237). Each code list holds entry `v`'s code for copy head `h` at
/// `v * copy + h`; the control's turned copy coordinates hold entry `v`'s coordinate `c` at
/// `v * copy_width + c`.
#[derive(Clone, Debug, PartialEq)]
pub struct CopyCodeRead {
    pub depth: usize,
    pub control_codes: Vec<u64>,
    pub control_values: Vec<f32>,
    pub integer_codes: Vec<u64>,
    /// The control's codes with the key rotation first snapped by [`snap_icosian_f32`]: what the
    /// GPU's D11 forward computes, whose gate hook snaps the read's rotations too.
    pub snapped_codes: Vec<u64>,
}

/// Each read's [`CopyCodeRead`] (claude-237), from the artifact's own values: per read its depth,
/// its raw key rotation at `2^rotation_exp` and the session's codes, and the Q12 writer `table`.
/// The control reads the rotation as `raw · 2^rotation_exp` and the table as `q / 4096` (the
/// grid [`q12_value`] rounds to), each in f32, so it and the session share their inputs.
pub fn copy_code_reads(
    reads: &[(usize, &[i32], Vec<u64>)],
    rotation_exp: i32,
    table: &[i16],
    vocab: usize,
    width: usize,
    heads: usize,
) -> Result<Vec<CopyCodeRead>, String> {
    let points: Vec<f32> = table.iter().map(|&q| f32::from(q) / 4096.0).collect();
    let scale = 2f64.powi(rotation_exp);
    reads
        .iter()
        .map(|(depth, raw, integer_codes)| {
            let rotation: Vec<f32> = raw.iter().map(|&g| (f64::from(g) * scale) as f32).collect();
            let at = |e: String| format!("the read at depth {depth}: {e}");
            let (control_codes, control_values) =
                float_copy_codes(&rotation, &points, vocab, width, heads).map_err(at)?;
            let (snapped_codes, _) =
                float_copy_codes(&snap_icosian_f32(&rotation), &points, vocab, width, heads)
                    .map_err(at)?;
            Ok(CopyCodeRead {
                depth: *depth,
                control_codes,
                control_values,
                integer_codes: integer_codes.clone(),
                snapped_codes,
            })
        })
        .collect()
}

/// The schema `fbs-toks --copy-codes` writes (claude-237).
pub const COPY_CODES_SCHEMA: &str = "uor-r4.copy-codes/1";

/// The bins of a copy-code value histogram: the control's `|value|` below `1e-6`, `1e-5`,
/// `1e-4`, `1e-3`, `1e-2` and `1e-1`, then at or above `1e-1`.
pub const COPY_VALUE_BINS: usize = 7;

fn copy_value_bin(value: f32) -> usize {
    let size = f64::from(value).abs();
    [1e-6, 1e-5, 1e-4, 1e-3, 1e-2, 1e-1]
        .iter()
        .position(|&edge| size < edge)
        .unwrap_or(COPY_VALUE_BINS - 1)
}

/// The bits set in `word`, lowest first.
fn set_bits(word: u64) -> impl Iterator<Item = u32> {
    (0..64).filter(move |&b| (word >> b) & 1 == 1)
}

/// One bit where the control's code and the session's differ, with the control's turned
/// coordinate there.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyCodeBit {
    pub token: u32,
    pub head: usize,
    pub bit: u32,
    pub value: f32,
}

/// The control's codes against the session's, over every vocabulary entry and copy head.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyControlAgainstInteger {
    /// The codes compared (`vocab * copy`), and of them those equal.
    pub codes: usize,
    pub equal: usize,
    /// The bits that differ, in all.
    pub differing_bits: usize,
    /// The control's `|value|` at each differing bit, by [`COPY_VALUE_BINS`] bin.
    pub value_bins: Vec<usize>,
    /// The largest `|value|` at a differing bit; 0 when none differs.
    pub max_value: f32,
    /// The first [`MAX_FAULTS_LISTED`] differing bits.
    pub listed: Vec<CopyCodeBit>,
}

/// One trace's copy-head codes of one kind, query or key, against both sides' codes.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyCodeSamples {
    /// The samples, and of them those equal to the control's code, to the session's, and to the
    /// control's with the rotation snapped.
    pub samples: usize,
    pub equal_control: usize,
    pub equal_integer: usize,
    pub equal_snapped: usize,
    /// How many samples sit `d` bits from the control's code, at index `d` (0 to 64), and from
    /// the session's.
    pub bits_from_control: Vec<usize>,
    pub bits_from_integer: Vec<usize>,
    /// The control's `|value|` at each bit where a sample differs from the control's code, by
    /// [`COPY_VALUE_BINS`] bin.
    pub value_bins: Vec<usize>,
}

/// One trace's copy heads at one read.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyCodeTrace {
    pub side: String,
    pub arm: String,
    /// The query at each position `t`, which is `ids[t]`'s code, and the key at each `s ≥ 1`,
    /// which is `ids[s − 1]`'s.
    pub query: CopyCodeSamples,
    pub key: CopyCodeSamples,
    /// The (token, copy head) pairs the trace holds, and of them those it gives more than one
    /// code across its queries and keys.
    pub pairs: usize,
    pub inconsistent: usize,
}

/// One read's codes, three ways.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyCodeDepth {
    pub depth: usize,
    pub control_vs_integer: CopyControlAgainstInteger,
    pub float: CopyCodeTrace,
    pub integer: CopyCodeTrace,
}

/// The copy-code check's result (claude-237): a [`CopyCodeDepth`] per read, in depth order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyCodeCheck {
    pub vocab: usize,
    pub heads: usize,
    pub reads: Vec<CopyCodeDepth>,
}

/// Every vocabulary entry's copy-head code three ways (claude-237, the item's step 5).
///
/// It compares the CPU float control's codes with the integer session's, on the same inputs
/// (`reads`, in depth order), and both with the codes the float and the integer
/// [`HammingTrace`]s hold at each position, matched by token; each trace also against the
/// control with the rotation snapped. A copy code depends only on its token, so a trace that
/// gives one (token, head) pair two codes counts it `inconsistent`.
///
/// Inputs that do not line up are refused before anything is compared:
/// - codes of the wrong length;
/// - a trace of another schema or side, or with other heads;
/// - a window whose reads are not at `reads`' depths, whose read holds fewer than the copy heads,
///   whose codes are not one per position, or whose ids are not entries.
pub fn copy_code_check(
    vocab: usize,
    width: usize,
    heads: usize,
    reads: &[CopyCodeRead],
    float: &HammingTrace,
    integer: &HammingTrace,
) -> Result<CopyCodeCheck, String> {
    if heads == 0 || !heads.is_multiple_of(2) || !width.is_multiple_of(heads) || width / heads > 64
    {
        return Err(format!(
            "{heads} heads over width {width}: an even count of heads of at most 64 coordinates"
        ));
    }
    let (copy, head_width) = (heads / 2, width / heads);
    if reads.is_empty() {
        return Err("no read to check".to_owned());
    }
    for read in reads {
        let codes = vocab * copy;
        if read.control_codes.len() != codes
            || read.integer_codes.len() != codes
            || read.snapped_codes.len() != codes
            || read.control_values.len() != codes * head_width
        {
            return Err(format!(
                "the read at depth {}: {} control codes, {} control values, {} session codes and {} snapped codes, not {codes}, {}, {codes} and {codes}",
                read.depth,
                read.control_codes.len(),
                read.control_values.len(),
                read.integer_codes.len(),
                read.snapped_codes.len(),
                codes * head_width
            ));
        }
    }
    let depths: Vec<usize> = reads.iter().map(|r| r.depth).collect();
    check_copy_trace(float, "float-d11", vocab, heads, &depths)?;
    check_copy_trace(integer, "integer", vocab, heads, &depths)?;
    Ok(CopyCodeCheck {
        vocab,
        heads,
        reads: reads
            .iter()
            .enumerate()
            .map(|(r, read)| CopyCodeDepth {
                depth: read.depth,
                control_vs_integer: control_against_integer(read, copy, head_width),
                float: copy_trace(float, r, read, copy, head_width),
                integer: copy_trace(integer, r, read, copy, head_width),
            })
            .collect(),
    })
}

/// Why the `side` trace does not line up with the artifact, if it does not.
fn check_copy_trace(
    trace: &HammingTrace,
    side: &str,
    vocab: usize,
    heads: usize,
    depths: &[usize],
) -> Result<(), String> {
    if trace.schema != HAMMING_TRACE_SCHEMA {
        return Err(format!(
            "the {side} trace's schema is {}, not {HAMMING_TRACE_SCHEMA}",
            trace.schema
        ));
    }
    if trace.side != side {
        return Err(format!(
            "the {side} trace was written by the {} side",
            trace.side
        ));
    }
    if trace.heads != heads {
        return Err(format!(
            "the {side} trace holds {} heads, the artifact {heads}",
            trace.heads
        ));
    }
    if trace.windows.is_empty() {
        return Err(format!("the {side} trace holds no window"));
    }
    for (w, window) in trace.windows.iter().enumerate() {
        let time = window.ids.len();
        if let Some(id) = window.ids.iter().find(|&&id| id as usize >= vocab) {
            return Err(format!(
                "the {side} trace's window {w}: id {id} is not one of {vocab} entries"
            ));
        }
        if window
            .reads
            .iter()
            .map(|r| r.depth)
            .ne(depths.iter().copied())
        {
            return Err(format!(
                "the {side} trace's window {w}: reads at depths {:?}, the artifact's at {depths:?}",
                window.reads.iter().map(|r| r.depth).collect::<Vec<_>>()
            ));
        }
        for read in &window.reads {
            let Some(copies) = read.heads.get(..heads / 2) else {
                return Err(format!(
                    "the {side} trace's window {w}, depth {}: {} heads, fewer than the {} copy heads",
                    read.depth,
                    read.heads.len(),
                    heads / 2
                ));
            };
            if copies
                .iter()
                .any(|h| h.query.len() != time || h.key.len() != time)
            {
                return Err(format!(
                    "the {side} trace's window {w}, depth {}: copy codes not one per position of {time}",
                    read.depth
                ));
            }
        }
    }
    Ok(())
}

/// The control's codes against the session's at one read.
fn control_against_integer(
    read: &CopyCodeRead,
    copy: usize,
    head_width: usize,
) -> CopyControlAgainstInteger {
    let mut out = CopyControlAgainstInteger {
        codes: read.control_codes.len(),
        value_bins: vec![0; COPY_VALUE_BINS],
        ..Default::default()
    };
    for (i, (&control, &served)) in read
        .control_codes
        .iter()
        .zip(&read.integer_codes)
        .enumerate()
    {
        let diff = control ^ served;
        out.equal += usize::from(diff == 0);
        out.differing_bits += diff.count_ones() as usize;
        for bit in set_bits(diff) {
            let value = read.control_values[i * head_width + bit as usize];
            out.value_bins[copy_value_bin(value)] += 1;
            out.max_value = out.max_value.max(value.abs());
            if out.listed.len() < MAX_FAULTS_LISTED {
                out.listed.push(CopyCodeBit {
                    token: (i / copy) as u32,
                    head: i % copy,
                    bit,
                    value,
                });
            }
        }
    }
    out
}

/// The copy-head codes `trace` holds at its `r`-th read, against `read`'s two sides' codes.
fn copy_trace(
    trace: &HammingTrace,
    r: usize,
    read: &CopyCodeRead,
    copy: usize,
    head_width: usize,
) -> CopyCodeTrace {
    let samples = || CopyCodeSamples {
        bits_from_control: vec![0; 65],
        bits_from_integer: vec![0; 65],
        value_bins: vec![0; COPY_VALUE_BINS],
        ..Default::default()
    };
    let (mut query, mut key) = (samples(), samples());
    let mut codes: BTreeMap<(u32, usize), BTreeSet<u64>> = BTreeMap::new();
    let mut note = |into: &mut CopyCodeSamples, token: u32, head: usize, code: u64| {
        let i = token as usize * copy + head;
        let (control, served) = (read.control_codes[i], read.integer_codes[i]);
        into.samples += 1;
        into.equal_control += usize::from(code == control);
        into.equal_integer += usize::from(code == served);
        into.equal_snapped += usize::from(code == read.snapped_codes[i]);
        into.bits_from_control[(code ^ control).count_ones() as usize] += 1;
        into.bits_from_integer[(code ^ served).count_ones() as usize] += 1;
        for bit in set_bits(code ^ control) {
            into.value_bins[copy_value_bin(read.control_values[i * head_width + bit as usize])] +=
                1;
        }
        codes.entry((token, head)).or_default().insert(code);
    };
    for window in &trace.windows {
        for (head, h) in window.reads[r].heads[..copy].iter().enumerate() {
            for (t, &token) in window.ids.iter().enumerate() {
                note(&mut query, token, head, h.query[t]);
                if t + 1 < window.ids.len() {
                    note(&mut key, token, head, h.key[t + 1]);
                }
            }
        }
    }
    CopyCodeTrace {
        side: trace.side.clone(),
        arm: trace.arm.clone(),
        query,
        key,
        pairs: codes.len(),
        inconsistent: codes.values().filter(|c| c.len() > 1).count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// claude-237: the control turns each lane on the left by its gate's unit and packs each copy
    /// head's coordinates in coordinate order, laid out by entry and then copy head. Lane 0's
    /// gate `2i` turns `(a, b, c, d)` to `(−b, a, −d, c)`; lane 1 is past the copy heads.
    #[test]
    fn float_copy_codes_turn_each_lane_and_pack_each_copy_head() {
        let gate = [0.0, 2.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
        let table = [
            0.5, -0.25, 1.0, -2.0, 9.0, 9.0, 9.0, 9.0, //
            -1.0, 0.0, 3.0, -0.5, 9.0, 9.0, 9.0, 9.0,
        ];
        let (codes, values) = float_copy_codes(&gate, &table, 2, 8, 4).expect("the control");
        assert_eq!(values, [0.25, 0.5, 2.0, 1.0, 0.0, -1.0, 0.5, 3.0]);
        assert_eq!(codes, [0b11, 0b11, 0b01, 0b11], "entry, then copy head");
        assert!(
            float_copy_codes(&gate, &table, 2, 8, 3).is_err(),
            "odd heads"
        );
        assert!(
            float_copy_codes(&gate[..4], &table, 2, 8, 4).is_err(),
            "short rotation"
        );
        assert!(
            float_copy_codes(&gate, &table[..8], 2, 8, 4).is_err(),
            "short table"
        );
    }

    /// claude-237: the 120 roots are distinct unit quaternions; each, at any scale, snaps to
    /// itself; a lane near one snaps to it; and an all-zero lane takes the identity.
    #[test]
    fn the_snap_takes_the_nearest_of_the_120_icosians() {
        let roots = icosian_roots_f32();
        for (i, a) in roots.iter().enumerate() {
            let norm: f32 = a.iter().map(|x| x * x).sum();
            assert!((norm - 1.0).abs() < 1e-6, "root {i} has norm² {norm}");
            for b in &roots[i + 1..] {
                let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
                assert!(dot < 0.9, "root {i} repeats");
            }
            assert_eq!(snap_icosian_f32(&a.map(|x| 2.5 * x)), a);
        }
        // An identity-near lane, a lane near `(½)⁴`, then a zero lane.
        let lanes = [
            1.072_641_3,
            0.141_984_52,
            0.008_908_987,
            -0.047_924_876,
            0.6,
            0.4,
            0.5,
            0.55,
            0.0,
            0.0,
            0.0,
            0.0,
        ];
        let snapped = snap_icosian_f32(&lanes);
        assert_eq!(
            snapped,
            [1.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.5, 0.5, 1.0, 0.0, 0.0, 0.0]
        );
    }

    /// The planted inputs of the copy-code tests: 5 entries, width 8, 2 heads (one copy head of
    /// 4 coordinates), one read at depth 2, and one window. The session's code of entry 3 has bit
    /// 2 flipped, where the control's value is `3e-5`; the float trace's query at position 3
    /// (entry 3) has bit 1 flipped, where the control's value is `0.2`; the snapped control's
    /// code of entry 1 has bit 0 set.
    fn planted() -> (Vec<CopyCodeRead>, HammingTrace, HammingTrace) {
        let control: Vec<u64> = vec![0b0001, 0b0010, 0b0100, 0b1000, 0b1111];
        let mut values = vec![0.5f32; 20];
        values[3 * 4 + 2] = 3e-5;
        values[3 * 4 + 1] = 0.2;
        let mut served = control.clone();
        served[3] ^= 0b100;
        let ids: Vec<u32> = vec![0, 1, 2, 3, 4, 3, 1];
        let trace = |side: &str, codes: &[u64]| {
            let key = (0..ids.len())
                .map(|s| codes[ids[s.saturating_sub(1)] as usize])
                .collect();
            HammingTrace {
                schema: HAMMING_TRACE_SCHEMA.to_owned(),
                side: side.to_owned(),
                arm: side.to_owned(),
                heads: 2,
                kept: 16,
                windows: vec![HammingTraceWindow {
                    ids: ids.clone(),
                    reads: vec![HammingTraceRead {
                        depth: 2,
                        heads: vec![HammingTraceHead {
                            query: ids.iter().map(|&v| codes[v as usize]).collect(),
                            key,
                            kept: vec![Vec::new(); ids.len()],
                        }],
                    }],
                }],
            }
        };
        let mut float = trace("float-d11", &control);
        float.windows[0].reads[0].heads[0].query[3] ^= 0b10;
        let integer = trace("integer", &served);
        let mut snapped = control.clone();
        snapped[1] |= 0b1;
        let read = CopyCodeRead {
            depth: 2,
            control_codes: control,
            control_values: values,
            integer_codes: served,
            snapped_codes: snapped,
        };
        (vec![read], float, integer)
    }

    /// claude-237: the check finds the planted differences and nothing else: the session's one
    /// bit, with the control's value there and its bin; the float trace's one flipped query, and
    /// the pair it makes inconsistent; and the integer trace's agreement with its own side.
    #[test]
    fn the_copy_code_check_finds_each_planted_difference() {
        let (reads, float, integer) = planted();
        let check = copy_code_check(5, 8, 2, &reads, &float, &integer).expect("the check");
        assert_eq!((check.vocab, check.heads, check.reads.len()), (5, 2, 1));
        let read = &check.reads[0];
        assert_eq!(read.depth, 2);
        let against = &read.control_vs_integer;
        assert_eq!(
            (against.codes, against.equal, against.differing_bits),
            (5, 4, 1)
        );
        assert_eq!(
            against.value_bins,
            [0, 0, 1, 0, 0, 0, 0],
            "3e-5 is below 1e-4"
        );
        assert_eq!(against.max_value, 3e-5);
        let bit = CopyCodeBit {
            token: 3,
            head: 0,
            bit: 2,
            value: 3e-5,
        };
        assert_eq!(against.listed, [bit]);

        let float = &read.float;
        assert_eq!((float.query.samples, float.query.equal_control), (7, 6));
        assert_eq!(float.query.equal_integer, 5, "entry 3 at positions 3 and 5");
        assert_eq!(
            float.query.equal_snapped, 4,
            "entry 1 at positions 1 and 6, and position 3"
        );
        assert_eq!(&float.query.bits_from_control[..3], [6, 1, 0]);
        assert_eq!(&float.query.bits_from_integer[..3], [5, 1, 1]);
        assert_eq!(
            float.query.value_bins,
            [0, 0, 0, 0, 0, 0, 1],
            "0.2 at bit 1"
        );
        assert_eq!((float.key.samples, float.key.equal_control), (6, 6));
        assert_eq!((float.pairs, float.inconsistent), (5, 1));

        let integer = &read.integer;
        assert_eq!((integer.query.samples, integer.query.equal_integer), (7, 7));
        assert_eq!(integer.query.equal_control, 5);
        assert_eq!(integer.query.equal_snapped, 3, "entries 0, 2 and 4");
        assert_eq!(integer.query.value_bins, [0, 0, 2, 0, 0, 0, 0]);
        assert_eq!((integer.key.samples, integer.key.equal_control), (6, 4));
        assert_eq!((integer.pairs, integer.inconsistent), (5, 0));
    }

    /// claude-237: inputs that do not line up are refused, each for its own reason.
    #[test]
    fn the_copy_code_check_refuses_inputs_that_do_not_line_up() {
        type Plant = fn(&mut Vec<CopyCodeRead>, &mut HammingTrace, &mut HammingTrace);
        let cases: [(&str, Plant); 10] = [
            ("schema", |_, f, _| f.schema = "other".to_owned()),
            ("written by", |_, f, _| f.side = "integer".to_owned()),
            ("heads, the artifact", |_, _, i| i.heads = 4),
            ("depths", |_, f, _| f.windows[0].reads[0].depth = 3),
            ("not one of 5", |_, f, _| f.windows[0].ids[2] = 5),
            ("fewer than", |_, f, _| f.windows[0].reads[0].heads.clear()),
            ("one per position", |_, _, i| {
                i.windows[0].reads[0].heads[0].key.pop();
            }),
            ("session codes", |r, _, _| {
                r[0].integer_codes.pop();
            }),
            ("snapped codes", |r, _, _| {
                r[0].snapped_codes.pop();
            }),
            ("no window", |_, f, _| f.windows.clear()),
        ];
        for (why, plant) in cases {
            let (mut reads, mut float, mut integer) = planted();
            plant(&mut reads, &mut float, &mut integer);
            let error = copy_code_check(5, 8, 2, &reads, &float, &integer).expect_err(why);
            assert!(error.contains(why), "{why}: {error}");
        }
        let (reads, float, integer) = planted();
        assert!(copy_code_check(5, 8, 3, &reads, &float, &integer)
            .expect_err("odd heads")
            .contains("even count"));
        assert!(copy_code_check(5, 8, 2, &[], &float, &integer)
            .expect_err("no read")
            .contains("no read"));
    }

    #[test]
    fn write_new_file_never_replaces_and_never_leaves_a_partial_file() {
        let dir = std::env::temp_dir().join(format!("uor-fbs-write-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("out.json");
        let partial = dir.join("out.json.partial");
        let _ = std::fs::remove_file(&path);
        write_new_file(&path, b"first").expect("a new file is written");
        assert_eq!(std::fs::read(&path).expect("read"), b"first");
        assert!(!partial.exists());
        // An existing file is refused and kept as it was.
        assert!(write_new_file(&path, b"second").is_err());
        assert_eq!(std::fs::read(&path).expect("read"), b"first");
        assert!(!partial.exists());
        std::fs::remove_dir_all(&dir).expect("clean up");
    }

    #[test]
    fn float_readout_check_refuses_an_incomplete_or_misordered_readout() {
        let readout = |tokens: &[u32], mode: FbsReadoutMode| FbsFloatReadout {
            vocab: 3,
            width: 2,
            mode,
            positions: tokens
                .iter()
                .map(|&token| FbsFloatPosition {
                    token,
                    next: 0,
                    hidden: vec![0.0; 2],
                })
                .collect(),
            sequences: vec![FbsFloatSequence {
                tokens: vec![2, 0],
                next: vec![1, 1],
                hidden: vec![vec![0.0; 2]; 2],
            }],
        };
        assert!(readout(&[0, 1, 2], FbsReadoutMode::Plain)
            .check(3, 2, 4, FbsReadoutMode::Plain)
            .is_ok());
        assert!(readout(&[0, 1, 2], FbsReadoutMode::Export)
            .check(3, 2, 4, FbsReadoutMode::Export)
            .is_ok());
        for (what, r, reason) in [
            ("empty", readout(&[], FbsReadoutMode::Plain), "0 positions"),
            (
                "a token missing",
                readout(&[0, 1], FbsReadoutMode::Plain),
                "2 positions",
            ),
            (
                "a token repeated",
                readout(&[0, 1, 1], FbsReadoutMode::Plain),
                "position 2 is not token 2",
            ),
            (
                "out of order",
                readout(&[1, 0, 2], FbsReadoutMode::Plain),
                "position 0 is not token 0",
            ),
            (
                "the other mode",
                readout(&[0, 1, 2], FbsReadoutMode::Export),
                "mode Export",
            ),
        ] {
            match r.check(3, 2, 4, FbsReadoutMode::Plain) {
                Err(e) => assert!(e.contains(reason), "{what}: {e}"),
                Ok(()) => panic!("{what} must be refused"),
            }
        }
        let mut short = readout(&[0, 1, 2], FbsReadoutMode::Plain);
        short.positions[1].hidden.pop();
        assert!(short
            .check(3, 2, 4, FbsReadoutMode::Plain)
            .is_err_and(|e| e.contains("position 1")));
        assert!(readout(&[0, 1, 2], FbsReadoutMode::Plain)
            .check(4, 2, 4, FbsReadoutMode::Plain)
            .is_err());
        let mut none = readout(&[0, 1, 2], FbsReadoutMode::Plain);
        none.sequences.clear();
        assert!(none
            .check(3, 2, 4, FbsReadoutMode::Plain)
            .is_err_and(|e| e.contains("no sequences")));
        // A sequence must be 1 to `context` in-vocabulary tokens, each read.
        type Seq = fn(&mut FbsFloatSequence);
        let bad: [(&str, Seq); 5] = [
            ("empty", |q| {
                q.tokens.clear();
                q.next.clear();
                q.hidden.clear();
            }),
            ("past the context", |q| {
                q.tokens = vec![0; 5];
                q.next = vec![0; 5];
                q.hidden = vec![vec![0.0; 2]; 5];
            }),
            ("a token out of the vocabulary", |q| q.tokens[1] = 3),
            ("a position not read", |q| {
                q.next.pop();
            }),
            ("a short hidden vector", |q| {
                q.hidden[1].pop();
            }),
        ];
        for (what, mutate) in bad {
            let mut r = readout(&[0, 1, 2], FbsReadoutMode::Plain);
            mutate(&mut r.sequences[0]);
            match r.check(3, 2, 4, FbsReadoutMode::Plain) {
                Err(e) => assert!(e.contains("sequence 0"), "{what}: {e}"),
                Ok(()) => panic!("{what} must be refused"),
            }
        }
    }

    #[test]
    fn json_f32_round_trips_bit_exactly() {
        // The snapshot hands every f32 over as JSON; each must read back to
        // the same bits, signed zeros, subnormals and the extremes included.
        let mut values = vec![
            0.0f32,
            -0.0,
            f32::MIN_POSITIVE,
            -f32::MIN_POSITIVE,
            f32::from_bits(1),
            f32::from_bits(0x8000_0001),
            f32::from_bits(0x007f_ffff),
            f32::MAX,
            f32::MIN,
            f32::EPSILON,
            1.0 / 3.0,
            0.1,
        ];
        let mut state = 0x2026_1026_u64;
        while values.len() < 200_000 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let v = f32::from_bits((state >> 32) as u32);
            if v.is_finite() {
                values.push(v);
            }
        }
        let json = serde_json::to_vec(&values).expect("serialize");
        let back: Vec<f32> = serde_json::from_slice(&json).expect("deserialize");
        let changed = values
            .iter()
            .zip(&back)
            .filter(|(a, b)| a.to_bits() != b.to_bits())
            .count();
        assert_eq!(
            changed,
            0,
            "{changed} of {} values changed bits",
            values.len()
        );
    }
}
