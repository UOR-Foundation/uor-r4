# REPORT: training-step-cost, deepseek-2 (one-pass exact rival selection)

Lab: deepseek. Item: [training-step-cost](../../../../roadmap/items/training-step-cost.md).
Order: [ORDER.md](ORDER.md). Base: `11dd0b623` (after PR #3823).

## What did not work, first

- **The GPU parity and timing gates have not run yet.** The new selector is off by default; no
  training or evaluation path uses it until that CUDA test passes on an RTX 5090.
- **The first design would not have preserved gradients.** Replacing the nearest objective with
  the existing row-loss kernel acted on a deliberately detached score table, so the loss would
  have carried no gradient to the query or table. The unit now replaces only the exact position
  selection; the established differentiable rival and target path is unchanged.
- **The first draft changed the claim to a fast nearest loss and referenced the wrong kernels.**
  The delivered form is a selector, not a new loss, and it uses its own CUDA module.
- **Candle CUDA argmax is not deterministic on tied scores.** The first device parity run made
  an all-equal row and returned `[0,8,4,2,10,6,1,9]`; the new selector returns
  `[0,1,2,3,4,5,6,7]`. Non-tied rows agree. The flag is therefore an intentional deterministic
  tie mode, not a bit-identical drop-in on ties, and remains off by default.
- **The selector expands the item's scope.** Its own module needs `build.rs`, `lib.rs`,
  `cuda_stack_kernels.rs` and two new source files. The item file records that scope in this
  branch; the policy checker reports it as an advisory because it reads the pre-change scope at
  the merge base.

## What changed

- **`top8_positions.cu`:** one warp per row. Each lane keeps the top eight allowed positions of
  its strided scan; lane 0 merges the 32 lane lists. Order is score descending, token id
  ascending on ties, the same order as repeated `argmax`.
- **`top8_positions.rs`:** a Candle custom op that admits contiguous F32 scores and a U8 allowed
  mask, checks shape and the eight-rival cap, and launches one 32-thread block a row. It refuses
  the CPU and performs no fallback.
- **`in_space_long.rs`:** `--fast-rivals` is a thread-local flag read by `best_positions`. Off,
  the old repeated argmax code runs unchanged. On, the new selector returns the same ordered
  position tensor.
- **`train-in-space.rs`:** `--fast-rivals 1` applies only to `--out` training with a nearest-style
  objective. It is absent from `config.json` when off.
- **The build:** `top8_positions_f32.ptx` is its own ahead-of-time module. The pinned
  `cuda_stack.cu` and `spiralcore_fbs.cu` are byte-unchanged, so checkpoint fingerprints do not
  move.
- **Tests:** a CPU test compares the host model of the warp merge with repeated argmax, with ties,
  masks and fewer than eight allowed entries. A CUDA test checks exact agreement on non-tied rows
  and the lower-id rule on a fully tied row.

Size: 9 files, 450 insertions, 31 deletions in the code commit.

## Measurements

No device timing has run. The unit proves selection and configuration only; it makes no speed
claim until the RTX test and a b64L A/B complete.

## Checks run at the head

Code head: `db34e4475`.

- `cargo test --release --offline --locked -p uor-r4-float-train`: 296 library tests, 7 snapshot
  tests, 9 trainer tests, 2 dependency/kernel-scan tests and 12 finite controls passed; 0 failed.
- Stub-nvcc CUDA all-targets check: passed with `CUDARC_CUDA_VERSION=12080` and
  `UOR_NVCC_ARCH=compute_120`.
- Clippy on all targets: completed with only the crate's established warnings; none is on a
  changed line.
- `git diff --check`: passed.

Not run: the CUDA selector parity test, any model step, any scoring pass and any timing run.

## Skills used

No system-provided Codex skill applied. The repository's charter, geometry, D37 and loop rules
were used directly.

## Still open

1. Run `the_one_pass_rival_selector_is_the_repeated_argmax_order` on one RTX 5090.
2. Time the old and new selector at the row and vocabulary shape of a (b64L) micro-batch.
3. Run a short matched b64L A/B and require identical loss and variable gradients, then compare
   clean step times.
4. If both pass, give Claude the branch as an opt-in acceleration; default remains off until a
   pre-registered training comparison reads capability and cost together.

## Plain summary

The trainer's nearest-token search still scans the whole vocabulary eight times to pick the same
eight wrong tokens. This branch adds one warp-per-row CUDA pass that selects those eight in the
same order, including the lower-token-id tie rule. It is behind an off-by-default training flag,
and the existing gradient path is unchanged. CPU checks pass; the RTX parity and timing checks
remain before it is usable.
