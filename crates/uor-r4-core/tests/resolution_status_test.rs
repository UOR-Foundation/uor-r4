use uor_r4_core::transformerless::resolution_status::{
    CalibratedFeatures, FallbackAction, FallbackPolicy, ResolutionStatus,
};

#[test]
fn test_calibrated_feature_classification_theorem_12() {
    // 1. Supported
    let feat_supported = CalibratedFeatures {
        hamming_dist: 5,
        calibrated_radius: 20,
        score_margin: 50,
        frontier_density: 10,
        is_backed_off: false,
    };
    assert_eq!(feat_supported.classify(), ResolutionStatus::Supported);

    // 2. Boundary
    let feat_boundary = CalibratedFeatures {
        hamming_dist: 22,
        calibrated_radius: 20,
        score_margin: 50,
        frontier_density: 10,
        is_backed_off: false,
    };
    assert_eq!(feat_boundary.classify(), ResolutionStatus::Boundary);

    // 2b. Boundary via low score margin (abs margin < 10)
    let feat_low_margin = CalibratedFeatures {
        hamming_dist: 5,
        calibrated_radius: 20,
        score_margin: 0,
        frontier_density: 10,
        is_backed_off: false,
    };
    assert_eq!(feat_low_margin.classify(), ResolutionStatus::Boundary);

    // 2c. Extreme negative margin should not panic (i32::MIN)
    let feat_min_margin = CalibratedFeatures {
        hamming_dist: 5,
        calibrated_radius: 20,
        score_margin: i32::MIN,
        frontier_density: 10,
        is_backed_off: false,
    };
    assert_eq!(feat_min_margin.classify(), ResolutionStatus::Supported);

    // 3. BackedOff
    let feat_backed_off = CalibratedFeatures {
        hamming_dist: 5,
        calibrated_radius: 20,
        score_margin: 50,
        frontier_density: 10,
        is_backed_off: true,
    };
    assert_eq!(feat_backed_off.classify(), ResolutionStatus::BackedOff);

    // 4. Novel
    let feat_novel = CalibratedFeatures {
        hamming_dist: 50,
        calibrated_radius: 20,
        score_margin: 50,
        frontier_density: 10,
        is_backed_off: false,
    };
    assert_eq!(feat_novel.classify(), ResolutionStatus::Novel);

    // 5. Contradictory
    let feat_contradictory = CalibratedFeatures {
        hamming_dist: 5,
        calibrated_radius: 20,
        score_margin: 50,
        frontier_density: 150,
        is_backed_off: false,
    };
    assert_eq!(
        feat_contradictory.classify(),
        ResolutionStatus::Contradictory
    );
}

#[test]
fn test_fallback_policy_decision_d4() {
    let policy = FallbackPolicy::default();

    assert_eq!(
        policy.action_for(ResolutionStatus::Supported),
        FallbackAction::ConsultExact
    );
    assert_eq!(
        policy.action_for(ResolutionStatus::Boundary),
        FallbackAction::ConsultExact
    );
    assert_eq!(
        policy.action_for(ResolutionStatus::BackedOff),
        FallbackAction::Abstain
    );
    assert_eq!(
        policy.action_for(ResolutionStatus::Novel),
        FallbackAction::Abstain
    );
    assert_eq!(
        policy.action_for(ResolutionStatus::Contradictory),
        FallbackAction::Abstain
    );
}

#[test]
fn test_custom_fallback_policy() {
    let custom_policy = FallbackPolicy {
        supported_action: FallbackAction::ConsultExact,
        boundary_action: FallbackAction::ConsultExact,
        backed_off_action: FallbackAction::BasePrior,
        novel_action: FallbackAction::FallbackToken(42),
        contradictory_action: FallbackAction::Abstain,
    };

    assert_eq!(
        custom_policy.action_for(ResolutionStatus::Novel),
        FallbackAction::FallbackToken(42)
    );
}

#[test]
fn test_fallback_actions_cover_d4_behavior_codes() {
    let actions = [
        (FallbackAction::Continue, "Continue"),
        (FallbackAction::Widen, "Widen"),
        (FallbackAction::ConsultExact, "ConsultExact"),
        (FallbackAction::CertifiedFallback, "CertifiedFallback"),
        (FallbackAction::Abstain, "Abstain"),
    ];

    for (action, code) in actions {
        assert_eq!(serde_json::to_value(action).unwrap(), code);
    }
}
