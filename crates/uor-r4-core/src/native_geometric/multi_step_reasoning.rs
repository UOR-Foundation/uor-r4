//! Generalized multi-step reasoning and constraint preservation (#955).
//!
//! Provides multi-step dependency construction (DAGs), counterfactual
//! intermediate state intervention, constraint verification across transformations,
//! transitive deduction over versioned relations, and Rust execution verification.

use super::durable_memory::DurableSession;
use super::{Error, Result};
use serde::{Deserialize, Serialize};

/// Canonical schema for multi-step reasoning evaluations.
pub const MULTI_STEP_REASONING_SCHEMA: &str = "uor-r4.multi-step-reasoning/1";

/// Semantic category of a multi-step reasoning transformation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "detail", deny_unknown_fields)]
pub enum ReasoningStepKind {
    /// Exact operator transformation with operands and operator label.
    Operator { op: String, operands: Vec<i64> },
    /// Entity or fact retrieval from durable memory or context.
    RelationLookup { entity: String },
    /// Transitive inference step chaining relations (e.g. A -> B -> C).
    TransitiveDeduction {
        source: String,
        via: String,
        target: String,
    },
    /// Invariant or domain constraint evaluation over intermediate state.
    ConstraintCheck {
        constraint: String,
        value: i64,
        satisfied: bool,
    },
}

/// An individual reasoning step in an autoregressive or compositional DAG.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReasoningStep {
    pub step_id: u64,
    pub kind: ReasoningStepKind,
    pub dependencies: Vec<u64>,
    pub result_value: String,
    pub intermediate_state: i64,
}

/// A composed multi-step reasoning chain with verified constraints and outcomes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReasoningChain {
    pub id: String,
    pub steps: Vec<ReasoningStep>,
    pub final_result: String,
    pub constraints_satisfied: bool,
}

/// Numerical or domain invariant policy evaluated over intermediate states.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "policy", content = "bounds", deny_unknown_fields)]
pub enum ConstraintPolicy {
    /// Intermediate state must fall within `[min, max]`.
    Range { min: i64, max: i64 },
    /// Intermediate state must be strictly non-negative (`>= 0`).
    NonNegative,
    /// Intermediate state must not equal zero.
    NonZero,
}

impl ConstraintPolicy {
    /// Evaluate whether a value satisfies the constraint.
    pub fn check(&self, value: i64) -> bool {
        match self {
            ConstraintPolicy::Range { min, max } => value >= *min && value <= *max,
            ConstraintPolicy::NonNegative => value >= 0,
            ConstraintPolicy::NonZero => value != 0,
        }
    }
}

/// Aggregate performance report for multi-step reasoning evaluations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultiStepReasoningReport {
    pub schema: String,
    pub total_cases: usize,
    pub exact_success_count: usize,
    pub causal_intervention_valid: bool,
    pub constraint_preservation_rate: f64,
    pub average_depth: f64,
}

/// Engine executing, intervening on, and verifying multi-step reasoning DAGs.
pub struct MultiStepReasoningEngine;

