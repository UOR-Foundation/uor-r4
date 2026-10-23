# ORDER: legal-construction-exact-solve, antigravity-47 (certified root-LP optimum and exact dual bound via Dy<120> dual simplex)

Lab: antigravity. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `127632bc3`. Item PR (partial: `References #2828`).

## Goal

In Unit 3a (`claude-50`) and Unit 3b (`claude-54`), primal simplex on the saved 1,960-variable root relaxation (`bb::relaxation`, 440 rows) timed out after 7,800 s (775 exact pivots in `blp`; 48,025 `Dy<120>` pivots in `search::search::<120>`) without returning a basis or a certificate. In Unit 4 (`deepseek-57`), a greedy single-coefficient pass produced a feasible discrete move with objective `-426088804491/549755813888` (~`-0.775051`), but no LP optimum, dual bound, or certificate was established.

This unit (classification: exact-arithmetic host instrument under Kimi's PM ruling `6342969530` and Casey's standing direction `6355973721` on `#2828`) closes the root-LP and exact dual-bound blocker:

1. **Measure the primal stall.** Add `degenerate_pivots` and `bland_switches` counters to `SearchStats` and diagnose why primal simplex at `δ = 0` (where all 380 guard rows have right-hand side `0`) stalls after `refactor()` introduces `~2^-110` rounding dust on zero-valued basic variables.
2. **Bounded-variable Dual Simplex in `Dy<P>` (`no_floats` compliant).** Implement bounded-variable Dual Simplex with the bound-flipping ratio test (BFRT) in `Dy<P>`, plus exact projection lifting from the 380-row `δ`-space relaxation to the 440-row `bb::relaxation` basis and unit-column-first elimination in `blp::certify_basis`.
3. **Exact certificate (`blp::certify_basis`) and basis-free Lagrangian dual bound.** Certify the exact optimal basis and exact rational objective of both the 380-row convex-hull `δ`-relaxation (`delta_lp`) and the 440-row `bb::relaxation` on the verified 1,960-variable saved case (`case::load_verified`, 384/384 sha256 checks), and cross-check with a basis-free exact rational Lagrangian dual bound $L(y) = \sum_i \min_{\delta_i \in \mathcal{D}_i} (g_i - (J^\top y)_i)\delta_i$ for $y \ge 0$.
4. **Record** the certified exact rational objectives, wall times, active-guard structure (67 tight guards out of 380; 1,853 of 1,920 coefficients already at discrete legal values at the LP optimum), and re-run commands in `docs/labs/legal-construction-exact-solve/README.md` (Unit 5) and `docs/research/notebook/legal-construction.md`.

## Cancel probe (run first)

Ran `grep -rn "dual_simplex\|CERTIFIED\|certified.*true" docs/labs/legal-construction-exact-solve/` at `e8d6a624b`: no match. Neither a dual-simplex basis search nor any certified root-LP optimum or dual bound exists in the repository. Class: instrument.

## Files in scope

- `docs/mechanisms/legal-construction.md`
- `docs/labs/legal-construction-exact-solve/**`
- `docs/reference/storage.md`
- `docs/research/notebook/legal-construction.md`
- `docs/loop/tasks/legal-construction-exact-solve/antigravity-47/**`

## Done when

1. On the finite controls, random LP suite, and tightened-bound node LPs, the `Dy<120>` search plus `blp::certify_basis` produces the same status and exact rational objective as `lp::solve` and `blp::solve`.
2. **Planted faults:**
   - a corrupted basis (one basic index swapped or a flipped nonbasic bound position) is rejected by `blp::certify_basis`, and `blp::solve_from_basis` recovers the exact optimum;
   - a negative dual multiplier $y_j < 0$ is rejected by the basis-free Lagrangian dual certificate check.
3. **Real case (`attempt2`, 384/384 sha256 verified):** `root-search` certifies the exact optimal basis and exact rational objective of `bb::relaxation` (440 rows, 1,960 variables) and `delta_lp` (380 rows, 1,920 variables) via `blp::certify_basis` in seconds on the laptop, recorded in `docs/labs/legal-construction-exact-solve/README.md` (Unit 5) and one row in `docs/research/notebook/legal-construction.md`.
4. All crate tests, `tests/no_floats.rs` (`f32`/`f64` only in `exact.rs`), `cargo clippy --all-targets -- -D warnings`, `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.

## Stop and ask if

- The `Dy<120>` dual simplex basis fails `blp::certify_basis` on the real saved case and exact warm-start repair (`blp::solve_from_basis`) cannot close the remaining pivots within budget.
- Any file outside the declared scope needs to change.
