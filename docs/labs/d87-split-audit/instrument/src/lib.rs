//! The D87 donor corpus split by prompt, deduplicated and audited (claude-411).
//!
//! Split key: a keyed SHA-256 of the bank's `normalize` of a record's FIRST user message.
//! D87 records carry no prompt-id field; the bank (claude-400) derives each id from that
//! same normalized text, so every alternative or re-generation of one prompt shares the key.
//! The bucket is the key's first 8 bytes (big-endian) modulo the ratios' common denominator.
//!
//! Duplicates, judged in a fixed order (shard paths sorted, then line order), first kept:
//! exact (SHA-256 of the whole line), repeated id, and near (SHA-256 of the normalized
//! answer text of every turn, when it is at least `near_floor` bytes long).
//!
//! Audit: every 13-gram of a record's token stream (BOS..EOS, never across records) is packed
//! exactly, 13 ids of 12 bits in three u64 words, so a match is an exact 13-gram compare.
//! Each DEV position (the last token of a 13-gram) counts once if its gram occurs anywhere in
//! TRAIN. All counts are integers; ratios are [numerator, denominator] pairs.

use std::collections::{BTreeMap, HashSet};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;

use m2_student_corpus_ingest::{hex, parse_record, read_shard, Coder, Ingest, SEGMENTS, UORT_HEADER};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// The n of the overlap census.
pub const GRAM: usize = 13;
/// Bits per token id in a packed gram (ids are below 4,096).
const ID_BITS: u32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Split {
    Train,
    DevTune,
    DevTest,
}
pub const SPLITS: [Split; 3] = [Split::Train, Split::DevTune, Split::DevTest];
impl Split {
    pub fn name(self) -> &'static str {
        match self {
            Split::Train => "train",
            Split::DevTune => "dev-tune",
            Split::DevTest => "dev-test",
        }
    }
    fn ix(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dup {
    Exact,
    Id,
    Near,
}
const DUPS: [Dup; 3] = [Dup::Exact, Dup::Id, Dup::Near];
impl Dup {
    fn name(self) -> &'static str {
        match self {
            Dup::Exact => "exact",
            Dup::Id => "id",
            Dup::Near => "near",
        }
    }
}

/// The bank's prompt normalization (`docs/labs/m2-donor-corpus/instrument/src/prompts.rs`),
/// copied: alphanumerics lowercased, everything else a space, whitespace runs collapsed.
pub fn normalize(s: &str) -> String {
    let mapped: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { ' ' })
        .collect();
    mapped.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn sha(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}

/// Split parameters. Ratios are integer fractions of prompts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub key: String,
    pub dev_tune: (u64, u64),
    pub dev_test: (u64, u64),
    pub near_floor: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            key: "claude-411/d87-split/v1".into(),
            dev_tune: (1, 200),
            dev_test: (1, 100),
            near_floor: 32,
        }
    }
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// A validated config: the bucket modulus and the two cut points.
#[derive(Clone, Debug)]
pub struct Splitter {
    pub cfg: Config,
    pub modulus: u64,
    test_cut: u64,
    tune_cut: u64,
}

impl Splitter {
    pub fn new(cfg: Config) -> Result<Self, String> {
        let (tn, td) = cfg.dev_tune;
        let (sn, sd) = cfg.dev_test;
        if td == 0 || sd == 0 || tn > td || sn > sd {
            return Err("ratios must be N/D with 0 <= N <= D and D > 0".into());
        }
        let modulus = td / gcd(td, sd) * sd;
        let test_cut = sn * (modulus / sd);
        let tune_cut = test_cut + tn * (modulus / td);
        if tune_cut > modulus {
            return Err("DEV-tune plus DEV-test exceed all prompts".into());
        }
        Ok(Self { cfg, modulus, test_cut, tune_cut })
    }

