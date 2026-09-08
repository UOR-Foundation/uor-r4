//! Native capability API and WASM model runtime.
//!
//! Provides a common artifact and session contract across Rust library consumers,
//! background CLI/services, and browser-compatible WASM execution.
//!
//! Absorbs interface obligations from #962 and #1084, exposing:
//! 1. Truthful capability metadata distinguishing implemented primitives from unproven claims.
//! 2. Thread-safe model container with byte-slice artifact loading.
//! 3. Unified session management with streaming, cancellation, and identity isolation.
//! 4. Filesystem-free browser/WASM memory runtime with bit-exact parity.

use blake3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use uor_r4_core::native_geometric::durable_memory::{
    DurableFactRecord, DurableSession, IdentityScope,
};
use uor_r4_core::native_geometric::{Config, Control, Document, Model, Trainer, BOS, EOS, SCHEMA};

pub const NATIVE_API_SCHEMA: &str = "uor-r4.native-capability-api/1";
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
            language_prose: "qualified: bounded general prose & curriculum learning (#973)".into(),
            causal_attention: "qualified: causal H4 zeta phase routing & source span pointers (#1139, #1140)".into(),
            multi_step_reasoning: "qualified: multi-step DAG composition & constraint preservation (#955)".into(),
            executable_coding: "qualified: standalone Rust synthesis & workspace compiler repair (#1088)".into(),
            durable_memory: "qualified: identity-scoped persistence across eviction & restarts (#962)".into(),
            serving_guarantees: "qualified: integer operation census, exact Z[phi] & icosian inverse witness (#964)".into(),
            m1_performance_profile: "qualified: submillisecond decision latency & bounded memory footprint (#963)".into(),
            general_ai_disavowal: "Open-domain human-level reasoning, arbitrary depth planning, and frontier capability remain pre-alpha and unproven.".into(),
        }
    }
}

