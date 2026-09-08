//! Diagnostic timing of an in-process native model session.
//! This is not a complete deployed-path or quality-matched M1 benchmark.
//! Energy, thermals, RSS and hardware identity are unavailable without measurements.

use super::{Control, Error, Model, Result, Session, BOS, EOS};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Sum of selected measured in-process calls in microseconds; see timing_scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StageBreakdown {
    pub cold_load_us: u64,
    pub ingest_us: u64,
    pub geometric_lookup_us: Option<u64>,
    pub operator_execution_us: Option<u64>,
    /// Measured predict-call duration, without an invented lookup/operator split.
    pub prediction_us: u64,
    pub token_emission_us: u64,
    pub persistence_us: u64,
    pub total_us: u64,
}

impl StageBreakdown {
    pub fn compute_total(&mut self) {
        self.total_us = self
            .cold_load_us
            .saturating_add(self.ingest_us)
            .saturating_add(self.prediction_us)
            .saturating_add(self.token_emission_us)
            .saturating_add(self.persistence_us);
    }
}

/// Statistical distribution of microsecond latencies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LatencyDistribution {
    pub count: usize,
    pub min_us: u64,
    pub median_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
    pub max_us: u64,
    pub mean_us: u64,
}

impl LatencyDistribution {
    pub fn from_samples(mut samples: Vec<u64>) -> Self {
        if samples.is_empty() {
            return Self::default();
        }
        samples.sort_unstable();
        let count = samples.len();
        let min_us = samples[0];
        let max_us = samples[count - 1];
        let median_us = samples[count / 2];
        let p95_idx = ((count as f64) * 0.95).floor() as usize;
        let p95_us = samples[p95_idx.min(count - 1)];
        let p99_idx = ((count as f64) * 0.99).floor() as usize;
        let p99_us = samples[p99_idx.min(count - 1)];
        let sum: u64 = samples.iter().copied().sum();
        let mean_us = sum / (count as u64);

        Self {
            count,
            min_us,
            median_us,
            p95_us,
            p99_us,
            max_us,
            mean_us,
        }
    }
}

/// Hardware specification and memory footprint accounting on Apple M1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareMetrics {
    pub cpu_brand: Option<String>,
    pub architecture: String,
    pub logical_cores: usize,
    pub physical_cores: Option<usize>,
    pub total_memory_bytes: Option<u64>,
    pub resident_memory_bytes: Option<u64>,
    pub ring_storage_bytes: usize,
    pub candidate_storage_bytes: usize,
    pub table_storage_bytes: usize,
}

impl HardwareMetrics {
    pub fn capture(model: &Model, session: &Session) -> Self {
        let logical_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(8);
        let ring_storage_bytes = session.ring.len() * std::mem::size_of::<u32>();
        let candidate_storage_bytes =
            model.config.candidate_limit * std::mem::size_of::<super::Candidate>();
        let table_storage_bytes = model.rows.len() * std::mem::size_of::<super::ScoreRow>()
            + model.training.learned_associations * std::mem::size_of::<super::TokenScore>();

        Self {
            cpu_brand: None,
            architecture: std::env::consts::ARCH.to_string(),
            logical_cores,
            physical_cores: None,
            total_memory_bytes: None,
            resident_memory_bytes: None,
            ring_storage_bytes,
            candidate_storage_bytes,
            table_storage_bytes,
        }
    }
}

/// Missing energy and thermal measurements; no default power assumption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct M1EnergyModel {
    /// Active CPU power in milliwatts, unavailable without sensor evidence.
    pub estimated_power_mw: Option<u64>,
    /// Total duration in microseconds.
    pub duration_us: u64,
    /// Energy consumed per generated token in microjoules (µJ).
    pub energy_per_token_uj: Option<u64>,
    /// Total completed task energy in millijoules (mJ).
    pub total_energy_mj: Option<u64>,
    /// Sustained thermal operating condition.
    pub thermal_status: Option<String>,
}

