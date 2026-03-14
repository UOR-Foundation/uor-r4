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
- Current primary RR: `RR-067`
- Current primary INC: `INC-0163` -- **TBD** (Stage 7: finalize or next increment).
- Previous INC: `INC-0162` -- **Closed: KEEP** (2026-03-14). Routing compute scaling
  law. 6 K values (25-200), 5 seeds, 120 runs (40 new + 80 reused). Scaling exponents:
  TRANS ORIG alpha=0.50 (square-root), TRANS PERM alpha=0.79, BASE PERM alpha=0.98.
  Compression at K=200: TRANS 2.09x, BASE 1.43x. Phase transport amplifies
  geometric concentration. Stage 7: PARTIAL-PASS (strong, scaling law established).
- Previous INC: `INC-0161` -- **Closed: KEEP** (2026-03-14). Multi-seed routing cost
  confirm. 5 seeds, 4 K values, 80 runs. Effective bucket ratio (5-seed mean): K=25
  1.18x, K=50 1.36x, K=75 1.69x, K=100 1.97x. Gini ratio 1.63-2.11x. Ultra-low
  variance (std < 0.02). All seeds > 1.0x. Compression at 3/4 K values.
  Stage 7: PARTIAL-PASS (replicated 5-seed structural training sparsity).
- Previous INC: `INC-0160` -- **Closed: KEEP** (2026-03-14). Training routing cost screen.
  Effective bucket ratio PERM/ORIG: 1.67× (TRANS), 1.35× (BASE). Training Gini ratio
  ORIG/PERM: 1.59× (TRANS), 1.89× (BASE). Top-half concentration: ORIG 94% vs PERM 78%
  (TRANS). Training sparsity matches eval sparsity (Gini within 0.02). Stage 7: PARTIAL.
- Previous INC: `INC-0159` — **Closed: KEEP** (2026-03-14). Routing sparsity screen
  (seed 0). ORIG routing concentrates label signal into 3.68× (BASE) / 2.14× (TRANS)
  more high-purity buckets at t=0.15. Gini: ORIG 2.12× / 1.65× more concentrated.
  Spectral: lowfreq_max ratio 1.50× (replicates INC-0156). TRANS amplifies all effects.
  Info_density metric withdrawn (MI bounded by H(sector) in concentrated routing).
  Stage 6 → PARTIAL-PASS (strong).
- Previous INC: `INC-0158` — **Closed: KEEP** (2026-03-14). 4-seed finalize of bucket
  coherence + spectral compression. Purity: ORIG > PERM at 10/11 K values (91%),
  TRANS K=100 ratio 1.976×, all K ≥ 25 stable at 4 seeds. Entropy: ORIG < PERM at
  10/11 K (TRANS K=100: Δ = −0.955 bits). label_indicator_lowfreq_max: 4-seed mean
  ratio 1.688 (high variance). true_margin compression 2.40±0.97 at K ≤ 25.
  Stage 6: PARTIAL-PASS (finalized for bucket coherence).
- Previous INC: `INC-0157` — **Closed: KEEP** (2026-03-13). 2-seed confirm.
- Previous INC: `INC-0156` — **Closed: REFINE** (2026-03-13). Spectral compression at 1 seed.
- Previous INC: `INC-0155` — **Closed: REFINE** (2026-03-13). MSE not a valid observable.
- Previous INC: `INC-0154` — **Closed: REFINE** (2026-03-13). Event-gate routing-agnostic.
- Previous INC: `INC-0153` — **Closed: REFINE** (2026-03-14). Per-sample spectral ↔ gate
  geometry-agnostic.
- Previous INC: `INC-0152` — **Closed: REFINE** (2026-03-14). Gate saturated.
- Current primary increment doc:
  `docs/research/increments/INC_0160_sparse_event_training_efficiency_matched.md` (closed: KEEP)
- Kill-list stage: `Hardware-efficiency confirmation` (Stage 7 -- PARTIAL-PASS strong, active gate)
- Stage 7 definition: Routing compute scaling law. TRANS ORIG scales as K^0.50
  (square-root), PERM scales as K^0.79-0.98 (near-linear). Compression ratio grows
  with K. At K=200, TRANS ORIG uses 57 effective buckets vs 120 (2.09x).
- Mathematical object under test:
  `Routing compute scaling law: effective_buckets ~ K^alpha.
   TRANS ORIG alpha=0.50, TRANS PERM alpha=0.79, delta=0.295.
   Phase transport halves the scaling exponent relative to permuted baseline.
   Stage 7: PARTIAL-PASS (strong, scaling law established).`
- Success condition: TBD (next increment)
- Falsification condition: TBD (next increment)

## Latest Closed Increment
- `INC-0146`: **Closed: KEEP** (2026-03-13, Stage 4 PARTIAL-PASS confirmed).
  `docs/research/increments/INC_0146_phase_transport_k75_refine.md`
  - verdict: HOPF_TRANS K=75 (kalpha=3) rel_diff=65.1% (stable: 68.7%, 61.2%).
    HOPF_BASE_K75=46.7%. Fiber transport adds +18.4pp over same-K base. Both Stage 4
    mechanisms confirmed (HOPF_FULL 40.7% INC-0145 + HOPF_TRANS 65.1% INC-0146).
    K=50 confirmed to NOT resolve bin dilution (kalpha=2); K=75 is the correct threshold.

- `INC-0146`: **In progress** — Stage 4 REFINE, HOPF_TRANS K=75/K=100 (RR-067)
- `INC-0145`: **Closed: KEEP** (2026-03-13, Stage 4 PARTIAL-PASS)
  `docs/research/increments/INC_0145_phase_transport_fiber_stage4.md`
  - verdict: HOPF_FULL rel_diff=40.7% (stable). Geometry-induced theta_shift on phase angles
    improves routing by 30% relative over Hopf-base. Stage 4 → PARTIAL-PASS.
    HOPF_TRANS (K=25 triplet) variable: bin dilution, REFINE needed at K=50.

- `INC-0144`:
  `docs/research/increments/INC_0144_hopf_angular_vs_kmeans_stage3.md`
  - status: `Closed: KEEP`
  - verdict: Fixed H^4 Hopf-base (dims 3,65,2,21) rel_diff=31.2% (seeds 31.8%/30.6%).
      K-means (100D) rel_diff=3.1% (variable: −5.8%/+12.0%). Hopf 4D fixed >> K-means 100D adaptive.
      Fixed geometry is essential; adaptive clustering cannot discriminate col-perm.
      Stage 3 → PARTIAL-PASS. Stage 4 (Phase Transport on PPMI-SVD) is next.

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
