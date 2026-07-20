use std::collections::HashMap;
use uor_r4_core::semantic::{
    OperatorRegistry, TypedOperator, OperatorType, WeightedRoute
};

#[test]
fn test_operator_registry_and_evaluation() {
    let mut registry = OperatorRegistry::new("blake3:space_1".to_string());

    // Define relation transition: Germany -> Berlin
    let mut transition_table = HashMap::new();
    transition_table.insert(vec![4, 2], vec![vec![9, 8, 7]]); // Germany coords -> Berlin coords

    let op_capital = TypedOperator {
        cid: "blake3:op_capital".to_string(),
        name: "capital_of".to_string(),
        op_type: OperatorType::RelationTraversal,
        input_type: "Country".to_string(),
        output_type: "City".to_string(),
        transition_table,
    };

    registry.register_operator(op_capital);

    // Evaluate input route Germany (score = 1.0)
    let input = WeightedRoute {
        axis: 3,
        path: vec![4, 2],
        score: 1.0,
    };

    let result = registry.evaluate("blake3:op_capital", &input).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].axis, 3);
    assert_eq!(result[0].path, vec![9, 8, 7]);
    assert_eq!(result[0].score, 0.95); // transition decay applied
}

#[test]
fn test_operator_registry_backoff_fallback() {
    let mut registry = OperatorRegistry::new("blake3:space_1".to_string());

    let op_backoff = TypedOperator {
        cid: "blake3:op_backoff".to_string(),
        name: "backoff_operator".to_string(),
        op_type: OperatorType::Backoff,
        input_type: "Any".to_string(),
        output_type: "Any".to_string(),
        transition_table: HashMap::new(),
    };

    registry.register_operator(op_backoff);

    let input = WeightedRoute {
        axis: 1,
        path: vec![10, 20, 30],
        score: 1.0,
    };

    let result = registry.evaluate("blake3:op_backoff", &input).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].axis, 1);
    assert_eq!(result[0].path, vec![10, 20]); // backed off by 1 level
    assert_eq!(result[0].score, 0.8);
}

#[test]
fn test_reasoning_plan_execution_and_budget_enforcement() {
    use uor_r4_core::semantic::{
        ReasoningPlanV1, OperatorRegistry, TypedOperator, OperatorType, WeightedRoute
    };
    use uor_r4_core::semantic::reasoning::{FacetPolicy, BackoffPolicy, EvidencePolicy, Limits};

    let mut registry = OperatorRegistry::new("blake3:space_1".to_string());

    let mut transition_table = HashMap::new();
    transition_table.insert(vec![1, 2], vec![vec![3, 4]]);
    registry.register_operator(TypedOperator {
        cid: "blake3:op1".to_string(),
        name: "op1".to_string(),
        op_type: OperatorType::RelationTraversal,
        input_type: "A".to_string(),
        output_type: "B".to_string(),
        transition_table,
    });

    let plan = ReasoningPlanV1 {
        query_cid: "blake3:query_1".to_string(),
        semantic_space_cid: "blake3:space_1".to_string(),
        clauses: vec![],
        operators: vec!["blake3:op1".to_string()],
        facet_policy: FacetPolicy { priority_order: vec![] },
        backoff_policy: BackoffPolicy { max_backoff_steps: 2, allow_any_facet: true },
        required_evidence: EvidencePolicy { min_evidence_count: 1, require_provenance: false },
        deterministic_limits: Limits {
            max_probes: 10,
            max_operators: 5,
            timeout_ms: 100,
        },
    };

    let start = WeightedRoute {
        axis: 1,
        path: vec![1, 2],
        score: 1.0,
    };

    // 1. Success path
    let witness = plan.execute(&registry, &start).unwrap();
    assert_eq!(witness.operation_census.total_probes, 1);
    assert_eq!(witness.operation_census.total_operator_steps, 1);
    assert_eq!(witness.generated_routes[0].path, vec![3, 4]);

    // 2. Operator budget limit failure
    let plan_limited_op = ReasoningPlanV1 {
        deterministic_limits: Limits {
            max_probes: 10,
            max_operators: 0, // limit is 0
            timeout_ms: 100,
        },
        ..plan.clone()
    };
    let err_op = plan_limited_op.execute(&registry, &start);
    assert!(err_op.is_err());

    // 3. Probing budget limit failure
    let plan_limited_probe = ReasoningPlanV1 {
        deterministic_limits: Limits {
            max_probes: 0, // limit is 0
            max_operators: 5,
            timeout_ms: 100,
        },
        ..plan
    };
    let err_probe = plan_limited_probe.execute(&registry, &start);
    assert!(err_probe.is_err());
}

