//! Learned choice among already admitted reverse starts. ASCII text shape is
//! metadata for a learned H4 score, not a grammar or a direct capitalization rule.
use super::relation_span::{RelationSpan, ReverseCandidate, ReverseCandidates};
use super::value_lexemes::LexemeState;
use super::value_types::{ValueFeature, ValueWork};
use super::*;

pub(super) const FEATURE_COUNT: usize = 7;

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
/// 0 absent; 1 lowercase; 2 initial uppercase then lowercase; 3 uppercase;
/// 4 other case; 5 contains digit; 6 contains underscore (highest precedence).
fn shape(bytes: &[u8], work: &mut RoutingWork) -> u64 {
    if bytes.is_empty() {
        return 0;
    }
    let (mut lower, mut upper, mut title) = (true, true, true);
    let (mut digit, mut underscore) = (false, false);
    for (index, &byte) in bytes.iter().enumerate() {
        work.logical_bytes_read = work.logical_bytes_read.saturating_add(1);
        lower &= byte.is_ascii_lowercase();
        upper &= byte.is_ascii_uppercase();
        title &= if index == 0 {
            byte.is_ascii_uppercase()
        } else {
            byte.is_ascii_lowercase()
        };
        digit |= byte.is_ascii_digit();
        underscore |= byte == b'_';
    }
    if underscore {
        6
    } else if digit {
        5
    } else if lower {
        1
    } else if title {
        2
    } else if upper {
        3
    } else {
        4
    }
}

/// Ordered first/next/predecessor shapes, preceding gap class, two ordered
/// shape pairs and singleton flag. The next word is inside the admitted span.
pub(super) fn features(
    words: &LexemeState,
    endpoint: usize,
    candidate: ReverseCandidate,
    work: &mut ValueWork,
) -> [ValueFeature; FEATURE_COUNT] {
    let first = &words.recent[candidate.start];
    work.relations.record_reads = work.relations.record_reads.saturating_add(1);
    let first_shape = shape(
        &first.bytes[..usize::from(first.len)],
        &mut work.relations.span_routing,
    );
    let next_shape = if candidate.start > endpoint {
        let next = &words.recent[candidate.start - 1];
        work.relations.record_reads = work.relations.record_reads.saturating_add(1);
        shape(
            &next.bytes[..usize::from(next.len)],
            &mut work.relations.span_routing,
        )
    } else {
        0
    };
    let prior = &first.predecessors[0];
    let prior_shape = shape(
        &prior.bytes[..usize::from(prior.len)],
        &mut work.relations.span_routing,
    );
    let gap = match first.leading_gap {
        None => 0,
        Some(b' ') => 1,
        Some(b'\n' | b'\r') => 2,
        Some(_) => 3,
    };
    work.relations.feature_writes = work
        .relations
        .feature_writes
        .saturating_add(FEATURE_COUNT as u64);
    [
        ValueFeature {
            kind: 0,
            a: first_shape,
            b: 0,
        },
        ValueFeature {
            kind: 1,
            a: next_shape,
            b: 0,
        },
        ValueFeature {
            kind: 2,
            a: prior_shape,
            b: 0,
        },
        ValueFeature {
            kind: 3,
            a: gap,
            b: 0,
        },
        ValueFeature {
            kind: 4,
            a: first_shape,
            b: next_shape,
        },
        ValueFeature {
            kind: 5,
            a: prior_shape,
            b: first_shape,
        },
        ValueFeature {
            kind: 6,
            a: u64::from(candidate.start == endpoint),
            b: 0,
        },
    ]
}

pub(super) fn select(
    model: &Model,
    words: &LexemeState,
    endpoint: usize,
    candidates: &ReverseCandidates,
    work: &mut ValueWork,
) -> Option<RelationSpan> {
    let Some(block) = &model.relation_start else {
        return candidates.rows[candidates.len - 1].span;
    };
    let mut best = None;
    let mut score = i64::MIN;
    work.relations.span_routing.predictions =
        work.relations.span_routing.predictions.saturating_add(1);
    // Longer candidates first preserves the parent decision on an exact tie.
    for &candidate in candidates.rows[..candidates.len].iter().rev() {
        let features = features(words, endpoint, candidate, work);
        let state = block.encode(
            model,
            &features,
            Control::Full,
            &mut work.relations.span_routing,
        );
        let current = block.score(model, state, 0, &mut work.relations.span_routing);
        work.relations.span_routing.sources_examined = work
            .relations
            .span_routing
            .sources_examined
            .saturating_add(1);
        work.relations.span_routing.comparisons =
            work.relations.span_routing.comparisons.saturating_add(1);
        if current > score {
            score = current;
            best = candidate.span;
        }
    }
    best
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

#[cfg(test)]
mod tests {
    use super::super::value_types::ValueEntry;
    use super::*;

    fn words(text: &str) -> LexemeState {
        let mut words = LexemeState::default();
        for (sequence, &byte) in text.as_bytes().iter().enumerate() {
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
        words
    }

    #[test]
    fn relation_start_shape_classes_are_lexical_identity_independent() {
        for (bytes, expected) in [
            (b"".as_slice(), 0),
            (b"say", 1),
            (b"Amber", 2),
            (b"NASA", 3),
            (b"aB", 4),
            (b"Amber2", 5),
            (b"_Amber2", 6),
        ] {
            let mut work = RoutingWork::default();
            assert_eq!(shape(bytes, &mut work), expected);
            assert_eq!(work.logical_bytes_read, bytes.len() as u64);
        }
        assert_eq!(
            shape(b"Amber", &mut RoutingWork::default()),
            shape(b"Dawn", &mut RoutingWork::default())
        );
    }

    #[test]
    fn relation_start_features_respect_candidate_endpoint_and_context() {
        let mut words = words("Notes say Amber Meadow holds telra.");
        let candidate = ReverseCandidate {
            start: 3,
            span: None,
        };
        let base = features(&words, 2, candidate, &mut ValueWork::default());
        assert_eq!((base[0].a, base[1].a, base[2].a), (2, 2, 1));
        assert_eq!(base[6].a, 0);
        let singleton = features(
            &words,
            2,
            ReverseCandidate {
                start: 2,
                span: None,
            },
            &mut ValueWork::default(),
        );
        assert_eq!((singleton[1].a, singleton[6].a), (0, 1));
        let intro = features(
            &words,
            2,
            ReverseCandidate {
                start: 5,
                span: None,
            },
            &mut ValueWork::default(),
        );
        assert_ne!(intro, base);
        // Linker and owner lie after the endpoint and cannot alter the features.
        words.recent[0].bytes[0] = b'T';
        words.recent[1].bytes[0] = b'H';
        assert_eq!(
            features(&words, 2, candidate, &mut ValueWork::default()),
            base
        );
    }
}
