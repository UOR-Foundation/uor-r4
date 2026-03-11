# INC-0062: Hopf-Base Angular Route Law

## Status
Queued next.

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

## Acceptance
- improves angular-mass behavior and/or route health
- stays within the current routed quality/runtime band
- preserves the interpretation:
  - coarse routing on the base
  - phase on the fiber
