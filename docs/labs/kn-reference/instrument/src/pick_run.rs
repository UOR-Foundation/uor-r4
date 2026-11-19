//! claude-439: the `--pick-arms` mode. The query stream q is R, DEV-tune, R+ and DEV+;
//! SEGMENTS holds one byte per q position (0 R, 1 DEV-tune, 2 R+, 3 DEV+, 4 DEV+ second
//! copy). The main span is every position with segment at most 1; the plus span is the rest.
#![cfg_attr(not(feature = "cuda"), allow(dead_code))]

use crate::{fail, Failure};
use std::ops::Range;
use uor_r4_discrete::kn_ceiling::{PickTarget, RawTables};
use uor_r4_discrete::kn_pick::PickRow;
use uor_r4_discrete::ngram_count::NGRAM_BOS;

/// Most targets one GPU lookup call takes.
pub const LOOKUP_CHUNK: usize = 1 << 24;
/// The Pick read's window.
pub const CONTEXT: usize = 192;
/// The trunk's vocabulary.
pub const VOCAB: usize = 4096;

/// Where the plus span starts and how many targets the main span holds.
#[derive(Debug, PartialEq, Eq)]
pub struct Spans {
    /// First q index of the plus span.
    pub split: usize,
    /// Non-BOS positions before `split`.
    pub main_targets: usize,
}

/// Splits q into the main and plus spans by SEGMENTS.
pub fn split_spans(query: &[u16], segments: &[u8]) -> Result<Spans, Failure> {
    if query.len() != segments.len() {
        return Err(fail(format!(
            "segments hold {} bytes for {} query positions",
            segments.len(),
            query.len()
        )));
    }
    if let Some(bad) = segments.iter().find(|&&s| s > 4) {
        return Err(fail(format!("segment {bad} is above 4")));
    }
    let split = segments
        .iter()
        .position(|&s| s >= 2)
        .ok_or_else(|| fail("no plus span: no segment 2, 3 or 4"))?;
    if split == 0 {
        return Err(fail("the main span is empty"));
    }
    if let Some(at) = segments[split..].iter().position(|&s| s <= 1) {
        return Err(fail(format!(
            "segment {} at q {} follows the plus span's start",
            segments[split + at],
            split + at
        )));
    }
    if query[split] != NGRAM_BOS {
        return Err(fail("the plus span must start with BOS"));
    }
    let main_targets = query[..split].iter().filter(|&&t| t != NGRAM_BOS).count();
    Ok(Spans {
        split,
        main_targets,
    })
}

/// Cuts `span` into consecutive ranges, each starting at a BOS and holding at most
/// `max_targets` non-BOS positions. Refuses a record with more targets than that.
pub fn chunk_bounds(span: &[u16], max_targets: usize) -> Result<Vec<Range<usize>>, Failure> {
    if span.first() != Some(&NGRAM_BOS) {
        return Err(fail("a lookup span must start with BOS"));
    }
    struct Record {
        start: usize,
        targets: usize,
    }
    let mut records: Vec<Record> = Vec::new();
    for (at, &token) in span.iter().enumerate() {
        if token == NGRAM_BOS {
            records.push(Record {
                start: at,
                targets: 0,
            });
        } else if let Some(last) = records.last_mut() {
            last.targets += 1;
        }
    }
    let mut out = Vec::new();
    let mut begin = 0;
    let mut held = 0;
    for record in &records {
        if record.targets > max_targets {
            return Err(fail(format!(
                "the record at {} holds {} targets, above the chunk cap {max_targets}",
                record.start, record.targets
            )));
        }
        if held + record.targets > max_targets {
            out.push(begin..record.start);
            begin = record.start;
            held = 0;
        }
        held += record.targets;
    }
    out.push(begin..span.len());
    Ok(out)
}

/// The first index where two row tapes differ; a length difference counts at the shorter length.
pub fn first_row_mismatch(a: &[PickRow], b: &[PickRow]) -> Option<usize> {
    let shared = a.len().min(b.len());
    (0..shared)
        .find(|&i| a[i] != b[i])
        .or((a.len() != b.len()).then_some(shared))
}

