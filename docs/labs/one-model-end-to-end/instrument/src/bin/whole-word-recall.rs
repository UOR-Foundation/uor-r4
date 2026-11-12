//! `whole-word-recall` (claude-352, ORDER decision 10): the place-read sheet's I-w score.
//!
//! Exact match calls a recall reply wrong when it says the stated value and then goes on. The
//! sheet's retake reads each reply as text instead: a reply is right when the stated value's
//! text stands in it as a whole word, case included, and no other value stated in the same
//! conversation stands in it the same way. So "470" is not right for "47".
//!
//! Where each text comes from:
//! - **The stated value** is the reference reply's value run: the corpus ids at the kit's value
//!   start, for its length, decoded and trimmed. The kit's writer finds that run from the twin's
//!   `Memory: {value}.` line as whole words, so it is the twin's text.
//! - **The other values** of a conversation are the value texts of the kit's other entries in the
//!   same document (a document starts at the BOS mark), leaving out any equal to the stated one.
//!   A value stated and never asked is in no line of the twin and is not in the kit.
//! - **A reply** is decoded from its ids with the BOS and end marks left out.
//!
//! It reads a kit (`panel-places.json`), the corpus file the kit was written from, the tokenizer,
//! the panel's rows (`position`, `reference`) and one reader's replies (`i`, `ids`). It refuses a
//! corpus that is not the kit's, rows that are not that corpus's, and a set of rows that does not
//! hold every recall reply of the kit. It also scores the references: the corpus's own replies
//! must all be right, or the rule is refusing what the corpus says, and the program ends non-zero
//! after writing its rows.
//!
//! usage: whole-word-recall --tokenizer FILE --kit FILE --kit-tokens FILE --samples FILE
//!        --replies FILE --out FILE

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, String>;

const USAGE: &str = "usage: whole-word-recall --tokenizer FILE --kit FILE --kit-tokens FILE \
--samples FILE --replies FILE --out FILE";

/// A `UORT` corpus file: this many header bytes, then the ids, two little-endian bytes each.
const HEADER: usize = 64;

/// Whether `text` holds `value` as a whole word: an occurrence with no letter or digit directly
/// before it and none directly after it. The census's rule (`turn_census::holds`).
fn holds(text: &str, value: &str) -> bool {
    !value.is_empty()
        && text.match_indices(value).any(|(i, _)| {
            let before = text[..i].chars().next_back();
            let after = text[i + value.len()..].chars().next();
            !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
        })
}

/// What one recall reply must say and must not say.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Stated {
    value: String,
    others: Vec<String>,
}

/// One text read by the rule.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Judged {
    text: String,
    holds_value: bool,
    holds_other: bool,
}

impl Judged {
    fn right(&self) -> bool {
        self.holds_value && !self.holds_other
    }
}

/// Decodes `ids` with the marks left out and reads the text by the rule.
fn judge(ids: &[u32], stated: &Stated, marks: [u32; 2], decode: &dyn Fn(&[u32]) -> String) -> Judged {
    let kept: Vec<u32> = ids.iter().copied().filter(|id| !marks.contains(id)).collect();
    let text = decode(&kept);
    let holds_value = holds(&text, &stated.value);
    let holds_other = stated.others.iter().any(|other| holds(&text, other));
    Judged { text, holds_value, holds_other }
}

/// The ids of a `UORT` corpus file.
fn corpus_ids(bytes: &[u8], path: &str) -> Result<Vec<u32>> {
    if bytes.len() < HEADER || !bytes.starts_with(b"UORT") || (bytes.len() - HEADER) % 2 != 0 {
        return Err(format!("{path}: not a UORT corpus file"));
    }
    Ok(bytes[HEADER..]
        .chunks_exact(2)
        .map(|pair| u32::from(u16::from_le_bytes([pair[0], pair[1]])))
        .collect())
}

