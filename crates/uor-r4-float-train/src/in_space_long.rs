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

use crate::geometric_stack::{StackAdamW, StackModel};
use crate::hamilton_sheet::{self as hs, CausalWindow, HamiltonArm, SHEET_CONTEXT};
use crate::spiralcore_fbs_arms::{self as arms, FbsArm};
use crate::spiralcore_fbs_sheet::{build_fbs_model, panel_loss};
use crate::{invalid, sha256_bytes, Result};

/// One run's settings.
#[derive(Clone, Debug, Serialize)]
pub struct LongConfig {
    pub steps: usize,
    pub batch: usize,
    pub eval_every: usize,
    pub seed: u64,
    pub lr: f64,
    pub samples: usize,
    pub reply_tokens: usize,
}

/// One checkpoint's reading, one line of `progress.jsonl`.
#[derive(Clone, Debug, Serialize)]
pub struct Checkpoint {
    pub step: usize,
    pub epoch: f64,
    pub train_loss_mean: Option<f64>,
    pub dev_top1_hits: usize,
    pub dev_top8_hits: usize,
    pub dev_admitted_hits: usize,
    pub dev_scored_rows: usize,
    pub dev_loss: f64,
    pub train_seconds: f64,
    pub train_tokens_per_second: f64,
}

/// One fixed DEV prompt with the model's greedy reply and the true reply, as text.
#[derive(Clone, Debug, Serialize)]
pub struct Sample {
    pub step: usize,
    pub prompt: String,
    pub model: String,
    pub reference: String,
}

/// Clears the arm hook when the run ends, by any path.
struct ArmGuard;

impl Drop for ArmGuard {
    fn drop(&mut self) {
        arms::set_fbs_arm(None);
    }
}

