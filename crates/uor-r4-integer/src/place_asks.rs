//! The asks (claude-359): an exact find for the place register, fitted by counting.
//!
//! The place register ([`crate::fbs_snapshot::PlaceState`]) reads a told value from the position
//! it holds. This module answers where that position is, from tokens alone. A candidate place is
//! a position in a finished user turn whose token before it has stood before a true place in the
//! fitted split. A candidate's frame is the [`ASKS_FRAME`] tokens before it. Its score is a sum of
//! fitted integer weights: for each frame token, one weight when the question holds that token
//! too and another when it does not; and for each pair of a question token and a frame token, how
//! much more often the question holds that token at a true place than at any candidate. A place
//! is found when a candidate's score reaches the fitted floor.
//!
//! A second table scores whether a question asks for a stored fact at all. When it does and no
//! place is found, the answer is [`Asked::NotTold`], and the register reads the stored reply.
//!
//! Every weight is a difference of [`lg`] values of counts; the read is adds and compares.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};
use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::fbs_snapshot::{PlaceMarks, PlaceRole, PlaceTurns};

/// The tokens before a candidate place that are read as its frame.
pub const ASKS_FRAME: usize = 5;
/// A token may stand before a place when it did so at least this often in the fitted split.
const LINK_MIN: u32 = 3;
/// A frame token has its own weights when it was counted at least this often.
const SHARED_MIN: u32 = 8;
/// A pair has a weight when it stood at a true place at least this often.
const PAIR_MIN: u32 = 3;
/// A pair's weight is kept when it is at least this far from zero (half a bit).
const PAIR_KEEP: i32 = 8;
/// A question token has an asking weight when it was counted at least this often.
const ASKING_MIN: u32 = 8;
/// A reply is read as a form up to this many tokens.
const REPLY_CAP: usize = 64;
/// A stored-reply form is kept when it is at least one in this many of its kind.
const FORM_SHARE: u32 = 100;
/// Scores are binned inside this bound when the floors are fitted.
const LIM: i32 = 16_384;

/// A multiply-and-rotate hasher for the packed integer keys of the count tables.
#[derive(Clone, Copy, Default)]
pub struct Mix(u64);

impl Hasher for Mix {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_u64(u64::from(b));
        }
    }
    fn write_u64(&mut self, v: u64) {
        self.0 = (self.0.rotate_left(5) ^ v).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    }
    fn write_u32(&mut self, v: u32) {
        self.write_u64(u64::from(v));
    }
    fn write_usize(&mut self, v: usize) {
        self.write_u64(v as u64);
    }
}

type Map<K, V> = HashMap<K, V, BuildHasherDefault<Mix>>;

/// Sixteen times the base-two logarithm of `n`, from its leading bit and the four bits after it.
pub fn lg(n: u64) -> i32 {
    let n = n.max(1);
    let k = 63 - n.leading_zeros();
    let next = if k >= 4 { (n >> (k - 4)) & 15 } else { (n << (4 - k)) & 15 };
    (16 * k + next as u32) as i32
}

/// How much likelier a count is on the true side than on the other, in [`lg`] units: `t` of
/// `true_total` against `o` of `other_total`, each count raised by one.
fn odds(t: u32, true_total: u64, o: u32, other_total: u64) -> i32 {
    lg((u64::from(t) + 1) * (other_total + 1)) - lg((u64::from(o) + 1) * (true_total + 1))
}

fn key2(j: usize, c: u32) -> u64 {
    ((j as u64) << 16) | u64::from(c)
}

fn key3(q: u32, j: usize, c: u32) -> u64 {
    (u64::from(q) << 24) | key2(j, c)
}

/// The fitted tables, as a caller hands them to a session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlaceAsks {
    /// The hash of the token file the tables were fitted on.
    pub tokens_sha256: String,
    pub marks: PlaceMarks,
    pub vocab: u32,
    /// The tokens that may stand before a place; `vocab` stands for a turn's start.
    pub link: Vec<u32>,
    /// `[offset, frame token, weight when the question holds it, weight when it does not]`.
    pub shared: Vec<[i32; 4]>,
    /// For each offset, the two weights of a frame token with no row in `shared`.
    pub unseen: Vec<[i32; 2]>,
    /// `[question token, offset, frame token, weight]`.
    pub pairs: Vec<[i32; 4]>,
    /// The least score a place needs.
    pub floor: i32,
    /// Whether the latest candidate at or above the floor is the place; otherwise the best one,
    /// the later on a tie.
    pub latest: bool,
    /// `[question token, weight]`: whether a question asks for a stored fact.
    pub asking: Vec<[i32; 2]>,
    pub asking_prior: i32,
    pub asking_floor: i32,
    /// The stored reply for a fact never told, its end mark included.
    pub not_told: Vec<u32>,
    /// Every form the fitted split gives that reply in, the stored one first.
    pub forms: Vec<Vec<u32>>,
}

