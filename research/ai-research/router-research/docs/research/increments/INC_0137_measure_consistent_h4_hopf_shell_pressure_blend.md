# INC-0137: Measure-Consistent `H^4` / Hopf Shell-Pressure Blend

## Status
Queued next.

## Trigger
`INC-0136` completed a direct geodesic-shell substitution screen using
`phase4d_hopf_base_ball` and the result failed the route-health gate:
- `shell_pmax=0.8862` (over-concentrated)
- `shell_mass_l1=1.7724` (poor mass distribution)
- `knn_overlap=0.6362` (neighborhood preservation degraded)

All three geometry diagnostics were worse than the `HOPF_BASE_K25_PHI` healthy
control. The direct substitution approach is therefore dead.

The next honest correction is narrower: instead of replacing the shell law
outright, blend the route-chart shell radius with the original-ball geodesic
radius through a bounded pressure controller so the route law stays
chart-coherent while gently pulling shells toward geodesic-correct mass.

## Kill-List Stage
Primary:
1. measure-consistent shell routing

Secondary:
2. Hopf angular correctness

## Mathematical Object Under Test
- first-factor routing manifold on `H^4`
- bounded shell-pressure/controller blend between route-chart radius and
  original-ball geodesic radius
- Hopf-base angular allocation held fixed (not the variable under test here)
- preservation of geodesic neighborhood structure across the blend

The specific test is whether a soft interpolation between the chart shell and
the geodesic shell can correct mass concentration without triggering the shell
collapse / neighborhood degradation that killed `INC-0136`.

## Hypothesis
Direct geodesic-shell substitution over-corrects: it forces shells to live on
the raw ball radius, which conflicts with the chart transform and collapses the
routing geometry.

A bounded blend — parameterized by a shell-pressure weight `shell_pressure_w ∈
[0, 1]` — should allow the route law to stay chart-coherent near `w=0` while
incrementally pulling shell boundaries toward geodesic mass as `w` increases.

If the blend is well-bounded, there should exist an operating point where:
- shell mass concentration improves over `HOPF_BASE_K25_PHI`
- geodesic neighborhood overlap is preserved or improves
- route health (no shell collapse) is maintained
- proxy quality / runtime stays within the current routed band

## Minimal Scope
1. Add a `shell_pressure_w` parameter that interpolates the final shell radius
   between the route-chart radius and the original Poincaré-ball geodesic
   radius:
   - `r_eff = (1 - w) * r_chart + w * r_geodesic`
   - `w = 0.0` recovers the current chart-only behavior
   - `w = 1.0` matches the failed `INC-0136` geodesic substitution
2. Screen a narrow range: `w ∈ {0.1, 0.2, 0.3, 0.4}` against:
   - `HOPF_BASE_K25_PHI` (canonical no-fiber-phase coarse-address control)
   - `HOPF_K25_BASE_PHI` (pure Hopf quality reference)
   - `HOPF_PHI2_BAND_PHI` (widened efficient reference)
3. Evaluate geometry-first diagnostics before any downstream metrics:
   - `shell_pmax` (mass concentration — lower is better)
   - `shell_mass_l1` (total shell mass spread)
   - `knn_overlap` (geodesic neighborhood preservation)
   - route health / shell collapse indicator
   - `hopf_base_mass` / `hopf_sector_chi_std`
4. Only evaluate proxy quality / runtime after geometry gates are met.
5. Do not open translated retrieval or sparse-event surfaces during this
   branch.

## Success Condition
A bounded shell-pressure blend materially improves `shell_mass` and/or
`Hopf-base` diagnostics over `HOPF_BASE_K25_PHI`, preserves or improves
`knn_overlap`, and stays within the routed quality/runtime band without shell
collapse.

Quantitatively: at least one blend point satisfies all of:
- `shell_pmax < shell_pmax(HOPF_BASE_K25_PHI)` (less concentrated)
- `knn_overlap >= knn_overlap(HOPF_BASE_K25_PHI)` (no neighborhood degradation)
- proxy MSE stays within 3% of `HOPF_BASE_K25_PHI`
- no shell collapse (all shells with expected occupancy > 0 remain populated)

## Falsification Condition
If even bounded geodesic-shell blending (all tested `w` values) still worsens
shell concentration or neighborhood preservation relative to `HOPF_BASE_K25_PHI`,
keep `RR-061` open and narrow further into shell-controller diagnostics rather
than reopening later translated frontier work.

Specific: if no blend point satisfies both `shell_pmax` improvement and
`knn_overlap` preservation simultaneously, the branch closes as
negative/explanatory and the next step must identify a different lever.

## Acceptance
- one explicit geometry-return screen artifact exists in `results/analysis/`
- the branch ends with an explicit keep/kill/refine decision
- route-health and neighborhood diagnostics remain interpretable
- the branch conclusion updates `RR-061` status in `KILL_LIST_TRACKER.md`

## Scope Guardrails
- Do not reopen translated sparse-event or dual-anchor work.
- Do not change Hopf angular allocation (that is the next gate after this one).
- Do not change the chart / SO(8) rotation law.
- Keep the sparse-event frozen results as downstream supporting evidence only.
- Do not claim hardware progress from this branch.

## Why This Branch Exists
`INC-0136` proved that the direct geodesic-shell substitution path is dead. The
remaining principled correction is a soft blend. If that also fails, the
`RR-061` gate will need a mechanistically different approach (e.g., a
shell-density controller rather than a radius interpolation), and the project
will need to document that as a narrower architectural constraint on what
measure-consistent shell routing is even achievable in the current chart
framework.

This branch preserves the project goal:

`geometry -> routing -> phase / operator structure -> sparse event compute -> hardware savings`

Progress here means the coarse route law is more defensible as a geometric
primitive, which is the foundation that all later claims rest on.
