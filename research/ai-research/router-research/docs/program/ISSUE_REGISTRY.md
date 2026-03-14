# Issue Registry

Use this file as the local GitHub-style issue list for router research.

## Canonical Active Queue
- Current primary RR: `RR-067`
- Current primary INC: `INC-0161` — TBD. Stage 7: multi-seed or next increment.
- 2026-03-13 INC-0154 REFINE: Event-gate efficiency routing-agnostic (gate_mean delta <0.1pp, ORIG vs PERM).
- 2026-03-14 INC-0153 REFINE: Per-sample spectral-gate r ≈ 0.47–0.53 but geometry-agnostic (delta < 10pp).
- 2026-03-14 INC-0152 REFINE: Gate saturated at INC-0125 params. Spectral roughness ↔ error confirmed.
- 2026-03-13 INC-0151 KEEP: 4-seed finalize confirmed. Stage 5 PARTIAL-PASS (strong).
  true_margin_lowfreq +40–48%, label_indicator_max +57%, sector +72–77%.
- 2026-03-13 INC-0149 KEEP: Task-signal smoothness confirmed (screen). error_indicator +109%, true_margin +28–134%.
- 2026-03-13 INC-0147 REFINE: Stage 4 mechanism revised. Raw alpha (λ=0) rel_diff=66.7% vs
  full transport (λ=1) at 68.7%: gap=+2.0pp. Fiber alpha confirmed (+20.6pp over base).
- 2026-03-13 INC-0145 KEEP: HOPF_FULL rel_diff=40.7% (stable: 38.6%/42.7%). Stage 4 → PARTIAL-PASS.
- 2026-03-13 INC-0144 KEEP: Hopf fixed (phase4d_hopf_base, 4D) vs K-means adaptive (100D). HOPF rel_diff=31.2%
  (stable: 31.8%/30.6%), KMEANS rel_diff=3.1% (variable: −5.8%/+12.0%). Stage 3 → PARTIAL-PASS.
- 2026-03-13 INC-0143 KEEP: 4-seed finalize. rel_diff=38.5% (range 30.6%–54.6%), all seeds pass. Stage 2 closed.
- 2026-03-13 INC-0142 KEEP: PPMI-SVD semantic embeddings confirm H^4 Hopf routing discrimination (rel_diff=31.2%, z≈4.2, 2 seeds). Stage 2 → PARTIAL-PASS.

## Active
- `RR-050` `[research][math-review][active]`
  - Title: Dynamic hyperbolic state branch (`H^4 + T_xH^4` vs `H^4 x H^4`)
  - Branch: `codex/RR-050-dynamic-h4-state`
  - Canonical doc: `docs/research/increments/INC_0050_dynamic_h4_state.md`
  - Goal: carry dynamic geometry from Slice A confirm into the next implementation slice
- `RR-059` `[research][dynamic-geometry][active]`
  - Title: Lock the coupled `H^4 x H^4` branch contract
  - Branch: `codex/RR-059-h4x4-polar-flow`
  - Depends on: `RR-050`, `RR-055`, `RR-056`, `RR-058`
  - Goal: keep the product branch asymmetric:
    - first factor = routing geometry
    - second factor = discrete complex-value field
    - geometry-induced phase transport
- `RR-060` `[research][measure][done]`
  - Title: Add `H^4` / Hopf measure diagnostics to the routed frontier
  - Branch: `codex/RR-060-h4-hopf-measure-diagnostics`
  - Depends on: `RR-059`
  - Canonical doc: `docs/research/increments/INC_0060_h4_hopf_measure_diagnostics.md`
  - Goal: gate the current frontier with geometry-specific shell, angular, entropy, and geodesic-neighborhood diagnostics
