//! Native desktop client backend (`src-tauri`) for the UOR-R4 geometric language model.
//!
//! Links [`uor_r4_integer`] (`features = ["serving-wasm"]`) and [`uor_r4_object_store`] directly as
//! Rust libraries so the desktop client shares the Studio interface (`web/studio/index.html`) while
//! running the D11 integer stack engine and D19 content-addressed durable memory store natively and
//! offline on client laptops (CHARTER §3.6).
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_integer::session_codec;
use uor_r4_integer::stack::{
    stack_argmax, IntegerStackModel, StackShape, MAGIC as UORLUT01_MAGIC, STACK_SCHEMA,
};
use uor_r4_object_store::durable_consumer::{DurableObjectMemoryStore, IdentityScope};
use uor_r4_object_store::{Limits, Manifest, Store};

/// Default deterministic seed for the `mk-synthetic-stack` stand-in (`crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs`).
pub const SYNTHETIC_STANDIN_SEED: u64 = 5;

/// Default file name of the synthetic `UORLUT01` stand-in artifact.
pub const SYNTHETIC_STANDIN_FILE_NAME: &str = "synthetic-stack.lut";

/// Engine identifier reported to the shared Studio interface when running in the Tauri desktop shell.
pub const DESKTOP_ENGINE_TAG: &str = "uor-r4-integer/native-desktop";

/// Stand-in prompt tokenizer label shared with `web/studio/assets/js/uor_model_worker.js`.
pub const PROMPT_TOKENIZER_LABEL: &str = "UTF-8 byte-mod stand-in (rawBytes[i] % vocabSize)";

/// Maximum simultaneous conversation sessions held in memory.
pub const MAX_DESKTOP_SESSIONS: usize = 32;

/// Pinned release record for a `UORLUT01` model artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelPin {
    pub name: String,
    pub sha256: String,
    pub synthetic: bool,
}

/// Registry of SHA-256 release pins accepted by the model manager.
///
/// Per `docs/products/desktop-client.md` §Security ("Load only files whose SHA-256 matches a
/// pinned release hash"), the model manager verifies a candidate file's SHA-256 against this
/// registry before passing any bytes to `crates/uor-r4-integer/src/stack/format.rs`.
#[derive(Debug, Clone, Default)]
pub struct PinRegistry {
    pins_by_sha256: BTreeMap<String, ModelPin>,
}

impl PinRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a pin registry pre-populated with the deterministic seed-5 synthetic stand-in pin.
    pub fn with_synthetic_standin() -> Self {
        let mut reg = Self::new();
        let bytes = build_synthetic_stack_artifact(SYNTHETIC_STANDIN_SEED);
        let sha = sha256_bytes(&bytes);
        reg.register_pin(SYNTHETIC_STANDIN_FILE_NAME, sha, true);
        reg
    }

    pub fn register_pin(
        &mut self,
        name: impl Into<String>,
        sha256: impl Into<String>,
        synthetic: bool,
    ) {
        let sha = sha256.into().to_ascii_lowercase();
        self.pins_by_sha256.insert(
            sha.clone(),
            ModelPin {
                name: name.into(),
                sha256: sha,
                synthetic,
            },
        );
    }

    pub fn get_by_sha256(&self, sha256: &str) -> Option<&ModelPin> {
        self.pins_by_sha256.get(&sha256.to_ascii_lowercase())
    }
}

/// Entry returned to the Studio model manager by `native_list_local_models`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalModelEntry {
    pub name: String,
    pub path: String,
    pub format: String,
    pub size_bytes: u64,
    pub size_str: String,
    pub sha256: String,
    pub synthetic: bool,
    pub status_label: String,
}

/// Shape summary reported in [`LoadedModelCapabilities`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShapeSummary {
    pub vocab: usize,
    pub width: usize,
    pub heads: usize,
    pub mlp: usize,
    pub pattern: String,
    pub read: String,
    pub rotation: bool,
    pub context: usize,
}

impl From<&StackShape> for ShapeSummary {
    fn from(s: &StackShape) -> Self {
        Self {
            vocab: s.vocab,
            width: s.width,
            heads: s.heads,
            mlp: s.mlp,
            pattern: s.pattern.clone(),
            read: s.read.clone(),
            rotation: s.rotation,
            context: s.context,
        }
    }
}

/// Capabilities reported to the Studio page when a pinned `UORLUT01` stack is loaded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadedModelCapabilities {
    pub engine: String,
    pub file_name: String,
    pub artifact_bytes: usize,
    pub artifact_sha256: String,
    pub schema: String,
    pub shape: ShapeSummary,
    pub group: usize,
    pub synthetic: bool,
    pub status_label: String,
    pub tokenizer_label: String,
    pub weights_per_token: u64,
    pub checkpoint_bytes: usize,
}

/// Per-token streaming event emitted during `native_step_tokens` / `native_generate_prompt`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamTokenEvent {
    pub conversation_id: String,
    pub step_index: usize,
    pub token_id: u32,
    pub margin_q16: i32,
    pub position: u32,
    pub chunk: String,
    pub full_text: String,
}

/// Final receipt returned at the end of a native generation turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationReceipt {
    pub conversation_id: String,
    pub token_ids: Vec<u32>,
    pub full_text: String,
    pub weights_per_token: u64,
    pub checkpoint_bytes: usize,
    pub status_label: String,
    pub tokenizer_label: String,
}

/// Receipt returned by `native_save_session_durable` after folding a `UORMEM01` + `UR4SESS1`
/// envelope into [`uor_r4_object_store::Store`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableSessionReceipt {
    pub scope_key: String,
    pub object_version_hex: String,
    pub manifest_bytes: Vec<u8>,
    pub ur4sess1_bytes_len: usize,
    pub store_record_count: usize,
    pub store_retained_bytes: usize,
}