/// What a question finds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asked {
    /// A place, and the end of the turn content it stands in.
    Place(usize, usize),
    /// The question asks for a stored fact and no place holds it.
    NotTold,
    Nothing,
}

/// The tables as a session reads them.
#[derive(Clone, Debug)]
pub struct AsksIndex {
    start: u32,
    link: Vec<bool>,
    shared: Map<u64, [i32; 2]>,
    unseen: [[i32; 2]; ASKS_FRAME],
    pairs: Map<u64, i32>,
    floor: i32,
    latest: bool,
    asking: Map<u32, i32>,
    asking_prior: i32,
    asking_floor: i32,
    not_told: Vec<u32>,
}

/// The frame of a candidate place `s` in a turn whose content starts at `lo`: the token at each
/// offset before it, nearest first, with `start` standing once for the turn's start.
fn frame(tokens: &[u32], lo: usize, s: usize, start: u32) -> ([u32; ASKS_FRAME], usize) {
    let mut out = [0u32; ASKS_FRAME];
    let mut n = 0;
    while n < ASKS_FRAME {
        if s < lo + n + 1 {
            out[n] = start;
            n += 1;
            break;
        }
        out[n] = tokens[s - n - 1];
        n += 1;
    }
    (out, n)
}

/// A question's distinct tokens, ascending.
fn question(tokens: &[u32], asked: &Range<usize>) -> Vec<u32> {
    let mut q = tokens[asked.clone()].to_vec();
    q.sort_unstable();
    q.dedup();
    q
}

impl AsksIndex {
    pub fn new(asks: &PlaceAsks) -> Result<Self, String> {
        let vocab = asks.vocab;
        if asks.unseen.len() != ASKS_FRAME {
            return Err(format!("the asks have {} unseen rows for a frame of {ASKS_FRAME}", asks.unseen.len()));
        }
        let token = |v: i32| u32::try_from(v).ok().filter(|&t| t <= vocab).ok_or_else(|| format!("the asks name token {v} past a vocabulary of {vocab}"));
        let offset = |v: i32| usize::try_from(v).ok().filter(|j| (1..=ASKS_FRAME).contains(j)).ok_or_else(|| format!("the asks name offset {v} outside the frame"));
        let mut link = vec![false; vocab as usize + 1];
        for &t in &asks.link {
            *link.get_mut(t as usize).ok_or_else(|| format!("the asks link token {t} past a vocabulary of {vocab}"))? = true;
        }
        let mut shared = Map::default();
        for row in &asks.shared {
            shared.insert(key2(offset(row[0])?, token(row[1])?), [row[2], row[3]]);
        }
        let mut pairs = Map::default();
        for row in &asks.pairs {
            pairs.insert(key3(token(row[0])?, offset(row[1])?, token(row[2])?), row[3]);
        }
        let mut asking = Map::default();
        for row in &asks.asking {
            asking.insert(token(row[0])?, row[1]);
        }
        if asks.not_told.last().is_some_and(|&t| t != asks.marks.eos) {
            return Err("the asks' stored reply does not end at the end mark".to_owned());
        }
        let mut unseen = [[0i32; 2]; ASKS_FRAME];
        unseen.copy_from_slice(&asks.unseen);
        Ok(Self {
            start: vocab,
            link,
            shared,
            unseen,
            pairs,
            floor: asks.floor,
            latest: asks.latest,
            asking,
            asking_prior: asks.asking_prior,
            asking_floor: asks.asking_floor,
            not_told: asks.not_told.clone(),
        })
    }

    /// The stored reply for a fact never told; empty when the tables hold none.
    pub fn not_told(&self) -> &[u32] {
        &self.not_told
    }