- `RR-061` `[research][measure][active]`
  - Title: Derive a measure-consistent `H^4` / Hopf route law
  - Branch: `codex/RR-061-measure-consistent-route-law`
  - Depends on: `RR-060`
  - Canonical doc: `docs/research/increments/INC_0061_measure_consistent_route_law.md`
  - Current continuation increment:
    `docs/research/increments/INC_0137_measure_consistent_h4_hopf_shell_pressure_blend.md`
  - Goal: correct shell and angular laws before phase or event-driven branches reopen
  - Queue reset note: restored on 2026-03-12 as the next primary gate after a
    root-theory audit found drift into late-stage translated sparse-event
    frontier work
  - Latest result:
    - `INC-0136` direct geodesic-shell substitution failed the health gate and
      worsened shell concentration / neighborhood preservation
    - queue next moves to a narrower shell-pressure blend rather than more
      downstream refinement
- `RR-063` `[research][phase][done]`
  - Title: Test phase-transport necessity on top of corrected coarse routing
  - Branch: `codex/RR-063-phase-transport-necessity`
  - Depends on: `RR-061`, `RR-062`
  - Canonical doc: `docs/research/increments/INC_0063_phase_transport_necessity.md`
  - Goal: prove or kill transported phase as a geometry-induced mechanism
  - Result: the corrected rerun invalidated the old inert negative; standalone
    transported phase is mechanism-live once `alpha` bins are active

## Queued
- `RR-069` `[research][translation][queued]`
  - Title: Translate the confirmed product phase-field branch into the routed retrieval harness
  - Branch: `codex/RR-069-product-phase-translation-eval`
  - Depends on: `RR-065`, `RR-066`, `RR-068`
  - Canonical doc: `docs/research/increments/INC_0069_product_phase_translation_eval.md`
  - Goal: evaluate whether the confirmed product branch is useful as retrieval/addressing geometry even though its proxy task-signal probes stayed negative
  - Status note: valid supporting branch, but deferred behind the restored
    `RR-061` geometry return
- `RR-053` `[systems][translation][queued]`
  - Title: Package routed retrieval index reuse if amortization confirm passes
  - Branch: `codex/RR-053-index-reuse-packaging`
  - Depends on: `RR-052`
  - Goal: turn the amortized crossover into a reusable systems path with persistent offline index artifacts

## Recently Closed
- `RR-068` `[research][spectral][done]`
  - Title: Probe residual and task-error signals on the confirmed operator family
  - Branch: `codex/RR-068-spectral-residual-task-signals`
  - Canonical doc: `docs/research/increments/INC_0068_spectral_residual_task_signals.md`
  - Result: all routed residual/task-error probes stayed negative versus the Hopf controls on the proxy target, so the branch was closed at screen stage
- `RR-067` `[research][spectral][done]`
  - Title: Probe direct task-label signals on the confirmed operator family
  - Branch: `codex/RR-067-spectral-signal-probes`
  - Canonical doc: `docs/research/increments/INC_0067_spectral_signal_probes.md`
  - Result: raw one-hot label projections stayed inconclusive/negative versus the Hopf controls even though the operator distinction survived
- `RR-066` `[research][spectral][done]`
  - Title: Measure the route-graph operator on the confirmed product phase-field family
  - Branch: `codex/RR-066-spectral-route-operator`
  - Canonical doc: `docs/research/increments/INC_0066_spectral_route_operator.md`
  - Result: the confirmed product branch carried a distinct connected low-mode signature relative to the controls
- `RR-065` `[research][phase][done]`
  - Title: Extend corrected phase evidence into the explicit product phase-field branch
  - Branch: `codex/RR-065-product-phase-field`
  - Depends on: `RR-059`, `RR-063`, `RR-064`
  - Canonical doc: `docs/research/increments/INC_0065_product_phase_field.md`
  - Result: explicit asymmetric `H^4 x H^4` phase-field coupling stayed mechanism-live and health-passing through confirm
- `RR-062` `[research][hopf-base][done]`
  - Title: Derive the Hopf-base angular route law
  - Branch: `codex/RR-062-hopf-base-angular-law`
  - Canonical doc: `docs/research/increments/INC_0062_hopf_base_angular_law.md`
  - Result: corrected reruns promoted `phase4d_hopf_base` to the canonical
    no-fiber-phase coarse-address control with stronger base/fiber separation
    evidence, while pure Hopf remained the routed quality lead