/// For each recall reply of the kit, by its start: the value it states and the other values
/// stated in its conversation. `places` are the kit's `[reply start, place, value start, value
/// length]` rows.
fn stated(
    corpus: &[u32],
    places: &[[usize; 4]],
    bos: u32,
    decode: &dyn Fn(&[u32]) -> String,
) -> Result<HashMap<usize, Stated>> {
    let mut document = 0usize;
    let document_of: Vec<usize> = corpus
        .iter()
        .map(|&id| {
            document += usize::from(id == bos);
            document
        })
        .collect();
    let mut by_document: HashMap<usize, Vec<(usize, String)>> = HashMap::new();
    for &[reply, _, start, len] in places {
        let (Some(&doc), Some(run)) = (document_of.get(reply), corpus.get(start..start + len)) else {
            return Err(format!("the kit's reply at {reply} is past the corpus"));
        };
        let value = decode(run).trim().to_owned();
        if value.is_empty() {
            return Err(format!("the kit's reply at {reply} states no text"));
        }
        by_document.entry(doc).or_default().push((reply, value));
    }
    let mut out = HashMap::new();
    for entries in by_document.values() {
        for (reply, value) in entries {
            let mut others: Vec<String> =
                entries.iter().map(|(_, v)| v.clone()).filter(|v| v != value).collect();
            others.sort();
            others.dedup();
            out.insert(*reply, Stated { value: value.clone(), others });
        }
    }
    Ok(out)
}

fn ids_of(value: &Value, what: &str) -> Result<Vec<u32>> {
    value
        .as_array()
        .ok_or_else(|| format!("{what}: not a list of ids"))?
        .iter()
        .map(|id| id.as_u64().and_then(|id| u32::try_from(id).ok()))
        .collect::<Option<Vec<u32>>>()
        .ok_or_else(|| format!("{what}: not a list of ids"))
}

fn index_of(value: &Value, what: &str) -> Result<usize> {
    value
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| format!("{what}: not a whole number"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn read(path: &str) -> Result<Vec<u8>> {
    fs::read(path).map_err(|e| format!("{path}: {e}"))
}

fn flag(args: &[String], name: &str) -> Result<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1))
        .cloned()
        .ok_or_else(|| format!("{name} is needed\n{USAGE}"))
}

