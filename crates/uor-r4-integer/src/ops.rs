//! Exact integer operation counters for the standalone integer session.
//!
//! CAR-LM build-order step 1 (measurement). Counters are ordinary integer
//! accumulators: increments only, no allocation and no floating point in the
//! counted model path. This measures the retained dense read path; cache hits,
//! routing, operator compositions and candidate reads do not exist yet and are
//! explicitly unmeasured.
//!
//! Counter definitions:
//! * `matrix_work_calls` — entries into the model's dense matrix-work routine,
//!   one per learned affine weight application in a model step.
//! * `matrix_work_inspections` — exact learned-code entries visited in those
//!   routines; each signed4/signed16 weight code is inspected once per call.
//!   For a full-context read-enabled step this is the plan's "cells read per
//!   token". Bias/vector work, table lookups and attention dot products are not
//!   included.
//! * `code_store.codes` / `code_store.nonzero_codes` — total and nonzero learned
//!   code entries across all loaded parameters, computed once at model load.
//! * `model_steps` — accepted (committed) model steps.
//! * `tokens_generated` — generated token IDs across the run.
//! * `inspections_per_step` — distribution over accepted steps of that step's
//!   `matrix_work_inspections`.

use serde::Serialize;
use std::path::Path;

use crate::{invalid, Result};

/// Per-model-step matrix-work counters.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StepOps {
    pub matrix_work_calls: u64,
    pub matrix_work_inspections: u64,
}

/// Static learned code-store size, computed once when the model is loaded.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CodeStoreStats {
    pub codes: u64,
    pub nonzero_codes: u64,
}

/// Accumulate one parameter's code entries into the store statistics. Called
/// once per parameter at load; not part of the model step path.
pub fn accumulate_store(codes: &[i16], stats: &mut CodeStoreStats) {
    stats.codes += codes.len() as u64;
    stats.nonzero_codes += codes.iter().filter(|&&code| code != 0).count() as u64;
}

/// Per-request counters returned by `Bundle::generate_counted`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GenerationOps {
    pub steps: u64,
    pub tokens_generated: u64,
    pub matrix_work_calls: u64,
    pub matrix_work_inspections: u64,
    pub per_step_inspections: Vec<u64>,
}

/// Running totals across a request batch.
#[derive(Clone, Debug, Default)]
pub struct OpsAccumulator {
    totals: GenerationOps,
}

impl OpsAccumulator {
    /// Add one request's counters. Per-step inspection counts are appended so
    /// the batch distribution covers every accepted step exactly once.
    pub fn record(&mut self, ops: &GenerationOps) {
        self.totals.steps += ops.steps;
        self.totals.tokens_generated += ops.tokens_generated;
        self.totals.matrix_work_calls += ops.matrix_work_calls;
        self.totals.matrix_work_inspections += ops.matrix_work_inspections;
        self.totals
            .per_step_inspections
            .extend_from_slice(&ops.per_step_inspections);
    }

    pub fn totals(&self) -> &GenerationOps {
        &self.totals
    }

    pub fn totals_mut(&mut self) -> &mut GenerationOps {
        &mut self.totals
    }
}

/// Distribution of `matrix_work_inspections` over accepted model steps.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct InspectionDistribution {
    pub steps: u64,
    pub min: u64,
    pub median: u64,
    pub p95: u64,
    pub max: u64,
    pub sum: u64,
}

/// Nearest-rank distribution over accepted-step inspection counts.
///
/// `values` is sorted in place. Median is the middle value, or the integer floor
/// of the two middle values' mean for an even count. `p95` is the nearest-rank
/// value at `ceil(0.95 * steps)`. Empty input yields an all-zero distribution.
pub fn summarize(values: &mut [u64]) -> InspectionDistribution {
    if values.is_empty() {
        return InspectionDistribution::default();
    }
    values.sort_unstable();
    let steps = values.len() as u64;
    let sum = values.iter().copied().sum::<u64>();
    let last = values.len() - 1;
    let median = if values.len() % 2 == 1 {
        values[values.len() / 2]
    } else {
        let low = values[values.len() / 2 - 1];
        let high = values[values.len() / 2];
        low + (high - low) / 2
    };
    // Integer nearest rank without floating point: ceil(95*n/100), then 1-based
    // rank to a 0-based sorted index clamped into range.
    let rank = ((95 * u128::from(steps) + 99) / 100) as usize;
    let index = rank.saturating_sub(1).min(last);
    InspectionDistribution {
        steps,
        min: values[0],
        median,
        p95: values[index],
        max: values[last],
        sum,
    }
}

