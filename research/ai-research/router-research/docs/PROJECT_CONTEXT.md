# Project Context: 4D Polar / Hyperbolic Routing

## The mechanism
We route points using:

1) Map points from Poincaré ball to tangent at origin:
   - `v = log_0(x)`
2) Apply chart (optional):
   - `z = chart(v)`
     - rotation `R ∈ SO(d)` (optional)
     - scaling:
       - global diagonal `diag(exp(s))`, or
       - radial-bin diagonal `diag(exp(S[bin(r)]))`
3) Route:
   - `shell = floor(||z|| / delta_r)`
   - `sector = argmax( normalize(z) · C_k )` (k-means on sphere), OR a phase/polar sectoring mode

Then local memory and growth operate within each `(shell, sector)` bucket.

## Global Alignment Constraint
- Poincare-ball structure is a foundational part of the routing architecture, not just a preprocessing map.
- A previous session identified Poincare balls as important for globally aligned routing. That should remain active context for all future geometry branches.
- The current mathematical drift risk is flattening into tangent/chart space and then overfitting shell/sector laws that no longer preserve enough of the original hyperbolic global alignment.
- Future routing laws should either preserve that global alignment from the original `B^4` / `H^4` picture or explain precisely what replaces it.

## "What we learned so far" (from pasted runs)
- Radial scaling + growth budget helps.
- `sector_mode=kmeans` is generally stronger than `sector_mode=phase2` on post-growth test MSE (seed variance exists).
- `time_pressure_lambda` in [0.25, 1.2] tends to hurt pre-growth MSE and doesn't reliably improve post-growth.

## Why we’re moving to Codex
Runs are too slow for manual exploration. We need:
- machine-readable run summaries
- caching + fast dev mode
- automated staged sweeps
- a decisions log

## Implementation status (2026-03-05)
- CLI now includes runtime controls (`fast_dev`, early-stop, cache knobs, run tags).
- Route modes include `kmeans`, `phase2`, `phase4d`, and `complex2`.
- Runs emit `__JSON_SUMMARY__` for parser/summarizer automation.
- Staged sweep and gate-note automation are available via `runs/run_pipeline.sh`.
