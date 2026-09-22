//! Observed-text memory answers through one reusable session boundary.
//!
//! The serving input is **observed tokens plus declared system metadata only**: a tokenized document
//! of statements, an observed question clause, and declared clause boundaries. An entity registry
//! is supplied only to the explicitly named membership control. No gold semantic role,
//! subject/object/answer extent, follow pointer, depth or target is available to the primary path.
//!
//! What is learned from declared development text:
//!
//! * `feature_weights`: signed categorical weights for a candidate marker's endpoints, nearby
//!   tokens, length and clause-edge flags, fitted by a structured perceptron;
//! * `action_per_role`: the action semantics of each role, fitted from declared gold actions starting
//!   from a declared uninformed policy.
//! * `goal_per_role`: the declared goal associated with each supervised role.
//!
//! Candidate support is bounded disjoint subject/cue/optional-object spans in any order. The cue
//! remains at most four tokens because its retained role scorer has that feature bound. Unassigned
//! tokens have fixed zero background score; background boundaries are not learned. Segment weights
//! learn the argument extents, with optional ordered finite-group features. Interior categorical
//! tokens are a bag, not an order encoding. General readable-language performance remains empirical.
//!
//! What is exact and typed: entity/answer spans keep full multiword token identity together with
//! their occurrence `(segment, start, len)`. Joins use tagged exact byte keys for adapter-validated
//! aligned text, or explicitly legacy token keys. The source-bound document is immutable here.
//!
//! The loaded session boundary [`ObservedTextRuntime::step`] is the single causal implementation used by teacher
//! forcing, generation, interventions, resumption and timing. `RelAction` is reused from the corrected predecessor; complete owned spans retain source provenance.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use super::realtext_support::sha256_hex;
use super::relational_session::RelAction;

/// Classified failures at the public observation/model/session boundaries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedTextError {
    Model(String),
    Observation(String),
    Session(String),
    Supervision(String),
    Serialization(String),
}

impl std::fmt::Display for ObservedTextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (category, detail) = match self {
            Self::Model(detail) => ("model", detail),
            Self::Observation(detail) => ("observation", detail),
            Self::Session(detail) => ("session", detail),
            Self::Supervision(detail) => ("supervision", detail),
            Self::Serialization(detail) => ("serialization", detail),
        };
        write!(f, "observed-text {category}: {detail}")
    }
}

impl std::error::Error for ObservedTextError {}

/// Experiment/CLI adapters may retain their existing textual error boundary.
impl From<ObservedTextError> for String {
    fn from(error: ObservedTextError) -> Self {
        error.to_string()
    }
}

/// Feature kinds for the shared candidate extractor.
pub const F_FIRST: u64 = 1;
pub const F_LAST: u64 = 2;
pub const F_LEFT: u64 = 3;
pub const F_RIGHT: u64 = 4;
pub const F_LEFT2: u64 = 5;
pub const F_RIGHT2: u64 = 6;
pub const F_LEN: u64 = 7;
pub const F_INITIAL: u64 = 8;
pub const F_FINAL: u64 = 9;
/// Declared maximum marker length considered by the candidate enumerator.
pub const OB_MAX_MARKER: usize = 4;

/// **The one candidate observation extractor**, used identically by fitting and serving. It encodes
/// the candidate's ordered local context, never the candidate's role or the expected answer.
pub fn candidate_features(tokens: &[u32], start: usize, len: usize) -> Vec<u64> {
    let n = tokens.len();
    let key = |kind: u64, value: u64| (kind << 40) | value;
    let mut f = Vec::with_capacity(9);
    let Some(end) = start
        .checked_add(len)
        .filter(|end| len > 0 && len <= OB_MAX_MARKER && *end <= n)
    else {
        return f;
    };
    f.push(key(F_FIRST, u64::from(tokens[start])));
    f.push(key(F_LAST, u64::from(tokens[end - 1])));
    f.push(key(F_LEN, len as u64));
    if start > 0 {
        f.push(key(F_LEFT, u64::from(tokens[start - 1])));
    }
    if end < n {
        f.push(key(F_RIGHT, u64::from(tokens[end])));
    }
    if start >= 2 {
        f.push(key(F_LEFT2, u64::from(tokens[start - 2])));
    }
    if let Some(right2) = end.checked_add(1).filter(|right2| *right2 < n) {
        f.push(key(F_RIGHT2, u64::from(tokens[right2])));
    }
    if start == 0 {
        f.push(key(F_INITIAL, 0));
    }
    if end == n {
        f.push(key(F_FINAL, 0));
    }
    f
}

/// Number of learned statement roles: office assertion/redirect and project assertion/redirect.
pub const OB_N_ROLES: usize = 4;
/// Declared execution cap; reaching it is reported as an exhausted/cycle outcome, never a success.
pub const OB_MAX_READS: u8 = 6;
/// Declared maximum clause length considered by the extractor.
pub const OB_MAX_CLAUSE: usize = 24;

/// A declared goal: which fact a request asks for.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum Goal {
    Office,
    Project,
}

impl Goal {
    pub fn index(self) -> usize {
        match self {
            Goal::Office => 0,
            Goal::Project => 1,
        }
    }
    pub fn from_index(i: usize) -> Option<Goal> {
        match i {
            0 => Some(Goal::Office),
            1 => Some(Goal::Project),
            _ => None,
        }
    }
}

/// One exact observed span: its tokens plus its occurrence identity inside the document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedSpan {
    pub seg: u32,
    pub start: u32,
    pub tokens: Vec<u32>,
}

impl ObservedSpan {
    pub fn len(&self) -> usize {
        self.tokens.len()
    }
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
    fn position(&self) -> (u32, u32, usize) {
        (self.seg, self.start, self.tokens.len())
    }
}

/// One statement clause as the *extractor* sees it, with no gold fields attached.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clause {
    pub seg: u32,
    pub tokens: Vec<u32>,
    /// The original observed text this clause was tokenized from.
    #[serde(default)]
    pub text: String,
    /// Exact byte length of each token inside `text`, so byte offsets and lexical keys are exact
    /// rather than recovered by lossy per-token decoding. May be empty for legacy fixtures.
    #[serde(default)]
    pub byte_lengths: Vec<u32>,
}

impl Clause {
    /// A clause observed without byte alignment (legacy fixtures fall back to token identity).
    pub fn of(seg: u32, tokens: Vec<u32>) -> Self {
        Clause {
            seg,
            tokens,
            text: String::new(),
            byte_lengths: Vec::new(),
        }
    }
}

/// Declared offline supervision for one development clause. Never available at serving.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClauseLabel {
    pub seg: u32,
    pub subject: (u32, usize),
    pub marker: (u32, usize),
    /// The observed object extent, or `None` for a question clause.
    pub object: Option<(u32, usize)>,
    pub role: usize,
    /// Declared offline goal for this role; supervised, never served.
    pub goal: Goal,
    /// Declared offline action semantics for this role; supervised, never served.
    pub action: RelAction,
}
/// Segment roles for the bounded structured span model: background, subject, cue, object.
pub const SEG_BACKGROUND: usize = 0;
pub const SEG_SUBJECT: usize = 1;
pub const SEG_CUE: usize = 2;
pub const SEG_OBJECT: usize = 3;
pub const OB_SEG_ROLES: usize = 4;
/// Declared maximum length of one contiguous segment.
pub const OB_MAX_SEG: usize = OB_MAX_CLAUSE;

/// Feature kinds for the segment potential.
pub const G_FIRST: u64 = 16;
pub const G_LAST: u64 = 17;
pub const G_LEFT: u64 = 18;
pub const G_RIGHT: u64 = 19;
pub const G_LEN: u64 = 20;
pub const G_INITIAL: u64 = 21;
pub const G_FINAL: u64 = 22;
/// Every interior token of the segment, which the previous endpoint-only features omitted.
pub const G_INTERIOR: u64 = 23;
/// Ordered interval element of the segment under the learned per-token group elements.
pub const G_H4_INTERVAL: u64 = 24;
/// Relative element between the segment interval and its left/right neighbour intervals.
pub const G_H4_LEFT: u64 = 25;
pub const G_H4_RIGHT: u64 = 26;

/// Exact lexical key of a selected span: the span's original bytes with only exterior ASCII
/// whitespace removed. Case and all interior bytes are preserved, so `Cedar` and `Cedar Annex` stay
/// different and the same name keeps one key across BPE boundary differences at sentence positions.
/// The surface span remains the provenance and copy source; this key never replaces it.
pub fn lexical_key(text: &str, byte_lengths: &[u32], start: usize, len: usize) -> Option<Vec<u8>> {
    let end = start
        .checked_add(len)
        .filter(|end| len > 0 && *end <= byte_lengths.len())?;
    if byte_lengths.contains(&0) {
        return None;
    }
    let sum = |lengths: &[u32]| {
        lengths
            .iter()
            .try_fold(0usize, |n, b| n.checked_add(*b as usize))
    };
    if sum(byte_lengths)? != text.len() {
        return None;
    }
    let offset = sum(&byte_lengths[..start])?;
    let span = sum(&byte_lengths[start..end])?;
    let slice = text.as_bytes().get(offset..offset.checked_add(span)?)?;
    let lo = slice
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .unwrap_or(slice.len());
    let hi = slice
        .iter()
        .rposition(|b| !b.is_ascii_whitespace())
        .map(|i| i + 1)
        .unwrap_or(lo);
    (lo < hi).then(|| slice[lo..hi].to_vec())
}

/// The adapter must additionally verify decoded token bytes against the original input. Lengths
/// alone establish extents, not authenticity of a tokenizer/text pairing. Empty lengths explicitly
/// select the legacy token-only regime; malformed supplied alignment is never silently downgraded.
fn validate_alignment(clause: &Clause) -> Result<(), String> {
    if clause.byte_lengths.is_empty() {
        return Ok(());
    }
    if clause.byte_lengths.len() != clause.tokens.len()
        || clause.byte_lengths.contains(&0)
        || clause
            .byte_lengths
            .iter()
            .try_fold(0usize, |n, b| n.checked_add(*b as usize))
            != Some(clause.text.len())
    {
        return Err("supplied byte alignment does not tile the observed text".into());
    }
    Ok(())
}

fn identity_key(clause: &Clause, start: usize, len: usize) -> Result<Vec<u8>, String> {
    let mut key;
    if clause.byte_lengths.is_empty() {
        key = vec![0]; // distinct from exact text bytes, even when payload bytes coincide
        let end = start.checked_add(len).ok_or("token key extent overflow")?;
        for token in clause
            .tokens
            .get(start..end)
            .ok_or("token key outside clause")?
        {
            key.extend_from_slice(&token.to_le_bytes());
        }
    } else {
        key = vec![1];
        key.extend(
            lexical_key(&clause.text, &clause.byte_lengths, start, len)
                .ok_or("selected span has no non-whitespace lexical identity")?,
        );
    }
    Ok(key)
}

fn valid_identity_key(key: &[u8]) -> bool {
    match key.split_first() {
        Some((0, rest)) => !rest.is_empty() && rest.len() % 4 == 0,
        Some((1, rest)) => {
            !rest.is_empty()
                && !rest[0].is_ascii_whitespace()
                && !rest[rest.len() - 1].is_ascii_whitespace()
        }
        _ => false,
    }
}

/// Recover a segment's interval product from ordered group prefixes: `inverse(P[start]) * P[end]`.
fn interval_product(prefix: &[usize], start: usize, end: usize) -> Option<usize> {
    let t = super::group_table::group_table();
    let a = *prefix.get(start)?;
    let b = *prefix.get(end)?;
    if start > end || a >= super::group_table::GROUP_ORDER || b >= super::group_table::GROUP_ORDER {
        return None;
    }
    Some(t.product[t.inverse[a] as usize * super::group_table::ROW_STRIDE + b] as usize)
}

