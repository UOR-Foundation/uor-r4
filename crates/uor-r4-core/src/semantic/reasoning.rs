use serde::{Deserialize, Serialize};
use super::reference::KappaLabel;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Constraint {
    pub facet: String,
    pub path: Vec<u16>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FacetPolicy {
    pub priority_order: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackoffPolicy {
    pub max_backoff_steps: u32,
    pub allow_any_facet: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidencePolicy {
    pub min_evidence_count: u32,
    pub require_provenance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Limits {
    pub max_probes: u32,
    pub max_operators: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeightedRoute {
    pub axis: u32,
    pub path: Vec<u16>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegionProbe {
    pub region_id: String,
    pub depth: u32,
    pub matched: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OperatorExecution {
    pub operator_cid: KappaLabel,
    pub input_route: Vec<u16>,
    pub output_routes: Vec<Vec<u16>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateScore {
    pub candidate_cid: KappaLabel,
    pub raw_score: f32,
    pub breakdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OperationCensus {
    pub total_probes: u64,
    pub total_operator_steps: u64,
    pub total_joins: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReasoningPlanV1 {
    pub query_cid: KappaLabel,
    pub semantic_space_cid: KappaLabel,
    pub clauses: Vec<Constraint>,
    pub operators: Vec<KappaLabel>,
    pub facet_policy: FacetPolicy,
    pub backoff_policy: BackoffPolicy,
    pub required_evidence: EvidencePolicy,
    pub deterministic_limits: Limits,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticInferenceWitnessV1 {
    pub query_cid: KappaLabel,
    pub plan_cid: KappaLabel,
    pub semantic_space_cid: KappaLabel,
    pub store_root_cids: Vec<KappaLabel>,
    pub generated_routes: Vec<WeightedRoute>,
    pub probed_regions: Vec<RegionProbe>,
    pub applied_operators: Vec<OperatorExecution>,
    pub evidence_cids: Vec<KappaLabel>,
    pub contradiction_cids: Vec<KappaLabel>,
    pub score_components: Vec<CandidateScore>,
    pub result_cid: KappaLabel,
    pub operation_census: OperationCensus,
}
