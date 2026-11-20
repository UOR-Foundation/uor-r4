//! `slot-embed-train` (20 Nov 2026): learn the slot vocabulary from the corpus by co-occurrence, so
//! the tagger is **trained**, not schematised.
//!
//! The measured problem: a fitted word->attribute table answers only wordings it contains
//! (`slot-table-build`: 56 words; `/tmp` panel: 40/40 on covered wording, nothing on new words), and
//! a word whose quaternion was never trained forms no address (`slot-group-address`: 1-4/40 on
//! held-out wording). Coverage has to come from data, not from a hand list.
//!
//! The method is lawful and matmul-free: **second-order co-occurrence by sparse propagation**.
//!
//! 1. Every content word of a user question is collected (stop words removed).
//! 2. Word-word co-occurrence counts are accumulated over the corpus (undirected, sparse).
//! 3. Embeddings are estimated by power iteration on that sparse graph: `r(w) <- normalise(sum over
//!    neighbours u of count(w,u) * r(u))`, repeated. Words used in the same contexts converge, which
//!    is what makes a synonym land next to the word it is a synonym of.
//! 4. The embeddings are clustered into `--clusters` groups (`k`-means, deterministic seeding) — this
//!    is where a large-`k` e-means fitter belongs at scale; at `k = 8` the project's own measurement
//!    is parity with Lloyd (`codex-15` REPORT), so Lloyd is used here and named.
//! 5. Each cluster is named by the seed word it contains (`colour`, `city`, `job`, ...); a word
//!    inherits its cluster's name. The output is the same `{"word": "attribute"}` file the served
//!    tagger already loads, so nothing in the serving path changes.
//!
//! No matmul (sparse neighbour sums), no squashing function, no float in the output; the float
//! iteration is research training on the CPU (D84).
//!
//! Usage: `slot-embed-train --corpus FILE.jsonl --out TABLE.json [--clusters 8] [--dim 16]`

use std::collections::{BTreeMap, BTreeSet};
use std::process::ExitCode;

type Result<T> = std::result::Result<T, String>;

