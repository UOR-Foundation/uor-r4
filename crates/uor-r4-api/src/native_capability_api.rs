//! Native capability API and WASM model runtime.
//!
//! Provides a common artifact and session contract across Rust library consumers,
//! background CLI/services, and browser-compatible WASM execution.
//!
//! Absorbs interface obligations from #962 and #1084, exposing:
//! 1. Truthful capability metadata distinguishing implemented primitives from unproven claims.
//! 2. Thread-safe model container with byte-slice artifact loading.
//! 3. Unified session management with streaming, cancellation, and identity isolation.
//! 4. Filesystem-free byte loading and bounded incremental generation.
//! Cross-target parity and model capability require separate executed evidence.

use blake3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use uor_r4_core::native_geometric::durable_memory::{
    DurableFactRecord, DurableSession, IdentityScope,
};
use uor_r4_core::native_geometric::{Control, Model, BOS, EOS, SCHEMA};

pub const NATIVE_API_SCHEMA: &str = "uor-r4.native-capability-api/2";
pub const BACKEND_IDENTIFIER: &str = "native-geometric-language-v1";

// ============================================================================
// 1. Typed Errors
// ============================================================================

/// Typed errors produced by the native capability API and runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeApiError {
    ModelLoad(String),
    SchemaMismatch { expected: String, found: String },
    SessionNotFound(u32),
    SessionBusy(String),
    Cancelled,
    ContextOverflow { tokens: usize, limit: usize },
    TokenLimitExceeded { tokens: usize, limit: usize },
    IdentityViolation(String),
    Serialization(String),
    ResourceLimit(String),
    InvalidRequest(String),
}

impl std::fmt::Display for NativeApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModelLoad(msg) => write!(f, "Model loading error: {msg}"),
            Self::SchemaMismatch { expected, found } => {
                write!(f, "Schema mismatch: expected '{expected}', found '{found}'")
            }
            Self::SessionNotFound(id) => write!(f, "Session handle {id} not found"),
            Self::SessionBusy(msg) => write!(f, "Session busy: {msg}"),
            Self::Cancelled => write!(f, "Operation was cancelled by caller"),
            Self::ContextOverflow { tokens, limit } => write!(
                f,
                "Context length {tokens} exceeds maximum allowed context {limit}"
            ),
            Self::TokenLimitExceeded { tokens, limit } => {
                write!(f, "Generated tokens {tokens} exceeds request limit {limit}")
            }
            Self::IdentityViolation(msg) => write!(f, "Identity scope isolation violation: {msg}"),
            Self::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            Self::ResourceLimit(msg) => write!(f, "Resource limit exceeded: {msg}"),
            Self::InvalidRequest(msg) => write!(f, "Invalid API request: {msg}"),
        }
    }
}

impl std::error::Error for NativeApiError {}

// ============================================================================
// 2. Capability Metadata & Truth Matrix
// ============================================================================

/// Truthful record of model capability statuses.
/// Explicitly distinguishes qualified milestones from unproven claims.
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
            language_prose: "UNQUALIFIED: general prose is not established".into(),
            causal_attention: "Artifact-dependent bounded context and exact retained copying; consult artifact evidence".into(),
            multi_step_reasoning: "UNQUALIFIED: generalized multi-step reasoning".into(),
            executable_coding: "UNQUALIFIED: general Rust synthesis and workspace repair".into(),
            durable_memory: "Explicit scoped checkpoint API; learned memory depends on the supplied artifact".into(),
            serving_guarantees: "Native geometric model path; wrapper allocates and has no blanket proof claim".into(),
            m1_performance_profile: "NOT_MEASURED: no complete-path energy or comparative performance qualification".into(),
            general_ai_disavowal: "Pre-alpha; general prose, general reasoning and frontier capability remain unproven".into(),
        }
    }
}