    /// The prompt key and split of a record whose first user message is `first_user`.
    pub fn split_of(&self, first_user: &str) -> ([u8; 32], Split) {
        let key = sha(&[self.cfg.key.as_bytes(), &[0], normalize(first_user).as_bytes()]);
        let bucket = u64::from_be_bytes(key[..8].try_into().unwrap()) % self.modulus;
        let split = if bucket < self.test_cut {
            Split::DevTest
        } else if bucket < self.tune_cut {
            Split::DevTune
        } else {
            Split::Train
        };
        (key, split)
    }

    pub fn config_json(&self) -> Value {
        let c = &self.cfg;
        json!({"key": c.key, "dev_tune": [c.dev_tune.0, c.dev_tune.1],
               "dev_test": [c.dev_test.0, c.dev_test.1], "near_floor": c.near_floor,
               "bucket_modulus": self.modulus})
    }

    pub fn from_config_json(v: &Value) -> Result<Self, String> {
        let pair = |k: &str| -> Result<(u64, u64), String> {
            let a = v[k].as_array().ok_or(format!("config.{k}"))?;
            match (a.first().and_then(Value::as_u64), a.get(1).and_then(Value::as_u64)) {
                (Some(n), Some(d)) if a.len() == 2 => Ok((n, d)),
                _ => Err(format!("config.{k} is not [N, D]")),
            }
        };
        Self::new(Config {
            key: v["key"].as_str().ok_or("config.key")?.to_string(),
            dev_tune: pair("dev_tune")?,
            dev_test: pair("dev_test")?,
            near_floor: v["near_floor"].as_u64().ok_or("config.near_floor")? as usize,
        })
    }
}

/// What the split needs from one record line.
#[derive(Clone, Debug)]
pub struct Info {
    pub id: String,
    pub kind: String,
    pub source: String,
    pub prompt: [u8; 32],
    pub split: Split,
    line: [u8; 32],
    answer: Option<[u8; 32]>,
}

/// Validate one line as a D87 record (through student-ingest's parser) and key it.
pub fn scan_line(sp: &Splitter, line: &str) -> Result<Info, String> {
    let rec = parse_record(line)?;
    let v: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
    let id = rec.id.clone();
    let source = v["source"].as_str().ok_or(format!("record {id}: not a D87 record"))?;
    let first = v["messages"][0]["content"].as_str().ok_or(format!("record {id}: no first message"))?;
    let turns = v["turns"].as_array().ok_or(format!("record {id}: no turns"))?;
    let answers: Vec<&str> = turns.iter().filter_map(|t| t["answer"].as_str()).collect();
    let norm = normalize(&answers.join(" "));
    let (prompt, split) = sp.split_of(first);
    Ok(Info {
        id,
        kind: rec.kind,
        source: source.to_string(),
        prompt,
        split,
        line: sha(&[line.as_bytes()]),
        answer: (norm.len() >= sp.cfg.near_floor).then(|| sha(&[norm.as_bytes()])),
    })
}

/// Run `f` over shards on `threads` threads; results come back in shard order.
fn par_shards<T: Send>(
    paths: &[PathBuf],
    threads: usize,
    f: impl Fn(&Path) -> Result<T, String> + Sync,
) -> Result<Vec<T>, String> {
    let next = AtomicUsize::new(0);
    let slots: Vec<Mutex<Option<Result<T, String>>>> = paths.iter().map(|_| Mutex::new(None)).collect();
    std::thread::scope(|s| {
        for _ in 0..threads.clamp(1, paths.len().max(1)) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= paths.len() {
                    break;
                }
                let r = f(&paths[i]).map_err(|e| format!("{}: {e}", paths[i].display()));
                *slots[i].lock().unwrap() = Some(r);
            });
        }
    });
    slots.into_iter().map(|m| m.into_inner().unwrap().unwrap()).collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fate {
    Keep(Split),
    Drop(Split, Dup),
}

