//! W2 read-code capacity: host counting, scoring and the band decision of the
//! registered sheet `docs/harness/prep/m1-read-code-capacity-2026-10-17.md`
//! (§4.2 hypotheses, §4.5 step 5, §4.6 evaluability, §4.7 decision rule).
//!
//! Pure host post-processing of GPU-produced read codes (D24 "CPU audits
//! stored GPU observations"). Counts and every decision are exact integers:
//! no floating point, no squashing, no matrix product. Each irrational floor
//! `2·sqrt(b+c)` is compared by squaring both sides in integers.

use std::collections::BTreeMap;

mod geometry {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/geometry.rs"));
}

pub const VOCABULARY: usize = 4096;
pub const ROOT_COUNT: usize = 16;
pub const BLOCKS: usize = 4;
pub const BLOCK_WIDTH: usize = 8;
pub const TOP_K: usize = 8;
/// §4.6: rounded 2-SE floor at 149/14,287.
pub const FULL_PANEL_MIN_FLOOR: i64 = 24;
/// §4.6 multiplicity bar: distinct codes with ≥2 matched TRAIN counts.
pub const MIN_CODES_WITH_COUNT_GE_2: u64 = 1024;
/// §4.6 power rule: below this `b+c` the exact McNemar test must reject.
pub const MIN_POWERED_DISCORDANCE: u64 = 100;
/// §4.2: Q4 writer full-panel top-1.
pub const W_WRITER_TOP1: u64 = 149;
pub const MATCHED_TRAIN_ROWS: usize = 1_049_856;
pub const DEV_ROWS: usize = 14_287;
/// §4.5 fallback regime label the consumer verifier requires.
pub const FALLBACK_CAPTURE_MODE: &str = "production_forward_matched_1049856_fallback";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapacityError {
    /// §4.5 step 5 / §4.7 precedence 0: a block matched no root. Instrument fault.
    AbsentRoot { block: [i32; BLOCK_WIDTH] },
    /// §4.8 item 4: no token ≥ 4,096.
    TokenOutOfRange { token: u16 },
}

impl std::fmt::Display for CapacityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AbsentRoot { block } => {
                write!(f, "INSTRUMENT_FAULT: ABSENT root for block {block:?}")
            }
            Self::TokenOutOfRange { token } => {
                write!(f, "INSTRUMENT_FAULT: token {token} >= {VOCABULARY}")
            }
        }
    }
}

impl std::error::Error for CapacityError {}

/// Exact-match root decode, mirroring `root_id` in `discrete.cu`: the first of
/// the 16 geometry roots equal to the block, else ABSENT.
pub fn decode_block(block: &[i32; BLOCK_WIDTH]) -> Result<u8, CapacityError> {
    geometry::ROOTS
        .iter()
        .position(|root| root.iter().zip(block).all(|(&r, &v)| i32::from(r) == v))
        .map(|r| r as u8)
        .ok_or(CapacityError::AbsentRoot { block: *block })
}

/// The 32-int read vector of one root per block.
pub fn root_vector(roots: [u8; BLOCKS]) -> [i32; BLOCKS * BLOCK_WIDTH] {
    let mut q = [0i32; BLOCKS * BLOCK_WIDTH];
    for (b, &r) in roots.iter().enumerate() {
        for d in 0..BLOCK_WIDTH {
            q[b * BLOCK_WIDTH + d] = i32::from(geometry::ROOTS[usize::from(r)][d]);
        }
    }
    q
}

/// `code = r0 | r1<<4 | r2<<8 | r3<<12` (consumer schema §5.2 nibble order).
pub fn pack_code(roots: [u8; BLOCKS]) -> u16 {
    roots
        .iter()
        .enumerate()
        .fold(0u16, |code, (b, &r)| code | (u16::from(r & 0xF) << (4 * b)))
}

pub fn unpack_code(code: u16) -> [u8; BLOCKS] {
    [0, 1, 2, 3].map(|b| ((code >> (4 * b)) & 0xF) as u8)
}

