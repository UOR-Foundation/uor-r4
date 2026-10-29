//! Long training of the SpiralCore/FBS geometric trunk in its own space, with text at every
//! checkpoint.
//!
//! The model is the FBS sheet's trunk, `spiralcore_fbs_sheet::build_fbs_model(OneAtomActive)`.
//! - **Space:** 6 recurrence layers with the Hamilton gate and SpiralCore/FBS projections.
//! - **One table:** the native vector writer's table encodes the input token and holds every
//!   output token's vector.
//! - **Loss:** the writer's `MarginHinge` on the L1 displacement between the predicted point and
//!   the target's own vector, so training happens in the same space as transport and prediction.
//! - **Tokens:** they appear only at the writer's final nearest pick.
//!
//! The sheet trained this trunk for 1,024 steps of 4 windows: about 4,096 of TRAIN's ~39k usable
//! windows, seen once. This runner instead trains for `steps` steps, over epochs of every usable
//! window, each epoch in its own seeded order. Every `eval_every` steps it does three things:
//! 1. scores the whole DEV panel;
//! 2. writes greedy replies to fixed DEV prompts as text, beside the true replies;
//! 3. saves the model.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{DType, Device, Tensor};
use serde::Serialize;
use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::geometric_stack::stage_clock::{self, StageTime};
use crate::geometric_stack::{StackAdamW, StackModel};
use crate::hamilton_sheet::{self as hs, CausalWindow, HamiltonArm, SHEET_CONTEXT};
use crate::spiralcore_fbs_arms::{self as arms, FbsArm};
use crate::spiralcore_fbs_sheet::{build_fbs_model_at, panel_loss};
use crate::{invalid, sha256_bytes, Result};

/// What the trunk's predicted point is trained against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PointObjective {
    /// The writer's own margin: the token hinge plus the address hinge at margin 1. The 16 roots
    /// cannot meet that margin: by the triangle inequality a cell beats the cell sharing its other
    /// port by at most the nearest root separation, 0.5 (`root_separation`).
    Sheet,
    /// The same token hinge, with the address hinge's margin at half the nearest root separation,
    /// which a query port meets inside its target root's Voronoi cell.
    Voronoi,
    /// No admission: the target must be nearer than the `SHEET_RIVALS` nearest wrong tokens of the
    /// whole table, and the reply is the whole table's nearest token.
    Nearest,
    /// The target token's cell: the predicted point must be a margin nearer the target than the
    /// whole table's single nearest wrong token, `relu(1 + max_r s(r) − s(target))`, so only the
    /// rival that would win is pushed. Decoded as [`PointObjective::Nearest`] is (claude-152).
    Cell,
}

impl PointObjective {
    /// Whether replies decode as the whole table's nearest token, not the writer's pick.
    pub fn decodes_nearest(self) -> bool {
        matches!(self, Self::Nearest | Self::Cell)
    }
}

/// One TRAIN split other than the pinned one, named by its directory and checked by the sha256s
/// of its `tokens.u16` and `response_mask.u8` (claude-152).
#[derive(Clone, Debug, Serialize)]
pub struct TrainSplit {
    pub dir: PathBuf,
    pub tokens_sha256: String,
    pub mask_sha256: String,
}

/// One run's settings.
#[derive(Clone, Debug, Serialize)]
pub struct LongConfig {
    pub objective: PointObjective,
    pub steps: usize,
    pub batch: usize,
    pub eval_every: usize,
    pub seed: u64,
    pub lr: f64,
    /// The learning rate falls linearly from `lr` at step 0 to `lr_end` at the last step.
    pub lr_end: f64,
    /// The `rec.decay` parameters' rate as a multiple of the main rate (the sheet's is 20).
    pub decay_mult: f64,
    /// Read the recurrence's rate table as retention (`geometric_stack::set_retention_rates`).
    pub retain_rates: bool,
    /// Fold the recurrence's rate score into [0, 1] (`geometric_stack::set_folded_scores`).
    pub fold_scores: bool,
    /// A reflecting floor and clamped ceiling for the rate score (`geometric_stack::set_reflected_floor`).
    pub reflect_floor: bool,
    /// Keep this many earlier positions in a Flock read on the trunk's points; 0 is no read
    /// (`flock_read`).
    pub flock_read: usize,
    /// Keep this many earlier positions in a content-keyed read inside the trunk; 0 is no read
    /// (`flock_read_content`).
    pub content_read: usize,
    /// Hamming flock attention with this many heads after layer 2 and on the readout; 0 is no read
    /// (`flock_read_hamming`).
    pub hamming_read: usize,
    /// The opening read (claude-154) after layer 2: the mean of up to this many opening token
    /// points; 0 is no read (`flock_read_opening`). Left out of `config.json` when 0.
    #[serde(skip_serializing_if = "is_zero")]
    pub opening_read: usize,
    /// Where the opening read's slots start; set exactly when `opening_read` is, and left out of
    /// `config.json` when none, so a run without the read writes the same file as before.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opening_anchor: Option<crate::flock_read_opening::OpeningAnchor>,
    /// The prompt register (claude-155) after layer 2 and on the readout: this many slots over
    /// the current turn's span; 0 is no register (`flock_read_register`). Left out of
    /// `config.json` when 0.
    #[serde(skip_serializing_if = "is_zero")]
    pub prompt_register: usize,
    /// How many span positions each register slot weighs; set exactly when `prompt_register` is,
    /// and left out of `config.json` when none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_picks: Option<usize>,
    /// How the register's slots choose their positions; set exactly when `prompt_register` is,
    /// and left out of `config.json` when none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_select: Option<crate::flock_read_register::RegisterSelect>,
    /// The trunk's width: the shared table's columns and every read's width (default 256,
    /// `hamilton_sheet::SHEET_WIDTH`); a multiple of 8.
    pub width: usize,
    /// The Hamming read's type heads compare positions through SpiralCore's path-dependent
    /// icosian transport (`HammingReads::with_type_spin`, claude-151); off by default.
    pub type_spin: bool,
    /// Pass no gradient from the input encoding into the shared table
    /// (`geometric_stack::set_input_table_fixed`); it changes training only, never a forward.
    pub fix_input_table: bool,
    /// Train on this split in place of the pinned TRAIN; DEV stays the pinned one.
    pub train_split: Option<TrainSplit>,
    pub samples: usize,
    pub reply_tokens: usize,
    /// Profile this many training steps by stage (fbs-cost-profile, claude-157): see
    /// [`profile_window`]. 0 is off, and left out of `config.json`.
    #[serde(skip_serializing_if = "is_zero")]
    pub profile: usize,
    /// The turn memory (route-plus-memory, claude-161): key each reply opening on this many ids
    /// of its turn; 0 is no memory, and left out of `config.json`.
    #[serde(skip_serializing_if = "is_zero")]
    pub turn_memory: usize,
    /// The turn memory with an empty table, the no-memory control (L-010); left out when false.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub turn_memory_empty: bool,
    /// The turn memory's reply table (turn-memory-reply, claude-193): the rows past each reply's
    /// opening keyed by `turn-8 + position + reply-last-1` at stored share at least
    /// [`REPLY_MEMORY_SHARE`]; needs `turn_memory`, and is emptied by `turn_memory_empty`. Left
    /// out when false.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub turn_memory_reply: bool,
}

/// The stored share the reply table keeps its keys at: the `--reply-bound` probe's choice
/// (claude-192).
pub const REPLY_MEMORY_SHARE: f64 = 0.5;

/// One checkpoint's reading, one line of `progress.jsonl`.
#[derive(Clone, Debug, Serialize)]
pub struct Checkpoint {
    pub step: usize,
    pub epoch: f64,
    pub train_loss_mean: Option<f64>,
    /// The writer's own DEV panel (`hamilton_sheet::evaluate_dev_panel`): its top-1, top-8 and
    /// admitted counts, and `dev_loss` with the token and address margins below, read the trunk
    /// without the Flock read (a content read, being inside the trunk, is in them). `dev_nearest`
    /// goes through [`crate::flock_read::points`].
    pub dev_top1_hits: usize,
    pub dev_top8_hits: usize,
    pub dev_admitted_hits: usize,
    /// DEV decoded as the whole table's nearest token to the predicted point.
    pub dev_nearest: NearestReading,
    /// DEV decoded by the writer, the served decoder, on the same points and rows as
    /// `dev_nearest`; its `hits` must equal `dev_top1_hits`.
    pub dev_writer: WriterReading,
    pub dev_scored_rows: usize,
    pub dev_loss: f64,
    /// DEV's token hinge: the target's displacement score against the best admitted rivals.
    pub dev_token_margin: f64,
    /// DEV's address hinge: the target's cell against the best other admitted cell.
    pub dev_address_margin: f64,
    pub train_seconds: f64,
    /// Every context position of every trained window per second (not scored targets).
    pub train_context_tokens_per_second: f64,
}

/// One saved checkpoint's DEV reading, with the loss in its two parts and how varied the
/// predicted points are across DEV rows (`inspect_run`).
#[derive(Clone, Debug, Serialize)]
pub struct Inspection {
    pub checkpoint: String,
    pub scored_rows: usize,
    pub top1_hits: usize,
    pub nearest: NearestReading,
    pub admitted_hits: usize,
    pub token_unviolated_hits: usize,
    pub token_margin: f64,
    pub address_margin: f64,
    /// Over the first DEV windows' scored rows: how many distinct cells are admitted first.
    pub probe_rows: usize,
    pub distinct_first_cells: usize,
    /// Each query dimension's standard deviation across those rows, averaged over dimensions.
    pub query_spread: f64,
    pub query_mean_abs: f64,
    pub table_mean_abs: f64,
    /// The Flock read's per-coordinate weight, mean of its absolute values, when the checkpoint
    /// has a read.
    pub flock_weight_mean_abs: Option<f64>,
    /// The same for the content read, when the checkpoint has one.
    pub content_weight_mean_abs: Option<f64>,
    /// The same for the Hamming reads, over both depths, when the checkpoint has them.
    pub hamming_weight_mean_abs: Option<f64>,
    /// The same for the opening read (claude-154), when the checkpoint has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opening_weight_mean_abs: Option<f64>,
    /// The same for the prompt register (claude-155), when the checkpoint has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_weight_mean_abs: Option<f64>,
    /// The last checkpoint's nearest-token DEV reading with each register slot dropped in turn,
    /// then all of them, when it has a register (claude-155).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_drop: Option<Vec<RegisterDrop>>,
    /// The turn memory's weight (claude-161), when the checkpoint has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_weight_mean_abs: Option<f64>,
    /// The turn memory's reply weight (claude-193), when its memory has a reply table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_reply_weight_mean_abs: Option<f64>,
    /// The last checkpoint's nearest reading with the turn memory taken out (claude-161).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_drop: Option<RegisterDrop>,
}

/// The nearest-token DEV reading with one prompt-register slot, or every slot, dropped
/// ([`PromptRegister::without_slot`](crate::flock_read_register::PromptRegister::without_slot)).
#[derive(Clone, Debug, Serialize)]
pub struct RegisterDrop {
    /// The dropped slot's index, or `"all"`.
    pub slot: serde_json::Value,
    pub hits: usize,
    pub opening_hits: usize,
    pub copied_hits: usize,
}

/// One fixed DEV prompt with the model's greedy reply and the true reply, as text.
#[derive(Clone, Debug, Serialize)]
pub struct Sample {
    pub step: usize,
    pub prompt: String,
    pub model: String,
    pub reference: String,
}

/// Clears the arm hook and the retention reading when the run ends, by any path.
struct ArmGuard;

impl Drop for ArmGuard {
    fn drop(&mut self) {
        arms::set_fbs_arm(None);
        crate::geometric_stack::set_retention_rates(false);
        crate::geometric_stack::set_folded_scores(false);
        crate::geometric_stack::set_reflected_floor(false);
        crate::geometric_stack::set_input_table_fixed(false);
        crate::flock_read::set_flock_read(None);
        crate::flock_read_content::set_content_read(None);
        crate::flock_read_hamming::set_hamming_reads(None);
        crate::flock_read_opening::set_opening_read(None);
        crate::flock_read_register::set_prompt_register(None);
        crate::flock_read_register::set_register_slots(None);
        crate::flock_read_memory::set_turn_memory(None);
        crate::flock_read_memory::set_memory_rows(None);
        crate::flock_read_memory::set_reply_rows(None);
    }
}

/// Set this thread's Flock read, content read, Hamming reads, opening read and prompt register
/// from the checkpoint `directory`, each to none when the checkpoint has none.
fn load_reads(directory: &Path, device: &Device) -> Result<()> {
    crate::flock_read::set_flock_read(crate::flock_read::FlockRead::load(directory, device)?);
    crate::flock_read_content::set_content_read(crate::flock_read_content::ContentRead::load(
        directory, device,
    )?);
    crate::flock_read_hamming::set_hamming_reads(crate::flock_read_hamming::HammingReads::load(
        directory, device,
    )?);
    crate::flock_read_opening::set_opening_read(crate::flock_read_opening::OpeningRead::load(
        directory, device,
    )?);
    crate::flock_read_register::set_prompt_register(
        crate::flock_read_register::PromptRegister::load(directory, device)?,
    );
    crate::flock_read_memory::set_turn_memory(crate::flock_read_memory::TurnMemory::load(
        directory, device,
    )?);
    Ok(())
}

/// The Hamming reads `config` asks for, set on this thread, with their optimizer; none when off.
fn start_hamming(
    config: &LongConfig,
    device: &Device,
) -> Result<
    Option<(
        crate::flock_read_hamming::HammingReads,
        crate::flock_read::FlockAdamW,
    )>,
> {
    if config.hamming_read == 0 {
        return Ok(None);
    }
    let mut reads =
        crate::flock_read_hamming::HammingReads::new(config.hamming_read, config.width, device)?;
    if config.type_spin {
        reads = reads.with_type_spin()?;
    }
    crate::flock_read_hamming::set_hamming_reads(Some(reads.clone()));
    let opt = crate::flock_read::FlockAdamW::for_vars(&reads.variables())?;
    Ok(Some((reads, opt)))
}

/// The opening read `config` asks for (claude-154), set on this thread, with its own optimizer;
/// none when off.
fn start_opening(
    config: &LongConfig,
    device: &Device,
) -> Result<
    Option<(
        crate::flock_read_opening::OpeningRead,
        crate::flock_read::FlockAdamW,
    )>,
> {
    if config.opening_read == 0 {
        return Ok(None);
    }
    let read = crate::flock_read_opening::OpeningRead::new(
        config.opening_read,
        config.width,
        config.opening_anchor.unwrap_or_default(),
        device,
    )?;
    crate::flock_read_opening::set_opening_read(Some(read.clone()));
    let opt = crate::flock_read::FlockAdamW::for_vars(&read.variables())?;
    Ok(Some((read, opt)))
}

/// The prompt register `config` asks for (claude-155), set on this thread, with its own
/// optimizer; none when off.
fn start_register(
    config: &LongConfig,
    device: &Device,
) -> Result<
    Option<(
        crate::flock_read_register::PromptRegister,
        crate::flock_read::FlockAdamW,
    )>,
> {
    if config.prompt_register == 0 {
        return Ok(None);
    }
    let reg = crate::flock_read_register::PromptRegister::new(
        config.prompt_register,
        config.register_picks.unwrap_or(DEFAULT_REGISTER_PICKS),
        config.register_select.unwrap_or_default(),
        config.width,
        device,
    )?;
    crate::flock_read_register::set_prompt_register(Some(reg.clone()));
    let opt = crate::flock_read::FlockAdamW::for_vars(&reg.variables())?;
    Ok(Some((reg, opt)))
}

/// How many span positions a register slot weighs when the run names none (claude-155).
pub const DEFAULT_REGISTER_PICKS: usize = 8;

/// How many of the opening rows `rows` (each a window's index into `windows` and its opening
/// offsets) the memory's own key finds in its table: `(rows, hits)`.
fn memory_hits(
    memory: &crate::flock_read_memory::TurnMemory,
    windows: &[CausalWindow],
    rows: &[(usize, Vec<usize>)],
) -> Result<(usize, usize)> {
    let (mut total, mut hits) = (0, 0);
    for (index, offsets) in rows {
        if offsets.is_empty() {
            continue;
        }
        let ids = &windows[*index].input_ids;
        let (_, hit) = memory.stored(ids, ids.len(), offsets)?;
        total += offsets.len();
        hits += hit.iter().filter(|&&h| h > 0.0).count();
    }
    Ok((total, hits))
}

/// A turn memory as a run starts it, with its own optimizer.
type MemoryStart = (
    crate::flock_read_memory::TurnMemory,
    crate::flock_read::FlockAdamW,
);

