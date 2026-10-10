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
}
