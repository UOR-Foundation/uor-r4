//! One learned relation/operator choice followed by one exact dependent lookup.
//! Geometry carries selection metadata; both payloads remain versioned words.
use super::relation::{RelationRecord, RelationState, RELATION_SOURCE};
use super::source_routing::SourceRouting;
use super::value_types::{ValueFeature, ValueState, ValueWork};
use super::word_copy_types::{WordCopyAddress, WordCopyWork};
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DependentRead {
    pub router: SourceRouting,
    pub dictionary: Vec<WordCopyAddress>,
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
pub(super) fn addresses(
    block: &DependentRead,
    values: &ValueState,
    work: &mut WordCopyWork,
) -> [u32; 16] {
    addresses_scoped(block, values, false, work)
}

fn addresses_scoped(
    block: &DependentRead,
    values: &ValueState,
    local: bool,
    work: &mut WordCopyWork,
) -> [u32; 16] {
    let mut out = [0; 16];
    let Some(words) = &values.lexemes else {
        return out;
    };
    let words = super::historical_read::query_view(words, values.query_boundary, local, work);
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
    record: &RelationRecord,
    addr: &[u32; 16],
    work: &mut WordCopyWork,
) -> ([ValueFeature; 96], usize) {
    features_scoped(model, values, record, addr, false, work)
}

fn features_scoped(
    model: &Model,
    values: &ValueState,
    record: &RelationRecord,
    addr: &[u32; 16],
    local: bool,
    work: &mut WordCopyWork,
) -> ([ValueFeature; 96], usize) {
    let mut out = [ValueFeature::default(); 96];
    let Some(words) = &values.lexemes else {
        return (out, 0);
    };
    let words = super::historical_read::query_view(words, values.query_boundary, local, work);
    let (base, mut n) =
        super::relation::read_features(model, record, &words, addr, &mut work.persistent_read);
    out[..n].copy_from_slice(&base[..n]);
    // Ordered query words remain exact dictionary keys, never hash distances.
    // These are learned context features, not an interpreted grammar or cue list.
    for &prime in &addr[..words.query_len] {
        if prime != 0 {
            out[n] = ValueFeature {
                kind: 6,
                a: u64::from(prime),
                b: 0,
            };
            n += 1;
            work.persistent_read.relations.feature_writes += 1;
        }
    }
    (out, n)
}

fn current(state: &RelationState, id: u64, work: &mut ValueWork) -> bool {
    state.directory.iter().any(|&candidate| {
        work.relations.directory_reads += 1;
        candidate == id && id != 0
    })
}

pub(super) fn follow<'a>(
    state: &'a RelationState,
    first: &RelationRecord,
    work: &mut ValueWork,
) -> Option<&'a RelationRecord> {
    if first.conflict || first.span.is_some() {
        return None;
    }
    for &id in &state.directory {
        work.relations.directory_reads += 1;
        work.relations.record_reads += 1;
        if let Some(record) = state.record(id) {
            if first.value.matches(&record.owner, work) {
                return (!record.conflict).then_some(record);
            }
        }
    }
    None
}

pub(super) fn valid(values: &ValueState, ids: [u64; 2], work: &mut ValueWork) -> bool {
    let Some(state) = &values.relations else {
        return false;
    };
    if !current(state, ids[0], work) || !current(state, ids[1], work) {
        return false;
    }
    work.relations.record_reads += 2;
    match (state.record(ids[0]), state.record(ids[1])) {
        (Some(first), Some(last)) => {
            !first.conflict
                && first.span.is_none()
                && !last.conflict
                && first.value.matches(&last.owner, work)
        }
        _ => false,
    }
}

/// None defers to the entire frozen parent. A selected missing/conflicted link
/// abstains through its existing NoRead action instead of changing the query.
pub(super) fn choose(
    model: &Model,
    values: &ValueState,
    control: Control,
    work: &mut WordCopyWork,
) -> Option<(u8, usize, Option<[u64; 2]>)> {
    if matches!(
        control,
        Control::LearnedRoutingSelectionDisabled | Control::LearnedRoutingDisabled
    ) {
        return None;
    }
    let block = model.dependent_read.as_ref()?;
    let read = super::role_read::head(model)?;
    let state = values.relations.as_ref()?;
    let defer = read.actions.iter().position(|a| !a.copy)?;
    let local = super::historical_read::local_query_scope(model, control);
    let addr = addresses_scoped(block, values, local, work);
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
        let Some(record) = state.record(id) else {
            continue;
        };
        let (f, n) = features_scoped(model, values, record, &addr, local, work);
        let roots = block
            .router
            .encode(model, &f[..n], control, &mut work.routing);
        work.routing.sources_examined += 1;
        for (action, a) in read.actions.iter().enumerate() {
            if !a.copy {
                continue;
            }
            let score = block.router.score(model, roots, action, &mut work.routing);
            work.routing.comparisons += 1;
            if score > best_score {
                best_score = score;
                best = Some((record, action));
            }
        }
    }
    let (first, action) = best?;
    let last = if control == Control::LearnedRoutingChainDisabled {
        None
    } else {
        follow(state, first, &mut work.persistent_read)
    };
    if let Some(last) = last {
        let source = RELATION_SOURCE + ((last.id - 1) & 15) as u8;
        if super::source_routing::allowed(model, values, source, action) {
            work.persistent_read.relations.reads += 1;
            return Some((source, action, Some([first.id, last.id])));
        }
    }
    work.persistent_read.relations.abstentions += 1;
    Some((super::role_read::NO_SOURCE, defer, Some([first.id, 0])))
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END