/// The turn memory `config` asks for (claude-161), set on this thread, with its own optimizer,
/// and each of `usable`'s windows' opening offsets (the input offsets that predict a reply's
/// first token, by the split's response mask); none and no offsets when off. The table is built
/// from every TRAIN window, as the `--opening-bound` probe builds it, or left empty for the
/// no-memory control. With `turn_memory_reply` it also has the reply table (claude-193), built
/// from TRAIN's rows past their openings as the `--reply-bound` probe counted them, and returns
/// each usable window's rows past their openings with their keys.
fn start_memory(
    data_root: &Path,
    config: &LongConfig,
    usable: &[CausalWindow],
    device: &Device,
) -> Result<(Option<MemoryStart>, Vec<Vec<usize>>, ReplyOffsets)> {
    if config.turn_memory == 0 {
        if config.turn_memory_empty {
            return Err(invalid("--turn-memory-empty needs --turn-memory K"));
        }
        if config.turn_memory_reply {
            return Err(invalid("--turn-memory-reply needs --turn-memory K"));
        }
        return Ok((None, Vec::new(), Vec::new()));
    }
    if config.turn_memory_reply && config.turn_memory != TURN_KEY {
        return Err(invalid(format!(
            "--turn-memory-reply keys on the turn's first {TURN_KEY} ids, as the probe did: it needs --turn-memory {TURN_KEY}"
        )));
    }
    let (tokens, mask) = match config.train_split.as_ref() {
        None => load_train_split(data_root)?,
        Some(split) => load_named_split(split)?,
    };
    let (windows, _) = hs::build_document_windows(&tokens, &mask, SHEET_CONTEXT, hs::SHEET_BOS_ID)?;
    let train = openings(&windows, &mask);
    let table = if config.turn_memory_empty {
        std::collections::HashMap::new()
    } else {
        crate::flock_read_memory::build_table(&train, config.turn_memory)
    };
    println!(
        "[memory] key {} ids; TRAIN openings {}; table keys {}{}",
        config.turn_memory,
        train.len(),
        table.len(),
        if config.turn_memory_empty {
            " (the empty control)"
        } else {
            ""
        }
    );
    let mut memory =
        crate::flock_read_memory::TurnMemory::new(config.turn_memory, table, config.width, device)?;
    // The reply table (claude-193): TRAIN's rows past their openings, at the probe's key and share.
    let mut replies = Vec::new();
    if config.turn_memory_reply {
        let rows: Vec<(Vec<u32>, u32)> = reply_rows(&windows, &mask)
            .into_iter()
            .filter(|row| !row.opening)
            .map(|row| (reply_key(&row), row.target))
            .collect();
        let table = if config.turn_memory_empty {
            std::collections::HashMap::new()
        } else {
            crate::flock_read_memory::build_reply_table(&rows, REPLY_MEMORY_SHARE)
        };
        println!(
            "[memory] reply table: TRAIN rows past their openings {}; keys at share {REPLY_MEMORY_SHARE} {}",
            rows.len(),
            table.len()
        );
        memory = memory.with_reply(table)?;
        replies = reply_offsets(usable, &mask);
    }
    crate::flock_read_memory::set_turn_memory(Some(memory.clone()));
    let opt = crate::flock_read::FlockAdamW::for_vars(&memory.variables())?;
    let offsets = usable
        .iter()
        .map(|w| {
            w.scored_offsets
                .iter()
                .zip(&w.scored_corpus_positions)
                .filter(|(_, &position)| opens_reply(&mask, position))
                .map(|(&offset, _)| offset)
                .collect()
        })
        .collect();
    Ok((Some((memory, opt)), offsets, replies))
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

const ABSENT: f32 = -1.0e30;
/// At most this many F32 elements in one detached whole-table score chunk (256 MiB).
const NEAREST_CHUNK_ELEMENTS: usize = 64 * 1024 * 1024;

fn table(model: &StackModel) -> Result<Tensor> {
    let var = model
        .variables()
        .get("embedding.weight")
        .ok_or_else(|| invalid("the model has no shared vector table"))?;
    Ok(var.as_tensor().clone())
}

/// `relu(margin + rival − target)`: `rival` is (rows, k), `target` is (rows, 1).
fn hinge(rival: &Tensor, target: &Tensor, margin: f64) -> Result<Tensor> {
    Ok(rival.broadcast_sub(target)?.affine(1.0, margin)?.relu()?)
}

/// The `k` best positions of each row of `scores` among those `allowed` (u8, same shape) keeps,
/// as (rows, k), by repeated argmax on a detached copy.
fn best_positions(scores: &Tensor, allowed: &Tensor, k: usize) -> Result<Tensor> {
    let absent = Tensor::full(ABSENT, scores.shape(), scores.device())?;
    let mut left = allowed.where_cond(&scores.detach(), &absent)?;
    let columns = Tensor::arange(0u32, scores.dim(1)? as u32, scores.device())?
        .unsqueeze(0)?
        .broadcast_as(scores.shape())?;
    let mut picks = Vec::with_capacity(k);
    for _ in 0..k {
        let at = left.argmax_keepdim(1)?;
        left = columns.broadcast_eq(&at)?.where_cond(&absent, &left)?;
        picks.push(at);
    }
    Ok(Tensor::cat(&picks, 1)?)
}

/// The negative mean L1 displacement of each `query` row to every table row, without gradient,
/// in row chunks: (rows, vocabulary).
fn nearest_scores(query: &Tensor, table: &Tensor) -> Result<Tensor> {
    let (rows, width) = query.dims2()?;
    let chunk = (NEAREST_CHUNK_ELEMENTS / (table.dim(0)? * width)).max(1);
    let table = table.detach().unsqueeze(0)?;
    let mut parts = Vec::with_capacity(rows.div_ceil(chunk));
    let mut start = 0;
    while start < rows {
        let n = chunk.min(rows - start);
        let q = query.narrow(0, start, n)?.detach().unsqueeze(1)?;
        parts.push(q.broadcast_sub(&table)?.abs()?.mean(2)?.neg()?);
        start += n;
    }
    Ok(Tensor::cat(&parts, 0)?)
}

/// [`PointObjective::Voronoi`]: the writer's token hinge, and its address hinge at `address_margin`.
fn voronoi_loss(
    model: &StackModel,
    query: &Tensor,
    targets: &[u32],
    address_margin: f64,
) -> Result<Tensor> {
    let writer = model.native_vector_writer()?;
    let c = writer.candidates(query)?;
    let rows = targets.len();
    let device = query.device();
    let target_score = query
        .sub(&writer.encode(targets)?)?
        .abs()?
        .mean_keepdim(1)?
        .neg()?;
    let target_ids = Tensor::from_vec(targets.to_vec(), (rows, 1), device)?;
    // Every one of the 256 cells holds 16 real tokens at this vocabulary, so 8 kept cells always
    // leave at least `SHEET_RIVALS` non-target candidates.
    let available = c.token_ids().broadcast_ne(&target_ids)?.mul(c.valid())?;
    let at = best_positions(c.raw_scores(), &available, hs::SHEET_RIVALS)?;
    let token = hinge(&c.raw_scores().gather(&at, 1)?, &target_score, 1.0)?.mean(1)?;
    let cells = crate::native_vector_writer::CELLS as u32;
    let target_cells = Tensor::from_vec(
        targets.iter().map(|t| t % cells).collect::<Vec<_>>(),
        (rows, 1),
        device,
    )?;
    let target_address = c.address_scores().gather(&target_cells, 1)?;
    let admitted = c.address_scores().gather(c.cell_ids(), 1)?;
    let rival_address = c
        .cell_ids()
        .broadcast_ne(&target_cells)?
        .where_cond(&admitted, &Tensor::full(ABSENT, admitted.shape(), device)?)?
        .max_keepdim(1)?;
    let address = hinge(&rival_address, &target_address, address_margin)?;
    Ok(token.mean_all()?.add(&address.mean_all()?)?)
}

/// [`PointObjective::Nearest`] (each of the `rival` scores' hinges, averaged) or
/// [`PointObjective::Cell`] (only the hinge of the best-scoring rival), against `target`.
fn rival_hinge(rival: &Tensor, target: &Tensor, cell: bool) -> Result<Tensor> {
    if cell {
        Ok(hinge(&rival.max_keepdim(1)?, target, 1.0)?.mean_all()?)
    } else {
        Ok(hinge(rival, target, 1.0)?.mean_all()?)
    }
}

/// [`PointObjective::Nearest`] or [`PointObjective::Cell`]: the target against the whole table's
/// `SHEET_RIVALS` nearest wrong tokens. Their choice carries no gradient; their displacement and
/// the target's do.
fn nearest_loss(model: &StackModel, query: &Tensor, targets: &[u32], cell: bool) -> Result<Tensor> {
    let table = table(model)?;
    let (rows, width) = query.dims2()?;
    let device = query.device();
    let scores = nearest_scores(query, &table)?;
    let target_ids = Tensor::from_vec(targets.to_vec(), (rows, 1), device)?;
    let allowed = Tensor::arange(0u32, table.dim(0)? as u32, device)?
        .unsqueeze(0)?
        .broadcast_as(scores.shape())?
        .broadcast_ne(&target_ids)?;
    let rivals = best_positions(&scores, &allowed, hs::SHEET_RIVALS)?;
    stage_clock::mark("objective.table_search");
    let rival_vectors =
        table
            .index_select(&rivals.flatten_all()?, 0)?
            .reshape((rows, hs::SHEET_RIVALS, width))?;
    let rival = query
        .unsqueeze(1)?
        .broadcast_sub(&rival_vectors)?
        .abs()?
        .mean(2)?
        .neg()?;
    let target_score = query
        .sub(&table.index_select(&target_ids.flatten_all()?, 0)?)?
        .abs()?
        .mean_keepdim(1)?
        .neg()?;
    let loss = rival_hinge(&rival, &target_score, cell)?;
    stage_clock::mark("objective.hinge");
    Ok(loss)
}

fn batch_loss(
    model: &StackModel,
    objective: PointObjective,
    ids: &[u32],
    scored: &[u32],
    targets: &[u32],
    batch: usize,
) -> Result<Tensor> {
    let hidden = crate::flock_read::points(model, ids, batch, SHEET_CONTEXT)?;
    let idx = Tensor::from_vec(scored.to_vec(), scored.len(), model.device())?;
    let query = hidden.index_select(&idx, 0)?;
    stage_clock::mark("points.select");
    query_loss(model, objective, &query, targets)
}

/// `objective`'s loss for the predicted points `query`, one row per target.
fn query_loss(
    model: &StackModel,
    objective: PointObjective,
    query: &Tensor,
    targets: &[u32],
) -> Result<Tensor> {
    match objective {
        PointObjective::Sheet => model.native_head_loss(query, targets, hs::SHEET_RIVALS),
        PointObjective::Voronoi => voronoi_loss(model, query, targets, root_separation()?.0 / 2.0),
        PointObjective::Nearest => nearest_loss(model, query, targets, false),
        PointObjective::Cell => nearest_loss(model, query, targets, true),
    }
}

/// Whole-table nearest decoding over DEV. Rows that open a reply are counted apart: their target
/// follows the prompt, not the reply's own earlier words, so only context can name it.
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct NearestReading {
    pub hits: usize,
    pub opening_rows: usize,
    pub opening_hits: usize,
    /// Openings whose target is DEV's most common opening token: what always naming it scores.
    pub commonest_opening_rows: usize,
    /// How many distinct tokens the openings are decoded as.
    pub opening_distinct_picks: usize,
    /// Each query dimension's standard deviation across the opening rows, and across all rows,
    /// averaged over dimensions: whether the prompt moves the opening point at all.
    pub opening_query_spread: f64,
    pub query_spread: f64,
    /// Reply rows that copy the prompt ([`copies_prompt`]): the reply's previous token and its
    /// target stand side by side in the window's prompt, as in a code word or a number the user
    /// gave. Only a model that reads the prompt names these above chance (claude-152).
    pub copied_rows: usize,
    pub copied_hits: usize,
    /// Reply rows that neither open a reply nor copy the prompt.
    pub rest_rows: usize,
    pub rest_hits: usize,
}

/// Whether the reply row whose input sits at window `offset` (target at corpus `position`) copies
/// the window's prompt: the row's previous token and its target stand at two adjacent earlier
/// positions of the window that are both outside any reply. `mask` is the corpus response mask.
fn copies_prompt(
    window: &CausalWindow,
    offset: usize,
    position: usize,
    target: u32,
    mask: &[u8],
) -> bool {
    // The window's offset 0 sits at corpus position `position − 1 − offset`.
    let start = position - 1 - offset;
    let previous = window.input_ids[offset];
    (0..offset.saturating_sub(1)).any(|j| {
        window.input_ids[j] == previous
            && window.input_ids[j + 1] == target
            && mask[start + j] == 0
            && mask[start + j + 1] == 0
    })
}

/// The writer's decoding of the same DEV rows as [`NearestReading`]: the token the served
/// decoder (`native_vector_writer`, the one `fbs-toks` decodes with) picks for each scored row.
/// Its `hits` must equal the writer panel's own top-1 (`dev_top1_hits`) on the same checkpoint.
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct WriterReading {
    pub hits: usize,
    pub opening_hits: usize,
    /// How many distinct tokens the openings are decoded as.
    pub opening_distinct_picks: usize,
}

/// Each column's standard deviation across `rows`' rows, averaged over columns.
fn spread(rows: &Tensor) -> Result<f64> {
    let centred = rows.broadcast_sub(&rows.mean_keepdim(0)?)?;
    Ok(f64::from(
        centred
            .sqr()?
            .mean(0)?
            .sqrt()?
            .mean_all()?
            .to_scalar::<f32>()?,
    ))
}

/// [`NearestReading`] over the DEV windows, and the writer's [`WriterReading`] of the same rows
/// from the same points; `mask` is DEV's response mask, by corpus position.
fn nearest_reading(
    model: &StackModel,
    dev: &[CausalWindow],
    mask: &[u8],
    seed: u64,
    step: usize,
) -> Result<(NearestReading, WriterReading)> {
    let table = table(model)?;
    let writer = model.native_vector_writer()?;
    hs::with_gate_mode(HamiltonArm::HamiltonActive, seed, step, false, || {
        let mut reading = NearestReading::default();
        let mut written = WriterReading::default();
        let mut openings = std::collections::BTreeMap::<u32, usize>::new();
        let mut picks = std::collections::BTreeSet::<u32>::new();
        let mut written_picks = std::collections::BTreeSet::<u32>::new();
        let (mut all_queries, mut opening_queries) = (Vec::new(), Vec::new());
        for chunk in dev.chunks(hs::SHEET_EVAL_BATCH) {
            let windows: Vec<&CausalWindow> = chunk.iter().collect();
            let (ids, scored, targets) = assemble(&windows);
            if scored.is_empty() {
                continue;
            }
            let opens: Vec<bool> = windows
                .iter()
                .flat_map(|w| w.scored_corpus_positions.iter())
                .map(|&p| opens_reply(mask, p))
                .collect();
            let copies: Vec<bool> = windows
                .iter()
                .flat_map(|w| {
                    w.scored_offsets
                        .iter()
                        .zip(&w.scored_corpus_positions)
                        .zip(&w.scored_targets)
                        .map(move |((&o, &p), &t)| copies_prompt(w, o, p, t, mask))
                })
                .collect();
            // The turn memory (claude-161) moves the opening rows only, as training names them.
            if crate::flock_read_memory::turn_memory().is_some() {
                crate::flock_read_memory::set_memory_rows(Some(
                    scored
                        .iter()
                        .zip(&opens)
                        .filter(|(_, &open)| open)
                        .map(|(&row, _)| row as usize)
                        .collect(),
                ));
                // Its reply table (claude-193) moves the later reply rows, as training names them, at
                // `assemble`'s row for each: the window's index times the context, plus the offset.
                if crate::flock_read_memory::turn_memory().is_some_and(|m| m.reply_len().is_some())
                {
                    crate::flock_read_memory::set_reply_rows(Some(
                        reply_rows(chunk, mask)
                            .iter()
                            .filter(|row| !row.opening)
                            .map(|row| (row.window * SHEET_CONTEXT + row.offset, reply_key(row)))
                            .collect(),
                    ));
                }
            }
            let hidden = crate::flock_read::points(model, &ids, windows.len(), SHEET_CONTEXT);
            crate::flock_read_memory::set_memory_rows(None);
            crate::flock_read_memory::set_reply_rows(None);
            let hidden = hidden?;
            let idx = Tensor::from_vec(scored.clone(), scored.len(), model.device())?;
            let query = hidden.index_select(&idx, 0)?.detach();
            let best = nearest_scores(&query, &table)?
                .argmax(1)?
                .to_vec1::<u32>()?;
            // The served decoder on the same points: the writer's pick among its kept cells.
            let served = writer
                .write(&query)?
                .flatten_all()?
                .to_dtype(DType::U32)?
                .to_vec1::<u32>()?;
            let rows = best
                .iter()
                .zip(&served)
                .zip(&targets)
                .zip(&opens)
                .zip(&copies);
            for ((((pick, served), target), open), copy) in rows {
                reading.hits += usize::from(pick == target);
                written.hits += usize::from(served == target);
                if *copy {
                    reading.copied_rows += 1;
                    reading.copied_hits += usize::from(pick == target);
                } else if !*open {
                    reading.rest_rows += 1;
                    reading.rest_hits += usize::from(pick == target);
                }
                if *open {
                    reading.opening_rows += 1;
                    reading.opening_hits += usize::from(pick == target);
                    written.opening_hits += usize::from(served == target);
                    *openings.entry(*target).or_default() += 1;
                    picks.insert(*pick);
                    written_picks.insert(*served);
                }
            }
            let at: Vec<u32> = (0..opens.len() as u32)
                .filter(|&i| opens[i as usize])
                .collect();
            if !at.is_empty() {
                let at = Tensor::from_vec(at.clone(), at.len(), model.device())?;
                opening_queries.push(query.index_select(&at, 0)?);
            }
            all_queries.push(query);
        }
        reading.commonest_opening_rows = openings.values().copied().max().unwrap_or(0);
        reading.opening_distinct_picks = picks.len();
        written.opening_distinct_picks = written_picks.len();
        if !opening_queries.is_empty() {
            reading.opening_query_spread = spread(&Tensor::cat(&opening_queries, 0)?)?;
        }
        if !all_queries.is_empty() {
            reading.query_spread = spread(&Tensor::cat(&all_queries, 0)?)?;
        }
        Ok((reading, written))
    })
}

/// A seeded order of `0..n` (xorshift64*, Fisher-Yates), one per epoch.
fn epoch_order(n: usize, seed: u64, epoch: u64) -> Vec<usize> {
    let mut state = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(epoch.wrapping_mul(0xD1B5_4A32_D192_ED03))
        | 1;
    let mut next = || {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    };
    let mut order: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (next() % (i as u64 + 1)) as usize;
        order.swap(i, j);
    }
    order
}

/// The flat ids, scored indices and targets of `windows`, as the sheet's batches lay them out.
fn assemble(windows: &[&CausalWindow]) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let mut ids = Vec::with_capacity(windows.len() * SHEET_CONTEXT);
    let (mut scored, mut targets) = (Vec::new(), Vec::new());
    for (b, window) in windows.iter().enumerate() {
        ids.extend_from_slice(&window.input_ids);
        for (&off, &target) in window.scored_offsets.iter().zip(&window.scored_targets) {
            scored.push((b * SHEET_CONTEXT + off) as u32);
            targets.push(target);
        }
    }
    (ids, scored, targets)
}

/// Whether the target at corpus `position` opens a reply: the token before it is not a response
/// token, so the target follows the prompt and not the reply's own earlier words.
fn opens_reply(mask: &[u8], position: usize) -> bool {
    position > 0 && mask[position - 1] == 0
}

/// How well the prompt words a window shows could name its reply's first token: TRAIN's
/// commonest opening for each key, scored on DEV's openings ([`opening_bound`]). A count over the
/// corpus, not a model: it bounds what a read that finds those words could add to the openings.
#[derive(Clone, Debug, Serialize)]
pub struct OpeningBound {
    /// `none`, `first-k` (the first `k` prompt tokens the window shows) or `last-k` (the `k`
    /// tokens before the opening).
    pub key: String,
    pub train_rows: usize,
    pub train_keys: usize,
    pub dev_rows: usize,
    /// DEV openings whose key TRAIN never saw; TRAIN's commonest opening names them.
    pub dev_unseen: usize,
    pub dev_hits: usize,
    /// TRAIN's most frequent keys, decoded, with their commonest opening and its share.
    pub top: Vec<OpeningKey>,
}

#[derive(Clone, Debug, Serialize)]
pub struct OpeningKey {
    pub key: String,
    pub opening: String,
    pub train_rows: usize,
    pub opening_rows: usize,
}

/// Each opening in `windows`: the prompt tokens its window shows before it, in order, and its
/// target. The prompt runs back from the opening to the last response token or BOS.
fn openings(windows: &[CausalWindow], mask: &[u8]) -> Vec<(Vec<u32>, u32)> {
    opening_rows(windows, mask)
        .into_iter()
        .map(|row| (row.prompt, row.target))
        .collect()
}

/// One opening [`openings`] yields, with where it stands: its window's index and the offset of
/// the input token it is predicted from.
struct OpeningRow {
    window: usize,
    offset: usize,
    prompt: Vec<u32>,
    target: u32,
}

fn opening_rows(windows: &[CausalWindow], mask: &[u8]) -> Vec<OpeningRow> {
    let mut rows = Vec::new();
    for (index, w) in windows.iter().enumerate() {
        let scored = w
            .scored_offsets
            .iter()
            .zip(&w.scored_targets)
            .zip(&w.scored_corpus_positions);
        for ((&offset, &target), &position) in scored {
            if !opens_reply(mask, position) {
                continue;
            }
            // The corpus position of `input_ids[0]`.
            let base = position - 1 - offset;
            let mut first = offset + 1;
            while first > 0 {
                let i = first - 1;
                if w.input_ids[i] == hs::SHEET_BOS_ID || mask[base + i] == 1 {
                    break;
                }
                first = i;
            }
            rows.push(OpeningRow {
                window: index,
                offset,
                prompt: w.input_ids[first..=offset].to_vec(),
                target,
            });
        }
    }
    rows
}

/// How often the opening read's turn anchor (claude-154) keeps, at an opening, the same tokens
/// as the probe's first-8 key: `opening_slots(input_ids, 8, Turn)[offset]` as tokens against
/// `prompt[..8]` ([`opening_bound`]).
#[derive(Clone, Debug, Serialize)]
pub struct AnchorCheck {
    /// The split, `dev` or `train`.
    pub anchor_check: String,
    pub openings: usize,
    pub agree: usize,
}

fn anchor_check(split: &str, windows: &[CausalWindow], mask: &[u8]) -> AnchorCheck {
    use crate::flock_read_opening::{opening_slots, OpeningAnchor};
    let key = PromptKey::First(8);
    let rows = opening_rows(windows, mask);
    let mut agree = 0;
    let mut slots = (usize::MAX, Vec::new());
    for row in &rows {
        // Rows come window by window, so each window's slots are worked out once.
        let w = &windows[row.window];
        if slots.0 != row.window {
            slots = (
                row.window,
                opening_slots(&w.input_ids, 8, OpeningAnchor::Turn),
            );
        }
        let kept: Vec<u32> = slots.1[row.offset]
            .iter()
            .map(|&s| w.input_ids[s])
            .collect();
        agree += usize::from(kept == key.of(&row.prompt));
    }
    AnchorCheck {
        anchor_check: split.to_owned(),
        openings: rows.len(),
        agree,
    }
}

/// The commonest target in `counts` and its count, ties to the smaller token id.
fn commonest(counts: &std::collections::BTreeMap<u32, usize>) -> Option<(u32, usize)> {
    counts
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&t, &c)| (t, c))
}

/// Which prompt tokens an [`OpeningBound`] keys on.
#[derive(Clone, Copy, Debug)]
enum PromptKey {
    None,
    First(usize),
    Last(usize),
}

impl PromptKey {
    fn of(self, prompt: &[u32]) -> &[u32] {
        match self {
            PromptKey::None => &[],
            PromptKey::First(k) => &prompt[..k.min(prompt.len())],
            PromptKey::Last(k) => &prompt[prompt.len() - k.min(prompt.len())..],
        }
    }

    fn name(self) -> String {
        match self {
            PromptKey::None => "none".to_owned(),
            PromptKey::First(k) => format!("first-{k}"),
            PromptKey::Last(k) => format!("last-{k}"),
        }
    }
}

fn bound(
    key: PromptKey,
    train: &[(Vec<u32>, u32)],
    dev: &[(Vec<u32>, u32)],
    decode: &dyn Fn(&[u32]) -> String,
) -> OpeningBound {
    let pick = |prompt| key.of(prompt);
    let mut table =
        std::collections::BTreeMap::<Vec<u32>, std::collections::BTreeMap<u32, usize>>::new();
    let mut overall = std::collections::BTreeMap::<u32, usize>::new();
    for (prompt, target) in train {
        *table
            .entry(pick(prompt).to_vec())
            .or_default()
            .entry(*target)
            .or_default() += 1;
        *overall.entry(*target).or_default() += 1;
    }
    let fallback = commonest(&overall).map(|(t, _)| t);
    let (mut dev_unseen, mut dev_hits) = (0, 0);
    for (prompt, target) in dev {
        let named = match table.get(pick(prompt)) {
            Some(counts) => commonest(counts).map(|(t, _)| t),
            None => {
                dev_unseen += 1;
                fallback
            }
        };
        dev_hits += usize::from(named == Some(*target));
    }
    let mut keys: Vec<(&Vec<u32>, usize)> = table
        .iter()
        .map(|(k, counts)| (k, counts.values().sum()))
        .collect();
    keys.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let top = keys
        .into_iter()
        .take(8)
        .filter_map(|(k, rows)| {
            let (opening, opening_rows) = commonest(&table[k])?;
            Some(OpeningKey {
                key: decode(k),
                opening: decode(&[opening]),
                train_rows: rows,
                opening_rows,
            })
        })
        .collect();
    OpeningBound {
        key: key.name(),
        train_rows: train.len(),
        train_keys: table.len(),
        dev_rows: dev.len(),
        dev_unseen,
        dev_hits,
        top,
    }
}

/// [`OpeningBound`] for no key, the first 1–8 prompt tokens and the last 1–4, and the opening
/// read's [`AnchorCheck`] on DEV and on TRAIN.
pub fn opening_bound(data_root: &Path) -> Result<(Vec<OpeningBound>, Vec<AnchorCheck>)> {
    let tokenizer = pinned_tokenizer(data_root)?;
    let decode = |ids: &[u32]| tail(&tokenizer.decode(ids), 80);
    let (train_tokens, train_mask) = load_train_split(data_root)?;
    let (train_windows, _) =
        hs::build_document_windows(&train_tokens, &train_mask, SHEET_CONTEXT, hs::SHEET_BOS_ID)?;
    let (dev_windows, dev_mask) = load_dev(data_root)?;
    let train = openings(&train_windows, &train_mask);
    let dev = openings(&dev_windows, &dev_mask);
    let keys = std::iter::once(PromptKey::None)
        .chain((1..=8).map(PromptKey::First))
        .chain((1..=4).map(PromptKey::Last));
    let checks = vec![
        anchor_check("dev", &dev_windows, &dev_mask),
        anchor_check("train", &train_windows, &train_mask),
    ];
    Ok((
        keys.map(|key| bound(key, &train, &dev, &decode)).collect(),
        checks,
    ))
}

/// One reply row (turn-memory-reply, claude-192): the prompt its window shows for its turn, back
/// from the reply's start to the last response token or BOS; the reply's own earlier tokens; its
/// target; and whether it opens the reply or copies the prompt ([`copies_prompt`]).
#[derive(Clone, Debug, PartialEq, Eq)]
struct ReplyRow {
    /// The row's window, an index into the windows it was read from, and the input offset it is
    /// predicted from.
    window: usize,
    offset: usize,
    prompt: Vec<u32>,
    reply: Vec<u32>,
    target: u32,
    opening: bool,
    copy: bool,
}

/// The reply table's key for `row` (claude-193): the probe's `turn-8 + position + reply-last-1`,
/// so the read keys exactly as the probe counted.
fn reply_key(row: &ReplyRow) -> Vec<u32> {
    ReplyKey::TurnPosition(1).of(row)
}

/// For each of `windows`, its rows past their reply's opening, each as its input offset and its
/// [`reply_key`]: the rows the reply table is named at (claude-193).
/// Each window's rows past their openings, as [`reply_offsets`] gives them.
type ReplyOffsets = Vec<Vec<(usize, Vec<u32>)>>;

fn reply_offsets(windows: &[CausalWindow], mask: &[u8]) -> ReplyOffsets {
    let mut out = vec![Vec::new(); windows.len()];
    for row in reply_rows(windows, mask) {
        if !row.opening {
            out[row.window].push((row.offset, reply_key(&row)));
        }
    }
    out
}

/// The reply key a greedy reply's next step reads (claude-193): the turn's first
/// [`TURN_KEY`] ids, by the turn anchor at the prompt's last token, which equals the probe's
/// first-8 key at every opening (claude-154), then the reply's length and its last id. `window`
/// is the step's window and `opening` the window index of the prompt's last token.
fn greedy_reply_key(window: &[u32], opening: usize, reply: &[u32]) -> Option<Vec<u32>> {
    use crate::flock_read_opening::{opening_slots, OpeningAnchor};
    let last = *reply.last()?;
    let slots = opening_slots(&window[..=opening], TURN_KEY, OpeningAnchor::Turn);
    let turn = slots[opening].iter().map(|&s| window[s]);
    Some(
        turn.chain([u32::MAX, reply.len() as u32, u32::MAX, last])
            .collect(),
    )
}