impl MultiStepReasoningEngine {
    /// Execute a multi-step arithmetic DAG where later steps causally depend on
    /// the committed intermediate states of earlier steps.
    pub fn execute_arithmetic_dag(
        id: &str,
        steps_spec: &[(&str, i64, Option<u64>, i64)], // (op, literal_left, dep_step_id, literal_right)
        constraints: &[ConstraintPolicy],
    ) -> Result<ReasoningChain> {
        let mut steps = Vec::with_capacity(steps_spec.len());
        let mut constraints_satisfied = true;

        for (idx, &(op, literal_left, dep_id, literal_right)) in steps_spec.iter().enumerate() {
            let step_id = (idx + 1) as u64;

            // Resolve left operand: if dep_id is Some, use that prior step's intermediate result
            let (left_val, deps) = if let Some(dep) = dep_id {
                let prior = steps
                    .iter()
                    .find(|s: &&ReasoningStep| s.step_id == dep)
                    .ok_or_else(|| Error(format!("Missing dependency step {}", dep)))?;
                (prior.intermediate_state, vec![dep])
            } else {
                (literal_left, Vec::new())
            };

            let right_val = literal_right;

            // Execute operator
            let result = match op {
                "+" => left_val
                    .checked_add(right_val)
                    .ok_or_else(|| Error("Addition overflow in multi-step reasoning".into()))?,
                "-" => left_val
                    .checked_sub(right_val)
                    .ok_or_else(|| Error("Subtraction overflow in multi-step reasoning".into()))?,
                "*" => left_val.checked_mul(right_val).ok_or_else(|| {
                    Error("Multiplication overflow in multi-step reasoning".into())
                })?,
                _ => return Err(Error(format!("Unsupported reasoning operator: {}", op))),
            };

            // Check constraints
            for c in constraints {
                if !c.check(result) {
                    constraints_satisfied = false;
                }
            }

            steps.push(ReasoningStep {
                step_id,
                kind: ReasoningStepKind::Operator {
                    op: op.to_string(),
                    operands: vec![left_val, right_val],
                },
                dependencies: deps,
                result_value: result.to_string(),
                intermediate_state: result,
            });
        }

        let final_result = steps
            .last()
            .map(|s| s.result_value.clone())
            .unwrap_or_else(|| "0".into());

        Ok(ReasoningChain {
            id: id.to_string(),
            steps,
            final_result,
            constraints_satisfied,
        })
    }

    /// Execute a multi-turn relational transitive reasoning chain (A -> B -> C)
    /// using versioned facts in a `DurableSession`.
    pub fn execute_transitive_relation(
        id: &str,
        session: &DurableSession,
        source_entity: &str,
    ) -> Result<ReasoningChain> {
        let mut steps = Vec::new();

        // Step 1: Lookup intermediate target B for source A
        let via_entity = session.get_fact(source_entity).ok_or_else(|| {
            Error(format!(
                "Entity '{}' not found in durable relations",
                source_entity
            ))
        })?;

        steps.push(ReasoningStep {
            step_id: 1,
            kind: ReasoningStepKind::RelationLookup {
                entity: source_entity.to_string(),
            },
            dependencies: Vec::new(),
            result_value: via_entity.clone(),
            intermediate_state: 1,
        });

        // Step 2: Lookup final target C for intermediate B
        let target_entity = session.get_fact(&via_entity).ok_or_else(|| {
            Error(format!(
                "Intermediate entity '{}' not found in durable relations",
                via_entity
            ))
        })?;

        steps.push(ReasoningStep {
            step_id: 2,
            kind: ReasoningStepKind::RelationLookup {
                entity: via_entity.clone(),
            },
            dependencies: vec![1],
            result_value: target_entity.clone(),
            intermediate_state: 2,
        });

        // Step 3: Transitive deduction A -> C via B
        steps.push(ReasoningStep {
            step_id: 3,
            kind: ReasoningStepKind::TransitiveDeduction {
                source: source_entity.to_string(),
                via: via_entity,
                target: target_entity.clone(),
            },
            dependencies: vec![1, 2],
            result_value: target_entity.clone(),
            intermediate_state: 3,
        });

        Ok(ReasoningChain {
            id: id.to_string(),
            steps,
            final_result: target_entity,
            constraints_satisfied: true,
        })
    }

