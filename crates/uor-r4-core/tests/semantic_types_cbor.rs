use uor_r4_core::semantic::{
    SemanticRouteReferenceV1, FacetRoute, SemanticSpaceManifestV1, ReasoningPlanV1,
    SemanticInferenceWitnessV1, Constraint, WeightedRoute
};
use uor_r4_core::semantic::reasoning::{FacetPolicy, BackoffPolicy, EvidencePolicy, Limits, RegionProbe, OperatorExecution, CandidateScore, OperationCensus};

#[test]
fn test_semantic_reference_roundtrip() {
    let route = FacetRoute {
        axis: 1,
        path: vec![12, 5, 91],
        confidence_q16: 32768,
        valid_from_epoch: 100,
        evidence_root_cid: "blake3:evidence_hash".to_string(),
    };
    let reference = SemanticRouteReferenceV1 {
        object_cid: "blake3:object_hash".to_string(),
        schema_cid: "blake3:schema_hash".to_string(),
        semantic_space_cid: "blake3:space_hash".to_string(),
        geometry_manifest_cid: "blake3:manifest_hash".to_string(),
        routes: vec![route],
        bindings_cid: "blake3:bindings_hash".to_string(),
        grounding_witness_cid: "blake3:grounding_witness_hash".to_string(),
    };

    let serialized = serde_json::to_string(&reference).unwrap();
    let deserialized: SemanticRouteReferenceV1 = serde_json::from_str(&serialized).unwrap();
    assert_eq!(reference, deserialized);
}

#[test]
fn test_manifest_roundtrip() {
    let manifest = SemanticSpaceManifestV1 {
        space_name: "test_space".to_string(),
        parent_space_cid: None,
        schema_roots: vec!["blake3:schema_root".to_string()],
        axis_definitions: vec!["blake3:axis_def".to_string()],
        codebook_cids: vec!["blake3:codebook".to_string()],
        threshold_cids: vec!["blake3:threshold".to_string()],
        metric_cids: vec!["blake3:metric".to_string()],
        operator_registry_cid: "blake3:operator_registry".to_string(),
        corpus_root_cids: vec!["blake3:corpus_root".to_string()],
        compiler_cid: "blake3:compiler".to_string(),
        quality_certificate_cid: "blake3:quality_cert".to_string(),
        epoch: 42,
    };

    let serialized = serde_json::to_string(&manifest).unwrap();
    let deserialized: SemanticSpaceManifestV1 = serde_json::from_str(&serialized).unwrap();
    assert_eq!(manifest, deserialized);
}

#[test]
fn test_plan_and_witness_roundtrip() {
    let plan = ReasoningPlanV1 {
        query_cid: "blake3:query".to_string(),
        semantic_space_cid: "blake3:space".to_string(),
        clauses: vec![Constraint {
            facet: "type".to_string(),
            path: vec![1, 2],
            required: true,
        }],
        operators: vec!["blake3:operator".to_string()],
        facet_policy: FacetPolicy {
            priority_order: vec!["type".to_string(), "entity".to_string()],
        },
        backoff_policy: BackoffPolicy {
            max_backoff_steps: 3,
            allow_any_facet: false,
        },
        required_evidence: EvidencePolicy {
            min_evidence_count: 5,
            require_provenance: true,
        },
        deterministic_limits: Limits {
            max_probes: 100,
            max_operators: 10,
            timeout_ms: 1000,
        },
    };

    let serialized_plan = serde_json::to_string(&plan).unwrap();
    let deserialized_plan: ReasoningPlanV1 = serde_json::from_str(&serialized_plan).unwrap();
    assert_eq!(plan, deserialized_plan);

    let witness = SemanticInferenceWitnessV1 {
        query_cid: "blake3:query".to_string(),
        plan_cid: "blake3:plan".to_string(),
        semantic_space_cid: "blake3:space".to_string(),
        store_root_cids: vec!["blake3:store_root".to_string()],
        generated_routes: vec![WeightedRoute {
            axis: 1,
            path: vec![1, 2],
            score: 0.95,
        }],
        probed_regions: vec![RegionProbe {
            region_id: "region_1".to_string(),
            depth: 2,
            matched: true,
        }],
        applied_operators: vec![OperatorExecution {
            operator_cid: "blake3:operator".to_string(),
            input_route: vec![1],
            output_routes: vec![vec![1, 2]],
        }],
        evidence_cids: vec!["blake3:evidence".to_string()],
        contradiction_cids: vec![],
        score_components: vec![CandidateScore {
            candidate_cid: "blake3:candidate".to_string(),
            raw_score: 9.5,
            breakdown: "depth=2".to_string(),
        }],
        result_cid: "blake3:result".to_string(),
        operation_census: OperationCensus {
            total_probes: 12,
            total_operator_steps: 1,
            total_joins: 3,
        },
    };

    let serialized_witness = serde_json::to_string(&witness).unwrap();
    let deserialized_witness: SemanticInferenceWitnessV1 = serde_json::from_str(&serialized_witness).unwrap();
    assert_eq!(witness, deserialized_witness);
}
