//! Explicit fact lookup and integer calculator utilities.
//! These deterministic rules do not measure or implement learned groundedness,
//! natural-language understanding, or calibrated uncertainty in the model.

use super::durable_memory::DurableSession;
use super::Model;
use serde::{Deserialize, Serialize};

/// Canonical schema for groundedness evaluations.
pub const GROUNDEDNESS_SCHEMA: &str = "uor-r4.groundedness-evaluation/1";

/// Exact causal provenance attributing a generated answer to its ground truth source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "detail", deny_unknown_fields)]
pub enum GroundedProvenance {
    /// Supported by an exact versioned relation in durable memory.
    DurableRelation {
        id: u64,
        owner: String,
        value: String,
    },
    /// Supported by a verified text span in the context window.
    ContextSpan {
        start: usize,
        end: usize,
        text: String,
    },
    /// Derived from an exact typed arithmetic or geometric operator.
    Computed {
        operator: String,
        operands: Vec<i64>,
        result: i64,
    },
    /// Direct lexical emission grounded in verified vocabulary.
    DirectLexical { token: String },
}

/// A grounded answer accompanied by its exact source provenance and confidence margin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundedAnswer {
    pub text: String,
    pub provenance: GroundedProvenance,
    pub confidence_margin: u32,
}

/// Status of an unannounced or ongoing contradictory assertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictStatus {
    PendingRevision,
    SurfacedInDialogue,
}

/// A detected contradiction or conflict against established ground truth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundedConflict {
    pub entity: String,
    pub existing_value: String,
    pub conflicting_value: String,
    pub relation_id: u64,
    pub status: ConflictStatus,
}

/// A request for conversational clarification when evidence is ambiguous or split.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundedClarification {
    pub ambiguous_term: String,
    pub candidate_entities: Vec<String>,
    pub prompt: String,
}

/// Categorized refusal reason preserving architectural distinctions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AbstentionReason {
    /// No admissible source or fact exists in context or relation memory.
    NoAdmissibleSource,
    /// Lexical candidate copy withheld because the role/word reader selected `NoRead`.
    NoReadSelected,
    /// Numeric/algorithmic operator rejected because legality or preconditions failed.
    NoOperationPrecondition,
    /// Model candidate confidence margin is below the calibrated decision threshold.
    UncertaintyMarginLow,
}

/// Calibrated abstention explicitly withholding a response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundedAbstention {
    pub reason: AbstentionReason,
    pub detail: String,
}

/// Typed serving outcome distinguishing grounded answers from conflicts,
/// clarifications, and abstentions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", content = "payload", deny_unknown_fields)]
pub enum GroundedOutcome {
    Answer(GroundedAnswer),
    Conflict(GroundedConflict),
    Clarify(GroundedClarification),
    Abstain(GroundedAbstention),
}

impl GroundedOutcome {
    /// Returns true if this outcome is a supported grounded answer.
    pub fn is_answer(&self) -> bool {
        matches!(self, GroundedOutcome::Answer(_))
    }

    /// Returns true if a conflict or contradiction was flagged.
    pub fn is_conflict(&self) -> bool {
        matches!(self, GroundedOutcome::Conflict(_))
    }

    /// Returns true if clarification was requested.
    pub fn is_clarify(&self) -> bool {
        matches!(self, GroundedOutcome::Clarify(_))
    }

    /// Returns true if the model cleanly abstained.
    pub fn is_abstain(&self) -> bool {
        matches!(self, GroundedOutcome::Abstain(_))
    }
}

/// Specification for an individual evaluation case in the groundedness probe suite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundedCase {
    pub id: String,
    pub query: String,
    pub expected_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_entity: Option<String>,
}

/// Result of evaluating a single groundedness case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundedCaseResult {
    pub id: String,
    pub query: String,
    pub expected_kind: String,
    pub outcome: GroundedOutcome,
    pub is_correct: bool,
    pub was_answered: bool,
}