    /// The score of the candidate place `s` in a turn whose content starts at `lo`, for a
    /// question of the distinct tokens `q`; `None` when `s` is no candidate.
    pub fn score(&self, tokens: &[u32], lo: usize, s: usize, q: &[u32]) -> Option<i32> {
        let (f, n) = frame(tokens, lo, s, self.start);
        if !self.link.get(f[0] as usize).copied().unwrap_or(false) {
            return None;
        }
        let mut score = 0i32;
        for (j, &c) in f[..n].iter().enumerate() {
            let held = q.binary_search(&c).is_ok();
            let side = usize::from(!held);
            score += self.shared.get(&key2(j + 1, c)).map_or(self.unseen[j][side], |w| w[side]);
            for &qt in q {
                if qt != c {
                    if let Some(w) = self.pairs.get(&key3(qt, j + 1, c)) {
                        score += w;
                    }
                }
            }
        }
        Some(score)
    }

    /// The score that a question of the distinct tokens `q` asks for a stored fact.
    pub fn asking(&self, q: &[u32]) -> i32 {
        self.asking_prior + q.iter().filter_map(|t| self.asking.get(t)).sum::<i32>()
    }

    /// Every candidate of the finished user turns `prior`, in order, with its score.
    fn candidates(&self, tokens: &[u32], prior: &[Range<usize>], q: &[u32], mut each: impl FnMut(usize, usize, i32)) {
        for turn in prior {
            for s in turn.clone() {
                if let Some(score) = self.score(tokens, turn.start, s, q) {
                    each(s, turn.end, score);
                }
            }
        }
    }

    /// What the question in `asked` finds among the finished user turns `prior`.
    pub fn find(&self, tokens: &[u32], prior: &[Range<usize>], asked: &Range<usize>) -> Asked {
        let q = question(tokens, asked);
        let mut pick: Option<(i32, usize, usize)> = None;
        self.candidates(tokens, prior, &q, |s, end, score| {
            let take = if self.latest { score >= self.floor } else { pick.map_or(true, |(best, _, _)| score >= best) };
            if take {
                pick = Some((score, s, end));
            }
        });
        match pick {
            Some((score, s, end)) if score >= self.floor => Asked::Place(s, end),
            _ if !self.not_told.is_empty() && self.asking(&q) >= self.asking_floor => Asked::NotTold,
            _ => Asked::Nothing,
        }
    }
}

/// A reply that answers a user turn: that turn's content, the finished user turns before it, and
/// the reply's first row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskCase {
    pub asked: Range<usize>,
    pub prior: Vec<Range<usize>>,
    pub reply: usize,
}

/// Every reply of `tokens` that answers a user turn, as the place register sees it: the first
/// model turn after each user turn.
pub fn ask_cases(tokens: &[u32], marks: &PlaceMarks) -> Vec<AskCase> {
    let mut turns = PlaceTurns::default();
    let mut answered = false;
    let mut out = Vec::new();
    for t in 0..tokens.len() {
        if tokens[t] == marks.bos {
            answered = false;
        }
        match turns.push(&tokens[..=t], marks) {
            Some(PlaceRole::User) => answered = false,
            Some(PlaceRole::Assistant) => {
                if !answered {
                    if let Some((asked, prior)) = turns.users().split_last() {
                        out.push(AskCase {
                            asked: asked.clone(),
                            prior: prior.to_vec(),
                            reply: t + 1,
                        });
                    }
                }
                answered = true;
            }
            Some(PlaceRole::System) | None => {}
        }
    }
    out
}

/// The reply that starts at row `a`, to its end mark.
fn reply_form<'a>(tokens: &'a [u32], a: usize, marks: &PlaceMarks) -> Option<&'a [u32]> {
    let rest = tokens.get(a..)?;
    let end = rest.iter().take(REPLY_CAP).position(|&x| x == marks.eos)?;
    Some(&rest[..=end])
}

fn bin(score: i32) -> usize {
    (score.clamp(-LIM, LIM) + LIM) as usize
}

/// The floor that keeps the most right picks over wrong ones, from each side's change at each
/// bin: the count of picks a floor at that bin keeps is the running sum up to it.
fn best_floor(right: &[i64], wrong: &[i64]) -> (i32, i64, i64) {
    let (mut r, mut w) = (0i64, 0i64);
    let mut best = (i32::MAX, 0i64, 0i64);
    for (i, (dr, dw)) in right.iter().zip(wrong).enumerate() {
        r += dr;
        w += dw;
        if i <= 2 * LIM as usize && r - w >= best.1 - best.2 {
            best = (i as i32 - LIM, r, w);
        }
    }
    best
}

