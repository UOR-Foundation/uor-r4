# Route Matrix

## Stage policy
- Screen: 1-2 seeds
- Confirm: 2 seeds
- Finalize: 4 seeds

## Routes
- `R0`: `sector_mode=kmeans`, `scale_mode=radial`, `time_pressure_lambda=0.0`
  - Hypothesis: strongest control baseline.
  - Current status: standing transfer control, but collapsed on shells.
  - Kill: only replaced as control if a better control definition is needed.

- `R5`: `sector_mode=phase4d`, `phase4_dims=i,j,k,l`
  - Hypothesis: 4D polar structuring improves long-tail routing.
  - Current status: best-known synthetic route family via `R5B`.
  - Kill: no confirm-stage gain.

- `R8`: `sector_mode=phase4d_adaptive`
  - Hypothesis: time-expanded phi-balanced widening prevents angular collapse while preserving the quality/runtime benefit of `phase4d`.
  - Current status:
    - fixed-controller branch exists as historical comparator (`D30_FIXED_SG16`)
    - continuous `phi_ratio` branch exists as historical comparator (`PHI_D32_L120`, `PHI_D30_L120`)
    - `PHI_PHI_PHI v1` is the current lead family:
      - normalized artifact label: `PHI3_K25_D36_L065`
      - historical artifact label: `PHILOG_D36_L065`
      - superseded inside the routed family by the phase-coupled `INC-0024` branch
    - `PHASE_K25_C035` is the current routed family lead candidate:
      - beats `PHI3_K25_D36_L065` on confirm-stage quality and runtime
      - still misses the strict runtime gate vs `R0` by a small margin
      - keep it as the routed family lead candidate, not the runtime lead
    - `PHI3_K20_D36_L065` is the active compression comparator:
      - won screen
      - lost 4-seed confirm to `K25`
  - Kill: fails larger-subset control, cannot maintain shell activation, survives only on a narrow and unstable timing edge, or fails seed-wise health review.

- `R6`: `sector_mode=complex2`, `complex_dims=i,j`
  - Hypothesis: complex-plane routing may help as a local discriminator even if it is weak globally.
  - Current status: global use underperformed; local use remains research-only.
  - Kill: unstable or no efficiency improvement.

- `R9`: `sector_mode=phase4d_complex_local`
  - Hypothesis: use `phase4d_adaptive` for coarse routing and local complex zoom for neighborhood refinement.
  - Current status:
    - `INC-0020` rescued the branch with local convergence and `local_min_k=2`
    - `HYB4_M2_T010_C005` is the healthiest hybrid local-quality branch
    - branch is still quality-oriented, not hardware-efficient
  - Kill: any branch that regresses back into unseen-route explosion or cannot justify its runtime cost.

- `R10`: `sector_mode=phase4d_hopf`
  - Hypothesis: direct shell-capacity coupling from capped `H^4` growth plus Hopf-aware pair-bin allocation fixes the remaining global geometry mismatch.
  - Current status:
    - `HOPF_K25_BASE_IT40_P2_STATIC` is the current operational routed lead
    - `HOPF_K25_BASE_IT60_P4_STATIC` is now the historical static reference
    - `HOPF_K25_BASE_IT60_P4` remains the dynamic quality reference
    - cheap routed frontier: `chart_iters=40`, `so8_candidates=2`, `scale_candidates=2`
    - static training-route reuse is now part of the active systems stack for this family
    - still compressed to only about `4` effective sectors
    - cheap static variant now beats cheap `R0` on both quality and runtime
  - Kill: if the phi/Fibonacci lattice cannot widen this branch without losing the quality gain.

- `R11`: `sector_mode=phase4d_hopf_fib_rung`
  - Hypothesis: recurrence-constrained `phi^2` rung forcing can widen Hopf without losing the core quality signal.
  - Current status:
    - `INC-0031` widened Hopf to about `10.5` sectors and `20` buckets
    - active `chi` usage stayed at `2` bins
    - quality stayed close to Hopf and slightly better than `R0`
    - runtime regressed too sharply to promote
    - `INC-0032` threshold gating reduced cost modestly but still left the family far slower than Hopf and `R0`
    - keep as a geometry candidate family, not an operational route
  - Kill: replaced as the main widened-Hopf candidate by `R12`.

- `R12`: `sector_mode=phase4d_hopf_fib_band`
  - Hypothesis: shared-state `phi^2` banding can preserve widened Hopf geometry while cutting the runtime cost of ungated rung forcing.
  - Current status:
    - `INC-0033` kept the widened Hopf signal (`10.5` sectors) and cut runtime sharply relative to `R11`
    - `CTRL-0003` kept the widened quality signal on 4 seeds
    - `INC-0040` reduced schedule rescued runtime
    - `INC-0041` showed the widened efficient lead does not hold under larger load
    - `HOPF_PHI2_BAND_IT40_P2_STATIC` is now the widened cheap routed lead
    - `HOPF_PHI2_BAND_IT48_P3_STATIC` and `HOPF_PHI2_BAND_IT60_P4_STATIC` are historical static references
    - `HOPF_PHI2_BAND_IT60_P4` remains the dynamic widened reference
    - `chi` concentration remained severe
    - keep as the widened efficient route family
  - Kill: if a stronger global-alignment branch replaces it as the better widened Hopf candidate.

