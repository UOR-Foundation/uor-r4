//! Observed-text memory answers through one reusable session boundary.
//!
//! The serving input is **observed tokens plus declared system metadata only**: a tokenized document
//! of statements, an observed question clause, and declared clause boundaries. An entity registry
//! is supplied only to the explicitly named membership control. No gold semantic role,
//! subject/object/answer extent, follow pointer, depth or target is available to the primary path.
//!
//! What is learned from declared development text:
//!
//! * `token_votes`: which observed marker tokens vote for which role (so a familiar word in a new
//!   combination still scores), used to locate the marker run inside a clause;
//! * `edge_votes`: which observed tokens are boundary fillers rather than part of an entity span;
//! * `action_per_role`: the action semantics of each role, fitted from declared gold actions starting
//!   from a declared uninformed policy.
//!
//! What is exact and typed: entity/answer spans keep full multiword token identity together with
//! their occurrence `(segment, start, len)`, so a repeated word or a shared suffix cannot collapse two
//! different spans, and a captured phrase stays usable after its origin is overwritten.
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
}

/// Declared offline supervision for one development clause. Never available at serving.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

/// The learned observation model.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedTextModel {
    pub version: u8,
    /// Learned marker token votes, sorted by token.
    pub token_votes: Vec<(u32, [i16; OB_N_ROLES])>,
    /// Learned boundary votes: positive means part of an entity span, negative a filler.
    pub edge_votes: Vec<(u32, i16)>,
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
    /// A declared uninformed start: no marker votes, no boundary votes, every role terminal office.
    pub fn uninformed() -> Self {
        ObservedTextModel {
            version: 2,
            token_votes: Vec::new(),
            edge_votes: Vec::new(),
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
        if self.version != 2 {
            return Err("unsupported observed-text model version".into());
        }
        if max_vocab == 0 {
            return Err("empty vocabulary".into());
        }
        if self.token_votes.windows(2).any(|w| w[0].0 >= w[1].0)
            || self.edge_votes.windows(2).any(|w| w[0].0 >= w[1].0)
        {
            return Err("observed-text tables must have unique sorted tokens".into());
        }
        if self
            .token_votes
            .iter()
            .any(|(t, v)| *t as usize >= max_vocab || v.iter().any(|x| *x < 0))
            || self
                .edge_votes
                .iter()
                .any(|(t, _)| *t as usize >= max_vocab)
        {
            return Err("observed-text vote token or weight is invalid".into());
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

    pub fn votes_of(&self, token: u32) -> [i16; OB_N_ROLES] {
        self.token_votes
            .iter()
            .find(|(t, _)| *t == token)
            .map(|(_, v)| *v)
            .unwrap_or([0; OB_N_ROLES])
    }

    pub fn edge_of(&self, token: u32) -> i16 {
        self.edge_votes
            .iter()
            .find(|(t, _)| *t == token)
            .map(|(_, v)| *v)
            .unwrap_or(0)
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

    /// Locate the marker run of a clause from the learned votes alone. Returns the marker start and
    /// length plus the summed votes of the chosen run.
    pub fn locate_marker(&self, tokens: &[u32]) -> Option<(usize, usize, [i16; OB_N_ROLES])> {
        if tokens.len() > OB_MAX_CLAUSE {
            return None;
        }
        let n = tokens.len();
        if n < 2 {
            return None;
        }
        let mut best: Option<(i32, usize, usize, [i16; OB_N_ROLES])> = None;
        for start in 0..n {
            for len in 1..=(n - start) {
                // A marker must leave an observed subject and, for a statement, an object.
                if start == 0 {
                    continue;
                }
                let mut sum = [0i16; OB_N_ROLES];
                let mut magnitude = 0i32;
                for t in tokens[start..start + len].iter() {
                    let v = self.votes_of(*t);
                    for r in 0..OB_N_ROLES {
                        sum[r] = sum[r].saturating_add(v[r]);
                    }
                }
                for r in 0..OB_N_ROLES {
                    magnitude += i32::from(sum[r]).max(0);
                }
                if magnitude == 0 {
                    continue;
                }
                let better = match best {
                    None => true,
                    Some((bm, _, blen, _)) => magnitude > bm || (magnitude == bm && len < blen),
                };
                if better {
                    best = Some((magnitude, start, len, sum));
                }
            }
        }
        best.map(|(_, start, len, sum)| (start, len, sum))
    }

    /// Trim learned boundary fillers from both ends of a candidate span.
    pub fn trim_bounds(&self, tokens: &[u32]) -> (usize, usize) {
        let mut lo = 0usize;
        let mut hi = tokens.len();
        while lo < hi && self.edge_of(tokens[lo]) < 0 {
            lo += 1;
        }
        while hi > lo && self.edge_of(tokens[hi - 1]) < 0 {
            hi -= 1;
        }
        (lo, hi)
    }

    pub fn trim(&self, tokens: &[u32]) -> Vec<u32> {
        let (lo, hi) = self.trim_bounds(tokens);
        tokens[lo..hi].to_vec()
    }
}

/// The learned observation extractor: clause -> (subject, marker, object, role, question goal).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub seg: u32,
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
    let (mstart, mlen, votes) = model
        .locate_marker(&clause.tokens)
        .ok_or("no learned marker run in clause")?;
    let role = (0..OB_N_ROLES).max_by_key(|r| votes[*r]).unwrap_or(0);
    let subject_raw = &clause.tokens[..mstart];
    let tail = &clause.tokens[mstart + mlen..];
    let (subject_lo, subject_hi) = model.trim_bounds(subject_raw);
    let subject = subject_raw[subject_lo..subject_hi].to_vec();
    if subject.is_empty() {
        return Err("clause has no observed subject".into());
    }
    let (object_lo, object_hi) = model.trim_bounds(tail);
    let object = tail[object_lo..object_hi].to_vec();
    let question_goal = if object.is_empty() {
        model.role_goal(role)
    } else {
        None
    };
    Ok(Observation {
        seg: clause.seg,
        subject_start: subject_lo as u32,
        marker_start: mstart as u32,
        object_start: if object.is_empty() {
            None
        } else {
            Some((mstart + mlen + object_lo) as u32)
        },
        subject,
        marker: clause.tokens[mstart..mstart + mlen].to_vec(),
        object: if object.is_empty() {
            None
        } else {
            Some(object)
        },
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
    fn start(binding: TextBinding, goal: Goal, tokens: Vec<u32>) -> Self {
        Self {
            version: 2,
            eos: binding.eos,
            binding,
            goal,
            query: TextObjective { goal, tokens },
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
        if self.version != 2
            || &self.binding != expected
            || max_vocab != expected.max_vocab
            || self.eos != expected.eos
        {
            return Err("session version or runtime binding mismatch".into());
        }
        if self.goal != self.query.goal
            || self.query.tokens.is_empty()
            || self.query.tokens.len() > OB_MAX_CLAUSE
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
        let frame = TextSession::start(self.binding.clone(), goal, observed.subject);
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
        for &(segment, start, len) in &session.visited {
            if !self.observations.iter().any(|observed| {
                observed.seg == segment
                    && observed.object_start == Some(start)
                    && observed
                        .object
                        .as_ref()
                        .is_some_and(|tokens| tokens.len() == len)
                    && self.model.role_goal(observed.role) == Some(session.goal)
            }) {
                return Err(
                    "visited reference is not an observed object for the active goal".into(),
                );
            }
        }
        if session.captured.is_some() && !self.origin_is_live(session) {
            return Err("captured phrase disagrees with its bound source occurrence".into());
        }
        Ok(())
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
                        if observed.subject != session.query.tokens
                            || self.model.role_goal(observed.role) != Some(session.goal)
                            || observed.object.is_none()
                        {
                            continue;
                        }
                        let votes = self
                            .model
                            .votes_of(observed.marker.first().copied().unwrap_or(0));
                        let score: i32 = votes.iter().map(|v| i32::from(*v)).sum();
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
                                world_id: self.binding.world_id,
                                world_version: self.binding.world_version,
                                doc_sha256: self.binding.doc_sha256.clone(),
                                segment: observed.seg,
                                start,
                                len: object.len(),
                            });
                            session.captured_span = Some(span);
                            session.captured_tokens = object.clone();
                            session.pending = match &self.control {
                                TextControl::Membership { entities } => {
                                    if entities.contains(object) {
                                        RelAction::Continue
                                    } else {
                                        RelAction::Emit
                                    }
                                }
                                _ => self.model.role_action(observed.role),
                            };
                        }
                    } else {
                        session.terminate(RelAction::Unresolved);
                    }
                }
            }
            RelAction::Continue => {
                session.query = TextObjective {
                    goal: session.goal,
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
    pub marker_tokens: usize,
    pub boundary_tokens: usize,
    pub policy_initial_correct: usize,
    pub policy_final_correct: usize,
    pub policy_examples: usize,
    pub roles_observed: Vec<usize>,
}

/// Fit the observation model from declared development clauses and their gold labels.
pub fn fit_observed_text_model(
    clauses: &[Clause],
    labels: &[ClauseLabel],
) -> Result<(ObservedTextModel, ObFitReport), ObservedTextError> {
    fit_observed_text_model_inner(clauses, labels).map_err(ObservedTextError::Supervision)
}

fn fit_observed_text_model_inner(
    clauses: &[Clause],
    labels: &[ClauseLabel],
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
    let mut semantics = BTreeMap::new();
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
        if !within(label.subject)
            || !within(label.marker)
            || label.object.is_some_and(|span| !within(span))
        {
            return Err("supervised extent outside observed clause".into());
        }
        if label.subject.0 as usize + label.subject.1 > label.marker.0 as usize
            || label
                .object
                .is_some_and(|span| (span.0 as usize) < label.marker.0 as usize + label.marker.1)
        {
            return Err("supervised spans violate declared subject-marker-object order".into());
        }
        if !matches!(label.action, RelAction::Emit | RelAction::Continue) {
            return Err("unsupported supervised observed-text action".into());
        }
        let entry = semantics.entry(label.role).or_insert((label.goal, None));
        if entry.0 != label.goal {
            return Err("conflicting goal labels for role".into());
        }
        if label.object.is_some() {
            if entry.1.is_some_and(|action| action != label.action) {
                return Err("conflicting action labels for role".into());
            }
            entry.1 = Some(label.action);
        }
    }
    let mut model = ObservedTextModel::uninformed();
    // Marker token votes: each token of a gold marker votes for that role.
    let mut votes: BTreeMap<u32, [i16; OB_N_ROLES]> = BTreeMap::new();
    for label in labels {
        let Some(clause) = clauses.iter().find(|c| c.seg == label.seg) else {
            continue;
        };
        let (ms, ml) = label.marker;
        for t in clause.tokens.iter().skip(ms as usize).take(ml) {
            let e = votes.entry(*t).or_insert([0; OB_N_ROLES]);
            e[label.role.min(OB_N_ROLES - 1)] = e[label.role.min(OB_N_ROLES - 1)].saturating_add(1);
        }
    }
    model.token_votes = votes.into_iter().collect();
    // Boundary votes: tokens inside a gold entity span are positive, adjacent fillers negative.
    let mut edges: BTreeMap<u32, i16> = BTreeMap::new();
    for label in labels {
        let Some(clause) = clauses.iter().find(|c| c.seg == label.seg) else {
            continue;
        };
        let (ss, sl) = label.subject;
        if let Some(w) = clause
            .tokens
            .get(ss.checked_sub(1).map(|x| x as usize).unwrap_or(usize::MAX))
        {
            *edges.entry(*w).or_default() -= 1;
        }
        for t in clause.tokens.iter().skip(ss as usize).take(sl) {
            *edges.entry(*t).or_default() += 1;
        }
        if let Some((os, ol)) = label.object {
            if let Some(w) = clause
                .tokens
                .get(os.checked_sub(1).map(|x| x as usize).unwrap_or(usize::MAX))
            {
                *edges.entry(*w).or_default() -= 1;
            }
            for t in clause.tokens.iter().skip(os as usize).take(ol) {
                *edges.entry(*t).or_default() += 1;
            }
        }
    }
    model.edge_votes = edges.into_iter().filter(|(_, v)| *v != 0).collect();
    // Action/policy per role, fitted from declared gold actions. The *inputs* are the fitted roles
    // and the *labels* are the declared actions; the uninformed start is all-emit.
    // Declared offline supervision supplies the role's goal and action semantics. Serving never sees
    // these labels: it uses only the fitted tables below.
    let mut per_role_action: BTreeMap<usize, Vec<u8>> = BTreeMap::new();
    let mut per_role_goal: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for label in labels {
        per_role_goal
            .entry(label.role)
            .or_default()
            .push(label.goal.index());
        if label.object.is_some() {
            per_role_action
                .entry(label.role)
                .or_default()
                .push(action_tag(label.action));
        }
    }
    for (role, actions) in per_role_action.iter() {
        let mut counts: BTreeMap<u8, usize> = BTreeMap::new();
        for a in actions {
            *counts.entry(*a).or_default() += 1;
        }
        if let Some((tag, _)) = counts.iter().max_by(|a, b| a.1.cmp(b.1).then(a.0.cmp(b.0))) {
            let r = (*role).min(OB_N_ROLES - 1);
            model.action_per_role[r] = *tag;
            model.redirect_per_role[r] = *tag == action_tag(RelAction::Continue);
        }
    }
    for (role, goals) in per_role_goal.iter() {
        let mut counts: BTreeMap<usize, usize> = BTreeMap::new();
        for g in goals {
            *counts.entry(*g).or_default() += 1;
        }
        if let Some((g, _)) = counts.iter().max_by(|a, b| a.1.cmp(b.1).then(a.0.cmp(b.0))) {
            model.goal_per_role[(*role).min(OB_N_ROLES - 1)] = *g as u8;
        }
    }
    // Pre/post policy accounting on the same units, from a declared uninformed start.
    let mut initial_correct = 0usize;
    let mut final_correct = 0usize;
    let mut examples = 0usize;
    for label in labels {
        if label.object.is_none() {
            continue; // question clauses carry no continuation decision
        }
        examples += 1;
        let want = label.action;
        if RelAction::Emit == want {
            initial_correct += 1;
        }
        if model.role_action(label.role) == want {
            final_correct += 1;
        }
    }
    let report = ObFitReport {
        clauses: clauses.len(),
        marker_tokens: model.token_votes.len(),
        boundary_tokens: model.edge_votes.len(),
        policy_initial_correct: initial_correct,
        policy_final_correct: final_correct,
        policy_examples: examples,
        roles_observed: per_role_goal
            .keys()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
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
            Clause {
                seg: 0,
                tokens: vec![10, 20, 21, 30],
            }, // Mara works-in Cedar
            Clause {
                seg: 1,
                tokens: vec![10, 22, 23, 11],
            }, // Mara office-follows Ivo
            Clause {
                seg: 2,
                tokens: vec![10, 24, 25, 31],
            }, // Mara is-on-project Atlas
            Clause {
                seg: 3,
                tokens: vec![10, 26, 27, 12],
            }, // Mara project-follows Nila
            Clause {
                seg: 4,
                tokens: vec![10, 20, 21],
            }, // Mara works-in ?
            Clause {
                seg: 5,
                tokens: vec![10, 24, 25],
            }, // Mara is-on-project ?
            Clause {
                seg: 6,
                tokens: vec![11, 20, 21, 32],
            }, // Ivo works-in Fen
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
        let (model, report) = fit_observed_text_model(&clauses, &labels).unwrap();
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

    fn runtime(clauses: Vec<Clause>, eos: Option<u32>) -> ObservedTextRuntime {
        let (development, labels) = dev();
        let (model, _) = fit_observed_text_model(&development, &labels).unwrap();
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
        Clause { seg: 999, tokens }
    }

    #[test]
    fn loaded_runtime_preserves_goal_and_exact_multiword_source() {
        let rt = runtime(
            vec![
                Clause {
                    seg: 10,
                    tokens: vec![10, 11, 22, 23, 12, 11],
                },
                Clause {
                    seg: 11,
                    tokens: vec![12, 11, 20, 21, 32, 33],
                },
                // Shared suffix is not an exact entity match.
                Clause {
                    seg: 12,
                    tokens: vec![13, 11, 20, 21, 34],
                },
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
            vec![Clause {
                seg: 9,
                tokens: vec![10, 22, 23, 10],
            }],
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
            vec![Clause {
                seg: 9,
                tokens: vec![10, 20, 21, 30],
            }],
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
            vec![Clause {
                seg: 10,
                tokens: vec![10, 20, 21, 30, 31, 32],
            }],
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
        let clauses = vec![Clause {
            seg: 10,
            tokens: vec![10, 20, 21, 30, 31],
        }];
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
            vec![Clause {
                seg: 10,
                tokens: vec![10, 20, 21, 32, 31],
            }],
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
        changed_model.token_votes[0].1[0] += 1;
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
        let rt = runtime(vec![Clause { seg: 1, tokens }], Some(u32::MAX - 1));
        let mut session = rt.start(&question(vec![10], &[20, 21])).unwrap();
        rt.run(&mut session).unwrap();
        let mut expected = answer;
        expected.push(u32::MAX - 1);
        assert_eq!(session.emitted, expected);
        assert_eq!(session.terminal, Some(RelAction::Stop));
    }

    #[test]
    fn trim_offsets_preserve_both_ends_of_exact_spans() {
        let (clauses, labels) = dev();
        let (mut model, _) = fit_observed_text_model(&clauses, &labels).unwrap();
        model.edge_votes.push((90, -4));
        model.edge_votes.sort_by_key(|x| x.0);
        let clause = Clause {
            seg: 7,
            tokens: vec![90, 10, 11, 90, 20, 21, 90, 30, 31, 90],
        };
        let observed = observe_clause(&model, &clause).unwrap();
        assert_eq!(
            (observed.subject_start, observed.subject),
            (1, vec![10, 11])
        );
        assert_eq!(
            (observed.object_start, observed.object),
            (Some(7), Some(vec![30, 31]))
        );
    }

    #[test]
    fn model_loader_and_fit_reject_invalid_semantics_and_source_labels() {
        let (clauses, labels) = dev();
        let (model, _) = fit_observed_text_model(&clauses, &labels).unwrap();
        assert_eq!(
            ObservedTextModel::from_bytes(&model.to_bytes().unwrap(), 4096).unwrap(),
            model
        );
        assert!(matches!(
            ObservedTextModel::from_bytes(b"{", 4096),
            Err(ObservedTextError::Serialization(_))
        ));
        assert!(matches!(
            observe_clause(
                &model,
                &Clause {
                    seg: 999,
                    tokens: Vec::new()
                }
            ),
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
        invalid.token_votes.push(invalid.token_votes[0]);
        assert!(ObservedTextModel::from_bytes(&invalid.to_bytes().unwrap(), 4096).is_err());
        invalid = model.clone();
        invalid.goal_per_role[0] = 99;
        assert!(ObservedTextModel::from_bytes(&invalid.to_bytes().unwrap(), 4096).is_err());
        let mut bad_labels = labels.clone();
        bad_labels[0].marker.1 = 100;
        assert!(fit_observed_text_model(&clauses, &bad_labels).is_err());
        bad_labels = labels.clone();
        bad_labels[0].role = OB_N_ROLES;
        assert!(matches!(
            fit_observed_text_model(&clauses, &bad_labels),
            Err(ObservedTextError::Supervision(_))
        ));
    }
}
