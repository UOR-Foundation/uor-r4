//! claude-439: fit and score the Pick arms on the host and return their JSON.
//!
//! A span is R + DEV (fit segment 0, scored segment 1) or R+ + DEV+ (fit segment 2,
//! scored segments 3 and 4). Every arm fits from the KN start for `EM_ITERATIONS` passes
//! on the fit segment and is scored on the rest with its weights frozen.
#![allow(dead_code)]

use crate::pick_arms::{b2_pass, paired_band, record_bits, segment_tally};
use crate::{fail, Failure, EM_ITERATIONS};
use serde_json::{json, Map, Value};
use uor_r4_discrete::kn_ceiling::{
    bucket_count, CeilingTable, EmStart, PickTarget, PICK_FIRED_EXACT, PICK_NOT_FIRED,
};
use uor_r4_discrete::ngram_count::NGRAM_BOS;
use uor_r4_integer::lawful_likelihood::bits_q12;

pub const BAND_RESAMPLES: u32 = 1000;
pub const BAND_SEED: u64 = 4390;

pub struct SpanInputs<'a> {
    pub highest: u32,
    pub query: &'a [u16],
    pub segments: &'a [u8],
    pub classes: &'a [u8],
    pub id_bytes: &'a [u8],
    pub kn_masses: &'a [u64],
    pub counts: &'a [u32],
    pub totals: &'a [u64],
    pub fit_segment: u8,
}