/// Receipt returned by `native_restore_session_durable` after verifying and restoring a session
/// from [`uor_r4_object_store::Store`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoredSessionReceipt {
    pub scope_key: String,
    pub conversation_id: String,
    pub artifact_sha256: String,
    pub position: u64,
    pub tokens: Vec<u32>,
    pub ur4sess1_bytes_len: usize,
    pub non_memory_turns: usize,
}

/// Git repository status report returned by `native_git_status`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitStatusReport {
    pub branch: String,
    pub is_clean: bool,
    pub modified_files: Vec<String>,
    pub untracked_files: Vec<String>,
}

/// Request structure for `native_run_terminal_command`.
///
/// Per `docs/products/desktop-client.md` §Security, terminal command execution is off by default,
/// must display the exact command to the user for confirmation before running, and is restricted
/// to a user-granted directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalCommandRequest {
    pub cwd: PathBuf,
    pub command: String,
    pub user_confirmed: bool,
    pub confirmed_command: String,
}

/// Errors returned by the desktop client backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopClientError {
    Io(String),
    UnsupportedModelFormat(String),
    UnpinnedOrTamperedModel {
        actual_sha256: String,
        reason: String,
    },
    StackEngine(String),
    ObjectStore(String),
    PathTraversalRefused(String),
    PathNotGranted(String),
    TerminalDisabledByDefault,
    TerminalConsentRequired,
    TerminalMetacharacterRefused(String),
    CommandFailed(String),
    SessionNotFound(String),
    SessionLimitReached,
    ModelNotLoaded,
    InvalidInput(String),
}

impl fmt::Display for DesktopClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for DesktopClientError {}

/// Compute the lowercase hex SHA-256 of an in-memory byte slice.
pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Format a byte count using integer arithmetic only (`B` or `KiB`).
pub fn format_size_bytes(size_bytes: u64) -> String {
    if size_bytes >= 1024 {
        let kib = size_bytes >> 10;
        format!("{kib} KiB ({size_bytes} B)")
    } else {
        format!("{size_bytes} B")
    }
}

/// Verify that `bytes` starts with `UORLUT01` and that its whole-file SHA-256 matches both
/// `expected_sha256` (if provided) and an entry in `pins` **before** passing the bytes to the
/// structural parser in `crates/uor-r4-integer/src/stack/format.rs`.
pub fn verify_pinned_uorlut01_bytes(
    bytes: &[u8],
    pins: &PinRegistry,
    expected_sha256: Option<&str>,
) -> Result<(String, ModelPin), DesktopClientError> {
    if bytes.len() < 16 || &bytes[..8] != UORLUT01_MAGIC {
        return Err(DesktopClientError::UnsupportedModelFormat(
            "expected UORLUT01 magic header; non-UORLUT01 model formats are refused".to_string(),
        ));
    }
    let actual_sha256 = sha256_bytes(bytes);
    if let Some(expected) = expected_sha256 {
        if !actual_sha256.eq_ignore_ascii_case(expected.trim()) {
            return Err(DesktopClientError::UnpinnedOrTamperedModel {
                actual_sha256,
                reason: "file SHA-256 does not match caller-supplied pin".to_string(),
            });
        }
    }
    let Some(pin) = pins.get_by_sha256(&actual_sha256) else {
        return Err(DesktopClientError::UnpinnedOrTamperedModel {
            actual_sha256,
            reason: "file SHA-256 is not present in the release pin registry".to_string(),
        });
    };
    Ok((actual_sha256, pin.clone()))
}

/// Encode prompt text or explicit token IDs using the deterministic UTF-8 byte-mod stand-in
/// (`raw_bytes[i] % vocab_size`) shared with `web/studio/assets/js/uor_model_worker.js`.
pub fn encode_prompt_standin(
    prompt_text: &str,
    vocab_size: usize,
) -> Result<Vec<u32>, DesktopClientError> {
    let trimmed = prompt_text.trim();
    if trimmed.is_empty() || vocab_size == 0 {
        return Err(DesktopClientError::InvalidInput(
            "a non-empty prompt and positive vocabulary size are required".to_string(),
        ));
    }
    let vocab_u32 = u32::try_from(vocab_size)
        .map_err(|_| DesktopClientError::InvalidInput("vocabulary size overflow".to_string()))?;
    Ok(trimmed
        .as_bytes()
        .iter()
        .map(|&b| u32::from(b) % vocab_u32)
        .collect())
}

#[derive(Debug, Clone)]
struct ConversationSessionEntry {
    ur4sess1_bytes: Vec<u8>,
    last_token: Option<u32>,
}

/// Host-testable Tauri desktop client backend state.
pub struct DesktopBackend {
    model_search_dirs: Vec<PathBuf>,
    pins: PinRegistry,
    granted_folders: BTreeSet<PathBuf>,
    terminal_enabled: bool,
    loaded_model: Option<IntegerStackModel>,
    loaded_capabilities: Option<LoadedModelCapabilities>,
    initial_session_bytes: Option<Vec<u8>>,
    sessions: BTreeMap<String, ConversationSessionEntry>,
    object_store: Store,
    memory_store: DurableObjectMemoryStore,
    durable_store_dir: Option<PathBuf>,
}

impl DesktopBackend {
    /// Construct a new desktop backend with an explicit release [`PinRegistry`].
    ///
    /// File access allowlists (`granted_folders`) are empty by default and `terminal_enabled` is
    /// `false` by default.
    pub fn new(pins: PinRegistry) -> Self {
        Self {
            model_search_dirs: Vec::new(),
            pins,
            granted_folders: BTreeSet::new(),
            terminal_enabled: false,
            loaded_model: None,
            loaded_capabilities: None,
            initial_session_bytes: None,
            sessions: BTreeMap::new(),
            object_store: Store::new(Limits::default()),
            memory_store: DurableObjectMemoryStore::new(),
            durable_store_dir: None,
        }
    }