/// The scorer's view of each row: its class and whether the pick hit.
pub fn fire_rows_to_targets(rows: &[PickRow]) -> Vec<PickTarget> {
    rows.iter()
        .map(|row| PickTarget {
            class: row.class,
            hit: row.hit,
        })
        .collect()
}

/// The `VmHWM:` value of a `/proc/self/status` text, in KiB.
pub fn vm_hwm_kib(status: &str) -> Option<u64> {
    let line = status.lines().find(|line| line.starts_with("VmHWM:"))?;
    line.split_whitespace().nth(1)?.parse().ok()
}

/// What main.rs hands the Pick arms.
pub struct PickRun<'a> {
    pub highest: u32,
    pub query: &'a [u16],
    pub classes: &'a [u8],
    pub id_bytes: &'a [u8],
    pub masses: &'a [u64],
    pub raw: &'a RawTables,
    pub segments_path: &'a str,
    pub trunk_path: &'a str,
    pub check_host: bool,
    pub profile_phases: bool,
    pub out: &'a str,
    pub header: serde_json::Value,
}

#[cfg(feature = "cuda")]
fn span_lookups(
    lookup: &uor_r4_discrete::kn_lookup_device::KnLookup,
    raw: &RawTables,
    span: &[u16],
    check_host: bool,
    name: &str,
) -> Result<(Vec<u32>, Vec<u64>), Failure> {
    let mut counts = Vec::new();
    let mut totals = Vec::new();
    for range in chunk_bounds(span, LOOKUP_CHUNK)? {
        let (c, t) = lookup.lookups(&span[range])?;
        counts.extend(c);
        totals.extend(t);
    }
    if check_host {
        let (host_counts, host_totals) = raw.lookups(span)?;
        if let Some(i) =
            (0..host_counts.len().max(counts.len())).find(|&i| host_counts.get(i) != counts.get(i))
        {
            return Err(fail(format!(
                "{name} lookup counts differ from host at {i}"
            )));
        }
        if let Some(i) =
            (0..host_totals.len().max(totals.len())).find(|&i| host_totals.get(i) != totals.get(i))
        {
            return Err(fail(format!(
                "{name} lookup totals differ from host at {i}"
            )));
        }
        eprintln!("{name}: host lookups = GPU at {} targets", totals.len());
    }
    Ok((counts, totals))
}

#[cfg(feature = "cuda")]
fn rows_per_second(rows: usize, ms: u128) -> u128 {
    rows as u128 * 1000 / ms.max(1)
}

#[cfg(feature = "cuda")]
fn check_rows(device: &[PickRow], host: &[PickRow], arm: &str) -> Result<usize, Failure> {
    if let Some(at) = first_row_mismatch(device, host) {
        return Err(fail(format!(
            "B1: {arm} device row differs from the host rows at target {at}"
        )));
    }
    Ok(device.len())
}

/// The trunk oracle at every kept target: all four blocks' words, and a fired class
/// wherever the oracle has a successor. Returns the first mismatch by target.
#[cfg(feature = "cuda")]
fn check_oracle(
    model: &uor_r4_integer::discrete_d11::IntegerDiscreteModel,
    main: &[u16],
    tape: &crate::pick_device::DeviceTape,
) -> Result<usize, Failure> {
    let windows = crate::pick_device::windows(main, CONTEXT)?;
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let per = tape.words.len().div_ceil(threads).max(1);
    let first: Option<(usize, String)> = std::thread::scope(|scope| {
        let handles: Vec<_> = tape
            .words
            .chunks(per)
            .map(|kept_chunk| {
                let windows = &windows;
                scope.spawn(move || -> Option<(usize, String)> {
                    for kept in kept_chunk {
                        let k = kept.ordinal;
                        let Some(&(_, window)) = windows.get(k) else {
                            return Some((k, format!("no window for target {k}")));
                        };
                        let prefix: Vec<u32> = window.iter().map(|&id| u32::from(id)).collect();
                        let seen = match model.observe_identity_pick(&prefix) {
                            Ok(seen) => seen,
                            Err(e) => return Some((k, format!("oracle at target {k}: {e}"))),
                        };
                        if seen.block_words.as_slice() != kept.blocks.as_slice() {
                            return Some((k, format!("A1 block words differ at target {k}")));
                        }
                        let class = tape.rows.get(k).map(|row| row.class);
                        if seen.successor.is_some() && class.is_none_or(|c| c == 0) {
                            return Some((
                                k,
                                format!("A1 row at target {k} has class 0 where the oracle has a successor"),
                            ));
                        }
                    }
                    None
                })
            })
            .collect();
        handles
            .into_iter()
            .filter_map(|h| {
                h.join()
                    .unwrap_or_else(|_| Some((0, "an oracle thread panicked".into())))
            })
            .min_by_key(|found| found.0)
    });
    if let Some((_, message)) = first {
        return Err(fail(format!("B1: {message}")));
    }
    Ok(tape.words.len())
}