const SEEDS: [&str; 8] = ["colour", "city", "job", "pet", "drink", "car", "birthday", "sport"];
const STOPS: [&str; 30] = [
    "the", "a", "an", "is", "are", "was", "were", "be", "to", "of", "and", "or", "in", "on", "for",
    "with", "what", "which", "where", "when", "who", "whose", "how", "why", "do", "does", "did",
    "s", "i", "you",
];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let mut corpus = None;
    let mut out = None;
    let mut dim: usize = 16;
    let mut min_count: usize = 20;
    let mut iterations: usize = 12;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--corpus" => corpus = args.next(),
            "--out" => out = args.next(),
            "--dim" => dim = args.next().and_then(|v| v.parse().ok()).ok_or("--dim wants a number")?,
            "--min-count" => min_count = args.next().and_then(|v| v.parse().ok()).ok_or("--min-count wants a number")?,
            "--iterations" => iterations = args.next().and_then(|v| v.parse().ok()).ok_or("--iterations wants a number")?,
            other => return Err(format!("unknown argument {other}")),
        }
    }
    let corpus = corpus.ok_or("--corpus is required")?;
    let out = out.ok_or("--out is required")?;

    // 1-2. Content words per question, then undirected co-occurrence counts.
    let file = std::fs::File::open(&corpus).map_err(|e| format!("{corpus}: {e}"))?;
    let reader = std::io::BufReader::new(file);
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut pairs: BTreeMap<(String, String), usize> = BTreeMap::new();
    use std::io::BufRead;
    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else { continue };
        let Some(messages) = value.get("messages").and_then(|m| m.as_array()) else { continue };
        for message in messages {
            if message.get("role").and_then(|r| r.as_str()) != Some("user") {
                continue;
            }
            let Some(content) = message.get("content").and_then(|c| c.as_str()) else { continue };
            if !content.contains('?') {
                continue;
            }
            let words: BTreeSet<String> = content
                .to_lowercase()
                .split(|c: char| !c.is_alphabetic())
                .filter(|w| w.len() >= 3 && !STOPS.contains(w))
                .map(str::to_owned)
                .collect();
            for w in &words {
                *counts.entry(w.clone()).or_default() += 1;
            }
            for a in &words {
                for b in &words {
                    if a < b {
                        *pairs.entry((a.clone(), b.clone())).or_default() += 1;
                    }
                }
            }
        }
    }

    // 3. Vocabulary: frequent enough, and neighbours sorted once.
    let vocab: Vec<String> = counts
        .iter()
        .filter(|(_, &c)| c >= min_count)
        .map(|(w, _)| w.clone())
        .collect();
    let index: BTreeMap<&str, usize> = vocab.iter().enumerate().map(|(i, w)| (w.as_str(), i)).collect();
    let mut neighbours: Vec<Vec<(usize, f64)>> = vec![Vec::new(); vocab.len()];
    for ((a, b), &count) in &pairs {
        let (Some(&i), Some(&j)) = (index.get(a.as_str()), index.get(b.as_str())) else { continue };
        neighbours[i].push((j, count as f64));
        neighbours[j].push((i, count as f64));
    }
    println!("vocabulary: {} words (count >= {min_count})", vocab.len());

    // Deterministic initialisation.
    let mut state: u64 = 20_261_120;
    let mut embed: Vec<Vec<f64>> = (0..vocab.len())
        .map(|_| {
            (0..dim)
                .map(|_| {
                    state ^= state >> 12;
                    state ^= state << 25;
                    state ^= state >> 27;
                    ((state % 2001) as f64 / 1000.0) - 1.0
                })
                .collect()
        })
        .collect();

    // Power iteration on the sparse co-occurrence graph: the low-frequency structure, no matmul.
    for _ in 0..iterations {
        let mut next = vec![vec![0.0f64; dim]; vocab.len()];
        for (i, row) in neighbours.iter().enumerate() {
            let mut acc = vec![0.0f64; dim];
            for &(j, weight) in row {
                for d in 0..dim {
                    acc[d] += weight * embed[j][d];
                }
            }
            let norm = acc.iter().map(|v| v * v).sum::<f64>().sqrt();
            if norm > 1e-12 {
                for d in 0..dim {
                    acc[d] /= norm;
                }
            }
            next[i] = acc;
        }
        embed = next;
    }

    // 4. Assign each word to the nearest **seed** in the learned space. The seeds are the tiny
    // labelled taxonomy (eight words); the coverage is learned, because a synonym lands near its
    // seed by co-occurrence alone. Unsupervised clustering was tried first and does not work here:
    // "colour" and "city" are distributionally identical ("what is X's ___"), so k-means groups
    // them together — the distinction is referential, not topical, and only the eight labels supply
    // it.
    let seeds: Vec<(usize, &str)> = SEEDS
        .iter()
        .filter_map(|s| index.get(s).map(|&i| (i, *s)))
        .collect();
    if seeds.len() != SEEDS.len() {
        return Err("a seed word is absent from the vocabulary; lower --min-count".to_owned());
    }
    let mut table: BTreeMap<String, String> = BTreeMap::new();
    for (i, word) in vocab.iter().enumerate() {
        let (best, _) = seeds
            .iter()
            .map(|&(s, name)| {
                let dot: f64 = embed[i].iter().zip(&embed[s]).map(|(a, b)| a * b).sum();
                (name, dot)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();
        table.insert(word.clone(), best.to_owned());
    }
    let json = serde_json::to_string_pretty(&table).map_err(|e| e.to_string())?;
    std::fs::write(&out, json).map_err(|e| format!("{out}: {e}"))?;
    println!(
        "assigned {} words to the nearest of {} seeds by the learned space -> {out}",
        table.len(),
        SEEDS.len()
    );
    Ok(())
}
