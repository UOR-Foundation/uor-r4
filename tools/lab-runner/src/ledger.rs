//! Shared model-time ledger charging and rebuild.
//!
//! Every completed job appends a `charge-{utc}-{lab}-{id}.json` record
//! (schema `uor-r4.model-time-charge/1`) and rewrites `model-time.json`
//! under an exclusive `File` lock on `model-time.json.lock`, so concurrent
//! runners never lose each other's charges. `rebuild` recomputes
//! `model-time.json` from the records alone.

use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{atomic_write, invalid, sanitize_component, utc_now_compact, utc_now_iso, Result};

pub const CHARGE_SCHEMA: &str = "uor-r4.model-time-charge/1";
pub const MODEL_TIME_FILE: &str = "model-time.json";
pub const LOCK_FILE: &str = "model-time.json.lock";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LedgerState {
    pub cumulative_ms: u64,
    pub limit_ms: u64,
}

/// Guard holding the ledger lock; the lock releases on drop.
struct LedgerLock {
    _file: fs::File,
}

fn acquire_lock(dir: &Path) -> Result<LedgerLock> {
    let path = dir.join(LOCK_FILE);
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)?;
    // Bounded blocking retry: about 5 s in 50 ms slices.
    for _ in 0..100 {
        match file.try_lock() {
            Ok(()) => return Ok(LedgerLock { _file: file }),
            Err(fs::TryLockError::WouldBlock) => std::thread::sleep(Duration::from_millis(50)),
            Err(fs::TryLockError::Error(error)) => return Err(error.into()),
        }
    }
    Err(invalid(format!(
        "ledger lock {} stayed busy for about 5 s",
        path.display()
    )))
}

fn read_state_unlocked(dir: &Path) -> Result<LedgerState> {
    let path = dir.join(MODEL_TIME_FILE);
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(LedgerState {
                cumulative_ms: 0,
                limit_ms: 0,
            })
        }
        Err(error) => return Err(error.into()),
    };
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    let cumulative_ms = value
        .get("cumulative_ms")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| invalid(format!("{} lacks cumulative_ms", path.display())))?;
    let limit_ms = value
        .get("limit_ms")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| invalid(format!("{} lacks limit_ms", path.display())))?;
    Ok(LedgerState {
        cumulative_ms,
        limit_ms,
    })
}

fn write_state_unlocked(dir: &Path, state: LedgerState) -> Result<()> {
    crate::write_json_atomic(
        &dir.join(MODEL_TIME_FILE),
        &json!({"cumulative_ms": state.cumulative_ms, "limit_ms": state.limit_ms}),
    )
}

/// Append one charge record and advance `model-time.json`, under the lock.
/// Returns the state after the charge.
pub fn record_charge(
    dir: &Path,
    lab: &str,
    job_id: &str,
    charged_ms: u64,
    wall_s: u64,
    outcome: &str,
) -> Result<LedgerState> {
    fs::create_dir_all(dir)?;
    let _lock = acquire_lock(dir)?;
    let before = read_state_unlocked(dir)?;
    let after = LedgerState {
        cumulative_ms: before.cumulative_ms + charged_ms,
        limit_ms: before.limit_ms,
    };
    let record = json!({
        "schema": CHARGE_SCHEMA,
        "recorded_utc": utc_now_iso(),
        "lab": lab,
        "job_id": job_id,
        "charged_ms": charged_ms,
        "wall_s": wall_s,
        "outcome": outcome,
        "runner": "lab-runner",
        "before": {"cumulative_ms": before.cumulative_ms, "limit_ms": before.limit_ms},
        "after": {"cumulative_ms": after.cumulative_ms, "limit_ms": after.limit_ms},
    });
    let name = format!(
        "charge-{}-{}-{}.json",
        utc_now_compact(),
        sanitize_component(lab),
        sanitize_component(job_id)
    );
    atomic_write(&dir.join(name), &serde_json::to_vec_pretty(&record)?)?;
    write_state_unlocked(dir, after)?;
    Ok(after)
}

