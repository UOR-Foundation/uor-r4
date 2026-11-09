//! Audit one real carried artifact against its exact ask-preserving `−C` control.
//!
//! This is an instrument only. It checks artifact identity and ask inventory, reproduces the
//! carry cut, and compares CPU and integer panel reports before and after the cut. Optional
//! teacher-forced/paired-first reports expose whether the carry changes actual ranks.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_integer::stack::{
    drop_hamming_carry_uorlut01, read_hamming_uorlut01, HammingImage, IntegerStackModel,
};

const SCHEMA: &str = "uor-r4.real-artifact-audit/1";
const USAGE: &str = "usage: real-artifact-audit --carried ARTIFACT --minus-c ARTIFACT \
--carried-cpu REPORT --carried-integer REPORT --minus-c-cpu REPORT --minus-c-integer REPORT \
[--carried-rank REPORT --minus-c-rank REPORT] --out NEW_FILE";

type Result<T> = std::result::Result<T, String>;

fn main() {
    if let Err(error) = run() {
        eprintln!("real-artifact-audit: {error}");
        std::process::exit(1);
    }
}

#[derive(Debug)]
struct Args {
    carried: String,
    minus_c: String,
    carried_cpu: String,
    carried_integer: String,
    minus_c_cpu: String,
    minus_c_integer: String,
    carried_rank: Option<String>,
    minus_c_rank: Option<String>,
    out: String,
}

