//! Learned geometric metadata selection; exact numeric payloads remain separate.
use super::source_routing::SourceRouting;
use super::value_types::{ValueFeature, ValueRecord, ValueState, ValueWork, VALUES};
use super::word_copy_types::WordCopyAddress;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TypedRouting {
    pub router: SourceRouting,
    pub dictionary: Vec<WordCopyAddress>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub fold_ascii_case: bool,
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
pub(super) fn context(
    model: &Model,
    values: &ValueState,
    control: Control,
    work: &mut ValueWork,
) -> Option<[u32; 16]> {
    let block = model.typed_routing.as_ref()?;
    if matches!(
        control,
        Control::LearnedRoutingDisabled | Control::GeometryDisabled | Control::H4Disabled
    ) {
        return None;
    }
    let mut derived = false;
    for source in &values.sources {
        work.routing.sources_examined += 1;
        derived |= source.derived;
    }
    // Structural scope: initial literal-only decisions retain their parent.
    if !derived {
        return None;
    }
    Some(addresses(block, values, work))
}

pub(super) fn addresses(
    block: &TypedRouting,
    values: &ValueState,
    work: &mut ValueWork,
) -> [u32; 16] {
    let mut out = [0; 16];
    let Some(words) = &values.lexemes else {
        return out;
    };
    for (i, word) in words.queries[..words.query_len].iter().enumerate() {
        work.routing.context_tokens_read += 1;
        let found = block.dictionary.binary_search_by(|d| {
            work.routing.comparisons += 1;
            for j in 0..usize::from(word.len.min(d.len)) {
                work.lexical_byte_comparisons += 1;
                work.routing.logical_bytes_read += 2;
                let byte = if block.fold_ascii_case {
                    word.bytes[j].to_ascii_lowercase()
                } else {
                    word.bytes[j]
                };
                let cmp = d.bytes[j].cmp(&byte);
                if !cmp.is_eq() {
                    return cmp;
                }
            }
            d.len.cmp(&word.len)
        });
        out[i] = found.map_or(0, |i| block.dictionary[i].prime);
    }
    out
}

pub(super) fn features(
    values: &ValueState,
    operands: Option<(ValueRecord, ValueRecord)>,
    addr: &[u32; 16],
    work: &mut ValueWork,
) -> ([ValueFeature; 36], usize) {
    let mut out = [ValueFeature::default(); 36];
    let (flags, rank_a, rank_b) = if let Some((a, b)) = operands {
        let mut ra = VALUES;
        let mut rb = VALUES;
        for (rank, r) in values.sources.iter().rev().enumerate() {
            work.routing.sources_examined += 1;
            work.routing.comparisons += 2;
            if r.id == a.id {
                ra = rank;
            }
            if r.id == b.id {
                rb = rank;
            }
        }
        (u64::from(a.derived) | (u64::from(b.derived) << 1), ra, rb)
    } else {
        (0, VALUES, VALUES)
    };
    out[0] = ValueFeature {
        kind: 0,
        a: flags,
        b: 0,
    };
    out[1] = ValueFeature {
        kind: 1,
        a: rank_a as u64,
        b: rank_b as u64,
    };
    let mut n = 2;
    // Ordered exact word identities, not distances computed from hash bits.
    for (position, &prime) in addr.iter().enumerate() {
        if prime == 0 {
            continue;
        }
        out[n] = ValueFeature {
            kind: 2,
            a: u64::from(prime),
            b: 0,
        };
        n += 1;
        out[n] = ValueFeature {
            kind: 3,
            a: position as u64,
            b: u64::from(prime),
        };
        n += 1;
    }
    (out, n)
}

pub(super) fn score(
    model: &Model,
    values: &ValueState,
    operands: Option<(ValueRecord, ValueRecord)>,
    action: usize,
    addr: &[u32; 16],
    control: Control,
    work: &mut ValueWork,
) -> i64 {
    let Some(block) = &model.typed_routing else {
        return 0;
    };
    let (f, n) = features(values, operands, addr, work);
    let state = block
        .router
        .encode(model, &f[..n], control, &mut work.routing);
    block.router.score(model, state, action, &mut work.routing)
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END
