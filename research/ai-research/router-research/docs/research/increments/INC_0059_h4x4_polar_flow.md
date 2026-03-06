# INC-0059: Coupled `H^4 x H^4` Polar Flow Field

## Status
Active next.

## Trigger
The translated retrieval branches have established two things:
- discrete complex / imaginary route keys are real and useful
- local repair by candidate expansion or simple in-bucket reranking is not enough to cash in the full advantage

## Hypothesis
The second `H^4` should stop being treated as only a key field or a local correction field.
Instead, the routing position factor and the retrieval / dynamic factor should be coupled directly as a shared polar-flow geometry on `H^4 x H^4`.
That should let time evolution live in the geometry itself rather than only in post-hoc scoring or shell heuristics.

## Minimal Scope
1. Keep the first factor as routed positional geometry.
2. Treat the second factor as a true flow / retrieval manifold, not only a discrete key space.
3. Add a minimal coupled score or transport diagnostic that uses both factors together.
4. Screen against:
   - static Hopf translated retrieval
   - exact complex translated retrieval
   - the new coupled `H^4 x H^4` flow variant
5. Measure:
   - candidate fraction
   - top-1
   - proxy MSE
   - online and amortized retrieval cost
   - any new coupling diagnostics

## Acceptance
- demonstrates a quality or efficiency signal not reachable by the exact-key branch alone
- stays geometrically interpretable
- does not collapse back into a flat Euclidean auxiliary field

## Mathematical Rationale
The project goal is not just cheaper lookup; it is routing geometry that can scale better than brute-force dense hardware allocation.
The current evidence suggests:
- static hyperbolic geometry helps globally
- discrete complex addressing helps locally
- post-hoc local repair is too weak
The next coherent move is therefore to let the product space itself carry more of the routing and retrieval dynamics.