/// Parse the ledger's ISO-8601 UTC timestamps (`...Z` or `...+00:00`, with
/// optional fractional seconds) into a sortable key. Unparseable values sort
/// at the epoch so a malformed record cannot silently anchor the fold.
fn parse_utc_key(raw: &str) -> Option<(i64, u32)> {
    let body = raw
        .strip_suffix('Z')
        .or_else(|| raw.strip_suffix("+00:00"))?;
    let (hms, frac) = match body.split_once('.') {
        Some((hms, frac)) => (hms, frac),
        None => (body, "0"),
    };
    let (date, time) = hms.split_once('T')?;
    let mut date_parts = date.split('-');
    let year: i64 = date_parts.next()?.parse().ok()?;
    let month: i64 = date_parts.next()?.parse().ok()?;
    let day: i64 = date_parts.next()?.parse().ok()?;
    let mut time_parts = time.split(':');
    let hour: i64 = time_parts.next()?.parse().ok()?;
    let minute: i64 = time_parts.next()?.parse().ok()?;
    let second: i64 = time_parts.next()?.parse().ok()?;
    // days-from-civil (Hinnant).
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    let secs = days * 86_400 + hour * 3600 + minute * 60 + second;
    let nanos: u32 = format!("{frac:0<9}")[..9.min(frac.len().max(9))]
        .parse()
        .ok()
        .unwrap_or(0);
    Some((secs, nanos))
}

fn is_additive_ms_field(key: &str) -> bool {
    key.ends_with("_ms") && !key.ends_with("_cumulative_ms") && key != "base_read_cumulative_ms"
}

/// Sum a charge record's additive top-level `*_ms` fields.
fn additive_ms(record: &serde_json::Value) -> u64 {
    record
        .as_object()
        .map(|object| {
            object
                .iter()
                .filter(|(key, _)| is_additive_ms_field(key))
                .filter_map(|(_, value)| value.as_u64())
                .sum()
        })
        .unwrap_or(0)
}

fn anchor_of(record: &serde_json::Value, is_charge: bool) -> Option<(u64, Option<u64>)> {
    if is_charge {
        let base = record.get("base_read_cumulative_ms")?.as_u64()?;
        // A charge anchor's base read is a stale "before" snapshot; its own
        // additive fields restore the missing work on top of it.
        Some((base + additive_ms(record), None))
    } else {
        let after = record.get("after")?;
        let cumulative = after.get("cumulative_ms")?.as_u64()?;
        let limit = after.get("limit_ms").and_then(serde_json::Value::as_u64);
        Some((cumulative, limit))
    }
}

/// Recompute the ledger state from the charge/extension records alone.
/// Does not take the lock; callers that rewrite `model-time.json` hold it.
pub fn fold_records(dir: &Path) -> Result<LedgerState> {
    let mut records: Vec<((i64, u32), u8, String, serde_json::Value)> = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_charge = name.starts_with("charge-") && name.ends_with(".json");
        let is_extension = name.starts_with("extension-") && name.ends_with(".json");
        if !is_charge && !is_extension {
            continue;
        }
        let value: serde_json::Value = serde_json::from_slice(&fs::read(entry.path())?)?;
        let key = value
            .get("recorded_utc")
            .and_then(serde_json::Value::as_str)
            .and_then(parse_utc_key)
            .unwrap_or((0, 0));
        // Charges sort before extensions at equal timestamps, so an
        // extension recorded in the same second anchors above its charge.
        let kind_rank = if is_charge { 0 } else { 1 };
        records.push((key, kind_rank, name, value));
    }
    records.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));

    let mut anchor_index = None;
    for (index, (_, _, name, value)) in records.iter().enumerate() {
        let is_charge = name.starts_with("charge-");
        if anchor_of(value, is_charge).is_some() {
            anchor_index = Some(index);
        }
    }
    let anchor_index = anchor_index.ok_or_else(|| {
        invalid(format!(
            "ledger {} has no record with an explicit cumulative anchor; a genesis record is needed",
            dir.display()
        ))
    })?;

    let (_, _, anchor_name, anchor_value) = &records[anchor_index];
    let (mut cumulative_ms, mut limit_ms) =
        anchor_of(anchor_value, anchor_name.starts_with("charge-"))
            .ok_or_else(|| invalid("ledger anchor disappeared during fold"))?;
    for (_, _, name, value) in &records[anchor_index + 1..] {
        if name.starts_with("charge-") {
            cumulative_ms += additive_ms(value);
        } else {
            let increment = value
                .get("increment_ms")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
            let base = limit_ms.ok_or_else(|| {
                invalid(format!(
                    "ledger anchor {anchor_name} carries no limit, so later extension {name} cannot be folded"
                ))
            })?;
            limit_ms = Some(base + increment);
        }
    }
    let limit_ms = limit_ms.ok_or_else(|| {
        invalid(format!(
            "ledger anchor {anchor_name} carries no limit and no later extension sets one"
        ))
    })?;
    Ok(LedgerState {
        cumulative_ms,
        limit_ms,
    })
}