/// What the asks do on a split whose true places are known.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AskCensus {
    /// Replies with a place: found right, found wrong, raised "not told", nothing.
    pub recall: [usize; 4],
    /// Replies that say a fact was not told: a place set, "not told" raised, nothing.
    pub abstain: [usize; 3],
    /// Every other reply: a place set, "not told" raised, nothing.
    pub general: [usize; 3],
}

impl AskCensus {
    pub fn lines(&self, split: &str) -> Vec<String> {
        let (r, a, g) = (self.recall, self.abstain, self.general);
        vec![
            format!("asks-census split={split} recall={} true_place={} wrong_place={} not_told={} nothing={}", r.iter().sum::<usize>(), r[0], r[1], r[2], r[3]),
            format!("asks-census split={split} abstain={} place_set={} not_told={} nothing={}", a.iter().sum::<usize>(), a[0], a[1], a[2]),
            format!("asks-census split={split} general={} place_set={} not_told={} nothing={}", g.iter().sum::<usize>(), g[0], g[1], g[2]),
        ]
    }
}

/// The asks on a split: `places` are its true places, `[reply start, place, ..]`, and a reply
/// with no place counts as one that says a fact was not told when it is one of `forms`. The
/// second value lists up to `show` replies whose place was missed, as token ids.
pub fn census(tokens: &[u32], marks: &PlaceMarks, places: &[[u32; 4]], index: &AsksIndex, forms: &[Vec<u32>], show: usize) -> (AskCensus, Vec<String>) {
    let truth: Map<usize, usize> = places.iter().map(|p| (p[0] as usize, p[1] as usize)).collect();
    let forms: HashSet<&[u32]> = forms.iter().map(Vec::as_slice).collect();
    let mut out = AskCensus::default();
    let mut missed = Vec::new();
    for case in ask_cases(tokens, marks) {
        let found = index.find(tokens, &case.prior, &case.asked);
        match truth.get(&case.reply) {
            Some(&place) => {
                let slot = match found {
                    Asked::Place(s, _) if s == place => 0,
                    Asked::Place(..) => 1,
                    Asked::NotTold => 2,
                    Asked::Nothing => 3,
                };
                out.recall[slot] += 1;
                if slot != 0 && missed.len() < show {
                    let stated = case.prior.iter().find(|turn| turn.contains(&place)).cloned().unwrap_or(place..place);
                    missed.push(format!(
                        "asks-miss {} | asked={:?} | stated={:?} | place_offset={} | got={found:?}",
                        ["right", "wrong", "not_told", "nothing"][slot],
                        &tokens[case.asked.clone()],
                        &tokens[stated.clone()],
                        place - stated.start
                    ));
                }
            }
            None => {
                let slot = match found {
                    Asked::Place(..) => 0,
                    Asked::NotTold => 1,
                    Asked::Nothing => 2,
                };
                if reply_form(tokens, case.reply, marks).is_some_and(|form| forms.contains(form)) {
                    out.abstain[slot] += 1;
                } else {
                    out.general[slot] += 1;
                }
            }
        }
    }
    (out, missed)
}