/// The relative group element between two interval products, `inverse(a) * b`.
fn relative_element(a: usize, b: usize) -> usize {
    let t = super::group_table::group_table();
    t.product[t.inverse[a.min(super::group_table::GROUP_ORDER - 1)] as usize
        * super::group_table::ROW_STRIDE
        + b.min(super::group_table::GROUP_ORDER - 1)] as usize
}

/// One decoded segmentation: exact token extents for subject, cue and optional object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segmentation {
    pub subject: (usize, usize),
    pub cue: (usize, usize),
    pub object: Option<(usize, usize)>,
    pub score: i64,
}

/// The learned observation model.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedTextModel {
    pub version: u8,
    /// Sparse learned candidate-feature weights, sorted by feature key. A candidate is scored from
    /// its **ordered local context** (its own first/last tokens, the tokens immediately before and
    /// after it, its length and its clause-relative position). These observations can distinguish
    /// some permutations and contextual roles; four-token marker interior order is not represented.
    pub feature_weights: Vec<(u64, [i32; OB_N_ROLES])>,
    /// Learned segment potentials: `(feature key, weight per segment role)`.
    pub segment_weights: Vec<(u64, [i32; OB_SEG_ROLES])>,
    /// Learned per-token group elements used by the ordered interval features when `use_h4`.
    pub token_elements: Vec<(u32, u8)>,
    /// Whether the ordered H4 interval features are part of this model's feature family.
    pub use_h4: bool,
    /// Learned action semantics per role, fitted from declared gold actions.
    pub action_per_role: [u8; OB_N_ROLES],
    /// Learned goal per role.
    pub goal_per_role: [u8; OB_N_ROLES],
    /// Whether that role's action is a redirect (continue) rather than a terminal assertion.
    pub redirect_per_role: [bool; OB_N_ROLES],
}

fn action_tag(a: RelAction) -> u8 {
    match a {
        RelAction::Read => 1,
        RelAction::Continue => 2,
        RelAction::Emit => 3,
        RelAction::Stop => 4,
        RelAction::Unresolved => 5,
        RelAction::Exhausted => 6,
    }
}

fn action_from_tag(t: u8) -> RelAction {
    match t {
        1 => RelAction::Read,
        2 => RelAction::Continue,
        3 => RelAction::Emit,
        4 => RelAction::Stop,
        5 => RelAction::Unresolved,
        _ => RelAction::Exhausted,
    }
}

impl ObservedTextModel {
    /// A declared uninformed start: no candidate weights, every role terminal office.
    pub fn uninformed() -> Self {
        ObservedTextModel {
            version: 5,
            feature_weights: Vec::new(),
            segment_weights: Vec::new(),
            token_elements: Vec::new(),
            use_h4: false,
            action_per_role: [action_tag(RelAction::Emit); OB_N_ROLES],
            goal_per_role: [0; OB_N_ROLES],
            redirect_per_role: [false; OB_N_ROLES],
        }
    }

    pub fn validate(&self, max_vocab: usize) -> Result<(), ObservedTextError> {
        self.validate_model(max_vocab)
            .map_err(ObservedTextError::Model)
    }

    fn validate_model(&self, max_vocab: usize) -> Result<(), String> {
        if self.version != 5 {
            return Err("unsupported observed-text model version".into());
        }
        if max_vocab == 0 {
            return Err("empty vocabulary".into());
        }
        if self.feature_weights.windows(2).any(|w| w[0].0 >= w[1].0) {
            return Err("observed-text feature weights must have unique sorted keys".into());
        }
        for (key, _) in self.feature_weights.iter() {
            let kind = key >> 40;
            let value = key & ((1u64 << 40) - 1);
            if !(F_FIRST..=F_FINAL).contains(&kind) {
                return Err("unknown observed-text feature kind".into());
            }
            match kind {
                F_LEN if value == 0 || value > OB_MAX_MARKER as u64 => {
                    return Err(
                        "observed-text feature length is outside the candidate bound".into(),
                    );
                }
                F_INITIAL | F_FINAL if value != 0 => {
                    return Err("observed-text edge feature has a nonzero payload".into());
                }
                F_FIRST..=F_RIGHT2 if value > u32::MAX as u64 || value >= max_vocab as u64 => {
                    return Err("observed-text feature token is outside the vocabulary".into());
                }
                _ => {}
            }
        }
        if self.segment_weights.windows(2).any(|w| w[0].0 >= w[1].0)
            || self.token_elements.windows(2).any(|w| w[0].0 >= w[1].0)
        {
            return Err("segment weights and token elements must have unique sorted keys".into());
        }
        for (key, weights) in &self.segment_weights {
            let kind = key >> 40;
            let value = key & ((1u64 << 40) - 1);
            if !(G_FIRST..=G_H4_RIGHT).contains(&kind) || weights[SEG_BACKGROUND] != 0 {
                return Err("unknown segment feature or nonzero fixed-background weight".into());
            }
            match kind {
                G_FIRST..=G_RIGHT | G_INTERIOR
                    if value > u32::MAX as u64 || value >= max_vocab as u64 =>
                {
                    return Err("segment token outside vocabulary".into())
                }
                G_LEN if value == 0 || value > OB_MAX_SEG as u64 => {
                    return Err("segment length outside bound".into())
                }
                G_INITIAL | G_FINAL if value != 0 => {
                    return Err("segment edge feature has a payload".into())
                }
                G_H4_INTERVAL..=G_H4_RIGHT
                    if !self.use_h4 || value >= super::group_table::GROUP_ORDER as u64 =>
                {
                    return Err("invalid or disabled group feature".into())
                }
                _ => {}
            }
        }
        if self.token_elements.iter().any(|(token, element)| {
            *token as usize >= max_vocab || *element as usize >= super::group_table::GROUP_ORDER
        }) || (!self.use_h4 && !self.token_elements.is_empty())
        {
            return Err("invalid token/group map".into());
        }
        for role in 0..OB_N_ROLES {
            let action = self.action_per_role[role];
            if ![action_tag(RelAction::Emit), action_tag(RelAction::Continue)].contains(&action)
                || Goal::from_index(self.goal_per_role[role] as usize).is_none()
                || self.redirect_per_role[role] != (action == action_tag(RelAction::Continue))
            {
                return Err("invalid observed-text action/goal/redirect table".into());
            }
        }
        Ok(())
    }

    pub fn from_bytes(bytes: &[u8], max_vocab: usize) -> Result<Self, ObservedTextError> {
        let model: Self = serde_json::from_slice(bytes)
            .map_err(|e| ObservedTextError::Serialization(e.to_string()))?;
        model.validate(max_vocab)?;
        Ok(model)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ObservedTextError> {
        serde_json::to_vec(self).map_err(|e| ObservedTextError::Serialization(e.to_string()))
    }

    /// The one candidate scorer. Fitting and serving both call it through [`candidate_features`].
    pub fn candidate_scores(&self, tokens: &[u32], start: usize, len: usize) -> [i32; OB_N_ROLES] {
        let mut out = [0i32; OB_N_ROLES];
        for key in candidate_features(tokens, start, len) {
            if let Ok(i) = self.feature_weights.binary_search_by_key(&key, |(k, _)| *k) {
                let row = self.feature_weights[i].1;
                for r in 0..OB_N_ROLES {
                    out[r] = out[r].saturating_add(row[r]);
                }
            }
        }
        out
    }

    pub fn role_action(&self, role: usize) -> RelAction {
        self.action_per_role
            .get(role)
            .copied()
            .map(action_from_tag)
            .unwrap_or(RelAction::Unresolved)
    }

    pub fn role_goal(&self, role: usize) -> Option<Goal> {
        self.goal_per_role
            .get(role)
            .and_then(|g| Goal::from_index(*g as usize))
    }

    pub fn role_is_redirect(&self, role: usize) -> bool {
        self.redirect_per_role.get(role).copied().unwrap_or(false)
    }

    /// Learned per-token group element for the ordered interval features.
    pub fn token_element(&self, token: u32) -> usize {
        self.token_elements
            .binary_search_by_key(&token, |(t, _)| *t)
            .ok()
            .map(|i| self.token_elements[i].1 as usize)
            .unwrap_or_else(|| super::group_table::group_table().identity as usize)
    }

    /// Ordered group prefixes `P[j] = g(x_0) ... g(x_{j-1})` for one clause.
    fn prefix_products(&self, tokens: &[u32]) -> Vec<usize> {
        let t = super::group_table::group_table();
        let mut p = Vec::with_capacity(tokens.len() + 1);
        let mut acc = t.identity as usize;
        p.push(acc);
        for tok in tokens {
            acc = t.product[acc * super::group_table::ROW_STRIDE
                + self
                    .token_element(*tok)
                    .min(super::group_table::GROUP_ORDER - 1)] as usize;
            p.push(acc);
        }
        p
    }

    /// The one segment observation extractor, used by fitting and serving. It adds every interior
    /// token and, when the arm enables them, the exact ordered interval element and its relative
    /// relations to the neighbouring intervals.
    pub fn segment_features(
        &self,
        tokens: &[u32],
        start: usize,
        len: usize,
        prefix: &[usize],
    ) -> Vec<u64> {
        let n = tokens.len();
        let key = |kind: u64, value: u64| (kind << 40) | value;
        let mut f = Vec::with_capacity(12);
        let Some(end) = start
            .checked_add(len)
            .filter(|end| len > 0 && len <= OB_MAX_SEG && *end <= n)
        else {
            return f;
        };
        if self.use_h4
            && (prefix.len() != n + 1
                || prefix.iter().any(|p| *p >= super::group_table::GROUP_ORDER))
        {
            return f;
        }
        f.push(key(G_FIRST, u64::from(tokens[start])));
        f.push(key(G_LAST, u64::from(tokens[start + len - 1])));
        f.push(key(G_LEN, len as u64));
        if start > 0 {
            f.push(key(G_LEFT, u64::from(tokens[start - 1])));
        }
        if start + len < n {
            f.push(key(G_RIGHT, u64::from(tokens[start + len])));
        }
        for i in (start + 1)..(start + len).saturating_sub(1) {
            if i + 1 < start + len {
                f.push(key(G_INTERIOR, u64::from(tokens[i])));
            }
        }
        if start == 0 {
            f.push(key(G_INITIAL, 0));
        }
        if start + len == n {
            f.push(key(G_FINAL, 0));
        }
        if self.use_h4 {
            let Some(interval) = interval_product(prefix, start, end) else {
                return Vec::new();
            };
            f.push(key(G_H4_INTERVAL, interval as u64));
            if start == 0 {
                f.push(key(G_H4_LEFT, interval as u64));
            } else {
                let Some(prev_interval) = interval_product(prefix, start - 1, start) else {
                    return Vec::new();
                };
                f.push(key(
                    G_H4_LEFT,
                    relative_element(prev_interval, interval) as u64,
                ));
            }
            if start + len == n {
                f.push(key(G_H4_RIGHT, interval as u64));
            } else {
                let Some(next_interval) = interval_product(prefix, end, end + 1) else {
                    return Vec::new();
                };
                f.push(key(
                    G_H4_RIGHT,
                    relative_element(interval, next_interval) as u64,
                ));
            }
        }
        f
    }

    pub fn segment_score(
        &self,
        tokens: &[u32],
        start: usize,
        len: usize,
        role: usize,
        prefix: &[usize],
    ) -> i64 {
        if role == SEG_BACKGROUND || role >= OB_SEG_ROLES {
            return 0;
        }
        let mut acc = 0i64;
        for key in self.segment_features(tokens, start, len, prefix) {
            if let Ok(i) = self.segment_weights.binary_search_by_key(&key, |(k, _)| *k) {
                acc += i64::from(self.segment_weights[i].1[role]);
            }
        }
        acc
    }

    /// **Bounded structured decode.** Subject, cue and object spans may occur in any order and each
    /// role at most once; tokens not covered by a role are fixed zero-score background. The state is the set of
    /// placed roles, so the decoder no longer fixes the subject to a prefix or the object to a suffix
    /// and admits a trailing adjunct after the answer and an object that precedes its subject.
    pub fn decode_segmentation(&self, tokens: &[u32]) -> Option<Segmentation> {
        let n = tokens.len();
        if n == 0 || n > OB_MAX_CLAUSE {
            return None;
        }
        let prefix = self.prefix_products(tokens);
        // At most 24 segments, each with at most 32 i32-valued features: i64 is exact here.
        const NEG: i64 = i64::MIN / 4;
        let mut best = vec![[NEG; 8]; n + 1];
        let mut choice: Vec<[Option<(usize, u8, usize)>; 8]> = vec![[None; 8]; n + 1];
        best[0][0] = 0;
        // Bounded disjoint role assignment with a unit zero-score background transition. Coverage
        // alone does not prevent a spurious role; learned role potentials must distinguish it.
        for pos in 0..n {
            for mask in 0..8usize {
                let cur = best[pos][mask];
                if cur == NEG {
                    continue;
                }
                for len in 1..=(n - pos).min(OB_MAX_SEG) {
                    let end = pos + len;
                    if len == 1 && cur > best[end][mask] {
                        best[end][mask] = cur;
                        choice[end][mask] = Some((pos, 0, 1));
                    }
                    for (bit, role) in [(1u8, SEG_SUBJECT), (2u8, SEG_CUE), (4u8, SEG_OBJECT)] {
                        if mask & (bit as usize) != 0 || (role == SEG_CUE && len > OB_MAX_MARKER) {
                            continue;
                        }
                        let sc = self.segment_score(tokens, pos, len, role, &prefix);
                        let nm = mask | bit as usize;
                        if cur.saturating_add(sc) > best[end][nm] {
                            best[end][nm] = cur.saturating_add(sc);
                            choice[end][nm] = Some((pos, bit, len));
                        }
                    }
                }
            }
        }
        // A question states a subject and a cue; a statement adds an object.
        // Choose by score, not by preferring the richer parse: a question clause whose object
        // hypothesis scores no higher must stay a question.
        let statement = best[n][1 | 2 | 4];
        let question = best[n][1 | 2];
        let (mask, score) = if statement != NEG && statement > question {
            (1usize | 2 | 4, statement)
        } else if question != NEG {
            (1usize | 2, question)
        } else if statement != NEG {
            (1usize | 2 | 4, statement)
        } else {
            return None;
        };
        let mut spans: [Option<(usize, usize)>; 4] = [None; 4];
        let mut pos = n;
        let mut m = mask;
        while pos > 0 {
            let (prev, bit, len) = choice[pos][m]?;
            match bit {
                0 => {}
                1 => spans[SEG_SUBJECT] = Some((prev, len)),
                2 => spans[SEG_CUE] = Some((prev, len)),
                _ => spans[SEG_OBJECT] = Some((prev, len)),
            }
            m &= !(bit as usize);
            pos = prev;
        }
        let subject = spans[SEG_SUBJECT]?;
        let cue = spans[SEG_CUE]?;
        let object = spans[SEG_OBJECT];
        Some(Segmentation {
            subject,
            cue,
            object,
            score,
        })
    }

    /// The learned four-way cue role of an observed cue span, from the retained categorical scorer.
    pub fn cue_role(&self, tokens: &[u32], start: usize, len: usize) -> usize {
        let scores = self.candidate_scores(tokens, start, len);
        (0..OB_N_ROLES)
            .max_by_key(|r| (scores[*r], std::cmp::Reverse(*r)))
            .unwrap_or(0)
    }

    /// Locate the marker candidate with the best contextual score. Bounded contiguous candidates
    /// must leave a nonempty observed prefix, which is declared candidate scaffolding.
    pub fn locate_marker(&self, tokens: &[u32]) -> Option<(usize, usize, [i32; OB_N_ROLES])> {
        if tokens.len() > OB_MAX_CLAUSE || tokens.len() < 2 {
            return None;
        }
        let n = tokens.len();
        let mut best: Option<(i32, usize, usize, [i32; OB_N_ROLES])> = None;
        for start in 1..n {
            for len in 1..=(n - start).min(OB_MAX_MARKER) {
                let scores = self.candidate_scores(tokens, start, len);
                let score = *scores.iter().max().unwrap_or(&0);
                let better = match best {
                    None => true,
                    Some((bs, _, blen, _)) => score > bs || (score == bs && len < blen),
                };
                if better {
                    best = Some((score, start, len, scores));
                }
            }
        }
        best.map(|(_, start, len, v)| (start, len, v))
    }
}

/// The learned observation extractor: clause -> (subject, marker, object, role, question goal).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub seg: u32,
    /// Exact lexical key of the subject span; this is the identity used for joins.
    pub subject_key: Vec<u8>,
    /// Exact lexical key of the object span, when the clause states an object.
    pub object_key: Option<Vec<u8>>,
    pub subject: Vec<u32>,
    pub marker: Vec<u32>,
    pub object: Option<Vec<u32>>,
    pub subject_start: u32,
    pub marker_start: u32,
    pub object_start: Option<u32>,
    pub role: usize,
    /// Present when the clause is a question clause (the marker ends the clause).
    pub question_goal: Option<Goal>,
}

