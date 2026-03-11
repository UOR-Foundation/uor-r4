# INC-0062: Hopf-Base Angular Route Law

## Status
Complete.

## Trigger
`INC-0061` showed that shell-only measure correction is not enough:
- raw equal-mass `H^4` shells fail
- bounded `H^4`-mass shells also fail

The local theory corpus points to a stronger structural split:
- coarse routing should live on the Hopf base
- common fiber phase should remain separate

## Hypothesis
The current route law is using the wrong angular variables for coarse routing.

Instead of routing directly on `(theta1, theta2)`, the coarse route should use the Hopf-base coordinates:
- `eta` / `chi`
- `delta`

and keep
- `alpha`

as the fiber phase.

If that is right, then coarse routing on the Hopf base should:
- improve angular measure behavior
- preserve or improve route health
- create a cleaner foundation for later phase-necessity testing

## Minimal Scope
1. Add a Hopf-base sector mode that routes on coarse base coordinates.
2. Keep the current cheap routed training schedule fixed.
3. Screen against:
   - `phase4d_hopf`
   - `phase4d_hopf_fib_band`
   - the new Hopf-base angular law
4. Measure:
   - proxy quality/runtime
   - Hopf angular-mass diagnostics
   - geodesic neighborhood preservation
   - route health

## Screen
- Config:
  - `configs/proxy_transfer_inc0062_hopf_base_screen.json`
- Analysis:
  - `results/analysis/inc0062_hopf_base_screen.json`
- Gate:
  - `docs/governance/gates/gate_20260310_233347.md`

2-seed means:
- `HOPF_BASE_K25_PHI`
  - `mse=0.003900`
  - `total=6.057s`
  - `buckets=22.5`
  - `sectors=15.0`
  - `shell_pmax=0.522`
  - `hopf_base_mass=1.0218`
- `HOPF_K25_BASE_PHI`
  - `mse=0.003903`
  - `total=6.751s`
- `HOPF_PHI2_BAND_PHI`
  - `mse=0.003905`
  - `total=6.188s`
- `R0`
  - `mse=0.003916`
  - `total=7.472s`
  - shell-collapse fail

Reading:
- the Hopf-base route law is viable and healthy
- it improved quality and runtime vs `R0`
- it also beat both routed references on the 2-seed screen
- that was strong enough to justify a 4-seed confirm

## Confirm
- Config:
  - `configs/proxy_transfer_inc0062_hopf_base_confirm.json`
- Analysis:
  - `results/analysis/inc0062_hopf_base_confirm.json`
- Gate:
  - `docs/governance/gates/gate_20260310_233631.md`

4-seed means:
- `HOPF_K25_BASE_PHI`
  - `mse=0.003895`
  - `total=6.763s`
  - `hopf_base_mass=1.0168`
  - `knn_overlap=0.7365`
- `HOPF_PHI2_BAND_PHI`
  - `mse=0.003903`
  - `total=6.503s`
  - `hopf_base_mass=0.9963`
  - `knn_overlap=0.8241`
- `HOPF_BASE_K25_PHI`
  - `mse=0.003906`
  - `total=6.412s`
  - `buckets=21.0`
  - `sectors=14.25`
  - `shell_pmax=0.5378`
  - `sector_pmax=0.6021`
  - `hopf_base_mass=1.0234`
  - `knn_overlap=0.6932`
- `R0`
  - `mse=0.003911`
  - `total=7.656s`
  - shell-collapse fail

## Conclusion
- `phase4d_hopf_base` is real and healthy.
- It is the fastest healthy routed variant in this confirm.
- It did **not** become the routed quality lead; pure Hopf kept that.
- It also did **not** validate the stronger angular-correction hypothesis by itself.

The main reason is diagnostic:
- the new Hopf-base mass metrics are mostly coordinate-distribution diagnostics on the shared charted manifold
- they do not move enough across same-chart route families to prove that the base-space address law is closer to the intended measure law

So the durable outcome is narrower:
- `phase4d_hopf_base` is now the correct no-fiber-phase coarse-address control
- it provides the clean baseline for phase-transport necessity testing
- the project should not yet claim that coarse Hopf-base routing is the final angular law

## Acceptance
- route-health preserved: yes
- stayed inside the routed quality/runtime band: yes
- cleaner base/fiber interpretation: yes
- proved the angular-correction hypothesis outright: no
