# INC-0064: Coupled Complex-Field Phase Transport

## Status
Queued next.

## Trigger
`INC-0063` showed that a standalone Hopf-base transport law is not enough.
The transported-phase variants produced zero shell or sector differences versus `phase4d_hopf_base`, so the branch was mechanism-inert on the current proxy schedule.

User clarification sharpens the intended mechanism:
- first `H^4` = routing geometry
- second `H^4` = discrete complex-value field
- the two factors are coupled
- minima routing should come from the first factor
- phase jumping should come from the coupled complex field

## Hypothesis
Phase remains a live claim only if it is induced by the coupled `H^4 x H^4` geometry.
A phase law that couples the second-factor complex field into the transported fiber phase should:
- change addresses relative to `phase4d_hopf_base`
- remain healthier than raw `phase4d_hopf`
- preserve or improve at least one primary routing/retrieval metric without unacceptable cost

## Minimal Scope
1. Keep the first factor on Hopf-base coarse routing.
2. Keep the second factor as the discrete complex-value field.
3. Implement one explicit coupling law where the complex field modulates phase transport.
4. Compare:
   - `phase4d_hopf_base`
   - `phase4d_hopf`
   - new coupled complex-phase route law
5. Measure:
   - proxy quality/runtime
   - route health
   - address-difference versus `phase4d_hopf_base`
   - coupled-field occupancy
   - phase-shift contribution from the complex field

## Acceptance
- coupled complex phase changes addresses materially versus `phase4d_hopf_base`
- route health remains acceptable
- the branch beats either the no-phase control or raw Hopf phase on at least one primary metric
- the gain is attributable to the coupled field rather than a free local heuristic

## Failure Meaning
If the coupled complex-field law still does not move addresses or improve metrics, then the near-term phase claim should narrow sharply:
- geometry routing remains live
- phase remains unproven as a necessary mechanism in this routing stack
- spectral claims should still be tested, but phase should stop driving route-law changes until a stronger mathematical transport law is identified

## First Candidate Law
Use the second-factor complex field to modulate fiber transport directly, rather than using only a base-space connection term.
A practical first family is:
- base routing on `(r, chi, delta)`
- complex field phase `phi_F = atan2(F_j, F_i)`
- transported fiber phase `alpha_tilde = alpha + lambda_conn * A(R) + lambda_field * phi_F`

The exact score law can change, but the branch contract is fixed:
- phase transport must be coupled to the second `H^4`
- it must create real address movement
- it must be judged against the no-phase coarse-address control