/// Observe one clause through the learned model only.
pub fn observe_clause(
    model: &ObservedTextModel,
    clause: &Clause,
) -> Result<Observation, ObservedTextError> {
    observe_clause_inner(model, clause).map_err(ObservedTextError::Observation)
}

fn observe_clause_inner(model: &ObservedTextModel, clause: &Clause) -> Result<Observation, String> {
    if clause.tokens.is_empty() || clause.tokens.len() > OB_MAX_CLAUSE {
        return Err("clause outside the declared token bound".into());
    }
    validate_alignment(clause)?;
    let seg = model
        .decode_segmentation(&clause.tokens)
        .ok_or("no legal structured span assignment in clause")?;
    let subject = clause.tokens[seg.subject.0..seg.subject.0 + seg.subject.1].to_vec();
    let role = model.cue_role(&clause.tokens, seg.cue.0, seg.cue.1);
    let object = seg.object.map(|(s, l)| clause.tokens[s..s + l].to_vec());
    let question_goal = if object.is_none() {
        model.role_goal(role)
    } else {
        None
    };
    Ok(Observation {
        seg: clause.seg,
        subject_key: identity_key(clause, seg.subject.0, seg.subject.1)?,
        object_key: seg
            .object
            .map(|(s, l)| identity_key(clause, s, l))
            .transpose()?,
        subject_start: seg.subject.0 as u32,
        marker_start: seg.cue.0 as u32,
        object_start: seg.object.map(|(s, _)| s as u32),
        subject,
        marker: clause.tokens[seg.cue.0..seg.cue.0 + seg.cue.1].to_vec(),
        object,
        role,
        question_goal,
    })
}

/// Immutable identities prepared from the actual loaded bytes and observed document once.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextBinding {
    pub model_sha256: String,
    pub tokenizer_sha256: String,
    pub world_id: u32,
    pub world_version: u32,
    pub doc_sha256: String,
    pub control_sha256: String,
    pub max_vocab: usize,
    pub eos: Option<u32>,
}

/// Declared experimental controls are immutable runtime inputs and part of snapshot identity.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TextControl {
    #[default]
    Normal,
    ReadsDisabled,
    Membership {
        entities: Vec<Vec<u32>>,
    },
    ReadCap {
        max_reads: u8,
    },
}

/// Exact source provenance for an owned phrase, including document identity and full extent.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextCapture {
    /// Exact lexical key of the captured object span: the identity later joins are made on.
    pub key: Vec<u8>,
    pub world_id: u32,
    pub world_version: u32,
    pub doc_sha256: String,
    pub segment: u32,
    pub start: u32,
    pub len: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextObjective {
    pub goal: Goal,
    /// Exact lexical key of the current query argument; joins use this, not the token form.
    pub key: Vec<u8>,
    /// Surface tokens of the current query argument, kept for provenance and emission.
    pub tokens: Vec<u32>,
}

/// A versioned owned session, containing every control and emission position needed to resume.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextSession {
    pub version: u8,
    pub binding: TextBinding,
    pub goal: Goal,
    pub query: TextObjective,
    pub captured: Option<TextCapture>,
    pub captured_span: Option<(u32, u32, usize)>,
    pub captured_tokens: Vec<u32>,
    pub pending: RelAction,
    pub reads: u8,
    pub emitted: Vec<u32>,
    pub cursor: usize,
    pub visited: Vec<(u32, u32, usize)>,
    pub terminal: Option<RelAction>,
    /// This typed terminator is separate from ordinary vocabulary tokens.
    pub eos: Option<u32>,
}

/// Performed action and its next phase are separate fields; a Read must be recorded as a Read.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StepEffect {
    pub action: RelAction,
    pub next_action: Option<RelAction>,
    pub selected_segment: Option<u32>,
    pub selected_role: Option<usize>,
    pub emitted: Option<u32>,
    pub terminal: Option<RelAction>,
}

impl TextSession {
    fn start(binding: TextBinding, goal: Goal, key: Vec<u8>, tokens: Vec<u32>) -> Self {
        Self {
            version: 3,
            eos: binding.eos,
            binding,
            goal,
            query: TextObjective { goal, key, tokens },
            captured: None,
            captured_span: None,
            captured_tokens: Vec::new(),
            pending: RelAction::Read,
            reads: 0,
            emitted: Vec::new(),
            cursor: 0,
            visited: Vec::new(),
            terminal: None,
        }
    }

    /// Structural checks complement the runtime's verification against the bound observed source.
    pub fn validate(
        &self,
        expected: &TextBinding,
        max_vocab: usize,
    ) -> Result<(), ObservedTextError> {
        self.validate_structure(expected, max_vocab)
            .map_err(ObservedTextError::Session)
    }

    fn validate_structure(&self, expected: &TextBinding, max_vocab: usize) -> Result<(), String> {
        if self.version != 3
            || &self.binding != expected
            || max_vocab != expected.max_vocab
            || self.eos != expected.eos
        {
            return Err("session version or runtime binding mismatch".into());
        }
        if self.goal != self.query.goal
            || self.query.tokens.is_empty()
            || self.query.tokens.len() > OB_MAX_CLAUSE
            || !valid_identity_key(&self.query.key)
        {
            return Err("inconsistent observed request objective".into());
        }
        if self
            .query
            .tokens
            .iter()
            .chain(self.captured_tokens.iter())
            .any(|t| *t as usize >= max_vocab)
        {
            return Err("ordinary session token outside vocabulary".into());
        }
        if self.captured_tokens.len() > OB_MAX_CLAUSE
            || self.cursor > self.captured_tokens.len()
            || self.reads > OB_MAX_READS
            || self.visited.len() != self.reads as usize
            || self.visited.iter().copied().collect::<BTreeSet<_>>().len() != self.visited.len()
        {
            return Err("invalid session span, cursor or read history".into());
        }
        match &self.captured {
            Some(capture) => {
                if capture.world_id != expected.world_id
                    || capture.world_version != expected.world_version
                    || capture.doc_sha256 != expected.doc_sha256
                    || !valid_identity_key(&capture.key)
                    || capture.len == 0
                    || capture.len != self.captured_tokens.len()
                    || self.captured_span != Some((capture.segment, capture.start, capture.len))
                    || self.visited.last().copied() != self.captured_span
                {
                    return Err("inconsistent captured phrase provenance".into());
                }
            }
            None => {
                if self.captured_span.is_some()
                    || !self.captured_tokens.is_empty()
                    || self.reads != 0
                    || self.cursor != 0
                {
                    return Err("session has content without an owned capture".into());
                }
            }
        }
        let successful_stop = self.terminal == Some(RelAction::Stop);
        let terminators = usize::from(successful_stop && self.eos.is_some());
        if self.emitted.len() != self.cursor + terminators
            || self.emitted.get(..self.cursor) != self.captured_tokens.get(..self.cursor)
            || (terminators == 1 && self.emitted.last().copied() != self.eos)
        {
            return Err(
                "emissions are not the exact captured prefix and declared terminator".into(),
            );
        }
        if self.terminal.is_some() {
            if self.pending != RelAction::Stop
                || !matches!(
                    self.terminal,
                    Some(RelAction::Stop | RelAction::Unresolved | RelAction::Exhausted)
                )
            {
                return Err("invalid terminal session phase".into());
            }
            if successful_stop
                && (self.captured.is_none()
                    || self.cursor != self.captured_tokens.len()
                    || self.cursor == 0)
            {
                return Err("successful Stop requires a complete nonempty owned answer".into());
            }
        } else {
            match self.pending {
                RelAction::Read if self.cursor == 0 && self.emitted.is_empty() => {}
                RelAction::Continue
                    if self.captured.is_some() && self.cursor == 0 && self.emitted.is_empty() => {}
                RelAction::Emit
                    if self.captured.is_some() && self.cursor < self.captured_tokens.len() => {}
                RelAction::Stop
                    if self.captured.is_some()
                        && self.cursor > 0
                        && self.cursor == self.captured_tokens.len() => {}
                _ => return Err("invalid active session phase".into()),
            }
        }
        Ok(())
    }

