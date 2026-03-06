# Issue Registry

Use this file as the local GitHub-style issue list for router research.

## Active
- `RR-050` `[research][math-review][active]`
  - Title: Dynamic hyperbolic state branch (`H^4 + T_xH^4` vs `H^4 x H^4`)
  - Branch: `codex/RR-050-dynamic-h4-state`
  - Canonical doc: `docs/research/increments/INC_0050_dynamic_h4_state.md`
  - Goal: carry dynamic geometry from Slice A confirm into the next implementation slice
- `RR-057` `[research][translation][active-next]`
  - Title: Recover top-1 with hierarchical complex-key backfill
  - Branch: `codex/RR-057-product-complex-backfill`
  - Depends on: `RR-056`
  - Goal: keep the translated complex-key pruning gain while repairing recall with a small coarse Hopf backfill

## Queued
- `RR-053` `[systems][translation][queued]`
  - Title: Package routed retrieval index reuse if amortization confirm passes
  - Branch: `codex/RR-053-index-reuse-packaging`
  - Depends on: `RR-052`
  - Goal: turn the amortized crossover into a reusable systems path with persistent offline index artifacts

## Recently Closed
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