fn reply_rows(windows: &[CausalWindow], mask: &[u8]) -> Vec<ReplyRow> {
    let mut rows = Vec::new();
    for (index, w) in windows.iter().enumerate() {
        let scored = w
            .scored_offsets
            .iter()
            .zip(&w.scored_targets)
            .zip(&w.scored_corpus_positions);
        for ((&offset, &target), &position) in scored {
            if mask[position] == 0 {
                continue;
            }
            // The corpus position of `input_ids[0]`.
            let base = position - 1 - offset;
            let part = |i: usize, reply: bool| {
                w.input_ids[i] != hs::SHEET_BOS_ID && (mask[base + i] == 1) == reply
            };
            let mut start = offset + 1;
            while start > 0 && part(start - 1, true) {
                start -= 1;
            }
            let mut first = start;
            while first > 0 && part(first - 1, false) {
                first -= 1;
            }
            rows.push(ReplyRow {
                window: index,
                offset,
                prompt: w.input_ids[first..start].to_vec(),
                reply: w.input_ids[start..=offset].to_vec(),
                target,
                opening: opens_reply(mask, position),
                copy: copies_prompt(w, offset, position, target, mask),
            });
        }
    }
    rows
}

/// How many of the turn's first prompt tokens a turn key holds, as claude-161's memory. The reply
/// table's keys hold this many, whatever the opening table's `k`; the export carries it
/// (claude-198).
pub(crate) const TURN_KEY: usize = 8;

/// Which part of a [`ReplyRow`] a [`ReplyBound`] keys on.
#[derive(Clone, Copy, Debug)]
enum ReplyKey {
    None,
    /// The reply's last `j` tokens alone: the control with no turn.
    Reply(usize),
    /// The turn's first 8 tokens and the reply's last `j`.
    Turn(usize),
    /// The turn's first 8 tokens, the reply position, and the reply's last `j`. At position 0
    /// this is claude-161's opening key.
    TurnPosition(usize),
}

impl ReplyKey {
    fn of(self, row: &ReplyRow) -> Vec<u32> {
        let last = |j: usize| &row.reply[row.reply.len() - j.min(row.reply.len())..];
        let turn = &row.prompt[..TURN_KEY.min(row.prompt.len())];
        // The parts are joined by `u32::MAX`, which no token id takes.
        match self {
            ReplyKey::None => Vec::new(),
            ReplyKey::Reply(j) => last(j).to_vec(),
            ReplyKey::Turn(j) => [turn, &[u32::MAX], last(j)].concat(),
            ReplyKey::TurnPosition(j) => {
                [turn, &[u32::MAX, row.reply.len() as u32, u32::MAX], last(j)].concat()
            }
        }
    }

    fn name(self) -> String {
        match self {
            ReplyKey::None => "none".to_owned(),
            ReplyKey::Reply(j) => format!("reply-last-{j}"),
            ReplyKey::Turn(j) => format!("turn-{TURN_KEY}+reply-last-{j}"),
            ReplyKey::TurnPosition(j) => format!("turn-{TURN_KEY}+position+reply-last-{j}"),
        }
    }
}

/// DEV rows of one kind: how many there are, how many a key the table keeps names, and how many
/// of those it names right.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct KindCount {
    pub rows: usize,
    pub named: usize,
    pub hits: usize,
}

/// What an exact memory of TRAIN, keyed by one [`ReplyKey`], could name among DEV's reply rows
/// (`--reply-bound`). The table keeps, for each key, TRAIN's commonest target (ties to the smaller
/// id), and only where that target's share of the key's rows is at least `min_share`. A DEV row
/// whose key is not kept is named by nothing: the trunk alone would decide it. A count over the
/// corpus, not a model.
#[derive(Clone, Debug, Serialize)]
pub struct ReplyBound {
    pub key: String,
    pub min_share: f64,
    pub train_rows: usize,
    /// Keys the table keeps at `min_share`.
    pub train_keys: usize,
    pub all: KindCount,
    pub openings: KindCount,
    pub copies: KindCount,
    /// Rows that neither open a reply nor copy the prompt.
    pub rest: KindCount,
    /// Openings named right when an opening whose key is not kept takes TRAIN's commonest
    /// opening, as [`opening_bound`] counts: claude-161's 607 for the opening key.
    pub opening_hits_with_fallback: usize,
}

/// TRAIN's table for `key`: each key's commonest target (ties to the smaller id), that target's
/// count, and the key's rows ([`reply_bound_rows`], and the census's row selection).
type ReplyTableCounts = std::collections::HashMap<Vec<u32>, (u32, usize, usize)>;

fn reply_table(key: ReplyKey, train: &[ReplyRow]) -> ReplyTableCounts {
    let mut counts = std::collections::HashMap::<Vec<u32>, Vec<(u32, usize)>>::new();
    for row in train {
        let targets = counts.entry(key.of(row)).or_default();
        match targets.iter_mut().find(|(t, _)| *t == row.target) {
            Some((_, n)) => *n += 1,
            None => targets.push((row.target, 1)),
        }
    }
    counts
        .into_iter()
        .map(|(k, targets)| {
            let rows = targets.iter().map(|(_, n)| n).sum();
            let (target, n) = targets
                .into_iter()
                .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
                .unwrap_or((0, 0));
            (k, (target, n, rows))
        })
        .collect()
}

/// Whether a table entry `(target, count, rows)` is kept at `min_share`.
fn table_keeps(stored: &(u32, usize, usize), min_share: f64) -> bool {
    stored.1 as f64 >= min_share * stored.2 as f64
}

/// [`ReplyBound`] for `key` at each of `shares`, the table built from `train`.
fn reply_bound_rows(
    key: ReplyKey,
    train: &[ReplyRow],
    dev: &[ReplyRow],
    shares: &[f64],
) -> Vec<ReplyBound> {
    let mut openings = std::collections::BTreeMap::<u32, usize>::new();
    for row in train.iter().filter(|row| row.opening) {
        *openings.entry(row.target).or_default() += 1;
    }
    let table = reply_table(key, train);
    let fallback = commonest(&openings).map(|(t, _)| t);
    shares
        .iter()
        .map(|&min_share| {
            let kept = |stored: &(u32, usize, usize)| table_keeps(stored, min_share);
            let mut out = ReplyBound {
                key: key.name(),
                min_share,
                train_rows: train.len(),
                train_keys: table.values().filter(|s| kept(s)).count(),
                all: KindCount::default(),
                openings: KindCount::default(),
                copies: KindCount::default(),
                rest: KindCount::default(),
                opening_hits_with_fallback: 0,
            };
            for row in dev {
                let named = table.get(&key.of(row)).filter(|s| kept(s)).map(|s| s.0);
                let mut kinds = vec![&mut out.all];
                if row.opening {
                    kinds.push(&mut out.openings);
                }
                if row.copy {
                    kinds.push(&mut out.copies);
                }
                if !row.opening && !row.copy {
                    kinds.push(&mut out.rest);
                }
                for kind in kinds {
                    kind.rows += 1;
                    kind.named += usize::from(named.is_some());
                    kind.hits += usize::from(named == Some(row.target));
                }
                if row.opening {
                    out.opening_hits_with_fallback +=
                        usize::from(named.or(fallback) == Some(row.target));
                }
            }
            out
        })
        .collect()
}

/// The stored-share thresholds `--reply-bound` reports.
const REPLY_BOUND_SHARES: [f64; 4] = [0.0, 0.5, 0.8, 0.95];

/// [`ReplyBound`] over DEV's reply rows, with TRAIN's table, for each [`ReplyKey`]: none, the
/// reply's last 1–4 tokens, the turn with the reply's last 0–4, and the turn with the position
/// and the reply's last 0–1 (turn-memory-reply's probe, claude-192).
pub fn reply_bound(data_root: &Path) -> Result<Vec<ReplyBound>> {
    let (train_tokens, train_mask) = load_train_split(data_root)?;
    let (train_windows, _) =
        hs::build_document_windows(&train_tokens, &train_mask, SHEET_CONTEXT, hs::SHEET_BOS_ID)?;
    let (dev_windows, dev_mask) = load_dev(data_root)?;
    let train = reply_rows(&train_windows, &train_mask);
    let dev = reply_rows(&dev_windows, &dev_mask);
    let keys = std::iter::once(ReplyKey::None)
        .chain((1..=4).map(ReplyKey::Reply))
        .chain((0..=4).map(ReplyKey::Turn))
        .chain((0..=1).map(ReplyKey::TurnPosition));
    Ok(keys
        .flat_map(|key| reply_bound_rows(key, &train, &dev, &REPLY_BOUND_SHARES))
        .collect())
}

/// Pinned SHA-256 of the sealed Fresh V2 panel's `tokens.u16`, opened once for M1 bar (e)
/// (`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md:69`). The census only counts it.
pub const PINNED_FRESH_V2_TOKENS_SHA256: &str =
    "0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8";
/// Pinned SHA-256 of Fresh V2's `response_mask.u8`.
pub const PINNED_FRESH_V2_MASK_SHA256: &str =
    "a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e";
/// Fresh V2's payload tokens and eligible response targets.
pub const EXPECTED_FRESH_V2_TOKENS: usize = 44_678;
pub const EXPECTED_FRESH_V2_ELIGIBLE_TARGETS: usize = 14_479;

/// The census's value masks keep TRAIN's K most frequent tokens (one-model-end-to-end, claude-202).
pub const CENSUS_MASK_KS: [usize; 2] = [256, 1024];
/// The n-gram lengths the census compares.
pub const CENSUS_NGRAMS: [usize; 2] = [8, 13];
/// A panel holds new templates when its value-masked prompt share at K = 1,024 is at most this,
/// fixed in claude-202's ORDER before any count.
pub const CENSUS_NEW_TEMPLATE_SHARE: f64 = 0.5;
/// The precision a no-turn key needs to count as the reply-only control (claude-192's ORDER).
const CENSUS_CONTROL_PRECISION: f64 = 0.83;
/// Stands for a masked run in a value-masked sequence; never a token id.
const CENSUS_MASK: u32 = u32::MAX;

/// One split's size (census M0).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SplitSize {
    pub file_bytes: u64,
    pub payload_tokens: usize,
    pub eligible_targets: usize,
    pub documents: usize,
    pub prompts: usize,
    pub replies: usize,
}

/// A split's prompts and replies, each a maximal run of mask-0 or mask-1 tokens inside one
/// document; `<|bos|>` starts a document and belongs to neither.
#[derive(Debug, Default, PartialEq, Eq)]
struct Turns {
    documents: usize,
    prompts: Vec<Vec<u32>>,
    replies: Vec<Vec<u32>>,
}

fn split_turns(tokens: &[u32], mask: &[u8], bos: u32) -> Turns {
    let mut turns = Turns::default();
    let mut run: Vec<u32> = Vec::new();
    let mut run_mask = 0u8;
    let close = |run: &mut Vec<u32>, run_mask: u8, turns: &mut Turns| {
        if !run.is_empty() {
            let done = std::mem::take(run);
            if run_mask == 1 {
                turns.replies.push(done);
            } else {
                turns.prompts.push(done);
            }
        }
    };
    for (&t, &m) in tokens.iter().zip(mask) {
        if t == bos {
            close(&mut run, run_mask, &mut turns);
            turns.documents += 1;
            continue;
        }
        if !run.is_empty() && m != run_mask {
            close(&mut run, run_mask, &mut turns);
        }
        run_mask = m;
        run.push(t);
    }
    close(&mut run, run_mask, &mut turns);
    turns
}

/// A 128-bit hash of `ids`, two FNV-1a passes with different offsets: the census's set key, so its
/// n-gram sets fit in memory. Deterministic.
fn census_hash(ids: &[u32]) -> u128 {
    let pass = |mut h: u64| {
        for &id in ids {
            for b in id.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        }
        h ^ (ids.len() as u64)
    };
    (u128::from(pass(0xcbf2_9ce4_8422_2325)) << 64) | u128::from(pass(0x84222325_cbf29ce4))
}

/// The hashes of every `n`-token window inside each run (no window crosses a run).
fn ngram_set(runs: &[Vec<u32>], n: usize) -> std::collections::HashSet<u128> {
    runs.iter()
        .flat_map(|run| run.windows(n).map(census_hash))
        .collect()
}

/// The share of `runs`' `n`-token windows that `train` holds, and the window count.
fn ngram_share(runs: &[Vec<u32>], train: &std::collections::HashSet<u128>, n: usize) -> (f64, usize) {
    let (mut found, mut total) = (0usize, 0usize);
    for window in runs.iter().flat_map(|run| run.windows(n)) {
        total += 1;
        found += usize::from(train.contains(&census_hash(window)));
    }
    (if total == 0 { 0.0 } else { found as f64 / total as f64 }, total)
}

/// The share of `runs` equal to a run of `train` (as hashed by [`census_hash`]).
fn exact_share(runs: &[Vec<u32>], train: &std::collections::HashSet<u128>) -> f64 {
    let found = runs.iter().filter(|run| train.contains(&census_hash(run))).count();
    if runs.is_empty() { 0.0 } else { found as f64 / runs.len() as f64 }
}

/// `run` with every token outside `keep` (or in `digits`) replaced by one [`CENSUS_MASK`], and
/// each run of masks collapsed to one.
fn value_mask(
    run: &[u32],
    keep: &std::collections::HashSet<u32>,
    digits: &std::collections::HashSet<u32>,
) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::with_capacity(run.len());
    for &t in run {
        let id = if keep.contains(&t) && !digits.contains(&t) { t } else { CENSUS_MASK };
        if !(id == CENSUS_MASK && out.last() == Some(&CENSUS_MASK)) {
            out.push(id);
        }
    }
    out
}

/// TRAIN's `k` most frequent tokens, ties to the smaller id.
fn top_tokens(tokens: &[u32], k: usize, bos: u32) -> std::collections::HashSet<u32> {
    let mut counts = std::collections::HashMap::<u32, usize>::new();
    for &t in tokens.iter().filter(|&&t| t != bos) {
        *counts.entry(t).or_default() += 1;
    }
    let mut ranked: Vec<(u32, usize)> = counts.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    ranked.into_iter().take(k).map(|(t, _)| t).collect()
}

/// The ids of the tokenizer's single-digit pieces (a digit, with or without the byte-level space
/// mark `Ġ` before it): the census masks them as values, since numbers are split into digits.
fn digit_tokens(tokenizer_json: &str) -> Result<std::collections::HashSet<u32>> {
    let value: serde_json::Value = serde_json::from_str(tokenizer_json)?;
    let vocab = value["model"]["vocab"]
        .as_object()
        .ok_or_else(|| invalid("tokenizer.json has no model.vocab"))?;
    Ok(vocab
        .iter()
        .filter(|(piece, _)| {
            let bare = piece.strip_prefix('\u{120}').unwrap_or(piece);
            bare.len() == 1 && bare.as_bytes()[0].is_ascii_digit()
        })
        .filter_map(|(_, id)| id.as_u64().map(|id| id as u32))
        .collect())
}

/// One panel's overlap with TRAIN (census M3–M6).
#[derive(Clone, Debug, Serialize)]
pub struct PanelOverlap {
    pub panel: String,
    pub prompts: usize,
    pub replies: usize,
    pub exact_prompt_share: f64,
    pub exact_reply_share: f64,
    /// `(n, share, windows)` for each n in [`CENSUS_NGRAMS`].
    pub prompt_ngram_share: Vec<(usize, f64, usize)>,
    pub reply_ngram_share: Vec<(usize, f64, usize)>,
    /// `(k, share)` for each k in [`CENSUS_MASK_KS`].
    pub prompt_template_share: Vec<(usize, f64)>,
    pub reply_template_share: Vec<(usize, f64)>,
    /// Openings whose turn's first-8 key TRAIN's opening table holds, and all openings (M6).
    pub keyed_openings: usize,
    pub openings: usize,
    /// The ORDER's rule: the prompt template share at K = 1,024 is at most
    /// [`CENSUS_NEW_TEMPLATE_SHARE`].
    pub new_templates: bool,
}

/// One no-turn key's later-row count, for the reply-only control (census M1).
#[derive(Clone, Debug, Serialize)]
pub struct ControlCell {
    pub key: String,
    pub min_share: f64,
    pub named: usize,
    pub hits: usize,
}

/// The pinned numbers the census must reproduce (M1), and step 2's rows (M2).
#[derive(Clone, Debug, Serialize)]
pub struct CensusRows {
    pub dev_openings: usize,
    pub dev_keyed_openings: usize,
    /// DEV's later rows (past their openings) the turn-8 + position + reply-last-1 table names at
    /// share 0.5, and how many it names right.
    pub later_rows: usize,
    pub later_named: usize,
    pub later_hits: usize,
    /// The most later-row hits among no-turn keys with precision at least 0.83, and the grid.
    pub reply_only_control: usize,
    pub reply_only_grid: Vec<ControlCell>,
    /// DEV corpus positions of the later rows the table does not name and the openings no key
    /// names: step 2's decision rows, pooled.
    pub unnamed_later: usize,
    pub unkeyed_openings: usize,
    pub decision_positions: Vec<usize>,
}

/// The data census (one-model-end-to-end, claude-202): every count the ORDER fixes.
#[derive(Clone, Debug, Serialize)]
pub struct Census {
    pub train: SplitSize,
    pub dev: SplitSize,
    pub fresh_v2: SplitSize,
    pub rows: CensusRows,
    pub panels: Vec<PanelOverlap>,
    /// The `--panel` splits (claude-204): each one's name, its `tokens.u16` sha256 and its size.
    /// They are counted, not pinned.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<(String, String, SplitSize)>,
}

/// An unpinned UORT split (census `--panel`, claude-204): `DIR/tokens.u16` and
/// `DIR/response_mask.u8`, refused unless the header is UORT, the payload and mask lengths match
/// its count, the first token is BOS and every token is in the sheet's vocabulary. Returns the
/// tokens, the mask and the token file's sha256.
fn load_unpinned_uort(dir: &Path) -> Result<(Vec<u32>, Vec<u8>, String)> {
    let token_bytes = fs::read(dir.join("tokens.u16"))?;
    let mask = fs::read(dir.join("response_mask.u8"))?;
    if token_bytes.len() < hs::UORT_HEADER_BYTES || &token_bytes[..4] != b"UORT" {
        return Err(invalid(format!(
            "{}: tokens.u16 has no 64-byte UORT header",
            dir.display()
        )));
    }
    let count = u64::from_le_bytes(token_bytes[8..16].try_into().unwrap()) as usize;
    let payload = &token_bytes[hs::UORT_HEADER_BYTES..];
    if payload.len() != count * 2 || mask.len() != count {
        return Err(invalid(format!(
            "{}: the UORT payload or mask does not match its count {count}",
            dir.display()
        )));
    }
    let tokens: Vec<u32> = payload
        .chunks_exact(2)
        .map(|w| u32::from(u16::from_le_bytes([w[0], w[1]])))
        .collect();
    if tokens.first() != Some(&hs::SHEET_BOS_ID)
        || tokens.iter().any(|&t| t as usize >= hs::SHEET_VOCAB)
    {
        return Err(invalid(format!(
            "{}: the tokens are out of range or do not start with BOS",
            dir.display()
        )));
    }
    Ok((tokens, mask, crate::sha256_bytes(&token_bytes)))
}

/// The corpus position of `row`, from its window.
fn row_position(row: &ReplyRow, windows: &[CausalWindow]) -> Option<usize> {
    let w = windows.get(row.window)?;
    let i = w.scored_offsets.iter().position(|&off| off == row.offset)?;
    w.scored_corpus_positions.get(i).copied()
}

/// Step 2's rows: DEV's later rows the reply table does not name at share 0.5, and its openings
/// no first-8 key names, as `(later, openings)` index lists into `dev`.
fn census_decision_rows(
    reply: &ReplyTableCounts,
    opening: &ReplyTableCounts,
    dev: &[ReplyRow],
) -> (Vec<usize>, Vec<usize>) {
    let mut later = Vec::new();
    let mut openings = Vec::new();
    for (i, row) in dev.iter().enumerate() {
        if row.opening {
            if !opening.contains_key(&ReplyKey::TurnPosition(0).of(row)) {
                openings.push(i);
            }
        } else if !reply
            .get(&ReplyKey::TurnPosition(1).of(row))
            .is_some_and(|s| table_keeps(s, REPLY_MEMORY_SHARE))
        {
            later.push(i);
        }
    }
    (later, openings)
}

/// The split's size from its file and turns.
fn split_size(path: &Path, tokens: &[u32], mask: &[u8], turns: &Turns) -> Result<SplitSize> {
    Ok(SplitSize {
        file_bytes: fs::metadata(path)?.len(),
        payload_tokens: tokens.len(),
        eligible_targets: tokens
            .iter()
            .zip(mask)
            .enumerate()
            .filter(|&(p, (&t, &m))| p > 0 && t != hs::SHEET_BOS_ID && m == 1)
            .count(),
        documents: turns.documents,
        prompts: turns.prompts.len(),
        replies: turns.replies.len(),
    })
}