/// Metadata describing the loaded native model, its identity, and supported capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeModelMetadata {
    /// Version of the API wire contract; separate from the artifact schema.
    pub api_schema_version: String,
    pub schema_version: String,
    pub model_cid: String,
    pub canonical_uor_address: String,
    pub backend_name: String,
    pub max_context_tokens: usize,
    pub max_sessions: usize,
    pub is_provider_free: bool,
    pub zero_matmul_serving: bool,
    pub zero_heap_alloc_hot_path: bool,
    pub supported_modalities: Vec<String>,
    pub truth_matrix: CapabilityTruthMatrix,
}

// ============================================================================
// 3. Request / Response Contracts
// ============================================================================

/// Configuration for instantiating a native session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_id: String,
    pub user_id: String,
    pub project_id: String,
    pub control: Control,
    pub max_output_tokens: usize,
    pub temperature: f64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_id: "default-session".into(),
            user_id: "default-user".into(),
            project_id: "default-project".into(),
            control: Control::Full,
            max_output_tokens: 128,
            temperature: 0.0,
        }
    }
}

/// Completion request payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub prompt: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f64>,
    pub stop_sequences: Vec<String>,
}

impl CompletionRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            max_tokens: None,
            temperature: None,
            stop_sequences: Vec::new(),
        }
    }
}

/// Completion response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub text: String,
    pub token_count: usize,
    pub stopped_by: String, // "eos", "length", "stop_sequence", "cancelled"
    /// None on targets without an implemented monotonic host timer.
    pub elapsed_us: Option<u64>,
    /// None: the wrapper does not measure actual causal memory reads.
    pub memory_facts_read: Option<usize>,
}

/// Receipt confirming text ingestion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestReceipt {
    pub ingested_bytes: usize,
    pub total_context_tokens: usize,
    pub text_cid: String,
}

// ============================================================================
// 4. Native Model Container
// ============================================================================

/// Thread-safe container managing a validated native geometric model.
#[derive(Clone)]
pub struct NativeModel {
    model: Arc<Model>,
    metadata: NativeModelMetadata,
}

impl NativeModel {
    /// Kept for source compatibility; serving requires an explicit artifact.
    pub fn default_baseline() -> Result<Self, NativeApiError> {
        Err(NativeApiError::InvalidRequest(
            "an explicit native model artifact is required".into(),
        ))
    }

    /// Canonical UOR address of the model.
    pub fn canonical_address(&self) -> &str {
        &self.metadata.canonical_uor_address
    }

    /// Model artifact CID digest.
    pub fn artifact_cid(&self) -> &str {
        &self.metadata.model_cid
    }

    /// Load a native model from bytes.
    pub fn load_from_bytes(bytes: &[u8]) -> Result<Self, NativeApiError> {
        if bytes.is_empty() {
            return Err(NativeApiError::InvalidRequest(
                "an explicit native model artifact is required; empty bytes are not a model".into(),
            ));
        }
        let model = Model::from_bytes(bytes).map_err(|e| NativeApiError::ModelLoad(e.0))?;
        Self::from_model(Arc::new(model))
    }

    pub fn from_model(model: Arc<Model>) -> Result<Self, NativeApiError> {
        model
            .config()
            .validate()
            .map_err(|e| NativeApiError::ModelLoad(e.0))?;
        let metadata = NativeModelMetadata {
            api_schema_version: NATIVE_API_SCHEMA.into(),
            schema_version: SCHEMA.into(),
            model_cid: model.artifact_cid().into(),
            canonical_uor_address: model.uor_model_address().into(),
            backend_name: BACKEND_IDENTIFIER.into(),
            max_context_tokens: model.config().context_tokens,
            max_sessions: 32,
            is_provider_free: true,
            zero_matmul_serving: true,
            zero_heap_alloc_hot_path: false,
            supported_modalities: vec!["text".into()],
            truth_matrix: CapabilityTruthMatrix::default(),
        };
        Ok(Self { model, metadata })
    }

    pub fn metadata(&self) -> &NativeModelMetadata {
        &self.metadata
    }

    pub fn inner_model(&self) -> Arc<Model> {
        Arc::clone(&self.model)
    }

