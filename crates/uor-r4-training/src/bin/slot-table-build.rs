//! `slot-table-build` (20 Nov 2026): fit the slot tagger's attribute table from the real corpus.
//!
//! The exact-address mechanism's store and address are measured and working; the measured limit is
//! the tagger's **wording coverage** (`slot-address-check`: 17/40 on wordings a 40-row panel never
//! saw). Coverage is a data problem, so this fits the table from the project's own dialogue corpus
//! instead of a panel.
//!
//! Method, and its one assumption stated plainly: a question is labelled with an attribute when its
//! words match **exactly one** attribute's surface list; questions matching two or none are dropped
//! rather than guessed. A word is kept only when its dominant attribute has at least
//! `--min-count` observations and at least `--margin` times the runner-up, so an ambiguous word does
//! not enter the table at all.
//!
//! Integer counts and word sets only: no matmul, no squashing, no float, deterministic.
//!
//! Usage: `slot-table-build --corpus FILE.jsonl --out TABLE.json [--min-count 3] [--margin 2]`

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::process::ExitCode;

type Result<T> = std::result::Result<T, String>;

/// The closed attribute set and its surface words. The lists are the *hypothesis* being tested: the
/// corpus decides which words actually name which attribute, and a word that does not separate them
/// is dropped by the margin rule.
const ATTRS: [(&str, &[&str]); 8] = [
    (
        "colour",
        &["colour", "color", "colours", "colors", "shade", "shades", "hue", "tint", "favourite colour", "favorite color"],
    ),
    ("city", &["city", "town", "cities", "towns", "live", "lives", "reside", "resides", "located"]),
    (
        "job",
        &["job", "jobs", "profession", "professions", "occupation", "employed", "career", "work"],
    ),
    ("pet", &["pet", "pets", "animal", "animals", "companion", "companions"]),
    ("drink", &["drink", "drinks", "beverage", "beverages", "coffee", "tea", "juice"]),
    ("car", &["car", "cars", "vehicle", "vehicles", "drive", "drives"]),
    ("birthday", &["birthday", "birthdays", "born", "anniversary", "anniversaries"]),
    ("sport", &["sport", "sports", "game", "games", "hobby", "hobbies", "played", "plays"]),
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
    let mut min_count: usize = 3;
    let mut margin: f64 = 2.0;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--corpus" => corpus = args.next(),
            "--out" => out = args.next(),
            "--min-count" => {
                min_count = args.next().and_then(|v| v.parse().ok()).ok_or("--min-count wants a number")?
            }
            "--margin" => margin = args.next().and_then(|v| v.parse().ok()).ok_or("--margin wants a number")?,
            other => return Err(format!("unknown argument {other}")),
        }
    }
    let corpus = corpus.ok_or("--corpus is required")?;
    let out = out.ok_or("--out is required")?;

    let file = std::fs::File::open(&corpus).map_err(|e| format!("{corpus}: {e}"))?;
    let mut words: BTreeMap<String, [usize; 8]> = BTreeMap::new();
    let mut questions = 0usize;
    let mut labelled = 0usize;

    for line in BufReader::new(file).lines() {
        let line = line.map_err(|e| e.to_string())?;
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        let Some(messages) = value.get("messages").and_then(|m| m.as_array()) else {
            continue;
        };
        for message in messages {
            if message.get("role").and_then(|r| r.as_str()) != Some("user") {
                continue;
            }
            let Some(content) = message.get("content").and_then(|c| c.as_str()) else {
                continue;
            };
            if !content.contains('?') {
                continue;
            }
            questions += 1;
            let lower = content.to_lowercase();
            let tokens: std::collections::BTreeSet<&str> = lower
                .split(|c: char| !c.is_alphabetic())
                .filter(|w| !w.is_empty())
                .collect();
            // The question is labelled only when exactly one attribute's words appear in it.
            let mut hits: Vec<(usize, Vec<&str>)> = Vec::new();
            for (index, (_, surfaces)) in ATTRS.iter().enumerate() {
                let found: Vec<&str> = surfaces
                    .iter()
                    .copied()
                    .filter(|w| tokens.contains(w))
                    .collect();
                if !found.is_empty() {
                    hits.push((index, found));
                }
            }
            if hits.len() != 1 {
                continue;
            }
            labelled += 1;
            let (index, found) = &hits[0];
            for word in found {
                words.entry((*word).to_string()).or_insert([0; 8])[*index] += 1;
            }
        }
    }

    // Keep a word only when it separates: dominant attribute at least `min_count` and `margin` times
    // the runner-up.
    let mut table: BTreeMap<String, String> = BTreeMap::new();
    let mut dropped = 0usize;
    for (word, counts) in &words {
        let mut order: Vec<usize> = (0..8).collect();
        order.sort_by_key(|&i| std::cmp::Reverse(counts[i]));
        let (best, second) = (order[0], order[1]);
        let dominant = counts[best];
        let runner = counts[second];
        if dominant >= min_count && (runner == 0 || dominant as f64 >= margin * runner as f64) {
            table.insert(word.clone(), ATTRS[best].0.to_string());
        } else {
            dropped += 1;
        }
    }

    let json = serde_json::to_string_pretty(&table).map_err(|e| e.to_string())?;
    std::fs::write(&out, json).map_err(|e| format!("{out}: {e}"))?;
    println!(
        "corpus: {questions} user questions, {labelled} labelled by exactly one attribute; table: {} words, {dropped} dropped by the margin rule -> {out}",
        table.len()
    );
    Ok(())
}
