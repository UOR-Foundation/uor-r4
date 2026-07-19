//! Example direct-chat application built on R⁴ transformerless inference.
//!
//! Chat is a consumer of the core runtime, not a separate inference layer.

use std::fmt;
use std::path::PathBuf;

use crate::model::{default_model_reference, ModelError, ModelStore};
use uor_r4_core::transformerless::compiler::{self, Compiled};
use uor_r4_core::transformerless::runtime::{self, Runtime, Store};
use uor_r4_core::transformerless::scenarios::Tokenizer;

const MAX_CHAT_TOKENS: usize = 256;
const MAX_CHAT_HISTORY: usize = 4096;
const MAX_ANSWER_BYTES: usize = 16 * 1024;

/// A completed local chat turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatAnswer {
    /// Generated assistant text.
    pub text: String,
    /// Number of tokens generated for this turn.
    pub generated_tokens: usize,
}

/// Failure to load or run the local transformerless chat engine.
#[derive(Debug)]
#[non_exhaustive]
pub enum ChatError {
    /// A required file could not be read.
    Io(std::io::Error),
    /// The compiled artifact container was invalid.
    InvalidArtifacts,
    /// The graded store container was invalid.
    InvalidStore,
    /// Generation produced no tokens or could not be decoded.
    EmptyGeneration,
    /// Generation entered a repeated-token loop and was rejected.
    RepetitiveGeneration,
    /// No CID-addressed, capability-attested model was selected.
    MissingModel,
    /// The model bundle or its CID verification failed.
    Model(ModelError),
}