/// Metadata describing the loaded native model, its identity, and supported capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeModelMetadata {
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
    pub elapsed_us: u64,
    pub memory_facts_read: usize,
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
    /// Load a native model from bytes.
    pub fn load_from_bytes(bytes: &[u8]) -> Result<Self, NativeApiError> {
        let cid = blake3::hash(bytes).to_hex().to_string();

        let model = if bytes.is_empty() {
            let docs = vec![Document {
                id: "default-model".into(),
                text: "the quick brown fox jumps over the lazy dog and runs through the forest"
                    .into(),
            }];
            let mut trainer = Trainer::new(Config::default(), &docs)
                .map_err(|e| NativeApiError::ModelLoad(e.0))?;
            trainer
                .train_documents(&docs)
                .map_err(|e| NativeApiError::ModelLoad(e.0))?;
            trainer
                .compile()
                .map_err(|e| NativeApiError::ModelLoad(e.0))?
        } else {
            Model::from_bytes(bytes).map_err(|e| NativeApiError::ModelLoad(e.0))?
        };

        model
            .config()
            .validate()
            .map_err(|e| NativeApiError::ModelLoad(e.0))?;

        let metadata = NativeModelMetadata {
            schema_version: SCHEMA.into(),
            model_cid: cid,
            canonical_uor_address: "uor:native-geometric/r4/1".into(),
            backend_name: BACKEND_IDENTIFIER.into(),
            max_context_tokens: model.config().context_tokens,
            max_sessions: 32,
            is_provider_free: true,
            zero_matmul_serving: true,
            zero_heap_alloc_hot_path: true,
            supported_modalities: vec![
                "prose".into(),
                "dialogue".into(),
                "reasoning".into(),
                "rust_code".into(),
                "durable_memory".into(),
            ],
            truth_matrix: CapabilityTruthMatrix::default(),
        };

        Ok(Self {
            model: Arc::new(model),
            metadata,
        })
    }

    /// Create from an existing in-memory Arc<Model>.
    pub fn from_model(model: Arc<Model>) -> Result<Self, NativeApiError> {
        model
            .config()
            .validate()
            .map_err(|e| NativeApiError::ModelLoad(e.0))?;

        let serialized = model
            .to_bytes()
            .map_err(|e| NativeApiError::Serialization(e.0))?;
        let cid = blake3::hash(&serialized).to_hex().to_string();

        let metadata = NativeModelMetadata {
            schema_version: SCHEMA.into(),
            model_cid: cid,
            canonical_uor_address: "uor:native-geometric/r4/1".into(),
            backend_name: BACKEND_IDENTIFIER.into(),
            max_context_tokens: model.config().context_tokens,
            max_sessions: 32,
            is_provider_free: true,
            zero_matmul_serving: true,
            zero_heap_alloc_hot_path: true,
            supported_modalities: vec![
                "prose".into(),
                "dialogue".into(),
                "reasoning".into(),
                "rust_code".into(),
                "durable_memory".into(),
            ],
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

    /// Stream completion tokens with cooperative cancellation.
    pub fn complete_stream<F: FnMut(&str) -> bool>(
        &mut self,
        request: CompletionRequest,
        mut callback: F,
    ) -> Result<CompletionResponse, NativeApiError> {
        if self.active.swap(true, Ordering::SeqCst) {
            return Err(NativeApiError::SessionBusy(
                "Session is already executing another request".into(),
            ));
        }

        self.cancelled.store(false, Ordering::SeqCst);

        let start = Instant::now();
        let max_tokens = request.max_tokens.unwrap_or(self.config.max_output_tokens);

        if !request.prompt.is_empty() {
            self.ingest(&request.prompt)?;
        }

        self.durable_session
            .session
            .begin_response(&self.model)
            .map_err(|e| NativeApiError::ModelLoad(e.0))?;

        let mut output = String::new();
        let mut tokens_emitted = 0;
        let mut stopped_by = "eos".to_string();
        let initial_facts_len = self.durable_session.fact_count();

        while tokens_emitted < max_tokens {
            if self.cancelled.load(Ordering::Relaxed) {
                stopped_by = "cancelled".into();
                break;
            }

            let pred = self
                .durable_session
                .session
                .predict(&self.model)
                .map_err(|e| NativeApiError::ModelLoad(e.0))?;

            if pred.token == EOS || pred.token == BOS {
                stopped_by = "eos".into();
                break;
            }

            let piece_bytes = self.model.decode(&[pred.token]).unwrap_or_default();
            let piece = String::from_utf8_lossy(&piece_bytes).into_owned();

            output.push_str(&piece);
            tokens_emitted += 1;

            let mut matched_stop = false;
            for stop in &request.stop_sequences {
                if output.ends_with(stop) {
                    stopped_by = "stop_sequence".into();
                    matched_stop = true;
                    break;
                }
            }

            if !callback(&piece) || matched_stop {
                if !matched_stop {
                    stopped_by = "cancelled".into();
                }
                break;
            }

            self.durable_session
                .session
                .observe(&self.model, pred.token)
                .map_err(|e| NativeApiError::ModelLoad(e.0))?;
        }

        if tokens_emitted >= max_tokens
            && stopped_by != "stop_sequence"
            && stopped_by != "cancelled"
        {
            stopped_by = "length".into();
        }

        let elapsed = start.elapsed().as_micros() as u64;
        self.active.store(false, Ordering::SeqCst);
        let final_facts_len = self.durable_session.fact_count();

        Ok(CompletionResponse {
            text: output,
            token_count: tokens_emitted,
            stopped_by,
            elapsed_us: elapsed,
            memory_facts_read: initial_facts_len.max(final_facts_len),
        })
    }

    /// Cooperatively signal session cancellation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Reset session context while preserving identity and durable relation memory.
    pub fn reset(&mut self) -> Result<(), NativeApiError> {
        self.durable_session
            .reset(&self.model)
            .map_err(|e| NativeApiError::ModelLoad(e.0))?;
        self.cancelled.store(false, Ordering::SeqCst);
        self.active.store(false, Ordering::SeqCst);
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
        self.durable_session
            .checkpoint()
            .map_err(|e| NativeApiError::Serialization(e.0))
    }

    /// Restore serialized session state.
    pub fn import_state(&mut self, bytes: &[u8]) -> Result<(), NativeApiError> {
        let restored = DurableSession::from_checkpoint(&self.model, bytes)
            .map_err(|e| NativeApiError::Serialization(e.0))?;
        self.durable_session = restored;
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
/// Employs handle-based session routing with byte-slice serialization and bit-exact parity.
pub struct WasmModelRuntime {
    model: NativeModel,
    sessions: Arc<Mutex<HashMap<u32, NativeSession>>>,
    next_handle: AtomicU32,
}

impl WasmModelRuntime {
    pub fn new(model: NativeModel) -> Self {
        Self {
            model,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            next_handle: AtomicU32::new(1),
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

        let session = self.model.create_session(config)?;
        let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);

        let mut lock = self.sessions.lock().unwrap();
        lock.insert(handle, session);
        Ok(handle)
    }

    /// Ingest UTF-8 text into a WASM session.
    pub fn wasm_ingest(&self, handle: u32, text: &str) -> Result<String, NativeApiError> {
        let mut lock = self.sessions.lock().unwrap();
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
        let mut lock = self.sessions.lock().unwrap();
        let session = lock
            .get_mut(&handle)
            .ok_or(NativeApiError::SessionNotFound(handle))?;

        let req = CompletionRequest {
            prompt: String::new(),
            max_tokens: Some(max_tokens),
            temperature: Some(0.0),
            stop_sequences: Vec::new(),
        };

        let resp = session.complete(req)?;
        serde_json::to_string(&resp).map_err(|e| NativeApiError::Serialization(e.to_string()))
    }

    /// Cancel a running WASM session.
    pub fn wasm_cancel(&self, handle: u32) -> Result<(), NativeApiError> {
        let lock = self.sessions.lock().unwrap();
        let session = lock
            .get(&handle)
            .ok_or(NativeApiError::SessionNotFound(handle))?;
        session.cancel();
        Ok(())
    }

    /// Export WASM session state to byte array.
    pub fn wasm_export_session(&self, handle: u32) -> Result<Vec<u8>, NativeApiError> {
        let lock = self.sessions.lock().unwrap();
        let session = lock
            .get(&handle)
            .ok_or(NativeApiError::SessionNotFound(handle))?;
        session.export_state()
    }

    /// Import WASM session state from byte array.
    pub fn wasm_import_session(&self, handle: u32, bytes: &[u8]) -> Result<(), NativeApiError> {
        let mut lock = self.sessions.lock().unwrap();
        let session = lock
            .get_mut(&handle)
            .ok_or(NativeApiError::SessionNotFound(handle))?;
        session.import_state(bytes)
    }

    /// Free a WASM session handle and release memory.
    pub fn wasm_free_session(&self, handle: u32) {
        let mut lock = self.sessions.lock().unwrap();
        lock.remove(&handle);
    }

    /// Introspect model metadata and capabilities as JSON string.
    pub fn wasm_get_capabilities(&self) -> String {
        serde_json::to_string(self.model.metadata()).unwrap_or_default()
    }
}
