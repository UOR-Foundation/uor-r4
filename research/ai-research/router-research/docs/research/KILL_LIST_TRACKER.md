# Kill List Tracker

Purpose: single canonical tracker for kill-list stage status.

Use statuses:
- `open` = unresolved and currently blocking or live
- `partial` = positive evidence exists, but not enough to close the stage
- `closed` = sufficiently established for the current repo scope
- `killed` = falsified or explicitly abandoned

## Canonical Queue
- Current primary RR: `RR-061`
- Current primary INC: `INC-0141` (raw embedding angular routing test — does removing L2-normalization restore angular routing discrimination?)

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
- Blocker:
  - `INC-0136–0140 collectively exhaust ALL fixed-geometry routing paths on L2-normalized
    embeddings: (1) shell routing via geodesic substitution (KILLED), (2) shell pressure
    blend (KILLED), (3) geometry-only controls establish r≡1 and shell-indistinguishability
    (REFINE), (4) SO(8) chart learning destroys routing quality (REFINE), (5) angular sector
    routing via phase4d_hopf_base also indistinguishable from col-perm (KILLED). Root cause:
    L2-normalization collapses embeddings to S^127; fixed Hopf projection onto any 4D
    subspace is near-isotropic (within-pair correlations ≈ −0.04). Next test: raw
    (non-L2-normalized) embeddings.`
- Decision: All fixed-geometry routing variants on L2-normalized embeddings have failed
  to pass the real-vs-col-perm discrimination threshold (|diff|/mean > 0.2). Stage 2
  requires either (a) raw embeddings or (b) a sector law that does not depend on a fixed
  coordinate subspace.
- Next branch:
  - `INC-0141: raw embedding angular routing test (does removing L2-normalization restore
    within-pair Hopf angular signal?)`

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
