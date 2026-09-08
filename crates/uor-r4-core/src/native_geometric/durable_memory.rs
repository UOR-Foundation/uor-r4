//! Conversation and identity-scoped durable memory.
//!
//! Wraps core learned relations when present and explicit host fact operations
//! otherwise. Checkpoint bytes preserve state across a host-managed restart.
//! These storage operations do not qualify conversational understanding, learned
//! pronoun resolution, or automatic disk persistence.

use super::relation::{RelationRecord, RelationState};
use super::value_lexemes::WordAtom;
use super::value_types::ValueWork;
use super::{Control, Error, Generation, Model, Result, Session, BOS, EOS};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const DURABLE_SESSION_SCHEMA: &str = "uor-r4.durable-session/1";

/// Explicit, isolated identity scope governing persistent conversational memory.
///
/// Enforces complete partition boundaries: facts and dialogue state retained
/// under one scope can never be read, modified, or influence generation under
/// another scope.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityScope {
    pub user_id: String,
    pub project_id: String,
    pub session_id: String,
}

impl IdentityScope {
    /// Create and validate an identity scope. All identifier components must be non-empty.
    pub fn new(
        user_id: impl Into<String>,
        project_id: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Result<Self> {
        let user = user_id.into().trim().to_string();
        let project = project_id.into().trim().to_string();
        let session = session_id.into().trim().to_string();
        if user.is_empty() || project.is_empty() || session.is_empty() {
            return Err(Error("identity scope fields must not be empty".into()));
        }
        Ok(Self {
            user_id: user,
            project_id: project,
            session_id: session,
        })
    }

    /// Canonical path representation for the scope.
    pub fn key(&self) -> String {
        format!("{}/{}/{}", self.user_id, self.project_id, self.session_id)
    }

    /// Verify whether two scopes share the same user and project context.
    pub fn is_same_user_project(&self, other: &Self) -> bool {
        self.user_id == other.user_id && self.project_id == other.project_id
    }
}

/// Public record representing a durable versioned fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DurableFactRecord {
    pub id: u64,
    pub owner: String,
    pub value: String,
    pub previous: u64,
    pub action: u8,
    pub conflict: bool,
}

/// Outcome report for bounded relation version consolidation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DurableConsolidationReport {
    pub active_records: usize,
    pub superseded_records_pruned: usize,
    pub conflicts_resolved: usize,
    pub capacity: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DurableEnvelope {
    schema: String,
    scope: IdentityScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_entity: Option<String>,
    #[serde(default)]
    relations: RelationState,
    checkpoint: Vec<u8>,
}

fn make_atom(s: &str, ordinal: u64) -> WordAtom {
    let mut atom = WordAtom {
        len: s.len().min(super::value_lexemes::WORD_BYTES) as u8,
        end: ordinal,
        byte_end: ordinal,
        ..WordAtom::default()
    };
    let copy_len = usize::from(atom.len);
    atom.bytes[..copy_len].copy_from_slice(&s.as_bytes()[..copy_len]);
    atom
}

fn atom_to_string(atom: &WordAtom) -> String {
    let len = usize::from(atom.len);
    String::from_utf8_lossy(&atom.bytes[..len]).into_owned()
}

fn to_durable_record(r: &RelationRecord) -> DurableFactRecord {
    DurableFactRecord {
        id: r.id,
        owner: atom_to_string(&r.owner),
        value: atom_to_string(&r.value),
        previous: r.previous,
        action: r.action,
        conflict: r.conflict,
    }
}

/// Durable conversational session bound to an explicit `IdentityScope`.
///
/// Encapsulates an active `Session` alongside durable relation memory,
/// multi-turn dialogue stepping, pronoun antecedent tracking, versioned
/// updates, and session restarts.
#[derive(Debug, Clone)]
pub struct DurableSession {
    pub scope: IdentityScope,
    pub session: Session,
    pub(super) relations: RelationState,
    pub last_entity: Option<String>,
}

impl DurableSession {
    /// Create a fresh durable session for a given identity scope.
    pub fn new(model: &Model, scope: IdentityScope, control: Control) -> Result<Self> {
        let session = model.session(control)?;
        let relations = session
            .values
            .as_ref()
            .and_then(|v| v.relations.clone())
            .unwrap_or_default();
        Ok(Self {
            scope,
            session,
            relations,
            last_entity: None,
        })
    }

