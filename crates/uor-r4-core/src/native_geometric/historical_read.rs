//! Learned Base/previous-record selection; source payloads remain exact.
use super::relation::{RelationRecord, RelationState, RELATION_SOURCE};
use super::source_routing::SourceRouting;
use super::value_types::{ValueFeature, ValueState, ValueWork};
use super::word_copy_types::{WordCopyAddress, WordCopyWork};
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HistoricalRead {
    pub router: SourceRouting,
    pub dictionary: Vec<WordCopyAddress>,
    #[serde(
        default = "legacy_query_scope",
        skip_serializing_if = "is_legacy_query_scope"
    )]
    pub query_scope: u8,
}
fn legacy_query_scope() -> u8 {
    1
}
fn is_legacy_query_scope(scope: &u8) -> bool {
    *scope == 1
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
/// Select the artifact-bound context intervention without altering its inner reader.
pub(super) fn head(model: &Model, control: Control) -> Option<&HistoricalRead> {
    if control != Control::HistoricalQueryContextDisabled {
        if let Some(outer) = &model.historical_query_context {
            return Some(&outer.active);
        }
    }
    model.historical_read.as_ref()
}

pub(super) fn effective_window(model: &Model, control: Control) -> usize {
    query_window(model.historical_query_context.is_some(), control)
}

fn query_window(active: bool, control: Control) -> usize {
    if active
        && !matches!(
            control,
            Control::HistoricalQueryContextDisabled | Control::HistoricalQueryWindowDisabled
        )
    {
        16
    } else {
        8
    }
}

/// A selector-local view; source windows, exact identities and record state stay intact.
pub(super) fn query_view(
    words: &super::value_lexemes::LexemeState,
    boundary: Option<u64>,
    local: bool,
    work: &mut WordCopyWork,
) -> super::value_lexemes::LexemeState {
    let mut view = *words;
    if local {
        work.selector.state_copies += 1;
        if let Some(start) = boundary {
            view.query_len = 0;
            for word in &words.queries[..words.query_len] {
                work.word_record_reads += 1;
                work.routing.comparisons += 1;
                if word.end < start {
                    break;
                }
                view.query_len += 1;
            }
        }
    }
    view
}

pub(super) fn local_query_scope(model: &Model, control: Control) -> bool {
    control != Control::HistoricalReadDisabled
        && model
            .historical_read
            .as_ref()
            .is_some_and(|b| b.query_scope == 2)
}

pub(super) fn addresses(
    block: &HistoricalRead,
    values: &ValueState,
    work: &mut WordCopyWork,
) -> [u32; 16] {
    let mut out = [0; 16];
    let Some(words) = &values.lexemes else {
        return out;
    };
    let words = query_view(words, values.query_boundary, block.query_scope == 2, work);
    for (i, word) in words.queries[..words.query_len].iter().enumerate() {
        work.dictionary_lookups += 1;
        work.word_record_reads += 1;
        let found = block.dictionary.binary_search_by(|d| {
            work.dictionary_comparisons += 1;
            for j in 0..usize::from(word.len.min(d.len)) {
                work.dictionary_byte_comparisons += 1;
                let order = d.bytes[j].cmp(&word.bytes[j]);
                if !order.is_eq() {
                    return order;
                }
            }
            d.len.cmp(&word.len)
        });
        out[i] = found.map_or(0, |j| block.dictionary[j].prime);
        work.selector.state_copies += 1;
    }
    out
}

pub(super) fn features(
    model: &Model,
    values: &ValueState,
    current: &RelationRecord,
    addr: &[u32; 16],
    local: bool,
    work: &mut WordCopyWork,
) -> ([ValueFeature; 96], usize) {
    features_with_window(model, values, current, addr, local, 8, work)
}

/// Only ordered historical context widens; owner matching remains bounded at eight.
pub(super) fn features_with_window(
    model: &Model,
    values: &ValueState,
    current: &RelationRecord,
    addr: &[u32; 16],
    local: bool,
    window: usize,
    work: &mut WordCopyWork,
) -> ([ValueFeature; 96], usize) {
    let mut out = [ValueFeature::default(); 96];
    let Some(words) = &values.lexemes else {
        return (out, 0);
    };
    let words = query_view(words, values.query_boundary, local, work);
    let (base, mut n) =
        super::relation::read_features(model, current, &words, addr, &mut work.persistent_read);
    out[..n].copy_from_slice(&base[..n]);
    let inherited_n = n;
    n = append_ordered_context(&mut out, n, addr, words.query_len, window);
    work.persistent_read.relations.feature_writes += (n - inherited_n) as u64;
    (out, n)
}

/// Append bounded ordered metadata after the inherited owner-match features.
pub(super) fn append_ordered_context(
    out: &mut [ValueFeature; 96],
    mut n: usize,
    addr: &[u32; 16],
    query_len: usize,
    window: usize,
) -> usize {
    // The caller supplies at most 34 inherited features: two globals and
    // four features for each of eight owner matches. This adds at most 31.
    let limit = query_len.min(window.min(16));
    for q in 0..limit {
        out[n] = ValueFeature {
            kind: 6,
            a: u64::from(addr[q]),
            b: 0,
        };
        n += 1;
        if q + 1 < limit {
            out[n] = ValueFeature {
                kind: 7,
                a: u64::from(addr[q + 1]),
                b: u64::from(addr[q]),
            };
            n += 1;
        }
    }
    n
}

/// Structural admission is independent of question spelling and learned scores.
pub(super) fn previous<'a>(
    state: &'a RelationState,
    current: &RelationRecord,
    work: &mut ValueWork,
) -> Option<&'a RelationRecord> {
    let is_current = state.directory.iter().any(|&id| {
        work.relations.directory_reads += 1;
        id != 0 && id == current.id
    });
    if !is_current
        || current.id == 0
        || current.conflict
        || current.action != 2
        || current.previous == 0
        || current.previous >= current.id
    {
        return None;
    }
    work.relations.record_reads += 1;
    let previous = state.record(current.previous)?;
    if previous.conflict || !current.owner.matches(&previous.owner, work) {
        return None;
    }
    Some(previous)
}