/// The data census of claude-202's ORDER over TRAIN, DEV and the sealed Fresh V2 panel in
/// `fresh_v2` (its `tokens.u16` and `response_mask.u8`, pinned), and over each `(name, dir)` of
/// `extra`, unpinned (claude-204). Counts only; no model, no GPU.
pub fn census(data_root: &Path, fresh_v2: &Path, extra: &[(String, PathBuf)]) -> Result<Census> {
    let bos = hs::SHEET_BOS_ID;
    let (train_tokens, train_mask) = load_train_split(data_root)?;
    let (dev_tokens, dev_mask) = hs::load_pinned_uort_split(
        &data_root.join("ft/dev/tokens.u16"),
        &data_root.join("ft/dev/response_mask.u8"),
        hs::PINNED_DEV_TOKENS_SHA256,
        hs::PINNED_DEV_MASK_SHA256,
        hs::EXPECTED_DEV_TOKENS,
        hs::EXPECTED_DEV_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        bos,
    )?;
    let (fresh_tokens, fresh_mask) = hs::load_pinned_uort_split(
        &fresh_v2.join("tokens.u16"),
        &fresh_v2.join("response_mask.u8"),
        PINNED_FRESH_V2_TOKENS_SHA256,
        PINNED_FRESH_V2_MASK_SHA256,
        EXPECTED_FRESH_V2_TOKENS,
        EXPECTED_FRESH_V2_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        bos,
    )?;
    let windows = |tokens: &[u32], mask: &[u8]| -> Result<Vec<CausalWindow>> {
        Ok(hs::build_document_windows(tokens, mask, SHEET_CONTEXT, bos)?.0)
    };
    let (train_w, dev_w, fresh_w) = (
        windows(&train_tokens, &train_mask)?,
        windows(&dev_tokens, &dev_mask)?,
        windows(&fresh_tokens, &fresh_mask)?,
    );
    let train_rows = reply_rows(&train_w, &train_mask);
    let dev_rows = reply_rows(&dev_w, &dev_mask);
    let fresh_rows = reply_rows(&fresh_w, &fresh_mask);

    // M1 and M2, with the probe's own tables.
    let opening_table = reply_table(ReplyKey::TurnPosition(0), &train_rows);
    let reply = reply_table(ReplyKey::TurnPosition(1), &train_rows);
    let keyed = |rows: &[ReplyRow]| {
        rows.iter()
            .filter(|r| r.opening && opening_table.contains_key(&ReplyKey::TurnPosition(0).of(r)))
            .count()
    };
    let later: Vec<&ReplyRow> = dev_rows.iter().filter(|r| !r.opening).collect();
    let named_later = |table: &ReplyTableCounts, key: ReplyKey, share: f64| {
        let mut named = 0;
        let mut hits = 0;
        for row in &later {
            if let Some(stored) = table.get(&key.of(row)).filter(|s| table_keeps(s, share)) {
                named += 1;
                hits += usize::from(stored.0 == row.target);
            }
        }
        (named, hits)
    };
    let (later_named, later_hits) = named_later(&reply, ReplyKey::TurnPosition(1), REPLY_MEMORY_SHARE);
    let mut grid = Vec::new();
    for key in std::iter::once(ReplyKey::None).chain((1..=4).map(ReplyKey::Reply)) {
        let table = reply_table(key, &train_rows);
        for &share in &REPLY_BOUND_SHARES {
            let (named, hits) = named_later(&table, key, share);
            grid.push(ControlCell { key: key.name(), min_share: share, named, hits });
        }
    }
    let reply_only_control = grid
        .iter()
        .filter(|c| c.named > 0 && c.hits as f64 >= CENSUS_CONTROL_PRECISION * c.named as f64)
        .map(|c| c.hits)
        .max()
        .unwrap_or(0);
    let (unnamed, unkeyed) = census_decision_rows(&reply, &opening_table, &dev_rows);
    let mut decision_positions = unnamed
        .iter()
        .chain(&unkeyed)
        .map(|&i| {
            row_position(&dev_rows[i], &dev_w)
                .ok_or_else(|| invalid("a DEV reply row has no corpus position"))
        })
        .collect::<Result<Vec<usize>>>()?;
    decision_positions.sort_unstable();
    let rows = CensusRows {
        dev_openings: dev_rows.iter().filter(|r| r.opening).count(),
        dev_keyed_openings: keyed(&dev_rows),
        later_rows: later.len(),
        later_named,
        later_hits,
        reply_only_control,
        reply_only_grid: grid,
        unnamed_later: unnamed.len(),
        unkeyed_openings: unkeyed.len(),
        decision_positions,
    };

    // M0 and M3–M6.
    let train_turns = split_turns(&train_tokens, &train_mask, bos);
    let dev_turns = split_turns(&dev_tokens, &dev_mask, bos);
    let fresh_turns = split_turns(&fresh_tokens, &fresh_mask, bos);
    let digits = digit_tokens(&fs::read_to_string(data_root.join("tokenizer.json"))?)?;
    let exact = |runs: &[Vec<u32>]| runs.iter().map(|r| census_hash(r)).collect();
    let (train_prompts, train_replies): (std::collections::HashSet<u128>, std::collections::HashSet<u128>) =
        (exact(&train_turns.prompts), exact(&train_turns.replies));
    let ngram_sets: Vec<(usize, std::collections::HashSet<u128>, std::collections::HashSet<u128>)> =
        CENSUS_NGRAMS
            .iter()
            .map(|&n| (n, ngram_set(&train_turns.prompts, n), ngram_set(&train_turns.replies, n)))
            .collect();
    let masks: Vec<(usize, std::collections::HashSet<u32>)> = CENSUS_MASK_KS
        .iter()
        .map(|&k| (k, top_tokens(&train_tokens, k, bos)))
        .collect();
    let template_sets: Vec<(usize, std::collections::HashSet<u128>, std::collections::HashSet<u128>)> = masks
        .iter()
        .map(|(k, keep)| {
            let set = |runs: &[Vec<u32>]| {
                runs.iter().map(|r| census_hash(&value_mask(r, keep, &digits))).collect()
            };
            (*k, set(&train_turns.prompts), set(&train_turns.replies))
        })
        .collect();
    let overlap = |panel: &str, turns: &Turns, rows: &[ReplyRow]| {
        let template = |runs: &[Vec<u32>], prompts: bool| -> Vec<(usize, f64)> {
            template_sets
                .iter()
                .zip(&masks)
                .map(|((k, p, r), (_, keep))| {
                    let set = if prompts { p } else { r };
                    let masked: Vec<Vec<u32>> =
                        runs.iter().map(|run| value_mask(run, keep, &digits)).collect();
                    (*k, exact_share(&masked, set))
                })
                .collect()
        };
        let prompt_template_share = template(&turns.prompts, true);
        let new_templates = prompt_template_share
            .iter()
            .find(|(k, _)| *k == 1024)
            .is_some_and(|(_, share)| *share <= CENSUS_NEW_TEMPLATE_SHARE);
        PanelOverlap {
            panel: panel.to_owned(),
            prompts: turns.prompts.len(),
            replies: turns.replies.len(),
            exact_prompt_share: exact_share(&turns.prompts, &train_prompts),
            exact_reply_share: exact_share(&turns.replies, &train_replies),
            prompt_ngram_share: ngram_sets
                .iter()
                .map(|(n, p, _)| {
                    let (share, windows) = ngram_share(&turns.prompts, p, *n);
                    (*n, share, windows)
                })
                .collect(),
            reply_ngram_share: ngram_sets
                .iter()
                .map(|(n, _, r)| {
                    let (share, windows) = ngram_share(&turns.replies, r, *n);
                    (*n, share, windows)
                })
                .collect(),
            prompt_template_share,
            reply_template_share: template(&turns.replies, false),
            keyed_openings: keyed(rows),
            openings: rows.iter().filter(|r| r.opening).count(),
            new_templates,
        }
    };
    let mut panels = vec![
        overlap("ft/dev", &dev_turns, &dev_rows),
        overlap("fresh_v2", &fresh_turns, &fresh_rows),
    ];
    let mut extra_sizes = Vec::with_capacity(extra.len());
    for (name, dir) in extra {
        let (tokens, mask, sha) = load_unpinned_uort(dir)?;
        let turns = split_turns(&tokens, &mask, bos);
        let rows = reply_rows(&windows(&tokens, &mask)?, &mask);
        panels.push(overlap(name, &turns, &rows));
        extra_sizes.push((
            name.clone(),
            sha,
            split_size(&dir.join("tokens.u16"), &tokens, &mask, &turns)?,
        ));
    }
    Ok(Census {
        extra: extra_sizes,
        train: split_size(&data_root.join("mw-balp/train/tokens.u16"), &train_tokens, &train_mask, &train_turns)?,
        dev: split_size(&data_root.join("ft/dev/tokens.u16"), &dev_tokens, &dev_mask, &dev_turns)?,
        fresh_v2: split_size(&fresh_v2.join("tokens.u16"), &fresh_tokens, &fresh_mask, &fresh_turns)?,
        rows,
        panels,
    })
}

/// `samples` DEV windows spread over the panel whose first scored target opens a reply, with at
/// least 16 tokens of prompt before it. `mask` is DEV's response mask.
fn sample_windows<'a>(
    dev: &'a [CausalWindow],
    mask: &[u8],
    samples: usize,
) -> Vec<&'a CausalWindow> {
    let eligible: Vec<&CausalWindow> = dev
        .iter()
        .filter(|w| w.scored_offsets.first().is_some_and(|&off| off >= 16))
        .filter(|w| {
            w.scored_corpus_positions
                .first()
                .is_some_and(|&p| opens_reply(mask, p))
        })
        .collect();
    if eligible.is_empty() || samples == 0 {
        return Vec::new();
    }
    let stride = (eligible.len() / samples).max(1);
    eligible.into_iter().step_by(stride).take(samples).collect()
}

/// The model's greedy reply to `prompt`: `reply_tokens` tokens on the last `SHEET_CONTEXT`
/// tokens, stopping early at BOS. Each is the whole table's nearest token under
/// [`PointObjective::Nearest`] or [`PointObjective::Cell`], else the writer's pick among its
/// admitted cells.
fn greedy_reply(
    model: &StackModel,
    objective: PointObjective,
    prompt: &[u32],
    reply_tokens: usize,
    seed: u64,
) -> Result<Vec<u32>> {
    let table = table(model)?;
    hs::with_gate_mode(HamiltonArm::HamiltonActive, seed, 0, false, || {
        let mut ids = prompt.to_vec();
        let mut reply = Vec::with_capacity(reply_tokens);
        for _ in 0..reply_tokens {
            let window = &ids[ids.len().saturating_sub(SHEET_CONTEXT)..];
            let next = if objective.decodes_nearest() {
                // The reply's first step is its opening row, the one the turn memory moves.
                if reply.is_empty()
                    && !window.is_empty()
                    && crate::flock_read_memory::turn_memory().is_some()
                {
                    crate::flock_read_memory::set_memory_rows(Some(vec![window.len() - 1]));
                }
                // A later step reads the reply table (claude-193), keyed by the turn at the prompt's
                // last token, the reply's length and its last id.
                let start = ids.len().saturating_sub(SHEET_CONTEXT);
                if !reply.is_empty()
                    && prompt.len() > start
                    && crate::flock_read_memory::turn_memory()
                        .is_some_and(|m| m.reply_len().is_some())
                {
                    if let Some(key) = greedy_reply_key(window, prompt.len() - 1 - start, &reply) {
                        crate::flock_read_memory::set_reply_rows(Some(vec![(
                            window.len() - 1,
                            key,
                        )]));
                    }
                }
                let hidden = crate::flock_read::points(model, window, 1, window.len());
                crate::flock_read_memory::set_memory_rows(None);
                crate::flock_read_memory::set_reply_rows(None);
                let hidden = hidden?;
                nearest_scores(&hidden.narrow(0, window.len() - 1, 1)?, &table)?
                    .argmax(1)?
                    .to_vec1::<u32>()?
            } else {
                model
                    .native_next_token_ids(window, 1, window.len())?
                    .flatten_all()?
                    .to_dtype(DType::U32)?
                    .to_vec1::<u32>()?
            };
            let token = *next
                .last()
                .ok_or_else(|| invalid("the writer returned no token"))?;
            if token == hs::SHEET_BOS_ID {
                break;
            }
            ids.push(token);
            reply.push(token);
        }
        Ok(reply)
    })
}

fn append_line(path: &Path, line: &str) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{line}")?;
    Ok(())
}

/// The last `chars` characters of `text`, on one line.
fn tail(text: &str, chars: usize) -> String {
    let flat: String = text
        .chars()
        .map(|c| if c == '\n' { ' ' } else { c })
        .collect();
    let n = flat.chars().count();
    flat.chars().skip(n.saturating_sub(chars)).collect()
}

/// The nearest and farthest pair of the writer's 16 address roots, as the mean L1 distance over
/// a port's 8 coordinates, the unit the address score uses. A query port sitting exactly on its
/// root beats the next cell sharing the other port by at most the nearest separation.
pub fn root_separation() -> Result<(f64, f64)> {
    let roots = crate::spiralcore_fbs::ProjectionPlan::new(8, 8)?
        .geometry
        .roots;
    let (mut nearest, mut farthest) = (f64::INFINITY, 0.0f64);
    for (i, a) in roots.iter().enumerate() {
        for b in &roots[i + 1..] {
            let l1: i32 = a
                .iter()
                .zip(b)
                .map(|(x, y)| (i32::from(*x) - i32::from(*y)).abs())
                .sum();
            let mean = f64::from(l1) / a.len() as f64;
            nearest = nearest.min(mean);
            farthest = farthest.max(mean);
        }
    }
    Ok((nearest, farthest))
}

/// The pinned DEV panel's windows and its response mask.
fn load_dev(data_root: &Path) -> Result<(Vec<CausalWindow>, Vec<u8>)> {
    let (dev_tokens, dev_mask) = hs::load_pinned_uort_split(
        &data_root.join("ft/dev/tokens.u16"),
        &data_root.join("ft/dev/response_mask.u8"),
        hs::PINNED_DEV_TOKENS_SHA256,
        hs::PINNED_DEV_MASK_SHA256,
        hs::EXPECTED_DEV_TOKENS,
        hs::EXPECTED_DEV_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        hs::SHEET_BOS_ID,
    )?;
    let (dev_windows, _) =
        hs::build_document_windows(&dev_tokens, &dev_mask, SHEET_CONTEXT, hs::SHEET_BOS_ID)?;
    Ok((dev_windows, dev_mask))
}

/// How far back a checkpoint reads from where a reply opens ([`context_reach`]): the opening
/// point's mean absolute shift per dimension when the token `distance` places before it is
/// replaced by BOS, over each DEV window's first opening with that many tokens before it.
#[derive(Clone, Debug, Serialize)]
pub struct Reach {
    pub distance: usize,
    pub rows: usize,
    pub mean_shift: f64,
}

pub const REACH_DISTANCES: [usize; 13] = [0, 1, 2, 3, 4, 5, 6, 8, 10, 12, 16, 24, 32];

/// One recurrence layer's selected rate-table entries over the first 16 DEV windows that open a
/// reply, at their openings and at every position. Entry `k` read as retention holds about
/// `2^(k+1)` tokens; read as the default rate, it holds at most a few.
#[derive(Clone, Debug, Serialize)]
pub struct RateBins {
    pub layer: usize,
    pub openings: [u64; 16],
    pub all_positions: [u64; 16],
}

/// [`Reach`] at each of [`REACH_DISTANCES`] for the checkpoint saved in `checkpoint`, and each
/// layer's [`RateBins`]. Distance 0 replaces the opening's own input token, the end of the prompt.
pub fn context_reach(
    data_root: &Path,
    checkpoint: &Path,
    retain_rates: bool,
    fold_scores: bool,
    reflect_floor: bool,
    seed: u64,
    device: &Device,
) -> Result<(Vec<Reach>, Vec<RateBins>)> {
    let (dev_windows, dev_mask) = load_dev(data_root)?;
    arms::set_fbs_arm(Some(FbsArm::OneAtomActive));
    crate::geometric_stack::set_retention_rates(retain_rates);
    crate::geometric_stack::set_folded_scores(fold_scores);
    crate::geometric_stack::set_reflected_floor(reflect_floor);
    let _guard = ArmGuard;
    let model = StackModel::load(checkpoint, device)?;
    load_reads(checkpoint, device)?;
    let firsts: Vec<(&CausalWindow, usize)> = dev_windows
        .iter()
        .filter_map(|w| {
            w.scored_offsets
                .iter()
                .zip(&w.scored_corpus_positions)
                .find(|&(_, &p)| opens_reply(&dev_mask, p))
                .map(|(&off, _)| (w, off))
        })
        .collect();
    hs::with_gate_mode(HamiltonArm::HamiltonActive, seed, 0, false, || {
        let points = |ids: &[u32], offs: &[usize]| -> Result<Tensor> {
            let hidden = crate::flock_read::points(&model, ids, offs.len(), SHEET_CONTEXT)?;
            let idx: Vec<u32> = offs
                .iter()
                .enumerate()
                .map(|(b, &off)| (b * SHEET_CONTEXT + off) as u32)
                .collect();
            Ok(hidden
                .index_select(&Tensor::from_vec(idx, offs.len(), device)?, 0)?
                .detach())
        };
        let mut reach = Vec::with_capacity(REACH_DISTANCES.len());
        for &distance in &REACH_DISTANCES {
            let eligible: Vec<&(&CausalWindow, usize)> = firsts
                .iter()
                // A row whose replaced token is already BOS would add a zero shift.
                .filter(|(w, off)| {
                    *off >= distance && w.input_ids[*off - distance] != hs::SHEET_BOS_ID
                })
                .collect();
            let (mut total, mut rows) = (0.0f64, 0usize);
            for chunk in eligible.chunks(hs::SHEET_EVAL_BATCH) {
                let offs: Vec<usize> = chunk.iter().map(|(_, off)| *off).collect();
                let mut ids: Vec<u32> = chunk
                    .iter()
                    .flat_map(|(w, _)| w.input_ids.iter().copied())
                    .collect();
                let base = points(&ids, &offs)?;
                for (b, off) in offs.iter().enumerate() {
                    ids[b * SHEET_CONTEXT + off - distance] = hs::SHEET_BOS_ID;
                }
                let moved = points(&ids, &offs)?;
                total += f64::from(
                    moved
                        .sub(&base)?
                        .abs()?
                        .mean(1)?
                        .sum_all()?
                        .to_scalar::<f32>()?,
                );
                rows += chunk.len();
            }
            reach.push(Reach {
                distance,
                rows,
                mean_shift: total / rows.max(1) as f64,
            });
        }
        let probe = &firsts[..firsts.len().min(hs::SHEET_EVAL_BATCH)];
        let ids: Vec<u32> = probe
            .iter()
            .flat_map(|(w, _)| w.input_ids.iter().copied())
            .collect();
        let (_, captured) = crate::secant_descent::with_rate_indices(|| {
            crate::flock_read::points(&model, &ids, probe.len(), SHEET_CONTEXT)
        })?;
        let lanes = table(&model)?.dim(1)? / 4;
        let per_layer = probe.len() * SHEET_CONTEXT * lanes;
        if per_layer == 0 || captured.is_empty() || captured.len() % per_layer != 0 {
            return Err(invalid("rate capture does not split into whole layers"));
        }
        let bins = captured
            .chunks(per_layer)
            .enumerate()
            .map(|(layer, values)| {
                let mut bins = RateBins {
                    layer,
                    openings: [0; 16],
                    all_positions: [0; 16],
                };
                for (at, &k) in values.iter().enumerate() {
                    let position = at / lanes;
                    let (b, t) = (position / SHEET_CONTEXT, position % SHEET_CONTEXT);
                    // BOS padding after a window's last scored target is not text.
                    if probe[b]
                        .0
                        .scored_offsets
                        .last()
                        .is_some_and(|&last| t <= last)
                    {
                        bins.all_positions[usize::from(k)] += 1;
                    }
                    if t == probe[b].1 {
                        bins.openings[usize::from(k)] += 1;
                    }
                }
                bins
            })
            .collect();
        Ok((reach, bins))
    })
}

/// Greedy decoding speed at one context length ([`decode_speed`]).
#[derive(Clone, Debug, Serialize)]
pub struct DecodeSpeed {
    pub context_tokens: usize,
    pub decoded_tokens: usize,
    pub seconds: f64,
    pub tokens_per_second: f64,
}

/// Context lengths for [`decode_speed`].
pub const DECODE_CONTEXTS: [usize; 3] = [16, 64, SHEET_CONTEXT];

/// Greedy decoding speed of the checkpoint in `checkpoint` on one RTX-class GPU: `tokens` tokens
/// after a prompt of each of [`DECODE_CONTEXTS`] tokens taken from the first context-long DEV
/// window. Each token re-runs the trunk over the whole window (no carried state), so this times
/// this runner's decoding, not a served recurrent decode. Nearest decoding under `nearest`, else
/// the writer's.
pub fn decode_speed(
    data_root: &Path,
    checkpoint: &Path,
    retain_rates: bool,
    fold_scores: bool,
    reflect_floor: bool,
    nearest: bool,
    tokens: usize,
    seed: u64,
    device: &Device,
) -> Result<Vec<DecodeSpeed>> {
    let (dev_windows, _) = load_dev(data_root)?;
    arms::set_fbs_arm(Some(FbsArm::OneAtomActive));
    crate::geometric_stack::set_retention_rates(retain_rates);
    crate::geometric_stack::set_folded_scores(fold_scores);
    crate::geometric_stack::set_reflected_floor(reflect_floor);
    let _guard = ArmGuard;
    let model = StackModel::load(checkpoint, device)?;
    load_reads(checkpoint, device)?;
    let source = dev_windows
        .iter()
        .find(|w| w.input_ids.iter().all(|&id| id != hs::SHEET_BOS_ID))
        .unwrap_or(&dev_windows[0]);
    let objective = if nearest {
        PointObjective::Nearest
    } else {
        PointObjective::Sheet
    };
    // One untimed reply first, so no context is charged for kernel loading.
    greedy_reply(&model, objective, &source.input_ids[..16], 4, seed)?;
    let mut speeds = Vec::with_capacity(DECODE_CONTEXTS.len());
    for &context in &DECODE_CONTEXTS {
        let prompt = &source.input_ids[..context.min(source.input_ids.len())];
        let started = Instant::now();
        // BOS stops a reply early; count what was decoded.
        let reply = greedy_reply(&model, objective, prompt, tokens, seed)?;
        device.synchronize()?;
        let seconds = started.elapsed().as_secs_f64();
        // A reply cut short by BOS ran one more forward pass, the one that produced BOS.
        let passes = (reply.len() + 1).min(tokens);
        speeds.push(DecodeSpeed {
            context_tokens: prompt.len(),
            decoded_tokens: passes,
            seconds,
            tokens_per_second: passes as f64 / seconds.max(1e-9),
        });
    }
    Ok(speeds)
}

/// Read every checkpoint a `run_long` run saved under `run/models`, in step order, as an
/// [`Inspection`]. `retain_rates`, `fold_scores` and `reflect_floor` must match the run's.
pub fn inspect_run(
    data_root: &Path,
    run: &Path,
    retain_rates: bool,
    fold_scores: bool,
    reflect_floor: bool,
    seed: u64,
    device: &Device,
) -> Result<Vec<Inspection>> {
    let (dev_windows, dev_mask) = load_dev(data_root)?;
    arms::set_fbs_arm(Some(FbsArm::OneAtomActive));
    crate::geometric_stack::set_retention_rates(retain_rates);
    crate::geometric_stack::set_folded_scores(fold_scores);
    crate::geometric_stack::set_reflected_floor(reflect_floor);
    let _guard = ArmGuard;
    let mut dirs: Vec<PathBuf> = fs::read_dir(run.join("models"))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.join("model.safetensors").exists())
        .collect();
    dirs.sort();
    let probe: Vec<&CausalWindow> = dev_windows.iter().take(hs::SHEET_EVAL_BATCH).collect();
    let (ids, scored, _) = assemble(&probe);
    let mut readings = Vec::with_capacity(dirs.len());
    let last = dirs.len().saturating_sub(1);
    for (index, dir) in dirs.into_iter().enumerate() {
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let step: usize = name.trim_start_matches("step-").parse().unwrap_or(0);
        let model = StackModel::load(&dir, device)?;
        load_reads(&dir, device)?;
        let (dev, _) = hs::evaluate_dev_panel(
            &model,
            &dev_windows,
            HamiltonArm::HamiltonActive,
            seed,
            step,
            false,
            false,
        )?;
        let query = hs::with_gate_mode(HamiltonArm::HamiltonActive, seed, step, false, || {
            let hidden = crate::flock_read::points(&model, &ids, probe.len(), SHEET_CONTEXT)?;
            let idx = Tensor::from_vec(scored.clone(), scored.len(), device)?;
            Ok(hidden.index_select(&idx, 0)?)
        })?;
        let (nearest, _) = nearest_reading(&model, &dev_windows, &dev_mask, seed, step)?;
        // The last checkpoint's register, one slot dropped at a time and then all of them
        // (claude-155): the same nearest-token reading, with the register restored after each.
        let register_drop = match crate::flock_read_register::prompt_register() {
            Some(reg) if index == last => {
                let mut rows = Vec::with_capacity(reg.q() + 1);
                for slot in (0..reg.q()).map(Some).chain([None]) {
                    crate::flock_read_register::set_prompt_register(Some(reg.without_slot(slot)?));
                    let dropped = nearest_reading(&model, &dev_windows, &dev_mask, seed, step);
                    crate::flock_read_register::set_prompt_register(Some(reg.clone()));
                    let (dropped, _) = dropped?;
                    rows.push(RegisterDrop {
                        slot: slot.map_or_else(|| "all".into(), serde_json::Value::from),
                        hits: dropped.hits,
                        opening_hits: dropped.opening_hits,
                        copied_hits: dropped.copied_hits,
                    });
                }
                Some(rows)
            }
            _ => None,
        };
        // The last checkpoint's reading with the turn memory taken out (claude-161), restored after.
        let memory_drop = match crate::flock_read_memory::turn_memory() {
            Some(memory) if index == last => {
                crate::flock_read_memory::set_turn_memory(None);
                let dropped = nearest_reading(&model, &dev_windows, &dev_mask, seed, step);
                crate::flock_read_memory::set_turn_memory(Some(memory));
                let (dropped, _) = dropped?;
                Some(RegisterDrop {
                    slot: "memory".into(),
                    hits: dropped.hits,
                    opening_hits: dropped.opening_hits,
                    copied_hits: dropped.copied_hits,
                })
            }
            _ => None,
        };
        let first_cells = model
            .native_head_candidates(&query)?
            .cell_ids()
            .narrow(1, 0, 1)?
            .flatten_all()?
            .to_vec1::<u32>()?;
        let centred = query.broadcast_sub(&query.mean_keepdim(0)?)?;
        let scalar = |t: Tensor| -> Result<f64> { Ok(f64::from(t.to_scalar::<f32>()?)) };
        readings.push(Inspection {
            checkpoint: name,
            scored_rows: dev.scored_rows,
            top1_hits: dev.top1_hits,
            nearest,
            admitted_hits: dev.admitted_target_hits,
            token_unviolated_hits: dev.token_unviolated_hits,
            token_margin: dev.token_margin,
            address_margin: dev.address_margin,
            probe_rows: first_cells.len(),
            distinct_first_cells: first_cells
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            query_spread: scalar(centred.sqr()?.mean(0)?.sqrt()?.mean_all()?)?,
            query_mean_abs: scalar(query.abs()?.mean_all()?)?,
            table_mean_abs: scalar(
                model.variables()["embedding.weight"]
                    .as_tensor()
                    .abs()?
                    .mean_all()?,
            )?,
            flock_weight_mean_abs: match crate::flock_read::flock_read() {
                Some(read) => Some(scalar(read.weight_mean_abs()?)?),
                None => None,
            },
            content_weight_mean_abs: match crate::flock_read_content::content_read() {
                Some(content) => Some(scalar(content.read.weight_mean_abs()?)?),
                None => None,
            },
            hamming_weight_mean_abs: match crate::flock_read_hamming::hamming_reads() {
                Some(reads) => Some(scalar(reads.weight_mean_abs()?)?),
                None => None,
            },
            opening_weight_mean_abs: match crate::flock_read_opening::opening_read() {
                Some(read) => Some(scalar(read.weight_mean_abs()?)?),
                None => None,
            },
            register_weight_mean_abs: match crate::flock_read_register::prompt_register() {
                Some(reg) => Some(scalar(reg.weight_mean_abs()?)?),
                None => None,
            },
            register_drop,
            memory_weight_mean_abs: match crate::flock_read_memory::turn_memory() {
                Some(memory) => Some(scalar(memory.weight_mean_abs()?)?),
                None => None,
            },
            memory_reply_weight_mean_abs: match crate::flock_read_memory::turn_memory() {
                Some(memory) => match memory.reply_weight_mean_abs()? {
                    Some(mean) => Some(scalar(mean)?),
                    None => None,
                },
                None => None,
            },
            memory_drop,
        });
    }
    Ok(readings)
}

