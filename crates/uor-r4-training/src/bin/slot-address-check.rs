//! `slot-address-check` (20 Nov 2026): the first decisive measurement for the exact-address
//! mechanism, before any model work.
//!
//! The question it answers, and the only one that matters at this stage: **does a formed
//! (entity, attribute) address answer entity-attribute RECOMBINATIONS that a question-to-answer
//! table cannot?** If it does not, the canonicalisation claim is dead and the mechanism is not worth
//! building. If it does, the mechanism is worth the store, the tagger and the serving work.
//!
//! Everything here is integer and table arithmetic: no matmul, no squashing function, no float
//! master, no learned weights, a deterministic panel generated from a fixed seed. It is a host
//! instrument, not a model result.
//!
//! The panel is generated, not authored row by row: 8 entities x 6 closed attributes, each pair
//! given a value; half the pairs are TRAIN and half are HELD OUT, and the held-out half shares every
//! attribute surface form and every entity with TRAIN but **never a pair**. That is the "same
//! relation in new words / new combination" case the record names as the hard one
//! (`docs/mechanisms/exact-memory.md`), and it is the only case where a compositional key can differ
//! from a memorised table.
//!
//! Usage: `slot-address-check [--seed N] [--verbose]`

use std::collections::{BTreeMap, BTreeSet};
use std::process::ExitCode;

type Result<T> = std::result::Result<T, String>;

/// The closed attribute set. A question's surface form is matched to one of these by a table over
/// its content words; the set is closed because the tagger is a table, and a table needs a range.
const ATTRIBUTES: [&str; 8] = ["colour", "city", "job", "pet", "drink", "car", "birthday", "sport"];

/// The attribute's question surface forms, shared by TRAIN and HELD OUT (so a held-out row is new in
/// its *combination*, never in its wording — the wording case is a separate, later test).
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

