//! Learned initial-versus-previous version intent over bounded exact ancestor admission.
//! The complete parent, its frozen historical reader and every inner parameter stay intact.
use super::current_query_handoff::scoped_words;
use super::historical_read::{self, ANCESTOR_DEPTH};
use super::relation::{RelationRecord, RelationState, RELATION_SOURCE};
use super::source_routing::{SourceCode, SourceRouting};
use super::source_routing_training::{learn_code_subset, Alternative, Frame};
use super::value_lexemes::LexemeState;
use super::value_types::{ValueFeature, ValueState};
use super::word_copy_types::{WordCopyAddress, WordCopyWork};
use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoricalVersionExample {
    pub id: String,
    pub prompt: String,
    /// Offline exact ancestor record identity; None with `inherit` labels complete deferral.
    pub target_record: Option<u64>,
    /// Preserve the entire parent dispatch; `target_record` must be None.
    pub inherit: bool,
    /// Offline label: the named chain is truncated and the answer is an abstention.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub abstain: bool,
    /// Prior turns replayed before `prompt`, each answered by the model itself with the
    /// complete frozen dispatch, so follow-up request views (which carry the prior
    /// response's words) are part of the construction. Empty for a single turn.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HistoricalVersionIntent {
    pub parent_artifact: String,
    pub router: SourceRouting,
    pub config: SourceRoutingConfig,
    pub training: Vec<DocumentReceipt>,
    /// Construction-prompt vocabulary in its own namespace; unknown words stay zero.
    pub dictionary: Vec<WordCopyAddress>,
    /// Limit selector context to request words after the latest committed fact.
    pub committed_query_scope: bool,
    /// Offer a learned abstention when a named chain's validated links never
    /// reach a genuine root. Absent in legacy artifacts, which never abstain.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub abstention: bool,
    /// Versioned chain contract: validated links also follow resident same-value
    /// reassertions, and a chain counts as truncated only when the next predecessor
    /// is proven evicted from the ring. Absent in legacy artifacts, whose links are
    /// explicit revisions only and whose truncation is any unreached root.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reassertion_links: bool,
    /// Head contract: the first hop below a live head that is itself a resident
    /// same-value reassertion is valid, so repeating the current fact keeps the retained
    /// history readable. Record-hop semantics are unchanged: previous is the head's
    /// immediate previous record and initial is the assertion root. Candidates reached
    /// through that hop carry their own chain classes. Absent in legacy artifacts.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reassertion_heads: bool,
    /// Reader-request-window contract: the frozen persistent relation reader scans
    /// the request words after the latest committed fact, up to the sixteen-word
    /// view, for the named owner instead of the eight most recent words of any kind,
    /// so a current request whose instructions follow the question still reaches its
    /// committed record while an owner spelled inside an earlier record is never
    /// taken for the requested owner. The inherited eight-word feature base of every
    /// later learner is unchanged. Absent in legacy artifacts.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reader_request_window: bool,
    /// Reader-turn-window contract: the request-window scan also stops at the current
    /// turn's input boundary, so a previous question or generated answer never names
    /// the owner requested now; stored records stay eligible for lookup. Builds on the
    /// request-window contract. Absent in legacy artifacts and in `f3620cb7`, whose
    /// post-fact-only scan is preserved as measured.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reader_turn_window: bool,
}

/// Structural candidate classes exposed to the learned selector. They describe a
/// validated chain position; they are not a semantic distance or an answer rule.
const CLASS_PREVIOUS: u64 = 1;
const CLASS_PREVIOUS_ROOT: u64 = 2;
const CLASS_ANCESTOR: u64 = 3;
const CLASS_ANCESTOR_ROOT: u64 = 4;
/// A named chain whose validated links end before any genuine root.
const CLASS_TRUNCATED: u64 = 5;
/// The same four chain positions reached through a same-value reassertion head; the
/// frozen parent cannot answer these, so the selector must learn them separately.
const CLASS_HEAD_OFFSET: u64 = 5;
#[cfg(test)]
const CLASS_HEAD_ANCESTOR_ROOT: u64 = CLASS_ANCESTOR_ROOT + CLASS_HEAD_OFFSET;
/// A same-value reassertion head whose predecessor is proven evicted: truncated below
/// a head the frozen parent never reads, so it is learned apart from a revised head's
/// truncation, whose deferral keeps the parent's answer.
const CLASS_HEAD_TRUNCATED: u64 = CLASS_TRUNCATED + CLASS_HEAD_OFFSET;
const KIND_WORD_CLASS: u8 = 8;
const KIND_DEPTH_CLASS: u8 = 9;
/// How many request words the bounded committed view actually retained.
const KIND_VIEW_CLASS: u8 = 10;
/// A request word receives a lexical identity only when it occurs in at least this
/// many distinct construction request views; singletons stay at prime zero, like fresh words.
const DICTIONARY_MIN_OCCURRENCES: u64 = 2;
/// Query words matching the candidate chain's owner enter the geometry as this
/// role sentinel, never as their spelling; primes begin at two.
const OWNER_ROLE: u32 = 1;
/// Query words matching a different live head's owner use this non-prime sentinel.
const OTHER_OWNER_ROLE: u32 = 4;
/// Preparation replays every construction prompt; learning keeps its own configured limit.
const PREPARATION_SECONDS: u64 = 600;
/// Construction documents and bound receipts per witness. Raised from 4096 for the
/// reassertion repair construction (4,131 documents); frames stay capped at 4096 and
/// the 8 MiB prompt bound is unchanged, so legacy artifacts remain valid.
const DOCUMENT_BOUND: usize = 8192;
/// Prior turns a construction document may replay before its labeled prompt.
const HISTORY_TURNS: usize = 8;

fn seed(model: &Model) -> Result<SourceRouting> {
    let h = historical_read::head(model, Control::Full)
        .ok_or_else(|| Error("historical version context absent".into()))?;
    if h.query_scope != 2 {
        return Err(Error("historical version requires local scope".into()));
    }
    let mut router = h.router.clone();
    router.codes.clear();
    Ok(router)
}

fn config_valid(config: &SourceRoutingConfig, previous: &SourceRouting) -> Result<()> {
    config.validate()?;
    if config.learned_features > previous.config.learned_features
        || config.mode != previous.config.mode
        || config.role_context_only != previous.config.role_context_only
    {
        return Err(Error(
            "historical version changed frozen capacity or mode".into(),
        ));
    }
    Ok(())
}