/// Key, value and filler token ranges for the recall task: disjoint, and clear of BOS (0).
pub const RECALL_KEYS: std::ops::Range<u32> = 16..528;
pub const RECALL_VALUES: std::ops::Range<u32> = 528..1_040;
pub const RECALL_FILLER: std::ops::Range<u32> = 1_040..4_096;
/// The recall task's query distance buckets, `d = q − p`, each drawn uniformly inside.
pub const RECALL_BUCKETS: [(usize, usize); 4] = [(2, 8), (9, 32), (33, 96), (97, 180)];
/// Held-out recall windows scored at each checkpoint.
pub const RECALL_EVAL_WINDOWS: u64 = 128;
/// The held-out windows' seed offset. A held-out window equals a training window only if their
/// generator states collide, which for these seeds needs a training index far past any run.
const RECALL_EVAL_SEED: u64 = 0x5EED_F00D;

/// One window of the multi-query associative recall task, after
/// `uor-r4-training/examples/mqar-bench.rs`, and each scored row's distance bucket.
///
/// The window is `context` tokens, BOS first. Each pair writes its key at `p` and its value at
/// `p + 1`. Its query writes the key again at `q = p + d`, with `d` drawn inside the pair's
/// bucket, and the only scored target, at offset `q`, is the value. Keys, values and filler come
/// from disjoint ranges. Keys and values are distinct within a window, and pairings are fresh
/// for every `(seed, index)`, so a value can only be recalled from the context.
///
/// The longest bucket is placed first, while the window is empty, so its distances stay uniform.
/// A window that cannot hold every pair is an error, never a thinner bucket.
pub fn recall_window(
    seed: u64,
    index: u64,
    context: usize,
    pairs_per_bucket: usize,
) -> Result<(CausalWindow, Vec<usize>)> {
    let mut state = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(index.wrapping_mul(0xD1B5_4A32_D192_ED03))
        | 1;
    let mut next = move || {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    };
    let draw = |range: &std::ops::Range<u32>, x: u64| {
        range.start + (x % u64::from(range.end - range.start)) as u32
    };
    let mut ids: Vec<u32> = (0..context).map(|_| draw(&RECALL_FILLER, next())).collect();
    ids[0] = hs::SHEET_BOS_ID;
    let mut taken = vec![false; context];
    taken[0] = true;
    let (mut keys, mut values) = (Vec::new(), Vec::new());
    let mut rows: Vec<(usize, u32, usize)> = Vec::new();
    for (bucket, &(low, high)) in RECALL_BUCKETS.iter().enumerate().rev() {
        let (mut placed, mut tries) = (0, 0);
        while placed < pairs_per_bucket && tries < 10_000 {
            tries += 1;
            let d = low + (next() % (high - low + 1) as u64) as usize;
            if d + 3 >= context {
                continue;
            }
            let p = 1 + (next() % (context - d - 2) as u64) as usize;
            let q = p + d;
            if [p, p + 1, q, q + 1]
                .iter()
                .any(|&i| i >= context || taken[i])
            {
                continue;
            }
            let (key, value) = (draw(&RECALL_KEYS, next()), draw(&RECALL_VALUES, next()));
            if keys.contains(&key) || values.contains(&value) {
                continue;
            }
            for i in [p, p + 1, q, q + 1] {
                taken[i] = true;
            }
            (ids[p], ids[p + 1], ids[q], ids[q + 1]) = (key, value, key, value);
            keys.push(key);
            values.push(value);
            rows.push((q, value, bucket));
            placed += 1;
        }
        if placed < pairs_per_bucket {
            return Err(invalid(format!(
                "a {context}-token recall window holds only {placed} of {pairs_per_bucket} pairs in bucket {low}-{high}"
            )));
        }
    }
    rows.sort_unstable();
    let base = index as usize * context;
    let window = CausalWindow {
        doc_id: index as usize,
        input_ids: ids,
        scored_offsets: rows.iter().map(|r| r.0).collect(),
        scored_targets: rows.iter().map(|r| r.1).collect(),
        scored_corpus_positions: rows.iter().map(|r| base + r.0 + 1).collect(),
    };
    Ok((window, rows.iter().map(|r| r.2).collect()))
}

/// One recall checkpoint's reading, one line of `recall.jsonl` ([`run_recall`]).
#[derive(Clone, Debug, Serialize)]
pub struct RecallReading {
    pub step: usize,
    pub train_loss_mean: Option<f64>,
    /// Held-out queries per distance bucket ([`RECALL_BUCKETS`]), and how many of them the whole
    /// table's nearest token to the predicted point names right.
    pub rows: [usize; 4],
    pub hits: [usize; 4],
    pub train_seconds: f64,
    pub train_context_tokens_per_second: f64,
    /// Each read's weight, mean of its absolute values, when the run has that read.
    pub content_weight_mean_abs: Option<f64>,
    pub hamming_weight_mean_abs: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opening_weight_mean_abs: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_weight_mean_abs: Option<f64>,
    /// The Hamming reads' margins in bits, read by read and head by head: a margin below every
    /// distance weighs nothing and gets no gradient back.
    pub hamming_margins: Option<Vec<f64>>,
}

/// Recall hits per bucket on the held-out `windows`, decoded as the whole table's nearest token.
fn recall_hits(
    model: &StackModel,
    windows: &[(CausalWindow, Vec<usize>)],
    seed: u64,
) -> Result<([usize; 4], [usize; 4])> {
    let table = table(model)?;
    let (mut rows, mut hits) = ([0usize; 4], [0usize; 4]);
    hs::with_gate_mode(HamiltonArm::HamiltonActive, seed, 0, false, || {
        for chunk in windows.chunks(16) {
            let refs: Vec<&CausalWindow> = chunk.iter().map(|(w, _)| w).collect();
            let (ids, scored, targets) = assemble(&refs);
            let hidden = crate::flock_read::points(model, &ids, refs.len(), SHEET_CONTEXT)?;
            let idx = Tensor::from_vec(scored, targets.len(), model.device())?;
            let picks = nearest_scores(&hidden.index_select(&idx, 0)?, &table)?
                .argmax(1)?
                .to_vec1::<u32>()?;
            let buckets = chunk.iter().flat_map(|(_, b)| b.iter().copied());
            for ((pick, target), bucket) in picks.iter().zip(&targets).zip(buckets) {
                rows[bucket] += 1;
                hits[bucket] += usize::from(pick == target);
            }
        }
        Ok(())
    })?;
    Ok((rows, hits))
}

/// Train the trunk on the recall task, fresh windows every step, with `config`'s reads and
/// optimizer settings as [`run_long`] uses them, scoring held-out recall per distance bucket
/// every `eval_every` steps into `out/recall.jsonl`.
pub fn run_recall(
    out: &Path,
    config: &LongConfig,
    pairs_per_bucket: usize,
    device: &Device,
) -> Result<()> {
    if config.steps == 0 || config.batch == 0 || config.eval_every == 0 || pairs_per_bucket == 0 {
        return Err(invalid(
            "steps, batch, eval-every and pairs must be positive",
        ));
    }
    if out.exists() {
        return Err(invalid(format!(
            "{} exists; a run never writes over one",
            out.display()
        )));
    }
    fs::create_dir_all(out)?;
    fs::write(out.join("config.json"), serde_json::to_vec_pretty(config)?)?;
    // The task itself, for the record: `config.json` holds only the runner's settings.
    let task =
        serde_json::json!({ "pairs_per_bucket": pairs_per_bucket, "context": SHEET_CONTEXT });
    fs::write(out.join("recall.json"), serde_json::to_vec_pretty(&task)?)?;
    arms::set_fbs_arm(Some(FbsArm::OneAtomActive));
    crate::geometric_stack::set_retention_rates(config.retain_rates);
    crate::geometric_stack::set_folded_scores(config.fold_scores);
    crate::geometric_stack::set_reflected_floor(config.reflect_floor);
    crate::geometric_stack::set_input_table_fixed(config.fix_input_table);
    let _guard = ArmGuard;
    let model = build_fbs_model_at(FbsArm::OneAtomActive, config.seed, config.width, device)?;
    let mut flock = if config.flock_read > 0 {
        let read = crate::flock_read::FlockRead::new(config.flock_read, config.width, device)?;
        crate::flock_read::set_flock_read(Some(read.clone()));
        let opt = crate::flock_read::FlockAdamW::new(&read)?;
        Some((read, opt))
    } else {
        None
    };
    let mut content = if config.content_read > 0 {
        let read =
            crate::flock_read_content::ContentRead::new(config.content_read, config.width, device)?;
        crate::flock_read_content::set_content_read(Some(read.clone()));
        let opt = crate::flock_read::FlockAdamW::new(&read.read)?;
        Some((read, opt))
    } else {
        None
    };
    // The Hamming reads, when on: their own optimizer, applied inside the trunk.
    let mut hamming = start_hamming(config, device)?;
    // The opening read (claude-154), when on: its own optimizer, applied inside the trunk.
    let mut opening = start_opening(config, device)?;
    // The prompt register (claude-155), when on: its own optimizer, applied inside the trunk.
    let mut register = start_register(config, device)?;
    let mut opt_main = StackAdamW::new(&model, hs::SHEET_WEIGHT_DECAY, hs::SHEET_CLIP)?;
    let mut opt_decay = StackAdamW::new(&model, 0.0, hs::SHEET_CLIP)?;
    let held_out: Vec<(CausalWindow, Vec<usize>)> = (0..RECALL_EVAL_WINDOWS)
        .map(|i| {
            recall_window(
                config.seed ^ RECALL_EVAL_SEED,
                i,
                SHEET_CONTEXT,
                pairs_per_bucket,
            )
        })
        .collect::<Result<_>>()?;
    let log = out.join("recall.jsonl");
    let (mut train_seconds, mut loss_sum, mut loss_steps) = (0.0f64, 0.0f64, 0usize);
    let reading = |step: usize, loss: Option<f64>, seconds: f64| -> Result<()> {
        let (rows, hits) = recall_hits(&model, &held_out, config.seed)?;
        let context_tokens = (step * config.batch * SHEET_CONTEXT) as f64;
        let scalar = |t: Tensor| -> Result<f64> { Ok(f64::from(t.to_scalar::<f32>()?)) };
        let line = RecallReading {
            step,
            train_loss_mean: loss,
            rows,
            hits,
            train_seconds: seconds,
            train_context_tokens_per_second: if seconds > 0.0 {
                context_tokens / seconds
            } else {
                0.0
            },
            content_weight_mean_abs: match crate::flock_read_content::content_read() {
                Some(read) => Some(scalar(read.read.weight_mean_abs()?)?),
                None => None,
            },
            hamming_weight_mean_abs: match crate::flock_read_hamming::hamming_reads() {
                Some(reads) => Some(scalar(reads.weight_mean_abs()?)?),
                None => None,
            },
            opening_weight_mean_abs: match crate::flock_read_opening::opening_read() {
                Some(read) => Some(scalar(read.weight_mean_abs()?)?),
                None => None,
            },
            register_weight_mean_abs: match crate::flock_read_register::prompt_register() {
                Some(reg) => Some(scalar(reg.weight_mean_abs()?)?),
                None => None,
            },
            hamming_margins: match crate::flock_read_hamming::hamming_reads() {
                Some(reads) => Some(reads.margins()?),
                None => None,
            },
        };
        println!(
            "[recall] step {step} hits {:?} of {:?} train {seconds:.0}s",
            line.hits, line.rows
        );
        append_line(&log, &serde_json::to_string(&line)?)
    };
    reading(0, None, 0.0)?;
    for step in 0..config.steps {
        let windows: Vec<CausalWindow> = (0..config.batch)
            .map(|b| {
                recall_window(
                    config.seed,
                    (step * config.batch + b) as u64,
                    SHEET_CONTEXT,
                    pairs_per_bucket,
                )
                .map(|(window, _)| window)
            })
            .collect::<Result<_>>()?;
        let refs: Vec<&CausalWindow> = windows.iter().collect();
        let (ids, scored, targets) = assemble(&refs);
        let started = Instant::now();
        let (value, grads) = hs::with_gate_mode(
            HamiltonArm::HamiltonActive,
            config.seed,
            step + 1,
            false,
            || {
                let loss = batch_loss(
                    &model,
                    config.objective,
                    &ids,
                    &scored,
                    &targets,
                    config.batch,
                )?;
                let value = f64::from(loss.to_scalar::<f32>()?);
                if !value.is_finite() {
                    return Err(invalid(format!("non-finite recall loss at step {step}")));
                }
                Ok((value, loss.backward()?))
            },
        )?;
        let progress = if config.steps == 1 {
            1.0
        } else {
            step as f64 / (config.steps - 1) as f64
        };
        let lr = config.lr + (config.lr_end - config.lr) * progress;
        opt_main.update_filtered(&model, &grads, lr, |name| !name.ends_with("rec.decay"))?;
        if let Some((read, opt)) = flock.as_mut() {
            opt.step(read, &grads, lr)?;
        }
        if let Some((read, opt)) = content.as_mut() {
            opt.step(&read.read, &grads, lr)?;
        }
        if let Some((reads, opt)) = hamming.as_mut() {
            opt.step_vars(&reads.variables(), &grads, lr)?;
        }
        if let Some((read, opt)) = opening.as_mut() {
            opt.step_vars(&read.variables(), &grads, lr)?;
        }
        if let Some((reg, opt)) = register.as_mut() {
            opt.step_vars(&reg.variables(), &grads, lr)?;
        }
        opt_decay.update_filtered(&model, &grads, lr * config.decay_mult, |name| {
            name.ends_with("rec.decay")
        })?;
        device.synchronize()?;
        train_seconds += started.elapsed().as_secs_f64();
        loss_sum += value;
        loss_steps += 1;
        if (step + 1) % config.eval_every == 0 || step + 1 == config.steps {
            reading(step + 1, Some(loss_sum / loss_steps as f64), train_seconds)?;
            (loss_sum, loss_steps) = (0.0, 0);
        }
    }
    // The final model and its reads, for the record and for any later reading.
    let directory = out.join(format!("models/step-{:06}", config.steps));
    model.save(&directory)?;
    if let Some(read) = crate::flock_read::flock_read() {
        read.save(&directory)?;
    }
    if let Some(read) = crate::flock_read_content::content_read() {
        read.save(&directory)?;
    }
    if let Some(reads) = crate::flock_read_hamming::hamming_reads() {
        reads.save(&directory)?;
    }
    if let Some(read) = crate::flock_read_opening::opening_read() {
        read.save(&directory)?;
    }
    if let Some(reg) = crate::flock_read_register::prompt_register() {
        reg.save(&directory)?;
    }
    Ok(())
}

/// The pinned tokenizer.
fn pinned_tokenizer(data_root: &Path) -> Result<ByteBpeTokenizer> {
    let tokenizer_bytes = fs::read(data_root.join("tokenizer.json"))?;
    if sha256_bytes(&tokenizer_bytes) != hs::PINNED_TOKENIZER_SHA256 {
        return Err(invalid(
            "tokenizer.json differs from the pinned d36d3e87 tokenizer",
        ));
    }
    ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| invalid("tokenizer.json is not a byte-level BPE tokenizer"))
}

/// The pinned TRAIN split's tokens and response mask.
fn load_train_split(data_root: &Path) -> Result<(Vec<u32>, Vec<u8>)> {
    hs::load_pinned_uort_split(
        &data_root.join("mw-balp/train/tokens.u16"),
        &data_root.join("mw-balp/train/response_mask.u8"),
        hs::PINNED_TRAIN_TOKENS_SHA256,
        hs::PINNED_TRAIN_MASK_SHA256,
        hs::EXPECTED_TRAIN_TOKENS,
        hs::EXPECTED_TRAIN_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        hs::SHEET_BOS_ID,
    )
}

/// `split`'s tokens and response mask, refused unless both files hash to the sha256s it names,
/// the tokens carry a `UORT` header whose count matches both payloads, and every token is a
/// vocabulary id after an opening BOS (claude-152).
fn load_named_split(split: &TrainSplit) -> Result<(Vec<u32>, Vec<u8>)> {
    let token_bytes = fs::read(split.dir.join("tokens.u16"))?;
    let mask = fs::read(split.dir.join("response_mask.u8"))?;
    for (name, bytes, want) in [
        ("tokens.u16", &token_bytes, &split.tokens_sha256),
        ("response_mask.u8", &mask, &split.mask_sha256),
    ] {
        let got = sha256_bytes(bytes);
        if &got != want {
            return Err(invalid(format!(
                "{name} in {} hashes to {got}, not {want}",
                split.dir.display()
            )));
        }
    }
    const HEADER: usize = 64;
    if token_bytes.len() < HEADER || &token_bytes[..4] != b"UORT" {
        return Err(invalid("tokens.u16 lacks its 64-byte UORT header"));
    }
    let count = u64::from_le_bytes(
        token_bytes[8..16]
            .try_into()
            .map_err(|_| invalid("a short UORT header"))?,
    ) as usize;
    if token_bytes.len() != HEADER + 2 * count || mask.len() != count {
        return Err(invalid(format!(
            "the UORT header says {count} tokens; the payload or the mask disagrees"
        )));
    }
    let tokens: Vec<u32> = token_bytes[HEADER..]
        .chunks_exact(2)
        .map(|w| u32::from(u16::from_le_bytes([w[0], w[1]])))
        .collect();
    if tokens.first() != Some(&hs::SHEET_BOS_ID)
        || tokens.iter().any(|&t| t as usize >= hs::SHEET_VOCAB)
    {
        return Err(invalid(
            "the split does not open with BOS, or has an id past the vocabulary",
        ));
    }
    Ok((tokens, mask))
}

/// TRAIN's windows with at least 4 scored targets, each context-long; at least `batch` of them.
/// TRAIN is the pinned split, or `split` when a run names one.
fn load_train(
    data_root: &Path,
    batch: usize,
    split: Option<&TrainSplit>,
) -> Result<Vec<CausalWindow>> {
    let (train_tokens, train_mask) = match split {
        None => load_train_split(data_root)?,
        Some(split) => load_named_split(split)?,
    };
    let (train_windows, _) =
        hs::build_document_windows(&train_tokens, &train_mask, SHEET_CONTEXT, hs::SHEET_BOS_ID)?;
    let usable: Vec<CausalWindow> = train_windows
        .into_iter()
        .filter(|w| w.scored_offsets.len() >= 4)
        .collect();
    if usable.len() < batch || usable.iter().any(|w| w.input_ids.len() != SHEET_CONTEXT) {
        return Err(invalid("TRAIN windows are too few or not context-long"));
    }
    Ok(usable)
}

/// One parameter's gradient norm on the first training batch ([`gradient_norms`]).
#[derive(Clone, Debug, Serialize)]
pub struct GradientNorm {
    pub name: String,
    pub norm: f64,
}