/// Decode a 32-int read vector (four 8-int blocks) to its packed code.
pub fn decode_read_vector(q: &[i32; BLOCKS * BLOCK_WIDTH]) -> Result<u16, CapacityError> {
    let mut roots = [0u8; BLOCKS];
    for (b, root) in roots.iter_mut().enumerate() {
        let block: [i32; BLOCK_WIDTH] = q[b * BLOCK_WIDTH..(b + 1) * BLOCK_WIDTH]
            .try_into()
            .expect("block width");
        *root = decode_block(&block)?;
    }
    Ok(pack_code(roots))
}

/// 8-bit `(first,last)` address key: nibbles 0 and 3.
pub fn address_key(code: u16) -> u16 {
    let r = unpack_code(code);
    u16::from(r[0]) | (u16::from(r[3]) << 4)
}

/// 8-bit `(root1,root2)` interior key: nibbles 1 and 2.
pub fn interior_key(code: u16) -> u16 {
    let r = unpack_code(code);
    u16::from(r[1]) | (u16::from(r[2]) << 4)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrainRow {
    pub code: u16,
    pub query_token: u16,
    pub target_token: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DevRow {
    pub code: u16,
    pub query_token: u16,
    pub target_token: u16,
    /// Writer hits from the production forward, re-verified against the sealed receipt.
    pub writer_hit1: bool,
    pub writer_hit8: bool,
}

/// Sheet tie rule: highest count, then lowest token id; at most `TOP_K` tokens.
fn rank<I: IntoIterator<Item = (u16, u64)>>(entries: I) -> Vec<u16> {
    let mut v: Vec<(u16, u64)> = entries.into_iter().filter(|&(_, c)| c > 0).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v.into_iter().take(TOP_K).map(|(t, _)| t).collect()
}

/// `count[key][token]`, sparse and deterministic.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyedTable {
    counts: BTreeMap<u16, BTreeMap<u16, u64>>,
    total: u64,
}

impl KeyedTable {
    fn add(&mut self, key: u16, token: u16) {
        *self
            .counts
            .entry(key)
            .or_default()
            .entry(token)
            .or_default() += 1;
        self.total += 1;
    }
    pub fn total(&self) -> u64 {
        self.total
    }
    /// Sum over every cell; equals `total()` when counts are conserved.
    pub fn cell_sum(&self) -> u64 {
        self.counts.values().flat_map(|m| m.values()).sum()
    }
    pub fn key_count(&self, key: u16) -> u64 {
        self.counts.get(&key).map_or(0, |m| m.values().sum())
    }
    pub fn distinct_keys(&self) -> u64 {
        self.counts.len() as u64
    }
    pub fn keys_with_count_ge(&self, n: u64) -> u64 {
        self.counts
            .values()
            .filter(|m| m.values().sum::<u64>() >= n)
            .count() as u64
    }
    /// Histogram `key count -> number of keys` (§4.6 count histogram).
    pub fn count_histogram(&self) -> BTreeMap<u64, u64> {
        let mut h = BTreeMap::new();
        for m in self.counts.values() {
            *h.entry(m.values().sum::<u64>()).or_default() += 1;
        }
        h
    }
    /// Majority count of the key's top token (numerator of `code_majority_fraction`).
    pub fn key_majority(&self, key: u16) -> u64 {
        self.counts
            .get(&key)
            .and_then(|m| m.values().max().copied())
            .unwrap_or(0)
    }
    fn ranked(&self, key: u16) -> Option<Vec<u16>> {
        self.counts
            .get(&key)
            .map(|m| rank(m.iter().map(|(&t, &c)| (t, c))))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unigram {
    counts: Vec<u64>,
    total: u64,
}

impl Default for Unigram {
    fn default() -> Self {
        Self {
            counts: vec![0; VOCABULARY],
            total: 0,
        }
    }
}

impl Unigram {
    pub fn total(&self) -> u64 {
        self.total
    }
    pub fn ranked(&self) -> Vec<u16> {
        rank(self.counts.iter().enumerate().map(|(t, &c)| (t as u16, c)))
    }
}

/// The model-matched tables, all built on the same TRAIN rows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tables {
    /// Exact-code table T.
    pub code: KeyedTable,
    /// Query-token bigram G.
    pub bigram: KeyedTable,
    /// Secondary, descriptive: `(first,last)` and `(root1,root2)` tables.
    pub address_only: KeyedTable,
    pub interior_only: KeyedTable,
    /// Model-matched unigram U, also the fallback for every keyed table.
    pub unigram: Unigram,
}

pub fn build_tables(train: &[TrainRow]) -> Result<Tables, CapacityError> {
    let mut t = Tables::default();
    for row in train {
        for token in [row.query_token, row.target_token] {
            if usize::from(token) >= VOCABULARY {
                return Err(CapacityError::TokenOutOfRange { token });
            }
        }
        t.code.add(row.code, row.target_token);
        t.bigram.add(row.query_token, row.target_token);
        t.address_only.add(address_key(row.code), row.target_token);
        t.interior_only
            .add(interior_key(row.code), row.target_token);
        t.unigram.counts[usize::from(row.target_token)] += 1;
        t.unigram.total += 1;
    }
    Ok(t)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prediction {
    pub top8: Vec<u16>,
    pub fallback_used: bool,
}

impl Prediction {
    pub fn top1(&self) -> Option<u16> {
        self.top8.first().copied()
    }
    pub fn hit1(&self, target: u16) -> bool {
        self.top1() == Some(target)
    }
    pub fn hit8(&self, target: u16) -> bool {
        self.top8.contains(&target)
    }
}

/// Keyed prediction with the sheet's fallback: an unseen key uses the
/// model-matched unigram ranking. A seen key's top-8 is its own ranking only
/// (not padded), so top-8 is never inflated by the fallback.
pub fn predict(table: &KeyedTable, key: u16, unigram: &Unigram) -> Prediction {
    match table.ranked(key) {
        Some(top8) => Prediction {
            top8,
            fallback_used: false,
        },
        None => Prediction {
            top8: unigram.ranked(),
            fallback_used: true,
        },
    }
}

/// One DEV row's scored outcome (the host-computed part of the §4.3 per-row record).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevScore {
    pub table: Prediction,
    pub seen_code: bool,
    pub code_train_count: u64,
    pub code_majority_count: u64,
    pub unigram_hit1: bool,
    pub unigram_hit8: bool,
    pub bigram: Prediction,
    pub address_only: Prediction,
    pub interior_only: Prediction,
    pub target_token: u16,
    pub writer_hit1: bool,
    pub writer_hit8: bool,
}

pub fn score_dev(tables: &Tables, dev: &[DevRow]) -> Result<Vec<DevScore>, CapacityError> {
    let uni = tables.unigram.ranked();
    dev.iter()
        .map(|row| {
            for token in [row.query_token, row.target_token] {
                if usize::from(token) >= VOCABULARY {
                    return Err(CapacityError::TokenOutOfRange { token });
                }
            }
            let target = row.target_token;
            let code_train_count = tables.code.key_count(row.code);
            Ok(DevScore {
                table: predict(&tables.code, row.code, &tables.unigram),
                seen_code: code_train_count >= 1,
                code_train_count,
                code_majority_count: tables.code.key_majority(row.code),
                unigram_hit1: uni.first() == Some(&target),
                unigram_hit8: uni.contains(&target),
                bigram: predict(&tables.bigram, row.query_token, &tables.unigram),
                address_only: predict(&tables.address_only, address_key(row.code), &tables.unigram),
                interior_only: predict(
                    &tables.interior_only,
                    interior_key(row.code),
                    &tables.unigram,
                ),
                target_token: target,
                writer_hit1: row.writer_hit1,
                writer_hit8: row.writer_hit8,
            })
        })
        .collect()
}

/// Paired discordance against comparator X: `b` = X hit & table miss, `c` = X miss & table hit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Discordance {
    pub b: u64,
    pub c: u64,
}

impl Discordance {
    pub fn n(&self) -> u64 {
        self.b + self.c
    }
    fn push(&mut self, comparator_hit: bool, table_hit: bool) {
        match (comparator_hit, table_hit) {
            (true, false) => self.b += 1,
            (false, true) => self.c += 1,
            _ => {}
        }
    }
}

/// Exact §4.6 aggregates over the DEV panel.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Aggregates {
    pub dev_rows: u64,
    pub t_top1: u64,
    pub t_top8: u64,
    pub u_top1: u64,
    pub u_top8: u64,
    pub g_top1: u64,
    pub g_top8: u64,
    pub w_top1: u64,
    pub w_top8: u64,
    pub address_only_top1: u64,
    pub interior_only_top1: u64,
    pub coverage_rows: u64,
    pub dev_rows_code_count_ge_2: u64,
    pub fallback_rows: u64,
    pub codes_with_count_ge_2: u64,
    pub distinct_codes: u64,
    pub matched_train_total: u64,
    pub bigram_train_total: u64,
    pub d_u: Discordance,
    pub d_g: Discordance,
    pub d_w: Discordance,
}