/// Fit the asks on a split by counting: `places` are its true places, `[reply start, place, ..]`
/// in the split's positions. The second value is the fit's own account, line by line.
pub fn fit(tokens: &[u32], marks: &PlaceMarks, vocab: u32, places: &[[u32; 4]], tokens_sha256: String) -> Result<(PlaceAsks, Vec<String>), String> {
    if vocab >= 1 << 16 {
        return Err(format!("a vocabulary of {vocab} does not fit the asks' keys"));
    }
    if let Some(&t) = tokens.iter().find(|&&t| t >= vocab) {
        return Err(format!("token {t} is past a vocabulary of {vocab}"));
    }
    let start = vocab;
    let truth: Map<usize, usize> = places.iter().map(|p| (p[0] as usize, p[1] as usize)).collect();
    let cases = ask_cases(tokens, marks);
    let placed = cases.iter().filter(|c| truth.contains_key(&c.reply)).count();
    if placed != places.len() {
        return Err(format!("{} of the kit's {} places start a reply the turns give", placed, places.len()));
    }
    let mut lines = vec![format!("asks-fit replies={} with_a_place={placed}", cases.len())];

    // The tokens that may stand before a place.
    let mut before = vec![0u32; vocab as usize + 1];
    let mut outside = 0usize;
    for case in &cases {
        let Some(&place) = truth.get(&case.reply) else { continue };
        match case.prior.iter().find(|turn| turn.contains(&place)) {
            Some(turn) => before[frame(tokens, turn.start, place, start).0[0] as usize] += 1,
            None => outside += 1,
        }
    }
    let link: Vec<bool> = before.iter().map(|&n| n >= LINK_MIN).collect();
    lines.push(format!("asks-fit link tokens={} place_outside_the_earlier_user_turns={outside}", link.iter().filter(|&&on| on).count()));

    // The pairs that stand at a true place, and the questions that have one.
    let mut pair: Map<u64, [u32; 2]> = Map::default();
    let mut recall_questions: HashSet<&[u32]> = HashSet::new();
    for case in &cases {
        let Some(&place) = truth.get(&case.reply) else { continue };
        recall_questions.insert(&tokens[case.asked.clone()]);
        let Some(turn) = case.prior.iter().find(|turn| turn.contains(&place)) else { continue };
        let (f, n) = frame(tokens, turn.start, place, start);
        if !link[f[0] as usize] {
            continue;
        }
        let q = question(tokens, &case.asked);
        for (j, &c) in f[..n].iter().enumerate() {
            for &qt in &q {
                if qt != c {
                    pair.entry(key3(qt, j + 1, c)).or_default()[0] += 1;
                }
            }
        }
    }

    // Every candidate of every reply: the frame tokens by side, and the pairs away from a place.
    // `[held at a true place, not held there, held at another candidate, not held there]`.
    let mut shared: Map<u64, [u32; 4]> = Map::default();
    let (mut true_total, mut other_total) = (0u64, 0u64);
    let mut asking: Map<u32, [u32; 2]> = Map::default();
    let mut asking_total = [0u64; 2];
    let mut forms: HashMap<&[u32], u32> = HashMap::new();
    let asks_a_fact = |case: &AskCase| truth.contains_key(&case.reply) || recall_questions.contains(&tokens[case.asked.clone()]);
    for case in &cases {
        let q = question(tokens, &case.asked);
        let place = truth.get(&case.reply).copied();
        let side = usize::from(!asks_a_fact(case));
        asking_total[side] += 1;
        for &qt in &q {
            asking.entry(qt).or_default()[side] += 1;
        }
        if place.is_none() && side == 0 {
            if let Some(form) = reply_form(tokens, case.reply, marks) {
                *forms.entry(form).or_default() += 1;
            }
        }
        for turn in &case.prior {
            for s in turn.clone() {
                let (f, n) = frame(tokens, turn.start, s, start);
                if !link[f[0] as usize] {
                    continue;
                }
                let is_true = place == Some(s);
                if is_true {
                    true_total += 1;
                } else {
                    other_total += 1;
                }
                for (j, &c) in f[..n].iter().enumerate() {
                    let held = q.binary_search(&c).is_ok();
                    shared.entry(key2(j + 1, c)).or_default()[usize::from(!held) + 2 * usize::from(!is_true)] += 1;
                    if !is_true {
                        for &qt in &q {
                            if qt != c {
                                if let Some(seen) = pair.get_mut(&key3(qt, j + 1, c)) {
                                    seen[1] += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    lines.push(format!("asks-fit candidates true={true_total} other={other_total} frame_tokens={} pairs_at_a_true_place={}", shared.len(), pair.len()));

    // The weights.
    let mut shared_rows = Vec::new();
    let mut low = [[0u32; 4]; ASKS_FRAME];
    for (&key, n) in &shared {
        let (j, c) = ((key >> 16) as usize, (key & 0xFFFF) as i32);
        if n.iter().sum::<u32>() >= SHARED_MIN {
            shared_rows.push([j as i32, c, odds(n[0], true_total, n[2], other_total), odds(n[1], true_total, n[3], other_total)]);
        } else {
            for (sum, count) in low[j - 1].iter_mut().zip(n) {
                *sum += count;
            }
        }
    }
    shared_rows.sort_unstable();
    let unseen: Vec<[i32; 2]> = low.iter().map(|n| [odds(n[0], true_total, n[2], other_total), odds(n[1], true_total, n[3], other_total)]).collect();
    let mut pair_rows = Vec::new();
    for (&key, seen) in &pair {
        if seen[0] < PAIR_MIN {
            continue;
        }
        let base = shared.get(&(key & 0xFF_FFFF)).copied().unwrap_or_default();
        let weight = odds(seen[0], u64::from(base[0] + base[1]) + 1, seen[1], u64::from(base[2] + base[3]) + 1);
        if weight.abs() >= PAIR_KEEP {
            pair_rows.push([(key >> 24) as i32, ((key >> 16) & 0xFF) as i32, (key & 0xFFFF) as i32, weight]);
        }
    }
    pair_rows.sort_unstable();
    let mut asking_rows: Vec<[i32; 2]> = asking
        .iter()
        .filter(|(_, n)| n[0] + n[1] >= ASKING_MIN)
        .map(|(&t, n)| [t as i32, odds(n[0], asking_total[0], n[1], asking_total[1])])
        .collect();
    asking_rows.sort_unstable();
    let asking_prior = lg(asking_total[0] + 1) - lg(asking_total[1] + 1);
    let mut ranked: Vec<(&[u32], u32)> = forms.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let said: u32 = ranked.iter().map(|(_, n)| n).sum();
    let kept: Vec<Vec<u32>> = ranked.iter().filter(|(_, n)| n * FORM_SHARE >= said).map(|(form, _)| form.to_vec()).collect();
    lines.push(format!(
        "asks-fit weights shared={} pairs={} asking={} asks_a_fact={} other={} not_told_replies={said} forms_kept={} top_forms={:?}",
        shared_rows.len(),
        pair_rows.len(),
        asking_rows.len(),
        asking_total[0],
        asking_total[1],
        kept.len(),
        ranked.iter().take(4).map(|(_, n)| *n).collect::<Vec<_>>()
    ));
    let mut asks = PlaceAsks {
        tokens_sha256,
        marks: marks.clone(),
        vocab,
        link: (0..=vocab).filter(|&t| link[t as usize]).collect(),
        shared: shared_rows,
        unseen,
        pairs: pair_rows,
        floor: 0,
        latest: false,
        asking: asking_rows,
        asking_prior,
        asking_floor: 0,
        not_told: kept.first().cloned().unwrap_or_default(),
        forms: kept,
    };

    // The floors, from every reply's candidates scored with the weights.
    let index = AsksIndex::new(&asks)?;
    let bins = 2 * LIM as usize + 2;
    // For each rule, each side's change at each bin. A pick with score `a` over a running best of
    // `m` is kept by every floor in `(m, a]`.
    let mut best = [vec![0i64; bins], vec![0i64; bins]];
    let mut late = [vec![0i64; bins], vec![0i64; bins]];
    let mut asked_hist = [vec![0i64; bins], vec![0i64; bins]];
    let mut scored: Vec<(i32, bool)> = Vec::new();
    for case in &cases {
        let q = question(tokens, &case.asked);
        let place = truth.get(&case.reply).copied();
        let side = usize::from(!asks_a_fact(case));
        asked_hist[side][0] += 1;
        asked_hist[side][bin(index.asking(&q)) + 1] -= 1;
        scored.clear();
        index.candidates(tokens, &case.prior, &q, |s, _, score| scored.push((score, place == Some(s))));
        // The later candidate wins a tie, as the find takes it.
        if let Some(&(score, right)) = scored.iter().max_by_key(|(score, _)| *score) {
            best[usize::from(!right)][0] += 1;
            best[usize::from(!right)][bin(score) + 1] -= 1;
        }
        let mut over = -LIM - 1;
        for &(score, right) in scored.iter().rev() {
            let score = score.clamp(-LIM, LIM);
            if score > over {
                late[usize::from(!right)][(over + LIM + 1) as usize] += 1;
                late[usize::from(!right)][bin(score) + 1] -= 1;
                over = score;
            }
        }
    }
    let (best_at, best_right, best_wrong) = best_floor(&best[0], &best[1]);
    let (late_at, late_right, late_wrong) = best_floor(&late[0], &late[1]);
    lines.push(format!("asks-fit floor rule=best floor={best_at} right={best_right} wrong={best_wrong}"));
    lines.push(format!("asks-fit floor rule=latest floor={late_at} right={late_right} wrong={late_wrong}"));
    asks.latest = late_right - late_wrong > best_right - best_wrong;
    asks.floor = if asks.latest { late_at } else { best_at };
    let (asking_at, asks_right, asks_wrong) = best_floor(&asked_hist[0], &asked_hist[1]);
    asks.asking_floor = asking_at;
    lines.push(format!("asks-fit asking floor={asking_at} asks_a_fact_kept={asks_right} other_kept={asks_wrong}"));
    lines.push(format!("asks-fit decide rule={} floor={}", if asks.latest { "latest" } else { "best" }, asks.floor));
    Ok((asks, lines))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOS: u32 = 0;
    const EOS: u32 = 1;
    const SEP: u32 = 2;
    const USER: u32 = 4;
    const MODEL: u32 = 5;
    const SAYS: u32 = 10;
    const IS: u32 = 11;
    const WHAT: u32 = 12;
    const OK: u32 = 13;
    const NOT: u32 = 14;
    const TOLD: u32 = 15;
    const VOCAB: u32 = 64;

    fn marks() -> PlaceMarks {
        PlaceMarks {
            bos: BOS,
            eos: EOS,
            sep: vec![SEP],
            system: vec![3],
            user: vec![USER],
            assistant: vec![MODEL],
        }
    }

    /// Documents that each state two keyed values in two user turns and then ask for one key:
    /// a stated one, an unstated one, or a general question. Returns the tokens and the places.
    fn corpus(docs: usize) -> (Vec<u32>, Vec<[u32; 4]>) {
        let (mut x, mut places) = (Vec::new(), Vec::new());
        for d in 0..docs {
            let keys = [20 + (d % 5) as u32, 20 + ((d + 2) % 5) as u32];
            let values = [30 + (d % 7) as u32, 40 + (d % 3) as u32];
            x.push(BOS);
            let mut at = [0usize; 2];
            for i in 0..2 {
                x.extend([USER, SAYS, keys[i], IS]);
                at[i] = x.len();
                x.extend([values[i], SEP, MODEL, OK, EOS, SEP]);
            }
            match d % 4 {
                0 | 1 => {
                    let i = d % 2;
                    x.extend([USER, WHAT, keys[i], SEP, MODEL]);
                    places.push([x.len() as u32, at[i] as u32, x.len() as u32, 1]);
                    x.extend([values[i], EOS, SEP]);
                }
                2 => {
                    let other = 20 + ((d + 1) % 5) as u32;
                    x.extend([USER, WHAT, other, SEP, MODEL, NOT, TOLD, EOS, SEP]);
                }
                _ => x.extend([USER, 16, 17, SEP, MODEL, 18, EOS, SEP]),
            }
        }
        (x, places)
    }

    #[test]
    fn lg_rises_with_its_argument_and_doubles_by_sixteen() {
        assert_eq!(lg(1), 0);
        assert_eq!(lg(2), 16);
        assert_eq!(lg(3), 24);
        assert_eq!(lg(1 << 20), 320);
        assert!((1..4096u64).all(|n| lg(n) <= lg(n + 1)));
    }

    #[test]
    fn a_fitted_table_finds_the_stated_value_and_raises_not_told() {
        let (x, places) = corpus(240);
        let (asks, lines) = fit(&x, &marks(), VOCAB, &places, "test".to_owned()).expect("the fit");
        assert_eq!(asks.not_told, vec![NOT, TOLD, EOS], "{lines:#?}");
        let index = AsksIndex::new(&asks).expect("the index");
        let (held, held_places) = corpus(60);
        let (counts, missed) = census(&held, &marks(), &held_places, &index, &asks.forms, 4);
        assert_eq!(counts.recall, [30, 0, 0, 0], "{missed:#?} {lines:#?}");
        assert_eq!(counts.abstain, [0, 15, 0]);
        assert_eq!(counts.general, [0, 0, 15 + 120]);
    }

    #[test]
    fn the_tables_round_trip_and_refuse_a_token_past_the_vocabulary() {
        let (x, places) = corpus(240);
        let (asks, _) = fit(&x, &marks(), VOCAB, &places, "test".to_owned()).expect("the fit");
        let text = serde_json::to_string(&asks).expect("written");
        assert_eq!(serde_json::from_str::<PlaceAsks>(&text).expect("read"), asks);
        let mut bad = asks.clone();
        bad.link.push(VOCAB + 1);
        assert!(AsksIndex::new(&bad).is_err());
        let mut bad = asks;
        bad.not_told = vec![NOT, TOLD];
        assert!(AsksIndex::new(&bad).is_err());
    }
}
