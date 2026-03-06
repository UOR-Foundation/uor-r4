# INC-0057: Product Complex-Key Hierarchical Backfill

## Status
Active next.

## Trigger
`INC-0056` confirmed that discrete complex / imaginary route-key storage survives translation into the routed retrieval harness:
- candidate fraction improved materially
- online and amortized cost improved materially
- fallback stayed at zero
- top-1 regressed slightly

## Hypothesis
The current translated complex key is a strong address field but a slightly incomplete recall field.
A small coarse Hopf backfill on top of the exact complex key should recover top-1 without giving back most of the candidate-pruning win.

## Minimal Scope
1. Keep exact complex key lookup as the first candidate stage.
2. Add a bounded coarse Hopf backfill stage only when retrieving from the translated harness.
3. Compare:
   - plain Hopf translated retrieval
   - exact complex-key translated retrieval
   - complex-key plus coarse-backfill translated retrieval
4. Measure:
   - candidate fraction
   - fallback rate
   - online retrieval time
   - amortized retrieval time
   - top-1 recovery
   - proxy MSE drift

## Acceptance
- preserves most of the complex-key pruning gain
- improves top-1 versus exact complex-key retrieval
- keeps fallback low
- keeps translated runtime materially below plain Hopf translated retrieval
