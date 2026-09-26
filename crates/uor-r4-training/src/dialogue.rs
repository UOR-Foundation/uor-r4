//! R1b dialogue learner: response-masked next-token objective over chat-v0.
//!
//! The learner is the existing shared recurrent-memory graph, constructed at a
//! declared dialogue width through `JointModel::new_dialogue`. Loss is masked to
//! assistant response targets only. This is offline floating training; it makes
//! no serving, integer, geometry-advantage or energy claim.

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::backprop::GradStore;
use candle_core::{Device, Tensor};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;
use uor_r4_core::report_output;

use crate::joint_model::{JointConfig, JointModel, ReadMode};
use crate::joint_optimizer::{AdamConfig, NamedAdamW, StepReport};
use crate::ngram::{KneserNey5Gram, NgramConfig};
use crate::{invalid, sha256_file, Result};

pub const DIALOGUE_SCHEMA: &str = "uor-r4.dialogue-campaign/1";
pub const DIALOGUE_MIN_PARAMETERS: usize = 5_000_000;
pub const DIALOGUE_MAX_PARAMETERS: usize = 15_000_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueCampaign {
    pub schema: String,
    pub train_tokens: PathBuf,
    pub train_mask: PathBuf,
    pub heldout_tokens: PathBuf,
    pub heldout_mask: PathBuf,
    pub model: JointConfig,
    pub optimizer: AdamConfig,
    pub data_seed: u64,
    pub batch: usize,
    pub context: usize,
    pub total_steps: usize,
    pub max_process_seconds: u64,
    pub eval_stride: usize,
    pub eval_max_windows: usize,
    pub eval_batch: usize,
    pub tune_windows: usize,
    #[serde(default)]
    pub checkpoint_steps: Vec<usize>,
    #[serde(default)]
    pub count_report: Option<PathBuf>,
    #[serde(default)]
    pub stop_file: Option<PathBuf>,
    pub trial_scope: String,
}

impl DialogueCampaign {
    pub fn load(path: &Path) -> Result<Self> {
        let cfg: Self = serde_json::from_slice(&fs::read(path)?)?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn parameter_count(&self) -> usize {
        self.model
            .shapes()
            .values()
            .map(|shape| shape.iter().product::<usize>())
            .sum()
    }

    fn validate(&self) -> Result<()> {
        let parameters = self.parameter_count();
        if self.schema != DIALOGUE_SCHEMA
            || self.model.vocab_size != 4096
            || self.model.width % 4 != 0
            || self.model.width < 128
            || self.model.read_width != 64
            || self.context != 256
            || self.model.context != 256
            || !(DIALOGUE_MIN_PARAMETERS..=DIALOGUE_MAX_PARAMETERS).contains(&parameters)
            || !(1..=16).contains(&self.batch)
            || self.total_steps == 0
            || self.total_steps > 100_000
            || self.max_process_seconds == 0
            || self.max_process_seconds > 86_400
            || self.eval_stride == 0
            || self.eval_max_windows == 0
            || !(1..=16).contains(&self.eval_batch)
            || self.tune_windows == 0
            || self.checkpoint_steps.len() > 8
            || self
                .checkpoint_steps
                .iter()
                .any(|&step| step == 0 || step >= self.total_steps)
            || self
                .checkpoint_steps
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || self.trial_scope.trim().is_empty()
        {
            return Err(invalid(format!(
                "unsupported dialogue campaign configuration ({parameters} parameters)"
            )));
        }
        self.optimizer
            .validate()
            .map_err(|error| invalid(error.to_string()))?;
        Ok(())
    }
}

struct Split {
    tokens: MmapCorpusReader,
    mask: Vec<u8>,
}

impl Split {
    fn open(tokens: &Path, mask: &Path) -> Result<Self> {
        let reader =
            MmapCorpusReader::open(tokens).map_err(|error| invalid(format!("corpus: {error}")))?;
        if reader.vocab_size() != 4096 {
            return Err(invalid("chat corpus vocabulary must be 4096"));
        }
        let bytes = fs::read(mask)?;
        if bytes.len() != reader.total_tokens() {
            return Err(invalid("response_mask length differs from token count"));
        }
        if bytes.iter().any(|&value| value > 1) {
            return Err(invalid("response_mask must be binary"));
        }
        Ok(Self {
            tokens: reader,
            mask: bytes,
        })
    }

