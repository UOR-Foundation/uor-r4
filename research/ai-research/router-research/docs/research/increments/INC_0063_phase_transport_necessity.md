# INC-0063: Phase-Transport Necessity

## Status
Closed negative at screen stage.

## Trigger
`INC-0062` established the right coarse-address control:
- `phase4d_hopf_base` routes on the Hopf base
- common fiber phase `alpha` is excluded from the coarse address
- the route is healthy and fast

That means the project now has the right baseline to test the user’s core claim:
- geometry itself should force the phase shifts
- phase should survive because it improves routing, not because it is left in as a free score heuristic

## Hypothesis
There is a measurable gap between:
- no fiber phase in the address
- raw phase coordinates in the address
- geometry-induced transported phase

If the theory corpus is directionally right, then a transported-phase law should beat both:
- the no-phase coarse-address control
- the raw-phase routed family

without unacceptable cost or route collapse.

## Minimal Scope
1. Keep the current cheap static proxy schedule fixed.
2. Compare three branches:
   - `phase4d_hopf_base`
   - `phase4d_hopf`
   - a new transported-phase route law
3. Do not reopen event-driven or gated-intelligence branches.
4. Measure:
   - proxy quality/runtime
   - route health
   - phase-specific diagnostics:
     - transported-phase coherence
     - phase ablation deltas
     - whether any gain is attributable to transported phase rather than generic angular access

## Acceptance
- transported phase beats both controls on at least one primary task metric
- route health remains acceptable
- cost stays inside the current routed band
- transported phase changes route addresses relative to the no-fiber-phase control
- phase survives as a geometry-induced mechanism, not as a free heuristic

## Artifacts
- Screen config:
  - `configs/proxy_transfer_inc0063_phase_transport_screen.json`
- Screen analysis:
  - `results/analysis/inc0063_phase_transport_screen.json`
- Address-diff audit:
  - `results/analysis/inc0063_phase_transport_address_diff.json`
- Gate notes:
  - `docs/governance/gates/gate_20260310_235845.md`
  - `docs/governance/gates/gate_20260311_000712.md`

## Result
2-seed screen means:
- `HOPF_BASE_K25_PHI`
  - `mse=0.003900382`
  - `total=6.085s`
  - health pass
- `HOPF_K25_BASE_PHI`
  - `mse=0.003902717`
  - `total=6.712s`
  - health pass
- `HOPF_TRANSPORT_L050`
  - `mse=0.003900382`
  - `total=5.950s`
  - `phase_transport_coherence=0.9817`
  - health pass
- `HOPF_TRANSPORT_L100`
  - `mse=0.003900382`
  - `total=6.010s`
  - `phase_transport_coherence=0.9295`
  - health pass
- `HOPF_TRANSPORT_L150`
  - `mse=0.003900382`
  - `total=6.030s`
  - `phase_transport_coherence=0.8510`
  - health pass
- `R0`
  - `mse=0.003916428`
  - `total=8.149s`
  - health fail

Address-diff audit against `phase4d_hopf_base`:
- `phase4d_hopf_transport` with `lambda in {0.5, 1.0, 1.5}` changed:
  - `0` sector assignments
  - `0` shell assignments
- `phase4d_hopf` changed:
  - `2500` sector assignments
  - `0` shell assignments

## Reading
- The current connection-like transport law is mechanism-inert on the RR-063 proxy schedule.
- It produces phase diagnostics, but no address changes.
- Because it does not move addresses, its neutral task metrics are not meaningful evidence for phase transport.
- `phase4d_hopf_base` remains the correct no-fiber-phase control.
- Pure `phase4d_hopf` still shows that raw fiber phase can change addresses, but the transported law does not.

## Decision
- Close `INC-0063` negative at screen stage.
- Do not promote the standalone Hopf transport law to confirm.
- Narrow the claim precisely:
  - geometry routing remains live
  - this specific base-only transported-phase law is not a necessary mechanism
  - phase remains live only through the coupled `H^4 x H^4` complex-field branch

## Failure Meaning
If transported phase does not beat both controls, the project should narrow its near-term claim:
- geometry routing remains live
- standalone base-only phase transport remains unproven as a necessary computational mechanism
- the next valid phase branch must couple the second `H^4` discrete complex field into the phase law
- spectral/event-driven branches should still wait on a phase law that is mechanically non-inert