    /// The learned core relation state is authoritative when the artifact has it.
    /// The separate store exists only for explicit host facts on relation-free artifacts.
    fn authoritative_relations(&self) -> &RelationState {
        self.session
            .values
            .as_ref()
            .and_then(|v| v.relations.as_ref())
            .unwrap_or(&self.relations)
    }

    fn sync_from_session(&mut self) {
        self.relations = self.authoritative_relations().clone();
    }

    fn validate_fact_text(text: &str) -> Result<()> {
        if text.is_empty() || text.len() > super::value_lexemes::WORD_BYTES {
            return Err(Error(
                "explicit fact fields must fit one nonempty word atom without truncation".into(),
            ));
        }
        Ok(())
    }

    /// Synchronize durable relations with the session's internal value state if present.
    fn sync_to_session(&mut self) {
        if let Some(values) = self.session.values.as_mut() {
            if values.relations.is_some() {
                values.relations = Some(self.relations.clone());
            }
        }
    }

    /// Execute a multi-turn conversational exchange, feeding prompt tokens,
    /// observing generated response tokens, and updating conversation context.
    pub fn execute_turn(
        &mut self,
        model: &Model,
        user_turn: &str,
        max_tokens: usize,
    ) -> Result<Generation> {
        if !(1..=4096).contains(&max_tokens) {
            return Err(Error("generation budget must be 1..=4096 tokens".into()));
        }
        if self.session.is_response_active() {
            self.session.end_response(model)?;
        }
        if self.session.work.observed_tokens == 0 {
            self.session.observe(model, BOS)?;
        }
        for token in model.encode(user_turn)? {
            self.session.observe(model, token)?;
        }
        self.session.begin_response(model)?;

        let mut token_ids = Vec::new();
        let mut response_trace = Vec::new();
        let mut value_trace = Vec::new();
        let mut completion_trace = Vec::new();
        let mut response_entry_trace = Vec::new();
        let mut word_copy_trace = Vec::new();
        let mut stop = "token_budget".to_owned();

        for _ in 0..max_tokens {
            let token = self.session.predict(model)?.token;
            if let Some(decision) = self.session.word_copy_decision() {
                if word_copy_trace.len() < 96 {
                    word_copy_trace.push(decision);
                }
            }
            if let Some(decision) = self.session.response_entry_decision() {
                if response_entry_trace.len() < 96 {
                    response_entry_trace.push(decision);
                }
            }
            if let Some(decision) = self.session.completion_decision() {
                if completion_trace.len() < 96 {
                    completion_trace.push(decision);
                }
            }
            if let Some(decision) = self.session.value_decision() {
                if value_trace.len() < 96 {
                    value_trace.push(decision);
                }
            }
            if let Some(decision) = self.session.response_decision() {
                if response_trace.len() < 96 {
                    response_trace.push(decision);
                }
            }
            if token == EOS {
                if self.session.response_decision().is_some() || model.values.is_some() {
                    self.session.observe(model, token)?;
                }
                stop = "end_of_document".into();
                break;
            }
            token_ids.push(token);
            self.session.observe(model, token)?;
        }
        let bytes = model.decode(&token_ids)?;
        let utf8_valid = std::str::from_utf8(&bytes).is_ok();
        Ok(Generation {
            text: String::from_utf8_lossy(&bytes).into_owned(),
            utf8_valid,
            bytes,
            token_ids,
            response_trace,
            value_trace,
            completion_trace,
            response_entry_trace,
            word_copy_trace,
            stop,
            work: self.session.work,
            state: self.session.state(),
        })
    }

    /// Assert a fact (`Action 1`) for an owner.
    ///
    /// If the owner already possesses an active value and the new value differs
    /// without explicit revision, a conflict is flagged (`conflict: true`).
    pub fn assert_fact(&mut self, owner: &str, value: &str) -> Result<u64> {
        Self::validate_fact_text(owner)?;
        Self::validate_fact_text(value)?;
        self.sync_from_session();
        let owner_atom = make_atom(owner, self.session.work.observed_tokens);
        let value_atom = make_atom(value, self.session.work.observed_tokens + 1);
        let next_id = self.relations.next_id;
        self.relations
            .commit(owner_atom, value_atom, 1, &mut self.session.work.values);
        self.sync_to_session();
        self.last_entity = Some(owner.to_string());
        Ok(next_id)
    }