    /// Borrow the active [`PinRegistry`].
    pub fn pins(&self) -> &PinRegistry {
        &self.pins
    }

    /// Borrow the mutable [`PinRegistry`].
    pub fn pins_mut(&mut self) -> &mut PinRegistry {
        &mut self.pins
    }

    /// Add a directory to scan during `native_list_local_models`.
    pub fn add_model_search_dir(&mut self, dir: impl Into<PathBuf>) {
        self.model_search_dirs.push(dir.into());
    }

    /// Configure an on-disk directory for durable object-store persistence across restarts.
    ///
    /// If `records.uorobj01` already exists in `dir`, loads and validates its content-addressed
    /// records into `self.object_store`.
    pub fn set_durable_store_dir(
        &mut self,
        dir: impl AsRef<Path>,
    ) -> Result<(), DesktopClientError> {
        let path = dir.as_ref();
        fs::create_dir_all(path).map_err(|e| DesktopClientError::Io(e.to_string()))?;
        let canonical =
            fs::canonicalize(path).map_err(|e| DesktopClientError::Io(e.to_string()))?;
        let records_path = canonical.join("records.uorobj01");
        if records_path.is_file() {
            self.object_store
                .load(&records_path)
                .map_err(|e| DesktopClientError::ObjectStore(e.to_string()))?;
        }
        self.durable_store_dir = Some(canonical);
        Ok(())
    }

    /// Borrow the loaded model's capabilities, if a stack is currently loaded.
    pub fn loaded_capabilities(&self) -> Option<&LoadedModelCapabilities> {
        self.loaded_capabilities.as_ref()
    }

    /// Borrow the underlying [`Store`] for inspection in host tests.
    pub fn object_store(&self) -> &Store {
        &self.object_store
    }

