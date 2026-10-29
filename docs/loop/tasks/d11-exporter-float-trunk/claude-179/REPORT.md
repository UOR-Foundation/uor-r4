# REPORT: d11-exporter-float-trunk, claude-179 (R6: a fixed-order FBS backward)

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [d11-exporter-float-trunk-fixed-order-2026-10-29.md](../../../../harness/prep/d11-exporter-float-trunk-fixed-order-2026-10-29.md). References #2820. The item is not done: R6's second half, the export gap re-measured with repeats, is next.

**Speed.** Not a serving result. Training arms took 7.7 to 8.5 minutes each, with B_fixed (7.9, 7.9) no slower than B_base (8.5, 7.7).

## What did not work, first

- **My first job stopped at the first training arm.** Its `step()` set `name` globally, so the arm ran with its env step's save dir, whose parent did not exist. Everything before it had run: the builds, the kernel test, the negative gate and the input checks.
  - A second job (`arms.sh`, with `local` variables) ran the four arms on the same pod, from the verified binaries and inputs.
  - This was an instrument fault, repaired, not a result.
- **`cargo` was not on the pod's non-login PATH.** I found it before the job ran, and the job sets PATH itself.
- **The FIT changed the control.** The first sheet compared against kimi-46's archived binary, which differs from the fixed code by nine more training-path commits. B_base is instead built on the pod from the same source with only the three fix files reverted, so the arms isolate the backward.
- **No test compares the base backward's GPU output with its CPU reference.** Its raw output goes through the secant transform before autograd sees it. The kernel test covers determinism and the arm tests relate the arms to the base path. Correctness otherwise rests on the code review, which mapped every scratch slot to the old `atomicAdd` target, and on B_fixed learning like B_base.
- **`nvcc` keeps fused multiply-add on** (`build.rs` sets only `-ftz=false -prec-div=true -prec-sqrt=true`). So GPU and CPU differ in the last bits, and the bar is equal bits across GPU runs, not equality with the CPU.
- **The device was recorded once per pod, not before each arm** as the sheet's I-Device says; `env` was recorded per arm. On one pod this is enough.
- **The pod cost $0.975/h, not the sheet's $0.6.** About 46 minutes came to about $0.75.
- **`uor-pod down` could not verify the archive over SSH,** as for claude-140. The 145 files were read back from the store first, then `down --confirm-archived` deleted the pod.

## What changed

- **`spiralcore_fbs.cu` and `spiralcore_fbs_arms.cu`:**
  - The backward kernels write each item's contributions to scratch: one input slot and value per lane, and per-stage query sums over siblings in order.
  - New `spiralcore_fbs_reduce_dx` adds them per row, over output blocks in order. New `spiralcore_fbs_reduce_dq` adds them per query slot, over rows in order.
  - No float `atomicAdd` is left on a gradient.
- **`geometric_stack_native_cuda.rs`:** `FbsBackwardScratch` and one `reduce` serve the f32, bf16 and arms modules.
- **`lib.rs` and `tests/no_training_crate_dep.rs`:** the kernel inventory is now 26.
- **`spiralcore_fbs_arms.rs`:** the new pod test `the_backward_gives_the_same_bits_on_every_run` runs the base kernel and the arm kernel with one and two paths, 20 repeats each.
- **The prep sheet and one notebook row.**

## Measurements

One RTX 5090, pod `vast-58203454`: driver 595.71.05, CUDA 12.8 (V12.8.93). Code `470b898ed`. Sealed to `caseyallard/uor-r4-store:results/claude/d11-fbs-fixed-order-2026-10-29/` (145 files, `SHA256SUMS` read back OK).

| Check | Result |
|---|---|
| B_fixed and B_base | sha256 `0ef16e2a…a3b8` and `ca7a64e7…66f9`. `diff -r` of the crate's `src`: exactly the three fix files |
| Kernel test, fixed code | 6 of 6 GPU arm tests pass, including the same-bits test |
| Negative gate, the same test on the old `atomicAdd` code | **fails**: "OneAtomActive: d_queries changed on run 1" (an assertion, not a build error) |
| Step 0 | equal in all four arms, both seeds (seed 1: float top-1 1, top-8 14, admitted 348; seed 2: 1, 128, 1,347) |
| **N1 against N2** | **byte-identical**: all 8 saved files per arm, 4 per seed (both seeds' `model.safetensors`, `preds-float.json`, `snapshot.json`, `config.json`), and every loss and count at every step |
| O1 against O2 | differ: both seeds' weights, predictions and snapshots. Readings diverge from step 256 |

**Step-1,024 dev float top-1, of 14,287 rows:**

| Seed | O1 | N1 = N2 | O2 |
|---|---|---|---|
| 1 | 882 | 1,475 | 1,627 |
| 2 | 2,848 | 1,313 | 1,384 |

B_fixed lies inside B_base's spread, and inside claude-140's 745 to 3,845. In every arm, float top-1 peaks at step 512 (seed 1: 3,120 to 3,509) and then falls by step 1,024.

## Checks run at the head

- The stub-`nvcc` `cargo check --features cuda --all-targets` and clippy: no warnings in the changed files. `tests/no_training_crate_dep.rs`: 2 passed. fmt: clean.
- The pod's kernel tests and negative gate, above.
- `loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **holding-the-goal.** The change keeps the gradient as it was, the same sums in a fixed order. It adds no float in serving and no softmax. The question was answered by the registered bar alone.

## Still open

- **R6's second half (next unit):** the export gap re-measured with repeats. Because training is now repeatable, a single pair of runs of one recipe is comparable, so the gap can get a verdict against the two-seed band.
- **Why float top-1 falls between steps 512 and 1,024** in every arm. This is a training-schedule question for the export-gap unit, or for whichever unit owns the recipe.
- **A GPU-to-CPU parity test of the base backward,** which would need the raw kernel output before the secant transform.
- **The mechanism page** `docs/mechanisms/lut-serving.md` is out of this item's scope (claude-121).

## Plain summary

Training the model twice used to give two different models, because the GPU added its numbers in a different order each time. The training now adds them in a fixed order. Two training runs of the fixed version produced exactly the same model, down to the last byte, while two runs of the old version still differed. With repeatable training, the next step can measure honestly how much the model loses when it is exported to the integer engine.
