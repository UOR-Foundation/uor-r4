//! Learned adjacent-source transport; payload and separators remain exact bytes.
//! This complete file is a bounded integer/table serving kernel.
use super::value_types::{ValueFeature, ValueState};
use super::word_copy_types::WordCopyWork;
use super::*;

fn extended_length(total: u8, next: u8) -> Option<u8> {
    total
        .checked_add(1)?
        .checked_add(next)
        .filter(|&len| len <= 28)
}

/// Only frozen query occurrences have an adjacent source sequence. Persistent
/// relation/dependency values deliberately retain their single-word behavior.
pub(super) fn edge(values: &ValueState, origin: u8, work: &mut WordCopyWork) -> Option<(u8, u8)> {
    let words = values.lexemes.as_ref()?;
    work.selector.metadata_reads = work.selector.metadata_reads.saturating_add(1);
    let index = usize::from(origin);
    if index == 0 || index >= words.query_len {
        return None;
    }
    let from = &words.queries[index];
    let to = &words.queries[index - 1];
    work.word_record_reads = work.word_record_reads.saturating_add(2);
    if to.byte_end.checked_sub(u64::from(to.len)) != from.byte_end.checked_add(1) {
        return None;
    }
    Some((origin - 1, to.leading_gap?))
}

/// Follow exact source identity through the operator-local registry's prime address.
/// Prime integers select learned codes; their magnitude is never a semantic metric.
pub(super) fn features(
    model: &Model,
    values: &ValueState,
    origin: u8,
    next: u8,
    separator: u8,
    contextual: bool,
    paired: bool,
    work: &mut WordCopyWork,
) -> ([ValueFeature; 3], usize) {
    let mut features = [
        ValueFeature {
            kind: 0,
            a: u64::from(separator),
            b: 0,
        },
        ValueFeature::default(),
        ValueFeature::default(),
    ];
    if !contextual {
        return (features, 1);
    }
    if let Some((head, word)) = model
        .source_span_context
        .as_ref()
        .zip(super::relation::source(values, next))
    {
        work.word_record_reads = work.word_record_reads.saturating_add(1);
        let prime = super::word_copy_runtime::address_in(head, word, work);
        if prime != 0 {
            features[1] = ValueFeature {
                kind: 1,
                a: u64::from(prime),
                b: 0,
            };
            if paired {
                if let Some(original) = super::relation::source(values, origin) {
                    work.word_record_reads = work.word_record_reads.saturating_add(1);
                    let prior = original.predecessors[0];
                    let cue = super::value_lexemes::WordAtom {
                        bytes: prior.bytes,
                        len: prior.len,
                        ..Default::default()
                    };
                    let cue_prime = super::word_copy_runtime::address_in(head, &cue, work);
                    features[2] = ValueFeature {
                        kind: 2,
                        a: u64::from(prime),
                        b: u64::from(cue_prime),
                    };
                    return (features, 3);
                }
            }
            return (features, 2);
        }
    }
    (features, 1)
}

pub(super) fn extent(
    model: &Model,
    values: &ValueState,
    origin: u8,
    control: Control,
    work: &mut WordCopyWork,
) -> u8 {
    let Some(block) = model
        .source_span
        .as_ref()
        .filter(|_| control != Control::SourceSpanDisabled)
    else {
        return 0;
    };
    let Some(first) = super::relation::source(values, origin) else {
        return 0;
    };
    work.word_record_reads = work.word_record_reads.saturating_add(1);
    if origin >= 16 {
        return 0;
    }
    let mut total = first.len;
    let mut extra = 0;
    let mut current = origin;
    while let Some((next, separator)) = edge(values, current, work) {
        let (features, count) = features(
            model,
            values,
            origin,
            next,
            separator,
            model.source_span_context.is_some() && control != Control::SourceSpanContextDisabled,
            control != Control::SourceSpanPairDisabled,
            work,
        );
        let feature = features[0];
        // Unseen transitions default to the inherited finish behavior.
        work.routing.emission_queries = work.routing.emission_queries.saturating_add(1);
        if block
            .codes
            .binary_search_by(|c| {
                work.routing.comparisons = work.routing.comparisons.saturating_add(1);
                work.routing.logical_bytes_read =
                    work.routing.logical_bytes_read.saturating_add(17);
                c.feature.cmp(&feature)
            })
            .is_err()
        {
            break;
        }
        let state = block.encode(model, &features[..count], control, &mut work.routing);
        let stop = block.score(model, state, 0, &mut work.routing);
        let advance = block.score(model, state, 1, &mut work.routing);
        work.routing.comparisons = work.routing.comparisons.saturating_add(1);
        if advance <= stop {
            break;
        }
        let Some(word) = super::relation::source(values, next) else {
            break;
        };
        work.word_record_reads = work.word_record_reads.saturating_add(1);
        // Keep the existing 32-step response bound, reserving prefix/termination.
        let Some(length) = extended_length(total, word.len) else {
            work.bound_rejections = work.bound_rejections.saturating_add(1);
            break;
        };
        total = length;
        extra += 1;
        current = next;
    }
    extra
}