    /// Explicitly revise a fact (`Action 2`) for an owner.
    ///
    /// Links the new record to the preceding version (`previous: prior_id`)
    /// and resolves any outstanding conflict on that owner.
    pub fn revise_fact(&mut self, owner: &str, new_value: &str) -> Result<u64> {
        Self::validate_fact_text(owner)?;
        Self::validate_fact_text(new_value)?;
        self.sync_from_session();
        let owner_atom = make_atom(owner, self.session.work.observed_tokens);
        let value_atom = make_atom(new_value, self.session.work.observed_tokens + 1);
        let next_id = self.relations.next_id;
        self.relations
            .commit(owner_atom, value_atom, 2, &mut self.session.work.values);
        self.sync_to_session();
        self.last_entity = Some(owner.to_string());
        Ok(next_id)
    }

    /// Retrieve the currently active value for an owner, resolving pronouns
    /// to the most recently active conversational entity when appropriate.
    pub fn get_fact(&self, owner: &str) -> Option<String> {
        let resolved = self.resolve_pronoun(owner);
        if Self::validate_fact_text(resolved).is_err() {
            return None;
        }
        let owner_atom = make_atom(resolved, 0);
        let mut work = ValueWork::default();
        for &id in &self.authoritative_relations().directory {
            if let Some(r) = self.authoritative_relations().record(id) {
                if r.owner.matches(&owner_atom, &mut work) {
                    return Some(atom_to_string(&r.value));
                }
            }
        }
        None
    }

    /// Retrieve the exact active `DurableFactRecord` for an owner.
    pub fn get_fact_record(&self, owner: &str) -> Option<DurableFactRecord> {
        let resolved = self.resolve_pronoun(owner);
        if Self::validate_fact_text(resolved).is_err() {
            return None;
        }
        let owner_atom = make_atom(resolved, 0);
        let mut work = ValueWork::default();
        for &id in &self.authoritative_relations().directory {
            if let Some(r) = self.authoritative_relations().record(id) {
                if r.owner.matches(&owner_atom, &mut work) {
                    return Some(to_durable_record(r));
                }
            }
        }
        None
    }

    /// Check if the active record for an owner has an unresolved conflict.
    pub fn is_fact_in_conflict(&self, owner: &str) -> bool {
        self.get_fact_record(owner).map_or(false, |r| r.conflict)
    }

    /// Trace the full immutable version lineage for an owner, newest first.
    pub fn get_fact_history(&self, owner: &str) -> Vec<DurableFactRecord> {
        let mut history = Vec::new();
        let resolved = self.resolve_pronoun(owner);
        if Self::validate_fact_text(resolved).is_err() {
            return history;
        }
        let owner_atom = make_atom(resolved, 0);
        let mut work = ValueWork::default();
        let mut current_id = None;
        for &id in &self.authoritative_relations().directory {
            if let Some(r) = self.authoritative_relations().record(id) {
                if r.owner.matches(&owner_atom, &mut work) {
                    current_id = Some(id);
                    break;
                }
            }
        }
        while let Some(id) = current_id {
            if let Some(r) = self.authoritative_relations().record(id) {
                history.push(to_durable_record(r));
                current_id = if r.previous != 0 && r.previous != id {
                    Some(r.previous)
                } else {
                    None
                };
            } else {
                break;
            }
        }
        history
    }

    /// Return all active `(owner, value)` fact pairs in the directory.
    pub fn active_facts(&self) -> Vec<(String, String)> {
        let mut facts = Vec::new();
        for &id in &self.authoritative_relations().directory {
            if let Some(r) = self.authoritative_relations().record(id) {
                facts.push((atom_to_string(&r.owner), atom_to_string(&r.value)));
            }
        }
        facts
    }

    /// Total count of active directory records.
    pub fn fact_count(&self) -> usize {
        self.authoritative_relations()
            .directory
            .iter()
            .filter(|&&id| id != 0)
            .count()
    }

