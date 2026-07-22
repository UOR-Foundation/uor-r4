//! Explicit ResolutionStatus and manifest-defined fallback policy engine (Phase 5 / Decision D4 / Theorem 12 / Plan §9.15).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionStatus {
    Supported,
    Boundary,
    BackedOff,
    Novel,
    Contradictory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalibratedFeatures {
    pub hamming_dist: u32,
    pub calibrated_radius: u32,
    pub score_margin: i32,
    pub frontier_density: u32,
    pub is_backed_off: bool,
}

impl CalibratedFeatures {
    /// Classify resolution status using integer features (Theorem 12).
    pub fn classify(&self) -> ResolutionStatus {
        if self.frontier_density > 100 {
            return ResolutionStatus::Contradictory;
        }
        if self.hamming_dist > self.calibrated_radius.saturating_mul(2) {
            return ResolutionStatus::Novel;
        }
        if self.is_backed_off {
            return ResolutionStatus::BackedOff;
        }
        if self.hamming_dist > self.calibrated_radius || self.score_margin.unsigned_abs() < 10 {
            return ResolutionStatus::Boundary;
        }
        ResolutionStatus::Supported
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackAction {
    ConsultExact,
    Abstain,
    BasePrior,
    FallbackToken(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FallbackPolicy {
    pub supported_action: FallbackAction,
    pub boundary_action: FallbackAction,
    pub backed_off_action: FallbackAction,
    pub novel_action: FallbackAction,
    pub contradictory_action: FallbackAction,
}

impl Default for FallbackPolicy {
    /// Default policy per Decision D4: consult EXCT for Supported/Boundary, back off to BasePrior, and abstain on Novel/Contradictory.
    fn default() -> Self {
        FallbackPolicy {
            supported_action: FallbackAction::ConsultExact,
            boundary_action: FallbackAction::ConsultExact,
            backed_off_action: FallbackAction::BasePrior,
            novel_action: FallbackAction::Abstain,
            contradictory_action: FallbackAction::Abstain,
        }
    }
}

impl FallbackPolicy {
    pub fn action_for(&self, status: ResolutionStatus) -> FallbackAction {
        match status {
            ResolutionStatus::Supported => self.supported_action.clone(),
            ResolutionStatus::Boundary => self.boundary_action.clone(),
            ResolutionStatus::BackedOff => self.backed_off_action.clone(),
            ResolutionStatus::Novel => self.novel_action.clone(),
            ResolutionStatus::Contradictory => self.contradictory_action.clone(),
        }
    }
}
