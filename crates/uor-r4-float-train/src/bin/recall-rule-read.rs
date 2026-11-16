//! Two-stage rule read, derived from the stream.
//!
//! Stage 1 is not a saved label: the stream writes each fact as `K v` before the
//! tails, and the query token is `Q(k) = k + RECALL_QUERY_SHIFT`. The scan finds
//! the asked fact by matching the query's relation against the fact relations.
//! Stage 2 is the 397 rule: among the facts at the query's distance bucket,
//! excluding the asked fact and the earlier slots' asked facts, take the
//! consistent candidate. On a `K = 1` row that candidate is the other fact.
//!
//! `usage: recall-rule-read --seed S --header H --windows N (--held | --train)
//!  [--picks FILE] --out FILE`
//!
//! A host count: no model and no device.

use std::fs;
use std::path::{Path, PathBuf};

use uor_r4_float_train::in_space_long::{
    recall_query_window, RecallQuery, RECALL_QUERY_BUCKETS, RECALL_QUERY_SHIFT, RECALL_RELATIONS,
    RECALL_VALUES,
};

const CONTEXT: usize = 192;

struct Args {
    seed: u64,
    header: usize,
    windows: usize,
    held: bool,
    picks: Option<PathBuf>,
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
        picks,
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
struct Previous {
    rows: u64,
    scan_found: u64,
    asked_pick: u64,
    picks_seen: u64,
}

#[derive(Clone, Default)]
struct Bucket {
    rows: u64,
    k1_rows: u64,
    k1_other: u64,
    all_other: u64,
    asked_scan_right: u64,
    scan_matches_pick: u64,
    picks_seen: u64,
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
    let picks = match &args.picks {
        Some(path) => {
            let picks = u32s(path)?;
            if picks.len() != rows {
                return Err(format!(
                    "{}: {} picks, the windows have {rows} rows",
                    path.display(),
                    picks.len()
                ));
            }
            Some(picks)
        }
        None => None,
    };
    let mut buckets = vec![Bucket::default(); RECALL_QUERY_BUCKETS.len()];
    let mut previous = Previous::default();
    for w in 0..args.windows {
        let (window, queries) = recall_query_window(seed, w as u64, CONTEXT, args.header)
            .map_err(|e| format!("window {w}: {e}"))?;
        let ids = &window.input_ids;
        // Every fact is `relation value` before the tails: relation i at p-1, value at p.
        let facts: Vec<usize> = (1..ids.len())
            .filter(|&p| {
                RECALL_RELATIONS.contains(&ids[p - 1]) && RECALL_VALUES.contains(&ids[p])
            })
            .collect();
        let asked_of = |q: &RecallQuery| -> Option<(usize, usize)> {
            let relation = ids[q.query_pos].checked_sub(RECALL_QUERY_SHIFT)?;
            if !RECALL_RELATIONS.contains(&relation) {
                return None;
            }
            let position = *facts
                .iter()
                .find(|&&p| ids[p - 1] == relation && p < q.query_pos)?;
            let distance = q.query_pos - position;
            let bucket = RECALL_QUERY_BUCKETS
                .iter()
                .position(|&(low, high)| (low..=high).contains(&distance))?;
            Some((position, bucket))
        };
        for q in &queries {
            let Some((asked, bucket)) = asked_of(q) else {
                continue;
            };
            let (low, high) = RECALL_QUERY_BUCKETS[bucket];
            let earlier: Vec<usize> = queries
                .iter()
                .filter(|e| e.slot < q.slot)
                .filter_map(asked_of)
                .map(|(position, _)| position)
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
            if let Some(prev) = queries.iter().find(|e| e.slot + 1 == q.slot) {
                let prev_relation = ids[prev.query_pos].checked_sub(RECALL_QUERY_SHIFT);
                let prev_fact = prev_relation.and_then(|relation| {
                    facts
                        .iter()
                        .copied()
                        .find(|&p| ids[p - 1] == relation && p < prev.query_pos)
                });
                previous.rows += 1;
                previous.scan_found += u64::from(prev_fact == Some(prev.ask_value_pos));
                if let Some(picks) = &picks {
                    let pick = picks[w * CONTEXT + q.scored_offset] as usize;
                    if pick != 0 {
                        previous.picks_seen += 1;
                        previous.asked_pick += u64::from(pick == prev.ask_value_pos);
                    }
                }
            }
            let b = &mut buckets[bucket];
            b.rows += 1;
            b.asked_scan_right += u64::from(asked == q.ask_value_pos);
            if let Some(picks) = &picks {
                let pick = picks[w * CONTEXT + q.scored_offset] as usize;
                if pick != 0 {
                    b.picks_seen += 1;
                    b.scan_matches_pick += u64::from(pick == asked);
                }
            }
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
        "previous": {
            "rows": previous.rows,
            "scan_found": previous.scan_found,
            "asked_pick": previous.asked_pick,
            "picks_seen": previous.picks_seen,
        },
        "buckets": buckets.iter().enumerate().map(|(i, b)| serde_json::json!({
            "bucket": i,
            "rows": b.rows,
            "k1_rows": b.k1_rows,
            "k1_other": b.k1_other,
            "all_other": b.all_other,
            "asked_scan_right": b.asked_scan_right,
            "scan_matches_pick": b.scan_matches_pick,
            "picks_seen": b.picks_seen,
        })).collect::<Vec<_>>(),
    });
    let bytes = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    fs::write(&args.out, bytes).map_err(|e| format!("{}: {e}", args.out.display()))?;
    Ok(())
}
