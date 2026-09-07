//! Deferred relation writes preserve one exact bounded value extent. The first
//! WordAtom remains the semantic anchor; its bytes and geometry are not rewritten.
use super::relation::{self, RelationState};
use super::value_lexemes::{LexemeState, WordAtom};
use super::value_types::{ValueState, ValueWork};
use super::word_copy_types::WordCopyWork;
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RelationSpan {
    /// Complete value bytes, including the unchanged first-word payload.
    pub bytes: [u8; 28],
    pub len: u8,
    pub extra_words: u8,
    pub terminal: WordAtom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PendingRelation {
    pub owner: WordAtom,
    pub value: WordAtom,
    pub action: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<RelationSpan>,
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
pub(super) fn payload<'a>(value: &'a WordAtom, span: Option<&'a RelationSpan>) -> Option<&'a [u8]> {
    match span {
        Some(span) => span.bytes.get(..usize::from(span.len)),
        None => value.bytes.get(..usize::from(value.len)),
    }
}

pub(super) fn same_value(
    a: &WordAtom,
    a_span: Option<&RelationSpan>,
    b: &WordAtom,
    b_span: Option<&RelationSpan>,
    work: &mut ValueWork,
) -> bool {
    if a_span.is_none() && b_span.is_none() {
        return a.matches(b, work);
    }
    work.lexical_comparisons = work.lexical_comparisons.saturating_add(1);
    let Some((a, b)) = payload(a, a_span).zip(payload(b, b_span)) else {
        return false;
    };
    if a.is_empty() || a.len() != b.len() {
        return false;
    }
    for (&a, &b) in a.iter().zip(b) {
        work.lexical_byte_comparisons = work.lexical_byte_comparisons.saturating_add(1);
        if a != b {
            return false;
        }
    }
    true
}

impl PendingRelation {
    fn terminal(&self) -> &WordAtom {
        self.span
            .as_ref()
            .map_or(&self.value, |span| &span.terminal)
    }

    /// Return true only when the actual next source word was consumed.
    fn extend(&mut self, model: &Model, next: WordAtom, work: &mut ValueWork) -> bool {
        work.relations.span_edge_checks = work.relations.span_edge_checks.saturating_add(1);
        let prior = *self.terminal();
        let Some(separator) = next.leading_gap else {
            return false;
        };
        if next.byte_end.checked_sub(u64::from(next.len)) != prior.byte_end.checked_add(1) {
            return false;
        }
        let length = self.span.as_ref().map_or(self.value.len, |span| span.len);
        let Some(new_length) = length
            .checked_add(1)
            .and_then(|len| len.checked_add(next.len))
            .filter(|&len| len <= 28)
        else {
            return false;
        };
        let mut routing = WordCopyWork::default();
        let advance = super::source_span::learned_advance(
            model,
            &self.value,
            &next,
            separator,
            Control::Full,
            &mut routing,
        );
        work.relations.span_routing.add(routing.routing);
        work.relations.record_reads = work
            .relations
            .record_reads
            .saturating_add(routing.word_record_reads);
        work.relations.dictionary_comparisons = work
            .relations
            .dictionary_comparisons
            .saturating_add(routing.dictionary_comparisons);
        work.relations.dictionary_byte_comparisons = work
            .relations
            .dictionary_byte_comparisons
            .saturating_add(routing.dictionary_byte_comparisons);
        if !advance {
            return false;
        }
        if self.span.is_none() {
            work.relations.span_byte_writes = work
                .relations
                .span_byte_writes
                .saturating_add(u64::from(self.value.len));
        }
        let span = self.span.get_or_insert_with(|| {
            let mut bytes = [0; 28];
            bytes[..usize::from(self.value.len)]
                .copy_from_slice(&self.value.bytes[..usize::from(self.value.len)]);
            RelationSpan {
                bytes,
                len: self.value.len,
                extra_words: 0,
                terminal: self.value,
            }
        });
        span.bytes[usize::from(length)] = separator;
        span.bytes[usize::from(length) + 1..usize::from(new_length)]
            .copy_from_slice(&next.bytes[..usize::from(next.len)]);
        span.len = new_length;
        span.extra_words += 1;
        span.terminal = next;
        work.relations.span_byte_writes = work
            .relations
            .span_byte_writes
            .saturating_add(u64::from(next.len) + 1);
        true
    }
}

impl RelationState {
    /// Explicit response boundaries finalize source-only pending writes once.
    pub(super) fn finish_span(&mut self, work: &mut ValueWork) {
        if let Some(pending) = self.pending.take() {
            self.commit_span(
                pending.owner,
                pending.value,
                pending.span,
                pending.action,
                work,
            );
        }
    }

    pub(super) fn observe_span(
        &mut self,
        model: &Model,
        words: &LexemeState,
        work: &mut ValueWork,
    ) {
        if let Some(pending) = &mut self.pending {
            if pending.extend(model, words.recent[0], work) {
                return;
            }
            self.finish_span(work);
        }
        let Some((owner, value, action)) =
            relation::write_choice(model, &words.recent[..words.recent_len], work)
        else {
            work.relations.no_writes = work.relations.no_writes.saturating_add(1);
            return;
        };
        // A reverse write completes at the owner and binds only an earlier
        // value anchor. It supplies no extent boundary for intervening words.
        if value != 0 {
            self.commit_span(words.recent[owner], words.recent[value], None, action, work);
            return;
        }
        let pending = PendingRelation {
            owner: words.recent[owner],
            value: words.recent[value],
            action,
            span: None,
        };
        self.pending = Some(pending);
    }
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

fn initial(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}
fn component(byte: u8) -> bool {
    initial(byte) || byte.is_ascii_digit()
}

impl RelationSpan {
    fn shape_valid(
        &self,
        anchor: &WordAtom,
        seen: u64,
        source_bytes: u64,
        geometry_len: usize,
    ) -> bool {
        let len = usize::from(self.len);
        let anchor_len = usize::from(anchor.len);
        if self.extra_words == 0
            || self.extra_words > 15
            || len > 28
            || len <= anchor_len
            || self.bytes[len..].iter().any(|&b| b != 0)
            || self.bytes.get(..anchor_len) != anchor.bytes.get(..anchor_len)
            || !self
                .terminal
                .snapshot_valid(seen, source_bytes, geometry_len)
            || self.terminal.end < anchor.end
            || self.terminal.byte_end.checked_sub(anchor.byte_end)
                != Some(u64::from(self.len) - u64::from(anchor.len))
        {
            return false;
        }
        let mut words = 0_u8;
        let mut offset = 0;
        let mut final_start = 0;
        while offset < len {
            if !initial(self.bytes[offset]) {
                return false;
            }
            final_start = offset;
            words += 1;
            while offset < len && component(self.bytes[offset]) {
                offset += 1;
            }
            if offset < len {
                if !self.bytes[offset].is_ascii() || offset + 1 == len {
                    return false;
                }
                offset += 1;
            }
        }
        words == self.extra_words + 1
            && len - final_start == usize::from(self.terminal.len)
            && self.bytes[final_start..len] == self.terminal.bytes[..usize::from(self.terminal.len)]
            && self.terminal.leading_gap == Some(self.bytes[final_start - 1])
    }

    fn learned_valid(&self, anchor: &WordAtom, model: &Model) -> bool {
        let mut offset = usize::from(anchor.len);
        while offset < usize::from(self.len) {
            let separator = self.bytes[offset];
            offset += 1;
            let start = offset;
            while offset < usize::from(self.len) && component(self.bytes[offset]) {
                offset += 1;
            }
            let mut next = WordAtom {
                len: (offset - start) as u8,
                ..Default::default()
            };
            next.bytes[..usize::from(next.len)].copy_from_slice(&self.bytes[start..offset]);
            if !super::source_span::learned_advance(
                model,
                anchor,
                &next,
                separator,
                Control::Full,
                &mut WordCopyWork::default(),
            ) {
                return false;
            }
        }
        true
    }
}

impl RelationState {
    pub(super) fn validate_spans(&self, values: &ValueState, model: &Model) -> Result<()> {
        let fail = || Error("invalid retained relation span or pending write".into());
        let words = values.lexemes.as_ref().ok_or_else(fail)?;
        if words.recent_len > words.recent.len() {
            return Err(fail());
        }
        if model.relation_spans.is_none()
            && (self.pending.is_some() || self.records.iter().any(|r| r.span.is_some()))
        {
            return Err(fail());
        }
        let valid_atom = |atom: &WordAtom| {
            atom.len != 0
                && atom.snapshot_valid(
                    values.seen,
                    words.source_bytes_seen,
                    model.geometry.inverses.len(),
                )
        };
        let visible_atom = |atom: &WordAtom| {
            words.recent[..words.recent_len]
                .iter()
                .filter(|word| word.byte_end == atom.byte_end)
                .all(|word| word == atom)
        };
        let valid_span = |anchor: &WordAtom, span: &RelationSpan| {
            if !span.shape_valid(
                anchor,
                values.seen,
                words.source_bytes_seen,
                model.geometry.inverses.len(),
            ) || !span.learned_valid(anchor, model)
                || !visible_atom(&span.terminal)
            {
                return false;
            }
            let start = anchor.byte_end + 1 - u64::from(anchor.len);
            words.recent[..words.recent_len]
                .iter()
                .filter(|word| word.byte_end >= start && word.byte_end <= span.terminal.byte_end)
                .all(|word| {
                    let Some(end) = word
                        .byte_end
                        .checked_sub(start)
                        .and_then(|n| n.checked_add(1))
                    else {
                        return false;
                    };
                    let Some(begin) = end.checked_sub(u64::from(word.len)) else {
                        return false;
                    };
                    span.bytes.get(begin as usize..end as usize)
                        == word.bytes.get(..usize::from(word.len))
                        && (begin == 0
                            || word
                                .leading_gap
                                .is_none_or(|gap| span.bytes[begin as usize - 1] == gap))
                })
        };
        for record in &self.records {
            if let Some(span) = &record.span {
                if record.owner.byte_end >= record.value.byte_end
                    || !valid_atom(&record.value)
                    || !visible_atom(&record.value)
                    || !valid_span(&record.value, span)
                {
                    return Err(fail());
                }
            }
        }
        if let Some(pending) = &self.pending {
            if values.active
                || pending.owner.byte_end >= pending.value.byte_end
                || !(1..=3).contains(&pending.action)
                || !valid_atom(&pending.owner)
                || !valid_atom(&pending.value)
                || !visible_atom(&pending.owner)
                || !visible_atom(&pending.value)
                || pending
                    .span
                    .as_ref()
                    .is_some_and(|span| !valid_span(&pending.value, span))
                || words.recent_len == 0
                || words.recent[0] != *pending.terminal()
                || self.last_word_end != Some(pending.terminal().byte_end)
            {
                return Err(fail());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom(text: &str, end: u64) -> WordAtom {
        let mut word = WordAtom {
            len: text.len() as u8,
            end,
            byte_end: end,
            ..Default::default()
        };
        word.bytes[..text.len()].copy_from_slice(text.as_bytes());
        word
    }
    fn span(text: &str, end: u64) -> RelationSpan {
        let mut bytes = [0; 28];
        bytes[..text.len()].copy_from_slice(text.as_bytes());
        let last = text.rsplit(' ').next().unwrap();
        let mut terminal = atom(last, end);
        terminal.leading_gap = Some(b' ');
        RelationSpan {
            bytes,
            len: text.len() as u8,
            extra_words: 1,
            terminal,
        }
    }

    #[test]
    fn retained_relation_span_conflict_compares_complete_value_and_revision_clears_it() {
        let mut state = RelationState::default();
        let mut work = ValueWork::default();
        state.commit_span(
            atom("ada", 2),
            atom("New", 10),
            Some(span("New York", 15)),
            1,
            &mut work,
        );
        state.commit_span(
            atom("ada", 22),
            atom("New", 30),
            Some(span("New Jersey", 37)),
            1,
            &mut work,
        );
        assert!(state.record(2).unwrap().conflict);
        assert_eq!(state.record(2).unwrap().previous, 1);
        state.commit_span(
            atom("ada", 42),
            atom("New", 50),
            Some(span("New York", 55)),
            2,
            &mut work,
        );
        assert!(!state.record(3).unwrap().conflict);
        assert_eq!(
            payload(
                &state.record(1).unwrap().value,
                state.record(1).unwrap().span.as_ref()
            )
            .unwrap(),
            b"New York"
        );
        assert_eq!(state.record(1).unwrap().value, atom("New", 10));
    }

    #[test]
    fn retained_relation_pending_flush_commits_once_and_old_fields_stay_absent() {
        let mut state = RelationState::default();
        let mut work = ValueWork::default();
        let wire = serde_json::to_string(&state).unwrap();
        assert!(!wire.contains("pending"));
        assert!(!wire.contains("span"));
        state.pending = Some(PendingRelation {
            owner: atom("ada", 2),
            value: atom("New", 10),
            action: 1,
            span: Some(span("New York", 15)),
        });
        assert_eq!(state.next_id, 1);
        let mut restored: RelationState =
            serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
        restored.finish_span(&mut work);
        restored.finish_span(&mut work);
        assert_eq!(restored.next_id, 2);
        assert_eq!(work.relations.record_writes, 1);
        assert_eq!(restored.record(1).unwrap().span.unwrap().len, 8);
    }

    #[test]
    fn retained_relation_span_shape_binds_padding_anchor_terminal_and_word_count() {
        let anchor = atom("New", 10);
        let good = span("New York", 15);
        assert!(good.shape_valid(&anchor, 100, 100, 1));
        let mut bad = good;
        bad.bytes[27] = 1;
        assert!(!bad.shape_valid(&anchor, 100, 100, 1));
        let mut bad = good;
        bad.bytes[0] = b'O';
        assert!(!bad.shape_valid(&anchor, 100, 100, 1));
        let mut bad = good;
        bad.terminal.byte_end += 1;
        assert!(!bad.shape_valid(&anchor, 100, 100, 1));
        let mut bad = good;
        bad.extra_words = 2;
        assert!(!bad.shape_valid(&anchor, 100, 100, 1));
        let mut bad = good;
        bad.len = 255;
        assert!(!bad.shape_valid(&anchor, 100, 100, 1));
    }
}