impl M1EnergyModel {
    /// No sensor input is available. Preserve elapsed time and report missing data.
    pub fn estimate(duration_us: u64, _token_count: usize) -> Self {
        Self {
            estimated_power_mw: None,
            duration_us,
            energy_per_token_uj: None,
            total_energy_mj: None,
            thermal_status: None,
        }
    }
}

/// Canonical workload tasks for quality/cost benchmarking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskKind {
    NarrativeProse,
    ProceduralDialogue,
    GroundedMemory,
    MultiStepReasoning,
    RustCodeSynthesis,
}

/// Comprehensive benchmark report for a single workload task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskBenchmarkResult {
    pub task: TaskKind,
    pub prompt: String,
    pub output_tokens: usize,
    pub output_text: String,
    pub stages: StageBreakdown,
    pub decision_latency: LatencyDistribution,
    pub token_latency: LatencyDistribution,
    pub tokens_per_second: u64,
    pub hardware: HardwareMetrics,
    pub energy: M1EnergyModel,
    /// Execution emitted tokens; this is not a correctness or quality verdict.
    pub success: bool,
    pub quality_verified: bool,
    pub timing_scope: String,
}

/// Diagnostic timer, portable to native hosts; not a hardware qualification.
pub struct M1Profiler;

impl M1Profiler {
    /// Times selected in-process operations without a speed or quality acceptance floor.
    pub fn profile_task(
        model: &Model,
        task: TaskKind,
        prompt: &str,
        max_tokens: usize,
        control: Control,
    ) -> Result<TaskBenchmarkResult> {
        let mut stages = StageBreakdown::default();

        // 1. Cold Load: serialize and deserialize model artifact to simulate loading
        let t_cold_start = Instant::now();
        let serialized = serde_json::to_vec(model)
            .map_err(|e| Error(format!("model serialization error: {e}")))?;
        let loaded_model: Model = serde_json::from_slice(&serialized)
            .map_err(|e| Error(format!("model deserialization error: {e}")))?;
        stages.cold_load_us = t_cold_start.elapsed().as_micros() as u64;

        // 2. Ingestion: initialize session, observe BOS and prompt tokens
        let t_ingest_start = Instant::now();
        let mut session = loaded_model.session(control)?;
        session.observe(&loaded_model, BOS)?;
        let prompt_tokens = loaded_model.encode(prompt)?;
        for token in prompt_tokens {
            session.observe(&loaded_model, token)?;
        }
        session.begin_response(&loaded_model)?;
        stages.ingest_us = t_ingest_start.elapsed().as_micros() as u64;

        // 3 & 4 & 5. Generation Loop: geometric lookup, operator execution, and token emission
        let mut decision_samples = Vec::new();
        let mut token_samples = Vec::new();
        let mut generated_tokens = Vec::new();

        let mut total_prediction_us = 0u64;
        let mut total_emission_us = 0u64;

        for _ in 0..max_tokens {
            let t_token_start = Instant::now();

            // Measure lookup & operator execution via predict
            let t_decision_start = Instant::now();
            let pred = session.predict(&loaded_model)?;
            let decision_nanos = t_decision_start.elapsed().as_nanos() as u64;
            let decision_dur_us = (decision_nanos + 999) / 1000;
            decision_samples.push(decision_dur_us);

            total_prediction_us = total_prediction_us.saturating_add(decision_dur_us);

            // Measure token emission & state update
            let t_emission_start = Instant::now();
            let token = pred.token;
            generated_tokens.push(token);

            if token == EOS {
                if session.response_decision().is_some() || loaded_model.values.is_some() {
                    session.observe(&loaded_model, token)?;
                }
                let emission_nanos = t_emission_start.elapsed().as_nanos() as u64;
                let emission_dur = (emission_nanos + 999) / 1000;
                total_emission_us = total_emission_us.saturating_add(emission_dur);
                let token_nanos = t_token_start.elapsed().as_nanos() as u64;
                token_samples.push((token_nanos + 999) / 1000);
                break;
            }

            session.observe(&loaded_model, token)?;
            let emission_nanos = t_emission_start.elapsed().as_nanos() as u64;
            let emission_dur = (emission_nanos + 999) / 1000;
            total_emission_us = total_emission_us.saturating_add(emission_dur);

            let token_nanos = t_token_start.elapsed().as_nanos() as u64;
            token_samples.push((token_nanos + 999) / 1000);
        }

        stages.prediction_us = total_prediction_us;
        stages.token_emission_us = total_emission_us;

        // 6. Persistence: serialize session checkpoint
        let t_persist_start = Instant::now();
        let _checkpoint = session.checkpoint();
        stages.persistence_us = t_persist_start.elapsed().as_micros() as u64;

        // Sum the selected measured calls; omitted deployment costs remain excluded.
        stages.compute_total();

        let decoded_bytes = loaded_model.decode(&generated_tokens)?;
        let output_text = String::from_utf8_lossy(&decoded_bytes).into_owned();
        let output_tokens = generated_tokens.len();

        let decision_latency = LatencyDistribution::from_samples(decision_samples);
        let token_latency = LatencyDistribution::from_samples(token_samples);

        let gen_time_us = stages
            .prediction_us
            .saturating_add(stages.token_emission_us);

        let tokens_per_second = if gen_time_us > 0 {
            ((output_tokens as u64) * 1_000_000) / gen_time_us
        } else {
            0
        };

        let hardware = HardwareMetrics::capture(&loaded_model, &session);
        let energy = M1EnergyModel::estimate(stages.total_us, output_tokens);

        Ok(TaskBenchmarkResult {
            task,
            prompt: prompt.to_string(),
            output_tokens,
            output_text,
            stages,
            decision_latency,
            token_latency,
            tokens_per_second,
            hardware,
            energy,
            success: output_tokens > 0,
            quality_verified: false,
            timing_scope: "In-process serialize/deserialize, ingest, predict, observe and checkpoint calls; excludes file I/O, artifact validation, decoding, UI and persistence writes. No separate lookup/operator timing or energy/thermal measurement.".into(),
        })
    }

