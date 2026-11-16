//! The read-binding hinge's trainer: the gates of the native Q31 read, stepped against
//! the rank hinge of [`uor_r4_native_vector::bind_train`].
//!
//! `usage: native_bind_train --dump F [--reverse] [--start identity|dump|FILE]
//!   [--steps N] [--step-shift S] [--batch N] [--eval-at L,...]
//!   --out-jsonl F --save-read F`
//!
//! The labels are the dump's own sidecars: `<dump>.ask.u32` (the asked fact's value
//! position) bounds the row and `<dump>.other.u32` (the other fact's) competes, or the
//! other way round under `--reverse` — the codex-18 control that asks whether hinge
//! supervision can reach the other fact at all.
//!
//! This is the host reference step. It has no device twin yet: the pick trainer's CUDA
//! kernel (`native_q31_read_pick_loss`) computes a distance rival, and the rank hinge
//! needs the two ranks, so the device port is its own unit. One JSON line an eval step,
//! the read saved with `Q31FirstPickRead::to_bytes`.

use std::io::Write;
use std::path::Path;

use uor_r4_discrete::native_vector::{
    BindLabels, BindReport, Error, Q31FirstPickRead, Q31Quaternion, Q31ReadDump, Result,
};

/// The identity gate: the unit quaternion `(1, 0, 0, 0)` at Q31.
const IDENTITY_GATE: Q31Quaternion = [1 << 31, 0, 0, 0];

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

#[derive(Clone, Debug)]
struct Args {
    dump: String,
    eval_dump: Option<String>,
    reverse: bool,
    start: String,
    steps: usize,
    step_shift: u32,
    batch: usize,
    eval_at: Vec<usize>,
    out_jsonl: String,
    save_read: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            dump: String::new(),
            eval_dump: None,
            reverse: false,
            start: "identity".to_owned(),
            steps: 512,
            step_shift: 8,
            batch: 64,
            eval_at: vec![0, 64, 128, 256, 512],
            out_jsonl: String::new(),
            save_read: String::new(),
        }
    }
}

fn number<T: std::str::FromStr>(flag: &str, value: &str) -> Result<T> {
    value
        .parse()
        .map_err(|_| invalid(format!("{flag} needs a whole number, got {value}")))
}

fn parse_args(argv: &[String]) -> Result<Args> {
    let mut args = Args::default();
    let mut i = 0;
    while i < argv.len() {
        let flag = argv[i].as_str();
        let value = || {
            argv.get(i + 1)
                .cloned()
                .ok_or_else(|| invalid(format!("{flag} needs a value")))
        };
        match flag {
            "--dump" => {
                args.dump = value()?;
                i += 1;
            }
            "--eval-dump" => {
                args.eval_dump = Some(value()?);
                i += 1;
            }
            "--reverse" => args.reverse = true,
            "--start" => {
                args.start = value()?;
                i += 1;
            }
            "--steps" => {
                args.steps = number(flag, &value()?)?;
                i += 1;
            }
            "--step-shift" => {
                args.step_shift = number(flag, &value()?)?;
                i += 1;
            }
            "--batch" => {
                args.batch = number(flag, &value()?)?;
                i += 1;
            }
            "--eval-at" => {
                let list = value()?;
                args.eval_at = list
                    .split(',')
                    .map(|step| number(flag, step))
                    .collect::<Result<Vec<usize>>>()?;
                i += 1;
            }
            "--out-jsonl" => {
                args.out_jsonl = value()?;
                i += 1;
            }
            "--save-read" => {
                args.save_read = value()?;
                i += 1;
            }
            other => return Err(invalid(format!("unknown flag {other}"))),
        }
        i += 1;
    }
    for (flag, path) in [
        ("--dump", &args.dump),
        ("--out-jsonl", &args.out_jsonl),
        ("--save-read", &args.save_read),
    ] {
        if path.is_empty() {
            return Err(invalid(format!("{flag} is required")));
        }
    }
    if args.steps > 0 && args.eval_at.iter().any(|&step| step > args.steps) {
        return Err(invalid("an eval step is past the step budget"));
    }
    Ok(args)
}

fn read_file(path: &str) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| invalid(format!("{path}: {e}")))
}

fn write_file(path: &str, bytes: &[u8]) -> Result<()> {
    if Path::new(path).exists() {
        return Err(invalid(format!("{path} exists; never written over")));
    }
    std::fs::write(path, bytes).map_err(|e| invalid(format!("{path}: {e}")))
}

/// A `u32` array from a path: one entry a row.
fn u32s(path: &str) -> Result<Vec<u32>> {
    let bytes = read_file(path)?;
    if !bytes.len().is_multiple_of(4) {
        return Err(invalid(format!("{path} is not a u32 array")));
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect())
}

/// The labels of the dump's rows: `bound` from one sidecar, `competing` from the other.
fn read_labels(path: &str, args: &Args, rows: usize) -> Result<BindLabels> {
    let ask = u32s(&format!("{path}.ask.u32"))?;
    let other = u32s(&format!("{path}.other.u32"))?;
    if ask.len() != rows || other.len() != rows {
        return Err(invalid(format!(
            "the targets hold {} / {} rows, the dump {rows}",
            ask.len(),
            other.len()
        )));
    }
    let (bound, competing) = if args.reverse {
        (&other, &ask)
    } else {
        (&ask, &other)
    };
    let mut labels = BindLabels::new(rows);
    for row in 0..rows {
        if bound[row] == 0 && competing[row] == 0 {
            continue;
        }
        let rivals: Vec<u32> = [competing[row]].into_iter().filter(|&p| p != 0).collect();
        labels.set(row, bound[row], &rivals)?;
    }
    Ok(labels)
}

