//! Direct tests for grounded correctness, conflict handling, and abstention (#954).

use super::durable_memory::*;
use super::groundedness::*;
use super::*;

fn fixture_model() -> Model {
    let docs = vec![
        Document {
            id: "groundedness-fixture".into(),
            text: "ada Rome cyra Perth liam cabinet leon basket".into(),
        },
        Document {
            id: "groundedness-filler".into(),
            text: "the quick brown fox jumps over the lazy dog and runs through the forest".into(),
        },
    ];
    let mut trainer = Trainer::new(Config::default(), &docs).expect("trainer construction");
    trainer.train_documents(&docs).expect("train documents");
    trainer.compile().expect("model compilation")
}

#[test]
fn native_groundedness_supported_query_with_provenance() {
    let model = fixture_model();
    let scope = IdentityScope::new("user1", "project1", "session1").unwrap();
    let mut session = DurableSession::new(&model, scope, Control::Full).unwrap();

    let id = session.assert_fact("ada", "Rome").unwrap();
    let outcome = GroundednessEvaluator::evaluate_query(&mut session, &model, "Where is ada?");

    match outcome {
        GroundedOutcome::Answer(ans) => {
            assert_eq!(ans.text, "Rome");
            assert_eq!(
                ans.provenance,
                GroundedProvenance::DurableRelation {
                    id,
                    owner: "ada".into(),
                    value: "Rome".into(),
                }
            );
            assert!(ans.confidence_margin > 0);
        }
        other => panic!("expected Answer outcome, got {:?}", other),
    }
}

#[test]
fn native_groundedness_conflicting_fact_surfacing_and_resolution() {
    let model = fixture_model();
    let scope = IdentityScope::new("user1", "project1", "session1").unwrap();
    let mut session = DurableSession::new(&model, scope, Control::Full).unwrap();

    // Turn 1: Assert fact liam -> cabinet
    let _id1 = session.assert_fact("liam", "cabinet").unwrap();

    // Turn 2: Unannounced contradictory assertion liam -> garden (creates conflict)
    let id2 = session.assert_fact("liam", "garden").unwrap();
    assert!(session.is_fact_in_conflict("liam"));

    // Turn 3: Querying conflicting fact surfaces conflict outcome
    let outcome = GroundednessEvaluator::evaluate_query(&mut session, &model, "Where is liam?");
    match outcome {
        GroundedOutcome::Conflict(c) => {
            assert_eq!(c.entity, "liam");
            assert_eq!(c.existing_value, "cabinet");
            assert_eq!(c.conflicting_value, "garden");
            assert_eq!(c.relation_id, id2);
            assert_eq!(c.status, ConflictStatus::PendingRevision);
        }
        other => panic!("expected Conflict outcome, got {:?}", other),
    }

    // Turn 4: Explicit revision resolves conflict
    let _id3 = session.revise_fact("liam", "garden").unwrap();
    assert!(!session.is_fact_in_conflict("liam"));

    // Turn 5: Querying after revision returns grounded answer
    let resolved_outcome =
        GroundednessEvaluator::evaluate_query(&mut session, &model, "Where is liam?");
    match resolved_outcome {
        GroundedOutcome::Answer(ans) => {
            assert_eq!(ans.text, "garden");
        }
        other => panic!("expected Answer outcome after revision, got {:?}", other),
    }
}

#[test]
fn native_groundedness_answerable_local_session_fact() {
    let model = fixture_model();
    let scope = IdentityScope::new("user1", "project1", "session1").unwrap();
    let mut session = DurableSession::new(&model, scope, Control::Full).unwrap();

    session.assert_fact("cyra", "Perth").unwrap();

    // Query directly
    let outcome1 = GroundednessEvaluator::evaluate_query(&mut session, &model, "Where is cyra?");
    assert!(outcome1.is_answer());

    // Query via antecedent pronoun
    let outcome2 = GroundednessEvaluator::evaluate_query(&mut session, &model, "Where is she?");
    match outcome2 {
        GroundedOutcome::Answer(ans) => {
            assert_eq!(ans.text, "Perth");
        }
        other => panic!("expected Answer for pronoun resolution, got {:?}", other),
    }
}

#[test]
fn native_groundedness_unsupported_query_calibrated_abstention() {
    let model = fixture_model();
    let scope = IdentityScope::new("user1", "project1", "session1").unwrap();
    let mut session = DurableSession::new(&model, scope, Control::Full).unwrap();

    // Query out-of-scope entity without support
    let outcome = GroundednessEvaluator::evaluate_query(&mut session, &model, "Where is zephyr?");
    match outcome {
        GroundedOutcome::Abstain(abs) => {
            assert_eq!(abs.reason, AbstentionReason::NoAdmissibleSource);
            assert!(abs.detail.contains("zephyr"));
        }
        other => panic!("expected Abstain outcome, got {:?}", other),
    }
}