    /// Measures parallel throughput scaling (1 worker vs N workers).
    pub fn profile_parallel_scaling(
        model: &Model,
        prompt: &str,
        tokens_per_task: usize,
        workers: usize,
    ) -> Result<(u64, u64, bool)> {
        let iterations = workers.max(1);

        // 1. Sequential execution (1 worker)
        let t_seq_start = Instant::now();
        let mut seq_outputs = Vec::with_capacity(iterations);
        for _ in 0..iterations {
            let res = Self::profile_task(
                model,
                TaskKind::NarrativeProse,
                prompt,
                tokens_per_task,
                Control::Full,
            )?;
            seq_outputs.push(res.output_text);
        }
        let seq_total_us = t_seq_start.elapsed().as_micros() as u64;

        // 2. Parallel execution using standard thread scoped workers
        let t_par_start = Instant::now();
        let par_outputs = std::thread::scope(|s| {
            let mut handles = Vec::with_capacity(iterations);
            for _ in 0..iterations {
                handles.push(s.spawn(|| {
                    Self::profile_task(
                        model,
                        TaskKind::NarrativeProse,
                        prompt,
                        tokens_per_task,
                        Control::Full,
                    )
                }));
            }
            let mut results = Vec::with_capacity(iterations);
            for h in handles {
                results.push(h.join().unwrap());
            }
            results
        });
        let par_total_us = t_par_start.elapsed().as_micros() as u64;

        // Verify deterministic output equality across all executions
        let mut deterministic = true;
        for (seq_out, par_res) in seq_outputs.iter().zip(par_outputs.iter()) {
            if let Ok(par_item) = par_res {
                if seq_out != &par_item.output_text {
                    deterministic = false;
                    break;
                }
            } else {
                deterministic = false;
                break;
            }
        }

        Ok((seq_total_us, par_total_us, deterministic))
    }
}