    /// Create an isolated native session with the specified configuration.
    pub fn create_session(&self, config: SessionConfig) -> Result<NativeSession, NativeApiError> {
        validate_budget(config.max_output_tokens)?;
        validate_temperature(config.temperature)?;
        let scope = IdentityScope::new(&config.user_id, &config.project_id, &config.session_id)
            .map_err(|e| NativeApiError::IdentityViolation(e.0))?;

        let durable_session = DurableSession::new(&self.model, scope.clone(), config.control)
            .map_err(|e| NativeApiError::ModelLoad(e.0))?;

        Ok(NativeSession {
            model: Arc::clone(&self.model),
            config,
            scope,
            durable_session,
            cancelled: Arc::new(AtomicBool::new(false)),
            active: Arc::new(AtomicBool::new(false)),
            generation: GenerationState::default(),
        })
    }
}

// ============================================================================
// 5. Native Session & Streaming Execution
// ============================================================================

/// An isolated conversation and execution session over the native model.
pub struct NativeSession {
    model: Arc<Model>,
    config: SessionConfig,
    scope: IdentityScope,
    durable_session: DurableSession,
    cancelled: Arc<AtomicBool>,
    active: Arc<AtomicBool>,
    generation: GenerationState,
}

const MAX_OUTPUT_TOKENS: usize = 4096;
const MAX_INGEST_BYTES: usize = 1024 * 1024;

fn validate_budget(tokens: usize) -> Result<(), NativeApiError> {
    if !(1..=MAX_OUTPUT_TOKENS).contains(&tokens) {
        return Err(NativeApiError::TokenLimitExceeded {
            tokens,
            limit: MAX_OUTPUT_TOKENS,
        });
    }
    Ok(())
}
fn validate_temperature(temperature: f64) -> Result<(), NativeApiError> {
    if temperature != 0.0 {
        return Err(NativeApiError::InvalidRequest(
            "only deterministic temperature 0 is implemented".into(),
        ));
    }
    Ok(())
}

fn utf8_prefix(bytes: &[u8], terminal: bool) -> Result<usize, NativeApiError> {
    match std::str::from_utf8(bytes) {
        Ok(_) => Ok(bytes.len()),
        Err(e) if e.error_len().is_none() && !terminal => Ok(e.valid_up_to()),
        Err(_) => Err(NativeApiError::Serialization(
            "model emitted invalid or incomplete UTF-8".into(),
        )),
    }
}

#[derive(Default)]
struct GenerationState {
    started: bool,
    finished: bool,
    pending: Vec<u8>,
    stop_sequences: Vec<String>,
    total_tokens: usize,
}

impl NativeSession {
    pub fn session_id(&self) -> &str {
        &self.config.session_id
    }

    pub fn identity_scope(&self) -> &IdentityScope {
        &self.scope
    }

    /// Ingest text into the session context.
    pub fn ingest(&mut self, text: &str) -> Result<IngestReceipt, NativeApiError> {
        let bytes_len = text.len();
        let cid = blake3::hash(text.as_bytes()).to_hex().to_string();
        if text.len() > MAX_INGEST_BYTES {
            return Err(NativeApiError::ResourceLimit(
                "ingest exceeds 1 MiB request limit".into(),
            ));
        }
        // Input capture is disabled while the core is in response mode. End it
        // even after restoring a checkpoint whose wrapper state is not present.
        if self.generation.started || self.durable_session.session.needs_input_boundary() {
            self.durable_session
                .session
                .end_response(&self.model)
                .map_err(|e| NativeApiError::ModelLoad(e.0))?;
        }
        self.generation = GenerationState::default();
        self.cancelled.store(false, Ordering::SeqCst);

        if self.durable_session.session.work.observed_tokens == 0 {
            self.durable_session
                .session
                .observe(&self.model, BOS)
                .map_err(|e| NativeApiError::ModelLoad(e.0))?;
        }

        let tokens = self
            .model
            .encode(text)
            .map_err(|e| NativeApiError::ModelLoad(e.0))?;

        for token in tokens {
            self.durable_session
                .session
                .observe(&self.model, token)
                .map_err(|e| NativeApiError::ModelLoad(e.0))?;
        }

        Ok(IngestReceipt {
            ingested_bytes: bytes_len,
            total_context_tokens: self.durable_session.session.work.observed_tokens as usize,
            text_cid: cid,
        })
    }