/// The batch's slice of the labels, re-indexed from row 0.
fn batch_labels(labels: &BindLabels, rows: std::ops::Range<usize>) -> BindLabels {
    let mut batch = BindLabels::new(rows.len());
    for (at, row) in rows.enumerate() {
        batch.bound[at] = labels.bound[row];
        batch.competing[at] = labels.competing[row].clone();
    }
    batch
}

fn start_read(start: &str, dump: &Q31ReadDump) -> Result<Q31FirstPickRead> {
    let read = if start == "dump" {
        dump.read.clone()
    } else if start == "identity" {
        let gates = dump.read.width() / 4;
        Q31FirstPickRead::new(
            vec![IDENTITY_GATE; gates],
            vec![IDENTITY_GATE; gates],
            dump.read.weights().to_vec(),
            dump.read.gain_shift(),
        )?
    } else {
        Q31FirstPickRead::from_bytes(&read_file(start)?)?
    };
    if read.width() != dump.read.width() {
        return Err(invalid(format!(
            "the start read has width {}, the dump {}",
            read.width(),
            dump.read.width()
        )));
    }
    Ok(read)
}

/// The served picks' agreement with the two targets.
fn agreement(
    dump: &Q31ReadDump,
    read: &Q31FirstPickRead,
    labels: &BindLabels,
) -> Result<(usize, usize, usize)> {
    let picks = read
        .apply(&dump.stream, &dump.ids, &dump.table, dump.batch, dump.time)?
        .picks;
    let mut asked = 0;
    let mut other = 0;
    let mut rows = 0;
    for row in 0..picks.len() {
        if labels.bound[row] == 0 && labels.competing[row].is_empty() {
            continue;
        }
        rows += 1;
        if picks[row] == labels.bound[row] {
            asked += 1;
        }
        if labels.competing[row].contains(&picks[row]) {
            other += 1;
        }
    }
    Ok((asked, other, rows))
}

fn report_json(report: &BindReport) -> String {
    format!(
        "{{\"rows\":{},\"total\":\"{}\",\"margin_cost\":\"{}\",\"gap\":\"{}\",\
         \"misses\":{},\"bound_first\":{},\"comp_first\":{},\"satisfied\":{}}}",
        report.rows,
        report.total,
        report.margin_cost,
        report.gap,
        report.misses,
        report.bound_first,
        report.comp_first,
        report.satisfied
    )
}

fn run(args: &Args) -> Result<()> {
    let dump = Q31ReadDump::from_bytes(&read_file(&args.dump)?)?;
    let rows = dump.batch * dump.time;
    let labels = read_labels(&args.dump, args, rows)?;
    let eval = match &args.eval_dump {
        None => None,
        Some(path) => {
            let dump = Q31ReadDump::from_bytes(&read_file(path)?)?;
            let held_rows = dump.batch * dump.time;
            let held_labels = read_labels(path, args, held_rows)?;
            Some((dump, held_labels))
        }
    };
    let windows = dump.batch;
    if windows % args.batch != 0 {
        return Err(invalid("the batch must divide the dump's windows"));
    }
    let batches = windows / args.batch;
    let mut read = start_read(&args.start, &dump)?;
    if eval
        .as_ref()
        .is_some_and(|(held, _)| held.read.width() != read.width())
    {
        return Err(invalid(
            "the eval dump's read width is not the start read's",
        ));
    }
    let mut out = std::fs::File::create_new(&args.out_jsonl)
        .map_err(|e| invalid(format!("creating {}: {e}", args.out_jsonl)))?;
    let mut accepted = 0usize;
    for step in 0..=args.steps {
        if args.eval_at.contains(&step) {
            let (report, _) = read.bind_hinge(&dump.stream, &labels, dump.batch, dump.time)?;
            let (bound_picks, comp_picks, scored) = agreement(&dump, &read, &labels)?;
            let (held_bound, held_comp, held_scored, held_total, held_miss) = match &eval {
                None => (0, 0, 0, 0, 0),
                Some((held_dump, held_labels)) => {
                    let (held, _) = read.bind_hinge(
                        &held_dump.stream,
                        held_labels,
                        held_dump.batch,
                        held_dump.time,
                    )?;
                    let (b, c, s) = agreement(held_dump, &read, held_labels)?;
                    (b, c, s, held.total, held.misses)
                }
            };
            let line = format!(
                "{{\"step\":{step},\"step_shift\":{},\"accepted_steps\":{accepted},\
                 \"hinge\":{},\"bound_picks\":{bound_picks},\"comp_picks\":{comp_picks},\
                 \"scored\":{scored},\"held\":{{\"bound_picks\":{held_bound},\
                 \"comp_picks\":{held_comp},\"scored\":{held_scored},\
                 \"total\":\"{held_total}\",\"misses\":{held_miss}}}}}",
                args.step_shift,
                report_json(&report)
            );
            writeln!(out, "{line}").map_err(|e| invalid(format!("writing jsonl: {e}")))?;
        }
        if step == args.steps {
            break;
        }
        let first = (step % batches) * args.batch;
        let row_range = first * dump.time..(first + args.batch) * dump.time;
        let width = dump.read.width();
        let stream = &dump.stream[row_range.start * width..row_range.end * width];
        let batch = batch_labels(&labels, row_range);
        let result =
            read.train_bind_step(stream, &batch, args.batch, dump.time, args.step_shift)?;
        if result.accepted {
            accepted += 1;
        }
        read = result.read;
    }
    write_file(&args.save_read, &read.to_bytes())?;
    Ok(())
}

fn main() -> Result<()> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    run(&parse_args(&argv)?)
}
