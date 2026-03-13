# Kill List Tracker

Purpose: single canonical tracker for kill-list stage status.

Use statuses:
- `open` = unresolved and currently blocking or live
- `partial` = positive evidence exists, but not enough to close the stage
- `closed` = sufficiently established for the current repo scope
- `killed` = falsified or explicitly abandoned

## Canonical Queue
- Current primary RR: `RR-061`
- Current primary INC: `INC-0142` (semantic embedding proxy task — does Hopf angular routing discriminate real from col-perm when embeddings have genuine semantic structure?)

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
- Status: `open`
- Canonical evidence:
  - `docs/research/increments/INC_0060_h4_hopf_measure_diagnostics.md`
  - `docs/research/increments/INC_0061_measure_consistent_route_law.md`
  - `docs/research/increments/INC_0136_measure_consistent_h4_hopf_route_return.md`
  - `docs/research/increments/INC_0137_measure_consistent_h4_hopf_shell_pressure_blend.md`
  - `docs/research/increments/INC_0138_geometry_only_shell_activation_controls.md`
  - `docs/research/increments/INC_0139_TBD.md`
  - `docs/research/increments/INC_0140_angular_sector_routing_measure_consistency.md`
  - `docs/research/increments/INC_0141_TBD.md`
- Blocker:
  - `INC-0136–0141 collectively exhaust ALL routing paths on the wikitext2 hash embedding:
    (1) shell routing via geodesic substitution (KILLED), (2) shell pressure blend (KILLED),
    (3) geometry-only controls: r≡1, shells indistinguishable (REFINE), (4) SO(8) chart
    learning: destroys routing quality (REFINE), (5) angular sector routing, default dims
    (0,2,4,6): indistinguishable from col-perm (KILLED, INC-0140), (6) angular sector
    routing, optimal dims (46,117,62,78) with max within-pair correlation |corr|=0.479:
    still indistinguishable (rel_diff=0.025, wrong direction, KILLED, INC-0141).
    Mathematical proof: chi_u and delta are scale-invariant; L2-normalization is irrelevant.
    Root cause: hash embedding is designed to be ISOTROPIC — no 4D Hopf subspace of a
    hash feature can produce semantic angular concentration. Stage 2 is proxy-task-blocked.
    Next path: semantically structured embeddings (GloVe, LM activations) where Hopf
    angular sectors correspond to genuine semantic clusters.`
- Decision: Hash embedding proxy task is permanently blocked for Stage 2. Must switch to
  semantically structured embeddings for any further Stage 2 routing discrimination tests.
- Next branch:
  - `INC-0142: semantic embedding proxy task`

## 3. Hopf Angular Correctness
- Status: `partial`
- Canonical evidence:
  - `docs/research/increments/INC_0062_hopf_base_angular_law.md`
- Blocker:
  - `base/fiber separation is real, but full angular mass allocation still depends on closing RR-061`
- Next branch:
  - `revisit immediately after INC-0137`

## 4. Phase Transport Usefulness
- Status: `partial`
- Canonical evidence:
  - `docs/research/increments/INC_0063_phase_transport_necessity.md`
  - `docs/research/increments/INC_0064_coupled_complex_phase_transport.md`
  - `docs/research/increments/INC_0065_product_phase_field.md`
- Blocker:
  - `phase and coupled-field motion are mechanism-live, but they still rest on an unresolved coarse route law`
- Next branch:
  - `rerun minimal dependency chain only if INC-0137 materially changes the route law`

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
  - `deferred while RR-061 is open`

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
