//! `corpus-train`: runs the integer block-coordinate trainer over any pinned
//! corpus at up to 4,096 rows a batch, with checkpoints, resume, progress
//! lines and DEV evaluation. It adds no model operation: every update is
//! `PreparedDriver::run_block_transitions`.
#![forbid(unsafe_code)]
use crate::{Error, Result};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(format!("corpus-train: {}", message.into()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// Batches to add in this invocation.
    pub batches: u64,
    /// DEV evaluation cadence in completed batches; 0 evaluates only at the end.
    pub eval_every: u64,
    pub checkpoint_every: u64,
    pub progress_every: u64,
    pub resume: bool,
}

/// Parses `<config.json> <out_dir> --batches N [--eval-every E]
/// [--checkpoint-every C] [--progress-every P] [--resume]`.
pub fn parse(args: &[String]) -> Result<(PathBuf, PathBuf, Options)> {
    if args.len() < 2 {
        return Err(invalid("needs <config.json> <out_dir> --batches N"));
    }
    let mut options = Options {
        batches: 0,
        eval_every: 0,
        checkpoint_every: 256,
        progress_every: 32,
        resume: false,
    };
    let mut rest = args[2..].iter();
    while let Some(flag) = rest.next() {
        let mut number = |name: &str| -> Result<u64> {
            rest.next()
                .ok_or_else(|| invalid(format!("{name} needs a number")))?
                .parse::<u64>()
                .map_err(|e| invalid(format!("{name}: {e}")))
        };
        match flag.as_str() {
            "--batches" => options.batches = number("--batches")?,
            "--eval-every" => options.eval_every = number("--eval-every")?,
            "--checkpoint-every" => options.checkpoint_every = number("--checkpoint-every")?,
            "--progress-every" => options.progress_every = number("--progress-every")?,
            "--resume" => options.resume = true,
            other => return Err(invalid(format!("unknown argument {other}"))),
        }
    }
    if options.batches == 0 || options.checkpoint_every == 0 || options.progress_every == 0 {
        return Err(invalid(
            "--batches, --checkpoint-every and --progress-every must be positive",
        ));
    }
    Ok((PathBuf::from(&args[0]), PathBuf::from(&args[1]), options))
}

pub fn checkpoint_name(completed_batches: u64) -> String {
    format!("checkpoint-{completed_batches:09}.json")
}

/// The checkpoint with the largest batch count in `dir`, if any.
pub fn newest_checkpoint(dir: &Path) -> Result<Option<(u64, PathBuf)>> {
    let mut newest: Option<(u64, PathBuf)> = None;
    for entry in fs::read_dir(dir).map_err(|e| invalid(format!("{}: {e}", dir.display())))? {
        let path = entry.map_err(|e| invalid(e.to_string()))?.path();
        let batches = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_prefix("checkpoint-"))
            .and_then(|n| n.strip_suffix(".json"))
            .filter(|n| n.len() == 9)
            .and_then(|n| n.parse::<u64>().ok());
        if let Some(batches) = batches {
            if newest.as_ref().map_or(true, |(best, _)| batches > *best) {
                newest = Some((batches, path));
            }
        }
    }
    Ok(newest)
}

/// A fresh start needs an empty (or absent) folder; a resume needs a checkpoint.
pub fn prepare_out(dir: &Path, resume: bool) -> Result<Option<(u64, PathBuf)>> {
    if resume {
        return newest_checkpoint(dir)?
            .map(Some)
            .ok_or_else(|| invalid("--resume found no checkpoint in the output folder"));
    }
    if dir.exists()
        && fs::read_dir(dir)
            .map_err(|e| invalid(e.to_string()))?
            .next()
            .is_some()
    {
        return Err(invalid("a fresh start needs an empty output folder"));
    }
    fs::create_dir_all(dir).map_err(|e| invalid(format!("{}: {e}", dir.display())))?;
    Ok(None)
}

/// Batches until the next multiple of any nonzero cadence, at most `remaining`.
pub fn next_chunk(completed: u64, remaining: u64, cadences: &[u64]) -> u64 {
    cadences
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| c - completed % c)
        .fold(remaining, u64::min)
}

/// The per-batch row lists are megabytes per evaluation; the aggregate is kept.
fn without_rows(mut dev: serde_json::Value) -> serde_json::Value {
    if let Some(fields) = dev.as_object_mut() {
        fields.remove("batches");
    }
    dev
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let temp = path.with_extension("json.partial");
    let mut file =
        fs::File::create(&temp).map_err(|e| invalid(format!("{}: {e}", temp.display())))?;
    file.write_all(bytes).map_err(|e| invalid(e.to_string()))?;
    file.sync_all().map_err(|e| invalid(e.to_string()))?;
    fs::rename(&temp, path).map_err(|e| invalid(e.to_string()))
}