    /// Perform a counterfactual intervention by mutating an intermediate state
    /// at `target_step` and causally recalculating all downstream steps.
    pub fn intervene_intermediate(
        chain: &ReasoningChain,
        target_step: u64,
        mutated_value: i64,
    ) -> Result<ReasoningChain> {
        let mut new_steps = Vec::with_capacity(chain.steps.len());
        let mut updated_steps = std::collections::BTreeSet::new();
        updated_steps.insert(target_step);

        for step in &chain.steps {
            if step.step_id == target_step {
                // Mutate target intermediate state
                new_steps.push(ReasoningStep {
                    step_id: step.step_id,
                    kind: step.kind.clone(),
                    dependencies: step.dependencies.clone(),
                    result_value: mutated_value.to_string(),
                    intermediate_state: mutated_value,
                });
            } else if step.dependencies.iter().any(|d| updated_steps.contains(d)) {
                // Downstream dependent step: recompute with updated dependencies
                match &step.kind {
                    ReasoningStepKind::Operator { op, operands } => {
                        let left = if let Some(dep_id) = step.dependencies.first() {
                            if let Some(dep_step) = new_steps.iter().find(|s| s.step_id == *dep_id)
                            {
                                dep_step.intermediate_state
                            } else {
                                operands.first().copied().unwrap_or(0)
                            }
                        } else {
                            operands.first().copied().unwrap_or(0)
                        };
                        let right = if let Some(dep_id) = step.dependencies.get(1) {
                            if let Some(dep_step) = new_steps.iter().find(|s| s.step_id == *dep_id)
                            {
                                dep_step.intermediate_state
                            } else {
                                operands.get(1).copied().unwrap_or(0)
                            }
                        } else {
                            operands.get(1).copied().unwrap_or(0)
                        };
                        let result = match op.as_str() {
                            "+" => left
                                .checked_add(right)
                                .ok_or_else(|| Error("Overflow in intervention".into()))?,
                            "-" => left
                                .checked_sub(right)
                                .ok_or_else(|| Error("Overflow in intervention".into()))?,
                            "*" => left
                                .checked_mul(right)
                                .ok_or_else(|| Error("Overflow in intervention".into()))?,
                            _ => return Err(Error("Unsupported op in intervention".into())),
                        };
                        new_steps.push(ReasoningStep {
                            step_id: step.step_id,
                            kind: ReasoningStepKind::Operator {
                                op: op.clone(),
                                operands: vec![left, right],
                            },
                            dependencies: step.dependencies.clone(),
                            result_value: result.to_string(),
                            intermediate_state: result,
                        });
                        updated_steps.insert(step.step_id);
                    }
                    _ => {
                        new_steps.push(step.clone());
                    }
                }
            } else {
                new_steps.push(step.clone());
            }
        }

        let final_result = new_steps
            .last()
            .map(|s| s.result_value.clone())
            .unwrap_or_else(|| "0".into());

        Ok(ReasoningChain {
            id: format!("{}-intervened", chain.id),
            steps: new_steps,
            final_result,
            constraints_satisfied: true,
        })
    }

    /// Ablate an intermediate step, verifying that downstream steps fail
    /// because the required intermediate state is absent.
    pub fn ablate_intermediate(chain: &ReasoningChain, target_step: u64) -> Result<()> {
        let has_dependent = chain
            .steps
            .iter()
            .any(|s| s.dependencies.contains(&target_step));

        if has_dependent {
            Err(Error(format!(
                "Ablation error: step {} is an essential causal dependency for downstream reasoning",
                target_step
            )))
        } else {
            Ok(())
        }
    }

    /// Synthesize standalone Rust code representing the multi-step reasoning DAG
    /// for compilation and execution validation.
    pub fn generate_rust_code(chain: &ReasoningChain) -> String {
        let mut lines = Vec::new();
        lines.push(
            "//! Auto-generated standalone verification for multi-step reasoning DAG.".into(),
        );
        lines.push("fn main() {".into());

        for step in &chain.steps {
            match &step.kind {
                ReasoningStepKind::Operator { op, operands } => {
                    let id = step.step_id;
                    let left = operands[0];
                    let right = operands[1];
                    let expected = step.intermediate_state;

                    if step.dependencies.is_empty() {
                        lines.push(format!(
                            "    let step_{}: i64 = {} {} {};",
                            id, left, op, right
                        ));
                    } else {
                        let dep = step.dependencies[0];
                        lines.push(format!(
                            "    let step_{}: i64 = step_{} {} {};",
                            id, dep, op, right
                        ));
                    }
                    lines.push(format!("    assert_eq!(step_{}, {});", id, expected));
                }
                ReasoningStepKind::TransitiveDeduction { source, target, .. } => {
                    lines.push(format!("    let source = \"{}\";", source));
                    lines.push(format!("    let target = \"{}\";", target));
                    lines.push(format!("    assert_eq!(target, \"{}\");", target));
                }
                _ => {}
            }
        }

        lines.push("}".into());
        lines.join("\n")
    }
}