/// Parent restoration and non-code equality validate the router's remaining shape.
fn validate_lexical_codes(dictionary: &[WordCopyAddress], router: &SourceRouting) -> Result<()> {
    if dictionary.is_empty() || dictionary.len() > 128 {
        return Err(Error("invalid historical version dictionary bound".into()));
    }
    let expected = crate::corpus_induced_spin_placement::first_primes(dictionary.len())
        .map_err(|e| Error(e.to_string()))?;
    if dictionary.iter().zip(expected).any(|(word, prime)| {
        word.len == 0
            || word.len > 32
            || u64::from(word.prime) != prime
            || !(word.bytes[0].is_ascii_alphabetic() || word.bytes[0] == b'_')
            || word.bytes[..usize::from(word.len)]
                .iter()
                .any(|b| !b.is_ascii_alphanumeric() && *b != b'_')
            || word.bytes[usize::from(word.len)..].iter().any(|b| *b != 0)
    }) || dictionary.windows(2).any(|w| w[0].bytes >= w[1].bytes)
    {
        return Err(Error("invalid historical version dictionary".into()));
    }
    let primes: BTreeSet<u64> = [0, u64::from(OWNER_ROLE), u64::from(OTHER_OWNER_ROLE)]
        .into_iter()
        .chain(dictionary.iter().map(|w| u64::from(w.prime)))
        .collect();
    let class = |c: u64| (CLASS_PREVIOUS..=CLASS_HEAD_TRUNCATED).contains(&c);
    let valid = |f: ValueFeature| match f.kind {
        0 | 2 => f.a == 1 && f.b == 0,
        1 => f.a <= 1 && f.b == 0,
        3 | 7 => primes.contains(&f.a) && primes.contains(&f.b),
        4 | 5 => f.a <= 1 && primes.contains(&f.b),
        6 => primes.contains(&f.a) && f.b == 0,
        KIND_WORD_CLASS => primes.contains(&f.a) && class(f.b),
        KIND_DEPTH_CLASS => {
            f.a <= u64::from(ANCESTOR_DEPTH)
                && class(f.b)
                && (f.a > 0 || f.b == CLASS_TRUNCATED || f.b == CLASS_HEAD_TRUNCATED)
        }
        KIND_VIEW_CLASS => f.a <= 16 && class(f.b),
        _ => false,
    };
    if router.codes.is_empty()
        || router.codes.len() > router.config.learned_features
        || router
            .codes
            .windows(2)
            .any(|c| c[0].feature >= c[1].feature)
        || router
            .codes
            .iter()
            .any(|c| !valid(c.feature) || c.roots.iter().any(|r| *r >= 120))
    {
        return Err(Error("invalid historical version codes".into()));
    }
    Ok(())
}

/// The selector's vocabulary is drawn from its own bounded request views, using
/// the same deterministic frequency-then-bytes law and prime assignment as the
/// prompt dictionaries. Fact, owner and value words never consume a slot unless
/// they also occur inside a request view.
fn request_dictionary(
    views: &[LexemeState],
    limit: usize,
) -> Result<(Vec<WordCopyAddress>, usize, u64)> {
    if !(1..=128).contains(&limit) {
        return Err(Error("invalid request dictionary limit".into()));
    }
    // A word counts once per view, so a filler repeated inside one prompt stays a singleton.
    let mut counts = BTreeMap::<Vec<u8>, u64>::new();
    for view in views {
        let mut seen = BTreeSet::new();
        for word in &view.queries[..view.query_len] {
            if word.len == 0 || !seen.insert(word.bytes[..usize::from(word.len)].to_vec()) {
                continue;
            }
            *counts
                .entry(word.bytes[..usize::from(word.len)].to_vec())
                .or_default() += 1;
        }
    }
    let mut ranked: Vec<_> = counts.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let recurring = ranked
        .iter()
        .filter(|(_, n)| *n >= DICTIONARY_MIN_OCCURRENCES)
        .count();
    let kept = recurring.min(limit);
    let omitted_words = ranked.len().saturating_sub(kept);
    let omitted_occurrences = ranked.iter().skip(kept).map(|(_, n)| n).sum();
    ranked.truncate(kept);
    ranked.sort_by(|a, b| a.0.cmp(&b.0));
    let primes = crate::corpus_induced_spin_placement::first_primes(ranked.len())
        .map_err(|e| Error(e.to_string()))?;
    let words = ranked
        .into_iter()
        .zip(primes)
        .map(|((word, _), prime)| {
            let mut bytes = [0; 32];
            bytes[..word.len()].copy_from_slice(&word);
            Ok(WordCopyAddress {
                bytes,
                len: word.len() as u8,
                prime: u32::try_from(prime).map_err(|e| Error(e.to_string()))?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok((words, omitted_words, omitted_occurrences))
}

impl HistoricalVersionIntent {
    fn parent(&self, model: &Model) -> Result<Model> {
        let mut parent = model.clone();
        parent.historical_version_intent = None;
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("historical version parent differs".into()));
        }
        parent.validate()?;
        Ok(parent)
    }

    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        let parent = self.parent(model)?;
        let previous = seed(&parent)?;
        config_valid(&self.config, &previous)?;
        validate_lexical_codes(&self.dictionary, &self.router)?;
        let mut restored = self.router.clone();
        restored.codes.clear();
        if !self.committed_query_scope
            || restored != previous
            || self.router.codes.len() > self.config.learned_features
            || self.training.is_empty()
            || self.training.len() > DOCUMENT_BOUND
            || self.training.iter().any(|r| {
                r.id.trim().is_empty() || r.bytes == 0 || !r.text_cid.starts_with("blake3:")
            })
            || self
                .training
                .iter()
                .map(|r| &r.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.training.len()
        {
            return Err(Error(
                "invalid historical version frozen parameters or receipts".into(),
            ));
        }
        let mut duplicate = model.clone();
        duplicate.refresh_identity()?;
        if duplicate.artifact_cid != model.artifact_cid
            || duplicate.uor_model_address != model.uor_model_address
        {
            return Err(Error("historical version identity differs".into()));
        }
        Ok(())
    }
}

// NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN
/// Whether the versioned chain contract is active for this model under `control`.
pub(super) fn reassertion_links(model: &Model, control: Control) -> bool {
    model
        .historical_version_intent
        .as_ref()
        .is_some_and(|b| b.reassertion_links)
        && control != Control::HistoricalVersionIntentReassertionDisabled
        && control != Control::HistoricalVersionIntentDisabled
}

/// Whether the head contract (first hop through a same-value reassertion head) is
/// active for this model under `control`.
/// Owner-scan scope of the frozen persistent relation reader under `control`: the
/// word window, whether words at or before the latest committed fact are excluded
/// (request-window contract) and whether the current turn's input boundary also
/// bounds the scan (turn-window contract).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ReaderScope {
    pub window: usize,
    pub post_fact: bool,
    pub turn: bool,
}

pub(super) fn reader_scope(model: &Model, control: Control) -> ReaderScope {
    let witness = model.historical_version_intent.as_ref();
    reader_scope_for(
        witness.is_some_and(|b| b.reader_request_window),
        witness.is_some_and(|b| b.reader_turn_window),
        control,
    )
}

fn reader_scope_for(request: bool, turn: bool, control: Control) -> ReaderScope {
    let legacy = ReaderScope {
        window: 8,
        post_fact: false,
        turn: false,
    };
    if !request
        || control == Control::HistoricalVersionIntentReaderWindowDisabled
        || control == Control::HistoricalVersionIntentDisabled
    {
        return legacy;
    }
    ReaderScope {
        window: 16,
        post_fact: true,
        turn: turn && control != Control::HistoricalVersionIntentReaderTurnScopeDisabled,
    }
}

pub(super) fn reassertion_heads(model: &Model, control: Control) -> bool {
    model
        .historical_version_intent
        .as_ref()
        .is_some_and(|b| b.reassertion_heads)
        && control != Control::HistoricalVersionIntentReassertionHeadDisabled
        && control != Control::HistoricalVersionIntentDisabled
}

/// Selected exact ancestor and the live head that proves its path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct VersionChoice {
    pub source: u8,
    pub action: usize,
    pub current: u64,
    pub record: u64,
    pub depth: u8,
    /// The validated path to `record` passes through a same-value reassertion link.
    pub reassertion: bool,
    /// The path's first hop passes through a same-value reassertion head.
    pub head: bool,
}