    fn terminate(&mut self, reason: RelAction) {
        self.terminal = Some(reason);
        self.pending = RelAction::Stop;
    }
}

/// A loaded, source-bound runtime. The primary receives no entity registry or gold goal.
/// Documents are immutable here: an explicit new document/version creates a new runtime identity.
pub struct ObservedTextRuntime {
    model: ObservedTextModel,
    clauses: Vec<Clause>,
    observations: Vec<Observation>,
    binding: TextBinding,
    control: TextControl,
}

impl ObservedTextRuntime {
    pub fn load(
        model_bytes: &[u8],
        tokenizer_sha256: &str,
        clauses: Vec<Clause>,
        world_id: u32,
        world_version: u32,
        max_vocab: usize,
        eos: Option<u32>,
    ) -> Result<Self, ObservedTextError> {
        let model = ObservedTextModel::from_bytes(model_bytes, max_vocab)?;
        if tokenizer_sha256.len() != 64 || !tokenizer_sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(ObservedTextError::Session(
                "tokenizer identity must be an actual SHA-256 digest".into(),
            ));
        }
        let mut seen = BTreeSet::new();
        for clause in &clauses {
            if !seen.insert(clause.seg)
                || clause.tokens.is_empty()
                || clause.tokens.len() > OB_MAX_CLAUSE
                || clause.tokens.iter().any(|t| *t as usize >= max_vocab)
            {
                return Err(ObservedTextError::Observation(
                    "document has duplicate segments or invalid observed tokens".into(),
                ));
            }
        }
        let observations = clauses
            .iter()
            .map(|clause| observe_clause(&model, clause))
            .collect::<Result<Vec<_>, _>>()?;
        let binding = TextBinding {
            model_sha256: sha256_hex(model_bytes),
            tokenizer_sha256: tokenizer_sha256.to_owned(),
            world_id,
            world_version,
            doc_sha256: sha256_hex(
                &serde_json::to_vec(&clauses)
                    .map_err(|e| ObservedTextError::Serialization(e.to_string()))?,
            ),
            control_sha256: sha256_hex(
                &serde_json::to_vec(&TextControl::Normal)
                    .map_err(|e| ObservedTextError::Serialization(e.to_string()))?,
            ),
            max_vocab,
            eos,
        };
        Ok(Self {
            model,
            clauses,
            observations,
            binding,
            control: TextControl::Normal,
        })
    }

    pub fn with_control(mut self, control: TextControl) -> Result<Self, ObservedTextError> {
        match &control {
            TextControl::Membership { entities }
                if entities.iter().any(|e| {
                    e.is_empty()
                        || e.len() > OB_MAX_CLAUSE
                        || e.iter().any(|t| *t as usize >= self.binding.max_vocab)
                }) =>
            {
                return Err(ObservedTextError::Observation(
                    "invalid membership-control input".into(),
                ))
            }
            TextControl::ReadCap { max_reads } if *max_reads == 0 || *max_reads > OB_MAX_READS => {
                return Err(ObservedTextError::Session(
                    "invalid read-cap control".into(),
                ))
            }
            _ => {}
        }
        self.binding.control_sha256 = sha256_hex(
            &serde_json::to_vec(&control)
                .map_err(|e| ObservedTextError::Serialization(e.to_string()))?,
        );
        self.control = control;
        Ok(self)
    }

    pub fn model(&self) -> &ObservedTextModel {
        &self.model
    }
    pub fn binding(&self) -> &TextBinding {
        &self.binding
    }
    pub fn observations(&self) -> &[Observation] {
        &self.observations
    }
    pub fn clauses(&self) -> &[Clause] {
        &self.clauses
    }

    /// Parse only the actually observed question; desired goal/answer remains outside this API.
    pub fn start(&self, question: &Clause) -> Result<TextSession, ObservedTextError> {
        if question
            .tokens
            .iter()
            .any(|t| *t as usize >= self.binding.max_vocab)
        {
            return Err(ObservedTextError::Observation(
                "question contains an out-of-vocabulary token".into(),
            ));
        }
        let observed = observe_clause(&self.model, question)?;
        let goal = observed.question_goal.ok_or_else(|| {
            ObservedTextError::Observation("observed clause is not a recognized question".into())
        })?;
        let frame = TextSession::start(
            self.binding.clone(),
            goal,
            observed.subject_key.clone(),
            observed.subject.clone(),
        );
        self.validate(&frame)?;
        Ok(frame)
    }

    pub fn origin_is_live(&self, session: &TextSession) -> bool {
        let Some(c) = &session.captured else {
            return false;
        };
        if c.world_id != self.binding.world_id
            || c.world_version != self.binding.world_version
            || c.doc_sha256 != self.binding.doc_sha256
        {
            return false;
        }
        let Some(clause) = self.clauses.iter().find(|clause| clause.seg == c.segment) else {
            return false;
        };
        (c.start as usize)
            .checked_add(c.len)
            .and_then(|end| clause.tokens.get(c.start as usize..end))
            == Some(session.captured_tokens.as_slice())
    }

    pub fn validate(&self, session: &TextSession) -> Result<(), ObservedTextError> {
        self.validate_session(session)
            .map_err(ObservedTextError::Session)
    }

    fn validate_session(&self, session: &TextSession) -> Result<(), String> {
        session.validate(&self.binding, self.binding.max_vocab)?;
        // Every retained source reference denotes an actually parsed object for the active goal.
        // A substring that happens to match raw document bytes is not sufficient provenance.
        let mut history = Vec::with_capacity(session.visited.len());
        for &(segment, start, len) in &session.visited {
            let observed = self
                .observations
                .iter()
                .find(|observed| {
                    observed.seg == segment
                        && observed.object_start == Some(start)
                        && observed
                            .object
                            .as_ref()
                            .is_some_and(|tokens| tokens.len() == len)
                        && self.model.role_goal(observed.role) == Some(session.goal)
                })
                .ok_or("visited reference is not an observed object for the active goal")?;
            history.push(observed);
        }
        if session.captured.is_some() && !self.origin_is_live(session) {
            return Err("captured phrase disagrees with its bound source occurrence".into());
        }
        if matches!(self.control, TextControl::ReadsDisabled) && !history.is_empty() {
            return Err("read-disabled session contains a read history".into());
        }
        if let TextControl::ReadCap { max_reads } = self.control {
            if session.reads > max_reads {
                return Err("session history exceeds its bound read control".into());
            }
        }
        // The original question is not retained: these checks establish internal chain and phase
        // consistency, not authentication of an external request or proof of ranking optimality.
        for pair in history.windows(2) {
            if self.action_after_read(pair[0])? != RelAction::Continue
                || pair[0].object_key.as_ref() != Some(&pair[1].subject_key)
            {
                return Err("visited objects do not form a dependent continuation chain".into());
            }
        }
        if let Some(last) = history.last() {
            if session.captured.as_ref().map(|capture| &capture.key) != last.object_key.as_ref() {
                return Err("captured lexical identity disagrees with its bound source".into());
            }
            let action = self.action_after_read(last)?;
            let followed = session.pending == RelAction::Read
                || matches!(
                    session.terminal,
                    Some(RelAction::Unresolved | RelAction::Exhausted)
                );
            if followed {
                if action != RelAction::Continue
                    || last.object_key.as_ref() != Some(&session.query.key)
                    || last.object.as_ref() != Some(&session.query.tokens)
                {
                    return Err("followed session does not preserve its dependent query".into());
                }
            } else {
                let expected_action = if session.pending == RelAction::Continue {
                    RelAction::Continue
                } else {
                    RelAction::Emit
                };
                if action != expected_action || last.subject_key != session.query.key {
                    return Err(
                        "session phase disagrees with its captured source role/control".into(),
                    );
                }
            }
        }
        Ok(())
    }

    /// The fixed control and the learned role table determine the next phase in both step and
    /// restoration validation. The membership registry is available only to that named control.
    fn action_after_read(&self, observed: &Observation) -> Result<RelAction, String> {
        let object = observed
            .object
            .as_ref()
            .ok_or("selected fact lacks an object")?;
        Ok(match &self.control {
            TextControl::Membership { entities } => {
                if entities.contains(object) {
                    RelAction::Continue
                } else {
                    RelAction::Emit
                }
            }
            _ => self.model.role_action(observed.role),
        })
    }

    pub fn snapshot(&self, session: &TextSession) -> Result<Vec<u8>, ObservedTextError> {
        self.validate(session)?;
        serde_json::to_vec(session).map_err(|e| ObservedTextError::Serialization(e.to_string()))
    }

    pub fn restore(&self, bytes: &[u8]) -> Result<TextSession, ObservedTextError> {
        let session: TextSession = serde_json::from_slice(bytes)
            .map_err(|e| ObservedTextError::Serialization(e.to_string()))?;
        self.validate(&session)?;
        Ok(session)
    }

    pub fn step(&self, session: &mut TextSession) -> Result<StepEffect, ObservedTextError> {
        self.step_inner(session).map_err(ObservedTextError::Session)
    }

    fn step_inner(&self, session: &mut TextSession) -> Result<StepEffect, String> {
        self.validate(session)?;
        if session.terminal.is_some() {
            return Err("session already terminal".into());
        }
        let performed = session.pending;
        let mut effect = StepEffect {
            action: performed,
            next_action: None,
            selected_segment: None,
            selected_role: None,
            emitted: None,
            terminal: None,
        };
        match performed {
            RelAction::Read => {
                let cap = match self.control {
                    TextControl::ReadCap { max_reads } => max_reads,
                    _ => OB_MAX_READS,
                };
                if matches!(self.control, TextControl::ReadsDisabled) {
                    effect.action = RelAction::Unresolved;
                    session.terminate(RelAction::Unresolved);
                } else if session.reads >= cap {
                    effect.action = RelAction::Exhausted;
                    session.terminate(RelAction::Exhausted);
                } else {
                    let mut best: Option<(i32, usize)> = None;
                    for (i, observed) in self.observations.iter().enumerate() {
                        if observed.subject_key != session.query.key
                            || self.model.role_goal(observed.role) != Some(session.goal)
                            || observed.object.is_none()
                        {
                            continue;
                        }
                        // Rank admitted statements by their retained categorical cue-role score.
                        // Argument selection is a separate structured span objective.
                        let clause = self
                            .clauses
                            .iter()
                            .find(|c| c.seg == observed.seg)
                            .ok_or("observed statement lost its clause")?;
                        let scores = self.model.candidate_scores(
                            &clause.tokens,
                            observed.marker_start as usize,
                            observed.marker.len(),
                        );
                        let score = scores[observed.role.min(OB_N_ROLES - 1)];
                        if best.is_none_or(|(old, _)| score > old) {
                            best = Some((score, i));
                        }
                    }
                    if let Some((_, idx)) = best {
                        let observed = &self.observations[idx];
                        let object = observed
                            .object
                            .as_ref()
                            .ok_or("selected fact lacks an object")?;
                        let start = observed
                            .object_start
                            .ok_or("selected fact lacks an extent")?;
                        let span = (observed.seg, start, object.len());
                        effect.selected_segment = Some(observed.seg);
                        effect.selected_role = Some(observed.role);
                        if session.visited.contains(&span) {
                            session.terminate(RelAction::Exhausted);
                        } else {
                            session.visited.push(span);
                            session.reads += 1;
                            session.captured = Some(TextCapture {
                                key: observed.object_key.clone().unwrap_or_default(),
                                world_id: self.binding.world_id,
                                world_version: self.binding.world_version,
                                doc_sha256: self.binding.doc_sha256.clone(),
                                segment: observed.seg,
                                start,
                                len: object.len(),
                            });
                            session.captured_span = Some(span);
                            session.captured_tokens = object.clone();
                            session.pending = self.action_after_read(observed)?;
                        }
                    } else {
                        session.terminate(RelAction::Unresolved);
                    }
                }
            }
            RelAction::Continue => {
                session.query = TextObjective {
                    goal: session.goal,
                    key: session
                        .captured
                        .as_ref()
                        .map(|c| c.key.clone())
                        .unwrap_or_default(),
                    tokens: session.captured_tokens.clone(),
                };
                session.pending = RelAction::Read;
            }
            RelAction::Emit => {
                let token = *session
                    .captured_tokens
                    .get(session.cursor)
                    .ok_or("emission cursor past captured phrase")?;
                session.emitted.push(token);
                session.cursor += 1;
                effect.emitted = Some(token);
                if session.cursor == session.captured_tokens.len() {
                    session.pending = RelAction::Stop;
                }
            }
            RelAction::Stop => {
                // Stop is legal only after the full phrase; the typed EOS is appended exactly once.
                if let Some(eos) = session.eos {
                    session.emitted.push(eos);
                    effect.emitted = Some(eos);
                }
                session.terminate(RelAction::Stop);
            }
            _ => return Err("unsupported active observed-text phase".into()),
        }
        self.validate(session)?;
        effect.next_action = if session.terminal.is_none() {
            Some(session.pending)
        } else {
            None
        };
        effect.terminal = session.terminal;
        Ok(effect)
    }

    /// At most six reads, five continuation steps, 24 answer tokens and one Stop are possible.
    pub fn run(&self, session: &mut TextSession) -> Result<Vec<StepEffect>, ObservedTextError> {
        self.run_inner(session).map_err(ObservedTextError::Session)
    }

    fn run_inner(&self, session: &mut TextSession) -> Result<Vec<StepEffect>, String> {
        self.validate(session)?;
        let mut effects = Vec::new();
        for _ in 0..(2 * OB_MAX_READS as usize + OB_MAX_CLAUSE + 2) {
            if session.terminal.is_some() {
                return Ok(effects);
            }
            effects.push(self.step(session)?);
        }
        if session.terminal.is_none() {
            session.terminate(RelAction::Exhausted);
            self.validate(session)?;
            effects.push(StepEffect {
                action: RelAction::Exhausted,
                next_action: None,
                selected_segment: None,
                selected_role: None,
                emitted: None,
                terminal: session.terminal,
            });
        }
        Ok(effects)
    }
}