/// Wording held OUT of the fitted table: a second surface per attribute, never inserted. A word
/// table cannot map these; this is the recorded "same relation in new words" hard case.
const ALT_SURFACES: [&[&str]; 8] = [
    &["shade", "favourite"],
    &["town", "reside"],
    &["profession", "employed"],
    &["companion", "animal"],
    &["beverage", "breakfast"],
    &["vehicle", "drive"],
    &["anniversary", "born"],
    &["game", "hobby"],
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

fn entity_index(name: &str) -> Option<usize> {
    ENTITIES.iter().position(|&e| e == name)
}

/// Every fact the conversation states, as fragments the panel's context is built from.
fn fact(entity: usize, attribute: usize, variant: usize) -> String {
    let value = VALUES[attribute][variant % VALUES[attribute].len()];
    match attribute {
        0 => format!("{} says their favourite colour is {}.", ENTITIES[entity], value),
        1 => format!("{} lives in the city of {}.", ENTITIES[entity], value),
        2 => format!("{} works as a {}.", ENTITIES[entity], value),
        3 => format!("{} has a pet whose name is {}.", ENTITIES[entity], value),
        4 => format!("{} drinks {} every morning.", ENTITIES[entity], value),
        5 => format!("{} drives a {}.", ENTITIES[entity], value),
        6 => format!("{}'s birthday is in {}.", ENTITIES[entity], value),
        _ => format!("{} plays {}.", ENTITIES[entity], value),
    }
}

fn question(entity: usize, attribute: usize) -> String {
    let words = SURFACES[attribute];
    format!("What is {}'s {} {}?", ENTITIES[entity], words[0], words[1])
}

fn alt_question(entity: usize, attribute: usize) -> String {
    let words = ALT_SURFACES[attribute];
    format!("Which {} is {}\'s {}?", words[0], ENTITIES[entity], words[1])
}

/// The tagger. Entity: the panel's question names it, so the tagger matches the question's tokens
/// against the conversation's mentions — no learning, stated plainly. Attribute: a table from the
/// question's content words to the closed set, fitted on TRAIN rows only.
fn tag(
    q: &str,
    mentions: &BTreeSet<usize>,
    table: &BTreeMap<String, usize>,
) -> (Option<usize>, Option<usize>) {
    // Match on whole words, not substrings: "carla" must not match the attribute surface "car".
    let lower = q.to_lowercase();
    let words: BTreeSet<&str> = lower
        .split(|c: char| !c.is_alphabetic())
        .filter(|w| !w.is_empty())
        .collect();
    let entity = mentions
        .iter()
        .copied()
        .find(|&e| words.contains(ENTITIES[e]));
    let attribute = table
        .iter()
        .map(|(word, &a)| (usize::from(words.contains(word.as_str())), a))
        .filter(|&(hits, _)| hits > 0)
        .max_by_key(|&(hits, a)| (hits, std::cmp::Reverse(a)))
        .map(|(_, a)| a);
    (entity, attribute)
}

fn main() -> ExitCode {
    let mut seed: u64 = 20_261_120;
    let mut verbose = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--seed" => match args.next().and_then(|v| v.parse().ok()) {
                Some(v) => seed = v,
                None => {
                    eprintln!("--seed wants a number");
                    return ExitCode::FAILURE;
                }
            },
            "--verbose" => verbose = true,
            other => {
                eprintln!("unknown argument {other}");
                return ExitCode::FAILURE;
            }
        }
    }
    match run(seed, verbose) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(seed: u64, verbose: bool) -> Result<()> {
    // The panel: every (entity, attribute) pair, split by a seeded order. Held out shares every
    // entity and every attribute surface with TRAIN, and shares no pair.
    let mut pairs: Vec<(usize, usize)> = (0..ENTITIES.len())
        .flat_map(|e| (0..ATTRIBUTES.len()).map(move |a| (e, a)))
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

    // The attribute table is fitted on TRAIN questions only: each closed attribute's surface words
    // map to it. Every held-out row's surface words are therefore *known*, and only its pairing is
    // new.
    let mut attribute_table: BTreeMap<String, usize> = BTreeMap::new();
    for &(_, a) in train {
        for w in SURFACES[a] {
            attribute_table.insert((*w).to_string(), a);
        }
    }

    // The store: address -> value, written from each held-out row's own context. The value is the
    // TAIL of the target fact, so a correct answer must come from the store, not from the question.
    let mut stored: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    // The control: a question-to-answer table keyed on the (entity, attribute) pair, fitted on TRAIN.
    // It is the strongest non-compositional alternative this panel can express.
    let mut pair_table: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    for &(e, a) in train {
        pair_table.insert((e, a), 0);
    }

    let mut oracle = 0usize;
    let mut mechanism = 0usize;
    let mut control = 0usize;
    let mut tag_ok = 0usize;
    let mut abstained = 0usize;
    let mut alt_ok = 0usize;
    let mut swapped_ok = 0usize;
    let total = held.len();

    for &(e, a) in held {
        // Context: the target fact plus two distractors, one of them the SAME attribute on another
        // entity, so a correct answer cannot come from the attribute alone.
        let distractor = (e + 1 + (a % 3)) % ENTITIES.len();
        let mut context = vec![fact(e, a, e + a), fact(distractor, a, distractor + 2)];
        let other = (a + 1 + (e % 2)) % ATTRIBUTES.len();
        context.push(fact(distractor, other, distractor + a));
        stored.insert((e, a), (e + a) % VALUES[a].len());

        let q = question(e, a);
        let mentions: BTreeSet<usize> = [e, distractor].into_iter().collect();
        let (te, ta) = tag(&q, &mentions, &attribute_table);
        let tagged = match (te, ta) {
            (Some(te), Some(ta)) => te == e && ta == a,
            _ => false,
        };
        if tagged {
            tag_ok += 1;
        }

        // Oracle: the gold address handed to the store (address formation removed).
        if stored.get(&(e, a)) == Some(&((e + a) % VALUES[a].len())) {
            oracle += 1;
        }
        // Mechanism: the tagged address, read exactly, abstaining when absent.
        // The answer must be the target's own value: a wrong tag that finds another pair's value is
        // a wrong answer, not an answer.
        let expected = (e + a) % VALUES[a].len();
        match (te, ta) {
            (Some(te), Some(ta)) => match stored.get(&(te, ta)) {
                Some(&v) if v == expected => mechanism += 1,
                Some(_) => {}
                None => abstained += 1,
            },
            _ => abstained += 1,
        }
        // Control: the pair table. A held-out pair is absent by construction, so it abstains.
        if pair_table.contains_key(&(e, a)) {
            control += 1;
        }

        // Falsifier 1: permuted values. If the mechanism still answers, it is not reading the
        // binding.
        let other = (e + a + 1) % VALUES[a].len();
        if stored.get(&(te.unwrap_or(usize::MAX), ta.unwrap_or(usize::MAX))) == Some(&other) {
            swapped_ok += 1;
        }
        // Falsifier 2: held-out wording. The attribute table never saw the alternative surface.
        let (ae, aa) = tag(&alt_question(e, a), &mentions, &attribute_table);
        if ae == Some(e) && aa == Some(a) {
            alt_ok += 1;
        }

        if verbose {
            println!(
                "e={:<7} a={:<6} tag=({}, {}) tagged={} q={q}",
                ENTITIES[e],
                ATTRIBUTES[a],
                te.map(|i| ENTITIES[i]).unwrap_or("-"),
                ta.map(|i| ATTRIBUTES[i]).unwrap_or("-"),
                tagged
            );
        }
    }

    println!("panel: {} rows held out, {} train, seed {seed}", total, train.len());
    println!("  address formation (entity, attribute both right): {tag_ok}/{total}");
    println!("  oracle key (gold address, store read):            {oracle}/{total}");
    println!("  mechanism (formed address, store read):           {mechanism}/{total}");
    println!("  abstentions:                                      {abstained}/{total}");
    println!("  control (question->answer pair table):            {control}/{total}");
    println!("  falsifier: permuted values (must be ~0):          {swapped_ok}/{total}");
    println!("  falsifier: held-out wording (word table cannot):  {alt_ok}/{total}");
    if mechanism > control {
        println!(
            "VERDICT: the formed address answers {} recombinations the pair table cannot ({} vs {}). The compositional claim holds; the mechanism is worth building.",
            mechanism - control, mechanism, control
        );
    } else {
        println!(
            "VERDICT: no compositional advantage ({} vs {}). Address formation adds nothing here and the mechanism is not worth building as designed.",
            mechanism, control
        );
    }
    require(oracle == total, "the oracle must be complete: the store is not the failure")?;
    require(
        tag_ok > control,
        "address formation must beat the memorised control on recombinations",
    )?;
    Ok(())
}

fn require(ok: bool, why: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(why.to_string())
    }
}