fn run() -> Result<()> {
    let args = parse_args(std::env::args().skip(1))?;
    let carried = fs::read(&args.carried).map_err(|e| format!("{}: {e}", args.carried))?;
    let minus_c = fs::read(&args.minus_c).map_err(|e| format!("{}: {e}", args.minus_c))?;
    let artifacts = audit_artifacts(&carried, &minus_c)?;

    let carried_cpu = read_json("--carried-cpu", &args.carried_cpu)?;
    let carried_integer = read_json("--carried-integer", &args.carried_integer)?;
    let minus_c_cpu = read_json("--minus-c-cpu", &args.minus_c_cpu)?;
    let minus_c_integer = read_json("--minus-c-integer", &args.minus_c_integer)?;
    require_report_parity("carried CPU/integer", &carried_cpu, &carried_integer)?;
    require_report_parity("−C CPU/integer", &minus_c_cpu, &minus_c_integer)?;
    let generation = generation_delta(&carried_cpu, &minus_c_cpu)?;

    let ranks = match (&args.carried_rank, &args.minus_c_rank) {
        (Some(carried), Some(minus_c)) => Some(rank_delta(
            &read_json("--carried-rank", carried)?,
            &read_json("--minus-c-rank", minus_c)?,
        )?),
        (None, None) => None,
        _ => return Err("--carried-rank and --minus-c-rank must be given together".to_owned()),
    };

    let report = json!({
        "schema": SCHEMA,
        "verdict": "instrument-parity",
        "artifacts": artifacts,
        "reports": {
            "carried_cpu_equals_integer": true,
            "minus_c_cpu_equals_integer": true,
            "generation": generation,
        },
        "ranks": ranks,
    });
    let mut bytes = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    write_new(Path::new(&args.out), &bytes)?;
    println!(
        "{}",
        json!({
            "schema": SCHEMA,
            "out": args.out,
            "sha256": sha256(&bytes),
            "carry_changes_generation": report["reports"]["generation"]["changed_rows"].as_u64().unwrap_or(0) > 0,
            "carry_changes_ranks": report["ranks"]["changed_by_cut"].as_bool().unwrap_or(false),
        })
    );
    Ok(())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Args> {
    const FLAGS: [&str; 9] = [
        "carried",
        "minus-c",
        "carried-cpu",
        "carried-integer",
        "minus-c-cpu",
        "minus-c-integer",
        "carried-rank",
        "minus-c-rank",
        "out",
    ];
    let mut values = BTreeMap::new();
    let mut args = args;
    while let Some(arg) = args.next() {
        let name = arg
            .strip_prefix("--")
            .filter(|name| FLAGS.contains(name))
            .ok_or(format!("unknown argument {arg}\n{USAGE}"))?;
        let value = args.next().ok_or(format!("{arg} needs a value\n{USAGE}"))?;
        if values.insert(name.to_owned(), value).is_some() {
            return Err(format!("{arg} given twice"));
        }
    }
    let carried = take_required(&mut values, "carried")?;
    let minus_c = take_required(&mut values, "minus-c")?;
    let carried_cpu = take_required(&mut values, "carried-cpu")?;
    let carried_integer = take_required(&mut values, "carried-integer")?;
    let minus_c_cpu = take_required(&mut values, "minus-c-cpu")?;
    let minus_c_integer = take_required(&mut values, "minus-c-integer")?;
    let carried_rank = values.remove("carried-rank");
    let minus_c_rank = values.remove("minus-c-rank");
    let out = take_required(&mut values, "out")?;
    Ok(Args {
        carried,
        minus_c,
        carried_cpu,
        carried_integer,
        minus_c_cpu,
        minus_c_integer,
        carried_rank,
        minus_c_rank,
        out,
    })
}

fn take_required(values: &mut BTreeMap<String, String>, name: &str) -> Result<String> {
    values
        .remove(name)
        .ok_or_else(|| format!("--{name} is required\n{USAGE}"))
}

fn audit_artifacts(carried: &[u8], minus_c: &[u8]) -> Result<Value> {
    let carried_model =
        IntegerStackModel::parse(carried).map_err(|e| format!("carried artifact: {e}"))?;
    let minus_c_model =
        IntegerStackModel::parse(minus_c).map_err(|e| format!("minus-C artifact: {e}"))?;
    if carried_model.shape() != minus_c_model.shape() {
        return Err("carried and minus-C artifact shapes differ".to_owned());
    }
    let carried_image = read_hamming_uorlut01(carried)
        .map_err(|e| format!("carried Hamming image: {e}"))?
        .ok_or("carried artifact has no Hamming image")?;
    let minus_c_image = read_hamming_uorlut01(minus_c)
        .map_err(|e| format!("minus-C Hamming image: {e}"))?
        .ok_or("minus-C artifact has no Hamming image")?;
    validate_cut(&carried_image, &minus_c_image)?;

    let reproduced =
        drop_hamming_carry_uorlut01(carried).map_err(|e| format!("carry reproduction: {e}"))?;
    if reproduced != minus_c {
        return Err(format!(
            "published minus-C differs from the reproduced cut: published {} bytes {}, reproduced {} bytes {}",
            minus_c.len(),
            sha256(minus_c),
            reproduced.len(),
            sha256(&reproduced)
        ));
    }

    let last = carried_image
        .reads
        .last()
        .ok_or("carried artifact has no read")?;
    let (carry_rows, gains) = last
        .ask
        .as_ref()
        .and_then(|ask| ask.carry.as_ref())
        .ok_or("carried artifact has no readout carry")?;
    Ok(json!({
        "carried": {
            "bytes": carried.len(),
            "sha256": sha256(carried),
            "shape": carried_model.shape(),
        },
        "minus_c": {
            "bytes": minus_c.len(),
            "sha256": sha256(minus_c),
        },
        "hamming": {
            "depths": carried_image.record.depths,
            "asks": carried_image.record.asks,
            "carry_rows": carry_rows,
            "carry_depth": last.depth,
            "carry_gains": gains.len(),
            "gain_stats": gain_stats(gains),
            "all_ask_tables_preserved": true,
        },
        "reproduced_minus_c_byte_equal": true,
    }))
}

fn gain_stats(gains: &[i32]) -> Value {
    let min = gains.iter().min().copied();
    let max = gains.iter().max().copied();
    let sum: i128 = gains.iter().map(|&gain| i128::from(gain)).sum();
    let positive = gains.iter().filter(|&&gain| gain > 0).count();
    let negative = gains.iter().filter(|&&gain| gain < 0).count();
    let zero = gains.iter().filter(|&&gain| gain == 0).count();
    json!({
        "min": min,
        "max": max,
        "sum": sum.to_string(),
        "positive": positive,
        "negative": negative,
        "zero": zero,
    })
}

fn validate_cut(carried: &HammingImage, minus_c: &HammingImage) -> Result<()> {
    if carried.record.depths != minus_c.record.depths {
        return Err("carried and minus-C depths differ".to_owned());
    }
    if carried.record.asks != minus_c.record.asks {
        return Err("carried and minus-C ask flags differ".to_owned());
    }
    if carried.record.carry_rows.is_none() || minus_c.record.carry_rows.is_some() {
        return Err("the control does not remove exactly the recorded carry".to_owned());
    }
    if carried.reads.len() != minus_c.reads.len() {
        return Err("carried and minus-C read counts differ".to_owned());
    }
    for (index, (carried_read, minus_read)) in carried.reads.iter().zip(&minus_c.reads).enumerate()
    {
        if carried_read.depth != minus_read.depth {
            return Err(format!("read {index}: depths differ"));
        }
        let (Some(carried_ask), Some(minus_ask)) =
            (carried_read.ask.as_ref(), minus_read.ask.as_ref())
        else {
            if carried_read.ask.is_none() && minus_read.ask.is_none() {
                continue;
            }
            return Err(format!("read {index}: ask inventory differs"));
        };
        if carried_ask.table != minus_ask.table {
            return Err(format!("read {index}: ask table differs"));
        }
        if minus_ask.carry.is_some() {
            return Err(format!("read {index}: minus-C retains a carry"));
        }
        if index + 1 != carried.reads.len() && carried_ask.carry.is_some() {
            return Err(format!("read {index}: a non-readout read carries"));
        }
    }
    Ok(())
}

fn read_json(label: &str, path: &str) -> Result<Value> {
    let bytes = fs::read(path).map_err(|e| format!("{label} {path}: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{label} {path}: {e}"))
}

fn normalize_report(report: &Value) -> Value {
    let mut report = report.clone();
    if let Some(object) = report.as_object_mut() {
        for key in [
            "wall_milliseconds",
            "decode_milliseconds",
            "tokens_per_second",
            "decode_tokens_per_second",
        ] {
            object.remove(key);
        }
        if let Some(settings) = object.get_mut("settings").and_then(Value::as_object_mut) {
            settings.remove("engine");
        }
        for key in ["teacher_forced", "paired_first"] {
            if let Some(section) = object.get_mut(key).and_then(Value::as_object_mut) {
                section.remove("wall_milliseconds");
            }
        }
    }
    report
}

fn require_report_parity(label: &str, cpu: &Value, integer: &Value) -> Result<()> {
    if normalize_report(cpu) != normalize_report(integer) {
        return Err(format!(
            "{label} reports differ beyond timing and engine identity"
        ));
    }
    Ok(())
}

fn rows_by_id(report: &Value, section: Option<&str>) -> Result<BTreeMap<String, Value>> {
    let root = section.and_then(|name| report.get(name)).unwrap_or(report);
    let rows = root
        .get("rows")
        .and_then(Value::as_array)
        .ok_or("report has no rows array")?;
    let mut by_id = BTreeMap::new();
    for row in rows {
        let id = row
            .get("id")
            .and_then(Value::as_str)
            .ok_or("report row has no id")?;
        if by_id.insert(id.to_owned(), row.clone()).is_some() {
            return Err(format!("report repeats row {id}"));
        }
    }
    Ok(by_id)
}

fn require_same_ids(
    label: &str,
    left: &BTreeMap<String, Value>,
    right: &BTreeMap<String, Value>,
) -> Result<()> {
    let left_ids: BTreeSet<_> = left.keys().collect();
    let right_ids: BTreeSet<_> = right.keys().collect();
    if left_ids != right_ids {
        return Err(format!("{label} reports have different case inventories"));
    }
    Ok(())
}

fn generation_delta(carried: &Value, minus_c: &Value) -> Result<Value> {
    let carried = rows_by_id(carried, None)?;
    let minus_c = rows_by_id(minus_c, None)?;
    require_same_ids("generation", &carried, &minus_c)?;
    let mut changed_rows = 0usize;
    let mut changed_tokens = 0usize;
    let mut first_divergence = Value::Null;
    for (id, carried_row) in &carried {
        let minus_row = &minus_c[id];
        let same = ["reply_ids", "reply_text", "stop", "complete"]
            .into_iter()
            .all(|field| carried_row.get(field) == minus_row.get(field));
        if same {
            continue;
        }
        changed_rows += 1;
        let carried_ids = carried_row
            .get("reply_ids")
            .and_then(Value::as_array)
            .ok_or(format!("{id}: carried reply_ids missing"))?;
        let minus_ids = minus_row
            .get("reply_ids")
            .and_then(Value::as_array)
            .ok_or(format!("{id}: minus-C reply_ids missing"))?;
        let length = carried_ids.len().max(minus_ids.len());
        for index in 0..length {
            let carried_token = carried_ids.get(index).cloned().unwrap_or(Value::Null);
            let minus_token = minus_ids.get(index).cloned().unwrap_or(Value::Null);
            if carried_token != minus_token {
                changed_tokens += 1;
                if first_divergence.is_null() {
                    first_divergence = json!({
                        "id": id,
                        "index": index,
                        "carried_token": carried_token,
                        "minus_c_token": minus_token,
                    });
                }
                break;
            }
        }
    }
    Ok(json!({
        "rows": carried.len(),
        "changed_rows": changed_rows,
        "changed_token_rows": changed_tokens,
        "first_divergence": first_divergence,
    }))
}

fn rank_delta(carried: &Value, minus_c: &Value) -> Result<Value> {
    let carried_teacher = rows_by_id(carried, Some("teacher_forced"))?;
    let minus_teacher = rows_by_id(minus_c, Some("teacher_forced"))?;
    require_same_ids("teacher-forced", &carried_teacher, &minus_teacher)?;
    let mut teacher_changed = Vec::new();
    let mut first_target_changed = Vec::new();
    for (id, carried_row) in &carried_teacher {
        let minus_row = &minus_teacher[id];
        if [
            "target_ranks",
            "raw_target_ranks",
            "chosen_token",
            "raw_nearest_token",
        ]
        .into_iter()
        .any(|field| carried_row.get(field) != minus_row.get(field))
        {
            teacher_changed.push(id.clone());
        }
        let first_rank =
            |row: &Value, field: &str| row[field].get(0).cloned().unwrap_or(Value::Null);
        if first_rank(carried_row, "target_ranks") != first_rank(minus_row, "target_ranks")
            || first_rank(carried_row, "raw_target_ranks")
                != first_rank(minus_row, "raw_target_ranks")
        {
            first_target_changed.push(id.clone());
        }
    }

    let carried_paired = rows_by_id(carried, Some("paired_first"))?;
    let minus_paired = rows_by_id(minus_c, Some("paired_first"))?;
    require_same_ids("paired-first", &carried_paired, &minus_paired)?;
    let mut paired_changed = Vec::new();
    for (id, carried_row) in &carried_paired {
        let minus_row = &minus_paired[id];
        if [
            "asked_rank",
            "other_rank",
            "raw_asked_rank",
            "raw_other_rank",
        ]
        .into_iter()
        .any(|field| carried_row.get(field) != minus_row.get(field))
        {
            paired_changed.push(id.clone());
        }
    }

    let first = |report: &Value| {
        let target = &report["teacher_forced"]["first_target"];
        json!({
            "top1": target["top1"],
            "top8": target["top8"],
            "top32": target["top32"],
            "rank_p50": target["rank_p50"],
        })
    };
    Ok(json!({
        "changed_by_cut": !teacher_changed.is_empty() || !paired_changed.is_empty(),
        "teacher_forced": {
            "carried": first(carried),
            "minus_c": first(minus_c),
            "changed_rows": teacher_changed.len(),
            "first_target_changed_rows": first_target_changed.len(),
            "first_changed_id": teacher_changed.first(),
        },
        "paired_first": {
            "carried": carried["paired_first"]["totals"].clone(),
            "minus_c": minus_c["paired_first"]["totals"].clone(),
            "changed_rows": paired_changed.len(),
            "first_changed_id": paired_changed.first(),
        },
    }))
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    file.write_all(bytes)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    file.sync_all()
        .map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::stack::synthetic::{
        synthetic_fbs_trunk_with, synthetic_hamming, FBS_SYNTH_NON_SQUARE,
    };
    use uor_r4_integer::stack::{add_hamming_ask_uorlut01, export_fbs_trunk_uorlut01, AskImage};

    fn artifacts() -> (Vec<u8>, Vec<u8>) {
        let mut snapshot = synthetic_fbs_trunk_with(264, FBS_SYNTH_NON_SQUARE);
        snapshot.readout_nearest = true;
        let (vocab, width, layers) = (snapshot.vocab, snapshot.width, snapshot.layers.len());
        snapshot.hamming = Some(synthetic_hamming(width, layers, 4, 264));
        let plain = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let ask = |carry: Option<usize>| {
            Some(AskImage {
                table: (0..vocab * width)
                    .map(|index| ((index * 37 + 11) % 4096) as i16 - 2048)
                    .collect(),
                carry: carry.map(|reach| {
                    (
                        reach,
                        (0..vocab)
                            .map(|token| (token as i32 * 17) % 65_536 - 32_768)
                            .collect(),
                    )
                }),
            })
        };
        let carried = add_hamming_ask_uorlut01(&plain, &[ask(None), ask(Some(4))]).unwrap();
        let minus_c = drop_hamming_carry_uorlut01(&carried).unwrap();
        (carried, minus_c)
    }

    #[test]
    fn real_cut_audit_accepts_only_the_reproduced_control() {
        let (carried, minus_c) = artifacts();
        let report = audit_artifacts(&carried, &minus_c).unwrap();
        assert_eq!(report["hamming"]["carry_rows"], 4);
        assert_eq!(report["hamming"]["carry_gains"], 512);

        let mut changed = minus_c;
        let last = changed.len() - 1;
        changed[last] ^= 1;
        assert!(audit_artifacts(&carried, &changed).is_err());
    }

    #[test]
    fn report_parity_ignores_only_timing_and_engine() {
        let cpu = json!({
            "settings": {"engine": "integer", "reply_tokens": 64},
            "wall_milliseconds": 10,
            "decode_milliseconds": 9,
            "tokens_per_second": 1,
            "decode_tokens_per_second": 2,
            "rows": [],
        });
        let mut integer = cpu.clone();
        integer["settings"]["engine"] = json!("integer");
        integer["wall_milliseconds"] = json!(99);
        integer["decode_milliseconds"] = json!(88);
        integer["tokens_per_second"] = json!(3);
        integer["decode_tokens_per_second"] = json!(4);
        integer["settings"]
            .as_object_mut()
            .unwrap()
            .remove("engine");
        assert_eq!(normalize_report(&cpu), normalize_report(&integer));
    }

    #[test]
    fn generation_delta_follows_case_ids_and_finds_the_first_token() {
        let carried = json!({"rows": [
            {"id": "a", "reply_ids": [7, 8], "reply_text": "x", "stop": "cap", "complete": false},
            {"id": "b", "reply_ids": [9], "reply_text": "y", "stop": "eos", "complete": false}
        ]});
        let minus_c = json!({"rows": [
            {"id": "b", "reply_ids": [9], "reply_text": "y", "stop": "eos", "complete": false},
            {"id": "a", "reply_ids": [7, 3], "reply_text": "z", "stop": "cap", "complete": false}
        ]});
        let delta = generation_delta(&carried, &minus_c).unwrap();
        assert_eq!(delta["rows"], 2);
        assert_eq!(delta["changed_rows"], 1);
        assert_eq!(delta["first_divergence"]["id"], "a");
        assert_eq!(delta["first_divergence"]["index"], 1);
    }

    #[test]
    fn rank_delta_detects_a_real_case_change_and_ignores_row_order() {
        let carried = json!({
            "teacher_forced": {
                "first_target": {"top1": 0, "top8": 2, "top32": 3, "rank_p50": 10},
                "rows": [{"id": "a", "target_ranks": [3], "raw_target_ranks": [3], "chosen_token": 4, "raw_nearest_token": 4}]
            },
            "paired_first": {
                "totals": {"asked_better": 1},
                "rows": [{"id": "a", "asked_rank": 3, "other_rank": 5, "raw_asked_rank": 3, "raw_other_rank": 5}]
            }
        });
        let minus_c = json!({
            "teacher_forced": {
                "first_target": {"top1": 0, "top8": 1, "top32": 3, "rank_p50": 12},
                "rows": [{"id": "a", "target_ranks": [5], "raw_target_ranks": [5], "chosen_token": 4, "raw_nearest_token": 4}]
            },
            "paired_first": {
                "totals": {"asked_better": 0},
                "rows": [{"id": "a", "asked_rank": 5, "other_rank": 5, "raw_asked_rank": 5, "raw_other_rank": 5}]
            }
        });
        let delta = rank_delta(&carried, &minus_c).unwrap();
        assert_eq!(delta["changed_by_cut"], true);
        assert_eq!(delta["teacher_forced"]["changed_rows"], 1);
        assert_eq!(delta["paired_first"]["changed_rows"], 1);
    }
}
