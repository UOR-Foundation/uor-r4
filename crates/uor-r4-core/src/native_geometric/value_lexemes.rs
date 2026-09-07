//! Fixed whole-word identity across lexical-token and byte-token boundaries.
//! This codec recognizes no task grammar. ASCII letter/underscore words may
//! contain later digits; numeric-leading, non-ASCII and oversized runs are
//! rejected through the next delimiter. Equality is exact bytes, never a hash
//! distance. Punctuation separates words and is not assigned semantic meaning.
use super::value_types::{ValueEntry, ValueWork};
use super::PHASE_CHANNELS;
use serde::{Deserialize, Serialize};

pub(super) const WORD_BYTES: usize = 32;
pub(super) const WORD_QUERY: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct WordAtom {
    /// Four exact preceding completed words, owned by this occurrence.
    #[serde(default, skip_serializing_if = "predecessors_empty")]
    pub predecessors: [WordIdentity; 4],
    pub bytes: [u8; WORD_BYTES],
    pub len: u8,
    /// Inclusive token sequence of the final byte.
    pub end: u64,
    /// Zero-based source-byte ordinal, excluding generated response bytes.
    pub byte_end: u64,
    pub pose: u16,
    pub phases: [u16; PHASE_CHANNELS],
}

/// Nonrecursive source identity; no payload or geometry is inferred from a hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct WordIdentity {
    pub bytes: [u8; WORD_BYTES],
    pub len: u8,
    pub end: u64,
    pub byte_end: u64,
}
fn predecessors_empty(value: &[WordIdentity; 4]) -> bool {
    *value == [WordIdentity::default(); 4]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct WordScanner {
    pub pending: WordAtom,
    pub rejected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct LexemeState {
    pub scanner: WordScanner,
    pub source_bytes_seen: u64,
    /// Completed words, newest first; unused entries are exactly default.
    pub recent: [WordAtom; WORD_QUERY],
    pub recent_len: usize,
    /// Frozen at the caller's response boundary, newest first.
    pub queries: [WordAtom; WORD_QUERY],
    pub query_len: usize,
    /// Captured upon entering an open numeric fragment, before its delimiter.
    pub literal_cues: [WordAtom; 4],
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
fn initial(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn component(byte: u8) -> bool {
    initial(byte) || byte.is_ascii_digit() || !byte.is_ascii()
}

impl WordAtom {
    pub(super) fn matches(&self, other: &Self, work: &mut ValueWork) -> bool {
        work.lexical_comparisons = work.lexical_comparisons.saturating_add(1);
        if self.len == 0 || self.len != other.len {
            return false;
        }
        for index in 0..usize::from(self.len) {
            work.lexical_byte_comparisons = work.lexical_byte_comparisons.saturating_add(1);
            if self.bytes[index] != other.bytes[index] {
                return false;
            }
        }
        true
    }
}

impl WordScanner {
    fn feed(&mut self, byte: u8, ordinal: u64, entry: ValueEntry) -> Option<WordAtom> {
        if !component(byte) {
            return self.finish();
        }
        if self.rejected {
            return None;
        }
        if !byte.is_ascii()
            || (self.pending.len == 0 && !initial(byte))
            || usize::from(self.pending.len) == WORD_BYTES
        {
            self.pending = WordAtom::default();
            self.rejected = true;
            return None;
        }
        self.pending.bytes[usize::from(self.pending.len)] = byte;
        self.pending.len += 1;
        self.pending.end = entry.sequence;
        self.pending.byte_end = ordinal;
        self.pending.pose = entry.pose;
        self.pending.phases = entry.phases;
        None
    }

    fn finish(&mut self) -> Option<WordAtom> {
        let word = (self.pending.len != 0 && !self.rejected).then_some(self.pending);
        *self = Self::default();
        word
    }
}

impl LexemeState {
    fn append(&mut self, mut word: WordAtom, work: &mut ValueWork) {
        for (prior, source) in word.predecessors.iter_mut().zip(&self.recent[..4]) {
            *prior = WordIdentity {
                bytes: source.bytes,
                len: source.len,
                end: source.end,
                byte_end: source.byte_end,
            };
        }
        for index in (1..WORD_QUERY).rev() {
            self.recent[index] = self.recent[index - 1];
        }
        self.recent[0] = word;
        self.recent_len = (self.recent_len + 1).min(WORD_QUERY);
        work.lexical_writes = work.lexical_writes.saturating_add(1);
    }

    pub(super) fn feed(&mut self, byte: u8, entry: ValueEntry, work: &mut ValueWork) {
        let Some(next) = self.source_bytes_seen.checked_add(1) else {
            self.scanner.pending = WordAtom::default();
            self.scanner.rejected = true;
            return;
        };
        if let Some(word) = self.scanner.feed(byte, self.source_bytes_seen, entry) {
            self.append(word, work);
        }
        self.source_bytes_seen = next;
    }

    pub(super) fn finish(&mut self, work: &mut ValueWork) {
        if let Some(word) = self.scanner.finish() {
            self.append(word, work);
        }
    }

    pub(super) fn capture_literal(&mut self) {
        self.literal_cues.copy_from_slice(&self.recent[..4]);
    }

    pub(super) fn clear_literal(&mut self) {
        self.literal_cues = [WordAtom::default(); 4];
    }

    pub(super) fn begin(&mut self) {
        self.queries = self.recent;
        self.query_len = self.recent_len;
        self.clear_literal();
    }

    pub(super) fn end(&mut self) {
        self.scanner = WordScanner::default();
        self.queries = [WordAtom::default(); WORD_QUERY];
        self.query_len = 0;
        self.clear_literal();
    }
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

impl WordAtom {
    /// Older checkpoints omit this metadata. When present, copies of identities
    /// still in the same window must agree with those original occurrences.
    pub(super) fn predecessors_match_window(&self, older: &[WordAtom]) -> bool {
        predecessors_empty(&self.predecessors)
            || self.predecessors.iter().zip(older).all(|(prior, word)| {
                prior.bytes == word.bytes
                    && prior.len == word.len
                    && prior.end == word.end
                    && prior.byte_end == word.byte_end
            })
    }

    fn predecessors_valid(&self) -> bool {
        let mut later_end = self.end;
        let mut later_byte = self.byte_end;
        let mut later_len = self.len;
        let mut empty = false;
        for prior in &self.predecessors {
            if prior.len == 0 {
                if *prior != WordIdentity::default() {
                    return false;
                }
                empty = true;
                continue;
            }
            let len = usize::from(prior.len);
            if empty
                || len > WORD_BYTES
                || !initial(prior.bytes[0])
                || !prior.bytes[..len]
                    .iter()
                    .all(|&b| initial(b) || b.is_ascii_digit())
                || prior.bytes[len..].iter().any(|&b| b != 0)
                || prior.end > later_end
                || later_byte < u64::from(later_len)
                || prior.byte_end > later_byte - u64::from(later_len)
                || prior.byte_end < u64::from(prior.len) - 1
            {
                return false;
            }
            later_end = prior.end;
            later_byte = prior.byte_end;
            later_len = prior.len;
        }
        true
    }
    pub(super) fn snapshot_valid(&self, seen: u64, source_bytes: u64, geometry_len: usize) -> bool {
        if self.len == 0 {
            return *self == Self::default();
        }
        let len = usize::from(self.len);
        len <= WORD_BYTES
            && initial(self.bytes[0])
            && self.bytes[..len]
                .iter()
                .all(|&byte| initial(byte) || byte.is_ascii_digit())
            && self.bytes[len..].iter().all(|&byte| byte == 0)
            && self.end < seen
            && self.byte_end < source_bytes
            && self.byte_end >= u64::from(self.len) - 1
            && usize::from(self.pose) < geometry_len
            && self.predecessors_valid()
    }
}

impl LexemeState {
    pub(super) fn snapshot_valid(&self, seen: u64, geometry_len: usize) -> bool {
        let valid =
            |word: &WordAtom| word.snapshot_valid(seen, self.source_bytes_seen, geometry_len);
        let valid_words = |words: &[WordAtom], len: usize| {
            len <= words.len()
                && words[..len].iter().all(|word| word.len != 0 && valid(word))
                && words[len..].iter().all(|word| *word == WordAtom::default())
                && words[..len]
                    .iter()
                    .enumerate()
                    .all(|(index, word)| word.predecessors_match_window(&words[index + 1..len]))
                && words[..len].windows(2).all(|pair| {
                    pair[0].byte_end >= u64::from(pair[0].len)
                        && pair[1].byte_end <= pair[0].byte_end - u64::from(pair[0].len)
                        && pair[1].end <= pair[0].end
                })
        };
        self.recent_len <= WORD_QUERY
            && self.query_len <= WORD_QUERY
            && valid_words(&self.recent, self.recent_len)
            && valid_words(&self.queries, self.query_len)
            && valid_words(
                &self.literal_cues,
                self.literal_cues
                    .iter()
                    .take_while(|word| word.len != 0)
                    .count(),
            )
            && valid(&self.scanner.pending)
            && (!self.scanner.rejected || self.scanner.pending == WordAtom::default())
            && (self.scanner.pending.len == 0
                || self.scanner.pending.byte_end.checked_add(1) == Some(self.source_bytes_seen))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(state: &mut LexemeState, text: &[u8], token: u64) {
        for &byte in text {
            state.feed(
                byte,
                ValueEntry {
                    sequence: token,
                    ..ValueEntry::default()
                },
                &mut ValueWork::default(),
            );
        }
    }

    #[test]
    fn candidate_predecessors_survive_window_eviction_and_bind_snapshot() {
        let mut a = LexemeState::default();
        let mut b = LexemeState::default();
        feed(&mut a, b"User: ada lives in Rome. ada has 13 coins. other has 7 coins. User: Where is the location of ada? Assistant: ", 0);
        feed(&mut b, b"User: cyra lives in Rome. ada has 13 coins. other has 7 coins. User: Where is the location of ada? Assistant: ", 0);
        a.begin();
        b.begin();
        assert_eq!(a.query_len, WORD_QUERY);
        for (left, right) in a.queries.iter().zip(&b.queries) {
            assert!(left.matches(right, &mut ValueWork::default()));
        }
        let rome = a
            .queries
            .iter()
            .position(|w| &w.bytes[..usize::from(w.len)] == b"Rome")
            .unwrap();
        assert_eq!(rome, 14);
        assert_eq!(&a.queries[rome].predecessors[2].bytes[..3], b"ada");
        assert_eq!(&b.queries[rome].predecessors[2].bytes[..4], b"cyra");
        assert!(a.snapshot_valid(1, 1));
        assert!(b.snapshot_valid(1, 1));
        let bytes = serde_json::to_vec(&a).unwrap();
        let restored: LexemeState = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored, a);
        let mut forged = a;
        // Keep the copied identity's length and ordinals valid while changing
        // a byte. The original predecessor still exists in the same window.
        forged.queries[0].predecessors[0].bytes[0] = b'x';
        assert!(forged.queries[0].snapshot_valid(1, forged.source_bytes_seen, 1));
        assert!(!forged.snapshot_valid(1, 1));
        let mut legacy = a;
        for word in legacy.recent.iter_mut().chain(&mut legacy.queries) {
            word.predecessors = [WordIdentity::default(); 4];
        }
        let bytes = serde_json::to_vec(&legacy).unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("predecessors"));
        let legacy: LexemeState = serde_json::from_slice(&bytes).unwrap();
        assert!(legacy.snapshot_valid(1, 1));
        a.queries[rome].predecessors[2].byte_end = a.queries[rome].byte_end;
        assert!(!a.snapshot_valid(1, 1));
    }

    #[test]
    fn complete_word_identity_is_independent_of_token_boundaries() {
        let mut whole = LexemeState::default();
        let mut split = LexemeState::default();
        feed(&mut whole, b"suri has 73 coins. ", 0);
        for (index, byte) in b"suri has 73 coins. ".iter().enumerate() {
            feed(&mut split, &[*byte], index as u64);
        }
        assert_eq!(whole.recent_len, 3);
        assert_eq!(split.recent_len, 3);
        for (left, right) in whole.recent[..3].iter().zip(&split.recent[..3]) {
            assert!(left.matches(right, &mut ValueWork::default()));
        }
        assert_eq!(&whole.recent[2].bytes[..4], b"suri");
        assert!(whole.snapshot_valid(1, 1));
        assert!(split.snapshot_valid(19, 1));
    }

    #[test]
    fn rejects_whole_oversized_numeric_and_non_ascii_runs() {
        let mut state = LexemeState::default();
        feed(
            &mut state,
            b"abcdefghijklmnopqrstuvwxyzabcdefG x_1 17 17abc a\xc3\xa9z ok ",
            0,
        );
        assert_eq!(state.recent_len, 2);
        assert_eq!(&state.recent[0].bytes[..2], b"ok");
        assert_eq!(&state.recent[1].bytes[..3], b"x_1");
        assert!(state.snapshot_valid(1, 1));
    }

    #[test]
    fn numeric_start_cues_survive_same_token_later_words() {
        let mut state = LexemeState::default();
        feed(&mut state, b"suri has ", 0);
        state.capture_literal();
        feed(&mut state, b"73 coins tavi has 301 coins ", 0);
        assert_eq!(&state.literal_cues[1].bytes[..4], b"suri");
        assert_eq!(&state.recent[1].bytes[..3], b"has");
        state.begin();
        assert_eq!(state.queries, state.recent);
        assert_eq!(state.literal_cues, [WordAtom::default(); 4]);
        state.end();
        assert_eq!(state.query_len, 0);
        assert_ne!(state.recent_len, 0);
    }

    #[test]
    fn unfinished_word_snapshot_resumes_and_bad_shapes_reject() {
        let mut state = LexemeState::default();
        feed(&mut state, b"sur", 0);
        let mut restored: LexemeState =
            serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
        feed(&mut state, b"i ", 1);
        feed(&mut restored, b"i ", 1);
        assert_eq!(restored, state);
        assert!(state.snapshot_valid(2, 1));
        state.recent[0].len = 33;
        assert!(!state.snapshot_valid(2, 1));
    }

    #[test]
    fn value_capture_and_joint_match_preserve_names_inside_one_lexical_token() {
        use super::super::numeral::NUMERAL_CODEC;
        use super::super::value_types::{
            ValueAction, ValueModel, ValueState, LEXEME_VALUE_SCHEMA, VALUES,
        };
        use super::super::{Config, Control, Document, Trainer, BOS, LEXICAL_BASE};

        let documents = [Document {
            id: "word-capture-unit".into(),
            text: "source query".into(),
        }];
        let mut trainer = Trainer::new(Config::default(), &documents).unwrap();
        trainer.train_documents(&documents).unwrap();
        let mut model = trainer.compile().unwrap();
        model.values = Some(ValueModel {
            schema: LEXEME_VALUE_SCHEMA.into(),
            codec: NUMERAL_CODEC.into(),
            capacity: VALUES,
            rows: Vec::new(),
            continuation_score: 0,
            fit_config: [0; 4],
            training: Vec::new(),
        });
        // Mechanical token fixture: the entire source has one geometric token
        // endpoint while literal cue capture still follows decoded byte order.
        model.lexical_pieces[0] = b"suri has 73; tavi has 301; suri".to_vec();
        let mut original = ValueState::new(&model);
        original.observe(&model, BOS, &mut ValueWork::default());
        original.observe(&model, LEXICAL_BASE, &mut ValueWork::default());
        original.begin(&mut ValueWork::default());
        assert_eq!(original.sources.len(), 2);
        assert_eq!(original.sources[0].start, original.sources[1].start);
        let first = original.sources[0].lexical.unwrap();
        let second = original.sources[1].lexical.unwrap();
        assert_eq!(&first[1].bytes[..4], b"suri");
        assert_eq!(&second[1].bytes[..4], b"tavi");
        assert!(first[1].byte_end < second[1].byte_end);

        model.lexical_pieces[0] = b"tavi has 73; suri has 301; suri".to_vec();
        let mut swapped = ValueState::new(&model);
        swapped.observe(&model, BOS, &mut ValueWork::default());
        swapped.observe(&model, LEXICAL_BASE, &mut ValueWork::default());
        swapped.begin(&mut ValueWork::default());
        let joint_first = |state: &ValueState| {
            let operand = state.sources[0];
            let (features, len) = state.features(
                &model,
                ValueAction::Copy,
                operand,
                operand,
                Control::GeometryDisabled,
                &mut ValueWork::default(),
            );
            features[..len]
                .iter()
                .find(|feature| feature.kind == 16 && feature.a == 0)
                .unwrap()
                .b
        };
        assert_eq!(joint_first(&original), 0x22);
        assert_eq!(joint_first(&swapped), 0);
    }
}
