# Current Direction

## Latest Update
- `INC-0052` confirm is complete.
- The translated routed-retrieval crossover did not survive confirm.
- Next primary action:
  - `INC-0050`
  - reopen the dynamic geometry branch (`H^4 + T_xH^4` vs `H^4 x H^4`).

## Current Best-Known Routes
- Synthetic lead: `R5B`
- Transfer control baseline: `R0`
  - still shell-collapsed and health-failing on strict gate
- Current operational routed lead: `HOPF_K25_BASE_IT40_P2_STATIC`
  - `phase4d_hopf`
  - `train_route_mode=final_static`
  - `chart_iters=40`, `so8_candidates=2`, `scale_candidates=2`
  - 4-seed confirm beats `R0` on both quality and runtime near full proxy scale
- Current hardware-efficiency routed lead: `HOPF_PHI2_BAND_IT40_P2_STATIC`
  - `phase4d_hopf_fib_band`
  - `train_route_mode=final_static`
  - 4-seed confirm also beats `R0` on both quality and runtime near full proxy scale
  - slightly worse than pure Hopf on MSE and slightly slower on total at confirm, so it remains the widened systems lead, not the primary one
- Current translated retrieval control:
  - `DENSE`
  - exact dense token-memory retrieval over the LM proxy contexts
- Current translated retrieval candidates:
  - `HOPF_RET_P1`
    - translated retrieval family with the cleanest pruning-preserving systems result
    - not operationally promotable after 4-seed amortization confirm
  - `HOPF_PHI2_RET_P1`
    - still prunes slightly harder than `HOPF_RET_P1`
    - did not cash in the pruning signal under amortization
  - both still lose single-batch total wall-clock because offline chart/index cost dominates
- Historical cheap static references:
  - `HOPF_PHI2_BAND_IT48_P3_STATIC`
  - `HOPF_K25_BASE_IT60_P4_STATIC`

## Current Mechanistic Reading
- `INC-0042` proved the large-subset EMA bottleneck was per-step training rerouting.
- `INC-0043` solved that bottleneck with final static training-route reuse.
- `INC-0044` and `INC-0045` proved chart schedule could be reduced without erasing route health.
- `INC-0046` and `INC-0047` proved the cheap routed frontier survives larger and near-full proxy scale.
- `INC-0048` proved the first translated retrieval path preserves a real pruning signal:
  - `HOPF_RET_P1`: `cand_frac=0.3488`, `total=15.867s`
  - `HOPF_PHI2_RET_P1`: `cand_frac=0.3415`, `total=11.070s`
  - `DENSE`: `cand_frac=1.0`, `total=0.936s`
- `INC-0049` proved the translated loss is now mostly offline cost, not query-time routed search:
  - `DENSE`: `offline=0.000s`, `online=0.879s`, `total=1.332s`
  - `HOPF_RET_P1`: `offline=9.694s`, `online=0.401s`, `total=10.687s`
  - `HOPF_PHI2_RET_P1`: `offline=7.664s`, `online=0.299s`, `total=8.525s`
- `INC-0051` proved the translated path is only interesting under amortization:
  - `DENSE_Q24`: `amortized_per_repeat=0.5545s`
  - `HOPF_RET_P1_Q24`: `amortized_per_repeat=0.5399s`
  - `HOPF_PHI2_RET_P1_Q24`: `amortized_per_repeat=0.6311s`
  - routed crossover exists, but only narrowly and only for plain Hopf so far
- `INC-0052` killed operational promotion of the translated branch:
  - `DENSE_Q24`: `amortized_per_repeat=0.5051s`
  - `HOPF_RET_P1_Q24`: `amortized_per_repeat=0.5938s`
  - `DENSE_Q32`: `amortized_per_repeat=0.5586s`
  - `HOPF_RET_P1_Q32`: `amortized_per_repeat=0.6544s`
- Current timing read at near-full proxy scale:
  - `HOPF_K25_BASE_IT40_P2_STATIC`: `chart_opt=13.057s`, `training_ema=0.194s`, `total=15.904s`
  - `HOPF_PHI2_BAND_IT40_P2_STATIC`: `chart_opt=12.973s`, `training_ema=0.209s`, `total=16.079s`
  - `R0`: `chart_opt=11.659s`, `training_ema=15.907s`, `total=31.826s`
- The translated frontier is still evidence-positive but operationally incomplete.
- The next question is whether a dynamic hyperbolic state geometry can lower translated cost enough to matter.
- The next deep geometry branch is still queued behind systems rescue, but it is now sharper:
  - compare `H^4 + T_xH^4` against `H^4 x H^4`
  - do not flatten this into “just use `R^8`”

## Current Risk
- The win is still proxy-harness evidence, not an end-to-end model claim.
- The cheap routed proxy lead is stable enough to carry forward.
- The translated retrieval harness no longer fails mainly on routed local search.
- It currently fails on single-batch total wall-clock because the offline build is paid every run.
- The widened fast lead remains useful as a second routed family, but pure Hopf is still the clean primary lead.

## Current Stop Conditions
- The translated integration path has now failed operational confirm.
- Reopen geometry, but keep the translated retrieval harness as a future evaluation target.
- Do not describe this as dense-model replacement from proxy evidence alone.
- Do not treat `R0` as healthy even when used as a speed control; it remains shell-collapsed.