/// Bounded exact ancestor admission; the ring retains at most sixteen records.
pub(super) const ANCESTOR_DEPTH: u8 = (super::relation::RELATIONS - 1) as u8;

/// One immutable descending link below a record that is not itself a live head.
pub(super) fn link<'a>(
    state: &'a RelationState,
    record: &RelationRecord,
    work: &mut ValueWork,
) -> Option<&'a RelationRecord> {
    if record.id == 0
        || record.conflict
        || record.action != 2
        || record.previous == 0
        || record.previous >= record.id
    {
        return None;
    }
    work.relations.record_reads += 1;
    let older = state.record(record.previous)?;
    if older.conflict || !record.owner.matches(&older.owner, work) {
        return None;
    }
    Some(older)
}

/// Exact record `depth` validated links below a live head; depth one is `previous`.
pub(super) fn ancestor<'a>(
    state: &'a RelationState,
    current: &RelationRecord,
    depth: u8,
    work: &mut ValueWork,
) -> Option<&'a RelationRecord> {
    if depth == 0 || depth > ANCESTOR_DEPTH {
        return None;
    }
    let mut record = previous(state, current, work)?;
    let mut remaining = depth - 1;
    while remaining > 0 {
        record = link(state, record, work)?;
        remaining -= 1;
    }
    Some(record)
}

/// A genuine assertion root: the chain's first stored version, not merely its oldest survivor.
pub(super) fn is_root(record: &RelationRecord) -> bool {
    record.previous == 0 && record.action == 1
}

pub(super) fn action_indices(model: &Model) -> Option<(usize, usize)> {
    let read = super::role_read::head(model)?;
    let mut defer = None;
    let mut copy = None;
    for (i, a) in read.actions.iter().enumerate() {
        if !a.copy {
            if defer.replace(i).is_some() {
                return None;
            }
        } else if a.prefix.is_some() && copy.replace(i).is_some() {
            return None;
        }
    }
    Some((defer?, copy?))
}

pub(super) fn representable(
    model: &Model,
    values: &ValueState,
    previous: &RelationRecord,
    action: usize,
    work: &mut WordCopyWork,
) -> bool {
    let source = RELATION_SOURCE + ((previous.id - 1) & 15) as u8;
    let Some(read) = super::role_read::head(model) else {
        return false;
    };
    let Some(a) = read.actions.get(action) else {
        return false;
    };
    a.copy
        && super::source_span::len(values, source, 0, work).is_some_and(|len| {
            usize::from(len) + 1 + usize::from(a.prefix.is_some())
                <= usize::from(super::response_entry_types::RESPONSE_ENTRY_STEPS)
        })
}