/// One structurally admitted ancestor with its selector features.
pub(super) struct Scan {
    pub source: u8,
    pub current: u64,
    pub record: u64,
    pub depth: u8,
    pub root: bool,
    /// The chain ended without a root; `depth` counts its validated links.
    pub truncated: bool,
    /// The validated path so far passes through a same-value reassertion link.
    pub reassertion: bool,
    /// The first hop passed through a same-value reassertion head.
    pub head: bool,
    pub representable: bool,
    pub features: [ValueFeature; 96],
    pub n: usize,
}

fn class(depth: u8, root: bool) -> u64 {
    match (depth, root) {
        (1, false) => CLASS_PREVIOUS,
        (1, true) => CLASS_PREVIOUS_ROOT,
        (_, false) => CLASS_ANCESTOR,
        (_, true) => CLASS_ANCESTOR_ROOT,
    }
}

/// Chain position class, offset when the path starts through a reassertion head.
fn path_class(depth: u8, root: bool, head: bool) -> u64 {
    if head {
        class(depth, root) + CLASS_HEAD_OFFSET
    } else {
        class(depth, root)
    }
}

fn addresses(
    dictionary: &[WordCopyAddress],
    words: &LexemeState,
    work: &mut WordCopyWork,
) -> [u32; 16] {
    let mut out = [0; 16];
    for (i, word) in words.queries[..words.query_len].iter().enumerate() {
        work.word_record_reads += 1;
        out[i] = super::word_copy_runtime::address_in(dictionary, word, work);
        work.selector.state_copies += 1;
    }
    out
}

/// Inherited owner-context features, bounded ordered request context, then the
/// request words paired with the candidate's chain class and its exact depth.
/// The queried owner's own spelling is replaced by a role sentinel so a fresh
/// owner name and a construction name yield identical selector features.
fn features(
    model: &Model,
    state: &RelationState,
    current: &RelationRecord,
    words: &LexemeState,
    dictionary_addr: &[u32; 16],
    depth: u8,
    root: bool,
    truncated: bool,
    head: bool,
    work: &mut WordCopyWork,
) -> ([ValueFeature; 96], usize) {
    let mut masked = *dictionary_addr;
    for q in 0..words.query_len.min(16) {
        if current
            .owner
            .matches(&words.queries[q], &mut work.persistent_read)
        {
            masked[q] = OWNER_ROLE;
            continue;
        }
        for &id in &state.directory {
            work.persistent_read.relations.directory_reads += 1;
            if id == 0 || id == current.id {
                continue;
            }
            work.persistent_read.relations.record_reads += 1;
            if state.record(id).is_some_and(|head| {
                head.owner
                    .matches(&words.queries[q], &mut work.persistent_read)
            }) {
                masked[q] = OTHER_OWNER_ROLE;
                break;
            }
        }
    }
    let addr = &masked;
    let mut out = [ValueFeature::default(); 96];
    let (base, mut n) =
        super::relation::read_features(model, current, words, addr, &mut work.persistent_read);
    out[..n].copy_from_slice(&base[..n]);
    let inherited = n;
    // At most 34 inherited and 31 ordered features precede at most 18 class features.
    n = historical_read::append_ordered_context(&mut out, n, addr, words.query_len, 16);
    let c = if truncated {
        if head {
            CLASS_HEAD_TRUNCATED
        } else {
            CLASS_TRUNCATED
        }
    } else {
        path_class(depth, root, head)
    };
    let limit = words.query_len.min(16);
    for q in 0..limit {
        out[n] = ValueFeature {
            kind: KIND_WORD_CLASS,
            a: u64::from(addr[q]),
            b: c,
        };
        n += 1;
    }
    out[n] = ValueFeature {
        kind: KIND_DEPTH_CLASS,
        a: u64::from(depth),
        b: c,
    };
    n += 1;
    out[n] = ValueFeature {
        kind: KIND_VIEW_CLASS,
        a: limit as u64,
        b: c,
    };
    n += 1;
    work.persistent_read.relations.feature_writes += (n - inherited) as u64;
    (out, n)
}

/// Enumerate every validated ancestor of every live head whose owner the request
/// names, newest first, stopping at a genuine assertion root. Admission compares
/// exact owner identity; it never inspects request spelling otherwise, and chains
/// the request does not name are never candidates.
fn scan(
    model: &Model,
    values: &ValueState,
    words: &LexemeState,
    addr: &[u32; 16],
    copy: usize,
    ancestors: bool,
    abstention: bool,
    reassertions: bool,
    heads: bool,
    work: &mut WordCopyWork,
    visit: &mut impl FnMut(&Scan, &mut WordCopyWork),
) -> Option<()> {
    let state = values.relations.as_ref()?;
    for &id in &state.directory {
        work.persistent_read.relations.directory_reads += 1;
        work.persistent_read.relations.record_reads += 1;
        let Some(current) = state.record(id) else {
            continue;
        };
        let mut named = false;
        for q in 0..words.query_len.min(16) {
            if current
                .owner
                .matches(&words.queries[q], &mut work.persistent_read)
            {
                named = true;
                break;
            }
        }
        if !named {
            continue;
        }
        let mut record = current;
        let mut depth: u8 = 0;
        let mut reached_root = false;
        let mut stopped_by_control = false;
        let mut evicted = false;
        let mut via_reassertion = false;
        let mut via_head = false;
        while depth < ANCESTOR_DEPTH {
            let mut next = if depth == 0 {
                historical_read::previous(state, current, &mut work.persistent_read)
            } else {
                historical_read::link(state, record, &mut work.persistent_read)
            };
            if next.is_none() && depth == 0 && heads {
                // Head contract: the first hop below a same-value reassertion head.
                next = historical_read::head_reassertion(state, current, &mut work.persistent_read);
                via_head |= next.is_some();
            }
            if next.is_none() && depth > 0 && reassertions {
                next = historical_read::reassertion(state, record, &mut work.persistent_read);
                via_reassertion |= next.is_some();
            }
            let Some(old) = next else {
                // Absence is proven separately from link rejection: only an
                // overwritten predecessor slot counts as an evicted chain.
                evicted =
                    historical_read::evicted_predecessor(state, record, &mut work.persistent_read);
                break;
            };
            depth += 1;
            record = old;
            let root = historical_read::is_root(old);
            let representable = historical_read::representable(model, values, old, copy, work);
            let (f, n) = features(
                model, state, current, words, addr, depth, root, false, via_head, work,
            );
            work.routing.sources_examined += 1;
            let entry = Scan {
                source: RELATION_SOURCE + ((old.id - 1) & 15) as u8,
                current: id,
                record: old.id,
                depth,
                root,
                truncated: false,
                reassertion: via_reassertion,
                head: via_head,
                representable,
                features: f,
                n,
            };
            visit(&entry, work);
            if root {
                reached_root = true;
                break;
            }
            if !ancestors {
                stopped_by_control = true;
                break;
            }
        }
        // A revised head whose validated links end before any root is offered as
        // a truncated candidate. Under the versioned contract that requires the
        // next predecessor to be proven evicted from the ring; a rejected link is
        // not absence. The legacy contract offered it on any unreached root, which
        // mistook a resident same-value reassertion for truncation. Whether to
        // abstain remains learned.
        // Under the head contract a same-value reassertion head whose predecessor is
        // proven evicted is truncated as well.
        let reassertion_head =
            heads && current.action == 1 && current.previous != 0 && current.previous < current.id;
        if abstention
            && !reached_root
            && !stopped_by_control
            && (current.action == 2 || reassertion_head)
            && !current.conflict
            && (evicted || !reassertions)
        {
            // A truncated head chain is classed by the head that could not be
            // followed, whether the walk stopped at the head or below it.
            let (f, n) = features(
                model,
                state,
                current,
                words,
                addr,
                depth,
                false,
                true,
                via_head || reassertion_head,
                work,
            );
            work.routing.sources_examined += 1;
            let entry = Scan {
                source: super::role_read::NO_SOURCE,
                current: id,
                record: 0,
                depth,
                root: false,
                truncated: true,
                reassertion: via_reassertion,
                head: via_head || reassertion_head,
                representable: true,
                features: f,
                n,
            };
            visit(&entry, work);
        }
    }
    Some(())
}

