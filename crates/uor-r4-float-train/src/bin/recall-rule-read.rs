//! Two-stage rule read: the native asked pick, then the stream's other-fact rule.
//!
//! Stage 1 is a saved native first pick (the asked fact, ~0.98). Stage 2 asks the
//! 397 rule: among the window's fact positions at the query's distance bucket,
//! excluding the asked position and the earlier slots' picks, take the consistent
//! candidate. On a `K = 1` row that candidate is the other fact by construction;
//! on `K >= 2` the stream does not determine which, so the tool takes the first.
//!
//! `usage: recall-rule-read --seed S --header H --windows N (--held | --train)
//!  --picks FILE --out FILE`
//!
//! A host count: no model and no device.

use std::fs;
use std::path::{Path, PathBuf};

use uor_r4_float_train::in_space_long::{
    recall_query_window, RecallQuery, RECALL_QUERY_BUCKETS,
};

const CONTEXT: usize = 192;

struct Args {
    seed: u64,
    header: usize,
    windows: usize,
    held: bool,
    picks: PathBuf,
    out: PathBuf,
}

fn parse(mut argv: impl Iterator<Item = String>) -> Result<Args, String> {
    let (mut seed, mut header, mut windows) = (None, None, None);
    let (mut held, mut train) = (false, false);
    let (mut picks, mut out) = (None, None);
    while let Some(flag) = argv.next() {
        let mut value = || argv.next().ok_or_else(|| format!("{flag} needs a value"));
        let number = |v: String| v.parse::<u64>().map_err(|e| format!("{flag} {v}: {e}"));
        match flag.as_str() {
            "--seed" => seed = Some(number(value()?)?),
            "--header" => header = Some(number(value()?)? as usize),
            "--windows" => windows = Some(number(value()?)? as usize),
            "--held" => held = true,
            "--train" => train = true,
            "--picks" => picks = Some(PathBuf::from(value()?)),
            "--out" => out = Some(PathBuf::from(value()?)),
            other => return Err(format!("unknown flag {other}")),
        }
    }
    if held == train {
        return Err("give exactly one of --held and --train".to_owned());
    }
    Ok(Args {
        seed: seed.ok_or("--seed is required")?,
        header: header.ok_or("--header is required")?,
        windows: windows.ok_or("--windows is required")?,
        held,
        picks: picks.ok_or("--picks is required")?,
        out: out.ok_or("--out is required")?,
    })
}

fn u32s(path: &Path) -> Result<Vec<u32>, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !bytes.len().is_multiple_of(4) {
        return Err(format!("{}: not a u32 array", path.display()));
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect())
}

#[derive(Clone, Default)]
struct Bucket {
    rows: u64,
    k1_rows: u64,
    k1_other: u64,
    all_other: u64,
    asked_stage1_right: u64,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("recall-rule-read: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse(std::env::args().skip(1))?;
    if args.out.exists() {
        return Err(format!("{} exists; never written over", args.out.display()));
    }
    let seed = if args.held { args.seed ^ 0x5EED_F00D } else { args.seed };
    let rows = args.windows * CONTEXT;
    let picks = u32s(&args.picks)?;
    if picks.len() != rows {
        return Err(format!(
            "{}: {} picks, the windows have {rows} rows",
            args.picks.display(),
            picks.len()
        ));
    }
    let mut buckets = vec![Bucket::default(); RECALL_QUERY_BUCKETS.len()];
    for w in 0..args.windows {
        let (_, queries) = recall_query_window(seed, w as u64, CONTEXT, args.header)
            .map_err(|e| format!("window {w}: {e}"))?;
        let row = |q: &RecallQuery| w * CONTEXT + q.scored_offset;
        let mut facts: Vec<usize> = queries
            .iter()
            .flat_map(|q| [q.ask_value_pos, q.other_value_pos])
            .collect();
        facts.sort_unstable();
        for q in &queries {
            let asked = picks[row(q)] as usize;
            let (low, high) = RECALL_QUERY_BUCKETS[q.bucket];
            if asked == 0 || asked >= q.query_pos {
                continue;
            }
            let earlier: Vec<usize> = queries
                .iter()
                .filter(|e| e.slot < q.slot)
                .map(|e| picks[row(e)] as usize)
                .collect();
            let candidates: Vec<usize> = facts
                .iter()
                .copied()
                .filter(|&p| {
                    p != asked
                        && p < q.query_pos
                        && (low..=high).contains(&(q.query_pos - p))
                        && !earlier.contains(&p)
                })
                .collect();
            if candidates.is_empty() {
                continue;
            }
            let b = &mut buckets[q.bucket];
            b.rows += 1;
            b.asked_stage1_right += u64::from(asked == q.ask_value_pos);
            let pick = candidates[0];
            let hit = pick == q.other_value_pos;
            b.all_other += u64::from(hit);
            if candidates.len() == 1 {
                b.k1_rows += 1;
                b.k1_other += u64::from(hit);
            }
        }
    }
    let report = serde_json::json!({
        "seed": args.seed,
        "held": args.held,
        "windows": args.windows,
        "buckets": buckets.iter().enumerate().map(|(i, b)| serde_json::json!({
            "bucket": i,
            "rows": b.rows,
            "k1_rows": b.k1_rows,
            "k1_other": b.k1_other,
            "all_other": b.all_other,
            "stage1_asked_right": b.asked_stage1_right,
        })).collect::<Vec<_>>(),
    });
    let bytes = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    fs::write(&args.out, bytes).map_err(|e| format!("{}: {e}", args.out.display()))?;
    Ok(())
}
