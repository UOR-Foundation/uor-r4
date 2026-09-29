//! Safe loading and position-complete evaluation of the upstream Candle Llama.
//!
//! All source projections retain their checkpoint dimensions. Checkpoint BF16
//! values are converted exactly to F32 before computation. The upstream model
//! returns only the last position, so [`CandleLlamaTeacher::logits`] advances a
//! fresh KV cache one token at a time. This is a parity interface, not a throughput
//! claim for a batched student.

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::llama::{Cache, Config, Llama, LlamaConfig};

/// Errors at the offline conversion boundary.
#[derive(Debug)]
pub enum ConversionError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Tensor(candle_core::Error),
    UnsupportedConfig(String),
    InvalidInput(String),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "conversion input: {error}"),
            Self::Json(error) => write!(f, "conversion config: {error}"),
            Self::Tensor(error) => write!(f, "Candle Llama: {error}"),
            Self::UnsupportedConfig(error) => write!(f, "unsupported Llama config: {error}"),
            Self::InvalidInput(error) => write!(f, "invalid Llama input: {error}"),
        }
    }
}

impl std::error::Error for ConversionError {}

impl From<std::io::Error> for ConversionError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for ConversionError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<candle_core::Error> for ConversionError {
    fn from(error: candle_core::Error) -> Self {
        Self::Tensor(error)
    }
}

pub type Result<T> = std::result::Result<T, ConversionError>;

/// Reusable full-width floating-point teacher, for offline conversion only.
pub struct CandleLlamaTeacher {
    model: Llama,
    config: Config,
    empty_cache: Cache,
    device: Device,
    tokenizer_path: PathBuf,
}

impl CandleLlamaTeacher {
    /// Load the local single-file Hugging Face Llama checkpoint without mmap or
    /// unsafe code. Token IDs must come from this source's `tokenizer.json`.
    pub fn load(source: &Path, device: &Device) -> Result<Self> {
        let config = parse_config(&fs::read(source.join("config.json"))?)?;
        let tokenizer_path = source.join("tokenizer.json");
        if !fs::metadata(&tokenizer_path)?.is_file() {
            return Err(ConversionError::InvalidInput(format!(
                "{} is not a tokenizer file",
                tokenizer_path.display()
            )));
        }
        let tensors = candle_core::safetensors::load(source.join("model.safetensors"), device)?
            .into_iter()
            .map(|(name, tensor)| Ok((name, tensor.to_dtype(DType::F32)?)))
            .collect::<candle_core::Result<HashMap<_, _>>>()?;
        Self::from_tensors(tensors, config, device, tokenizer_path)
    }

