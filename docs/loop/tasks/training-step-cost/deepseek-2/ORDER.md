# ORDER: training-step-cost, deepseek-2 (fast nearest rival kernel)

Lab: deepseek. Item: [training-step-cost](../../../../roadmap/items/training-step-cost.md).
Base: `11dd0b623` (depends on PR #3823).

## Goal

Wire the crate's existing pinned `margin_rank_rows` / `margin_rank_grad` CUDA kernels into the
`nearest` training objective behind `--fast-nearest 1`. The kernel selects eight rivals in one
warp per row, descending by score with ties to the lower token id, and computes the same mean
hinge and its gradient directly from the whole-table scores.

The old path remains the default and is the exact reference. This unit adds no CUDA kernel and
does not change `cuda_stack.cu`, `spiralcore_fbs.cu` or any checkpoint fingerprint.

## Cancel probe

At the base, `cuda_stack.cu` contains `margin_rank_rows` and `margin_rank_grad`, both in the
generated stack-kernel list, and their CPU control uses the same descending/lower-id tie rule as
`best_positions`. If either kernel is absent or not admitted, stop. If the fast path differs in
loss or a gradient beyond floating reduction tolerance, keep the flag off and report a negative.

## Done when

1. `train-in-space --fast-nearest 1` sets a thread-local training flag; default 0 preserves every
   existing file and code path. The flag is written to `config.json` only when on.
2. The nearest objective computes whole-table scores, then obtains its per-row eight-rival hinge
   loss from `margin_rank_rows`; the custom backward calls `margin_rank_grad`.
3. Rows are launched as one 32-thread block each. Vocabulary, row count, targets and the eight
   rival cap are checked before launch; no CPU or silent fallback runs under the fast flag.
4. A finite CUDA test compares loss and gradients with and without the flag on random scores,
   masked targets and exact score ties. A CPU test pins that the source kernel's reference ties
   to the lower index.
5. The float-train CPU tests, stub-CUDA all-targets check, Clippy on changed lines,
   `git diff --check`, loop report and local PR checks pass. No training or scoring run is made
   in this unit; device timing is the next unit.

## Stop and ask if

- Wiring the existing kernel would require editing a pinned CUDA source or changing model math.
- The kernel's forward and backward cannot be exposed without a host sync or CPU fallback.
- Any checkpoint's plan fingerprint would move.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/loop/tasks/training-step-cost/deepseek-2/**`
