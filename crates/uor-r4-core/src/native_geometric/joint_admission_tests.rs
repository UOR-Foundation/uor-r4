//! Tests for learned four-operator joint admission, exact legality checking,
//! feature encoding, arbitration, and control bypass.

use super::joint_admission::{self, JointAdmission};
use super::source_routing::SourceRouting;
use super::typed_routing::TypedContext;
use super::value_types::{ValueAction, ValueRecord, ValueState, ValueWork};
use super::*;

#[test]
fn native_joint_admission_overflow_rejection_add_sub_mul() {
    // 1. Boundary checking across all four operators
    let boundaries = [
        i64::MIN,
        i64::MIN + 1,
        -1000,
        -1,
        0,
        1,
        1000,
        i64::MAX - 1,
        i64::MAX,
    ];
    for &a in &boundaries {
        for &b in &boundaries {
            assert_eq!(
                joint_admission::legal(ValueAction::Add, a, b),
                a.checked_add(b).is_some(),
                "Add legality must match checked_add for ({a}, {b})"
            );
            assert_eq!(
                joint_admission::legal(ValueAction::Sub, a, b),
                a.checked_sub(b).is_some(),
                "Sub legality must match checked_sub for ({a}, {b})"
            );
            assert_eq!(
                joint_admission::legal(ValueAction::Mul, a, b),
                a.checked_mul(b).is_some(),
                "Mul legality must match checked_mul for ({a}, {b})"
            );
            assert!(
                joint_admission::legal(ValueAction::Copy, a, b),
                "Copy is always legal regardless of operand values"
            );
        }
    }

    // 2. Specific overflow cases
    assert!(!joint_admission::legal(ValueAction::Add, i64::MAX, 1));
    assert!(!joint_admission::legal(ValueAction::Add, i64::MIN, -1));
    assert!(!joint_admission::legal(ValueAction::Sub, i64::MIN, 1));
    assert!(!joint_admission::legal(ValueAction::Sub, i64::MAX, -1));
    assert!(!joint_admission::legal(ValueAction::Mul, i64::MAX, 2));
    assert!(!joint_admission::legal(ValueAction::Mul, i64::MIN, -1));

    // 3. Valid normal arithmetic cases
    assert!(joint_admission::legal(ValueAction::Add, 13, 4));
    assert!(joint_admission::legal(ValueAction::Sub, 13, 4));
    assert!(joint_admission::legal(ValueAction::Mul, 13, 4));
    assert!(joint_admission::legal(ValueAction::Copy, 13, 4));

    // 4. Verification of work accounting when simulated in runtime selection loop
    let mut work = ValueWork::default();
    for (action, a, b, should_overflow) in [
        (ValueAction::Add, i64::MAX, 1, true),
        (ValueAction::Sub, i64::MIN, 1, true),
        (ValueAction::Mul, i64::MAX, 2, true),
        (ValueAction::Add, 10, 20, false),
        (ValueAction::Sub, 20, 10, false),
        (ValueAction::Mul, 5, 6, false),
        (ValueAction::Copy, 42, 0, false),
    ] {
        work.admission_legality_checks += 1;
        if !joint_admission::legal(action, a, b) {
            work.overflow_rejections += 1;
        }
        assert_eq!(
            !joint_admission::legal(action, a, b),
            should_overflow,
            "Overflow expectation failed for {action:?} with {a}, {b}"
        );
    }
    assert_eq!(work.admission_legality_checks, 7);
    assert_eq!(work.overflow_rejections, 3);
}

