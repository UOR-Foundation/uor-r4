//! Unified Capability API for Native UOR-R4 Integer Serving.
//!
//! Provides a standardized, thread-safe interface for host applications,
//! background services, and WebAssembly browser environments (Pages Studio).
//! Absorbs interface obligations from #962, #1172, and #963.
//! Strictly 100% safe Rust (`#![forbid(unsafe_code)]`).

use crate::bundle::Bundle;
use crate::config::ReadMode;
use crate::model::{SessionState, SlotTarget, PERSISTENT_CAPACITY};
use crate::sampling::{SamplePolicy, Sampler};
use crate::session::{RoleTokens, SerializedChatSession};
use crate::{invalid, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

pub const CAPABILITY_API_SCHEMA: &str = "uor-r4.integer-capability-api/1";

/// Truthful record of model capability statuses distinguishing
/// verified milestones from unproven claims.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityTruthMatrix {
    pub language_prose: String,
    pub causal_attention: String,
    pub multi_step_reasoning: String,
    pub executable_coding: String,
    pub durable_memory: String,
    pub serving_guarantees: String,
    pub m1_performance_profile: String,
    pub general_ai_disavowal: String,
}

impl Default for CapabilityTruthMatrix {
    fn default() -> Self {
        Self {
            language_prose: "Pre-alpha: grounded multi-turn dialogue and prompt response verified on fixed 58-turn panel; general open-domain prose unproven.".into(),
            causal_attention: "Verified: exact prime-addressed memory and discrete circular ring buffer; zero soft-attention matrices.".into(),
            multi_step_reasoning: "UNQUALIFIED: generalized reasoning not established.".into(),
            executable_coding: "UNQUALIFIED: general coding not established.".into(),
            durable_memory: "Verified: 256-slot prime memory hierarchy (persistent system persona + active dialogue ring).".into(),
            serving_guarantees: "D11 Certified: strictly 0 transformers, 0 hardware integer multipliers, 0 hardware dividers, 0 floats in serving.".into(),
            m1_performance_profile: "M1 Qualified: mean step latency 3.16 ms/tok, peak RSS < 30 MB, 1.737 MiB/tok analytical traffic.".into(),
            general_ai_disavowal: "Pre-alpha: general prose, general reasoning, and frontier capability remain unproven.".into(),
        }
    }
}

/// Metadata describing the loaded native integer model and its architectural invariants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelCapabilityMetadata {
    pub api_schema_version: String,
    pub bundle_identity: String,
    pub width: usize,
    pub vocab_size: usize,
    pub context_capacity: usize,
    pub read_geometry: String,
    pub zero_transformers: bool,
    pub zero_hardware_multipliers: bool,
    pub zero_hardware_dividers: bool,
    pub zero_floats_in_serving: bool,
    pub truth_matrix: CapabilityTruthMatrix,
}

/// Configuration for creating a managed integer chat session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionConfig {
    pub system_prompt: Option<String>,
    pub seed: u64,
    pub read_mode: ReadMode,
    pub policy: SamplePolicy,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            system_prompt: None,
            seed: 0,
            read_mode: ReadMode::Enabled,
            policy: SamplePolicy::Greedy,
        }
    }
}

/// Session telemetry snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionTelemetry {
    pub session_id: u32,
    pub active_tokens: usize,
    pub context_capacity: usize,
    pub read_mode: ReadMode,
    pub policy: SamplePolicy,
    pub prng_state: u64,
}

/// Owned conversational session state managed by the API without unsafe lifetimes.
pub struct ManagedSession {
    pub state: SessionState,
    pub roles: RoleTokens,
    pub sampler: Sampler,
    pub read_mode: ReadMode,
    pub policy: SamplePolicy,
}

/// Unified capability API container managing bundle lifetime and session state.
pub struct IntegerCapabilityApi {
    bundle: Bundle,
    sessions: Mutex<HashMap<u32, ManagedSession>>,
    next_session_id: AtomicU32,
}

