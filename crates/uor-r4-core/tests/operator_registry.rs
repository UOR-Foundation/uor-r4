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