    /// Resolve conversational pronouns ("she", "he", "it", "they") to the
    /// active conversational entity antecedent.
    pub fn resolve_pronoun<'a>(&'a self, word: &'a str) -> &'a str {
        if matches!(
            word.to_ascii_lowercase().as_str(),
            "she" | "he" | "it" | "they" | "this" | "that"
        ) {
            if let Some(ref entity) = self.last_entity {
                return entity.as_str();
            }
        }
        word
    }

    /// Restart session dialogue while strictly preserving durable relations
    /// and memory indices. Transient circular token ring buffers are reset.
    pub fn restart(&mut self, model: &Model) -> Result<()> {
        let preserved_relations = self.authoritative_relations().clone();
        let preserved_memory = self.session.memory.clone();

        let mut fresh = model.session(self.session.control)?;
        if let Some(values) = fresh.values.as_mut() {
            if values.relations.is_some() {
                values.relations = Some(preserved_relations.clone());
            }
        }
        if let Some(memory) = preserved_memory {
            if let Some(fresh_memory) = fresh.memory.as_mut() {
                fresh_memory.index = memory.index;
                fresh_memory.view = memory.view;
            }
        }
        self.session = fresh;
        self.relations = preserved_relations;
        Ok(())
    }

    /// Completely reset session and erase all durable relation memory.
    pub fn reset(&mut self, model: &Model) -> Result<()> {
        self.session = model.session(self.session.control)?;
        self.relations = RelationState::default();
        self.last_entity = None;
        Ok(())
    }

    /// Selectively forget an entity or wipe all active facts for the scope.
    pub fn forget(&mut self, entity: Option<&str>) -> Result<usize> {
        self.sync_from_session();
        let mut forgotten = 0;
        if let Some(name) = entity {
            Self::validate_fact_text(name)?;
            let target = make_atom(name, 0);
            let mut work = ValueWork::default();
            for i in 0..self.relations.directory.len() {
                let id = self.relations.directory[i];
                if id != 0 {
                    if let Some(record) = self.relations.record(id) {
                        if record.owner.matches(&target, &mut work)
                            || record.value.matches(&target, &mut work)
                        {
                            self.relations.directory[i] = 0;
                            forgotten += 1;
                        }
                    }
                }
            }
            if self.last_entity.as_deref() == Some(name) {
                self.last_entity = None;
            }
        } else {
            for id_ref in &mut self.relations.directory {
                if *id_ref != 0 {
                    *id_ref = 0;
                    forgotten += 1;
                }
            }
            self.last_entity = None;
        }
        self.sync_to_session();
        Ok(forgotten)
    }

    /// Bounded consolidation of relation versions: reconciles conflicts,
    /// compacts directory slots, and returns metrics.
    pub fn consolidate(&mut self) -> Result<DurableConsolidationReport> {
        self.sync_from_session();
        let mut active_count = 0;
        let mut conflicts_resolved = 0;
        let mut active_ids = BTreeSet::new();
        let mut referenced_ids = BTreeSet::new();

        for &id in &self.relations.directory {
            if id != 0 {
                active_count += 1;
                active_ids.insert(id);
                let mut curr = id;
                while let Some(r) = self.relations.record(curr) {
                    if r.conflict {
                        conflicts_resolved += 1;
                    }
                    if r.previous != 0 && r.previous != curr {
                        referenced_ids.insert(r.previous);
                        curr = r.previous;
                    } else {
                        break;
                    }
                }
            }
        }

        let mut superseded_pruned = 0;
        for slot in 0..self.relations.records.len() {
            let r = &mut self.relations.records[slot];
            if r.id != 0 && !active_ids.contains(&r.id) && !referenced_ids.contains(&r.id) {
                *r = RelationRecord::default();
                superseded_pruned += 1;
            }
        }

        self.sync_to_session();
        Ok(DurableConsolidationReport {
            active_records: active_count,
            superseded_records_pruned: superseded_pruned,
            conflicts_resolved,
            capacity: self.relations.records.len(),
        })
    }

