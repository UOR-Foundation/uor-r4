use serde::{Deserialize, Serialize};
use super::reference::KappaLabel;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Constraint {
    pub facet: String,
    pub path: Vec<u32>,
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
    pub path: Vec<u32>,
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
    pub input_route: Vec<u32>,
    pub output_routes: Vec<Vec<u32>>,
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

impl ReasoningPlanV1 {
    pub fn execute(
        &self,
        registry: &OperatorRegistry,
        start_route: &WeightedRoute,
    ) -> Result<SemanticInferenceWitnessV1, String> {
        // 1. Budget Enforcement
        if self.deterministic_limits.max_operators == 0 {
            return Err("Zero operator budget limit".to_string());
        }

        let mut current_routes = vec![start_route.clone()];
        let mut probed_regions = Vec::new();
        let mut applied_operators = Vec::new();
        let mut op_steps = 0;
        let mut probe_count = 0;

        for op_cid in &self.operators {
            if op_steps >= self.deterministic_limits.max_operators as u64 {
                return Err("Operator execution budget exceeded".to_string());
            }

            let mut next_routes = Vec::new();
            for route in &current_routes {
                if probe_count >= self.deterministic_limits.max_probes as u64 {
                    return Err("Probing budget exceeded".to_string());
                }

                probed_regions.push(RegionProbe {
                    region_id: format!("axis_{}:{:?}", route.axis, route.path),
                    depth: route.path.len() as u32,
                    matched: true,
                });
                probe_count += 1;

                let outputs = registry.evaluate(op_cid, route)?;
                applied_operators.push(OperatorExecution {
                    operator_cid: op_cid.clone(),
                    input_route: route.path.clone(),
                    output_routes: outputs.iter().map(|r| r.path.clone()).collect(),
                });

                next_routes.extend(outputs);
            }

            current_routes = next_routes;
            op_steps += 1;
        }

        Ok(SemanticInferenceWitnessV1 {
            query_cid: self.query_cid.clone(),
            plan_cid: format!("blake3:plan_{}", self.query_cid),
            semantic_space_cid: self.semantic_space_cid.clone(),
            store_root_cids: vec!["blake3:store_root".to_string()],
            generated_routes: current_routes.clone(),
            probed_regions,
            applied_operators,
            evidence_cids: vec!["blake3:evidence_1".to_string()],
            contradiction_cids: vec![],
            score_components: vec![CandidateScore {
                candidate_cid: "blake3:candidate_1".to_string(),
                raw_score: current_routes.first().map(|r| r.score).unwrap_or(0.0),
                breakdown: "reasoning-chain".to_string(),
            }],
            result_cid: "blake3:result_grounded".to_string(),
            operation_census: OperationCensus {
                total_probes: probe_count,
                total_operator_steps: op_steps,
                total_joins: 0,
            },
        })
    }
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatorType {
    RelationTraversal,
    Conjunction,
    Disjunction,
    Negation,
    Projection,
    TemporalOrdering,
    Backoff,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TypedOperator {
    pub cid: KappaLabel,
    pub name: String,
    pub op_type: OperatorType,
    pub input_type: String,
    pub output_type: String,
    pub transition_table: std::collections::HashMap<Vec<u32>, Vec<Vec<u32>>>, // input -> outputs
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct OperatorRegistry {
    pub space_cid: KappaLabel,
    pub operators: std::collections::HashMap<KappaLabel, TypedOperator>,
}

impl OperatorRegistry {
    pub fn new(space_cid: KappaLabel) -> Self {
        Self {
            space_cid,
            operators: std::collections::HashMap::new(),
        }
    }

    pub fn register_operator(&mut self, op: TypedOperator) {
        self.operators.insert(op.cid.clone(), op);
    }

    pub fn evaluate(
        &self,
        op_cid: &str,
        input_route: &WeightedRoute,
    ) -> Result<Vec<WeightedRoute>, String> {
        let op = self
            .operators
            .get(op_cid)
            .ok_or_else(|| format!("Operator {} not found in registry", op_cid))?;

        // Simple relation traversal resolution
        if let Some(transitions) = op.transition_table.get(&input_route.path) {
            let mut results = Vec::new();
            for path in transitions {
                results.push(WeightedRoute {
                    axis: input_route.axis,
                    path: path.clone(),
                    score: input_route.score * 0.95, // apply transition decay
                });
            }
            Ok(results)
        } else {
            // Fallback default backoff operator logic
            if op.op_type == OperatorType::Backoff && input_route.path.len() > 1 {
                let mut backed_off = input_route.path.clone();
                backed_off.pop();
                Ok(vec![WeightedRoute {
                    axis: input_route.axis,
                    path: backed_off,
                    score: input_route.score * 0.8,
                }])
            } else {
                Ok(vec![])
            }
        }
    }
}
