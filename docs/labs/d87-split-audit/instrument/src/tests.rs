//! Fixture tests on small synthetic shards.

use super::*;
use std::collections::BTreeSet;

/// GPT-2's byte-to-character table, as the tokenizer uses it.
fn byte_char(b: u8) -> char {
    let printable = |x: u8| (33..=126).contains(&x) || (161..=172).contains(&x) || x >= 174;
    if printable(b) {
        return b as char;
    }
    let n = (0u8..=255).filter(|&x| !printable(x) && x < b).count() as u32;
    char::from_u32(256 + n).unwrap()
}

/// A byte-level tokenizer with no merges: one token per text byte.
fn coder() -> Coder {
    let mut vocab = serde_json::Map::new();
    for (i, name) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
        vocab.insert(name.to_string(), json!(i));
    }
    for b in 0u16..256 {
        let id = vocab.len();
        vocab.insert(byte_char(b as u8).to_string(), json!(id));
    }
    let added: Vec<Value> = ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate()
        .map(|(i, c)| json!({"id": i, "content": c, "special": true})).collect();
    let j = json!({"added_tokens": added,
        "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false, "trim_offsets": true, "use_regex": true},
        "model": {"type": "BPE", "vocab": vocab, "merges": []}});
    Coder::new(&serde_json::to_vec(&j).unwrap()).unwrap()
}

fn rec(id: &str, users: &[&str], turns: &[(&str, &str)]) -> String {
    let kind = if users.len() == 1 { "single" } else { "two-turn" };
    let msgs: Vec<Value> = users.iter().map(|u| json!({"role": "user", "content": u})).collect();
    let ts: Vec<Value> = turns.iter().map(|(r, a)| json!({"reasoning": r, "answer": a})).collect();
    json!({"id": id, "kind": kind, "source": "fx", "messages": msgs, "turns": ts}).to_string()
}

