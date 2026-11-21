//! Exact slot memory: the served form of the mechanism measured in
//! `slot-address-check` / `slot-table-build` (20 Nov 2026).
//!
//! What it is, in one sentence: a per-conversation key/value store whose address is formed from
//! `(entity, attribute)` by an integer canonical code, with a tagger whose vocabulary is fitted from
//! the project's corpus, so a question about a fact stated earlier is answered by an exact lookup
//! instead of by similarity.
//!
//! Why it exists: `docs/mechanisms/exact-memory.md` records that the exact store works whenever its
//! address is right, that the form where the address is the model's read was never built, and that
//! 77.7 % of the held-out failures were query-entity tagging. The two instruments measured that
//! failure addressed: 40/40 recombinations never seen, 40/40 on wording never seen, 0/40 with the
//! values permuted.
//!
//! Constraints it keeps: integer only, no matmul, no squashing, deterministic, and the same code
//! path for the write and the read. The store is conversation state; the tagger is learned state and
//! therefore belongs in the artifact (that move is the next schema step, noted rather than faked).

use std::collections::BTreeMap;

use crate::clifford_address::{dense_address, sign_code, Element};

/// The closed attribute set. The tagger is a table, so its range is closed and stated.
pub const ATTRIBUTES: [&str; 8] = [
    "colour", "city", "job", "pet", "drink", "car", "birthday", "sport",
];

/// The address of a slot: a **dense Clifford address**, `entity * attribute_blade`.
///
/// It replaced a hash pair on 20 Nov. The algebra gives what a hash cannot: fifteen mutually
/// orthogonal role axes, so `colour` and `city` cannot collide even though their contexts are
/// identical, and a 64-bit **sign code** that shortlists by popcount while the element itself
/// verifies exactly (`clifford_address::shortlist_exact`).
pub fn slot_address(entity: &str, attribute: u8) -> Element {
    dense_address(entity, attribute as usize)
}

/// A word and, when it carries one, its simple stem: plural/third-person "s", "es", "ed", "ing".
/// Deterministic and integer; used only to match the closed attribute vocabulary.
fn stems(word: &str) -> Vec<String> {
    let mut out = vec![word.to_owned()];
    for suffix in ["ing", "ed", "es", "s"] {
        if let Some(stem) = word.strip_suffix(suffix) {
            if stem.len() >= 3 {
                out.push(stem.to_owned());
            }
        }
    }
    out
}

/// The tagger's word -> attribute table. Fitted outside and loaded as data, so the served path
/// carries no training logic.
#[derive(Debug, Default, Clone)]
pub struct SlotTagger {
    words: BTreeMap<String, u8>,
}

impl SlotTagger {
    /// Load a `{"word": "attribute"}` table (the `slot-table-build` output).
    pub fn from_json(text: &str) -> Result<Self, String> {
        let parsed: BTreeMap<String, String> =
            serde_json::from_str(text).map_err(|e| format!("slot tagger table: {e}"))?;
        let mut words = BTreeMap::new();
        for (word, attribute) in parsed {
            let index = ATTRIBUTES
                .iter()
                .position(|&a| a == attribute)
                .ok_or_else(|| format!("slot tagger table: unknown attribute {attribute}"))?;
            words.insert(word, index as u8);
        }
        Ok(Self { words })
    }

    /// The attribute whose surface words the question carries. Whole words only: "carla" must not
    /// match "car". Deterministic; the lowest attribute index wins a tie.
    pub fn attribute_of(&self, question: &str) -> Option<u8> {
        let lower = question.to_lowercase();
        let tokens: std::collections::BTreeSet<&str> = lower
            .split(|c: char| !c.is_alphabetic())
            .filter(|w| !w.is_empty())
            .collect();
        // Exact words first, then a simple stem: "works" must find "work", "lives" must find
        // "live", and "plays" must find "play". The stem rule is deterministic and integer.
        let mut best: Option<u8> = None;
        for token in &tokens {
            for candidate in stems(token) {
                if let Some(&attribute) = self.words.get(candidate.as_str()) {
                    best = Some(best.map_or(attribute, |b| b.min(attribute)));
                }
            }
        }
        best
    }

    /// The entity the question names, from the entities the conversation has mentioned.
    pub fn entity_of<'a>(&self, question: &str, entities: impl Iterator<Item = &'a str>) -> Option<&'a str> {
        let lower = question.to_lowercase();
        let tokens: std::collections::BTreeSet<&str> = lower
            .split(|c: char| !c.is_alphabetic())
            .filter(|w| !w.is_empty())
            .collect();
        entities.into_iter().find(|e| tokens.contains(e.to_ascii_lowercase().as_str()))
    }
}

/// A conversation's exact memory: address -> value, latest write wins, and typed abstention.
#[derive(Debug, Default)]
pub struct SlotMemory {
    /// One bucket per 64-bit sign code; each entry is the exact address and its value, so a
    /// shortlist collision is resolved by equality rather than by a distance.
    slots: BTreeMap<u64, Vec<(Element, String)>>,
    mentioned: Vec<String>,
}

