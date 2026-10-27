//! `train-in-space --data-root DIR --out DIR [--steps N] [--batch B] [--eval-every K] [--seed S]
//! [--lr X] [--lr-end X] [--decay-mult X] [--retain-rates 0|1] [--samples N] [--reply-tokens N]`
//!
//! Long training of the SpiralCore/FBS geometric trunk in its own space, with text at every
//! checkpoint (`uor_r4_float_train::in_space_long`). GPU only. `--data-root` holds `tokenizer.json`,
//! `mw-balp/train/` and `ft/dev/`, each split with `tokens.u16` and `response_mask.u8`. `--out`
//! must not exist; it receives `config.json`, `progress.jsonl`, `samples.md`, `samples.jsonl` and
//! `models/step-*/`.
use std::env;
use std::path::PathBuf;

use candle_core::Device;
use uor_r4_float_train::in_space_long::{run_long, LongConfig};
use uor_r4_float_train::{FloatTrainError, Result};

fn main() -> Result<()> {
    let mut data_root = None;
    let mut out = None;
    let mut config = LongConfig {
        steps: 32_768,
        batch: 16,
        eval_every: 2_048,
        seed: 1,
        lr: 0.002,
        lr_end: f64::NAN,
        decay_mult: 20.0,
        retain_rates: false,
        samples: 6,
        reply_tokens: 48,
    };
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| FloatTrainError::Invalid(format!("missing {arg} value")))?;
        let number = |v: &str| {
            v.parse::<usize>()
                .map_err(|_| FloatTrainError::Invalid(format!("{arg} wants a whole number")))
        };
        match arg.as_str() {
            "--data-root" => data_root = Some(PathBuf::from(&value)),
            "--out" => out = Some(PathBuf::from(&value)),
            "--steps" => config.steps = number(&value)?,
            "--batch" => config.batch = number(&value)?,
            "--eval-every" => config.eval_every = number(&value)?,
            "--seed" => config.seed = number(&value)? as u64,
            "--samples" => config.samples = number(&value)?,
            "--reply-tokens" => config.reply_tokens = number(&value)?,
            "--retain-rates" => config.retain_rates = number(&value)? == 1,
            "--lr" | "--lr-end" | "--decay-mult" => {
                let v: f64 = value
                    .parse()
                    .map_err(|_| FloatTrainError::Invalid(format!("{arg} wants a number")))?;
                match arg.as_str() {
                    "--lr" => config.lr = v,
                    "--lr-end" => config.lr_end = v,
                    _ => config.decay_mult = v,
                }
            }
            other => {
                return Err(FloatTrainError::Invalid(format!(
                    "unrecognized argument: {other}"
                )))
            }
        }
    }
    if config.lr_end.is_nan() {
        config.lr_end = config.lr;
    }
    let data_root =
        data_root.ok_or_else(|| FloatTrainError::Invalid("--data-root is required".to_owned()))?;
    let out = out.ok_or_else(|| FloatTrainError::Invalid("--out is required".to_owned()))?;
    let device = Device::new_cuda(0)?;
    run_long(&data_root, &out, &config, &device)
}
