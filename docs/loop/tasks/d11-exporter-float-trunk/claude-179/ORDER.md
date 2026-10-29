# ORDER: d11-exporter-float-trunk, claude-179 (R6: a fixed-order FBS backward)

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md) (#2820). Base: `0b441db58`.

## Goal

Casey: "fix the exporter". claude-140 found the float trunk's training nondeterministic: one binary on one device gave seed 1's top-1 between 745 and 3,845 of 14,287 rows. Its prime suspects are the float `atomicAdd` sums in the FBS backward (`spiralcore_fbs.cu`, `spiralcore_fbs_arms.cu`), whose order changes run to run. R6 (`pipeline-map.md`) replaces them with a fixed-order reduction, then re-measures the export gap with repeats.

**Design.**
- The backward kernels keep one thread per `(row, output)` but write their contributions to scratch, not `atomicAdd`:
  - for each input slot, its value and target;
  - for each `(path, stage, axis)`, the sum over siblings in order.
- Two new kernels add them in a fixed order:
  - `d_input`: one thread per row, over outputs in order;
  - `d_queries`: one thread per `(output, path, stage, axis)`, over rows in order.
- **The bar is GPU bits equal across repeats.** `build.rs` keeps `nvcc`'s default fused multiply-add on (only `-ftz=false -prec-div=true -prec-sqrt=true` are set), so GPU and CPU differ in the last bits. The CPU references keep their tolerance.

## Cancel probe (run first)

Run at `0b441db58`:
- `spiralcore_fbs.cu:151, :178` and `spiralcore_fbs_arms.cu:114, :141` still `atomicAdd` float gradients.
- No open PR touches them.
- `FBS_SOURCE_SHA256` is a fixed reference digest, not the `.cu` digest. The `.cu` digest is computed at build time into receipts, so no pin breaks.

All held. **Not cancelled.**

## Files in scope

- `crates/uor-r4-float-train/src/{spiralcore_fbs.cu, spiralcore_fbs_arms.cu, geometric_stack_native_cuda.rs, spiralcore_fbs_arms.rs, kernel_controls.rs}` and their CUDA tests;
- `docs/harness/prep/d11-exporter-float-trunk-fixed-order-*`;
- `docs/labs/d11-exporter-float-trunk/**`;
- `docs/research/notebook/m4-serving.md`;
- this task folder.

## Done when

1. **Code.** Both backwards are fixed-order, with no float `atomicAdd` left on a gradient. The stub-nvcc `cargo check --features cuda` passes on the Mac, and the CPU references and their tests pass.
2. **GPU, under a FIT'd prep sheet and a pre-run review.**
   - The new backward gives the same bits across repeats (`the_backward_gives_the_same_bits_on_every_run`, base and arm kernels).
   - The existing pod tests pass.
   - Two trainings of the same recipe on one device give byte-identical checkpoints. If not, the cause is classified as candle's kernels versus ours.
3. One notebook row and a REPORT. The export gap re-measurement with repeats is the next unit.

## Stop and ask if

Stop and decide, never wait: the scratch buffers exceed the GPU's memory at the sheet's shapes. Then reduce in tiles, and record it.