    /// Generate completion synchronously.
    pub fn complete(
        &mut self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, NativeApiError> {
        self.complete_stream(request, |_| true)
    }

    /// Generate directly through the loaded model. The callback receives valid UTF-8
    /// chunks, which may span multiple model tokens. No canned replies are added.
    pub fn complete_stream<F: FnMut(&str) -> bool>(
        &mut self,
        request: CompletionRequest,
        mut callback: F,
    ) -> Result<CompletionResponse, NativeApiError> {
        let max_tokens = request.max_tokens.unwrap_or(self.config.max_output_tokens);
        validate_budget(max_tokens)?;
        validate_temperature(request.temperature.unwrap_or(self.config.temperature))?;
        if request.stop_sequences.len() > 16
            || request
                .stop_sequences
                .iter()
                .any(|s| s.is_empty() || s.len() > 256)
        {
            return Err(NativeApiError::InvalidRequest(
                "stop sequences must be 1..=256 bytes, at most 16".into(),
            ));
        }
        if !request.prompt.is_empty() {
            self.ingest(&request.prompt)?;
        }
        self.generation = GenerationState {
            stop_sequences: request.stop_sequences,
            ..Default::default()
        };
        self.cancelled.store(false, Ordering::SeqCst);
        let mut result = self.generate_chunk(max_tokens, &mut callback)?;
        if result.stopped_by == "length" {
            // A complete request ends at its budget; a possible stop prefix is
            // ordinary output if the stop was not completed.
            let tail = std::str::from_utf8(&self.generation.pending).map_err(|_| {
                NativeApiError::Serialization("token budget ended within a UTF-8 scalar".into())
            })?;
            if !tail.is_empty() {
                result.text.push_str(tail);
                if !callback(tail) {
                    result.stopped_by = "cancelled".into();
                }
            }
            self.generation.pending.clear();
            self.generation.finished = true;
        }
        Ok(result)
    }

    /// Continue a bounded response across host event-loop yields. `length` is
    /// resumable; `eos`, `cancelled`, and `stop_sequence` are terminal.
    fn generate_chunk<F: FnMut(&str) -> bool>(
        &mut self,
        max_tokens: usize,
        mut callback: F,
    ) -> Result<CompletionResponse, NativeApiError> {
        validate_budget(max_tokens)?;
        if self.active.swap(true, Ordering::SeqCst) {
            return Err(NativeApiError::SessionBusy("session is executing".into()));
        }
        let result = self.generate_chunk_inner(max_tokens, &mut callback);
        // Recoverable model/UTF-8 errors must not permanently mark a session busy.
        self.active.store(false, Ordering::SeqCst);
        result
    }

    fn generate_chunk_inner<F: FnMut(&str) -> bool>(
        &mut self,
        max_tokens: usize,
        callback: &mut F,
    ) -> Result<CompletionResponse, NativeApiError> {
        #[cfg(not(target_arch = "wasm32"))]
        let start = Instant::now();
        let mut text = String::new();
        let mut token_count = 0;
        let mut stopped_by = "length";
        if !self.generation.started && !self.generation.finished {
            self.durable_session
                .session
                .begin_response(&self.model)
                .map_err(|e| NativeApiError::ModelLoad(e.0))?;
            self.generation.started = true;
        }
        if self.generation.finished {
            stopped_by = "eos";
        }
        while token_count < max_tokens && !self.generation.finished {
            if self.cancelled.load(Ordering::Relaxed) {
                stopped_by = "cancelled";
                self.generation.finished = true;
                break;
            }
            if self.generation.total_tokens >= MAX_OUTPUT_TOKENS {
                stopped_by = "token_limit";
                self.generation.finished = true;
                break;
            }
            let pred = self
                .durable_session
                .session
                .predict(&self.model)
                .map_err(|e| NativeApiError::ModelLoad(e.0))?;
            // Observe every chosen token, including EOS, before exposing the result.
            self.durable_session
                .session
                .observe(&self.model, pred.token)
                .map_err(|e| NativeApiError::ModelLoad(e.0))?;
            if pred.token == EOS {
                stopped_by = "eos";
                self.generation.finished = true;
            } else {
                let bytes = self
                    .model
                    .decode(&[pred.token])
                    .map_err(|e| NativeApiError::ModelLoad(e.0))?;
                self.generation.pending.extend_from_slice(&bytes);
                token_count += 1;
                self.generation.total_tokens += 1;
            }
            let mut emit_len = self.generation.pending.len();
            if let Some(pos) = self
                .generation
                .stop_sequences
                .iter()
                .filter_map(|s| {
                    self.generation
                        .pending
                        .windows(s.len())
                        .position(|w| w == s.as_bytes())
                })
                .min()
            {
                emit_len = pos;
                stopped_by = "stop_sequence";
                self.generation.finished = true;
            } else if !self.generation.finished {
                // Keep possible stop prefixes until a later token disambiguates them.
                let mut withheld = 0;
                for stop in &self.generation.stop_sequences {
                    for n in 1..stop.len() {
                        if self.generation.pending.ends_with(&stop.as_bytes()[..n]) {
                            withheld = withheld.max(n);
                        }
                    }
                }
                emit_len -= withheld;
            }
            let valid_len = utf8_prefix(
                &self.generation.pending[..emit_len],
                self.generation.finished,
            )?;
            if valid_len != 0 {
                let piece = std::str::from_utf8(&self.generation.pending[..valid_len])
                    .map_err(|e| NativeApiError::Serialization(e.to_string()))?;
                text.push_str(piece);
                let keep_going = callback(piece);
                self.generation.pending.drain(..valid_len);
                if !keep_going {
                    stopped_by = "cancelled";
                    self.generation.finished = true;
                }
            }
            if self.generation.finished {
                self.generation.pending.clear();
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let elapsed_us = Some(start.elapsed().as_micros().min(u64::MAX as u128) as u64);
        #[cfg(target_arch = "wasm32")]
        let elapsed_us = None;
        Ok(CompletionResponse {
            text,
            token_count,
            stopped_by: stopped_by.into(),
            elapsed_us,
            memory_facts_read: None,
        })
    }

    /// Finish a host-bounded incremental response without predicting another token.
    /// Any already selected UTF-8 tail is returned once; an incomplete scalar is
    /// a typed error rather than silently rewritten or dropped. A successful
    /// finish permits checkpoint export, including after a cancellation signal.
    pub fn finish_generation(&mut self) -> Result<CompletionResponse, NativeApiError> {
        if self.active.load(Ordering::SeqCst) {
            return Err(NativeApiError::SessionBusy(
                "generation is executing".into(),
            ));
        }
        let text = std::str::from_utf8(&self.generation.pending)
            .map_err(|_| {
                NativeApiError::Serialization("cannot finish response inside a UTF-8 scalar".into())
            })?
            .to_owned();
        let stopped_by = if self.cancelled.load(Ordering::SeqCst) {
            "cancelled"
        } else {
            "finished"
        };
        self.generation.pending.clear();
        self.generation.finished = true;
        if self.generation.started || self.durable_session.session.needs_input_boundary() {
            self.durable_session
                .session
                .end_response(&self.model)
                .map_err(|e| NativeApiError::ModelLoad(e.0))?;
        }
        // end_response has already consumed the boundary; do not reset a later
        // input chunk a second time merely because this wrapper generated earlier.
        self.generation.started = false;
        Ok(CompletionResponse {
            text,
            token_count: 0,
            stopped_by: stopped_by.into(),
            elapsed_us: None,
            memory_facts_read: None,
        })
    }

    /// A cancellation token can be retained by the host without locking the session.
    pub fn cancellation_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancelled)
    }

    /// Cooperatively signal session cancellation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Reset session context and memory, preserving the identity scope.
    pub fn reset(&mut self) -> Result<(), NativeApiError> {
        self.durable_session
            .reset(&self.model)
            .map_err(|e| NativeApiError::ModelLoad(e.0))?;
        self.cancelled.store(false, Ordering::SeqCst);
        self.active.store(false, Ordering::SeqCst);
        self.generation = GenerationState::default();
        Ok(())
    }

    /// Store a persistent fact in the session's isolated memory.
    pub fn store_fact(&mut self, owner: &str, value: &str) -> Result<u64, NativeApiError> {
        self.durable_session
            .assert_fact(owner, value)
            .map_err(|e| NativeApiError::IdentityViolation(e.0))
    }

    /// Query the session's isolated memory for an active fact value.
    pub fn query_memory(&self, owner: &str) -> Result<Option<String>, NativeApiError> {
        Ok(self.durable_session.get_fact(owner))
    }

    /// Query the session's isolated memory for an exact fact record.
    pub fn query_memory_record(
        &self,
        owner: &str,
    ) -> Result<Option<DurableFactRecord>, NativeApiError> {
        Ok(self.durable_session.get_fact_record(owner))
    }

    /// Export serialized session state.
    pub fn export_state(&self) -> Result<Vec<u8>, NativeApiError> {
        if self.generation.started && !self.generation.finished {
            return Err(NativeApiError::InvalidRequest(
                "finish or cancel generation before exporting a checkpoint".into(),
            ));
        }
        self.durable_session
            .checkpoint()
            .map_err(|e| NativeApiError::Serialization(e.0))
    }

    /// Restore serialized session state.
    pub fn import_state(&mut self, bytes: &[u8]) -> Result<(), NativeApiError> {
        let restored = DurableSession::from_checkpoint(&self.model, bytes)
            .map_err(|e| NativeApiError::Serialization(e.0))?;
        if restored.scope != self.scope {
            return Err(NativeApiError::IdentityViolation(
                "checkpoint scope differs from target session".into(),
            ));
        }
        self.durable_session = restored;
        self.generation = GenerationState::default();
        self.cancelled.store(false, Ordering::SeqCst);
        Ok(())
    }
}

// ============================================================================
// 6. WASM Model Runtime Bridge
// ============================================================================

/// Lightweight handle referencing an active session in the WASM table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WasmSessionHandle(pub u32);

/// Filesystem-free, browser-compatible WASM runtime bridge.
/// Employs bounded handle routing and byte-slice serialization. WASM parity is
/// not inferred from this Rust interface.
pub struct WasmModelRuntime {
    model: NativeModel,
    sessions: Arc<Mutex<HashMap<u32, NativeSession>>>,
    next_handle: AtomicU32,
    cancellations: Mutex<HashMap<u32, Arc<AtomicBool>>>,
}

impl WasmModelRuntime {
    pub fn new(model: NativeModel) -> Self {
        Self {
            model,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            next_handle: AtomicU32::new(1),
            cancellations: Mutex::new(HashMap::new()),
        }
    }

