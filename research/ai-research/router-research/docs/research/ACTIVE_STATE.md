# Active Research State

Purpose: this is the single canonical live-queue file.

Use this file to answer:
- what the active research gate is
- which increment is actually next
- what mathematical object is under test
- what counts as success or falsification

Do not treat `CURRENT_DIRECTION.md`, `HANDOFF_CURRENT.md`, or `LIVE_WORKLOG.md`
as authoritative when they disagree with this file.

## Canonical Queue
- Current primary RR: `RR-061`
- Current primary INC: `INC-0143` — **Closed: KEEP** (2026-03-13). 4-seed finalize:
  PPMI-SVD ORIG mean_pmax_after=0.0905 vs COL_PERM=0.0613, rel_diff=38.5% across all
  4 seeds (range 30.6%–54.6%). Stage 2 closed as PARTIAL-PASS.
- Next INC: TBD — Stage 3 (Hopf angular correctness) can now be opened.
  Immediate action: queue first Stage 3 increment.
- Current primary increment doc:
  `docs/research/increments/INC_0143_TBD.md`
- Kill-list stage: `measure-consistent shell routing` (Stage 2 → CLOSED/PARTIAL-PASS)
- Mathematical object under test:
  `next: first-factor H^4 routing manifold, Hopf angular law — Stage 3`
- Success condition: `Stage 3 RR queued with verified falsification condition`
- Falsification condition: `n/a — planning phase`

## Latest Closed Increment
- `INC-0143`:
  `docs/research/increments/INC_0143_TBD.md`
  - status: `Closed: KEEP`
  - verdict: 4-seed finalize of PPMI-SVD discrimination. SEM_ORIG mean_pmax=0.0905,
      SEM_COL_PERM=0.0613, rel_diff=38.5% across 4 seeds (30.6%–54.6%). All seeds pass.
      Stage 2 closed as PARTIAL-PASS. H^4 Hopf routing is semantically discriminative
      with PPMI-SVD embeddings and seed-stable; production routing requires structured
      embeddings.

- `INC-0142`:
  `docs/research/increments/INC_0142_TBD.md`
  - status: `Closed: KEEP`
  - verdict: PPMI-SVD semantic embeddings with H^4 Hopf routing (dims 3,65,2,21) show
      ORIG > COL_PERM > GAUSSIAN in the correct direction across both seeds.
      Mean pmax_after: ORIG=0.0874, COL_PERM=0.0638. rel_diff=31.2% (z≈4.2).
      Both seeds pass individually (seed0 z=4.21, seed1 z=4.15). This confirms the
      H^4 Hopf routing geometry IS semantically discriminative with structured
      embeddings. INC-0136–0141 failures were proxy-task failures, not geometry failures.
      Stage 2 status → PARTIAL-PASS.

- `INC-0141`:
  `docs/research/increments/INC_0141_TBD.md`
  - status: `Closed: KILL`
  - verdict: Routing with optimal dims (46,117,62,78) — highest within-pair correlations
    in the 128-dim hash embedding (|corr|=0.479) — does not discriminate real from
    col-perm. OPT_ORIG pmax_after=0.379 vs OPT_COL_PERM=0.388 (wrong direction),
    rel_diff=0.025 (<<0.2 threshold). Pre-screen TV=0.109 signal does not survive.
    Cascade conclusion: hash embedding is proxy-task-blocked for Stage 2 — no 4D
    Hopf subspace of a hash feature can produce semantic angular concentration;
    Stage 2 requires semantically structured embeddings.

- `INC-0140`:
  `docs/research/increments/INC_0140_angular_sector_routing_measure_consistency.md`
  - status: `Closed: KILL`
  - verdict: Angular sector routing (phase4d_hopf_base, learn_so8=0) is measure-degenerate
    on L2-normalized embeddings. ANG_ORIG vs ANG_COL_PERM pmax_after ratio: 0.004 (<<0.2
    threshold). Forensic audit confirms genuine kill: col-perm changes 66% of per-sample
    sector assignments (delta KS=0.621) but sector SIZE distribution is nearly invariant
    (TV=0.009), so prediction performance is concentration-driven not semantically aligned.
    Within-pair Hopf correlations near-zero (corr≈−0.04, −0.02) on L2-normalized embeddings.
    Root cause: L2-normalization projects to S^127; fixed 4D Hopf subspace sees no
    structured angular signal.

