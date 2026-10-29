//! `train-in-space --data-root DIR --out DIR [--objective sheet|voronoi|nearest|cell] [--steps N]
//! [--train-split DIR --train-sha256 HEX --train-mask-sha256 HEX]
//! [--batch B] [--eval-every K] [--seed S] [--lr X] [--lr-end X] [--decay-mult X]
//! [--retain-rates 0|1] [--fold-scores 0|1] [--reflect-floor 0|1] [--fix-input-table 0|1] [--flock-read K] [--content-read K] [--hamming-read H] [--opening-read K] [--opening-anchor turn|last] [--prompt-register Q] [--register-picks K] [--register-select learned|all] [--width N] [--type-spin 0|1] [--samples N] [--reply-tokens N] [--profile K] [--turn-memory K] [--turn-memory-empty 0|1] [--turn-memory-reply 0|1] [--turn-memory-dropout P] [--fbs-arm one-atom|identity|frozen]`
//!
//! `--profile K` times each stage of K training steps on the GPU after 8 of warm-up, each against
//! the unprofiled step after it, into `DIR/profile.jsonl` and `DIR/profile.json`
//! (`in_space_long::profile_window`; fbs-cost-profile, claude-157).
//!
//! `--turn-memory-dropout P` (thousandths, 0 to 1000; needs `--turn-memory`) drops the memory from
//! that share of training windows, which then name no memory row (one-model-end-to-end protocol
//! I-B, claude-207); scoring keeps it everywhere. `--fbs-arm` picks the SpiralCore/FBS transport
//! arm for training and for `--score-rows`, `--inspect` and `--decode-speed` (I-C): `one-atom`, the
//! trained transport (default), `identity` or `frozen` (routing queries kept at their seed).
//! Scoring refuses an arm other than the run's own config.json names.
//!
//! `--turn-memory K` keys each reply opening on the first K ids of its turn and moves its readout
//! point toward the token TRAIN's table stores for that key, through a weight that starts at 0
//! (`flock_read_memory`; route-plus-memory, claude-161). `--turn-memory-empty 1` runs it with an
//! empty table, the no-memory control. `--turn-memory-reply 1` (with `--turn-memory 8`) adds its
//! reply table (turn-memory-reply, claude-193): the rows past each opening, keyed as the
//! `--reply-bound` probe's `turn-8 + position + reply-last-1` at stored share 0.5, with their own
//! weight; with `--turn-memory-empty 1` both tables are empty.
//!
//! `train-in-space --data-root DIR --inspect RUN [--retain-rates 0|1] [--fold-scores 0|1] [--reflect-floor 0|1] [--seed S]`
//!
//! `train-in-space --data-root DIR --reach CHECKPOINT [--retain-rates 0|1] [--fold-scores 0|1] [--reflect-floor 0|1] [--seed S]`
//!
//! `train-in-space --data-root DIR --grad-norms 1 [--objective ...] [--retain-rates 0|1] [--fold-scores 0|1] [--reflect-floor 0|1] [--batch B] [--seed S]`
//!
//! `train-in-space --data-root DIR --decode-speed CHECKPOINT [--objective nearest] [--retain-rates 0|1] [--reply-tokens N]`
//!
//! `train-in-space --recall PAIRS --out DIR [--steps N] [--batch B] [--eval-every K] [--seed S] [--lr X] [--lr-end X] [--retain-rates 0|1] [--flock-read K] [--content-read K] [--hamming-read H] [--opening-read K] [--opening-anchor turn|last] [--prompt-register Q] [--register-picks K] [--register-select learned|all]`:
//! the multi-query associative recall task (`in_space_long::run_recall`), `PAIRS` key-value pairs
//! per distance bucket in each 192-token window, fresh every step; held-out recall per bucket goes
//! to `DIR/recall.jsonl`, and the final model to `DIR/models/`. Needs no `--data-root`, and
//! ignores it if given.
//!
//! `train-in-space --data-root DIR --opening-bound 1`: how well the prompt words a window shows
//! could name each reply opening, as TRAIN's commonest opening per key scored on DEV. A count
//! over the corpus that needs no GPU; one JSON line per key, then one line per split
//! (`dev`, `train`) with how many openings the opening read's turn anchor keeps the first-8 key at.
//!
//! `train-in-space --data-root DIR --reply-bound 1`: what an exact memory of TRAIN could name
//! among DEV's reply rows, for each candidate key and stored-share threshold, by kind (openings,
//! copies, the rest): turn-memory-reply's probe. A count that needs no GPU; one JSON line per
//! key and threshold.
//!
//! `train-in-space --data-root DIR --fresh-v2 DIR [--panel NAME=DIR ...] --census 1`: one-model-end-to-end's data census
//! (claude-202). TRAIN's and the panels' sizes, the probe's pinned counts reproduced, step 2's
//! decision rows, and each panel's exact, n-gram and value-masked template overlap with TRAIN. A
//! count that needs no GPU; one JSON line.
//!
//! `train-in-space --data-root DIR --score-rows CHECKPOINT --out DIR [--fresh-v2 DIR] [--panel NAME=DIR ...] [--samples N] [--reply-tokens N] [--retain-rates 0|1] [--fold-scores 0|1] [--reflect-floor 0|1] [--seed S]`:
//! one-model-end-to-end's per-row scorer (claude-205), on the GPU. Every scored DEV row (and
//! Fresh V2 row) of the step-N checkpoint, with its memory on and taken out, and greedy samples,
//! as JSON files in `--out` (`in_space_long::score_rows`). Prints one JSON line per file. The rate,
//! fold and floor flags must be the checkpoint's run's, as for `--inspect`. Each `--panel` (an unpinned UORT
//! split, its sha recorded) is scored and sampled like DEV; the samples come greedy, and under the
//! Flock rank rule with the memory on, each with its reply-table hits (protocol I-A, I-E, I-F,
//! claude-208).
//!
//! `train-in-space --data-root DIR --overturn 1 --scored ARM-sSEED=DIR ...`: grades the overturn
//! check from those files, on the laptop (`in_space_long::overturn`). Needs `r-s1`, `r0-s1`,
//! `r-s2` and `r0-s2`; counts only; one JSON line.
//!
//! Long training of the SpiralCore/FBS geometric trunk in its own space, with text at every
//! checkpoint (`uor_r4_float_train::in_space_long`). GPU only. `--data-root` holds `tokenizer.json`,
//! `mw-balp/train/` and `ft/dev/`, each split with `tokens.u16` and `response_mask.u8`. `--out`
//! must not exist; it receives `config.json`, `progress.jsonl`, `samples.md`, `samples.jsonl` and
//! `models/step-*/`. `--inspect` instead reads every checkpoint a run saved and prints one JSON
//! line each, after one line with the address roots' separation.
use std::env;
use std::path::PathBuf;

