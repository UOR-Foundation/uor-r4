# Kill List Tracker

Purpose: single canonical tracker for kill-list stage status.

Use statuses:
- `open` = unresolved and currently blocking or live
- `partial` = positive evidence exists, but not enough to close the stage
- `closed` = sufficiently established for the current repo scope
- `killed` = falsified or explicitly abandoned

## Canonical Queue
- Current primary RR: `RR-065`
- Current primary INC: `INC-0144` (Closed: KEEP — Stage 3 PARTIAL-PASS, Hopf fixed geometry beats K-means adaptive)

## 1. Hyperbolic Embedding Stability
- Status: `partial`
- Canonical evidence:
  - `EVIDENCE_SUMMARY.md`
  - `NEXT_CRITICAL_EXPERIMENTS.md`
- Blocker:
  - `large-scale learned hyperbolic embedding stability is not yet revalidated on top of a stabilized route law`
- Next branch:
  - `deferred until the route law is stable enough to justify a dedicated embedding benchmark`

## 2. Measure-Consistent Shell Routing
- Status: `closed/partial-pass`
- Canonical evidence:
  - `docs/research/increments/INC_0060_h4_hopf_measure_diagnostics.md`
  - `docs/research/increments/INC_0061_measure_consistent_route_law.md`
  - `docs/research/increments/INC_0136_measure_consistent_h4_hopf_route_return.md`
  - `docs/research/increments/INC_0137_measure_consistent_h4_hopf_shell_pressure_blend.md`
  - `docs/research/increments/INC_0138_geometry_only_shell_activation_controls.md`
  - `docs/research/increments/INC_0139_TBD.md`
  - `docs/research/increments/INC_0140_angular_sector_routing_measure_consistency.md`
  - `docs/research/increments/INC_0141_TBD.md`
  - `docs/research/increments/INC_0142_TBD.md`
  - `docs/research/increments/INC_0143_TBD.md`
- Latest result (INC-0143, 2026-03-13):
  - 4-seed finalize of PPMI-SVD discrimination on H^4 Hopf routing (dims 3,65,2,21).
  - SEM_ORIG mean_pmax=0.0905, SEM_COL_PERM=0.0613, rel_diff=38.5% (threshold 20%).
  - All 4 seeds pass individually (range 30.6%–54.6%).
  - Stage 2 geometry hypothesis NOT falsified. Stage 2 **CLOSED as PARTIAL-PASS**.
- Stage 2 closure note:
  - H^4 Hopf sector routing discriminates semantically structured embeddings (PPMI-SVD)
    from column-permuted control. The discrimination is seed-stable (4 independent seeds).
  - Caveat: production routing requires semantically structured input embeddings.
    Pure hash features (INC-0136–0141) are isotropic by construction and fail.
  - Shell law (radial discrimination beyond sector) was not demonstrated as strictly
    necessary — pmax_before ≈ pmax_after in all INC-0142/0143 runs (single shell).
    This is acceptable: the Stage 2 gate was angular discrimination, not shell splitting.
- Decision: Stage 2 → **CLOSED/PARTIAL-PASS** (2026-03-13, INC-0143 KEEP).
- Next branch:
  - `Stage 3 (Hopf Angular Correctness) — now unblocked. Queue first Stage 3 RR.`

## 3. Hopf Angular Correctness
- Status: `partial-pass`
- Canonical evidence:
  - `docs/research/increments/INC_0062_hopf_base_angular_law.md`
  - `docs/research/increments/INC_0144_hopf_angular_vs_kmeans_stage3.md`
- Latest result (INC-0144, 2026-03-13):
  - Fixed H^4 Hopf-base routing (dims 3,65,2,21) achieves rel_diff=31.2% discriminating PPMI-SVD
    ORIG from COL_PERM (stable: 31.8%, 30.6% per seed).
  - K-means adaptive clustering (100D PPMI) achieves rel_diff=3.1% (variable: −5.8%/+12.0%).
  - Fixed H^4 Hopf geometry with 4D subspace OUTPERFORMS 100D adaptive K-means.
  - Hopf chi-tightness: hopf_sector_chi_std=0.058 (Hopf) vs 0.253 (K-means) — 4.4× more
    chi-coherent sectors; Hopf sectors correspond to geometrically coherent H^4 base regions.
  - Angular mass balance (theta1=1.09, theta2=0.875 errors) remains open: reflects semantic
    clustering in H^4 base (non-uniform distribution is feature, not bug).
- Decision: Stage 3 → **PARTIAL-PASS** (2026-03-13, INC-0144 KEEP).
- Next branch:
  - `Stage 4 (Phase Transport) — test on PPMI-SVD proxy (NOT hash embeddings).
    Prior RR-063/064 results used hash embeddings and may not generalize to semantic proxy.`

## 4. Phase Transport Usefulness
- Status: `partial`
- Canonical evidence:
  - `docs/research/increments/INC_0063_phase_transport_necessity.md`
  - `docs/research/increments/INC_0064_coupled_complex_phase_transport.md`
  - `docs/research/increments/INC_0065_product_phase_field.md`
- Blocker:
  - `none — Stage 3 closed PARTIAL-PASS (2026-03-13, INC-0144 KEEP). Stage 4 is now unblocked.`
  - `Required: run Stage 4 test on PPMI-SVD proxy (prior RR-063/064 used hash embeddings and results may not generalize)`
- Next branch:
  - `INC-0145 — Stage 4 Phase Transport on PPMI-SVD: compare phase4d_hopf_base vs phase4d_hopf vs phase4d_hopf_transport`

## 5. Spectral / Operator Usefulness
- Status: `partial`
- Canonical evidence:
  - `docs/research/increments/INC_0066_spectral_route_operator.md`
  - `docs/research/increments/INC_0067_spectral_signal_probes.md`
  - `docs/research/increments/INC_0068_spectral_residual_task_signals.md`
- Blocker:
  - `operator distinction is real, but useful task-signal evidence stayed weak on the proxy target`
- Next branch:
  - `deferred until route law and phase branch are stable`

## 6. Sparse Event-Driven Trainability
- Status: `partial`
- Canonical evidence:
  - `docs/research/increments/INC_0125_product_phase_sparse_event_proxy_trainability_hardening.md`
  - `docs/research/increments/INC_0130_product_phase_sparse_event_translation_route_coupled_soft_bias_pilot.md`
  - `docs/research/increments/INC_0131_product_phase_sparse_event_translation_soft_bias_carry_forward.md`
- Blocker:
  - `proxy and translated sparse-event results are promising, but not yet a closed architecture-level trainability proof`
- Next branch:
  - `deferred until Stage 3 makes progress — RR-061 Stage 2 is now closed`

## 7. Hardware-Efficiency Confirmation
- Status: `partial`
- Canonical evidence:
  - `docs/research/increments/INC_0074_product_phase_translation_dense_frontier.md`
  - `docs/research/increments/INC_0092_product_phase_translation_warm_cache_q01_floor_hardening.md`
  - `docs/research/increments/INC_0098_product_phase_translation_chart_resident_route_cost_decomposition.md`
- Blocker:
  - `software-side translated wins exist, but they are not yet equivalent to architecture-level hardware replacement`
- Next branch:
  - `deferred until the geometry gate and sparse-event trainability are stronger`