/// Aggregate performance report with whole-population and answered-conditional metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundednessReport {
    pub schema: String,
    pub total_cases: usize,
    pub total_correct: usize,
    pub total_answered: usize,
    pub total_abstained: usize,
    pub total_conflicts_surfaced: usize,
    pub whole_population_accuracy: f64,
    pub answered_conditional_accuracy: f64,
    pub coverage: f64,
    pub abstention_accuracy: f64,
    pub conflict_detection_accuracy: f64,
}

/// Groundedness engine evaluating queries over a `DurableSession`.
pub struct GroundednessEvaluator;

impl GroundednessEvaluator {
    /// Evaluate a query against the active `DurableSession` state, producing
    /// a typed `GroundedOutcome` with causal provenance and calibrated abstention.
    pub fn evaluate_query(
        session: &mut DurableSession,
        _model: &Model,
        query: &str,
    ) -> GroundedOutcome {
        let trimmed = query.trim();

        // 1. Check for arithmetic pattern (e.g. "A + B =", "A - B =", "A * B =")
        if let Some(outcome) = Self::try_evaluate_arithmetic(trimmed) {
            return outcome;
        }

        // 2. Extract query subject/entity
        let candidate_entity = Self::extract_entity_subject(session, trimmed);

        if let Some(entity) = candidate_entity {
            // Check if entity has an active relation record
            if let Some(record) = session.get_fact_record(&entity) {
                if record.conflict {
                    // Contradiction detected: surface conflict and require revision
                    let history = session.get_fact_history(&entity);
                    let previous_val = history
                        .get(1)
                        .map(|r| r.value.clone())
                        .unwrap_or_else(|| "unknown".into());

                    return GroundedOutcome::Conflict(GroundedConflict {
                        entity,
                        existing_value: previous_val,
                        conflicting_value: record.value,
                        relation_id: record.id,
                        status: ConflictStatus::PendingRevision,
                    });
                } else {
                    // Clean supported grounded answer
                    return GroundedOutcome::Answer(GroundedAnswer {
                        text: record.value.clone(),
                        provenance: GroundedProvenance::DurableRelation {
                            id: record.id,
                            owner: record.owner,
                            value: record.value,
                        },
                        confidence_margin: 64,
                    });
                }
            }
        }

        // 3. If no admissible source/fact matches, abstain with calibrated reason
        GroundedOutcome::Abstain(GroundedAbstention {
            reason: AbstentionReason::NoAdmissibleSource,
            detail: format!(
                "No admissible supporting evidence found for query: '{}'",
                query
            ),
        })
    }

