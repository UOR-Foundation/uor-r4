# ORDER: legal-construction-exact-solve, antigravity-55

Lab: antigravity. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `642674f97`.

## Goal

The item's done-when reads:

> The saved 1,960-variable case is solved in exact arithmetic and the result is recorded as an optimal assignment with its objective, or as an infeasibility proof. Scoring that result on the real panel is a separate step.

This repair unit (`antigravity-55`, repairing `kimi`'s `CHANGES` review on PR `#3152` at `953a2312932299d6d223ae1668a8e3d52d7dcd9a`) resolves the `disjunctive_ms` runtime regression (`1,809,808 ms` vs Unit 10's `1,116,550 ms`) and the `REPORT.md` accuracy findings:

1. **Cut `disjunctive_ms` below Unit 10 (`1,116,550 ms`) while preserving bit-for-bit identical Unit 11 rational bounds and RCBT eliminations (`src/search.rs`):**
   - Use `BigRational::new_raw` in `dyadic_to_rational` after stripping shared factors of two via `trailing_zeros` (eliminating `1,920` redundant `BigInt::gcd` calls per `compute_rc` call) and add dyadic shift-and-add `sum_rationals` for summing $n$-element `BigRational` vectors without per-element GCDs.
   - Cache `DualPoolEntry { pi, rc, root_terms, root_lb, root_span }` in `dual_rc_pool` so `eval_branch`, cross-branch leaf bound tightening, `eval_rc_mask`, and `tighten_leaf_mask` compute `leaf_lb` incrementally over only the split coordinates `code_bounds[i] != (-7, 7)` ($\le 23$ coordinates instead of $1,920$), and `tighten_leaf_mask` skips any coefficient with `root_span[i] <= leaf_slack` in $O(1)$.
2. **Repair `antigravity-54/REPORT.md` and record `antigravity-55/REPORT.md`:**
   - Replace the stale descriptions (`tab.col_s1`, `max_pool_duals = 64`, `seen_pis`, `split_sub`) in `antigravity-54/REPORT.md` with the actual delivered code (`arj`-first pricing in `dual_simplex_run::<P>`, `dual_rc_pool` capped at `160` with `.find(...)` deduplication, and `split66`/`split66_none` assertions in `tests/search_certify.rs`) and explicitly record `disjunctive_ms` and its direction in both reports.

## Cancel probe (run first)

Verified `kimi`'s review comment `IC_kwDOS1s4Zc8AAAABfI8RYg` on PR `#3152` and inspected `src/search.rs:2012-2025,2178-2567` and `antigravity-54/REPORT.md`: `eval_branch`, leaf pool tightening, and `tighten_leaf_mask` re-sum all `1,920` `BigRational` terms from scratch across all `24 * 160` `(leaf, pool_rc)` pairs, causing the `+693,258 ms` increase in `disjunctive_ms`. Probe confirms the repair is needed.

## Files in scope

- `docs/labs/legal-construction-exact-solve/solver/src/search.rs`
- `docs/labs/legal-construction-exact-solve/README.md`
- `docs/research/notebook/legal-construction.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-54/REPORT.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-55/ORDER.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-55/REPORT.md`
- `docs/labs/legal-construction-exact-solve/solver/tests/search_certify.rs` (added by claude, 24 Oct, finishing this PR under owner direction: kimi's addendum and claude's CHANGES need tests)

## Done when

- Both `solve_ms` and `disjunctive_ms` on `attempt2` are lower than Unit 10 (`264,832 ms` and `1,116,550 ms`) with bit-for-bit identical Unit 11 rational outputs (`disjunctive_tree_lower_bound`, `disjunctive_tree_eliminated_codes = 6,395`, `disjunctive_tree_fixed_coeffs = 115`, `disjunctive_tree_binary_or_fixed_coeffs = 185`), and `antigravity-54/REPORT.md` + `antigravity-55/REPORT.md` accurately describe the delivered code, tests, and timings.
- `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked`, `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.
- The split-candidate sort visits scored candidates before unscored ones, pinned by a >16-candidate ordering test; pooling is pinned by a with-pool versus without-pool test (both with planted faults).

## Stop and ask if

- Any file outside the listed scope would need to change.