#[derive(Clone, Copy)]
pub enum ArmPick<'a> {
    Absent,
    Read(&'a [PickTarget]),
    Masked(&'a [PickTarget]),
}

pub struct PickWeight {
    pub bucket: usize,
    pub class: u8,
    pub fit_targets: u64,
    pub weight_q32: u64,
}

pub struct ArmScore {
    pub fit_bits: Vec<u128>,
    pub gap_q12: u128,
    /// `bits_q12` of each target's mixed mass.
    pub bits: Vec<u64>,
    pub pick_weights: Vec<PickWeight>,
}

pub struct B3 {
    pub last_at_most_first: bool,
    pub scored_equals_last: bool,
    pub rises: Vec<u32>,
}

pub struct PickStats {
    pub targets: u64,
    pub fired: u64,
    pub exact: u64,
}

/// The segment of every non-BOS position, in stream order.
fn target_segments(span: &SpanInputs) -> Result<Vec<u8>, Failure> {
    if span.segments.len() != span.query.len() || span.classes.len() != span.query.len() {
        return Err(fail("pick_fit: segments/classes length differs from query"));
    }
    let segments: Vec<u8> = span
        .query
        .iter()
        .zip(span.segments)
        .filter(|(&token, _)| token != NGRAM_BOS)
        .map(|(_, &segment)| segment)
        .collect();
    if segments.len() != span.kn_masses.len() {
        return Err(fail("pick_fit: non-BOS count differs from kn_masses"));
    }
    Ok(segments)
}

pub fn fit_arm(span: &SpanInputs, pick: ArmPick) -> Result<ArmScore, Failure> {
    let segments = target_segments(span)?;
    let fit: Vec<bool> = segments.iter().map(|&s| s == span.fit_segment).collect();
    if !fit.contains(&true) {
        return Err(fail("fit_arm: the span holds no fit target"));
    }
    let (table, has_pick) = match pick {
        ArmPick::Absent => (
            CeilingTable::from_lookups(span.highest, span.kn_masses, span.counts, span.totals)?,
            false,
        ),
        ArmPick::Read(rows) | ArmPick::Masked(rows) => (
            CeilingTable::from_lookups_with_pick(
                span.highest,
                span.kn_masses,
                span.counts,
                span.totals,
                rows,
            )?,
            true,
        ),
    };
    let mut on = vec![true; table.experts()];
    let pick_expert = table.pick_expert();
    if let ArmPick::Masked(_) = pick {
        let expert = pick_expert.ok_or_else(|| fail("fit_arm: masked arm has no Pick expert"))?;
        on[expert] = false;
    }
    let result = table.fit_em_from(&fit, &on, EmStart::Kn, EM_ITERATIONS, bits_q12)?;
    let gap = table.gap_bound_q12(&result.weights, &fit, &on, bits_q12)?;
    let masses = table.mixed_masses(&result.weights)?;
    let bits = masses
        .iter()
        .map(|&mass| bits_q12(mass).map_err(|e| fail(format!("{e:?}"))))
        .collect::<Result<Vec<u64>, Failure>>()?;
    let mut pick_weights = Vec::new();
    if let (true, Some(expert)) = (has_pick, pick_expert) {
        let per_class = bucket_count(span.highest);
        let mut fit_targets = vec![0u64; table.bucket_total()];
        for (t, &bucket) in table.buckets().iter().enumerate() {
            if fit[t] {
                fit_targets[bucket as usize] += 1;
            }
        }
        for (bucket, &count) in fit_targets.iter().enumerate() {
            if bucket / per_class != 0 && count > 0 {
                pick_weights.push(PickWeight {
                    bucket,
                    class: (bucket / per_class) as u8,
                    fit_targets: count,
                    weight_q32: result.weights[bucket * table.experts() + expert],
                });
            }
        }
    }
    Ok(ArmScore {
        fit_bits: result.fit_bits,
        gap_q12: gap,
        bits,
        pick_weights,
    })
}

pub fn b3(span: &SpanInputs, score: &ArmScore) -> Result<B3, Failure> {
    let segments = target_segments(span)?;
    if score.bits.len() != segments.len() {
        return Err(fail("b3: bits length differs from the span's targets"));
    }
    let (first, last) = match (score.fit_bits.first(), score.fit_bits.last()) {
        (Some(&first), Some(&last)) => (first, last),
        _ => return Err(fail("b3: empty fit_bits")),
    };
    let scored: u128 = segments
        .iter()
        .zip(&score.bits)
        .filter(|(&s, _)| s == span.fit_segment)
        .map(|(_, &b)| u128::from(b))
        .sum();
    let rises = (0..score.fit_bits.len() - 1)
        .filter(|&i| score.fit_bits[i + 1] > score.fit_bits[i])
        .map(|i| i as u32)
        .collect();
    Ok(B3 {
        last_at_most_first: last <= first,
        scored_equals_last: scored == last,
        rises,
    })
}

pub fn pick_stats(
    span: &SpanInputs,
    rows: &[PickTarget],
    segment: u8,
) -> Result<PickStats, Failure> {
    let segments = target_segments(span)?;
    if rows.len() != segments.len() {
        return Err(fail(
            "pick_stats: rows length differs from the span's targets",
        ));
    }
    let mut stats = PickStats {
        targets: 0,
        fired: 0,
        exact: 0,
    };
    for (row, &s) in rows.iter().zip(&segments) {
        if s != segment {
            continue;
        }
        stats.targets += 1;
        stats.fired += u64::from(row.class != PICK_NOT_FIRED);
        stats.exact += u64::from(row.class == PICK_FIRED_EXACT);
    }
    Ok(stats)
}

fn tally_json(span: &SpanInputs, bits: &[u64], keep: &[u8]) -> Result<Value, Failure> {
    let (all, novel) = segment_tally(
        span.query,
        span.segments,
        span.classes,
        span.id_bytes,
        bits,
        keep,
    )?;
    Ok(json!({"all": all.json(), "novel": novel.json()}))
}

/// The JSON common to every arm and whether its B3 holds.
fn arm_json(
    span: &SpanInputs,
    score: &ArmScore,
    scored: &[u8],
) -> Result<(Map<String, Value>, bool), Failure> {
    let check = b3(span, score)?;
    let ok = check.last_at_most_first && check.scored_equals_last;
    let mut out = Map::new();
    let fit_bits: Vec<String> = score.fit_bits.iter().map(|b| b.to_string()).collect();
    out.insert("fit_bits_q12".into(), json!(fit_bits));
    out.insert("gap_bound_q12".into(), json!(score.gap_q12.to_string()));
    out.insert(
        "b3".into(),
        json!({"last_at_most_first": check.last_at_most_first,
               "scored_equals_last": check.scored_equals_last, "rises": check.rises}),
    );
    out.insert(
        "fit".into(),
        tally_json(span, &score.bits, &[span.fit_segment])?,
    );
    out.insert("scored".into(), tally_json(span, &score.bits, scored)?);
    let weights: Vec<Value> = score
        .pick_weights
        .iter()
        .map(|w| json!({"bucket": w.bucket, "class": w.class, "fit_targets": w.fit_targets, "weight_q32": w.weight_q32}))
        .collect();
    out.insert("pick_weights".into(), json!(weights));
    Ok((out, ok))
}

struct MainArm {
    json: Value,
    all: Vec<u128>,
    novel: Vec<u128>,
    b3_ok: bool,
}

fn main_arm(
    span: &SpanInputs,
    pick: ArmPick,
    stats: Option<&[PickTarget]>,
) -> Result<MainArm, Failure> {
    let score = fit_arm(span, pick)?;
    let (mut out, b3_ok) = arm_json(span, &score, &[1])?;
    if let Some(rows) = stats {
        let s = pick_stats(span, rows, 1)?;
        out.insert(
            "pick_stats".into(),
            json!({"targets": s.targets, "fired": s.fired, "exact": s.exact}),
        );
    }
    let all = record_bits(
        span.query,
        span.segments,
        span.classes,
        &score.bits,
        1,
        false,
    )?;
    let novel = record_bits(
        span.query,
        span.segments,
        span.classes,
        &score.bits,
        1,
        true,
    )?;
    Ok(MainArm {
        json: Value::Object(out),
        all,
        novel,
        b3_ok,
    })
}

/// One plus-span arm: its JSON, its second-copy bits and whether its B3 holds.
fn plus_arm(span: &SpanInputs, pick: ArmPick) -> Result<(Value, u128, bool), Failure> {
    let score = fit_arm(span, pick)?;
    let (mut out, b3_ok) = arm_json(span, &score, &[3, 4])?;
    let (second, second_novel) = segment_tally(
        span.query,
        span.segments,
        span.classes,
        span.id_bytes,
        &score.bits,
        &[4],
    )?;
    out.insert(
        "second_copies".into(),
        json!({"all": second.json(), "novel": second_novel.json()}),
    );
    Ok((Value::Object(out), second.bits_q12, b3_ok))
}

fn band_side(x: &[u128], y: &[u128]) -> Result<Value, Failure> {
    let (low, high) = paired_band(x, y, BAND_RESAMPLES, BAND_SEED)?;
    Ok(json!({"low": low.to_string(), "high": high.to_string(), "beats": high < 0}))
}

fn band(a1: &MainArm, other: &MainArm) -> Result<Value, Failure> {
    Ok(
        json!({"all": band_side(&a1.all, &other.all)?, "novel": band_side(&a1.novel, &other.novel)?}),
    )
}

pub fn run_arms(
    main: &SpanInputs,
    a1: &[PickTarget],
    c2: &[PickTarget],
    c3: &[PickTarget],
    plus: &SpanInputs,
    plus_a1: &[PickTarget],
) -> Result<Value, Failure> {
    // One arm at a time: each table and score drops inside its own call.
    let arm_a0 = main_arm(main, ArmPick::Absent, None)?;
    let arm_a1 = main_arm(main, ArmPick::Read(a1), Some(a1))?;
    let arm_c1 = main_arm(main, ArmPick::Masked(a1), None)?;
    let arm_c2 = main_arm(main, ArmPick::Read(c2), Some(c2))?;
    let arm_c3 = main_arm(main, ArmPick::Read(c3), Some(c3))?;
    let (plus_a0_json, plus_a0_second, plus_a0_ok) = plus_arm(plus, ArmPick::Absent)?;
    let (plus_a1_json, plus_a1_second, plus_a1_ok) = plus_arm(plus, ArmPick::Read(plus_a1))?;
    let b2 = b2_pass(plus_a1_second, plus_a0_second);
    let bands = json!({
        "a1_minus_a0": band(&arm_a1, &arm_a0)?,
        "a1_minus_c1": band(&arm_a1, &arm_c1)?,
        "a1_minus_c2": band(&arm_a1, &arm_c2)?,
        "a1_minus_c3": band(&arm_a1, &arm_c3)?,
    });
    let b3_all = [&arm_a0, &arm_a1, &arm_c1, &arm_c2, &arm_c3]
        .iter()
        .all(|arm| arm.b3_ok)
        && plus_a0_ok
        && plus_a1_ok;
    Ok(json!({
        "main": {"a0": arm_a0.json, "a1": arm_a1.json, "c1": arm_c1.json,
                 "c2": arm_c2.json, "c3": arm_c3.json},
        "plus": {"a0": plus_a0_json, "a1": plus_a1_json},
        "b2": {"a1_bits_q12": plus_a1_second.to_string(),
               "a0_bits_q12": plus_a0_second.to_string(), "pass": b2},
        "bands": bands,
        "reading_allowed": b2 && b3_all,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: usize = 40;

    struct Owned {
        query: Vec<u16>,
        segments: Vec<u8>,
        classes: Vec<u8>,
        id_bytes: Vec<u8>,
        kn: Vec<u64>,
        counts: Vec<u32>,
        totals: Vec<u64>,
        pick: Vec<PickTarget>,
    }

    impl Owned {
        fn span(&self, fit_segment: u8) -> SpanInputs<'_> {
            SpanInputs {
                highest: 2,
                query: &self.query,
                segments: &self.segments,
                classes: &self.classes,
                id_bytes: &self.id_bytes,
                kn_masses: &self.kn,
                counts: &self.counts,
                totals: &self.totals,
                fit_segment,
            }
        }
    }

    /// One record `[BOS body EOS]` per entry of `record_segments`.
    fn synthetic(record_segments: &[u8]) -> Owned {
        let (mut query, mut segments, mut classes) = (Vec::new(), Vec::new(), Vec::new());
        for &segment in record_segments {
            for p in 0..BODY + 2 {
                let token = match p {
                    0 => 0,
                    p if p == BODY + 1 => 1,
                    p => 2 + (p % 60) as u16,
                };
                query.push(token);
                segments.push(segment);
                classes.push(if p == 0 { 3 } else { (query.len() % 2) as u8 });
            }
        }
        let targets = query.iter().filter(|&&t| t != 0).count();
        let (mut kn, mut counts, mut totals, mut pick) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut s = 12345u64;
        for t in 0..targets as u64 {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let h = (t % 3) as usize;
            let mut row_t = [0u64; 3];
            let mut row_a = [0u32; 3];
            for k in 0..=h {
                let total = 1 + ((s >> (8 * k + 3)) % 200);
                row_t[k] = total;
                row_a[k] = ((s >> (8 * k + 20)) % (total + 1)) as u32;
            }
            kn.push(1 + (s >> 20) % (1 << 44));
            counts.extend_from_slice(&row_a);
            totals.extend_from_slice(&row_t);
            let class = ((t / 3) % 3) as u8;
            pick.push(PickTarget {
                class,
                hit: class != 0 && t % 7 != 0,
            });
        }
        Owned {
            query,
            segments,
            classes,
            id_bytes: vec![1; 64],
            kn,
            counts,
            totals,
            pick,
        }
    }

    fn main_data() -> Owned {
        // Enough records that every fired bucket meets a miss in the fit set.
        let mut records = vec![0u8; 20];
        records.extend([1u8; 20]);
        synthetic(&records)
    }

    fn scored_sum(span: &SpanInputs, score: &ArmScore, segment: u8) -> u128 {
        target_segments(span)
            .unwrap()
            .iter()
            .zip(&score.bits)
            .filter(|(&s, _)| s == segment)
            .map(|(_, &b)| u128::from(b))
            .sum()
    }

    #[test]
    fn a1_beats_a0_on_repeats() {
        let data = main_data();
        let span = data.span(0);
        let a0 = fit_arm(&span, ArmPick::Absent).unwrap();
        let a1 = fit_arm(&span, ArmPick::Read(&data.pick)).unwrap();
        assert!(scored_sum(&span, &a1, 1) < scored_sum(&span, &a0, 1));
    }

    #[test]
    fn c1_pick_weights_are_zero() {
        let data = main_data();
        let c1 = fit_arm(&data.span(0), ArmPick::Masked(&data.pick)).unwrap();
        assert!(!c1.pick_weights.is_empty());
        assert!(c1.pick_weights.iter().all(|w| w.weight_q32 == 0));
    }

    #[test]
    fn b3_holds_and_flags_a_mismatch() {
        let data = main_data();
        let span = data.span(0);
        let mut score = fit_arm(&span, ArmPick::Read(&data.pick)).unwrap();
        let check = b3(&span, &score).unwrap();
        assert!(check.last_at_most_first && check.scored_equals_last);
        score.bits[0] += 1; // target 0 sits in record 0, segment 0
        assert!(!b3(&span, &score).unwrap().scored_equals_last);
    }

    #[test]
    fn run_arms_emits_bands_and_b2() {
        let data = main_data();
        let plus = synthetic(&[2, 2, 2, 3, 3, 4, 4]);
        let out = run_arms(
            &data.span(0),
            &data.pick,
            &data.pick,
            &data.pick,
            &plus.span(2),
            &plus.pick,
        )
        .unwrap();
        for key in ["bands", "b2", "reading_allowed"] {
            assert!(out.get(key).is_some(), "missing {key}");
        }
        for arm in ["a0", "a1", "c1", "c2", "c3"] {
            assert!(out["main"].get(arm).is_some(), "missing arm {arm}");
        }
        for (_, side) in out["bands"].as_object().unwrap() {
            for part in ["all", "novel"] {
                let low: i128 = side[part]["low"].as_str().unwrap().parse().unwrap();
                let high: i128 = side[part]["high"].as_str().unwrap().parse().unwrap();
                assert!(low <= high);
            }
        }
    }

    #[test]
    fn refuses_a_span_with_no_fit_target() {
        let data = synthetic(&[1, 1, 1]);
        assert!(fit_arm(&data.span(0), ArmPick::Absent).is_err());
    }
}