/// Serialized code-store view with deterministic field order.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct CodeStoreView {
    pub codes: u64,
    pub nonzero_codes: u64,
}

pub const OPS_SCHEMA: &str = "uor-r4.integer-ops-report/1";
pub const OPS_SCOPE: &str = "Exact integer counts for the retained dense integer session: matrix_work code entries inspected, matrix_work calls, learned code-store size, accepted model steps and generated tokens. Counters only; no change to model arithmetic or output.";
pub const OPS_UNMEASURED: [&str; 4] = [
    "cache hits, routing and address selection (not implemented)",
    "operator compositions and candidate reads (not implemented)",
    "bias/vector work, tanh/sigmoid/exp table lookups and attention dot products",
    "latency, RAM and energy (separate wall/energy instruments)",
];

/// Machine-readable operations report. Field declaration order is the
/// serialization order, so the JSON is deterministic across runs.
#[derive(Clone, Debug, Serialize)]
pub struct OpsReport {
    pub schema: &'static str,
    pub bundle_sha256: String,
    pub requests: u64,
    pub model_steps: u64,
    pub tokens_generated: u64,
    pub matrix_work_calls: u64,
    pub matrix_work_inspections: u64,
    pub code_store: CodeStoreView,
    pub inspections_per_step: InspectionDistribution,
    pub scope: &'static str,
    pub unmeasured: [&'static str; 4],
}

impl OpsReport {
    /// Build the report from batch totals and the load-time code-store size.
    /// Consumes the per-step counts into the distribution.
    pub fn build(
        bundle_sha256: String,
        requests: u64,
        totals: &mut GenerationOps,
        code_store: CodeStoreStats,
    ) -> OpsReport {
        let inspections_per_step = summarize(&mut totals.per_step_inspections);
        OpsReport {
            schema: OPS_SCHEMA,
            bundle_sha256,
            requests,
            model_steps: totals.steps,
            tokens_generated: totals.tokens_generated,
            matrix_work_calls: totals.matrix_work_calls,
            matrix_work_inspections: totals.matrix_work_inspections,
            code_store: CodeStoreView {
                codes: code_store.codes,
                nonzero_codes: code_store.nonzero_codes,
            },
            inspections_per_step,
            scope: OPS_SCOPE,
            unmeasured: OPS_UNMEASURED,
        }
    }
}

/// Fail before model work when the report path already exists, so a run never
/// overwrites a retained report.
pub fn ensure_absent(path: &Path) -> Result<()> {
    if path.exists() {
        return Err(invalid(format!(
            "ops report path {} already exists; choose a new path",
            path.display()
        )));
    }
    Ok(())
}

/// Write the report exclusively (create-new) as pretty JSON. Parent directories
/// are created as containers; an existing leaf fails. Never part of the model
/// step path.
pub fn write_report(path: &Path, report: &OpsReport) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::File::create_new(path).map_err(|error| {
        invalid(format!(
            "ops report {} was not created exclusively ({error}); choose a new path",
            path.display()
        ))
    })?;
    std::io::Write::write_all(&mut file, &serde_json::to_vec_pretty(report)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_store_stats_count_total_and_nonzero_exactly() {
        let mut stats = CodeStoreStats::default();
        accumulate_store(&[0, 1, -1, 0, 7], &mut stats);
        accumulate_store(&[], &mut stats);
        accumulate_store(&[0, 0], &mut stats);
        assert_eq!(stats.codes, 7);
        assert_eq!(stats.nonzero_codes, 3);
    }

    #[test]
    fn distribution_uses_nearest_rank_and_integer_median() {
        let odd = summarize(&mut [10, 30, 20]);
        assert_eq!(odd.steps, 3);
        assert_eq!(odd.min, 10);
        assert_eq!(odd.median, 20);
        assert_eq!(odd.p95, 30);
        assert_eq!(odd.max, 30);
        assert_eq!(odd.sum, 60);

        let even = summarize(&mut [10, 20, 30, 40]);
        assert_eq!(even.steps, 4);
        assert_eq!(even.min, 10);
        assert_eq!(even.median, 25);
        assert_eq!(even.p95, 40);
        assert_eq!(even.max, 40);
        assert_eq!(even.sum, 100);

        // 100 steps: nearest rank ceil(95) -> index 94 of the sorted values.
        let mut hundred: Vec<u64> = (1..=100).collect();
        let hundred = summarize(&mut hundred);
        assert_eq!(hundred.steps, 100);
        assert_eq!(hundred.median, 50);
        assert_eq!(hundred.p95, 95);
        assert_eq!(hundred.max, 100);
        assert_eq!(hundred.sum, 5050);

        assert_eq!(summarize(&mut []), InspectionDistribution::default());
    }

    #[test]
    fn accumulator_sums_counters_and_keeps_every_step() {
        let mut accumulator = OpsAccumulator::default();
        accumulator.record(&GenerationOps {
            steps: 3,
            tokens_generated: 2,
            matrix_work_calls: 30,
            matrix_work_inspections: 300,
            per_step_inspections: vec![100, 100, 100],
        });
        accumulator.record(&GenerationOps {
            steps: 2,
            tokens_generated: 2,
            matrix_work_calls: 12,
            matrix_work_inspections: 80,
            per_step_inspections: vec![40, 40],
        });
        let totals = accumulator.totals();
        assert_eq!(totals.steps, 5);
        assert_eq!(totals.tokens_generated, 4);
        assert_eq!(totals.matrix_work_calls, 42);
        assert_eq!(totals.matrix_work_inspections, 380);
        assert_eq!(totals.per_step_inspections, vec![100, 100, 100, 40, 40]);
    }

    #[test]
    fn report_serializes_in_deterministic_declared_order() {
        let mut totals = GenerationOps {
            steps: 2,
            tokens_generated: 2,
            matrix_work_calls: 20,
            matrix_work_inspections: 150,
            per_step_inspections: vec![100, 50],
        };
        let report = OpsReport::build(
            "bundle-sha".into(),
            1,
            &mut totals,
            CodeStoreStats {
                codes: 300,
                nonzero_codes: 250,
            },
        );
        let json = serde_json::to_string(&report).unwrap();
        let keys = [
            "\"schema\"",
            "\"bundle_sha256\"",
            "\"requests\"",
            "\"model_steps\"",
            "\"tokens_generated\"",
            "\"matrix_work_calls\"",
            "\"matrix_work_inspections\"",
            "\"code_store\"",
            "\"inspections_per_step\"",
            "\"scope\"",
            "\"unmeasured\"",
        ];
        let mut position = 0usize;
        for key in keys {
            let found = json[position..]
                .find(key)
                .map(|offset| position + offset)
                .unwrap_or_else(|| panic!("missing or out-of-order key {key} in {json}"));
            position = found + key.len();
        }
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["schema"], OPS_SCHEMA);
        assert_eq!(parsed["bundle_sha256"], "bundle-sha");
        assert_eq!(parsed["requests"], 1);
        assert_eq!(parsed["model_steps"], 2);
        assert_eq!(parsed["tokens_generated"], 2);
        assert_eq!(parsed["matrix_work_calls"], 20);
        assert_eq!(parsed["matrix_work_inspections"], 150);
        assert_eq!(parsed["code_store"]["codes"], 300);
        assert_eq!(parsed["code_store"]["nonzero_codes"], 250);
        assert_eq!(parsed["inspections_per_step"]["steps"], 2);
        assert_eq!(parsed["inspections_per_step"]["min"], 50);
        assert_eq!(parsed["inspections_per_step"]["median"], 75);
        assert_eq!(parsed["inspections_per_step"]["p95"], 100);
        assert_eq!(parsed["inspections_per_step"]["max"], 100);
        assert_eq!(parsed["inspections_per_step"]["sum"], 150);
        assert_eq!(parsed["unmeasured"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn write_report_creates_exclusively_and_reparses() {
        let base = std::env::temp_dir().join(format!(
            "uor-r4-ops-report-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = base.join("nested").join("ops.json");
        let mut totals = GenerationOps {
            steps: 1,
            tokens_generated: 1,
            matrix_work_calls: 10,
            matrix_work_inspections: 42,
            per_step_inspections: vec![42],
        };
        let report = OpsReport::build(
            "bundle-sha".into(),
            1,
            &mut totals,
            CodeStoreStats::default(),
        );
        assert!(ensure_absent(&path).is_ok());
        write_report(&path, &report).unwrap();
        assert!(ensure_absent(&path).is_err());
        assert!(write_report(&path, &report).is_err());
        let parsed: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(parsed["matrix_work_inspections"], 42);
        assert_eq!(parsed["inspections_per_step"]["steps"], 1);
        std::fs::remove_dir_all(base).unwrap();
    }
}
