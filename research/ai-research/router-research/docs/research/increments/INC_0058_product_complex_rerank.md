# INC-0058: Product Complex Exact-Bucket Rerank

## Status
Active next.

## Trigger
`INC-0057` showed that hierarchical backfill is the wrong recall repair path for the translated complex-key branch:
- broad coarse backfill is expensive
- low-margin backfill over-triggers and destroys cost
- small-bucket backfill is almost inert and does not recover top-1

## Hypothesis
The remaining translated top-1 gap is more likely a ranking problem than a recall problem.
A rerank term derived from the second `H^4` complex / imaginary field, applied inside the already-pruned exact complex bucket, may improve top-1 without expanding the candidate set.

## Minimal Scope
1. Keep the translated exact complex-key candidate set unchanged.
2. Add an optional rerank term inside routed retrieval:
   - base score: full routed chart similarity
   - rerank correction: local complex-plane similarity on the designated complex dims
3. Screen a small lambda family.
4. Compare against:
   - dense exact retrieval
   - plain Hopf translated retrieval
   - exact complex translated retrieval
5. Measure:
   - top-1
   - proxy MSE
   - candidate fraction
   - online retrieval time
   - amortized per-repeat time

## Acceptance
- improves top-1 versus `HOPF_RET_CPX_P1_Q24`
- preserves candidate fraction within a narrow tolerance
- does not materially worsen amortized per-repeat cost
- keeps fallback at zero

## Mathematical Rationale
If the translated complex key is already a strong address field, then the remaining error is likely within-bucket ordering, not bucket coverage.
That points toward using the second `H^4` as a local ordering field rather than a candidate-expansion field.
This is consistent with the broader project direction:
- global alignment from the hyperbolic / Poincare structure
- discrete addressing from the complex route key
- local repair from the same secondary geometric field, not from coarse candidate inflation