    /// Try parsing and evaluating arithmetic queries while respecting integer kernel
    /// boundary checks and `NoOperation` refusal.
    pub(super) fn try_evaluate_arithmetic(query: &str) -> Option<GroundedOutcome> {
        if !query.bytes().any(|b| matches!(b, b'+' | b'-' | b'*')) {
            return None;
        }
        // Exact grammar: signed integer, one operator, signed integer, optional '='.
        // Never strip punctuation or words: "1.5 + 2" must not become "15 + 2".
        let expression = query
            .trim()
            .strip_suffix('=')
            .unwrap_or(query.trim())
            .trim();
        fn integer(input: &str) -> Option<(i64, &str)> {
            let input = input.trim_start();
            let bytes = input.as_bytes();
            let mut end = usize::from(matches!(bytes.first(), Some(b'+') | Some(b'-')));
            let digits = end;
            while bytes.get(end).is_some_and(u8::is_ascii_digit) {
                end += 1;
            }
            if end == digits {
                return None;
            }
            Some((input[..end].parse().ok()?, &input[end..]))
        }
        let parsed = (|| {
            let (left, rest) = integer(expression)?;
            let rest = rest.trim_start();
            let op = match rest.as_bytes().first()? {
                b'+' => "+",
                b'-' => "-",
                b'*' => "*",
                _ => return None,
            };
            let (right, tail) = integer(&rest[1..])?;
            if !tail.trim().is_empty() {
                return None;
            }
            Some((left, op, right))
        })();
        let Some((left, op, right)) = parsed else {
            return Some(GroundedOutcome::Abstain(GroundedAbstention {
                reason: AbstentionReason::NoOperationPrecondition,
                detail: "Expected exactly two signed i64 integers and one +, - or * operator"
                    .into(),
            }));
        };

        // Enforce operator legality bounds
        match op {
            "+" => {
                if let Some(sum) = left.checked_add(right) {
                    Some(GroundedOutcome::Answer(GroundedAnswer {
                        text: sum.to_string(),
                        provenance: GroundedProvenance::Computed {
                            operator: "Add".into(),
                            operands: vec![left, right],
                            result: sum,
                        },
                        confidence_margin: 127,
                    }))
                } else {
                    Some(GroundedOutcome::Abstain(GroundedAbstention {
                        reason: AbstentionReason::NoOperationPrecondition,
                        detail: "Addition overflow rejected by integer kernel".into(),
                    }))
                }
            }
            "-" => {
                if let Some(diff) = left.checked_sub(right) {
                    Some(GroundedOutcome::Answer(GroundedAnswer {
                        text: diff.to_string(),
                        provenance: GroundedProvenance::Computed {
                            operator: "Sub".into(),
                            operands: vec![left, right],
                            result: diff,
                        },
                        confidence_margin: 127,
                    }))
                } else {
                    Some(GroundedOutcome::Abstain(GroundedAbstention {
                        reason: AbstentionReason::NoOperationPrecondition,
                        detail: "Subtraction overflow rejected by integer kernel".into(),
                    }))
                }
            }
            "*" => {
                if let Some(prod) = left.checked_mul(right) {
                    Some(GroundedOutcome::Answer(GroundedAnswer {
                        text: prod.to_string(),
                        provenance: GroundedProvenance::Computed {
                            operator: "Mul".into(),
                            operands: vec![left, right],
                            result: prod,
                        },
                        confidence_margin: 127,
                    }))
                } else {
                    Some(GroundedOutcome::Abstain(GroundedAbstention {
                        reason: AbstentionReason::NoOperationPrecondition,
                        detail: "Multiplication overflow rejected by integer kernel".into(),
                    }))
                }
            }
            _ => None,
        }
    }

    /// Extract entity candidate or pronoun from query.
    fn extract_entity_subject(session: &DurableSession, query: &str) -> Option<String> {
        let words: Vec<&str> = query
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .collect();

        // Check for pronouns first
        for &word in &words {
            if matches!(
                word.to_ascii_lowercase().as_str(),
                "she" | "he" | "it" | "they" | "this" | "that"
            ) {
                if let Some(ref entity) = session.last_entity {
                    return Some(entity.clone());
                }
            }
        }

        // Check if any word matches an owner in active directory
        for &word in &words {
            let lower = word.to_ascii_lowercase();
            if lower == "where"
                || lower == "what"
                || lower == "who"
                || lower == "is"
                || lower == "the"
            {
                continue;
            }
            // Check if active in session relations
            if session.get_fact(&lower).is_some() || session.is_fact_in_conflict(&lower) {
                return Some(lower);
            }
        }

        // Return first non-stopword as candidate entity
        for &word in &words {
            let lower = word.to_ascii_lowercase();
            if lower != "where"
                && lower != "what"
                && lower != "who"
                && lower != "is"
                && lower != "the"
                && lower != "in"
            {
                return Some(lower);
            }
        }

        None
    }

