# INC-0063: Phase-Transport Necessity

## Status
Queued next.

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
- phase survives as a geometry-induced mechanism, not as a free heuristic

## Failure Meaning
If transported phase does not beat both controls, the project should narrow its near-term claim:
- geometry routing remains live
- phase transport remains unproven as a necessary computational mechanism
- spectral/event-driven branches should be delayed until the transport law is better justified
