//! Direct tests for generalized multi-step reasoning (#955).

use super::durable_memory::*;
use super::multi_step_reasoning::*;
use super::*;

fn fixture_model() -> Model {
    let docs = vec![
        Document {
            id: "reasoning-fixture".into(),
            text: "ada Rome cyra Perth liam cabinet leon basket alpha beta".into(),
        },
        Document {
            id: "reasoning-filler".into(),
            text: "the quick brown fox jumps over the lazy dog and runs through the forest".into(),
        },
    ];
    let mut trainer = Trainer::new(Config::default(), &docs).expect("trainer construction");
    trainer.train_documents(&docs).expect("train documents");
    trainer.compile().expect("model compilation")
}

#[test]
fn native_multi_step_arithmetic_dag_composition() {
    // 3-step DAG: (13 + 4) * 2 - 10 = 24
    let steps_spec = vec![
        ("+", 13, None, 4),    // Step 1: 13 + 4 = 17
        ("*", 0, Some(1), 2),  // Step 2: Step 1 * 2 = 34
        ("-", 0, Some(2), 10), // Step 3: Step 2 - 10 = 24
    ];

    let constraints = vec![
        ConstraintPolicy::Range { min: 0, max: 100 },
        ConstraintPolicy::NonNegative,
    ];

    let chain = MultiStepReasoningEngine::execute_arithmetic_dag(
        "dag-arithmetic-3step",
        &steps_spec,
        &constraints,
    )
    .expect("execute arithmetic DAG");

    assert_eq!(chain.steps.len(), 3);
    assert_eq!(chain.final_result, "24");
    assert!(chain.constraints_satisfied);

    // Verify intermediate states
    assert_eq!(chain.steps[0].intermediate_state, 17);
    assert_eq!(chain.steps[0].dependencies, Vec::<u64>::new());

    assert_eq!(chain.steps[1].intermediate_state, 34);
    assert_eq!(chain.steps[1].dependencies, vec![1]);

    assert_eq!(chain.steps[2].intermediate_state, 24);
    assert_eq!(chain.steps[2].dependencies, vec![2]);
}

#[test]
fn native_multi_step_relational_transitive_inference() {
    let model = fixture_model();
    let scope = IdentityScope::new("user1", "project1", "session1").unwrap();
    let mut session = DurableSession::new(&model, scope, Control::Full).unwrap();

    // Fact 1: alpha -> beta
    session.assert_fact("alpha", "beta").unwrap();
    // Fact 2: beta -> Perth
    session.assert_fact("beta", "Perth").unwrap();

    // Transitive deduction: alpha -> Perth via beta
    let chain = MultiStepReasoningEngine::execute_transitive_relation(
        "relational-transitive-chain",
        &session,
        "alpha",
    )
    .expect("execute transitive relation");

    assert_eq!(chain.steps.len(), 3);
    assert_eq!(chain.final_result, "Perth");
    assert!(chain.constraints_satisfied);

    match &chain.steps[2].kind {
        ReasoningStepKind::TransitiveDeduction {
            source,
            via,
            target,
        } => {
            assert_eq!(source, "alpha");
            assert_eq!(via, "beta");
            assert_eq!(target, "Perth");
        }
        other => panic!("expected TransitiveDeduction step, got {:?}", other),
    }
}

#[test]
fn native_multi_step_counterfactual_intermediate_mutation() {
    // Baseline: (13 + 4) * 2 - 10 = 24
    let steps_spec = vec![
        ("+", 13, None, 4),
        ("*", 0, Some(1), 2),
        ("-", 0, Some(2), 10),
    ];

    let chain =
        MultiStepReasoningEngine::execute_arithmetic_dag("dag-baseline", &steps_spec, &[]).unwrap();
    assert_eq!(chain.final_result, "24");

    // Counterfactual intervention: mutate Step 1 from 17 to 20
    // Expected recomputation: (20 * 2) - 10 = 40 - 10 = 30
    let intervened = MultiStepReasoningEngine::intervene_intermediate(&chain, 1, 20)
        .expect("intervene intermediate");

    assert_eq!(intervened.steps[0].intermediate_state, 20);
    assert_eq!(intervened.steps[1].intermediate_state, 40);
    assert_eq!(intervened.steps[2].intermediate_state, 30);
    assert_eq!(intervened.final_result, "30");

    // Causal sensitivity confirmed: output changes deterministically from 24 -> 30
    assert_ne!(intervened.final_result, chain.final_result);
}

