# INC-0139: Does Hopf Fiber Balance Variation Drive Semantically Meaningful Shell Assignment?

## Status
Queued next.

## Summary
Post-INC-0138 structural finding: all embeddings are L2-normalized (`||v||=1.0`),
so the chart radius `r ≡ 1.0` (identity chart). Shells form ONLY because the Hopf
fiber balance `balance=(rho1-rho2)/(rho1+rho2)` varies across embeddings and the
`adaptive_shell_growth` parameter amplifies this into effective-radius spread.

The column-permuted control produces similar `balance` distributions because
column permutation preserves each column's marginal, and `balance` depends only
on marginal norms of column pairs.

**The corrected test for Stage 2:** Can chart rotation (SO(8) learning) concentrate
Hopf fiber energy asymmetry in ways that make shell assignment semantically
discriminating — i.e., does the learned chart create a fiber balance distribution
that differs between real and col-perm inputs?

This increment must decide:

**Option A (LEARN):** Run with `learn_so8=1` and measure whether the learned rotation
changes the `balance` distribution enough to make shells discriminate real from col-perm.

**Option B (ACCEPT CONSTRAINT):** Accept that on unit-norm embeddings with identity chart,
shells are fiber-balance-driven and fiber balance is marginal-norm-driven — hence
semantically blind. Document this as a Stage 2 architectural constraint and formally
redirect Stage 2 to the angular law (where real vs col-perm separation was observed).

## Kill-List Stage
Primary: 2. Measure-Consistent Shell Routing

## Mathematical Object Under Test
- First-factor H^4 routing manifold, Hopf fiber balance field
- Whether SO(8) chart learning changes `balance` distribution between real and col-perm inputs
- Identity chart (learn_so8=0): baseline — fiber balance is marginal-norm-driven
- Learned chart (learn_so8=1): test — does learned rotation create semantic fiber asymmetry?

## Success Condition
With `learn_so8=1`, the shell-level separation |shell_pmax(GEOM_ORIG) - shell_pmax(GEOM_COL_PERM)| > 0.05
across 2 seeds, OR the `balance` distribution differs between real and col-perm by KL > 0.1.
This would show the geometry (via chart learning) can make fiber balance semantically meaningful.

## Falsification Condition
With `learn_so8=1`, shell metrics remain indistinguishable between real and col-perm inputs
(|shell_pmax diff| < 0.02). This confirms fiber balance on unit-norm embeddings is
structurally marginal-norm-driven regardless of rotation, and Stage 2 must redirect
to the angular law as the primary semantic routing dimension.

## Trigger
INC-0138 Closed: REFINE (2026-03-13) + post-experiment structural finding:
`r ≡ 1.0` on unit-norm embeddings with identity chart; shells driven by Hopf
fiber balance via adaptive_shell_growth; column permutation preserves balance
distribution because it preserves column marginals.