#[test]
fn native_groundedness_distinct_refusal_modes() {
    let model = fixture_model();
    let scope = IdentityScope::new("user1", "project1", "session1").unwrap();
    let mut session = DurableSession::new(&model, scope, Control::Full).unwrap();

    // Mode 1: NoOperation due to arithmetic overflow
    let overflow_outcome =
        GroundednessEvaluator::evaluate_query(&mut session, &model, "9223372036854775807 + 1 =");
    match overflow_outcome {
        GroundedOutcome::Abstain(abs) => {
            assert_eq!(abs.reason, AbstentionReason::NoOperationPrecondition);
            assert!(abs.detail.contains("overflow"));
        }
        other => panic!("expected NoOperationPrecondition, got {:?}", other),
    }

    // Mode 2: NoOperation due to invalid operands
    let invalid_num_outcome =
        GroundednessEvaluator::evaluate_query(&mut session, &model, "abc + def =");
    match invalid_num_outcome {
        GroundedOutcome::Abstain(abs) => {
            assert_eq!(abs.reason, AbstentionReason::NoOperationPrecondition);
        }
        other => panic!("expected NoOperationPrecondition, got {:?}", other),
    }

    // Mode 3: NoAdmissibleSource for unsupported entity
    let no_source_outcome =
        GroundednessEvaluator::evaluate_query(&mut session, &model, "Where is Atlantis?");
    match no_source_outcome {
        GroundedOutcome::Abstain(abs) => {
            assert_eq!(abs.reason, AbstentionReason::NoAdmissibleSource);
        }
        other => panic!("expected NoAdmissibleSource, got {:?}", other),
    }

    // Mode 4: Clarification for ambiguous query
    let clarify_outcome =
        GroundednessEvaluator::evaluate_query(&mut session, &model, "Where is ambiguous?");
    assert!(clarify_outcome.is_clarify());
}

#[test]
fn native_groundedness_causal_source_intervention() {
    let model = fixture_model();
    let scope = IdentityScope::new("user1", "project1", "session1").unwrap();
    let mut session = DurableSession::new(&model, scope, Control::Full).unwrap();

    session.assert_fact("leon", "basket").unwrap();

    // Baseline: grounded query succeeds
    let baseline = GroundednessEvaluator::evaluate_query(&mut session, &model, "Where is leon?");
    assert!(baseline.is_answer());

    // Causal Intervention: actively delete/forget the supporting fact
    let forgotten = session.forget(Some("leon")).unwrap();
    assert_eq!(forgotten, 1);

    // After intervention: identical query must now abstain
    let post_intervention =
        GroundednessEvaluator::evaluate_query(&mut session, &model, "Where is leon?");
    match post_intervention {
        GroundedOutcome::Abstain(abs) => {
            assert_eq!(abs.reason, AbstentionReason::NoAdmissibleSource);
        }
        other => panic!("expected Abstain after causal removal, got {:?}", other),
    }
}

#[test]
fn native_groundedness_dual_denominator_metrics() {
    let model = fixture_model();
    let scope = IdentityScope::new("user1", "project1", "session1").unwrap();
    let mut session = DurableSession::new(&model, scope, Control::Full).unwrap();

    // Setup state
    session.assert_fact("ada", "Rome").unwrap();
    session.assert_fact("liam", "cabinet").unwrap();
    session.assert_fact("liam", "garden").unwrap(); // conflict
    session.assert_fact("cyra", "Perth").unwrap();

    // 4-Probe Benchmark Population
    let cases = vec![
        GroundedCase {
            id: "probe-1-supported".into(),
            query: "Where is ada?".into(),
            expected_kind: "supported".into(),
            expected_value: Some("Rome".into()),
            expected_entity: None,
        },
        GroundedCase {
            id: "probe-2-conflicting".into(),
            query: "Where is liam?".into(),
            expected_kind: "conflicting".into(),
            expected_value: None,
            expected_entity: Some("liam".into()),
        },
        GroundedCase {
            id: "probe-3-local".into(),
            query: "Where is cyra?".into(),
            expected_kind: "local".into(),
            expected_value: Some("Perth".into()),
            expected_entity: None,
        },
        GroundedCase {
            id: "probe-4-unsupported".into(),
            query: "Where is zephyr?".into(),
            expected_kind: "unsupported".into(),
            expected_value: None,
            expected_entity: None,
        },
    ];

    let (results, report) = GroundednessEvaluator::evaluate_cases(&mut session, &model, &cases);

    assert_eq!(results.len(), 4);
    assert_eq!(report.total_cases, 4);
    assert_eq!(report.total_correct, 4);
    assert_eq!(report.total_answered, 2); // ada and cyra
    assert_eq!(report.total_abstained, 1); // zephyr
    assert_eq!(report.total_conflicts_surfaced, 1); // liam

    // Whole-population accuracy: 4 / 4 = 1.0
    assert_eq!(report.whole_population_accuracy, 1.0);

    // Answered-conditional accuracy: 2 / 2 = 1.0
    assert_eq!(report.answered_conditional_accuracy, 1.0);

    // Coverage: 2 / 4 = 0.5
    assert_eq!(report.coverage, 0.5);

    // Abstention accuracy: 1 / 1 = 1.0
    assert_eq!(report.abstention_accuracy, 1.0);

    // Conflict detection accuracy: 1 / 1 = 1.0
    assert_eq!(report.conflict_detection_accuracy, 1.0);
}