    fn slice(&self) -> &[u16] {
        self.tokens.as_slice()
    }
}

fn splitmix(mut state: u64) -> u64 {
    state = state.wrapping_add(0x9e3779b97f4a7c15);
    state = (state ^ (state >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    state = (state ^ (state >> 27)).wrapping_mul(0x94d049bb133111eb);
    state ^ (state >> 31)
}

fn sample_batch(
    split: &Split,
    cfg: &DialogueCampaign,
    step: usize,
) -> Result<(Vec<u32>, Vec<u32>, Vec<u8>)> {
    let tokens = split.slice();
    let windows = tokens.len().saturating_sub(cfg.context);
    if windows == 0 {
        return Err(invalid("chat train split is shorter than the context"));
    }
    let mut inputs = Vec::with_capacity(cfg.batch * cfg.context);
    let mut targets = Vec::with_capacity(cfg.batch * cfg.context);
    let mut masks = Vec::with_capacity(cfg.batch * cfg.context);
    for lane in 0..cfg.batch {
        let counter = cfg.data_seed
            ^ (step as u64).wrapping_mul(0xd1342543de82ef95)
            ^ (lane as u64).wrapping_mul(0x9e3779b97f4a7c15);
        let start = (splitmix(counter) % windows as u64) as usize;
        for j in 0..cfg.context {
            inputs.push(u32::from(tokens[start + j]));
            let target_index = start + j + 1;
            targets.push(u32::from(tokens[target_index]));
            masks.push(split.mask[target_index]);
        }
    }
    Ok((inputs, targets, masks))
}

fn window_starts(total: usize, context: usize, stride: usize, max_windows: usize) -> Vec<usize> {
    if max_windows == 0 || total < context + 1 {
        return Vec::new();
    }
    let last_start = total - context - 1;
    let natural = stride.saturating_mul(context).max(1);
    let spread = last_start / max_windows + 1;
    let step = natural.max(spread);
    let mut starts = Vec::new();
    let mut start = 0usize;
    while start <= last_start && starts.len() < max_windows {
        starts.push(start);
        start += step;
    }
    starts
}

fn masked_nats(probabilities: &Tensor, targets: &[u32], masks: &[u8]) -> Result<(Tensor, f64)> {
    let (batch, time, vocab) = probabilities.dims3()?;
    let count = batch * time;
    if targets.len() != count || masks.len() != count {
        return Err(invalid("masked target shape"));
    }
    let device = probabilities.device();
    let target_tensor = Tensor::from_vec(targets.to_vec(), (count, 1), device)?;
    let gathered = probabilities
        .reshape((count, vocab))?
        .gather(&target_tensor, 1)?
        .squeeze(1)?;
    let log_probability = gathered.log()?;
    let weight: Vec<f32> = masks.iter().map(|&value| f32::from(value)).collect();
    let supervised: f64 = weight.iter().map(|&value| f64::from(value)).sum();
    let weight_tensor = Tensor::from_vec(weight, (count,), device)?;
    let nats = log_probability.mul(&weight_tensor)?.sum_all()?.neg()?;
    Ok((nats, supervised))
}

fn audit_gradients(model: &JointModel, gradients: &GradStore) -> Result<Value> {
    let mut paths = Vec::new();
    let mut all_paths = true;
    for (name, variable) in model.variables() {
        match gradients.get(variable.as_tensor()) {
            Some(gradient) => {
                let values = gradient.flatten_all()?.to_vec1::<f32>()?;
                let nonzero = values.iter().filter(|value| **value != 0.0).count();
                let finite = values.iter().all(|value| value.is_finite());
                let l2 = values
                    .iter()
                    .map(|&value| f64::from(value).powi(2))
                    .sum::<f64>()
                    .sqrt();
                let max_abs = values
                    .iter()
                    .map(|&value| f64::from(value).abs())
                    .fold(0.0f64, f64::max);
                all_paths &= finite && nonzero > 0;
                paths.push(
                    json!({"name":name,"elements":values.len(),"nonzero":nonzero,
                    "finite":finite,"l2":l2,"max_abs":max_abs}),
                );
            }
            None => {
                all_paths = false;
                paths.push(json!({"name":name,"present":false,"nonzero":0}));
            }
        }
    }
    Ok(json!({"all_paths_present_finite_nonzero":all_paths,"paths":paths}))
}

fn save_parameters(model: &JointModel, path: &Path) -> Result<Value> {
    let mut writer = BufWriter::new(File::create(path)?);
    let mut index = Vec::new();
    let mut offset = 0u64;
    for (name, variable) in model.variables() {
        let values = variable
            .as_detached_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        for value in &values {
            writer.write_all(&value.to_le_bytes())?;
        }
        index.push(
            json!({"name":name,"shape":variable.dims(),"offset":offset,"elements":values.len()}),
        );
        offset += values.len() as u64;
    }
    writer.flush()?;
    Ok(json!({"file":"parameters.f32","coordinate_bytes":offset*4,"index":index}))
}

fn response_nll(
    model: &JointModel,
    split: &Split,
    starts: &[usize],
    cfg: &DialogueCampaign,
) -> Result<(f64, u64)> {
    let tokens = split.slice();
    let mut nats = 0.0f64;
    let mut supervised = 0u64;
    for chunk in starts.chunks(cfg.eval_batch) {
        let mut inputs = Vec::with_capacity(chunk.len() * cfg.context);
        let mut targets = Vec::with_capacity(chunk.len() * cfg.context);
        let mut masks = Vec::with_capacity(chunk.len() * cfg.context);
        for &start in chunk {
            for j in 0..cfg.context {
                inputs.push(u32::from(tokens[start + j]));
                targets.push(u32::from(tokens[start + j + 1]));
                masks.push(split.mask[start + j + 1]);
            }
        }
        let batch = chunk.len();
        let output = model.forward(&inputs, batch, cfg.context, ReadMode::Enabled, false)?;
        let (sum, count) = masked_nats(&output.probabilities, &targets, &masks)?;
        nats += f64::from(sum.to_scalar::<f32>()?);
        supervised += count as u64;
    }
    if supervised == 0 {
        return Err(invalid(
            "held-out population has no supervised response targets",
        ));
    }
    Ok((nats, supervised))
}

fn count_nll(
    model: &KneserNey5Gram,
    split: &Split,
    starts: &[usize],
    context: usize,
    discount: f64,
) -> Result<(f64, u64)> {
    let tokens = split.slice();
    let mut nats = 0.0f64;
    let mut supervised = 0u64;
    for &start in starts {
        for j in 0..context {
            let target_index = start + j + 1;
            if split.mask[target_index] == 0 {
                continue;
            }
            let low = start + j.saturating_sub(3);
            let history = &tokens[low..=start + j];
            let probability = model
                .probability_with_discount(history, tokens[target_index], discount)
                .map_err(|error| invalid(error.to_string()))?;
            if !probability.is_finite() || probability <= 0.0 {
                return Err(invalid("count probability is not in (0,1]"));
            }
            nats -= probability.ln();
            supervised += 1;
        }
    }
    Ok((nats, supervised))
}

fn output_root(args: &[String]) -> Result<PathBuf> {
    let path = args.get(2).ok_or_else(|| invalid("missing report root"))?;
    Ok(PathBuf::from(path))
}

fn load_campaign(path: &str) -> Result<DialogueCampaign> {
    DialogueCampaign::load(Path::new(path))
}

pub fn run_count(args: &[String]) -> Result<()> {
    let cfg = load_campaign(&args[1])?;
    let out = output_root(args)?;
    report_output::claim(&out)?;
    let result = run_count_inner(&cfg, &out);
    if let Err(error) = &result {
        crate::baseline_protocol::save_json(
            &out.join("failed-attempt.json"),
            &json!({"status":"FAILED_ATTEMPT","error":error.to_string()}),
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result
}

fn run_count_inner(cfg: &DialogueCampaign, out: &Path) -> Result<()> {
    let started = Instant::now();
    let train = Split::open(&cfg.train_tokens, &cfg.train_mask)?;
    let heldout = Split::open(&cfg.heldout_tokens, &cfg.heldout_mask)?;
    if sha256_file(&cfg.train_tokens)? == sha256_file(&cfg.heldout_tokens)? {
        return Err(invalid("train and held-out token stores are identical"));
    }
    let ngram_config = NgramConfig {
        vocab_size: 4096,
        discount: 0.75,
        unigram_alpha: 1e-4,
        max_train_tokens: train.slice().len(),
        row_caps: [4_000_000, 4_000_000, 8_000_000, 8_000_000],
    };
    ngram_config
        .validate()
        .map_err(|error| invalid(error.to_string()))?;
    let fit_started = Instant::now();
    let (model, fit) = KneserNey5Gram::fit(&[train.slice()], ngram_config.clone())
        .map_err(|error| invalid(error.to_string()))?;
    let fit_seconds = fit_started.elapsed().as_secs_f64();
    let model_bytes = model.model_bytes();
    if model_bytes > 700 * 1024 * 1024 {
        return Err(invalid("count model exceeds declared retained storage"));
    }

    let tune_starts = window_starts(
        train.slice().len(),
        cfg.context,
        cfg.eval_stride,
        cfg.tune_windows,
    );
    let discounts = [0.5, 0.75, 0.9];
    let mut tune = Vec::new();
    for discount in discounts {
        let (nats, count) = count_nll(&model, &train, &tune_starts, cfg.context, discount)?;
        tune.push((discount, nats / count as f64));
    }
    let selected = tune
        .iter()
        .copied()
        .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.total_cmp(&b.0)))
        .ok_or_else(|| invalid("empty discount grid"))?;

    let starts = window_starts(
        heldout.slice().len(),
        cfg.context,
        cfg.eval_stride,
        cfg.eval_max_windows,
    );
    let (nats, supervised) = count_nll(&model, &heldout, &starts, cfg.context, selected.0)?;
    let matched_nll = nats / supervised as f64;
    let full_starts = window_starts(heldout.slice().len(), cfg.context, 1, usize::MAX);
    let (full_nats, full_supervised) =
        count_nll(&model, &heldout, &full_starts, cfg.context, selected.0)?;
    let model_path = out.join("model.ng5");
    model
        .save(&model_path)
        .map_err(|error| invalid(error.to_string()))?;
    let model_sha256 = sha256_file(&model_path)?;

    let report = json!({
        "schema":"uor-r4.dialogue-count-report/1",
        "mode":"dialogue-count",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND"),
        "executable_sha256":sha256_file(&std::env::current_exe()?)?,
        "scope":cfg.trial_scope,
        "configuration":cfg,
        "train_tokens":{"path":cfg.train_tokens,"sha256":sha256_file(&cfg.train_tokens)?,"tokens":train.slice().len(),"response_tokens":train.mask.iter().filter(|&&m| m==1).count()},
        "heldout_tokens":{"path":cfg.heldout_tokens,"sha256":sha256_file(&cfg.heldout_tokens)?,"tokens":heldout.slice().len(),"response_tokens":heldout.mask.iter().filter(|&&m| m==1).count()},
        "ngram_config":ngram_config,
        "fit":fit,
        "model_bytes":model_bytes,
        "model_sha256":model_sha256,
        "fit_seconds":fit_seconds,
        "discount_tune":{"grid":discounts,"train_window_nll":tune.iter().map(|(d,n)| json!({"discount":d,"nll":n})).collect::<Vec<_>>(),"selected_discount":selected.0,"selected_train_nll":selected.1,"tune_windows":tune_starts.len(),"tune_scope":"count selection uses train-split windows only; no held-out selection"},
        "matched_evaluation":{"population":"heldout windows at eval_stride with eval_max_windows","windows":starts.len(),"supervised_targets":supervised,"response_masked_nll_nats":matched_nll},
        "full_heldout":{"population":"all 256-token windows, stride 1","windows":full_starts.len(),"supervised_targets":full_supervised,"response_masked_nll_nats":full_nats/full_supervised as f64},
        "status":"COUNT_COMPLETE",
        "elapsed_seconds":started.elapsed().as_secs_f64()
    });
    crate::baseline_protocol::save_json(&out.join("dialogue-count.json"), &report)?;
    eprintln!("count complete: matched NLL {matched_nll:.6} over {supervised} targets; fit {fit_seconds:.2}s");
    Ok(())
}

pub fn run_fit(args: &[String]) -> Result<()> {
    let cfg = load_campaign(&args[1])?;
    let out = output_root(args)?;
    report_output::claim(&out)?;
    let result = run_fit_inner(&cfg, &out);
    if let Err(error) = &result {
        crate::baseline_protocol::save_json(
            &out.join("failed-attempt.json"),
            &json!({"status":"FAILED_ATTEMPT","error":error.to_string()}),
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result
}

fn run_fit_inner(cfg: &DialogueCampaign, out: &Path) -> Result<()> {
    let started = Instant::now();
    let train = Split::open(&cfg.train_tokens, &cfg.train_mask)?;
    let heldout = Split::open(&cfg.heldout_tokens, &cfg.heldout_mask)?;
    if sha256_file(&cfg.train_tokens)? == sha256_file(&cfg.heldout_tokens)? {
        return Err(invalid("train and held-out token stores are identical"));
    }
    let starts = window_starts(
        heldout.slice().len(),
        cfg.context,
        cfg.eval_stride,
        cfg.eval_max_windows,
    );
    let device = Device::Cpu;
    let model = JointModel::new_dialogue(cfg.model.clone(), &device)?;
    if model.parameter_count() != cfg.parameter_count() {
        return Err(invalid(
            "constructed parameter count differs from the declared configuration",
        ));
    }
    let mut optimizer = NamedAdamW::new(model.variables(), cfg.optimizer.clone())
        .map_err(|error| invalid(error.to_string()))?;

    let (heldout_nats0, heldout_count0) = response_nll(&model, &heldout, &starts, cfg)?;
    let initial_heldout = heldout_nats0 / heldout_count0 as f64;

    let mut curve = BufWriter::new(File::create(out.join("learning-curve.jsonl"))?);
    let mut gradient_audit = Value::Null;
    let mut step_times = Vec::new();
    let mut losses = Vec::new();
    let mut complete = 0usize;
    let mut checkpoint_manifest = Vec::new();
    for step in 0..cfg.total_steps {
        if started.elapsed().as_secs() >= cfg.max_process_seconds
            || cfg.stop_file.as_ref().is_some_and(|path| path.exists())
        {
            break;
        }
        let step_started = Instant::now();
        let (inputs, targets, masks) = sample_batch(&train, cfg, step)?;
        let output = model.forward(&inputs, cfg.batch, cfg.context, ReadMode::Enabled, true)?;
        let (nats, supervised) = masked_nats(&output.probabilities, &targets, &masks)?;
        if supervised <= 0.0 {
            return Err(invalid("sampled batch has no supervised response targets"));
        }
        let loss = nats.affine(1.0 / supervised, 0.0)?;
        let value = loss.to_scalar::<f32>()?;
        if !value.is_finite() {
            return Err(invalid("nonfinite response-masked loss"));
        }
        let gradients = loss.backward()?;
        if step == 0 {
            gradient_audit = audit_gradients(&model, &gradients)?;
            if !gradient_audit["all_paths_present_finite_nonzero"]
                .as_bool()
                .unwrap_or(false)
            {
                return Err(invalid("a learned path has no finite nonzero gradient"));
            }
        }
        let update: StepReport = optimizer
            .step(model.variables(), &gradients)
            .map_err(|error| invalid(error.to_string()))?;
        drop(gradients);
        let elapsed = step_started.elapsed().as_secs_f64();
        step_times.push(elapsed);
        losses.push(value);
        complete = step + 1;
        let row = json!({"step":complete,"supervised_targets":supervised as u64,
            "sampled_targets":(cfg.batch*cfg.context) as u64,"masked_batch_nll":value,
            "step_seconds":elapsed,"optimizer":update});
        serde_json::to_writer(&mut curve, &row)?;
        writeln!(curve)?;
        if cfg.checkpoint_steps.contains(&complete) {
            let manifest =
                save_parameters(&model, &out.join(format!("parameters-step-{complete}.f32")))?;
            checkpoint_manifest.push(json!({"step":complete,"artifact":manifest}));
        }
        if complete % 50 == 0 {
            eprintln!("dialogue step {complete}: masked NLL {value:.6} ({elapsed:.2}s)");
        }
    }
    curve.flush()?;
    let parameter_manifest = save_parameters(&model, &out.join("parameters.f32"))?;
    let (heldout_nats1, heldout_count1) = response_nll(&model, &heldout, &starts, cfg)?;
    let final_heldout = heldout_nats1 / heldout_count1 as f64;

    let window = |count: usize| -> Option<(f64, f64, f64)> {
        if losses.len() < count {
            return None;
        }
        let slice = &losses[..count];
        let mean = slice.iter().map(|&v| f64::from(v)).sum::<f64>() / count as f64;
        Some((f64::from(slice[0]), mean, f64::from(*slice.last().unwrap())))
    };
    let first10 = window(10);
    let step_seconds = if step_times.is_empty() {
        0.0
    } else {
        step_times.iter().sum::<f64>() / step_times.len() as f64
    };

    let count_report = match &cfg.count_report {
        Some(path) if path.exists() => {
            let document: Value = serde_json::from_slice(&fs::read(path)?)?;
            let count_nll_value = document["matched_evaluation"]["response_masked_nll_nats"]
                .as_f64()
                .ok_or_else(|| invalid("count report has no matched NLL"))?;
            Some(json!({
                "path":path,"sha256":sha256_file(path)?,
                "count_response_masked_nll_nats":count_nll_value,
                "neural_response_masked_nll_nats":final_heldout,
                "margin_nats":count_nll_value-final_heldout,
                "population_matches":document["matched_evaluation"]["windows"].as_u64()==Some(starts.len() as u64)
            }))
        }
        _ => None,
    };

    let report = json!({
        "schema":"uor-r4.dialogue-fit-report/1",
        "mode":"dialogue-fit",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND"),
        "executable_sha256":sha256_file(&std::env::current_exe()?)?,
        "scope":cfg.trial_scope,
        "configuration":cfg,
        "parameter_count":model.parameter_count(),
        "train_tokens":{"path":cfg.train_tokens,"sha256":sha256_file(&cfg.train_tokens)?,"tokens":train.slice().len(),"response_tokens":train.mask.iter().filter(|&&m| m==1).count()},
        "heldout_tokens":{"path":cfg.heldout_tokens,"sha256":sha256_file(&cfg.heldout_tokens)?,"tokens":heldout.slice().len(),"response_tokens":heldout.mask.iter().filter(|&&m| m==1).count()},
        "evaluation_population":{"windows":starts.len(),"context":cfg.context,"stride":cfg.eval_stride,"max_windows":cfg.eval_max_windows},
        "initial_heldout_response_masked_nll_nats":initial_heldout,
        "final_heldout_response_masked_nll_nats":final_heldout,
        "heldout_nll_change_nats":final_heldout-initial_heldout,
        "steps_completed":complete,
        "sampled_targets":(complete*cfg.batch*cfg.context) as u64,
        "supervised_targets":(complete*cfg.batch*cfg.context) as u64,
        "mean_step_seconds":step_seconds,
        "loss_curve":{"first":losses.first().copied(),
            "last":losses.last().copied(),"count":losses.len(),
            "first_10":{"first":first10.map(|w|w.0),"mean":first10.map(|w|w.1),"last":first10.map(|w|w.2)},
            "mean":if losses.is_empty(){None}else{Some(losses.iter().map(|&v|f64::from(v)).sum::<f64>()/losses.len() as f64)}},
        "gradient_audit":gradient_audit,
        "checkpoints":checkpoint_manifest,
        "final_parameter_artifact":{"file":"parameters.f32","sha256":sha256_file(&out.join("parameters.f32"))?,"manifest":parameter_manifest},
        "count_reference":count_report,
        "status":"FIT_COMPLETE",
        "elapsed_seconds":started.elapsed().as_secs_f64()
    });
    crate::baseline_protocol::save_json(&out.join("dialogue-fit.json"), &report)?;
    eprintln!(
        "fit complete: {complete} steps, train NLL {:?} -> {:?}, heldout NLL {initial_heldout:.6} -> {final_heldout:.6}",
        losses.first(),
        losses.last()
    );
    Ok(())
}

pub fn run_panel(args: &[String]) -> Result<()> {
    let panel_path = Path::new(&args[1]);
    let out_path = Path::new(&args[2]);
    let panel: Value = serde_json::from_slice(&fs::read(panel_path)?)?;
    if panel["schema"] != "uor-r4.chat-panel/1" {
        return Err(invalid("unsupported chat panel schema"));
    }
    let rows = panel["development"]
        .as_array()
        .ok_or_else(|| invalid("panel has no development rows"))?;
    if rows.len() != 38 {
        return Err(invalid("panel development population changed"));
    }
    let mut requests = Vec::with_capacity(rows.len());
    for row in rows {
        let id = row["id"].as_str().ok_or_else(|| invalid("panel row id"))?;
        let group = row["group"]
            .as_str()
            .ok_or_else(|| invalid("panel row group"))?;
        let turns = row["user_turns"]
            .as_array()
            .ok_or_else(|| invalid("panel row user_turns"))?;
        let mut pieces = Vec::with_capacity(turns.len());
        for turn in turns {
            let text = turn.as_str().ok_or_else(|| invalid("panel user turn"))?;
            pieces.push(format!("User: {text}"));
        }
        let prompt = format!("{}\nAssistant: ", pieces.join("\n"));
        requests.push(json!({
            "id":id,"group":group,
            "prompt":prompt,
            "max_new_tokens":64,
            "selection":{"kind":"greedy"},
            "read_mode":"enabled"
        }));
    }
    let document = json!({
        "schema":"uor-r4.dialogue-panel-requests/1",
        "panel_path":panel_path,
        "panel_sha256":sha256_file(panel_path)?,
        "panel_panel_sha256":panel["panel_sha256"],
        "template":"User: {turn}\\nAssistant: {response}<|eos|> with turns separated by a single newline and <|bos|> inserted by the session",
        "note":"Development rows only. The panel stores no intermediate assistant replies for memory rows, so consecutive User: blocks are emitted; run these only after R3/R4 design selection.",
        "requests":requests
    });
    let mut file = File::create(out_path)?;
    serde_json::to_writer_pretty(&mut file, &document)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    eprintln!(
        "panel requests written: {} rows -> {}",
        requests.len(),
        out_path.display()
    );
    Ok(())
}

pub fn run_cli(args: &[String]) -> Result<()> {
    let mode = args.first().map(String::as_str);
    let result = match mode {
        Some("dialogue-count") if args.len() == 3 => run_count(args),
        Some("dialogue-fit") if args.len() == 3 => run_fit(args),
        Some("dialogue-panel") if args.len() == 3 => run_panel(args),
        _ => Err(invalid(
            "usage:\n  uor-r4-training dialogue-count CAMPAIGN_JSON NEW_REPORT_ROOT\n  uor-r4-training dialogue-fit CAMPAIGN_JSON NEW_REPORT_ROOT\n  uor-r4-training dialogue-panel PANEL_JSON OUT_JSON",
        )),
    };
    result?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joint_model::Transport;

    fn dialogue_config(width: usize, context: usize) -> JointConfig {
        JointConfig {
            vocab_size: 4096,
            width,
            read_width: 64,
            context,
            transport: Transport::Quaternion,
            seed: 20260926,
        }
    }

    #[test]
    fn scaled_dialogue_width_is_in_the_declared_parameter_band() {
        let cfg = DialogueCampaign {
            schema: DIALOGUE_SCHEMA.into(),
            train_tokens: PathBuf::from("t"),
            train_mask: PathBuf::from("m"),
            heldout_tokens: PathBuf::from("t"),
            heldout_mask: PathBuf::from("m"),
            model: dialogue_config(576, 256),
            optimizer: AdamConfig::default(),
            data_seed: 1,
            batch: 8,
            context: 256,
            total_steps: 1,
            max_process_seconds: 1,
            eval_stride: 4,
            eval_max_windows: 4,
            eval_batch: 8,
            tune_windows: 4,
            checkpoint_steps: Vec::new(),
            count_report: None,
            stop_file: None,
            trial_scope: "test".into(),
        };
        assert_eq!(cfg.parameter_count(), 5_429_826);
        assert!(
            (DIALOGUE_MIN_PARAMETERS..=DIALOGUE_MAX_PARAMETERS).contains(&cfg.parameter_count())
        );
        cfg.validate()
            .expect("width 576 must be a valid dialogue configuration");
        let narrow = DialogueCampaign {
            model: dialogue_config(128, 256),
            ..cfg
        };
        assert!(
            narrow.validate().is_err(),
            "128-width dialogue config is below the band"
        );
    }

    #[test]
    fn new_dialogue_constructs_above_the_serving_width_contract() {
        let window = dialogue_config(256, 8);
        JointModel::new(window, &Device::Cpu).expect("serving width is accepted");

        let wide = dialogue_config(512, 8);
        assert!(
            JointModel::new(wide.clone(), &Device::Cpu).is_err(),
            "serving constructor must retain the 128/256 width contract"
        );
        let model =
            JointModel::new_dialogue(wide, &Device::Cpu).expect("dialogue width is accepted");
        assert_eq!(model.parameter_count(), 4_531_850);
    }

    #[test]
    fn window_starts_respect_the_target_boundary_and_cap() {
        let starts = window_starts(1000, 256, 1, 3);
        assert_eq!(starts, vec![0, 256, 512]);
        assert!(starts.iter().all(|&start| start + 256 + 1 <= 1000));
        assert_eq!(window_starts(256, 256, 1, 4), Vec::<usize>::new());
        assert_eq!(window_starts(769, 256, 2, 8), vec![0, 512]);
        assert_eq!(window_starts(768, 256, 2, 8), vec![0]);
    }

    #[test]
    fn masked_objective_supervises_only_masked_targets() {
        let probabilities = Tensor::from_vec(
            vec![
                0.25f32, 0.25, 0.25, 0.25, // position 0, target id 1 -> p 0.25 (masked)
                0.25, 0.25, 0.25, 0.25, // position 1, target id 0 -> p 0.25 (unmasked)
            ],
            (1, 2, 4),
            &Device::Cpu,
        )
        .unwrap();
        let (nats, supervised) = masked_nats(&probabilities, &[1, 0], &[1, 0]).unwrap();
        assert_eq!(supervised, 1.0);
        assert!((nats.to_scalar::<f32>().unwrap() as f64 - (-0.25f64.ln())).abs() < 1e-6);
        let (zero_nats, zero_supervised) = masked_nats(&probabilities, &[1, 0], &[0, 0]).unwrap();
        assert_eq!(zero_supervised, 0.0);
        assert_eq!(zero_nats.to_scalar::<f32>().unwrap(), 0.0);
    }

    #[test]
    fn masked_loss_reaches_every_learned_path() {
        let model = JointModel::new_dialogue(dialogue_config(576, 16), &Device::Cpu).unwrap();
        let inputs: Vec<u32> = (0..16).map(|index| (index * 7) % 4096).collect();
        let output = model
            .forward(&inputs, 1, 16, ReadMode::Enabled, true)
            .unwrap();
        let targets: Vec<u32> = (1..=16).map(|index| (index * 11) % 4096).collect();
        let masks: Vec<u8> = (0..16).map(|index| u8::from(index % 2 == 1)).collect();
        let (nats, supervised) = masked_nats(&output.probabilities, &targets, &masks).unwrap();
        assert!(supervised > 0.0);
        let loss = nats.affine(1.0 / supervised, 0.0).unwrap();
        let gradients = loss.backward().unwrap();
        let audit = audit_gradients(&model, &gradients).unwrap();
        assert_eq!(audit["all_paths_present_finite_nonzero"], true);
    }
}
