# INC-0141: Raw Embedding Angular Routing — Does Removing L2-Normalization Restore Hopf Angular Signal?

## Status
Queued next.

## Summary
INC-0136 through INC-0140 have exhausted all fixed-geometry routing paths on L2-normalized
embeddings:

- **INC-0136:** Direct geodesic shell substitution KILLED
- **INC-0137:** Shell pressure blend KILLED
- **INC-0138:** r≡1 structural finding; shells fiber-balance-driven; real vs col-perm
  INDISTINGUISHABLE at shell level
- **INC-0139:** SO(8) learning nominally passes threshold but destroys routing quality
  (pmax_after collapses from 0.5→0.1); REFINE
- **INC-0140:** Angular sector routing (phase4d_hopf_base) also indistinguishable from
  col-perm; KILL. Forensic audit identifies root cause: L2-normalization collapses
  embeddings to S^127, leaving within-pair Hopf correlations near-zero
  (corr(z[0],z[2])=−0.039, corr(z[4],z[6])=−0.018)

**New hypothesis:** L2-normalization is the structural barrier. Raw (non-L2-normalized)
embeddings may retain within-pair Hopf angular structure that is destroyed by the
unit-sphere projection.

**The test for this increment:** Run the same angular sector routing experiment (ANG_ORIG
vs ANG_COL_PERM vs ANG_GAUSSIAN) on raw (non-normalized) embeddings by skipping the L2
normalization step in the eval harness, and measure:
1. Within-pair correlations (corr(z[0],z[2]), corr(z[4],z[6])) — are they non-trivially
   higher than the near-zero values on L2-normalized embeddings?
2. Sector-level discrimination: does |pmax_after ORIG - pmax_after COL_PERM| / mean > 0.2?
3. Are sector assignments more semantically structured (lower sector_entropy, higher
   hopf_base_mass_error separation)?

## Kill-List Stage
Primary: 2. Measure-Consistent Shell Routing (all L2-normalized paths exhausted; testing
normalization as root structural cause)

## Mathematical Object Under Test
- First-factor H^4 routing manifold, Hopf base projection on raw embeddings
- Whether the unit-sphere constraint (L2-normalization) is the root cause of angular
  routing degeneracy
- Within-pair Hopf correlations in raw vs L2-normalized embedding space

## Success Condition
On raw (non-L2-normalized) embeddings with `sector_mode=phase4d_hopf_base`, `learn_so8=0`:
- |pmax_after diff| / mean (ORIG vs COL_PERM) > 0.2, AND
- within-pair correlations (corr(z[i],z[j])) measurably higher than ~−0.04

This would demonstrate that L2-normalization was the specific barrier, and raw embeddings
provide the angular routing signal needed for Stage 2.

## Falsification Condition
Raw embeddings also fail to discriminate real from col-perm at the sector level
(|diff|/mean < 0.2). This would confirm Stage 2 is structurally blocked regardless of
embedding normalization, and the kill-list stage must be revised or re-scoped.

## Trigger
INC-0140 Closed: KILL (2026-03-13) — angular sector routing on L2-normalized embeddings
indistinguishable from col-perm. Forensic audit identifies L2-normalization as root cause.
