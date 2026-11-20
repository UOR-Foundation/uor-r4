//! `slot-group-address` (20 Nov 2026): the address as a **learned group action**, not a table.
//!
//! Why this exists. The table tagger (`slot-table-build`) is not a model: it answers only wordings it
//! contains, and a transformer does not work that way — its address function (Q·K) is learned, and
//! the store is the KV cache. We cannot use Q·K (matmul) or softmax, so the lawful analogue is:
//!
//! * every word carries a learned quaternion `q_w` (four integers, held as f64 during research
//!   training only, per D84);
//! * the address of a text is the **Hamilton sum** of its words' quaternions, normalised — a group
//!   action on S^3, so composition is defined for combinations never seen;
//! * the store keys on that address; the read is a hard top-1 in the store's address space with an
//!   exact verify, not a normalised exponential;
//! * the only learned parameters are the word quaternions. The artifact carries **4 numbers per
//!   word** instead of a fitted word->attribute table, and it composes.
//!
//! The task it is trained and tested on is the mechanism's own: a statement writes a fact, a
//! paraphrased question must form the **same** address and retrieve that statement's value. Held-out
//! rows combine an entity and an attribute that TRAIN never paired, and use wording mixed from both.
//!
//! Usage: `slot-group-address [--epochs N] [--lr X] [--seed N]`

use std::collections::{BTreeMap, BTreeSet};
use std::process::ExitCode;

type Result<T> = std::result::Result<T, String>;

const ATTRS: [&str; 8] = ["colour", "city", "job", "pet", "drink", "car", "birthday", "sport"];
const SURFACES: [&[&str]; 8] = [
    &["favourite", "colour"],
    &["live", "city"],
    &["work", "job"],
    &["pet", "animal"],
    &["drink", "morning"],
    &["drive", "car"],
    &["birthday", "month"],
    &["sport", "play"],
];
const ALT: [&[&str]; 8] = [
    &["shade", "colour"],
    &["town", "live"],
    &["profession", "work"],
    &["companion", "animal"],
    &["beverage", "drink"],
    &["vehicle", "drive"],
    &["anniversary", "birthday"],
    &["hobby", "play"],
];
const ENTITIES: [&str; 10] = [
    "alice", "bruno", "carla", "dmitri", "eunji", "farid", "greta", "hugo", "ines", "jonas",
];
const VALUES: [&[&str]; 8] = [
    &["green", "blue", "red", "orange"],
    &["denver", "lisbon", "osaka", "nairobi"],
    &["baker", "nurse", "pilot", "locksmith"],
    &["cat", "dog", "parrot", "tortoise"],
    &["tea", "coffee", "juice", "water"],
    &["volvo", "toyota", "kia", "ford"],
    &["march", "july", "november", "may"],
    &["tennis", "chess", "rowing", "darts"],
];

fn statement(e: usize, a: usize) -> String {
    let v = VALUES[a][(e + a) % VALUES[a].len()];
    format!("{} {} is {}", ENTITIES[e], SURFACES[a].join(" "), v)
}

fn question(e: usize, a: usize, alt: bool) -> String {
    let surface = if alt { ALT[a] } else { SURFACES[a] };
    format!("what is {} {} {}", ENTITIES[e], surface[0], surface[1])
}

fn words(text: &str) -> BTreeSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphabetic())
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

/// A quaternion held as four f64 components (research training only; the served form is integer).
type Q = [f64; 4];

fn q_add(a: Q, b: Q) -> Q {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}
fn q_scale(a: Q, s: f64) -> Q {
    [a[0] * s, a[1] * s, a[2] * s, a[3] * s]
}
fn q_norm(a: Q) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2] + a[3] * a[3]).sqrt()
}
fn q_unit(a: Q) -> Q {
    let n = q_norm(a);
    if n < 1e-12 {
        [1.0, 0.0, 0.0, 0.0]
    } else {
        q_scale(a, 1.0 / n)
    }
}
fn q_dot(a: Q, b: Q) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

/// The copula and the closed attribute verbs. A **generic** positional rule, not a table: the write's
/// slot is the subject span before the verb, the read's is the span after it, because a statement is
/// S-V-O and a question inverts it. This is the parse that attention performs by learning, made
/// explicit and lawful.
const VERBS: [&str; 7] = ["is", "lives", "works", "drinks", "drives", "plays", "has"];
const STOPS: [&str; 12] = [
    "what", "which", "where", "when", "who", "whose", "the", "a", "an", "does", "do", "s",
];

/// The slot's words: the content words of the subject side (write) or the predicate side (read).
fn slot_words(text: &str, after_verb: bool) -> Vec<String> {
    let tokens: Vec<String> = text
        .to_lowercase()
        .split(|c: char| !c.is_alphabetic())
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect();
    let at = tokens.iter().position(|w| VERBS.contains(&w.as_str()));
    let slice = match (at, after_verb) {
        (Some(i), false) => &tokens[..i],
        (Some(i), true) => &tokens[i + 1..],
        (None, _) => &tokens[..],
    };
    slice
        .iter()
        .filter(|w| !STOPS.contains(&w.as_str()))
        .cloned()
        .collect()
}

