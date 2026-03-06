# INC-0050: Dynamic H4 State

## Status
Queued.

## Trigger
The current route still treats time mostly as a scalar schedule parameter. If the geometry itself is evolving, a static shell/sector law may be the wrong mathematical object.

## Hypothesis
The next missing piece may be a dynamic state space, and there are now two live formulations:
1. `H^4` position plus tangent-flow state in `T_xH^4`
2. a coupled product geometry `H^4 x H^4`

The stronger user hypothesis is that the advantage may require the second formulation:
- one `H^4` for position / routing location
- one `H^4` for dynamic flow, memory pressure, or convergent/divergent transport state

That would still give 8 real coordinates, but with hyperbolic structure on both halves instead of one hyperbolic half plus one auxiliary Euclidean half.

## Why This Is Not Plain `R^8`
- `R^8` as a flat replacement would throw away the hyperbolic global-alignment structure that has repeatedly mattered in this project.
- The first stronger formulation is a tangent-bundle / geodesic-flow picture:
  - `x = (r, chi, theta1, theta2)`
  - `u = (p_r, p_chi, p_theta1, p_theta2)`
- The even stronger formulation is a product manifold:
  - `(x, y) in H^4 x H^4`
  - `x` carries the routing location
  - `y` carries dynamic pressure, transport state, or memory-field state
- In other words: 8 real state variables, but potentially with hyperbolic structure on both halves.

## Current Mathematical Preference
- Do not assume plain `R^8`.
- Keep `H^4 x H^4` live as a distinct branch, not just a rewording of tangent flow.
- First formal test should compare:
  1. `H^4` plus tangent flow
  2. `H^4 x H^4`
  3. explain which one better matches:
     - time-evolving geometry
     - divergence/convergence
     - Poincare-ball global alignment
     - Hopf angular routing

## Minimal Scope
1. Keep current systems work separate.
2. Write the formal route law for:
   - `H^4 + T_xH^4`
   - `H^4 x H^4`
3. Define what divergence/convergence means in each formulation.
4. Decide whether the second `H^4` should represent:
   - flow / momentum
   - memory pressure
   - imaginary / retrieval field
5. Only then decide whether it needs a code pilot.

## Decision Rule
- Promote this branch only if the formalism explains a failure that the current static route cannot explain cleanly.
- Do not let it interrupt the current translated retrieval cost-rescue branch unless the systems rescue fails.
