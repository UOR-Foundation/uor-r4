//! DRAFT / NOT_RUN: bounded, isolated-layer teacher-forced transfer.
//!
//! Future registration under `track_b` needs sibling `transfer` and `fit`
//! modules pointing at these drafts. This file is not registered or executed.
//! There is no dataset, arm, optimizer, seed, schedule or resource default.
//! Dense and harmonic arms are supported; flock/hybrid composition is absent.
//! Execute only after shared-model dense parity passes for the bound source and
//! backend; this draft does not replace or waive that prerequisite.
//!
//! Every update captures the original teacher's actual post-WO/pre-residual
//! target and RMS+gain input, reconstructs that layer's frozen full-width QKV,
//! then applies QKV LoRA through the existing AttentionKernel contract. Previous
//! layers are the dense teacher. This does NOT train a composed student's
//! changed prefix. A public hooked early-exit model capture is still needed for
//! that later stage. The local QKV/RoPE preparation below mirrors model.rs;
//! replacing it with a shared public preparation API is an integration task.
//!
//! Evaluation of both initial and final parameters happens after all prescribed
//! updates. There is no intermediate development selection, early stopping by
//! quality, or best-checkpoint search. The explicit evaluation-role declaration
//! is provenance supplied by the coordinator, not a proof of prior non-exposure.
//! Proposed diagnostics not yet implemented: zero/constructor-mean predictor
//! baselines and fixed pre-normalization norm-bin error stratification. These
//! omissions do not add owner acceptance gates. The reduction decision below
//! covers only the isolated-layer MSE gate; all-layer hybrid NLL/KL and the
//! matched dense comparison remain separate work.
//!
//! The caller must claim its report attempt before loading the teacher, own the
//! machine compute slot, set the declared thread limits, provide a real resource
//! observer, and enforce a hard watchdog. Checks here are cooperative between
//! tensor operations and cannot interrupt a running backend kernel. No fake
//! flock/slot/OS observer is provided. Errors retain the session for checkpoint.
//! A checkpoint is an exclusive child of that claimed attempt; its complete
//! manifest is written last. This scaffold does not seal the enclosing report.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use candle_core::{DType, Tensor};
use safetensors::tensor::TensorView;
use safetensors::{Dtype as SafeDtype, SafeTensors};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::joint_optimizer::{AdamConfig, NamedAdamW, StepReport};
use crate::kappa_llama::{rope, LlamaShape};
use crate::track_b::model::{AttentionKernel, AttentionPositions, AttentionQkv, TrackBModel};
use crate::{invalid, Result};

use super::transfer::{
    LayerTransfer, TransferArm, TransferDiagnostics, HARMONIC_DELTA, INITIALIZATION_VERSION,
    LORA_ALPHA, LORA_RANK, NORM_EPSILON,
};