/// Base returns None so the complete frozen dispatch, including the frozen
/// immediate-previous reader, remains available unchanged.
pub(super) fn choose_detail(
    model: &Model,
    values: &ValueState,
    control: Control,
    work: &mut WordCopyWork,
) -> Option<VersionChoice> {
    if matches!(
        control,
        Control::HistoricalVersionIntentDisabled
            | Control::LearnedRoutingDisabled
            | Control::LearnedRoutingSelectionDisabled
            | Control::H4Disabled
    ) {
        return None;
    }
    let block = model.historical_version_intent.as_ref()?;
    let (defer, copy) = historical_read::action_indices(model)?;
    work.routing.predictions += 1;
    let (words, _) = scoped_words(
        values.lexemes.as_ref()?,
        values.query_boundary,
        values.relations.as_ref(),
        block.committed_query_scope && control != Control::HistoricalVersionIntentScopeDisabled,
        work,
    );
    let addr = addresses(&block.dictionary, &words, work);
    let transform = if control == Control::HistoricalVersionIntentTransformDisabled {
        Control::LearnedRoutingTransformDisabled
    } else {
        control
    };
    let mut best_score = block.router.score(
        model,
        [model.geometry.identity; 2],
        defer,
        &mut work.routing,
    );
    let mut best = None;
    scan(
        model,
        values,
        &words,
        &addr,
        copy,
        control != Control::HistoricalVersionIntentAncestorDisabled,
        block.abstention && control != Control::HistoricalVersionIntentAbstainDisabled,
        reassertion_links(model, control),
        reassertion_heads(model, control),
        work,
        &mut |s: &Scan, work: &mut WordCopyWork| {
            if !s.representable {
                return;
            }
            // An ancestor competes under the copy action; a truncated chain
            // competes for the same no-read action the parent uses to abstain.
            let action = if s.truncated { defer } else { copy };
            let roots =
                block
                    .router
                    .encode(model, &s.features[..s.n], transform, &mut work.routing);
            let score = block.router.score(model, roots, action, &mut work.routing);
            work.routing.comparisons += 1;
            if score > best_score {
                best_score = score;
                best = Some(VersionChoice {
                    source: s.source,
                    action,
                    current: s.current,
                    record: s.record,
                    depth: s.depth,
                    reassertion: s.reassertion,
                    head: s.head,
                });
            }
        },
    )?;
    best
}

pub(super) fn choose(
    model: &Model,
    values: &ValueState,
    control: Control,
    work: &mut WordCopyWork,
) -> Option<(u8, usize)> {
    choose_detail(model, values, control, work).map(|v| (v.source, v.action))
}
// NATIVE_GEOMETRIC_INTEGER_KERNEL_END

impl Model {
    /// Allocating host diagnostic using the actual admission, lexical and H4 laws.
    /// Without the witness it reports structural admission alone: an empty dictionary,
    /// no scores and no choice, so a parent can be inspected before any fit.
    pub fn historical_version_trace(&self, prompt: &str) -> Result<serde_json::Value> {
        if prompt.is_empty() || prompt.len() > 65536 {
            return Err(Error("historical version trace input bound".into()));
        }
        let block = self.historical_version_intent.as_ref();
        let dictionary: &[WordCopyAddress] = block.map_or(&[], |b| b.dictionary.as_slice());
        let committed = block.is_none_or(|b| b.committed_query_scope);
        let tokens = self.encode(prompt)?;
        if tokens.len() > 8192 {
            return Err(Error("historical version trace token bound".into()));
        }
        let mut session = self.session(Control::Full)?;
        session.observe(self, BOS)?;
        for token in tokens {
            session.observe(self, token)?;
        }
        session.begin_response(self)?;
        session.predict(self)?;
        let values = session
            .values
            .as_ref()
            .ok_or_else(|| Error("historical version trace values absent".into()))?;
        let entry = session
            .response_entry
            .as_ref()
            .ok_or_else(|| Error("historical version trace entry absent".into()))?;
        let eligible = super::word_copy_runtime::eligible(self, entry, values, Control::Full);
        let (defer, copy) = historical_read::action_indices(self)
            .ok_or_else(|| Error("historical version trace actions absent".into()))?;
        let mut work = WordCopyWork::default();
        let (words, cutoff) = scoped_words(
            values
                .lexemes
                .as_ref()
                .ok_or_else(|| Error("historical version trace words absent".into()))?,
            values.query_boundary,
            values.relations.as_ref(),
            committed,
            &mut work,
        );
        let addr = addresses(dictionary, &words, &mut work);
        let captured: Vec<_> = words.queries[..words.query_len].iter().enumerate().map(|(q, word)| {
            serde_json::json!({"reverse_index":q,"word":String::from_utf8_lossy(&word.bytes[..usize::from(word.len)]),
                "prime":addr[q],"end":word.end,"byte_end":word.byte_end})
        }).collect();
        let base = block.map(|b| {
            b.router
                .score(self, [self.geometry.identity; 2], defer, &mut work.routing)
        });
        let mut candidates = Vec::new();
        scan(
            self,
            values,
            &words,
            &addr,
            copy,
            true,
            block.is_none_or(|b| b.abstention),
            block.is_none_or(|b| b.reassertion_links),
            block.is_none_or(|b| b.reassertion_heads),
            &mut work,
            &mut |s: &Scan, work: &mut WordCopyWork| {
                let scored = block.map(|b| {
                    let roots =
                        b.router
                            .encode(self, &s.features[..s.n], Control::Full, &mut work.routing);
                    let mapped: Vec<_> = s.features[..s.n]
                        .iter()
                        .map(|f| {
                            b.router
                                .codes
                                .binary_search_by_key(f, |c| c.feature)
                                .ok()
                                .map(|i| &b.router.codes[i])
                        })
                        .collect();
                    serde_json::json!({"mapped_codes":mapped,"roots":roots,
                        "score":b.router.score(self,roots,if s.truncated{defer}else{copy},&mut work.routing)})
                });
                candidates.push(serde_json::json!({"current":s.current,"record":s.record,"depth":s.depth,"root":s.root,"truncated":s.truncated,"reassertion":s.reassertion,
                    "class":if s.truncated{if s.head{CLASS_HEAD_TRUNCATED}else{CLASS_TRUNCATED}}else{path_class(s.depth,s.root,s.head)},"head":s.head,"source":s.source,"representable":s.representable,"features":s.features[..s.n],
                    "scored":scored}));
            },
        );
        let choice = choose_detail(self, values, Control::Full, &mut WordCopyWork::default());
        Ok(
            serde_json::json!({"artifact":self.artifact_cid(),"version_witness":block.is_some(),"query_boundary":values.query_boundary,
            "committed_query_scope":committed,"committed_source_byte_cutoff":cutoff,"reassertion_links":block.is_none_or(|b| b.reassertion_links),"reassertion_heads":block.is_none_or(|b| b.reassertion_heads),
            "captured":captured,"base_score":base,"candidates":candidates,"word_copy_eligible":eligible,
            "choice":choice.map(|v| serde_json::json!({"source":v.source,"action":v.action,"current":v.current,"record":v.record,"depth":v.depth,"reassertion":v.reassertion,"head":v.head,"abstain":v.source==super::role_read::NO_SOURCE})),
            "frozen_historical_choice":historical_read::choose(self,values,Control::Full,&mut WordCopyWork::default()),
            "actual_word_copy":session.word_copy_decision(),"actual_field":session.field_composition_decision()}),
        )
    }