pub(super) fn len(
    values: &ValueState,
    origin: u8,
    extra: u8,
    work: &mut WordCopyWork,
) -> Option<u8> {
    let mut total = super::relation::source(values, origin)?.len;
    work.word_record_reads = work.word_record_reads.saturating_add(1);
    let mut current = origin;
    for _ in 0..extra {
        let (next, _) = edge(values, current, work)?;
        total = total
            .checked_add(1)?
            .checked_add(super::relation::source(values, next)?.len)?;
        work.word_record_reads = work.word_record_reads.saturating_add(1);
        current = next;
    }
    Some(total)
}

pub(super) fn byte(
    values: &ValueState,
    origin: u8,
    extra: u8,
    mut cursor: u8,
    work: &mut WordCopyWork,
) -> Option<u8> {
    let mut current = origin;
    for step in 0..=extra {
        let word = super::relation::source(values, current)?;
        work.word_record_reads = work.word_record_reads.saturating_add(1);
        if cursor < word.len {
            return Some(word.bytes[usize::from(cursor)]);
        }
        cursor -= word.len;
        if step == extra {
            return None;
        }
        let (next, separator) = edge(values, current, work)?;
        if cursor == 0 {
            return Some(separator);
        }
        cursor -= 1;
        current = next;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::super::value_lexemes::LexemeState;
    use super::super::value_types::{ValueEntry, ValueWork, QUERY};
    use super::*;

    fn values(text: &[u8]) -> ValueState {
        let mut words = LexemeState::default();
        for (sequence, &byte) in text.iter().enumerate() {
            words.feed(
                byte,
                ValueEntry {
                    sequence: sequence as u64,
                    ..Default::default()
                },
                &mut ValueWork::default(),
            );
        }
        words.finish(&mut ValueWork::default());
        words.begin();
        ValueState {
            relations: None,
            scanner: Default::default(),
            lexemes: Some(words),
            recent: [ValueEntry::default(); 32],
            recent_len: 0,
            recent_cursor: 0,
            records: Vec::new(),
            sources: Vec::new(),
            next_id: 0,
            seen: 0,
            pose: 0,
            phases: [0; PHASE_CHANNELS],
            active: false,
            consumed: false,
            started_at: 0,
            query_boundary: None,
            queries: [ValueEntry::default(); QUERY],
            query_len: 0,
            emission: None,
            pending: None,
        }
    }

    #[test]
    fn source_span_exact_bytes_count_traversal_and_refuse_wider_gaps() {
        let source = b"Rio de Janeiro";
        let values = values(source);
        let mut work = WordCopyWork::default();
        assert_eq!(len(&values, 2, 2, &mut work), Some(source.len() as u8));
        assert_eq!(work.word_record_reads, 7);
        for (cursor, &expected) in source.iter().enumerate() {
            assert_eq!(byte(&values, 2, 2, cursor as u8, &mut work), Some(expected));
        }
        assert_eq!(byte(&values, 2, 2, source.len() as u8, &mut work), None);
        for gap in [b' ', b'-', b'.', b'\n'] {
            let mut source = *b"New York";
            source[3] = gap;
            let single = self::values(&source);
            assert_eq!(edge(&single, 1, &mut work), Some((0, gap)));
            assert_eq!(byte(&single, 1, 1, 3, &mut work), Some(gap));
        }
        assert_eq!(edge(&self::values(b"New. York"), 1, &mut work), None);
        assert_eq!(edge(&values, 0, &mut work), None);
        assert_eq!(edge(&values, 16, &mut work), None);
    }

    #[test]
    fn source_span_length_reserves_four_response_steps() {
        assert_eq!(extended_length(20, 7), Some(28));
        assert_eq!(extended_length(20, 8), None);
        assert_eq!(extended_length(255, 1), None);
    }
}
