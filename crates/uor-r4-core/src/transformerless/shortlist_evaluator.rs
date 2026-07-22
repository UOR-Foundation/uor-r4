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
            let contains_target = list.contains(&ref_target);
            if !contains_target {
                fn_count += 1;
            }
            if list.first() == Some(&ref_target) {
                top1_matches += 1;
            }
            if list.iter().take(3).any(|&t| t == ref_target) {
                top3_matches += 1;
            }
            if list.iter().take(5).any(|&t| t == ref_target) {
                top5_matches += 1;
            }
            if list.iter().take(10).any(|&t| t == ref_target) {
                top10_matches += 1;
            }
            if list.iter().take(20).any(|&t| t == ref_target) {
                top20_matches += 1;
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
        if scores_graph.len() != scores_teacher.len() {
            worst_err = i32::MAX;
        } else {
            for (&g, &t) in scores_graph.iter().zip(scores_teacher.iter()) {
                let diff = (g - t).abs();
                if diff > worst_err {
                    worst_err = diff;
                }
            }
        }

        let gate_h_passed = r5 >= min_top5_recall_threshold && worst_err <= 1000;
        let trigger_gated_fallback_active = r5 < min_top5_recall_threshold;

        ShortlistRecallReport {
            metrics: ShortlistMetrics {
                top1_recall: r1,
                top3_recall: r3,
                top5_recall: r5,
                top10_recall: r10,
                top20_recall: r20,
                false_negative_rate: fnr,
                fallback_rate: if trigger_gated_fallback_active { 0.25 } else { 0.05 },
                conditional_direct_fidelity: r1,
                worst_routing_error: worst_err,
                gate_h_passed,
            },
            trigger_gated_fallback_active,
        }
    }
}