#[test]
fn native_joint_admission_feature_encoding_all_actions() {
    let (model, _) = learned_routing_tests::fixture();
    let values = ValueState::new(&model);
    let operands = (
        ValueRecord {
            id: 1,
            value: 13,
            start: 0,
            end: 2,
            derived: false,
            derivation: None,
            ..Default::default()
        },
        ValueRecord {
            id: 2,
            value: 4,
            start: 3,
            end: 4,
            derived: false,
            derivation: None,
            ..Default::default()
        },
    );
    let context = TypedContext {
        literal_component: true,
        addresses: [0; 16],
        depths: None,
        origins: None,
        provenance: None,
    };

    let actions = [
        (ValueAction::Copy, 0u64),
        (ValueAction::Add, 1u64),
        (ValueAction::Sub, 2u64),
        (ValueAction::Mul, 3u64),
    ];

    let mut encodings = Vec::new();
    for (action, expected_code) in actions {
        let mut work = ValueWork::default();
        let (feat, n) =
            joint_admission::features(&values, action, operands, 25, &context, &mut work);
        assert!(n >= 2, "Feature length must include action and margin");
        assert_eq!(feat[n - 2].kind, 6, "Action feature must be kind 6");
        assert_eq!(
            feat[n - 2].a,
            expected_code,
            "Action encoding for {action:?} must be {expected_code}"
        );
        assert_eq!(feat[n - 2].b, 0);

        assert_eq!(feat[n - 1].kind, 7, "Margin feature must be kind 7");
        assert_eq!(
            feat[n - 1].a,
            25,
            "Margin value must match un-clamped input"
        );
        assert_eq!(feat[n - 1].b, 0);

        encodings.push(feat[n - 2].a);
    }

    // Verify all 4 encodings are pairwise distinct
    for i in 0..encodings.len() {
        for j in (i + 1)..encodings.len() {
            assert_ne!(
                encodings[i], encodings[j],
                "Action encodings must be distinct"
            );
        }
    }

    // Verify margin clamping
    let mut work = ValueWork::default();
    let (feat_neg, n_neg) = joint_admission::features(
        &values,
        ValueAction::Add,
        operands,
        -50,
        &context,
        &mut work,
    );
    assert_eq!(
        feat_neg[n_neg - 1].a,
        0,
        "Negative margin must be clamped to 0"
    );

    let (feat_high, n_high) = joint_admission::features(
        &values,
        ValueAction::Add,
        operands,
        500,
        &context,
        &mut work,
    );
    assert_eq!(
        feat_high[n_high - 1].a,
        127,
        "High margin must be clamped to 127"
    );
}

#[test]
fn native_joint_admission_lexical_prose_arbitration() {
    let (model, _) = learned_routing_tests::fixture();
    let values = ValueState::new(&model);
    let operands = (
        ValueRecord {
            id: 1,
            value: 13,
            start: 0,
            end: 2,
            derived: false,
            derivation: None,
            ..Default::default()
        },
        ValueRecord {
            id: 2,
            value: 4,
            start: 3,
            end: 4,
            derived: false,
            derivation: None,
            ..Default::default()
        },
    );

    // 1. Without joint_admission, permits always returns true
    let mut work = ValueWork::default();
    let context_literal = TypedContext {
        literal_component: true,
        addresses: [0; 16],
        depths: None,
        origins: None,
        provenance: None,
    };
    assert!(joint_admission::permits(
        &model,
        &values,
        ValueAction::Add,
        operands,
        10,
        &context_literal,
        Control::Full,
        &mut work
    ));
    assert_eq!(work.admission_decisions, 0);

    // 2. Non-literal context bypasses admission
    let context_non_literal = TypedContext {
        literal_component: false,
        addresses: [0; 16],
        depths: None,
        origins: None,
        provenance: None,
    };
    assert!(joint_admission::permits(
        &model,
        &values,
        ValueAction::Add,
        operands,
        10,
        &context_non_literal,
        Control::Full,
        &mut work
    ));
    assert_eq!(work.admission_decisions, 0);

    // 3. Construct a model with a JointAdmission gate favoring lexical
    let router_lexical_favored = SourceRouting {
        schema: "uor-r4.geometric-source-routing/1".into(),
        parent_artifact: model.artifact_cid.clone(),
        codes: Vec::new(),
        landmarks: vec![[model.geometry.identity; 2]; 2],
        biases: vec![0, 20], // bias[1] (lexical) > bias[0] (numeric)
        ranks: vec![0; 120],
        training: Vec::new(),
        config: SourceRoutingConfig::default(),
    };
    let mut model_lexical = model.clone();
    model_lexical.joint_admission = Some(JointAdmission {
        router: router_lexical_favored,
    });

    let mut work_lex = ValueWork::default();
    let permitted = joint_admission::permits(
        &model_lexical,
        &values,
        ValueAction::Add,
        operands,
        10,
        &context_literal,
        Control::Full,
        &mut work_lex,
    );
    assert!(
        !permitted,
        "Lexical-favored gate must reject numeric proposal"
    );
    assert_eq!(work_lex.admission_decisions, 1);
    assert_eq!(work_lex.admission_rejections, 1);

    // 4. Construct a model with a JointAdmission gate favoring numeric
    let router_numeric_favored = SourceRouting {
        schema: "uor-r4.geometric-source-routing/1".into(),
        parent_artifact: model.artifact_cid.clone(),
        codes: Vec::new(),
        landmarks: vec![[model.geometry.identity; 2]; 2],
        biases: vec![20, 0], // bias[0] (numeric) > bias[1] (lexical)
        ranks: vec![0; 120],
        training: Vec::new(),
        config: SourceRoutingConfig::default(),
    };
    let mut model_numeric = model.clone();
    model_numeric.joint_admission = Some(JointAdmission {
        router: router_numeric_favored,
    });

    let mut work_num = ValueWork::default();
    let permitted = joint_admission::permits(
        &model_numeric,
        &values,
        ValueAction::Add,
        operands,
        10,
        &context_literal,
        Control::Full,
        &mut work_num,
    );
    assert!(
        permitted,
        "Numeric-favored gate must permit numeric proposal"
    );
    assert_eq!(work_num.admission_decisions, 1);
    assert_eq!(work_num.admission_rejections, 0);
}

