# REPORT: training-step-cost, deepseek-3 (fused L1 rival search)

Lab: deepseek. Item: [training-step-cost](../../../../roadmap/items/training-step-cost.md).
Order: [ORDER.md](ORDER.md). Base: `2b46c7080` (after PR #3825).

## What did not work, first

- The fused scoring and selection path has not run on a CUDA device yet. It remains behind the
  existing off-by-default `--fast-rivals` flag.
- The first selector-only path saved just 1.07% of a synthetic step and changed tied gradients,
  so this unit measures the larger fused search rather than promoting that result.

## What changed

- **`top8_positions.cu`:** `nearest_top8_positions` gives one block a query row. Eight warps
  score table candidates with coalesced width scans and keep local top-eight lists; thread 0
  merges them. The `(rows, vocabulary)` score matrix is never written.
- **`top8_positions.rs`:** a three-input Candle op checks query/table/target shapes, widths up to
  1,024 and the eight-rival cap, then launches 256 threads a row. CPU use is refused.
- **`in_space_long.rs`:** the nearest objective uses fused positions under `--fast-rivals 1`.
  The target score, rival-vector gather, hinge and gradients remain the existing path.
- Tests compare fused positions with the old score-plus-selection path and time both search
  shapes. The prior model-step timing test now exercises the fused path.

No default path, serving path, pinned CUDA source or checkpoint fingerprint changes.

## Measurements

GPU parity and timing are pending.

## Checks run at the head

- CPU all-targets check: passed.
- CUDA parity, timing and the full checks are pending.

## Skills used

No system-provided Codex skill applied.

## Still open

1. Compile and run the fused solver on an RTX 5090.
2. Compare its untied positions with the old search.
3. Measure score-plus-selection and the synthetic nearest step with and without fusion.
4. Keep the flag off and report a negative unless the positions, loss, time and gradient behavior
   support a later controlled training unit.

## Plain summary

The prior selector accelerated only the last part of the rival search. This branch fuses the
whole-table L1 scoring and top-eight selection so the large score matrix is never materialized.
It is off by default and awaits CUDA parity and timing.
