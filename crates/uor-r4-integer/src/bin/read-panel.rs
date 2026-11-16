//! The C2′ panel through the integer artifact: the learned `/14` read's picks against the float
//! read and the targets, and the `/15` relation read's picks.
//!
//! `usage: read-panel --artifact FILE --dump DUMP [--float-picks FILE] --out FILE`
//!
//! The dump is the probe's held `Q31ReadDump` (its `ids` are the windows' tokens); the target
//! files `.ask.u32`, `.other.u32` and `.bucket.u8` sit beside it. The Q31 read's step is traced
//! (`set_q31_read_trace`); the relation read's pick is `last_relation_pick`. A host run; no GPU.

use std::fs;
use std::path::{Path, PathBuf};

use uor_r4_integer::stack::{IntegerStackModel, RelationRead};
use uor_r4_native_vector::Q31ReadDump;

const HEADER: usize = 4;

struct Args {
    artifact: PathBuf,
    dump: PathBuf,
    float_picks: Option<PathBuf>,
    out: PathBuf,
}

fn parse(mut argv: impl Iterator<Item = String>) -> Result<Args, String> {
    let (mut artifact, mut dump, mut float_picks, mut out) = (None, None, None, None);
    while let Some(flag) = argv.next() {
        let mut value = || argv.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--artifact" => artifact = Some(PathBuf::from(value()?)),
            "--dump" => dump = Some(PathBuf::from(value()?)),
            "--float-picks" => float_picks = Some(PathBuf::from(value()?)),
            "--out" => out = Some(PathBuf::from(value()?)),
            other => return Err(format!("unknown flag {other}")),
        }
    }
    Ok(Args {
        artifact: artifact.ok_or("--artifact is required")?,
        dump: dump.ok_or("--dump is required")?,
        float_picks,
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

fn u8s(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}

#[derive(Default, Clone)]
struct Counts {
    scored: u64,
    q31_ask: u64,
    q31_other: u64,
    q31_k1: u64,
    q31_k1_other: u64,
    relation_other: u64,
    relation_k1: u64,
    relation_k1_other: u64,
    float_agree: u64,
    float_rows: u64,
    x_rows: u64,
    x_sign_agree: u64,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("read-panel: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse(std::env::args().skip(1))?;
    if args.out.exists() {
        return Err(format!("{} exists; never written over", args.out.display()));
    }
    let dump_bytes = fs::read(&args.dump).map_err(|e| format!("{}: {e}", args.dump.display()))?;
    let dump = Q31ReadDump::from_bytes(&dump_bytes).map_err(|e| e.to_string())?;
    let ask = u32s(&PathBuf::from(format!("{}.ask.u32", args.dump.display())))?;
    let other = u32s(&PathBuf::from(format!("{}.other.u32", args.dump.display())))?;
    let bucket = u8s(&PathBuf::from(format!("{}.bucket.u8", args.dump.display())))?;
    let rows = dump.batch * dump.time;
    let width = dump.read.width();
    if ask.len() != rows || other.len() != rows || bucket.len() != rows {
        return Err(format!(
            "the targets hold {} / {} / {rows} rows",
            ask.len(),
            other.len()
        ));
    }
    let float = match &args.float_picks {
        Some(path) => {
            let picks = u32s(path)?;
            if picks.len() != rows {
                return Err(format!("{}: {} picks for {rows} rows", path.display(), picks.len()));
            }
            Some(picks)
        }
        None => None,
    };
    let artifact = fs::read(&args.artifact).map_err(|e| format!("{}: {e}", args.artifact.display()))?;
    let model = IntegerStackModel::parse(&artifact).map_err(|e| e.to_string())?;
    let mut buckets = vec![Counts::default(); 2];
    let mut all = Counts::default();
    let mut ratios: Vec<f64> = Vec::new();
    for w in 0..dump.batch {
        let tokens = &dump.ids[w * dump.time..(w + 1) * dump.time];
        let mut session = model.session();
        let traced = session.set_q31_read_trace(true).is_ok();
        let mut earlier: Vec<usize> = Vec::new();
        for (i, &token) in tokens.iter().enumerate() {
            session.step_native(token).map_err(|e| e.to_string())?;
            let row = w * dump.time + i;
            if ask[row] == 0 && other[row] == 0 {
                continue;
            }
            let Some(query_pos) = i.checked_sub(HEADER) else {
                continue;
            };
            let k = RelationRead::PROBE.candidate_count(&tokens[..=i], query_pos, &earlier);
            let b = usize::from(bucket[row].saturating_sub(1)).min(1);
            let q31 = if traced {
                session
                    .q31_read_trace()
                    .and_then(|trace| trace.last())
                    .map(|step| step.pick)
                    .unwrap_or(0)
            } else {
                0
            };
            let relation = session.last_relation_pick().map_or(0, |pick| pick as u32);
            if traced {
                if let Some(step) = session.q31_read_trace().and_then(|t| t.last()) {
                    let stream = &dump.stream[row * width..(row + 1) * width];
                    for (&a, &b) in step.x_in.iter().zip(stream) {
                        if a != 0 && b != 0 {
                            all.x_rows += 1;
                            if (a > 0) == (b > 0) {
                                all.x_sign_agree += 1;
                            }
                            ratios.push(a as f64 / b as f64);
                        }
                    }
                }
            }
            let c = &mut buckets[b];
            c.scored += 1;
            all.scored += 1;
            let q31_ask = q31 != 0 && q31 == ask[row];
            let q31_other = q31 != 0 && q31 == other[row];
            c.q31_ask += u64::from(q31_ask);
            all.q31_ask += u64::from(q31_ask);
            c.q31_other += u64::from(q31_other);
            all.q31_other += u64::from(q31_other);
            if k == Some(1) {
                c.q31_k1 += 1;
                all.q31_k1 += 1;
                c.q31_k1_other += u64::from(q31_other);
                all.q31_k1_other += u64::from(q31_other);
                c.relation_k1 += 1;
                all.relation_k1 += 1;
                let hit = relation != 0 && relation == other[row];
                c.relation_k1_other += u64::from(hit);
                all.relation_k1_other += u64::from(hit);
            }
            let hit = relation != 0 && relation == other[row];
            c.relation_other += u64::from(hit);
            all.relation_other += u64::from(hit);
            if let Some(float) = &float {
                if float[row] != 0 {
                    c.float_rows += 1;
                    all.float_rows += 1;
                    let agree = q31 == float[row];
                    c.float_agree += u64::from(agree);
                    all.float_agree += u64::from(agree);
                }
            }
            earlier.push(ask[row] as usize);
        }
    }
    let report = serde_json::json!({
        "artifact": args.artifact.display().to_string(),
        "windows": dump.batch,
        "all": counts_json(&all),
        "x_ratio_median": median(&mut ratios),
        "buckets": buckets.iter().enumerate().map(|(i, c)| serde_json::json!({
            "bucket": i,
            "counts": counts_json(c),
        })).collect::<Vec<_>>(),
    });
    let bytes = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    fs::write(&args.out, bytes).map_err(|e| format!("{}: {e}", args.out.display()))?;
    Ok(())
}

fn counts_json(c: &Counts) -> serde_json::Value {
    serde_json::json!({
        "scored": c.scored,
        "q31_ask": c.q31_ask,
        "q31_other": c.q31_other,
        "q31_k1": c.q31_k1,
        "q31_k1_other": c.q31_k1_other,
        "relation_other": c.relation_other,
        "relation_k1": c.relation_k1,
        "relation_k1_other": c.relation_k1_other,
        "float_rows": c.float_rows,
        "float_agree": c.float_agree,
        "x_rows": c.x_rows,
        "x_sign_agree": c.x_sign_agree,
    })
}

fn median(values: &mut [f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    values[values.len() / 2]
}
