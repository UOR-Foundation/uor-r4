//! Observed-text memory answers through one reusable session boundary.
//!
//! The serving input is **observed tokens plus declared system metadata only**: a tokenized document
//! of statements, an observed question clause, a declared clause delimiter and a declared entity
//! registry. No gold semantic role, subject/object/answer extent, follow pointer, depth or target is
//! available to the served path.
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
//! The session boundary [`TextSession::step`] is the single causal implementation used by teacher
//! forcing, generation, interventions, resumption and timing. `RelAction` and `CapturedPayload` are
//! the corrected predecessor's action/ownership vocabulary, reused rather than re-invented.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use super::relational_session::{CapturedPayload, RelAction};

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clause {
    pub seg: u32,
    pub tokens: Vec<u32>,
}

/// Declared offline supervision for one development clause. Never available at serving.
#[derive(Clone, Debug, PartialEq, Eq)]
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
            version: 1,
            token_votes: Vec::new(),
            edge_votes: Vec::new(),
            action_per_role: [action_tag(RelAction::Emit); OB_N_ROLES],
            goal_per_role: [0; OB_N_ROLES],
            redirect_per_role: [false; OB_N_ROLES],
        }
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
        action_from_tag(self.action_per_role[role.min(OB_N_ROLES - 1)])
    }

    pub fn role_goal(&self, role: usize) -> Option<Goal> {
        Goal::from_index(self.goal_per_role[role.min(OB_N_ROLES - 1)] as usize)
    }

    pub fn role_is_redirect(&self, role: usize) -> bool {
        self.redirect_per_role[role.min(OB_N_ROLES - 1)]
    }

    /// Locate the marker run of a clause from the learned votes alone. Returns the marker start and
    /// length plus the summed votes of the chosen run.
    pub fn locate_marker(&self, tokens: &[u32]) -> Option<(usize, usize, [i16; OB_N_ROLES])> {
        let n = tokens.len().min(OB_MAX_CLAUSE);
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
    pub fn trim(&self, tokens: &[u32]) -> Vec<u32> {
        let mut lo = 0usize;
        let mut hi = tokens.len();
        while lo < hi && self.edge_of(tokens[lo]) < 0 {
            lo += 1;
        }
        while hi > lo && self.edge_of(tokens[hi - 1]) < 0 {
            hi -= 1;
        }
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
pub fn observe_clause(model: &ObservedTextModel, clause: &Clause) -> Result<Observation, String> {
    if clause.tokens.is_empty() {
        return Err("empty clause".into());
    }
    let (mstart, mlen, votes) = model
        .locate_marker(&clause.tokens)
        .ok_or("no learned marker run in clause")?;
    let role = (0..OB_N_ROLES).max_by_key(|r| votes[*r]).unwrap_or(0);
    let subject_raw = &clause.tokens[..mstart];
    let tail = &clause.tokens[mstart + mlen..];
    let subject = model.trim(subject_raw);
    if subject.is_empty() {
        return Err("clause has no observed subject".into());
    }
    let object = model.trim(tail);
    let question_goal = if object.is_empty() {
        model.role_goal(role)
    } else {
        None
    };
    Ok(Observation {
        seg: clause.seg,
        subject_start: clause.tokens[..mstart].len().saturating_sub(subject.len()) as u32,
        marker_start: mstart as u32,
        object_start: if object.is_empty() {
            None
        } else {
            Some((mstart + mlen + (tail.len() - object.len())) as u32)
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

/// The reusable session binding: immutable identities prepared once, validated per session.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TextBinding {
    pub model_sha256: String,
    pub tokenizer_sha256: String,
    pub world_id: u32,
    pub world_version: u32,
    pub doc_sha256: String,
}

/// Evidence retained across steps: the expanded query objective and the owned captured phrase.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TextObjective {
    pub goal: Goal,
    pub tokens: Vec<u32>,
}

/// One resumable observed-text session frame. It carries everything needed to continue without
/// rebuilding the answer or re-running an earlier read.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TextSession {
    pub binding: TextBinding,
    pub goal: Goal,
    pub query: TextObjective,
    /// Owned captured answer phrase plus its exact occurrence provenance.
    pub captured: Option<CapturedPayload>,
    pub captured_span: Option<(u32, u32, usize)>,
    pub captured_tokens: Vec<u32>,
    pub pending: RelAction,
    pub reads: u8,
    pub emitted: Vec<u32>,
    pub cursor: u8,
    pub visited: Vec<(u32, u32, usize)>,
    pub terminal: Option<RelAction>,
    /// Declared emission terminator, emitted through this same boundary.
    pub eos: Option<u32>,
}

/// What one step did, for evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepEffect {
    pub action: RelAction,
    pub selected_segment: Option<u32>,
    pub selected_role: Option<usize>,
    pub emitted: Option<u32>,
    pub terminal: Option<RelAction>,
}

impl TextSession {
    pub fn start(binding: TextBinding, goal: Goal, query: TextObjective) -> Self {
        TextSession {
            binding,
            goal,
            query,
            captured: None,
            captured_span: None,
            captured_tokens: Vec::new(),
            pending: RelAction::Read,
            reads: 0,
            emitted: Vec::new(),
            cursor: 0,
            visited: Vec::new(),
            terminal: None,
            eos: None,
        }
    }

    pub fn with_eos(mut self, eos: u32) -> Self {
        self.eos = Some(eos);
        self
    }

    pub fn validate(&self, expected: &TextBinding, max_vocab: usize) -> Result<(), String> {
        if &self.binding != expected {
            return Err("session binding does not match the loaded runtime".into());
        }
        if self.terminal.is_some() && self.pending != RelAction::Stop {
            return Err("terminal session must not carry a pending action".into());
        }
        if self.reads > OB_MAX_READS {
            return Err("session read count exceeds the declared cap".into());
        }
        if self.emitted.iter().any(|t| *t as usize >= max_vocab)
            || self.query.tokens.iter().any(|t| *t as usize >= max_vocab)
        {
            return Err("session token outside the vocabulary".into());
        }
        if self.captured.is_some() && self.captured_tokens.is_empty() {
            return Err("captured payload without an observed span".into());
        }
        Ok(())
    }

    /// **The one causal step.** Consumes the current pending action and the bound world/model, and
    /// returns the typed effect. Used by teacher forcing, generation, interventions and resumption.
    pub fn step(
        &mut self,
        model: &ObservedTextModel,
        statements: &[Observation],
    ) -> Result<StepEffect, String> {
        if let Some(t) = self.terminal {
            return Err(format!("session already terminal: {t:?}"));
        }
        match self.pending {
            RelAction::Read => {
                if self.reads >= OB_MAX_READS {
                    self.terminal = Some(RelAction::Exhausted);
                    self.pending = RelAction::Stop;
                    return Ok(StepEffect {
                        action: RelAction::Exhausted,
                        selected_segment: None,
                        selected_role: None,
                        emitted: None,
                        terminal: Some(RelAction::Exhausted),
                    });
                }
                // Exact span admission: a statement whose observed subject equals the query tokens
                // and whose learned goal equals the active goal, ranked by the learned role votes.
                let mut best: Option<(i32, usize)> = None;
                for (i, st) in statements.iter().enumerate() {
                    if st.subject != self.query.tokens {
                        continue;
                    }
                    let Some(st_goal) = model.role_goal(st.role) else {
                        continue;
                    };
                    if st_goal != self.goal || st.object.is_none() {
                        continue;
                    }
                    let votes = model.votes_of(st.marker.iter().copied().next().unwrap_or(0));
                    let score: i32 = votes.iter().map(|v| i32::from(*v)).sum();
                    if best.map(|(bs, _)| score > bs).unwrap_or(true) {
                        best = Some((score, i));
                    }
                }
                let Some((_, idx)) = best else {
                    self.terminal = Some(RelAction::Unresolved);
                    self.pending = RelAction::Stop;
                    return Ok(StepEffect {
                        action: RelAction::Unresolved,
                        selected_segment: None,
                        selected_role: None,
                        emitted: None,
                        terminal: Some(RelAction::Unresolved),
                    });
                };
                let st = &statements[idx];
                let object = st.object.clone().unwrap_or_default();
                let span = (st.seg, st.object_start.unwrap_or(0), object.len());
                if self.visited.contains(&span) {
                    self.terminal = Some(RelAction::Exhausted);
                    self.pending = RelAction::Stop;
                    return Ok(StepEffect {
                        action: RelAction::Exhausted,
                        selected_segment: Some(st.seg),
                        selected_role: Some(st.role),
                        emitted: None,
                        terminal: Some(RelAction::Exhausted),
                    });
                }
                self.visited.push(span);
                self.reads = self.reads.saturating_add(1);
                self.captured = Some(CapturedPayload {
                    seq: self.binding.world_version,
                    abs: st.object_start.unwrap_or(0),
                    payload: object.first().copied().unwrap_or(0),
                    version: self.binding.world_version,
                });
                self.captured_span = Some(span);
                self.captured_tokens = object;
                // The learned action semantics decide continue versus emit.
                let action = model.role_action(st.role);
                self.pending = action;
                Ok(StepEffect {
                    action,
                    selected_segment: Some(st.seg),
                    selected_role: Some(st.role),
                    emitted: None,
                    terminal: None,
                })
            }
            RelAction::Continue => {
                // Goal preserved; only the owned objective advances.
                self.query = TextObjective {
                    goal: self.goal,
                    tokens: self.captured_tokens.clone(),
                };
                self.pending = RelAction::Read;
                Ok(StepEffect {
                    action: RelAction::Continue,
                    selected_segment: self.captured_span.map(|s| s.0),
                    selected_role: None,
                    emitted: None,
                    terminal: None,
                })
            }
            RelAction::Emit => {
                let token = self
                    .captured_tokens
                    .get(usize::from(self.cursor))
                    .copied()
                    .ok_or("emission cursor past the captured phrase")?;
                self.emitted.push(token);
                self.cursor = self.cursor.saturating_add(1);
                if usize::from(self.cursor) >= self.captured_tokens.len() {
                    self.pending = RelAction::Stop;
                }
                Ok(StepEffect {
                    action: RelAction::Emit,
                    selected_segment: self.captured_span.map(|s| s.0),
                    selected_role: None,
                    emitted: Some(token),
                    terminal: None,
                })
            }
            RelAction::Stop => {
                // The declared terminator is emitted through the same boundary before terminality.
                let mut emitted = None;
                if let Some(eos) = self.eos {
                    if self.emitted.last() != Some(&eos) {
                        self.emitted.push(eos);
                        emitted = Some(eos);
                    }
                }
                self.terminal = Some(RelAction::Stop);
                Ok(StepEffect {
                    action: RelAction::Stop,
                    selected_segment: None,
                    selected_role: None,
                    emitted,
                    terminal: Some(RelAction::Stop),
                })
            }
            other => {
                self.terminal = Some(other);
                Ok(StepEffect {
                    action: other,
                    selected_segment: None,
                    selected_role: None,
                    emitted: None,
                    terminal: Some(other),
                })
            }
        }
    }

    /// Drive the boundary to completion under the declared cap, returning every effect.
    pub fn run(
        &mut self,
        model: &ObservedTextModel,
        statements: &[Observation],
    ) -> Result<Vec<StepEffect>, String> {
        let mut out = Vec::new();
        for _ in 0..(2 * OB_MAX_READS as usize + 4) {
            if self.terminal.is_some() {
                break;
            }
            out.push(self.step(model, statements)?);
        }
        Ok(out)
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
) -> (ObservedTextModel, ObFitReport) {
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
    (model, report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding() -> TextBinding {
        TextBinding {
            model_sha256: "m".into(),
            tokenizer_sha256: "t".into(),
            world_id: 1,
            world_version: 7,
            doc_sha256: "d".into(),
        }
    }

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
        let (model, report) = fit_observed_text_model(&clauses, &labels);
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
    fn the_goal_survives_a_redirect_and_the_span_identity_is_exact() {
        let (clauses, labels) = dev();
        let (model, _) = fit_observed_text_model(&clauses, &labels);
        // Only a redirect answers office for the subject; the terminal fact is one hop further away.
        let statements: Vec<Observation> = [1usize, 6]
            .iter()
            .map(|i| observe_clause(&model, &clauses[*i]).unwrap())
            .collect();
        let mut session = TextSession::start(
            binding(),
            Goal::Office,
            TextObjective {
                goal: Goal::Office,
                tokens: vec![10],
            },
        );
        session.validate(&binding(), 4096).unwrap();
        session.run(&model, &statements).unwrap();
        assert_eq!(session.terminal, Some(RelAction::Stop));
        assert_eq!(
            session.goal,
            Goal::Office,
            "goal preserved across the redirect"
        );
        assert_eq!(
            session.emitted,
            vec![32],
            "the terminal office assertion is emitted"
        );
        assert_eq!(session.reads, 2);
    }

    #[test]
    fn a_missing_fact_is_unresolved_and_a_cycle_is_exhausted() {
        let (clauses, labels) = dev();
        let (model, _) = fit_observed_text_model(&clauses, &labels);
        // A genuine self-redirect: the object span is the same occurrence on every read.
        let cycle_clause = Clause {
            seg: 9,
            tokens: vec![10, 22, 23, 10],
        };
        let only_redirect = vec![observe_clause(&model, &cycle_clause).unwrap()];
        let mut s = TextSession::start(
            binding(),
            Goal::Office,
            TextObjective {
                goal: Goal::Office,
                tokens: vec![10],
            },
        );
        s.run(&model, &only_redirect).unwrap();
        assert_eq!(
            s.terminal,
            Some(RelAction::Exhausted),
            "a redirect cycle exhausts"
        );

        let mut s2 = TextSession::start(
            binding(),
            Goal::Office,
            TextObjective {
                goal: Goal::Office,
                tokens: vec![99],
            },
        );
        s2.run(&model, &only_redirect).unwrap();
        assert_eq!(s2.terminal, Some(RelAction::Unresolved));
        assert!(s2.emitted.is_empty());
    }

    #[test]
    fn the_frame_round_trips_and_rejects_a_foreign_binding() {
        let (clauses, labels) = dev();
        let (model, _) = fit_observed_text_model(&clauses, &labels);
        let statements: Vec<Observation> = clauses
            .iter()
            .filter(|c| c.seg < 4)
            .map(|c| observe_clause(&model, c).unwrap())
            .collect();
        let mut session = TextSession::start(
            binding(),
            Goal::Office,
            TextObjective {
                goal: Goal::Office,
                tokens: vec![10],
            },
        );
        session.step(&model, &statements).unwrap();
        let bytes = serde_json::to_vec(&session).unwrap();
        let mut restored: TextSession = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(restored, session);
        let mut foreign = binding();
        foreign.world_id = 2;
        assert!(restored.validate(&foreign, 4096).is_err());
        let mut restored = restored.with_eos(99);
        restored.run(&model, &statements).unwrap();
        assert_eq!(
            restored.emitted,
            vec![30, 99],
            "the terminator is emitted by the boundary"
        );
    }
}
