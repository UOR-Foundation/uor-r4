# Issue Registry

Use this file as the local GitHub-style issue list for router research.

## Active
- `RR-050` `[research][math-review][active]`
  - Title: Dynamic hyperbolic state branch (`H^4 + T_xH^4` vs `H^4 x H^4`)
  - Branch: `codex/RR-050-dynamic-h4-state`
  - Canonical doc: `docs/research/increments/INC_0050_dynamic_h4_state.md`
  - Goal: carry dynamic geometry from Slice A confirm into the next implementation slice

## Queued
- `RR-054` `[research][dynamic-geometry][queued]`
  - Title: Tangent-flow route law pilot
  - Branch: `codex/RR-054-tangent-flow-route-law`
  - Depends on: `RR-050`
  - Goal: turn the `H^4 + T_xH^4` Slice A win into a route-law or retrieval-law pilot
- `RR-055` `[research][dynamic-geometry][queued]`
  - Title: Product `H^4 x H^4` retrieval-field pilot
  - Branch: `codex/RR-055-product-h4x4-retrieval-field`
  - Depends on: `RR-050`
  - Goal: test whether the second hyperbolic factor is primarily a retrieval / discrete-decision field

- `RR-053` `[systems][translation][queued]`
  - Title: Package routed retrieval index reuse if amortization confirm passes
  - Branch: `codex/RR-053-index-reuse-packaging`
  - Depends on: `RR-052`
  - Goal: turn the amortized crossover into a reusable systems path with persistent offline index artifacts

## Recently Closed
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