struct Dir(PathBuf);
impl Dir {
    fn new(name: &str) -> Self {
        let d = std::env::temp_dir().join(format!("d87-split-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        Dir(d)
    }
    fn shard(&self, name: &str, lines: &[String]) -> PathBuf {
        let p = self.0.join(name);
        let text: String = lines.iter().map(|l| format!("{l}\n")).collect();
        if name.ends_with(".zst") {
            std::fs::write(&p, zstd::encode_all(text.as_bytes(), 3).unwrap()).unwrap();
        } else {
            std::fs::write(&p, text).unwrap();
        }
        p
    }
}
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn kept(p: &Plan) -> BTreeMap<String, Split> {
    SPLITS.iter().flat_map(|&s| p.manifest(s).lines().map(move |i| (i.to_string(), s)).collect::<Vec<_>>()).collect()
}

#[test]
fn alternatives_of_one_prompt_stay_together() {
    let d = Dir::new("alt");
    let sp = Splitter::new(Config { dev_tune: (1, 4), dev_test: (1, 4), ..Config::default() }).unwrap();
    let (mut a, mut b) = (vec![], vec![]);
    for i in 0..200 {
        let prompt = format!("Prompt number {i} about things");
        for k in 0..3 {
            let line = rec(&format!("p{i}-a{k}"), &[&prompt], &[("think", &format!("alternative {k} answer for prompt {i}"))]);
            if k == 1 { b.push(line) } else { a.push(line) }
        }
        // The same first message, recased and repunctuated, as a two-turn record.
        b.push(rec(&format!("p{i}-t2"), &[&format!("PROMPT number {i}, about things!"), "More?"],
            &[("r", &format!("two-turn first answer {i}")), ("r", &format!("two-turn second answer {i}"))]));
    }
    let p = plan(&sp, &[d.shard("s0.jsonl.zst", &a), d.shard("s1.jsonl", &b)], 3).unwrap();
    let k = kept(&p);
    assert_eq!(k.len(), 800);
    let mut used = BTreeSet::new();
    for i in 0..200 {
        let s = k[&format!("p{i}-a0")];
        for id in [format!("p{i}-a1"), format!("p{i}-a2"), format!("p{i}-t2")] {
            assert_eq!(k[&id], s, "{id} left its prompt's split");
        }
        used.insert(s);
    }
    assert_eq!(used.len(), 3, "every split holds prompts at 1/4 + 1/4");
    let s = p.summary();
    let prompts: u64 = SPLITS.iter().map(|x| s["splits"][x.name()]["prompts"].as_u64().unwrap()).sum();
    assert_eq!((s["prompts"].as_u64(), prompts), (Some(200), 200));
}

#[test]
fn split_is_deterministic() {
    let d = Dir::new("det");
    let sp = Splitter::new(Config::default()).unwrap();
    let lines: Vec<String> = (0..300).map(|i| rec(&format!("r{i}"), &[&format!("question {i}")], &[("", &format!("reply {i}"))])).collect();
    let (x, y) = (d.shard("a.jsonl.zst", &lines[..150]), d.shard("b.jsonl.zst", &lines[150..]));
    let read = |out: &Path| -> Vec<u8> {
        let mut all = vec![];
        for f in ["train.ids", "dev-tune.ids", "dev-test.ids", "split-summary.json"] {
            all.extend(std::fs::read(out.join(f)).unwrap());
        }
        all
    };
    let (o1, o2) = (d.0.join("out1"), d.0.join("out2"));
    plan(&sp, &[x.clone(), y.clone()], 1).unwrap().write(&o1).unwrap();
    plan(&sp, &[y, x], 4).unwrap().write(&o2).unwrap();
    let (h1, h2) = (sha(&[&read(&o1)]), sha(&[&read(&o2)]));
    assert_eq!(h1, h2);
    // A different key moves prompts: the hash really depends on the assignment.
    let other = Splitter::new(Config { key: "other".into(), ..Config::default() }).unwrap();
    let moved = (0..300).filter(|i| sp.split_of(&format!("question {i}")).1 != other.split_of(&format!("question {i}")).1).count();
    assert!(moved > 0);
}

#[test]
fn planted_duplicates_are_removed() {
    let d = Dir::new("dup");
    let sp = Splitter::new(Config::default()).unwrap();
    let long = "The capital of France is Paris, a city on the Seine.";
    let r1 = rec("r1", &["Where is Paris?"], &[("", long)]);
    let a = vec![r1.clone(), rec("r2", &["Is it?"], &[("", "yes")]), rec("r3", &["Was it?"], &[("", "yes")])];
    let b = vec![
        r1,                                                                  // exact copy
        rec("r1", &["Where is Paris?"], &[("", "Another answer entirely, long enough to count.")]), // repeated id
        rec("r4", &["Name France's capital."], &[("hm", "the CAPITAL of france is paris -- a city on the seine")]), // near
        rec("r5", &["Something else"], &[("", "A different and sufficiently long answer text.")]),
    ];
    let p = plan(&sp, &[d.shard("b.jsonl", &b), d.shard("a.jsonl", &a)], 2).unwrap();
    let k = kept(&p);
    assert_eq!(k.keys().cloned().collect::<Vec<_>>(), ["r1", "r2", "r3", "r5"]);
    let s = p.summary();
    assert_eq!(s["dropped"], json!({"exact": 1, "id": 1, "near": 1}));
    assert_eq!((s["lines"].as_u64(), s["records"].as_u64()), (Some(7), Some(4)));
}

/// A prompt text `"{stem} {n}{end}"` that lands in `want`.
fn prompt_in(sp: &Splitter, stem: &str, end: char, want: Split) -> String {
    (0..).map(|n| format!("{stem} {n}{end}")).find(|t| sp.split_of(t).1 == want).unwrap()
}

fn census(plant: bool) -> (Value, u64) {
    let d = Dir::new(if plant { "plant" } else { "clean" });
    let sp = Splitter::new(Config { dev_tune: (0, 1), dev_test: (1, 2), ..Config::default() }).unwrap();
    let dev_user = prompt_in(&sp, "devq", '?', Split::DevTest);
    let dev_answer = if plant { "xhQWERTYUIOPLKJzv dev tail" } else { "xhQWERTYUIOPLKXzv dev tail" };
    let t1 = prompt_in(&sp, "trn one", '.', Split::Train);
    let t2 = prompt_in(&sp, "trn two", '.', Split::Train);
    let lines = vec![
        rec("dev", &[&dev_user], &[("", dev_answer)]),
        rec("t1", &[&t1], &[("", "moQWERTYUIOPLKJnp moQWERTYUIOPLKJnp train tail")]),
        rec("t2", &[&t2], &[("", "moQWERTYUIOPLKJns other words")]),
    ];
    let p = plan(&sp, &[d.shard("s.jsonl.zst", &lines)], 2).unwrap();
    assert_eq!(kept(&p)["dev"], Split::DevTest);
    // BOS + "\nUser:" + " " prompt + "\nAssistant:" + " " answer + EOS, one token per byte.
    let len = 1 + 6 + 1 + dev_user.len() + 11 + 1 + dev_answer.len() + 1;
    (audit(&p, &coder(), 2, 12).unwrap(), (len - (GRAM - 1)) as u64)
}

#[test]
fn planted_train_dev_gram_counts_once() {
    let (a, positions) = census(true);
    let t = &a["overlap"]["dev-test"];
    assert_eq!(t["positions"].as_u64(), Some(positions));
    assert_eq!(t["overlapped"].as_u64(), Some(1), "{a:#}");
    assert_eq!(t["overlap_by_last_token_segment"]["answer"][0], 1);
    assert_eq!(a["overlap"]["dev-tune"]["positions"].as_u64(), Some(0));
    assert_eq!(a["splits"]["train"]["records"].as_u64(), Some(2));
}

#[test]
fn dev_with_no_overlap_reports_zero() {
    let (a, positions) = census(false);
    assert_eq!(a["overlap"]["dev-test"]["overlap"], json!([0, positions]));
    assert_eq!(a["gate"]["verdict"], "PASS");
}

#[test]
fn packing_is_exact() {
    let base: Vec<u32> = (0..13).map(|i| 4095 - i).collect();
    let mut other = base.clone();
    other[12] = 4094;
    assert_ne!(pack(&base), pack(&other));
    let w = pack(&base);
    let back: Vec<u32> = (0..13).map(|i| (w[i / 5] >> (12 * (i % 5)) & 0xfff) as u32).collect();
    assert_eq!(back, base);
}