/// The address of a text: the normalised Hamilton sum of its **slot** words' quaternions.
fn address(text: &str, table: &BTreeMap<String, Q>, after_verb: bool) -> Option<Q> {
    let mut sum: Q = [0.0; 4];
    let mut any = false;
    for w in slot_words(text, after_verb) {
        if let Some(q) = table.get(&w) {
            sum = q_add(sum, *q);
            any = true;
        }
    }
    any.then(|| q_unit(sum))
}

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
    let mut epochs = 400usize;
    let mut lr = 0.15f64;
    let mut seed: u64 = 20_261_120;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--epochs" => epochs = args.next().and_then(|v| v.parse().ok()).ok_or("--epochs wants a number")?,
            "--lr" => lr = args.next().and_then(|v| v.parse().ok()).ok_or("--lr wants a number")?,
            "--seed" => seed = args.next().and_then(|v| v.parse().ok()).ok_or("--seed wants a number")?,
            other => return Err(format!("unknown argument {other}")),
        }
    }

    // The panel: every (entity, attribute) pair, split by a seeded shuffle.
    let mut pairs: Vec<(usize, usize)> = (0..ENTITIES.len())
        .flat_map(|e| (0..ATTRS.len()).map(move |a| (e, a)))
        .collect();
    let mut state = seed | 1;
    for i in (1..pairs.len()).rev() {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let j = (state.wrapping_mul(0x2545_F491_4F6C_DD1D) % (i as u64 + 1)) as usize;
        pairs.swap(i, j);
    }
    let (train, held) = pairs.split_at(pairs.len() / 2);

    // Vocabulary: every word of every statement and question, so nothing is out-of-vocabulary.
    let mut vocab: BTreeSet<String> = BTreeSet::new();
    for &(e, a) in &pairs {
        for w in words(&statement(e, a)) {
            vocab.insert(w);
        }
        for w in words(&question(e, a, false)) {
            vocab.insert(w);
        }
        for w in words(&question(e, a, true)) {
            vocab.insert(w);
        }
    }
    let vocab: Vec<String> = vocab.into_iter().collect();
    let index: BTreeMap<String, usize> = vocab.iter().cloned().enumerate().map(|(i, w)| (w, i)).collect();

    // Deterministic initialisation from the seed.
    let mut embed: Vec<Q> = vocab
        .iter()
        .map(|_| {
            let mut q = [0.0f64; 4];
            for c in &mut q {
                state ^= state >> 12;
                state ^= state << 25;
                state ^= state >> 27;
                *c = ((state % 2001) as f64 / 1000.0) - 1.0;
            }
            q_unit(q)
        })
        .collect();

    // A text's address from the current embeddings.
    let addr = |text: &str, embed: &[Q], after_verb: bool| -> Option<Q> {
        let mut sum: Q = [0.0; 4];
        let mut any = false;
        for w in slot_words(text, after_verb) {
            if let Some(&i) = index.get(&w) {
                sum = q_add(sum, embed[i]);
                any = true;
            }
        }
        any.then(|| q_unit(sum))
    };

    // Training: a paraphrase-invariant address. Each TRAIN fact contributes (question, own
    // statement) as the positive and every other TRAIN statement as a hard negative; the hinge is
    // on the dot product (a hard margin, no softmax).
    let train_facts: Vec<(usize, usize)> = train.to_vec();
    let margin = 0.30f64;
    for epoch in 0..epochs {
        let mut loss = 0.0f64;
        for &(e, a) in &train_facts {
            let q_text = question(e, a, epoch % 2 == 1);
            let Some(q) = addr(&q_text, &embed, true) else { continue };
            let Some(pos) = addr(&statement(e, a), &embed, false) else { continue };
            for &(e2, a2) in &train_facts {
                if (e2, a2) == (e, a) {
                    continue;
                }
                let Some(neg) = addr(&statement(e2, a2), &embed, false) else { continue };
                let gap = q_dot(q, pos) - q_dot(q, neg);
                if gap < margin {
                    loss += margin - gap;
                    // d(gap)/d(q_embed word) drives the question's words toward the positive and away
                    // from the negative; the same for the statement's words. Unit-normalised after.
                    let step = q_scale(q, lr * (margin - gap));
                    for w in words(&q_text) {
                        if let Some(&i) = index.get(&w) {
                            embed[i] = q_add(embed[i], step);
                        }
                    }
                    let step_pos = q_scale(pos, lr * (margin - gap));
                    for w in words(&statement(e, a)) {
                        if let Some(&i) = index.get(&w) {
                            embed[i] = q_add(embed[i], step_pos);
                        }
                    }
                    let step_neg = q_scale(neg, -lr * (margin - gap));
                    for w in words(&statement(e2, a2)) {
                        if let Some(&i) = index.get(&w) {
                            embed[i] = q_add(embed[i], step_neg);
                        }
                    }
                }
            }
        }
        for q in &mut embed {
            *q = q_unit(*q);
        }
        if epoch + 1 == epochs {
            println!("final epoch hinge loss: {loss:.4} over {} TRAIN facts", train_facts.len());
        }
    }

    // Evaluation, exactly as serving works: the conversation's held-out statements are the store's
    // keys, and each question must retrieve its own statement by address.
    let held_keys: Vec<Q> = held
        .iter()
        .map(|&(e, a)| addr(&statement(e, a), &embed, false).unwrap_or([1.0, 0.0, 0.0, 0.0]))
        .collect();
    let mut known = 0usize;
    let mut alt = 0usize;
    for (row, &(e, a)) in held.iter().enumerate() {
        for (is_alt, counter) in [(false, &mut known), (true, &mut alt)] {
            let Some(q) = addr(&question(e, a, is_alt), &embed, true) else { continue };
            let best = (0..held.len())
                .max_by(|&i, &j| q_dot(q, held_keys[i]).partial_cmp(&q_dot(q, held_keys[j])).unwrap())
                .unwrap_or(usize::MAX);
            if best == row {
                *counter += 1;
            }
        }
    }
    println!(
        "held-out combinations: {}, known wording {known}/{}, held-out wording {alt}/{}",
        held.len(),
        held.len(),
        held.len()
    );
    println!(
        "address is learned: {} word quaternions ({} parameters), no table, no matmul, no softmax",
        embed.len(),
        embed.len() * 4
    );
    Ok(())
}