#[derive(Default)]
struct SplitCount {
    ids: Vec<String>,
    prompts: HashSet<[u8; 32]>,
    kinds: BTreeMap<String, u64>,
    sources: BTreeMap<String, u64>,
    dropped: [u64; 3],
}

/// The split of a set of shards: per shard its sha256 and the fate of every line.
pub struct Plan {
    pub splitter: Splitter,
    pub shards: Vec<(PathBuf, String)>,
    pub fate: Vec<Vec<Fate>>,
    counts: Vec<SplitCount>,
}

pub fn plan(sp: &Splitter, paths: &[PathBuf], threads: usize) -> Result<Plan, String> {
    let mut paths = paths.to_vec();
    paths.sort();
    if paths.windows(2).any(|w| w[0] == w[1]) {
        return Err("a shard is given twice".into());
    }
    let scanned = par_shards(&paths, threads, |p| {
        let mut infos = vec![];
        let sha = read_shard(p, &mut |line| {
            infos.push(scan_line(sp, line)?);
            Ok(())
        })?;
        Ok((sha, infos))
    })?;
    let (mut lines, mut ids, mut answers) = (HashSet::new(), HashSet::new(), HashSet::new());
    let mut counts: Vec<SplitCount> = SPLITS.iter().map(|_| SplitCount::default()).collect();
    let (mut shards, mut fate) = (vec![], vec![]);
    for (p, (sha, infos)) in paths.into_iter().zip(scanned) {
        let mut f = Vec::with_capacity(infos.len());
        for info in infos {
            let dup = if !lines.insert(info.line) {
                Some(Dup::Exact)
            } else if !ids.insert(info.id.clone()) {
                Some(Dup::Id)
            } else if info.answer.is_some_and(|a| !answers.insert(a)) {
                Some(Dup::Near)
            } else {
                None
            };
            let c = &mut counts[info.split.ix()];
            if let Some(d) = dup {
                c.dropped[d as usize] += 1;
                f.push(Fate::Drop(info.split, d));
                continue;
            }
            c.prompts.insert(info.prompt);
            *c.kinds.entry(info.kind).or_insert(0) += 1;
            *c.sources.entry(info.source).or_insert(0) += 1;
            c.ids.push(info.id);
            f.push(Fate::Keep(info.split));
        }
        shards.push((p, sha));
        fate.push(f);
    }
    for c in &mut counts {
        c.ids.sort();
    }
    Ok(Plan { splitter: sp.clone(), shards, fate, counts })
}

impl Plan {
    /// The sorted-id manifest text of one split.
    pub fn manifest(&self, s: Split) -> String {
        self.counts[s.ix()].ids.iter().map(|i| format!("{i}\n")).collect()
    }

    pub fn summary(&self) -> Value {
        let total_records: u64 = self.counts.iter().map(|c| c.ids.len() as u64).sum();
        let total_prompts: u64 = self.counts.iter().map(|c| c.prompts.len() as u64).sum();
        let mut splits = serde_json::Map::new();
        let mut dropped_total = [0u64; 3];
        for s in SPLITS {
            let c = &self.counts[s.ix()];
            let dropped: serde_json::Map<String, Value> =
                DUPS.iter().map(|&d| (d.name().to_string(), json!(c.dropped[d as usize]))).collect();
            for d in DUPS {
                dropped_total[d as usize] += c.dropped[d as usize];
            }
            splits.insert(s.name().into(), json!({
                "records": c.ids.len(), "prompts": c.prompts.len(), "kinds": c.kinds,
                "sources": c.sources, "dropped": dropped,
                "prompt_share": [c.prompts.len(), total_prompts],
                "ids_sha256": hex(&sha(&[self.manifest(s).as_bytes()])),
            }));
        }
        let inputs: Vec<Value> = self.shards.iter().zip(&self.fate)
            .map(|((p, h), f)| json!({"path": p.display().to_string(), "sha256": h, "lines": f.len()}))
            .collect();
        let lines: u64 = self.fate.iter().map(|f| f.len() as u64).sum();
        let dropped: serde_json::Map<String, Value> =
            DUPS.iter().map(|&d| (d.name().to_string(), json!(dropped_total[d as usize]))).collect();
        json!({
            "unit": "claude-411",
            "config": self.splitter.config_json(),
            "rules": {
                "split_key": "sha256(key || 0x00 || normalize(first user message)); bucket = first 8 bytes big-endian mod bucket_modulus; bucket < dev_test cut is dev-test, then dev-tune, else train",
                "order": "shard paths sorted, then line order; the first record is kept",
                "exact": "sha256 of the whole line", "id": "a repeated id",
                "near": "sha256 of normalize(every turn's answer joined by a space), when at least near_floor bytes",
            },
            "inputs": inputs, "lines": lines, "records": total_records, "prompts": total_prompts,
            "dropped": dropped, "splits": splits,
            "ratios": "every pair is an exact integer [numerator, denominator]",
        })
    }