/// Receipt for the observation-model fit.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObFitReport {
    pub clauses: usize,
    pub feature_weights: usize,
    /// Exact cue extent and cue-role matches on the same clauses (separate from all-span accuracy).
    pub cue_role_correct: usize,
    pub h4_enabled: bool,
    pub h4_moves: usize,
    pub candidate_updates: usize,
    pub epochs_run: usize,
    /// Same-unit exact subject/cue/object extents with the uninformed start and after fitting.
    /// The historical field names are retained; cue-role accuracy is reported separately.
    pub role_initial_correct: usize,
    pub role_final_correct: usize,
    /// Role-action table accounting on the same gold statement-action units as before.
    pub policy_initial_correct: usize,
    pub policy_final_correct: usize,
    pub policy_examples: usize,
    pub roles_observed: Vec<usize>,
}

/// Fit the observation model from declared development clauses and their gold labels.
///
/// Supervision: the declared marker extent and role are the gold candidate. Candidate features come
/// from the **same** extractor serving uses, so no gold field can become an input feature. The learner
/// is a bounded candidate perceptron; the declared uninformed start has no weights at all.
pub fn fit_observed_text_model(
    clauses: &[Clause],
    labels: &[ClauseLabel],
    use_h4: bool,
) -> Result<(ObservedTextModel, ObFitReport), ObservedTextError> {
    fit_observed_text_model_inner(clauses, labels, use_h4).map_err(ObservedTextError::Supervision)
}

