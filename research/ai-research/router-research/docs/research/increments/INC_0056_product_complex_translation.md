# INC-0056: Product Complex-Key Translation

## Status
Active next.

## Trigger
`INC-0055` confirmed that discrete complex route-key storage on the second `H^4` is efficiency-positive:
- candidate fraction dropped materially
- runtime improved materially
- fallback stayed low

## Hypothesis
The complex-key law should be more valuable in a translated retrieval harness than in a pure proxy-regression harness, because its main signal is candidate addressing efficiency rather than average reconstruction quality.

## Minimal Scope
1. Keep the first `H^4` factor as the coarse routed position field.
2. Keep the second `H^4` factor as the retrieval / imaginary field.
3. Translate the discrete complex key into the routed retrieval harness.
4. Compare:
   - plain Hopf translated retrieval
   - product complex-key translated retrieval
5. Measure:
   - candidate fraction
   - fallback rate
   - online retrieval time
   - amortized retrieval time
   - quality / top-1 drift

## Acceptance
- improves translated candidate pruning or online/amortized retrieval cost materially
- keeps fallback low
- keeps quality loss bounded enough to remain decision-useful
