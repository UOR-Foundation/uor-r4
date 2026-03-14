# Project Board

## Canonical Active Queue
- Current primary RR: `RR-067`
- Current primary INC: `INC-0160` — TBD. Stage 7: hardware-efficiency confirmation.
- 2026-03-13 INC-0154 REFINE: Event-gate efficiency routing-agnostic (gate_mean delta <0.1pp, ORIG vs PERM).
- 2026-03-14 INC-0153 REFINE: Per-sample spectral-gate r ≈ 0.47–0.53 but geometry-agnostic (delta < 10pp).
- 2026-03-14 INC-0152 REFINE: Gate saturated at INC-0125 params. Spectral roughness ↔ error confirmed.
- 2026-03-13 INC-0151 KEEP: 4-seed finalize confirmed. Stage 5 PARTIAL-PASS (strong).
  true_margin_lowfreq +40–48%, label_indicator_max +57%, sector +72–77%.
- 2026-03-13 INC-0149 KEEP: Task-signal smoothness confirmed (screen). error_indicator +109%, true_margin +28–134%.
- 2026-03-13 INC-0148 KEEP: Geometry-native operator confirmed. poincaré-4d +91–95% sector alignment.
- 2026-03-13 INC-0145 KEEP: HOPF_FULL rel_diff=40.7% (stable). Geometry-induced theta_shift on phase angles. Stage 4 → PARTIAL-PASS.
- 2026-03-13 INC-0144 KEEP: Hopf fixed (phase4d_hopf_base) vs K-means adaptive. HOPF rel_diff=31.2%
  (stable), KMEANS rel_diff=3.1% (variable). Stage 3 → PARTIAL-PASS.
- 2026-03-13 INC-0143 KEEP: 4-seed finalize. rel_diff=38.5% (range 30.6%–54.6%), all seeds pass. Stage 2 closed.
- 2026-03-13 INC-0142 KEEP: PPMI-SVD semantic embeddings confirm H^4 Hopf routing discrimination (rel_diff=31.2%, z≈4.2, 2 seeds). Stage 2 → PARTIAL-PASS.
- INC-0141: Closed KILL (2026-03-13) — optimal-dim (46,117,62,78) Hopf routing also indistinguishable from col-perm (rel_diff=0.025, wrong direction); hash embedding proxy-task-blocked
- INC-0140: Closed KILL (2026-03-13) — angular routing indistinguishable from col-perm on L2-normalized embeddings (rel_diff=0.004)
- INC-0138: Closed REFINE (2026-03-13) — geometry-only confirmed; norm-driven shell finding

## Now
- `RR-050` Dynamic hyperbolic state branch (`H^4 + T_xH^4` vs `H^4 x H^4`)
- `RR-059` Lock the coupled `H^4 x H^4` branch contract
- `RR-061` Derive a measure-consistent `H^4` / Hopf route law

Queue reset note:
- 2026-03-12 audit restored `RR-061` as the next primary research gate.
- Later translated sparse-event frontier work remains valid supporting evidence,
  but it is not the main next proof step.
- 2026-03-12 `INC-0136` then closed negative/explanatory:
  direct geodesic shell substitution with Hopf-base sectors failed the health
  gate, so the next primary branch is now the narrower shell-pressure blend
  correction in `INC-0137` (Closed: KILL — radius blend falsified 2026-03-13).

## Next
- `RR-069` Translate the confirmed product phase-field branch into the routed retrieval harness
  - deferred behind the `RR-061` geometry return
- `RR-053` Package routed retrieval index reuse only if a future translated branch clears confirm

## Done Recently
- `RR-068` Probe residual and task-error signals on the confirmed operator family
- `RR-067` Probe direct task-label signals on the confirmed operator family
- `RR-066` Measure the route-graph operator on the confirmed product phase-field family
- `RR-065` Extend corrected phase evidence into the explicit product phase-field branch
- `RR-064` Couple complex-field phase transport into the routing law
- `RR-063` Test phase-transport necessity on top of corrected coarse routing
- `RR-062` Derive the Hopf-base angular route law
- `RR-060` Add `H^4` / Hopf measure diagnostics to the routed frontier
- `RR-058` Recover translated top-1 with exact-bucket complex rerank
- `RR-057` Recover top-1 with hierarchical complex-key backfill
- `RR-056` Translate product complex-key retrieval field
- `RR-055` Product `H^4 x H^4` retrieval-field pilot
- `RR-054` Tangent-flow route law pilot
- `RR-052` Retrieval amortization confirm
- `RR-051` Retrieval amortization screen
- `RR-049` Retrieval cost rescue
- `RR-048` Integration translation
