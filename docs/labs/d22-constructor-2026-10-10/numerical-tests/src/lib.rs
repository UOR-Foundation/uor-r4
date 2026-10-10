//! Dedicated synthetic numerical fixtures; no model inputs or model evaluation.
#[cfg(test)]
mod tests {
    use microlp::repair;
    fn columns(a: &[&[f64]]) -> Vec<(Vec<usize>, Vec<f64>)> {
        (0..a.len())
            .map(|c| {
                let entries: Vec<_> = (0..a.len())
                    .filter_map(|r| (a[r][c] != 0.0).then_some((r, a[r][c])))
                    .collect();
                entries.into_iter().unzip()
            })
            .collect()
    }
    #[test]
    fn small_pivots_and_rescaled_matrices() -> Result<(), String> {
        for scale in [1e-100, 1.0, 1e100] {
            repair::qualify(columns(&[&[scale, 0.0], &[0.0, scale * 1e-13]]))?;
        }
        Ok(())
    }
    #[test]
    fn exact_fallback_recovers_nonzero_dyadic_pivot() -> Result<(), String> {
        repair::reset_stats();
        let near = 1.0 + f64::EPSILON;
        repair::qualify(columns(&[&[1.0, 1.0], &[1.0, near]]))?;
        assert!(repair::stats().exact_fallbacks > 0);
        assert_eq!(repair::stats().exact_singular, 0);
        Ok(())
    }
    #[test]
    fn exact_singularity_stays_a_numerical_error() {
        repair::reset_stats();
        assert!(repair::qualify(columns(&[&[1.0, 1.0], &[2.0, 2.0]])).is_err());
        assert_eq!(repair::stats().exact_singular, 1);
    }
    #[test]
    fn nonsymmetric_dense_sparse_normal_transpose() -> Result<(), String> {
        repair::qualify(columns(&[
            &[2.0, 3.0, 0.0],
            &[0.0, 4.0, 1.0],
            &[1.0, 0.0, 5.0],
        ]))
    }
    #[test]
    fn malformed_and_nonfinite_inputs_fail() {
        assert!(repair::qualify(vec![(vec![1], vec![1.0])]).is_err());
        assert!(repair::qualify(vec![(vec![0], vec![f64::NAN])]).is_err());
        assert!(repair::qualify(vec![(vec![0, 0], vec![1.0, 1.0])]).is_err());
    }
    #[test]
    fn rejected_reset_preserves_old_basis() -> Result<(), String> {
        repair::transaction_fixture("failed_reset_is_transactional")
    }
    #[test]
    fn corrupted_eta_is_refreshed_before_new_selection() -> Result<(), String> {
        repair::transaction_fixture("corrupt_eta_refreshes_and_reselects")
    }
    #[test]
    fn rejected_pivot_preserves_whole_state() -> Result<(), String> {
        repair::transaction_fixture("failed_pivot_is_transactional")
    }
    #[test]
    fn refresh_does_not_switch_artificial_objective() -> Result<(), String> {
        repair::transaction_fixture("refresh_preserves_artificial_objective")
    }
    #[test]
    fn disagreeing_pivot_uses_original_basis_and_phase_checks() -> Result<(), String> {
        repair::transaction_fixture("disagreeing_pivot_rebuilds_original_basis")
    }
    #[test]
    fn rejected_disagreeing_pivot_preserves_whole_state() -> Result<(), String> {
        repair::transaction_fixture("disagreeing_pivot_rollback")
    }
    #[test]
    fn refresh_rejects_recomputed_primal_infeasibility() -> Result<(), String> {
        repair::transaction_fixture("refresh_rejects_lost_primal")
    }
    #[test]
    fn refresh_rejects_recomputed_working_dual_infeasibility() -> Result<(), String> {
        repair::transaction_fixture("refresh_rejects_lost_working_dual")
    }
    #[test]
    fn fixing_basic_variable_allows_cost_increase_and_interior_bound() -> Result<(), String> {
        repair::transaction_fixture("fix_variable_ambiguous_exchange")
    }
}

#[test]
fn regular_pivot_rejects_phase_loss_transactionally() -> Result<(), String> {
    microlp::repair::transaction_fixture("regular_pivot_phase_rollback")
}

#[test]
fn compensated_cost_and_original_transpose_refinement() -> Result<(), String> {
    microlp::repair::transaction_fixture("compensated_cost_and_refinement")
}

#[test]
fn signed_ratio_reselection_avoids_harris_double_allowance() -> Result<(), String> {
    microlp::repair::transaction_fixture("signed_ratio_avoids_double_allowance")
}

#[test]
fn step_interval_includes_leaving_ineligible_and_free_columns() -> Result<(), String> {
    microlp::repair::transaction_fixture("step_interval_covers_all_columns")
}

#[test]
fn signed_phase_direct_caller_restores_original_objective() -> Result<(), String> {
    microlp::repair::transaction_fixture("signed_phase_direct_restore")
}

#[test]
fn signed_phase_limit_before_feasibility_resumes_honestly() -> Result<(), String> {
    microlp::repair::transaction_fixture("signed_phase_limit_before_feasibility")
}

#[test]
fn signed_phase_limit_during_original_optimization_resumes_honestly() -> Result<(), String> {
    microlp::repair::transaction_fixture("signed_phase_limit_during_original_optimize")
}

#[test]
fn signed_phase_keeps_interior_fixed_variable() -> Result<(), String> {
    microlp::repair::transaction_fixture("signed_phase_preserves_fixed_interior")
}

#[test]
fn basis_load_resets_pending_phase() -> Result<(), String> {
    microlp::repair::transaction_fixture("basis_load_resets_pending_phase")
}

#[test]
fn both_infeasible_entry_uses_signed_phase() -> Result<(), String> {
    microlp::repair::transaction_fixture("both_infeasible_entry_uses_signed_phase")
}

#[test]
fn dual_refresh_recovery_keeps_factor_errors() -> Result<(), String> {
    microlp::repair::transaction_fixture("dual_refresh_recovery_keeps_factor_errors")
}

#[test]
fn actual_rhs_refinement_and_rollback() -> Result<(), String> {
    microlp::repair::transaction_fixture("actual_rhs_refinement_and_rollback")
}

#[test]
fn signed_phase_mixed_states_and_objective() -> Result<(), String> {
    microlp::repair::transaction_fixture("signed_phase_mixed_states_and_objective")
}

#[test]
fn signed_phase_vector_fixed_across_pivot() -> Result<(), String> {
    microlp::repair::transaction_fixture("signed_phase_vector_fixed_across_pivot")
}