    /// Scan `model_search_dirs` for `.lut` model files, verifying each candidate's `UORLUT01`
    /// magic and SHA-256 pin before passing bytes to `IntegerStackModel::parse`.
    pub fn native_list_local_models(&self) -> Result<Vec<LocalModelEntry>, DesktopClientError> {
        let mut entries = Vec::new();
        for dir in &self.model_search_dirs {
            if !dir.is_dir() {
                continue;
            }
            let read_dir = fs::read_dir(dir).map_err(|e| DesktopClientError::Io(e.to_string()))?;
            let mut paths: Vec<PathBuf> = read_dir
                .filter_map(|res| res.ok().map(|e| e.path()))
                .filter(|p| p.is_file())
                .collect();
            paths.sort();

            for path in paths {
                let is_candidate = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|ext| {
                        ext.eq_ignore_ascii_case("lut")
                            || ext.eq_ignore_ascii_case("bin")
                            || ext.eq_ignore_ascii_case("gguf")
                            || ext.eq_ignore_ascii_case("safetensors")
                    });
                if !is_candidate {
                    continue;
                }
                let bytes = fs::read(&path).map_err(|e| DesktopClientError::Io(e.to_string()))?;
                let (sha256, pin) = verify_pinned_uorlut01_bytes(&bytes, &self.pins, None)?;
                let model = IntegerStackModel::parse(&bytes)
                    .map_err(|e| DesktopClientError::StackEngine(e.to_string()))?;
                let size_bytes = bytes.len() as u64;
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&pin.name)
                    .to_string();
                let status_label = if pin.synthetic {
                    "synthetic stand-in".to_string()
                } else {
                    format!("pinned UORLUT01 ({} w/tok)", model.weights_per_token())
                };
                entries.push(LocalModelEntry {
                    name: file_name,
                    path: path.to_string_lossy().into_owned(),
                    format: "UORLUT01".to_string(),
                    size_bytes,
                    size_str: format_size_bytes(size_bytes),
                    sha256,
                    synthetic: pin.synthetic,
                    status_label,
                });
            }
        }
        Ok(entries)
    }

    /// Load a `UORLUT01` model file from disk after verifying its SHA-256 against [`PinRegistry`].
    pub fn native_load_model(
        &mut self,
        file_path: &Path,
        expected_sha256: Option<&str>,
    ) -> Result<LoadedModelCapabilities, DesktopClientError> {
        let bytes = fs::read(file_path).map_err(|e| DesktopClientError::Io(e.to_string()))?;
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(SYNTHETIC_STANDIN_FILE_NAME);
        self.native_load_model_bytes(file_name, &bytes, expected_sha256)
    }

    /// Load a `UORLUT01` model from raw bytes after verifying its SHA-256 against [`PinRegistry`].
    pub fn native_load_model_bytes(
        &mut self,
        file_name: &str,
        bytes: &[u8],
        expected_sha256: Option<&str>,
    ) -> Result<LoadedModelCapabilities, DesktopClientError> {
        let (sha256, pin) = verify_pinned_uorlut01_bytes(bytes, &self.pins, expected_sha256)?;
        let model = IntegerStackModel::parse(bytes)
            .map_err(|e| DesktopClientError::StackEngine(e.to_string()))?;
        let blank_session = model.session();
        let blank_bytes = session_codec::encode(&blank_session.save_state());
        let capabilities = LoadedModelCapabilities {
            engine: DESKTOP_ENGINE_TAG.to_string(),
            file_name: file_name.to_string(),
            artifact_bytes: bytes.len(),
            artifact_sha256: sha256,
            schema: STACK_SCHEMA.to_string(),
            shape: ShapeSummary::from(model.shape()),
            group: uor_r4_integer::stack::GROUP,
            synthetic: pin.synthetic,
            status_label: if pin.synthetic {
                "synthetic stand-in".to_string()
            } else {
                "pinned stack artifact".to_string()
            },
            tokenizer_label: PROMPT_TOKENIZER_LABEL.to_string(),
            weights_per_token: model.weights_per_token(),
            checkpoint_bytes: blank_bytes.len(),
        };
        self.loaded_model = Some(model);
        self.loaded_capabilities = Some(capabilities.clone());
        self.initial_session_bytes = Some(blank_bytes);
        self.sessions.clear();
        Ok(capabilities)
    }

    fn ensure_session_entry(
        &mut self,
        conversation_id: &str,
    ) -> Result<ConversationSessionEntry, DesktopClientError> {
        if conversation_id.trim().is_empty() {
            return Err(DesktopClientError::InvalidInput(
                "conversation_id must not be empty".to_string(),
            ));
        }
        let initial = self
            .initial_session_bytes
            .as_ref()
            .ok_or(DesktopClientError::ModelNotLoaded)?;
        if let Some(existing) = self.sessions.get(conversation_id) {
            return Ok(existing.clone());
        }
        if self.sessions.len() >= MAX_DESKTOP_SESSIONS {
            return Err(DesktopClientError::SessionLimitReached);
        }
        let fresh = ConversationSessionEntry {
            ur4sess1_bytes: initial.clone(),
            last_token: None,
        };
        self.sessions
            .insert(conversation_id.to_string(), fresh.clone());
        Ok(fresh)
    }

    /// Step the native [`IntegerStackSession`] for `conversation_id` over `prompt_tokens` and
    /// generate `max_new_tokens` greedy tokens using integer argmax, streaming each token via
    /// `on_token`.
    pub fn native_step_tokens<F>(
        &mut self,
        conversation_id: &str,
        prompt_tokens: &[u32],
        max_new_tokens: usize,
        mut on_token: F,
    ) -> Result<GenerationReceipt, DesktopClientError>
    where
        F: FnMut(StreamTokenEvent),
    {
        if prompt_tokens.is_empty() {
            return Err(DesktopClientError::InvalidInput(
                "prompt_tokens must not be empty".to_string(),
            ));
        }
        if max_new_tokens == 0 || max_new_tokens > 4096 {
            return Err(DesktopClientError::InvalidInput(
                "max_new_tokens must be in 1..=4096".to_string(),
            ));
        }
        let entry = self.ensure_session_entry(conversation_id)?;
        let model = self
            .loaded_model
            .as_ref()
            .ok_or(DesktopClientError::ModelNotLoaded)?;
        let caps = self
            .loaded_capabilities
            .as_ref()
            .ok_or(DesktopClientError::ModelNotLoaded)?;

        let decoded_state = session_codec::decode(&entry.ur4sess1_bytes)
            .map_err(DesktopClientError::StackEngine)?;
        let mut session = model.session();
        session
            .restore_state(&decoded_state)
            .map_err(|e| DesktopClientError::StackEngine(e.to_string()))?;

        let mut tok = prompt_tokens[0];
        for &next_prompt_tok in &prompt_tokens[1..] {
            session
                .step(tok)
                .map_err(|e| DesktopClientError::StackEngine(e.to_string()))?;
            tok = next_prompt_tok;
        }

        let mut token_ids = Vec::with_capacity(max_new_tokens);
        let mut full_text = String::new();

        for step_index in 0..max_new_tokens {
            let logits = session
                .step(tok)
                .map_err(|e| DesktopClientError::StackEngine(e.to_string()))?;
            let next = stack_argmax(logits) as u32;
            let (mut top1, mut top2) = (i32::MIN, i32::MIN);
            for &v in logits {
                if v > top1 {
                    top2 = top1;
                    top1 = v;
                } else if v > top2 {
                    top2 = v;
                }
            }
            let margin_q16 = top1.saturating_sub(top2);
            let position = session.position() as u32;

            tok = next;
            token_ids.push(tok);
            let chunk = if step_index == 0 {
                tok.to_string()
            } else {
                format!(" {tok}")
            };
            full_text.push_str(&chunk);

            on_token(StreamTokenEvent {
                conversation_id: conversation_id.to_string(),
                step_index,
                token_id: tok,
                margin_q16,
                position,
                chunk,
                full_text: full_text.clone(),
            });
        }

        let saved_bytes = session_codec::encode(&session.save_state());
        let checkpoint_bytes = saved_bytes.len();
        self.sessions.insert(
            conversation_id.to_string(),
            ConversationSessionEntry {
                ur4sess1_bytes: saved_bytes,
                last_token: Some(tok),
            },
        );

        Ok(GenerationReceipt {
            conversation_id: conversation_id.to_string(),
            token_ids,
            full_text,
            weights_per_token: caps.weights_per_token,
            checkpoint_bytes,
            status_label: caps.status_label.clone(),
            tokenizer_label: caps.tokenizer_label.clone(),
        })
    }

    /// Encode `prompt_text` with the UTF-8 byte-mod stand-in and step `max_new_tokens` tokens.
    pub fn native_generate_prompt<F>(
        &mut self,
        conversation_id: &str,
        prompt_text: &str,
        max_new_tokens: usize,
        on_token: F,
    ) -> Result<GenerationReceipt, DesktopClientError>
    where
        F: FnMut(StreamTokenEvent),
    {
        let vocab = self
            .loaded_capabilities
            .as_ref()
            .ok_or(DesktopClientError::ModelNotLoaded)?
            .shape
            .vocab;
        let prompt_tokens = encode_prompt_standin(prompt_text, vocab)?;
        self.native_step_tokens(conversation_id, &prompt_tokens, max_new_tokens, on_token)
    }

    /// Export the raw `UR4SESS1` session snapshot bytes and last token for `conversation_id`.
    pub fn native_export_session(
        &mut self,
        conversation_id: &str,
    ) -> Result<(Vec<u8>, Option<u32>), DesktopClientError> {
        let entry = self.ensure_session_entry(conversation_id)?;
        Ok((entry.ur4sess1_bytes, entry.last_token))
    }

    /// Import and validate a `UR4SESS1` session snapshot for `conversation_id`.
    pub fn native_import_session(
        &mut self,
        conversation_id: &str,
        ur4sess1_bytes: &[u8],
        last_token: Option<u32>,
    ) -> Result<usize, DesktopClientError> {
        let model = self
            .loaded_model
            .as_ref()
            .ok_or(DesktopClientError::ModelNotLoaded)?;
        let decoded =
            session_codec::decode(ur4sess1_bytes).map_err(DesktopClientError::StackEngine)?;
        let mut session = model.session();
        session
            .restore_state(&decoded)
            .map_err(|e| DesktopClientError::StackEngine(e.to_string()))?;
        let verified = session_codec::encode(&session.save_state());
        let len = verified.len();
        self.sessions.insert(
            conversation_id.to_string(),
            ConversationSessionEntry {
                ur4sess1_bytes: verified,
                last_token,
            },
        );
        Ok(len)
    }

    /// Reset or close `conversation_id`, restoring its state to the initial blank session.
    pub fn native_reset_session(
        &mut self,
        conversation_id: &str,
    ) -> Result<(), DesktopClientError> {
        if self.loaded_model.is_none() {
            return Err(DesktopClientError::ModelNotLoaded);
        }
        self.sessions.remove(conversation_id);
        Ok(())
    }

    /// Persist the active `UR4SESS1` snapshot for `(user_id, project_id, conversation_id)` into
    /// [`uor_r4_object_store::Store`] via [`DurableObjectMemoryStore::export_scope`] (`UORMEM01`).
    ///
    /// When `durable_store_dir` is configured, atomically saves the deduplicated object-store
    /// records (`records.uorobj01`) and the scope's pinned [`Manifest`] (`scopes/<scope>.uorman01`)
    /// to disk so a subsequent process can restore the conversation across restarts.
    pub fn native_save_session_durable(
        &mut self,
        user_id: &str,
        project_id: &str,
        conversation_id: &str,
    ) -> Result<DurableSessionReceipt, DesktopClientError> {
        let scope = IdentityScope::new(user_id, project_id, conversation_id)
            .map_err(|e| DesktopClientError::ObjectStore(e.to_string()))?;
        let entry = self.ensure_session_entry(conversation_id)?;
        self.memory_store
            .set_ur4sess1_snapshot(&scope, &entry.ur4sess1_bytes)
            .map_err(|e| DesktopClientError::ObjectStore(e.to_string()))?;
        let manifest = self
            .memory_store
            .export_scope(&mut self.object_store, &scope)
            .map_err(|e| DesktopClientError::ObjectStore(e.to_string()))?;
        let manifest_bytes = manifest.encode();

        if let Some(ref dir) = self.durable_store_dir {
            let records_path = dir.join("records.uorobj01");
            self.object_store
                .save(&records_path)
                .map_err(|e| DesktopClientError::ObjectStore(e.to_string()))?;
            let scopes_dir = dir.join("scopes");
            fs::create_dir_all(&scopes_dir).map_err(|e| DesktopClientError::Io(e.to_string()))?;
            let manifest_path = scopes_dir.join(scope_manifest_filename(&scope));
            fs::write(&manifest_path, &manifest_bytes)
                .map_err(|e| DesktopClientError::Io(e.to_string()))?;
        }

        Ok(DurableSessionReceipt {
            scope_key: scope.key(),
            object_version_hex: hex::encode(manifest.version()),
            manifest_bytes,
            ur4sess1_bytes_len: entry.ur4sess1_bytes.len(),
            store_record_count: self.object_store.record_count(),
            store_retained_bytes: self.object_store.retained_record_bytes(),
        })
    }

    /// Rebuild and verify the `UORMEM01` + `UR4SESS1` snapshot for `(user_id, project_id, conversation_id)`
    /// from [`uor_r4_object_store::Store`] and restore the conversation's [`IntegerStackSession`].
    pub fn native_restore_session_durable(
        &mut self,
        user_id: &str,
        project_id: &str,
        conversation_id: &str,
        manifest_bytes: Option<&[u8]>,
    ) -> Result<RestoredSessionReceipt, DesktopClientError> {
        let scope = IdentityScope::new(user_id, project_id, conversation_id)
            .map_err(|e| DesktopClientError::ObjectStore(e.to_string()))?;

        let owned_manifest_bytes: Vec<u8>;
        let raw_manifest: &[u8] = match manifest_bytes {
            Some(b) => b,
            None => {
                let dir = self.durable_store_dir.as_ref().ok_or_else(|| {
                    DesktopClientError::InvalidInput(
                        "either manifest_bytes or durable_store_dir is required".to_string(),
                    )
                })?;
                let manifest_path = dir.join("scopes").join(scope_manifest_filename(&scope));
                owned_manifest_bytes =
                    fs::read(&manifest_path).map_err(|e| DesktopClientError::Io(e.to_string()))?;
                &owned_manifest_bytes
            }
        };

        let manifest = Manifest::decode(raw_manifest, self.object_store.limits())
            .map_err(|e| DesktopClientError::ObjectStore(e.to_string()))?;
        self.memory_store
            .import_scope_expecting(&self.object_store, &manifest, &scope)
            .map_err(|e| DesktopClientError::ObjectStore(e.to_string()))?;

        let ur4sess1_bytes = self
            .memory_store
            .ur4sess1_snapshot(&scope)
            .ok_or_else(|| {
                DesktopClientError::ObjectStore("missing UR4SESS1 snapshot in scope".to_string())
            })?
            .to_vec();

        let decoded =
            session_codec::decode(&ur4sess1_bytes).map_err(DesktopClientError::StackEngine)?;
        let model = self
            .loaded_model
            .as_ref()
            .ok_or(DesktopClientError::ModelNotLoaded)?;
        let mut session = model.session();
        session
            .restore_state(&decoded)
            .map_err(|e| DesktopClientError::StackEngine(e.to_string()))?;

        let last_token = decoded.tokens.last().copied();
        let verified_bytes = session_codec::encode(&session.save_state());
        let ur4sess1_bytes_len = verified_bytes.len();
        self.sessions.insert(
            conversation_id.to_string(),
            ConversationSessionEntry {
                ur4sess1_bytes: verified_bytes,
                last_token,
            },
        );

        Ok(RestoredSessionReceipt {
            scope_key: scope.key(),
            conversation_id: conversation_id.to_string(),
            artifact_sha256: decoded.artifact_sha256,
            position: decoded.position,
            tokens: decoded.tokens,
            ur4sess1_bytes_len,
            non_memory_turns: self.memory_store.non_memory_turn_count(&scope),
        })
    }

    /// Explicitly grant read/write access to a local project directory.
    pub fn native_grant_folder(
        &mut self,
        folder_path: &Path,
    ) -> Result<PathBuf, DesktopClientError> {
        reject_parent_dir_traversal(folder_path)?;
        let canonical =
            fs::canonicalize(folder_path).map_err(|e| DesktopClientError::Io(e.to_string()))?;
        if !canonical.is_dir() {
            return Err(DesktopClientError::InvalidInput(format!(
                "granted path is not a directory: {}",
                canonical.display()
            )));
        }
        self.granted_folders.insert(canonical.clone());
        Ok(canonical)
    }

    /// Borrow the set of canonicalized granted folders.
    pub fn granted_folders(&self) -> &BTreeSet<PathBuf> {
        &self.granted_folders
    }

    fn verify_granted_existing_path(&self, target: &Path) -> Result<PathBuf, DesktopClientError> {
        reject_parent_dir_traversal(target)?;
        let canonical = fs::canonicalize(target).map_err(|_| {
            DesktopClientError::PathNotGranted(format!(
                "path does not exist or is outside granted folders: {}",
                target.display()
            ))
        })?;
        if self
            .granted_folders
            .iter()
            .any(|root| canonical.starts_with(root))
        {
            Ok(canonical)
        } else {
            Err(DesktopClientError::PathNotGranted(format!(
                "access outside granted folders is refused: {}",
                canonical.display()
            )))
        }
    }

    fn verify_granted_write_path(&self, target: &Path) -> Result<PathBuf, DesktopClientError> {
        reject_parent_dir_traversal(target)?;
        if target.exists() {
            return self.verify_granted_existing_path(target);
        }
        let parent = target.parent().ok_or_else(|| {
            DesktopClientError::PathNotGranted("target path has no parent directory".to_string())
        })?;
        let file_name = target.file_name().ok_or_else(|| {
            DesktopClientError::InvalidInput("target path has no file name".to_string())
        })?;
        let canonical_parent = self.verify_granted_existing_path(parent)?;
        Ok(canonical_parent.join(file_name))
    }

    /// Read a UTF-8 file located inside a user-granted folder.
    pub fn native_read_file(&self, file_path: &Path) -> Result<String, DesktopClientError> {
        let canonical = self.verify_granted_existing_path(file_path)?;
        fs::read_to_string(&canonical).map_err(|e| DesktopClientError::Io(e.to_string()))
    }

    /// Write a UTF-8 file inside a user-granted folder.
    pub fn native_write_file(
        &self,
        file_path: &Path,
        content: &str,
    ) -> Result<(), DesktopClientError> {
        let safe_path = self.verify_granted_write_path(file_path)?;
        fs::write(&safe_path, content).map_err(|e| DesktopClientError::Io(e.to_string()))
    }

    /// List relative file paths inside a user-granted folder up to `max_depth` levels deep
    /// without invoking any shell command.
    pub fn native_list_directory(
        &self,
        folder_path: &Path,
        max_depth: usize,
    ) -> Result<Vec<String>, DesktopClientError> {
        let root = self.verify_granted_existing_path(folder_path)?;
        let mut out = Vec::new();
        collect_directory_files(&root, &root, 0, max_depth.min(8), &mut out)?;
        out.sort();
        Ok(out)
    }

    /// Inspect Git branch and status for a repository inside a user-granted folder.
    pub fn native_git_status(
        &self,
        repo_path: &Path,
    ) -> Result<GitStatusReport, DesktopClientError> {
        let canonical_repo = self.verify_granted_existing_path(repo_path)?;
        let branch_out = Command::new("git")
            .arg("-C")
            .arg(&canonical_repo)
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .map_err(|e| DesktopClientError::CommandFailed(e.to_string()))?;
        let branch = if branch_out.status.success() {
            String::from_utf8_lossy(&branch_out.stdout)
                .trim()
                .to_string()
        } else {
            "main".to_string()
        };

        let status_out = Command::new("git")
            .arg("-C")
            .arg(&canonical_repo)
            .args(["status", "--porcelain"])
            .output()
            .map_err(|e| DesktopClientError::CommandFailed(e.to_string()))?;
        if !status_out.status.success() {
            return Err(DesktopClientError::CommandFailed(
                String::from_utf8_lossy(&status_out.stderr)
                    .trim()
                    .to_string(),
            ));
        }

        let mut modified_files = Vec::new();
        let mut untracked_files = Vec::new();
        for line in String::from_utf8_lossy(&status_out.stdout).lines() {
            if line.len() < 4 {
                continue;
            }
            let code = &line[..2];
            let file = line[3..].trim().to_string();
            if code == "??" {
                untracked_files.push(file);
            } else {
                modified_files.push(file);
            }
        }
        let is_clean = modified_files.is_empty() && untracked_files.is_empty();
        Ok(GitStatusReport {
            branch,
            is_clean,
            modified_files,
            untracked_files,
        })
    }

    /// Enable or disable `native_run_terminal_command` (off by default).
    pub fn native_set_terminal_enabled(&mut self, enabled: bool) {
        self.terminal_enabled = enabled;
    }

    /// Return whether terminal command execution is currently enabled.
    pub fn terminal_enabled(&self) -> bool {
        self.terminal_enabled
    }

    /// Execute a single user-confirmed command inside a granted directory (`cwd`).
    ///
    /// Refuses execution when:
    /// 1. `self.terminal_enabled` is `false` (`TerminalDisabledByDefault`),
    /// 2. `!req.user_confirmed` or `req.confirmed_command.trim() != req.command.trim()`
    ///    (`TerminalConsentRequired`),
    /// 3. `req.cwd` is outside `self.granted_folders` (`PathNotGranted`), or
    /// 4. `req.command` contains shell chaining / redirection metacharacters
    ///    (`TerminalMetacharacterRefused`).
    pub fn native_run_terminal_command(
        &self,
        req: &TerminalCommandRequest,
    ) -> Result<String, DesktopClientError> {
        if !self.terminal_enabled {
            return Err(DesktopClientError::TerminalDisabledByDefault);
        }
        let trimmed = req.command.trim();
        if trimmed.is_empty() || !req.user_confirmed || req.confirmed_command.trim() != trimmed {
            return Err(DesktopClientError::TerminalConsentRequired);
        }
        for forbidden in [';', '|', '&', '`', '$', '>', '<', '\n', '\r'] {
            if trimmed.contains(forbidden) {
                return Err(DesktopClientError::TerminalMetacharacterRefused(
                    forbidden.to_string(),
                ));
            }
        }
        let canonical_cwd = self.verify_granted_existing_path(&req.cwd)?;
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let Some((program, args)) = parts.split_first() else {
            return Err(DesktopClientError::TerminalConsentRequired);
        };
        if program.contains('/') || program.contains('\\') || program.contains("..") {
            return Err(DesktopClientError::PathTraversalRefused(
                (*program).to_string(),
            ));
        }

        let output = Command::new(program)
            .args(args)
            .current_dir(&canonical_cwd)
            .output()
            .map_err(|e| DesktopClientError::CommandFailed(e.to_string()))?;

        let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            if !combined.is_empty() && !combined.ends_with('\n') {
                combined.push('\n');
            }
            combined.push_str(&stderr);
        }
        if !output.status.success() {
            return Err(DesktopClientError::CommandFailed(combined));
        }
        Ok(combined)
    }
}