    fn from_tensors(
        tensors: HashMap<String, Tensor>,
        config: Config,
        device: &Device,
        tokenizer_path: PathBuf,
    ) -> Result<Self> {
        let vb = VarBuilder::from_tensors(tensors, DType::F32, device);
        // Candle 0.9.2's Llama::load unwraps errors from block construction.
        // Resolve every tensor and shape first, so absent/malformed checkpoint
        // weights return a typed error before entering that upstream code.
        validate_weights(&vb, &config)?;
        let model = Llama::load(vb, &config)?;
        let empty_cache = Cache::new(true, DType::F32, &config, device)?;
        Ok(Self {
            model,
            config,
            empty_cache,
            device: device.clone(),
            tokenizer_path,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn tokenizer_path(&self) -> &Path {
        &self.tokenizer_path
    }

    /// Return `[position][vocabulary]` logits. Row `t` predicts the token after
    /// `tokens[t]`; every call starts at absolute position zero with empty KV.
    pub fn logits(&self, tokens: &[u32]) -> Result<Vec<Vec<f32>>> {
        self.validate_tokens(tokens)?;
        let mut cache = self.empty_cache.clone();
        let mut rows = Vec::with_capacity(tokens.len());
        for (position, token) in tokens.iter().enumerate() {
            let input = Tensor::from_slice(&[*token], (1, 1), &self.device)?;
            let logits = self.model.forward(&input, position, &mut cache)?;
            rows.push(logits.squeeze(0)?.to_vec1::<f32>()?);
        }
        Ok(rows)
    }

    /// Return the final-position logits using one dense prefill, or a prefill
    /// ending at `split` followed by one-token cached continuation.
    ///
    /// Candle 0.9.2 creates a square causal mask for multi-token inputs; cached
    /// multi-token tails need a rectangular mask that its public implementation
    /// does not provide. This interface deliberately continues with singletons.
    pub fn logits_prefill(&self, tokens: &[u32], split: Option<usize>) -> Result<Vec<f32>> {
        self.validate_tokens(tokens)?;
        let prefix = split.unwrap_or(tokens.len());
        if prefix == 0 || prefix > tokens.len() {
            return Err(ConversionError::InvalidInput(format!(
                "prefill split {prefix} must be within 1..={}",
                tokens.len()
            )));
        }
        let mut cache = self.empty_cache.clone();
        let input = Tensor::from_slice(&tokens[..prefix], (1, prefix), &self.device)?;
        let mut logits = self.model.forward(&input, 0, &mut cache)?;
        for (position, token) in tokens.iter().enumerate().skip(prefix) {
            let input = Tensor::from_slice(&[*token], (1, 1), &self.device)?;
            logits = self.model.forward(&input, position, &mut cache)?;
        }
        Ok(logits.squeeze(0)?.to_vec1::<f32>()?)
    }

    fn validate_tokens(&self, tokens: &[u32]) -> Result<()> {
        if tokens.is_empty() || tokens.len() > self.config.max_position_embeddings {
            return Err(ConversionError::InvalidInput(format!(
                "token count {} must be within 1..={}",
                tokens.len(),
                self.config.max_position_embeddings
            )));
        }
        if let Some((position, token)) = tokens
            .iter()
            .enumerate()
            .find(|(_, token)| **token as usize >= self.config.vocab_size)
        {
            return Err(ConversionError::InvalidInput(format!(
                "token {token} at position {position} exceeds vocabulary {}",
                self.config.vocab_size
            )));
        }
        Ok(())
    }
}

fn unsupported(message: impl Into<String>) -> ConversionError {
    ConversionError::UnsupportedConfig(message.into())
}

pub(super) fn parse_config(bytes: &[u8]) -> Result<Config> {
    let raw: serde_json::Value = serde_json::from_slice(bytes)?;
    if raw.get("model_type").and_then(|v| v.as_str()) != Some("llama") {
        return Err(unsupported("model_type must be llama"));
    }
    if raw.get("hidden_act").and_then(|v| v.as_str()) != Some("silu") {
        return Err(unsupported("hidden_act must be silu"));
    }
    for field in ["attention_bias", "mlp_bias", "rope_interleaved"] {
        if let Some(value) = raw.get(field) {
            if value.as_bool() != Some(false) {
                return Err(unsupported(format!("{field} must be false")));
            }
        }
    }
    for field in ["rope_scaling", "sliding_window"] {
        if raw.get(field).is_some_and(|value| !value.is_null()) {
            return Err(unsupported(format!("{field} is not supported")));
        }
    }
    if raw
        .get("pretraining_tp")
        .is_some_and(|value| value.as_u64() != Some(1))
    {
        return Err(unsupported("pretraining_tp must be 1"));
    }
    if raw
        .get("partial_rotary_factor")
        .is_some_and(|value| value.as_f64() != Some(1.0))
    {
        return Err(unsupported("partial_rotary_factor must be 1"));
    }
    let config = serde_json::from_value::<LlamaConfig>(raw.clone())?.into_config(false);
    if config.hidden_size == 0
        || config.intermediate_size == 0
        || config.vocab_size == 0
        || config.num_hidden_layers == 0
        || config.num_attention_heads == 0
        || config.num_key_value_heads == 0
        || config.max_position_embeddings == 0
        || config.max_position_embeddings > u32::MAX as usize
    {
        return Err(unsupported(
            "dimensions must be nonzero; positions must fit u32",
        ));
    }
    if config.hidden_size % config.num_attention_heads != 0
        || config.num_attention_heads % config.num_key_value_heads != 0
        || (config.hidden_size / config.num_attention_heads) % 2 != 0
    {
        return Err(unsupported(
            "head dimensions must be even and Q/KV grouping integral",
        ));
    }
    if raw.get("head_dim").is_some_and(|value| {
        value.as_u64() != Some((config.hidden_size / config.num_attention_heads) as u64)
    }) {
        return Err(unsupported(
            "head_dim differs from hidden_size / num_attention_heads",
        ));
    }
    if !config.rms_norm_eps.is_finite()
        || config.rms_norm_eps <= 0.0
        || !config.rope_theta.is_finite()
        || config.rope_theta <= 0.0
    {
        return Err(unsupported(
            "RMS epsilon and RoPE theta must be finite and positive",
        ));
    }
    Ok(config)
}

fn weight_shapes(config: &Config) -> Vec<(String, Vec<usize>)> {
    let width = config.hidden_size;
    let kv_width = width / config.num_attention_heads * config.num_key_value_heads;
    let mut weights = vec![
        (
            "model.embed_tokens.weight".into(),
            vec![config.vocab_size, width],
        ),
        ("model.norm.weight".into(), vec![width]),
    ];
    if !config.tie_word_embeddings {
        weights.push(("lm_head.weight".into(), vec![config.vocab_size, width]));
    }
    for layer in 0..config.num_hidden_layers {
        let prefix = format!("model.layers.{layer}");
        for norm in ["input_layernorm", "post_attention_layernorm"] {
            weights.push((format!("{prefix}.{norm}.weight"), vec![width]));
        }
        for (name, rows, cols) in [
            ("self_attn.q_proj", width, width),
            ("self_attn.k_proj", kv_width, width),
            ("self_attn.v_proj", kv_width, width),
            ("self_attn.o_proj", width, width),
            ("mlp.gate_proj", config.intermediate_size, width),
            ("mlp.up_proj", config.intermediate_size, width),
            ("mlp.down_proj", width, config.intermediate_size),
        ] {
            weights.push((format!("{prefix}.{name}.weight"), vec![rows, cols]));
        }
    }
    weights
}

fn validate_weights(vb: &VarBuilder<'_>, config: &Config) -> Result<()> {
    for (name, shape) in weight_shapes(config) {
        let _ = vb.get(shape, &name)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_config() -> serde_json::Value {
        serde_json::json!({
            "model_type": "llama", "hidden_act": "silu",
            "hidden_size": 8, "intermediate_size": 12, "vocab_size": 7,
            "num_hidden_layers": 2, "num_attention_heads": 2,
            "num_key_value_heads": 1, "rms_norm_eps": 1e-5,
            "rope_theta": 100000.0, "max_position_embeddings": 32,
            "tie_word_embeddings": true
        })
    }

    fn tiny_model() -> Result<CandleLlamaTeacher> {
        let config = parse_config(&serde_json::to_vec(&tiny_config())?)?;
        let mut tensors = HashMap::new();
        for (ordinal, (name, shape)) in weight_shapes(&config).into_iter().enumerate() {
            let values = (0..shape.iter().product::<usize>())
                .map(|index| {
                    if shape.len() == 1 {
                        1.0
                    } else {
                        ((index * 13 + ordinal * 7) % 37) as f32 / 37.0 - 0.5
                    }
                })
                .collect::<Vec<_>>();
            tensors.insert(name, Tensor::from_vec(values, shape, &Device::Cpu)?);
        }
        CandleLlamaTeacher::from_tensors(tensors, config, &Device::Cpu, PathBuf::new())
    }

    #[test]
    fn cached_positions_match_full_prefix_and_cache_resets() -> Result<()> {
        let model = tiny_model()?;
        let tokens = [1, 3, 2, 5];
        let rows = model.logits(&tokens)?;
        for (position, row) in rows.iter().enumerate() {
            let dense = model.logits_prefill(&tokens[..=position], None)?;
            for (cached, full) in row.iter().zip(&dense) {
                assert!((cached - full).abs() < 1e-5, "{cached} vs {full}");
            }
        }
        let split = model.logits_prefill(&tokens, Some(2))?;
        for (cached, full) in split.iter().zip(&rows[3]) {
            assert!((cached - full).abs() < 1e-5);
        }
        assert_eq!(rows, model.logits(&tokens)?);
        assert!(model.logits(&[]).is_err());
        assert!(model.logits(&[7]).is_err());
        assert!(model.logits(&[0; 33]).is_err());
        assert!(model.logits_prefill(&tokens, Some(0)).is_err());
        Ok(())
    }

    #[test]
    fn malformed_heads_and_unsupported_config_are_errors() -> Result<()> {
        for (field, value) in [
            ("num_attention_heads", serde_json::json!(0)),
            ("num_key_value_heads", serde_json::json!(3)),
            ("hidden_size", serde_json::json!(6)),
            ("hidden_act", serde_json::json!("gelu")),
            ("attention_bias", serde_json::json!(true)),
            ("rope_interleaved", serde_json::json!(true)),
            ("rope_scaling", serde_json::json!({"factor": 2})),
        ] {
            let mut config = tiny_config();
            config[field] = value;
            assert!(
                parse_config(&serde_json::to_vec(&config)?).is_err(),
                "{field}"
            );
        }
        Ok(())
    }

    #[test]
    fn missing_block_weight_is_an_error_before_upstream_loader() -> Result<()> {
        let config = parse_config(&serde_json::to_vec(&tiny_config())?)?;
        let mut tensors = HashMap::new();
        for (name, shape) in weight_shapes(&config) {
            if name != "model.layers.1.self_attn.q_proj.weight" {
                tensors.insert(name, Tensor::zeros(shape, DType::F32, &Device::Cpu)?);
            }
        }
        let result =
            CandleLlamaTeacher::from_tensors(tensors, config, &Device::Cpu, PathBuf::new());
        assert!(matches!(result, Err(ConversionError::Tensor(_))));
        Ok(())
    }
}