/// Rebuild `model-time.json` from the records, under the ledger lock.
pub fn rebuild(dir: &Path) -> Result<LedgerState> {
    fs::create_dir_all(dir)?;
    let _lock = acquire_lock(dir)?;
    let state = fold_records(dir)?;
    write_state_unlocked(dir, state)?;
    Ok(state)
}

/// Path of the ledger directory used by the CLI default.
pub fn default_dir() -> Result<PathBuf> {
    crate::default_ledger_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tempdir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "lab-runner-ledger-{}-{}-{name}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn copy_fixtures(target: &Path) {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ledger");
        for entry in fs::read_dir(&fixtures).unwrap() {
            let entry = entry.unwrap();
            fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
        }
    }

    #[test]
    fn real_records_fold_to_the_known_totals() {
        let dir = tempdir("real");
        copy_fixtures(&dir);
        let state = rebuild(&dir).unwrap();
        assert_eq!(state.cumulative_ms, 782_538_181);
        assert_eq!(state.limit_ms, 1_130_000_000);
        let written: serde_json::Value =
            serde_json::from_slice(&fs::read(dir.join(MODEL_TIME_FILE)).unwrap()).unwrap();
        assert_eq!(written["cumulative_ms"].as_u64().unwrap(), 782_538_181);
        assert_eq!(written["limit_ms"].as_u64().unwrap(), 1_130_000_000);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn fold_without_anchor_errors_clearly() {
        let dir = tempdir("no-anchor");
        atomic_write(
            &dir.join("charge-20260101T000000Z-lab-a.json"),
            br#"{"schema":"uor-r4.model-time-charge/1","recorded_utc":"2026-01-01T00:00:00Z","charged_ms":5}"#,
        )
        .unwrap();
        let error = rebuild(&dir).unwrap_err();
        assert!(error.to_string().contains("genesis"), "{error}");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn concurrent_charges_are_both_counted() {
        let dir = tempdir("concurrent");
        // Genesis: an anchored extension so rebuild has a base.
        atomic_write(
            &dir.join("extension-20260101T000000Z-genesis.json"),
            br#"{"schema":"uor-r4.model-time-extension/1","recorded_utc":"2026-01-01T00:00:00Z","increment_ms":1000,"after":{"cumulative_ms":100,"limit_ms":1000}}"#,
        )
        .unwrap();
        write_state_unlocked(
            &dir,
            LedgerState {
                cumulative_ms: 100,
                limit_ms: 1000,
            },
        )
        .unwrap();

        let mut handles = Vec::new();
        for index in 0..2 {
            let dir = dir.clone();
            handles.push(std::thread::spawn(move || {
                record_charge(
                    &dir,
                    "lab-test",
                    &format!("job-{index}"),
                    250 + index as u64,
                    1,
                    "completed",
                )
                .unwrap()
            }));
        }
        let states: Vec<LedgerState> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        // The two charges serialize through the lock: the second writer
        // observes the first writer's "after".
        assert_ne!(states[0].cumulative_ms, states[1].cumulative_ms);
        let final_state = read_state_unlocked(&dir).unwrap();
        assert_eq!(final_state.cumulative_ms, 100 + 250 + 251);

        let rebuilt = rebuild(&dir).unwrap();
        assert_eq!(rebuilt.cumulative_ms, 100 + 250 + 251);
        assert_eq!(rebuilt.limit_ms, 1000);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parse_utc_key_handles_z_and_offset_and_fraction() {
        assert_eq!(
            parse_utc_key("2026-09-29T20:05:25Z"),
            parse_utc_key("2026-09-29T20:05:25+00:00")
        );
        let (whole, _) = parse_utc_key("2026-09-28T20:24:46Z").unwrap();
        let (frac_secs, frac_nanos) = parse_utc_key("2026-09-28T20:24:46.930406+00:00").unwrap();
        assert_eq!(whole, frac_secs);
        assert_eq!(frac_nanos, 930_406_000);
        assert!(parse_utc_key("not a timestamp").is_none());
    }
}