    pub fn without_historical_version_intent(&self) -> Result<Model> {
        match &self.historical_version_intent {
            Some(w) => w.parent(self),
            None => Ok(self.clone()),
        }
    }

    /// The same learned witness under the head contract as well: a diagnostic
    /// candidate showing whether the learned codes already cover head-hop classes.
    /// The same learned witness under the reader-window contract: no parameter,
    /// dictionary or receipt changes, only the contract flag and the identity it
    /// binds. A repair candidate that decides whether a refit is needed at all.
    /// The same learned witness under the reader-turn-window contract, which builds on
    /// the request-window contract: no parameter, dictionary or receipt changes.
    pub fn with_reader_turn_window(&self) -> Result<Model> {
        let mut model = self.clone();
        let witness = model
            .historical_version_intent
            .as_mut()
            .ok_or_else(|| Error("historical version contract requires a witness".into()))?;
        if !witness.reader_request_window {
            return Err(Error(
                "historical version turn-window contract requires the request-window contract"
                    .into(),
            ));
        }
        if witness.reader_turn_window {
            return Err(Error(
                "historical version reader-turn-window contract already set".into(),
            ));
        }
        witness.reader_turn_window = true;
        model.refresh_identity()?;
        model.validate()?;
        Ok(model)
    }

    pub fn with_reader_window(&self) -> Result<Model> {
        let mut model = self.clone();
        let witness = model
            .historical_version_intent
            .as_mut()
            .ok_or_else(|| Error("historical version contract requires a witness".into()))?;
        if witness.reader_request_window {
            return Err(Error(
                "historical version reader-request-window contract already set".into(),
            ));
        }
        witness.reader_request_window = true;
        model.refresh_identity()?;
        model.validate()?;
        Ok(model)
    }

    pub fn with_reassertion_heads(&self) -> Result<Model> {
        let mut model = self.clone();
        let witness = model
            .historical_version_intent
            .as_mut()
            .ok_or_else(|| Error("historical version contract requires a witness".into()))?;
        if witness.reassertion_heads {
            return Err(Error("historical version head contract already set".into()));
        }
        witness.reassertion_heads = true;
        model.refresh_identity()?;
        model.validate()?;
        Ok(model)
    }

    /// The same learned witness under the versioned chain contract: no parameter,
    /// dictionary or receipt changes, only the contract flag and the identity it
    /// binds. A diagnostic repair candidate that decides whether a refit is needed.
    pub fn with_reassertion_links(&self) -> Result<Model> {
        let mut model = self.clone();
        let witness = model
            .historical_version_intent
            .as_mut()
            .ok_or_else(|| Error("historical version contract requires a witness".into()))?;
        if witness.reassertion_links {
            return Err(Error(
                "historical version contract already versioned".into(),
            ));
        }
        witness.reassertion_links = true;
        model.refresh_identity()?;
        model.validate()?;
        Ok(model)
    }