impl Aggregates {
    pub fn m_top1(&self) -> u64 {
        self.u_top1.max(self.g_top1)
    }
}

pub fn aggregate(tables: &Tables, scores: &[DevScore]) -> Aggregates {
    let mut a = Aggregates {
        dev_rows: scores.len() as u64,
        codes_with_count_ge_2: tables.code.keys_with_count_ge(2),
        distinct_codes: tables.code.distinct_keys(),
        matched_train_total: tables.code.cell_sum(),
        bigram_train_total: tables.bigram.cell_sum(),
        ..Aggregates::default()
    };
    for s in scores {
        let t = s.target_token;
        let th = s.table.hit1(t);
        let gh = s.bigram.hit1(t);
        a.t_top1 += u64::from(th);
        a.t_top8 += u64::from(s.table.hit8(t));
        a.u_top1 += u64::from(s.unigram_hit1);
        a.u_top8 += u64::from(s.unigram_hit8);
        a.g_top1 += u64::from(gh);
        a.g_top8 += u64::from(s.bigram.hit8(t));
        a.w_top1 += u64::from(s.writer_hit1);
        a.w_top8 += u64::from(s.writer_hit8);
        a.address_only_top1 += u64::from(s.address_only.hit1(t));
        a.interior_only_top1 += u64::from(s.interior_only.hit1(t));
        a.coverage_rows += u64::from(s.seen_code);
        a.dev_rows_code_count_ge_2 += u64::from(s.code_train_count >= 2);
        a.fallback_rows += u64::from(s.table.fallback_used);
        a.d_u.push(s.unigram_hit1, th);
        a.d_g.push(gh, th);
        a.d_w.push(s.writer_hit1, th);
    }
    a
}