- `RR-064` `[research][phase][done]`
  - Title: Couple complex-field phase transport into the routing law
  - Branch: `codex/RR-064-coupled-complex-phase-transport`
  - Depends on: `RR-059`, `RR-063`
  - Canonical doc: `docs/research/increments/INC_0064_coupled_complex_phase_transport.md`
  - Result: corrected same-chart coupling is mechanism-live and health-passing,
    with strong field-shift metrics and material address movement, but it is not
    yet the routed quality lead
- `RR-058` `[research][translation][done]`
  - Title: Recover translated top-1 with exact-bucket complex rerank
  - Branch: `codex/RR-058-product-complex-rerank`
  - Canonical doc: `docs/research/increments/INC_0058_product_complex_rerank.md`
  - Result: simple in-bucket complex-plane reranking kept candidate fraction fixed but did not rescue translated quality cleanly enough to justify promotion
- `RR-057` `[research][translation][done]`
  - Title: Recover top-1 with hierarchical complex-key backfill
  - Branch: `codex/RR-057-product-complex-backfill`
  - Canonical doc: `docs/research/increments/INC_0057_product_complex_backfill.md`
  - Result: broad and margin-triggered backfill were too expensive, while small-bucket backfill was almost inert; the next recall branch should rerank inside the exact complex bucket instead of adding candidates
- `RR-056` `[research][translation][done]`
  - Title: Translate product complex-key retrieval field
  - Branch: `codex/RR-056-product-complex-translation`
  - Canonical doc: `docs/research/increments/INC_0056_product_complex_translation.md`
  - Result: the discrete complex / imaginary key survived translation, cut candidate fraction from about `0.351` to `0.210`, improved translated online and amortized cost, and slightly improved proxy MSE versus plain Hopf while paying a small top-1 penalty
- `RR-055` `[research][dynamic-geometry][done]`
  - Title: Product `H^4 x H^4` retrieval-field pilot
  - Branch: `codex/RR-055-product-h4x4-retrieval-field`
  - Canonical doc: `docs/research/increments/INC_0055_product_h4x4_retrieval_field.md`
  - Result: discrete complex route-key storage became the product efficiency lead, while plain product bucket stayed the quality/top-1 reference
- `RR-054` `[research][dynamic-geometry][done]`
  - Title: Tangent-flow route law pilot
  - Branch: `codex/RR-054-tangent-flow-route-law`
  - Canonical doc: `docs/research/increments/INC_0054_tangent_flow_route_law.md`
  - Result: same-bucket locality was real, but the bucketed tangent branch did not beat the global dynamic baseline on MSE
- `RR-052` `[research][systems][confirm][done]`
  - Title: Confirm translated retrieval amortization crossover
  - Branch: `codex/RR-052-retrieval-amortization-confirm`
  - Canonical doc: `docs/research/increments/INC_0052_retrieval_amortization_confirm.md`
  - Result: screen-stage crossover did not survive 4-seed confirm
- `RR-051` `[research][systems][done]`
  - Title: Measure translated retrieval amortization screen
  - Canonical doc: `docs/research/increments/INC_0051_retrieval_amortization.md`
  - Result: `HOPF_RET_P1_Q24` crossed matched dense narrowly on amortized per-repeat cost
- `RR-049` `[systems][translation][done]`
  - Title: Rescue translated retrieval cost with grouped same-bucket search
  - Canonical doc: `docs/research/increments/INC_0049_retrieval_cost_rescue.md`
  - Result: routed online retrieval became faster than dense exact retrieval, but total still lost due offline build
- `RR-048` `[research][translation][done]`
  - Title: Build the first translated retrieval harness
  - Canonical doc: `docs/research/increments/INC_0048_integration_translation.md`
  - Result: pruning signal survived translation; systems cost dominated