    /// Evaluate a population of cases, calculating dual-denominator metrics.
    pub fn evaluate_cases(
        session: &mut DurableSession,
        model: &Model,
        cases: &[GroundedCase],
    ) -> (Vec<GroundedCaseResult>, GroundednessReport) {
        let mut results = Vec::with_capacity(cases.len());
        let mut total_correct = 0;
        let mut total_answered = 0;
        let mut total_abstained = 0;
        let mut total_conflicts_surfaced = 0;

        let mut unsupported_count = 0;
        let mut correct_abstentions = 0;

        let mut conflicting_count = 0;
        let mut detected_conflicts = 0;

        for case in cases {
            let outcome = Self::evaluate_query(session, model, &case.query);
            let mut is_correct = false;
            let mut was_answered = false;

            match (&case.expected_kind.as_str(), &outcome) {
                (&"supported", GroundedOutcome::Answer(ans)) => {
                    was_answered = true;
                    if let Some(ref expected) = case.expected_value {
                        if &ans.text == expected {
                            is_correct = true;
                            total_correct += 1;
                        }
                    } else {
                        is_correct = true;
                        total_correct += 1;
                    }
                    total_answered += 1;
                }
                (&"conflicting", GroundedOutcome::Conflict(c)) => {
                    conflicting_count += 1;
                    detected_conflicts += 1;
                    total_conflicts_surfaced += 1;
                    if let Some(ref expected) = case.expected_entity {
                        if &c.entity == expected {
                            is_correct = true;
                            total_correct += 1;
                        }
                    } else {
                        is_correct = true;
                        total_correct += 1;
                    }
                }
                (&"local", GroundedOutcome::Answer(ans)) => {
                    was_answered = true;
                    if let Some(ref expected) = case.expected_value {
                        if &ans.text == expected {
                            is_correct = true;
                            total_correct += 1;
                        }
                    } else {
                        is_correct = true;
                        total_correct += 1;
                    }
                    total_answered += 1;
                }
                (&"unsupported", GroundedOutcome::Abstain(_)) => {
                    unsupported_count += 1;
                    correct_abstentions += 1;
                    total_abstained += 1;
                    is_correct = true;
                    total_correct += 1;
                }
                _ => {
                    if case.expected_kind == "unsupported" {
                        unsupported_count += 1;
                    }
                    if case.expected_kind == "conflicting" {
                        conflicting_count += 1;
                    }
                    if outcome.is_answer() {
                        total_answered += 1;
                    }
                    if outcome.is_abstain() {
                        total_abstained += 1;
                    }
                }
            }

            results.push(GroundedCaseResult {
                id: case.id.clone(),
                query: case.query.clone(),
                expected_kind: case.expected_kind.clone(),
                outcome,
                is_correct,
                was_answered,
            });
        }

        let total_cases = cases.len();
        let whole_pop_acc = if total_cases > 0 {
            total_correct as f64 / total_cases as f64
        } else {
            0.0
        };

        let ans_cond_acc = if total_answered > 0 {
            let correct_answered = results
                .iter()
                .filter(|r| r.was_answered && r.is_correct)
                .count();
            correct_answered as f64 / total_answered as f64
        } else {
            0.0
        };

        let coverage = if total_cases > 0 {
            total_answered as f64 / total_cases as f64
        } else {
            0.0
        };

        let abstention_acc = if unsupported_count > 0 {
            correct_abstentions as f64 / unsupported_count as f64
        } else {
            1.0
        };

        let conflict_det_acc = if conflicting_count > 0 {
            detected_conflicts as f64 / conflicting_count as f64
        } else {
            1.0
        };

        let report = GroundednessReport {
            schema: GROUNDEDNESS_SCHEMA.into(),
            total_cases,
            total_correct,
            total_answered,
            total_abstained,
            total_conflicts_surfaced,
            whole_population_accuracy: whole_pop_acc,
            answered_conditional_accuracy: ans_cond_acc,
            coverage,
            abstention_accuracy: abstention_acc,
            conflict_detection_accuracy: conflict_det_acc,
        };

        (results, report)
    }
}