/// Exact two-sided binomial (McNemar) test rejects at α = 0.05:
/// `2·Σ_{i≤min(b,c)} C(n,i) ≤ 0.05·2^n  ⇔  40·Σ C(n,i) ≤ 2^n`. Only used for `n < 100`.
pub fn exact_mcnemar_rejects_005(d: Discordance) -> bool {
    let n = d.n();
    if n == 0 {
        return false;
    }
    assert!(n < 128, "exact test is only applied below the b+c=100 bar");
    let k = d.b.min(d.c);
    let (mut tail, mut binom) = (0u128, 1u128);
    for i in 0..=k {
        if i > 0 {
            binom = binom * u128::from(n - i + 1) / u128::from(i);
        }
        tail += binom;
    }
    40u128.saturating_mul(tail) <= (1u128 << n)
}

/// §4.6 coverage bar: `ceil(0.9·n)` rows (12,859 of 14,287).
pub fn coverage_bar(dev_rows: u64) -> u64 {
    (9 * dev_rows).div_ceil(10)
}

/// `d > max(24, 2·sqrt(n))`, exact.
fn exceeds_floor(d: i64, n: u64) -> bool {
    d > FULL_PANEL_MIN_FLOOR && i128::from(d) * i128::from(d) > 4 * i128::from(n)
}