fn fit_observed_text_model_inner(
    clauses: &[Clause],
    labels: &[ClauseLabel],
    use_h4: bool,
) -> Result<(ObservedTextModel, ObFitReport), String> {
    let mut segments = BTreeSet::new();
    for clause in clauses {
        if !segments.insert(clause.seg)
            || clause.tokens.is_empty()
            || clause.tokens.len() > OB_MAX_CLAUSE
        {
            return Err("duplicate segment or invalid clause bound in fitting".into());
        }
    }
    let mut labeled = BTreeSet::new();
    let mut semantics: BTreeMap<usize, RelAction> = BTreeMap::new();
    let mut role_goals: BTreeMap<usize, Goal> = BTreeMap::new();
    for clause in clauses {
        validate_alignment(clause)?;
    }
    for label in labels {
        if !labeled.insert(label.seg) || label.role >= OB_N_ROLES {
            return Err("duplicate supervision or out-of-range role".into());
        }
        let clause = clauses
            .iter()
            .find(|c| c.seg == label.seg)
            .ok_or("label has no observed clause")?;
        let within = |span: (u32, usize)| {
            span.1 > 0
                && (span.0 as usize)
                    .checked_add(span.1)
                    .is_some_and(|end| end <= clause.tokens.len())
        };
        if !within(label.subject) || !within(label.marker) {
            return Err("declared span is outside its clause".into());
        }
        if let Some(object) = label.object {
            if !within(object) {
                return Err("declared object is outside its clause".into());
            }
        }
        let mut extents = vec![label.subject, label.marker];
        extents.extend(label.object);
        extents.sort_by_key(|span| span.0);
        if label.marker.1 > OB_MAX_MARKER
            || extents
                .windows(2)
                .any(|pair| pair[0].0 as usize + pair[0].1 > pair[1].0 as usize)
        {
            return Err("gold spans overlap or cue exceeds its feature bound".into());
        }
        if !matches!(label.action, RelAction::Emit | RelAction::Continue) {
            return Err("supervised role action must be Emit or Continue".into());
        }
        if role_goals
            .insert(label.role, label.goal)
            .is_some_and(|prior| prior != label.goal)
        {
            return Err("one observed role has conflicting supervised goals".into());
        }
        if label.object.is_some()
            && semantics
                .insert(label.role, label.action)
                .is_some_and(|prior| prior != label.action)
        {
            return Err("one observed role has conflicting supervised actions".into());
        }
    }
    // The declared gold assignment per clause: exact subject, cue and optional object extents.
    let gold_of = |label: &ClauseLabel| -> Segmentation {
        Segmentation {
            subject: (label.subject.0 as usize, label.subject.1),
            cue: (label.marker.0 as usize, label.marker.1),
            object: label.object.map(|(s, l)| (s as usize, l)),
            score: 0,
        }
    };
    let exact = |model: &ObservedTextModel| -> usize {
        let mut hits = 0usize;
        for label in labels {
            let Some(clause) = clauses.iter().find(|c| c.seg == label.seg) else {
                continue;
            };
            let gold = gold_of(label);
            // Compare the assigned spans only: the decoded score is a real-valued potential, not a
            // label, so including it would make every exact span assignment unequal.
            if model
                .decode_segmentation(&clause.tokens)
                .is_some_and(|pred| {
                    pred.subject == gold.subject
                        && pred.cue == gold.cue
                        && pred.object == gold.object
                })
            {
                hits += 1;
            }
        }
        hits
    };
    let mut model = ObservedTextModel::uninformed();
    model.use_h4 = use_h4;
    if use_h4 {
        let vocab: BTreeSet<u32> = clauses
            .iter()
            .flat_map(|c| c.tokens.iter().copied())
            .collect();
        // Deterministic initial codes are installed before any feature weights are fitted. They
        // carry no presumed semantics; accepted coordinate moves must improve the same decoder.
        model.token_elements = vocab
            .into_iter()
            .map(|t| (t, (t as usize % super::group_table::GROUP_ORDER) as u8))
            .collect();
    }
    let initial_correct = exact(&model);
    // ---- structured perceptron over the bounded span assignment ----
    let epochs = 6usize;
    let mut updates = 0usize;
    let mut epochs_run = 0usize;
    for _epoch in 0..epochs {
        epochs_run += 1;
        let mut changed = false;

        for label in labels {
            let clause = clauses
                .iter()
                .find(|c| c.seg == label.seg)
                .ok_or("label lost its clause")?;
            let gold = gold_of(label);
            let pred = model.decode_segmentation(&clause.tokens);
            let same = pred.as_ref().is_some_and(|p| {
                p.subject == gold.subject && p.cue == gold.cue && p.object == gold.object
            });
            if same {
                continue;
            }
            let prefix = model.prefix_products(&clause.tokens);
            let apply_seg = |m: &mut ObservedTextModel, seg: &Segmentation, delta: i32| {
                let spans: Vec<((usize, usize), usize)> =
                    [(seg.subject, SEG_SUBJECT), (seg.cue, SEG_CUE)]
                        .into_iter()
                        .chain(seg.object.map(|o| (o, SEG_OBJECT)))
                        .collect();
                for (span, role) in spans {
                    for key in m.segment_features(&clause.tokens, span.0, span.1, &prefix) {
                        match m.segment_weights.binary_search_by_key(&key, |(k, _)| *k) {
                            Ok(i) => {
                                m.segment_weights[i].1[role] =
                                    m.segment_weights[i].1[role].saturating_add(delta);
                            }
                            Err(i) => {
                                let mut w = [0i32; OB_SEG_ROLES];
                                w[role] = delta;
                                m.segment_weights.insert(i, (key, w));
                            }
                        }
                    }
                }
            };
            apply_seg(&mut model, &gold, 1);
            if let Some(p) = pred.as_ref() {
                apply_seg(&mut model, p, -1);
            }
            updates += 1;
            changed = true;
        }
        if !changed {
            break;
        }
    }
    // ---- ordered H4 interval elements, fitted by a bounded coordinate search ----
    let mut h4_moves = 0usize;
    if use_h4 {
        let base = exact(&model);
        let mut best = base;
        for _pass in 0..2 {
            let mut improved = false;
            let tokens_in_order: Vec<u32> = model.token_elements.iter().map(|(t, _)| *t).collect();
            for (idx, _token) in tokens_in_order.iter().enumerate() {
                let saved = model.token_elements[idx].1;
                let mut best_val = saved;
                let mut best_score = best;
                for cand in 0..super::group_table::GROUP_ORDER {
                    model.token_elements[idx].1 = cand as u8;
                    let sc = exact(&model);
                    if sc > best_score {
                        best_score = sc;
                        best_val = cand as u8;
                    }
                }
                model.token_elements[idx].1 = best_val;
                if best_score > best {
                    best = best_score;
                    h4_moves += 1;
                    improved = true;
                }
            }
            if !improved {
                break;
            }
        }
    }
    // ---- supervised categorical cue-role scoring on observed gold cue spans ----
    // Span fitting and role classification are separate objectives. Multiple passes remove the
    // last-example bias of the previous single pass, without using gold spans at serving.
    for _epoch in 0..8 {
        let mut changed = false;
        for label in labels {
            let clause = clauses
                .iter()
                .find(|c| c.seg == label.seg)
                .ok_or("label lost its clause")?;
            let (cs, cl) = label.marker;
            let gold = (cs as usize, cl, label.role);
            let pred = Some((
                gold.0,
                gold.1,
                model.cue_role(&clause.tokens, gold.0, gold.1),
            ));
            if pred == Some(gold) {
                continue;
            }
            changed = true;
            for key in candidate_features(&clause.tokens, gold.0, gold.1) {
                match model
                    .feature_weights
                    .binary_search_by_key(&key, |(k, _)| *k)
                {
                    Ok(i) => {
                        model.feature_weights[i].1[gold.2] =
                            model.feature_weights[i].1[gold.2].saturating_add(1)
                    }
                    Err(i) => {
                        let mut w = [0i32; OB_N_ROLES];
                        w[gold.2] = 1;
                        model.feature_weights.insert(i, (key, w));
                    }
                }
            }
            if let Some((ps, pl, pr)) = pred {
                for key in candidate_features(&clause.tokens, ps, pl) {
                    if let Ok(i) = model
                        .feature_weights
                        .binary_search_by_key(&key, |(k, _)| *k)
                    {
                        model.feature_weights[i].1[pr] =
                            model.feature_weights[i].1[pr].saturating_sub(1);
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    let cue_hits = labels
        .iter()
        .filter(|label| {
            let Some(clause) = clauses.iter().find(|c| c.seg == label.seg) else {
                return false;
            };
            model.decode_segmentation(&clause.tokens).is_some_and(|s| {
                s.cue == (label.marker.0 as usize, label.marker.1)
                    && model.cue_role(&clause.tokens, s.cue.0, s.cue.1) == label.role
            })
        })
        .count();
    // ---- role semantics from the declared labels ----
    for (&role, &goal) in &role_goals {
        model.goal_per_role[role] = goal.index() as u8;
    }
    for (&role, &action) in &semantics {
        model.action_per_role[role] = action_tag(action);
        model.redirect_per_role[role] = action == RelAction::Continue;
    }
    model.validate(usize::MAX).map_err(|e| e.to_string())?;
    let final_correct = exact(&model);
    let mut initial_action = 0usize;
    let mut policy_final_correct = 0usize;
    let mut examples = 0usize;
    for label in labels {
        if label.object.is_none() {
            continue;
        }
        examples += 1;
        let want = label.action;
        if want == RelAction::Emit {
            initial_action += 1;
        }
        if model.role_action(label.role) == want {
            policy_final_correct += 1;
        }
    }
    let report = ObFitReport {
        clauses: clauses.len(),
        feature_weights: model.segment_weights.len(),
        candidate_updates: updates,
        epochs_run,
        // Initial and final exact bounded span assignment, on the same clauses.
        role_initial_correct: initial_correct,
        role_final_correct: final_correct,
        policy_initial_correct: initial_action,
        policy_final_correct,
        policy_examples: examples,
        roles_observed: role_goals.keys().copied().collect(),
        cue_role_correct: cue_hits,
        h4_enabled: use_h4,
        h4_moves,
    };
    Ok((model, report))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Declared development text for the test: two goals, an assertion and a redirect each.
    fn dev() -> (Vec<Clause>, Vec<ClauseLabel>) {
        // tokens: subject marker object
        let clauses = vec![
            Clause::of(0, vec![10, 20, 21, 30]), // Mara works-in Cedar
            Clause::of(1, vec![10, 22, 23, 11]), // Mara office-follows Ivo
            Clause::of(2, vec![10, 24, 25, 31]), // Mara is-on-project Atlas
            Clause::of(3, vec![10, 26, 27, 12]), // Mara project-follows Nila
            Clause::of(4, vec![10, 20, 21]),     // Mara works-in ?
            Clause::of(5, vec![10, 24, 25]),     // Mara is-on-project ?
            Clause::of(6, vec![11, 20, 21, 32]), // Ivo works-in Fen
        ];
        let labels = vec![
            ClauseLabel {
                seg: 0,
                subject: (0, 1),
                marker: (1, 2),
                object: Some((3, 1)),
                role: 0,
                goal: Goal::Office,
                action: RelAction::Emit,
            },
            ClauseLabel {
                seg: 1,
                subject: (0, 1),
                marker: (1, 2),
                object: Some((3, 1)),
                role: 1,
                goal: Goal::Office,
                action: RelAction::Continue,
            },
            ClauseLabel {
                seg: 2,
                subject: (0, 1),
                marker: (1, 2),
                object: Some((3, 1)),
                role: 2,
                goal: Goal::Project,
                action: RelAction::Emit,
            },
            ClauseLabel {
                seg: 3,
                subject: (0, 1),
                marker: (1, 2),
                object: Some((3, 1)),
                role: 3,
                goal: Goal::Project,
                action: RelAction::Continue,
            },
            ClauseLabel {
                seg: 4,
                subject: (0, 1),
                marker: (1, 2),
                object: None,
                role: 0,
                goal: Goal::Office,
                action: RelAction::Emit,
            },
            ClauseLabel {
                seg: 5,
                subject: (0, 1),
                marker: (1, 2),
                object: None,
                role: 2,
                goal: Goal::Project,
                action: RelAction::Emit,
            },
            ClauseLabel {
                seg: 6,
                subject: (0, 1),
                marker: (1, 2),
                object: Some((3, 1)),
                role: 0,
                goal: Goal::Office,
                action: RelAction::Emit,
            },
        ];
        (clauses, labels)
    }

    #[test]
    fn the_extractor_locates_roles_and_goals_without_gold_fields() {
        let (clauses, labels) = dev();
        let (model, report) = fit_observed_text_model(&clauses, &labels, false).unwrap();
        assert_eq!(report.policy_final_correct, report.policy_examples);
        assert!(report.policy_final_correct > report.policy_initial_correct);
        for (clause, label) in clauses.iter().zip(labels.iter()) {
            let obs = observe_clause(&model, clause).expect("clause must be observable");
            assert_eq!(obs.role, label.role, "seg {}", clause.seg);
            let (ss, sl) = label.subject;
            let expected_subject = clause.tokens[ss as usize..(ss as usize + sl)].to_vec();
            assert_eq!(obs.subject, expected_subject, "seg {}", clause.seg);
            if label.object.is_some() {
                assert!(obs.object.is_some());
                assert!(obs.question_goal.is_none());
            } else {
                assert!(obs.object.is_none());
                assert_eq!(obs.question_goal, Some(label.goal));
            }
        }
    }

    #[test]
    fn fit_reports_extractor_and_action_baselines_on_their_own_units() {
        let (clauses, labels) = dev();
        let (_, report) = fit_observed_text_model(&clauses, &labels, false).unwrap();
        // Baseline is computed on the new structured decoder, rather than borrowed from the old
        // marker extractor. An all-zero decoder can correctly place some unlabeled spans by tie.
        let zero = ObservedTextModel::uninformed();
        let initial = clauses
            .iter()
            .zip(&labels)
            .filter(|(clause, label)| {
                zero.decode_segmentation(&clause.tokens).is_some_and(|seg| {
                    seg.subject == (label.subject.0 as usize, label.subject.1)
                        && seg.cue == (label.marker.0 as usize, label.marker.1)
                        && seg.object == label.object.map(|(s, l)| (s as usize, l))
                })
            })
            .count();
        assert_eq!(report.role_initial_correct, initial);
        assert_eq!(report.role_final_correct, labels.len());
        assert_eq!(report.policy_initial_correct, 3);
        assert_eq!(report.policy_examples, 5);
        assert_eq!(report.policy_final_correct, 5);
    }

    #[test]
    fn full_width_candidate_scores_preserve_the_winning_role_and_ties() {
        for weights in [[40_000, 50_000, 0, 0], [-40_000, -35_000, -50_000, -60_000]] {
            let mut model = ObservedTextModel::uninformed();
            model.feature_weights = vec![((F_FIRST << 40) | 20, weights)];
            let loaded = ObservedTextModel::from_bytes(&model.to_bytes().unwrap(), 4096).unwrap();
            let clause = Clause::of(1, vec![10, 20]);
            assert_eq!(loaded.candidate_scores(&clause.tokens, 1, 1), weights);
            assert_eq!(loaded.locate_marker(&clause.tokens), Some((1, 1, weights)));
            assert_eq!(observe_clause(&loaded, &clause).unwrap().role, 1);
        }
        let mut model = ObservedTextModel::uninformed();
        model.feature_weights = vec![((F_FIRST << 40) | 20, [50_000, 50_000, 0, 0])];
        let clause = Clause::of(1, vec![10, 20, 20]);
        assert_eq!(
            model.locate_marker(&clause.tokens).map(|(s, l, _)| (s, l)),
            Some((1, 1))
        );
        assert_eq!(observe_clause(&model, &clause).unwrap().role, 0);
    }

    #[test]
    fn candidate_and_feature_bounds_reject_unrepresentable_inputs() {
        let tokens = [10, 20, 30, 40, 50, 60];
        for (start, len) in [(usize::MAX, 1), (1, usize::MAX), (0, 0), (6, 1), (0, 5)] {
            assert!(candidate_features(&tokens, start, len).is_empty());
        }
        for key in [
            F_LEN << 40,
            (F_LEN << 40) | (OB_MAX_MARKER as u64 + 1),
            (F_INITIAL << 40) | 1,
            (F_FINAL << 40) | 1,
            (F_FIRST << 40) | 4096,
            (F_FIRST << 40) | (u32::MAX as u64 + 1),
            (F_FINAL + 1) << 40,
        ] {
            let mut model = ObservedTextModel::uninformed();
            model.feature_weights = vec![(key, [0; OB_N_ROLES])];
            assert!(matches!(
                ObservedTextModel::from_bytes(&model.to_bytes().unwrap(), 4096),
                Err(ObservedTextError::Model(_))
            ));
        }
    }

    #[test]
    fn supervision_rejects_conflicts_overlaps_and_unrepresentable_cues() {
        let (clauses, labels) = dev();
        for mutation in 0..6 {
            let mut bad = labels.clone();
            match mutation {
                0 => bad[0].subject = (1, 1),
                1 => bad[0].object = Some((2, 1)),
                2 => bad[0].subject = (u32::MAX, 1),
                3 => bad[6].goal = Goal::Project,
                4 => bad[6].action = RelAction::Continue,
                _ => bad[0].action = RelAction::Read,
            }
            assert!(
                matches!(
                    fit_observed_text_model(&clauses, &bad, false),
                    Err(ObservedTextError::Supervision(_))
                ),
                "mutation {mutation}"
            );
        }
        // A valid in-clause span can still be outside the candidate admission bound.
        let clause = Clause::of(1, vec![10, 20, 21, 22, 23, 24, 30]);
        let label = ClauseLabel {
            seg: 1,
            subject: (0, 1),
            marker: (1, 5),
            object: Some((6, 1)),
            role: 0,
            goal: Goal::Office,
            action: RelAction::Emit,
        };
        assert!(matches!(
            fit_observed_text_model(&[clause], &[label], false),
            Err(ObservedTextError::Supervision(_))
        ));
    }

    /// These tests qualify source ownership, continuation and emission, not length generalization
    /// from the one-token training fixture. Supply transparent boundary potentials for their known
    /// prefix/cue/suffix inputs; separate tests above exercise actual learned span/role fitting.
    fn runtime(clauses: Vec<Clause>, eos: Option<u32>) -> ObservedTextRuntime {
        let (development, labels) = dev();
        let (mut model, _) = fit_observed_text_model(&development, &labels, false).unwrap();
        let mut potentials = BTreeMap::<u64, [i32; OB_SEG_ROLES]>::new();
        potentials.entry(G_INITIAL << 40).or_default()[SEG_SUBJECT] = 10;
        potentials.entry(G_FINAL << 40).or_default()[SEG_OBJECT] = 10;
        potentials.entry((G_LEN << 40) | 2).or_default()[SEG_CUE] = 2;
        for token in [20, 22, 24, 26] {
            potentials.entry((G_RIGHT << 40) | token).or_default()[SEG_SUBJECT] = 20;
            potentials.entry((G_FIRST << 40) | token).or_default()[SEG_CUE] = 20;
        }
        for token in [21, 23, 25, 27] {
            potentials.entry((G_LEFT << 40) | token).or_default()[SEG_OBJECT] = 20;
            potentials.entry((G_LAST << 40) | token).or_default()[SEG_CUE] = 20;
        }
        model.segment_weights = potentials.into_iter().collect();
        ObservedTextRuntime::load(
            &model.to_bytes().unwrap(),
            &"ab".repeat(32),
            clauses,
            1,
            7,
            4096,
            eos,
        )
        .unwrap()
    }

    fn question(subject: Vec<u32>, marker: &[u32]) -> Clause {
        let mut tokens = subject;
        tokens.extend_from_slice(marker);
        Clause::of(999, tokens)
    }

    #[test]
    fn loaded_runtime_preserves_goal_and_exact_multiword_source() {
        let rt = runtime(
            vec![
                Clause::of(10, vec![10, 11, 22, 23, 12, 11]),
                Clause::of(11, vec![12, 11, 20, 21, 32, 33]),
                // Shared suffix is not an exact entity match.
                Clause::of(12, vec![13, 11, 20, 21, 34]),
            ],
            Some(u32::MAX - 1),
        );
        let mut session = rt.start(&question(vec![10, 11], &[20, 21])).unwrap();
        let effects = rt.run(&mut session).unwrap();
        assert_eq!(session.terminal, Some(RelAction::Stop));
        assert_eq!(session.goal, Goal::Office);
        assert_eq!(session.query.goal, Goal::Office);
        assert_eq!(session.emitted, vec![32, 33, u32::MAX - 1]);
        assert_eq!(session.reads, 2);
        assert_eq!(session.captured_span, Some((11, 4, 2)));
        assert_eq!(
            effects
                .iter()
                .filter(|e| e.action == RelAction::Read)
                .count(),
            2
        );
        assert!(rt.origin_is_live(&session));
        rt.validate(&session).unwrap();
    }

    #[test]
    fn missing_fact_cycle_and_read_controls_have_explicit_terminal_reasons() {
        let cycle = runtime(
            vec![Clause::of(9, vec![10, 22, 23, 10])],
            Some(u32::MAX - 1),
        );
        let q = question(vec![10], &[20, 21]);
        let mut s = cycle.start(&q).unwrap();
        cycle.run(&mut s).unwrap();
        assert_eq!(s.terminal, Some(RelAction::Exhausted));
        assert!(s.emitted.is_empty());
        let mut missing = cycle.start(&question(vec![99], &[20, 21])).unwrap();
        cycle.run(&mut missing).unwrap();
        assert_eq!(missing.terminal, Some(RelAction::Unresolved));
        assert!(missing.emitted.is_empty());
        let disabled = runtime(
            vec![Clause::of(9, vec![10, 20, 21, 30])],
            Some(u32::MAX - 1),
        )
        .with_control(TextControl::ReadsDisabled)
        .unwrap();
        let mut no_read = disabled.start(&q).unwrap();
        disabled.run(&mut no_read).unwrap();
        assert_eq!(no_read.terminal, Some(RelAction::Unresolved));
        assert!(no_read.emitted.is_empty());
    }

    #[test]
    fn every_emission_phase_and_final_eos_resume_independently() {
        let rt = runtime(
            vec![Clause::of(10, vec![10, 20, 21, 30, 31, 32])],
            Some(u32::MAX - 1),
        );
        let mut original = rt.start(&question(vec![10], &[20, 21])).unwrap();
        loop {
            let bytes = rt.snapshot(&original).unwrap();
            let mut restored = rt.restore(&bytes).unwrap();
            let mut continued = original.clone();
            assert_eq!(
                rt.run(&mut restored).unwrap(),
                rt.run(&mut continued).unwrap()
            );
            assert_eq!(restored, continued);
            if original.terminal.is_some() {
                break;
            }
            rt.step(&mut original).unwrap();
        }
        assert_eq!(original.emitted, vec![30, 31, 32, u32::MAX - 1]);
        assert!(rt.step(&mut original).is_err());
    }

    #[test]
    fn malformed_or_foreign_snapshots_cannot_emit_unowned_content() {
        let clauses = vec![Clause::of(10, vec![10, 20, 21, 30, 31])];
        let rt = runtime(clauses.clone(), Some(u32::MAX - 1));
        let mut session = rt.start(&question(vec![10], &[20, 21])).unwrap();
        let mut empty_stop = session.clone();
        empty_stop.pending = RelAction::Stop;
        assert!(rt
            .restore(&serde_json::to_vec(&empty_stop).unwrap())
            .is_err());
        empty_stop.terminal = Some(RelAction::Stop);
        assert!(rt
            .restore(&serde_json::to_vec(&empty_stop).unwrap())
            .is_err());
        let mut unowned = session.clone();
        unowned.pending = RelAction::Emit;
        unowned.captured_tokens = vec![100];
        assert!(rt.restore(&serde_json::to_vec(&unowned).unwrap()).is_err());
        rt.step(&mut session).unwrap();
        let valid = rt.snapshot(&session).unwrap();
        // A fake earlier source must not pass merely because counts and the final capture agree.
        let mut bad_history = session.clone();
        bad_history.visited.insert(0, (10, 1, 2));
        bad_history.reads = 2;
        assert!(rt
            .restore(&serde_json::to_vec(&bad_history).unwrap())
            .is_err());
        // These bytes really occur in the document, but they are the marker, not an object.
        let mut marker_capture = session.clone();
        marker_capture.captured.as_mut().unwrap().start = 1;
        marker_capture.captured_span = Some((10, 1, 2));
        marker_capture.visited = vec![(10, 1, 2)];
        marker_capture.captured_tokens = vec![20, 21];
        assert!(rt
            .restore(&serde_json::to_vec(&marker_capture).unwrap())
            .is_err());
        let mut bad_cursor = session.clone();
        bad_cursor.cursor = 3;
        assert!(rt
            .restore(&serde_json::to_vec(&bad_cursor).unwrap())
            .is_err());
        let mut wrong_prefix = session.clone();
        wrong_prefix.cursor = 1;
        wrong_prefix.emitted = vec![99];
        assert!(rt
            .restore(&serde_json::to_vec(&wrong_prefix).unwrap())
            .is_err());
        let mut wrong_source = session.clone();
        wrong_source.captured_tokens[0] = 99;
        assert!(rt
            .restore(&serde_json::to_vec(&wrong_source).unwrap())
            .is_err());
        let mut wrong_goal = session.clone();
        wrong_goal.query.goal = Goal::Project;
        assert!(rt
            .restore(&serde_json::to_vec(&wrong_goal).unwrap())
            .is_err());
        let mut wrong_version = session.clone();
        wrong_version.version = 99;
        assert!(rt
            .restore(&serde_json::to_vec(&wrong_version).unwrap())
            .is_err());
        let other = runtime(
            vec![Clause::of(10, vec![10, 20, 21, 32, 31])],
            Some(u32::MAX - 1),
        );
        assert!(other.restore(&valid).is_err());
        assert!(!other.origin_is_live(&session));
        assert_eq!(session.captured_tokens, vec![30, 31]); // immutable owned copy was not changed
        let control = runtime(clauses.clone(), Some(u32::MAX - 1))
            .with_control(TextControl::ReadsDisabled)
            .unwrap();
        assert!(control.restore(&valid).is_err());
        let model_bytes = rt.model().to_bytes().unwrap();
        let other_namespace = ObservedTextRuntime::load(
            &model_bytes,
            &"ab".repeat(32),
            clauses.clone(),
            2,
            7,
            4096,
            Some(u32::MAX - 1),
        )
        .unwrap();
        assert!(other_namespace.restore(&valid).is_err());
        let other_tokenizer = ObservedTextRuntime::load(
            &model_bytes,
            &"cd".repeat(32),
            clauses.clone(),
            1,
            7,
            4096,
            Some(u32::MAX - 1),
        )
        .unwrap();
        assert!(other_tokenizer.restore(&valid).is_err());
        let mut changed_model = rt.model().clone();
        changed_model.feature_weights[0].1[0] += 1;
        let other_model = ObservedTextRuntime::load(
            &changed_model.to_bytes().unwrap(),
            &"ab".repeat(32),
            clauses,
            1,
            7,
            4096,
            Some(u32::MAX - 1),
        )
        .unwrap();
        assert!(other_model.restore(&valid).is_err());
    }

    #[test]
    fn long_legal_phrase_completes_instead_of_being_silently_truncated() {
        let mut tokens = vec![10, 20, 21];
        let answer: Vec<u32> = (100..121).collect();
        tokens.extend_from_slice(&answer);
        assert_eq!(tokens.len(), OB_MAX_CLAUSE);
        let rt = runtime(vec![Clause::of(1, tokens)], Some(u32::MAX - 1));
        let mut session = rt.start(&question(vec![10], &[20, 21])).unwrap();
        rt.run(&mut session).unwrap();
        let mut expected = answer;
        expected.push(u32::MAX - 1);
        assert_eq!(session.emitted, expected);
        assert_eq!(session.terminal, Some(RelAction::Stop));
    }

    #[test]
    fn restore_rejects_forged_role_phases_and_unrelated_valid_source_history() {
        let clauses = vec![
            Clause::of(10, vec![10, 22, 23, 11]),
            Clause::of(11, vec![11, 20, 21, 32]),
            Clause::of(12, vec![12, 20, 21, 33]),
        ];
        let rt = runtime(clauses.clone(), Some(u32::MAX - 1));
        let q = question(vec![10], &[20, 21]);
        let mut session = rt.start(&q).unwrap();
        rt.step(&mut session).unwrap();
        assert_eq!(session.pending, RelAction::Continue);
        let mut forged = session.clone();
        forged.pending = RelAction::Emit;
        assert!(rt.restore(&serde_json::to_vec(&forged).unwrap()).is_err());
        rt.step(&mut session).unwrap();
        rt.step(&mut session).unwrap();
        assert_eq!(session.pending, RelAction::Emit);
        forged = session.clone();
        forged.pending = RelAction::Continue;
        assert!(rt.restore(&serde_json::to_vec(&forged).unwrap()).is_err());
        forged = session.clone();
        // The unrelated source is a genuine same-goal object, but not a predecessor of the capture.
        forged.visited[0] = (12, 3, 1);
        assert!(rt.restore(&serde_json::to_vec(&forged).unwrap()).is_err());
        rt.run(&mut session).unwrap();
        assert_eq!(session.emitted, vec![32, u32::MAX - 1]);

        // Explicit controls may legitimately override the role table; restoration must agree with
        // the bound control instead of hard-coding the primary's role decision.
        let membership = runtime(clauses, Some(u32::MAX - 1))
            .with_control(TextControl::Membership {
                entities: Vec::new(),
            })
            .unwrap();
        let mut controlled = membership.start(&q).unwrap();
        membership.step(&mut controlled).unwrap();
        assert_eq!(controlled.pending, RelAction::Emit);
        let mut restored = membership
            .restore(&membership.snapshot(&controlled).unwrap())
            .unwrap();
        membership.run(&mut restored).unwrap();
        assert_eq!(restored.emitted, vec![11, u32::MAX - 1]);
    }

    #[test]
    fn candidate_features_distinguish_local_order_but_not_four_token_interiors() {
        // The token is identical, while its observed neighbors differ.
        let a = candidate_features(&[10, 20, 30, 20, 40], 1, 1);
        let b = candidate_features(&[10, 20, 30, 20, 40], 3, 1);
        assert_ne!(a, b);
        // This is an actual permutation: token identity and bag are preserved.
        let fwd = candidate_features(&[10, 20, 30, 40], 1, 2);
        let rev = candidate_features(&[10, 30, 20, 40], 1, 2);
        assert_ne!(fwd, rev);
        // Endpoints and external context cannot distinguish this interior permutation. Record the
        // representation limit instead of claiming every permutation changes the model's score.
        assert_eq!(
            candidate_features(&[10, 20, 30, 40, 50, 60], 1, 4),
            candidate_features(&[10, 20, 40, 30, 50, 60], 1, 4)
        );
        let inner = candidate_features(&[10, 20, 30, 40], 1, 1);
        let outer = candidate_features(&[10, 20, 30, 40], 0, 1);
        assert!(outer.contains(&((F_INITIAL << 40) | 0)));
        assert!(!inner.contains(&((F_INITIAL << 40) | 0)));
    }

    #[test]
    fn model_loader_and_fit_reject_invalid_semantics_and_source_labels() {
        let (clauses, labels) = dev();
        let (model, _) = fit_observed_text_model(&clauses, &labels, false).unwrap();
        assert_eq!(
            ObservedTextModel::from_bytes(&model.to_bytes().unwrap(), 4096).unwrap(),
            model
        );
        assert!(matches!(
            ObservedTextModel::from_bytes(b"{", 4096),
            Err(ObservedTextError::Serialization(_))
        ));
        assert!(matches!(
            observe_clause(&model, &Clause::of(999, Vec::new())),
            Err(ObservedTextError::Observation(_))
        ));
        let mut invalid = model.clone();
        invalid.version = 1;
        assert!(matches!(
            ObservedTextModel::from_bytes(&invalid.to_bytes().unwrap(), 4096),
            Err(ObservedTextError::Model(_))
        ));
        invalid = model.clone();
        invalid.action_per_role[0] = 99;
        assert!(ObservedTextModel::from_bytes(&invalid.to_bytes().unwrap(), 4096).is_err());
        invalid = model.clone();
        invalid.feature_weights.push(invalid.feature_weights[0]);
        assert!(ObservedTextModel::from_bytes(&invalid.to_bytes().unwrap(), 4096).is_err());
        invalid = model.clone();
        invalid.goal_per_role[0] = 99;
        assert!(ObservedTextModel::from_bytes(&invalid.to_bytes().unwrap(), 4096).is_err());
        let mut bad_labels = labels.clone();
        bad_labels[0].marker.1 = 100;
        assert!(fit_observed_text_model(&clauses, &bad_labels, false).is_err());
        bad_labels = labels.clone();
        bad_labels[0].role = OB_N_ROLES;
        assert!(matches!(
            fit_observed_text_model(&clauses, &bad_labels, false),
            Err(ObservedTextError::Supervision(_))
        ));
    }

    #[test]
    fn structured_decoder_admits_reordering_and_background_with_exact_scores() {
        let mut model = ObservedTextModel::uninformed();
        // Authored component test, not a learned-language result: object precedes subject and cue;
        // a trailing token receives the fixed zero background score.
        model.segment_weights = vec![
            ((G_FIRST << 40) | 10, [0, i32::MAX, 0, 0]),
            ((G_FIRST << 40) | 20, [0, 0, i32::MAX, 0]),
            ((G_FIRST << 40) | 30, [0, 0, 0, i32::MAX]),
            ((G_LEN << 40) | 1, [0, 1, 1, 1]),
        ];
        let loaded = ObservedTextModel::from_bytes(&model.to_bytes().unwrap(), 4096).unwrap();
        let decoded = loaded.decode_segmentation(&[30, 10, 20, 99]).unwrap();
        assert_eq!(decoded.subject, (1, 1));
        assert_eq!(decoded.cue, (2, 1));
        assert_eq!(decoded.object, Some((0, 1)));
        assert_eq!(decoded.score, 3 * (i64::from(i32::MAX) + 1));
        for (start, len) in [
            (usize::MAX, 1),
            (1, usize::MAX),
            (0, 0),
            (0, OB_MAX_SEG + 1),
        ] {
            assert!(model
                .segment_features(&[10, 20], start, len, &[])
                .is_empty());
        }
    }

    #[test]
    fn segment_model_loader_rejects_invalid_fields_and_nonzero_background() {
        for key in [
            G_LEN << 40,
            (G_LEN << 40) | 25,
            (G_FIRST << 40) | 4096,
            (G_INITIAL << 40) | 1,
            (G_FINAL << 40) | 1,
            (G_H4_RIGHT + 1) << 40,
            (G_H4_INTERVAL << 40) | 120,
        ] {
            let mut bad = ObservedTextModel::uninformed();
            bad.segment_weights = vec![(key, [0; OB_SEG_ROLES])];
            assert!(ObservedTextModel::from_bytes(&bad.to_bytes().unwrap(), 4096).is_err());
        }
        let mut bad = ObservedTextModel::uninformed();
        bad.segment_weights = vec![(G_INITIAL << 40, [1, 0, 0, 0])];
        assert!(bad.validate(4096).is_err());
        bad.segment_weights = vec![(G_INITIAL << 40, [0; 4]); 2];
        assert!(bad.validate(4096).is_err());
        for elements in [
            vec![(1, 120)],
            vec![(4096, 0)],
            vec![(1, 0), (1, 0)],
            vec![(2, 0), (1, 0)],
        ] {
            bad = ObservedTextModel::uninformed();
            bad.use_h4 = true;
            bad.token_elements = elements;
            assert!(bad.validate(4096).is_err());
        }
        bad = ObservedTextModel::uninformed();
        bad.token_elements = vec![(1, 0)];
        assert!(bad.validate(4096).is_err());
    }

    #[test]
    fn lexical_keys_are_exact_and_identity_regimes_are_disjoint() {
        assert_eq!(lexical_key(" Ivo", &[2, 2], 0, 2), Some(b"Ivo".to_vec()));
        assert_eq!(lexical_key("Ivo", &[3], 0, 1), Some(b"Ivo".to_vec()));
        assert_ne!(
            lexical_key("Ivo", &[3], 0, 1),
            lexical_key("ivo", &[3], 0, 1)
        );
        assert_ne!(
            lexical_key("Cedar Annex", &[5, 6], 0, 2),
            lexical_key("CedarAnnex", &[5, 5], 0, 2)
        );
        assert_eq!(lexical_key("Ivo", &[1, 1], 0, 1), None);
        assert_eq!(lexical_key("Ivo", &[3], usize::MAX, 2), None);
        assert_eq!(lexical_key("Ivo", &[3], 0, usize::MAX), None);
        let raw = Clause {
            seg: 0,
            tokens: vec![1],
            text: "abcd".into(),
            byte_lengths: vec![4],
        };
        let legacy = Clause::of(1, vec![u32::from_le_bytes(*b"abcd")]);
        assert_ne!(
            identity_key(&raw, 0, 1).unwrap(),
            identity_key(&legacy, 0, 1).unwrap()
        );
        let rt = runtime(vec![Clause::of(1, vec![10, 20, 21, 30])], None);
        let mut invalid = Clause {
            seg: 0,
            tokens: vec![10, 20, 21],
            text: "Mara works".into(),
            byte_lengths: vec![1, 1, 1],
        };
        assert!(rt.start(&invalid).is_err());
        invalid.byte_lengths.clear(); // explicitly legacy token-only is still supported
        assert!(rt.start(&invalid).is_ok());
    }

    #[test]
    fn byte_identity_joins_distinct_token_forms_and_restore_binds_capture_key() {
        // Token IDs deliberately differ across the first redirect object and the next subject.
        // This component test supplies verified extents; the real tokenizer adapter owns the
        // token-byte authenticity check and is exercised by the runner, not assumed here.
        let clauses = vec![
            Clause {
                seg: 1,
                tokens: vec![10, 22, 23, 11],
                text: "Mara office follows Ivo".into(),
                byte_lengths: vec![4, 7, 8, 4],
            },
            Clause {
                seg: 2,
                tokens: vec![12, 20, 21, 30],
                text: "Ivo works in Fen".into(),
                byte_lengths: vec![3, 6, 3, 4],
            },
        ];
        let rt = runtime(clauses, Some(u32::MAX - 1));
        let q = Clause {
            seg: 999,
            tokens: vec![13, 20, 21],
            text: "Mara works in".into(),
            byte_lengths: vec![4, 6, 3],
        };
        let mut session = rt.start(&q).unwrap();
        rt.step(&mut session).unwrap();
        assert_eq!(session.pending, RelAction::Continue);
        let mut forged = session.clone();
        forged.captured.as_mut().unwrap().key = [vec![1], b"Fen".to_vec()].concat();
        assert!(rt.restore(&serde_json::to_vec(&forged).unwrap()).is_err());
        loop {
            let bytes = rt.snapshot(&session).unwrap();
            let mut loaded = rt.restore(&bytes).unwrap();
            let mut same = session.clone();
            assert_eq!(rt.run(&mut loaded).unwrap(), rt.run(&mut same).unwrap());
            assert_eq!(loaded, same);
            if session.terminal.is_some() {
                break;
            }
            rt.step(&mut session).unwrap();
        }
        assert_eq!(session.reads, 2);
        assert_eq!(session.emitted, vec![30, u32::MAX - 1]);
    }

    #[test]
    fn ordered_interval_products_equal_direct_noncommutative_composition() {
        let t = super::super::group_table::group_table();
        let mut pair = None;
        for a in 0..super::super::group_table::GROUP_ORDER {
            for b in 0..super::super::group_table::GROUP_ORDER {
                if t.product[a * super::super::group_table::ROW_STRIDE + b]
                    != t.product[b * super::super::group_table::ROW_STRIDE + a]
                {
                    pair = Some((a, b));
                    break;
                }
            }
            if pair.is_some() {
                break;
            }
        }
        let (a, b) = pair.unwrap();
        let mut model = ObservedTextModel::uninformed();
        model.use_h4 = true;
        model.token_elements = vec![(10, a as u8), (20, b as u8)];
        let tokens = [10, 20, 10, 20];
        let prefix = model.prefix_products(&tokens);
        for start in 0..=tokens.len() {
            let mut direct = t.identity as usize;
            for end in start..=tokens.len() {
                assert_eq!(interval_product(&prefix, start, end), Some(direct));
                if end < tokens.len() {
                    direct = t.product[direct * super::super::group_table::ROW_STRIDE
                        + model.token_element(tokens[end])] as usize;
                }
            }
        }
        let reversed = model.prefix_products(&[20, 10]);
        assert_ne!(
            interval_product(&prefix, 0, 2),
            interval_product(&reversed, 0, 2)
        );
        assert_eq!(interval_product(&[], 0, 0), None);
        assert!(model.segment_features(&tokens, 0, 2, &[]).is_empty());
        assert_eq!(model.token_element(999), t.identity as usize);
    }
}