/// Scores one reader's replies. The rows written and the counts returned are the reading; the
/// error after them, if any, says the references themselves are not all right by the rule.
fn run_with_args(args: &[String]) -> Result<Value> {
    if let Some(odd) = args.chunks(2).find(|pair| pair.len() != 2 || !pair[0].starts_with("--")) {
        return Err(format!("{}: every argument is a `--name value` pair\n{USAGE}", odd[0]));
    }
    let paths = ["--tokenizer", "--kit", "--kit-tokens", "--samples", "--replies"]
        .map(|name| flag(args, name));
    let [tokenizer_path, kit_path, tokens_path, samples_path, replies_path] = paths;
    let (tokenizer_path, kit_path, tokens_path) = (tokenizer_path?, kit_path?, tokens_path?);
    let (samples_path, replies_path, out_path) = (samples_path?, replies_path?, flag(args, "--out")?);

    let tokenizer_bytes = read(&tokenizer_path)?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| format!("{tokenizer_path}: not a tokenizer this crate reads"))?;
    let decode = |ids: &[u32]| tokenizer.decode(ids);

    let kit: Value = serde_json::from_slice(&read(&kit_path)?).map_err(|e| format!("{kit_path}: {e}"))?;
    let tokens_bytes = read(&tokens_path)?;
    let tokens_sha256 = sha256_hex(&tokens_bytes);
    if kit["tokens_sha256"].as_str() != Some(tokens_sha256.as_str()) {
        return Err(format!("{tokens_path}: the kit was written from another corpus"));
    }
    let corpus = corpus_ids(&tokens_bytes, &tokens_path)?;
    let bos = u32::try_from(index_of(&kit["marks"]["bos"], "the kit's BOS mark")?)
        .map_err(|_| "the kit's BOS mark is past 32 bits".to_owned())?;
    let eos = u32::try_from(index_of(&kit["marks"]["eos"], "the kit's end mark")?)
        .map_err(|_| "the kit's end mark is past 32 bits".to_owned())?;
    let mut places = Vec::new();
    for row in kit["places"].as_array().ok_or("the kit has no places")? {
        let cells = ids_of(row, "a place of the kit")?;
        let [reply, place, start, len] = cells[..] else {
            return Err("a place of the kit is not four numbers".to_owned());
        };
        places.push([reply as usize, place as usize, start as usize, len as usize]);
    }
    let by_reply = stated(&corpus, &places, bos, &decode)?;

    let samples_bytes = read(&samples_path)?;
    let samples: Vec<Value> =
        serde_json::from_slice(&samples_bytes).map_err(|e| format!("{samples_path}: {e}"))?;
    let replies_bytes = read(&replies_path)?;
    let mut replies: HashMap<usize, Vec<u32>> = HashMap::new();
    for line in String::from_utf8_lossy(&replies_bytes).lines().filter(|l| !l.trim().is_empty()) {
        let row: Value = serde_json::from_str(line).map_err(|e| format!("{replies_path}: {e}"))?;
        let i = index_of(&row["i"], "a reply's row number")?;
        if replies.insert(i, ids_of(&row["ids"], "a reply's ids")?).is_some() {
            return Err(format!("{replies_path}: row {i} is there twice"));
        }
    }
    if replies.len() != samples.len() {
        return Err(format!(
            "{replies_path}: {} replies for {} rows",
            replies.len(),
            samples.len()
        ));
    }

    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&out_path)
        .map_err(|e| format!("{out_path}: {e}"))?;
    let (mut recall, mut right, mut value_held, mut other_held) = (0usize, 0usize, 0usize, 0usize);
    let (mut exact, mut exact_not_right, mut references_right) = (0usize, 0usize, 0usize);
    for (i, sample) in samples.iter().enumerate() {
        let position = index_of(&sample["position"], "a row's position")?;
        let reference = ids_of(&sample["reference"], "a row's reference")?;
        if corpus.get(position..position + reference.len()) != Some(&reference[..]) {
            return Err(format!("{samples_path}: row {i} is not this corpus's"));
        }
        let Some(want) = by_reply.get(&position) else {
            continue;
        };
        let ids = replies.get(&i).ok_or_else(|| format!("{replies_path}: no reply for row {i}"))?;
        let reply = judge(ids, want, [bos, eos], &decode);
        let corpus_says = judge(&reference, want, [bos, eos], &decode);
        let same = *ids == reference;
        recall += 1;
        right += usize::from(reply.right());
        value_held += usize::from(reply.holds_value);
        other_held += usize::from(reply.holds_value && reply.holds_other);
        exact += usize::from(same);
        exact_not_right += usize::from(same && !reply.right());
        references_right += usize::from(corpus_says.right());
        let row = json!({
            "i": i,
            "position": position,
            "value": want.value,
            "others": want.others,
            "reply": reply.text,
            "holds_value": reply.holds_value,
            "holds_other": reply.holds_other,
            "right": reply.right(),
            "exact": same,
            "reference": corpus_says.text,
            "reference_right": corpus_says.right(),
        });
        writeln!(out, "{row}").map_err(|e| format!("{out_path}: {e}"))?;
    }
    if recall != places.len() {
        return Err(format!(
            "{samples_path}: {recall} of the kit's {} recall replies are among the rows",
            places.len()
        ));
    }
    let receipt = json!({
        "recall": recall,
        "right": right,
        "holds_the_value": value_held,
        "holds_the_value_and_another": other_held,
        "exact": exact,
        "exact_and_not_right": exact_not_right,
        "references_right": references_right,
        "kit_tokens_sha256": tokens_sha256,
        "tokenizer_sha256": sha256_hex(&tokenizer_bytes),
        "samples_sha256": sha256_hex(&samples_bytes),
        "replies_sha256": sha256_hex(&replies_bytes),
    });
    if references_right != recall {
        return Err(format!(
            "{receipt}\n{references_right} of {recall} references are right by the rule: the \
             rule refuses what the corpus says"
        ));
    }
    Ok(receipt)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run_with_args(&args) {
        Ok(receipt) => println!("{receipt}"),
        Err(message) => {
            eprintln!("whole-word-recall: {message}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORDS: [&str; 13] = [
        "<bos>", "<eos>", " It", " is", " 47", ".", " Lisbon", " 4", "0", " x", "47", " in", "y",
    ];

    fn decode(ids: &[u32]) -> String {
        ids.iter().map(|&id| WORDS[id as usize]).collect()
    }

    fn ids(words: &[&str]) -> Vec<u32> {
        words
            .iter()
            .map(|word| WORDS.iter().position(|w| w == word).expect("a word of the test") as u32)
            .collect()
    }

    /// Two conversations. The first states 47 and Lisbon and is asked for each; the second
    /// states 4 and is asked for it.
    fn corpus() -> (Vec<u32>, Vec<[usize; 4]>) {
        let words = [
            "<bos>", " 47", " Lisbon", " It", " is", " 47", ".", "<eos>", " It", " is", " Lisbon",
            ".", "<eos>", "<bos>", " 4", " It", " is", " 4", ".", "<eos>",
        ];
        (ids(&words), vec![[3, 1, 5, 1], [8, 2, 10, 1], [15, 14, 17, 1]])
    }

    #[test]
    fn the_values_are_the_kit_runs_and_the_others_are_the_conversations() {
        let (tokens, places) = corpus();
        let by_reply = stated(&tokens, &places, 0, &decode).expect("a kit of this corpus");
        let lisbon = vec!["Lisbon".to_owned()];
        assert_eq!(by_reply[&3], Stated { value: "47".to_owned(), others: lisbon });
        assert_eq!(by_reply[&8], Stated { value: "Lisbon".to_owned(), others: vec!["47".to_owned()] });
        // Another conversation's value is not one of this conversation's others.
        assert_eq!(by_reply[&15], Stated { value: "4".to_owned(), others: Vec::new() });
        assert!(stated(&tokens, &[[3, 1, 19, 4]], 0, &decode).is_err());
    }

    /// The sheet's four hand-built replies, each decoded from ids: one right, one that holds
    /// another value, one that holds half the value, one that holds the value inside a longer
    /// word or number.
    #[test]
    fn the_four_hand_built_replies() {
        let (tokens, places) = corpus();
        let by_reply = stated(&tokens, &places, 0, &decode).expect("a kit of this corpus");
        let want = &by_reply[&3];
        let read = |words: &[&str]| judge(&ids(words), want, [0, 1], &decode);

        let right = read(&[" It", " is", " 47", ".", "<eos>"]);
        assert_eq!(right.text, " It is 47.");
        assert!(right.right());
        // The value and then more text is still right: this is what exact match refuses.
        assert!(read(&[" It", " is", " 47", " in", " x", "<eos>"]).right());

        let another = read(&[" It", " is", " 47", " in", " Lisbon", "."]);
        assert!(another.holds_value && another.holds_other && !another.right());

        let half = read(&[" It", " is", " 4", "."]);
        assert!(!half.holds_value && !half.right());

        let longer_number = read(&[" It", " is", " 47", "0", "."]);
        assert_eq!(longer_number.text, " It is 470.");
        assert!(!longer_number.right());
        assert!(!read(&[" It", " is", " x", "47", "."]).right());
        assert!(!read(&[" It", " is", " 47", "y"]).right());
    }

    #[test]
    fn a_word_is_matched_whole_and_by_case() {
        assert!(holds("You grew up in Lisbon", "Lisbon"));
        assert!(holds("Lisbon.", "Lisbon"));
        assert!(!holds("You grew up in Lisb", "Lisbon"));
        assert!(!holds("You grew up in lisbon", "Lisbon"));
        assert!(!holds("the Lisbonian", "Lisbon"));
        assert!(!holds("anything", ""));
    }

    #[test]
    fn a_corpus_file_is_read_past_its_header() {
        let mut bytes = b"UORT".to_vec();
        bytes.resize(HEADER, 0);
        bytes.extend_from_slice(&[7, 0, 1, 1]);
        assert_eq!(corpus_ids(&bytes, "t").expect("a corpus file"), vec![7, 257]);
        assert!(corpus_ids(&bytes[..HEADER + 3], "t").is_err());
        assert!(corpus_ids(&bytes[4..], "t").is_err());
    }

    #[test]
    fn a_name_without_its_value_is_refused() {
        let args = ["--kit".to_owned()];
        assert!(run_with_args(&args).is_err_and(|e| e.contains("pair")));
    }
}