use candle_core::Device;
use uor_r4_float_train::flock_read_opening::OpeningAnchor;
use uor_r4_float_train::flock_read_register::RegisterSelect;
use uor_r4_float_train::in_space_long::{
    census, context_reach, decode_speed, fbs_arm_named, gradient_norms, inspect_run, opening_bound,
    overturn, reply_bound, root_separation, run_long, run_recall, score_rows, LongConfig,
    PointObjective, TrainSplit, DEFAULT_REGISTER_PICKS, MEMORY_DROPOUT_MAX,
};
use uor_r4_float_train::{FloatTrainError, Result};

fn main() -> Result<()> {
    let mut data_root = None;
    let mut out = None;
    let mut inspect = None;
    let mut reach = None;
    let mut grad_norms = false;
    let mut decode = None;
    let mut bound = false;
    let mut reply = false;
    let mut census_on = false;
    let mut fresh_v2 = None;
    let mut extra_panels: Vec<(String, PathBuf)> = Vec::new();
    let mut score = None;
    let mut overturn_on = false;
    let mut scored: Vec<(String, PathBuf)> = Vec::new();
    let (mut split_dir, mut split_tokens, mut split_mask) = (None, None, None);
    let mut recall = 0usize;
    let (mut opening_read, mut opening_anchor) = (None, None);
    let (mut prompt_register, mut register_picks, mut register_select) = (None, None, None);
    let mut config = LongConfig {
        objective: PointObjective::Sheet,
        steps: 32_768,
        batch: 16,
        eval_every: 2_048,
        seed: 1,
        lr: 0.002,
        lr_end: f64::NAN,
        decay_mult: 20.0,
        retain_rates: false,
        fold_scores: false,
        reflect_floor: false,
        fix_input_table: false,
        train_split: None,
        flock_read: 0,
        content_read: 0,
        hamming_read: 0,
        opening_read: 0,
        opening_anchor: None,
        prompt_register: 0,
        register_picks: None,
        register_select: None,
        width: uor_r4_float_train::hamilton_sheet::SHEET_WIDTH,
        type_spin: false,
        samples: 6,
        reply_tokens: 48,
        profile: 0,
        turn_memory: 0,
        turn_memory_empty: false,
        turn_memory_reply: false,
        turn_memory_dropout: 0,
        fbs_arm: uor_r4_float_train::spiralcore_fbs_arms::FbsArm::OneAtomActive,
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
        let switch = |v: &str| match v {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(FloatTrainError::Invalid(format!("{arg} wants 0 or 1"))),
        };
        match arg.as_str() {
            "--data-root" => data_root = Some(PathBuf::from(&value)),
            "--out" => out = Some(PathBuf::from(&value)),
            "--inspect" => inspect = Some(PathBuf::from(&value)),
            "--reach" => reach = Some(PathBuf::from(&value)),
            "--grad-norms" => grad_norms = switch(&value)?,
            "--decode-speed" => decode = Some(PathBuf::from(&value)),
            "--opening-bound" => bound = switch(&value)?,
            "--reply-bound" => reply = switch(&value)?,
            "--census" => census_on = switch(&value)?,
            "--score-rows" => score = Some(PathBuf::from(&value)),
            "--overturn" => overturn_on = switch(&value)?,
            "--scored" => {
                let arm = panel_arg(&value)?;
                if scored.iter().any(|(name, _)| *name == arm.0) {
                    return Err(FloatTrainError::Invalid(format!(
                        "--scored {} is given twice",
                        arm.0
                    )));
                }
                scored.push(arm);
            }
            "--fresh-v2" => fresh_v2 = Some(PathBuf::from(&value)),
            "--panel" => {
                let panel = panel_arg(&value)?;
                if extra_panels.iter().any(|(name, _)| *name == panel.0) {
                    return Err(FloatTrainError::Invalid(format!(
                        "--panel {} is given twice",
                        panel.0
                    )));
                }
                extra_panels.push(panel);
            }
            "--recall" => recall = number(&value)?,
            "--objective" => {
                config.objective = match value.as_str() {
                    "sheet" => PointObjective::Sheet,
                    "voronoi" => PointObjective::Voronoi,
                    "nearest" => PointObjective::Nearest,
                    "cell" => PointObjective::Cell,
                    other => {
                        return Err(FloatTrainError::Invalid(format!(
                            "--objective wants sheet, voronoi, nearest or cell, not {other}"
                        )))
                    }
                }
            }
            "--steps" => config.steps = number(&value)?,
            "--profile" => config.profile = number(&value)?,
            "--turn-memory" => config.turn_memory = number(&value)?,
            "--turn-memory-empty" => config.turn_memory_empty = switch(&value)?,
            "--turn-memory-reply" => config.turn_memory_reply = switch(&value)?,
            "--turn-memory-dropout" => {
                config.turn_memory_dropout = u32::try_from(number(&value)?)
                    .ok()
                    .filter(|&p| p <= MEMORY_DROPOUT_MAX)
                    .ok_or_else(|| {
                        FloatTrainError::Invalid(
                            "--turn-memory-dropout wants thousandths, 0 to 1000".to_owned(),
                        )
                    })?
            }
            "--fbs-arm" => {
                config.fbs_arm = fbs_arm_named(&value).ok_or_else(|| {
                    FloatTrainError::Invalid(format!(
                        "--fbs-arm wants one-atom, identity or frozen, not {value}"
                    ))
                })?
            }
            "--batch" => config.batch = number(&value)?,
            "--eval-every" => config.eval_every = number(&value)?,
            "--seed" => config.seed = number(&value)? as u64,
            "--samples" => config.samples = number(&value)?,
            "--reply-tokens" => config.reply_tokens = number(&value)?,
            "--retain-rates" => config.retain_rates = switch(&value)?,
            "--fold-scores" => config.fold_scores = switch(&value)?,
            "--reflect-floor" => config.reflect_floor = switch(&value)?,
            "--fix-input-table" => config.fix_input_table = switch(&value)?,
            "--flock-read" => config.flock_read = number(&value)?,
            "--content-read" => config.content_read = number(&value)?,
            "--hamming-read" => config.hamming_read = number(&value)?,
            "--opening-read" => match number(&value)? {
                k @ 1..=16 => opening_read = Some(k),
                _ => {
                    return Err(FloatTrainError::Invalid(
                        "--opening-read wants 1 to 16".to_owned(),
                    ))
                }
            },
            "--opening-anchor" => {
                opening_anchor = Some(match value.as_str() {
                    "turn" => OpeningAnchor::Turn,
                    "last" => OpeningAnchor::Last,
                    other => {
                        return Err(FloatTrainError::Invalid(format!(
                            "--opening-anchor wants turn or last, not {other}"
                        )))
                    }
                })
            }
            "--prompt-register" => match number(&value)? {
                q @ 1..=16 => prompt_register = Some(q),
                _ => {
                    return Err(FloatTrainError::Invalid(
                        "--prompt-register wants 1 to 16".to_owned(),
                    ))
                }
            },
            "--register-picks" => match number(&value)? {
                k @ 1..=32 => register_picks = Some(k),
                _ => {
                    return Err(FloatTrainError::Invalid(
                        "--register-picks wants 1 to 32".to_owned(),
                    ))
                }
            },
            "--register-select" => {
                register_select = Some(match value.as_str() {
                    "learned" => RegisterSelect::Learned,
                    "all" => RegisterSelect::All,
                    other => {
                        return Err(FloatTrainError::Invalid(format!(
                            "--register-select wants learned or all, not {other}"
                        )))
                    }
                })
            }
            "--width" => config.width = number(&value)?,
            "--type-spin" => config.type_spin = switch(&value)?,
            "--train-split" => split_dir = Some(PathBuf::from(&value)),
            "--train-sha256" => split_tokens = Some(value.clone()),
            "--train-mask-sha256" => split_mask = Some(value.clone()),
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
    // The opening read (claude-154): its anchor is written exactly when the read is on.
    match (opening_read, opening_anchor) {
        (Some(k), anchor) => {
            config.opening_read = k;
            config.opening_anchor = Some(anchor.unwrap_or_default());
        }
        (None, Some(_)) => {
            return Err(FloatTrainError::Invalid(
                "--opening-anchor needs --opening-read".to_owned(),
            ))
        }
        (None, None) => {}
    }
    // The prompt register (claude-155): its picks and select are written exactly when it is on.
    match (prompt_register, register_picks, register_select) {
        (Some(q), _, Some(RegisterSelect::All)) if q != 1 => {
            return Err(FloatTrainError::Invalid(
                "--register-select all wants --prompt-register 1".to_owned(),
            ))
        }
        (Some(q), picks, select) => {
            config.prompt_register = q;
            config.register_picks = Some(picks.unwrap_or(DEFAULT_REGISTER_PICKS));
            config.register_select = Some(select.unwrap_or_default());
        }
        (None, None, None) => {}
        (None, _, _) => {
            return Err(FloatTrainError::Invalid(
                "--register-picks and --register-select need --prompt-register".to_owned(),
            ))
        }
    }
    if config.lr_end.is_nan() {
        config.lr_end = config.lr;
    }
    config.train_split = match (split_dir, split_tokens, split_mask) {
        (None, None, None) => None,
        (Some(dir), Some(tokens_sha256), Some(mask_sha256)) => Some(TrainSplit {
            dir,
            tokens_sha256,
            mask_sha256,
        }),
        _ => {
            return Err(FloatTrainError::Invalid(
                "--train-split needs --train-sha256 and --train-mask-sha256, and they need it"
                    .to_owned(),
            ))
        }
    };
    let modes = [
        inspect.is_some(),
        reach.is_some(),
        grad_norms,
        decode.is_some(),
        bound,
        reply,
        census_on,
        // `--score-rows` writes to `--out` without training.
        out.is_some() && score.is_none(),
        score.is_some(),
        overturn_on,
    ];
    if modes.iter().filter(|&&m| m).count() > 1 {
        return Err(FloatTrainError::Invalid(
            "give one of --out, --inspect, --reach, --grad-norms 1, --decode-speed, --opening-bound 1, --reply-bound 1, --census 1, --score-rows or --overturn 1"
                .to_owned(),
        ));
    }
    if overturn_on && fresh_v2.is_some() {
        return Err(FloatTrainError::Invalid(
            "--overturn reads Fresh V2 from the scored files; it takes no --fresh-v2".to_owned(),
        ));
    }
    if !scored.is_empty() && !overturn_on {
        return Err(FloatTrainError::Invalid(
            "--scored needs --overturn 1".to_owned(),
        ));
    }
    // These modes run the default transport only; another arm would be scored or trained wrong.
    if (recall > 0 || grad_norms || reach.is_some())
        && config.fbs_arm != uor_r4_float_train::spiralcore_fbs_arms::FbsArm::OneAtomActive
    {
        return Err(FloatTrainError::Invalid(
            "--fbs-arm applies to training, --score-rows, --inspect and --decode-speed; --recall, --grad-norms and --reach run one-atom only".to_owned(),
        ));
    }
    if recall > 0 {
        let out = out.ok_or_else(|| FloatTrainError::Invalid("--recall needs --out".to_owned()))?;
        if modes[..5].iter().any(|&m| m) || score.is_some() || overturn_on {
            return Err(FloatTrainError::Invalid(
                "--recall trains; it takes --out and no other mode".to_owned(),
            ));
        }
        let device = Device::new_cuda(0)?;
        return run_recall(&out, &config, recall, &device);
    }
    let data_root =
        data_root.ok_or_else(|| FloatTrainError::Invalid("--data-root is required".to_owned()))?;
    if overturn_on {
        println!("{}", serde_json::to_string(&overturn(&data_root, &scored)?)?);
        return Ok(());
    }
    if census_on {
        let fresh_v2 = fresh_v2
            .ok_or_else(|| FloatTrainError::Invalid("--census needs --fresh-v2 DIR".to_owned()))?;
        println!(
            "{}",
            serde_json::to_string(&census(&data_root, &fresh_v2, &extra_panels)?)?
        );
        return Ok(());
    }
    if reply {
        for row in reply_bound(&data_root)? {
            println!("{}", serde_json::to_string(&row)?);
        }
        return Ok(());
    }
    if bound {
        let (rows, checks) = opening_bound(&data_root)?;
        for row in rows {
            println!("{}", serde_json::to_string(&row)?);
        }
        for check in checks {
            println!("{}", serde_json::to_string(&check)?);
        }
        return Ok(());
    }
    let device = Device::new_cuda(0)?;
    if let Some(checkpoint) = score {
        let out = out
            .ok_or_else(|| FloatTrainError::Invalid("--score-rows needs --out DIR".to_owned()))?;
        for file in score_rows(
            &data_root,
            &checkpoint,
            &out,
            fresh_v2.as_deref(),
            &extra_panels,
            config.samples,
            config.reply_tokens,
            config.retain_rates,
            config.fold_scores,
            config.reflect_floor,
            config.seed,
            config.fbs_arm,
            &device,
        )? {
            println!("{}", serde_json::to_string(&file)?);
        }
        return Ok(());
    }
    if let Some(checkpoint) = decode {
        let nearest = config.objective.decodes_nearest();
        for speed in decode_speed(
            &data_root,
            &checkpoint,
            config.retain_rates,
            config.fold_scores,
            config.reflect_floor,
            nearest,
            config.reply_tokens,
            config.seed,
            config.fbs_arm,
            &device,
        )? {
            println!("{}", serde_json::to_string(&speed)?);
        }
        return Ok(());
    }
    if grad_norms {
        for norm in gradient_norms(&data_root, &config, &device)? {
            println!("{}", serde_json::to_string(&norm)?);
        }
        return Ok(());
    }
    if let Some(checkpoint) = reach {
        let (reach, bins) = context_reach(
            &data_root,
            &checkpoint,
            config.retain_rates,
            config.fold_scores,
            config.reflect_floor,
            config.seed,
            &device,
        )?;
        for point in reach {
            println!("{}", serde_json::to_string(&point)?);
        }
        for layer in bins {
            println!("{}", serde_json::to_string(&layer)?);
        }
        return Ok(());
    }
    if let Some(run) = inspect {
        let (nearest, farthest) = root_separation()?;
        println!(
            "{}",
            serde_json::json!({"address_root_separation": {"nearest": nearest, "farthest": farthest}})
        );
        for reading in inspect_run(
            &data_root,
            &run,
            config.retain_rates,
            config.fold_scores,
            config.reflect_floor,
            config.seed,
            config.fbs_arm,
            &device,
        )? {
            println!("{}", serde_json::to_string(&reading)?);
        }
        return Ok(());
    }
    let out = out.ok_or_else(|| FloatTrainError::Invalid("--out is required".to_owned()))?;
    run_long(&data_root, &out, &config, &device)
}

/// `--panel NAME=DIR` (claude-204): an extra UORT split for the census, counted unpinned.
fn panel_arg(value: &str) -> Result<(String, PathBuf)> {
    match value.split_once('=') {
        Some((name, dir)) if !name.is_empty() && !dir.is_empty() => {
            Ok((name.to_owned(), PathBuf::from(dir)))
        }
        _ => Err(FloatTrainError::Invalid(
            "--panel takes NAME=DIR".to_owned(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_panel_argument_is_a_name_and_a_directory() {
        let (name, dir) = panel_arg("p-dev=/data/p-dev").unwrap();
        assert_eq!((name.as_str(), dir), ("p-dev", PathBuf::from("/data/p-dev")));
        assert!(panel_arg("p-dev").is_err());
        assert!(panel_arg("=dir").is_err());
        assert!(panel_arg("name=").is_err());
    }
}
