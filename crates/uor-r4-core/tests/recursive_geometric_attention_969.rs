use std::collections::BTreeSet;

use uor_r4_core::recursive_geometric_attention::{
    a1ql_public_contract_identities, run_a1ql_non_h4_channel_capacity_probe,
    NOT_RUN_CHANNEL_CAPACITY_HARD_STOP, REDESIGN_NON_H4_ORDERED_TRANSPORT_REPRESENTATION,
};

const CHANNEL_CAPACITY_REPORT_KAPPA: &str =
    "blake3:da72cd766bdc2a938286a44de0be60a66242ed9691a02add025e1ea3609b7ae5";

fn parity(history: &[String], roles: &[String]) -> &'static str {
    assert_eq!(history.len(), roles.len() + 1);
    assert_eq!(history.get(roles.len() - 1).map(String::as_str), Some("aa"));
    assert_eq!(history.last().map(String::as_str), Some("qq"));
    let permutation = history[..roles.len()]
        .iter()
        .map(|token| {
            roles
                .iter()
                .position(|role| role == token)
                .expect("declared role")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        permutation.iter().copied().collect::<BTreeSet<_>>().len(),
        roles.len()
    );
    let inversions = permutation
        .iter()
        .enumerate()
        .map(|(left_index, left)| {
            permutation[left_index + 1..]
                .iter()
                .filter(|right| left > *right)
                .count()
        })
        .sum::<usize>();
    if inversions % 2 == 0 {
        "EVEN"
    } else {
        "ODD"
    }
}