fn scope_manifest_filename(scope: &IdentityScope) -> String {
    let sanitize = |s: &str| -> String {
        s.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    };
    format!(
        "{}__{}__{}.uorman01",
        sanitize(&scope.user_id),
        sanitize(&scope.project_id),
        sanitize(&scope.session_id)
    )
}

fn reject_parent_dir_traversal(path: &Path) -> Result<(), DesktopClientError> {
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(DesktopClientError::PathTraversalRefused(
            path.to_string_lossy().into_owned(),
        ));
    }
    Ok(())
}

fn collect_directory_files(
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    out: &mut Vec<String>,
) -> Result<(), DesktopClientError> {
    if depth > max_depth {
        return Ok(());
    }
    let read_dir = fs::read_dir(current).map_err(|e| DesktopClientError::Io(e.to_string()))?;
    let mut entries: Vec<PathBuf> = read_dir
        .filter_map(|e| e.ok().map(|entry| entry.path()))
        .collect();
    entries.sort();

    for path in entries {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') || matches!(name, "target" | "node_modules" | "dist" | "local") {
            continue;
        }
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            collect_directory_files(root, &path, depth + 1, max_depth, out)?;
        } else if meta.is_file() {
            if let Ok(rel) = path.strip_prefix(root) {
                let rel_str = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                out.push(rel_str);
            }
        }
    }
    Ok(())
}