/// Base returns None so the complete frozen dispatch remains available.
pub(super) fn choose(
    model: &Model,
    values: &ValueState,
    control: Control,
    work: &mut WordCopyWork,
) -> Option<(u8, usize)> {
    if matches!(
        control,
        Control::HistoricalReadDisabled
            | Control::LearnedRoutingDisabled
            | Control::LearnedRoutingSelectionDisabled
            | Control::H4Disabled
    ) {
        return None;
    }
    let block = head(model, control)?;
    let state = values.relations.as_ref()?;
    let (defer, copy) = action_indices(model)?;
    let addr = addresses(block, values, work);
    let mut best_score = block.router.score(
        model,
        [model.geometry.identity; 2],
        defer,
        &mut work.routing,
    );
    let mut best = None;
    work.routing.predictions += 1;
    for &id in &state.directory {
        work.persistent_read.relations.directory_reads += 1;
        work.persistent_read.relations.record_reads += 1;
        let Some(current) = state.record(id) else {
            continue;
        };
        let Some(old) = previous(state, current, &mut work.persistent_read) else {
            continue;
        };
        if !representable(model, values, old, copy, work) {
            continue;
        }
        let (f, n) = features_with_window(
            model,
            values,
            current,
            &addr,
            block.query_scope == 2,
            effective_window(model, control),
            work,
        );
        let roots = block
            .router
            .encode(model, &f[..n], control, &mut work.routing);
        work.routing.sources_examined += 1;
        let score = block.router.score(model, roots, copy, &mut work.routing);
        work.routing.comparisons += 1;
        if score > best_score {
            best_score = score;
            best = Some((RELATION_SOURCE + ((old.id - 1) & 15) as u8, copy));
        }
    }
    best
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

#[cfg(test)]
mod tests {
    use super::*;
    fn atom(text: &str, end: u64) -> super::super::value_lexemes::WordAtom {
        let mut a = super::super::value_lexemes::WordAtom {
            len: text.len() as u8,
            end,
            byte_end: end,
            ..Default::default()
        };
        a.bytes[..text.len()].copy_from_slice(text.as_bytes());
        a
    }
    #[test]
    fn historical_read_query_scope_clips_old_cues_without_mutating_sources() {
        let mut words = super::super::value_lexemes::LexemeState::default();
        words.queries[0] = atom("Answer", 24);
        words.queries[1] = atom("owner", 20);
        words.queries[2] = atom("Where", 19);
        words.queries[3] = atom("before", 10);
        words.query_len = 4;
        words.recent = words.queries;
        words.recent_len = 4;
        let original = words;
        let local = query_view(&words, Some(19), true, &mut Default::default());
        assert_eq!(
            local.query_len, 3,
            "inclusive boundary retains its exact word"
        );
        assert_eq!(local.queries[..local.query_len], words.queries[..3]);
        assert_eq!(local.recent, words.recent, "source window remains exact");
        assert_eq!(
            local.queries, words.queries,
            "only the selector range is clipped"
        );
        assert_eq!(words, original, "caller state is never mutated");
        assert_eq!(
            query_view(&words, Some(19), false, &mut Default::default()),
            words,
            "legacy law stays exact"
        );
        assert_eq!(
            query_view(&words, None, true, &mut Default::default()),
            words,
            "missing legacy boundary stays exact"
        );
        assert_eq!(
            query_view(&words, Some(0), true, &mut Default::default()),
            words,
            "first turn stays exact"
        );
        assert_eq!(
            query_view(&words, Some(25), true, &mut Default::default()).query_len,
            0
        );
    }

    fn chain() -> RelationState {
        let mut s = RelationState::default();
        for i in 0..3 {
            s.records[i] = RelationRecord {
                id: (i + 1) as u64,
                owner: atom("owner", 10 + i as u64 * 10),
                value: atom("same", 15 + i as u64 * 10),
                previous: i as u64,
                action: if i == 0 { 1 } else { 2 },
                ..Default::default()
            };
        }
        s.directory[0] = 3;
        s.next_id = 4;
        s
    }
    #[test]
    fn historical_read_immediate_previous_retains_occurrence_not_oldest_or_spelling() {
        let s = chain();
        let old = previous(&s, &s.records[2], &mut Default::default()).unwrap();
        assert_eq!(old.id, 2);
        assert_eq!(old.value.byte_end, 25);
        assert_ne!(old.value, s.records[0].value);
        assert_ne!(old.value, s.records[2].value);
        assert_eq!(old.previous, 1);
    }
    #[test]
    fn historical_read_previous_rejects_broken_forward_self_and_noncurrent_links() {
        for bad in [0, 3, 4, 99] {
            let mut s = chain();
            s.records[2].previous = bad;
            assert!(previous(&s, &s.records[2], &mut Default::default()).is_none());
        }
        let mut s = chain();
        s.directory[0] = 2;
        assert!(previous(&s, &s.records[2], &mut Default::default()).is_none());
        s.directory[0] = 3;
        s.records[1].id = 18;
        assert!(previous(&s, &s.records[2], &mut Default::default()).is_none());
    }
    #[test]
    fn historical_read_ancestor_walks_exact_validated_links_to_the_assertion_root() {
        let s = chain();
        let head = &s.records[2];
        let mut work = ValueWork::default();
        assert_eq!(ancestor(&s, head, 1, &mut work).unwrap().id, 2);
        let root = ancestor(&s, head, 2, &mut work).unwrap();
        assert_eq!(root.id, 1);
        assert!(is_root(root));
        assert!(!is_root(&s.records[1]));
        assert_eq!(
            root.value.byte_end, 15,
            "exact first occurrence, not a spelling"
        );
        assert!(ancestor(&s, head, 0, &mut work).is_none());
        assert!(ancestor(&s, head, 3, &mut work).is_none());
        assert!(ancestor(&s, head, ANCESTOR_DEPTH + 1, &mut work).is_none());
        assert!(
            link(&s, &s.records[0], &mut work).is_none(),
            "roots have no link"
        );
        assert!(work.relations.record_reads > 0);
    }
    #[test]
    fn historical_read_ancestor_rejects_broken_overwritten_and_cross_owner_paths() {
        let mut s = chain();
        s.records[1].previous = 0;
        assert!(ancestor(&s, &s.records[2], 2, &mut Default::default()).is_none());
        assert_eq!(
            ancestor(&s, &s.records[2], 1, &mut Default::default())
                .unwrap()
                .id,
            2,
            "a broken deeper link keeps the immediate previous read"
        );
        let mut s = chain();
        s.records[0].id = 17;
        assert!(ancestor(&s, &s.records[2], 2, &mut Default::default()).is_none());
        let mut s = chain();
        s.records[0].owner = atom("other", 10);
        assert!(ancestor(&s, &s.records[2], 2, &mut Default::default()).is_none());
        let mut s = chain();
        s.records[1].action = 1;
        assert!(ancestor(&s, &s.records[2], 2, &mut Default::default()).is_none());
        let mut s = chain();
        s.records[0].conflict = true;
        assert!(ancestor(&s, &s.records[2], 2, &mut Default::default()).is_none());
        s.records[0].conflict = false;
        s.directory[0] = 2;
        assert!(ancestor(&s, &s.records[2], 1, &mut Default::default()).is_none());
    }
    #[test]
    fn historical_read_previous_rejects_conflict_assertion_and_cross_owner() {
        let mut s = chain();
        s.records[2].conflict = true;
        assert!(previous(&s, &s.records[2], &mut Default::default()).is_none());
        s.records[2].conflict = false;
        s.records[1].conflict = true;
        assert!(previous(&s, &s.records[2], &mut Default::default()).is_none());
        s.records[1].conflict = false;
        s.records[2].action = 1;
        assert!(previous(&s, &s.records[2], &mut Default::default()).is_none());
        s.records[2].action = 2;
        s.records[1].owner = atom("other", 20);
        assert!(previous(&s, &s.records[2], &mut Default::default()).is_none());
    }
}

#[cfg(test)]
mod query_context_tests {
    use super::*;
    #[test]
    fn historical_query_window_control_changes_only_exposure_limit() {
        assert_eq!(query_window(true, Control::Full), 16);
        assert_eq!(
            query_window(true, Control::HistoricalQueryWindowDisabled),
            8
        );
        assert_eq!(
            query_window(true, Control::HistoricalQueryContextDisabled),
            8
        );
        assert_eq!(query_window(false, Control::Full), 8);
        assert_eq!(
            query_window(false, Control::HistoricalQueryWindowDisabled),
            8
        );
    }
    #[test]
    fn historical_query_context_window_preserves_prefix_and_bounded_order() {
        let addr = std::array::from_fn(|i| (i + 1) as u32);
        let mut old = [ValueFeature::default(); 96];
        let mut extended = old;
        let old_n = append_ordered_context(&mut old, 34, &addr, 16, 8);
        let new_n = append_ordered_context(&mut extended, 34, &addr, 16, 16);
        assert_eq!(old_n, 49);
        assert_eq!(new_n, 65);
        assert_eq!(&old[..old_n], &extended[..old_n]);
        assert_eq!(
            extended[old_n],
            ValueFeature {
                kind: 7,
                a: 9,
                b: 8
            }
        );
        assert_eq!(
            extended[new_n - 1],
            ValueFeature {
                kind: 6,
                a: 16,
                b: 0
            }
        );
        assert!(extended[new_n..]
            .iter()
            .all(|f| *f == ValueFeature::default()));
        let mut clamped = [ValueFeature::default(); 96];
        assert_eq!(
            append_ordered_context(&mut clamped, 34, &addr, 100, 100),
            65
        );
        assert_eq!(clamped, extended);
    }
    #[test]
    fn historical_query_context_short_query_has_identical_features() {
        let addr = std::array::from_fn(|i| i as u32);
        for len in 0..=8 {
            let mut old = [ValueFeature::default(); 96];
            let mut extended = old;
            assert_eq!(
                append_ordered_context(&mut old, 0, &addr, len, 8),
                append_ordered_context(&mut extended, 0, &addr, len, 16)
            );
            assert_eq!(old, extended);
        }
    }
}
