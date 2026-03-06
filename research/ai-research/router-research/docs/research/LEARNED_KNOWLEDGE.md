# Learned Knowledge

This file stores durable mathematical and architectural findings that should survive across sessions, branches, and handoffs.

## Geometry
- Poincare-ball global alignment is foundational, not an optional visualization detail.
- Pure Hopf-aware routing repeatedly carries the cleanest routed-quality signal.
- `phase4_dims=0,2,4,6` is a stable clue, not a random projection choice.
- Static shell and sector laws were too separable; true `H^4` behavior couples radial and angular capacity.
- `pi` belongs to angular periodic geometry.
- Hyperbolic / exponential law belongs to continuous radial growth.
- `phi` helps most as a discrete branching/control constant, not as a universal replacement constant.
- `log(phi)` is meaningful for additive ladder-like shell or convergence pressure.

## Dynamic Geometry
- The project should not collapse the next dynamic branch into plain `R^8`.
- The two live dynamic candidates are:
  - `H^4 + T_xH^4`
  - `H^4 x H^4`
- When the product branch is discussed, the intended object is `H^4 x H^4` in hyperbolic polar structure on both factors, not flat Euclidean 8D polar coordinates.
- `H^4 x H^4` remains a real hypothesis when the second hyperbolic factor has a distinct job:
  - transport state
  - memory pressure
  - convergence/divergence field
  - retrieval / imaginary field
- `INC-0050` Slice A confirmed that dynamic state is not just theory:
  - tangent surrogate `H^4 + T_xH^4` improved proxy MSE and runtime over static `H^4`
  - product surrogate `H^4 x H^4` also improved over static on proxy MSE, but its cleaner signal was stronger top-1 rather than best MSE
- Current reading:
  - tangent flow is the primary next implementation path
  - product `H^4 x H^4` remains a secondary retrieval / discrete-decision branch
- `INC-0054` added a stronger locality fact:
  - static Hopf route keys cut candidate fraction to about `0.34` with zero fallback on ordered proxy windows
  - same-bucket pruning alone loses MSE vs global dynamic search
  - tangent flow repairs part of that loss
  - product `H^4 x H^4` keeps the strongest top-1 signal under bucketed retrieval
- New live hypothesis:
  - route keys may want discrete storage in a complex / imaginary field associated with the second `H^4`
  - that is a better fit for the product branch than for the tangent-flow branch
- `INC-0055` turned that hypothesis into evidence:
  - discrete complex route-key storage on the second `H^4` cut candidate fraction further, from about `0.334` to `0.267`
  - runtime improved materially
  - fallback remained low
  - the tradeoff is bounded quality loss, not route failure
- `INC-0056` showed that the same law survives translation into the routed retrieval harness:
  - translated candidate fraction dropped from about `0.351` to `0.210`
  - translated online and amortized retrieval cost improved
  - fallback stayed at zero
  - plain Hopf and dense still hold a small top-1 advantage
- Current reading:
  - discrete complex / imaginary keys are now evidence-positive at two levels:
    - product `H^4 x H^4` retrieval-state evaluation
    - translated routed retrieval
  - the main remaining weakness is recall/backfill, not address collapse
- Current reading:
  - plain product bucket is the quality/top-1 reference
  - product complex key is the discrete-key efficiency reference

## Systems
- Route-health failures in `R0` can make it look fast while still being geometrically unhealthy.
- `train_route_mode=final_static` was the decisive systems fix for the large-subset EMA bottleneck.
- Current translated retrieval evidence is useful but not operationally promoted; offline build cost still dominates.
- Same-bucket locality can be measured cleanly with:
  - `retrieval_candidate_count_mean`
  - `retrieval_candidate_fraction_mean`
  - `retrieval_probe_bucket_mean`
  - `retrieval_bucket_fallback_rate`

## Research Discipline
- Results need to be recorded in docs and structured artifacts immediately, not reconstructed later from memory.
- Every new branch should leave enough math and implementation context that another session can resume without re-deriving the branch intent.
- Deep math branches should first prove signal with a minimal surrogate before rewriting the main route law.
- `INC-0057` partial result:
  - naive hierarchical coarse backfill on top of the exact complex key remains materially expensive even after removing the obvious repeated set-diff bug
  - low-margin selective backfill over-triggers and becomes operationally dead
  - small-bucket selective backfill triggers too rarely to change top-1
  - this suggests recall repair should avoid candidate expansion and prefer reranking or other no-expansion local repair inside the exact complex bucket