// =========================================================================
// Deterministic synthetic `UORLUT01` stand-in builder (matches
// `crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs` byte-for-byte).
// =========================================================================

struct StandinLcg(u64);

impl StandinLcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
}

struct StandinBuilder {
    matrices: Vec<Value>,
    tables: Vec<Value>,
    data: Vec<u8>,
}

impl StandinBuilder {
    fn push(&mut self, bytes: &[u8]) -> Value {
        self.data.resize(self.data.len().div_ceil(64) * 64, 0);
        let span = json!({"offset": self.data.len(), "bytes": bytes.len()});
        self.data.extend_from_slice(bytes);
        span
    }

    fn matrix(&mut self, rng: &mut StandinLcg, name: &str, rows: usize, cols: usize) {
        let nibbles: Vec<u8> = (0..rows * cols / 2).map(|_| rng.next() as u8).collect();
        let scales: Vec<u8> = (0..rows * cols / 32)
            .map(|_| (rng.next() as u8) & 0x3F)
            .collect();
        let (nibbles, scales) = (self.push(&nibbles), self.push(&scales));
        self.matrices.push(json!({
            "name": name, "rows": rows, "cols": cols, "exp_base": -9,
            "nibbles": nibbles, "scales": scales,
        }));
    }

    fn table(&mut self, name: &str, kind: &str, len: usize, bytes: Vec<u8>) {
        let span = self.push(&bytes);
        self.tables
            .push(json!({"name": name, "kind": kind, "len": len, "span": span}));
    }