impl IntegerCapabilityApi {
    /// Create a new Capability API instance over a loaded bundle.
    pub fn new(bundle: Bundle) -> Self {
        Self {
            bundle,
            sessions: Mutex::new(HashMap::new()),
            next_session_id: AtomicU32::new(1),
        }
    }

    /// Access the underlying bundle identity and configuration metadata.
    pub fn metadata(&self) -> ModelCapabilityMetadata {
        let config = self.bundle.model().config();
        ModelCapabilityMetadata {
            api_schema_version: CAPABILITY_API_SCHEMA.into(),
            bundle_identity: self.bundle.identity().into(),
            width: config.width,
            vocab_size: config.vocab_size,
            context_capacity: config.context,
            read_geometry: format!("{:?}", self.bundle.model().config().read_geometry),
            zero_transformers: true,
            zero_hardware_multipliers: true,
            zero_hardware_dividers: true,
            zero_floats_in_serving: true,
            truth_matrix: CapabilityTruthMatrix::default(),
        }
    }

    /// Instantiate a new managed conversational session with configuration.
    pub fn create_session(&self, config: SessionConfig) -> Result<u32> {
        let session_id = self.next_session_id.fetch_add(1, Ordering::SeqCst);
        let roles = RoleTokens::from_tokenizer(self.bundle.tokenizer())?;
        let mut state = self.bundle.model().new_conversational_session();
        if self.bundle.model().is_lorentz() {
            state.enable_hyperbolic_cache();
        }

        let mut managed = ManagedSession {
            state,
            roles,
            sampler: Sampler::new(config.seed),
            read_mode: config.read_mode,
            policy: config.policy,
        };

        if let Some(prompt) = config.system_prompt {
            if !prompt.trim().is_empty() {
                let text_tokens = self.bundle.tokenizer().encode(&prompt);
                let mut tokens = Vec::with_capacity(text_tokens.len() + 2);
                tokens.push(managed.roles.system_id);
                tokens.extend(text_tokens);
                tokens.push(managed.roles.turn_end_id);

                if tokens.len() > PERSISTENT_CAPACITY {
                    return Err(invalid(format!(
                        "system prompt length {} exceeds persistent slot capacity {}",
                        tokens.len(),
                        PERSISTENT_CAPACITY
                    )));
                }
                for &token in &tokens {
                    self.bundle.model().step_conversational_into(
                        &mut managed.state,
                        token,
                        SlotTarget::Persistent,
                        ReadMode::Enabled,
                    )?;
                }
                managed.state.seal_persistent();
            }
        }

        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| invalid("session lock poisoned"))?;
        lock.insert(session_id, managed);
        Ok(session_id)
    }

    /// Ingest a user prompt into the specified session.
    pub fn ingest_user_turn(&self, session_id: u32, prompt: &str) -> Result<usize> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| invalid("session lock poisoned"))?;
        let session = lock
            .get_mut(&session_id)
            .ok_or_else(|| invalid(format!("session {session_id} not found")))?;

        session.state.start_turn();
        if prompt.trim().is_empty() {
            return Err(invalid("empty user turn prompt"));
        }
        let user_tokens = self.bundle.tokenizer().encode(prompt);
        let mut tokens = Vec::with_capacity(user_tokens.len() + 3);
        tokens.push(session.roles.user_id);
        tokens.extend(user_tokens);
        tokens.push(session.roles.turn_end_id);
        tokens.push(session.roles.assistant_id);

        for &token in &tokens {
            self.bundle.model().step_conversational_into(
                &mut session.state,
                token,
                SlotTarget::Dialogue,
                session.read_mode,
            )?;
        }
        Ok(tokens.len())
    }

    /// Execute a single step with a token ID and return the next selected token.
    pub fn step_token(&self, session_id: u32, token_id: u32) -> Result<u32> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| invalid("session lock poisoned"))?;
        let session = lock
            .get_mut(&session_id)
            .ok_or_else(|| invalid(format!("session {session_id} not found")))?;

        self.bundle.model().step_conversational_into(
            &mut session.state,
            token_id,
            SlotTarget::Dialogue,
            session.read_mode,
        )?;

        let vocab_size = self.bundle.model().config().vocab_size;
        let probs = &session.state.last_probabilities[..vocab_size];
        let selected = session
            .sampler
            .select(probs, session.policy)
            .map_err(|e| invalid(format!("sampling error: {e}")))?;
        Ok(selected as u32)
    }

    /// Generate an assistant response turn up to `max_tokens`.
    pub fn generate_text(&self, session_id: u32, max_tokens: usize) -> Result<String> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| invalid("session lock poisoned"))?;
        let session = lock
            .get_mut(&session_id)
            .ok_or_else(|| invalid(format!("session {session_id} not found")))?;

        let vocab_size = self.bundle.model().config().vocab_size;
        let turn_end_id = session.roles.turn_end_id;
        let eos_id = session.roles.eos_id;

        let mut generated_tokens = Vec::with_capacity(max_tokens);
        for _ in 0..max_tokens {
            let probs = &session.state.last_probabilities[..vocab_size];
            let next_tok = session
                .sampler
                .select(probs, session.policy)
                .map_err(|e| invalid(format!("sampling error: {e}")))?
                as u32;

            if next_tok == turn_end_id || next_tok == eos_id {
                let _ = self.bundle.model().step_conversational_into(
                    &mut session.state,
                    next_tok,
                    SlotTarget::Dialogue,
                    session.read_mode,
                );
                break;
            }

            generated_tokens.push(next_tok);
            self.bundle.model().step_conversational_into(
                &mut session.state,
                next_tok,
                SlotTarget::Dialogue,
                session.read_mode,
            )?;
        }

        Ok(self.bundle.tokenizer().decode(&generated_tokens))
    }

    /// Save session state for durable memory persistence.
    pub fn save_session(&self, session_id: u32) -> Result<SerializedChatSession> {
        let lock = self
            .sessions
            .lock()
            .map_err(|_| invalid("session lock poisoned"))?;
        let session = lock
            .get(&session_id)
            .ok_or_else(|| invalid(format!("session {session_id} not found")))?;

        let temp_session = crate::session::ChatSession::from_parts(
            &self.bundle,
            session.state.clone(),
            session.roles,
            session.sampler.clone(),
            session.read_mode,
            session.policy,
        );
        Ok(temp_session.to_serialized(Some(self.bundle.identity())))
    }

    /// Restore session state from serialized DTO.
    pub fn restore_session(&self, serialized: SerializedChatSession) -> Result<u32> {
        let session_id = self.next_session_id.fetch_add(1, Ordering::SeqCst);
        let restored = crate::session::ChatSession::from_serialized(
            &self.bundle,
            serialized,
            self.bundle.identity(),
        )?;

        let managed = ManagedSession {
            state: restored.into_state(),
            roles: RoleTokens::from_tokenizer(self.bundle.tokenizer())?,
            sampler: Sampler::new(0),
            read_mode: ReadMode::Enabled,
            policy: SamplePolicy::Greedy,
        };

        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| invalid("session lock poisoned"))?;
        lock.insert(session_id, managed);
        Ok(session_id)
    }

    /// Inspect session telemetry.
    pub fn get_telemetry(&self, session_id: u32) -> Result<SessionTelemetry> {
        let lock = self
            .sessions
            .lock()
            .map_err(|_| invalid("session lock poisoned"))?;
        let session = lock
            .get(&session_id)
            .ok_or_else(|| invalid(format!("session {session_id} not found")))?;
        Ok(SessionTelemetry {
            session_id,
            active_tokens: session.state.dialogue_len(),
            context_capacity: self.bundle.model().config().context,
            read_mode: session.read_mode,
            policy: session.policy,
            prng_state: session.sampler.state(),
        })
    }

    /// Free a session handle and release memory.
    pub fn close_session(&self, session_id: u32) -> bool {
        if let Ok(mut lock) = self.sessions.lock() {
            lock.remove(&session_id).is_some()
        } else {
            false
        }
    }
}