#[test]
fn native_multi_step_intermediate_ablation() {
    let steps_spec = vec![
        ("+", 13, None, 4),
        ("*", 0, Some(1), 2),
        ("-", 0, Some(2), 10),
    ];

    let chain =
        MultiStepReasoningEngine::execute_arithmetic_dag("dag-ablation-test", &steps_spec, &[])
            .unwrap();

    // Ablating an essential intermediate dependency must fail
    let ablate_step1 = MultiStepReasoningEngine::ablate_intermediate(&chain, 1);
    assert!(ablate_step1.is_err());
    assert!(ablate_step1
        .unwrap_err()
        .0
        .contains("essential causal dependency"));

    // Ablating the final leaf step has no downstream dependents
    let ablate_step3 = MultiStepReasoningEngine::ablate_intermediate(&chain, 3);
    assert!(ablate_step3.is_ok());
}

#[test]
fn native_multi_step_constraint_preservation() {
    // Test 1: Constraint violation in intermediate step
    let steps_spec = vec![
        ("+", 10, None, 5),    // Step 1: 15
        ("*", 0, Some(1), 3),  // Step 2: 45 (violates max 30)
        ("-", 0, Some(2), 20), // Step 3: 25
    ];

    let strict_range = vec![ConstraintPolicy::Range { min: 0, max: 30 }];
    let chain = MultiStepReasoningEngine::execute_arithmetic_dag(
        "dag-range-violation",
        &steps_spec,
        &strict_range,
    )
    .unwrap();

    // Step 2 reached 45, violating the constraint
    assert!(!chain.constraints_satisfied);

    // Test 2: Non-negative constraint violation
    let underflow_spec = vec![
        ("-", 5, None, 12), // Step 1: -7
    ];
    let non_neg = vec![ConstraintPolicy::NonNegative];
    let underflow_chain = MultiStepReasoningEngine::execute_arithmetic_dag(
        "dag-underflow",
        &underflow_spec,
        &non_neg,
    )
    .unwrap();

    assert!(!underflow_chain.constraints_satisfied);
}

#[test]
fn native_multi_step_chained_rust_execution() {
    let steps_spec = vec![
        ("+", 13, None, 4),
        ("*", 0, Some(1), 2),
        ("-", 0, Some(2), 10),
    ];

    let chain =
        MultiStepReasoningEngine::execute_arithmetic_dag("dag-rust-gen", &steps_spec, &[]).unwrap();

    let rust_code = MultiStepReasoningEngine::generate_rust_code(&chain);

    let temp_dir =
        std::env::temp_dir().join(format!("uor_r4_reasoning_rust_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let src_path = temp_dir.join("main.rs");
    let bin_path = temp_dir.join("main_bin");

    std::fs::write(&src_path, rust_code).unwrap();

    let compile_status = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg(&src_path)
        .arg("-o")
        .arg(&bin_path)
        .status()
        .expect("compile generated rust");

    assert!(
        compile_status.success(),
        "Generated Rust code must compile cleanly"
    );

    let run_status = std::process::Command::new(&bin_path)
        .status()
        .expect("run generated rust binary");

    assert!(
        run_status.success(),
        "Generated Rust binary must execute successfully"
    );

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn native_multi_step_novel_operand_transfer() {
    // Novel operands on identical DAG structure: (100 + 25) * 3 - 75 = 300
    let novel_steps = vec![
        ("+", 100, None, 25),
        ("*", 0, Some(1), 3),
        ("-", 0, Some(2), 75),
    ];

    let chain = MultiStepReasoningEngine::execute_arithmetic_dag(
        "dag-novel-operands",
        &novel_steps,
        &[ConstraintPolicy::NonNegative],
    )
    .expect("execute novel operand DAG");

    assert_eq!(chain.steps.len(), 3);
    assert_eq!(chain.steps[0].intermediate_state, 125);
    assert_eq!(chain.steps[1].intermediate_state, 375);
    assert_eq!(chain.steps[2].intermediate_state, 300);
    assert_eq!(chain.final_result, "300");
    assert!(chain.constraints_satisfied);
}
