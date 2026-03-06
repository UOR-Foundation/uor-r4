# INC-0054: Tangent-Flow Route Law Pilot

## Status
Queued.

## Trigger
`INC-0050` Slice A confirm showed that the tangent surrogate `H^4 + T_xH^4` beats static `H^4` on the main proxy MSE objective while staying slightly faster.

## Hypothesis
The cheapest next architectural step is not a full `H^4 x H^4` rewrite.
It is a route law that lets tangent-flow state influence shell/sector allocation or retrieval locality while keeping the current Hopf geometry as the position branch.

## Minimal Scope
1. Keep position geometry fixed to the current Hopf lead.
2. Inject a low-rank flow term derived from sequential past context change.
3. Test whether flow-aware routing or retrieval narrows neighborhoods more cleanly than the static branch.
4. Do not rewrite the whole router manifold until the pilot proves signal.

## Candidate Mechanisms
- shell bias from flow radial component
- sector tie-break or local band choice from flow angular component
- flow-aware retrieval metric on top of static route buckets

## Acceptance
- beats static dynamic baseline on proxy MSE or translated retrieval efficiency
- does not destroy current route health or alignment