/// The L2 norm of every parameter's gradient on the first `config.batch` TRAIN windows of a
/// freshly built model under `config`'s objective and switches, largest first, after one line
/// named `total` and one with the norm the main optimizer clips. No update is made.
pub fn gradient_norms(
    data_root: &Path,
    config: &LongConfig,
    device: &Device,
) -> Result<Vec<GradientNorm>> {
    let usable = load_train(data_root, config.batch, config.train_split.as_ref())?;
    arms::set_fbs_arm(Some(FbsArm::OneAtomActive));
    crate::geometric_stack::set_retention_rates(config.retain_rates);
    crate::geometric_stack::set_folded_scores(config.fold_scores);
    crate::geometric_stack::set_reflected_floor(config.reflect_floor);
    crate::geometric_stack::set_input_table_fixed(config.fix_input_table);
    let _guard = ArmGuard;
    let model = build_fbs_model_at(FbsArm::OneAtomActive, config.seed, config.width, device)?;
    if config.flock_read > 0 {
        crate::flock_read::set_flock_read(Some(crate::flock_read::FlockRead::new(
            config.flock_read,
            config.width,
            device,
        )?));
    }
    if config.content_read > 0 {
        crate::flock_read_content::set_content_read(Some(
            crate::flock_read_content::ContentRead::new(config.content_read, config.width, device)?,
        ));
    }
    let _hamming = start_hamming(config, device)?;
    let _opening = start_opening(config, device)?;
    let _register = start_register(config, device)?;
    let windows: Vec<&CausalWindow> = usable.iter().take(config.batch).collect();
    let (ids, scored, targets) = assemble(&windows);
    let grads = hs::with_gate_mode(HamiltonArm::HamiltonActive, config.seed, 0, false, || {
        Ok(batch_loss(
            &model,
            config.objective,
            &ids,
            &scored,
            &targets,
            config.batch,
        )?
        .backward()?)
    })?;
    let mut norms = Vec::new();
    let (mut total, mut main) = (0.0f64, 0.0f64);
    for (name, var) in model.variables() {
        if let Some(grad) = grads.get(var.as_tensor()) {
            let squared = f64::from(grad.sqr()?.sum_all()?.to_scalar::<f32>()?);
            total += squared;
            if !name.ends_with("rec.decay") {
                main += squared;
            }
            norms.push(GradientNorm {
                name: name.clone(),
                norm: squared.sqrt(),
            });
        }
    }
    // The shared table's gradient split by role: the output side alone, from the loss on the
    // predicted points with the trunk cut off, and the input side, the rest.
    let table_var = model
        .variables()
        .get("embedding.weight")
        .ok_or_else(|| invalid("the model has no shared vector table"))?;
    let output_side =
        hs::with_gate_mode(HamiltonArm::HamiltonActive, config.seed, 0, false, || {
            let hidden = crate::flock_read::points(&model, &ids, config.batch, SHEET_CONTEXT)?;
            let idx = Tensor::from_vec(scored.clone(), scored.len(), device)?;
            let query = hidden.index_select(&idx, 0)?.detach();
            Ok(query_loss(&model, config.objective, &query, &targets)?.backward()?)
        })?;
    if let (Some(full), Some(output)) = (
        grads.get(table_var.as_tensor()),
        output_side.get(table_var.as_tensor()),
    ) {
        let norm = |t: &Tensor| -> Result<f64> {
            Ok(f64::from(t.sqr()?.sum_all()?.to_scalar::<f32>()?).sqrt())
        };
        norms.push(GradientNorm {
            name: "embedding.weight, output side".to_owned(),
            norm: norm(output)?,
        });
        norms.push(GradientNorm {
            name: "embedding.weight, input side".to_owned(),
            norm: norm(&full.sub(output)?)?,
        });
    }
    norms.sort_by(|a, b| b.norm.total_cmp(&a.norm));
    norms.insert(
        0,
        GradientNorm {
            name: "total".to_owned(),
            norm: total.sqrt(),
        },
    );
    // The main optimizer clips everything but `rec.decay` together (`run_long`).
    norms.insert(
        1,
        GradientNorm {
            name: "total_without_rec.decay".to_owned(),
            norm: main.sqrt(),
        },
    );
    Ok(norms)
}

/// Train, score, write text and save, as the module documentation says. `out` must not exist.
/// Training steps run before the profile's window opens (fbs-cost-profile, claude-157): past
/// step 0's diagnostics and the first steps' allocations.
pub const PROFILE_WARMUP: usize = 8;

/// Where `step` sits in a `profile`-step window: `Some(true)` for a profiled step, `Some(false)`
/// for the unprofiled step after it, whose time it is compared with, and `None` outside the
/// window or when `profile` is 0. The window is the `2 * profile` steps after
/// [`PROFILE_WARMUP`], profiled and unprofiled in turn.
pub fn profile_window(step: usize, profile: usize) -> Option<bool> {
    if profile == 0
        || step < PROFILE_WARMUP
        || step >= PROFILE_WARMUP.saturating_add(profile.saturating_mul(2))
    {
        return None;
    }
    Some((step - PROFILE_WARMUP).is_multiple_of(2))
}

/// One profiled training step, one line of `profile.jsonl`.
#[derive(Clone, Debug, Serialize)]
pub struct ProfileRow {
    pub step: usize,
    pub loss: f64,
    /// The step's host time through its closing wait for the GPU, as every step is timed.
    pub step_ms: f64,
    /// Each stage from the mark before it, in step order; the last, `end`, is the closing mark,
    /// with nothing queued before it since `optimizer.decay`, so about zero GPU time.
    pub stages: Vec<StageTime>,
    /// The stages' GPU times summed: the GPU's span from the step's first mark to its last.
    pub gpu_span_ms: f64,
    /// The unprofiled step after this one, timed as every training step is.
    pub next_unprofiled_ms: Option<f64>,
}

/// One stage's medians over the profiled steps.
#[derive(Clone, Debug, Serialize)]
pub struct StageSummary {
    pub name: String,
    pub gpu_ms: f64,
    pub host_ms: f64,
}

/// `profile.json`: the profiled steps' stages against the unprofiled steps' time.
#[derive(Clone, Debug, Serialize)]
pub struct ProfileSummary {
    pub profiled_steps: usize,
    /// Each stage's median, in step order.
    pub stages: Vec<StageSummary>,
    /// The stage medians' GPU times summed by group: `forward`, `objective`, `backward`,
    /// `optimizer` and `end`.
    pub groups: Vec<(String, f64)>,
    /// The stage medians' GPU times summed.
    pub stage_sum_ms: f64,
    /// The median of the unprofiled steps' times.
    pub median_unprofiled_ms: f64,
    /// `stage_sum_ms / median_unprofiled_ms`.
    pub ratio: f64,
    /// The item's bar: the stages sum to the measured step time within 10%.
    pub within_10pct: bool,
    /// The median of the profiled steps' host times, timed as the unprofiled ones are.
    pub median_profiled_ms: f64,
    /// `median_profiled_ms / median_unprofiled_ms - 1`: what the clock itself adds to a step. The
    /// stage sum cannot show it, since both spans start and end with the GPU idle.
    pub clock_overhead: f64,
    /// The three stages with the largest median GPU time, largest first.
    pub top3: Vec<String>,
    /// The same, among the forward and objective stages only: the backward holds every stage's
    /// gradient in one time.
    pub top3_forward: Vec<String>,
}

/// The median of `values` (the mean of the middle two for an even count); NaN when empty.
fn median(values: &mut [f64]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    values.sort_by(f64::total_cmp);
    let mid = values.len() / 2;
    if values.len() % 2 == 1 {
        values[mid]
    } else {
        (values[mid - 1] + values[mid]) / 2.0
    }
}

/// The group a stage's time counts in.
fn stage_group(name: &str) -> &'static str {
    if name == "points.select" || name.starts_with("objective.") {
        "objective"
    } else if name == "backward" {
        "backward"
    } else if name.starts_with("optimizer.") {
        "optimizer"
    } else if name == "end" {
        "end"
    } else {
        "forward"
    }
}

/// The profile's summary: each stage's medians over `rows` in the first row's stage order, the
/// group sums, and the sum against the unprofiled steps' median.
pub fn summarize_profile(rows: &[ProfileRow]) -> ProfileSummary {
    let names: Vec<String> = rows
        .first()
        .map(|r| r.stages.iter().map(|s| s.name.clone()).collect())
        .unwrap_or_default();
    let stages: Vec<StageSummary> = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let mut gpu: Vec<f64> = rows
                .iter()
                .filter_map(|r| r.stages.get(i).and_then(|s| s.gpu_ms))
                .collect();
            let mut host: Vec<f64> = rows
                .iter()
                .filter_map(|r| r.stages.get(i).map(|s| s.host_ms))
                .collect();
            StageSummary {
                name: name.clone(),
                gpu_ms: median(&mut gpu),
                host_ms: median(&mut host),
            }
        })
        .collect();
    let mut groups: Vec<(String, f64)> = ["forward", "objective", "backward", "optimizer", "end"]
        .iter()
        .map(|g| (g.to_string(), 0.0))
        .collect();
    for s in &stages {
        if let Some(g) = groups.iter_mut().find(|(g, _)| g == stage_group(&s.name)) {
            g.1 += s.gpu_ms;
        }
    }
    let stage_sum_ms: f64 = stages.iter().map(|s| s.gpu_ms).sum();
    let mut unprofiled: Vec<f64> = rows.iter().filter_map(|r| r.next_unprofiled_ms).collect();
    let median_unprofiled_ms = median(&mut unprofiled);
    let ratio = stage_sum_ms / median_unprofiled_ms;
    let mut profiled: Vec<f64> = rows.iter().map(|r| r.step_ms).collect();
    let median_profiled_ms = median(&mut profiled);
    let mut ranked: Vec<&StageSummary> = stages.iter().collect();
    ranked.sort_by(|a, b| b.gpu_ms.total_cmp(&a.gpu_ms));
    let names_of = |keep: &dyn Fn(&str) -> bool| -> Vec<String> {
        ranked
            .iter()
            .filter(|s| keep(&s.name))
            .take(3)
            .map(|s| s.name.clone())
            .collect()
    };
    let top3 = names_of(&|_| true);
    let top3_forward = names_of(&|n| matches!(stage_group(n), "forward" | "objective"));
    ProfileSummary {
        profiled_steps: rows.len(),
        top3,
        top3_forward,
        stages,
        groups,
        stage_sum_ms,
        median_unprofiled_ms,
        ratio,
        within_10pct: (ratio - 1.0).abs() <= 0.10,
        median_profiled_ms,
        clock_overhead: median_profiled_ms / median_unprofiled_ms - 1.0,
    }
}

