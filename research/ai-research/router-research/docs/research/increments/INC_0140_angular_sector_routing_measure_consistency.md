# INC-0140: Measure-Consistent Angular Routing — Does Sector Assignment Discriminate Real Structure?

## Status
Queued next.

## Summary
INC-0136 through INC-0139 collectively establish that the Hopf fiber balance
(shell routing dimension) is a structural dead end for Stage 2 on L2-normalized
embeddings:

- **INC-0136:** Direct geodesic shell substitution KILLED (shell_pmax=0.886)
- **INC-0137:** Shell pressure blend KILLED (all weights worsen routing vs baseline)
- **INC-0138:** r ≡ 1.0 — shells are fiber-balance-driven, not radius-driven;
  real vs col-perm INDISTINGUISHABLE at shell level
- **INC-0139:** SO(8) learning nominally passes discrimination threshold (|diff|=0.0622)
  but via generic concentration; routing quality destroyed (pmax_after collapses to 0.10)

**Stage 2 redirect:** The angular routing law (sector assignment via Hopf base
coordinates delta, chi, theta1/theta2) showed quantitatively meaningful real vs
Gaussian separation in INC-0138 (buckets=15.5 vs 50.0 Gaussian, ~3× difference;
sector_entropy gap). This increment formally tests whether sector-level angular
routing is measure-consistent with the H^4 angular measure.

**The new test for Stage 2:** Does angular sector routing distribute tokens
consistently with the H^4 surface measure over the Hopf base S^3 — and does
it discriminate semantic structure (real embeddings) more than col-perm controls?

## Kill-List Stage
Primary: 2. Measure-Consistent Shell Routing (redirect to angular law)
Cross-reference: 3. Hopf Angular Correctness (overlapping question)

## Mathematical Object Under Test
- First-factor H^4 routing manifold, Hopf base projection (delta, chi, theta angles)
- Sector assignment consistency with H^4 angular measure on S^3 base
- Whether sector routing produces semantically meaningful distribution differences
  between real vs col-perm inputs at the sector level (not shell level)

## Success Condition
With `sector_mode=phase4d_hopf_base`, `learn_so8=0`, `learn_scale=0`:
- Sector-level `pmax_after` or `sector_entropy` DIFFERS between GEOM_ORIG and
  GEOM_COL_PERM by more than double the noise floor (|diff|/mean > 0.2), OR
- The Hopf angular mass error (`mean_hopf_base_mass_error`, `mean_hopf_angular_mass_error`)
  is meaningfully lower for real embeddings than for col-perm
This would show angular routing carries semantic information that shells cannot.

## Falsification Condition
Sector metrics (pmax_after, sector_entropy, hopf mass errors) are indistinguishable
between real and col-perm inputs at the sector level. This would confirm that Stage 2
cannot be resolved via current routing geometry alone and the resolution must come
from a different architectural modification (non-L2-normalized embeddings, or a
fundamentally different sector law).

## Trigger
INC-0139 Closed: REFINE (2026-03-13) — fiber balance / SO(8) path exhausted.
Angular sector dimension showed real vs Gaussian separation in INC-0138 but
the real vs col-perm comparison at sector level was not fully characterized.