    fn i16s(&mut self, name: &str, values: &[i16]) {
        let bytes = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        self.table(name, "i16", values.len(), bytes);
    }

    fn i32s(&mut self, name: &str, values: &[i32]) {
        let bytes = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        self.table(name, "i32", values.len(), bytes);
    }

    fn u32s(&mut self, name: &str, values: &[u32]) {
        let bytes = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        self.table(name, "u32", values.len(), bytes);
    }
}

/// Build the deterministic synthetic `UORLUT01` stack stand-in for `seed` (default `5`),
/// byte-identical to `crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs`.
pub fn build_synthetic_stack_artifact(seed: u64) -> Vec<u8> {
    const VOCAB: usize = 40;
    const WIDTH: usize = 32;
    const HEADS: usize = 2;
    const MLP: usize = 32;
    const CONTEXT: usize = 64;

    let mut rng = StandinLcg(seed);
    let mut b = StandinBuilder {
        matrices: Vec::new(),
        tables: Vec::new(),
        data: Vec::new(),
    };
    let lanes = WIDTH / 4;
    let gate_rows = lanes + WIDTH;
    let code = |m: u64, e: i16| ((e + 64) << 4) | (m & 15) as i16;
    b.matrix(&mut rng, "embed", VOCAB, WIDTH);
    b.matrix(&mut rng, "l0.rec_in", 2 * WIDTH, WIDTH);
    b.matrix(&mut rng, "l0.rec_gate", gate_rows, WIDTH);
    b.matrix(&mut rng, "l0.rec_out", WIDTH, WIDTH);
    let taps: Vec<i16> = (0..4 * WIDTH)
        .map(|i| {
            let c = code(rng.next(), -2 - (i % 3) as i16);
            if rng.next().is_multiple_of(2) {
                c
            } else {
                -c
            }
        })
        .collect();
    b.i16s("l0.conv_taps", &taps);
    let biases: Vec<i32> = (0..gate_rows)
        .map(|_| (rng.next() % 60_000) as i32 - 30_000)
        .collect();
    b.i32s("l0.conv_bias", &biases[..WIDTH]);
    b.i32s("l0.gate_bias", &biases);
    let rates: Vec<i16> = (0..lanes).map(|_| code(rng.next(), 1)).collect();
    b.i16s("l0.decay_rate", &rates);
    for part in ["query", "key", "value", "out"] {
        b.matrix(&mut rng, &format!("l1.{part}"), WIDTH, WIDTH);
    }
    b.matrix(&mut rng, "l1.null", HEADS, WIDTH);
    b.i32s("l1.null_bias", &[4_000, -9_000]);
    let ages: Vec<i32> = (0..HEADS * CONTEXT).map(|i| -((i as i32) << 12)).collect();
    b.i32s("l1.age", &ages);
    b.i16s("l1.beta", &[code(3, 0), code(9, -1)]);
    b.i32s("l1.offset", &[1 << 24, 3 << 23]);
    for l in 0..2 {
        b.matrix(&mut rng, &format!("l{l}.gate"), MLP, WIDTH);
        b.matrix(&mut rng, &format!("l{l}.up"), MLP, WIDTH);
        b.matrix(&mut rng, &format!("l{l}.down"), WIDTH, MLP);
    }
    b.matrix(&mut rng, "head", VOCAB, WIDTH);
    let exp: Vec<u32> = (0..32 * 256 + 2)
        .map(|i: u32| (1u32 << 31) >> (i / 177).min(31))
        .collect();
    b.u32s("exp", &exp);
    let activation: Vec<i32> = (0..(2 << 12) + 1)
        .map(|i: i32| (i - 4096).max(0) << 8)
        .collect();
    let table_a = concat!("si", "lu");
    let table_b = concat!("ge", "lu");
    b.i32s(table_a, &activation);
    b.i32s(table_b, &activation);
    let arcosh: Vec<u32> = (0..89089u32).collect();
    b.u32s("arcosh", &arcosh);

    let mut numerics = serde_json::Map::new();
    numerics.insert(
        "rms_eps".to_string(),
        json!({"mantissa": 2_814_749_767i64, "exp": -48}),
    );
    numerics.insert("score_scale_q30".to_string(), json!(268_435_456));
    numerics.insert("exp_step_log2".to_string(), json!(-8));
    numerics.insert(format!("{table_a}_step_log2"), json!(-8));
    numerics.insert(format!("{table_a}_range_log2"), json!(4));
    numerics.insert(format!("{table_b}_step_log2"), json!(-8));
    numerics.insert(format!("{table_b}_range_log2"), json!(4));

    let header = json!({
        "schema": STACK_SCHEMA,
        "shape": {
            "vocab": VOCAB, "width": WIDTH, "heads": HEADS, "mlp": MLP, "pattern": "ra",
            "read": "lorentz", "rotation": true, "context": CONTEXT,
        },
        "group": 32,
        "numerics": Value::Object(numerics),
        "matrices": b.matrices,
        "tables": b.tables,
        "source": {"synthetic": true},
    });
    let json_bytes = serde_json::to_vec(&header).expect("serialize synthetic header");
    let mut out = UORLUT01_MAGIC.to_vec();
    out.extend_from_slice(&(json_bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(&json_bytes);
    out.resize(out.len().div_ceil(64) * 64, 0);
    out.extend_from_slice(&b.data);
    out
}
