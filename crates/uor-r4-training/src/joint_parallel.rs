//! Synchronous CPU gradients over independent full-window batch rows.
//!
//! Shared model variables are read-only until every worker has joined. Each
//! worker owns its complete recurrent graph; no sequence or gradient history
//! is truncated. Named mean gradients are combined in shard order, before the
//! caller performs one global clip and one optimizer update. F32 reductions
//! differ from an unsplit batch; bitwise trajectory equivalence is not claimed.
//!
//! [`batch_gradients`] is the historical population-mean objective.
//! [`masked_batch_gradients`] is the response-masked dialogue objective: each
//! shard reports its own supervised mean, and shards are combined by supervised
//! target count so the result is the full-batch masked mean gradient.

use candle_core::backprop::GradStore;
use candle_core::{DType, Device};

use crate::joint_model::{JointModel, ReadMode};
use crate::{invalid, Result, TrainingError};

const WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;

pub struct BatchGradients {
    pub mean_nll: f32,
    pub gradients: GradStore,
}

/// Response-masked gradients over one batch. `mean_nll` is the full-batch
/// masked mean and `supervised` is the number of contributing response targets.
pub struct MaskedBatchGradients {
    pub mean_nll: f32,
    pub supervised: f64,
    pub gradients: GradStore,
}

struct ShardGradients {
    mean_nll: f32,
    supervised: f64,
    gradients: GradStore,
}

fn shard_gradients(
    model: &JointModel,
    inputs: &[u32],
    targets: &[u32],
    batch: usize,
    time: usize,
) -> Result<ShardGradients> {
    let output = model.forward(inputs, batch, time, ReadMode::Enabled, true)?;
    let loss = output.loss(targets)?;
    let mean_nll = loss.to_scalar::<f32>()?;
    if !mean_nll.is_finite() {
        return Err(invalid("nonfinite training shard loss"));
    }
    let gradients = loss.backward()?;
    Ok(ShardGradients {
        mean_nll,
        supervised: (batch * time) as f64,
        gradients,
    })
}

fn masked_shard_gradients(
    model: &JointModel,
    inputs: &[u32],
    targets: &[u32],
    masks: &[u8],
    batch: usize,
    time: usize,
) -> Result<ShardGradients> {
    let output = model.forward(inputs, batch, time, ReadMode::Enabled, true)?;
    let (nats, supervised) = crate::dialogue::masked_nats(&output.probabilities, targets, masks)?;
    if supervised <= 0.0 {
        return Err(invalid("gradient shard has no supervised response targets"));
    }
    let loss = nats.affine(1.0 / supervised, 0.0)?;
    let mean_nll = loss.to_scalar::<f32>()?;
    if !mean_nll.is_finite() {
        return Err(invalid("nonfinite masked training shard loss"));
    }
    let gradients = loss.backward()?;
    Ok(ShardGradients {
        mean_nll,
        supervised,
        gradients,
    })
}