fn append_line(path: &Path, value: &serde_json::Value) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| invalid(format!("{}: {e}", path.display())))?;
    writeln!(file, "{value}").map_err(|e| invalid(e.to_string()))
}

#[cfg(feature = "cuda")]
pub fn run(
    config: crate::training::DriverConfig,
    out: &Path,
    options: &Options,
) -> Result<serde_json::Value> {
    use crate::training::{DriverCheckpoint, PreparedDriver};
    use serde_json::json;
    use std::time::Instant;

    config.validate()?;
    if config.coordinate_block.is_one() {
        return Err(invalid(
            "needs coordinate_block above 1 (the rows_v1 block path)",
        ));
    }
    let resume_from = prepare_out(out, options.resume)?;
    let rows = u64::from(config.batch_rows);
    let config_sha256 =
        crate::corpus::sha256(&serde_json::to_vec(&config).map_err(|e| invalid(e.to_string()))?);
    let load_started = Instant::now();
    let driver = PreparedDriver::load(config)?;
    let lineage_sha256 = driver.lineage_sha256()?;
    let (mut model, mut cursor) = match &resume_from {
        Some((_, path)) => {
            let bytes = fs::read(path).map_err(|e| invalid(format!("{}: {e}", path.display())))?;
            let checkpoint: DriverCheckpoint =
                serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?;
            driver.restore(checkpoint, 0)?
        }
        None => (driver.new_model(0)?, driver.initial_cursor()),
    };
    let start_batches = cursor.completed_batches;
    let end_batches = start_batches
        .checked_add(options.batches)
        .ok_or_else(|| invalid("batch count overflow"))?;
    let admission = crate::training::issue_carrier_arm_admission(&driver, end_batches)?;
    append_line(
        &out.join("runs.jsonl"),
        &json!({
            "schema": "uor-r4.discrete-corpus-train/1",
            "config_sha256": config_sha256,
            "lineage_sha256": lineage_sha256,
            "driver_source_sha256": crate::training::driver_source_sha256(),
            "compiled": crate::device::compiled_identity(),
            "rows_per_batch": rows,
            "train_eligible_targets": driver.train().eligible_len(),
            "dev_eligible_targets": driver.dev().eligible_len(),
            "start_batches": start_batches,
            "end_batches": end_batches,
            "resumed": options.resume,
            "load_ms": load_started.elapsed().as_millis() as u64,
        }),
    )?;
    if resume_from.is_none() {
        let checkpoint = driver.checkpoint(&model, cursor)?;
        write_atomic(
            &out.join(checkpoint_name(0)),
            &serde_json::to_vec(&checkpoint).map_err(|e| invalid(e.to_string()))?,
        )?;
        let dev = without_rows(driver.evaluate(&model)?);
        append_line(
            &out.join("dev.jsonl"),
            &json!({"completed_batches": 0, "dev": dev}),
        )?;
    }
    let cadences = [
        options.checkpoint_every,
        options.eval_every,
        options.progress_every,
    ];
    let run_started = Instant::now();
    let mut total_moves = 0u64;
    let mut last_dev = cursor.completed_batches;
    while cursor.completed_batches < end_batches {
        let chunk = next_chunk(
            cursor.completed_batches,
            end_batches - cursor.completed_batches,
            &cadences,
        );
        let chunk = std::num::NonZeroU64::new(chunk).ok_or_else(|| invalid("empty chunk"))?;
        let started = Instant::now();
        let phases_before = crate::training::block_phase_micros();
        let (next, receipts) =
            driver.run_block_transitions(&mut model, cursor, &admission, chunk)?;
        let elapsed_ms = (started.elapsed().as_millis() as u64).max(1);
        cursor = next;
        let moves = receipts
            .iter()
            .map(|r| r.commit_ordinals.len() as u64)
            .sum::<u64>();
        let before = receipts
            .iter()
            .fold(0u64, |s, r| s.saturating_add(r.l_before));
        let after = receipts
            .iter()
            .fold(0u64, |s, r| s.saturating_add(r.committed_loss));
        total_moves += moves;
        let phases_now = crate::training::block_phase_micros();
        let phase_ms = |i: usize| (phases_now[i] - phases_before[i]) / 1000;
        let phases = json!({"host_batch": phase_ms(0), "carry": phase_ms(1), "propose": phase_ms(2), "commit": phase_ms(3)});
        let line = json!({
            "completed_batches": cursor.completed_batches,
            "corpus_pass": cursor.corpus_pass,
            "next_target": cursor.next_target,
            "accepted_updates": cursor.accepted_updates,
            "chunk_batches": chunk.get(),
            "chunk_moves": moves,
            "chunk_loss_before": before,
            "chunk_loss_committed": after,
            "chunk_ms": elapsed_ms,
            "chunk_phase_ms": phases,
            "score_phase_us_total": crate::device::score_phase_micros().to_vec(),
            "targets_per_second": chunk.get() * rows * 1000 / elapsed_ms,
        });
        append_line(&out.join("progress.jsonl"), &line)?;
        println!("{line}");
        let done = cursor.completed_batches == end_batches;
        if done || cursor.completed_batches % options.checkpoint_every == 0 {
            let checkpoint = driver.checkpoint(&model, cursor)?;
            write_atomic(
                &out.join(checkpoint_name(cursor.completed_batches)),
                &serde_json::to_vec(&checkpoint).map_err(|e| invalid(e.to_string()))?,
            )?;
        }
        if done || (options.eval_every > 0 && cursor.completed_batches % options.eval_every == 0) {
            let dev = without_rows(driver.evaluate(&model)?);
            let line = json!({"completed_batches": cursor.completed_batches, "dev": dev});
            append_line(&out.join("dev.jsonl"), &line)?;
            println!("{line}");
            last_dev = cursor.completed_batches;
        }
    }
    let done = json!({
        "schema": "uor-r4.discrete-corpus-train-done/1",
        "lineage_sha256": lineage_sha256,
        "start_batches": start_batches,
        "completed_batches": cursor.completed_batches,
        "accepted_updates": cursor.accepted_updates,
        "moves_this_run": total_moves,
        "last_dev_batches": last_dev,
        "run_ms": run_started.elapsed().as_millis() as u64,
    });
    write_atomic(
        &out.join(format!("done-{:09}.json", cursor.completed_batches)),
        &serde_json::to_vec(&done).map_err(|e| invalid(e.to_string()))?,
    )?;
    Ok(done)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("uor-corpus-train-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn parses_flags_and_refuses_bad_ones() {
        let (config, out, options) = parse(&args(&[
            "c.json",
            "out",
            "--batches",
            "40",
            "--eval-every",
            "8",
            "--resume",
        ]))
        .unwrap();
        assert_eq!(
            (config, out),
            (PathBuf::from("c.json"), PathBuf::from("out"))
        );
        assert_eq!(
            options,
            Options {
                batches: 40,
                eval_every: 8,
                checkpoint_every: 256,
                progress_every: 32,
                resume: true
            }
        );
        assert!(parse(&args(&["c.json", "out"])).is_err());
        assert!(parse(&args(&["c.json", "out", "--batches"])).is_err());
        assert!(parse(&args(&[
            "c.json",
            "out",
            "--batches",
            "4",
            "--checkpoint-every",
            "0"
        ]))
        .is_err());
        assert!(parse(&args(&["c.json", "out", "--batches", "4", "--what"])).is_err());
    }

    #[test]
    fn picks_the_newest_checkpoint_by_batch_count() {
        let dir = scratch("newest");
        assert_eq!(newest_checkpoint(&dir).unwrap(), None);
        for name in [
            checkpoint_name(0),
            checkpoint_name(512),
            checkpoint_name(256),
        ] {
            fs::write(dir.join(name), b"{}").unwrap();
        }
        fs::write(dir.join("checkpoint-999999999999.json"), b"{}").unwrap();
        fs::write(dir.join("checkpoint-000000900.json.partial"), b"{}").unwrap();
        let (batches, path) = newest_checkpoint(&dir).unwrap().unwrap();
        assert_eq!(batches, 512);
        assert_eq!(path, dir.join("checkpoint-000000512.json"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn fresh_start_refuses_a_used_folder_and_resume_needs_a_checkpoint() {
        let dir = scratch("fresh");
        assert_eq!(prepare_out(&dir, false).unwrap(), None);
        assert!(prepare_out(&dir, true).is_err());
        fs::write(dir.join("progress.jsonl"), b"").unwrap();
        assert!(prepare_out(&dir, false).is_err());
        fs::write(dir.join(checkpoint_name(8)), b"{}").unwrap();
        let (batches, _) = prepare_out(&dir, true).unwrap().unwrap();
        assert_eq!(batches, 8);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn chunks_stop_on_every_cadence_boundary() {
        assert_eq!(next_chunk(0, 100, &[256, 0, 32]), 32);
        assert_eq!(next_chunk(250, 100, &[256, 0, 32]), 6);
        assert_eq!(next_chunk(256, 3, &[256, 8, 32]), 3);
        assert_eq!(next_chunk(4, 100, &[256, 8, 32]), 4);
    }
}
