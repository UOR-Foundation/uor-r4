# INC-0061: Measure-Consistent `H^4` / Hopf Route Law

## Status
Queued next.

## Trigger
`INC-0060` established that geometry routing is real enough to keep pursuing:
- both routed Hopf families beat the collapsed `R0` control on proxy quality and runtime
- widened Hopf improved geodesic neighborhood preservation

But it also showed the current route law is still mathematically off:
- shell occupancy is far from `H^4` shell mass
- Hopf angular mass remains concentrated, especially in `theta1/theta2`

## Hypothesis
The next correction should be structural, not another controller repair:
- shell boundaries should follow hyperbolic measure more directly
- angular binning should be derived from Hopf coordinates with explicit equal-mass treatment

If that is right, then a measure-consistent route law should improve:
- shell-mass diagnostics
- Hopf angular-mass diagnostics
- geodesic-neighborhood preservation

without giving back the current routed task win.

## Minimal Scope
1. Add a measure-consistent shell law derived from `H^4` shell mass.
2. Add a measure-consistent Hopf angular law with explicit equal-mass `chi` handling.
3. Keep the current cheap static training schedule fixed.
4. Screen against:
   - `HOPF_K25_BASE_IT40_P2_STATIC`
   - `HOPF_PHI2_BAND_IT40_P2_STATIC`
   - the new measure-consistent route variant(s)
5. Evaluate both task and geometry metrics.

## Acceptance
- materially reduces shell-mass error and/or Hopf angular-mass error
- preserves or improves geodesic neighborhood overlap
- stays within the current routed quality/runtime band

## Scope Guardrail
- Do not open the event-driven / gated-intelligence branch yet.
- Do not move to spectral claims until the measure-consistent route law is tested.