/// Run one worker per shard and join every spawned worker even after an error.
/// Returning early would let an unjoined worker panic escape through
/// `thread::scope`.
fn parallel_shards<W>(shards: usize, worker: W) -> Result<Vec<ShardGradients>>
where
    W: Fn(usize) -> Result<ShardGradients> + Sync,
{
    let worker = &worker;
    std::thread::scope(|scope| -> Result<Vec<ShardGradients>> {
        let mut handles = Vec::with_capacity(shards);
        let mut first_error: Option<TrainingError> = None;
        for shard in 0..shards {
            match std::thread::Builder::new()
                .name(format!("joint-gradient-{shard}"))
                .stack_size(WORKER_STACK_BYTES)
                .spawn_scoped(scope, move || worker(shard))
            {
                Ok(handle) => handles.push(handle),
                Err(error) => {
                    first_error = Some(error.into());
                    break;
                }
            }
        }
        let mut outputs = Vec::with_capacity(shards);
        for handle in handles {
            match handle.join() {
                Ok(Ok(output)) => outputs.push(output),
                Ok(Err(error)) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
                Err(_) => {
                    if first_error.is_none() {
                        first_error = Some(invalid("CPU gradient worker panicked"));
                    }
                }
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }
        Ok(outputs)
    })
}

/// Combine shard gradients in fixed shard order using `weights[i]` for shard
/// `i`. The first store is reused because Candle intentionally keeps
/// `GradStore::new` private; insertion uses the shared original `Var` IDs,
/// never shard IDs. `weights` must have one entry per shard.
fn combine_shard_gradients(
    model: &JointModel,
    shards: &mut [ShardGradients],
    weights: &[f64],
) -> Result<()> {
    if shards.is_empty() || shards.len() != weights.len() {
        return Err(invalid("CPU gradient shard result count mismatch"));
    }
    let (first, others) = shards
        .split_first_mut()
        .ok_or_else(|| invalid("CPU gradient workers returned no results"))?;
    for (name, variable) in model.variables() {
        let first_gradient = first
            .gradients
            .remove(variable.as_tensor())
            .ok_or_else(|| invalid(format!("shard0 missing gradient {name}")))?;
        let mut combined = first_gradient.detach().affine(weights[0], 0.0)?;
        for (index, result) in others.iter_mut().enumerate() {
            let gradient = result
                .gradients
                .remove(variable.as_tensor())
                .ok_or_else(|| invalid(format!("shard{} missing gradient {name}", index + 1)))?;
            if gradient.dtype() != DType::F32
                || gradient.dims() != variable.dims()
                || !gradient.device().same_device(variable.device())
            {
                return Err(invalid(format!(
                    "CPU shard gradient binding differs for {name}"
                )));
            }
            combined = combined.add(&gradient.detach().affine(weights[index + 1], 0.0)?)?;
        }
        first.gradients.insert(variable.as_tensor(), combined);
    }
    Ok(())
}

fn validate_shard_layout(batch: usize, time: usize, shards: usize, rows: usize) -> Result<()> {
    if !matches!(shards, 1 | 2 | 4)
        || batch == 0
        || batch > 64
        || batch % shards != 0
        || time == 0
        || batch.checked_mul(time) != Some(rows)
    {
        return Err(invalid(
            "invalid full-window CPU gradient shard configuration",
        ));
    }
    Ok(())
}

/// Split only independent batch rows. Sampling, target count and optimizer
/// update frequency remain the caller's original full-batch configuration.
pub fn batch_gradients(
    model: &JointModel,
    inputs: &[u32],
    targets: &[u32],
    batch: usize,
    time: usize,
    shards: usize,
) -> Result<BatchGradients> {
    validate_shard_layout(batch, time, shards, inputs.len())?;
    if time > model.config.context || targets.len() != inputs.len() {
        return Err(invalid(
            "invalid full-window CPU gradient shard configuration",
        ));
    }
    if shards == 1 {
        let shard = shard_gradients(model, inputs, targets, batch, time)?;
        return Ok(BatchGradients {
            mean_nll: shard.mean_nll,
            gradients: shard.gradients,
        });
    }
    if !matches!(model.device(), Device::Cpu) {
        return Err(invalid("multiple gradient shards require a CPU model"));
    }
    let shard_batch = batch / shards;
    let shard_tokens = shard_batch * time;
    let mut results = parallel_shards(shards, |shard| {
        let start = shard * shard_tokens;
        shard_gradients(
            model,
            &inputs[start..start + shard_tokens],
            &targets[start..start + shard_tokens],
            shard_batch,
            time,
        )
    })?;

    let weight = shard_batch as f64 / batch as f64;
    let mean_nll = results
        .iter()
        .map(|result| f64::from(result.mean_nll) * weight)
        .sum::<f64>() as f32;
    let weights = vec![weight; results.len()];
    combine_shard_gradients(model, &mut results, &weights)?;
    let first = results.remove(0);
    Ok(BatchGradients {
        mean_nll,
        gradients: first.gradients,
    })
}

/// Response-masked analogue of [`batch_gradients`]. Shards are combined by
/// supervised target count, so the result is the gradient of the full batch's
/// masked mean, not the mean of per-shard masked means.
pub fn masked_batch_gradients(
    model: &JointModel,
    inputs: &[u32],
    targets: &[u32],
    masks: &[u8],
    batch: usize,
    time: usize,
    shards: usize,
) -> Result<MaskedBatchGradients> {
    validate_shard_layout(batch, time, shards, inputs.len())?;
    if time > model.config.context
        || targets.len() != inputs.len()
        || masks.len() != inputs.len()
        || masks.iter().any(|&mask| mask > 1)
    {
        return Err(invalid(
            "invalid masked full-window CPU gradient shard configuration",
        ));
    }
    if shards == 1 {
        let shard = masked_shard_gradients(model, inputs, targets, masks, batch, time)?;
        return Ok(MaskedBatchGradients {
            mean_nll: shard.mean_nll,
            supervised: shard.supervised,
            gradients: shard.gradients,
        });
    }
    if !matches!(model.device(), Device::Cpu) {
        return Err(invalid("multiple gradient shards require a CPU model"));
    }
    let shard_batch = batch / shards;
    let shard_tokens = shard_batch * time;
    let mut results = parallel_shards(shards, |shard| {
        let start = shard * shard_tokens;
        masked_shard_gradients(
            model,
            &inputs[start..start + shard_tokens],
            &targets[start..start + shard_tokens],
            &masks[start..start + shard_tokens],
            shard_batch,
            time,
        )
    })?;

    let supervised: f64 = results.iter().map(|result| result.supervised).sum();
    if supervised <= 0.0 {
        return Err(invalid("masked shards have no supervised response targets"));
    }
    let weights: Vec<f64> = results
        .iter()
        .map(|result| result.supervised / supervised)
        .collect();
    let mean_nll = results
        .iter()
        .zip(&weights)
        .map(|(result, weight)| f64::from(result.mean_nll) * weight)
        .sum::<f64>() as f32;
    combine_shard_gradients(model, &mut results, &weights)?;
    let first = results.remove(0);
    Ok(MaskedBatchGradients {
        mean_nll,
        supervised,
        gradients: first.gradients,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joint_model::{JointConfig, Transport};

    #[test]
    fn full_window_shards_match_loss_and_every_named_gradient() -> Result<()> {
        check_full_window_shards(false)
    }

    #[test]
    fn quantized_shards_keep_original_variable_gradients() -> Result<()> {
        check_full_window_shards(true)
    }

    fn check_full_window_shards(quantized: bool) -> Result<()> {
        let mut model = JointModel::new(
            JointConfig {
                width: 128,
                context: 8,
                transport: Transport::Quaternion,
                seed: 53,
                ..JointConfig::default()
            },
            &Device::Cpu,
        )?;
        if quantized {
            model.configure_quantization(0, 1)?;
        }
        let inputs: Vec<u32> = (0..32).map(|index| (index * 7 + 3) % 101).collect();
        let targets: Vec<u32> = (0..32).map(|index| (index * 7 + 10) % 101).collect();
        let complete = batch_gradients(&model, &inputs, &targets, 4, 8, 1)?;
        for shards in [2, 4] {
            let split = batch_gradients(&model, &inputs, &targets, 4, 8, shards)?;
            assert!((complete.mean_nll - split.mean_nll).abs() < 2e-5);
            for (name, variable) in model.variables() {
                let expected = complete
                    .gradients
                    .get(variable.as_tensor())
                    .ok_or_else(|| invalid(format!("full batch missing gradient {name}")))?;
                let actual = split
                    .gradients
                    .get(variable.as_tensor())
                    .ok_or_else(|| invalid(format!("split batch missing gradient {name}")))?;
                let delta = actual.sub(expected)?.abs()?.max_all()?.to_scalar::<f32>()?;
                let scale = expected.abs()?.max_all()?.to_scalar::<f32>()?;
                assert!(
                    delta.is_finite() && delta <= 2e-5 + 2e-4 * scale,
                    "{shards} shards, {name}: delta {delta}, scale {scale}"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn masked_shards_match_supervised_mean_loss_and_every_named_gradient() -> Result<()> {
        let model = JointModel::new(
            JointConfig {
                width: 128,
                context: 8,
                transport: Transport::Quaternion,
                seed: 53,
                ..JointConfig::default()
            },
            &Device::Cpu,
        )?;
        let inputs: Vec<u32> = (0..32).map(|index| (index * 7 + 3) % 101).collect();
        let targets: Vec<u32> = (0..32).map(|index| (index * 7 + 10) % 101).collect();
        // Uneven supervision across rows so the supervised-count weighting is
        // actually exercised rather than reducing to the uniform case.
        let masks: Vec<u8> = (0..32).map(|index| u8::from(index % 3 != 0)).collect();
        let complete = masked_batch_gradients(&model, &inputs, &targets, &masks, 4, 8, 1)?;
        for shards in [2, 4] {
            let split = masked_batch_gradients(&model, &inputs, &targets, &masks, 4, 8, shards)?;
            assert!((complete.mean_nll - split.mean_nll).abs() < 2e-5);
            assert_eq!(complete.supervised, split.supervised);
            for (name, variable) in model.variables() {
                let expected = complete
                    .gradients
                    .get(variable.as_tensor())
                    .ok_or_else(|| invalid(format!("full batch missing gradient {name}")))?;
                let actual = split
                    .gradients
                    .get(variable.as_tensor())
                    .ok_or_else(|| invalid(format!("split batch missing gradient {name}")))?;
                let delta = actual.sub(expected)?.abs()?.max_all()?.to_scalar::<f32>()?;
                let scale = expected.abs()?.max_all()?.to_scalar::<f32>()?;
                assert!(
                    delta.is_finite() && delta <= 5e-5 + 5e-4 * scale,
                    "{shards} shards, {name}: delta {delta}, scale {scale}"
                );
            }
        }
        Ok(())
    }
}