    /// Write `<split>.ids` and `split-summary.json` into `out`.
    pub fn write(&self, out: &Path) -> Result<String, String> {
        for (p, _) in &self.shards {
            if p.parent() == Some(out) {
                return Err(format!("{} is inside --out", p.display()));
            }
        }
        std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
        for s in SPLITS {
            std::fs::write(out.join(format!("{}.ids", s.name())), self.manifest(s)).map_err(|e| e.to_string())?;
        }
        let text = serde_json::to_string_pretty(&self.summary()).map_err(|e| e.to_string())? + "\n";
        std::fs::write(out.join("split-summary.json"), &text).map_err(|e| e.to_string())?;
        Ok(text)
    }
}

/// One tokenized record: student-ingest's stream for it, with segment tags.
pub struct Tok {
    pub tokens: Vec<u32>,
    pub segments: Vec<u8>,
    pub response: u64,
    pub seg_bytes: [u64; 4],
    pub seg_tokens: [u64; 4],
}

pub fn tokenize(coder: &Coder, line: &str) -> Result<Tok, String> {
    let rec = parse_record(line)?;
    let mut ing = Ingest::new(Cursor::new(vec![]), vec![], vec![]).map_err(|e| e.to_string())?;
    ing.add(coder, &rec)?;
    let (t, mask, segments, stats) = ing.finish().map_err(|e| e.to_string())?;
    let t = t.into_inner();
    let tokens = t[UORT_HEADER..].chunks_exact(2).map(|b| u16::from_le_bytes([b[0], b[1]]) as u32).collect();
    let (mut seg_bytes, mut seg_tokens) = ([0u64; 4], [0u64; 4]);
    for s in SEGMENTS {
        let c = &stats.per_segment[&s];
        seg_bytes[s as usize] = c.text_bytes;
        seg_tokens[s as usize] = c.tokens;
    }
    Ok(Tok { tokens, segments, response: mask.iter().filter(|&&m| m == 1).count() as u64, seg_bytes, seg_tokens })
}

/// Pack 13 ids (each below 4,096) exactly into three words.
pub fn pack(ids: &[u32]) -> [u64; 3] {
    debug_assert_eq!(ids.len(), GRAM);
    let mut w = [0u64; 3];
    for (i, &id) in ids.iter().enumerate() {
        debug_assert!(id < 1 << ID_BITS);
        w[i / 5] |= (id as u64) << (ID_BITS * (i % 5) as u32);
    }
    w
}