fn batch_loss(
    model: &StackModel,
    ids: &[u32],
    scored: &[u32],
    targets: &[u32],
    batch: usize,
) -> Result<Tensor> {
    let hidden = model.hidden(ids, batch, SHEET_CONTEXT)?;
    let idx = Tensor::from_vec(scored.to_vec(), scored.len(), model.device())?;
    model.native_head_loss(&hidden.index_select(&idx, 0)?, targets, hs::SHEET_RIVALS)
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

/// `samples` DEV windows spread over the panel, each with a prompt before its first reply token.
fn sample_windows(dev: &[CausalWindow], samples: usize) -> Vec<&CausalWindow> {
    let eligible: Vec<&CausalWindow> = dev
        .iter()
        .filter(|w| w.scored_offsets.first().is_some_and(|&off| off >= 16))
        .collect();
    if eligible.is_empty() || samples == 0 {
        return Vec::new();
    }
    let stride = (eligible.len() / samples).max(1);
    eligible.into_iter().step_by(stride).take(samples).collect()
}

/// The model's greedy reply to `prompt`: `reply_tokens` tokens, each the writer's nearest pick on
/// the last `SHEET_CONTEXT` tokens, stopping early at BOS.
fn greedy_reply(model: &StackModel, prompt: &[u32], reply_tokens: usize, seed: u64) -> Result<Vec<u32>> {
    hs::with_gate_mode(HamiltonArm::HamiltonActive, seed, 0, false, || {
        let mut ids = prompt.to_vec();
        let mut reply = Vec::with_capacity(reply_tokens);
        for _ in 0..reply_tokens {
            let window = &ids[ids.len().saturating_sub(SHEET_CONTEXT)..];
            let next = model
                .native_next_token_ids(window, 1, window.len())?
                .flatten_all()?
                .to_dtype(DType::U32)?
                .to_vec1::<u32>()?;
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
    let flat: String = text.chars().map(|c| if c == '\n' { ' ' } else { c }).collect();
    let n = flat.chars().count();
    flat.chars().skip(n.saturating_sub(chars)).collect()
}

/// Train, score, write text and save, as the module documentation says. `out` must not exist.
pub fn run_long(data_root: &Path, out: &Path, config: &LongConfig, device: &Device) -> Result<()> {
    if config.steps == 0 || config.batch == 0 || config.eval_every == 0 {
        return Err(invalid("steps, batch and eval-every must be positive"));
    }
    if out.exists() {
        return Err(invalid(format!("{} exists; a run never writes over one", out.display())));
    }
    fs::create_dir_all(out)?;
    fs::write(out.join("config.json"), serde_json::to_vec_pretty(config)?)?;

    let tokenizer_bytes = fs::read(data_root.join("tokenizer.json"))?;
    if sha256_bytes(&tokenizer_bytes) != hs::PINNED_TOKENIZER_SHA256 {
        return Err(invalid("tokenizer.json differs from the pinned d36d3e87 tokenizer"));
    }
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| invalid("tokenizer.json is not a byte-level BPE tokenizer"))?;

    let (train_tokens, train_mask) = hs::load_pinned_uort_split(
        &data_root.join("mw-balp/train/tokens.u16"),
        &data_root.join("mw-balp/train/response_mask.u8"),
        hs::PINNED_TRAIN_TOKENS_SHA256,
        hs::PINNED_TRAIN_MASK_SHA256,
        hs::EXPECTED_TRAIN_TOKENS,
        hs::EXPECTED_TRAIN_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        hs::SHEET_BOS_ID,
    )?;
    let (train_windows, _) =
        hs::build_document_windows(&train_tokens, &train_mask, SHEET_CONTEXT, hs::SHEET_BOS_ID)?;
    let usable: Vec<CausalWindow> = train_windows
        .into_iter()
        .filter(|w| w.scored_offsets.len() >= 4)
        .collect();
    if usable.len() < config.batch || usable.iter().any(|w| w.input_ids.len() != SHEET_CONTEXT) {
        return Err(invalid("TRAIN windows are too few or not context-long"));
    }
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
    let prompts = sample_windows(&dev_windows, config.samples);
    println!(
        "[in-space] usable TRAIN windows {} (batch {}, {} steps = {:.2} epochs), DEV windows {}",
        usable.len(),
        config.batch,
        config.steps,
        (config.steps * config.batch) as f64 / usable.len() as f64,
        dev_windows.len()
    );

    arms::set_fbs_arm(Some(FbsArm::OneAtomActive));
    let _guard = ArmGuard;
    let model = build_fbs_model(FbsArm::OneAtomActive, config.seed, device)?;
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
        let point = Checkpoint {
            step,
            epoch: (step * config.batch) as f64 / usable.len() as f64,
            train_loss_mean,
            dev_top1_hits: dev.top1_hits,
            dev_top8_hits: dev.top8_hits,
            dev_admitted_hits: dev.admitted_target_hits,
            dev_scored_rows: dev.scored_rows,
            dev_loss,
            train_seconds,
            train_tokens_per_second: (step * config.batch * SHEET_CONTEXT) as f64
                / train_seconds.max(1e-9),
        };
        append_line(&progress, &serde_json::to_string(&point)?)?;
        println!(
            "[in-space] step {} epoch {:.2} train_loss {} dev top-1 {}/{} top-8 {} admitted {} dev_loss {:.5} train {:.0}s",
            step,
            point.epoch,
            train_loss_mean.map_or("-".to_owned(), |v| format!("{v:.5}")),
            dev.top1_hits,
            dev.scored_rows,
            dev.top8_hits,
            dev.admitted_target_hits,
            dev_loss,
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
            let reply = greedy_reply(model, prompt, config.reply_tokens, config.seed)?;
            let reference: Vec<u32> = window
                .scored_targets
                .iter()
                .take(config.reply_tokens)
                .copied()
                .collect();
            let sample = Sample {
                step,
                prompt: tokenizer.decode(prompt),
                model: tokenizer.decode(&reply),
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
        model.save(&save_root.join(format!("step-{step:06}")))?;
        Ok(())
    };

    checkpoint(&model, 0, None, 0.0)?;
    let mut order = epoch_order(usable.len(), config.seed, 0);
    let mut cursor = 0usize;
    let mut epoch = 0u64;
    for step in 0..config.steps {
        if cursor + config.batch > order.len() {
            epoch += 1;
            order = epoch_order(usable.len(), config.seed, epoch);
            cursor = 0;
        }
        let windows: Vec<&CausalWindow> = order[cursor..cursor + config.batch]
            .iter()
            .map(|&i| &usable[i])
            .collect();
        cursor += config.batch;
        let (ids, scored, targets) = assemble(&windows);
        let started = Instant::now();
        let loss = batch_loss(&model, &ids, &scored, &targets, config.batch)?;
        let value = f64::from(loss.to_scalar::<f32>()?);
        if !value.is_finite() {
            return Err(invalid(format!("non-finite training loss at step {step}")));
        }
        let grads = loss.backward()?;
        opt_main.update_filtered(&model, &grads, config.lr, |name| !name.ends_with("rec.decay"))?;
        opt_decay.update_filtered(
            &model,
            &grads,
            config.lr * hs::SHEET_DECAY_LR_MULT,
            |name| name.ends_with("rec.decay"),
        )?;
        device.synchronize()?;
        train_seconds += started.elapsed().as_secs_f64();
        loss_sum += value;
        loss_steps += 1;
        if (step + 1) % config.eval_every == 0 || step + 1 == config.steps {
            checkpoint(&model, step + 1, Some(loss_sum / loss_steps as f64), train_seconds)?;
            (loss_sum, loss_steps) = (0.0, 0);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn tail_keeps_the_last_characters_on_one_line() {
        assert_eq!(tail("ab\ncdé", 3), "cdé");
        assert_eq!(tail("ab", 10), "ab");
    }
}