- `R13`: `sector_mode=phase4d_hopf_blend`
  - Hypothesis: a Hopf-anchored blended capacity law can widen only where the local geometry is under-allocated, avoiding the global cost of `phi^2` overlays.
  - Current status:
    - `INC-0034` completed
    - the branch widened Hopf to about `8-9` sectors
    - it reduced `chi` concentration relative to `R12`
    - it did not beat `HOPF_K25_BASE` on quality
    - it did not recover the runtime story vs `R0`
    - keep only as a historical negative result
  - Kill: already killed as the primary next branch.

- `R14`: `sector_mode=phase4d_hopf_ball`
  - Hypothesis: keep Hopf angular routing intact, but anchor shells to original-ball geodesic radius to recover global radial structure.
  - Current status:
    - `INC-0035` Slice B completed
    - improved proxy MSE relative to `HOPF_K25_BASE`
    - worsened shell concentration and Poincare alignment
    - keep only as a negative control for shell-only global repair
  - Kill: already killed as the primary next branch.

- `R15`: `sector_mode=phase4d_hopf_iso`
  - Hypothesis: route shells and sectors from a shared rotation-only near-isometric chart coordinate to recover global Poincare alignment.
  - Current status:
    - `INC-0036` completed
    - recovered exact Poincare alignment on the fast screen
    - remained compressed at about `4` sectors / `8` buckets
    - slower than both `HOPF_K25_BASE` and `R0`
    - keep only as a mathematical diagnostic and as a base for the next isometric-band branch
  - Kill: already killed as a standalone promotion branch.

- `R16`: `sector_mode=phase4d_hopf_fib_band_iso`
  - Hypothesis: exact alignment plus shared-state widening can coexist in one route family.
  - Current status:
    - `INC-0037` completed
    - exact Poincare alignment and widened sectors were both recovered
    - runtime regressed too sharply to promote
    - keep only as a mathematical diagnostic
  - Kill: already killed as an operational promotion branch.

- `R17`: `sector_mode=phase4d_hopf_fib_band_bound`
  - Hypothesis: partial chart scale can preserve part of the alignment win without paying the full isometric cost.
  - Current status:
    - `INC-0038` completed
    - bounded scale produced a real alignment/runtime interpolation
    - no bounded point passed the operational gate
    - keep only as a diagnostic family
  - Kill: already killed as an active promotion branch.

## Current Transfer Frontier
- control baseline = `R0`
- `R0` remains shell-collapsed and health-failing on strict confirm
- operational routed lead under larger load = `HOPF_K25_BASE_IT40_P2_STATIC`
- hardware-efficiency routed lead under larger load = `HOPF_PHI2_BAND_IT40_P2_STATIC`
- translated retrieval control = `DENSE`
- translated retrieval amortized lead candidate = `HOPF_RET_P1_Q24`
  - candidate fraction about `0.3511`
  - beat matched dense on the 2-seed screen
  - lost on the 4-seed confirm
  - keep only as translated evaluation evidence, not an operational lead
- translated widened retrieval comparator = `HOPF_PHI2_RET_P1_Q24`
  - stronger pruning but slower amortized systems result
- historical static references = `HOPF_PHI2_BAND_IT48_P3_STATIC`, `HOPF_K25_BASE_IT60_P4_STATIC`
- dynamic quality reference under larger load = `HOPF_K25_BASE_IT60_P4`
- widened dynamic reference under larger load = `HOPF_PHI2_BAND_IT60_P4`
- historical widened reference = `HOPF_PHI2_BAND`
- widened routed-family comparator = `PHASE_K25_C035`
- coarse family reference = `PHI_PHI_PHI v1` (`PHI3_K25_D36_L065` / `PHILOG_D36_L065`)
- compression comparator = `PHI3_K20_D36_L065`
- historical continuous-phi comparator = `PHI_D32_L120`
- routed quality-first comparator = `PHI_D30_L120`
- fixed-controller comparator = `D30_FIXED_SG16`
- hybrid local-quality comparator = `HYB4_M2_T010_C005`

## Next Live Branches
1. reopen the dynamic geometry branch (`INC-0050`)
2. keep translated retrieval as an evaluation harness for future geometry families
3. reopen chart-structure work only if the next geometry branch again produces a systems-positive translated signal
4. sparse / quantized phase-gated shell pilot only if the deeper geometry branch becomes preferable again
5. explicit precomputed Poincare-ball route coordinates only if the learned-chart geometry branch still cannot recover enough efficiency