- `INC-0139`:
  `docs/research/increments/INC_0139_TBD.md`
  - status: `Closed: REFINE`
  - verdict: SO(8) chart learning nominally passes shell-discrimination threshold
    (|LEARN_ORIG - LEARN_COL_PERM| shell_pmax = 0.0622 > 0.05) but via degenerate
    concentration — both input types collapse to single dominant shells while
    pmax_after drops from ~0.50 to ~0.10 (routing quality destroyed). Effect is
    generic concentration, not semantic fiber discrimination. Fiber balance +
    SO(8) path exhausted. Stage 2 formally redirects to angular routing.

- `INC-0138`:
  `docs/research/increments/INC_0138_geometry_only_shell_activation_controls.md`
  - status: `Closed: REFINE`
  - verdict: fixed H^4 geometry + adaptive shell activation (learn_so8=0, learn_scale=0) produces
    stable 2-shell structure (shell_pmax≈0.58, no collapse). Real embeddings separate from
    Gaussian noise strongly (buckets: 15.5 vs 50.0, pmax_after: 0.53 vs 0.05) but NOT from
    column-permuted controls at the shell level. Shell assignment is norm-driven. Angular/Hopf
    dimension is the primary carrier of semantic structure.

- `INC-0137`:
  `docs/research/increments/INC_0137_measure_consistent_h4_hopf_shell_pressure_blend.md`
  - status: `Closed: KILL`
  - verdict: bounded geodesic-radius blend (w=0.1–0.4) worsens shell_pmax at all weights
    vs the chart-only HOPF_BASE_K25_PHI baseline (0.5222); w=0.1 passes health but
    degrades pmax to 0.7464; w≥0.2 collapses to 1 shell. Radius-interpolation is not
    the right lever. Next increment must target an occupancy-feedback density controller.

- `INC-0136`:
  `docs/research/increments/INC_0136_measure_consistent_h4_hopf_route_return.md`
  - status: `complete, negative/explanatory`
  - read:
    - direct geodesic shell substitution via `phase4d_hopf_base_ball` failed the
      health gate
    - `shell_pmax=0.8862`, `shell_mass_l1=1.7724`, and `knn_overlap=0.6362`
      were all worse than the healthy Hopf-base control
    - the next honest correction is narrower shell-pressure blending, not more
      downstream packet work

## Earlier-Stage Justification
- Earlier unresolved stage:
  `hyperbolic embedding stability`
- Justification:
  `the architecture-level embedding question remains partial, but the current repo's live fixed-embedding harness makes RR-061 the next software-side falsification gate; we should close the route law before resuming new downstream translated frontier refinement`

## Frozen Supporting Evidence
- Supporting evidence index:
  `docs/research/SUPPORTING_EVIDENCE.md`
- Frozen supporting branch line:
  `INC-0130` through `INC-0134`
- Carry-forward note:
  `these translated sparse-event lower-bank and dual-anchor results remain valid downstream evidence, but they are provisional if RR-061 materially changes the route law`

## Deferred Branches
- `INC-0135`:
  `docs/research/increments/INC_0135_product_phase_sparse_event_translation_lower_bank_quality_systems_frontier.md`
  - reason: `supporting lower-bank translated frontier follow-up after the geometry return`

## Current Evaluation References
- Geometry references:
  - `HOPF_K25_BASE_PHI`
  - `HOPF_BASE_K25_PHI`
  - `HOPF_PHI2_BAND_PHI`
- Supporting downstream references:
  - lower-bank systems default:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500`
  - lower-bank balanced comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500`
  - lower-bank quality-first comparator:
    `CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500`
  - upper-bank default:
    `CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000`