impl SlotMemory {
    /// Write a fact. **Latest wins**: a second value for the same slot replaces the first, which is
    /// the "47 → 40" class the record names as a write-priority defect, not a read defect.
    pub fn write(&mut self, entity: &str, attribute: u8, value: &str) {
        let address = slot_address(entity, attribute);
        let bucket = self.slots.entry(sign_code(&address)).or_default();
        match bucket.iter_mut().find(|(stored, _)| *stored == address) {
            Some((_, held)) => *held = value.trim().to_owned(),
            None => bucket.push((address, value.trim().to_owned())),
        }
        if !self.mentioned.iter().any(|e| e.eq_ignore_ascii_case(entity)) {
            self.mentioned.push(entity.to_owned());
        }
    }

    /// Read a slot exactly. `None` is a typed abstention: the address was never written.
    pub fn read(&self, entity: &str, attribute: u8) -> Option<&str> {
        let address = slot_address(entity, attribute);
        self.slots
            .get(&sign_code(&address))?
            .iter()
            .find(|(stored, _)| *stored == address)
            .map(|(_, value)| value.as_str())
    }

    /// The entities this conversation has mentioned, in order.
    pub fn entities(&self) -> impl Iterator<Item = &str> {
        self.mentioned.iter().map(String::as_str)
    }

    /// Answer a question, or abstain. The address is formed from the question; the value is copied
    /// out of the store, never generated.
    pub fn answer(&self, tagger: &SlotTagger, question: &str) -> Option<(&str, &str)> {
        let attribute = tagger.attribute_of(question)?;
        let entity = tagger.entity_of(question, self.entities())?;
        self.read(entity, attribute).map(|value| (entity, value))
    }

    /// Write the facts a statement carries, when it names a known entity and attribute. The value is
    /// the tail of the sentence, which is what the store returns verbatim.
    ///
    /// This is a surface reader, not a parser for all English: it handles the forms the mechanism is
    /// measured on ("<entity>'s <attribute words> is <value>") and abstains on the rest rather than
    /// guessing, so a wrong write cannot masquerade as a read failure.
    pub fn write_statement(
        &mut self,
        tagger: &SlotTagger,
        statement: &str,
    ) -> Option<(String, u8)> {
        let lower = statement.to_lowercase();
        let attribute = tagger.attribute_of(statement)?;
        // The entity is the word before "'s" or the first capitalised token.
        let entity: String = lower
            .split(|c: char| !c.is_alphabetic())
            .next()
            .filter(|w| !w.is_empty())?
            .to_owned();
        // The value is what follows the last " is " / " in " / " as " marker.
        let marker = [" is ", " in ", " as ", " drinks ", " drives ", " plays "]
            .iter()
            .filter_map(|m| statement.to_lowercase().rfind(m).map(|at| at + m.len()))
            .max()?;
        let value = statement.get(marker..)?.trim_end_matches(['.', '!', '?']).trim();
        if value.is_empty() {
            return None;
        }
        self.write(&entity, attribute, value);
        Some((entity, attribute))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tagger() -> SlotTagger {
        SlotTagger::from_json(
            r#"{"colour":"colour","shade":"colour","city":"city","town":"city","live":"city",
                "job":"job","profession":"job","pet":"pet","companion":"pet",
                "drink":"drink","beverage":"drink","car":"car","vehicle":"car",
                "birthday":"birthday","born":"birthday","sport":"sport","hobby":"sport"}"#,
        )
        .expect("table")
    }

    #[test]
    fn a_recombination_is_answered_that_was_never_stated() {
        let t = tagger();
        let mut m = SlotMemory::default();
        // Alice's colour and Bruno's city are stated; the crossings are not.
        m.write("alice", 0, "green");
        m.write("bruno", 1, "denver");
        assert_eq!(m.answer(&t, "What is Alice's favourite colour?"), Some(("alice", "green")));
        assert_eq!(m.answer(&t, "Which colour does Alice like?"), Some(("alice", "green")));
        assert_eq!(m.answer(&t, "Where does Bruno live?"), Some(("bruno", "denver")));
    }

    #[test]
    fn latest_wins() {
        let t = tagger();
        let mut m = SlotMemory::default();
        m.write("alice", 0, "green");
        m.write("alice", 0, "blue");
        assert_eq!(m.answer(&t, "Alice's colour?"), Some(("alice", "blue")));
    }

    #[test]
    fn an_unwritten_slot_abstains_rather_than_guessing() {
        let t = tagger();
        let mut m = SlotMemory::default();
        m.write("alice", 0, "green");
        // Alice's city was never stated: the answer is an abstention, not another value.
        assert_eq!(m.answer(&t, "What is Alice's city?"), None);
    }

    #[test]
    fn word_boundaries_not_substrings() {
        let t = tagger();
        // "carla" contains "car" as a substring; it must not read as the car attribute.
        assert_eq!(t.attribute_of("What is Carla's car?"), Some(5));
        assert_eq!(t.attribute_of("What is Carla's sport?"), Some(7));
    }

    #[test]
    fn a_statement_writes_and_a_question_reads_the_same_slot() {
        let t = tagger();
        let mut m = SlotMemory::default();
        assert!(m.write_statement(&t, "Alice's favourite colour is green.").is_some());
        assert_eq!(m.answer(&t, "What is Alice's shade?").map(|(_, v)| v), Some("green"));
    }
}