#[test]
fn native_joint_admission_control_bypass() {
    let (model, _) = learned_routing_tests::fixture();
    let values = ValueState::new(&model);
    let operands = (
        ValueRecord {
            id: 1,
            value: 13,
            start: 0,
            end: 2,
            derived: false,
            derivation: None,
            ..Default::default()
        },
        ValueRecord {
            id: 2,
            value: 4,
            start: 3,
            end: 4,
            derived: false,
            derivation: None,
            ..Default::default()
        },
    );
    let context_literal = TypedContext {
        literal_component: true,
        addresses: [0; 16],
        depths: None,
        origins: None,
        provenance: None,
    };

    let router = SourceRouting {
        schema: "uor-r4.geometric-source-routing/1".into(),
        parent_artifact: model.artifact_cid.clone(),
        codes: Vec::new(),
        landmarks: vec![[model.geometry.identity; 2]; 2],
        biases: vec![0, 20],
        ranks: vec![0; 120],
        training: Vec::new(),
        config: SourceRoutingConfig::default(),
    };
    let mut model_gated = model.clone();
    model_gated.joint_admission = Some(JointAdmission { router });

    let mut work = ValueWork::default();
    let permitted = joint_admission::permits(
        &model_gated,
        &values,
        ValueAction::Add,
        operands,
        10,
        &context_literal,
        Control::JointAdmissionDisabled,
        &mut work,
    );
    assert!(
        permitted,
        "Control::JointAdmissionDisabled must bypass admission and return true"
    );
    assert_eq!(
        work.admission_decisions, 0,
        "Bypassed admission must not increment decisions counter"
    );
    assert_eq!(
        work.admission_rejections, 0,
        "Bypassed admission must not increment rejections counter"
    );
}

#[test]
fn native_joint_admission_training_validation_and_heterogeneous_curriculum() {
    let (model, _) = learned_routing_tests::fixture();

    let docs = [ValueExample {
        id: "ex-1".into(),
        prompt: "test".into(),
        response: "123".into(),
    }];
    let config = SourceRoutingConfig::default();

    // Missing literal/source parents
    let err = model
        .fit_joint_admission(&docs, config.clone())
        .unwrap_err();
    assert!(err.to_string().contains("literal/source parent"));

    // Role context only mode rejected for admission
    let role_config = SourceRoutingConfig {
        role_context_only: true,
        ..SourceRoutingConfig::default()
    };
    let err = model.fit_joint_admission(&docs, role_config).unwrap_err();
    assert!(err
        .to_string()
        .contains("role_context_only is not an admission feature mode"));

    // Empty docs rejected
    let err = model.fit_joint_admission(&[], config.clone()).unwrap_err();
    assert!(err.to_string().contains("1..1024 examples"));

    // Empty example ID rejected
    let empty_id = [ValueExample {
        id: "  ".into(),
        prompt: "p1".into(),
        response: "1".into(),
    }];
    let mut mock = model.clone();
    let mock_router = SourceRouting {
        schema: "uor-r4.geometric-source-routing/1".into(),
        parent_artifact: model.artifact_cid.clone(),
        codes: Vec::new(),
        landmarks: vec![[model.geometry.identity; 2]; 2],
        biases: vec![0; 2],
        ranks: vec![0; 120],
        training: Vec::new(),
        config: SourceRoutingConfig::default(),
    };
    mock.source_routing = Some(mock_router.clone());
    mock.typed_literals = Some(super::typed_routing::TypedRouting {
        router: mock_router,
        dictionary: Vec::new(),
        fold_ascii_case: false,
        canonical_copy_aliases: false,
        local_query: false,
        operand_provenance: false,
        literal_answers: false,
        initialization_artifact: None,
    });

    let err = mock.fit_joint_admission(&empty_id, config).unwrap_err();
    assert!(err
        .to_string()
        .contains("duplicate/empty admission example id"));
}