    /// Offline exact ancestor-record or explicit deferral labels refine fresh outer
    /// code roots over a construction dictionary. Response bytes are never inputs;
    /// the parent, its frozen historical reader and all inner parameters are unchanged.
    pub fn fit_historical_version_intent(
        &self,
        docs: &[HistoricalVersionExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        self.fit_historical_version_mode(docs, config, None)
    }

    /// Continue learning from an existing witness's code roots over identical
    /// data on its exact parent: a further bounded learning window, not a new
    /// design. The candidate binds the same receipts and dictionary law.
    pub fn refit_historical_version_intent(
        &self,
        docs: &[HistoricalVersionExample],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        let witness = self
            .historical_version_intent
            .as_ref()
            .ok_or_else(|| Error("historical version refit requires a witness".into()))?;
        let parent = witness.parent(self)?;
        parent.fit_historical_version_mode(docs, config, Some(witness))
    }

    /// Replay prior turns, each answered by this model with the complete frozen
    /// dispatch, then the labeled prompt to its response entry.
    fn version_session(&self, history: &[String], prompt: &str, start: Instant) -> Result<Session> {
        let mut session = self.session(Control::Full)?;
        session.observe(self, BOS)?;
        for prior in history {
            let tokens = self.encode(prior)?;
            if tokens.len() > 8192 {
                return Err(Error("historical version token bound".into()));
            }
            if session.needs_input_boundary() {
                session.end_response(self)?;
            }
            for token in tokens {
                if start.elapsed().as_secs() >= PREPARATION_SECONDS {
                    return Err(Error("historical version preparation time limit".into()));
                }
                session.observe(self, token)?;
            }
            session.begin_response(self)?;
            for _ in 0..96 {
                let predicted = session.predict(self)?;
                session.observe(self, predicted.token)?;
                if predicted.token == EOS {
                    break;
                }
            }
        }
        let tokens = self.encode(prompt)?;
        if tokens.len() > 8192 {
            return Err(Error("historical version token bound".into()));
        }
        if session.needs_input_boundary() {
            session.end_response(self)?;
        }
        for token in tokens {
            if start.elapsed().as_secs() >= PREPARATION_SECONDS {
                return Err(Error("historical version preparation time limit".into()));
            }
            session.observe(self, token)?;
        }
        session.begin_response(self)?;
        session.predict(self)?;
        Ok(session)
    }

    fn fit_historical_version_mode(
        &self,
        docs: &[HistoricalVersionExample],
        config: SourceRoutingConfig,
        warm: Option<&HistoricalVersionIntent>,
    ) -> Result<(Model, serde_json::Value)> {
        self.validate()?;
        let previous = seed(self)?;
        config_valid(&config, &previous)?;
        if self.historical_version_intent.is_some()
            || docs.is_empty()
            || docs.len() > DOCUMENT_BOUND
            || docs.iter().any(|d| {
                d.prompt.is_empty()
                    || d.prompt.len() > 65536
                    || d.history.len() > HISTORY_TURNS
                    || d.history.iter().any(|h| h.is_empty() || h.len() > 65536)
                    || d.target_record == Some(0)
                    || usize::from(d.inherit)
                        + usize::from(d.target_record.is_some())
                        + usize::from(d.abstain)
                        != 1
            })
            || docs
                .iter()
                .map(|d| d.prompt.len() + d.history.iter().map(String::len).sum::<usize>())
                .sum::<usize>()
                > 8 * 1024 * 1024
        {
            return Err(Error(
                "invalid historical version fit bounds or labels".into(),
            ));
        }
        let start = Instant::now();
        // First pass: the selector's own bounded request views supply its vocabulary.
        let mut views = Vec::with_capacity(docs.len());
        for d in docs {
            let session = self.version_session(&d.history, &d.prompt, start)?;
            let values = session
                .values
                .as_ref()
                .ok_or_else(|| Error("historical version values absent".into()))?;
            let (view, _) = scoped_words(
                values
                    .lexemes
                    .as_ref()
                    .ok_or_else(|| Error("historical version words absent".into()))?,
                values.query_boundary,
                values.relations.as_ref(),
                true,
                &mut WordCopyWork::default(),
            );
            views.push(view);
        }
        let (dictionary, omitted_words, omitted_occurrences) = request_dictionary(&views, 128)?;
        drop(views);
        if dictionary.is_empty() {
            return Err(Error(
                "historical version construction dictionary empty".into(),
            ));
        }
        let (defer, copy) = historical_read::action_indices(self)
            .ok_or_else(|| Error("historical version actions ambiguous".into()))?;
        let mut router = previous;
        let mut receipts = Vec::new();
        let mut ids = BTreeSet::new();
        let mut frames = Vec::new();
        let mut frame_ids = Vec::new();
        let mut seen = BTreeMap::new();
        let mut vocabulary = BTreeSet::new();
        let mut labels = Vec::new();
        let mut skipped = Vec::new();
        let mut duplicates = 0;
        let mut deepest_target: u8 = 0;
        let mut offered_total = 0usize;
        for d in docs {
            if start.elapsed().as_secs() >= PREPARATION_SECONDS {
                return Err(Error("historical version preparation time limit".into()));
            }
            if d.id.trim().is_empty() || !ids.insert(&d.id) {
                return Err(Error(
                    "historical version duplicate or empty document identity".into(),
                ));
            }
            receipts.push(super::training::receipt(&Document {
                id: d.id.clone(),
                text: serde_json::to_string(d).map_err(|e| Error(e.to_string()))?,
            }));
            let session = self.version_session(&d.history, &d.prompt, start)?;
            let values = session
                .values
                .as_ref()
                .ok_or_else(|| Error("historical version values absent".into()))?;
            let entry = session
                .response_entry
                .as_ref()
                .ok_or_else(|| Error("historical version entry absent".into()))?;
            if !super::word_copy_runtime::eligible(self, entry, values, Control::Full) {
                if d.target_record.is_some() {
                    return Err(Error(format!(
                        "historical version target ineligible: {}",
                        d.id
                    )));
                }
                skipped.push(d.id.clone());
                continue;
            }
            let mut work = WordCopyWork::default();
            let (words, _) = scoped_words(
                values
                    .lexemes
                    .as_ref()
                    .ok_or_else(|| Error("historical version words absent".into()))?,
                values.query_boundary,
                values.relations.as_ref(),
                true,
                &mut work,
            );
            let addr = addresses(&dictionary, &words, &mut work);
            let mut alternatives = vec![Alternative {
                features: Vec::new(),
                codes: Vec::new(),
                action: defer,
                correct: d.inherit,
            }];
            let mut offered = Vec::new();
            scan(
                self,
                values,
                &words,
                &addr,
                copy,
                true,
                true,
                true,
                true,
                &mut work,
                &mut |s: &Scan, _: &mut WordCopyWork| {
                    if !s.representable {
                        return;
                    }
                    alternatives.push(Alternative {
                        features: s.features[..s.n].to_vec(),
                        codes: Vec::new(),
                        action: if s.truncated { defer } else { copy },
                        correct: if s.truncated {
                            d.abstain
                        } else {
                            d.target_record == Some(s.record)
                        },
                    });
                    offered.push(serde_json::json!({"current":s.current,"record":s.record,"depth":s.depth,"root":s.root,"truncated":s.truncated,"reassertion":s.reassertion,"head":s.head}));
                },
            );
            if alternatives.iter().filter(|a| a.correct).count() != 1 {
                return Err(Error(format!(
                    "historical version exact target unavailable: {} target{:?} offered{:?}",
                    d.id, d.target_record, offered
                )));
            }
            if let Some(depth) = offered
                .iter()
                .find(|o| o["record"].as_u64() == d.target_record)
                .and_then(|o| o["depth"].as_u64())
            {
                deepest_target = deepest_target.max(depth as u8);
            }
            offered_total += offered.len();
            labels.push(serde_json::json!({"id":d.id,"inherit":d.inherit,"target_record":d.target_record,"abstain":d.abstain,"history_turns":d.history.len(),"offered":offered}));
            if alternatives.len() == 1 {
                skipped.push(d.id.clone());
                continue;
            }
            let signature: Vec<_> = alternatives
                .iter()
                .map(|a| (a.features.clone(), a.action))
                .collect();
            let target_index = alternatives
                .iter()
                .position(|a| a.correct)
                .ok_or_else(|| Error("historical version target absent".into()))?;
            if let Some((prior_target, prior_id)) = seen.get(&signature) {
                if *prior_target != target_index {
                    return Err(Error(format!(
                        "historical version conflicting exact frames: {} and {}",
                        prior_id, d.id
                    )));
                }
                duplicates += 1;
                continue;
            }
            seen.insert(signature, (target_index, d.id.clone()));
            for a in &alternatives {
                vocabulary.extend(a.features.iter().copied());
            }
            if vocabulary.len() > config.learned_features {
                return Err(Error(format!(
                    "historical version feature bound: {} > {}",
                    vocabulary.len(),
                    config.learned_features
                )));
            }
            if frames.len() >= 4096 {
                return Err(Error("historical version frame cap4096".into()));
            }
            frames.push(Frame { alternatives });
            frame_ids.push(d.id.clone());
        }
        if frames.is_empty() {
            return Err(Error("historical version has no competitive frames".into()));
        }
        let mut warm_roots = 0usize;
        router.codes = vocabulary
            .into_iter()
            .map(|feature| {
                let roots = warm
                    .and_then(|w| {
                        w.router
                            .codes
                            .binary_search_by_key(&feature, |c| c.feature)
                            .ok()
                            .map(|i| w.router.codes[i].roots)
                    })
                    .unwrap_or([self.geometry.identity; 2]);
                if roots != [self.geometry.identity; 2] {
                    warm_roots += 1;
                }
                SourceCode { feature, roots }
            })
            .collect();
        if let Some(w) = warm {
            if w.dictionary != dictionary || w.training != receipts {
                return Err(Error(
                    "historical version refit data or dictionary differs from the warm witness"
                        .into(),
                ));
            }
        }
        let original_config = router.config.clone();
        router.config = config.clone();
        let mutable = vec![true; router.codes.len()];
        let preparation_ms = start.elapsed().as_millis();
        let learning = Instant::now();
        let fit = learn_code_subset(self, &mut router, &mut frames, learning, &mutable)?;
        router.config = original_config;
        let feature_count = router.codes.len();
        // Name every frame the final roots still leave unsatisfied, by its first document.
        let mut unsatisfied = Vec::new();
        for (frame, id) in frames.iter().zip(&frame_ids) {
            let mut best: Option<(i64, bool)> = None;
            for a in &frame.alternatives {
                let mut work = WordCopyWork::default();
                let roots = router.encode(self, &a.features, Control::Full, &mut work.routing);
                let score = router.score(self, roots, a.action, &mut work.routing);
                if best.is_none_or(|(prior, _)| score > prior) {
                    best = Some((score, a.correct));
                }
            }
            if !best.is_some_and(|(_, correct)| correct) {
                unsatisfied.push(id.clone());
            }
        }
        let mut model = self.clone();
        model.historical_version_intent = Some(HistoricalVersionIntent {
            parent_artifact: self.artifact_cid().to_owned(),
            router,
            config,
            training: receipts,
            dictionary: dictionary.clone(),
            committed_query_scope: true,
            abstention: true,
            reassertion_links: true,
            reassertion_heads: true,
            reader_request_window: true,
            reader_turn_window: true,
        });
        model.refresh_identity()?;
        model.validate()?;
        let report = serde_json::json!({"schema":"uor-r4.historical-version-intent-fit/1","parent":self.artifact_cid(),
            "artifact":model.artifact_cid(),"documents":docs.len(),"frames":frames.len(),"duplicate_frames":duplicates,
            "skipped":skipped,"labels":labels,"features":feature_count,"offered_candidates":offered_total,
            "deepest_target_depth":deepest_target,"ancestor_depth_bound":ANCESTOR_DEPTH,"committed_query_scope":true,"abstention":true,"reassertion_links":true,"reassertion_heads":true,"abstain_documents":docs.iter().filter(|d| d.abstain).count(),"follow_up_documents":docs.iter().filter(|d| !d.history.is_empty()).count(),"history_turn_bound":HISTORY_TURNS,
            "dictionary_mode":"request_view_words","dictionary_limit":128,"dictionary_min_occurrences":DICTIONARY_MIN_OCCURRENCES,"dictionary_words":dictionary.len(),
            "dictionary_omitted_words":omitted_words,"dictionary_omitted_occurrences":omitted_occurrences,"dictionary":dictionary,
            "fit":fit,"unsatisfied_frames":unsatisfied,"warm_start":warm.map(|w| serde_json::json!({"codes":w.router.codes.len(),"nonidentity_roots_reused":warm_roots})),"preparation_ms":preparation_ms,"preparation_seconds_limit":PREPARATION_SECONDS,"learning_ms":learning.elapsed().as_millis(),"elapsed_ms":start.elapsed().as_millis(),
            "scope":"Exact validated ancestor candidates or complete deferral. Only new outer H4 code roots learned; parent, frozen historical reader and inner dictionaries unchanged. Generated behavior requires separate evaluation."});
        Ok((model, report))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn historical_version_reader_scope_follows_the_declared_contracts_and_controls() {
        use super::super::Control;
        use super::{reader_scope_for, ReaderScope};
        let legacy = ReaderScope {
            window: 8,
            post_fact: false,
            turn: false,
        };
        let request = ReaderScope {
            window: 16,
            post_fact: true,
            turn: false,
        };
        let turn = ReaderScope {
            window: 16,
            post_fact: true,
            turn: true,
        };
        assert_eq!(reader_scope_for(false, false, Control::Full), legacy);
        assert_eq!(reader_scope_for(false, true, Control::Full), legacy);
        assert_eq!(reader_scope_for(true, false, Control::Full), request);
        assert_eq!(reader_scope_for(true, true, Control::Full), turn);
        assert_eq!(
            reader_scope_for(
                true,
                true,
                Control::HistoricalVersionIntentReassertionHeadDisabled
            ),
            turn
        );
        assert_eq!(
            reader_scope_for(
                true,
                true,
                Control::HistoricalVersionIntentReaderTurnScopeDisabled
            ),
            request
        );
        assert_eq!(
            reader_scope_for(
                true,
                true,
                Control::HistoricalVersionIntentReaderWindowDisabled
            ),
            legacy
        );
        assert_eq!(
            reader_scope_for(true, true, Control::HistoricalVersionIntentDisabled),
            legacy
        );
    }

    use super::*;
    fn word(text: &str) -> WordCopyAddress {
        let mut w = WordCopyAddress {
            bytes: [0; 32],
            len: text.len() as u8,
            prime: 0,
        };
        w.bytes[..text.len()].copy_from_slice(text.as_bytes());
        w
    }
    fn fixture() -> (Vec<WordCopyAddress>, SourceRouting) {
        let docs = [ValueExample {
            id: "words".into(),
            prompt: "initial previous".into(),
            response: String::new(),
        }];
        let dictionary = super::super::word_copy_training::dictionary_with_limit(&docs, 128)
            .unwrap()
            .0;
        let router = SourceRouting {
            schema: "uor-r4.geometric-source-routing/1".into(),
            parent_artifact: "blake3:parent".into(),
            codes: vec![
                SourceCode {
                    feature: ValueFeature {
                        kind: KIND_WORD_CLASS,
                        a: 2,
                        b: CLASS_ANCESTOR_ROOT,
                    },
                    roots: [7, 11],
                },
                SourceCode {
                    feature: ValueFeature {
                        kind: KIND_DEPTH_CLASS,
                        a: 2,
                        b: CLASS_ANCESTOR_ROOT,
                    },
                    roots: [119, 119],
                },
            ],
            landmarks: Vec::new(),
            biases: Vec::new(),
            ranks: Vec::new(),
            training: Vec::new(),
            config: SourceRoutingConfig::default(),
        };
        (dictionary, router)
    }
    #[test]
    fn historical_version_classes_are_exact_chain_positions() {
        assert_eq!(class(1, false), CLASS_PREVIOUS);
        assert_eq!(class(1, true), CLASS_PREVIOUS_ROOT);
        assert_eq!(class(2, false), CLASS_ANCESTOR);
        assert_eq!(class(2, true), CLASS_ANCESTOR_ROOT);
        assert_eq!(class(ANCESTOR_DEPTH, true), CLASS_ANCESTOR_ROOT);
        assert_eq!(
            path_class(1, false, true),
            CLASS_PREVIOUS + CLASS_HEAD_OFFSET
        );
        assert_eq!(path_class(2, true, true), CLASS_HEAD_ANCESTOR_ROOT);
        assert_eq!(path_class(2, true, false), CLASS_ANCESTOR_ROOT);
    }
    #[test]
    fn historical_version_codes_require_known_primes_classes_and_depths() {
        let (dictionary, router) = fixture();
        assert!(validate_lexical_codes(&dictionary, &router).is_ok());
        let mut bad = router.clone();
        bad.codes[0].feature.b = CLASS_TRUNCATED;
        assert!(
            validate_lexical_codes(&dictionary, &bad).is_ok(),
            "the truncated class is a valid word pairing"
        );
        // Classes six to nine are the four chain positions reached through a
        // same-value reassertion head; ten is beyond the contract.
        bad.codes[0].feature.b = CLASS_TRUNCATED + 1;
        assert!(validate_lexical_codes(&dictionary, &bad).is_ok());
        bad.codes[0].feature.b = CLASS_HEAD_ANCESTOR_ROOT;
        assert!(validate_lexical_codes(&dictionary, &bad).is_ok());
        bad.codes[0].feature.b = CLASS_HEAD_TRUNCATED;
        assert!(validate_lexical_codes(&dictionary, &bad).is_ok());
        bad.codes[0].feature.b = CLASS_HEAD_TRUNCATED + 1;
        assert!(validate_lexical_codes(&dictionary, &bad).is_err());
        bad = router.clone();
        bad.codes[0].feature.a = 6;
        assert!(validate_lexical_codes(&dictionary, &bad).is_err());
        bad = router.clone();
        bad.codes[0].feature.a = u64::from(OTHER_OWNER_ROLE);
        assert!(validate_lexical_codes(&dictionary, &bad).is_ok());
        bad.codes[0].feature.a = u64::from(OWNER_ROLE);
        assert!(validate_lexical_codes(&dictionary, &bad).is_ok());
        bad = router.clone();
        bad.codes[1].feature.a = u64::from(ANCESTOR_DEPTH) + 1;
        assert!(validate_lexical_codes(&dictionary, &bad).is_err());
        bad = router.clone();
        bad.codes[1].feature.a = 0;
        assert!(validate_lexical_codes(&dictionary, &bad).is_err());
        bad.codes[1].feature.b = CLASS_TRUNCATED;
        assert!(
            validate_lexical_codes(&dictionary, &bad).is_ok(),
            "a truncated chain may have zero validated links"
        );
        bad.codes[1].feature.b = CLASS_HEAD_TRUNCATED;
        assert!(
            validate_lexical_codes(&dictionary, &bad).is_ok(),
            "a truncated head chain may have zero validated links"
        );
        bad.codes[1].feature.b = CLASS_HEAD_TRUNCATED + 1;
        assert!(validate_lexical_codes(&dictionary, &bad).is_err());
        bad = router.clone();
        bad.codes[1].feature.kind = 11;
        assert!(validate_lexical_codes(&dictionary, &bad).is_err());
        bad = router.clone();
        bad.codes[0].roots[1] = 120;
        assert!(validate_lexical_codes(&dictionary, &bad).is_err());
        bad = router;
        bad.codes.swap(0, 1);
        assert!(validate_lexical_codes(&dictionary, &bad).is_err());
        let mut unsorted = dictionary.clone();
        unsorted.swap(0, 1);
        assert!(validate_lexical_codes(&unsorted, &fixture().1).is_err());
        let mut renumbered = dictionary;
        renumbered[0].prime = 0;
        assert!(validate_lexical_codes(&renumbered, &fixture().1).is_err());
        assert!(validate_lexical_codes(&vec![word("only"); 129], &fixture().1).is_err());
    }
    #[test]
    fn historical_version_wire_rejects_unknown_fields_and_response_labels() {
        let (dictionary, router) = fixture();
        let witness = HistoricalVersionIntent {
            parent_artifact: "blake3:parent".into(),
            config: router.config.clone(),
            router,
            training: Vec::new(),
            dictionary,
            committed_query_scope: true,
            abstention: false,
            reassertion_links: false,
            reassertion_heads: false,
            reader_request_window: false,
            reader_turn_window: false,
        };
        let bytes = serde_json::to_vec(&witness).unwrap();
        let legacy = serde_json::to_value(&witness).unwrap();
        assert!(
            legacy.get("abstention").is_none()
                && legacy.get("reassertion_links").is_none()
                && legacy.get("reassertion_heads").is_none(),
            "legacy artifacts keep their exact wire form"
        );
        let mut abstaining = witness.clone();
        abstaining.abstention = true;
        let restored: HistoricalVersionIntent =
            serde_json::from_slice(&serde_json::to_vec(&abstaining).unwrap()).unwrap();
        assert!(restored.abstention);
        assert!(
            !restored.reassertion_links,
            "the abstention flag alone keeps the legacy chain contract"
        );
        let mut versioned = abstaining.clone();
        versioned.reassertion_links = true;
        let wire = serde_json::to_value(&versioned).unwrap();
        assert_eq!(wire["reassertion_links"], true);
        let restored: HistoricalVersionIntent = serde_json::from_value(wire).unwrap();
        assert!(restored.reassertion_links && restored.abstention);
        assert!(
            !restored.reassertion_heads,
            "the link contract alone keeps assertion heads unread"
        );
        let mut heads = restored.clone();
        heads.reassertion_heads = true;
        let wire = serde_json::to_value(&heads).unwrap();
        assert_eq!(wire["reassertion_heads"], true);
        assert!(
            serde_json::from_value::<HistoricalVersionIntent>(wire)
                .unwrap()
                .reassertion_heads
        );
        let restored: HistoricalVersionIntent = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored, witness);
        assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
        let mut unknown = serde_json::to_value(&witness).unwrap();
        unknown["max_depth"] = serde_json::json!(4);
        assert!(serde_json::from_value::<HistoricalVersionIntent>(unknown).is_err());
        let example = HistoricalVersionExample {
            id: "initial".into(),
            prompt: "prompt".into(),
            target_record: Some(1),
            inherit: false,
            abstain: false,
            history: Vec::new(),
        };
        assert!(
            serde_json::to_value(&example)
                .unwrap()
                .get("history")
                .is_none(),
            "single-turn documents keep their exact wire form"
        );
        let follow_up: HistoricalVersionExample = serde_json::from_value(serde_json::json!({
            "id":"follow","prompt":"What was the previous location of a? Answer:","target_record":null,"inherit":true,
            "history":["Record: a in b. a now in c. What was the initial location of a? Answer:"]
        }))
        .unwrap();
        assert_eq!(follow_up.history.len(), 1);
        assert!(serde_json::to_value(&example)
            .unwrap()
            .get("abstain")
            .is_none());
        let mut label = serde_json::to_value(&example).unwrap();
        label["response"] = serde_json::json!(" Dusk Ridge.\n");
        assert!(serde_json::from_value::<HistoricalVersionExample>(label).is_err());
        let mut legacy = serde_json::to_value(&example).unwrap();
        legacy["current_record"] = serde_json::json!(3);
        assert!(serde_json::from_value::<HistoricalVersionExample>(legacy).is_err());
    }
}