pub fn run_long(data_root: &Path, out: &Path, config: &LongConfig, device: &Device) -> Result<()> {
    if config.steps == 0 || config.batch == 0 || config.eval_every == 0 {
        return Err(invalid("steps, batch and eval-every must be positive"));
    }
    // A clock left on by a run that failed mid-step on this thread starts no stage of this one.
    stage_clock::reset();
    if config.profile > 0 {
        let window_end = config
            .profile
            .checked_mul(2)
            .and_then(|w| w.checked_add(PROFILE_WARMUP))
            .ok_or_else(|| invalid(format!("--profile {} is too large", config.profile)))?;
        if config.steps < window_end {
            return Err(invalid(format!(
                "--profile {} needs at least {window_end} steps: {PROFILE_WARMUP} of warm-up, \
                 then each profiled step and the unprofiled step after it",
                config.profile
            )));
        }
        if !device.is_cuda() {
            return Err(invalid(
                "--profile times the GPU's stages and needs a CUDA device",
            ));
        }
    }
    if out.exists() {
        return Err(invalid(format!(
            "{} exists; a run never writes over one",
            out.display()
        )));
    }
    fs::create_dir_all(out)?;
    fs::write(out.join("config.json"), serde_json::to_vec_pretty(config)?)?;

    let tokenizer = pinned_tokenizer(data_root)?;

    let usable = load_train(data_root, config.batch, config.train_split.as_ref())?;
    let (dev_windows, dev_mask) = load_dev(data_root)?;
    let prompts = sample_windows(&dev_windows, &dev_mask, config.samples);
    println!(
        "[in-space] usable TRAIN windows {} (batch {}, {} steps = {:.2} epochs), DEV windows {}",
        usable.len(),
        config.batch,
        config.steps,
        (config.steps * config.batch) as f64 / usable.len() as f64,
        dev_windows.len()
    );

    arms::set_fbs_arm(Some(FbsArm::OneAtomActive));
    crate::geometric_stack::set_retention_rates(config.retain_rates);
    crate::geometric_stack::set_folded_scores(config.fold_scores);
    crate::geometric_stack::set_reflected_floor(config.reflect_floor);
    crate::geometric_stack::set_input_table_fixed(config.fix_input_table);
    let _guard = ArmGuard;
    let model = build_fbs_model_at(FbsArm::OneAtomActive, config.seed, config.width, device)?;
    // The Flock read, when on: its own parameters and optimizer, applied by `points`.
    let flock = if config.flock_read > 0 {
        let read = crate::flock_read::FlockRead::new(config.flock_read, config.width, device)?;
        crate::flock_read::set_flock_read(Some(read.clone()));
        Some((read.clone(), crate::flock_read::FlockAdamW::new(&read)?))
    } else {
        None
    };
    let mut flock = flock;
    // The content read, when on: claude-144's parameters and optimizer, applied inside the trunk.
    let mut content = if config.content_read > 0 {
        let read =
            crate::flock_read_content::ContentRead::new(config.content_read, config.width, device)?;
        crate::flock_read_content::set_content_read(Some(read.clone()));
        let opt = crate::flock_read::FlockAdamW::new(&read.read)?;
        Some((read, opt))
    } else {
        None
    };
    // The Hamming reads, when on: their own optimizer, applied inside the trunk.
    let mut hamming = start_hamming(config, device)?;
    // The opening read (claude-154), when on: its own optimizer, applied inside the trunk.
    let mut opening = start_opening(config, device)?;
    // The prompt register (claude-155), when on: its own optimizer, applied inside the trunk.
    let mut register = start_register(config, device)?;
    // The turn memory (claude-161), when on: its table from TRAIN, its own optimizer, and each
    // usable window's opening offsets, which name the rows it moves.
    let (mut memory, train_openings, train_replies) =
        start_memory(data_root, config, &usable, device)?;
    // How often the read's own key finds the table at an opening row, on TRAIN's usable windows
    // and on DEV: the table is keyed from the probe's prompts and read through the turn anchor,
    // so this is the measured agreement, not one assumed.
    if let Some((mem, _)) = memory.as_ref() {
        let train_rows: Vec<(usize, Vec<usize>)> = train_openings
            .iter()
            .enumerate()
            .map(|(i, offsets)| (i, offsets.clone()))
            .collect();
        let (rows, hits) = memory_hits(mem, &usable, &train_rows)?;
        println!("[memory] TRAIN opening rows {rows}, hits {hits}");
        let dev_rows: Vec<(usize, Vec<usize>)> = dev_windows
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let offsets = w
                    .scored_offsets
                    .iter()
                    .zip(&w.scored_corpus_positions)
                    .filter(|(_, &position)| opens_reply(&dev_mask, position))
                    .map(|(&offset, _)| offset)
                    .collect();
                (i, offsets)
            })
            .collect();
        let (rows, hits) = memory_hits(mem, &dev_windows, &dev_rows)?;
        println!("[memory] DEV opening rows {rows}, hits {hits}");
        // The same for the reply table (claude-193), keyed by the probe's own key.
        if mem.reply_len().is_some() {
            let count = |rows: &[Vec<(usize, Vec<u32>)>]| {
                let named: Vec<&Vec<u32>> = rows.iter().flatten().map(|(_, key)| key).collect();
                let hits = named
                    .iter()
                    .filter(|key| mem.reply_stored(key).is_some())
                    .count();
                (named.len(), hits)
            };
            // "Named": the rows whose key the reply table holds, not the rows it names right.
            let (rows, held) = count(&train_replies);
            println!("[memory] TRAIN reply rows past their openings {rows}, named {held}");
            let (rows, held) = count(&reply_offsets(&dev_windows, &dev_mask));
            println!("[memory] DEV reply rows past their openings {rows}, named {held}");
        }
    }
    let mut opt_main = StackAdamW::new(&model, hs::SHEET_WEIGHT_DECAY, hs::SHEET_CLIP)?;
    let mut opt_decay = StackAdamW::new(&model, 0.0, hs::SHEET_CLIP)?;

    let progress = out.join("progress.jsonl");
    let samples_md = out.join("samples.md");
    let save_root: PathBuf = out.join("models");
    let mut train_seconds = 0.0f64;
    let (mut loss_sum, mut loss_steps) = (0.0f64, 0usize);
    let checkpoint = |model: &StackModel,
                      step: usize,
                      train_loss_mean: Option<f64>,
                      train_seconds: f64|
     -> Result<()> {
        let (dev, _) = hs::evaluate_dev_panel(
            model,
            &dev_windows,
            HamiltonArm::HamiltonActive,
            config.seed,
            step,
            false,
            false,
        )?;
        let dev_loss = panel_loss(model, &dev_windows, config.seed, step, false)?;
        let (nearest, written) =
            nearest_reading(model, &dev_windows, &dev_mask, config.seed, step)?;
        let point = Checkpoint {
            step,
            epoch: (step * config.batch) as f64 / usable.len() as f64,
            train_loss_mean,
            dev_top1_hits: dev.top1_hits,
            dev_top8_hits: dev.top8_hits,
            dev_admitted_hits: dev.admitted_target_hits,
            dev_nearest: nearest,
            dev_writer: written,
            dev_scored_rows: dev.scored_rows,
            dev_loss,
            dev_token_margin: dev.token_margin,
            dev_address_margin: dev.address_margin,
            train_seconds,
            train_context_tokens_per_second: (step * config.batch * SHEET_CONTEXT) as f64
                / train_seconds.max(1e-9),
        };
        append_line(&progress, &serde_json::to_string(&point)?)?;
        println!(
            "[in-space] step {} epoch {:.2} train_loss {} dev top-1 {}/{} nearest {} (openings {}/{}, commonest {}) top-8 {} admitted {} dev_loss {:.5} (token {:.5} address {:.5}) train {:.0}s",
            step,
            point.epoch,
            train_loss_mean.map_or("-".to_owned(), |v| format!("{v:.5}")),
            dev.top1_hits,
            dev.scored_rows,
            nearest.hits,
            nearest.opening_hits,
            nearest.opening_rows,
            nearest.commonest_opening_rows,
            dev.top8_hits,
            dev.admitted_target_hits,
            dev_loss,
            dev.token_margin,
            dev.address_margin,
            train_seconds
        );
        append_line(
            &samples_md,
            &format!(
                "\n## Step {step} (epoch {:.2}): DEV top-1 {}/{}\n",
                point.epoch, dev.top1_hits, dev.scored_rows
            ),
        )?;
        for window in &prompts {
            let off = window.scored_offsets[0];
            let prompt = &window.input_ids[..=off];
            let reply = greedy_reply(
                model,
                config.objective,
                prompt,
                config.reply_tokens,
                config.seed,
            )?;
            // The true reply: the scored targets while their offsets stay contiguous from the
            // prompt's end, so a mask gap or the window's end never splices text together.
            let reference: Vec<u32> = window
                .scored_offsets
                .iter()
                .zip(&window.scored_targets)
                .enumerate()
                .take_while(|&(i, (&o, _))| o == off + i)
                .take(config.reply_tokens)
                .map(|(_, (_, &t))| t)
                .collect();
            let sample = Sample {
                step,
                prompt: tokenizer.decode(prompt),
                model: {
                    // A reply ends at its first end-of-turn marker; what follows is not shown.
                    let text = tokenizer.decode(&reply);
                    match text.find("<|eos|>") {
                        Some(at) => text[..at + "<|eos|>".len()].to_owned(),
                        None => text,
                    }
                },
                reference: tokenizer.decode(&reference),
            };
            append_line(&out.join("samples.jsonl"), &serde_json::to_string(&sample)?)?;
            append_line(
                &samples_md,
                &format!(
                    "- **prompt:** …{}\n  - **model:** {}\n  - **true:** {}",
                    tail(&sample.prompt, 240),
                    tail(&sample.model, 400),
                    tail(&sample.reference, 400)
                ),
            )?;
        }
        let directory = save_root.join(format!("step-{step:06}"));
        model.save(&directory)?;
        if let Some(read) = crate::flock_read::flock_read() {
            read.save(&directory)?;
        }
        if let Some(read) = crate::flock_read_content::content_read() {
            read.save(&directory)?;
        }
        if let Some(reads) = crate::flock_read_hamming::hamming_reads() {
            reads.save(&directory)?;
        }
        if let Some(read) = crate::flock_read_opening::opening_read() {
            read.save(&directory)?;
        }
        if let Some(reg) = crate::flock_read_register::prompt_register() {
            reg.save(&directory)?;
        }
        if let Some(memory) = crate::flock_read_memory::turn_memory() {
            memory.save(&directory)?;
        }
        Ok(())
    };

    checkpoint(&model, 0, None, 0.0)?;
    let mut order = epoch_order(usable.len(), config.seed, 0);
    let mut cursor = 0usize;
    let mut epoch = 0u64;
    // The profile (fbs-cost-profile, claude-157): each profiled step waits for the unprofiled
    // step after it, whose time it is compared with.
    let mut profiled_row: Option<ProfileRow> = None;
    let mut profile_rows: Vec<ProfileRow> = Vec::new();
    for step in 0..config.steps {
        if cursor + config.batch > order.len() {
            epoch += 1;
            order = epoch_order(usable.len(), config.seed, epoch);
            cursor = 0;
        }
        let picked = &order[cursor..cursor + config.batch];
        let windows: Vec<&CausalWindow> = picked.iter().map(|&i| &usable[i]).collect();
        // The batch's opening rows, which the turn memory moves; none when it is off.
        crate::flock_read_memory::set_memory_rows(memory.as_ref().map(|_| {
            picked
                .iter()
                .enumerate()
                .flat_map(|(b, &i)| {
                    train_openings[i]
                        .iter()
                        .map(move |&offset| b * SHEET_CONTEXT + offset)
                })
                .collect()
        }));
        // Its reply rows past their openings, with their keys (claude-193); none when it has no
        // reply table.
        crate::flock_read_memory::set_reply_rows((!train_replies.is_empty()).then(|| {
            picked
                .iter()
                .enumerate()
                .flat_map(|(b, &i)| {
                    train_replies[i]
                        .iter()
                        .map(move |(offset, key)| (b * SHEET_CONTEXT + offset, key.clone()))
                })
                .collect()
        }));
        cursor += config.batch;
        let (ids, scored, targets) = assemble(&windows);
        // The memory's rows are `b · SHEET_CONTEXT + offset`, so every window must be whole.
        if ids.len() != config.batch * SHEET_CONTEXT {
            return Err(invalid("a training batch's windows are not context-long"));
        }
        let profiled = profile_window(step, config.profile);
        let started = Instant::now();
        if profiled == Some(true) {
            stage_clock::start(device);
        }
        // The gate hook carries the real step: at step 0 its backward runs diagnostics with host
        // syncs, and a checkpoint's replies leave it at 0.
        let (value, grads) = hs::with_gate_mode(
            HamiltonArm::HamiltonActive,
            config.seed,
            step + 1,
            false,
            || {
                let loss = batch_loss(
                    &model,
                    config.objective,
                    &ids,
                    &scored,
                    &targets,
                    config.batch,
                )?;
                // The copy to the host waits for the GPU to finish the forward. Only the nearest
                // and cell objectives mark their own stages; the sheet's and Voronoi objectives
                // count in this one.
                let value = f64::from(loss.to_scalar::<f32>()?);
                stage_clock::mark("objective.readback");
                if !value.is_finite() {
                    return Err(invalid(format!("non-finite training loss at step {step}")));
                }
                let grads = loss.backward()?;
                stage_clock::mark("backward");
                Ok((value, grads))
            },
        )?;
        // Linear from `lr` at the first step to `lr_end` at the last.
        let progress = if config.steps == 1 {
            1.0
        } else {
            step as f64 / (config.steps - 1) as f64
        };
        let lr = config.lr + (config.lr_end - config.lr) * progress;
        opt_main.update_filtered(&model, &grads, lr, |name| !name.ends_with("rec.decay"))?;
        stage_clock::mark("optimizer.main");
        if let Some((read, opt)) = flock.as_mut() {
            opt.step(read, &grads, lr)?;
        }
        if let Some((content, opt)) = content.as_mut() {
            opt.step(&content.read, &grads, lr)?;
        }
        if let Some((reads, opt)) = hamming.as_mut() {
            opt.step_vars(&reads.variables(), &grads, lr)?;
        }
        if let Some((read, opt)) = opening.as_mut() {
            opt.step_vars(&read.variables(), &grads, lr)?;
        }
        if let Some((reg, opt)) = register.as_mut() {
            opt.step_vars(&reg.variables(), &grads, lr)?;
        }
        if let Some((mem, opt)) = memory.as_mut() {
            opt.step_vars(&mem.variables(), &grads, lr)?;
        }
        crate::flock_read_memory::set_memory_rows(None);
        crate::flock_read_memory::set_reply_rows(None);
        stage_clock::mark("optimizer.reads");
        opt_decay.update_filtered(&model, &grads, lr * config.decay_mult, |name| {
            name.ends_with("rec.decay")
        })?;
        stage_clock::mark("optimizer.decay");
        // Reading the events waits for the GPU, as the synchronization below would.
        let stages = stage_clock::finish()?;
        device.synchronize()?;
        let step_seconds = started.elapsed().as_secs_f64();
        train_seconds += step_seconds;
        match (profiled, stages) {
            (Some(true), Some(stages)) => {
                let gpu_span_ms = stages.iter().filter_map(|s| s.gpu_ms).sum();
                profiled_row = Some(ProfileRow {
                    step,
                    loss: value,
                    step_ms: step_seconds * 1e3,
                    stages,
                    gpu_span_ms,
                    next_unprofiled_ms: None,
                });
            }
            (Some(false), _) => {
                if let Some(mut row) = profiled_row.take() {
                    row.next_unprofiled_ms = Some(step_seconds * 1e3);
                    append_line(&out.join("profile.jsonl"), &serde_json::to_string(&row)?)?;
                    profile_rows.push(row);
                }
                if Some(step + 1) == config.profile.checked_mul(2).map(|w| w + PROFILE_WARMUP) {
                    let summary = summarize_profile(&profile_rows);
                    fs::write(
                        out.join("profile.json"),
                        serde_json::to_vec_pretty(&summary)?,
                    )?;
                    println!(
                        "[profile] {} steps: stages sum to {:.3} ms against {:.3} ms unprofiled \
                         (ratio {:.3}); the clock adds {:.3}; top three {:?}",
                        summary.profiled_steps,
                        summary.stage_sum_ms,
                        summary.median_unprofiled_ms,
                        summary.ratio,
                        summary.clock_overhead,
                        summary.top3
                    );
                }
            }
            _ => {}
        }
        loss_sum += value;
        loss_steps += 1;
        if (step + 1) % config.eval_every == 0 || step + 1 == config.steps {
            checkpoint(
                &model,
                step + 1,
                Some(loss_sum / loss_steps as f64),
                train_seconds,
            )?;
            (loss_sum, loss_steps) = (0.0, 0);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "cuda")]
    use crate::spiralcore_fbs_sheet::build_fbs_model;

    #[test]
    fn each_epoch_order_is_a_seeded_permutation() {
        let a = epoch_order(1_000, 1, 0);
        let mut sorted = a.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..1_000).collect::<Vec<_>>());
        assert_eq!(a, epoch_order(1_000, 1, 0));
        assert_ne!(a, epoch_order(1_000, 1, 1));
        assert_ne!(a, epoch_order(1_000, 2, 0));
    }

    #[test]
    fn root_separation_reads_the_sixteen_roots() {
        let (nearest, farthest) = root_separation().unwrap();
        println!("address roots: nearest {nearest}, farthest {farthest}");
        assert!(nearest > 0.0 && nearest <= farthest);
    }

    #[test]
    fn a_reply_opens_where_the_token_before_is_not_a_response_token() {
        // prompt, prompt, reply, reply, prompt, reply: the scored targets are the reply tokens
        // 2, 3 and 5, and the first of each reply opens it.
        let mask = [0u8, 0, 1, 1, 0, 1];
        let opens: Vec<bool> = [2, 3, 5].iter().map(|&p| opens_reply(&mask, p)).collect();
        assert_eq!(opens, [true, false, true]);
    }

    #[test]
    fn a_copied_row_repeats_a_pair_the_prompt_holds_and_not_one_the_reply_made() {
        // BOS, a prompt "u q r a", then the reply "k z k z q r". The reply's own "k z" repeats
        // inside the reply, but only "q r" stands in the prompt.
        let tokens = [0u32, 5, 9, 10, 6, 7, 8, 7, 8, 9, 10];
        let mask = [0u8, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1];
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 16, 0).unwrap();
        let copied: Vec<usize> = windows
            .iter()
            .flat_map(|w| {
                w.scored_offsets
                    .iter()
                    .zip(&w.scored_corpus_positions)
                    .zip(&w.scored_targets)
                    .filter(|((&o, &p), &t)| copies_prompt(w, o, p, t, &mask))
                    .map(|((_, &p), _)| p)
            })
            .collect();
        assert_eq!(copied, [10], "only the reply's \"q r\" copies the prompt");
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_cell_objective_pushes_only_the_nearest_rival() {
        let device = Device::new_cuda(0).unwrap();
        // Scores are negative distances: the first rival is the nearest wrong token.
        let rival = Tensor::new(&[[-1.5f32, -1.8, -4.0]], &device).unwrap();
        let loss = |target: f32, cell: bool| -> f32 {
            let target = Tensor::new(&[[target]], &device).unwrap();
            rival_hinge(&rival, &target, cell)
                .unwrap()
                .to_scalar::<f32>()
                .unwrap()
        };
        // The nearest rival's hinge alone: 1 + (−1.5) − (−1.0).
        assert!((loss(-1.0, true) - 0.5).abs() < 1e-6);
        // The nearest objective averages all three hinges instead: (0.5 + 0.2 + 0) / 3.
        assert!((loss(-1.0, false) - 0.7 / 3.0).abs() < 1e-6);
        // A margin nearer than the nearest rival: inside the target's cell, no loss.
        assert_eq!(loss(-0.4, true), 0.0);
    }

    #[test]
    fn a_named_split_loads_only_under_the_sha256s_it_names() {
        let dir = std::env::temp_dir().join(format!("claude-152-split-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let mut header = vec![0u8; 64];
        header[..4].copy_from_slice(b"UORT");
        header[8..16].copy_from_slice(&3u64.to_le_bytes());
        let tokens: Vec<u8> = header.into_iter().chain([0u8, 0, 5, 0, 6, 0]).collect();
        let mask = vec![0u8, 0, 1];
        fs::write(dir.join("tokens.u16"), &tokens).unwrap();
        fs::write(dir.join("response_mask.u8"), &mask).unwrap();
        let split = |tokens_sha256: String| TrainSplit {
            dir: dir.clone(),
            tokens_sha256,
            mask_sha256: sha256_bytes(&mask),
        };
        let (ids, m) = load_named_split(&split(sha256_bytes(&tokens))).unwrap();
        assert_eq!((ids, m), (vec![0, 5, 6], mask.clone()));
        assert!(load_named_split(&split("0".repeat(64))).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_recall_query_names_the_value_after_its_keys_only_earlier_copy() {
        for index in 0..64 {
            let (w, buckets) = recall_window(7, index, SHEET_CONTEXT, 4).unwrap();
            assert_eq!(w.input_ids.len(), SHEET_CONTEXT);
            assert_eq!(w.input_ids[0], hs::SHEET_BOS_ID);
            assert_eq!(w.scored_offsets.len(), 16, "window {index}");
            let rows = w.scored_offsets.iter().zip(&w.scored_targets).zip(&buckets);
            for ((&q, &value), &bucket) in rows {
                let key = w.input_ids[q];
                assert!(RECALL_KEYS.contains(&key) && RECALL_VALUES.contains(&value));
                let earlier: Vec<usize> = (0..q).filter(|&i| w.input_ids[i] == key).collect();
                assert_eq!(earlier.len(), 1, "window {index}, query {q}");
                let p = earlier[0];
                assert_eq!((w.input_ids[p + 1], w.input_ids[q + 1]), (value, value));
                let (low, high) = RECALL_BUCKETS[bucket];
                assert!((low..=high).contains(&(q - p)), "d = {} in {bucket}", q - p);
            }
            // No two pairs share a value, so a value names one key only.
            let mut values = w.scored_targets.clone();
            values.sort_unstable();
            values.dedup();
            assert_eq!(values.len(), w.scored_targets.len(), "window {index}");
            let pair_positions = 4 * w.scored_offsets.len();
            let filler = w
                .input_ids
                .iter()
                .filter(|t| RECALL_FILLER.contains(t))
                .count();
            assert_eq!(filler + pair_positions + 1, SHEET_CONTEXT);
        }
    }

    #[test]
    fn a_window_too_small_for_its_pairs_is_refused_and_far_distances_stay_uniform() {
        // 12 pairs per bucket need 192 positions of a 192-token window that also holds BOS.
        assert!(recall_window(1, 0, SHEET_CONTEXT, 12).is_err());
        // The longest bucket is placed first, so even a crowded window (8 pairs per bucket, 128 of
        // 192 positions) fills it, with distances uniform over 97..=180. Placed last, it underfills.
        let mut far = Vec::new();
        for index in 0..256 {
            let (w, buckets) = recall_window(3, index, SHEET_CONTEXT, 8).unwrap();
            for (&q, &bucket) in w.scored_offsets.iter().zip(&buckets) {
                if bucket == 3 {
                    let key = w.input_ids[q];
                    let p = (0..q).find(|&i| w.input_ids[i] == key).unwrap();
                    far.push((q - p) as f64);
                }
            }
        }
        let mean = far.iter().sum::<f64>() / far.len() as f64;
        assert!((mean - 138.5).abs() < 6.0, "mean far distance {mean}");
    }

    #[test]
    fn recall_windows_are_seeded_and_fresh_per_index() {
        let ids = |seed, index| {
            recall_window(seed, index, SHEET_CONTEXT, 4)
                .unwrap()
                .0
                .input_ids
        };
        assert_eq!(ids(1, 3), ids(1, 3));
        assert_ne!(ids(1, 3), ids(1, 4));
        assert_ne!(ids(1, 3), ids(2, 3));
    }

    #[test]
    fn openings_carry_the_prompt_their_window_shows_back_to_the_last_reply() {
        // BOS, a prompt (10 11 12), a reply (20 21), a prompt (13 14), a reply (22).
        let tokens = [0u32, 10, 11, 12, 20, 21, 13, 14, 22];
        let mask = [0u8, 0, 0, 0, 1, 1, 0, 0, 1];
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 16, 0).unwrap();
        assert_eq!(
            openings(&windows, &mask),
            [(vec![10, 11, 12], 20), (vec![13, 14], 22)]
        );
    }

    #[test]
    fn the_anchor_check_counts_openings_whose_turn_slots_are_the_first_8_key() {
        // BOS, a prompt (10 11 12), a reply (20 21) closed by EOS, a prompt (13 14), a reply (22)
        // with no EOS, then a prompt (15) and a reply (23). The turn anchor starts after BOS and
        // after EOS, so it keeps the probe's key at the first two openings; at the third it still
        // runs from the EOS, through the reply 22.
        let tokens = [0u32, 10, 11, 12, 20, 21, 1, 13, 14, 22, 15, 23];
        let mask = [0u8, 0, 0, 0, 1, 1, 1, 0, 0, 1, 0, 1];
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 16, 0).unwrap();
        assert_eq!(
            openings(&windows, &mask),
            [(vec![10, 11, 12], 20), (vec![13, 14], 22), (vec![15], 23)]
        );
        let check = anchor_check("dev", &windows, &mask);
        assert_eq!((check.openings, check.agree), (3, 2));
        assert_eq!(
            serde_json::to_string(&check).unwrap(),
            r#"{"anchor_check":"dev","openings":3,"agree":2}"#
        );
    }

    fn config(opening_read: usize, opening_anchor: Option<OpeningAnchor>) -> LongConfig {
        LongConfig {
            objective: PointObjective::Sheet,
            steps: 1,
            batch: 1,
            eval_every: 1,
            seed: 1,
            lr: 0.002,
            lr_end: 0.002,
            decay_mult: 20.0,
            retain_rates: false,
            fold_scores: false,
            reflect_floor: false,
            flock_read: 0,
            content_read: 0,
            hamming_read: 0,
            opening_read,
            opening_anchor,
            prompt_register: 0,
            register_picks: None,
            register_select: None,
            width: hs::SHEET_WIDTH,
            type_spin: false,
            fix_input_table: false,
            train_split: None,
            samples: 6,
            reply_tokens: 48,
            profile: 0,
            turn_memory: 0,
            turn_memory_empty: false,
            turn_memory_reply: false,
        }
    }

    use crate::flock_read_opening::OpeningAnchor;

    #[test]
    fn a_config_without_the_opening_read_writes_neither_of_its_keys() {
        let json = String::from_utf8(serde_json::to_vec_pretty(&config(0, None)).unwrap()).unwrap();
        assert!(!json.contains("opening_read"), "{json}");
        assert!(!json.contains("opening_anchor"), "{json}");
        // The keys around them stand as they did before the read.
        assert!(
            json.contains("\"hamming_read\": 0,\n  \"width\": 256,"),
            "{json}"
        );
    }

    #[test]
    fn a_config_with_the_opening_read_writes_both_of_its_keys() {
        for (anchor, name) in [(OpeningAnchor::Turn, "turn"), (OpeningAnchor::Last, "last")] {
            let json =
                String::from_utf8(serde_json::to_vec_pretty(&config(8, Some(anchor))).unwrap())
                    .unwrap();
            let keys = format!(
                "\"hamming_read\": 0,\n  \"opening_read\": 8,\n  \"opening_anchor\": \"{name}\",\n  \"width\": 256,"
            );
            assert!(json.contains(&keys), "{json}");
        }
    }

    #[test]
    fn a_checkpoint_with_an_opening_read_sets_it_on_the_thread_until_the_guard_drops() {
        // Save and load only: the vendored candle computes nothing on the CPU, but files move.
        let device = Device::Cpu;
        let directory =
            std::env::temp_dir().join(format!("claude-154-load-reads-{}", std::process::id()));
        crate::flock_read_opening::OpeningRead::new(5, 8, OpeningAnchor::Last, &device)
            .unwrap()
            .save(&directory)
            .unwrap();
        {
            let _guard = ArmGuard;
            load_reads(&directory, &device).unwrap();
            let read = crate::flock_read_opening::opening_read().expect("the opening read is set");
            assert_eq!((read.k(), read.anchor()), (5, OpeningAnchor::Last));
        }
        assert!(crate::flock_read_opening::opening_read().is_none());
        fs::remove_dir_all(&directory).unwrap();
    }

    use crate::flock_read_register::{PromptRegister, RegisterSelect};

    #[test]
    fn a_config_without_the_register_writes_none_of_its_keys() {
        for json in [config(0, None), config(8, Some(OpeningAnchor::Turn))]
            .map(|c| String::from_utf8(serde_json::to_vec_pretty(&c).unwrap()).unwrap())
        {
            for key in ["prompt_register", "register_picks", "register_select"] {
                assert!(!json.contains(key), "{json}");
            }
        }
    }

    #[test]
    fn a_config_with_the_register_writes_its_three_keys_after_the_opening_reads() {
        for (q, select, name) in [
            (8, RegisterSelect::Learned, "learned"),
            (1, RegisterSelect::All, "all"),
        ] {
            let mut with = config(8, Some(OpeningAnchor::Turn));
            (with.prompt_register, with.register_picks) = (q, Some(8));
            with.register_select = Some(select);
            let json = String::from_utf8(serde_json::to_vec_pretty(&with).unwrap()).unwrap();
            let keys = format!(
                "\"opening_anchor\": \"turn\",\n  \"prompt_register\": {q},\n  \"register_picks\": 8,\n  \"register_select\": \"{name}\",\n  \"width\": 256,"
            );
            assert!(json.contains(&keys), "{json}");
        }
    }

    #[test]
    fn a_checkpoint_with_a_register_sets_it_on_the_thread_until_the_guard_drops() {
        // Save and load only: the vendored candle computes nothing on the CPU, but files move.
        let device = Device::Cpu;
        let directory =
            std::env::temp_dir().join(format!("claude-155-load-reads-{}", std::process::id()));
        PromptRegister::new(3, 5, RegisterSelect::Learned, 8, &device)
            .unwrap()
            .save(&directory)
            .unwrap();
        {
            let _guard = ArmGuard;
            load_reads(&directory, &device).unwrap();
            let reg = crate::flock_read_register::prompt_register().expect("the register is set");
            assert_eq!(
                (reg.q(), reg.picks(), reg.select()),
                (3, 5, RegisterSelect::Learned)
            );
            crate::flock_read_register::set_register_slots(Some(
                Tensor::zeros(1, candle_core::DType::F32, &device).unwrap(),
            ));
        }
        assert!(crate::flock_read_register::prompt_register().is_none());
        assert!(crate::flock_read_register::take_register_slots().is_none());
        fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn the_bound_names_each_key_by_its_commonest_train_opening() {
        let train = [
            (vec![10, 11, 12], 20),
            (vec![13, 14], 22),
            (vec![10, 30], 20),
            (vec![10, 31], 23),
        ];
        let dev = [(vec![10, 99], 20), (vec![13, 98], 22), (vec![50], 20)];
        let decode = |ids: &[u32]| format!("{ids:?}");
        let first = bound(PromptKey::First(1), &train, &dev, &decode);
        // Key 10 opens with 20 twice and 23 once; key 13 with 22; key 50 is unseen and takes
        // TRAIN's commonest opening, 20.
        assert_eq!(
            (first.dev_hits, first.dev_unseen, first.train_keys),
            (3, 1, 2)
        );
        assert_eq!(first.top[0].key, "[10]");
        assert_eq!((first.top[0].train_rows, first.top[0].opening_rows), (3, 2));
        let none = bound(PromptKey::None, &train, &dev, &decode);
        assert_eq!((none.dev_hits, none.dev_unseen), (2, 0));
    }

    #[test]
    fn reply_rows_carry_the_turn_the_reply_so_far_and_their_kinds() {
        // BOS, a prompt (10 11 12), a reply (11 12 30), a prompt (13), a reply (31).
        let tokens = [0u32, 10, 11, 12, 11, 12, 30, 13, 31];
        let mask = [0u8, 0, 0, 0, 1, 1, 1, 0, 1];
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 16, 0).unwrap();
        let rows = reply_rows(&windows, &mask);
        let row = |offset, prompt: &[u32], reply: &[u32], target, opening, copy| ReplyRow {
            window: 0,
            offset,
            prompt: prompt.to_vec(),
            reply: reply.to_vec(),
            target,
            opening,
            copy,
        };
        assert_eq!(
            rows,
            [
                row(3, &[10, 11, 12], &[], 11, true, false),
                // The reply's 11 then 12 stand side by side in the prompt: a copy.
                row(4, &[10, 11, 12], &[11], 12, false, true),
                row(5, &[10, 11, 12], &[11, 12], 30, false, false),
                row(7, &[13], &[], 31, true, false),
            ]
        );
        // claude-161's opening key is the turn at position 0; later rows key on their position.
        assert_eq!(
            ReplyKey::TurnPosition(0).of(&rows[0]),
            [10, 11, 12, u32::MAX, 0, u32::MAX]
        );
        assert_eq!(
            ReplyKey::TurnPosition(1).of(&rows[2]),
            [10, 11, 12, u32::MAX, 2, u32::MAX, 12]
        );
        assert_eq!(ReplyKey::Reply(4).of(&rows[2]), [11, 12]);
    }

    /// claude-202: a document starts at BOS, which belongs to neither turn; prompts and replies
    /// are maximal runs of one mask value inside a document.
    #[test]
    fn census_turns_split_at_bos_and_at_each_mask_change() {
        let tokens = [0, 5, 6, 7, 8, 9, 0, 5, 7, 3];
        let mask = [0, 0, 0, 1, 1, 0, 0, 0, 1, 1];
        let turns = split_turns(&tokens, &mask, 0);
        assert_eq!(turns.documents, 2);
        assert_eq!(turns.prompts, vec![vec![5, 6], vec![9], vec![5]]);
        assert_eq!(turns.replies, vec![vec![7, 8], vec![7, 3]]);
    }

    /// claude-202: n-gram windows never cross a run, and exact matches compare whole runs.
    #[test]
    fn census_ngrams_stay_inside_their_run_and_exact_matches_are_whole_runs() {
        let train = vec![vec![1, 2], vec![3, 4, 5]];
        let set = ngram_set(&train, 2);
        // TRAIN's 2-grams: [1, 2], [3, 4], [4, 5]; [2, 3] crosses runs and is absent.
        assert_eq!(set.len(), 3);
        let (share, windows) = ngram_share(&[vec![1, 2, 3]], &set, 2);
        assert_eq!((share, windows), (0.5, 2));
        let exact: std::collections::HashSet<u128> = train.iter().map(|r| census_hash(r)).collect();
        assert_eq!(exact_share(&[vec![1, 2], vec![1, 2, 3], vec![3, 4, 5], vec![9]], &exact), 0.5);
        // The length is part of the hash: [3, 4] and [3, 4, 5] differ.
        assert_ne!(census_hash(&[3, 4]), census_hash(&[3, 4, 5]));
    }

    /// claude-202: a value mask keeps the frequent tokens, masks the rest and every digit, and
    /// collapses each run of masks to one.
    #[test]
    fn census_value_masks_keep_frequent_tokens_and_collapse_masks() {
        let tokens = [0, 1, 1, 1, 2, 2, 4, 5, 3, 3, 3, 3];
        let keep = top_tokens(&tokens, 2, 0);
        // 3 is the commonest (4), then 1 (3); 2 (2) is third.
        assert_eq!(keep, [1u32, 3].into_iter().collect());
        // BOS is never counted, even when it is the commonest token; ties go to the smaller id.
        let frequent_bos = [0, 0, 0, 0, 0, 7, 7, 6, 6, 9];
        assert_eq!(top_tokens(&frequent_bos, 2, 0), [6u32, 7].into_iter().collect());
        let digits: std::collections::HashSet<u32> = [3u32].into_iter().collect();
        assert_eq!(
            value_mask(&[1, 4, 5, 3, 1, 2], &keep, &digits),
            vec![1, CENSUS_MASK, 1, CENSUS_MASK]
        );
        let tokenizer = r#"{"model": {"vocab": {"a": 4, "7": 5, "Ġ9": 6, "12": 7, "Ġx": 8}}}"#;
        assert_eq!(
            digit_tokens(tokenizer).unwrap(),
            [5u32, 6].into_iter().collect()
        );
    }

    /// claude-204: an unpinned `--panel` split loads only with a UORT header whose count matches
    /// its payload and mask, a first BOS, and ids inside the vocabulary.
    #[test]
    fn an_unpinned_panel_split_is_checked_before_it_is_counted() {
        let dir = std::env::temp_dir().join(format!("claude-204-panel-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let write = |tokens: &[u16], mask: &[u8], magic: &[u8; 4], count: u64| {
            let mut bytes = vec![0u8; hs::UORT_HEADER_BYTES];
            bytes[..4].copy_from_slice(magic);
            bytes[8..16].copy_from_slice(&count.to_le_bytes());
            bytes.extend(tokens.iter().flat_map(|t| t.to_le_bytes()));
            std::fs::write(dir.join("tokens.u16"), bytes).unwrap();
            std::fs::write(dir.join("response_mask.u8"), mask).unwrap();
        };
        write(&[0, 5, 6, 7], &[0, 0, 1, 1], b"UORT", 4);
        let (tokens, mask, sha) = load_unpinned_uort(&dir).unwrap();
        assert_eq!((tokens, mask, sha.len()), (vec![0, 5, 6, 7], vec![0, 0, 1, 1], 64));
        write(&[0, 5, 6, 7], &[0, 0, 1, 1], b"XXXX", 4);
        assert!(load_unpinned_uort(&dir).is_err(), "no UORT header");
        write(&[0, 5, 6, 7], &[0, 0, 1, 1], b"UORT", 3);
        assert!(load_unpinned_uort(&dir).is_err(), "a count that does not match");
        write(&[5, 0, 6, 7], &[0, 0, 1, 1], b"UORT", 4);
        assert!(load_unpinned_uort(&dir).is_err(), "no first BOS");
        write(&[0, 5, 9_000, 7], &[0, 0, 1, 1], b"UORT", 4);
        assert!(load_unpinned_uort(&dir).is_err(), "an id past the vocabulary");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// claude-202: step 2's rows are the later rows the turn-8 + position + reply-last-1 table
    /// does not keep at share 0.5, and the openings no first-8 key names.
    #[test]
    fn census_decision_rows_are_the_unnamed_later_rows_and_unkeyed_openings() {
        let row = |prompt: &[u32], reply: &[u32], target, opening| ReplyRow {
            window: 0,
            offset: 0,
            prompt: prompt.to_vec(),
            reply: reply.to_vec(),
            target,
            opening,
            copy: false,
        };
        let train = [
            // An opening after the prompt [1, 2], and two later rows after reply [10].
            row(&[1, 2], &[], 10, true),
            row(&[1, 2], &[10], 11, false),
            row(&[1, 2], &[10], 11, false),
            // After reply [20]: 21 once and 22 once, a 1/2 share, kept at 0.5.
            row(&[1, 2], &[20], 21, false),
            row(&[1, 2], &[20], 22, false),
            // After reply [30]: 31 once, 32 once, 33 once, a 1/3 share, not kept at 0.5.
            row(&[1, 2], &[30], 31, false),
            row(&[1, 2], &[30], 32, false),
            row(&[1, 2], &[30], 33, false),
        ];
        let reply = reply_table(ReplyKey::TurnPosition(1), &train);
        let opening = reply_table(ReplyKey::TurnPosition(0), &train);
        let dev = [
            row(&[1, 2], &[], 10, true),
            row(&[5, 6], &[], 10, true),
            row(&[1, 2], &[10], 11, false),
            row(&[1, 2], &[20], 21, false),
            row(&[1, 2], &[30], 31, false),
            row(&[1, 2], &[40], 41, false),
        ];
        let (later, openings) = census_decision_rows(&reply, &opening, &dev);
        // Rows 4 ([30], share 1/3) and 5 ([40], unseen) are unnamed; row 1's key is unseen.
        assert_eq!((later, openings), (vec![4, 5], vec![1]));
    }

    #[test]
    fn the_reply_bound_keeps_each_keys_commonest_target_at_its_share() {
        let row = |reply: &[u32], target, opening, copy| ReplyRow {
            window: 0,
            offset: 0,
            prompt: vec![1],
            reply: reply.to_vec(),
            target,
            opening,
            copy,
        };
        let train = [
            // After 5: 7 twice and 8 once, a 2/3 share.
            row(&[5], 7, false, false),
            row(&[5], 7, false, false),
            row(&[5], 8, false, false),
            // After 6: always 9.
            row(&[6], 9, false, true),
            // After 3: 4 and 2 once each; the tie goes to the smaller id.
            row(&[3], 4, false, false),
            row(&[3], 2, false, false),
            // Openings: 40 twice, 41 once.
            row(&[], 40, true, false),
            row(&[], 40, true, false),
            row(&[], 41, true, false),
        ];
        let dev = [
            row(&[5], 7, false, false),
            row(&[5], 8, false, false),
            row(&[6], 9, false, true),
            row(&[3], 2, false, false),
            row(&[4], 1, false, false),
            row(&[], 41, true, false),
        ];
        let counts = reply_bound_rows(ReplyKey::Reply(1), &train, &dev, &[0.0, 0.8]);
        let all = &counts[0];
        assert_eq!(all.train_keys, 4);
        // Named: after 5 (twice), 6, 3 and the opening; after 4 is unseen. Right: 7, 9, 2.
        assert_eq!(
            (all.all, all.copies, all.openings),
            (
                KindCount {
                    rows: 6,
                    named: 5,
                    hits: 3
                },
                KindCount {
                    rows: 1,
                    named: 1,
                    hits: 1
                },
                KindCount {
                    rows: 1,
                    named: 1,
                    hits: 0
                },
            )
        );
        assert_eq!(
            all.rest,
            KindCount {
                rows: 4,
                named: 3,
                hits: 2
            }
        );
        // At 0.8 only the key after 6 (share 1) is kept; 2/3 and 1/2 shares are dropped.
        let strict = &counts[1];
        assert_eq!(strict.train_keys, 1);
        assert_eq!(
            strict.all,
            KindCount {
                rows: 6,
                named: 1,
                hits: 1
            }
        );
        // An opening whose key is not kept takes TRAIN's commonest opening, 40: wrong for 41.
        assert_eq!(
            (
                all.opening_hits_with_fallback,
                strict.opening_hits_with_fallback
            ),
            (0, 0)
        );
    }

    #[test]
    fn the_reply_bound_falls_back_to_the_commonest_opening_only_for_openings() {
        let row = |prompt: &[u32], target, opening| ReplyRow {
            window: 0,
            offset: 0,
            prompt: prompt.to_vec(),
            reply: Vec::new(),
            target,
            opening,
            copy: false,
        };
        let train = [
            row(&[1], 40, true),
            row(&[2], 40, true),
            row(&[3], 41, true),
        ];
        // Turn 9 is unseen: the fallback names 40, which is right; turn 3 names 41, right.
        let dev = [
            row(&[9], 40, true),
            row(&[3], 41, true),
            row(&[9], 40, false),
        ];
        let counts = reply_bound_rows(ReplyKey::TurnPosition(0), &train, &dev, &[0.0]);
        assert_eq!(
            counts[0].openings,
            KindCount {
                rows: 2,
                named: 1,
                hits: 1
            }
        );
        assert_eq!(counts[0].opening_hits_with_fallback, 2);
        assert_eq!(
            counts[0].rest,
            KindCount {
                rows: 1,
                named: 0,
                hits: 0
            }
        );
    }

    #[test]
    fn a_row_that_opens_and_copies_counts_in_both_kinds_and_not_the_rest() {
        let row = ReplyRow {
            window: 0,
            offset: 0,
            prompt: vec![1],
            reply: Vec::new(),
            target: 40,
            opening: true,
            copy: true,
        };
        let counts = reply_bound_rows(
            ReplyKey::None,
            std::slice::from_ref(&row),
            std::slice::from_ref(&row),
            &[0.0],
        );
        let one = KindCount {
            rows: 1,
            named: 1,
            hits: 1,
        };
        assert_eq!((counts[0].openings, counts[0].copies), (one, one));
        assert_eq!(counts[0].rest, KindCount::default());
        assert_eq!(counts[0].all, one);
    }

    #[test]
    fn reply_offsets_name_the_rows_past_each_opening_with_the_probes_key() {
        // A prompt and a 5-token reply in windows of 4, so the reply runs into a second window.
        let tokens = [0u32, 10, 11, 20, 21, 22, 23, 24];
        let mask = [0u8, 0, 0, 1, 1, 1, 1, 1];
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 4, 0).unwrap();
        assert!(windows.len() >= 2, "the reply spans two windows");
        let offsets = reply_offsets(&windows, &mask);
        let want: Vec<Vec<(usize, Vec<u32>)>> = {
            let mut want = vec![Vec::new(); windows.len()];
            for row in reply_rows(&windows, &mask) {
                if !row.opening {
                    want[row.window].push((row.offset, reply_key(&row)));
                }
            }
            want
        };
        assert_eq!(offsets, want);
        // The first window sees the turn: the rows after the opening key on [10, 11], their
        // position and the last reply id.
        assert_eq!(offsets[0][0].1, [10, 11, u32::MAX, 1, u32::MAX, 20]);
        // A later window cannot see the turn: its key holds the window's own reply tokens only,
        // the probe's scope (claude-192's REPORT, "the keys see the window").
        let later = &offsets[windows.len() - 1];
        assert!(later.iter().all(|(_, key)| key[0] == u32::MAX));
    }

    #[test]
    fn reply_rows_land_on_the_rows_assemble_scores() {
        // Two documents, so two windows: the DEV reading names a reply row at its window's index
        // times the context plus its offset, which must be the row `assemble` scores.
        let tokens = [0u32, 10, 11, 20, 21, 22, 0, 12, 13, 23, 24];
        let mask = [0u8, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1];
        let (windows, _) =
            hs::build_document_windows(&tokens, &mask, SHEET_CONTEXT, hs::SHEET_BOS_ID).unwrap();
        assert_eq!(windows.len(), 2);
        let refs: Vec<&CausalWindow> = windows.iter().collect();
        let (_, scored, targets) = assemble(&refs);
        let rows = reply_rows(&windows, &mask);
        let named: Vec<u32> = rows
            .iter()
            .map(|row| (row.window * SHEET_CONTEXT + row.offset) as u32)
            .collect();
        assert_eq!(named, scored);
        assert_eq!(
            rows.iter().map(|row| row.target).collect::<Vec<_>>(),
            targets
        );
    }

    #[test]
    fn the_greedy_key_equals_the_corpus_key_at_the_same_row() {
        // BOS, a turn of 3, then the reply 20 21 22.
        let tokens = [0u32, 10, 11, 12, 20, 21, 22];
        let mask = [0u8, 0, 0, 0, 1, 1, 1];
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 16, 0).unwrap();
        let rows: Vec<ReplyRow> = reply_rows(&windows, &mask)
            .into_iter()
            .filter(|row| !row.opening)
            .collect();
        assert_eq!(rows.len(), 2);
        for row in &rows {
            // The greedy step that predicts this row's target sees the prompt and the reply so
            // far; the prompt's last token is window index 3.
            let window = &tokens[..=row.offset];
            assert_eq!(
                greedy_reply_key(window, 3, &row.reply),
                Some(reply_key(row)),
                "offset {}",
                row.offset
            );
        }
        // The opening step has no reply yet, so no reply key.
        assert_eq!(greedy_reply_key(&tokens[..4], 3, &[]), None);
    }

    #[test]
    fn the_reply_table_names_what_the_probe_counts() {
        // BOS, a turn, a reply; another turn with the same start, a reply that differs at its
        // second token; a third.
        let tokens = [
            0u32, 10, 11, 20, 21, 22, 13, 10, 11, 20, 25, 22, 13, 10, 11, 20, 21, 22,
        ];
        let mask = [0u8, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1];
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 32, 0).unwrap();
        let rows = reply_rows(&windows, &mask);
        let probe = &reply_bound_rows(ReplyKey::TurnPosition(1), &rows, &rows, &[0.5])[0];
        let later: Vec<(Vec<u32>, u32)> = rows
            .iter()
            .filter(|row| !row.opening)
            .map(|row| (reply_key(row), row.target))
            .collect();
        let table = crate::flock_read_memory::build_reply_table(&later, 0.5);
        let named = later
            .iter()
            .filter(|(key, _)| table.contains_key(key))
            .count();
        let hits = later
            .iter()
            .filter(|(key, target)| table.get(key) == Some(target))
            .count();
        assert_eq!(
            (named, hits),
            (
                probe.all.named - probe.openings.named,
                probe.all.hits - probe.openings.hits
            )
        );
        assert!(named > 0 && hits < named);
    }

    #[test]
    fn a_config_writes_the_reply_table_key_only_when_on_and_refuses_it_without_the_memory() {
        let mut config = config(0, None);
        let json = String::from_utf8(serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        assert!(!json.contains("turn_memory_reply"), "{json}");
        config.turn_memory_reply = true;
        let json = String::from_utf8(serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        assert!(json.contains("\"turn_memory_reply\": true"), "{json}");
        let missing = std::env::temp_dir().join(format!("claude-193-none-{}", std::process::id()));
        let refused = |config: &LongConfig| {
            start_memory(&missing, config, &[], &Device::Cpu)
                .err()
                .map(|e| e.to_string())
                .unwrap_or_default()
        };
        assert!(refused(&config).contains("--turn-memory-reply needs"));
        config.turn_memory = 6;
        assert!(refused(&config).contains("needs --turn-memory 8"));
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn best_positions_skip_what_is_not_allowed_and_rank_the_rest() {
        let device = Device::new_cuda(0).unwrap();
        let scores = Tensor::new(&[[0.9f32, 0.1, 0.8, 0.3, 0.5]], &device).unwrap();
        let allowed = Tensor::new(&[[0u8, 1, 1, 1, 1]], &device).unwrap();
        let at = best_positions(&scores, &allowed, 3).unwrap();
        assert_eq!(at.to_vec2::<u32>().unwrap(), vec![vec![2, 4, 3]]);
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn whole_table_scores_do_not_depend_on_the_chunk() {
        let device = Device::new_cuda(0).unwrap();
        let table = Tensor::new(
            &[[0.0f32, 1.0], [1.0, 0.0], [2.0, 2.0], [-1.0, 0.5]],
            &device,
        )
        .unwrap();
        let query = Tensor::new(&[[0.0f32, 0.0], [1.0, 1.0], [2.0, -1.0]], &device).unwrap();
        let whole = nearest_scores(&query, &table)
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        // Row by row, as a chunk of one row gives.
        for (r, row) in whole.iter().enumerate() {
            let one = nearest_scores(&query.narrow(0, r, 1).unwrap(), &table)
                .unwrap()
                .to_vec2::<f32>()
                .unwrap();
            assert_eq!(&one[0], row);
        }
        // Negative mean L1: query [0, 0] against [0, 1] is -(0 + 1) / 2.
        assert_eq!(whole[0][0], -0.5);
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_voronoi_loss_at_margin_one_is_the_writers_margin() {
        let device = Device::new_cuda(0).unwrap();
        let model = build_fbs_model(FbsArm::OneAtomActive, 1, &device).unwrap();
        let ids: Vec<u32> = (0..SHEET_CONTEXT as u32).map(|i| (i * 37) % 4096).collect();
        let hidden = model.hidden(&ids, 1, SHEET_CONTEXT).unwrap();
        let idx = Tensor::new(&[10u32, 50, 100, 191], &device).unwrap();
        let query = hidden.index_select(&idx, 0).unwrap();
        let targets = [11u32, 2000, 7, 300];
        let writer = model
            .native_head_loss(&query, &targets, hs::SHEET_RIVALS)
            .unwrap()
            .to_scalar::<f32>()
            .unwrap();
        let ours = voronoi_loss(&model, &query, &targets, 1.0)
            .unwrap()
            .to_scalar::<f32>()
            .unwrap();
        assert!((writer - ours).abs() < 1e-6, "{writer} != {ours}");
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_stage_clock_times_a_step_without_changing_its_loss() {
        let device = Device::new_cuda(0).unwrap();
        let model = build_fbs_model(FbsArm::OneAtomActive, 1, &device).unwrap();
        let ids: Vec<u32> = (0..SHEET_CONTEXT as u32).map(|i| (i * 37) % 4096).collect();
        let (scored, targets) = ([10u32, 50, 100, 191], [11u32, 2000, 7, 300]);
        let loss = || batch_loss(&model, PointObjective::Cell, &ids, &scored, &targets, 1).unwrap();
        let off = loss().to_scalar::<f32>().unwrap();
        stage_clock::start(&device);
        let on = loss();
        on.backward().unwrap();
        stage_clock::mark("backward");
        let stages = stage_clock::finish().unwrap().expect("the clock was on");
        // Events mark the stream and change no kernel: the loss is the same, bit for bit.
        assert_eq!(off.to_bits(), on.to_scalar::<f32>().unwrap().to_bits());
        let names: Vec<&str> = stages.iter().map(|s| s.name.as_str()).collect();
        for want in [
            "embed",
            "layer0.recurrence",
            "layer0.mlp",
            "final_norm",
            "points.select",
            "objective.table_search",
            "objective.hinge",
            "backward",
            "end",
        ] {
            assert!(names.contains(&want), "{names:?} lacks {want}");
        }
        assert!(
            stages
                .iter()
                .all(|s| s.gpu_ms.is_some_and(|ms| ms >= 0.0) && s.host_ms >= 0.0),
            "{stages:?}"
        );
        // Off again: a mark does nothing, and there is nothing to finish.
        stage_clock::mark("after");
        assert!(stage_clock::finish().unwrap().is_none());
    }

    #[test]
    fn the_profile_window_alternates_after_the_warm_up() {
        assert_eq!(profile_window(PROFILE_WARMUP, 0), None);
        assert_eq!(profile_window(PROFILE_WARMUP - 1, 3), None);
        let window: Vec<Option<bool>> = (PROFILE_WARMUP..PROFILE_WARMUP + 7)
            .map(|step| profile_window(step, 3))
            .collect();
        assert_eq!(
            window,
            [
                Some(true),
                Some(false),
                Some(true),
                Some(false),
                Some(true),
                Some(false),
                None
            ]
        );
    }

    fn profile_row(scale: f64, unprofiled_ms: f64) -> ProfileRow {
        let stage = |name: &str, gpu_ms: f64| StageTime {
            name: name.to_string(),
            gpu_ms: Some(gpu_ms * scale),
            host_ms: gpu_ms * scale / 2.0,
        };
        let stages = vec![
            stage("embed", 1.0),
            stage("layer0.recurrence", 6.0),
            stage("points.select", 0.5),
            stage("objective.table_search", 3.0),
            stage("backward", 8.0),
            stage("optimizer.main", 1.5),
            stage("end", 0.0),
        ];
        ProfileRow {
            step: PROFILE_WARMUP,
            loss: 1.0,
            step_ms: unprofiled_ms,
            gpu_span_ms: stages.iter().filter_map(|s| s.gpu_ms).sum(),
            stages,
            next_unprofiled_ms: Some(unprofiled_ms),
        }
    }

    #[test]
    fn the_profile_summary_takes_medians_sums_groups_and_reads_the_bar() {
        // Three steps: each stage's median is the middle step's, at scale 1.
        let summary = summarize_profile(&[
            profile_row(0.9, 21.0),
            profile_row(1.0, 20.0),
            profile_row(1.3, 19.0),
        ]);
        assert_eq!(summary.profiled_steps, 3);
        let gpu: Vec<f64> = summary.stages.iter().map(|s| s.gpu_ms).collect();
        assert_eq!(gpu, [1.0, 6.0, 0.5, 3.0, 8.0, 1.5, 0.0]);
        assert_eq!(summary.stages[1].host_ms, 3.0);
        let group = |g: &str| summary.groups.iter().find(|(n, _)| n == g).unwrap().1;
        assert_eq!(
            [
                group("forward"),
                group("objective"),
                group("backward"),
                group("optimizer"),
                group("end")
            ],
            [7.0, 3.5, 8.0, 1.5, 0.0]
        );
        assert_eq!(
            (summary.stage_sum_ms, summary.median_unprofiled_ms),
            (20.0, 20.0)
        );
        assert!(summary.within_10pct && summary.ratio == 1.0);
        assert_eq!(
            summary.top3,
            ["backward", "layer0.recurrence", "objective.table_search"]
        );
        assert_eq!(
            summary.top3_forward,
            ["layer0.recurrence", "objective.table_search", "embed"]
        );
        assert_eq!(
            (summary.median_profiled_ms, summary.clock_overhead),
            (20.0, 0.0)
        );
        // A clock that slows the profiled steps shows in its overhead, which the sum cannot see.
        let mut slowed = profile_row(1.0, 20.0);
        slowed.step_ms = 22.0;
        let slowed = summarize_profile(&[slowed]);
        assert!(slowed.within_10pct && (slowed.clock_overhead - 0.1).abs() < 1e-12);
        // The stages summing to far more than an unprofiled step, or far less, misses the bar.
        let over = summarize_profile(&[profile_row(1.0, 15.0), profile_row(1.0, 15.0)]);
        assert!(!over.within_10pct && over.ratio > 1.3);
        let under = summarize_profile(&[profile_row(1.0, 30.0)]);
        assert!(!under.within_10pct && under.ratio < 0.7);
    }

    #[test]
    fn a_profiled_run_refuses_a_window_it_cannot_hold_and_a_device_without_cuda() {
        let missing = std::env::temp_dir().join(format!("claude-157-none-{}", std::process::id()));
        let refusal = |profile: usize, steps: usize| -> String {
            let mut config = config(0, None);
            (config.profile, config.steps) = (profile, steps);
            match run_long(&missing, &missing.join("out"), &config, &Device::Cpu) {
                Err(e) => e.to_string(),
                Ok(()) => "ran".to_string(),
            }
        };
        assert!(refusal(usize::MAX, 72).contains("too large"));
        assert!(refusal(3, 13).contains("needs at least 14 steps"));
        assert!(refusal(3, 14).contains("needs a CUDA device"));
        assert!(!missing.exists());
    }

    #[test]
    fn a_config_without_the_profile_writes_no_profile_key() {
        let mut config = config(0, None);
        let json = String::from_utf8(serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        assert!(!json.contains("profile"), "{json}");
        config.profile = 3;
        let json = String::from_utf8(serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        assert!(json.contains("\"profile\": 3"), "{json}");
    }

    #[test]
    fn a_config_without_the_turn_memory_writes_neither_of_its_keys() {
        let mut config = config(0, None);
        let json = String::from_utf8(serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        assert!(!json.contains("turn_memory"), "{json}");
        config.turn_memory = 8;
        let json = String::from_utf8(serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        assert!(
            json.contains("\"turn_memory\": 8") && !json.contains("turn_memory_empty"),
            "{json}"
        );
        config.turn_memory_empty = true;
        let json = String::from_utf8(serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        assert!(json.contains("\"turn_memory_empty\": true"), "{json}");
    }

    #[test]
    fn the_empty_memory_control_refuses_a_run_with_no_memory() {
        let mut config = config(0, None);
        config.turn_memory_empty = true;
        let missing = std::env::temp_dir().join(format!("claude-161-none-{}", std::process::id()));
        let refused = start_memory(&missing, &config, &[], &Device::Cpu)
            .err()
            .map(|e| e.to_string());
        assert!(refused.is_some_and(|e| e.contains("--turn-memory-empty needs")));
        config.turn_memory_empty = false;
        let (memory, offsets, replies) =
            start_memory(&missing, &config, &[], &Device::Cpu).unwrap();
        assert!(memory.is_none() && offsets.is_empty() && replies.is_empty());
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_fold_reflects_into_the_unit_interval_and_keeps_its_gradient() {
        let device = Device::new_cuda(0).unwrap();
        let at = [-2.5f32, -0.3, 0.0, 0.4, 1.0, 1.2, 2.7, 3.0];
        let x = candle_core::Var::new(&at[..], &device).unwrap();
        let folded = crate::geometric_stack::fold_unit(x.as_tensor()).unwrap();
        let got = folded.to_vec1::<f32>().unwrap();
        let want = [0.5f32, 0.3, 0.0, 0.4, 1.0, 0.8, 0.7, 1.0];
        for (g, w) in got.iter().zip(&want) {
            assert!((g - w).abs() < 1e-6, "{got:?} != {want:?}");
        }
        let grads = folded.sum_all().unwrap().backward().unwrap();
        let slope = grads.get(x.as_tensor()).unwrap().to_vec1::<f32>().unwrap();
        // Away from the fold points the slope is +1 or -1: never zero, unlike a clamp's.
        for (i, &s) in slope.iter().enumerate() {
            if [1, 3, 5, 6].contains(&i) {
                assert_eq!(s.abs(), 1.0, "slope {slope:?}");
            }
        }
        assert_eq!(slope[1], -1.0);
        assert_eq!(slope[3], 1.0);
        assert_eq!(slope[5], -1.0);
    }

    #[test]
    fn tail_keeps_the_last_characters_on_one_line() {
        assert_eq!(tail("ab\ncdé", 3), "cdé");
        assert_eq!(tail("ab", 10), "ab");
    }
}
