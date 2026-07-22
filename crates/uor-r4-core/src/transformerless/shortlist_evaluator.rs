//! Shortlist top-M recall and fallback behavior evaluation vs reference classifier
//! (Phase 6 / Decision D5 / Gate H / Plan §9.16).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShortlistMetrics {
    pub top1_recall: f64,
    pub top3_recall: f64,
    pub top5_recall: f64,
    pub top10_recall: f64,
    pub top20_recall: f64,
    pub false_negative_rate: f64,
    pub fallback_rate: f64,
    pub conditional_direct_fidelity: f64,
    pub worst_routing_error: i32,
    pub gate_h_passed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShortlistRecallReport {
    pub metrics: ShortlistMetrics,
    pub trigger_gated_fallback_active: bool,
}

impl ShortlistRecallReport {
    pub fn to_cbor_bytes(&self) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf).map_err(|e| e.to_string())?;
        Ok(buf)
    }

    pub fn from_cbor_bytes(bytes: &[u8]) -> Result<Self, String> {
        ciborium::from_reader(bytes).map_err(|e| e.to_string())
    }
}

pub struct ShortlistEvaluator;

impl ShortlistEvaluator {
    /// Evaluate shortlist predictions against reference targets and score vectors.
    pub fn evaluate(
        shortlists: &[Vec<u32>],
        references: &[u32],
        scores_graph: &[i32],
        scores_teacher: &[i32],
        min_top5_recall_threshold: f64,
    ) -> ShortlistRecallReport {
        if shortlists.is_empty() || references.is_empty() || shortlists.len() != references.len() {
            return ShortlistRecallReport {
                metrics: ShortlistMetrics {
                    top1_recall: 0.0,
                    top3_recall: 0.0,
                    top5_recall: 0.0,
                    top10_recall: 0.0,
                    top20_recall: 0.0,
                    false_negative_rate: 1.0,
                    fallback_rate: 0.0,
                    conditional_direct_fidelity: 0.0,
                    worst_routing_error: 0,
                    gate_h_passed: false,
                },
                trigger_gated_fallback_active: true,
            };
        }

        let n = shortlists.len() as f64;
        let mut top1_matches = 0;
        let mut top3_matches = 0;
        let mut top5_matches = 0;
        let mut top10_matches = 0;
        let mut top20_matches = 0;
        let mut fn_count = 0;

        for (list, &ref_target) in shortlists.iter().zip(references.iter()) {
            match list.iter().position(|&t| t == ref_target) {
                None => {
                    fn_count += 1;
                }
                Some(idx) => {
                    if idx == 0 {
                        top1_matches += 1;
                    }
                    if idx < 3 {
                        top3_matches += 1;
                    }
                    if idx < 5 {
                        top5_matches += 1;
                    }
                    if idx < 10 {
                        top10_matches += 1;
                    }
                    if idx < 20 {
                        top20_matches += 1;
                    }
                }
            }
        }

        let r1 = top1_matches as f64 / n;
        let r3 = top3_matches as f64 / n;
        let r5 = top5_matches as f64 / n;
        let r10 = top10_matches as f64 / n;
        let r20 = top20_matches as f64 / n;
        let fnr = fn_count as f64 / n;

        // Compute worst routing error max |scores_graph - scores_teacher|
        let mut worst_err: i32 = 0;
        let len = scores_graph.len().min(scores_teacher.len());
        for i in 0..len {
            let diff = (scores_graph[i] - scores_teacher[i]).abs();
            if diff > worst_err {
                worst_err = diff;
            }
        }

        const MAX_WORST_ROUTING_ERROR: i32 = 1_000;

        // Fallback is required when the reference target is missing from the shortlist.
        let fallback_rate = fnr;

        // Fidelity of the direct (non-fallback) path, conditioned on not requiring fallback.
        let direct_denom = n - fn_count as f64;
        let conditional_direct_fidelity = if direct_denom > 0.0 {
            top1_matches as f64 / direct_denom
        } else {
            0.0
        };

        let gate_h_passed = r5 >= min_top5_recall_threshold && worst_err <= MAX_WORST_ROUTING_ERROR;
        let trigger_gated_fallback_active = r5 < min_top5_recall_threshold;

        ShortlistRecallReport {
            metrics: ShortlistMetrics {
                top1_recall: r1,
                top3_recall: r3,
                top5_recall: r5,
                top10_recall: r10,
                top20_recall: r20,
                false_negative_rate: fnr,
                fallback_rate,
                conditional_direct_fidelity,
                worst_routing_error: worst_err,
                gate_h_passed,
            },
            trigger_gated_fallback_active,
        }
    }
}