/// `d ≥ max(24, 2·sqrt(n))`, exact.
fn reaches_floor(d: i64, n: u64) -> bool {
    d >= FULL_PANEL_MIN_FLOOR && i128::from(d) * i128::from(d) >= 4 * i128::from(n)
}

/// `max(24, 2·sqrt(n)) + max(24, 2·sqrt(m)) < d`, exact in integers.
fn floor_sum_below(n: u64, m: u64, d: i64) -> bool {
    let d = i128::from(d);
    let rational = |x: u64| 4 * i128::from(x) <= 576; // floor is 24
    let (n4, m4) = (4 * i128::from(n), 4 * i128::from(m));
    match (rational(n), rational(m)) {
        (true, true) => 48 < d,
        (true, false) => d - 24 > 0 && (d - 24) * (d - 24) > m4,
        (false, true) => d - 24 > 0 && (d - 24) * (d - 24) > n4,
        // 2√n + 2√m < d ⇔ d > 0 ∧ r = d² − 4n − 4m > 0 ∧ r² > 64nm
        (false, false) => {
            let r = d * d - n4 - m4;
            d > 0 && r > 0 && r * r > 4 * n4 * m4
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Band {
    Underpowered,
    HLaw,
    HMixed,
    HRep,
    WriterDominates,
}

impl Band {
    /// Verdict label as the consumer verifier spells it.
    pub fn label(self) -> &'static str {
        match self {
            Self::Underpowered => "UNDERPOWERED",
            Self::HLaw => "H_law",
            Self::HMixed => "H_mixed",
            Self::HRep => "H_rep",
            Self::WriterDominates => "WRITER_DOMINATES",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub band: Band,
    /// §4.7 precedence row that fired (1..=5).
    pub precedence: u8,
    /// `M + F < W − H` (§4.2 guard); reported as an analysis note.
    pub pathological_ordering: bool,
    pub underpowered_reasons: Vec<String>,
}

/// §4.7 rows 2–5 (evaluability already passed). `n_ug = max(b_U+c_U, b_G+c_G)`
/// so `F = max(F_U, F_G) = max(24, 2·sqrt(n_ug))`; `H = max(24, 2·sqrt(n_w))`.
pub fn decide_band(t: u64, m: u64, w: u64, n_ug: u64, n_w: u64) -> (u8, Band, bool) {
    let (t, m, w) = (t as i64, m as i64, w as i64);
    // M + F < W − H  ⇔  F + H < W − M
    let pathological = floor_sum_below(n_ug, n_w, w - m);
    // T ≥ W − H  ⇔  W − T ≤ H
    let deficit = w - t;
    let t_ge_w_minus_h = deficit <= 0 || !exceeds_floor(deficit, n_w);
    if pathological && !t_ge_w_minus_h {
        (4, Band::HRep, true)
    } else if exceeds_floor(t - m, n_ug) {
        (2, Band::HLaw, pathological)
    } else if reaches_floor(t - w, n_w) {
        (3, Band::HMixed, pathological)
    } else if t_ge_w_minus_h {
        (4, Band::HRep, pathological)
    } else {
        (5, Band::WriterDominates, pathological)
    }
}

/// §4.6 evaluability; empty means every check passed.
pub fn underpowered_reasons(a: &Aggregates) -> Vec<String> {
    let mut r = Vec::new();
    let bar = coverage_bar(a.dev_rows);
    if a.coverage_rows < bar {
        r.push(format!(
            "coverage {}/{} < {bar}",
            a.coverage_rows, a.dev_rows
        ));
    }
    if a.codes_with_count_ge_2 < MIN_CODES_WITH_COUNT_GE_2 {
        r.push(format!(
            "codes_with_count_ge_2 {} < {MIN_CODES_WITH_COUNT_GE_2}",
            a.codes_with_count_ge_2
        ));
    }
    for (name, d) in [("U", a.d_u), ("G", a.d_g)] {
        if d.n() < MIN_POWERED_DISCORDANCE && !exact_mcnemar_rejects_005(d) {
            r.push(format!(
                "{name}: b+c={} < 100 and exact McNemar does not reject at 0.05",
                d.n()
            ));
        }
    }
    r
}

/// §4.7 in order: evaluability (row 1), then the band (rows 2–5).
pub fn decide(a: &Aggregates) -> Decision {
    let underpowered_reasons = underpowered_reasons(a);
    if !underpowered_reasons.is_empty() {
        return Decision {
            band: Band::Underpowered,
            precedence: 1,
            pathological_ordering: false,
            underpowered_reasons,
        };
    }
    let (precedence, band, pathological_ordering) = decide_band(
        a.t_top1,
        a.m_top1(),
        a.w_top1,
        a.d_u.n().max(a.d_g.n()),
        a.d_w.n(),
    );
    Decision {
        band,
        precedence,
        pathological_ordering,
        underpowered_reasons,
    }
}

/// Outputs of the kernel's signed-permutation `action()` (`discrete.cu`) over
/// every action × root that are not exactly one of the 16 roots. The root set
/// is closed under the 64 actions iff this is 0.
pub fn symmetry_action_non_roots() -> u64 {
    let mut misses = 0u64;
    for a in 0..geometry::AXES.len() / BLOCK_WIDTH {
        for x in &geometry::ROOTS {
            let mut y = [0i8; BLOCK_WIDTH];
            for (d, out) in y.iter_mut().enumerate() {
                let axis = geometry::AXES[a * BLOCK_WIDTH + d];
                let v = x[usize::from(axis & 7)];
                *out = if axis & 8 != 0 { -v } else { v };
            }
            misses += u64::from(!geometry::ROOTS.contains(&y));
        }
    }
    misses
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: u64 = 149;

    #[test]
    fn w2_symmetry_actions_keep_every_root_a_root() {
        assert_eq!(geometry::AXES.len() / BLOCK_WIDTH, 64);
        assert_eq!(symmetry_action_non_roots(), 0);
    }

    #[test]
    fn every_root_round_trips_and_absent_is_an_error() {
        for r in 0..ROOT_COUNT as u8 {
            let roots = [r, (r + 1) % 16, (r + 7) % 16, 15 - r];
            let q = root_vector(roots);
            assert_eq!(decode_read_vector(&q), Ok(pack_code(roots)));
            assert_eq!(unpack_code(pack_code(roots)), roots);
        }
        let zero = [0i32; BLOCK_WIDTH];
        assert_eq!(
            decode_block(&zero),
            Err(CapacityError::AbsentRoot { block: zero })
        );
        let mut q = root_vector([1, 2, 3, 4]);
        q[17] += 1; // corrupt block 2
        assert!(matches!(
            decode_read_vector(&q),
            Err(CapacityError::AbsentRoot { .. })
        ));
    }

    #[test]
    fn code_nibble_order_and_secondary_keys() {
        let code = pack_code([0x1, 0x2, 0x3, 0x4]);
        assert_eq!(code, 0x4321);
        assert_eq!(address_key(code), 0x41);
        assert_eq!(interior_key(code), 0x32);
    }

    fn tr(code: u16, q: u16, t: u16) -> TrainRow {
        TrainRow {
            code,
            query_token: q,
            target_token: t,
        }
    }

    #[test]
    fn tie_rule_is_highest_count_then_lowest_token() {
        // code 7: tokens 9 and 5 tie at 2; token 3 has 1.
        let train = [
            tr(7, 0, 9),
            tr(7, 0, 9),
            tr(7, 0, 5),
            tr(7, 0, 5),
            tr(7, 0, 3),
        ];
        let t = build_tables(&train).unwrap();
        let p = predict(&t.code, 7, &t.unigram);
        assert_eq!(p.top8, vec![5, 9, 3]);
        assert!(!p.fallback_used);
    }

    #[test]
    fn unseen_key_falls_back_to_matched_unigram() {
        let train = [tr(1, 10, 40), tr(2, 11, 40), tr(2, 11, 41)];
        let t = build_tables(&train).unwrap();
        let p = predict(&t.code, 99, &t.unigram);
        assert!(p.fallback_used);
        assert_eq!(p.top8, vec![40, 41]);
        let g = predict(&t.bigram, 12, &t.unigram);
        assert!(g.fallback_used);
        assert_eq!(g.top1(), Some(40));
        let dev = [DevRow {
            code: 99,
            query_token: 11,
            target_token: 41,
            writer_hit1: false,
            writer_hit8: false,
        }];
        let s = score_dev(&t, &dev).unwrap();
        assert!(!s[0].seen_code && s[0].table.fallback_used);
        assert!(!s[0].bigram.fallback_used && s[0].bigram.top1() == Some(40));
    }

    #[test]
    fn counts_conserve_and_tokens_are_bounded() {
        let train: Vec<TrainRow> = (0..1000u16)
            .map(|i| tr(i % 37, i % 101, (i * 7) % 4096))
            .collect();
        let t = build_tables(&train).unwrap();
        for table in [&t.code, &t.bigram, &t.address_only, &t.interior_only] {
            assert_eq!(table.cell_sum(), train.len() as u64);
            assert_eq!(table.total(), train.len() as u64);
            assert_eq!(
                table
                    .count_histogram()
                    .iter()
                    .map(|(k, v)| k * v)
                    .sum::<u64>(),
                1000
            );
        }
        assert_eq!(t.unigram.total(), 1000);
        assert_eq!(
            build_tables(&[tr(0, 0, 4096)]),
            Err(CapacityError::TokenOutOfRange { token: 4096 })
        );
    }

    #[test]
    fn aggregates_count_paired_discordance() {
        // code 1 -> 50, code 2 -> 60; bigram query 7 -> 50 (3 vs 2); unigram top-1 = 50.
        let mut train = vec![tr(1, 7, 50); 3];
        train.push(tr(2, 7, 60));
        train.push(tr(3, 7, 60));
        let t = build_tables(&train).unwrap();
        let dev = [
            DevRow {
                code: 1,
                query_token: 7,
                target_token: 50,
                writer_hit1: true,
                writer_hit8: true,
            },
            DevRow {
                code: 2,
                query_token: 7,
                target_token: 60,
                writer_hit1: false,
                writer_hit8: true,
            },
            DevRow {
                code: 1,
                query_token: 7,
                target_token: 60,
                writer_hit1: true,
                writer_hit8: true,
            },
        ];
        let a = aggregate(&t, &score_dev(&t, &dev).unwrap());
        assert_eq!((a.t_top1, a.u_top1, a.g_top1, a.w_top1), (2, 1, 1, 2));
        assert_eq!(a.d_u, Discordance { b: 0, c: 1 });
        assert_eq!(a.d_g, Discordance { b: 0, c: 1 });
        assert_eq!(a.d_w, Discordance { b: 1, c: 1 });
        assert_eq!(
            (
                a.coverage_rows,
                a.codes_with_count_ge_2,
                a.matched_train_total
            ),
            (3, 1, 5)
        );
    }

    /// Normal ordering, rational floors: n ≤ 144 so F = H = 24.
    /// M + F = 1024, W + H = 173, W − H = 125.
    #[test]
    fn band_boundaries_are_strict_as_registered() {
        let band = |t| decide_band(t, 1000, W, 100, 100).1;
        assert_eq!(band(1025), Band::HLaw);
        assert_eq!(band(1024), Band::HMixed, "T = M + F is never H_law");
        assert_eq!(band(174), Band::HMixed);
        assert_eq!(band(173), Band::HMixed, "T = W + H is H_mixed");
        assert_eq!(band(172), Band::HRep);
        assert_eq!(band(149), Band::HRep);
        assert_eq!(band(126), Band::HRep);
        assert_eq!(band(125), Band::HRep, "T = W - H is H_rep");
        assert_eq!(band(124), Band::WriterDominates);
        assert_eq!(band(0), Band::WriterDominates);
    }

    /// Irrational floors: F = 2·sqrt(401) ≈ 40.05, H = 2·sqrt(401).
    #[test]
    fn band_boundaries_with_paired_sqrt_floors() {
        let band = |t| decide_band(t, 1000, W, 401, 401).1;
        assert_eq!(band(1041), Band::HLaw);
        assert_eq!(band(1040), Band::HMixed);
        assert_eq!(band(190), Band::HMixed);
        assert_eq!(band(189), Band::HRep);
        assert_eq!(band(109), Band::HRep);
        assert_eq!(band(108), Band::WriterDominates);
        // F = 40 exactly (n = 400): T = M + 40 is not H_law.
        assert_eq!(decide_band(1040, 1000, W, 400, 100).1, Band::HMixed);
        assert_eq!(decide_band(1041, 1000, W, 400, 100).1, Band::HLaw);
    }

    #[test]
    fn empty_mixed_interval_and_pathological_guard() {
        // M + F = 134 < W + H = 173: no H_mixed; T = 150 > M + F is H_law.
        assert_eq!(decide_band(150, 110, W, 100, 100), (2, Band::HLaw, false));
        // M + F = 74 < W − H = 125: pathological. T = 100 > M + F but T < W − H → H_rep.
        assert_eq!(decide_band(100, 50, W, 100, 100), (4, Band::HRep, true));
        assert_eq!(decide_band(124, 50, W, 100, 100), (4, Band::HRep, true));
        assert_eq!(decide_band(125, 50, W, 100, 100), (2, Band::HLaw, true));
        // Guard boundary: F + H = 48 against W − M.
        assert!(!floor_sum_below(100, 100, 48));
        assert!(floor_sum_below(100, 100, 49));
        // 2√401 + 2√401 = 4√401 ≈ 80.1
        assert!(!floor_sum_below(401, 401, 80));
        assert!(floor_sum_below(401, 401, 81));
        // 24 + 2√401 ≈ 64.05
        assert!(!floor_sum_below(100, 401, 64));
        assert!(floor_sum_below(401, 100, 65));
    }

    fn powered() -> Aggregates {
        Aggregates {
            dev_rows: DEV_ROWS as u64,
            t_top1: 1100,
            u_top1: 1000,
            g_top1: 900,
            w_top1: W,
            coverage_rows: 12_859,
            codes_with_count_ge_2: 1024,
            d_u: Discordance { b: 50, c: 50 },
            d_g: Discordance { b: 60, c: 60 },
            d_w: Discordance { b: 40, c: 60 },
            ..Aggregates::default()
        }
    }

    #[test]
    fn underpowered_takes_precedence() {
        let a = powered();
        assert_eq!(decide(&a).band, Band::HLaw);
        let mut c = a.clone();
        c.coverage_rows = 12_858;
        assert_eq!(coverage_bar(DEV_ROWS as u64), 12_859);
        assert_eq!(
            (decide(&c).band, decide(&c).precedence),
            (Band::Underpowered, 1)
        );
        let mut m = a.clone();
        m.codes_with_count_ge_2 = 1023;
        assert_eq!(decide(&m).band, Band::Underpowered);
        // b+c = 99, balanced: exact test cannot reject.
        let mut p = a.clone();
        p.d_g = Discordance { b: 49, c: 50 };
        assert_eq!(decide(&p).band, Band::Underpowered);
        // b+c = 20, lopsided: exact test rejects, so evaluable.
        let mut q = a.clone();
        q.d_u = Discordance { b: 0, c: 20 };
        assert!(underpowered_reasons(&q).is_empty());
    }

    #[test]
    fn exact_mcnemar_matches_known_cutoffs() {
        // n = 20: two-sided p for k = 5 is 0.0414, for k = 6 is 0.1153.
        assert!(exact_mcnemar_rejects_005(Discordance { b: 5, c: 15 }));
        assert!(!exact_mcnemar_rejects_005(Discordance { b: 6, c: 14 }));
        assert!(!exact_mcnemar_rejects_005(Discordance { b: 0, c: 0 }));
        assert!(!exact_mcnemar_rejects_005(Discordance { b: 0, c: 5 }));
        assert!(exact_mcnemar_rejects_005(Discordance { b: 0, c: 6 }));
    }
}