pub const MAX_UPDATES: usize = 500;
const FORMAT: &str = "uor-r4.track-b-isolated-layer-fit/1";
const CONFIG_FILE: &str = "configuration.json";
const PARAMETERS_FILE: &str = "parameters.safetensors";
const MANIFEST_FILE: &str = "complete.json";
// Optimizer metadata has an existing 8 MiB ceiling. Reserve that, a bounded
// report, and generous headers before fit; this is storage, not a RAM estimate.
const CHECKPOINT_OVERHEAD_RESERVE: usize = 18 * 1024 * 1024;
const MAX_REPORT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenRow {
    /// Hash of the source document, not a label that can hide overlapping text.
    pub document_sha256: String,
    pub token_offset: u64,
    pub ids: Vec<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedWindow {
    pub id: String,
    /// Rectangular, unpadded rows. Every position contributes to the objective.
    pub rows: Vec<TokenRow>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationRole {
    OpenDevelopment,
    FrozenConfirmation,
    IndependentFinalHeldOut,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedWindows {
    pub corpus_sha256: String,
    pub tokenizer_sha256: String,
    pub constructor: Vec<FixedWindow>,
    pub evaluation: Vec<FixedWindow>,
    pub evaluation_role: EvaluationRole,
    pub evaluation_previously_exposed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceIdentity {
    pub weights_sha256: String,
    pub config_sha256: String,
    pub tokenizer_sha256: String,
    /// Coordinator-provided revision; compile-time source hashes are also saved.
    pub source_revision: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FitLimits {
    pub max_batch: usize,
    pub max_time: usize,
    pub max_windows: usize,
    pub max_total_data_tokens: usize,
    pub max_attention_matrix_elements: usize,
    pub max_parameter_bytes: usize,
    pub max_checkpoint_bytes: usize,
    pub max_wall_seconds: u64,
    pub max_rss_bytes: u64,
    pub minimum_available_bytes: u64,
    /// Set by the launcher; the observer must report the enforced setting.
    pub compute_threads: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedDesign {
    pub pair_id: String,
    pub seed: u64,
    pub optimizer: AdamConfig,
    /// Exact constructor window index for every update, length 1..=500.
    pub update_window_indices: Vec<usize>,
    pub limits: FitLimits,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FitConfig {
    pub layer: usize,
    pub arm: TransferArm,
    pub paired: PairedDesign,
    pub source: SourceIdentity,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceObservation {
    pub rss_bytes: u64,
    pub available_bytes: u64,
    pub enforced_compute_threads: usize,
}

/// Implement using actual coordinator/OS measurements. There is no default.
pub trait ResourceObserver {
    fn observe(&mut self) -> Result<ResourceObservation>;
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WorkCounters {
    pub capture_calls: u64,
    pub captured_positions: u64,
    pub student_forward_positions: u64,
    pub constructor_positions: u64,
    pub evaluation_positions: u64,
    pub backward_steps: u64,
    pub completed_updates: usize,
    pub observed_peak_rss_bytes: u64,
    pub observed_minimum_available_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub positions: u64,
    pub elements: u64,
    pub squared_error: f64,
    pub target_energy: f64,
    pub raw_mse: f64,
    /// Aggregate SSE / aggregate teacher energy; not the mean of window ratios.
    pub normalized_mse: Option<f64>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReductionGate {
    Pass,
    Fail,
    NotApplicableZeroBaseline,
    NotApplicableDenseControl,
    UndefinedZeroTargetEnergy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvaluationReport {
    pub role: EvaluationRole,
    pub previously_exposed: bool,
    pub initial: ErrorMetrics,
    pub final_parameters: ErrorMetrics,
    pub reduction_fraction: Option<f64>,
    pub gate: ReductionGate,
    pub initial_diagnostics: TransferDiagnostics,
    pub final_diagnostics: TransferDiagnostics,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FitReport {
    pub implementation: String,
    pub scope: String,
    pub configuration_sha256: String,
    pub windows_sha256: String,
    pub paired_design_sha256: String,
    pub source_hashes: BTreeMap<String, String>,
    pub shape: LlamaShape,
    pub layer: usize,
    pub arm: TransferArm,
    pub seed: u64,
    pub parameter_bytes: usize,
    pub elapsed_seconds: f64,
    pub work: WorkCounters,
    pub updates: Vec<StepReport>,
    pub constructor_diagnostics: TransferDiagnostics,
    /// None until every prescribed update AND both evaluation passes complete.
    pub evaluation: Option<EvaluationReport>,
    pub last_started_phase: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Configuration {
    format: String,
    fit: FitConfig,
    windows: FixedWindows,
    shape: LlamaShape,
    lora_rank: usize,
    lora_alpha: f64,
    harmonic_delta: f64,
    normalization_epsilon: f64,
    initialization_version: String,
    source_hashes: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileBinding {
    bytes: u64,
    sha256: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckpointManifest {
    format: String,
    files: BTreeMap<String, FileBinding>,
    report: FitReport,
    /// This is parameter/moment serialization, not an independent reload result.
    independent_reload_status: String,
}

/// Owns trainable state across cooperative stops/errors. On any error, retain
/// this session and save a partial checkpoint; never report an absent gate PASS.
pub struct FitSession<'a> {
    teacher: &'a TrackBModel,
    configuration: Configuration,
    transfer: LayerTransfer,
    optimizer: NamedAdamW,
    started: Instant,
    report: FitReport,
    run_started: bool,
}

impl<'a> FitSession<'a> {
    pub fn new(
        teacher: &'a TrackBModel,
        config: FitConfig,
        windows: FixedWindows,
        observer: &mut dyn ResourceObserver,
    ) -> Result<Self> {
        validate(&config, &windows, teacher)?;
        let started = Instant::now();
        let source_hashes = implementation_hashes();
        let configuration = Configuration {
            format: FORMAT.into(),
            shape: teacher.shape().clone(),
            fit: config,
            windows,
            lora_rank: LORA_RANK,
            lora_alpha: LORA_ALPHA,
            harmonic_delta: HARMONIC_DELTA,
            normalization_epsilon: NORM_EPSILON,
            initialization_version: INITIALIZATION_VERSION.into(),
            source_hashes: source_hashes.clone(),
        };
        // Preflight before allocation of trainable parameters or optimizer state.
        let mut work = WorkCounters::default();
        check_resources(
            &configuration.fit.paired.limits,
            started,
            observer,
            &mut work,
        )?;
        let parameter_bytes = projected_parameter_bytes(&configuration.fit, teacher.shape())?;
        if parameter_bytes > configuration.fit.paired.limits.max_parameter_bytes {
            return Err(invalid("transfer parameters exceed configured byte bound"));
        }
        let configuration_bytes = serde_json::to_vec(&configuration)?.len();
        let checkpoint_bound = parameter_bytes
            .checked_mul(3)
            .and_then(|n| n.checked_add(configuration_bytes))
            .and_then(|n| n.checked_add(CHECKPOINT_OVERHEAD_RESERVE))
            .ok_or_else(|| invalid("checkpoint reservation overflow"))?;
        if checkpoint_bound > configuration.fit.paired.limits.max_checkpoint_bytes {
            return Err(invalid(
                "checkpoint allowance cannot preserve parameters, moments and report",
            ));
        }
        let transfer = LayerTransfer::new(
            configuration.fit.layer,
            teacher.shape(),
            teacher.attention_weights(configuration.fit.layer)?,
            configuration.fit.arm,
            configuration.fit.paired.seed,
            teacher.device(),
        )?;
        let actual_bytes = transfer
            .parameters()
            .values()
            .try_fold(0usize, |sum, var| {
                sum.checked_add(var.elem_count().checked_mul(4)?)
            })
            .ok_or_else(|| invalid("parameter byte overflow"))?;
        if actual_bytes != parameter_bytes {
            return Err(invalid(
                "transfer parameter inventory differs from preflight",
            ));
        }
        let optimizer = NamedAdamW::new(
            transfer.parameters(),
            configuration.fit.paired.optimizer.clone(),
        )?;
        let report = FitReport {
            implementation: match configuration.fit.arm {
                TransferArm::DenseControl => "dense_softmax_qkv_lora_train",
                TransferArm::Harmonic { .. } => "quadratic_train_positive_polynomial_qkv_lora",
            }
            .into(),
            scope: "isolated_layer_teacher_forced_post_wo; composition_not_implemented".into(),
            configuration_sha256: json_sha(&configuration)?,
            windows_sha256: json_sha(&configuration.windows)?,
            paired_design_sha256: paired_key(&configuration.fit, &configuration.windows)?,
            source_hashes,
            shape: teacher.shape().clone(),
            layer: configuration.fit.layer,
            arm: configuration.fit.arm,
            seed: configuration.fit.paired.seed,
            parameter_bytes,
            elapsed_seconds: 0.0,
            work,
            updates: Vec::new(),
            constructor_diagnostics: TransferDiagnostics::default(),
            evaluation: None,
            last_started_phase: "constructed".into(),
        };
        Ok(Self {
            teacher,
            configuration,
            transfer,
            optimizer,
            started,
            report,
            run_started: false,
        })
    }

    pub fn report(&self) -> FitReport {
        let mut report = self.report.clone();
        report.elapsed_seconds = self.started.elapsed().as_secs_f64();
        report
    }

    pub fn transfer(&self) -> &LayerTransfer {
        &self.transfer
    }

    fn guard(&mut self, phase: &str, observer: &mut dyn ResourceObserver) -> Result<()> {
        self.report.last_started_phase = phase.into();
        check_resources(
            &self.configuration.fit.paired.limits,
            self.started,
            observer,
            &mut self.report.work,
        )
    }

    /// Exactly the frozen schedule, once. Resume is deliberately not provided;
    /// future resume must also bind cumulative budget and external authorization.
    pub fn run(&mut self, observer: &mut dyn ResourceObserver) -> Result<FitReport> {
        if self.run_started {
            return Err(invalid("fit session may run only once"));
        }
        self.run_started = true;
        let schedule = self.configuration.fit.paired.update_window_indices.clone();
        for index in schedule {
            self.guard("constructor_capture", observer)?;
            let window = self
                .configuration
                .windows
                .constructor
                .get(index)
                .ok_or_else(|| invalid("frozen constructor schedule index missing"))?;
            let (inputs, positions, target, count) =
                prepare(self.teacher, window, self.configuration.fit.layer)?;
            charge_capture(&mut self.report.work, count)?;
            self.report.work.constructor_positions =
                add_count(self.report.work.constructor_positions, count)?;
            self.guard("student_forward", observer)?;
            let attended =
                self.transfer
                    .attend(self.configuration.fit.layer, &inputs, &positions)?;
            self.report.work.student_forward_positions =
                add_count(self.report.work.student_forward_positions, count)?;
            let loss = self.transfer.post_wo_mse(&attended, &target)?;
            self.guard("backward", observer)?;
            let gradients = loss.objective.backward()?;
            self.report.work.backward_steps = add_count(self.report.work.backward_steps, 1)?;
            self.guard("adamw_update", observer)?;
            // Strict missing-gradient policy; NamedAdamW checks every gradient
            // norm, moment, and proposed parameter for finiteness before writes.
            // Zero gradients are valid (LoRA A starts behind zero B).
            let step = self
                .optimizer
                .step(self.transfer.parameters(), &gradients)?;
            self.report.updates.push(step);
            self.report.work.completed_updates += 1;
            self.report.constructor_diagnostics = self.transfer.diagnostics().clone();
        }
        self.evaluate_after_updates(observer)?;
        self.guard("completed", observer)?;
        Ok(self.report())
    }

    fn evaluate_after_updates(&mut self, observer: &mut dyn ResourceObserver) -> Result<()> {
        self.guard("evaluation_initialization", observer)?;
        let config = &self.configuration.fit;
        let mut initial = LayerTransfer::new(
            config.layer,
            self.teacher.shape(),
            self.teacher.attention_weights(config.layer)?,
            config.arm,
            config.paired.seed,
            self.teacher.device(),
        )?;
        self.transfer.reset_diagnostics();
        let mut initial_sum = MetricSum::default();
        let mut final_sum = MetricSum::default();
        for index in 0..self.configuration.windows.evaluation.len() {
            self.guard("evaluation_capture", observer)?;
            let (inputs, positions, target, count) = prepare(
                self.teacher,
                &self.configuration.windows.evaluation[index],
                self.configuration.fit.layer,
            )?;
            charge_capture(&mut self.report.work, count)?;
            self.report.work.evaluation_positions =
                add_count(self.report.work.evaluation_positions, count)?;
            self.guard("evaluation_initial_forward", observer)?;
            let before = initial.attend(self.configuration.fit.layer, &inputs, &positions)?;
            initial_sum.add(&initial, &before, &target, count)?;
            self.report.work.student_forward_positions =
                add_count(self.report.work.student_forward_positions, count)?;
            drop(before);
            self.guard("evaluation_final_forward", observer)?;
            let after = self
                .transfer
                .attend(self.configuration.fit.layer, &inputs, &positions)?;
            final_sum.add(&self.transfer, &after, &target, count)?;
            self.report.work.student_forward_positions =
                add_count(self.report.work.student_forward_positions, count)?;
        }
        let initial_metrics = initial_sum.finish()?;
        let final_metrics = final_sum.finish()?;
        let (gate, reduction_fraction) =
            reduction_gate(self.configuration.fit.arm, &initial_metrics, &final_metrics)?;
        self.report.evaluation = Some(EvaluationReport {
            role: self.configuration.windows.evaluation_role,
            previously_exposed: self.configuration.windows.evaluation_previously_exposed,
            initial: initial_metrics,
            final_parameters: final_metrics,
            reduction_fraction,
            gate,
            initial_diagnostics: initial.diagnostics().clone(),
            final_diagnostics: self.transfer.diagnostics().clone(),
        });
        Ok(())
    }

    /// Save complete or partial session state into a NEW leaf. No overwrite,
    /// no checkpoint pruning. A failure leaves an incomplete leaf to preserve.
    /// This may be used after a cooperative limit stops run(); the caller must
    /// have reserved the configured checkpoint allowance before starting.
    pub fn save_checkpoint(&self, directory: &Path) -> Result<()> {
        let limit = self.configuration.fit.paired.limits.max_checkpoint_bytes;
        let config = serde_json::to_vec(&self.configuration)?;
        let parameters = encode_parameters(&self.transfer)?;
        let report = self.report();
        if serde_json::to_vec(&report)?.len() > MAX_REPORT_BYTES {
            return Err(invalid("fit report exceeds checkpoint metadata bound"));
        }
        // Two moments per parameter, metadata and final manifest. Check before
        // creating output; an additional final actual byte check follows save.
        let bound = self
            .report
            .parameter_bytes
            .checked_mul(3)
            .and_then(|n| n.checked_add(config.len()))
            .and_then(|n| n.checked_add(CHECKPOINT_OVERHEAD_RESERVE))
            .ok_or_else(|| invalid("checkpoint projection overflow"))?;
        if bound > limit {
            return Err(invalid("checkpoint exceeds reserved byte bound"));
        }
        fs::create_dir(directory)?;
        write_new(&directory.join(CONFIG_FILE), &config)?;
        write_new(&directory.join(PARAMETERS_FILE), &parameters)?;
        self.optimizer.save(directory)?;
        let mut files = BTreeMap::new();
        let mut actual = 0usize;
        for name in [
            CONFIG_FILE,
            PARAMETERS_FILE,
            crate::joint_optimizer::MOMENT_FILE,
            crate::joint_optimizer::METADATA_FILE,
        ] {
            let bytes = fs::read(directory.join(name))?;
            actual = actual
                .checked_add(bytes.len())
                .ok_or_else(|| invalid("checkpoint byte overflow"))?;
            files.insert(
                name.into(),
                FileBinding {
                    bytes: bytes.len() as u64,
                    sha256: sha(&bytes),
                },
            );
        }
        let manifest = CheckpointManifest {
            format: FORMAT.into(),
            files,
            report,
            independent_reload_status: "NOT_RUN".into(),
        };
        let encoded = serde_json::to_vec(&manifest)?;
        if actual.checked_add(encoded.len()).is_none_or(|n| n > limit) {
            return Err(invalid(
                "actual checkpoint exceeds bound; leaf is incomplete",
            ));
        }
        write_new(&directory.join(MANIFEST_FILE), &encoded)
    }
}

#[derive(Default)]
struct MetricSum {
    positions: u64,
    elements: u64,
    squared_error: f64,
    target_energy: f64,
}
impl MetricSum {
    fn add(
        &mut self,
        transfer: &LayerTransfer,
        prediction: &Tensor,
        target: &Tensor,
        positions: u64,
    ) -> Result<()> {
        // Evaluation is detached: no backward graph is retained between windows.
        let loss = transfer.post_wo_mse(&prediction.detach(), &target.detach())?;
        let elements =
            u64::try_from(target.elem_count()).map_err(|_| invalid("metric element overflow"))?;
        self.positions = add_count(self.positions, positions)?;
        self.elements = add_count(self.elements, elements)?;
        self.squared_error += loss.raw_mse * elements as f64;
        self.target_energy += loss.target_energy;
        Ok(())
    }
    fn finish(self) -> Result<ErrorMetrics> {
        if self.elements == 0
            || !self.squared_error.is_finite()
            || !self.target_energy.is_finite()
            || self.squared_error < 0.0
            || self.target_energy < 0.0
        {
            return Err(invalid("invalid aggregate evaluation metrics"));
        }
        Ok(ErrorMetrics {
            positions: self.positions,
            elements: self.elements,
            squared_error: self.squared_error,
            target_energy: self.target_energy,
            raw_mse: self.squared_error / self.elements as f64,
            normalized_mse: (self.target_energy > 0.0)
                .then(|| self.squared_error / self.target_energy),
        })
    }
}

pub fn reduction_gate(
    arm: TransferArm,
    initial: &ErrorMetrics,
    final_metrics: &ErrorMetrics,
) -> Result<(ReductionGate, Option<f64>)> {
    if initial.positions != final_metrics.positions
        || initial.elements != final_metrics.elements
        || initial.target_energy != final_metrics.target_energy
    {
        return Err(invalid(
            "gate requires the identical evaluation target population",
        ));
    }
    let (Some(before), Some(after)) = (initial.normalized_mse, final_metrics.normalized_mse) else {
        return Ok((ReductionGate::UndefinedZeroTargetEnergy, None));
    };
    if !before.is_finite() || !after.is_finite() || before < 0.0 || after < 0.0 {
        return Err(invalid("nonfinite/negative normalized MSE"));
    }
    if before == 0.0 {
        return Ok((ReductionGate::NotApplicableZeroBaseline, None));
    }
    if matches!(arm, TransferArm::DenseControl) {
        return Ok((ReductionGate::NotApplicableDenseControl, None));
    }
    let reduction = 1.0 - after / before;
    Ok((
        if after <= 0.5 * before {
            ReductionGate::Pass
        } else {
            ReductionGate::Fail
        },
        Some(reduction),
    ))
}

/// Pair identity excludes ONLY the arm. A caller comparing independent runs
/// must require this key equality; seed equality alone does not match data/cost.
pub fn paired_key(config: &FitConfig, windows: &FixedWindows) -> Result<String> {
    json_sha(&(config.layer, &config.paired, &config.source, windows))
}

fn validate(config: &FitConfig, windows: &FixedWindows, teacher: &TrackBModel) -> Result<()> {
    for hash in [
        &config.source.weights_sha256,
        &config.source.config_sha256,
        &config.source.tokenizer_sha256,
        &windows.corpus_sha256,
        &windows.tokenizer_sha256,
    ] {
        validate_hash(hash)?;
    }
    if config.source.weights_sha256 != teacher.weights_sha256()
        || config.source.tokenizer_sha256 != windows.tokenizer_sha256
        || config.source.source_revision.is_empty()
        || config.paired.pair_id.is_empty()
        || config.layer >= teacher.shape().layers
    {
        return Err(invalid("fit source/layer/pair binding mismatch"));
    }
    let limits = &config.paired.limits;
    if limits.max_batch == 0
        || limits.max_time == 0
        || limits.max_windows == 0
        || limits.max_time > teacher.max_position_embeddings()
        || limits.max_total_data_tokens == 0
        || limits.max_attention_matrix_elements == 0
        || limits.max_parameter_bytes == 0
        || limits.max_checkpoint_bytes == 0
        || limits.max_wall_seconds == 0
        || limits.max_rss_bytes == 0
        || limits.minimum_available_bytes == 0
        || limits.compute_threads == 0
    {
        return Err(invalid("all resource limits must be explicit and positive"));
    }
    if config.paired.update_window_indices.is_empty()
        || config.paired.update_window_indices.len() > MAX_UPDATES
        || config
            .paired
            .update_window_indices
            .iter()
            .any(|i| *i >= windows.constructor.len())
        || windows.constructor.is_empty()
        || windows.evaluation.is_empty()
        || windows
            .constructor
            .len()
            .checked_add(windows.evaluation.len())
            .is_none_or(|n| n > limits.max_windows)
        || (windows.evaluation_role == EvaluationRole::IndependentFinalHeldOut
            && windows.evaluation_previously_exposed)
    {
        return Err(invalid(
            "invalid fixed schedule/window count/evaluation exposure",
        ));
    }
    config.paired.optimizer.validate()?;
    if !config.paired.optimizer.allowed_missing_gradients.is_empty() {
        return Err(invalid(
            "isolated transfer requires a gradient for every parameter",
        ));
    }
    if let TransferArm::Harmonic { dimensions, degree } = config.arm {
        if ![16, 32].contains(&dimensions) || !(1..=3).contains(&degree) {
            return Err(invalid(
                "harmonic arm requires explicit d16/32 and degree1..3",
            ));
        }
    }
    let mut ids = BTreeSet::new();
    let mut tokens = 0usize;
    let mut constructor_documents = BTreeSet::new();
    let mut evaluation_documents = BTreeSet::new();
    for (is_constructor, set) in [(true, &windows.constructor), (false, &windows.evaluation)] {
        for window in set {
            if window.id.is_empty() || !ids.insert(window.id.as_str()) {
                return Err(invalid("window ids must be unique"));
            }
            let time = window.rows.first().map_or(0, |row| row.ids.len());
            if window.rows.is_empty()
                || window.rows.len() > limits.max_batch
                || time == 0
                || time > limits.max_time
            {
                return Err(invalid("window batch/context exceeds declared limits"));
            }
            let gram = window
                .rows
                .len()
                .checked_mul(teacher.shape().heads)
                .and_then(|n| n.checked_mul(time))
                .and_then(|n| n.checked_mul(time))
                .ok_or_else(|| invalid("attention matrix element overflow"))?;
            if gram > limits.max_attention_matrix_elements {
                return Err(invalid("quadratic attention exceeds declared bound"));
            }
            for row in &window.rows {
                validate_hash(&row.document_sha256)?;
                if row.ids.len() != time
                    || row
                        .ids
                        .iter()
                        .any(|id| *id as usize >= teacher.shape().vocab)
                {
                    return Err(invalid(
                        "window must be rectangular and use source vocabulary ids",
                    ));
                }
                tokens = tokens
                    .checked_add(time)
                    .ok_or_else(|| invalid("data token overflow"))?;
                row.token_offset
                    .checked_add(time as u64)
                    .ok_or_else(|| invalid("source interval overflow"))?;
                if is_constructor {
                    constructor_documents.insert(row.document_sha256.as_str());
                } else {
                    evaluation_documents.insert(row.document_sha256.as_str());
                }
            }
        }
    }
    if tokens > limits.max_total_data_tokens {
        return Err(invalid("fixed data exceed token limit"));
    }
    require_document_separation(&constructor_documents, &evaluation_documents)?;
    Ok(())
}

fn require_document_separation(
    constructor: &BTreeSet<&str>,
    evaluation: &BTreeSet<&str>,
) -> Result<()> {
    if !constructor.is_disjoint(evaluation) {
        return Err(invalid(
            "constructor/evaluation documents overlap, regardless of token offsets",
        ));
    }
    Ok(())
}

fn projected_parameter_bytes(config: &FitConfig, shape: &LlamaShape) -> Result<usize> {
    let kv = shape
        .kv_heads
        .checked_mul(shape.head_dim)
        .ok_or_else(|| invalid("KV width overflow"))?;
    let lora = shape
        .width
        .checked_mul(4)
        .and_then(|n| n.checked_add(kv.checked_mul(2)?))
        .and_then(|n| n.checked_mul(LORA_RANK))
        .ok_or_else(|| invalid("LoRA parameter overflow"))?;
    let projection = match config.arm {
        TransferArm::DenseControl => 0,
        TransferArm::Harmonic { dimensions, .. } => shape
            .heads
            .checked_mul(dimensions)
            .and_then(|n| n.checked_mul(shape.head_dim))
            .and_then(|n| n.checked_mul(2))
            .ok_or_else(|| invalid("projection parameter overflow"))?,
    };
    lora.checked_add(projection)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| invalid("parameter byte overflow"))
}

fn prepare(
    teacher: &TrackBModel,
    window: &FixedWindow,
    layer: usize,
) -> Result<(AttentionQkv, AttentionPositions, Tensor, u64)> {
    let batch = window.rows.len();
    let time = window
        .rows
        .first()
        .ok_or_else(|| invalid("empty window"))?
        .ids
        .len();
    let ids = window
        .rows
        .iter()
        .flat_map(|row| row.ids.iter().copied())
        .collect::<Vec<_>>();
    let capture = teacher.capture_layer(&ids, batch, time, layer)?;
    let shape = teacher.shape();
    let weights = teacher.attention_weights(layer)?;
    let rows = batch
        .checked_mul(time)
        .ok_or_else(|| invalid("prepared row overflow"))?;
    let normalized = capture.normalized_input.detach();
    let flat = normalized.reshape((rows, shape.width))?;
    // Exact same F32 inverse frequencies and phase matmul as model.rs/Candle.
    let frequencies = (0..shape.head_dim)
        .step_by(2)
        .map(|i| 1.0f32 / (shape.rope_theta as f32).powf(i as f32 / shape.head_dim as f32))
        .collect::<Vec<_>>();
    let frequency = Tensor::from_vec(frequencies, (1, shape.head_dim / 2), teacher.device())?;
    let position = Tensor::arange(
        0u32,
        u32::try_from(time).map_err(|_| invalid("RoPE position overflow"))?,
        teacher.device(),
    )?
    .to_dtype(DType::F32)?
    .reshape((time, 1))?;
    let phase = position.matmul(&frequency)?;
    let cosine = phase.cos()?.reshape((1, 1, time, shape.head_dim / 2))?;
    let sine = phase.sin()?.reshape((1, 1, time, shape.head_dim / 2))?;
    let project = |weight: &Tensor, heads| -> Result<Tensor> {
        Ok(flat
            .matmul(&weight.t()?)?
            .reshape((batch, time, heads, shape.head_dim))?
            .transpose(1, 2)?
            .contiguous()?)
    };
    let mask_count = time
        .checked_mul(time)
        .ok_or_else(|| invalid("mask size overflow"))?;
    let mask = (0..mask_count)
        .map(|i| u8::from(i % time > i / time))
        .collect::<Vec<_>>();
    let inputs = AttentionQkv {
        query: rope(&project(&weights.query, shape.heads)?, &cosine, &sine)?,
        key: rope(&project(&weights.key, shape.kv_heads)?, &cosine, &sine)?,
        value: project(&weights.value, shape.kv_heads)?,
        excluded: Tensor::from_vec(mask, (1, 1, time, time), teacher.device())?,
        normalized_input: normalized,
        cosine,
        sine,
    };
    Ok((
        inputs,
        AttentionPositions {
            query: 0..time,
            key: 0..time,
        },
        capture.attention_output.detach(),
        rows as u64,
    ))
}

fn check_resources(
    limits: &FitLimits,
    started: Instant,
    observer: &mut dyn ResourceObserver,
    work: &mut WorkCounters,
) -> Result<()> {
    let observation = observer.observe()?;
    work.observed_peak_rss_bytes = work.observed_peak_rss_bytes.max(observation.rss_bytes);
    work.observed_minimum_available_bytes = Some(
        work.observed_minimum_available_bytes
            .map_or(observation.available_bytes, |n| {
                n.min(observation.available_bytes)
            }),
    );
    if started.elapsed().as_secs_f64() >= limits.max_wall_seconds as f64
        || observation.rss_bytes == 0
        || observation.rss_bytes > limits.max_rss_bytes
        || observation.available_bytes < limits.minimum_available_bytes
        || observation.enforced_compute_threads != limits.compute_threads
    {
        return Err(invalid(
            "fit resource bound reached; retain session and checkpoint, no implicit extension",
        ));
    }
    Ok(())
}

fn charge_capture(work: &mut WorkCounters, positions: u64) -> Result<()> {
    work.capture_calls = add_count(work.capture_calls, 1)?;
    work.captured_positions = add_count(work.captured_positions, positions)?;
    Ok(())
}
fn add_count(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| invalid("work counter overflow"))
}
fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn json_sha(value: &impl Serialize) -> Result<String> {
    Ok(sha(&serde_json::to_vec(value)?))
}
fn validate_hash(hash: &str) -> Result<()> {
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || hash.bytes().all(|b| b == b'0')
    {
        return Err(invalid("expected bound lowercase SHA-256"));
    }
    Ok(())
}
fn implementation_hashes() -> BTreeMap<String, String> {
    [
        ("fit", include_bytes!("track_b_fit.rs").as_slice()),
        ("transfer", include_bytes!("track_b_transfer.rs").as_slice()),
        (
            "model",
            include_bytes!("../src/track_b/model.rs").as_slice(),
        ),
        (
            "harmonics",
            include_bytes!("../src/track_b/harmonics.rs").as_slice(),
        ),
        (
            "optimizer",
            include_bytes!("../src/joint_optimizer.rs").as_slice(),
        ),
    ]
    .into_iter()
    .map(|(name, bytes)| (name.into(), sha(bytes)))
    .collect()
}

fn encode_parameters(transfer: &LayerTransfer) -> Result<Vec<u8>> {
    let mut encoded = BTreeMap::new();
    for (name, var) in transfer.parameters() {
        let values = var.as_detached_tensor().flatten_all()?.to_vec1::<f32>()?;
        if values.iter().any(|v| !v.is_finite()) {
            return Err(invalid("nonfinite checkpoint parameter"));
        }
        let data = values
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        encoded.insert(name.clone(), (var.dims().to_vec(), data));
    }
    // Explicit checked extraction avoids Candle Tensor-as-View's unwrap path.
    let views = encoded
        .iter()
        .map(|(name, (shape, bytes))| {
            Ok((
                name.as_str(),
                TensorView::new(SafeDtype::F32, shape.clone(), bytes)?,
            ))
        })
        .collect::<std::result::Result<Vec<_>, safetensors::SafeTensorError>>()?;
    Ok(safetensors::serialize(views, None)?)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

/// Independent parameter+moment reload into freshly initialized LoRA/projection
/// Vars. No training or gate is rerun here. Expected source/config and complete
/// file inventory must match; callers must subsequently evaluate loaded behavior.
pub fn reload_checkpoint(
    directory: &Path,
    teacher: &TrackBModel,
    expected_config: &FitConfig,
    expected_windows: &FixedWindows,
    observer: &mut dyn ResourceObserver,
) -> Result<(LayerTransfer, NamedAdamW, FitReport)> {
    validate(expected_config, expected_windows, teacher)?;
    let cap = expected_config.paired.limits.max_checkpoint_bytes;
    let expected_files = [
        CONFIG_FILE,
        PARAMETERS_FILE,
        MANIFEST_FILE,
        crate::joint_optimizer::MOMENT_FILE,
        crate::joint_optimizer::METADATA_FILE,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    let mut found = BTreeSet::new();
    let mut total = 0usize;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            return Err(invalid("checkpoint requires regular files only"));
        }
        let bytes = usize::try_from(entry.metadata()?.len())
            .map_err(|_| invalid("checkpoint file size overflow"))?;
        total = total
            .checked_add(bytes)
            .ok_or_else(|| invalid("checkpoint file total overflow"))?;
        found.insert(
            entry
                .file_name()
                .into_string()
                .map_err(|_| invalid("non-UTF8 checkpoint filename"))?,
        );
    }
    if found != expected_files || total > cap {
        return Err(invalid("checkpoint inventory/size mismatch"));
    }
    let manifest: CheckpointManifest =
        serde_json::from_slice(&fs::read(directory.join(MANIFEST_FILE))?)?;
    if manifest.format != FORMAT || manifest.files.len() != 4 {
        return Err(invalid("checkpoint schema/inventory mismatch"));
    }
    for name in [
        CONFIG_FILE,
        PARAMETERS_FILE,
        crate::joint_optimizer::MOMENT_FILE,
        crate::joint_optimizer::METADATA_FILE,
    ] {
        let binding = manifest
            .files
            .get(name)
            .ok_or_else(|| invalid("checkpoint binding absent"))?;
        let bytes = fs::read(directory.join(name))?;
        if bytes.len() as u64 != binding.bytes || sha(&bytes) != binding.sha256 {
            return Err(invalid("checkpoint content hash mismatch"));
        }
    }
    let configuration: Configuration =
        serde_json::from_slice(&fs::read(directory.join(CONFIG_FILE))?)?;
    let fresh = FitSession::new(
        teacher,
        expected_config.clone(),
        expected_windows.clone(),
        observer,
    )?;
    if json_sha(&configuration)? != fresh.report.configuration_sha256
        || manifest.report.configuration_sha256 != fresh.report.configuration_sha256
        || manifest.report.windows_sha256 != fresh.report.windows_sha256
        || manifest.report.paired_design_sha256 != fresh.report.paired_design_sha256
    {
        return Err(invalid("checkpoint source/configuration identity differs"));
    }
    let bytes = fs::read(directory.join(PARAMETERS_FILE))?;
    let tensors = SafeTensors::deserialize(&bytes)?;
    let tensor_names = tensors
        .names()
        .into_iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let expected_names = fresh
        .transfer
        .parameters()
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    if tensor_names != expected_names {
        return Err(invalid("checkpoint parameter inventory mismatch"));
    }
    // Validate and decode every parameter before setting any fresh variable.
    let mut restored = BTreeMap::new();
    for (name, var) in fresh.transfer.parameters() {
        let view = tensors.tensor(name)?;
        if view.dtype() != SafeDtype::F32 || view.shape() != var.dims() {
            return Err(invalid("checkpoint parameter dtype/shape mismatch"));
        }
        let values = view
            .data()
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect::<Vec<_>>();
        if values.len() != var.elem_count()
            || values.iter().any(|v| {
                !v.is_finite()
                    || f64::from(v.abs()) > expected_config.paired.optimizer.parameter_abs_limit
            })
        {
            return Err(invalid("checkpoint parameter finite bound failed"));
        }
        restored.insert(
            name.clone(),
            Tensor::from_vec(values, var.shape(), teacher.device())?,
        );
    }
    for (name, value) in restored {
        fresh
            .transfer
            .parameters()
            .get(&name)
            .ok_or_else(|| invalid("parameter disappeared"))?
            .set(&value)?;
    }
    let optimizer = NamedAdamW::load(
        directory,
        fresh.transfer.parameters(),
        &expected_config.paired.optimizer,
    )?;
    if optimizer.step_count() != manifest.report.work.completed_updates as u64
        || manifest.report.work.completed_updates
            > expected_config.paired.update_window_indices.len()
    {
        return Err(invalid("checkpoint update clocks disagree"));
    }
    Ok((fresh.transfer, optimizer, manifest.report))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(error: f64, energy: f64) -> ErrorMetrics {
        ErrorMetrics {
            positions: 4,
            elements: 16,
            squared_error: error,
            target_energy: energy,
            raw_mse: error / 16.0,
            normalized_mse: (energy > 0.0).then(|| error / energy),
        }
    }

    #[test]
    fn exact_half_reduction_passes_but_less_does_not() -> Result<()> {
        let arm = TransferArm::Harmonic {
            dimensions: 16,
            degree: 2,
        };
        assert_eq!(
            reduction_gate(arm, &metrics(8.0, 16.0), &metrics(4.0, 16.0))?.0,
            ReductionGate::Pass
        );
        assert_eq!(
            reduction_gate(arm, &metrics(8.0, 16.0), &metrics(4.01, 16.0))?.0,
            ReductionGate::Fail
        );
        assert!(reduction_gate(arm, &metrics(8.0, 16.0), &metrics(4.0, 8.0)).is_err());
        Ok(())
    }

    #[test]
    fn zero_baseline_and_zero_energy_never_become_pass() -> Result<()> {
        assert_eq!(
            reduction_gate(
                TransferArm::DenseControl,
                &metrics(0.0, 1.0),
                &metrics(0.0, 1.0)
            )?,
            (ReductionGate::NotApplicableZeroBaseline, None)
        );
        assert_eq!(
            reduction_gate(
                TransferArm::DenseControl,
                &metrics(1.0, 0.0),
                &metrics(0.0, 0.0)
            )?
            .0,
            ReductionGate::UndefinedZeroTargetEnergy
        );
        Ok(())
    }

    #[test]
    fn held_out_requires_document_separation_even_for_distant_offsets() -> Result<()> {
        let constructor = BTreeSet::from(["document_a"]);
        assert!(
            require_document_separation(&constructor, &BTreeSet::from(["document_a"])).is_err()
        );
        require_document_separation(&constructor, &BTreeSet::from(["document_b"]))
    }
}