fn mix(g: &[u64; 3]) -> u64 {
    let mut z = g[0] ^ g[1].rotate_left(21) ^ g[2].rotate_left(42);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Every DEV position of one split: its gram, sorted and deduplicated, with the number of
/// positions per gram by the segment of the gram's last token, and a hit bit per gram.
pub struct GramSet {
    uniq: Vec<[u64; 3]>,
    mult: Vec<[u32; 4]>,
    hit: Vec<AtomicU64>,
}

impl GramSet {
    pub fn new(mut v: Vec<([u64; 3], u8)>) -> Self {
        v.sort_unstable();
        let (mut uniq, mut mult): (Vec<[u64; 3]>, Vec<[u32; 4]>) = (vec![], vec![]);
        for (g, s) in v {
            if uniq.last() != Some(&g) {
                uniq.push(g);
                mult.push([0; 4]);
            }
            mult.last_mut().unwrap()[s as usize] += 1;
        }
        let hit = (0..uniq.len().div_ceil(64)).map(|_| AtomicU64::new(0)).collect();
        Self { uniq, mult, hit }
    }

    pub fn mark(&self, g: &[u64; 3]) {
        if let Ok(i) = self.uniq.binary_search(g) {
            self.hit[i / 64].fetch_or(1 << (i % 64), Ordering::Relaxed);
        }
    }

    /// (positions, overlapped positions) per segment of the gram's last token.
    pub fn census(&self) -> ([u64; 4], [u64; 4], u64, u64) {
        let (mut pos, mut over, mut hits) = ([0u64; 4], [0u64; 4], 0u64);
        for (i, m) in self.mult.iter().enumerate() {
            let h = self.hit[i / 64].load(Ordering::Relaxed) >> (i % 64) & 1 == 1;
            hits += h as u64;
            for s in 0..4 {
                pos[s] += m[s] as u64;
                if h {
                    over[s] += m[s] as u64;
                }
            }
        }
        (pos, over, self.uniq.len() as u64, hits)
    }
}

/// Per-split token counts for the target-count reconciliation.
#[derive(Clone, Copy, Default, Debug)]
pub struct TokCount {
    pub records: u64,
    pub tokens: u64,
    pub response: u64,
    pub seg_bytes: [u64; 4],
    pub seg_tokens: [u64; 4],
}
impl TokCount {
    fn add(&mut self, t: &Tok) {
        self.records += 1;
        self.tokens += t.tokens.len() as u64;
        self.response += t.response;
        for s in 0..4 {
            self.seg_bytes[s] += t.seg_bytes[s];
            self.seg_tokens[s] += t.seg_tokens[s];
        }
    }
    fn merge(&mut self, o: &TokCount) {
        self.records += o.records;
        self.tokens += o.tokens;
        self.response += o.response;
        for s in 0..4 {
            self.seg_bytes[s] += o.seg_bytes[s];
            self.seg_tokens[s] += o.seg_tokens[s];
        }
    }
    fn json(&self) -> Value {
        let bytes: u64 = self.seg_bytes.iter().sum();
        let seg: serde_json::Map<String, Value> = SEGMENTS.iter().map(|&s| {
            let i = s as usize;
            (s.name().to_string(), json!({"text_bytes": self.seg_bytes[i], "tokens": self.seg_tokens[i]}))
        }).collect();
        let next = self.tokens - self.records;
        json!({"records": self.records, "tokens": self.tokens, "response_tokens": self.response,
               "next_token_targets": next, "text_bytes": bytes, "per_segment": seg,
               "bytes_per_token": [bytes, self.tokens], "bytes_per_response_token": [bytes, self.response],
               "bytes_per_next_token_target": [bytes, next]})
    }
}

fn each_gram(t: &Tok, mut f: impl FnMut([u64; 3], u8)) {
    for e in GRAM..=t.tokens.len() {
        f(pack(&t.tokens[e - GRAM..e]), t.segments[e - 1]);
    }
}

/// The 13-gram census of a plan: DEV grams collected, TRAIN grams streamed against them.
pub fn audit(plan: &Plan, coder: &Coder, threads: usize, bloom_log2: u32) -> Result<Value, String> {
    let paths: Vec<PathBuf> = plan.shards.iter().map(|(p, _)| p.clone()).collect();
    let fate_of = |p: &Path| &plan.fate[paths.iter().position(|q| q == p).unwrap()];
    // Pass 1: DEV grams and DEV counts.
    let dev = par_shards(&paths, threads, |p| {
        let fate = fate_of(p);
        let (mut i, mut grams, mut cnt) = (0usize, [vec![], vec![]], [TokCount::default(); 2]);
        read_shard(p, &mut |line| {
            let f = fate[i];
            i += 1;
            let k = match f {
                Fate::Keep(Split::DevTune) => 0,
                Fate::Keep(Split::DevTest) => 1,
                _ => return Ok(()),
            };
            let t = tokenize(coder, line)?;
            cnt[k].add(&t);
            each_gram(&t, |g, s| grams[k].push((g, s)));
            Ok(())
        })?;
        Ok((grams, cnt))
    })?;
    let mut counts = [TokCount::default(); 3];
    let (mut tune, mut test) = (vec![], vec![]);
    for ([gt, gs], [ct, cs]) in dev {
        tune.extend(gt);
        test.extend(gs);
        counts[1].merge(&ct);
        counts[2].merge(&cs);
    }
    let sets = [GramSet::new(tune), GramSet::new(test)];
    let mut bloom = vec![0u64; 1usize << bloom_log2.saturating_sub(6)];
    let shift = 64 - bloom_log2;
    for set in &sets {
        for g in &set.uniq {
            let b = (mix(g) >> shift) as usize;
            bloom[b / 64] |= 1 << (b % 64);
        }
    }
    // Pass 2: TRAIN grams against the DEV sets.
    let train = par_shards(&paths, threads, |p| {
        let fate = fate_of(p);
        let (mut i, mut cnt) = (0usize, TokCount::default());
        read_shard(p, &mut |line| {
            let f = fate[i];
            i += 1;
            if f != Fate::Keep(Split::Train) {
                return Ok(());
            }
            let t = tokenize(coder, line)?;
            cnt.add(&t);
            each_gram(&t, |g, _| {
                let b = (mix(&g) >> shift) as usize;
                if bloom[b / 64] >> (b % 64) & 1 == 1 {
                    sets[0].mark(&g);
                    sets[1].mark(&g);
                }
            });
            Ok(())
        })?;
        Ok(cnt)
    })?;
    for c in &train {
        counts[0].merge(c);
    }
    let mut total = TokCount::default();
    let mut splits = serde_json::Map::new();
    for s in SPLITS {
        total.merge(&counts[s.ix()]);
        splits.insert(s.name().into(), counts[s.ix()].json());
    }
    let mut overlap = serde_json::Map::new();
    let mut gate = json!(null);
    for (set, s) in sets.iter().zip([Split::DevTune, Split::DevTest]) {
        let (pos, over, uniq, hits) = set.census();
        let (p, o): (u64, u64) = (pos.iter().sum(), over.iter().sum());
        let seg: serde_json::Map<String, Value> = SEGMENTS.iter()
            .map(|&g| (g.name().to_string(), json!([over[g as usize], pos[g as usize]]))).collect();
        overlap.insert(s.name().into(), json!({"positions": p, "overlapped": o, "overlap": [o, p],
            "overlap_by_last_token_segment": seg, "unique_grams": uniq, "unique_grams_in_train": hits}));
        if s == Split::DevTest {
            let verdict = if p == 0 { "NO-DATA" } else if o * 100 <= p { "PASS" } else { "FAIL" };
            gate = json!({"rule": "dev-test overlapped x 100 <= dev-test positions (P0(d): at most 1%)",
                          "overlap": [o, p], "verdict": verdict});
        }
    }
    Ok(json!({
        "unit": "claude-411", "gram": GRAM,
        "gram_compare": "exact: 13 ids x 12 bits packed in three u64 words, no hash; grams never cross a record",
        "bloom_log2": bloom_log2, "splits": splits, "total": total.json(), "overlap": overlap, "gate": gate,
        "ratios": "every pair is an exact integer [numerator, denominator]",
    }))
}

#[cfg(test)]
mod tests;
