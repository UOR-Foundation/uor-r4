//! Bounded, seeded batched generation for offline teacher data.
//!
//! Prompts are token IDs from the unchanged source tokenizer. This full-prefix
//! implementation right-pads only future positions, recomputes each prefix,
//! and has no KV-cache or optimized throughput claim. Record all options, source
//! identities and stable seed IDs with a teacher-data artifact.

use candle_core::{IndexOp, Tensor};
use candle_transformers::generation::{LogitsProcessor, Sampling};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::model::TrackBModel;
use crate::{invalid, Result};

pub const SEED_DERIVATION_VERSION: &str = "uor-r4-track-b-generation-seed-v1";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationPrompt {
    /// Stable across batching and scheduling; this selects the prompt's RNG.
    pub seed_id: u64,
    pub tokens: Vec<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SamplingOptions {
    Greedy,
    Sample {
        temperature: f64,
        top_k: Option<usize>,
        top_p: Option<f64>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationOptions {
    pub sampling: SamplingOptions,
    pub max_new_tokens: usize,
    /// Maximum B*T for any right-padded forward, before allocating its input.
    /// The full vocabulary output alone costs B*T*V*4 bytes. Callers must also
    /// budget dense attention, activations, weights and unified-memory use.
    pub max_padded_tokens: usize,
    pub pad_token: u32,
    /// Stop only on generated EOS, never because a prompt contains this ID.
    pub eos_tokens: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopReason {
    Eos,
    NewTokenLimit,
    ContextLimit,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationOutput {
    pub seed_id: u64,
    pub derived_seed: u64,
    /// Includes generated EOS when present, excludes all prompt/padding tokens.
    pub tokens: Vec<u32>,
    pub stop_reason: StopReason,
}

impl TrackBModel {
    /// Generate on the default dense attention kernel. Equal stable seed IDs
    /// select equal RNG streams regardless of row order or other rows stopping.
    /// Floating-point kernels may still differ across devices or builds.
    pub fn generate(
        &self,
        prompts: &[GenerationPrompt],
        options: &GenerationOptions,
        seed: u64,
    ) -> Result<Vec<GenerationOutput>> {
        generate_with(
            prompts,
            options,
            seed,
            self.shape().vocab,
            self.max_position_embeddings(),
            |ids, batch, time| self.forward(ids, batch, time),
        )
    }
}

fn sampling(options: &SamplingOptions, vocab: usize) -> Result<Sampling> {
    match options {
        SamplingOptions::Greedy => Ok(Sampling::ArgMax),
        SamplingOptions::Sample {
            temperature,
            top_k,
            top_p,
        } => {
            if !temperature.is_finite()
                || *temperature <= 0.0
                || !(*temperature as f32).is_finite()
                || (*temperature as f32) <= 0.0
                || top_k.is_some_and(|k| k == 0 || k > vocab)
                || top_p.is_some_and(|p| !p.is_finite() || p <= 0.0 || p > 1.0 || (p as f32) <= 0.0)
            {
                return Err(invalid("invalid Track B sampling temperature/top-k/top-p"));
            }
            Ok(match (top_k, top_p) {
                (None, None) => Sampling::All {
                    temperature: *temperature,
                },
                (Some(k), None) => Sampling::TopK {
                    k: *k,
                    temperature: *temperature,
                },
                (None, Some(p)) => Sampling::TopP {
                    p: *p,
                    temperature: *temperature,
                },
                // Mask to K before TopP below, so softmax renormalizes the
                // retained distribution. Candle 0.9.2 TopKThenTopP compares p
                // to original probability mass and does not renormalize it.
                (Some(_), Some(p)) => Sampling::TopP {
                    p: *p,
                    temperature: *temperature,
                },
            })
        }
    }
}

fn sample_token(
    logits: &Tensor,
    options: &SamplingOptions,
    processor: &mut LogitsProcessor,
) -> Result<u32> {
    if let SamplingOptions::Sample {
        top_k: Some(k),
        top_p: Some(_),
        ..
    } = options
    {
        let mut values = logits.to_vec1::<f32>()?;
        let mut order = (0..values.len()).collect::<Vec<_>>();
        order.sort_by(|&a, &b| values[b].total_cmp(&values[a]).then(a.cmp(&b)));
        for &index in order.iter().skip(*k) {
            values[index] = f32::NEG_INFINITY;
        }
        let masked = Tensor::from_vec(values, logits.shape(), &candle_core::Device::Cpu)?;
        Ok(processor.sample(&masked)?)
    } else {
        Ok(processor.sample(logits)?)
    }
}

fn row_seed(seed: u64, seed_id: u64) -> u64 {
    let mut hash = Sha256::new();
    hash.update(SEED_DERIVATION_VERSION.as_bytes());
    hash.update(seed.to_le_bytes());
    hash.update(seed_id.to_le_bytes());
    let bytes = hash.finalize();
    let mut prefix = [0u8; 8];
    prefix.copy_from_slice(&bytes[..8]);
    u64::from_le_bytes(prefix)
}

fn generate_with(
    prompts: &[GenerationPrompt],
    options: &GenerationOptions,
    seed: u64,
    vocab: usize,
    max_context: usize,
    mut forward: impl FnMut(&[u32], usize, usize) -> Result<Tensor>,
) -> Result<Vec<GenerationOutput>> {
    let sampling = sampling(&options.sampling, vocab)?;
    let unique_ids = prompts
        .iter()
        .map(|prompt| prompt.seed_id)
        .collect::<std::collections::BTreeSet<_>>();
    if prompts.is_empty()
        || vocab == 0
        || options.max_new_tokens == 0
        || options.max_padded_tokens == 0
        || unique_ids.len() != prompts.len()
        || options.pad_token as usize >= vocab
        || options.eos_tokens.iter().any(|id| *id as usize >= vocab)
        || prompts.iter().any(|prompt| {
            prompt.tokens.is_empty()
                || prompt.tokens.len() > max_context
                || prompt.tokens.iter().any(|id| *id as usize >= vocab)
        })
    {
        return Err(invalid("invalid Track B generation prompt or bounds"));
    }
    let mut sequences = prompts.iter().map(|p| p.tokens.clone()).collect::<Vec<_>>();
    let mut processors = prompts
        .iter()
        .map(|prompt| {
            LogitsProcessor::from_sampling(row_seed(seed, prompt.seed_id), sampling.clone())
        })
        .collect::<Vec<_>>();
    let mut result = prompts
        .iter()
        .map(|prompt| GenerationOutput {
            seed_id: prompt.seed_id,
            derived_seed: row_seed(seed, prompt.seed_id),
            tokens: Vec::new(),
            stop_reason: StopReason::ContextLimit,
        })
        .collect::<Vec<_>>();
    let mut done = sequences
        .iter()
        .map(|s| s.len() == max_context)
        .collect::<Vec<_>>();
    for _ in 0..options.max_new_tokens {
        // Compact active rows: stopped sequences consume no compute or RNG.
        let active = done
            .iter()
            .enumerate()
            .filter_map(|(i, done)| (!done).then_some(i))
            .collect::<Vec<_>>();
        if active.is_empty() {
            break;
        }
        let time = active
            .iter()
            .map(|&i| sequences[i].len())
            .max()
            .ok_or_else(|| invalid("empty active generation batch"))?;
        let padded_tokens = active
            .len()
            .checked_mul(time)
            .ok_or_else(|| invalid("generation padded-token count overflow"))?;
        if padded_tokens > options.max_padded_tokens {
            return Err(invalid(
                "generation exceeds configured padded-token allocation bound",
            ));
        }
        let mut ids = vec![options.pad_token; padded_tokens];
        for (row, &index) in active.iter().enumerate() {
            let sequence = &sequences[index];
            ids[row * time..row * time + sequence.len()].copy_from_slice(sequence);
        }
        let logits = forward(&ids, active.len(), time)?;
        if logits.dims() != [active.len(), time, vocab] {
            return Err(invalid(
                "generation forward must return [batch,time,vocabulary]",
            ));
        }
        for (row, &index) in active.iter().enumerate() {
            let row_logits = logits.i((row, sequences[index].len() - 1))?;
            if row_logits
                .to_vec1::<f32>()?
                .iter()
                .any(|value| !value.is_finite())
            {
                return Err(invalid("generation encountered nonfinite logits"));
            }
            let next = sample_token(&row_logits, &options.sampling, &mut processors[index])?;
            sequences[index].push(next);
            result[index].tokens.push(next);
            let stop = if options.eos_tokens.contains(&next) {
                Some(StopReason::Eos)
            } else if sequences[index].len() == max_context {
                Some(StopReason::ContextLimit)
            } else if result[index].tokens.len() == options.max_new_tokens {
                Some(StopReason::NewTokenLimit)
            } else {
                None
            };
            if let Some(reason) = stop {
                result[index].stop_reason = reason;
                done[index] = true;
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;

    fn options() -> GenerationOptions {
        GenerationOptions {
            sampling: SamplingOptions::Greedy,
            max_new_tokens: 4,
            max_padded_tokens: 100,
            pad_token: 0,
            eos_tokens: vec![3],
        }
    }

    fn next_logits(ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        let mut values = vec![-10f32; batch * time * 5];
        for (row, &id) in ids.iter().enumerate() {
            values[row * 5 + ((id + 1) % 5) as usize] = 10.0;
        }
        Ok(Tensor::from_vec(values, (batch, time, 5), &Device::Cpu)?)
    }

    #[test]
    fn variable_prompts_stop_independently_and_padding_is_not_sampled() -> Result<()> {
        let prompts = vec![
            GenerationPrompt {
                seed_id: 7,
                tokens: vec![1],
            },
            GenerationPrompt {
                seed_id: 9,
                tokens: vec![3, 0, 2],
            },
        ];
        let output = generate_with(&prompts, &options(), 42, 5, 8, next_logits)?;
        assert_eq!(output[0].tokens, [2, 3]);
        assert_eq!(output[1].tokens, [3]);
        assert!(output.iter().all(|row| row.stop_reason == StopReason::Eos));
        let limited = generate_with(&prompts[..1], &options(), 42, 5, 2, next_logits)?;
        assert_eq!(limited[0].tokens, [2]);
        assert_eq!(limited[0].stop_reason, StopReason::ContextLimit);
        Ok(())
    }

    #[test]
    fn random_streams_are_stable_when_batch_order_changes() -> Result<()> {
        let prompts = vec![
            GenerationPrompt {
                seed_id: 17,
                tokens: vec![1],
            },
            GenerationPrompt {
                seed_id: 23,
                tokens: vec![2, 1],
            },
        ];
        let mut options = options();
        options.eos_tokens.clear();
        options.sampling = SamplingOptions::Sample {
            temperature: 1.0,
            top_k: None,
            top_p: Some(0.9),
        };
        let uniform = |_ids: &[u32], b, t| -> Result<Tensor> {
            Ok(Tensor::zeros(
                (b, t, 5),
                candle_core::DType::F32,
                &Device::Cpu,
            )?)
        };
        let first = generate_with(&prompts, &options, 41, 5, 12, uniform)?;
        let reverse = generate_with(
            &[prompts[1].clone(), prompts[0].clone()],
            &options,
            41,
            5,
            12,
            uniform,
        )?;
        assert_eq!(first[0].tokens, reverse[1].tokens);
        assert_eq!(first[1].tokens, reverse[0].tokens);
        assert!(first
            .iter()
            .all(|row| row.stop_reason == StopReason::NewTokenLimit));
        Ok(())
    }

    #[test]
    fn invalid_generation_inputs_fail_before_forward() {
        let prompts = vec![GenerationPrompt {
            seed_id: 0,
            tokens: vec![1, 2],
        }];
        let mut options = options();
        options.max_padded_tokens = 1;
        assert!(generate_with(&prompts, &options, 0, 5, 8, |_, _, _| panic!(
            "must reject before forward"
        ))
        .is_err());
        options.sampling = SamplingOptions::Sample {
            temperature: f64::NAN,
            top_k: None,
            top_p: None,
        };
        assert!(generate_with(&prompts, &options, 0, 5, 8, |_, _, _| panic!(
            "must reject before forward"
        ))
        .is_err());
    }

    #[test]
    fn combined_top_k_top_p_renormalizes_before_nucleus_cutoff() -> Result<()> {
        let options = SamplingOptions::Sample {
            temperature: 1.0,
            top_k: Some(2),
            top_p: Some(0.5),
        };
        let logits = Tensor::from_vec(
            vec![
                0.30f32.ln(),
                0.20f32.ln(),
                0.18f32.ln(),
                0.17f32.ln(),
                0.15f32.ln(),
            ],
            5,
            &Device::Cpu,
        )?;
        for seed in 0..64 {
            let mut processor = LogitsProcessor::from_sampling(seed, sampling(&options, 5)?);
            // Retained K mass is .5; renormalization gives .6/.4. Nucleus .5
            // includes only token zero, independent of the sampled RNG value.
            assert_eq!(sample_token(&logits, &options, &mut processor)?, 0);
        }
        assert!(sampling(
            &SamplingOptions::Sample {
                temperature: 1.0,
                top_k: None,
                top_p: Some(1e-100)
            },
            5
        )
        .is_err());
        Ok(())
    }
}