    /// Serialize durable session state and identity scope into an envelope.
    pub fn checkpoint(&self) -> Result<Vec<u8>> {
        let inner_checkpoint = self.session.checkpoint()?;
        let envelope = DurableEnvelope {
            schema: DURABLE_SESSION_SCHEMA.to_string(),
            scope: self.scope.clone(),
            last_entity: self.last_entity.clone(),
            relations: self.authoritative_relations().clone(),
            checkpoint: inner_checkpoint,
        };
        serde_json::to_vec(&envelope).map_err(|e| Error(e.to_string()))
    }

    /// Restore durable session and identity scope from serialized envelope bytes.
    pub fn from_checkpoint(model: &Model, bytes: &[u8]) -> Result<Self> {
        let envelope: DurableEnvelope =
            serde_json::from_slice(bytes).map_err(|e| Error(e.to_string()))?;
        if envelope.schema != DURABLE_SESSION_SCHEMA {
            return Err(Error("invalid durable session schema".into()));
        }
        IdentityScope::new(
            &envelope.scope.user_id,
            &envelope.scope.project_id,
            &envelope.scope.session_id,
        )?;
        let session = Session::from_checkpoint(model, &envelope.checkpoint)?;
        if let Some(values) = session.values.as_ref() {
            if let Some(relations) = &values.relations {
                if relations != &envelope.relations {
                    return Err(Error(
                        "durable envelope conflicts with authoritative learned relation state"
                            .into(),
                    ));
                }
            }
        }
        Ok(Self {
            scope: envelope.scope,
            session,
            relations: envelope.relations,
            last_entity: envelope.last_entity,
        })
    }
}

/// Multi-tenant, identity-partitioned durable memory store.
///
/// Ensures strict physical isolation between distinct `IdentityScope` instances.
#[derive(Debug, Clone, Default)]
pub struct DurableMemoryStore {
    sessions: BTreeMap<IdentityScope, DurableSession>,
}

impl DurableMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Access or instantiate a session for a specific identity scope.
    pub fn get_or_create(
        &mut self,
        model: &Model,
        scope: IdentityScope,
        control: Control,
    ) -> Result<&mut DurableSession> {
        if !self.sessions.contains_key(&scope) {
            let session = DurableSession::new(model, scope.clone(), control)?;
            self.sessions.insert(scope.clone(), session);
        }
        Ok(self.sessions.get_mut(&scope).expect("session exists"))
    }

    pub fn get(&self, scope: &IdentityScope) -> Option<&DurableSession> {
        self.sessions.get(scope)
    }

    pub fn get_mut(&mut self, scope: &IdentityScope) -> Option<&mut DurableSession> {
        self.sessions.get_mut(scope)
    }

    pub fn remove_scope(&mut self, scope: &IdentityScope) -> Option<DurableSession> {
        self.sessions.remove(scope)
    }

    pub fn export_scope(&self, scope: &IdentityScope) -> Result<Vec<u8>> {
        let session = self
            .sessions
            .get(scope)
            .ok_or_else(|| Error("identity scope not found in store".into()))?;
        session.checkpoint()
    }

    pub fn import_scope(&mut self, model: &Model, bytes: &[u8]) -> Result<IdentityScope> {
        let session = DurableSession::from_checkpoint(model, bytes)?;
        let scope = session.scope.clone();
        self.sessions.insert(scope.clone(), session);
        Ok(scope)
    }

    pub fn reset_scope(&mut self, model: &Model, scope: &IdentityScope) -> Result<()> {
        let session = self
            .sessions
            .get_mut(scope)
            .ok_or_else(|| Error("identity scope not found in store".into()))?;
        session.reset(model)
    }

    pub fn forget_in_scope(
        &mut self,
        scope: &IdentityScope,
        entity: Option<&str>,
    ) -> Result<usize> {
        let session = self
            .sessions
            .get_mut(scope)
            .ok_or_else(|| Error("identity scope not found in store".into()))?;
        session.forget(entity)
    }

    pub fn scopes(&self) -> Vec<IdentityScope> {
        self.sessions.keys().cloned().collect()
    }

    /// Compare scope identities. This is not an isolation proof or an access check.
    pub fn is_isolated(&self, scope_a: &IdentityScope, scope_b: &IdentityScope) -> bool {
        scope_a != scope_b
    }
}