    /// Instantiate a new session and return an integer handle for JS/WASM interop.
    pub fn wasm_create_session(
        &self,
        session_id: &str,
        user_id: &str,
        project_id: &str,
    ) -> Result<u32, NativeApiError> {
        let config = SessionConfig {
            session_id: session_id.into(),
            user_id: user_id.into(),
            project_id: project_id.into(),
            control: Control::Full,
            max_output_tokens: 128,
            temperature: 0.0,
        };

        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| NativeApiError::ResourceLimit("session lock poisoned".into()))?;
        if lock.len() >= self.model.metadata.max_sessions {
            return Err(NativeApiError::ResourceLimit(
                "maximum live sessions reached".into(),
            ));
        }
        let session = self.model.create_session(config)?;
        let handle = self
            .next_handle
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_add(1))
            .map_err(|_| NativeApiError::ResourceLimit("session handles exhausted".into()))?;
        self.cancellations
            .lock()
            .map_err(|_| NativeApiError::ResourceLimit("cancellation lock poisoned".into()))?
            .insert(handle, session.cancellation_handle());
        lock.insert(handle, session);
        Ok(handle)
    }

    /// Ingest UTF-8 text into a WASM session.
    pub fn wasm_ingest(&self, handle: u32, text: &str) -> Result<String, NativeApiError> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| NativeApiError::ResourceLimit("session lock poisoned".into()))?;
        let session = lock
            .get_mut(&handle)
            .ok_or(NativeApiError::SessionNotFound(handle))?;

        let receipt = session.ingest(text)?;
        serde_json::to_string(&receipt).map_err(|e| NativeApiError::Serialization(e.to_string()))
    }

    /// Execute a generation step on a WASM session.
    pub fn wasm_generate_step(
        &self,
        handle: u32,
        max_tokens: usize,
    ) -> Result<String, NativeApiError> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| NativeApiError::ResourceLimit("session lock poisoned".into()))?;
        let session = lock
            .get_mut(&handle)
            .ok_or(NativeApiError::SessionNotFound(handle))?;

        let resp = session.generate_chunk(max_tokens, |_| true)?;
        serde_json::to_string(&resp).map_err(|e| NativeApiError::Serialization(e.to_string()))
    }

    /// Close incremental generation before checkpointing a host budget or stop.
    pub fn wasm_finish_generation(&self, handle: u32) -> Result<String, NativeApiError> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| NativeApiError::ResourceLimit("session lock poisoned".into()))?;
        let session = lock
            .get_mut(&handle)
            .ok_or(NativeApiError::SessionNotFound(handle))?;
        let response = session.finish_generation()?;
        serde_json::to_string(&response).map_err(|e| NativeApiError::Serialization(e.to_string()))
    }

    /// Cancel a running WASM session.
    pub fn wasm_cancel(&self, handle: u32) -> Result<(), NativeApiError> {
        let lock = self
            .cancellations
            .lock()
            .map_err(|_| NativeApiError::ResourceLimit("cancellation lock poisoned".into()))?;
        let token = lock
            .get(&handle)
            .ok_or(NativeApiError::SessionNotFound(handle))?;
        token.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Export WASM session state to byte array.
    pub fn wasm_export_session(&self, handle: u32) -> Result<Vec<u8>, NativeApiError> {
        let lock = self
            .sessions
            .lock()
            .map_err(|_| NativeApiError::ResourceLimit("session lock poisoned".into()))?;
        let session = lock
            .get(&handle)
            .ok_or(NativeApiError::SessionNotFound(handle))?;
        session.export_state()
    }

    /// Import WASM session state from byte array.
    pub fn wasm_import_session(&self, handle: u32, bytes: &[u8]) -> Result<(), NativeApiError> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| NativeApiError::ResourceLimit("session lock poisoned".into()))?;
        let session = lock
            .get_mut(&handle)
            .ok_or(NativeApiError::SessionNotFound(handle))?;
        session.import_state(bytes)
    }

    /// Free a WASM session handle and release memory.
    pub fn wasm_free_session(&self, handle: u32) {
        if let Ok(mut lock) = self.sessions.lock() {
            lock.remove(&handle);
        }
        if let Ok(mut lock) = self.cancellations.lock() {
            lock.remove(&handle);
        }
    }

    /// Introspect model metadata and capabilities as JSON string.
    pub fn wasm_get_capabilities(&self) -> String {
        serde_json::to_string(self.model.metadata()).unwrap_or_default()
    }
}

#[cfg(test)]
mod recovery_utf8_tests {
    use super::utf8_prefix;

    #[test]
    fn streaming_preserves_split_multibyte_scalars_and_rejects_invalid_bytes() {
        // Three model byte tokens must be emitted as one valid euro scalar.
        let mut pending = Vec::new();
        for byte in [0xe2, 0x82] {
            pending.push(byte);
            assert_eq!(utf8_prefix(&pending, false).unwrap(), 0);
            assert!(utf8_prefix(&pending, true).is_err());
        }
        pending.push(0xac);
        assert_eq!(utf8_prefix(&pending, true).unwrap(), 3);
        assert_eq!(std::str::from_utf8(&pending).unwrap(), "€");
        assert_eq!(utf8_prefix(b"hello\xe2", false).unwrap(), 5);
        assert!(utf8_prefix(&[0xff], false).is_err());
    }
}