impl fmt::Display for ChatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "failed to load chat data: {error}"),
            Self::InvalidArtifacts => formatter.write_str("invalid transformerless artifacts"),
            Self::InvalidStore => formatter.write_str("invalid transformerless store"),
            Self::EmptyGeneration => formatter.write_str("transformerless produced no text"),
            Self::RepetitiveGeneration => formatter.write_str(
                "transformerless generation became repetitive; refusing a low-quality answer",
            ),
            Self::MissingModel => {
                formatter.write_str("no chat model selected; set TLESS_MODEL or pass --model")
            }
            Self::Model(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ChatError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidArtifacts
            | Self::InvalidStore
            | Self::EmptyGeneration
            | Self::RepetitiveGeneration
            | Self::MissingModel => None,
            Self::Model(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for ChatError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<ModelError> for ChatError {
    fn from(error: ModelError) -> Self {
        Self::Model(error)
    }
}

/// Builder for a direct local [`ChatEngine`].
#[derive(Debug, Clone)]
pub struct ChatEngineBuilder {
    max_tokens: usize,
    model: Option<String>,
}

impl Default for ChatEngineBuilder {
    fn default() -> Self {
        Self {
            max_tokens: 96,
            model: Some(default_model_reference()),
        }
    }
}

impl ChatEngineBuilder {
    /// Set the maximum number of generated tokens per turn.
    #[must_use]
    pub fn max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens.clamp(1, MAX_CHAT_TOKENS);
        self
    }

    /// Select a CID-addressed model manifest by name or UOR CID.
    #[must_use]
    pub fn model(mut self, reference: impl Into<String>) -> Self {
        self.model = Some(reference.into());
        self
    }

    /// Load all local data and construct the engine.
    pub fn build(self) -> Result<ChatEngine, ChatError> {
        let reference = self.model.as_deref().ok_or(ChatError::MissingModel)?;
        let model_store = ModelStore::from_env();
        let manifest = match model_store.read_manifest(reference) {
            Ok(manifest) => manifest,
            Err(ModelError::CompiledNotImported(path)) => {
                return build_local_compiled_engine(
                    &model_store,
                    &path,
                    reference,
                    self.max_tokens,
                );
            }
            Err(error) => return Err(error.into()),
        };
        manifest.validate_for_chat()?;
        if let Some(report) = &manifest.evaluation_report {
            let _ = model_store.get(report)?;
        }
        let artifact_bytes = model_store.get(&manifest.artifacts)?;
        let artifacts =
            compiler::parse_artifacts(&artifact_bytes).ok_or(ChatError::InvalidArtifacts)?;
        let store_bytes = model_store.get(&manifest.store)?;
        let store = runtime::parse_store(&store_bytes).ok_or(ChatError::InvalidStore)?;
        let tokenizer_bytes = model_store.get(&manifest.tokenizer)?;
        let tokenizer_path = write_tokenizer_cache(&manifest.tokenizer.cid, &tokenizer_bytes)?;
        let tokenizer = Tokenizer::try_load(&tokenizer_path)?;
        tracing::info!(
            model = %manifest.name,
            source_model = %manifest.source_model,
            artifact_cid = %manifest.artifacts.cid,
            store_cid = %manifest.store.cid,
            max_tokens = self.max_tokens,
            "transformerless chat engine loaded"
        );
        Ok(ChatEngine {
            artifacts,
            store,
            tokenizer,
            history: [0; MAX_CHAT_HISTORY],
            history_len: 0,
            max_tokens: self.max_tokens,
        })
    }
}

fn build_local_compiled_engine(
    model_store: &ModelStore,
    directory: &std::path::Path,
    reference: &str,
    max_tokens: usize,
) -> Result<ChatEngine, ChatError> {
    let artifact_bytes = std::fs::read(directory.join("tless_artifacts.bin"))?;
    let store_bytes = std::fs::read(directory.join("tless_store.bin"))?;
    let tokenizer_bytes = std::fs::read(directory.join("tokenizer.bin"))?;
    let artifacts =
        compiler::parse_artifacts(&artifact_bytes).ok_or(ChatError::InvalidArtifacts)?;
    let store = runtime::parse_store(&store_bytes).ok_or(ChatError::InvalidStore)?;

    // Content-address all local compiler outputs immediately. A manifest and
    // quality report remain optional metadata; integrity does not.
    let artifact_object = model_store.put(&artifact_bytes)?;
    let store_object = model_store.put(&store_bytes)?;
    let tokenizer_object = model_store.put(&tokenizer_bytes)?;
    let tokenizer_path = write_tokenizer_cache(&tokenizer_object.cid, &tokenizer_bytes)?;
    let tokenizer = Tokenizer::try_load(&tokenizer_path)?;
    tracing::warn!(
        model = reference,
        directory = %directory.display(),
        artifact_cid = %artifact_object.cid,
        store_cid = %store_object.cid,
        tokenizer_cid = %tokenizer_object.cid,
        "using a locally compiled bundle without an instruction-quality attestation"
    );
    Ok(ChatEngine {
        artifacts,
        store,
        tokenizer,
        history: [0; MAX_CHAT_HISTORY],
        history_len: 0,
        max_tokens,
    })
}

/// Stateful local chat engine with no HTTP server or background worker.
pub struct ChatEngine {
    artifacts: Compiled,
    store: Store,
    tokenizer: Tokenizer,
    history: [u16; MAX_CHAT_HISTORY],
    history_len: usize,
    max_tokens: usize,
}

impl ChatEngine {
    /// Start configuring a local chat engine.
    #[must_use]
    pub fn builder() -> ChatEngineBuilder {
        ChatEngineBuilder::default()
    }

    /// Generate one answer and retain its tokens as context for the next turn.
    pub fn ask(&mut self, question: &str) -> Result<ChatAnswer, ChatError> {
        let span = tracing::debug_span!("ask", question_bytes = question.len());
        let _guard = span.enter();
        hologram_answer(
            &self.artifacts,
            &self.store,
            &self.tokenizer,
            &mut self.history,
            &mut self.history_len,
            question,
            self.max_tokens,
        )
    }
}

fn hologram_answer(
    artifacts: &Compiled,
    store: &Store,
    tokenizer: &Tokenizer,
    history: &mut [u16; MAX_CHAT_HISTORY],
    history_len: &mut usize,
    question: &str,
    max_tokens: usize,
) -> Result<ChatAnswer, ChatError> {
    let mut question_tokens = [0u16; MAX_CHAT_HISTORY];
    let question_count = tokenizer.encode_into(question, &mut question_tokens)?;
    let question_tokens = if *history_len == 0 {
        &question_tokens[..question_count]
    } else {
        &question_tokens[1..question_count]
    };
    append_history(history, history_len, question_tokens);
    let mut runtime = Runtime::new(artifacts);
    let mut predictions = [runtime::Prediction::default(); MAX_CHAT_TOKENS];
    let prediction_count = runtime.generate_greedy_into(
        store,
        &history[..*history_len],
        &mut predictions[..max_tokens.min(MAX_CHAT_TOKENS)],
    );
    let mut generated = [0u16; MAX_CHAT_TOKENS];
    let mut generated_count = 0usize;
    for prediction in &predictions[..prediction_count] {
        if prediction.token == 1 {
            break;
        }
        generated[generated_count] = prediction.token;
        generated_count += 1;
        if repeated_suffix(&generated[..generated_count], 8) {
            return Err(ChatError::RepetitiveGeneration);
        }
    }
    let generated = &generated[..generated_count];
    if generated.is_empty() {
        return Err(ChatError::EmptyGeneration);
    }
    let mut answer_bytes = [0u8; MAX_ANSWER_BYTES];
    let answer_len = tokenizer.decode_into(generated, &mut answer_bytes)?;
    let text = String::from_utf8_lossy(&answer_bytes[..answer_len])
        .trim()
        .to_owned();
    if text.is_empty() {
        return Err(ChatError::EmptyGeneration);
    }
    append_history(history, history_len, generated);
    tracing::debug!(generated_tokens = generated.len(), "answer generated");
    Ok(ChatAnswer {
        text,
        generated_tokens: generated.len(),
    })
}

fn append_history(history: &mut [u16; MAX_CHAT_HISTORY], len: &mut usize, tokens: &[u16]) {
    let tokens = &tokens[tokens.len().saturating_sub(MAX_CHAT_HISTORY)..];
    let overflow = len
        .saturating_add(tokens.len())
        .saturating_sub(MAX_CHAT_HISTORY);
    if overflow > 0 {
        history.copy_within(overflow..*len, 0);
        *len -= overflow;
    }
    history[*len..*len + tokens.len()].copy_from_slice(tokens);
    *len += tokens.len();
}

fn repeated_suffix(tokens: &[u16], width: usize) -> bool {
    if tokens.len() < width * 2 {
        return false;
    }
    let suffix = &tokens[tokens.len() - width..];
    tokens[..tokens.len() - width]
        .windows(width)
        .any(|window| window == suffix)
}

fn write_tokenizer_cache(cid: &str, bytes: &[u8]) -> Result<PathBuf, std::io::Error> {
    let hash = cid.strip_prefix("blake3:").unwrap_or("tokenizer");
    let directory = std::env::temp_dir().join("uor-r4-tokenizers");
    std::fs::create_dir_all(&directory)?;
    let path = directory.join(format!("{hash}.bin"));
    if !path.exists() {
        std::fs::write(&path, bytes)?;
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::repeated_suffix;

    #[test]
    fn repetition_guard_detects_repeated_token_windows() {
        assert!(repeated_suffix(&[1, 2, 3, 4, 1, 2, 3, 4], 4));
        assert!(!repeated_suffix(&[1, 2, 3, 4, 1, 2, 3, 5], 4));
    }
}