#[test]
fn public_contract_identities_are_deterministic_and_disjoint() {
    let first = a1ql_public_contract_identities().expect("frozen #969 public identities");
    let second = a1ql_public_contract_identities().expect("deterministic #969 identities");
    assert_eq!(first, second);

    let contract = &first.contract;
    assert_eq!(contract.role_order, ["aa", "bb", "cc", "dd"]);
    assert_eq!(contract.current_token, "qq");
    assert_eq!(contract.natural_candidates, ["ll", "rr"]);
    assert_eq!(contract.construction_fixture.observations.len(), 3);
    assert_eq!(contract.validation_fixture.observations.len(), 3);

    let histories = contract
        .construction_fixture
        .observations
        .iter()
        .chain(&contract.validation_fixture.observations)
        .map(|observation| observation.history.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(histories.len(), 6);
    for observation in contract
        .construction_fixture
        .observations
        .iter()
        .chain(&contract.validation_fixture.observations)
    {
        assert_eq!(
            observation.permutation_parity,
            parity(&observation.history, &contract.role_order)
        );
        assert_eq!(
            observation.observed_next,
            if observation.permutation_parity == "EVEN" {
                "ll"
            } else {
                "rr"
            }
        );
    }

    println!(
        "A1QL_CONSTRUCTION_FIXTURE_KAPPA={}",
        first.construction_fixture_kappa
    );
    println!(
        "A1QL_VALIDATION_FIXTURE_KAPPA={}",
        first.validation_fixture_kappa
    );
    println!("A1QL_PUBLIC_CONTRACT_KAPPA={}", first.contract_kappa);
}

#[test]
fn individual_non_h4_channel_capacity_hard_stops_before_selector() {
    let first =
        run_a1ql_non_h4_channel_capacity_probe().expect("frozen #969 channel-capacity probe");
    let second = run_a1ql_non_h4_channel_capacity_probe()
        .expect("deterministic second #969 channel-capacity probe");
    let first_bytes = first
        .canonical_bytes()
        .expect("canonical first #969 report");
    let second_bytes = second
        .canonical_bytes()
        .expect("canonical second #969 report");
    assert_eq!(first, second);
    assert_eq!(first.report_kappa, second.report_kappa);
    assert_eq!(first.report_kappa, CHANNEL_CAPACITY_REPORT_KAPPA);
    assert_eq!(first_bytes, second_bytes);

    let body = &first.body;
    assert_eq!(body.contract_status, "VALID_PUBLIC_CONTRACT");
    assert_eq!(
        body.probe_status,
        "EXERCISED_INDIVIDUAL_NON_H4_CHANNEL_CAPACITY_HARD_STOP"
    );
    assert_eq!(
        body.terminal_verdict,
        REDESIGN_NON_H4_ORDERED_TRANSPORT_REPRESENTATION
    );
    assert!(body
        .localized_defect
        .contains("ELIGIBLE_NON_H4_NON_DIGEST_CHANNEL_KEYS_ON_THE_FROZEN_MATCHED_SPLIT"));
    assert!(!body.omnibus_weighted_combination_opened);
    assert_eq!(body.scalar_functions_searched, 0);
    assert_eq!(body.selector_artifacts_compiled, 0);
    assert!(!body.validation_selector_outputs_opened);

    let fixtures = &body.fixture_integrity;
    assert!(fixtures.construction_kappa_reproduces);
    assert!(fixtures.validation_kappa_reproduces);
    assert!(fixtures.public_contract_kappa_reproduces);
    assert!(fixtures.construction_rule_derived_from_histories);
    assert!(fixtures.labels_attached_after_both_splits_prepared);
    assert!(fixtures.construction_validation_disjoint);
    assert!(fixtures.disjoint_from_a1_a1r_a1p);
    assert!(fixtures.reserved_from_953_and_973);
    assert!(fixtures.same_length_multiset_suffix);
    assert!(fixtures.natural_candidate_union_predeclared);

    assert_eq!(body.support_and_work.len(), 2);
    for split in &body.support_and_work {
        assert_eq!(split.histories, 3);
        assert_eq!(split.candidate_decisions, 6);
        assert_eq!(split.natural_candidates, ["ll", "rr"]);
        assert!(split.natural_candidate_union_exact);
        assert!(split.candidate_source_partition_equal);
        assert_eq!(split.rows_per_query, 7);
        assert_eq!(split.row_reads, 21);
        assert_eq!(split.candidate_entry_ceiling_per_query, 56);
        assert_eq!(split.candidate_ceiling, 8);
        assert_eq!(split.maximum_admitted_candidates, 2);
        assert_eq!(split.exact_payload_inversions, 6);
        assert!(split.exact_direct_rows_miss);
        assert!(!split.target_injected);
        assert!(!split.future_events_visible);
        assert!(split.support_and_work_contract_exact);
    }

    assert_eq!(body.channels.len(), 9);
    let unavailable = body
        .channels
        .iter()
        .filter(|channel| !channel.available)
        .collect::<Vec<_>>();
    assert_eq!(unavailable.len(), 2);
    for channel in unavailable {
        assert!(!channel.survives_in_non_digest_state);
        assert_eq!(channel.earlier_order_sensitivity.denominator, 0);
        assert_eq!(channel.candidate_sensitivity.denominator, 0);
        assert_eq!(channel.same_candidate_change.denominator, 0);
        assert!(channel.exact_classes.is_empty());
        assert_eq!(channel.sealed_validation_coverage.denominator, 0);
        assert_eq!(
            channel
                .sealed_validation_no_class_splitting_oracle_ceiling
                .denominator,
            0
        );
        assert_eq!(
            channel
                .sealed_validation_construction_transfer_query_ceiling
                .denominator,
            0
        );
        assert!(!channel.capacity_gate_pass);
    }

    for channel_name in [
        "session-hypersphere-state",
        "winding-window-state",
        "projection-energy",
        "factor-count",
        "cosine-resonance",
        "accumulated-hopf-phase",
    ] {
        let channel = body
            .channels
            .iter()
            .find(|channel| channel.channel == channel_name)
            .expect("declared scalar channel");
        assert!(channel.available);
        assert!(channel.survives_in_non_digest_state);
        assert_eq!(
            (
                channel.earlier_order_sensitivity.numerator,
                channel.earlier_order_sensitivity.denominator
            ),
            (0, 15)
        );
        assert_eq!(
            (
                channel.candidate_sensitivity.numerator,
                channel.candidate_sensitivity.denominator
            ),
            (0, 6)
        );
        assert_eq!(
            (
                channel.same_candidate_change.numerator,
                channel.same_candidate_change.denominator
            ),
            (0, 30)
        );
        assert_eq!(channel.exact_classes.len(), 1);
        assert_eq!(channel.construction_class_count, 1);
        assert_eq!(channel.construction_pure_class_count, 0);
        assert_eq!(channel.construction_impure_class_count, 1);
        assert!(!channel.construction_classes_pure);
        assert_eq!(
            (
                channel.sealed_validation_coverage.numerator,
                channel.sealed_validation_coverage.denominator
            ),
            (6, 6)
        );
        assert_eq!(
            (
                channel
                    .sealed_validation_no_class_splitting_oracle_ceiling
                    .numerator,
                channel
                    .sealed_validation_no_class_splitting_oracle_ceiling
                    .denominator
            ),
            (3, 6)
        );
        assert_eq!(
            (
                channel
                    .sealed_validation_construction_transfer_decision_ceiling
                    .numerator,
                channel
                    .sealed_validation_construction_transfer_decision_ceiling
                    .denominator
            ),
            (0, 6)
        );
        assert_eq!(
            (
                channel
                    .sealed_validation_construction_transfer_query_ceiling
                    .numerator,
                channel
                    .sealed_validation_construction_transfer_query_ceiling
                    .denominator
            ),
            (0, 3)
        );
        assert!(channel.support_equal);
        assert!(channel.work_equal);
        assert!(!channel.capacity_gate_pass);
    }

    let candidate_pair = body
        .channels
        .iter()
        .find(|channel| channel.channel == "session-candidate-transition")
        .expect("declared candidate-relative pair");
    assert_eq!(
        candidate_pair.status,
        "EXERCISED_PAIR_NO_INTERACTION_CAPACITY_GATE_FAIL"
    );
    assert_eq!(
        (
            candidate_pair.earlier_order_sensitivity.numerator,
            candidate_pair.earlier_order_sensitivity.denominator
        ),
        (0, 15)
    );
    assert_eq!(
        (
            candidate_pair.candidate_sensitivity.numerator,
            candidate_pair.candidate_sensitivity.denominator
        ),
        (6, 6)
    );
    assert_eq!(
        (
            candidate_pair.same_candidate_change.numerator,
            candidate_pair.same_candidate_change.denominator
        ),
        (0, 30)
    );
    assert_eq!(candidate_pair.exact_classes.len(), 2);
    assert_eq!(candidate_pair.construction_class_count, 2);
    assert_eq!(candidate_pair.construction_pure_class_count, 0);
    assert_eq!(candidate_pair.construction_impure_class_count, 2);
    assert!(!candidate_pair.construction_classes_pure);
    assert_eq!(
        (
            candidate_pair.sealed_validation_coverage.numerator,
            candidate_pair.sealed_validation_coverage.denominator
        ),
        (6, 6)
    );
    assert_eq!(
        (
            candidate_pair
                .sealed_validation_no_class_splitting_oracle_ceiling
                .numerator,
            candidate_pair
                .sealed_validation_no_class_splitting_oracle_ceiling
                .denominator
        ),
        (4, 6)
    );
    assert_eq!(
        (
            candidate_pair
                .sealed_validation_construction_transfer_decision_ceiling
                .numerator,
            candidate_pair
                .sealed_validation_construction_transfer_decision_ceiling
                .denominator
        ),
        (0, 6)
    );
    assert_eq!(
        (
            candidate_pair
                .sealed_validation_construction_transfer_query_ceiling
                .numerator,
            candidate_pair
                .sealed_validation_construction_transfer_query_ceiling
                .denominator
        ),
        (0, 3)
    );
    assert!(!candidate_pair.capacity_gate_pass);

    assert!(body
        .transformation_controls
        .iter()
        .all(|control| control.status == NOT_RUN_CHANNEL_CAPACITY_HARD_STOP));
    assert!(body
        .conditional_selector_controls
        .iter()
        .all(|control| control.status == NOT_RUN_CHANNEL_CAPACITY_HARD_STOP));
    assert!(body
        .gate_and_sentence_status
        .iter()
        .filter(|control| !control.control.starts_with("paragraph")
            && !control.control.starts_with("conversation")
            && !control.control.starts_with("global")
            && !control.control.starts_with("correctness")
            && !control.control.starts_with("reasoning")
            && !control.control.starts_with("chat"))
        .all(|control| control.status == NOT_RUN_CHANNEL_CAPACITY_HARD_STOP));
    assert!(body.claim_boundary.channel_capacity_falsifier_only);
    assert!(
        body.claim_boundary
            .codec_support_payload_h4_and_incremental_state_preserved
    );
    assert!(!body.claim_boundary.selector_frozen);
    assert!(!body.claim_boundary.gate_0_exercised);
    assert!(!body.claim_boundary.sentence_qualification_exercised);
    assert!(!body.claim_boundary.local_sentence_attention_established);
    assert!(!body.claim_boundary.inference_established);
    assert!(!body.claim_boundary.generation_established);
    assert!(!body.claim_boundary.digest_or_kappa_used_as_geometry);
    assert!(!body.claim_boundary.h4_or_heatmap_used_as_new_readout);
    assert!(
        !body
            .claim_boundary
            .target_future_exact_continuation_or_provider_used
    );
    assert!(!body.claim_boundary.candidate_support_or_admission_modified);

    println!("A1QL_REPORT_KAPPA={}", first.report_kappa);
    for channel in &body.channels {
        println!(
            "A1QL_CHANNEL={} status={} order={}/{} candidate={}/{} same_candidate={}/{} classes={} pure={}/{} coverage={}/{} oracle={}/{} transfer_decisions={}/{} transfer_queries={}/{}",
            channel.channel,
            channel.status,
            channel.earlier_order_sensitivity.numerator,
            channel.earlier_order_sensitivity.denominator,
            channel.candidate_sensitivity.numerator,
            channel.candidate_sensitivity.denominator,
            channel.same_candidate_change.numerator,
            channel.same_candidate_change.denominator,
            channel.exact_classes.len(),
            channel.construction_pure_class_count,
            channel.construction_class_count,
            channel.sealed_validation_coverage.numerator,
            channel.sealed_validation_coverage.denominator,
            channel.sealed_validation_no_class_splitting_oracle_ceiling.numerator,
            channel.sealed_validation_no_class_splitting_oracle_ceiling.denominator,
            channel.sealed_validation_construction_transfer_decision_ceiling.numerator,
            channel.sealed_validation_construction_transfer_decision_ceiling.denominator,
            channel.sealed_validation_construction_transfer_query_ceiling.numerator,
            channel.sealed_validation_construction_transfer_query_ceiling.denominator,
        );
    }
}