#[cfg(feature = "cuda")]
pub fn run(job: PickRun) -> Result<(), Failure> {
    use crate::pick_device::{load_trunk, root_codes, PickDevice, C3_SEED, TRUNK_SHA256};
    use crate::pick_fit::{combine, run_main, run_plus, SpanInputs};
    use serde_json::{json, Map, Value};
    use std::time::Instant;
    use uor_r4_discrete::kn_lookup_device::KnLookup;
    use uor_r4_discrete::kn_pick::{
        code_pick_rows, exact_pair_rows, permute_code_rows, row_permutation,
    };

    let started = Instant::now();
    let mut phases = Map::new();
    fn phase(phases: &mut Map<String, Value>, name: &str, at: Instant) -> u128 {
        let ms = at.elapsed().as_millis();
        eprintln!("pick-arms: {name}, {ms} ms");
        phases.insert(format!("{name}_ms"), json!(ms));
        ms
    }

    // 1. Segments, spans and the trunk.
    let at = Instant::now();
    let segments = std::fs::read(job.segments_path)?;
    let segments_sha256 = crate::sha256_hex(&segments);
    let spans = split_spans(job.query, &segments)?;
    if job.classes.len() != job.query.len() {
        return Err(fail("classes and query lengths differ"));
    }
    let model = load_trunk(std::path::Path::new(job.trunk_path))?;
    let codes = root_codes(&model.identity_pick_inventory()?.0)?;
    let perm = row_permutation(VOCAB, C3_SEED);
    let main = &job.query[..spans.split];
    let plus = &job.query[spans.split..];
    let main_target_segments: Vec<u8> = main
        .iter()
        .zip(&segments[..spans.split])
        .filter(|&(&t, _)| t != NGRAM_BOS)
        .map(|(_, &s)| s)
        .collect();
    let dev_tune_targets = main_target_segments.iter().filter(|&&s| s == 1).count();
    eprintln!(
        "pick-arms: main span {} positions, {} targets ({dev_tune_targets} DEV-tune); plus span {} positions",
        main.len(),
        spans.main_targets,
        plus.len()
    );
    phase(&mut phases, "load", at);

    let lookup = KnLookup::new(0, job.raw)?;
    let dev_a1 = PickDevice::new(&model, None)?;
    let mut b1 = Map::new();

    // 2-4. The main span: lookups, the three arms' reads and B1.
    let at = Instant::now();
    let (main_counts, main_totals) = span_lookups(&lookup, job.raw, main, job.check_host, "main")?;
    phase(&mut phases, "main_lookups", at);

    let at = Instant::now();
    let keep = |k: usize| job.check_host && main_target_segments.get(k) == Some(&1);
    let a1_main = dev_a1.rows(main, CONTEXT, &keep)?;
    let ms = phase(&mut phases, "a1_main_read", at);
    phases.insert(
        "a1_main_rows_per_s".into(),
        json!(rows_per_second(a1_main.rows.len(), ms)),
    );

    let at = Instant::now();
    let dev_c3 = PickDevice::new(&model, Some(&perm))?;
    let c3_main = dev_c3.rows(main, CONTEXT, &|_| false)?;
    drop(dev_c3);
    let ms = phase(&mut phases, "c3_main_read", at);
    phases.insert(
        "c3_main_rows_per_s".into(),
        json!(rows_per_second(c3_main.rows.len(), ms)),
    );

    let at = Instant::now();
    let c2_rows = exact_pair_rows(main, VOCAB, CONTEXT)?;
    phase(&mut phases, "c2_main_host", at);

    if job.check_host {
        let at = Instant::now();
        let a1_rows = check_rows(
            &a1_main.rows,
            &code_pick_rows(main, &codes, CONTEXT)?,
            "A1 main",
        )?;
        let permuted = permute_code_rows(&codes, C3_SEED);
        let c3_rows = check_rows(
            &c3_main.rows,
            &code_pick_rows(main, &permuted, CONTEXT)?,
            "C3 main",
        )?;
        if a1_main.words.len() != dev_tune_targets {
            return Err(fail(format!(
                "B1: kept words at {} targets, expected {dev_tune_targets} DEV-tune targets",
                a1_main.words.len()
            )));
        }
        let oracle_targets = check_oracle(&model, main, &a1_main)?;
        b1.insert("a1_main_rows".into(), json!(a1_rows));
        b1.insert("c3_rows".into(), json!(c3_rows));
        b1.insert("oracle_targets".into(), json!(oracle_targets));
        phase(&mut phases, "b1_main", at);
    }
    let a1 = fire_rows_to_targets(&a1_main.rows);
    let c3 = fire_rows_to_targets(&c3_main.rows);
    let c2 = fire_rows_to_targets(&c2_rows);
    drop((a1_main, c3_main, c2_rows));
    let main_inputs = SpanInputs {
        highest: job.highest,
        query: main,
        segments: &segments[..spans.split],
        classes: &job.classes[..spans.split],
        id_bytes: job.id_bytes,
        kn_masses: job
            .masses
            .get(..spans.main_targets)
            .ok_or_else(|| fail("fewer KN masses than main-span targets"))?,
        counts: &main_counts,
        totals: &main_totals,
        fit_segment: 0,
    };
    // The main arms run, and the main span's lookups are freed, before the
    // plus span is looked up: the two spans' lookups never coexist.
    let at = Instant::now();
    let main_result = run_main(&main_inputs, &a1, &c2, &c3)?;
    phase(&mut phases, "main_arms", at);
    drop(main_inputs);
    drop((main_counts, main_totals, a1, c2, c3));

    // The plus span: lookups, A1 and B1.
    let at = Instant::now();
    let (plus_counts, plus_totals) = span_lookups(&lookup, job.raw, plus, job.check_host, "plus")?;
    drop(lookup);
    phase(&mut phases, "plus_lookups", at);

    let at = Instant::now();
    let a1_plus = dev_a1.rows(plus, CONTEXT, &|_| false)?;
    drop(dev_a1);
    let ms = phase(&mut phases, "a1_plus_read", at);
    phases.insert(
        "a1_plus_rows_per_s".into(),
        json!(rows_per_second(a1_plus.rows.len(), ms)),
    );
    if job.check_host {
        let at = Instant::now();
        let rows = check_rows(
            &a1_plus.rows,
            &code_pick_rows(plus, &codes, CONTEXT)?,
            "A1 plus",
        )?;
        b1.insert("a1_plus_rows".into(), json!(rows));
        phase(&mut phases, "b1_plus", at);
    }
    let plus_a1 = fire_rows_to_targets(&a1_plus.rows);
    drop(a1_plus);
    let plus_inputs = SpanInputs {
        highest: job.highest,
        query: plus,
        segments: &segments[spans.split..],
        classes: &job.classes[spans.split..],
        id_bytes: job.id_bytes,
        kn_masses: job
            .masses
            .get(spans.main_targets..)
            .ok_or_else(|| fail("fewer KN masses than main-span targets"))?,
        counts: &plus_counts,
        totals: &plus_totals,
        fit_segment: 2,
    };

    // 5. The plus arms, then the record of both spans.
    let at = Instant::now();
    let plus_result = run_plus(&plus_inputs, &plus_a1)?;
    phase(&mut phases, "plus_arms", at);
    let arms = combine(main_result, plus_result);

    // 6. The record, never over an existing file.
    let mut summary = job.header;
    summary["pick_arms"] = arms;
    summary["b1"] = if job.check_host {
        Value::Object(b1)
    } else {
        Value::Null
    };
    summary["trunk_sha256"] = json!(TRUNK_SHA256);
    summary["segments_sha256"] = json!(segments_sha256);
    if job.profile_phases {
        summary["phases_ms"] = Value::Object(phases);
    }
    if let Some(kib) = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| vm_hwm_kib(&status))
    {
        eprintln!("pick-arms: peak RSS {kib} KiB");
        summary["peak_rss_kib"] = json!(kib);
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(job.out)?;
    use std::io::Write;
    file.write_all((serde_json::to_string_pretty(&summary)? + "\n").as_bytes())?;
    eprintln!("wrote {}, {} ms", job.out, started.elapsed().as_millis());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const B: u16 = NGRAM_BOS;

    #[test]
    fn split_spans_finds_the_plus_span() {
        assert_eq!(B, 0);
        let query = [B, 5, 6, B, 7, B, 8, 9, B, 10];
        let segments = [0, 0, 0, 1, 1, 2, 2, 2, 4, 3];
        assert_eq!(
            split_spans(&query, &segments).unwrap(),
            Spans {
                split: 5,
                main_targets: 3
            }
        );
        // Length mismatch, a segment above 4, a main segment after the plus span,
        // no plus span, an empty main span and a plus span not at a BOS.
        assert!(split_spans(&query, &segments[..9]).is_err());
        assert!(split_spans(&query, &[0, 0, 0, 1, 1, 2, 2, 2, 5, 3]).is_err());
        assert!(split_spans(&query, &[0, 0, 0, 1, 1, 2, 2, 1, 4, 3]).is_err());
        assert!(split_spans(&query, &[0; 10]).is_err());
        assert!(split_spans(&query, &[2; 10]).is_err());
        assert!(split_spans(&query, &[0, 0, 0, 1, 2, 2, 2, 2, 4, 3]).is_err());
    }

    #[test]
    fn chunk_bounds_cut_at_records() {
        let span = [B, 5, 6, B, 7, B, 8, 9, 10, B, B, 11];
        let ranges = chunk_bounds(&span, 3).unwrap();
        assert_eq!(ranges, vec![0..5, 5..10, 10..12]);
        let mut next = 0;
        for range in &ranges {
            assert_eq!(range.start, next);
            assert_eq!(span[range.start], B);
            let targets = span[range.clone()].iter().filter(|&&t| t != B).count();
            assert!(targets <= 3);
            next = range.end;
        }
        assert_eq!(next, span.len());
        assert_eq!(chunk_bounds(&span, 16).unwrap(), vec![0..12]);
        assert_eq!(
            chunk_bounds(&span, 2).unwrap_err().to_string(),
            "the record at 5 holds 3 targets, above the chunk cap 2"
        );
        assert!(chunk_bounds(&span[1..], 16).is_err());
    }

    #[test]
    fn first_row_mismatch_finds_the_first_difference() {
        let row = |class: u8| PickRow {
            class,
            hit: class == 2,
            pick: None,
            distance: None,
        };
        let a = [row(0), row(1), row(2)];
        assert_eq!(first_row_mismatch(&a, &a), None);
        assert_eq!(first_row_mismatch(&a, &[row(0), row(3), row(1)]), Some(1));
        assert_eq!(first_row_mismatch(&a, &a[..2]), Some(2));
        assert_eq!(first_row_mismatch(&a[..1], &a), Some(1));
        let targets = fire_rows_to_targets(&a);
        assert_eq!(
            targets[2],
            PickTarget {
                class: 2,
                hit: true
            }
        );
    }

    #[test]
    fn vm_hwm_reads_the_peak_line() {
        let status = "Name:\tkn\nVmPeak:\t  900 kB\nVmHWM:\t  4321 kB\nVmRSS:\t 12 kB\n";
        assert_eq!(vm_hwm_kib(status), Some(4321));
        assert_eq!(vm_hwm_kib("Name:\tkn\n"), None);
    }
}