#[test]
fn test_logical_and_temporal_operators() {
    let mut registry = OperatorRegistry::new("blake3:space_1".to_string());

    // Conjunction
    let mut transitions_conj = HashMap::new();
    transitions_conj.insert(vec![1, 1], vec![vec![1, 1, 1], vec![1, 1, 2]]);
    registry.register_operator(TypedOperator {
        cid: "blake3:op_conj".to_string(),
        name: "conj".to_string(),
        op_type: OperatorType::Conjunction,
        input_type: "Any".to_string(),
        output_type: "Any".to_string(),
        transition_table: transitions_conj,
    });

    // Negation
    let mut transitions_neg = HashMap::new();
    transitions_neg.insert(vec![2, 2], vec![vec![2, 2, 99]]);
    registry.register_operator(TypedOperator {
        cid: "blake3:op_neg".to_string(),
        name: "neg".to_string(),
        op_type: OperatorType::Negation,
        input_type: "Any".to_string(),
        output_type: "Any".to_string(),
        transition_table: transitions_neg,
    });

    // Projection
    registry.register_operator(TypedOperator {
        cid: "blake3:op_proj".to_string(),
        name: "proj".to_string(),
        op_type: OperatorType::Projection,
        input_type: "Any".to_string(),
        output_type: "Any".to_string(),
        transition_table: HashMap::new(),
    });

    // TemporalOrdering
    registry.register_operator(TypedOperator {
        cid: "blake3:op_temp".to_string(),
        name: "temp".to_string(),
        op_type: OperatorType::TemporalOrdering,
        input_type: "Any".to_string(),
        output_type: "Any".to_string(),
        transition_table: HashMap::new(),
    });

    // Assert Conjunction splits paths and applies conjunction decay (0.90)
    let input_conj = WeightedRoute { axis: 1, path: vec![1, 1], score: 1.0 };
    let res_conj = registry.evaluate("blake3:op_conj", &input_conj).unwrap();
    assert_eq!(res_conj.len(), 2);
    assert_eq!(res_conj[0].score, 0.90);

    // Assert Negation returns negative/complement scores (-1.0 multiplier)
    let input_neg = WeightedRoute { axis: 1, path: vec![2, 2], score: 1.0 };
    let res_neg = registry.evaluate("blake3:op_neg", &input_neg).unwrap();
    assert_eq!(res_neg.len(), 1);
    assert_eq!(res_neg[0].score, -1.0);

    // Assert default Projection truncates paths to length 1
    let input_proj = WeightedRoute { axis: 1, path: vec![10, 20, 30], score: 1.0 };
    let res_proj = registry.evaluate("blake3:op_proj", &input_proj).unwrap();
    assert_eq!(res_proj[0].path, vec![10]);
    assert_eq!(res_proj[0].score, 0.90);

    // Assert default TemporalOrdering appends 999
    let input_temp = WeightedRoute { axis: 1, path: vec![100], score: 1.0 };
    let res_temp = registry.evaluate("blake3:op_temp", &input_temp).unwrap();
    assert_eq!(res_temp[0].path, vec![100, 999]);
    assert_eq!(res_temp[0].score, 0.95);
}
